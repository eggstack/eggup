# Verified Update Core Milestone 016 — eggup-core contract-doc packaging

Status: ready for handoff

Repository baseline: `a811b3634a48c4c045b467cee70d8d9ccc4144c4`

Source roadmap:
- `plans/subsystems/verified-update-core-roadmap.md`

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: capability

## 1. Objective

Ship the three authoritative `eggup-core` contract documents in the published
crate package: `docs/domain.md`, `docs/verification.md`, and
`docs/transaction.md`. Publish them in a new `eggup-core 0.2.1` package after
the package list, dry-run, workspace gate, hosted CI, and registry-only proof
are qualified.

This is a package-content change only. The contract documents' contents, Rust
source, public API, runtime behavior, and dependency graph remain unchanged.
Only `eggup-core` is uploaded by this milestone. The other workspace crates
will declare source version `0.2.1` because they inherit the workspace version;
that does not authorize their publication.

## 2. Why this milestone is ready

- Core M019 published `eggup-core 0.2.0` and its closure explicitly left this
  contract-document gap to M016. M019's package inventory records 23 entries
  and no `docs/` entry.
- Planning C013 Finding G names M016 as the owner and defines the bounded
  remedy: remove `docs/` from the package exclusion, prove all three contract
  files appear in `cargo package --list`, and publish the package containing
  them. The finding is assessed as **medium** because published consumers
  cannot read the authoritative contract from the package they depend on.
- Core M019 and the workspace version-lockstep work are closed. The core has no
  in-workspace runtime dependency, so its package can be published on its own
  before any other crate at the new workspace version.
- At this plan's baseline, `cargo search eggup-core --limit 1` reports `0.2.0`.
  The implementation must repeat the registry preflight. `0.2.1` is the
  planned next patch only if that version remains unused.
- The milestone is ready for implementation handoff. A separate, recorded
  maintainer authorization is still required before the irreversible
  `cargo publish` step; authoring this plan does not grant that authorization.

## 3. Current implementation evidence

- `crates/eggup-core/Cargo.toml` declares
  `exclude = ["docs/", "tests/", "benches/"]`. The exclusion is the direct
  reason the contract documents are absent from the package.
- `crates/eggup-core/docs/` contains the three normative contract documents
  named in §1. They are already authoritative; this milestone does not edit
  their content.
- `plans/closure/verified-update-core/019-status.md` records the published
  `eggup-core 0.2.0` package inventory and names M016 as the open follow-on.
- The root `[workspace.package].version` is `0.2.0`, inherited by every crate.
  `crates/eggup-eggpack/Cargo.toml` exact-pins its three in-workspace
  dependencies at `=0.2.0`; those local pins must stay aligned if the shared
  source version advances.
- `eggup-core`'s runtime dependency surface is `sha2` plus the Windows-only
  `self-replace` dependency. Packaging documentation must not add a dependency
  or change either target's graph.
- The current published baseline is `eggup-core 0.2.0`. The repository's
  changelog, `docs/releases.md`, and `architecture/tooling-governance.md` own
  the publication history and must distinguish the new package from the
  immutable `0.2.0` package.

## 4. Invariants that must not regress

- The shipped files are byte-for-byte the existing authoritative documents;
  the plan changes which files enter the package, not what their contracts say.
- `eggup-core` remains policy-neutral, transport-neutral, and free of service
  manager or Eggpack dependencies. SHA-256 remains integrity evidence only;
  no authenticity or signature claim is introduced.
- No file under `crates/eggup-core/src/**` changes. No public type, signature,
  enum variant, error classification, or runtime behavior changes.
- `tests/` and `benches/` remain excluded from the published package. Only the
  `docs/` exclusion is removed.
- The shared workspace version and all local exact path requirements remain
  internally consistent. The published `eggup-eggpack 0.2.0` package remains
  unchanged and continues to pin `eggup-core = "=0.2.0"`.
- Publication is manual, versioned, and immutable. No existing package version
  or tag is overwritten, yanked, or moved; CI never publishes.

## 5. Scope

### In scope

- Advance `[workspace.package].version` from `0.2.0` to `0.2.1` and refresh
  `Cargo.lock` without unrelated dependency updates.
- Remove only `"docs/"` from `eggup-core`'s `exclude` list. Keep `tests/` and
  `benches/` excluded.
- Retarget the three local exact `eggup-eggpack` path pins to `=0.2.1` so the
  workspace remains resolvable at its shared source version. Do not publish
  `eggup-eggpack` as part of this milestone.
- Add accurate `Unreleased`/`0.2.1` release notes for the package-content
  change, with no-publication/no-migration language before publication.
- Qualify the package inventory, package build, publish dry-run, full local
  gate, hosted CI matrix, and a direct registry-only consumer of
  `eggup-core = "=0.2.1"`.
- Publish only `eggup-core 0.2.1`, read back the registry artifact and
  checksum, create the new `v0.2.1` tag and release after the registry proof,
  and close Finding 14 with the exact evidence.
- Update the current version/publication claims and the M016 status in the
  owning documentation and planning control surfaces.

### Explicitly out of scope

- Any edit to the content of `crates/eggup-core/docs/{domain,verification,transaction}.md`.
- Any Rust source, API, behavior, test-support, dependency, feature, or CI
  change. If package qualification requires one, stop and plan it separately.
- Publishing any crate other than `eggup-core`; in particular, no
  `eggup-eggpack 0.2.1` republication or exact-pin cascade beyond the local
  source alignment in §6. Consumers of the published `eggup-eggpack 0.2.0`
  remain pinned to core `0.2.0` until a separately authorized adapter release.
- Changing the published contents of `eggup-core 0.2.0`, moving `v0.2.0`,
  yanking any version, or changing the `0.1.x` line.
- Introducing new rules in `docs/`, choosing an authenticity standard, or
  changing any of the long-term contracts.

## 6. Required production changes

1. Set `[workspace.package].version` to `0.2.1`. Every workspace member
   inherits this source version; this milestone uploads only `eggup-core`.
2. In `crates/eggup-core/Cargo.toml`, remove `"docs/"` from `exclude` while
   retaining `"tests/"` and `"benches/"`.
3. In `crates/eggup-eggpack/Cargo.toml`, change the three local exact Eggup
   requirements from `=0.2.0` to `=0.2.1` so their path packages match the
   shared workspace version. Keep `eggpack-manifest = "=0.1.0"` unchanged.
   This is a source-workspace alignment only; the published `eggup-eggpack
   0.2.0` package is immutable and is not rebuilt or uploaded here.
4. Refresh only the affected workspace package entries in `Cargo.lock`.
   Review the complete lockfile diff and stop if unrelated registry dependency
   versions change.
5. Do not change any file under `crates/eggup-core/src/**` or any file under
   `crates/eggup-core/docs/**`.

## 7. Ordered work packages

1. **W1 — Registry and tag preflight.** Recheck the crates.io versions and
   yanked state for `eggup-core`; confirm `0.2.1` is unused. Confirm `v0.2.1`
   and the GitHub Release do not already exist. Record the check date and
   results. If the version or tag exists, stop and revise this plan's target;
   never overwrite it.
2. **W2 — Workspace version alignment.** Change the shared source version to
   `0.2.1`, retarget `eggup-eggpack`'s three local exact pins, refresh the
   lockfile, and inspect the dependency graph. Do not publish a dependent crate
   whose exact registry dependencies are not available at `0.2.1`.
3. **W3 — Package selection.** Remove only the core `docs/` exclusion. Run the
   package-list command and require exactly the three named contract docs to
   appear, with tests and benches still absent.
4. **W4 — Release notes and current claims.** Add the Core M016 note to the
   core changelog and root changelog. Record the unpublished `eggup-eggpack`
   local pin alignment in its `Unreleased` section without claiming that
   `eggup-eggpack 0.2.0` has changed. Update current published/source-version
   claims in `docs/crates.md`, `docs/releases.md`, `architecture/overview.md`,
   `architecture/tooling-governance.md`, and `README.md` where applicable.
   Keep dependency examples at `0.2.0` if they remain intentionally pinned and
   working; do not edit compiled examples merely to make their version string
   look newest. Remove the resolved M016 row from the live drift table only
   after the `0.2.1` package is verified. Update the registry and Core roadmap
   after qualification and closure, not before evidence exists.
5. **W5 — Local qualification.** Run `./scripts/check-local.sh`, inspect
   `cargo tree --workspace --locked`, and review the lockfile and package
   manifest diffs. Require no runtime source or dependency-edge changes.
6. **W6 — Package qualification.** Run `cargo package -p eggup-core --list
   --locked`, `cargo package -p eggup-core --locked`, and
   `cargo publish --dry-run -p eggup-core --locked`. Save the final package
   file list and checksum. Confirm the three contract docs are present, tests
   and benches are absent, and the normalized package has no path/git/patch
   dependency.
7. **W7 — Hosted qualification.** Run all five current CI lanes on the exact
   publication source commit: `stable`, `msrv`, `service-systemd`, `macos`,
   and `windows-check`. All must be green before upload. Record lane results
   separately; do not infer platform results.
8. **W8 — Publication authorization and upload.** Confirm the working tree is
   clean, the exact source SHA is fixed, all gates are green, and a maintainer
   has recorded explicit authorization for this upload. Then run
   `cargo publish -p eggup-core --locked`. No other upload is part of M016.
9. **W9 — Registry readback, consumer proof, and post-publication CI.** Read
   the crates.io metadata and download the published `.crate`. Confirm version
   `0.2.1`, checksum, package contents, and `.cargo_vcs_info.json` source SHA.
   Build a fresh registry-only fixture that depends on
   `eggup-core = "=0.2.1"`; confirm its lockfile uses the registry source.
   Inspect the downloaded archive to prove all three contract files are
   present. Then run all five hosted CI lanes again on the same source commit;
   the strict packaged-service step must resolve the newly published Core.
   Require that post-publication run to be green before creating the tag or
   closing M016.
10. **W10 — Tag and release.** Only after the registry readback and consumer
    proof pass, create `v0.2.1` at the exact publication source commit and
    create the GitHub Release. Do not move any existing tag. State that only
    `eggup-core 0.2.1` is published by M016; later `0.2.1` crate releases, if
    any, need their own qualification and authorization.
11. **W11 — Closure and reconciliation.** Write
    `plans/closure/verified-update-core/016-status.md` with the full evidence
    matrix, exact commands/results, registry metadata, authorization, and
    unresolved findings. Then reconcile `plans/registry.md`, the Core roadmap,
    changelogs, release-status docs, and the live drift table.

## 8. Failure, cancellation, restart, and contention semantics

This milestone does not mutate an installed application, create a candidate
process, or change runtime transaction state. Its external side effect is one
crates.io upload, which is irreversible once accepted.

- A failed registry preflight, package inventory, package build, dry-run, local
  gate, hosted lane, or source-delta review stops the sequence before upload.
- If `cargo publish` returns an ambiguous result, query crates.io before any
  retry. If `0.2.1` exists, do not retry or upload it again; continue with
  readback and record the actual result. If it does not exist, preserve the
  command output and re-evaluate the registry before retrying.
- After a successful upload, a failed checksum, archive inventory, or fixture
  check cannot be repaired by overwriting or yanking `0.2.1`. Stop, retain the
  artifact and logs, and record the milestone as conditionally closed or
  blocked with the exact consequence and a separately scoped corrective if
  needed.
- A cancellation before upload leaves only local source changes and can be
  resumed after repeating registry preflight and qualification. A cancellation
  after upload resumes from readback; it never repeats the upload.
- There is no lock, contention, timeout, rollback, or crash-journaling behavior
  to add or claim for this package-only operation.

## 9. Compatibility and migration

- The Rust API, source compatibility, runtime behavior, and dependency graph
  do not change. Direct consumers may continue using `eggup-core 0.2.0` without
  migration; consumers who need the packaged contracts can resolve or pin
  `eggup-core 0.2.1`.
- The addition of three Markdown files changes the published source archive,
  not the compiled library or rustdoc API surface. The `.crate` inventory and
  registry readback are the authoritative proof of that addition.
- The workspace version becomes `0.2.1` for all local members. Only the Core
  package is published. The local `eggup-eggpack` exact pins change to match
  the local path packages, but the published `eggup-eggpack 0.2.0` still pins
  all three dependencies at `=0.2.0`. This milestone does not promise that
  consumers of that published adapter receive Core's new docs transitively.
- Existing `0.2.0` and `0.1.x` artifacts remain immutable. No tag is moved,
  and no downstream source migration is required.

## 10. Required tests

No production-code tests are added because the change does not alter Rust
source. The existing workspace gate remains required. The packaging behavior
is covered by artifact and consumer evidence:

- The local package file list contains `docs/domain.md`,
  `docs/verification.md`, and `docs/transaction.md`, and no `tests/` or
  `benches/` files. The downloaded `.crate` contains the same files under the
  `eggup-core-0.2.1/` archive prefix.
- The downloaded crates.io archive has those same three entries and the
  checksum matches registry metadata.
- A fresh fixture outside the workspace depends only on
  `eggup-core = "=0.2.1"`, builds from crates.io, and has no path, git, or
  patch source. Preserve its manifest, lockfile, and result in the closure
  evidence.
- `cargo doc` and the hosted workspace lanes remain green. These prove the
  source workspace still builds; they do not replace the package-content proof.

## 11. Required verification commands

Run these from a clean publication source commit. The first command is the
ordered local gate; inspect its `cargo tree` output as a human review step.

```sh
./scripts/check-local.sh
cargo tree --workspace --locked
cargo package -p eggup-core --list --locked
cargo package -p eggup-core --locked
cargo publish --dry-run -p eggup-core --locked
```

Repeat the registry preflight immediately before publication:

```sh
cargo search eggup-core --limit 1
curl --fail --silent --show-error --output /tmp/eggup-core-registry.json https://crates.io/api/v1/crates/eggup-core
python3 -m json.tool /tmp/eggup-core-registry.json
```

The preflight output must show that `0.2.0` is the latest published version,
that `0.2.1` is unused, and that no existing version needed for this milestone
is yanked. If not, stop and revise the target.

After publication, run the checked-in registry-only fixture and inspect the
downloaded archive:

```sh
cargo check --manifest-path plans/closure/verified-update-core/016-package-fixture/Cargo.toml --locked
cargo tree --manifest-path plans/closure/verified-update-core/016-package-fixture/Cargo.toml --locked
curl --fail --silent --show-error --output /tmp/eggup-core-registry-after.json https://crates.io/api/v1/crates/eggup-core
python3 -m json.tool /tmp/eggup-core-registry-after.json
curl --fail --location --silent --show-error --output /tmp/eggup-core-0.2.1.crate https://static.crates.io/crates/eggup-core/eggup-core-0.2.1.crate
tar -tzf /tmp/eggup-core-0.2.1.crate > /tmp/eggup-core-0.2.1.contents
rg -n '^eggup-core-0\.2\.1/docs/(domain|verification|transaction)\.md$' /tmp/eggup-core-0.2.1.contents
tar -xOf /tmp/eggup-core-0.2.1.crate eggup-core-0.2.1/.cargo_vcs_info.json
shasum -a 256 /tmp/eggup-core-0.2.1.crate
```

The first `rg` command must print exactly the three contract-document paths.
Record the full output of all five hosted CI lanes before and after publication,
and the exact source commit, in the closure record. The post-publication run is
required so the strict packaged-service step resolves Core `0.2.1`. No local
package or registry command substitutes for a hosted platform lane.

## 12. Documentation updates

- `crates/eggup-core/CHANGELOG.md`: record M016 under `Unreleased` during
  implementation, then under `0.2.1` after publication. State that only the
  package file list changes and there is no API/runtime migration.
- `crates/eggup-eggpack/CHANGELOG.md`: record the local exact-pin alignment in
  `Unreleased`, with the disclaimer that published `eggup-eggpack 0.2.0` is
  unchanged and M016 does not publish it.
- Root `CHANGELOG.md`: aggregate the Core 0.2.1 package-content change without
  replacing the per-crate release notes.
- `docs/crates.md` and `docs/releases.md`: distinguish workspace source
  version `0.2.1` from registry state (Core `0.2.1`; the other published
  crates remain at `0.2.0` until their own releases). Explain that Core 0.2.0
  remains immutable and does not contain the three files.
- `architecture/overview.md` and `architecture/tooling-governance.md` §8:
  update current workspace/published-version claims, add the Core 0.2.1
  publication, and preserve the historical 0.2.0 train record. Remove the
  live Core M016 drift row only after the published archive proves the fix;
  retain any necessary note that older immutable package versions lack the
  files.
- `README.md`: update only any current workspace-version claim. Keep an
  intentionally pinned, compiling dependency example at `0.2.0` if it remains
  a valid example; do not silently change a snippet that must match a checked-in
  example.
- `plans/registry.md` and
  `plans/subsystems/verified-update-core-roadmap.md`: during this authoring
  change, list M016 as ready for handoff with this plan. At closure, replace
  that status with the closure evidence and record the publication result.
- `AGENTS.md` and `.opencode/skills/`: no change is required unless this work
  changes a rule they index; M016 introduces no new rule.

## 13. Acceptance criteria

1. The source workspace version is `0.2.1`; `eggup-eggpack`'s local exact pins
   match the workspace package versions; unrelated lockfile resolutions are
   unchanged.
2. `cargo package -p eggup-core --list --locked` lists all three contract
   documents and excludes tests and benches.
3. The normalized `eggup-core` package adds no dependency, path, git, or patch
   edge and contains no source delta under `crates/eggup-core/src/**`.
4. `./scripts/check-local.sh`, package build, and publish dry-run pass on the
   publication source commit, with the dependency tree reviewed.
5. All five hosted CI lanes pass on that exact commit both before publication
   and after registry visibility; the latter run proves the strict
   packaged-service step resolves Core `0.2.1`.
6. A maintainer's explicit upload authorization is recorded before
   `cargo publish -p eggup-core --locked` runs.
7. Only `eggup-core 0.2.1` is published by this milestone. Registry metadata,
   checksum, publisher, source SHA, tag, and release are recorded and agree.
8. The published `.crate` and registry-only fixture prove that a direct
   `eggup-core 0.2.1` consumer can obtain the package containing all three
   contract docs from the registry.
9. No wording claims that `eggup-eggpack 0.2.0` or `eggup-core 0.2.0` gained
   these files. Older package versions and tags remain untouched.
10. Finding 14 is closed in the registry with the residual limitation for
    immutable older versions stated accurately; roadmap and registry statuses
    agree with the closure record.

## 14. Stop conditions

- The exact baseline or relevant package manifests changed before work begins;
  refresh the baseline and re-review the version/pin plan first.
- Crates.io already contains `eggup-core 0.2.1`, the expected prior versions
  are yanked, or `v0.2.1` already exists. Do not overwrite or move anything;
  select the next unused patch only after updating the plan and registry.
- Cargo cannot resolve the workspace after the shared version and exact-pin
  alignment, or the lockfile diff changes unrelated dependencies.
- The package list does not contain exactly the three contract docs, includes
  tests/benches, or `cargo package` reports a dependency/source problem.
- Any runtime source, public API, dependency graph, or contract-document
  content changes. Stop and register a separate corrective rather than
  expanding M016.
- The local gate or any required hosted lane fails, or the registry-only proof
  resolves a path/git source or lacks any contract file.
- Explicit maintainer upload authorization is not recorded. Qualification
  alone does not authorize publication.
- Registry readback is inconsistent after upload. Do not retry, overwrite, or
  yank; preserve evidence and record the actual state.

## 15. Closure evidence required

The closure record must include:

- A status of `closed`, `conditionally closed`, `corrective pass required`, or
  `blocked`, plus the plan, roadmap, exact reviewed baseline, and source commit.
- A requirement-to-evidence matrix for every acceptance criterion.
- The before/after `Cargo.toml` and `Cargo.lock` diffs; the full package file
  list; and evidence that `src/**`, dependencies, and API surface are unchanged.
- Exact results for `./scripts/check-local.sh`, package-list, package,
  publish-dry-run, `cargo tree`, and all five hosted lanes before and after
  publication. Mark unrun lanes `not run`; do not infer them.
- The registry-only fixture manifest, lockfile, `cargo tree`, and build result,
  plus the downloaded `.crate` inventory and full SHA-256 checksum.
- The crates.io version/checksum/source SHA/publisher readback and the explicit
  maintainer authorization record preceding upload.
- Tag and GitHub Release evidence; confirmation that older versions and tags
  were not changed and no other crate was published.
- Compatibility, security, documentation, dependency-boundary, and
  failure/recovery reviews. State explicitly that no runtime or filesystem
  mutation path changed.
- Finding 14's severity and closure disposition, including the fact that
  published `eggup-core 0.2.0` and `eggup-eggpack 0.2.0` remain immutable and
  do not gain the M016 package contents.
- Updated registry and roadmap status rows, plus documentation-truth evidence.

## 16. Handoff notes

- The issue is an artifact-content gap, not missing or inaccurate contract
  text. Keep this milestone bounded to including the existing files and
  publishing the corrected Core package.
- Treat the `0.2.1` target as provisional until W1 is repeated at implementation
  and immediately before publication.
- Only Core is published. Because `eggup-eggpack 0.2.0` exact-pins Core
  `=0.2.0`, this milestone does not deliver the docs through that adapter;
  any such follow-on needs its own adapter publication plan and authorization.
- This plan authorizes planning and provides the implementation handoff. It
  does not itself authorize the crates.io upload; obtain and record that
  authorization after all pre-publication evidence is reviewable.
