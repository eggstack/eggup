# Archive Extraction Milestone 001c — Handle-Relative Member Materialization Corrective

Status: implemented (Section 14 stop; closure records blocked status, continued by M001d)

Repository baseline: `d46d35e891bbbc72fced3213ea28fdd6b1ec5ed9`

Source roadmap:

- `plans/subsystems/archive-extraction-roadmap.md`

Original work corrected by this pass:

- `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`
- `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md`
- `plans/closure/archive-extraction/001-status.md`
- `plans/closure/archive-extraction/001b-status.md`

Long-term requirements:

- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`

Applicable ADRs:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: invariant/corrective

## 1. Objective

Close the remaining archive materialization authority gap before Egress M006 or Eggpack Interoperability M002 consumes `eggup-archive`.

M001b correctly moved extraction-root creation and cleanup onto retained directory authority, but declared member creation still resolves through the recorded pathname:

~~~rust
let path = root.join(&member.output_name);
let mut output = create_private_file(&path)?;
~~~

If the operation-owned extraction root is renamed and a foreign directory is installed at the old pathname while extraction is still running, later declared members can be created inside the foreign replacement. `create_new` prevents member-name clobbering but does not bind the parent directory to the root handle Eggup actually owns.

M001c must make materialization use the same retained root authority already used by M001b cleanup. It must also prevent a successful `ExtractedArchive` from returning namespace paths that have silently become foreign or stale after a root rename/replacement.

## 2. Readiness and dependencies

M001c is dependency-ready.

Hard dependencies are closed:

- Archive M001b implementation `057399640b03bb0fcee1ea86fdc5707f812e3579`;
- M001b hosted qualification run `36222536670` green on Stable Linux, Rust 1.89 MSRV, macOS, and Windows;
- documentation/closure head `d46d35e891bbbc72fced3213ea28fdd6b1ec5ed9` with green current-head CI run `36222870057`.

The defect is directly visible on current main:

- `create_private_root` returns a retained `File` handle created through `fs_at::mkdir_at`;
- `extract_inner`, `extract_tar_gz`, and `extract_zip` receive only `&Path`, not the retained root handle;
- both tar and zip declared-member paths use `root.join(output_name)`;
- `create_private_file` uses `std::fs::OpenOptions::create_new(path)`;
- cleanup, by contrast, is already handle-relative.

The existing `fs_at 0.2.1` dependency is sufficient for the write-authority half of this corrective. Its `OpenOptions::open_at` resolves only relative to an open directory handle, and `create_new(true)` rejects existing entries atomically where the underlying filesystem supports that guarantee. No new filesystem dependency is required merely to make member creation handle-relative.

## 3. Current implementation and external primitive evidence

Current production flow:

~~~text
mkdir_at(parent_handle, root_name) -> retained root File
                  |
                  +--> extraction currently drops to root Path
                           |
                           +--> root.join(member)
                           +--> std OpenOptions::create_new(path)
                  |
                  +--> cleanup later returns to retained root File
                       and fs_at relative deletion
~~~

Required target flow:

~~~text
mkdir_at(parent_handle, root_name) -> retained root File
                  |
                  +--> fs_at create_new/open_at(root_handle, member_name)
                  |
                  +--> stream/hash through returned File handle
                  |
                  +--> handle-relative cleanup
~~~

Relevant `fs_at 0.2.1` API evidence:

- `OpenOptions::open_at(&File, path)` resolves relative to the supplied directory handle on Unix and Windows;
- `create_new(true)` rejects an existing file, link, or directory and requests an atomic create-new operation;
- `follow(false)` controls final-component symlink/reparse following;
- `OpenOptionsWriteMode::Write` provides ordinary writes;
- Unix `OpenOptionsExt::mode` can preserve the current owner-private `0600` member creation intent.

Research references:

- https://docs.rs/fs_at/0.2.1/fs_at/struct.OpenOptions.html
- https://docs.rs/fs_at/0.2.1/fs_at/enum.OpenOptionsWriteMode.html
- https://docs.rs/fs_at/0.2.1/fs_at/os/unix/trait.OpenOptionsExt.html

A second issue must be handled explicitly: `ExtractedMember::path()` and `ArtifactMember::new(... source_path ...)` are path-based. If the private root is renamed during extraction, writing through the retained handle remains safe, but the recorded pathname may no longer name the owned directory. A content/digest match alone is not ownership proof, and a weak file-identity heuristic with documented false-positive cases MUST NOT silently become Eggup's ownership authority.

## 4. Invariants that must not regress

- declared archive members are materialized only beneath the exact extraction-root object created by Eggup;
- changing the namespace entry at the recorded root pathname MUST NOT redirect any member write;
- member creation remains exclusive/no-clobber;
- existing files, directories, symlinks, and Windows reparse points at a declared output name MUST fail closed;
- member creation MUST NOT follow a final-component link/reparse point;
- member files remain owner-private (`0600` on Unix before any later explicit executable intent);
- the archive root remains outside the live installation;
- resource, path, entry-count, decompression, size, and digest bounds from M001 remain unchanged;
- M001b handle-relative cleanup semantics remain unchanged;
- a successful result MUST NOT expose a member path that silently refers to foreign replacement state;
- if namespace-to-handle binding cannot be proven at successful handoff, extraction MUST fail closed or use a stronger explicit handoff contract;
- SHA-256 remains integrity evidence, not ownership/authenticity evidence;
- `eggup-core` remains archive-format independent;
- no first-party `unsafe`;
- Rust 1.89 remains the MSRV;
- no Egress or Eggpack archive integration proceeds while M001c is open.

## 5. Scope and non-scope

### In scope

- pass the retained extraction-root handle through materialization rather than dropping to pathname authority;
- replace `create_private_file(path)` with a handle-relative exclusive creator;
- use `fs_at::OpenOptions` with write + create-new + no-follow semantics;
- preserve Unix `0600` creation mode;
- use the returned open member handle for streaming, flushing, hashing, size checks, and digest verification;
- add deterministic race seams that rename/replace the extraction root before and between member creations;
- prove foreign replacement directories remain completely untouched;
- define and enforce truthful successful-handoff semantics for `ExtractedMember::path()`;
- evaluate whether the existing path-only handoff can prove namespace binding safely across Linux/macOS/Windows;
- if an additional identity dependency is considered, review its false-positive semantics, MSRV, platform behavior, unsafe/native surface, and footprint;
- update archive docs/changelog/roadmap/registry and write M001c closure evidence;
- run full hosted Stable/MSRV/macOS/Windows qualification.

### Explicitly out of scope

- generic hardening of every path-based source in `eggup-core`;
- changing live-destination ownership semantics;
- Egress migration;
- Eggpack archive integration;
- Gregg migration;
- new archive formats or metadata types;
- root-directory unlink semantics from M001b;
- authenticity/signature work;
- first-party platform FFI/unsafe;
- accepting a digest-equal foreign replacement as ownership proof.

If the archive-to-core handoff cannot satisfy path truthfulness without changing a public source abstraction, stop under Section 14 and write the required API/ADR follow-up rather than broadening M001c silently.

## 6. Required production changes

### 6.1 Handle-relative private member creation

Replace pathname-authorized member creation with a helper conceptually equivalent to:

~~~rust
fn create_private_file_at(root: &File, name: &Path) -> Result<File, ExtractionError>
~~~

Required semantics:

- `fs_at::OpenOptions::default()`;
- `write(OpenOptionsWriteMode::Write)`;
- `create_new(true)`;
- `follow(false)`;
- Unix mode `0o600`;
- `open_at(root_handle, name)`.

The member name passed to `open_at` must remain the already-validated single-component `output_name`, not an archive-controlled multi-component path.

No production write to an extracted member may be authorized by `root.join(output_name)`.

### 6.2 Carry authority through tar and zip paths

Change the internal extraction call graph so tar and zip materialization receive the retained root handle.

Acceptable shapes include:

- `extract_inner(plan, root_path, root_handle)`;
- a small private `ExtractionRoot { path, handle }` object borrowed by format handlers.

Do not duplicate authority logic between tar and zip.

The returned member `File` remains the authoritative write target for the entire copy/hash/flush/validation sequence.

### 6.3 Namespace replacement race behavior

Add deterministic test hooks or a private test seam that can:

1. create the Eggup-owned root;
2. optionally materialize one declared member;
3. rename the owned root elsewhere;
4. create a foreign directory at the original pathname;
5. allow extraction to continue.

Required behavior:

- all later writes remain in the renamed original directory through the retained handle;
- the foreign replacement receives no file, directory, or metadata mutation from Eggup;
- a foreign pre-created output name remains untouched;
- symlink/reparse replacement at the root or output name is never followed.

### 6.4 Truthful `ExtractedMember::path()` handoff

Handle-relative writes alone are insufficient if success can return a stale/foreign pathname.

Before returning successful `ExtractedArchive`, implementation MUST establish one of these outcomes:

**A. Existing path API remains authoritative.** Prove, with a safe cross-platform mechanism, that the recorded root/member namespace paths still resolve to the operation-owned objects represented by retained handles. The proof must not rely on an identity mechanism whose documented behavior can false-positive a distinct file as the owned file under a supported filesystem.

**B. Stronger handoff replaces path authority.** If strict namespace identity cannot be proven with the existing API, stop and write the public API/ADR follow-up required to carry handle-backed source authority into the later staging boundary. Do not return success with a merely advisory path and rely on SHA-256 to substitute for ownership.

A final pathname existence/digest check, marker file, random filename, timestamp, or ordinary `canonicalize` is insufficient ownership evidence.

### 6.5 Error and residue semantics

If the namespace binding is lost during extraction:

- stop materialization;
- leave foreign replacement state untouched;
- empty only the operation-owned root through the retained handle;
- preserve the primary failure category where possible;
- attach truthful residue evidence;
- do not report successful extraction.

M001b's intentional empty-root residue behavior remains unchanged unless separately superseded.

## 7. Ordered work packages

1. Add a deterministic failing test that renames/replaces the root before a later declared member is created and demonstrates current pathname redirection.
2. Introduce one private root-authority object or equivalent call-chain change carrying `&File` into both tar and zip materialization.
3. Implement `create_private_file_at` with `fs_at` write/create-new/no-follow and Unix `0600`.
4. Convert tar declared-member creation to the handle-relative helper.
5. Convert zip declared-member creation to the same helper.
6. Add root replacement races before first member and between multiple members; assert replacement remains untouched.
7. Add output-name symlink/reparse/existing-entry races around create-new.
8. Determine whether current path-only successful handoff can be proven cross-platform without weak identity assumptions.
9. If yes, implement and test the strong binding proof; if no, stop and author the required public handoff/API ADR rather than completing M001c falsely.
10. Re-run the complete M001/M001a/M001b archive malicious-input and cleanup suite.
11. Measure dependency/binary delta; avoid a new dependency if `fs_at` alone suffices.
12. Run local Stable/MSRV qualification.
13. Push and require a fresh hosted Stable/MSRV/macOS/Windows matrix.
14. Update closure evidence and only then re-unblock Egress M006 and Eggpack M002.

## 8. Failure, restart, cancellation, and contention semantics

Extraction remains non-resumable and retries into a fresh private root.

If root namespace replacement occurs:

- handle-relative materialization may mutate only the original owned root;
- no member write may enter the replacement namespace;
- success is prohibited if later path-based handoff cannot be proven to reference owned state;
- cleanup operates only through retained authority;
- private residue is acceptable;
- foreign state is never removed to recover the old pathname.

If a declared output name appears concurrently inside the owned root, `create_new(true)` fails closed. Do not retry by truncating/reopening.

No contention path loops indefinitely.

## 9. Compatibility and migration

The preferred implementation is private/internal and should preserve public archive plan/member constructors and error categories.

Potential compatibility issue: a strict fix for path-handoff truthfulness may prove impossible without changing the source handoff model. If so:

- do not silently change `ExtractedMember::path()` semantics;
- do not broaden `eggup-core` inside M001c;
- stop and write an ADR/implementation plan for a handle-backed or otherwise object-bound source handoff;
- record existing consumers of the affected API before any public change.

No Egress/Eggpack consumer migration occurs in M001c.

## 10. Required tests

At minimum:

- tar: root renamed/replaced before declared member creation; member appears only in original handle-owned root;
- zip: same race;
- two-member archive: replacement between first and second member; second member cannot be redirected;
- foreign replacement remains byte-for-byte/entry-for-entry unchanged;
- root replacement with symlink/reparse point is not followed;
- raced existing output file fails with no clobber;
- raced output symlink/reparse point fails and target remains untouched;
- Unix member mode remains `0600`;
- exact member size/digest evidence remains correct;
- successful result cannot expose foreign/stale member paths;
- namespace-binding loss produces failure rather than success;
- extraction failure cleans only through retained handle authority;
- all M001/M001a/M001b traversal/link/decompression/cleanup tests remain green;
- stable Windows archive tests execute;
- Rust 1.89 workspace all-target check passes;
- `eggup-core` dependency tree remains archive-free.

The race tests must be deterministic synchronization tests, not timing/sleep-based probabilistic tests.

## 11. Required verification commands

~~~text
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
~~~

Required hosted evidence:

- Stable Linux full checks;
- Rust 1.89 MSRV all-target workspace check;
- macOS full workspace tests;
- Windows archive/acquisition/curl/service portable tests plus workspace all-target check.

M001c MUST NOT close on local-only evidence.

## 12. Documentation updates

Update:

- `crates/eggup-archive/README.md`;
- `architecture/archive-extraction.md`;
- root and archive changelogs;
- archive extraction roadmap;
- consumer adoption roadmap;
- Eggpack interoperability roadmap;
- registry;
- M001b closure addendum identifying the post-closure materialization-authority finding without retracting its cleanup qualification;
- M001c closure record.

If path-handoff semantics require an ADR/public API follow-up, document the stop result rather than claiming M001c closure.

## 13. Acceptance criteria

M001c closes only when:

- no declared-member write is authorized by joining an output filename to the root pathname;
- tar and zip writes are relative to the retained operation-owned root handle;
- member creation is create-new/no-follow and owner-private;
- root replacement before or between members cannot redirect writes;
- foreign replacement directories and links remain untouched;
- successful extraction exposes only source evidence that is still provably bound to operation-owned state;
- namespace-binding ambiguity fails closed;
- all previous archive safety/cleanup invariants remain green;
- Rust 1.89 remains supported;
- hosted Linux/MSRV/macOS/Windows qualification is green;
- no medium-or-higher archive authority finding remains open;
- only after closure may Egress M006 and Eggpack M002 return to ready-to-author.

## 14. Stop conditions

Stop and write a separate ADR/public-interface plan if:

- the existing `ExtractedMember::path()` / `ArtifactMember` source-path model cannot prove namespace-to-handle binding across supported platforms;
- the only proposed identity mechanism documents false-positive equality for distinct files on a supported filesystem and would be used as destructive/ownership authority;
- solving successful handoff requires a generic `eggup-core` source-handle abstraction;
- a proposed fix needs first-party unsafe/FFI;
- a new dependency exceeds Rust 1.89 or materially expands the optional archive dependency surface without justification;
- Windows cannot preserve equivalent create-new/no-follow semantics.

Do not weaken ADR-0005 to avoid an API decision.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- before/after materialization call graph;
- exact `fs_at` open options used for member creation;
- deterministic root replacement race results for tar and zip;
- deterministic between-members race evidence;
- existing-file/symlink/reparse no-clobber evidence;
- exact successful-handoff/path-binding mechanism;
- dependency/footprint delta;
- public auto-trait/API changes, if any;
- all prior archive regression results;
- hosted workflow run ID and job conclusions;
- unresolved findings by severity;
- downstream Egress/Eggpack gate disposition.

## 16. Handoff notes

This is a filesystem-authority corrective, not an archive parser rewrite.

M001b's cleanup design is sound and should not be replaced. Reuse its retained root handle and existing `fs_at 0.2.1` dependency.

The key rule is symmetrical authority: if cleanup is safe because it acts through the owned directory handle, materialization must act through that same authority. Do not permit writes through a pathname merely because the root was private when originally created.

The second rule is equally important: safe writes do not justify returning a stale path. If the current path-only handoff cannot be made truthful without a stronger interface, stop and surface that interface decision explicitly.
