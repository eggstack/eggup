# Verified Update Core Milestone 001 — Repository and Workspace Foundation

Status: ready for handoff

Repository baseline: `5b6cf13ed7e6e2bd40951216f29ea68d7333aadf`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md#M001--repositoryworkspace-foundation-and-contract-harness`

Long-term requirements:

- `plans/000-long-term-specification.md#4-crate-and-layer-model`
- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`
- `plans/000-long-term-specification.md#24-testing-requirements`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: infrastructure / invariant

## 1. Objective

Turn the planning-only repository into a minimal Rust workspace that makes Eggup's core ownership boundary executable and testable without implementing live update behavior.

The milestone establishes the package, lint, MSRV, documentation, test-support, and CI baseline that every later security-sensitive milestone will rely on.

## 2. Why this milestone is ready

There are no code dependencies because the repository is new.

The durable architecture decisions required to choose the initial workspace boundary are accepted in ADR-0001 through ADR-0003.

This milestone intentionally stops before any downloader, checksum parser, candidate execution, self replacement, service manager, or installer generation exists.

## 3. Current implementation evidence

At the baseline:

- the repository contains planning documents only;
- there is no `Cargo.toml`, source tree, CI, README, license file, changelog, or verification script;
- no public Rust API exists;
- no dependency graph exists;
- no release/package contract exists.

The plan must therefore establish foundations without pretending any update capability is complete.

## 4. Invariants that must not regress

- `eggup-core` has no HTTP/TLS dependency.
- `eggup-core` has no systemd/launchd/Windows-service dependency.
- No Eggstack consumer crate/repository is a dependency.
- Rust MSRV is explicitly 1.89.
- First-party core code forbids unsafe code unless a future accepted ADR creates a narrowly isolated exception.
- No network access is required by tests.
- No mutation of real installation/service paths occurs in tests.
- Public API scaffolding does not hard-code GitHub, crates.io, Cargo fallback, or a product name.

## 5. Scope

### In scope

- root Rust workspace;
- `crates/eggup-core` package;
- workspace package metadata and shared lints;
- baseline dependency policy;
- `README.md`, `CHANGELOG.md`, and repository-level architecture overview;
- license file consistent with the established Eggstack repository policy after verifying that policy;
- minimal `src/lib.rs`;
- public crate documentation describing non-capabilities and ownership;
- typed top-level `EggupError`/result shell only if useful without inventing future variants prematurely;
- internal `test_support` facilities under `cfg(test)` or integration-test helpers;
- temporary installation-root fixture helper;
- deterministic failure-point enum/hook scaffold that later transaction tests can extend;
- ordinary CI/check workflow;
- local verification script or documented command sequence;
- deny/audit policy file only if it is already standard across Eggstack and can be added without speculative exceptions.

### Explicitly out of scope

- fetching;
- checksums;
- signatures;
- staging production API;
- executable validation;
- update locks;
- replacement;
- rollback;
- uninstall;
- services;
- installers;
- release publishing;
- consumer integration.

## 6. Required production changes

### Workspace

Create a virtual workspace with resolver 2 and one initial member:

```text
crates/eggup-core
```

Use `[workspace.package]` for shared edition, rust-version, license/repository metadata where practical.

Do not create empty placeholder crates for Eggfetch, service, dist, or a facade merely to populate the future architecture.

### Core package

The package should compile as an independently consumable library.

Suggested initial crate root ownership:

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// crate-level docs describing policy-neutral transaction substrate
```

Avoid public types that are not needed until M002. A small stable namespace is preferable to speculative APIs.

### Dependency policy

M001 should need little or no runtime dependency.

If `thiserror` is introduced for a baseline error shell, justify it. Do not add Tokio, Eggfetch, semver, serde, sha2, tempfile, self-replace, windows-service, service-manager, archive libraries, or async runtimes merely because later milestones may use them.

Test-only temp helpers may use a dev dependency if standard-library temporary-path handling would be unnecessarily unsafe or flaky. Record the choice.

### Repository documentation

Create a concise root README explaining:

- Eggup's intended role;
- current status as pre-functional/foundation-stage;
- mechanism-versus-policy boundary;
- planned crate layers;
- no claim of production updater capability yet.

Create an architecture overview under a stable location such as `architecture/overview.md` linking the canonical planning documents rather than duplicating them.

### Verification tooling

A small `scripts/check-local.sh` is preferred if consistent with Eggstack practice. It should remain orchestration, not contain project policy.

Initial commands should include:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
```

If there is initially no lockfile until the first build, generate and commit it for reproducibility.

### CI

Add bounded ordinary correctness CI. Do not add release automation.

At minimum, verify Linux current toolchain and MSRV. If cheap, include `cargo check` for macOS/Windows through hosted runners, but do not let platform matrix expansion consume this milestone.

## 7. Ordered work packages

### Work package A — Workspace and package baseline

Intent: establish a minimal valid Cargo workspace.

Required changes:

- root `Cargo.toml`;
- `crates/eggup-core/Cargo.toml`;
- `crates/eggup-core/src/lib.rs`;
- Rust 1.89 declaration;
- workspace lints/package metadata.

Acceptance evidence:

- `cargo metadata --no-deps`;
- MSRV `cargo check`;
- no unexpected dependencies.

### Work package B — Repository contract documentation

Intent: make the non-capability boundary explicit before implementation starts.

Required changes:

- root README;
- architecture overview;
- changelog;
- license after confirming org standard;
- links to canonical plans/ADRs.

Acceptance evidence:

- docs accurately say live updating is not yet implemented;
- no consumer-specific release policy is described as core behavior.

### Work package C — Deterministic test harness foundation

Intent: create safe primitives later milestones can extend.

Required changes:

- temp installation-root fixture;
- helper for writing fixture files and reading exact bytes;
- deterministic failure-point scaffold with no production activation;
- tests proving fixture isolation/cleanup.

Acceptance evidence:

- tests never touch home, PATH-installed binaries, system service paths, or network.

### Work package D — Local/hosted verification baseline

Intent: create repeatable closure evidence.

Required changes:

- local verification entry point;
- ordinary CI;
- lockfile;
- static guard/test that core manifest does not acquire prohibited dependency classes, using the smallest maintainable mechanism.

Acceptance evidence:

- clean local verification;
- MSRV evidence;
- CI config has no publishing/tagging/release side effect.

## 8. Failure, cancellation, restart, and contention semantics

No live updater operation exists in this milestone.

The only relevant failure semantics are:

- test fixtures must clean their owned temporary state after test completion where possible;
- failure injection is deterministic and test-only;
- verification scripts propagate failures and do not mask nonzero commands.

Do not invent asynchronous cancellation or locking behavior before the transaction milestone.

## 9. Compatibility and migration

No existing Eggup code exists.

Crate/package names established here become the first compatibility surface. Keep it deliberately small.

Do not publish to crates.io in M001.

## 10. Required tests

### Focused unit tests

- fixture root is unique/private enough for tests;
- fixture writes/reads exact bytes;
- failure-point scaffold defaults to no failure.

### Integration tests

- crate can be consumed by a tiny test/example without application dependencies if useful.

### Static guards

- manifest/dependency inspection proving no HTTP/TLS/service-manager crate is directly owned by eggup-core;
- grep/architecture guard is acceptable initially if simpler than custom tooling.

### Negative tests

- test helper rejects paths escaping its owned root if it accepts relative member paths.

## 11. Required verification commands

At minimum:

```bash
rustc +1.89.0 --version
cargo +1.89.0 check --workspace --all-targets --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree --workspace --locked
```

Run the repository verification script if added.

Do not claim platform-native evidence that was not run.

## 12. Documentation updates

- root README;
- architecture overview;
- changelog entry for initial foundation;
- planning registry and verified-update-core roadmap status after closure.

## 13. Acceptance criteria

- repository is a valid Rust 1.89 workspace;
- eggup-core independently builds/tests/docs;
- no forbidden dependency class is present;
- test harness cannot mutate real installations;
- ordinary CI contains no release side effects;
- documentation states the correct ownership boundary;
- no update capability is falsely claimed.

## 14. Stop conditions

Stop and report if:

- the desired crate name is already occupied and publication naming must change;
- current Eggstack licensing policy cannot be determined safely;
- Rust 1.89 cannot build a required baseline dependency;
- implementing the foundation would require selecting an HTTP, service, archive, or replacement dependency that belongs to a later milestone;
- a public API decision materially contradicts the accepted ADRs.

## 15. Closure evidence required

The closure record must contain:

- exact implementation commit(s);
- resulting workspace/package tree;
- dependency list for eggup-core;
- MSRV command/result;
- full local verification results;
- CI status if available;
- static guard evidence;
- confirmation that no live updater/service/install feature exists yet;
- any naming/licensing issue discovered.

## 16. Handoff notes

Keep M001 small. The most important result is a clean boundary for M002, not a head start on later functionality.

Do not copy `gregg-update`, `eggsearch/src/update.rs`, or another consumer implementation into the new crate during this milestone.
