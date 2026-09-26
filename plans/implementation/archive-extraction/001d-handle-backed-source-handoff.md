# Archive Extraction Milestone 001d — Handle-Backed Source Handoff

Status: blocked on planning-hygiene C006; runtime implementation must not begin until C006 closes

Repository baseline: `1388a02356dfa01d72c63c97c1baca09ff6004a1`

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
- Archive M001c write-authority half is implemented (handle-relative tar/zip creation via one shared `ExtractionScope`, deterministic rename/replacement races green locally; closure `plans/closure/archive-extraction/001c-status.md` records the Section 14 stop);
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
                       (open member handle bundle, root-handle-bound
                        source token, or extraction-to-staging copy
                        that never re-opens the recorded pathname)
                  |
                  +--> core staging consumes the bound source without
                       pathname re-resolution
```

## 4. Invariants that must not regress

- declared members are materialized only beneath the exact extraction-root object;
- member creation remains exclusive/no-clobber, no-follow, owner-private (`0600` Unix);
- resource, path, entry-count, decompression, size, and digest bounds from M001 unchanged;
- M001b handle-relative cleanup semantics unchanged;
- M001c handle-relative write semantics unchanged;
- no successful result exposes a pathname that can silently resolve to foreign state;
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

Choose exactly one of:

- **A. Handle-carrying member.** `ExtractedMember` (and/or `PersistedExtraction`) carries the open member `File` (or a root-handle + name token opened only via `open_at`) alongside the advisory path, and core gains a minimal constructor (e.g. `ArtifactMember::from_handle` or equivalent) that stages from the open handle without re-opening the pathname.
- **B. Extraction-to-staging copy.** Extraction copies handle-owned bytes directly into core staging (or a sealed staging input) under the retained handle, so no later pathname open exists; the recorded path becomes purely diagnostic and is documented as such.
- **C. Equivalent object-bound token** with the same property: the staging boundary never resolves the recorded pathname to obtain bytes.

Do not keep two parallel source authorities. The advisory path (if retained for diagnostics) must be documented as non-authoritative.

### 6.2 Authority plumbing

- thread the bound source through `ExtractedArchive::persist` / `PersistedExtraction` so cleanup responsibility transfer does not drop handle authority;
- ensure `Drop` / explicit cleanup still empties only via the retained root handle;
- preserve declaration-order evidence, byte counts, and digests.

### 6.3 Binding races

- deterministic seams that rename/replace the root before, between, and after member writes, plus post-handoff replacement before staging;
- assert bound-source reads still return owned bytes, foreign replacements stay untouched, and no pathname-resolved read enters foreign state.

### 6.4 Error and residue semantics

- binding loss or handle failure fails closed with the original category where possible plus truthful residue;
- foreign state is never removed to recover a pathname;
- private residue remains acceptable; no indefinite retry loops.

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

The bound source is a public API change by design. Record every affected public item (`ExtractedMember`, `PersistedExtraction`, `ArtifactMember`, staging constructors), keep the old path-only constructors only as deprecated shims if strictly needed (prefer removal with a migration note), and migrate the one in-tree consumer test (`extracted_file_can_be_prepared_as_an_ordinary_core_artifact`) plus docs. No Egress/Eggpack consumer migration occurs in M001d; they re-gate on M001d closure.

## 10. Required tests

At minimum:

- tar/zip root rename before first bound-source write; bound reads return owned bytes; foreign untouched;
- two-member replacement between writes; both bound sources stay owned;
- post-handoff root replacement before staging; staging still consumes owned bytes, never foreign;
- raced existing output and symlink/reparse output cases still fail closed with targets untouched;
- Unix `0600` preserved on bound sources;
- size/digest evidence still exact;
- all M001/M001a/M001b/M001c traversal/link/bounds/cleanup/materialization tests green;
- stable Windows bound-source tests execute;
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

If a new public source abstraction is added, its rustdoc must state the authority model (handle-bound, pathname-advisory) and the cleanup responsibility transfer.

## 13. Acceptance criteria

M001d closes only when:

- no staged byte is obtained by re-resolving a recorded extraction pathname;
- tar and zip bound sources resolve only through retained handle authority;
- rename/replacement before, between, or after writes cannot redirect staged bytes;
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
- deterministic pre/between/post-handoff race results for tar and zip;
- no-clobber/no-follow evidence;
- dependency/footprint delta;
- all prior regression results;
- hosted workflow run ID and job conclusions;
- unresolved findings by severity;
- downstream Egress/Eggpack gate disposition.

## 16. Handoff notes

M001c fixed where bytes land (handle-relative writes); M001d fixes how bytes are proven owned at staging time (handle-backed handoff). Do not regress M001c write authority while changing the handoff. Keep the change minimal: one bound source, one staging entry point, explicit migration — not a generic file-handle framework.
