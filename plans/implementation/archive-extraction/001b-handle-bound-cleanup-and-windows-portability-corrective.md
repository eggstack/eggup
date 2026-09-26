# Archive Extraction Milestone 001b — Handle-Bound Cleanup and Windows Portability Corrective

Status: ready for handoff

Repository baseline: `4fb9d8ca1c1c46fa4b4a2976e6fde3a38f51abdb`

Source roadmap:

- `plans/subsystems/archive-extraction-roadmap.md`

Original work corrected by this pass:

- `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`
- `plans/implementation/archive-extraction/001a-owned-root-cleanup-authority-corrective.md`
- `plans/closure/archive-extraction/001-status.md`
- `plans/closure/archive-extraction/001a-status.md`

Related qualification evidence:

- current-head hosted CI run `36220815378` — Stable/MSRV/macOS passed; Windows failed compiling `eggup-archive`;
- Acquisition M008 implementation `bcf3084c2e31b497b2b9c2de61aa2b6e25c0a486`, whose hosted closure evidence remains incomplete until a current-head Windows lane can run through the archive build.

Long-term requirements:

- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`

Applicable ADRs:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: invariant/corrective

## 1. Objective

Close the two remaining defects in Archive Extraction M001a before Egress or Eggpack consumes the extraction layer:

1. the Windows identity implementation does not compile on stable Rust because `std::os::windows::fs::MetadataExt::file_index()` is still gated by the unstable `windows_by_handle` feature and returns `Option<u64>`;
2. cleanup still performs a pathname time-of-check/time-of-use sequence:

~~~rust
if !identity.matches(path)? {
    return Err(...);
}
fs::remove_dir_all(path)
~~~

so a directory can be replaced after identity validation but before recursive deletion.

M001b must make recursive deletion operate through retained directory authority/capability, not through a revalidated pathname. Where safe object-bound removal of the root itself is unavailable, fail closed and leave bounded empty residue rather than recursively touching a replacement path.

## 2. Readiness and dependencies

M001b is dependency-ready.

The defects are directly reproduced by current main and hosted CI:

- Linux Stable checks pass;
- Rust 1.89 MSRV check passes;
- macOS workspace tests pass;
- Windows run `36220815378` fails in `eggup-archive` before tests execute:
  - E0658: unstable library feature `windows_by_handle` at `meta.file_index()`;
  - E0308: `file_index()` is `Option<u64>`, not `u64`;
  - the remaining Windows steps are skipped.

The cleanup race is structural and does not require a probabilistic reproduction to establish: `OwnedRootIdentity::matches(path)` and `fs::remove_dir_all(path)` are separate pathname operations with an attacker-controlled scheduling window between them.

No Egress or Eggpack archive integration has landed, so the defect can still be corrected before a real consumer depends on the M001a cleanup semantics.

## 3. Current evidence and external primitive review

Current M001a behavior:

- captures `(dev, ino)` on Unix;
- attempts to capture `file_index` on Windows;
- carries that identity through `DirectoryGuard` and `PersistedExtraction`;
- re-stats the path immediately before `fs::remove_dir_all(path)`;
- correctly rejects replacements that occur before the identity check;
- does not bind the recursive delete to the object that was checked.

Relevant current Rust ecosystem evidence:

- stable Rust 1.98.1 documents `MetadataExt::file_index()` and related by-handle identity methods as nightly-only under `windows_by_handle`; M001b must not depend on that unstable API;
- the `remove_dir_all` crate exposes `RemoveDir::remove_dir_contents` on an already-open `std::fs::File` and documents it as safer than its path-based removal functions for filesystem-race-sensitive callers;
- the same crate explicitly documents its path-based functions as intrinsically race-sensitive;
- `cap-std::Dir::remove_open_dir_all` is handle-oriented but explicitly warns that removal is not guaranteed atomic with respect to concurrent rename, so adopting it without threat-model analysis does not by itself prove the M001b root-identity invariant.

Research references:

- https://doc.rust-lang.org/stable/std/os/windows/fs/trait.MetadataExt.html
- https://docs.rs/remove_dir_all/latest/remove_dir_all/
- https://docs.rs/remove_dir_all/latest/remove_dir_all/trait.RemoveDir.html
- https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html

These are candidate primitives, not preselected dependencies. The implementation must prove the actual authority semantics it relies on.

## 4. Invariants that must not regress

- recursive cleanup MUST operate on the directory object/capability Eggup owns, not a pathname merely checked moments earlier;
- a foreign replacement directory, file, symlink, junction, or reparse point MUST NOT be recursively traversed or deleted;
- an attacker racing the root between validation and deletion MUST NOT redirect recursive deletion;
- drop cleanup MUST be no more destructive than explicit cleanup;
- `persist()` MUST preserve cleanup authority;
- failure to prove root-removal authority MUST leave residue and report `CleanupFailed` for explicit cleanup;
- private residue is preferable to deleting foreign state;
- all M001 traversal/link/special-file/no-clobber/decompression/digest invariants remain unchanged;
- `eggup-core` remains archive-format independent;
- no first-party `unsafe` is introduced;
- Rust 1.89 remains the workspace MSRV;
- Windows stable Rust must compile and execute the portable archive suite;
- no consumer migration proceeds until M001b closes.

## 5. Scope and non-scope

### In scope

- replace pathname-authorized recursive root cleanup with retained handle/capability-authorized content deletion;
- retain/open the cleanup capability at extraction-root creation time before archive materialization;
- carry that capability through `DirectoryGuard` and `PersistedExtraction`;
- select the smallest safe cross-platform primitive/dependency that satisfies Rust 1.89 and Eggup's authority contract;
- fix Windows stable compilation without nightly APIs;
- separate safe content deletion from final root-directory unlink/removal;
- fail closed when final root removal cannot be proven object-bound;
- add deterministic race hooks/tests so replacement can occur after any last pathname check and before any pathname-based root operation;
- run a fresh hosted Stable/MSRV/macOS/Windows matrix;
- use that current-head hosted run to supplement Acquisition M008 closure evidence once Windows reaches its curl/acquisition tests;
- update archive docs/changelog/roadmap/registry and closure evidence.

### Explicitly out of scope

- Egress changes;
- Eggpack archive integration;
- Gregg migration;
- new archive formats;
- changes to archive member parsing/bounds unrelated to cleanup;
- live destination mutation;
- authenticity/signatures;
- first-party platform FFI/unsafe;
- weakening ADR-0005 to permit deletion of an unproven foreign root;
- treating the current identity-before-delete check as sufficient.

## 6. Required production changes

### 6.1 Retained cleanup authority

The extraction root must retain an object/capability suitable for recursive content deletion.

Preferred shape:

1. create the private extraction root;
2. immediately obtain a safe open-directory capability/handle representing that exact directory;
3. store it in `DirectoryGuard`;
4. transfer it into `PersistedExtraction` on `persist()`;
5. recursively remove contents through that open object/capability.

The implementation may use a narrow external crate if std cannot express this safely on every supported platform. Any dependency must be qualified for:

- Rust 1.89;
- Linux/macOS/Windows;
- safe public API from Eggup's perspective;
- race/rename behavior;
- dependency and binary-size cost;
- maintenance/security posture.

Do not add first-party `unsafe` merely to call `openat`, `unlinkat`, `CreateFileW`, `SetFileInformationByHandle`, or equivalent platform APIs.

### 6.2 Root contents versus root name

Treat these as two separate operations.

Recursive content deletion MUST be handle/capability-relative.

For removal of the root directory entry itself:

- use an object-bound safe primitive only if its concurrent-rename semantics satisfy ADR-0005;
- otherwise leave the now-empty owned root as residue and return `CleanupFailed` from explicit cleanup;
- drop may best-effort empty the owned root but MUST NOT fall back to pathname-recursive deletion;
- never use `identity.matches(path)` followed by `remove_dir_all(path)` as the final authority boundary;
- never recursively delete a replacement path in order to avoid residue.

A final pathname operation is acceptable only if failure cannot cause recursive traversal/deletion of a foreign object. If a race can at worst remove an empty foreign directory, treat that as still violating the ownership invariant unless an ADR explicitly narrows the threat model.

### 6.3 Windows portability

Remove the stable-Windows dependency on `MetadataExt::file_index()`.

The new authority design should ideally make a separate manually captured Windows file identity unnecessary.

If identity metadata remains useful as diagnostic/secondary evidence:

- use only stable APIs available at Rust 1.89;
- represent unavailable identity explicitly;
- do not unwrap an optional identity;
- do not substitute timestamps/path strings as security identity.

The hosted Windows lane MUST compile on stable and MSRV-compatible code paths.

### 6.4 Deterministic race test seam

Add a test-only synchronization point or equivalent deterministic harness that can pause cleanup after any final pathname validation but before a pathname-based root operation.

Use it to prove one of two acceptable outcomes:

- cleanup remains bound to the original owned object and the foreign replacement is untouched; or
- cleanup fails closed/leaves residue before any destructive operation against the replacement.

A test that performs replacement only before calling `cleanup()` is insufficient for M001b.

### 6.5 M008 qualification recovery

M008 production code is not reopened by this plan.

After Windows compilation is repaired, the same current-head hosted workflow must reach:

- `eggup-acquisition`;
- `eggup-curl`;
- portable Windows process/argument tests;
- workspace all-target check.

Record that fresh run in an M008 closure supplement or equivalent evidence update. Do not claim Windows live-loopback curl behavior beyond the existing M007 limitation.

## 7. Ordered work packages

1. Add deterministic regression demonstrating the M001a check→delete window.
2. Remove/disable the unstable Windows `file_index()` implementation so Windows stable can compile during development.
3. Evaluate std-only, `remove_dir_all::RemoveDir`, `cap-std`, or another narrow safe capability primitive against the exact root/content authority requirements.
4. Record the selected primitive and why rejected alternatives do not satisfy the contract.
5. Retain the chosen cleanup capability in `DirectoryGuard`.
6. Transfer it through `persist()`.
7. Replace recursive pathname deletion with capability-relative content deletion.
8. Implement fail-closed final-root handling.
9. Add rename/replacement races after the last pathname validation, including foreign non-empty and empty directories.
10. Add Windows replacement/reparse coverage where hosted permissions permit.
11. Re-run the full M001/M001a malicious archive and cleanup matrix.
12. Run dependency/MSRV/footprint review.
13. Push and wait for a fresh hosted Stable/MSRV/macOS/Windows workflow.
14. Update M001a/M001b closure evidence, Acquisition M008 hosted evidence, roadmaps, and registry only after the matrix result is known.

## 8. Failure, restart, cancellation, and contention semantics

Cleanup must fail closed under contention.

If the owned directory is renamed or the original pathname is replaced:

- retained capability operations may clean only the originally opened owned tree;
- no traversal may begin from the replacement pathname;
- final root removal may be skipped if authority is ambiguous;
- explicit cleanup returns `CleanupFailed` with the best available residue evidence;
- drop performs no more destructive fallback.

If concurrent mutation prevents complete capability-relative deletion, return/record cleanup failure; do not loop indefinitely.

No application transaction becomes `RecoveryRequired` solely because a private extraction residue remains.

## 9. Compatibility and migration

Archive plan/member APIs should remain source compatible.

`ExtractedArchive` and `PersistedExtraction` may become non-`Clone`/non-`Sync` internally if an owned handle/capability requires it, but any public auto-trait change must be identified in closure evidence.

No Egress/Eggpack consumer migration is allowed during M001b.

Adding one narrow dependency is acceptable only with recorded footprint/MSRV evidence. Prefer disabling optional parallel/log features.

If safe cleanup cannot meet current semantics without a public API change, stop and write the API/ADR follow-up before publishing.

## 10. Required tests

At minimum:

- deterministic race: foreign non-empty directory installed after last pathname check remains intact;
- deterministic race: foreign empty directory installed after last pathname check remains intact;
- symlink replacement after last pathname check does not redirect traversal;
- Windows reparse/junction replacement after the authority boundary is preserved where hosted permissions allow;
- persisted cleanup has identical authority semantics;
- drop cleanup uses the same capability path and never falls back to `fs::remove_dir_all(path)`;
- normal explicit cleanup succeeds when the selected safe primitive supports object-bound root removal;
- otherwise normal explicit cleanup empties only the owned root and returns truthful residue evidence;
- cleanup remains bounded under concurrent file creation;
- all prior M001/M001a archive tests remain green;
- Windows stable compilation succeeds;
- Rust 1.89 workspace check succeeds;
- `eggup-core` dependency tree remains archive-free;
- Acquisition M008 portable Windows tests execute after archive compilation is repaired.

## 11. Required verification commands

~~~text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-archive --all-targets --all-features --locked
cargo test -p eggup-acquisition -p eggup-curl --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree -p eggup-archive --locked
cargo tree -p eggup-core --locked
./scripts/check-local.sh
git diff --check
~~~

Required hosted evidence:

- Stable Linux: fmt/clippy/full workspace tests/docs;
- MSRV Linux: Rust 1.89 all-target workspace check;
- macOS: full workspace tests;
- Windows: archive/acquisition/curl runtime tests, service portable tests, Windows SCM tests, and workspace all-target check.

M001b MUST NOT close on local-only evidence.

## 12. Documentation updates

Update:

- `crates/eggup-archive/README.md`;
- `architecture/archive-extraction.md`;
- root and archive changelogs;
- archive extraction roadmap;
- consumer adoption roadmap;
- Eggpack interoperability roadmap;
- registry;
- M001a closure addendum identifying why M001b supersedes its cleanup-authority conclusion;
- M001b closure record;
- M008 closure/status evidence with the fresh hosted matrix result.

## 13. Acceptance criteria

M001b closes only when:

- no recursive cleanup is authorized by `stat/identity-check(path) -> remove_dir_all(path)`;
- recursive content deletion is bound to retained owned-directory authority;
- a replacement inserted after the last pathname validation cannot be recursively deleted;
- foreign empty and non-empty replacement directories remain intact;
- Windows uses no unstable `windows_by_handle` API;
- Rust 1.89 and stable Windows compile;
- prior archive safety tests remain green;
- current hosted Stable/MSRV/macOS/Windows lanes all pass, subject only to explicitly pre-existing M007 Windows live-loopback limitations;
- the M008 Windows portable tests actually execute in that hosted run;
- Egress M006 and Eggpack M002 remain blocked until this closure;
- no high- or medium-severity cleanup-authority/portability finding remains open.

## 14. Stop conditions

Stop and write an ADR or narrower platform plan if:

- no safe Rust 1.89 primitive can perform recursive content deletion through retained directory authority on all supported platforms;
- satisfying final root removal requires first-party unsafe/FFI;
- a candidate crate's MSRV exceeds 1.89;
- the only available root-removal primitive admits deletion of a foreign replacement and cannot fail closed;
- the dependency/footprint cost is disproportionate for the optional archive leaf crate;
- the required authority semantics force a breaking public API change.

Leaving a private empty residue is an acceptable safety fallback. Recursively deleting unproven state is not.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- exact Windows compile failure corrected;
- selected capability primitive and dependency version/features;
- Rust 1.89/MSRV evidence;
- before/after cleanup algorithm;
- deterministic after-check replacement race results;
- foreign empty/non-empty/symlink/reparse results per platform;
- dependency and footprint delta;
- whether root deletion is object-bound or intentionally leaves residue;
- all M001/M001a regression results;
- fresh hosted workflow run ID and per-job conclusions;
- explicit M008 hosted-evidence supplement;
- unresolved findings by severity.

## 16. Handoff notes

M001b is the sole runtime corrective that is dependency-ready at this point.

Acquisition M008's implementation remains valid; do not rewrite its deadline serializer unless the fresh matrix finds a separate defect. Its current closure evidence is incomplete because the first current-head Windows run failed in `eggup-archive` before M008's portable Windows tests could execute.

Do not author Egress M006 or Eggpack archive M002 while M001b is open. Once M001b closes with a green hosted matrix, re-evaluate those two handoffs and only then author their implementation plans.
