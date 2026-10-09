# Service Lifecycle M011 — Publish the Qualified Failed-unit Quiescence Correction

Status: closed; `eggup-service 0.1.3` published 2026-10-09 and post-publication qualification passed.
Repository planning baseline: `eggstack/eggup@0bde3fefbda07019529ad7566c02e6e4ec141fd6` (M010 strict closure and final hosted qualification, 2026-10-09).
Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`.
Primary class: infrastructure / release qualification.
Hard dependency: Service M010 (`plans/implementation/service-lifecycle/010-owned-failed-systemd-service-quiescence-corrective.md`) CLOSED with the required runtime change, real-systemd proof, and no public API break.
Operational dependency: explicit maintainer publishing authorization (satisfied 2026-10-09). The M010 implementation head passed hosted run `37875012280`; the pre-publication M011 head passed the complete hosted matrix in run `37876276212`.

Registry preflight on 2026-10-09 confirmed `eggup-service 0.1.2` and absence of `0.1.3`. Authorized publication completed for `eggup-service 0.1.3` only. Cargo downloaded and checksum-verified the published package: 10 files, 396.0 KiB unpacked, 72.7 KiB compressed, SHA-256 `9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`. Its `.cargo_vcs_info.json` names source commit `feb6ae5aea4c9b61c4051957f3662ca49d845f9e` and path `crates/eggup-service`.

Pre-publication qualification is complete. Hosted run
`37876276212` is green on Stable, Rust 1.89 MSRV, macOS, Windows, and the
privileged Linux systemd lane at exact head `f65496fbd8950cb50d46620f57da336b2da232fb`.
The systemd lane confirmed registry-only `eggup-service =0.1.2` classifies the
exact failed registration as Owned/Unknown but returns
`Manager("transition deadline exhausted before manager command")` despite
zero PIDs, empty ControlGroup, and no Job; current workspace 0.1.3 passed its
positive real-systemd failed-stop fixture in the same run. Three external
inspection controls pass for Owned, Foreign, and malformed registrations. The
local workspace gate, registry-only fixture, `cargo package --list`, package
verification, and `cargo publish --dry-run` passed before publication. The
package checksum and size are recorded above. Publication was explicitly
authorized and completed. A fresh registry-only
`=0.1.3` fixture now covers positive quiescence and Foreign/malformed controls.
Downstream M004 remains active with independent rootful qualification and
production trust-root blockers; this service publication does not close M004
or unblock M005.

## 1. Objective

Make the M010 corrected `eggup-service` state/quiescence contract consumable from crates.io by `wg-basic`, with exact reproducible packaged source, registry-only compatibility evidence and no accidental change to other Eggup crates. The current repository uses workspace package version `0.1.3`; the previously published `eggup-service 0.1.2` is immutable. Publish the next valid, unused service patch version after explicit registry inspection; `0.1.3` is the expected candidate, not permission to assume availability or overwrite anything.

## 2. Detection and cross-repository blocker

`wg-basic` pins `eggup-service = "=0.1.2"`, which lacks a safe completion path for its failed-candidate rollback. M010's real-systemd negative control proved the published stop reports incomplete even when the exact-owned failed unit has no remaining tasks; M010 therefore requires an Eggup runtime change. It is not acceptable to adopt unpublished Eggup Git/path code as production or to permit `wg-basic` release CI to drift to a dependency different from the frozen installed binary.

The M010 applicability decision is complete: this release plan is required to deliver the qualified behavior to registry-only consumers.

## 3. Invariants

- This plan never silently changes M010's reviewed service API or behaviour. Any regression first returns to M010.
- `eggup-service` remains compatible with `eggup-core ^0.1.0` and Rust 1.89; run an external, registry-only fixture to prove exact resolution of the published core 0.1.3 graph.
- All other crates preserve their existing published identity/source/version. Workspace version `0.1.3` is not authority to republish already-published core/archive/acquisition/eggpack crates.
- Do not move old `v0.1.2` or `v0.1.3` tags. If service source ships from a later commit, record package-specific source provenance, SHA, checksum and registry identity precisely.
- No automatic publication. An explicit maintainer-authorized crates.io publish operation is a separate irreversible boundary.
- No breaking enum variant or public signature change masquerading as `0.1.3`. Preserve exhaustive-match consumer compatibility and the no-unsafe rule.

## 4. Ordered work packages

1. Retrieve Service M010 strict closure evidence and exact implementation commit, full native/negative test matrix, API diff and affected platform list.
2. Inspect crates.io for `eggup-service` current versions, publish ownership and exact `0.1.3` absence. If `0.1.3` is already occupied, stop/replan next version with compatibility evaluation; do not republish or silently bump workspace/other crates.
3. Make the source changelog clearly identify the behavior correction and M010 runtime source revision. The exact release-prep commit must also be captured by packaged `.cargo_vcs_info.json`. Ensure package manifest exposes the expected version and dependency graph without Git/path replacement in packaged output.
4. Run `cargo package -p eggup-service --list --locked`, `cargo package -p eggup-service --locked`, `cargo publish -p eggup-service --dry-run --locked`, and the external registry-only fixture. Before publication, fixture must still resolve 0.1.2 and the privileged systemd negative control must demonstrate the failed-case limitation. Build/verify the packaged crate on Linux, macOS, Windows, and Rust 1.89 CI lanes.
5. Require hosted Stable + Rust 1.89 MSRV + macOS + Windows CI green at the exact candidate SHA. Re-run service real-systemd matrix or obtain an equally recent immutable qualified run on the same source.
6. Obtain explicit maintainer authorization; publish **only eggup-service** to crates.io. Completed 2026-10-09.
7. Verify registry checksum/source metadata, direct external `eggup-service = "=0.1.3"` dependency and a positive proof of owned failed-unit quiescence. Verify other published crates and existing tags untouched.
8. Write exact version+checksum+API migration handoff to `dbowm91/wg-basic` M004 C001a. Reconcile current Eggup roadmap, registry, changelog and release notes without rewriting historical service M009 closure.

## 5. Failure, restart, and contention

Publishing is immutable. On ambiguous registry result after an attempted publish, query the registry and package checksum before retrying; never assume timeout means no artifact exists. No live host services, install receipts or artifact bytes are mutated by release qualification. If core ABI compatibility, platform behaviour or ownership proof changes during package reconstruction, block and return to M010.

## 6. Public API and platform compatibility

No public enum variant changes. No loss of `ServiceManager` or `SystemdManager` methods, no new mandatory features/third-party dependencies. Cross-platform source compatibility for Unix non-systemd/macOS/Windows must be verified with the exact packaged crate, not merely workspace tests.

## 7. Tests and broad verification

```text
git status --porcelain
git rev-parse HEAD
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-service --list --locked
cargo package -p eggup-service --locked
cargo publish -p eggup-service --dry-run --locked
cargo fmt --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml -- --check
cargo check --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml --locked
cargo test --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml --locked -- --nocapture
sudo env EGGUP_SYSTEMD_INTEGRATION=1 cargo test --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml --locked -- --nocapture
sudo env EGGUP_SYSTEMD_INTEGRATION=1 cargo test -p eggup-service --test systemd_failed_service --locked -- --nocapture
cargo tree --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml --locked
cargo fmt --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml -- --check
cargo check --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml --locked
cargo test --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml --locked -- --nocapture
sudo env EGGUP_SYSTEMD_INTEGRATION=1 cargo test --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml --locked -- --nocapture
git diff --check
```

The compatibility fixture at `crates/eggup-service/tests/published-api-0.1.2/`
resolves `eggup-service = "=0.1.2"` and `eggup-core` purely from the registry,
records registry sources in its lockfile, and matches the non-exhaustive public
enum with a wildcard. Its privileged Linux integration test proves the old
failed-unit stop limitation. The packaged candidate itself is built by
`cargo package` in Stable/Linux, MSRV 1.89, macOS, and Windows CI jobs. After
publication, add a fresh external fixture at the exact version and test
positive failed-unit quiescence plus Foreign/malformed controls. Log exact
checksum, package byte size, and source revision. The external fixture and
hosted systemd run are now required for closure.

## 8. Acceptance criteria

Exact reviewed service correction is published under an unused immutable version; checksum/provenance and registry-only test pass; no other crate/tag changes; supported platform/MSRV tests pass; downstream adoption version is explicit; release/security policy is unchanged.

## 9. Stop conditions

Version occupied, publish authority absent, registry-only fixture fails, source/package drift, significant native failure, public API break, workspace version train conflict, or an attempt to move `v0.1.3`. Classify publication as `blocked` when authorization is absent; do not mark the release plan closed before registry verification and downstream handoff.

## 10. Closure evidence and handoff

After publication only, write `plans/closure/service-lifecycle/011-status.md` with M010 closure SHA, package/commit/version/checksum, registry preflight, exact gates, downstream fixture, operator authorization result, supported-platform matrix, tag integrity, known limitations, and `wg-basic` adoption pointer. This is a release step, not consent to auto-publish wg-basic itself.
