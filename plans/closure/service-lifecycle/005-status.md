# Service Lifecycle M005 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`

Reviewed repository baseline: `84076a066e498f2e4ec4ed5472af4359c0b93af7`

## Implementation commits and hosted CI

- `b6d45d52f943fbe66d8710d3bf37722413fc2722` — public manager-neutral lifecycle/transaction orchestration, tests, and README documentation.
- `84076a066e498f2e4ec4ed5472af4359c0b93af7` — add the required `0.1.0` version to the `eggup-core` path dependency so Cargo packaging accepts the manifest.
- Hosted [CI run 36008996209](https://github.com/eggstack/eggup/actions/runs/36008996209) passed all four lanes on `84076a0`: stable formatting, clippy, workspace tests and docs; Rust 1.89 workspace check; macOS workspace tests; Windows workspace check.
- No PR, publication, service migration, or privileged native service mutation was performed.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Direct Core composition without copied rollback | `eggup-service` depends on `eggup-core`; one `commit_with_lifecycle` entrypoint calls `ValidatedTransaction::commit_with_post_commit` and preserves the exact `TransactionReceipt` | passed |
| Acyclic, bounded dependency footprint | `cargo tree -p eggup-service --locked` shows only Core and `windows-args` on this host; Core does not depend on service; no acquisition/network dependency entered service | passed |
| Owned-only preflight | Tests reject Absent, Foreign, Unknown ownership and Transitioning/Unknown states before artifact mutation | passed |
| Quiesce a running service | Running pre-state is stopped and re-inspected; stop error, incomplete stop, and ownership drift prevent Core invocation; stopped pre-state receives no redundant stop | passed |
| Success `RestoreIntent` mapping | Tests cover Preserve from Running and Stopped, EnsureRunning from Stopped, and EnsureStopped from Running; the post-install test reads both committed artifact members | passed |
| KeepInstalled | Check and transition failures retain Core Committed disposition and separate lifecycle evidence; no hidden service retry or artifact rollback | passed |
| RollBack | Failed check and check panic attempt stop-before-rollback; Core restores old artifacts; prior lifecycle state is restored after rollback; restoration failure remains separate | passed |
| Core error and recovery behavior | Injected pre-receipt lock failure restores prior service state; RecoveryRequired preserves Core evidence and suppresses automatic restart | passed |
| Bounded callback | One monotonic deadline supplies remaining time to transitions/check/rollback quiescence; tests verify the check receives a shrinking budget. Caller callbacks must honor it; synchronous user code is not forcibly cancelled | passed |
| No service-definition migration or elevation | No automatic install/refresh; existing adapters and `ServiceManager` contracts remain intact; no elevation path added | passed |
| Package/docs/platform qualification | package archive generated; workspace docs passed; Rust 1.89 and Windows-target service check passed locally; hosted macOS and Windows lanes passed | passed |

## Public API and behavior

`crates/eggup-service/src/lifecycle_update.rs` adds `LifecycleUpdatePolicy`,
the caller-owned `PostInstallCheck` seam and `NoPostInstallCheck`, bounded
structured `LifecycleFailure`, `LifecycleUpdateError`, and
`LifecycleUpdateReceipt`. The API is re-exported from the crate root.

Preflight requires an existing `Owned` registration in Running or Stopped
state. Running services are stopped and confirmed before Core mutation. On
success, `RestoreIntent` selects the desired new-generation state; Preserve
keeps a previously stopped service stopped. On successful artifact rollback,
M005 restores the original pre-update state regardless of the success intent.
RecoveryRequired never triggers an automatic start. KeepInstalled retains the
new generation when post-commit service/check work fails. RollBack attempts to
quiesce a possibly running new generation before Core rollback. Manager/check
failure evidence remains distinct from Core artifact disposition.

The existing `HealthProbe` remains unchanged and is not treated as a bounded
check. The new callback receives the remaining shared budget and documents
that it must self-bound. Manager inspections continue to use the existing
adapter's bounded behavior. The optional final observation occurs after Core
returns and cannot change its terminal artifact disposition.

## Exact commands and results

Environment: macOS 15 / Darwin 25.6.0 arm64; stable Rust 1.98.1; Rust 1.89.0.

Passed locally:

```text
cargo fmt --all -- --check
scripts/check-local.sh
cargo test -p eggup-service --all-targets --all-features --locked       # 83 passed
cargo test -p eggup-core --all-targets --all-features --locked          # 43 passed
cargo test --workspace --all-targets --all-features --locked            # passed
cargo doc --workspace --no-deps --locked
cargo package -p eggup-service --locked --no-verify --allow-dirty        # 8 files, 271.4 KiB unpacked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-service --all-targets --locked              # 83 passed
cargo check -p eggup-service --all-targets --locked --target x86_64-pc-windows-msvc
cargo tree -p eggup-service --locked
cargo tree -p eggup-core --locked
git diff --check
```

The local Rust 1.89 workspace check emitted one existing unused-import warning
in `crates/eggup-core/src/stage.rs`; it completed with zero errors. Stable
hosted CI clippy passed with warnings denied. `cargo package --no-verify`
validates package construction/manifest but intentionally does not claim a
published-crate dependency resolution or full package verification.

Hosted CI passed stable formatting/clippy/workspace tests/docs, Rust 1.89
workspace check, macOS workspace tests, and Windows workspace check. Windows
evidence is compile-only; no live Windows SCM instance was mutated. macOS
workspace tests exercised the existing test/fake adapter coverage; no
privileged service mutation was performed.

## Invariant, recovery, compatibility, and security review

- Core remains authoritative for commit, rollback, and RecoveryRequired state;
  service code adds no artifact backup/restore or lock implementation.
- Ownership is rechecked at transitions. Invalid ownership/state fails closed.
  Stop-before-rollback is attempted under the shared remaining budget; a
  failure is retained separately and does not overwrite Core's receipt.
- RecoveryRequired suppresses automatic restart. RolledBack plus a failed
  service restoration remains explicitly representable.
- `HealthProbe`, platform adapters, service installation/refresh, and existing
  lifecycle APIs are source-compatible. The dependency is a versioned `0.1.0`
  path edge; no cycle, acquisition, Eggpack, HTTP, or TLS edge was introduced.
- Manual security review found no new elevation, shell, network, credential,
  registration-refresh, or artifact-rollback authority in this layer. No
  separate automated security scan was run. No medium-or-higher correctness
  or security finding remains open.

## Unresolved findings and downstream disposition

- Informational: the caller-owned synchronous check must obey its supplied
  budget; Rust cannot forcibly cancel arbitrary callback code safely.
- Informational: native Windows SCM runtime behavior was not exercised; the
  Windows hosted lane is a workspace compile check, complemented by fake
  backend tests.
- No downstream consumer plan becomes ready solely from M005. Eggsearch M003
  is already closed and contains no M005 lifecycle migration; no follow-up is
  currently scheduled. Gregg M004 remains blocked on corrected-path footprint
  evidence, which this milestone does not change. This dependency/status review
  is recorded in the registry and service lifecycle roadmap.

Service Lifecycle M005 is closed. No downstream blocker was newly cleared, and
there was no unforeseen issue requiring reassessment.
