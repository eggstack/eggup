# Archive Extraction Milestone 006 — eggup-archive 0.2.0 publication

Status: closed 2026-10-24; `eggup-archive 0.2.0` published from `b612608` (run `40079833945` green on all five lanes); closure record: `plans/closure/archive-extraction/006-status.md`

Repository baseline: `b75f73b`

Source roadmap:
- plans/subsystems/archive-extraction-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md
- plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md

Primary class: infrastructure

Hard dependencies:

- Archive Extraction M005 is closed and hosted-qualified
  (`plans/closure/archive-extraction/005-status.md`, hosted run `40008039843`), so the
  source that `0.2.0` publishes is already qualified.
- The `0.2.0` version bump is present in the working tree at the baseline. This
  milestone authorizes the upload; it does not introduce the bump.
- The train's first step, `eggup-core 0.2.0`, must be registry-visible before this
  upload, because the train publishes in one order from one source commit.

## 1. Objective

Publish `eggup-archive 0.2.0` to crates.io as the **second** crate of the lockstep
`0.1.3 → 0.2.0` train, immediately after `eggup-core 0.2.0`.

The published package must carry the Archive Extraction M005 work that the published
`0.1.3` does not:

- **The cleanup contract is truthful and observable.** The four `cleanup()` entry
  points (`ExtractedArchive::cleanup`, `PersistedExtraction::cleanup`,
  `BoundExtraction::cleanup`, and `DeferredCleanup::cleanup`) documented that they
  "remove" the extraction directory, but the shared implementation emptied the owned
  tree through the retained handle and then unconditionally returned
  `Err(ExtractionErrorKind::CleanupFailed)`, so the `Ok` path was unreachable in a
  published crate. All four now share one truthful contract: `Ok(())` means the owned
  root object held no entries at the final handle-relative listing; `Err(..)` with
  `kind() == CleanupFailed` means the owned root could **not** be fully emptied — the
  residue may be partially emptied, its emptiness is not guaranteed, and
  `residue_path()` names the recorded root path. The crate still never removes the
  root directory itself and never traverses the root pathname; the residual empty
  directory is the caller's to remove.
- **One member-write path.** The tar.gz and zip handlers each carried a full copy of
  the member-write body. The body now exists exactly once, in one shared helper
  parameterized by the per-format entry-kind predicate, and each handler keeps its own
  parsing. The refactor is behaviour-preserving: same bytes written, same order of
  operations, same error kinds, same evidence, same handle-authority calls.

The published package must **not** present as new work what `0.1.3` already
published. `eggup-archive 0.1.3` carried the Windows reserved-device validation, the
truthful-residue correction for the extraction failure path, and the hostile-entry
regression coverage. Those are the published baseline, not new content.

**Why `0.2.0` and not `0.1.4`.** The workspace is version-lockstep —
`[workspace.package] version` is inherited by all eight crates through
`version.workspace = true`. The acquisition seam's next release is source-breaking
(`plans/closure/acquisition-transport/016-status.md`: `eggup-acquisition 0.2.0` retypes
`AcquisitionError::Io` and drops `Clone`/`PartialEq`/`Eq` from `AcquisitionError`), so
the seam had to go to `0.2.0`, and the lockstep workspace carried every crate forward
with it. `eggup-archive 0.2.0` is therefore the first `eggup-archive` release that is
not a `0.1.x`. The number is a consequence of the seam's break, not a declaration that
the archive carries a comparable break of its own; the archive's one caller-visible
change is the cleanup result handling described in §2.

**Registry state at plan-writing time.** crates.io holds `eggup-archive 0.1.2`
(published 2026-09-28) and `0.1.3` (published 2026-10-06), both published by
`dbowm91`, neither yanked, and no `0.2.0`. This milestone is written before the
upload, so the publication described here is work to be done.

## 2. Publication boundary

Publish **only** `eggup-archive 0.2.0` in this milestone.

The train publishes in dependency order — `eggup-core` → `eggup-archive` →
`eggup-acquisition` → `eggup-curl` → `eggup-eggfetch` → `eggup-service` →
`eggup-eggpack` — and all seven publish from **one shared source commit**.
`eggup-core 0.2.0` must therefore reach the registry first; the other five wait for
this upload. `eggup-transport-footprint` carries `publish = false` and is never
published.

**This crate's position in that order is specific, and the distinction must not be
blurred.** `crates/eggup-archive/Cargo.toml:23-24` declares `eggup-core` as a
**path-only dev-dependency** (`[dev-dependencies]`, `eggup-core = { path =
"../eggup-core" }`), and dev-dependencies are stripped when a crate is packaged. The
published `eggup-archive` therefore has **no** `eggup-core` dependency at all and can
be packaged against the registry without `eggup-core 0.2.0` being present. The archive
still publishes second, because the train is published in one order from one source
commit. No artifact, release note, or closure sentence may imply a registry
`eggup-core` edge that does not exist. The published `0.1.3` registry dependency list
is the direct evidence: five normal dependencies — `flate2`, `fs_at`, `sha2`, `tar`,
`zip` — and no `eggup-core` entry.

**In scope.** The registry upload of `eggup-archive 0.2.0`, its package qualification,
its registry-only proof, its tag and release contribution, and its closure record.

**Explicitly out of scope.** Any change to `crates/eggup-archive/src/**`. Any version
change, any manifest edit, and any widening or narrowing of a dependency requirement —
the version bump is already in the tree and this milestone authorizes the upload only.
Any publication of the other six crates. Any change to `plans/registry.md`,
`plans/subsystems/**`, `architecture/**`, `docs/**`, `AGENTS.md`, `scripts/**`,
`.github/**`, or any `CHANGELOG.md`; those are owned by other writers and are
reconciled by the closure step in §6.

**Public API boundary of `0.2.0`.**

- No public signature, field, type, or constructor is added, removed, or retyped.
  `Result<(), ExtractionError>` is retained on all four `cleanup()` entry points.
  `ExtractionErrorKind` keeps its fourteen variants. `ArchiveLimits` defaults, plan and
  limit validation, path validation, the Windows reserved-device set, the finite
  member/aggregate/entry/path bounds, and the handle-relative write and handle-bound
  cleanup mechanics are unchanged.
- No new dependency. No new extraction feature. No change to the crate's ownership
  boundary: no transport, no authenticity policy, no live-installation mutation.
- **One caller-visible behaviour change: the `cleanup()` success path is now
  reachable.** A caller that wrote `cleanup().unwrap_err()` will now panic on the
  success path and must match on the result instead. A caller that wrote `cleanup()?`
  and aborted the flow on error now continues on success — the intended fix, but a
  control-flow change to review. A caller that needs the residual path must capture it
  from `root()` before the consuming call, because the call consumes the value. Apart
  from that result handling, no consumer migration is required — a claim that is
  bounded by the result handling itself, and is not a statement that the release is
  behaviour-neutral for existing callers.

**Dependency surface at publication.** `flate2`, `fs_at`, `sha2`, `tar`, `zip`
(`crates/eggup-archive/Cargo.toml:16-21`). The `eggup-core` edge is a path-only
dev-dependency and is stripped at packaging, so the published crate carries **no**
`eggup-core` dependency.

**Integrity.** SHA-256 checksum evidence only. No authenticity or signature claims.
The crate extracts from an already-verified archive; it never claims an archive is
authentic, and it selects no release.

**Package contents.** `crates/eggup-archive/Cargo.toml:14` sets
`exclude = ["tests/", "benches/"]`. The package therefore ships the crate's `src/`,
`README.md`, `CHANGELOG.md`, and the normalized `Cargo.toml`, and does **not** ship
the M005 integration fixtures under `crates/eggup-archive/tests/`. Nothing in the
package, the release notes, or the closure record may claim otherwise.

## 3. Pre-publication gates

Every gate below must hold at the publication source commit. Each gate names the
verification that proves it; the full command list is §7 and the corresponding
checkable criterion is §8.

- **G1 — Registry preflight.** `eggup-archive 0.2.0` is absent from crates.io.
  `0.1.2` and `0.1.3` are present and neither is yanked. No existing version is
  re-published, re-uploaded, or yanked by this milestone. At plan-writing time crates.io
  lists exactly `0.1.2` and `0.1.3` for `eggup-archive`, both published by `dbowm91`,
  neither yanked, and no `0.2.0`.
- **G2 — Publish authority.** The publishing identity holds publish authority for the
  `eggup-archive` crate name and a crates.io API token is available. The two existing
  versions were published by `dbowm91`; the same owner performs this upload.
- **G3 — Version identity.** The publication source commit carries
  `[workspace.package] version = "0.2.0"`, inherited by `crates/eggup-archive` through
  `version.workspace = true`, so the packaged crate version is exactly `0.2.0` and not
  `0.1.3` or a pre-release. At the baseline `b75f73b` the bump is still an
  **uncommitted** working-tree change to the root `Cargo.toml` and `Cargo.lock`; the
  publication source commit must carry it, or the packaged crate would resolve to
  `0.1.3`.
- **G4 — Public API surface.** The packaged crate exposes exactly the surface stated in
  §2: `ArchiveFormat`, `ArchiveLimits` (including `strict_default()`), `ArchiveMember`,
  `ArchivePlan`, `ExtractionErrorKind` with its fourteen variants, `ExtractionError`
  with `kind()` and `residue_path()`, `ExtractedMember`, `ExtractedArchive`,
  `PersistedExtraction`, `BoundMember`, `BoundExtraction`, `DeferredCleanup`, and the
  free function `extract`. The four `cleanup()` entry points keep the signature
  `fn cleanup(self) -> Result<(), ExtractionError>`; every `root()` accessor is
  present; no public item is added, removed, or retyped.
- **G5 — Dependency surface.** The packaged manifest names only `flate2`, `fs_at`,
  `sha2`, `tar`, and `zip`. No dependency is added, removed, or re-versioned.
- **G6 — No unpublished dependency edge.** The packaged manifest contains no `git`,
  `path`, or `patch` dependency, names no registry dependency absent from the registry,
  and carries **no** `eggup-core` entry at all — the path-only dev-dependency must be
  stripped.
- **G7 — Package contents.** `cargo package -p eggup-archive --list --locked` shows
  `src/`, `README.md`, `CHANGELOG.md`, and the normalized `Cargo.toml`, and shows **no**
  `tests/` entry.
- **G8 — Hosted matrix green.** The `stable`, `msrv`, `macos`, and `windows-check`
  lanes are green on the publication source commit. The Windows lane must **execute**
  the archive tests rather than compile them only: the crate's `#[cfg(windows)]`
  fixtures and the reserved-device validation are Windows-only evidence, which the M005
  closure records as its own open low-severity finding.
- **G9 — Changelog truthfulness.** `crates/eggup-archive/CHANGELOG.md` attributes the
  cleanup contract correction and the shared member-write helper to `0.2.0`, states the
  cleanup result-handling migration, and does not present `0.1.3` content as new. Its
  `0.2.0` publication statement becomes true only when the upload is accepted; the
  closure records the outcome either way.
- **G10 — No unclassified source delta.** No file under `crates/eggup-archive/src/**`
  differs from the qualified baseline. A source change requires its own corrective
  before publication and must not ride along in this milestone.
- **G11 — Train order.** `eggup-core 0.2.0` is registry-visible before the archive
  upload. This is a train-ordering gate, not a dependency gate: the published archive
  has no `eggup-core` edge (§2).
- **G12 — Maintainer authorization.** An explicit, recorded authorization to upload
  `eggup-archive 0.2.0` exists before the upload runs.

## 4. Registry-only proof

The fixture is a **fresh crate outside this repository**, with no `path`, `git`, or
`patch` override in the manifest, in the workspace, or in `.cargo/config.toml`. It
resolves only from crates.io. A fixture that resolves from the local workspace proves
nothing and must be discarded and rebuilt.

### Fixture — direct consumer of `eggup-archive 0.2.0`

- Depends on `eggup-archive = "=0.2.0"` and on no other `eggup-*` crate. It may
  depend on registry crates of its own to author archives and digests (`tar`, `zip`,
  `flate2`, `sha2`).
- Exercises representative use of the published surface:
  - **A bounded allowlisted extraction** in each supported format — a small tar.gz and
    a small zip, each holding one declared regular member. The fixture builds an
    `ArchivePlan` with `ArchiveLimits::strict_default()` and an `ArchiveMember` that
    declares the exact expected size and SHA-256, calls `extract(&plan, &parent)`,
    and asserts the member evidence (`bytes_written`, `sha256`) and the extracted bytes
    match the declared digest. The fixture uses only the public API and none of the
    crate's private test seams.
  - **A `cleanup()` call whose result is matched, not unwrapped.** The fixture captures
    the residual path from `root()` **before** the consuming call, then matches the
    result: on `Ok(())` it asserts the root directory still exists, holds zero
    entries, and that the **caller** removes it; on `Err(error)` it asserts
    `error.kind() == ExtractionErrorKind::CleanupFailed` and that
    `error.residue_path()` names the recorded root. The fixture must not call
    `cleanup().unwrap_err()`: that is the pattern `0.2.0` makes panic on the success
    path, and a fixture that used it would prove nothing about the new contract.
- The generated `Cargo.lock` must resolve `eggup-archive 0.2.0` from crates.io with a
  registry source and no local path, and `cargo tree --locked` must show only registry
  sources.
- The lockfile and the tree must contain **no** `eggup-core` entry. This is the proof
  that the path-only dev-dependency is stripped at packaging: the published crate's five
  runtime dependencies come from the registry, and the published archive's graph reaches
  no other Eggup crate.
- The fixture runs **after** the upload is accepted and the version is registry-visible.
  It cannot run before the upload, and no result may be claimed for it before the
  upload.

## 5. Tag and GitHub release semantics

- `v0.1.1`, `v0.1.2`, and `v0.1.3` exist and must not be moved, deleted, or
  re-pointed. The `0.1.x` line stays on crates.io; no published version is withdrawn.
- A `v0.2.0` tag may be created **only after** the registry-only proof passes, and it
  points at the exact publication source commit.
- Because all seven crates publish from one shared source commit, `v0.2.0` is the
  train's source commit. The later publications in the train append to the same release
  notes **without moving the tag**. The release notes state each package's exact
  publication source SHA rather than implying that every `0.2.0` package originated
  byte-for-byte at the tagged commit.
- No tag is created for an unpublished crate, and no tag is created for
  `eggup-transport-footprint`, which is never published.
- Package provenance is carried by `.cargo_vcs_info.json` in the published tarball; the
  closure records the source commit it names.

## 6. Ordered work packages

Perform the steps in order. Do not begin a step before the previous step's evidence is
recorded.

- **W1 — Registry preflight.** Read the crates.io version list for `eggup-archive` and
  confirm G1 and G2.
- **W2 — Source freeze and version commit.** Confirm the working tree carries the
  `0.2.0` bump (G3) and that no file under `crates/eggup-archive/src/**` differs from
  the qualified baseline (G10). The publication source commit must be a commit that
  carries the root `Cargo.toml` and `Cargo.lock` bump; publishing from the baseline
  `b75f73b` would package `0.1.3`. If the source drift check fails, stop under §9.
- **W3 — Hosted qualification.** Run the `stable`, `msrv`, `macos`, and `windows-check`
  lanes on the publication source commit and require green (G8).
- **W4 — Package qualification.** Run the package and dry-run steps of §7.3 and inspect
  the packaged manifest and file list against G4, G5, G6, and G7.
- **W5 — Authorization.** Obtain and record the maintainer authorization required by
  G12, and confirm `eggup-core 0.2.0` is registry-visible (G11).
- **W6 — Upload.** Run `cargo publish -p eggup-archive --locked` from the clean
  publication source commit. This is the only upload this milestone authorizes.
- **W7 — Registry readback.** Read the crates.io metadata for `eggup-archive 0.2.0` and
  the published tarball: version, checksum, crate size, publisher, publication
  timestamp, dependency list, and the source commit in `.cargo_vcs_info.json`. The
  published dependency list must name the five runtime dependencies and no
  `eggup-core`.
- **W8 — Registry-only fixture.** Build and run the fixture of §4; record its lockfile,
  its `cargo tree --locked` output, and its test result.
- **W9 — Tag and release.** Create `v0.2.0` and the GitHub Release, if absent, after W7
  and W8 pass, per §5.
- **W10 — Closure record.** Write `plans/closure/archive-extraction/006-status.md` with
  the requirement→evidence matrix required by §10.
- **W11 — Reconciliation.** Reconcile the registry findings, the archive subsystem
  roadmap status table, and the consumer-unblock state for the remaining train
  publications. Those files are owned by other writers; hand the intended prose over
  rather than editing them from this milestone.

## 7. Required verification

Run from the publication source commit with a clean working tree.

**7.1 — Workspace gate.**

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
```

**7.2 — MSRV floor.** The workspace `rust-version` is `1.89`. The `1.89.0` toolchain is
installed in the authoring environment — `rustup toolchain list` reports
`1.89.0-aarch64-apple-darwin` and `1.89.0-x86_64-apple-darwin` there — so these may be
run locally as well as in the hosted `msrv` lane. The hosted `msrv` lane remains the
authoritative cross-platform proof:

```sh
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
```

**7.3 — Package and dry run.**

```sh
cargo package -p eggup-archive --locked
cargo package -p eggup-archive --list --locked
cargo publish -p eggup-archive --dry-run --locked
```

**7.4 — Registry readback (read-only).** The crates.io API rejects requests that carry
no `User-Agent`, so the readback must send one:

```sh
curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' https://crates.io/api/v1/crates/eggup-archive/versions
curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' https://crates.io/api/v1/crates/eggup-archive/0.2.0/dependencies
curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' https://crates.io/api/v1/crates/eggup-core/versions
```

The first readback proves G1 and G2 — the version list names the publishing identity —
the second proves G5 and G6 on the published artifact, and the third proves G11.

**7.5 — Registry-only fixture.**

```sh
cd <fixture-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked
```

The fixture's `Cargo.lock` and `cargo tree --locked` output must be recorded, and the
tree must show no `eggup-core` entry.

**7.6 — Hosted lanes.** The `stable`, `msrv`, `macos`, and `windows-check` jobs in
`.github/workflows/ci.yml` must be green on the publication source commit, and the run
id must be recorded.

**7.7 — Publish.** Maintainer-authorized only, after §7.3 is green and G12 is recorded:

```sh
cargo publish -p eggup-archive --locked
```

## 8. Acceptance criteria

Each criterion is checkable against recorded evidence.

- **AC1 — `eggup-archive 0.2.0` is on crates.io**, with the recorded checksum, crate
  size, publisher, and publication timestamp; the recorded version list shows `0.1.2`,
  `0.1.3`, and `0.2.0`, with no existing version yanked. *(G1, G2, §7.4)*
- **AC2 — The published version is exactly `0.2.0`**, matching
  `[workspace.package] version` at the publication source commit. *(G3)*
- **AC3 — The public API boundary holds**: the surface enumerated in G4 is present and
  unchanged in the packaged build; the four `cleanup()` entry points keep the signature
  `fn cleanup(self) -> Result<(), ExtractionError>`; `ExtractionErrorKind` keeps its
  fourteen variants. *(G4)*
- **AC4 — The dependency surface is unchanged**: `flate2`, `fs_at`, `sha2`, `tar`, and
  `zip`; no added, removed, or re-versioned dependency. *(G5)*
- **AC5 — The packaged manifest carries no `git`, `path`, or `patch` dependency**, names
  no dependency absent from the registry, and carries **no** `eggup-core` entry; the
  published dependency list read back from crates.io agrees. *(G6)*
- **AC6 — The package file list is recorded** and contains no `tests/` entry; the plan,
  the release notes, and the closure record state that the M005 integration fixtures are
  not shipped. *(G7)*
- **AC7 — The hosted matrix is green on the publication source commit**: `stable`,
  `msrv`, `macos`, and `windows-check`, with the run id recorded, and the Windows lane
  having executed the archive tests. *(G8)*
- **AC8 — The local gate is green**: `cargo fmt`, `cargo clippy`, `cargo test`,
  `cargo doc` (7.1), the `1.89.0` check and test (7.2), and `cargo package` /
  `cargo package --list` / `cargo publish --dry-run` (7.3), all with recorded output.
  *(G4, G5, G6, G7)*
- **AC9 — The changelog is truthful**: `crates/eggup-archive/CHANGELOG.md` attributes
  the cleanup contract correction and the shared member-write helper to `0.2.0`, states
  the cleanup result-handling migration, and does not present `0.1.3` content as new.
  *(G9)*
- **AC10 — No source drift**: no file under `crates/eggup-archive/src/**` differs from
  the qualified baseline, or any difference is covered by its own closed corrective.
  *(G10)*
- **AC11 — Train order held**: `eggup-core 0.2.0` was registry-visible before the
  archive upload, and the closure states plainly that the published archive has no
  `eggup-core` edge, so the ordering is a train-order gate rather than a dependency
  gate. *(G11)*
- **AC12 — Maintainer authorization was recorded before the upload**, and the upload is
  the only publication act performed under this milestone. *(G12, §2)*
- **AC13 — The registry-only fixture passes**: its lockfile resolves `eggup-archive
  0.2.0` from crates.io with no path, git, or patch source; its tree contains no
  `eggup-core` entry; its bounded allowlisted extraction and its matched `cleanup()`
  assertion pass. *(§4, §7.5)*
- **AC14 — Tag and release state is correct**: `v0.1.1`, `v0.1.2`, and `v0.1.3` are
  unmoved; `v0.2.0` points at the publication source commit and was created only after
  the registry-only proof passed; the release notes state each package's exact
  publication source SHA and do not claim that the published archive ships its
  integration fixtures. *(§5)*
- **AC15 — The closure record exists** at `plans/closure/archive-extraction/006-status.md`
  with the requirement→evidence matrix required by §10, and the consumer-unblock
  reconciliation for the remaining train publications is recorded. *(§10, W11)*

## 9. Stop conditions

Stop before the upload, and raise a corrective, when any of the following holds:

- Any registry-only fixture fails, or the fixture resolves `eggup-archive` from a path,
  git, or patch source.
- The packaged public API differs materially from the surface stated in §2 and §3 — in
  particular if a `cleanup()` signature, the `ExtractionErrorKind` variant set,
  `ArchiveLimits::strict_default()`, or a `root()` accessor changed, or if the packaged
  build carried an `eggup-core` dependency.
- The hosted matrix is not green on the publication source commit.
- `eggup-archive 0.2.0` is already present on crates.io, whether published by this
  milestone, by a prior run, or by another actor.
- The upload would require moving, deleting, or re-pointing an existing tag.
- A dependency named by the package is not yet on the registry.
- The package file list shows a `tests/` entry, or the packaged manifest carries a `git`,
  `path`, or `patch` dependency.
- Any file under `crates/eggup-archive/src/**` differs from the qualified baseline
  without a closed corrective of its own.
- Publish authority for the `eggup-archive` crate name cannot be confirmed.
- `eggup-core 0.2.0` is not yet registry-visible and the train order cannot be
  preserved.

After an accepted upload, registry state is immutable: a defect in the published
`0.2.0` requires a **new patch version** (`0.2.1`), never an overwrite, a re-upload, or
a yank-and-republish of `0.2.0`.

## 10. Closure evidence required

`plans/closure/archive-extraction/006-status.md` must carry a requirement→evidence
matrix with, at minimum:

- the release-prep SHA — the full 40-character form of `b75f73b` — and the publication
  source commit, each identified as the same or as distinct.
- the hosted run id and the per-lane results for `stable`, `msrv`, `macos`, and
  `windows-check`, including the Windows lane's archive-test execution.
- the local gate output for 7.1, 7.2, and 7.3, including the `1.89.0` toolchain identity
  actually used.
- the package inventory: the `cargo package -p eggup-archive --list --locked` file list
  and the packaged manifest's dependency section.
- the dependency tree for `eggup-archive`, showing the five runtime dependencies and
  the dev-only `eggup-core` edge in the workspace, plus the **published** dependency
  list read back from crates.io showing the five runtime dependencies and **no**
  `eggup-core` entry — the proof that the path-only dev-dependency is stripped at
  packaging.
- the crates.io checksum, crate size, publisher, publication timestamp, published
  dependency list, and the source commit recorded in the published tarball's
  `.cargo_vcs_info.json`.
- the registry-only fixture's lockfile, its `cargo tree --locked` output, and its test
  result, including the explicit absence of any `eggup-core` entry.
- the tag and release state: the unmoved `v0.1.1`, `v0.1.2`, and `v0.1.3`; the `v0.2.0`
  target commit; the release URL.
- the consumer-unblock reconciliation: which train publications are now unblocked by
  `eggup-archive 0.2.0` being on the registry, and which remain blocked on an earlier
  step of the train.
- the truthful statement that the published package does not ship the crate's `tests/`
  fixtures, and the migration note that a caller which wrote `cleanup().unwrap_err()`
  must now match on the result.

The closure record must describe the publication as performed, with dated evidence, and
must not restate this plan's forward-looking language.
