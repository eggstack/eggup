# Verified Update Core M001 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/001-repository-workspace-foundation.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M001--repositoryworkspace-foundation-and-contract-harness`

Reviewed repository baseline: `5b6cf13ed7e6e2bd40951216f29ea68d7333aadf`

## Implementation commits/PRs

- `4ae7642` (`feat: establish eggup core workspace foundation`)
- No PR was required for this local implementation pass.

## Executive finding

M001 is complete. Eggup is now a Rust 1.89 virtual workspace containing only
the independently consumable, dependency-free `eggup-core` package. The
workspace has deterministic private filesystem fixtures, a test-only failure
point scaffold, repository documentation, a lockfile, and ordinary CI. No
live updater, network, service, installation, or release behavior was added.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Rust workspace with one core member | `Cargo.toml`, `cargo check --workspace --all-targets --locked` | passed |
| MSRV explicitly 1.89 | workspace `rust-version`, `rustc +1.89.0 --version`, MSRV CI job | passed |
| No HTTP/TLS/service dependency | `cargo tree --workspace --locked` shows only `eggup-core` | passed |
| Unsafe code forbidden | workspace lint and crate attribute | passed |
| Isolated temporary fixture support | `src/test_support.rs` and unit tests | passed |
| Escape paths rejected | `tests::fixture_rejects_escape_paths` | passed |
| No network/live updater behavior | crate API and architecture review | passed |
| Repeatable local verification | `scripts/check-local.sh` | passed |
| Ordinary CI without release side effects | `.github/workflows/ci.yml` review | passed |
| Licensing convention | MIT license matches adjacent Eggstack consumer crates | passed |

## Production implementation evidence

The package exposes only a version constant. Test-only support provides a
private temporary installation-root fixture and a deterministic failure-point
enum/injector scaffold. The fixture rejects absolute and parent-traversal
paths, writes exact bytes, and cleans up owned state on drop.

## Exact commands run and results

All commands passed on the local macOS environment with Rust 1.89.0:

```text
rustc +1.89.0 --version
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
./scripts/check-local.sh
```

The test suite passed 3/3 tests. Hosted CI was not available during this
pass; the workflow is present and limited to stable checks plus the Rust 1.89
MSRV check.

## Invariant review

- Core has no transport, TLS, service-manager, consumer, or runtime dependency.
- No unsafe code is permitted.
- Tests use private temporary directories and do not touch installation or
  service paths.
- The public foundation API does not encode a product, provider, or release
  policy.

## Failure/recovery review

There is no live mutation state machine in M001. Fixture creation, path
validation, and cleanup are covered. The failure injector defaults to no
failure and is not activated by production code; later transaction milestones
own mutation fault behavior.

## Compatibility and migration review

This is the first Eggup package surface, so no consumer migration is required.
The crate name, Rust MSRV, and MIT license are established. No package was
published.

## Security review

The foundation contains no external input processing beyond test-only relative
fixture paths. Escape paths are rejected before filesystem access, cleanup is
limited to the fixture's private root, and no privilege escalation or network
access exists.

## Documentation/operations evidence

`README.md`, `architecture/overview.md`, `CHANGELOG.md`, crate documentation,
the local verification script, and ordinary CI were added. The documentation
explicitly says that production updater capability is not yet present.

## Unresolved findings

None. Hosted CI evidence is pending operationally, not a code defect.

## Disposition and roadmap transition

M001 is closed. M002 is unblocked and ready for handoff against closure commit
`4ae7642`. M003 and M004 remain correctly blocked on M002 closure.

