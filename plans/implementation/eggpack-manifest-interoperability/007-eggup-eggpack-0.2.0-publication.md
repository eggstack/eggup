# Eggpack Manifest Interoperability Milestone 007 — eggup-eggpack 0.2.0 publication

Status: ready for handoff

Repository baseline: `b75f73b`

Source roadmap:
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md
- plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md

Primary class: infrastructure

Hard dependencies — do not start this milestone until each is satisfied:

| Dependency | Requirement |
|---|---|
| `eggup-core 0.2.0` | Published to crates.io and registry-visible. |
| `eggup-archive 0.2.0` | Published to crates.io and registry-visible. |
| `eggup-acquisition 0.2.0` | Published to crates.io and registry-visible. The seam's `0.2.0` is source-breaking. |
| `eggup-curl 0.2.0` | Published to crates.io and registry-visible. |
| `eggup-eggfetch 0.2.0` | Published to crates.io and registry-visible. |
| `eggup-service 0.2.0` | Published to crates.io and registry-visible. |

This milestone is the last of the seven. The train publishes in the order
`eggup-core` → `eggup-archive` → `eggup-acquisition` → `eggup-curl` →
`eggup-eggfetch` → `eggup-service` → `eggup-eggpack`, and all seven publish from
one shared source commit.

## 1. Objective

Publish `eggup-eggpack 0.2.0` to crates.io, and prove from outside this
repository that a consumer of that release resolves the fixed workspace crates.

`eggup-eggpack 0.2.0` is a republication of a crate whose public surface does
not move. The adapter's own behavior, public API, and error surface are
unchanged from `0.1.3`. The release carries two distinct changes relative to
the **published** `0.1.3`:

1. **The M006 digest-reuse change**, which is a content change, not only a
   manifest change. The published `eggup-eggpack 0.1.3` still carries a direct
   `sha2` runtime dependency and computes the archive digest with its own
   inline loop; `0.2.0` ships the adapter delegating to `eggup_core::hash_file`
   with the `sha2` runtime edge removed. The published `0.1.3` manifest and
   `0.1.2` manifest both list `sha2 ^0.10.9` as a normal dependency; `0.2.0`
   ships a runtime dependency list one entry shorter, with an unchanged
   transitive closure.
2. **The exact-pin re-pin** from `=0.1.3` to `=0.2.0` on all three in-workspace
   dependencies. This is the **consumer-facing mechanism**: it is what carries
   the workspace's `0.2.0` fixes to eggpack consumers.

The content that change (1) brings in is recorded in
`plans/closure/eggpack-manifest-interoperability/006-status.md` (Eggpack
Manifest Interoperability M006) and is verified against the source at the
baseline:

- The adapter delegates to `eggup_core::hash_file` (`crates/eggup-eggpack/src/lib.rs:522-523`,
  implementation at `crates/eggup-core/src/integrity.rs:72`) instead of running its
  own inline SHA-256 loop. The workspace holds exactly one file-digest
  implementation.
- The symlink rejection and the exact-size check still run and are still ordered
  **before** the digest computation (`crates/eggup-eggpack/src/lib.rs:513-517`
  above `:522`).
- The direct `sha2` runtime dependency, which existed only for that one call
  site, is removed. `sha2` remains a **dev-dependency** only
  (`crates/eggup-eggpack/Cargo.toml:19-23` runtime dependencies, `:31` dev-dependencies).
- The digest value, the error variant, and the error payload are unchanged:
  open failure, read failure, and digest mismatch all still report
  `AdapterError::InvalidAcquiredFile(artifact_name)`
  (`crates/eggup-eggpack/src/lib.rs:522-523`, `:525`).

The version is `0.2.0`, not `0.1.4`. The workspace is version-lockstep
(`version.workspace = true` in all eight crates), and `eggup-acquisition`'s next
release is source-breaking, so the seam had to move to `0.2.0` and the lockstep
workspace carried every crate with it. A `0.1.4` adapter release cannot express a
`0.2.0` dependency under that workspace discipline. Independently, the adapter
itself is not unchanged relative to the published `0.1.3`: its runtime
dependency list loses the direct `sha2` edge, so `0.2.0` is not a
`0.1.4`-style patch of the published release.

The consumer-visible consequence of the re-pin:

- `eggup-eggpack 0.1.3` exact-pinned `=0.1.3` on `eggup-core`,
  `eggup-archive`, and `eggup-acquisition`. A consumer of `0.1.3` therefore
  resolved all three at `0.1.3` and inherited their defects.
- `eggup-eggpack 0.2.0` re-pins all three at `=0.2.0`. A consumer of `0.2.0`
  resolves the fixed `eggup-core 0.2.0` — the release that carries the
  unconditional execution deadline (M014) and the bounded-stdin surface (M017)
  — the fixed `eggup-archive 0.2.0` — the release whose `cleanup()` success path
  is reachable — and the retyped seam `eggup-acquisition 0.2.0`.

**One consumer-visible source break arrives through this publication, and it
arrives through the pin rather than through the adapter's own API.** The seam's
`0.2.0` is source-breaking: `AcquisitionError::Io(String)` is replaced by
`AcquisitionError::Io { operation, source }`
(`crates/eggup-acquisition/src/lib.rs:283-288`), and `Clone`, `PartialEq`, and
`Eq` are removed from `AcquisitionError`, which now derives only `Debug`
(`crates/eggup-acquisition/src/lib.rs:251-256`, `:256`). A consumer of
`eggup-eggpack 0.2.0` inherits that break transitively, because the adapter
re-exposes the seam's request and limits types through `PlannedAcquisition`
(`crates/eggup-eggpack/src/lib.rs:310-323`). This break is stated plainly here
so that no consumer adopts `0.2.0` believing the adapter release is
source-compatible with `0.1.3`.

Registry state at the time this plan was written, read from the crates.io
public API: `eggup-eggpack` holds `0.1.2` and `0.1.3`, neither yanked, both
published by `dbowm91`. `eggup-eggpack 0.2.0` is absent. `eggup-core` holds
`0.1.0`–`0.1.3`, `eggup-archive` holds `0.1.2` and `0.1.3`, and
`eggup-acquisition` holds `0.1.0`–`0.1.3`. None of the three pinned
dependencies is at `0.2.0` yet, which is expected at this point in the train.

## 2. Publication boundary

### 2.1 In scope

Publish exactly one crate: `eggup-eggpack` at version `0.2.0`, from one source
commit, together with the `v0.2.0` tag and the GitHub release that the
workspace's release conventions define.

### 2.2 Out of scope

- Any source change under `crates/eggup-eggpack/`. This milestone publishes; it
  does not repair.
- Any version change. `0.2.0` is already set at the baseline through
  `[workspace.package] version = "0.2.0"` (`Cargo.toml:15`) and
  `version.workspace = true` (`crates/eggup-eggpack/Cargo.toml:3`).
- The other six crates in the train. Their publications are separate milestones
  with their own plans and closure records.
- `eggpack-manifest`. It is an external crate, pinned `=0.1.0`, and it is not
  republished here. Its version does not change.
- Any tag other than the new `v0.2.0`. `v0.1.1`, `v0.1.2`, and `v0.1.3` exist
  and are not moved.
- Any edit to `plans/registry.md`, the subsystem roadmap, the root `CHANGELOG.md`,
  `architecture/**`, `docs/**`, `.github/**`, `scripts/**`, or any crate
  `CHANGELOG.md`. Those files are owned by other actors. Where this milestone
  observes a defect in one of them, it records the observation and hands it to
  the owner; it does not edit it.

### 2.3 Public API boundary

The adapter's own public API is **unchanged** from `0.1.3`. The published
surface is:

- Types: `AdapterError`, `ArtifactRequirement`, `ArchiveMemberRequirement`,
  `ManifestProjection`, `PlannedAcquisition`.
- Constant: `MAX_MANIFEST_BYTES`.
- Free functions: `project`, `project_json`, `install_ids`,
  `archive_format_for_name`, `validate_acquired_archive`, `archive_plan_for`,
  `core_plan_for_archive_with_destinations`, `core_plan_for_archive`,
  `bind_archive_members`.
- Inherent methods on `ManifestProjection`: `default_destinations`,
  `bind_requests`, `materialize_artifact_set_with_destinations`,
  `materialize_artifact_set`.
- Public fields on `ArtifactRequirement` (`artifact_name`, `exact_size`,
  `sha256`, `member_id`, `destination`), `ArchiveMemberRequirement` (`source`,
  `install`, `exact_size`, `sha256`), the `ManifestProjection` variants
  (`Installable`, `Archive`) and their fields, and `PlannedAcquisition`
  (`artifact_name`, `request`, `limits`, `exact_size`, `sha256`, `member_id`).

`AdapterError` is `#[non_exhaustive]` and derives `Debug` only
(`crates/eggup-eggpack/src/lib.rs:19-21`); its eight variants are unchanged.

### 2.4 Ownership boundary

The adapter still stops at extraction-required evidence. It selects no release,
verifies no authenticity, and authorizes no filesystem destination. That is
ADR-0004's producer/consumer boundary, and it is unchanged by this publication.
The adapter projects a manifest into requirements; it does not decide which
release a consumer should take, it does not establish who vouched for a
digest — SHA-256 remains integrity evidence only, per ADR-0003 — and it does not
choose where bytes are installed. A consumer that adopts `eggup-eggpack 0.2.0`
inherits those constraints unchanged.

### 2.5 Dependency surface

| Dependency | Requirement | Role |
|---|---|---|
| `eggup-core` | `=0.2.0` | `hash_file`, member identity, plan construction. |
| `eggup-archive` | `=0.2.0` | Archive format, archive plan, bound members. |
| `eggup-acquisition` | `=0.2.0` | Request and limit types re-exposed by the adapter. |
| `eggpack-manifest` | `=0.1.0` | External manifest schema crate. Not republished here. |

The three in-workspace dependencies are **exact pins** and are declared with
both a registry version and an in-tree path
(`crates/eggup-eggpack/Cargo.toml:20-22`). The path is a development
convenience; the published package must carry only the registry requirement, and
`cargo package` rewrites the path dependency into a registry-only requirement.
That rewrite is what the registry-only proof in §4 exists to check.

Dev-dependencies `serde`, `serde_json`, `flate2`, `tar`, `zip`, and `sha2`
(`crates/eggup-eggpack/Cargo.toml:25-31`) are stripped at packaging and are not
part of the published dependency surface.

### 2.6 Package contents

`crates/eggup-eggpack/Cargo.toml` declares no `exclude` key. The package
therefore ships `src/`, `README.md`, `CHANGELOG.md`, the normalized
`Cargo.toml`, and the `tests/` tree — `archive_handoff.rs`,
`caller_destinations.rs`, `digest_reuse.rs`, `interoperability.rs`, and
`tests/fixtures/` with its JSON manifests. No packaging claim in the closure
record may state that the integration tests or the fixtures are excluded from
the package.

## 3. Pre-publication gates

Every gate below must hold before the upload. Each gate names the §7 command
or check that proves it.

**G1 — Registry preflight.** `eggup-eggpack 0.2.0` is absent from crates.io.
`0.1.2` and `0.1.3` are present and neither is yanked. No existing version is
re-published and no version is yanked by this milestone.

**G2 — Publish authority.** The crates.io API token in use can publish to
`eggup-eggpack`. `dbowm91` published `0.1.2` and `0.1.3`; the same identity
must hold the authority for `0.2.0`.

**G3 — Version identity.** The publication source commit carries
`[workspace.package] version = "0.2.0"` and the adapter inherits it through
`version.workspace = true`. At baseline `b75f73b` the bump is an **uncommitted
working-tree change** (`git diff -- Cargo.toml` shows `0.1.3` → `0.2.0`), so
the publication source commit must carry it. Do not publish from a worktree
whose version bump is not committed.

**G4 — Exact-pin identity.** The packaged manifest carries
`eggup-core = "=0.2.0"`, `eggup-archive = "=0.2.0"`,
`eggup-acquisition = "=0.2.0"`, and `eggpack-manifest = "=0.1.0"`, with no
`path`, `git`, or `patch` source for any of them.

**G5 — Dependency-order gate.** `eggup-core 0.2.0`, `eggup-archive 0.2.0`, and
`eggup-acquisition 0.2.0` are all registry-visible. Until this gate holds,
`cargo package -p eggup-eggpack --locked` cannot resolve the three exact pins
from the registry and fails. That failure is the expected, correct expression of
the dependency-order constraint; it is not a defect and must not be worked
around with a path, git, or patch override. Fixture P (§4.1) proves this gate
independently of the adapter, because the train's front is published before
this milestone starts.

**G6 — Public API surface.** The items enumerated in §2.3 are present and
unchanged in the packaged source, and `AdapterError`'s variant set is
unchanged.

**G7 — Dependency surface.** The runtime dependency set is exactly the four
entries of §2.5. `sha2` is absent from `[dependencies]` and present in
`[dev-dependencies]`. The packaged manifest strips the dev-dependencies.

**G8 — Package contents.** The package file list is recorded and matches §2.6.
It is checked against the actual `.crate` payload, not against an expectation.

**G9 — Digest-reuse content.** `validate_acquired_archive` calls
`eggup_core::hash_file`; the symlink and exact-size guards precede the digest
call in the source; and open failure, read failure, and digest mismatch all
map to `AdapterError::InvalidAcquiredFile`.

**G10 — Hosted matrix.** The hosted CI matrix is green on the publication
source commit: `stable`, `msrv`, `macos`, and `windows-check`. The
`service-systemd` lane belongs to the `eggup-service` milestone and is not a
gate for this crate.

**G11 — Changelog truthfulness.** `crates/eggup-eggpack/CHANGELOG.md` currently
asserts that `0.2.0` is "Published to crates.io" and that it is "the published
baseline on crates.io", and it cites
`plans/closure/eggpack-manifest-interoperability/007-status.md` as the
publication record. At the time this plan was written, none of that has
happened. The wording may stand only after the upload is accepted; if the
upload is not performed or is rejected, the section must be corrected to record
the unpublished state. This milestone does not edit that file — it records the
requirement here and hands the prose to the changelog owner before the upload.

**G12 — No unclassified source delta.** No file under
`crates/eggup-eggpack/src/**` differs from the qualified baseline. The only
adapter change in the `0.1.3` → `0.2.0` train is the manifest re-pin.

**G13 — Maintainer authorization.** The upload happens only on an explicit
maintainer authorization, naming `eggup-eggpack 0.2.0`, and only after every
pre-upload gate holds — including the pre-upload registry-only fixture P
(§4.1), which is a hard stop condition. `cargo publish --dry-run` is not an
upload and does not need this authorization; the real upload does. Fixture R
(§4.2) runs after the upload is accepted and gates **closure**, not the
upload, because `eggup-eggpack 0.2.0` cannot be resolved from the registry
before it exists.

**G14 — Train completion.** The six preceding crates are published and
registry-visible at `0.2.0`, and the `v0.2.0` tag and release cover the whole
train rather than only this crate.

## 4. Registry-only proof

The proof uses **fresh fixture crates created outside this repository**, in a
temporary directory that is not inside the workspace, with no `path`, `git`, or
`patch` override anywhere in the manifest, the workspace, or
`.cargo/config.toml`. A fixture that resolves from the local workspace proves
nothing and must be discarded and rebuilt.

The proof has two phases, because the crate under proof is published last. The
train's front — `eggup-core 0.2.0`, `eggup-archive 0.2.0`, and
`eggup-acquisition 0.2.0` — is on the registry before this milestone starts, so
Fixture P can run before the upload. `eggup-eggpack 0.2.0` cannot be resolved
from the registry until the upload is accepted, so Fixture R runs immediately
after it.

### 4.1 Fixture P — pre-upload, blocking

Fixture P is the pre-upload registry-only gate. It fails the milestone before
the upload if it fails.

It declares:

```toml
[dependencies]
eggup-core = "=0.2.0"
eggup-archive = "=0.2.0"
eggup-acquisition = "=0.2.0"
eggup-eggpack = "=0.1.3"
```

plus the registry crates it needs to author its own fixtures — `tar`, `zip`,
`flate2`, `sha2` — all resolved from the registry.

The `eggup-eggpack = "=0.1.3"` requirement is deliberate: it pins the
**currently published** adapter, so the fixture proves the registry state that
exists before the upload.

Assertions:

1. **The train's front resolves at `0.2.0`.** The tree contains exactly one
   `eggup-core` node at `0.2.0`, exactly one `eggup-archive` node at `0.2.0`,
   and exactly one `eggup-acquisition` node at `0.2.0`, each with source
   `registry+https://github.com/rust-lang/crates.io-index` and a recorded
   checksum in the generated `Cargo.lock`. No entry in the lockfile carries a
   `path`, `git`, or `patch` source, and no `[patch]` section exists.
2. **The published `0.1.3` carries the defect this milestone removes.** The
   published `eggup-eggpack 0.1.3` resolves `eggup-core`, `eggup-archive`, and
   `eggup-acquisition` at `0.1.3`, and its published manifest carries a direct
   `sha2` runtime dependency. Fixture P asserts the `0.1.3` resolution
   independently of this plan's registry reading, so the re-pin's necessity is
   demonstrated rather than asserted.
3. **The adapter's public API is unchanged.** The fixture compiles against the
   published `eggup-eggpack 0.1.3` using exactly the items enumerated in §2.3,
   and exercises the manifest projection and the
   `archive_plan_for`/`bind_archive_members` round trip against it. A consumer
   moving from `0.1.3` to `0.2.0` therefore faces no adapter-level API change.

### 4.2 Fixture R — post-upload, closure gate

Fixture R is the proof the milestone exists to produce. It depends on the
published adapter and on the three workspace crates it re-pins:

```toml
[dependencies]
eggup-eggpack = "=0.2.0"
eggup-acquisition = "=0.2.0"
eggup-archive = "=0.2.0"
eggup-core = "=0.2.0"
```

plus whatever registry crates it needs to author its own fixtures — `tar`,
`zip`, `flate2`, `sha2` — all resolved from the registry. The direct
`eggup-acquisition`, `eggup-archive`, and `eggup-core` requirements are what
make the transitive resolution **observable**: cargo permits more than one
version of the same crate in one graph, so if the adapter's pin were still
`=0.1.3` the tree would carry a second copy of each dependency rather than
resolving the fixture's `=0.2.0` requirement. The direct requirements are
therefore the witness.

The fixture's dependency on `eggup-acquisition` is not optional decoration:
`bind_requests` takes `eggup_acquisition::AcquisitionRequest` values, so a
consumer that exercises that path must name the seam crate directly.

**Lockfile assertions.** After `cargo generate-lockfile` and
`cargo tree --locked`:

- The tree contains exactly **one** `eggup-core` node, at `0.2.0`.
- The tree contains exactly **one** `eggup-archive` node, at `0.2.0`.
- The tree contains exactly **one** `eggup-acquisition` node, at `0.2.0`.
- Every one of those three nodes has source
  `registry+https://github.com/rust-lang/crates.io-index`, and the generated
  `Cargo.lock` records a `checksum` for each.
- No entry anywhere in the lockfile carries a `path`, `git`, or `patch` source,
  and no `[patch]` section exists.

A second copy of any of the three at `0.1.3` is a failure, not a warning. It
means the re-pin did not reach the registry.

**Behavioral assertions.** The fixture exercises representative consumer use
against the registry-resolved crate:

1. **Manifest projection.** Build a small `ReleaseManifest` document, project
   it with `project_json`, and assert the result is
   `ManifestProjection::Archive` with the exact expected member set, the exact
   expected archive size, and the expected digest bytes. Assert `install_ids`
   and `default_destinations` agree with the manifest.
2. **Archive plan and bound-member round trip.** Author a real archive whose
   member sizes and digests match the manifest, then exercise
   `archive_format_for_name` → `archive_plan_for` → `eggup_archive::extract` →
   `persist` → `into_bound_sources` → `into_members` →
   `eggup_eggpack::bind_archive_members` →
   `core_plan_for_archive_with_destinations`. Assert the bound member count
   equals the declared member count and that each bound member's
   `source_path`/`output_name` matches the declared identity. Assert that a
   projection/bound-member mismatch fails closed rather than binding foreign
   bytes.
3. **Delegated digest.** Call `validate_acquired_archive` on the authored
   archive and assert it succeeds; then tamper with one byte of the archive and
   assert it fails with `AdapterError::InvalidAcquiredFile`. This exercises the
   published `eggup-core 0.2.0`'s `hash_file` **through** the adapter's exact
   pin, from outside the workspace — it proves the fixed `eggup-core` is
   reachable, not merely present in the lockfile.
4. **Seam typing.** Assert the seam's retyped error is observable through the
   adapter's re-exposed seam types, so the fixture compiles against the
   published `eggup-acquisition 0.2.0` and not against a stale `0.1.3`.

### 4.3 Reporting

Record both fixtures' absolute paths, their `Cargo.toml` files, their generated
`Cargo.lock` files, their `cargo tree --locked` outputs, and their test
results. The fixtures are proof evidence for the closure record and are not
added to this repository.

## 5. Tag and GitHub release semantics

- `v0.1.1`, `v0.1.2`, and `v0.1.3` exist and point at `881c95f`, `e8e07eb`,
  and `bd43683` respectively. Do not move, delete, or re-create them.
- Create `v0.2.0` **only after** the upload is accepted and the §4.2
  registry-only proof passes, and point it at the exact publication source
  commit — the same commit whose tree produced the uploaded `.crate`. A tag
  that points at a different commit than the published artifact is a stop
  condition.
- The tag is annotated and names the train, not only this crate: the seven
  publishable crates share one `0.2.0` release.
- The GitHub release for `v0.2.0` covers the whole train and references the
  crates.io release of `eggup-eggpack 0.2.0` together with the six that
  precede it.
- CI never publishes. The upload is a manual, maintainer-authorized step, and
  the tag and release follow the accepted upload, not the other way round.

## 6. Ordered work packages

**W1 — Confirm the train's front is published.** Read the crates.io public API
for `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-curl`,
`eggup-eggfetch`, and `eggup-service`. Record the version list for each. Do
not proceed until `eggup-core 0.2.0`, `eggup-archive 0.2.0`, and
`eggup-acquisition 0.2.0` are visible (G5) and all six are visible (G14).

**W2 — Fix the publication source commit.** Ensure the source commit carries
the workspace version bump to `0.2.0`, the adapter's three re-pins to `=0.2.0`,
and the `Cargo.lock` that matches. All seven crates publish from this one
commit (G3, G4, G12).

**W3 — Run the local gate.** Run the §7 qualification commands that are
runnable locally, in order, and record their exact output. Note explicitly
which hosted lanes replace which local command.

**W4 — Package and inspect.** Run `cargo package -p eggup-eggpack --locked` and
`cargo package -p eggup-eggpack --list --locked`. Inspect the produced
`.crate`: confirm the dependency requirements are registry-only and exactly
`=0.2.0` / `=0.1.0`, confirm the dev-dependencies are stripped, confirm the
file list matches §2.6, and confirm the public items match §2.3.

**W5 — Dry run.** Run `cargo publish -p eggup-eggpack --dry-run --locked` and
record the result. A dry run performs no upload.

**W6 — Build and run the pre-upload registry-only fixture.** Create Fixture P
outside this repository per §4.1, generate its lockfile, assert the tree, and
run its tests. Record everything. A failure here stops the milestone before the
upload.

**W7 — Obtain maintainer authorization.** Present the §7 evidence and the §4.1
fixture result to the maintainer and obtain explicit authorization naming
`eggup-eggpack 0.2.0` (G13).

**W8 — Upload.** Run `cargo publish -p eggup-eggpack --locked`. Capture the
output, the crates.io checksum, and the version metadata.

**W9 — Verify the registry.** Read the crates.io public API for
`eggup-eggpack` and confirm `0.2.0` is present, not yanked, and that `0.1.2`
and `0.1.3` are untouched.

**W10 — Build and run the post-upload registry-only fixture.** Create Fixture R
outside this repository per §4.2, generate its lockfile, assert the tree, and
run its tests. Record everything. A failure here blocks **closure**, not the
upload, because the upload has already been accepted and cannot be undone.

**W11 — Tag and release.** Create the annotated `v0.2.0` tag at the publication
source commit and publish the GitHub release for the train.

**W12 — Reconcile the consumer unblock.** Record that consumers of
`eggup-eggpack 0.2.0` now resolve `eggup-core 0.2.0`,
`eggup-archive 0.2.0`, and `eggup-acquisition 0.2.0`, and that the seam's
source-breaking retyping is inherited transitively. Hand the matching prose to
the owners of `plans/registry.md`, the subsystem roadmap, and the changelogs —
this milestone edits none of them.

## 7. Required verification

Run in this order. The first nine commands are local and perform no upload; the
last one is the upload and requires G13.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
cargo package -p eggup-eggpack --locked
cargo package -p eggup-eggpack --list --locked
cargo publish -p eggup-eggpack --dry-run --locked
```

Then, maintainer-authorized only:

```sh
cargo publish -p eggup-eggpack --locked
```

Registry, tree-state, and content checks that the gates in §3 name, in the
order they are used:

```sh
# G1, G2, G5, G14 — registry state. crates.io's public API rejects a bare
# request without a User-Agent header.
curl -fsSL -H 'User-Agent: eggup-release-check' \
  https://crates.io/api/v1/crates/eggup-eggpack/versions
curl -fsSL -H 'User-Agent: eggup-release-check' \
  https://crates.io/api/v1/crates/eggup-core/versions
curl -fsSL -H 'User-Agent: eggup-release-check' \
  https://crates.io/api/v1/crates/eggup-archive/versions
curl -fsSL -H 'User-Agent: eggup-release-check' \
  https://crates.io/api/v1/crates/eggup-acquisition/versions

# G3, G12 — the publication source commit's tree state.
git diff b75f73b -- Cargo.toml crates/eggup-eggpack/Cargo.toml crates/eggup-eggpack/src
git status --short crates/eggup-eggpack/src

# G9 — the delegated digest and its guard ordering.
cargo test -p eggup-eggpack --locked

# G6, G7, G8 — packaged public API, dependency surface, and file list.
cargo package -p eggup-eggpack --list --locked
```

Sequencing notes that matter:

- The `cargo package -p eggup-eggpack --locked` step is the first command that
  requires G5. It resolves the three exact pins from the registry, so it
  succeeds only after `eggup-core 0.2.0`, `eggup-archive 0.2.0`, and
  `eggup-acquisition 0.2.0` are published. Running it earlier and recording
  the resolution failure is acceptable evidence of the constraint; it is not a
  pass, and the command must be re-run after G5 holds.
- `cargo +1.89.0 …` requires the `1.89.0` toolchain to be installed locally.
  Verify with `rustup toolchain list` before the run. At the time this plan was
  written that toolchain was present on the authoring machine, so the MSRV
  commands are expected to run locally as well as in the hosted `msrv` lane. If
  the toolchain is not installed on the machine that performs the publication,
  do not record a local MSRV result — record that the MSRV lane is covered by
  the hosted `msrv` lane on the publication source commit instead.
- The hosted matrix is the authority for the platform lanes: `stable`, `msrv`,
  `macos`, and `windows-check`, each recorded individually with the hosted run
  id and the qualified commit (G10).
- G11 is proven by reading `crates/eggup-eggpack/CHANGELOG.md` against the
  actual publication state, not by a cargo command: before the upload is
  accepted, the wording that asserts publication must not stand.
- The registry-only fixtures of §4 straddle the upload. Fixture P (§4.1) runs
  **before** the upload, because the train's front — `eggup-core 0.2.0`,
  `eggup-archive 0.2.0`, `eggup-acquisition 0.2.0` — is already on the
  registry; it is a hard stop condition. Fixture R (§4.2) runs **after** the
  upload, because it resolves `eggup-eggpack = "=0.2.0"` from the registry; it
  is a hard gate of closure, and the upload it proves cannot be undone.
- The tag and the GitHub release follow the accepted upload and the Fixture R
  result, not the other way round.
- `./scripts/check-local.sh` and `./scripts/check-docs-truth.sh` remain the
  aggregate gates; the commands above are the publication-scoped subset and do
  not replace them.

## 8. Acceptance criteria

| # | Criterion | Proven by |
|---|---|---|
| AC1 | `eggup-eggpack 0.2.0` is present on crates.io, not yanked, with a recorded checksum and version metadata. | §7 upload, W9 registry read |
| AC2 | `0.1.2` and `0.1.3` are unchanged and not yanked; no version was re-published or yanked. | §3 G1, W9 |
| AC3 | The published package's dependency requirements are exactly `eggup-core = "=0.2.0"`, `eggup-archive = "=0.2.0"`, `eggup-acquisition = "=0.2.0"`, `eggpack-manifest = "=0.1.0"`, with no path, git, or patch source. | §2.5, §3 G4, W4 package inspection |
| AC4 | The published package's public API is exactly the surface of §2.3, with `AdapterError`'s variant set unchanged. | §3 G6, W4 |
| AC5 | The published package's runtime dependency set is the four entries of §2.5; `sha2` is dev-only and absent from the packaged runtime requirements. | §3 G7, W4 |
| AC6 | The package file list is recorded and matches §2.6. | §3 G8, W4 |
| AC7 | `validate_acquired_archive` delegates to `eggup_core::hash_file`, the symlink and exact-size guards precede the digest call, and all three failure modes still report `AdapterError::InvalidAcquiredFile`. | §3 G9, `cargo test -p eggup-eggpack`, W3 |
| AC8 | The hosted matrix (`stable`, `msrv`, `macos`, `windows-check`) is green on the publication source commit, recorded per lane with the run id. | §3 G10, §7 |
| AC9 | The pre-upload registry-only fixture (Fixture P) resolves exactly one `eggup-core`, one `eggup-archive`, and one `eggup-acquisition`, each at `0.2.0` from crates.io, with no path, git, or patch source anywhere in its lockfile. | §4.1, W6 |
| AC10 | Fixture P independently confirms that the published `eggup-eggpack 0.1.3` resolves its three dependencies at `0.1.3` and carries the direct `sha2` runtime dependency, and that the §2.3 adapter API compiles and behaves unchanged against the published `0.1.3`. | §4.1, W6 |
| AC11 | The post-upload registry-only fixture (Fixture R) resolves exactly one `eggup-core`, one `eggup-archive`, and one `eggup-acquisition`, each at `0.2.0` from crates.io, with no path, git, or patch source anywhere in its lockfile. | §4.2, W10 |
| AC12 | Fixture R's projection, archive-plan/bound-member round trip, delegated-digest, and seam-typing assertions all pass. | §4.2, W10 |
| AC13 | `v0.2.0` exists, is annotated, and points at the exact publication source commit; `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved. | §5, W11 |
| AC14 | The GitHub release for `v0.2.0` covers the seven-crate train and references the crates.io release of `eggup-eggpack 0.2.0`. | §5, W11 |
| AC15 | The consumer-unblock reconciliation is recorded, including the transitive `eggup-acquisition 0.2.0` source break and the `AcquisitionError` typing change it carries. | §1, W12 |
| AC16 | The changelog's publication wording is either true because the upload was accepted, or corrected by its owner to record the unpublished state. | §3 G11 |

## 9. Stop conditions

Stop the milestone and report the exact failing condition. Do not improvise a
workaround, and do not publish a partial result.

- Any registry-only fixture assertion in §4 fails, including the exact-one-copy
  resolution of any of the three pinned dependencies. Fixture P (§4.1) failing
  stops the milestone **before** the upload; Fixture R (§4.2) failing stops
  **closure** and is reported as an open finding against the published
  `0.2.0`, because the upload it proves has already been accepted.
- The packaged public API differs materially from §2.3, or the packaged
  dependency requirements differ from §2.5.
- The hosted matrix is not green on the publication source commit.
- `eggup-eggpack 0.2.0` is already present on crates.io at the time of the
  upload.
- The upload would require moving, deleting, or re-creating `v0.1.1`,
  `v0.1.2`, or `v0.1.3`.
- **Any** of `eggup-core 0.2.0`, `eggup-archive 0.2.0`, or
  `eggup-acquisition 0.2.0` is not yet on the registry at `0.2.0`.
- The publication source commit does not carry the workspace version bump to
  `0.2.0`, or the adapter's manifest does not carry the three `=0.2.0` pins.
- Any file under `crates/eggup-eggpack/src/**` differs from the qualified
  baseline.
- The upload would require a path, git, or patch override to resolve.
- Maintainer authorization for the upload is absent or ambiguous.

**After an accepted upload, a corrective requires a new patch version — never
an overwrite.** crates.io does not permit republishing a version, and this
milestone never yanks a published version. A defect found in `0.2.0` after
acceptance is handled by publishing `0.2.1` under a new plan and closure
record, with its own exact-pin re-pin to `=0.2.1` on the three workspace
dependencies.

## 10. Closure evidence required

The closure record for this milestone is
`plans/closure/eggpack-manifest-interoperability/007-status.md`. It must carry:

1. **Release-prep SHA.** The exact publication source commit, and confirmation
   that it is the commit the uploaded artifact was built from and the commit
   `v0.2.0` points at.
2. **Hosted run id and per-lane results.** The hosted run id, the qualified
   commit, and the individual result for `stable`, `msrv`, `macos`, and
   `windows-check`. State which MSRV evidence is local and which is hosted.
3. **Package inventory and dependency tree.** The `cargo package --list`
   output, the packaged `Cargo.toml` as rewritten, and the dependency tree.
   State the four runtime requirements and confirm the dev-dependencies are
   stripped and the `tests/` tree is shipped.
4. **crates.io checksum and metadata.** The checksum, the version metadata, and
   the registry read that shows `0.2.0` present and not yanked, with `0.1.2`
   and `0.1.3` unchanged.
5. **Registry-only fixture evidence.** Both fixtures: each one's absolute path,
   its `Cargo.toml`, its generated `Cargo.lock`, its `cargo tree --locked`
   output, and its test result.
   - Fixture P (§4.1): the tree showing exactly one `eggup-core`, one
     `eggup-archive`, and one `eggup-acquisition` at `0.2.0` from crates.io
     with no path, git, or patch source, and the test result confirming the
     published `0.1.3` resolution at `0.1.3` and the unchanged §2.3 adapter
     API.
   - Fixture R (§4.2): the tree showing the same three exact-one-copy
     resolutions at `0.2.0`, and the test result covering the projection, the
     `archive_plan_for`/`bind_archive_members` round trip, the delegated digest,
     and the seam typing.
6. **Tag and release state.** The `v0.2.0` tag object and its target commit,
   proof that `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved, and the GitHub
   release reference.
7. **Consumer-unblock reconciliation.** The record that consumers of
   `eggup-eggpack 0.2.0` now resolve `eggup-core 0.2.0`, `eggup-archive 0.2.0`,
   and `eggup-acquisition 0.2.0`; the statement that the seam's `0.2.0` is
   source-breaking and that the retyping of `AcquisitionError::Io` and the
   removal of `Clone`/`PartialEq`/`Eq` from `AcquisitionError` are inherited
   transitively through the exact pin; the statement that `0.2.0` also ships
   the M006 digest-reuse change and therefore a runtime dependency list one
   entry shorter than the published `0.1.3`; and the hand-off to the owners of
   `plans/registry.md`, the subsystem roadmap, and the changelogs.
8. **Stop-condition ledger.** Any stop condition encountered, the exact
   failing condition, and what was done about it.
