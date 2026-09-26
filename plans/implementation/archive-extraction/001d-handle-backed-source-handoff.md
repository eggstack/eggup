# Archive Extraction Milestone 001d — Handle-Backed Source Handoff

Status: ready for handoff (corrected by planning-hygiene C006; see `plans/closure/planning-closure-hygiene-corrective/006-status.md`)

Repository baseline: `f463aa79398a0b1893eb117dc639d2f262fe1b86`

Planning correction: `plans/implementation/planning-closure-hygiene-corrective/006-m001d-readiness-and-bound-source-contract-reconciliation.md` (closed).

Runtime qualification carried over from `0d2f1f06110e3949755120fa9e80ff5a5b0b4b3c`: M001c hosted write-authority run `36257884083` passed Stable/MSRV/macOS/Windows, and current-head CI run `36257992801` also passed. Commits between `0d2f1f0` and this baseline are planning/registry/roadmap text only (no `.rs`, `Cargo.toml`, `Cargo.lock`, or workflow changes), so that qualification still applies.

Source roadmap:

- `plans/subsystems/archive-extraction-roadmap.md`

Original work continued by this pass:

- `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`
- `plans/closure/archive-extraction/001c-status.md` (Section 14 stop record)

Long-term requirements:

- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`

Applicable ADRs:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: invariant

## 1. Objective

Replace the path-only `ExtractedMember::path()` / `ArtifactMember` source-path handoff with an object-bound source authority so a successful extraction cannot expose a stale or foreign pathname after a root rename/replacement, and so the later core staging boundary cannot be redirected through the pathname between handoff and use.

M001c moved every declared-member write onto the retained root handle (`fs_at` write + create-new + no-follow, Unix `0600`) and proved with deterministic races that foreign replacements stay untouched. It stopped under its Section 14 because the existing path-only handoff cannot prove namespace-to-handle binding cross-platform: Windows stable has no file identity (`MetadataExt::file_index` / `volume_serial_number` remain nightly-only behind `windows_by_handle` #63010), the only portable identity crate documents false-positive equality for distinct files on supported filesystems (ReFS 128-bit IDs vs 64-bit `nFileIndex`; see `same-file` win.rs), and marker/random-filename/canonicalize checks are explicitly insufficient per M001c Section 6.4. M001d carries that stopped handoff half to a stronger explicit contract.

## 2. Why this milestone is ready

M001d is dependency-ready.

Hard dependencies are recorded:

- Archive M001b cleanup authority remains closed (`0573996`, hosted run `36222536670`);
- Archive M001c write-authority half is implemented at `09c953fe1fe512717584bb3e5509e79da894b35a` (handle-relative tar/zip creation via one shared `ExtractionScope`, deterministic rename/replacement races green locally; closure `plans/closure/archive-extraction/001c-status.md` records the Section 14 stop);
- M001c hosted write-authority qualification run `36257884083` passed Stable Linux, Rust 1.89 MSRV, macOS, and Windows; current-head CI run `36257992801` also passed at documentation head `0d2f1f0`, and this plan's baseline `f463aa7` adds only planning/registry/roadmap text on top of that head;
- the defect is directly visible: `ExtractedMember` still carries `path: root.join(output_name)` as evidence while bytes live in the handle-owned (possibly renamed) directory, and `ArtifactMember::new(id, source_path, destination)` re-opens that pathname later in core staging;
- no new filesystem dependency is assumed; the design must choose the smallest sound handoff (handle-carrying member, staging-from-handle, or equivalent) without first-party unsafe/FFI.

## 3. Current implementation evidence

Current production flow after M001c:

```text
mkdir_at(parent_handle, root_name) -> retained root File
                  |
                  +--> ExtractionScope { root_path, root_handle }
                         |
                         +--> create_private_file_at(root_handle, output_name)
                         |      (fs_at write/create-new/no-follow, Unix 0600)
                         +--> stream/hash/flush/validate via returned File
                         |
                         +--> ExtractedMember { path: root.join(name), bytes, sha256 }
                                |
                                +--> caller maps path -> ArtifactMember source
                                       |
                                       +--> core re-opens pathname later
```

Required target flow (one of the accepted shapes in Section 6):

```text
mkdir_at(parent_handle, root_name) -> retained root File
                  |
                  +--> handle-relative member creation (unchanged)
                  |
                  +--> success carries object-bound source evidence
                       (already-open member File, direct object-bound
                        extraction-to-staging copy, or equivalent that
                        never re-opens a member name after the handoff
                        boundary)
                  |
                  +--> core staging consumes the bound source without
                       pathname re-resolution
```

A root handle plus a member name reopened later with `open_at(root, name)` is explicitly NOT an accepted shape for the final handoff (see Section 6.1): the member directory entry itself can be replaced after extraction and before staging, so a later name lookup acquires the replacement object, not the extracted-and-verified one.

## 4. Invariants that must not regress

- declared members are materialized only beneath the exact extraction-root object;
- member creation remains exclusive/no-clobber, no-follow, owner-private (`0600` Unix);
- resource, path, entry-count, decompression, size, and digest bounds from M001 unchanged;
- M001b handle-relative cleanup semantics unchanged;
- M001c handle-relative write semantics unchanged;
- no successful result exposes a pathname that can silently resolve to foreign state;
- no staged archive byte is obtained by reopening a member name after the extracted member object could have been replaced;
- the ordinary path-source API remains available for non-archive callers; archive support must not force an unrelated breaking migration on path-source consumers;
- handle/cursor ownership is deterministic;
- cleanup failure remains fail-closed and never mutates foreign state;
- SHA-256 remains integrity evidence, not ownership/authenticity evidence;
- `eggup-core` remains archive-format independent;
- no first-party `unsafe`;
- Rust 1.89 remains the MSRV;
- no Egress or Eggpack archive integration proceeds while M001d is open.

## 5. Scope and non-scope

### In scope

- design and implement one handle-backed source handoff from extraction to core staging;
- carry per-member object authority (open file handles, root-handle-bound tokens, or equivalent) across `ExtractedArchive` / `PersistedExtraction` into `ArtifactSet` construction;
- eliminate pathname re-resolution between extraction success and staged-byte validation;
- deterministic rename/replacement races proving the bound source still resolves to owned bytes and foreign state stays untouched, including post-handoff replacement attempts;
- update archive docs/changelog/roadmap/registry and write M001d closure evidence;
- run full hosted Stable/MSRV/macOS/Windows qualification.

### Explicitly out of scope

- archive construction, release naming/discovery, authenticity/signature work;
- live-destination ownership semantic changes beyond consuming the new source type;
- Egress migration;
- Eggpack archive integration;
- Gregg migration;
- new archive formats or metadata types;
- generic hardening of every path-based source in `eggup-core` beyond the one bound-source path this milestone introduces;
- first-party platform FFI/unsafe;
- accepting digest-equal foreign bytes as ownership proof.

If the handoff requires a public `eggup-core` source abstraction change, that change is in scope here (unlike M001c) but must be minimal, reviewed, and migrated explicitly per Section 9 — not silently broadened.

## 6. Required production changes

### 6.1 Bound source type

Acceptable shapes are object-bound only. Choose exactly one of:

- **A. Moved open member object.** `ExtractedMember` (and/or `PersistedExtraction`) carries the already-open readable member `File` for each member alongside the advisory path, and core gains a minimal staging constructor (e.g. `ArtifactMember::from_handle` or equivalent) that stages from that moved object directly without re-opening any name.
- **B. Immediate object-bound copy into core-owned stage.** Extraction copies handle-owned bytes directly from the already-open member object into core staging (or a sealed staging input) before the extraction authority is released, so no later pathname or member-name open exists; the recorded path becomes purely diagnostic and is documented as such.
- **C. Equivalent object-bound source** with the same property: demonstrably tied to the same opened member object, performing no name lookup after the handoff boundary.

Explicitly forbidden as the final authority:

```text
root handle + member name -> later open_at(root, name) -> stage
```

A retained root handle prevents a root-path replacement from redirecting the lookup, but a concurrent actor can still replace the member entry inside the owned root after extraction and before staging. Reopening `name` with `open_at(root, name)` then acquires the replacement object, not the object whose bytes were extracted and verified. Only shapes A-C above satisfy the acceptance contract.

Do not keep two parallel source authorities. The advisory path (if retained for diagnostics) must be documented as non-authoritative.

### 6.1a Readable authority at object creation

The M001c creation helper opens members for write authority only (`write` + `create_new(true)` + `follow(false)`); that handle cannot be assumed suitable for later staging reads, and read authority MUST NOT be regained by reopening the member by pathname/name after the security boundary. The implementation must therefore establish readability on the same object:

- preferred: create the member with read + write authority from the outset using the same atomic handle-relative create-new/no-follow operation (add `read(true)` to the existing `fs_at::OpenOptions` creation), continue using the returned handle for extraction, then flush and seek the same owned object back to offset zero before staging consumption; or
- transfer bytes into core-owned staging by reading the already-open readable member object before closing/releasing that authority.

Any alternative must prove the same object identity without name re-resolution.

### 6.1b Cursor ownership

Rust `File::try_clone()` creates another `File` referring to the same underlying file description; reads, writes, and seeks affect both instances' shared cursor state. The implementation therefore MUST NOT assume clones have independent offsets:

- prefer moving a single owned member `File` into the staging operation;
- before staging reads, flush any writer state and seek the owned handle to `SeekFrom::Start(0)`;
- if cloning is unavoidable, document the shared-offset semantics and prove no aliased reader/writer can advance the cursor concurrently;
- do not expose a cloneable bound-source API unless clone semantics are explicitly safe.

Tests must include a non-zero cursor before staging and prove the full file is staged from byte zero.

### 6.1c Handle lifetime and cleanup ordering

The implementation must specify the state transition:

```text
Extracted member handle owned by extraction
        |
        v
handle transferred/borrowed exclusively into staging
        |
        v
stage copy complete + handle consumed/closed
        |
        v
extraction cleanup may empty root
```

Required behavior:

- explicit cleanup cannot destructively race a member still being staged;
- dropping `PersistedExtraction` while bound sources are outstanding must have deterministic ownership semantics (compile-time ownership preferred over runtime booleans);
- Windows hosted tests must exercise cleanup after bound member handles are consumed/closed;
- no pathname-recursive fallback if deletion is temporarily prevented by an open handle (open file handles can affect rename/delete behavior on Windows and other platforms);
- residue remains acceptable under M001b semantics.

### 6.2 Authority plumbing

- thread the bound source through `ExtractedArchive::persist` / `PersistedExtraction` so cleanup responsibility transfer does not drop handle authority;
- ensure `Drop` / explicit cleanup still empties only via the retained root handle, ordered after bound handles are consumed/closed per Section 6.1c (never pathname-recursive fallback);
- preserve declaration-order evidence, byte counts, and digests.

### 6.3 Binding races

- deterministic seams that rename/replace the root before, between, and after member writes, plus post-handoff replacement before staging;
- a dedicated deterministic race that specifically replaces the member directory entry inside the still-owned root after extraction but before staging (retaining the original member object), then stages from the bound source and proves staged bytes are the originally extracted bytes while the foreign replacement remains untouched — in addition to root rename/replacement races;
- assert bound-source reads still return owned bytes, foreign replacements stay untouched, and no pathname-resolved read enters foreign state.

### 6.4 Error and residue semantics

- binding loss or handle/read/seek failure fails closed with the original category where possible plus truthful residue;
- a failed stage copy does not reopen the path or member name as fallback;
- foreign state is never removed to recover a pathname;
- cleanup waits through ownership sequencing rather than retry loops;
- private residue remains acceptable; no indefinite retry loops;
- no hidden downgrade from bound source to path source.

## 7. Ordered work packages

1. Decide shape A/B/C with a short design note and public API diff.
2. Implement the bound source type and minimal core staging entry point.
3. Thread authority through persist/cleanup/drop without handle loss.
4. Add deterministic pre/between/post-handoff rename/replacement races.
5. Re-run the full M001/M001a/M001b/M001c malicious-input, cleanup, and materialization suites.
6. Measure dependency/binary delta; justify any new dependency under Section 14 review.
7. Run local Stable/MSRV qualification.
8. Push and require a fresh hosted Stable/MSRV/macOS/Windows matrix.
9. Update closure evidence and only then re-evaluate Egress M006 and Eggpack M002 readiness.

## 8. Failure, restart, cancellation, and contention semantics

Extraction remains non-resumable; retries use a fresh private root. If handle authority is lost, stop, empty only the owned root via any surviving handle authority, preserve the primary failure category where possible, attach truthful residue, and never report success. No contention path loops indefinitely; concurrent creation inside the owned root still fails closed via create-new.

## 9. Compatibility and migration

The bound source is an additive API change by design. `ArtifactMember::new(id, source_path, destination)` and its existing path-source semantics are preserved for ordinary already-acquired local artifacts where the caller intentionally supplies a path — existing simple consumers depend on this behavior and MUST NOT be required to migrate merely to support archive-bound sources. Do not deprecate the path constructor merely because archive extraction needs stronger authority.

The implementation adds a narrow orthogonal bound-source path for the archive handoff (a bound-source type, a bound-source staging constructor, or a separate archive-to-core staging helper); prefer an additive source/staging seam over changing `ArtifactMember` into a broad generic handle framework. Record every affected/new public item (`ExtractedMember`, `PersistedExtraction`, the new bound-source type/constructor), migrate only the one in-tree consumer test (`extracted_file_can_be_prepared_as_an_ordinary_core_artifact`) plus docs, and record any public type/auto-trait change caused by handle ownership (a type that begins owning `File` may lose `Clone`, `Eq`, or `Sync`; keep such effects limited to the new bound-source path where possible). No Egress/Eggpack consumer migration occurs in M001d; they re-gate on M001d closure. If a new public source abstraction is added, its rustdoc must state the authority model (object-bound member object, pathname-advisory) and the cleanup responsibility transfer.

## 10. Required tests

At minimum:

- tar/zip root rename before first bound-source write; bound reads return owned bytes; foreign untouched;
- two-member replacement between writes; both bound sources stay owned;
- post-extraction member-entry replacement inside the still-owned root before staging (original member object retained); staging from the bound source still consumes the originally extracted bytes, never the foreign replacement, and the replacement stays untouched;
- post-handoff root replacement before staging; staging still consumes owned bytes, never foreign;
- non-zero member cursor before staging; full file staged from byte zero;
- raced existing output and symlink/reparse output cases still fail closed with targets untouched;
- Unix `0600` preserved on bound sources;
- size/digest evidence still exact;
- all M001/M001a/M001b/M001c traversal/link/bounds/cleanup/materialization tests green;
- stable Windows bound-source tests execute, including extraction cleanup after bound member handles are consumed/closed;
- Rust 1.89 workspace all-target check passes;
- `eggup-core` remains archive-format independent (no `tar`/`zip`/`flate2`/`fs_at` in its tree).

Races must be deterministic (synchronous hooks), not sleep-based.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-archive --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree -p eggup-archive --locked
cargo tree -p eggup-core --locked
./scripts/check-local.sh
git diff --check
```

Required hosted evidence:

- Stable Linux full checks;
- Rust 1.89 MSRV all-target workspace check;
- macOS full workspace tests;
- Windows archive/acquisition/curl/service portable tests plus workspace all-target check.

M001d MUST NOT close on local-only evidence.

## 12. Documentation updates

Update:

- `crates/eggup-archive/README.md`;
- `crates/eggup-archive/CHANGELOG.md` and root `CHANGELOG.md`;
- `architecture/archive-extraction.md`;
- archive extraction roadmap;
- consumer adoption roadmap (Egress M006 gate);
- Eggpack interoperability roadmap (M002 gate);
- registry;
- M001d closure record.

If a new public source abstraction is added, its rustdoc must state the authority model (object-bound member object, pathname- and member-name-advisory) and the cleanup responsibility transfer.

## 13. Acceptance criteria

M001d closes only when:

- no staged byte is obtained by re-resolving a recorded extraction pathname or by reopening a member name after the handoff boundary;
- tar and zip bound sources resolve only through the already-open member object (or a direct object-bound stage copy from it);
- root-handle + later `open_at(root, member_name)` is explicitly rejected as final ownership authority;
- the existing path-source `ArtifactMember::new` API and semantics are preserved for non-archive callers;
- readable authority is established on the same member object without name re-resolution, with explicit cursor ownership and seek-to-zero/full-byte staging evidence;
- handle lifetime/cleanup ordering is explicit and exercised on Unix and Windows;
- rename/replacement before, between, or after writes cannot redirect staged bytes, including post-extraction member-entry replacement;
- foreign replacements, links, and reparse points stay untouched;
- all prior safety/cleanup/materialization invariants stay green;
- Rust 1.89 holds; hosted Linux/MSRV/macOS/Windows qualification is green;
- no medium-or-higher archive authority finding remains open;
- only after closure may Egress M006 and Eggpack M002 return to ready-to-author.

## 14. Stop conditions

Stop and write a further ADR/plan rather than weakening the contract if:

- the bound source requires first-party unsafe/FFI;
- the only viable identity mechanism documents false-positive equality for distinct files on a supported filesystem and would serve as ownership authority;
- a proposed fix needs to trust digest-equal foreign bytes as ownership proof;
- a new dependency exceeds Rust 1.89 or materially expands the archive/core surface without justification;
- Windows cannot preserve equivalent create-new/no-follow/bound-source semantics.

Do not weaken ADR-0005 to avoid the API decision.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- chosen handoff shape (A/B/C) with public API diff;
- before/after staging call graph;
- deterministic pre/between/post-handoff race results for tar and zip, including the post-extraction member-entry replacement race;
- readable-handle creation/transfer evidence (same-object identity, no name re-resolution) plus cursor/seek-to-zero ownership evidence;
- handle lifetime/cleanup ordering evidence, including Windows cleanup-after-consume/close results;
- path-source compatibility disposition (existing constructors preserved) plus any auto-trait/API effects of handle ownership;
- no-clobber/no-follow evidence;
- dependency/footprint delta;
- all prior regression results;
- hosted workflow run ID and job conclusions;
- unresolved findings by severity;
- downstream Egress/Eggpack gate disposition.

## 16. Handoff notes

M001c fixed where bytes land (handle-relative writes); M001d fixes how bytes are proven owned at staging time (object-bound handoff). Do not regress M001c write authority while changing the handoff. Keep the change minimal: preserve the ordinary path-source model, add one archive-specific object-bound source/staging seam that consumes the exact already-open extracted member object without any later namespace lookup — not a name-reopen design, not a generic file-handle framework.
