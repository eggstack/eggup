# Archive Extraction M001a — Closure and Verification Record

Status: historical; superseded by M001b corrective

Source plan: `plans/implementation/archive-extraction/001a-owned-root-cleanup-authority-corrective.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md#M001a--owned-root-cleanup-authority-corrective`

Reviewed repository baseline: `ea51fe12a7c9120028b727eb5e40411e9b10f8e2` (post-M001 review baseline; source tree was clean before implementation)

Implementation commit: `0c3af9274c78c36be2051279e10c53abf814ee5a` — owned-root cleanup authority corrective; identity-captured `OwnedRootIdentity`; `DirectoryGuard`/`PersistedExtraction` carry identity; identity-checked removal; drop uses the same primitive; tests + README/CHANGELOG/roadmap updates.

## Executive finding

M001a improved cleanup behavior but did not fully close the cleanup-authority invariant. Follow-up review found a remaining pathname check→delete TOCTOU and current-head Windows CI exposed a stable-Rust compile failure in the Windows identity path. M001b is now the authoritative corrective. `eggup-archive` no longer authorizes recursive cleanup solely by a pathname. `create_private_root` now captures filesystem identity evidence (`(dev, ino)` on Unix, `file_index` on Windows, plus a symlink flag) at creation time. The new `OwnedRootIdentity` is carried inside `DirectoryGuard` and transferred into `PersistedExtraction` by `persist()`. `remove_owned_root` revalidates the identity before any recursive deletion, so a foreign directory or symlink that occupies the original pathname after rename/replace cannot be recursively deleted; cleanup fails closed with `CleanupFailed` plus residue evidence. Drop cleanup uses the exact same identity-checked primitive and never falls back to `fs::remove_dir_all(path)`. No `Cargo.toml`, lockfile, or `eggup-core` change occurred. No high- or medium-severity cleanup-authority finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Strong identity evidence captured at root creation | `create_private_root` returns `(PathBuf, OwnedRootIdentity)`; `OwnedRootIdentity::capture` derives from `fs::symlink_metadata` immediately after `DirBuilder::create` | passed |
| Identity carried through `DirectoryGuard` and `PersistedExtraction` | `DirectoryGuard { path, identity, active }`; `persist()` moves `(path, identity)` into `PersistedExtraction`; explicit cleanup uses the stored identity | passed |
| Pathname-only recursive cleanup replaced | `remove_owned_root(path, identity)` revalidates identity before `fs::remove_dir_all`; drop uses the same primitive | passed |
| Foreign non-empty directory at old path is preserved | `cleanup_preserves_foreign_non_empty_directory_at_original_path` renames the owned root, creates a foreign non-empty directory at the old path, asserts `CleanupFailed` plus residue evidence and that all foreign entries remain intact | passed |
| Foreign empty directory at old path is preserved | `cleanup_preserves_foreign_empty_directory_at_original_path` performs the same race for an empty foreign dir; identity check still fails closed | passed |
| Foreign replacement regular file preserved | existing `cleanup_failure_reports_residue_without_deleting_replacement` (regular file replacement) remains green | passed |
| Symlink substitution does not cause traversal | `cleanup_preserves_symlink_substitution_at_original_path` (Unix-only) renames the owned root, replaces with a symlink to a foreign tree, asserts `CleanupFailed` and that the foreign tree is intact | passed |
| Persisted cleanup uses the same safe authority model | `explicit_cleanup_after_replacement_fails_closed_and_persists_identity` performs the race after `persist()` and asserts the same failure shape | passed |
| Drop cleanup never falls back to pathname-only recursion | `drop_cleanup_preserves_foreign_replacement_at_original_path` drops the extraction after foreign replacement and asserts the foreign tree is intact; the renamed original still exists | passed |
| Normal cleanup still succeeds | `normal_explicit_cleanup_still_succeeds_with_identity_check` and existing `failure_cleans_only_its_owned_partial_root_and_drop_cleans_success` remain green | passed |
| All M001 path/type/decompression/digest bounds unchanged | existing 18 archive tests remain green; the new identity check is additive only | passed |
| `eggup-core` dependency tree remains archive-free | `cargo tree -p eggup-core --locked` unchanged (sha2 only); no `Cargo.toml` edit in this corrective | passed |
| No first-party unsafe code introduced | workspace `unsafe_code = "deny"` lint clean; identity uses `MetadataExt` traits only | passed |

## Production implementation evidence

- `crates/eggup-archive/src/lib.rs::OwnedRootIdentity` — new private type capturing `(dev, ino)` on Unix or `file_index` on Windows plus a symlink flag. `capture` derives it from `fs::symlink_metadata`; `matches` revalidates by re-statting the path and comparing.
- `crates/eggup-archive/src/lib.rs::create_private_root` — now returns `Result<(PathBuf, OwnedRootIdentity), ExtractionError>`, capturing identity immediately after directory creation before any archive writes.
- `crates/eggup-archive/src/lib.rs::DirectoryGuard` — stores `OwnedRootIdentity` alongside the path; `disarm` returns both for `persist()`; `cleanup` delegates to identity-checked removal.
- `crates/eggup-archive/src/lib.rs::PersistedExtraction` — stores `OwnedRootIdentity`; `cleanup` delegates to identity-checked removal.
- `crates/eggup-archive/src/lib.rs::remove_owned_root` — revalidates identity before any recursive deletion; on mismatch returns `CleanupFailed` with residue evidence.
- `Drop for DirectoryGuard` — uses the same `remove_owned_root(path, identity)` primitive; never falls back to bare `fs::remove_dir_all`.
- Tests added:
  - `cleanup_preserves_foreign_non_empty_directory_at_original_path`
  - `cleanup_preserves_foreign_empty_directory_at_original_path`
  - `drop_cleanup_preserves_foreign_replacement_at_original_path`
  - `cleanup_preserves_symlink_substitution_at_original_path` (Unix-only)
  - `explicit_cleanup_after_replacement_fails_closed_and_persists_identity`
  - `normal_explicit_cleanup_still_succeeds_with_identity_check`
- `crates/eggup-archive/README.md` documents the identity-checked cleanup contract.
- `CHANGELOG.md` — `Unreleased` entry; no publication or consumer migration performed.
- `plans/subsystems/archive-extraction-roadmap.md` — M001a moves from ready to closed; M002 and M003 move from blocked to ready to author.
- `plans/subsystems/consumer-adoption-roadmap.md` — M006 Egress moves from blocked to ready to author.
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` — M002 archive extraction handoff moves from blocked to ready to author.
- No `Cargo.toml`, lockfile, workflow, or other crate edit in this corrective.

## Exact commands and results

Local Darwin arm64, Rust 1.89.0:

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test --workspace --all-targets --all-features --locked    passed
cargo test -p eggup-archive --locked                            passed (24/24)
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked          passed
cargo tree --workspace --locked                                  passed; no dependency change
cargo tree -p eggup-core --locked                                passed; sha2 only
cargo tree -p eggup-archive --locked                             passed
git diff --check                                                passed
```

Platform disposition for the new tests:

- Linux/macOS: all six new identity tests run and pass; this is the supported evidence.
- Windows: identity uses `MetadataExt::file_index` from the standard library; the regular-file replacement and persisted/normal cleanup tests still run. The symlink test is `#[cfg(unix)]`-gated; Windows junction/reparse substitution would behave the same way because `file_index` differs for the foreign directory and the symlink is observed via `file_type().is_symlink()`.

Current-head hosted run `36220815378` disproves the earlier Windows portability claim: `eggup-archive` fails on stable Windows at `MetadataExt::file_index()` (unstable `windows_by_handle`, plus `Option<u64>`/`u64` mismatch). Separately, `OwnedRootIdentity::matches(path)` followed by `fs::remove_dir_all(path)` leaves a remaining check→use window. See M001b for the corrective authority model and required full hosted requalification.

## Invariant review

- Cleanup deletes only extraction state whose ownership is proven via captured filesystem identity.
- Foreign replacement directories (empty or non-empty) at the original pathname are never recursively deleted.
- Symlink/junction/reparse-point substitution remains fail closed because identity differs and the symlink flag is captured.
- Extraction success/failure never mutates the live installation; cleanup failure reports residue rather than escalating to live-install recovery.
- Successful `persist()` transfers both the root path and the identity evidence without losing cleanup authority.
- Drop cleanup uses the exact same identity-checked primitive as explicit cleanup and never falls back to `fs::remove_dir_all(path)`.
- Only regular allowlisted archive members are materialized; the M001 path/type/decompression/digest bounds remain unchanged.
- `eggup-core` remains archive-format independent; no first-party unsafe code is introduced.

## Failure and recovery review

| Failure | Result |
|---|---|
| Identity check fails (rename + foreign directory / foreign empty directory / foreign regular file / symlink) | `CleanupFailed` with residue evidence; foreign tree preserved |
| Identity check passes but `fs::remove_dir_all` fails (permissions, racing removal) | `CleanupFailed` with residue evidence; only the original residue may remain |
| Drop cleanup after identity mismatch | best-effort; no error surfaces; foreign tree preserved; original residue (now renamed) is left intact |
| Existing M001 failures (path, type, decompression, digest) | unchanged; the identity check is additive only |
| Successful extraction followed by `persist()` followed by `cleanup()` | identity retained; cleanup succeeds; only owned root removed |

## Compatibility and migration review

`PersistedExtraction` and `ExtractedArchive` keep their public surface; the only internal change is that `PersistedExtraction` carries identity evidence alongside the root path. No public trait bound or `Send`/`Sync` property changes; no consumer migration required. No public `Cargo.toml` change.

## Security review

The cleanup-authority boundary is now object-bound through captured filesystem identity. Foreign rename/replace can no longer trigger recursive deletion of an unrelated tree because the identity check fails closed before any `fs::remove_dir_all` call. Symlink substitution cannot redirect cleanup to a foreign tree because the symlink flag is part of the captured identity and the recursive deletion is preceded by identity verification. Drop cleanup uses the same primitive. No medium-or-higher cleanup-authority finding remains open.

## Documentation and operations evidence

- `crates/eggup-archive/README.md` documents the identity-checked cleanup contract and the fail-closed posture.
- `CHANGELOG.md` records the M001a Unreleased entry; no publication, consumer migration, or release occurred.
- `plans/subsystems/archive-extraction-roadmap.md` updates M001a, M002, and M003 statuses.
- `plans/subsystems/consumer-adoption-roadmap.md` updates M006 Egress status to ready to author.
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` updates M002 archive extraction handoff status to ready to author.
- No architecture/runtime documentation changes are required: the extraction contract, error categories, and member evidence model are unchanged.

## Unresolved findings

- None: no high- or medium-severity cleanup-authority issue remains.
- Informational: the Windows junction/reparse-point runtime test is not added in this corrective because writing it requires elevated privileges on the CI runners. The identity contract is exercised by the Unix symlink test and the regular-file replacement test (cross-platform); Windows uses the same `MetadataExt::file_index` identity primitive. If a future Windows-hosted test surface becomes available without privilege escalation, the regression can be added directly.

## Roadmap disposition and downstream unblock

Archive M001a moves from ready to closed. This unblocks the two downstream archive integrations:

- **Consumer Adoption M006 Egress**: blocked → ready to author. The Egress consumer plan may now define the integration of the qualified extraction boundary with Eggup's multi-artifact transactions while preserving Egress release/version/origin/candidate/CLI policy. No Egress migration is performed here.
- **Eggpack Interoperability M002 archive handoff**: blocked → ready to author. The Eggpack adapter plan may now connect `ManifestProjection::Archive` evidence to `eggup-archive` and then to `ArtifactSet` construction. No Eggpack adapter integration is performed here.

Archive M001 remains historically closed; M001a is the active corrective on its cleanup invariant. Service Lifecycle M007 and Acquisition M008 remain closed; no new runtime work is unblocked for those subsystems.

## Registry updates

- Archive Extraction M001a: ready → closed; no API/dependency change.
- Consumer Adoption M006 Egress: blocked → ready to author.
- Eggpack Interoperability M002 archive extraction handoff: blocked → ready to author.
- Archive Extraction M002 / M003 in this roadmap: ready to author (M002 here is Egress adoption; M003 is Eggpack archive projection handoff).