# Verified Update Core Milestone 019 — eggup-core 0.2.0 publication

Status: closed 2026-10-24; `eggup-core 0.2.0` published from `b612608` (run `40079833945` green on all five lanes); closure record: `plans/closure/verified-update-core/019-status.md`

Repository baseline: `b75f73b`

Source roadmap:
- plans/subsystems/verified-update-core-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md

Primary class: infrastructure

Hard dependencies:

- Core M014, M015, M017, and M018 are closed with hosted-green evidence.
- Acquisition M016 is closed. It fixes the seam's next release at `0.2.0` and records the lockstep consequence that carries every crate in the workspace to `0.2.0`.
- The `0.2.0` version bump is present in the working tree at the baseline. This milestone authorizes the upload; it does not introduce the bump.

## 1. Objective

Publish `eggup-core 0.2.0` to crates.io as the **first** crate of the lockstep `0.1.3 → 0.2.0` train.

The published package must carry the four Core milestones closed since `0.1.3`:

- **M014 — the bounded candidate deadline.** `run_bounded` evaluated its wall-clock deadline only while the direct child was alive, so a descendant holding the output pipe open pinned the call indefinitely, and the timeout path left both reader threads unjoined. The deadline is now an unconditional maximum bound on the whole bounded execution: it is evaluated whether or not the direct child has already exited, `run_bounded` always returns, and a reader still blocked at the deadline is abandoned rather than joined. The crate has no `libc`, `nix`, or `windows-sys` dependency and `unsafe_code` is denied workspace-wide, so abandoning a reader is the only available teardown.
- **M015 — the public-failure-taxonomy correction.** The unreachable `FailureCategory::RetainedEvidence` variant and its `retained-evidence` string are removed. The remaining seven `as_str()` strings are byte-identical to the published ones, and every public fallible item now carries an `# Errors` section.
- **M017 — bounded stdin.** `CommandSpec::stdin_data` attaches a payload, `MAX_STDIN_BYTES` (1 MiB, inclusive) bounds it, an over-bound payload is rejected with `Error::InvalidInput` before any child is spawned, and the payload is written on a dedicated thread whose failure kills and reaps the child and returns a typed `Error::Io` rather than hanging.
- **M018 — deadline priority over the stdin writer's verdict.** The outcome is classified by the instant the **writer** observed the write: a failure observed before the deadline is an `Err`; one observed at or after it is the deadline outcome (`Ok`, `timed_out` set). An undelivered payload is never reported as success.

The published package must **not** be presented as carrying work that `0.1.3` already published. `eggup-core 0.1.3` already carried M010 current-executable transaction parity, M011 proof-authorized stale-lock recovery, the earlier Core audit fixes, and the M012 public-error compatibility correction. Those are the published baseline, not new content.

**Why `0.2.0` and not `0.1.4`.** The workspace is version-lockstep — `[workspace.package] version` is inherited by all eight crates through `version.workspace = true`. `eggup-acquisition`'s next release is source-breaking, so the seam had to go to `0.2.0`, and the lockstep workspace carried every crate forward with it. `eggup-core 0.2.0` is therefore the first `eggup-core` release that is not a `0.1.x`. The coupling is recorded in the seam's own changelog and in the Acquisition M016 closure record. Core's `0.2.0` number is a consequence of the seam's break, not a declaration that Core carries a comparable break of its own — with one exception, the `FailureCategory::RetainedEvidence` removal, which is a source break in its own right.

## 2. Publication boundary

Publish **only** `eggup-core 0.2.0` in this milestone.

The train publishes in dependency order — `eggup-core` → `eggup-archive` → `eggup-acquisition` → `eggup-curl` → `eggup-eggfetch` → `eggup-service` → `eggup-eggpack` — and all seven publish from **one shared source commit**. `eggup-core 0.2.0` must therefore reach the registry first; the other six wait for it. Do not publish, and do not upload a dry-run artifact for, any other crate under this milestone. `eggup-transport-footprint` carries `publish = false` and is never published.

`eggup-service 0.2.0`'s manifest declares `eggup-core = "0.2"` and `eggup-eggpack 0.2.0` pins `eggup-core = "=0.2.0"`. Both edges resolve against the registry, so `eggup-core 0.2.0` must be registry-visible before either crate is published.

**In scope.** The registry upload of `eggup-core 0.2.0`, its package qualification, its registry-only proofs, its tag and release, and its closure record.

**Explicitly out of scope.** Any change to `crates/eggup-core/src/**`. Any version change, any manifest edit, and any widening or narrowing of a dependency requirement — the version bump is already in the tree and this milestone authorizes the upload only. Any publication of the other six crates. Any change to `plans/registry.md`, `plans/subsystems/**`, `architecture/**`, `docs/**`, `AGENTS.md`, `scripts/**`, `.github/**`, or any `CHANGELOG.md`; those are owned by other writers and are reconciled by the closure step in §6.

**Public API boundary of `0.2.0`.**

- `Error` keeps **exactly the seven variants** `0.1.3` published — `InvalidInput`, `UnknownMember`, `Io`, `UpdateInProgress`, `DestinationConflict`, `VerificationFailed`, `CandidateExecution` — so an exhaustive `match` compiles unchanged. `Error::Injected` remains `#[cfg(test)]`-only and is in no packaged build.
- `FailureCategory` keeps `#[non_exhaustive]`.
- `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` are **additive**.
- The `Error::Io` operation strings are a published classification surface, observable through the public `operation` field of the `Error::Io` variant. `"spawning bounded candidate"` and `"polling bounded candidate"` are unchanged from `0.1.3`; `"writing bounded stdin"` is **new** in `0.2.0`, added by M017 for the stdin writer's failure path. All three are additive — no published operation string changes value.
- The **only** source break is the removal of `FailureCategory::RetainedEvidence` and its `"retained-evidence"` string.

**Dependency surface.** `sha2`, plus `self-replace` **Windows-only** for the deferred deletion of a replaced running image. The Unix/macOS package graph is unchanged and no new dependency is introduced.

**Integrity.** SHA-256 checksum evidence only. No authenticity or signature claims. The crate selects no release and infers no version.

**Package contents — a known open item, stated truthfully.** `crates/eggup-core/Cargo.toml` sets `exclude = ["docs/", "tests/", "benches/"]`. The crate's contract documents under `crates/eggup-core/docs/{domain,verification,transaction}.md` are therefore **not shipped in the published package**. This is an open registry finding owned by Verified Update Core M016. Neither the package, the release notes, nor any consumer-facing prose may claim that the published `eggup-core` crate ships its contracts; the contracts are reachable from the repository at the tagged commit.

## 3. Pre-publication gates

Every gate below must hold at the publication source commit. Each gate names the verification that proves it; the full command list is §7 and the corresponding checkable criterion is §8.

- **G1 — Registry preflight.** `eggup-core 0.2.0` is absent from crates.io. `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3` are present and none is yanked. No existing version is re-published, re-uploaded, or yanked by this milestone. At plan-writing time crates.io lists exactly `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3` for `eggup-core`, none yanked.
- **G2 — Publish authority.** The publishing identity holds publish authority for the `eggup-core` crate name and a `crates.io` API token is available. The four existing versions were published by `dbowm91`; the same owner performs this upload.
- **G3 — Version identity.** The publication source commit carries `[workspace.package] version = "0.2.0"`, inherited by `crates/eggup-core` through `version.workspace = true`, so the packaged crate version is exactly `0.2.0` and not `0.1.3` or `0.2.0-rc`. At the baseline `b75f73b` the bump is still an **uncommitted** working-tree change to the root `Cargo.toml` and `Cargo.lock`; the publication source commit must carry it, or the packaged crate would resolve to `0.1.3`.
- **G4 — Public API surface.** The packaged crate exposes exactly the surface stated in §2: the seven `Error` variants with `Error::Injected` behind `#[cfg(test)]`; `FailureCategory` `#[non_exhaustive]`; `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` present; `FailureCategory::RetainedEvidence` absent; the `Error::Io` operation strings `"spawning bounded candidate"` and `"polling bounded candidate"` unchanged from `0.1.3` and `"writing bounded stdin"` present; the seven published `FailureCategory::as_str()` strings byte-identical to `0.1.3`.
- **G5 — Dependency surface.** The packaged manifest names only `sha2`, plus `self-replace` under `cfg(windows)`. No dependency is added, removed, or re-versioned, and the Unix/macOS graph is unchanged.
- **G6 — No unpublished dependency edge.** The packaged manifest contains no `git`, `path`, or `patch` dependency, and names no registry dependency that is absent from the registry.
- **G7 — Package contents.** `cargo package -p eggup-core --list --locked` shows the crate's `src/`, `examples/`, `README.md`, `CHANGELOG.md`, and normalized `Cargo.toml`, and shows **no** `docs/` entry. The contract documents are not shipped, and nothing in the package or the release notes may claim otherwise.
- **G8 — Hosted matrix green.** The `stable`, `msrv`, `macos`, and `windows-check` lanes are green on the publication source commit.
- **G9 — Changelog truthfulness.** `crates/eggup-core/CHANGELOG.md` states that `0.2.0` carries M014, M015, M017, and M018, and states that M010, M011, the earlier Core audit fixes, and the M012 public-error compatibility correction were published at `0.1.3`. The changelog must not present `0.1.3` content as new.
- **G10 — No unclassified source delta.** No file under `crates/eggup-core/src/**` differs from the qualified baseline. A source change requires its own corrective before publication; it must not ride along in this milestone.
- **G11 — Maintainer authorization.** An explicit, recorded authorization to upload `eggup-core 0.2.0` exists before the upload runs.

## 4. Registry-only proof

Both fixtures are **fresh crates outside this repository**, with no `path`, `git`, or `patch` override in the manifest, in the workspace, or in `.cargo/config.toml`. They resolve only from crates.io. A fixture that resolves from the local workspace proves nothing and must be discarded and rebuilt.

### Fixture A — direct consumer of `eggup-core 0.2.0`

- Depends on `eggup-core = "=0.2.0"` and on nothing else from this workspace.
- Exercises representative use of the newly published surface: a `CommandSpec` carrying a stdin payload at exactly `MAX_STDIN_BYTES` (accepted, inclusive bound); an over-bound payload rejected with `Error::InvalidInput` before a child is spawned; a bounded run whose deadline is reached returning `Ok` with `timed_out()` set; an exhaustive `match` over the seven `Error` variants; a `FailureCategory::as_str()` value drawn from the seven published strings, with the removed `"retained-evidence"` string absent; and `InstallPlan::for_current_executable` planning against a temporary directory.
- The generated `Cargo.lock` must resolve `eggup-core 0.2.0` from crates.io with a registry source and no local path, and `cargo tree --locked` must show only registry sources.

### Fixture B — transitive consumer through `eggup-service 0.2.0`

- Depends only on `eggup-service = "=0.2.0"`.
- The generated `Cargo.lock` must resolve `eggup-core 0.2.0` from crates.io. This proves the widened `eggup-core = "0.2"` requirement is live on the registry and that a consumer's lockfile cannot resolve `eggup-core 0.1.3`: `eggup-service` builds its bounded execution through `eggup_core::run_bounded` with `CommandSpec::stdin_data`, and `eggup-core 0.1.3` has no `stdin_data`, so a lockfile pinning `0.1.3` would not compile `eggup-service 0.2.0`.
- **Ordering.** Fixture B cannot run until `eggup-service 0.2.0` is on the registry, which is the sixth step of the train and belongs to a later milestone. It is therefore a **deferred** proof: run it at the earliest point at which `eggup-service 0.2.0` is registry-visible, and record in the closure either its lockfile and test result or the outstanding proof together with the milestone that owns it. Fixture A is not deferred and is a hard gate of this milestone.

## 5. Tag and GitHub release semantics

- `v0.1.1`, `v0.1.2`, and `v0.1.3` exist and must not be moved. The `0.1.x` line stays on crates.io; no published version is withdrawn.
- A `v0.2.0` tag may be created **only after** the registry-only proofs pass, and it points at the exact publication source commit.
- Because all seven crates publish from one shared source commit, `v0.2.0` is the train's source commit. The later publications in the train append to the same release notes **without moving the tag**. The release notes must state each package's exact publication source SHA rather than implying that every `0.2.0` package originated byte-for-byte at the tagged commit.
- No tag is created for an unpublished crate, and no tag is created for `eggup-transport-footprint`, which is never published.
- Package provenance is carried by `.cargo_vcs_info.json` in the published tarball; the closure records the source commit it names.

## 6. Ordered work packages

Perform the steps in order. Do not begin a step before the previous step's evidence is recorded.

- **W1 — Registry preflight.** Read the crates.io version list for `eggup-core` and confirm G1 and G2.
- **W2 — Source freeze and version commit.** Confirm the working tree carries the `0.2.0` bump (G3) and that no file under `crates/eggup-core/src/**` differs from the qualified baseline (G10). The publication source commit must be a commit that carries the root `Cargo.toml` and `Cargo.lock` bump; publishing from the baseline `b75f73b` would package `0.1.3`. If the source drift check fails, stop under §9.
- **W3 — Hosted qualification.** Run the `stable`, `msrv`, `macos`, and `windows-check` lanes on the publication source commit and require green (G8).
- **W4 — Package qualification.** Run the package and dry-run steps of §7.3 and inspect the packaged manifest and file list against G4, G5, G6, and G7.
- **W5 — Authorization.** Obtain and record the maintainer authorization required by G11.
- **W6 — Upload.** Run `cargo publish -p eggup-core --locked` from the clean publication source commit. This is the only upload this milestone authorizes.
- **W7 — Registry readback.** Read the crates.io metadata for `eggup-core 0.2.0` and the published tarball: version, checksum, crate size, publisher, publication timestamp, dependency list, and the source commit in `.cargo_vcs_info.json`.
- **W8 — Fixture A.** Build and run the direct registry-only fixture; record its lockfile and test result.
- **W9 — Fixture B.** Build and run the transitive service fixture once `eggup-service 0.2.0` is registry-visible. If the service publication milestone has not yet run, record the outstanding proof and its owner rather than claiming it.
- **W10 — Tag and release.** Create `v0.2.0` and the GitHub Release, if absent, after W7 and W8 pass, per §5.
- **W11 — Closure record.** Write `plans/closure/verified-update-core/019-status.md` with the requirement→evidence matrix required by §10.
- **W12 — Reconciliation.** Reconcile the registry findings, the core subsystem roadmap status table, and the consumer-unblock state for the remaining train publications. Those files are owned by other writers; hand the intended prose over rather than editing them from this milestone.

## 7. Required verification

Run from the publication source commit with a clean working tree.

**7.1 — Workspace gate.**

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
```

**7.2 — MSRV floor.** The workspace `rust-version` is `1.89`. The `1.89.0` toolchain is installed in the authoring environment (`rustup toolchain list` reports `1.89.0-aarch64-apple-darwin` there), so these may be run locally as well as in the hosted `msrv` lane. The hosted `msrv` lane remains the authoritative cross-platform proof:

```sh
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
```

**7.3 — Package and dry run.**

```sh
cargo package -p eggup-core --locked
cargo package -p eggup-core --list --locked
cargo publish -p eggup-core --dry-run --locked
```

**7.4 — Registry readback (read-only).**

```sh
curl -fsSL https://crates.io/api/v1/crates/eggup-core/versions
curl -fsSL https://crates.io/api/v1/crates/eggup-core/0.2.0/dependencies
```

**7.5 — Registry-only fixtures.**

```sh
# Fixture A — direct.
cd <fixture-a-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked

# Fixture B — transitive through eggup-service.
cd <fixture-b-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked
```

**7.6 — Hosted lanes.** The `stable`, `msrv`, `macos`, and `windows-check` jobs in `.github/workflows/ci.yml` must be green on the publication source commit, and the run id must be recorded.

**7.7 — Publish.** Maintainer-authorized only, after §7.3 is green and G11 is recorded:

```sh
cargo publish -p eggup-core --locked
```

## 8. Acceptance criteria

Each criterion is checkable against recorded evidence.

- **AC1 — `eggup-core 0.2.0` is on crates.io**, with the recorded checksum, crate size, publisher, and publication timestamp; the recorded version list shows `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, and `0.2.0`, with no existing version yanked. *(G1, §7.4)*
- **AC2 — The published version is exactly `0.2.0`**, matching `[workspace.package] version` at the publication source commit. *(G3)*
- **AC3 — The public API boundary holds**: the seven `Error` variants with `Error::Injected` absent from the packaged build; `FailureCategory` `#[non_exhaustive]`; `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` present; `FailureCategory::RetainedEvidence` absent; the `Error::Io` operation strings `"spawning bounded candidate"` and `"polling bounded candidate"` unchanged from `0.1.3` and `"writing bounded stdin"` present; the seven `as_str()` strings byte-identical to `0.1.3`. *(G4)*
- **AC4 — The dependency surface is unchanged**: `sha2`, plus `self-replace` under `cfg(windows)`; no added, removed, or re-versioned dependency; the Unix/macOS graph unchanged. *(G5)*
- **AC5 — The packaged manifest carries no `git`, `path`, or `patch` dependency** and names no dependency absent from the registry. *(G6)*
- **AC6 — The package file list is recorded** and contains no `docs/` entry; the plan, the release notes, and the closure record state that the contract documents are not shipped. *(G7)*
- **AC7 — The hosted matrix is green on the publication source commit**: `stable`, `msrv`, `macos`, and `windows-check`, with the run id recorded. *(G8)*
- **AC8 — The local gate is green**: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo doc` (7.1), the `1.89.0` check and test (7.2), and `cargo package` / `cargo package --list` / `cargo publish --dry-run` (7.3), all with recorded output. *(G4, G5, G6, G7)*
- **AC9 — The changelog is truthful**: `crates/eggup-core/CHANGELOG.md` attributes M014, M015, M017, and M018 to `0.2.0` and attributes M010, M011, the earlier Core audit fixes, and the M012 correction to `0.1.3`. *(G9)*
- **AC10 — No source drift**: no file under `crates/eggup-core/src/**` differs from the qualified baseline, or any difference is covered by its own closed corrective. *(G10)*
- **AC11 — Maintainer authorization was recorded before the upload**, and the upload is the only publication act performed under this milestone. *(G11, §2)*
- **AC12 — Fixture A passes**: its lockfile resolves `eggup-core 0.2.0` from crates.io with no path, git, or patch source, and its test suite passes. *(§4, §7.5)*
- **AC13 — Fixture B passes or is explicitly outstanding**: its lockfile resolves `eggup-core 0.2.0` from crates.io through `eggup-service 0.2.0` and its test suite passes; or the closure records that `eggup-service 0.2.0` is not yet on the registry, names the milestone that owns the proof, and states the outstanding item as open rather than passed. *(§4, §7.5)*
- **AC14 — Tag and release state is correct**: `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved; `v0.2.0` points at the publication source commit and was created only after the registry-only proofs passed; the release notes state each package's exact publication source SHA and do not claim that the published package ships the crate's contract documents. *(§5)*
- **AC15 — The closure record exists** at `plans/closure/verified-update-core/019-status.md` with the requirement→evidence matrix required by §10, and the consumer-unblock reconciliation for the remaining train publications is recorded. *(§10, W12)*

## 9. Stop conditions

Stop before the upload, and raise a corrective, when any of the following holds:

- The publication source commit does not carry the `0.2.0` version bump, so the packaged crate would resolve to `0.1.3`.
- Any registry-only fixture fails, or a fixture resolves `eggup-core` from a path, git, or patch source.
- The packaged public API differs materially from the surface stated in §2 and §3 (G4) — in particular if the packaged build contains `Error::Injected`, still contains `FailureCategory::RetainedEvidence`, or lacks `CommandSpec::stdin_data` or `MAX_STDIN_BYTES`.
- The hosted matrix is not green on the publication source commit, **except for the train-order exception recorded in §9.1**.
- `eggup-core 0.2.0` is already present on crates.io, whether published by this milestone, by a prior run, or by another actor.
- The upload would require moving, deleting, or re-pointing an existing tag.
- A dependency named by the package is not yet on the registry.
- The package file list shows a `docs/` entry, or the package manifest carries a `git`, `path`, or `patch` dependency.
- Any file under `crates/eggup-core/src/**` differs from the qualified baseline without a closed corrective of its own.
- Publish authority for the `eggup-core` crate name cannot be confirmed.

### 9.1 Train-order exception — the first crate of the train

This milestone publishes the **first** crate of the 0.2.0 train, and that
position creates a structural circularity that the stop condition above cannot
resolve on its own. It is recorded here rather than left for a reader to
discover.

**The circularity.** The hosted lanes run
`cargo package -p eggup-service --locked --no-verify`. That step resolves the
packaged service crate's `eggup-core = "^0.2"` against **crates.io**, so it
cannot succeed until `eggup-core 0.2.0` is on the registry. Publishing
`eggup-core 0.2.0` requires a green matrix. A green matrix requires the
publication. The condition is unsatisfiable as literally written, for the one
crate whose publication discharges it.

**What was observed.** On the publication source commit the hosted run recorded
in the closure failed in exactly one step, in exactly four lanes, with the same
error in each: `failed to select a version for the requirement
`eggup-core = "^0.2"`` / `candidate versions found which didn't match: 0.1.3,
0.1.2, 0.1.1, ...` / `location searched: crates.io index`. The fifth lane,
which does not run the packaged-service step, was green. Every other step in
every lane — the workspace build, the test suites, clippy, rustdoc, the MSRV
check, the footprint configurations, the systemd fixtures — was green. The
failure is a **publication state**, the same class the repository already
recorded as finding 16, and it is discharged by this milestone's own upload.

**What qualifies in place of a fully green matrix for this upload only.** All
of the following must hold, and the closure record must carry the evidence:

- The local gate is green in full: `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D
  warnings`, `cargo test --workspace --all-targets --all-features --locked`,
  `cargo doc --workspace --no-deps --locked`, the `1.89.0` check and test, and
  `cargo package -p eggup-core --locked` plus
  `cargo publish -p eggup-core --dry-run --locked`.
- The hosted run on the publication source commit is green in **every** step
  except the single registry-resolved packaged-service step, and the failing
  step's error is the registry-selection error above. A failure in any other
  step, or a different error in that step, stops the upload.
- The packaged crate's own dependency surface resolves entirely against
  versions already on the registry — `sha2`, plus `self-replace` on Windows —
  so `eggup-core 0.2.0` is publishable on its own terms and does not depend
  on another crate in the train.

**What this exception does not authorise.** It does not authorise any other
upload in the train. Every later crate in the train is published only after the
full hosted matrix is green, which is possible only because this upload has
already put `eggup-core 0.2.0` on the registry. Once it has, the
packaged-service step resolves and the four lanes pass unchanged.

After an accepted upload, registry state is immutable: a defect in the published `0.2.0` requires a **new patch version** (`0.2.1`), never an overwrite, a re-upload, or a yank-and-republish of `0.2.0`.

## 10. Closure evidence required

`plans/closure/verified-update-core/019-status.md` must carry a requirement→evidence matrix with, at minimum:

- the release-prep SHA — the full 40-character form of `b75f73b` — and the publication source commit, each identified as the same or as distinct.
- the hosted run id and the per-lane results for `stable`, `msrv`, `macos`, and `windows-check`.
- the local gate output for 7.1, 7.2, and 7.3, including the `1.89.0` toolchain identity actually used.
- the package inventory: the `cargo package -p eggup-core --list --locked` file list, and the packaged manifest's dependency section.
- the dependency tree for `eggup-core` on a Unix target and on a Windows target, showing `sha2` alone and `sha2` plus `self-replace` respectively.
- the crates.io checksum, crate size, publisher, publication timestamp, published dependency list, and the source commit recorded in the published tarball's `.cargo_vcs_info.json`.
- the direct registry-only fixture's lockfile and test result.
- the transitive service fixture's lockfile and test result, or the explicit outstanding-proof entry required by AC13.
- the tag and release state: the unmoved `v0.1.1`, `v0.1.2`, and `v0.1.3`; the `v0.2.0` target commit; the release URL.
- the consumer-unblock reconciliation: which train publications are now unblocked by `eggup-core 0.2.0` being on the registry, and which remain blocked on an earlier step of the train.
- the truthful statement that the published package does not ship `crates/eggup-core/docs/`, and the reference to the open Verified Update Core M016 finding that owns it.

The closure record must describe the publication as performed, with dated evidence, and must not restate this plan's forward-looking language.
