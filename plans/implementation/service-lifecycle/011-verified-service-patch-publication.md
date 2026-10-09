# Service Lifecycle M011 — Publish the Qualified Failed-unit Quiescence Correction

Status: ready for release qualification; crates.io publication blocked on explicit maintainer authorization.
Repository planning baseline: `eggstack/eggup@0bde3fefbda07019529ad7566c02e6e4ec141fd6` (M010 strict closure and final hosted qualification, 2026-10-09).
Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`.
Primary class: infrastructure / release qualification.
Hard dependency: Service M010 (`plans/implementation/service-lifecycle/010-owned-failed-systemd-service-quiescence-corrective.md`) CLOSED with the required runtime change, real-systemd proof, and no public API break.
Operational dependency: explicit maintainer publishing authorization. The fresh hosted Stable/MSRV/macOS/Windows and systemd run `37875012280` is green on the M010 implementation head; M011 requires a new exact-source run after release preparation.

Registry preflight on 2026-10-09: `cargo info eggup-service@0.1.2 --registry crates-io` resolves the published baseline; `cargo info eggup-service@0.1.3 --registry crates-io` reports no such version. A preliminary `eggup-service 0.1.3` package/dry-run succeeded at `753bc7c`; M011 must rebuild and record final release-prep bytes and checksum after the exact changelog and external negative-control fixture are committed.

Pre-publication progress: M010 strict closure, registry version preflight, and
the 0.1.2 public-API compile fixture are complete. A systemd integration test
against registry-only `eggup-service =0.1.2` now serves as the required
behavioral negative control: it creates an exact-owned failed unit with no
remaining control group and asserts the published library cannot report stop
completion. The final release-prep source, hosted run, package bytes, and
checksum remain to be recorded. No non-dry-run publication is authorized.

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
6. Obtain explicit maintainer authorization; publish **only eggup-service** to crates.io. If authentication/publish permission is unavailable, stop with a blocked publication record; do not change the status to closed.
7. Verify registry checksum/source metadata, direct external `eggup-service = "=0.1.3"` (or newly authorized version) dependency and a positive proof of owned failed-unit quiescence. Verify other published crates and existing tags untouched.
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
sudo env EGGUP_SYSTEMD_INTEGRATION=1 cargo test --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml --test systemd_failed_stop_negative_control --locked -- --nocapture
git diff --check
```

The compatibility fixture at `crates/eggup-service/tests/published-api-0.1.2/`
resolves `eggup-service = "=0.1.2"` and `eggup-core` purely from the registry,
records registry sources in its lockfile, and matches the non-exhaustive public
enum with a wildcard. Its privileged Linux integration test proves the old
failed-unit stop limitation. The packaged candidate itself is built by
`cargo package` in Stable/Linux, MSRV 1.89, macOS, and Windows CI jobs. After
publication, add a fresh external fixture at the authorized exact version and
test positive failed-unit quiescence plus Foreign/malformed controls. Log exact
checksum, package byte size, and source revision before asking for publication
authorization.

## 8. Acceptance criteria

Exact reviewed service correction is published under an unused immutable version; checksum/provenance and registry-only test pass; no other crate/tag changes; supported platform/MSRV tests pass; downstream adoption version is explicit; release/security policy is unchanged.

## 9. Stop conditions

Version occupied, publish authority absent, registry-only fixture fails, source/package drift, significant native failure, public API break, workspace version train conflict, or an attempt to move `v0.1.3`. Classify publication as `blocked` when authorization is absent; do not mark the release plan closed before registry verification and downstream handoff.

## 10. Closure evidence and handoff

After publication only, write `plans/closure/service-lifecycle/011-status.md` with M010 closure SHA, package/commit/version/checksum, registry preflight, exact gates, downstream fixture, operator authorization result, supported-platform matrix, tag integrity, known limitations, and `wg-basic` adoption pointer. This is a release step, not consent to auto-publish wg-basic itself.
