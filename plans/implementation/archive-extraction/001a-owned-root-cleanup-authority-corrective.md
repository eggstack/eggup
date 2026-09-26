# Archive Extraction Milestone 001a — Owned-Root Cleanup Authority Corrective

Status: ready for handoff

Repository baseline: `ea51fe12a7c9120028b727eb5e40411e9b10f8e2`

Source roadmap:

- `plans/subsystems/archive-extraction-roadmap.md`

Original work corrected by this pass:

- `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`
- `plans/closure/archive-extraction/001-status.md`

Long-term requirements:

- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`

Applicable ADRs:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: invariant/corrective

## 1. Objective

Strengthen archive extraction cleanup so Eggup never recursively deletes a directory solely because it currently occupies the pathname originally assigned to an extraction root.

M001 creates each extraction root exclusively, but cleanup currently calls `fs::remove_dir_all(root_path)`. The existing replacement regression proves that a foreign regular file replacing the root is preserved because `remove_dir_all` rejects non-directories. It does not prove preservation when the original owned directory is renamed away and a foreign directory is installed at the same pathname.

ADR-0005 requires cleanup to remove only operation-owned extraction state. M001a must retain or prove cleanup authority through an open directory object or equivalent strong identity mechanism and fail closed when ownership cannot be established.

## 2. Why this milestone is ready

The issue is local to the new `eggup-archive` cleanup boundary and was identified immediately after M001 closure.

Current standard-library `remove_dir_all` protects against symlink traversal races on Eggup's primary platforms, but pathname-based invocation does not itself prove that the root directory at cleanup time is the same directory Eggup created.

Current Rust guidance for filesystem TOCTOU-sensitive code recommends keeping files open for the duration of sensitive operations. Cross-platform safe crates also expose deletion through open directory handles; these are candidates only if they satisfy Eggup's Rust 1.89, dependency, footprint, and no-first-party-unsafe requirements.

This corrective does not require changing archive parsing, member validation, or `eggup-core`.

## 3. Current implementation evidence

At baseline `ea51fe12a7c9120028b727eb5e40411e9b10f8e2`:

- `create_private_root` creates a unique child directory using create semantics and 0700 mode on Unix;
- `DirectoryGuard` stores only the root `PathBuf`;
- `PersistedExtraction` also stores only the root path and member evidence;
- `remove_owned_root(path)` delegates to `fs::remove_dir_all(path)`;
- drop cleanup ignores cleanup errors;
- explicit cleanup returns `CleanupFailed` with residue path;
- the regression `cleanup_failure_reports_residue_without_deleting_replacement` replaces the root with a regular file, not a directory.

The extraction body itself already uses exclusive output-file creation and does not follow archive-provided links.

## 4. Invariants that must not regress

- cleanup deletes only extraction state whose ownership is proven;
- foreign replacement directories are never recursively deleted;
- symlink/junction/reparse-point substitution remains fail closed;
- extraction success/failure never mutates the live installation;
- cleanup failure reports residue rather than escalating to live-install recovery;
- successful `persist` transfers cleanup responsibility without losing ownership evidence;
- drop cleanup must never be more destructive than explicit cleanup;
- only regular allowlisted archive members are materialized;
- all M001 path/type/decompression/digest bounds remain unchanged;
- `eggup-core` remains archive-format independent;
- no first-party unsafe code is introduced.

## 5. Scope

### In scope

- retain strong extraction-root ownership evidence from creation through guard/persisted cleanup;
- prefer an open-directory-handle/capability model for recursive content deletion;
- refuse pathname-only recursive deletion when ownership could have changed;
- preserve or strengthen cleanup on Linux, macOS, and Windows;
- add directory-replacement race regressions;
- test symlink/junction/reparse substitution where platform support permits;
- evaluate the smallest safe cross-platform dependency if std alone cannot meet the contract;
- record dependency/MSRV/footprint impact;
- update archive docs/roadmap/registry and closure evidence.

### Explicitly out of scope

- archive parsing changes;
- additional archive formats;
- live destination commit;
- service/acquisition changes;
- generic filesystem sandboxing;
- defending against an actor that already has arbitrary code execution inside the Eggup process;
- package publication;
- Egress or Eggpack integration implementation.

## 6. Required production changes

### 6.1 Ownership evidence lifetime

When creating the extraction root, retain evidence tied to the created directory object rather than only its pathname.

Preferred design order:

1. retain an open directory handle/capability from creation or immediately after creation with identity verified before any archive writes;
2. carry that handle/capability inside `DirectoryGuard`;
3. transfer it into `PersistedExtraction` on `persist`;
4. perform recursive content deletion relative to that retained directory object where the chosen platform abstraction supports it.

If the standard library cannot safely express this across all supported targets at Rust 1.89, evaluate a narrow safe dependency such as a handle-based removal/capability crate. Do not add first-party `unsafe` to call platform APIs.

Any new dependency must be reviewed for:

- Rust 1.89 compatibility;
- Windows/macOS/Linux behavior;
- no hidden privilege changes;
- race semantics;
- transitive footprint;
- maintenance/security posture.

### 6.2 Final root removal

Recursive content deletion and final root-name removal are distinct authority steps.

The implementation MUST NOT recursively traverse a replacement directory found at the old pathname.

If the chosen handle-based abstraction can delete the opened directory object itself safely, use that capability.

If final root removal cannot be made strongly object-bound on a supported platform:

- empty the proven owned directory through the retained handle;
- revalidate the pathname against strong retained identity where available;
- remove only an empty root;
- if identity is ambiguous or changes, leave residue and return/report `CleanupFailed` rather than deleting a foreign replacement.

A small empty residue is preferable to violating ADR-0005.

### 6.3 Drop behavior

`Drop` may remain best-effort, but it must use exactly the same ownership-safe primitive as explicit cleanup.

It MUST NOT fall back from a failed handle/identity check to `fs::remove_dir_all(path)`.

### 6.4 Persist semantics

`ExtractedArchive::persist` must transfer both:

- the root/member paths;
- the cleanup authority/identity evidence.

Do not reduce `PersistedExtraction` to a bare path capable of unsafe later recursive deletion.

## 7. Ordered work packages

1. Add a regression that renames the owned extraction root and places a foreign directory at the old pathname.
2. Demonstrate that the current pathname-only cleanup would target the replacement or otherwise lacks ownership proof.
3. Prototype the smallest open-handle cleanup design under Rust 1.89.
4. Select std-only or a narrow safe dependency based on cross-platform evidence.
5. Carry root cleanup authority through `DirectoryGuard` and `PersistedExtraction`.
6. Replace pathname-only recursive cleanup.
7. Add foreign-directory, symlink/junction, rename, and cleanup-failure regressions.
8. Re-run the complete M001 malicious archive/resource-bound matrix.
9. Run Linux/macOS/Windows runtime qualification.
10. Measure dependency/footprint impact.
11. Update docs/roadmap/registry and write M001a closure evidence.

## 8. Failure, cancellation, restart, and contention semantics

No live transaction state is involved.

When ownership proof is lost:

- do not recursively delete the current pathname;
- return `CleanupFailed` for explicit cleanup with residue evidence;
- best-effort drop cleanup leaves residue silently if it cannot prove authority;
- never classify this as `RecoveryRequired` for the installed application.

Concurrent rename/replacement should cause safe failure or deletion of only the opened owned object, never traversal of a foreign replacement tree.

Retry may be attempted only if the caller still holds valid cleanup authority/evidence.

## 9. Compatibility and migration

Public archive parsing/extraction plan types should remain source compatible if practical.

`PersistedExtraction` internal representation may change to carry a handle/capability.

Observable behavior changes only in race/cleanup edge cases:

- previous pathname-only cleanup may have recursively removed the current directory;
- corrected cleanup fails closed when root identity is ambiguous.

No consumer migration should be required unless a public trait bound or Send/Sync property changes; if it does, document it explicitly and stop for review before release.

## 10. Required tests

At minimum:

- normal explicit cleanup succeeds;
- normal drop cleanup removes owned root;
- persisted cleanup succeeds;
- original root renamed away + foreign non-empty directory at old path is preserved;
- original root renamed away + foreign empty directory at old path is preserved or, if platform object-bound deletion removes only the original, the replacement remains;
- foreign replacement file remains preserved;
- symlink replacement does not cause traversal;
- Windows junction/reparse replacement is preserved where test permissions allow;
- concurrent/identity mismatch returns cleanup residue evidence;
- cleanup failure never invokes fallback pathname-recursive deletion;
- all M001 traversal/link/special/member/digest/decompression tests remain green;
- `eggup-core` dependency tree remains archive-free.

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

Hosted qualification must exercise the replacement-directory cleanup regression on Linux, macOS, and Windows where the platform primitive supports it. State any platform limitation explicitly rather than inferring behavior.

## 12. Documentation updates

Update:

- `crates/eggup-archive/README.md`;
- `architecture/archive-extraction.md`;
- archive changelog;
- archive subsystem roadmap;
- registry;
- M001 closure addendum/cross-reference if useful;
- M001a closure record.

## 13. Acceptance criteria

M001a closes only when:

- recursive cleanup is no longer authorized solely by a pathname;
- a foreign directory replacing the root path is preserved;
- cleanup authority survives `persist`;
- drop and explicit cleanup use the same safe authority model;
- ambiguous identity fails closed with residue evidence;
- Linux/macOS/Windows evidence is recorded honestly;
- M001 extraction/security regression matrix remains green;
- dependency and footprint impact is documented;
- no high- or medium-severity cleanup-authority finding remains.

Until M001a closes, Egress M006 and Eggpack archive interoperability M002 must not proceed to implementation against the M001 cleanup contract.

## 14. Stop conditions

Stop and write an ADR/deeper platform plan if:

- no safe Rust 1.89 cross-platform abstraction can delete contents relative to retained directory authority;
- the only implementation requires first-party unsafe code;
- a new dependency materially exceeds the intended leaf-crate footprint;
- Windows requires privilege/ACL mutation;
- safe cleanup would require changing live-install transaction semantics.

Fail-closed residue is an acceptable interim result; foreign recursive deletion is not.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- selected root-authority primitive/dependency and rationale;
- Rust 1.89 compatibility;
- before/after cleanup algorithm;
- replacement-directory regression results on each platform;
- symlink/junction/reparse evidence;
- M001 full regression results;
- dependency/footprint delta;
- cleanup residue semantics;
- unresolved findings by severity.

## 16. Handoff notes

Keep this corrective in `eggup-archive`. Do not move generic archive cleanup into `eggup-core`.

The implementation should optimize for authority preservation, not eager deletion. When the choice is "leave a private empty residue" versus "recursively delete a path whose identity is uncertain," choose the residue.
