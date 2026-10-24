# Acquisition Transport Milestone 017 — eggup-acquisition 0.2.0 publication

Status: ready for handoff

Repository baseline: `b75f73b`

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md

Primary class: infrastructure

Hard dependencies:

- Acquisition M013 closed. The seam's `Io` variant is retyped to
  `Io { operation, source }` and `AcquisitionError` derives `Debug` only.
- Acquisition M014 closed. The curl adapter exposes the caller-settable
  `User-Agent`.
- Acquisition M015 closed. The curl spawn-failure detail names the `ErrorKind`
  and the OS code.
- Acquisition M016 closed. The `0.2.0` coupling is recorded in the seam's
  changelog and guarded by
  `crates/eggup-acquisition/tests/publication_coupling.rs`.
- The `0.2.0` version bump is present in the working tree at the baseline. This
  milestone authorizes the upload; it does not introduce the bump.
- `eggup-acquisition 0.2.0` must be on the registry before either adapter can be
  packaged. This is the mechanical dependency-order constraint, not a defect.

## 1. Objective

Publish three crates to crates.io, in this order:

1. `eggup-acquisition 0.2.0` — the transport-neutral seam.
2. `eggup-curl 0.2.0` — the external-curl adapter.
3. `eggup-eggfetch 0.2.0` — the native HTTP adapter.

They are one milestone because they are one coupled seam → adapter → adapter
chain. The seam's `0.2.0` release is **source-breaking**, and a caret
requirement does not cross a major boundary: `^0.1.0` does not accept `0.2.0`.
Both adapters declare `eggup-acquisition = "0.2"` in the working tree, so
neither adapter can be packaged against the registry until the seam's `0.2.0` is
visible there. Before this publication neither adapter could be packaged at
all — that is registry open finding 17, and closing it is the milestone's
reason to exist.

**Why `0.2.0` and not `0.1.4`.** The workspace is version-lockstep:
`[workspace.package] version` is inherited by all eight crates through
`version.workspace = true`. The seam's next release is source-breaking, so the
seam had to go to `0.2.0`, and the lockstep workspace carried every crate with
it. `eggup-acquisition 0.2.0` is therefore the first release of the seam that
is not a `0.1.x`, and the adapters' next published versions are `0.2.0` rather
than `0.1.3`. The coupling is recorded in the seam's own changelog and in the
Acquisition M016 closure record. The adapters' `0.2.0` numbers are a
consequence of the seam's break, not a declaration that either adapter carries a
comparable break of its own.

**The seam's break is exactly two changes.** A consumer that depends on the
seam directly, or on either adapter, must make both of them and nothing else:

- `AcquisitionError::Io(String)` became `AcquisitionError::Io { operation,
  source }`, with a live `source()` chain. A consumer that wrote
  `AcquisitionError::Io(_)` must write `AcquisitionError::Io { .. }`.
- `AcquisitionError` now derives **only** `Debug`. `Clone`, `PartialEq`, and
  `Eq` are removed, because `std::io::Error` implements none of them. A
  consumer that cloned or compared whole error values must match on the variant
  and its fields instead.

**Nothing else is consumer-visible.** The `Display` rendering stays bounded at
512 bytes and is truncated only at a UTF-8 character boundary. `Io` stays
terminal under every `CompositionPolicy`. The crate keeps **zero**
`[dependencies]`. The two `#[doc(hidden)]` adapter-facing entry points
(`__adapter_io`, `__adapter_bound_detail`) are source-compatible additions and
are not consumer contract.

**The adapters carry no source-breaking change of their own.** `eggup-curl
0.2.0` is transitively source-breaking through `eggup-acquisition ^0.2`; the
break is inherited from the seam, not introduced by the adapter. Its own public
API is additive: `CurlTransport::with_executable_and_user_agent` and
`CurlTransport::user_agent` are the only additions, `CurlConfig` is untouched,
and the observable wire behaviour of the published adapter does not change. The
rejected `User-Agent` value is a terminal `AcquisitionError::InvalidInput` and
never `Unavailable`. `eggup-eggfetch 0.2.0` is likewise transitively
source-breaking through the same seam requirement; its own public API is
unchanged and `EggfetchTransport` remains shareable across threads.

## 2. Publication boundary

Publish exactly `eggup-acquisition 0.2.0`, `eggup-curl 0.2.0`, and
`eggup-eggfetch 0.2.0`, in that order.

The lockstep train publishes in dependency order — `eggup-core` →
`eggup-archive` → `eggup-acquisition` → `eggup-curl` → `eggup-eggfetch` →
`eggup-service` → `eggup-eggpack` — and all seven publish from **one shared
source commit**. This milestone covers the seam-and-adapters half of that
train. Do not publish, and do not upload a dry-run artifact for, any other
crate under this milestone. `eggup-transport-footprint` carries `publish =
false` and is never published.

`eggup-curl 0.2.0` and `eggup-eggfetch 0.2.0` both depend on
`eggup-acquisition = "0.2"`, so `eggup-acquisition 0.2.0` must be registry-
visible before either adapter is published.

**In scope.** The registry upload of the three crates, their package
qualification, their registry-only proofs, the tag and release, and the closure
record.

**Explicitly out of scope.** Any change to `crates/eggup-acquisition/src/**`,
`crates/eggup-curl/src/**`, or `crates/eggup-eggfetch/src/**`. Any version
change, any manifest edit, and any widening or narrowing of a dependency
requirement — the bump and the widenings are already in the tree and this
milestone authorizes the upload only. Publication of `eggup-core 0.2.0`,
`eggup-archive 0.2.0`, `eggup-service 0.2.0`, or `eggup-eggpack 0.2.0`. Any
change to `plans/registry.md`, `plans/subsystems/**`, `architecture/**`,
`docs/**`, `AGENTS.md`, `scripts/**`, `.github/**`, or any `CHANGELOG.md`;
those are owned by other writers and are reconciled by the closure step in §6.

## 3. Pre-publication gates

Every gate below must hold at the publication source commit. Each gate names
the verification that proves it; the full command list is §7 and the
corresponding checkable criterion is §8.

- **G1 — Registry preflight.** `eggup-acquisition 0.2.0`, `eggup-curl 0.2.0`,
  and `eggup-eggfetch 0.2.0` are each absent from crates.io before their
  upload. The existing versions are present and none is yanked: `eggup-acquisition`
  at `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`; `eggup-curl` at `0.1.2` only — it
  never had a `0.1.3`, because the seam's break moved the whole workspace to
  `0.2.0`; `eggup-eggfetch` at `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`. No existing
  version is re-published, re-uploaded, or yanked by this milestone.
- **G2 — Publish authority.** The publishing identity holds publish authority
  for all three crate names and a crates.io API token is available.
- **G3 — Version identity.** The publication source commit carries
  `[workspace.package] version = "0.2.0"`, inherited by all three crates
  through `version.workspace = true`, so each packaged crate version is exactly
  `0.2.0`. At the baseline `b75f73b` the bump is still an **uncommitted**
  working-tree change to the root `Cargo.toml` and `Cargo.lock`; the
  publication source commit must carry it, or the packaged crates would resolve
  to `0.1.3`.
- **G4 — Seam public API surface.** The packaged seam exposes exactly the
  surface stated in §1: the `AcquisitionError` variants `InvalidInput(String)`,
  `Transport(String)`, `TooLarge { limit }`, `Timeout { phase }`, `Cancelled`,
  `Io { operation, source }`, and `Unavailable(String)`, with the enum
  `#[non_exhaustive]` and deriving `Debug` only; `AcquisitionRequest`,
  `FetchLimits`, `FetchOutcome`, `MetadataBytes`, `ArtifactEvidence`,
  `CancelFlag`, `CompositionPolicy`, `ComposedTransport`, `FixtureTransport`,
  and `AcquisitionTransport` unchanged in shape; the `Display` bound at 512
  bytes; `Io` terminal under every `CompositionPolicy`.
- **G5 — Adapter public API surface.** The packaged `eggup-curl 0.2.0` exposes
  `CurlConfig` with the same six public fields as the published `0.1.2`
  (`connect_timeout`, `total_timeout`, `max_redirects`, `follow_redirects`,
  `allowed_protocols`, `proxy`), the `CurlConfig` builder methods, `CurlProxy`,
  `CurlTransport` with `with_executable`, `with_executable_and_user_agent`,
  `executable`, `config`, and `user_agent`, `MAX_USER_AGENT_BYTES`, and
  `discover_curl_executable`. No published signature, field, constructor, or
  error variant is added, removed, or retyped. The packaged `eggup-eggfetch
  0.2.0` exposes `EggfetchConfig` with the same five public fields
  (`user_agent`, `connect_timeout`, `total_timeout`, `max_redirects`, `proxy`),
  `ProxyDecision`, `EggfetchTransport` with `strict`, `config`, and
  `eggfetch_version`, `DEFAULT_USER_AGENT`, and the default timeout and
  redirect constants — unchanged in shape from the published `0.1.3`.
- **G6 — Dependency surface.** The packaged seam manifest has an **empty**
  `[dependencies]` section. The published dependency surface of the packaged
  curl manifest names only `eggup-acquisition` (`^0.2`). The published
  dependency surface of the packaged eggfetch manifest names only
  `eggup-acquisition` (`^0.2`), `eggfetch-core`, `tokio`, and `futures-util`;
  its `[dev-dependencies]` are not part of the published dependency graph. No
  dependency is added, removed, or re-versioned relative to the published
  baseline — the only manifest change in either adapter is the seam requirement
  widening from `"0.1.0"` to `"0.2"`, and the seam's own manifest is
  unchanged.
- **G7 — No unpublished dependency edge.** No packaged manifest carries a
  `git`, `path`, or `patch` dependency, and each names only registry
  dependencies that are already on the registry at the moment it is packaged.
  The seam's requirement is the one edge that changes: `eggup-acquisition = "0.2"`
  resolves only against the seam's published `0.2.0`.
- **G8 — Package contents.** `cargo package -p <crate> --list --locked` shows
  the crate's `src/`, `README.md`, `CHANGELOG.md`, and normalized `Cargo.toml`,
  and shows **no** `tests/` or `benches/` entry — all three crates set
  `exclude = ["tests/", "benches/"]`, so the in-tree integration fixtures are
  not shipped.
- **G9 — Hosted matrix green.** The `stable`, `msrv`, `macos`, and
  `windows-check` lanes are green on the publication source commit.
- **G10 — Changelog truthfulness.** The three per-crate `## 0.2.0` sections
  state the seam's two source-break items, state that each adapter inherits the
  break rather than introducing one, and name
  `plans/closure/acquisition-transport/017-status.md` as the closure record.
  The "published to crates.io" wording is only truthful once the corresponding
  upload is accepted; if an upload is not accepted, the section must be
  corrected to record the unpublished state. The seam's `## Unreleased`
  boilerplate is moved under the published version at cut time, and the root
  `CHANGELOG.md` records the completed publication set with checksums.
- **G11 — The M016 coupling guard agrees with the cut changelog.** The guard
  in `crates/eggup-acquisition/tests/publication_coupling.rs` asserted that the
  seam's changelog carries a live `## Unreleased` section that declares the next
  release as `0.2.0`. The changelog has since been cut to a `## 0.2.0` section,
  so the guard failed as written
  (`seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release`
  and its two siblings). The guard has been re-pointed at the seam's
  **published** version — derived from the highest `## X.Y.Z` heading in the
  seam's own changelog, never from a literal — and the same three test names
  are retained verbatim because `crates/eggup-acquisition/tests/assertion_guard.rs`
  names them as its required behaviour witnesses. The re-target is recorded as
  an addendum in `plans/closure/acquisition-transport/016-status.md`. The
  milestone did not weaken the guard to make the gate pass: the invariant it
  enforces is the *post-publication* form of the same coupling rule, and it was
  proven to still discriminate.
- **G12 — No unclassified source delta.** No file under
  `crates/eggup-acquisition/src/**`, `crates/eggup-curl/src/**`, or
  `crates/eggup-eggfetch/src/**` differs from the qualified baseline. A source
  change requires its own corrective before publication; it must not ride along
  in this milestone.
- **G13 — Maintainer authorization.** An explicit, recorded authorization to
  upload the three crates exists before any upload runs.

## 4. Registry-only proof

All three fixtures are **fresh crates outside this repository**, with no `path`,
`git`, or `patch` override in the manifest, in the workspace, or in
`.cargo/config.toml`. They resolve only from crates.io. A fixture that resolves
from the local workspace proves nothing and must be discarded and rebuilt.

**The fixtures are not all runnable at the same time.** A fixture that depends
on a crate can only resolve it after that crate is on the registry. The table
below states which upload each fixture gates.

| Fixture | Depends on | First runnable | Hard gate of |
|---|---|---|---|
| A — direct seam | `eggup-acquisition = "=0.2.0"` | after upload 1 | upload 2 (`eggup-curl 0.2.0`) and upload 3 (`eggup-eggfetch 0.2.0`) |
| B — curl adapter | `eggup-curl = "=0.2.0"` | after upload 2 | upload 3 (`eggup-eggfetch 0.2.0`), and the milestone's completion |
| C — eggfetch adapter | `eggup-eggfetch = "=0.2.0"` | after upload 3 | the milestone's completion |

Upload 1 is gated by G1–G13 and by the package and dry-run steps of §7.4; its
own registry-only proof is Fixture A, which runs immediately after it and gates
both adapter uploads. Upload 2 is gated additionally by Fixture A. Upload 3 is
gated additionally by Fixture A and Fixture B: Fixture B is what proves that a
registry consumer can depend on an adapter and receive the published
`eggup-acquisition 0.2.0` transitively — the same edge upload 3 exercises — and
no third upload may run on an unproven seam-to-adapter edge. Fixture C is the
readback for upload 3 and gates the tag and the closure record.

### Fixture A — direct consumer of the seam

- Depends on `eggup-acquisition = "=0.2.0"` and on **nothing else**: no
  workspace crate, and no third-party dependency at all, in `[dependencies]` or
  `[dev-dependencies]`. The fixture therefore has **zero** dependencies beyond
  the seam — no `static_assertions`, no `trybuild`, no `tempfile`.
- The generated `Cargo.lock` must contain exactly one package entry,
  `eggup-acquisition 0.2.0`, with a registry source and the published
  checksum. `cargo tree --locked` must show no local path and no git source.
- The fixture asserts the **positive** surface of the retyped variant:
  - `AcquisitionError::Io { operation, source }` is constructible and
    matchable; the fixture matches on it, asserts the `operation` label equals
    the expected string, and asserts `source.kind()` equals the expected
    `std::io::ErrorKind`.
  - The `source()` chain is live: `std::error::Error::source(&err)` is `Some`
    for `Io` and downcasts to `std::io::Error`, and is `None` for every other
    variant.
  - The `Display` rendering stays bounded: the fixture renders a detail whose
    UTF-8 length exceeds the bound with a multi-byte character at the cut, and
    asserts the rendered output is at most 512 bytes, is valid UTF-8, and ends
    without a partial code point.
  - `Io` is terminal under composition: the fixture implements two trivial
    `AcquisitionTransport`s — one that returns
    `AcquisitionError::Io { operation, source }`, one that would succeed and
    records whether it was called — composes them with
    `CompositionPolicy::UnavailableOrTransport`, and asserts the secondary is
    never called. The trait is part of the seam's public surface, so the
    fixture needs no dependency and opens no socket to do this.
- **The absent derives are documented, not compiled.** The absence of `Clone`,
  `PartialEq`, and `Eq` cannot be asserted by a stable-Rust trait-absence
  test, and a negative compilation would need a dependency the fixture must not
  have. The fixture therefore carries a `README.md` next to it that states the
  compile-fail expectation for a consumer of the published `0.2.0`, with the
  two snippets a consumer must change: `AcquisitionError::Io(_)`, and
  `err.clone()` / `assert_eq!(err, other)` on whole error values. The closure
  records the compiler output observed when those two snippets are compiled ad
  hoc against the published `0.2.0`. The fixture asserts the **positive**
  replacement instead: the same information is available by matching on the
  variant and its fields, so `operation` and `source.kind()` are comparable
  with `assert_eq!` on the fields.

### Fixture B — consumer of the curl adapter

- Depends on `eggup-curl = "=0.2.0"` and on nothing else from this workspace.
- The generated `Cargo.lock` must resolve **both** `eggup-curl 0.2.0` **and**
  `eggup-acquisition 0.2.0` from crates.io, each with a registry source and the
  published checksum, with no `path`, `git`, or `patch` entry for any Eggup
  crate anywhere in the tree. `cargo tree --locked` must show the same.
- The fixture asserts:
  - `CurlTransport::with_executable_and_user_agent` accepts a well-formed
    value and `CurlTransport::user_agent` returns it.
  - A value containing `CR LF` is rejected with `AcquisitionError::InvalidInput`,
    and the rejection is **terminal**: the error is not `Unavailable` and
    `is_unavailable()` is false.
  - `CurlConfig` is unchanged: the fixture constructs `CurlConfig::strict()` and
    reads its public fields, so a widened, removed, or retyped field fails to
    compile against the published `0.2.0`.
  - The seam's retyped error is part of the adapter's own public surface:
    `CurlTransport` implements `AcquisitionTransport`, whose
    `fetch_metadata` and `fetch_artifact` return `AcquisitionError`, so the
    fixture coerces `&CurlTransport` to `&dyn AcquisitionTransport` and writes
    its error handler against the retyped variant,
    `AcquisitionError::Io { operation, source }`. The coercion and the match
    are compile-time facts; no request is sent.
- The fixture performs **no** fetch and opens no socket. It exercises
  construction, validation, and the type surface only.

### Fixture C — consumer of the eggfetch adapter

- Depends on `eggup-eggfetch = "=0.2.0"` and on nothing else from this
  workspace.
- The generated `Cargo.lock` must resolve `eggup-eggfetch 0.2.0`,
  `eggup-acquisition 0.2.0`, `eggfetch-core`, `tokio`, and `futures-util`
  from crates.io, with no `path`, `git`, or `patch` source for any of them.
- The fixture asserts:
  - `EggfetchTransport::strict(EggfetchConfig::strict())` constructs, and
    `EggfetchTransport::config()` returns the config it was built with.
  - `EggfetchTransport` is `Send + Sync`, proved positively: the fixture moves
    the transport into a `std::thread::spawn` closure, which requires `Send`,
    and borrows `&EggfetchTransport` inside a `std::thread::scope`, which
    requires `Sync`. No dependency and no negative compilation is needed.
  - `EggfetchConfig::user_agent` is readable and defaults to
    `eggup-eggfetch`.
  - The seam's retyped error is part of the adapter's own public surface:
    `EggfetchTransport` implements `AcquisitionTransport`, whose
    `fetch_metadata` and `fetch_artifact` return `AcquisitionError`, so the
    fixture coerces `&EggfetchTransport` to `&dyn AcquisitionTransport` and
    writes its error handler against the retyped variant,
    `AcquisitionError::Io { operation, source }`. The coercion and the match
    are compile-time facts; no request is sent.
- The fixture performs **no** fetch and opens no socket. Constructing the
  adapter configures a TLS root store and a proxy environment; it performs no
  network I/O. If construction fails in the fixture's environment, the closure
  records it as an environment limitation, never as a pass.

## 5. Tag and GitHub release semantics

- `v0.1.1`, `v0.1.2`, and `v0.1.3` exist and must not be moved. The `0.1.x`
  line stays on crates.io; no published version is withdrawn.
- A `v0.2.0` tag may be created **only after** the registry-only proofs pass —
  that is, after Fixture C is green — and it points at the exact publication
  source commit. If `v0.2.0` already exists, it is **not** moved: the
  milestone verifies that it points at the same shared publication source
  commit and appends to the release notes instead.
- Because all seven crates publish from one shared source commit, `v0.2.0` is
  the train's source commit. The later publications in the train append to the
  same release notes **without moving the tag**. The release notes must state
  each package's exact publication source SHA rather than implying that every
  `0.2.0` package originated byte-for-byte at the tagged commit.
- No tag is created for an unpublished crate, and no tag is created for
  `eggup-transport-footprint`, which is never published.
- Package provenance is carried by `.cargo_vcs_info.json` in the published
  tarball; the closure records the source commit it names.

## 6. Ordered work packages

Perform the steps in order. Do not begin a step before the previous step's
evidence is recorded.

- **W1 — Registry preflight.** Read the crates.io version lists for the three
  crates and confirm G1 and G2.
- **W2 — Source freeze and version commit.** Confirm the working tree carries
  the `0.2.0` bump (G3) and that no file under the three crates' `src/**`
  differs from the qualified baseline (G12). The publication source commit
  must carry the root `Cargo.toml` and `Cargo.lock` bump; publishing from the
  baseline `b75f73b` would package `0.1.3`. Reconcile the M016 coupling guard
  with the cut changelog (G11) and require the workspace gate green. If the
  source drift check fails, stop under §9.
- **W3 — Hosted qualification.** Run the `stable`, `msrv`, `macos`, and
  `windows-check` lanes on the publication source commit and require green
  (G9).
- **W4 — Seam package qualification.** Run the seam's package, file-list, and
  dry-run steps of §7.4 and inspect the packaged manifest and file list against
  G4, G6, G7, and G8.
- **W5 — Authorization.** Obtain and record the maintainer authorization
  required by G13.
- **W6 — Upload the seam.** Run `cargo publish -p eggup-acquisition --locked`
  from the clean publication source commit, then read the crates.io metadata
  and the published tarball: version, checksum, crate size, publisher,
  publication timestamp, dependency list, and the source commit in
  `.cargo_vcs_info.json`. Confirm the version list shows `0.2.0` and that no
  earlier version was yanked.
- **W7 — Fixture A.** Build and run the direct seam fixture; record its
  lockfile and test result. This is the hard gate of W9 and W11.
- **W8 — Adapter package qualification.** Only after the seam's `0.2.0` is
  registry-visible, run the curl and eggfetch package, file-list, and dry-run
  steps of §7.4 and inspect the packaged manifests and file lists against G5,
  G6, G7, and G8.
- **W9 — Upload the curl adapter.** Run `cargo publish -p eggup-curl --locked`
  from the clean publication source commit, then read the crates.io metadata
  and the published tarball as in W6.
- **W10 — Fixture B.** Build and run the curl fixture; record its lockfile and
  test result. This is the hard gate of W11 and of the milestone's completion.
- **W11 — Upload the eggfetch adapter.** Run `cargo publish -p eggup-eggfetch
  --locked` from the clean publication source commit, then read the crates.io
  metadata and the published tarball as in W6.
- **W12 — Fixture C.** Build and run the eggfetch fixture; record its lockfile
  and test result. This gates the tag and the closure record.
- **W13 — Tag and release.** Create `v0.2.0` and the GitHub Release, if
  absent, after W7, W10, and W12 pass, per §5.
- **W14 — Closure record.** Write
  `plans/closure/acquisition-transport/017-status.md` with the
  requirement→evidence matrix required by §10.
- **W15 — Reconciliation.** Reconcile the registry findings, the acquisition
  subsystem roadmap status table, and the consumer-unblock state for the
  remaining train publications. Those files are owned by other writers; hand
  the intended prose over rather than editing them from this milestone.

## 7. Required verification

Run from the publication source commit with a clean working tree.

**7.1 — Workspace gate.**

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
```

**7.2 — MSRV floor.** The workspace `rust-version` is `1.89`. The `1.89.0`
toolchain is installed in the authoring environment (`rustup toolchain list`
reports `1.89.0-aarch64-apple-darwin` and `1.89.0-x86_64-apple-darwin` there),
so these may be run locally as well as in the hosted `msrv` lane. The hosted
`msrv` lane remains the authoritative cross-platform proof:

```sh
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
```

**7.3 — Dependency surface.**

```sh
cargo tree -p eggup-acquisition --locked --depth 1   # must show no dependency line
cargo tree -p eggup-curl --locked --depth 1          # eggup-acquisition only
cargo tree -p eggup-eggfetch --locked --depth 1       # eggup-acquisition + three crates
```

**7.4 — Package and dry run.** The order is load-bearing: `cargo package`
resolves the packaged manifest's dependencies from the **registry**, so the
adapters' package and dry-run steps can only succeed once the seam's `0.2.0`
is registry-visible. This is the dependency-order constraint, not a defect, and
no step is skipped for the crate actually being uploaded.

```sh
# Step 1 — before upload 1.
cargo package -p eggup-acquisition --locked
cargo package -p eggup-acquisition --list --locked
cargo publish -p eggup-acquisition --dry-run --locked

# Step 2 — only after eggup-acquisition 0.2.0 is registry-visible.
cargo package -p eggup-curl --locked
cargo package -p eggup-curl --list --locked
cargo publish -p eggup-curl --dry-run --locked

cargo package -p eggup-eggfetch --locked
cargo package -p eggup-eggfetch --list --locked
cargo publish -p eggup-eggfetch --dry-run --locked
```

**7.5 — Registry readback (read-only).**

```sh
curl -fsSL https://crates.io/api/v1/crates/eggup-acquisition/versions
curl -fsSL https://crates.io/api/v1/crates/eggup-curl/versions
curl -fsSL https://crates.io/api/v1/crates/eggup-eggfetch/versions
curl -fsSL https://crates.io/api/v1/crates/eggup-acquisition/0.2.0/dependencies
curl -fsSL https://crates.io/api/v1/crates/eggup-curl/0.2.0/dependencies
curl -fsSL https://crates.io/api/v1/crates/eggup-eggfetch/0.2.0/dependencies
```

**7.6 — Registry-only fixtures.**

```sh
# Fixture A — direct seam. After upload 1, before uploads 2 and 3.
cd <fixture-a-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked

# Fixture B — curl adapter. After upload 2, before upload 3.
cd <fixture-b-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked

# Fixture C — eggfetch adapter. After upload 3.
cd <fixture-c-dir>
cargo generate-lockfile
cargo tree --locked
cargo test --locked
```

**7.7 — Hosted lanes.** The `stable`, `msrv`, `macos`, and `windows-check` jobs
in `.github/workflows/ci.yml` must be green on the publication source commit,
and the run id must be recorded.

**7.8 — Publish.** Maintainer-authorized only, after §7.4 is green for the
crate being uploaded and G13 is recorded:

```sh
cargo publish -p eggup-acquisition --locked
cargo publish -p eggup-curl --locked
cargo publish -p eggup-eggfetch --locked
```

## 8. Acceptance criteria

Each criterion is checkable against recorded evidence.

- **AC1 — All three crates are on crates.io**, with the recorded checksum, crate
  size, publisher, and publication timestamp for each; the recorded version
  lists show `eggup-acquisition` at `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`,
  `0.2.0`, `eggup-curl` at `0.1.2`, `0.2.0`, and `eggup-eggfetch` at
  `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0`, with no existing version yanked.
  *(G1, G2, §7.5)*
- **AC2 — Each published version is exactly `0.2.0`**, matching
  `[workspace.package] version` at the publication source commit. *(G3)*
- **AC3 — The seam's published API matches the stated boundary**: the seven
  `AcquisitionError` variants with `Io { operation, source }`; the enum
  `#[non_exhaustive]` and deriving `Debug` only; the `Display` bound at 512
  bytes; `Io` terminal under every `CompositionPolicy`; the rest of the seam's
  public surface unchanged in shape. *(G4, §4 Fixture A)*
- **AC4 — The adapters' published APIs match the stated boundary**: `CurlConfig`
  keeps its six public fields and the curl adapter adds only
  `with_executable_and_user_agent` and `user_agent`; `EggfetchConfig` keeps
  its five public fields and `EggfetchTransport` keeps `strict`, `config`, and
  `eggfetch_version`; no published signature, field, constructor, or error
  variant is added, removed, or retyped. *(G5)*
- **AC5 — The dependency surface is as stated**: the seam's packaged manifest
  has an empty `[dependencies]` section; curl names only `eggup-acquisition`
  (`^0.2`); eggfetch names only `eggup-acquisition` (`^0.2`), `eggfetch-core`,
  `tokio`, and `futures-util`. *(G6, §7.3)*
- **AC6 — No packaged manifest carries a `git`, `path`, or `patch` dependency**,
  and each names only registry dependencies that were on the registry when it
  was packaged. *(G7)*
- **AC7 — The package file lists are recorded** and contain no `tests/` or
  `benches/` entry. *(G8)*
- **AC8 — The hosted matrix is green on the publication source commit**:
  `stable`, `msrv`, `macos`, and `windows-check`, with the run id recorded.
  *(G9)*
- **AC9 — The local gate is green**: `cargo fmt`, `cargo clippy`, `cargo test`,
  `cargo doc` (7.1), the `1.89.0` check and test (7.2), the dependency trees
  (7.3), and the package / `package --list` / `publish --dry-run` steps (7.4),
  all with recorded output and with the ordering of 7.4 preserved.
  *(G4, G5, G6, G7, G8)*
- **AC10 — The changelogs are truthful**: the three `## 0.2.0` sections state
  the seam's two source-break items and the adapters' inherited break, name
  this milestone's closure record, and their publication wording matches the
  crates.io readback. *(G10)*
- **AC11 — The M016 coupling guard is reconciled with the cut changelog** and
  the workspace gate is green on the publication source commit; no guard was
  weakened to make the gate pass. *(G11)*
- **AC12 — No source drift**: no file under the three crates' `src/**` differs
  from the qualified baseline, or any difference is covered by its own closed
  corrective. *(G12)*
- **AC13 — Maintainer authorization was recorded before the first upload**,
  and the three uploads are the only publication acts performed under this
  milestone. *(G13, §2)*
- **AC14 — Fixture A passes**: its lockfile resolves `eggup-acquisition 0.2.0`
  from crates.io as the only package, with no path, git, or patch source; its
  test suite passes; and the closure records the documented compile-fail
  expectation for `AcquisitionError::Io(_)` and for whole-value cloning and
  comparison, with the observed compiler output. *(§4 Fixture A, §7.6)*
- **AC15 — Fixture B passes**: its lockfile resolves both `eggup-curl 0.2.0`
  and `eggup-acquisition 0.2.0` from crates.io with no path, git, or patch
  source; its test suite passes. *(§4 Fixture B, §7.6)*
- **AC16 — Fixture C passes**: its lockfile resolves `eggup-eggfetch 0.2.0`,
  `eggup-acquisition 0.2.0`, `eggfetch-core`, `tokio`, and `futures-util` from
  crates.io with no path, git, or patch source; its test suite passes.
  *(§4 Fixture C, §7.6)*
- **AC17 — The upload order was respected**: the seam's `0.2.0` was
  registry-visible before the curl and eggfetch package, dry-run, and upload
  steps ran. *(§7.4, W8, W9, W11)*
- **AC18 — Tag and release state is correct**: `v0.1.1`, `v0.1.2`, and
  `v0.1.3` are unmoved; `v0.2.0` points at the publication source commit and
  was created only after the registry-only proofs passed; the release notes
  state each package's exact publication source SHA. *(§5)*
- **AC19 — The closure record exists** at
  `plans/closure/acquisition-transport/017-status.md` with the
  requirement→evidence matrix required by §10, and the consumer-unblock
  reconciliation for the remaining train publications is recorded. *(§10, W15)*

## 9. Stop conditions

Stop before the affected upload, and raise a corrective, when any of the
following holds:

- Any registry-only fixture fails, or a fixture resolves any Eggup crate from a
  path, git, or patch source.
- The packaged public API differs materially from the surface stated in §1 and
  §3 — in particular if the packaged seam still derives `Clone`, `PartialEq`,
  or `Eq` on `AcquisitionError`, still carries `Io(String)`, or if the packaged
  curl adapter's `CurlConfig` gained, lost, or retyped a public field.
- The hosted matrix is not green on the publication source commit.
- `0.2.0` is already present on crates.io for the crate about to be uploaded,
  whether published by this milestone, by a prior run, or by another actor.
- The upload would require moving, deleting, or re-pointing an existing tag.
- A dependency named by the package is not yet on the registry. **In
  particular, stop before uploading `eggup-curl 0.2.0` or `eggup-eggfetch
  0.2.0` if `eggup-acquisition 0.2.0` is not yet on the registry.**
- The package file list shows a `tests/` or `benches/` entry, or the package
  manifest carries a `git`, `path`, or `patch` dependency.
- Any file under the three crates' `src/**` differs from the qualified baseline
  without a closed corrective of its own.
- Publish authority for any of the three crate names cannot be confirmed.
- The M016 coupling guard cannot be reconciled with the cut changelog without
  weakening it — the guard's owner must resolve that, not this milestone.

After an accepted upload, registry state is immutable: a defect in a published
`0.2.0` requires a **new patch version** (`0.2.1`), never an overwrite, a
re-upload, or a yank-and-republish of `0.2.0`.

Publication is not transactional. A partial state is acceptable only if each
published version is individually correct and the remainder is completed in
order; it is not acceptable to publish a crate whose declared dependency is not
yet registry-visible.

## 10. Closure evidence required

`plans/closure/acquisition-transport/017-status.md` must carry a
requirement→evidence matrix with, at minimum:

- the release-prep SHA — the full 40-character form of `b75f73b` — and the
  publication source commit, each identified as the same or as distinct.
- the hosted run id and the per-lane results for `stable`, `msrv`, `macos`, and
  `windows-check`.
- the local gate output for 7.1, 7.2, 7.3, and 7.4, including the `1.89.0`
  toolchain identity actually used, and the recorded order in which the three
  crates' package and dry-run steps ran.
- the package inventory for each of the three crates: the
  `cargo package -p <crate> --list --locked` file list and the packaged
  manifest's dependency section.
- the dependency tree for each of the three crates, showing the seam with no
  dependency line, curl with `eggup-acquisition` alone, and eggfetch with
  `eggup-acquisition` plus `eggfetch-core`, `tokio`, and `futures-util`.
- the crates.io checksum, crate size, publisher, publication timestamp,
  published dependency list, and the source commit recorded in each published
  tarball's `.cargo_vcs_info.json`, for all three crates.
- each registry-only fixture's lockfile and test result, including Fixture A's
  documented compile-fail expectation for `AcquisitionError::Io(_)` and for
  whole-value cloning and comparison, with the observed compiler output.
- the tag and release state: the unmoved `v0.1.1`, `v0.1.2`, and `v0.1.3`; the
  `v0.2.0` target commit; the release URL.
- the consumer-unblock reconciliation: which train publications are unblocked by
  `eggup-acquisition 0.2.0`, `eggup-curl 0.2.0`, and `eggup-eggfetch 0.2.0`
  being on the registry, and which remain blocked on an earlier step of the
  train.

The closure record must describe the publication as performed, with dated
evidence, and must not restate this plan's forward-looking language.
