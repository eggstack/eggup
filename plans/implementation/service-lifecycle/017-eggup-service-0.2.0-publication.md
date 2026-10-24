# Service Lifecycle Milestone 017 — eggup-service 0.2.0 publication

Status: closed 2026-10-24; `eggup-service 0.2.0` published from `b612608` (run `40079833945` green on all five lanes); closure record: `plans/closure/service-lifecycle/017-status.md`

Repository baseline: `b75f73b`

Source roadmap:
- plans/subsystems/service-lifecycle-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md

Primary class: infrastructure

Hard dependencies:

- Service M012, M013, M014, M015, and M016 are closed with hosted-green evidence.
- Verified Update Core M019 (`plans/implementation/verified-update-core/019-eggup-core-0.2.0-publication.md`) must publish `eggup-core 0.2.0` to crates.io before this milestone's package step runs. This milestone's package step resolves `eggup-core` against the registry rather than against the workspace path dependency, so the core publication is a precondition, not a parallel act. At plan-writing time crates.io still lists only `eggup-core 0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3`.
- The `0.2.0` version bump and the `eggup-core` requirement widening are present in the working tree at the baseline. This milestone authorizes the upload; it does not introduce them.

## 1. Objective

Publish `eggup-service 0.2.0` to crates.io as the sixth crate of the lockstep `0.1.3 → 0.2.0` train of the seven publishable crates.

**The consumer-facing reason for the republication is a defect the published `0.1.3` still carries.** The crate's own bounded executor consumed the stdin writer's verdict with `try_recv`, which *removes* the message it returns, so a verdict of `Ok(())` was consumed and discarded by a check that only looks for `Err(_)`, and the post-exit check then found an empty, disconnected channel and reported `Manager(Spawn, "manager stdin failed for {program}")` for a write that had **succeeded**. The affected production path is `crontab -`, the crate's only production stdin caller. A consumer of the published `0.1.3` therefore receives a typed `Spawn` failure where a successful install was intended. `eggup-service 0.2.0` is the first publication that carries the fix. This is the difference that turns a failure into a success; the other seven differences of the same consolidation are the recorded cost of moving the mechanic into core, and each is pinned by a test in `crates/eggup-service/tests/bounded_executor_consolidation.rs` except **D7**, which has no portable fixture. All eight are tabulated as D1–D8 in `architecture/service-lifecycle.md` § "Behaviour differences of the consolidation".

**Why `0.2.0` and not `0.1.4`.** The workspace is version-lockstep — `[workspace.package] version` is inherited by all eight crates through `version.workspace = true`. `eggup-acquisition`'s next release is source-breaking, so the seam had to go to `0.2.0`, and the lockstep workspace carried every crate forward with it. `eggup-service 0.2.0` is therefore the first `eggup-service` release that is not a `0.1.x`. `eggup-service` was published at `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3`. The service's `0.2.0` number is a consequence of the seam's break carried through the lockstep workspace, not a declaration that the service carries a comparable break of its own — though it does carry two source breaks of its own, stated in §2.

**What the published package must carry that `0.1.3` did not.** Each item below is verified against the source and against the closure record named with it.

- **M012 — one commit/rollback engine** (`plans/closure/service-lifecycle/012-status.md`). The two divergent copies of the destructive commit/rollback engine, one in `disposition.rs` and one in `lifecycle_update.rs`, are unified into `crates/eggup-service/src/commit_engine.rs`, and the stricter behaviour wins everywhere: a manager's own `completed` flag is re-verified by observation, an ownership that is no longer `Owned` is always a typed failure, an unobservable or non-owned service is never stopped before a rollback, and a desired state outside `{Running, Stopped}` produces a typed, bounded failure instead of a panic. The two surviving behavioural differences become named, separately testable policy inputs, `CommitEnginePolicy::post_transition_verification` (`PostTransitionVerification::{Verify, TrustManagerReport}`) and `CommitEnginePolicy::ownership_loss_tolerance` (`OwnershipLossTolerance::{Deny, TolerateAlreadyStopped}`), both defaulting to the stricter behaviour. The three structurally identical ownership enums become one type: `eggup_service::Ownership` re-exports `eggup_core::Ownership`, and `eggup_service::CronOwnership` is a public type alias of it. `CommandOutput` is renamed `ManagerCommandOutput`, with the old spelling kept as a **non-deprecated** type alias. Manager-output diagnostic text is bounded by one declared ceiling, `MAX_DIAGNOSTIC_BYTES` (512 bytes); the former second, smaller ceiling is gone.
- **M013 — one typed failure taxonomy and one timeout model** (`plans/closure/service-lifecycle/013-status.md`). `ServiceError::Manager` becomes a struct variant carrying `ManagerFailureKind` (`Spawn | Timeout | Permission | NotFound | Protocol | LocalFilesystem`), the bounded detail, and the platform OS code. The crate no longer classifies its **own** failures by substring-matching its own error text: the `permission_hint` re-classifier and its eleven call sites are deleted. Every adapter creates exactly one `OperationDeadline` per public operation and threads it through every manager invocation that operation makes. `CronManager` lists the crontab exactly once per `inspect` and states its ownership rule exactly once.
- **M014 — the native Windows SCM surface is real and typed** (`plans/closure/service-lifecycle/014-status.md`). The five former `unreachable!()` sites in the Windows SCM adapter become typed errors built from the crate's existing constructors, with the `1060`/`1072` mapping kept lossless. A Windows-only fixture registers a disposable service through the crate's **public** API against the genuine Service Control Manager. The public surface is pinned by a dependency-free, compile-only signature guard plus a committed snapshot.
- **M015 — one bounded-process-runner implementation** (`plans/closure/service-lifecycle/015-status.md`). The crate now runs manager commands through `eggup_core::run_bounded`, so the workspace holds one bounded-process-runner implementation instead of two. This is also the milestone that fixes the stdin-verdict defect described above.
- **M016 — the release note for that fix exists** (`plans/closure/service-lifecycle/016-status.md`). The crate's own `CHANGELOG.md` omitted the release note for the eighth behaviour difference of the M015 consolidation — the defect fix, not a cost — and the compatibility call-out did not flag the `LifecycleUpdateReceipt::engine_policy` field addition. M016 corrects both, and states the count as eight.

The published package must **not** be presented as carrying work that `0.1.3` already published. `eggup-service 0.1.3` carried the M010 failed-unit quiescence correction and everything before it. Those are the published baseline, not new content.

## 2. Publication boundary

Publish **only** `eggup-service 0.2.0` in this milestone.

The train publishes in dependency order — `eggup-core` → `eggup-archive` → `eggup-acquisition` → `eggup-curl` → `eggup-eggfetch` → `eggup-service` → `eggup-eggpack` — and all seven publish from **one shared source commit**. `eggup-service` is the sixth crate, so it waits for the first five. Do not publish, and do not upload a dry-run artifact for, any other crate under this milestone. `eggup-transport-footprint` carries `publish = false` and is never published.

**In scope.** The registry upload of `eggup-service 0.2.0`, its package qualification, its registry-only proof, its tag and release, its closure record, and the reconciliation hand-off.

**Explicitly out of scope.** Any change to `crates/eggup-service/src/**`. Any version change, any manifest edit, and any widening or narrowing of a dependency requirement — the version bump and the `eggup-core` widening are already in the working tree and this milestone authorizes the upload only. Any publication of the other six crates. Any re-publication or yank of `0.1.0`, `0.1.1`, `0.1.2`, or `0.1.3`. Any change to `plans/registry.md`, `plans/subsystems/**`, `architecture/**`, `docs/**`, `AGENTS.md`, `scripts/**`, `.github/**`, or any `CHANGELOG.md`; those are owned by other writers and are reconciled by the hand-off in W12.

**Public API boundary of `0.2.0`.**

- `ServiceError::Manager(detail)` → `ServiceError::Manager { kind, detail, os_code }`. **Source break.** A consumer must match the struct form and branch on `kind` — or on the programmatic discriminators `ServiceError::manager_kind()` and `ServiceError::manager_os_code()` — and must not branch on the message text. `ManagerFailureKind` is the six-variant set named in §1.
- `LifecycleUpdateReceipt` gained the public field `engine_policy` (`CommitEnginePolicy`). **Source break.** The struct derives only `Debug` and has **no `Default` impl**, so a struct literal is its only construction path: a consumer that constructs the receipt with a struct literal must add the field and stops compiling until it does. A consumer that only reads the receipt's fields is unaffected.
- The three ownership enums becoming one type, and `CommandOutput` surviving as an alias of `ManagerCommandOutput`, require **no** consumer migration: `eggup_service::Ownership`, `eggup_service::CronOwnership`, and `eggup_service::CommandOutput` all keep resolving under their existing spellings, `CommandOutput` is deliberately not deprecated, and no conversion is needed anywhere.
- `MAX_DIAGNOSTIC_BYTES` (512 bytes) is the one declared ceiling for manager-output diagnostic text.
- `CommitEnginePolicy` exposes the two named policy inputs, each defaulting to the stricter former behaviour.

**Consumer-visible behaviour changes.**

- **The cron managed block body must embed the absolute executable path of the spec.** A consumer whose `desired_block` does not contain `spec.executable()` now receives `ServiceError::InvalidInput` from `inspect`, `install`, and `uninstall`, where it previously received success (`install`) or a normal result. The gate runs before any listing or mutation, so a non-conforming request never reaches the crontab.
- **One deadline per operation — a fail-closed tightening.** Every adapter spends exactly one `OperationDeadline` per public operation: `install` / `uninstall` / `inspect` spend the descriptor's `transition_timeout` for the whole operation, and `start` / `stop` / `restart` spend the caller-supplied `timeout`. An operation that previously overran its budget now fails with a `Timeout` kind instead of overrunning.

**Dependency surface.** `eggup-core` (`"0.2"`), `windows-args` (`=0.2.0`), plus `windows-service` (`=0.8.1`) under `cfg(windows)` only. `windows-args` is declared unconditionally because the SCM command-line parser is not `cfg`-gated — the `windows_scm` module is compiled on every platform and calls `windows_args::Args::parse_cmd` — while only `windows-service` is Windows-only. No dependency is added, removed, or re-versioned by this milestone.

**The `eggup-core` widening is load-bearing.** `eggup-core 0.1.3` carries neither `CommandSpec::stdin_data` nor core's unconditional execution deadline, so it cannot compile `eggup-service 0.2.0`. Leaving the requirement at `^0.1.0` would let a consumer's lockfile resolve `eggup-core 0.1.3` and the service would fail to build. This is the defect class recorded as registry finding 16, and it is why the requirement must widen in the publication milestone rather than in a fix milestone: a manifest change that must reach the registry belongs to the publication, and the publication is where the widened edge becomes live.

**Package contents.** `crates/eggup-service/Cargo.toml` sets `exclude = ["tests/", "benches/"]`, so the crate's integration tests are not shipped in the published package. Nothing in the package or the release notes may claim otherwise.

**Integrity.** SHA-256 checksum evidence only. No authenticity or signature claims.

## 3. Pre-publication gates

Every gate below must hold at the publication source commit. Each gate names the verification that proves it; the full command list is §7 and the corresponding checkable criterion is §8.

- **G1 — Registry preflight.** `eggup-service 0.2.0` is absent from crates.io. `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3` are present and none is yanked. No existing version is re-published, re-uploaded, or yanked by this milestone. At plan-writing time crates.io lists exactly `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3` for `eggup-service`, none yanked. *(§7.4, W1)*
- **G2 — Publish authority.** The publishing identity holds publish authority for the `eggup-service` crate name and a `crates.io` API token is available. The four existing versions were published by `dbowm91`; the same owner performs this upload. *(W1, §7.7)*
- **G3 — Version identity.** The publication source commit carries `[workspace.package] version = "0.2.0"`, inherited by `crates/eggup-service` through `version.workspace = true`, so the packaged crate version is exactly `0.2.0` and not `0.1.3`. At the baseline `b75f73b` the bump is still an **uncommitted** working-tree change to the root `Cargo.toml` and `Cargo.lock`; the publication source commit must carry it, or the packaged crate would resolve to `0.1.3`. *(§7.1, §7.3, W2)*
- **G4 — Widened `eggup-core` requirement.** The publication source commit carries `eggup-core = { version = "0.2", path = "../eggup-core" }` in `crates/eggup-service/Cargo.toml`. At the baseline `b75f73b` that file still reads `eggup-core = { version = "0.1.0", path = "../eggup-core" }`; the widening to `"0.2"` is an **uncommitted** working-tree change. Publishing from the baseline would package a service that cannot be built against the registry's `eggup-core 0.1.3`, and the packaged manifest would carry the wrong edge. *(§7.3, W2)*
- **G5 — Public API surface.** The packaged crate exposes exactly the surface stated in §2: `ServiceError::Manager { kind, detail, os_code }` with `ManagerFailureKind` as the six-variant set named in §1 and with `manager_kind()` / `manager_os_code()` present; `LifecycleUpdateReceipt` carrying the public field `engine_policy`; `eggup_service::Ownership` re-exporting `eggup_core::Ownership`; `eggup_service::CronOwnership` as a public alias of it; `ManagerCommandOutput` present and `CommandOutput` present as a non-deprecated alias; `MAX_DIAGNOSTIC_BYTES` equal to 512; `CommitEnginePolicy` exposing the two named policy inputs. *(§7.3 — packaged manifest and file list; W4)*
- **G6 — Dependency surface.** The packaged manifest names `eggup-core` at `0.2`, `windows-args` at `=0.2.0`, and `windows-service` at `=0.8.1` under `cfg(windows)` only. No dependency is added, removed, or re-versioned, and the Windows and non-Windows graphs are unchanged in shape. *(§7.3 — packaged manifest and dependency tree; W4)*
- **G7 — No unpublished dependency edge.** The packaged manifest contains no `git`, `path`, or `patch` dependency, and names no registry dependency that is absent from the registry. `eggup-core 0.2.0` must be registry-visible before this package step can succeed. *(§7.3, §7.5, W4)*
- **G8 — Package contents.** `cargo package -p eggup-service --list --locked` shows the crate's `src/`, `README.md`, `CHANGELOG.md`, and normalized `Cargo.toml`, and shows **no** `tests/` or `benches/` entry, matching the manifest's `exclude`. *(§7.3, W4)*
- **G9 — Hosted matrix green.** The `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` lanes are green on the publication source commit. *(§7.6, W3)*
- **G10 — Changelog truthfulness.** `crates/eggup-service/CHANGELOG.md` states that `0.2.0` carries M012, M013, M014, M015, and M016; states the compatibility boundary of §2, including both source breaks and both consumer-visible behaviour changes; and states that the published `eggup-service 0.1.3` still carries the stdin-verdict defect that `0.2.0` fixes. The changelog must not present `0.1.3` content as new. At the baseline `b75f73b` the `## Unreleased` section still holds the `0.2.0` entries and the `## 0.2.0` cut is an **uncommitted** working-tree change; the publication source commit must carry it, or the published package ships no `0.2.0` release note. *(§7.3 — the packaged `CHANGELOG.md`; W2, W4)*
- **G11 — Maintainer authorization.** An explicit, recorded authorization to upload `eggup-service 0.2.0` exists before the upload runs. **The authorization has been granted for this milestone**: the lockstep `0.2.0` train decision that commissions the publication of the seven publishable crates authorizes the service half of that train, and this plan is that half's authorization. The closure record must name the authorization date and the decision that granted it. An upload without that record is a stop condition under §9. *(W5, §7.7)*
- **G12 — No unclassified source delta.** No file under `crates/eggup-service/src/**` differs from the qualified baseline. A source change requires its own corrective before publication; it must not ride along in this milestone. *(§7.1, W2)*

## 4. Registry-only proof

The fixture is a **fresh crate outside this repository**, with no `path`, `git`, or `patch` override in the manifest, in the workspace, or in `.cargo/config.toml`. It resolves only from crates.io. A fixture that resolves from the local workspace proves nothing and must be discarded and rebuilt.

- Depends on `eggup-service = "=0.2.0"` and on nothing else from this workspace.
- **The generated `Cargo.lock` must resolve `eggup-core 0.2.0` from crates.io, not `0.1.3`.** This transitive resolution is the point of the milestone: it proves the widened `eggup-core = "0.2"` requirement is live on the registry. `eggup-service 0.2.0` builds its bounded execution through `eggup_core::run_bounded` with `CommandSpec::stdin_data`, and `eggup-core 0.1.3` has no `stdin_data`, so a lockfile pinning `0.1.3` would not compile `eggup-service 0.2.0`. `cargo tree --locked` must show only registry sources for every Eggup crate.
- Exercises representative API use of the published surface:
  - a lifecycle or disposition call that produces a receipt and reads `engine_policy` off it;
  - the retyped `ServiceError::Manager` match — matching the struct form and branching on `kind`, and on `manager_os_code()`, **not** on the message text;
  - `eggup_service::Ownership`, `eggup_service::CronOwnership`, and `eggup_service::CommandOutput` still resolving under their existing spellings, with `CommandOutput` usable where `ManagerCommandOutput` is expected.
- **No live host manager is mutated.** The fixture must not install, start, stop, or uninstall a real service, and must not write to a real crontab. The real-systemd behaviour is proven by the hosted `service-systemd` lane, not by this fixture.
- The fixture's `Cargo.lock`, `cargo tree --locked` output, and test output are recorded in the closure.

The in-repository registry-only fixtures at `crates/eggup-service/tests/published-api-0.1.2/` and `crates/eggup-service/tests/published-api-0.1.3/` are separate workspaces pinned to `=0.1.2` and `=0.1.3`; the hosted `service-systemd` lane runs them, and they continue to run unchanged. A `published-api-0.2.0` fixture is a separate follow-on and is **not** claimed by this milestone unless it exists in the tree at the publication source commit.

## 5. Tag and GitHub release semantics

- `v0.1.1`, `v0.1.2`, and `v0.1.3` exist and must not be moved. The `0.1.x` line stays on crates.io; no published version is withdrawn.
- A `v0.2.0` tag may be created **only after** the registry-only proof passes, and it points at the exact publication source commit.
- Because all seven crates publish from one shared source commit, `v0.2.0` is the train's source commit. The later publications in the train append to the same release notes **without moving the tag**. The release notes must state each package's exact publication source SHA rather than implying that every `0.2.0` package originated byte-for-byte at the tagged commit.
- No tag is created for an unpublished crate, and no tag is created for `eggup-transport-footprint`, which is never published.
- Package provenance is carried by `.cargo_vcs_info.json` in the published tarball; the closure records the source commit it names.

## 6. Ordered work packages

Perform the steps in order. Do not begin a step before the previous step's evidence is recorded.

- **W1 — Registry preflight.** Read the crates.io version list for `eggup-service` and confirm G1 and G2.
- **W2 — Source freeze and version commit.** Confirm the working tree carries the `0.2.0` bump (G3) and the widened `eggup-core` requirement (G4), and that no file under `crates/eggup-service/src/**` differs from the qualified baseline (G12). The publication source commit must be a commit that carries both changes; publishing from the baseline `b75f73b` would package `0.1.3` with `eggup-core ^0.1.0`. If the source-drift check fails, stop under §9.
- **W3 — Hosted qualification.** Run the `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` lanes on the publication source commit and require green (G9). The `windows-check` lane runs ten `cargo test` steps, including the real Windows SCM fixture; the `service-systemd` lane runs the real-systemd fixtures as root.
- **W4 — Package qualification.** Run the package and dry-run steps of §7.3 and inspect the packaged manifest and file list against G5, G6, G7, and G8. `cargo package -p eggup-service --locked` can succeed only once `eggup-core 0.2.0` is on the registry; that is the dependency-order constraint, not a defect. The hosted workflow uses `cargo package -p eggup-service --locked --no-verify` for its packaged-service step, so the workflow stays green while the registry order is still pending.
- **W5 — Authorization.** Record the maintainer authorization required by G11, naming its date and the decision that granted it.
- **W6 — Upload.** Run `cargo publish -p eggup-service --locked` from the clean publication source commit. This is the only upload this milestone authorizes.
- **W7 — Registry readback.** Read the crates.io metadata for `eggup-service 0.2.0` and the published tarball: version, checksum, crate size, publisher, publication timestamp, dependency list, and the source commit in `.cargo_vcs_info.json`.
- **W8 — Registry-only fixture.** Build and run the external fixture of §4; record its lockfile, `cargo tree --locked` output, and test result.
- **W9 — Tag and release.** Create `v0.2.0` and the GitHub Release, if absent, after W7 and W8 pass, per §5.
- **W10 — Closure record.** Write `plans/closure/service-lifecycle/017-status.md` with the requirement→evidence matrix required by §10.
- **W11 — Consumer-unblock reconciliation.** Record which train publications are now unblocked by `eggup-service 0.2.0` being on the registry — `eggup-eggpack` is the train's last crate and does not depend on the service — and which remain blocked on an earlier step of the train.
- **W12 — Hand-off of owned files.** `plans/registry.md`, the service-lifecycle roadmap status table, the release table in `architecture/tooling-governance.md` §8, the publication paragraph in `architecture/overview.md`, and the root `CHANGELOG.md` are owned by other writers. Hand the intended prose over rather than editing them from this milestone. The hand-off must include the two finding dispositions: registry finding **15** (the published stdin-verdict defect) is discharged for consumers that resolve `0.2.0`, and registry finding **16** (the deferred packaged-service verification) is discharged for the strict step once `eggup-core 0.2.0` is on the registry, so the hosted workflow may restore `cargo package -p eggup-service --locked` without `--no-verify`.

## 7. Required verification

Run from the publication source commit with a clean working tree.

**7.1 — Release-prep and workspace gate.**

```sh
git status --porcelain
git rev-parse HEAD
git diff --name-only <qualified-baseline> -- crates/eggup-service/src/
```

The first two commands record the release-prep SHA and prove a clean tree; the third proves G12 — no production source file differs from the qualified baseline. Then run the workspace gate:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
```

**7.2 — MSRV floor.** The workspace `rust-version` is `1.89`. The `1.89.0` toolchain is installed in the authoring environment (`rustup toolchain list` reports `1.89.0-aarch64-apple-darwin` and `1.89.0-x86_64-apple-darwin` there), so these may be run locally as well as in the hosted `msrv` lane. The hosted `msrv` lane remains the authoritative cross-platform proof:

```sh
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
```

**7.3 — Package and dry run.**

```sh
cargo package -p eggup-service --locked
cargo package -p eggup-service --list --locked
cargo publish -p eggup-service --dry-run --locked
```

`cargo package -p eggup-service --locked` resolves `eggup-core` from crates.io, so it can succeed only after `eggup-core 0.2.0` is registry-visible. That is the dependency-order constraint of the train, not a defect of this source line. The hosted workflow's packaged-service step runs `cargo package -p eggup-service --locked --no-verify` for the same reason, so the workflow stays green while the registry order is still pending.

**7.4 — Registry readback (read-only).**

```sh
curl -fsSL https://crates.io/api/v1/crates/eggup-service/versions
curl -fsSL https://crates.io/api/v1/crates/eggup-service/0.2.0/dependencies
```

**7.5 — Registry-only fixture.**

```sh
cd <fixture-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked
```

The generated `Cargo.lock` must show `eggup-service 0.2.0` and `eggup-core 0.2.0` with registry sources and no path, git, or patch edge.

**7.6 — Hosted lanes.** The `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` jobs in `.github/workflows/ci.yml` must be green on the publication source commit, and the run id must be recorded. The `windows-check` lane runs ten `cargo test` steps; the `service-systemd` lane runs the real-systemd fixtures.

**7.7 — Publish.** Maintainer-authorized only, after §7.3 is green and G11 is recorded:

```sh
cargo publish -p eggup-service --locked
```

## 8. Acceptance criteria

Each criterion is checkable against recorded evidence.

- **AC1 — `eggup-service 0.2.0` is on crates.io**, with the recorded checksum, crate size, publisher, and publication timestamp; the recorded version list shows `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, and `0.2.0`, with no existing version yanked. *(G1, §7.4)*
- **AC2 — The published version is exactly `0.2.0`**, matching `[workspace.package] version` at the publication source commit. *(G3)*
- **AC3 — The published package carries the widened `eggup-core` edge**, and the packaged manifest names `eggup-core` at `0.2`. *(G4, G7)*
- **AC4 — The public API boundary holds**: `ServiceError::Manager` is a struct variant with `kind`, `detail`, and `os_code`; `ManagerFailureKind` carries exactly the six variants; `manager_kind()` and `manager_os_code()` are present; `LifecycleUpdateReceipt` carries the public field `engine_policy`; `Ownership` is the re-export of `eggup_core::Ownership`; `CronOwnership` is a public alias of it; `ManagerCommandOutput` is present and `CommandOutput` is present as a non-deprecated alias; `MAX_DIAGNOSTIC_BYTES` is 512; `CommitEnginePolicy` exposes the two named policy inputs. *(G5)*
- **AC5 — The dependency surface is unchanged in shape**: `eggup-core` at `0.2`, `windows-args` at `=0.2.0` unconditionally, `windows-service` at `=0.8.1` under `cfg(windows)` only; no added, removed, or re-versioned dependency. *(G6)*
- **AC6 — The packaged manifest carries no `git`, `path`, or `patch` dependency** and names no dependency absent from the registry. *(G7)*
- **AC7 — The package file list is recorded** and contains no `tests/` or `benches/` entry, matching the manifest's `exclude`. *(G8)*
- **AC8 — The hosted matrix is green on the publication source commit**: `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check`, with the run id and the per-lane results recorded. *(G9)*
- **AC9 — The local gate is green**: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo doc` (7.1), the `1.89.0` check and test (7.2), and `cargo package` / `cargo package --list` / `cargo publish --dry-run` (7.3), all with recorded output. *(G3, G4, G5, G6, G7, G8)*
- **AC10 — The changelog is truthful**: `crates/eggup-service/CHANGELOG.md` attributes M012, M013, M014, M015, and M016 to `0.2.0`, states both source breaks and both consumer-visible behaviour changes, and states that the published `eggup-service 0.1.3` still carries the stdin-verdict defect that `0.2.0` fixes. *(G10)*
- **AC11 — No source drift**: no file under `crates/eggup-service/src/**` differs from the qualified baseline, or any difference is covered by its own closed corrective. *(G12)*
- **AC12 — Publish authority was confirmed and the maintainer authorization was recorded before the upload**, naming its date and the decision that granted it, and the upload is the only publication act performed under this milestone. *(G2, G11, §2)*
- **AC13 — The registry-only fixture passes**: its lockfile resolves `eggup-service 0.2.0` and `eggup-core 0.2.0` from crates.io with no path, git, or patch source, `cargo tree --locked` shows only registry sources, and its test suite passes. *(§4, §7.5)*
- **AC14 — The published artifact carries the stdin-verdict fix**: the consolidated runner is present in the published bytes and the crate's release note for the eighth behaviour difference is present in the published `CHANGELOG.md`. *(§1, G10)*
- **AC15 — Tag and release state is correct**: `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved; `v0.2.0` points at the publication source commit and was created only after the registry-only proof passed; the release notes state each package's exact publication source SHA and do not claim that the published package ships the crate's integration tests. *(§5)*
- **AC16 — The closure record exists** at `plans/closure/service-lifecycle/017-status.md` with the requirement→evidence matrix required by §10, and the consumer-unblock reconciliation and the two finding dispositions are recorded. *(§10, W11, W12)*

## 9. Stop conditions

Stop before the upload, and raise a corrective, when any of the following holds:

- Any registry-only fixture fails, or the fixture resolves `eggup-core` from a path, git, or patch source, or its lockfile resolves `eggup-core 0.1.3` rather than `0.2.0`.
- The packaged public API differs materially from the surface stated in §2 and G5 — in particular if the packaged build lacks `ServiceError::manager_kind()` / `manager_os_code()`, still carries `ServiceError::Manager` as a tuple variant, lacks `LifecycleUpdateReceipt::engine_policy`, or lacks the `CommandOutput` alias.
- The hosted matrix is not green on the publication source commit.
- `eggup-service 0.2.0` is already present on crates.io, whether published by this milestone, by a prior run, or by another actor.
- The upload would require moving, deleting, or re-pointing an existing tag.
- `eggup-core 0.2.0` is not yet on the registry, so the package step cannot resolve the widened requirement.
- The publication source commit does not carry the `0.2.0` version bump or the widened `eggup-core` requirement, so the packaged crate would resolve to `0.1.3` with the wrong dependency edge.
- The maintainer authorization for publishing the service is withdrawn, or the authorization cannot be recorded with its date and its granting decision.
- Any file under `crates/eggup-service/src/**` differs from the qualified baseline without a closed corrective of its own.
- Publish authority for the `eggup-service` crate name cannot be confirmed.

After an accepted upload, registry state is immutable: a defect in the published `0.2.0` requires a **new patch version** (`0.2.1`), never an overwrite, a re-upload, or a yank-and-republish of `0.2.0`.

## 10. Closure evidence required

`plans/closure/service-lifecycle/017-status.md` must carry a requirement→evidence matrix with, at minimum:

- the release-prep SHA — the full 40-character form of `b75f73b` — and the publication source commit, each identified as the same or as distinct.
- the maintainer authorization: its date and the decision that granted it (G11).
- the hosted run id and the per-lane results for `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check`, including the ten `cargo test` steps of the `windows-check` lane and the real-systemd fixtures of the `service-systemd` lane.
- the local gate output for 7.1, 7.2, and 7.3, including the `1.89.0` toolchain identity actually used.
- the package inventory: the `cargo package -p eggup-service --list --locked` file list, and the packaged manifest's dependency section.
- the dependency tree for `eggup-service` on a Unix target and on a Windows target, showing `eggup-core` plus `windows-args` and `windows-service` respectively.
- the crates.io checksum, crate size, publisher, publication timestamp, published dependency list, and the source commit recorded in the published tarball's `.cargo_vcs_info.json`.
- the registry-only fixture's `Cargo.lock`, its `cargo tree --locked` output showing `eggup-core 0.2.0` resolved from crates.io, and its test result.
- the tag and release state: the unmoved `v0.1.1`, `v0.1.2`, and `v0.1.3`; the `v0.2.0` target commit; the release URL.
- the consumer-unblock reconciliation: which train publications are now unblocked by `eggup-service 0.2.0` being on the registry, and which remain blocked on an earlier step of the train.
- the two finding dispositions: registry finding **15** discharged by this publication for consumers that resolve `0.2.0`, and registry finding **16** discharged for the strict packaged-service step, so the hosted workflow may restore `cargo package -p eggup-service --locked` without `--no-verify`.
- the truthful statement that the published package does not ship `crates/eggup-service/tests/`, matching the manifest's `exclude`.

The closure record must describe the publication as performed, with dated evidence, and must not restate this plan's forward-looking language.
