# Verified Update Core M002 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/002-domain-and-prepared-transaction.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M002--core-domain-and-prepared-transaction-contract`

Reviewed repository baseline: `076a489`

## Implementation commits/PRs

- `5c259f3` (`feat: add prepared artifact transaction domain`)
- `adaada4` (`test: isolate prepared-stage cleanup checks`)
- No PR was required for this local implementation pass.

## Executive finding

M002 is complete. Eggup now models validated opaque product/release/member
identities, one- and multi-member artifact sets, explicit installation plans,
destination ownership and permission intent, declared integrity/authenticity
requirements, private staging, and a prepared transaction type. Preparation
validates all inputs and copies local regular files into private stage state;
it does not mutate live destinations and exposes no commit operation.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Validated opaque product/release/member identities | `ProductId`, `ReleaseId`, `MemberId` constructors and tests | passed |
| Non-empty artifact sets and unique member IDs | `ArtifactSet::new`, negative tests | passed |
| Exact installation root and normalized destinations | `InstallPlan`, `normalize_relative_path` | passed |
| Duplicate destination rejection | normalized `bin/./app` vs `bin/app` test | passed |
| Traversal/absolute/control-character rejection | destination validation tests | passed |
| Regular-file and symlink source policy | `symlink_metadata` and Unix symlink test | passed |
| Source/install-root alias protection | canonical source containment check | passed |
| Private staging with cleanup ownership | `Stage` drop and injected create/copy failures | passed |
| Single/two/representative bundle preparation | one-member and two-member tests | passed |
| No live destination mutation | prepared-bundle assertion and domain review | passed |
| Prepared-vs-commit phase separation | `PreparedTransaction` has no commit API | passed |

## Production implementation evidence

The public domain is split across `domain.rs`, `stage.rs`, and `error.rs`.
`InstallPlan::new` requires an existing absolute installation directory and
validates each source and destination before preparation. `Stage` creates an
owned hidden sibling directory on the same local temporary fixture scope,
copies regular files, applies executable intent on Unix, and removes only its
own directory on drop or preparation failure.

## Exact commands run and results

All commands passed on the local macOS environment with Rust 1.89.0:

```text
rustc +1.89.0 --version
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
./scripts/check-local.sh
```

The final test suite passed 8/8 tests. An earlier cleanup assertion was
corrected after it was shown to observe another parallel test's valid stage;
the final isolated assertion and complete suite pass.

## Invariant review

- Core remains free of network, TLS, service-manager, consumer, and runtime
  dependencies.
- Preparation performs no live destination creation, replacement, or deletion.
- Destination paths are explicit and root-relative; source paths are explicit
  and conservative link handling rejects symlinks.
- Product/release values remain opaque caller policy.

## Failure/rollback/recovery review

Rollback and live mutation are intentionally not present in M002. Injected
stage-creation and stage-copy failures return errors, leave destinations
untouched, and clean the transaction's private stage. Locking, revalidation,
backup, commit, rollback, and recovery evidence remain M003 responsibilities.

## Compatibility and migration review

No existing Eggup consumer exists, so no migration is required. The public
names follow canonical planning terminology and do not encode eggsact, Egress,
CodeGG, or release-ordering policy.

## Security review

Path traversal, absolute destinations, control characters, duplicate members,
duplicate normalized destinations, symlink sources, non-regular sources, and
sources inside the installation root fail closed. Stage ownership is explicit,
cleanup is scoped to the generated stage path, and no privilege escalation or
network access exists.

## Documentation/operations evidence

`crates/eggup-core/docs/domain.md` documents the state boundary and its limits;
the architecture overview links it. Local verification and CI remain the
M001 baseline. No service or deployment operation was added.

## Unresolved findings

None. Crash recovery and cross-platform live replacement are intentionally
deferred to M003 and are not defects in this milestone.

## Disposition and roadmap transition

M002 is closed. Its hard dependencies are satisfied for both M003 and M004;
both plans are ready against implementation baseline `adaada4`. The requested
next sequential handoff is M003.

