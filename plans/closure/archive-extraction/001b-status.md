# Archive Extraction M001b — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md#M001b--handle-bound-cleanup-and-windows-portability-corrective`

Reviewed repository baseline: `057399640b03bb0fcee1ea86fdc5707f812e3579` (implementation commit; source tree was clean before implementation)

Implementation commit: `057399640b03bb0fcee1ea86fdc5707f812e3579` — handle-bound cleanup and Windows portability corrective; `fs_at` retained directory authority; removed nightly `file_index()` identity; capability-relative content deletion; fail-closed empty residue; deterministic after-check race tests + Windows reparse coverage; README/architecture/changelog updates.

Hosted qualification: CI run `36222536670` on the implementation commit; Stable Linux, Rust 1.89 MSRV, macOS, and Windows archive/acquisition/service lanes all passed.

## Executive finding

M001b closes both remaining M001a defects. Recursive cleanup is no longer authorized by `stat/identity-check(path) -> remove_dir_all(path)`. The extraction root is created atomically via `mkdir_at` from the parent handle and its open `File` handle is retained through `DirectoryGuard` into `PersistedExtraction`; all recursive content deletion uses only `fs_at` handle-relative operations and never traverses the replacement pathname. The stable-Windows dependency on nightly `MetadataExt::file_index()` (`windows_by_handle`, `Option<u64>`) is removed entirely. No portable object-bound root unlink exists, so explicit cleanup empties only the owned tree and reports the now-empty directory as `CleanupFailed` residue; drop best-effort empties the same way and never falls back to `fs::remove_dir_all(path)`. The same hosted run reaches the Acquisition M008 portable Windows tests, completing M008 hosted qualification. Egress M006 and Eggpack M002 are unblocked to ready-to-author; no consumer migration is performed here.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| No recursive cleanup authorized by check-then-`remove_dir_all(path)` | `remove_owned_root`/`OwnedRootIdentity` deleted; only `empty_dir_contents` via retained handle + `fs_at::*_at` exists; `grep remove_dir_all` finds no production pathname recursion | passed |
| Recursive content deletion bound to retained owned-directory authority | `create_private_root` returns `(PathBuf, File)` via `mkdir_at`; `DirectoryGuard`/`PersistedExtraction` carry `File`; `empty_dir_contents` uses `fs_at::read_dir`/`open_dir_at`/`unlink_at`/`rmdir_at` only | passed |
| Replacement after last pathname validation cannot be recursively deleted | Deterministic hook `remove_owned_root_with_hook(path, handle, Some(hook))` installs foreign non-empty/empty/symlink after content deletion but before final handling; foreign intact in all cases | passed |
| Foreign non-empty replacement preserved | `cleanup_preserves_foreign_non_empty_directory_at_original_path` (before-call) + `deterministic_race_foreign_non_empty_after_content_deletion_remains_intact` (hook-during) | passed Linux/macOS/Windows |
| Foreign empty replacement preserved | `cleanup_preserves_foreign_empty_directory_at_original_path` + `deterministic_race_foreign_empty_after_content_deletion_remains_intact` | passed; hook version proves no `rmdir` of foreign empty |
| Symlink replacement does not redirect traversal | `cleanup_preserves_symlink_substitution_at_original_path` (Unix, before-call) + `deterministic_race_symlink_after_content_deletion_is_not_followed` (Unix, hook-during) | passed |
| Windows reparse/junction replacement preserved | `deterministic_race_windows_reparse_after_authority_boundary_is_preserved` (Windows-only; best-effort symlink_dir, skips gracefully without privileges) | passed on hosted Windows lane |
| Persisted cleanup identical authority | `explicit_cleanup_after_replacement_fails_closed_and_persists_identity` + `deterministic_race_persisted_cleanup_uses_same_handle_authority` (same shared primitive) | passed |
| Drop uses same capability path, never `fs::remove_dir_all(path)` | `drop_cleanup_preserves_foreign_replacement_at_original_path` strengthened to assert owned moved tree emptied + foreign intact; `Drop` calls only `empty_dir_contents(handle)` | passed |
| Normal explicit cleanup truthful under residue fallback | `normal_explicit_cleanup_empties_owned_root_and_reports_empty_residue` (renamed): empties owned file, returns `CleanupFailed` with empty-dir residue | passed; intentional semantic change documented |
| Cleanup bounded under concurrent creation | `cleanup_remains_bounded_under_concurrent_file_creation` spawns 200-file writer during handle deletion; terminates via bounded passes (8 x 10k) | passed |
| Prior M001/M001a archive tests green | All malicious classes (traversal, links, bounds, digests, truncation) pass; `extract()` preserves original category with empty-residue path so kind assertions hold | passed 29/29 Linux/macOS, 27/27 Windows |
| Windows stable compilation | No `windows_by_handle`/`file_index()` in tree; hosted Windows lane compiles `eggup-archive` on stable | passed |
| Rust 1.89 workspace check | `cargo +1.89.0 check --workspace --all-targets --locked` local + hosted MSRV job | passed |
| `eggup-core` archive-free | `cargo tree -p eggup-core --locked` shows `sha2` only | passed |
| M008 portable Windows tests execute | Hosted run `36222536670` Windows lane runs `eggup-acquisition` (34), `eggup-archive` (27), `eggup-curl` portable tests before service tests + workspace check | passed |

## Production implementation evidence

- `crates/eggup-archive/Cargo.toml` — adds `fs_at = { version = "0.2.1", default-features = false }` (log/workaround-procmon disabled). No other dependency change.
- `crates/eggup-archive/src/lib.rs::open_dir_handle` — portable directory handle open (`File::open` on Unix; `OpenOptions` + `FILE_FLAG_BACKUP_SEMANTICS` on Windows, stable `OpenOptionsExt` only).
- `crates/eggup-archive/src/lib.rs::create_private_root` — opens parent handle, creates each candidate via `fs_at::OpenOptions::mkdir_at` (mode `0700` on Unix via `fs_at::os::unix::OpenOptionsExt`), returns `(PathBuf, File)` atomically. Removes `DirBuilder` + `OwnedRootIdentity::capture`.
- `crates/eggup-archive/src/lib.rs::empty_dir_contents` — bounded (8 passes, 10k entries/pass, 32 depth) recursive deletion via `fs_at::read_dir` (collect-then-delete to satisfy borrow checker), `open_dir_at` (follow disabled by default), `unlink_at`/`rmdir_at`. Skips `.`/`..`, unlinks symlinks without following, treats `NotFound` as concurrent progress, fails closed without looping indefinitely.
- `crates/eggup-archive/src/lib.rs::remove_owned_root_with_hook` — empties via handle, invokes test-only `hook(&path)` (production `None`), then returns `CleanupFailed` with residue without any pathname recursion. `remove_owned_root_via_handle` is the production wrapper.
- `crates/eggup-archive/src/lib.rs::DirectoryGuard` — stores `Option<File>` + `active`; `disarm` returns `(PathBuf, File)`; `cleanup` delegates to handle primitive; `Drop` best-effort calls `empty_dir_contents` only.
- `crates/eggup-archive/src/lib.rs::PersistedExtraction` — stores `File` handle; `cleanup` delegates to same primitive. Manual `Debug` uses `finish_non_exhaustive` (handle omitted).
- `crates/eggup-archive/src/lib.rs::extract` — failure path preserves original category via new `with_empty_residue` (residue attached without overwriting kind); explicit cleanup uses `with_residue` (`CleanupFailed`).
- `crates/eggup-archive/src/lib.rs::OwnedRootIdentity` — deleted entirely, including Windows `file_index()` path.
- Primitive selection recorded: `fs_at 0.2.1` (MSRV 1.71, Unix + Windows, safe API, small closure `cfg-if`/`cvt`/`libc`/`nix`). Rejected: std-only (no stable `*at` APIs; `File::open` cannot open dirs on Windows); `remove_dir_all::RemoveDir` (secure variant MSRV is latest stable, exceeding 1.89; heavier); `cap-std` (MSRV 1.70 ok but heavy `rustix`/`cap-primitives` tree and `remove_open_dir_all` warns it is not atomic w.r.t. concurrent rename, offering no stronger root-removal guarantee for the added cost).
- Tests added/updated:
  - updated `failure_cleans_only_its_owned_partial_root_and_drop_cleans_success` to expect empty residue + preserved kind;
  - updated `persist_transfers_cleanup_responsibility_and_cleanup_is_explicit` to expect `CleanupFailed` + empty dir;
  - renamed `normal_explicit_cleanup_still_succeeds_with_identity_check` to `normal_explicit_cleanup_empties_owned_root_and_reports_empty_residue`;
  - updated `extracted_file_can_be_prepared_as_an_ordinary_core_artifact` for residue cleanup;
  - strengthened `drop_cleanup_preserves_foreign_replacement_at_original_path` and foreign tests to assert owned moved tree emptied;
  - new `deterministic_race_foreign_non_empty/empty_after_content_deletion_remains_intact`, `deterministic_race_symlink_after_content_deletion_is_not_followed` (Unix), `deterministic_race_persisted_cleanup_uses_same_handle_authority`, `cleanup_remains_bounded_under_concurrent_file_creation`, `deterministic_race_windows_reparse_after_authority_boundary_is_preserved` (Windows).

## Exact commands and results

Local Darwin arm64, Rust 1.89.0 (implementation commit `0573996`):

```text
cargo fmt --all -- --check                                      passed (after cargo fmt)
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test -p eggup-archive --all-targets --all-features --locked  passed (29/29)
cargo test -p eggup-acquisition -p eggup-curl --locked          passed (acquisition 36, curl 22)
cargo test --workspace --all-targets --all-features --locked    passed
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked          passed
cargo tree -p eggup-archive --locked                             passed; fs_at 0.2.1 + cvt/libc/nix/bitflags/cfg_aliases added, log disabled
cargo tree -p eggup-core --locked                                passed; sha2 only
./scripts/check-local.sh                                         passed
git diff --check                                                passed
```

Hosted CI run `36222536670` (implementation commit):

- Stable Linux: fmt/clippy/full workspace tests (archive 29/29)/docs passed.
- MSRV Linux: Rust 1.89 workspace all-target check passed.
- macOS: full workspace tests passed (archive 29/29).
- Windows: `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl` passed (acquisition 34, archive 27/27 including Windows reparse test, curl portable); service UTF-8/SCM portable tests passed (1+1+1+15); `cargo check --workspace --all-targets` passed. Archive compilation on stable Windows succeeds; no `windows_by_handle`/`file_index` error.

## Invariant review

- Recursive deletion operates only on the retained directory object; the replacement pathname is never traversed.
- Foreign non-empty/empty directories, regular files, symlinks, and Windows reparse points installed before or during cleanup (via deterministic hook after the last handle operation) are never recursively deleted.
- Drop is no more destructive than explicit cleanup (same `empty_dir_contents` via handle, no pathname fallback).
- `persist()` preserves cleanup authority by moving the handle.
- Failure to prove root-removal authority leaves bounded empty residue and reports `CleanupFailed` for explicit cleanup; private residue is preferred over deleting foreign state.
- All M001 traversal/link/special-file/no-clobber/decompression/digest invariants unchanged; `eggup-core` remains archive-format independent; no first-party `unsafe` (workspace deny clean); MSRV 1.89 preserved; Windows stable compiles and executes the portable suite.

## Failure and recovery review

| Failure | Result |
|---|---|
| Foreign non-empty/empty/file/symlink/reparse installed before or during cleanup | `CleanupFailed` with original-path residue; foreign fully intact; owned tree (possibly renamed) emptied via handle |
| Concurrent file creation during cleanup | Bounded passes terminate; `CleanupFailed` with residue; no indefinite loop |
| `empty_dir_contents` hits entry/depth/pass bound or undeletable entry | `Io` internally, surfaced as `CleanupFailed` with residue; no partial pathname deletion |
| Extraction inner failure (bad archive, bounds, digest) | Original category preserved with empty-residue path (via `with_empty_residue`); no live mutation |
| Explicit cleanup in normal case (no attacker) | Owned tree emptied, empty dir left, `CleanupFailed` with residue (documented fallback) |
| Drop without explicit cleanup | Best-effort empty via handle; empty dir left; no error surfaced; never `remove_dir_all(path)` |
| `mkdir_at` collision | Next unique name attempted (32 attempts); other errors map to `Io` |

No application transaction becomes `RecoveryRequired` solely because a private extraction residue remains.

## Compatibility and migration review

Archive plan/member APIs remain source compatible. `ExtractedArchive` and `PersistedExtraction` gain a retained `std::fs::File` handle: both are now `Send` but not `Sync` (previously `Send+Sync`); neither was or is `Clone`. No consumer migration proceeds during M001b; Egress M006 and Eggpack M002 remain un-authored here. `fs_at 0.2.1` added with default features disabled (no `log`/`parallel` surface). If safe cleanup later requires a public API change, that is a separate ADR/follow-up; leaving empty residue is the accepted fallback.

## Security review

The M001a check→delete TOCTOU (`matches(path)` then `remove_dir_all(path)`) is eliminated; no recursive pathname operation remains. Handle-relative `open_dir_at` defaults to no-follow, so symlinks/reparse points are unlinked, never traversed. Directory iteration state is handle-bound; `unlink_at`/`rmdir_at` resolve relative to the owned handle, so a renamed-away owned tree is still the deletion target while the replacement pathname is untouched. The deterministic hook proves the worst-case interleaving (replacement after the last handle operation) cannot redirect deletion. No authenticity/signature claim is added; SHA-256 remains integrity evidence only. No medium-or-higher cleanup-authority/portability finding remains open.

## Documentation and operations evidence

- `crates/eggup-archive/README.md` documents handle authority, `mkdir_at` creation, `fs_at` content deletion, empty-residue fallback, and `Send`-but-not-`Sync`.
- `architecture/archive-extraction.md` records `fs_at` dependency (features disabled), handle retention, empty-residue semantics, and failure preservation.
- `crates/eggup-archive/CHANGELOG.md` and root `CHANGELOG.md` record the M001b Unreleased entry with no-publication/no-migration disclaimer.
- `plans/subsystems/archive-extraction-roadmap.md` moves M001b to closed; M002/M003 to ready-to-author.
- `plans/subsystems/consumer-adoption-roadmap.md` moves Egress M006 to ready-to-author.
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` moves M002 to ready-to-author.
- `plans/subsystems/acquisition-transport-roadmap.md` and `plans/closure/acquisition-transport/008-status.md` record M008 hosted qualification from this same run.
- `plans/registry.md` records M001b/M008 closures and re-unblocks Egress M006 + Eggpack M002.

## Unresolved findings

- None high or medium. Informational: final root unlink remains intentionally unimplemented (no portable object-bound primitive without first-party unsafe/FFI); every explicit cleanup therefore reports `CleanupFailed` with an empty-dir residue even in the normal case. Callers treat the residue path as best-effort empty and remove it with a plain empty-dir removal when appropriate. A future ADR could narrow the empty-foreign-dir threat model or adopt a platform-specific object-bound unlink, but that is out of scope for M001b.
- Informational: Windows junction/reparse runtime coverage is best-effort (`symlink_dir` may require privileges; the test passes trivially if creation is denied while still asserting the owned target stays intact).

## Roadmap disposition and downstream unblock

Archive M001b moves from ready to closed. This unblocks (to ready-to-author only; no implementation plan is written here):

- **Consumer Adoption M006 Egress**: blocked → ready to author. The Egress plan may now define replacing duplicated generic extraction/pair rollback with the qualified handle-bound extraction plus multi-artifact transactions.
- **Eggpack Interoperability M002 archive handoff**: blocked → ready to author. The adapter plan may now connect `ManifestProjection::Archive` evidence to the qualified extraction boundary.

Archive M001 remains historically closed; M001a remains historical/superseded (its identity-check conclusion is superseded by handle authority; see its status addendum reference to M001b). Acquisition M008 moves to closed via the same hosted run (see its closure supplement). No Gregg migration is authorized. No Eggup installer-generator work is authorized.

## Registry updates

- Archive Extraction M001b: ready → closed; `fs_at` added; `Send`-but-not-`Sync` noted.
- Acquisition M008: implemented/hosted-pending → closed (Windows portable tests executed in run `36222536670`).
- Consumer Adoption M006 Egress: blocked → ready to author.
- Eggpack Interoperability M002: blocked → ready to author.
- Archive roadmap M002/M003 (Egress/Eggpack sides): blocked → ready to author.
