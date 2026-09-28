# Verified Update Core Milestone 009 — eggup-core / eggup-archive 0.1.2 Publication

Status: implemented; closed by `plans/closure/verified-update-core/009-status.md` (pair published 2026-09-28, smoke 3/3, `v0.1.2` + GitHub Release `0.1.2`)

Repository baseline: `71eb9bdf44cb739adc3701b00310f58430fa7bd1`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Hard dependencies:

- Verified Update Core M008 closed with clean package evidence under M008a.
- Eggpack Interop M002a closed with hosted Stable/MSRV/macOS/Windows qualification.
- current baseline CI run `36478308950` green.

Operational dependency:

- explicit maintainer authorization and valid crates.io credentials with authority to publish `eggup-core` and claim/publish `eggup-archive`.

Primary class: infrastructure / release operations

## 1. Objective

Publish the already-qualified 0.1.2 consumer package pair to crates.io in the established manual release model:

1. `eggup-core 0.1.2`;
2. verify registry acceptance/visibility;
3. `eggup-archive 0.1.2`;
4. verify the pair from a clean registry-only external consumer;
5. tag the exact publication source commit as `v0.1.2` and create the matching GitHub release;
6. reconcile Eggup/Egress planning so Egress Delivery M003 / Consumer M006 becomes executable.

M009 is an explicitly maintainer-controlled irreversible publication pass. It does not introduce automatic publishing or release CI.

## 2. Readiness and dependencies

M009 is dependency-ready from the repository side.

Evidence:

- workspace version is `0.1.2`;
- `eggup-core` and `eggup-archive` manifests are publication-ready;
- M008a ran exact clean-tree `cargo package` and `cargo publish --dry-run` for both crates without `--allow-dirty`;
- the external packaged-crate fixture passed 3/3;
- M002a restored Stable clippy and added direct Windows `eggup-eggpack` runtime execution;
- final current-head run `36478308950` is green.

The remaining dependency is operational: crates.io credentials and explicit maintainer authorization. The plan MUST NOT embed, print, commit, or log a registry token.

## 3. Current evidence

Current qualified package boundary:

- `eggup-core 0.1.2`
  - package: 19 files;
  - runtime dependency: `sha2` only;
  - post-M001d `BoundSources` + `InstallPlan::prepare_with_bound_sources`;
  - no transport/archive/service dependency.
- `eggup-archive 0.1.2`
  - package: 7 files;
  - bounded tar.gz/zip extraction;
  - object-bound member handoff;
  - no runtime dependency on `eggup-core`;
  - dev-only path dependency on `eggup-core`, already proven acceptable by clean `publish --dry-run`.

Prior release convention:

- `v0.1.1` exists as a Git tag and GitHub Release.
- 0.1.1 release notes explicitly identified which crates were published.
- publication is manual and ordered; no release workflow owns crates.io publication.

Important 0.1.2 distinction:

- the workspace version is 0.1.2;
- **only `eggup-core` and `eggup-archive` are authorized for crates.io publication in M009**;
- `eggup-acquisition`, `eggup-eggfetch`, `eggup-service`, `eggup-curl`, and unpublished leaf/internal crates MUST NOT be published merely because they inherit workspace version 0.1.2.

## 4. Invariants that must not regress

- publication remains a deliberate maintainer action, never an automatic CI side effect;
- publish exactly the package artifacts qualified from the release-prep commit;
- no `--allow-dirty` on real publish commands;
- no version overwrite/reuse assumption: a crates.io version is treated as immutable once accepted;
- core publishes before archive;
- archive name/ownership availability is proven before the first irreversible core upload;
- no unrelated 0.1.2 workspace crate is published;
- no package metadata claims authenticity/signature support;
- Rust 1.89 support remains documented;
- tag and GitHub release must identify the exact source commit used for both uploads;
- release notes must distinguish workspace source version from the crates actually published;
- Egress is not marked ready until both registry packages resolve from crates.io without local/git/path overrides.

## 5. Scope and non-scope

### In scope

- exact crates.io name/version/ownership preflight;
- release-changelog curation for the 0.1.2 published subset;
- one final release-prep commit;
- fresh CI/package/dry-run verification after changelog edits;
- manual `cargo publish` for core then archive;
- crates.io propagation checks;
- registry-only external consumer smoke;
- `v0.1.2` tag + GitHub Release from the exact publication commit;
- Eggup closure/registry/roadmap reconciliation;
- Egress planning status reconciliation after both packages are visible.

### Explicitly out of scope

- publishing acquisition, eggfetch, service, curl, eggup-eggpack, transport-footprint, or any other crate;
- changing runtime APIs;
- changing workspace version beyond 0.1.2;
- adding automated publication CI;
- publishing `eggup-eggpack`;
- changing Eggpack producer contracts;
- implementing Egress Delivery M003 itself;
- Gregg migration;
- signing/authenticity-system design.

## 6. Required release changes and publication procedure

### 6.1 Registry/name/credential preflight before any irreversible upload

Before changing publication state:

- confirm `eggup-core 0.1.2` does not already exist on crates.io;
- confirm `eggup-archive 0.1.2` does not already exist;
- because `eggup-archive` is a first publication, confirm the exact crate name is available or already controlled by the maintainer;
- confirm the publishing credential can publish `eggup-core`;
- confirm no token appears in shell output, committed config, command history captured into closure logs, or CI;
- confirm `main` is clean and current head is the expected release-prep ancestor.

If archive name availability/ownership is not clean, STOP **before** publishing core.

### 6.2 Changelog release cut

Prepare the source commit that the published packages and tag will identify.

Root `CHANGELOG.md`:

- create a `## 0.1.2` section using the actual publication date;
- move only changes represented by the crates published in M009 into that section;
- keep unrelated acquisition/service/curl/eggpack-adapter unpublished entries under `## Unreleased`;
- explicitly state the 0.1.2 crates.io publication set is `eggup-core` + `eggup-archive`.

`crates/eggup-archive/CHANGELOG.md`:

- cut `Unreleased` archive changes into `0.1.2` with the publication date;
- remove/replace statements saying this release has not been published;
- preserve the final M001b/M001c/M001d safety semantics and M008 package qualification accurately.

Do not rewrite historical closure evidence.

Commit these release-note changes before real publication. This commit becomes the candidate tag target and publication source.

### 6.3 Requalify the exact release-prep commit

Because the archive changelog is part of the packaged crate, rerun package evidence after the changelog cut:

~~~bash
test -z "$(git status --porcelain)"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked

cargo package -p eggup-core --locked
cargo publish -p eggup-core --dry-run --locked
cargo package -p eggup-archive --locked
cargo publish -p eggup-archive --dry-run --locked

cargo package -p eggup-core --list --locked
cargo package -p eggup-archive --list --locked
git diff --check
~~~

Push the release-prep commit and require the normal Stable/MSRV/macOS/Windows CI matrix to be green before real publication.

Record the exact release-prep SHA. All real publish commands MUST run from that clean commit.

### 6.4 Publish eggup-core 0.1.2

From the exact clean release-prep commit:

~~~bash
cargo publish -p eggup-core --locked
~~~

After Cargo reports success:

- do not immediately assume registry propagation;
- query crates.io until exact `eggup-core 0.1.2` is visible;
- verify the crate owner/metadata/repository/version are correct;
- verify a clean Cargo resolution can select `eggup-core = "=0.1.2"`.

Only after that verification may archive publication begin.

### 6.5 Publish eggup-archive 0.1.2

Reconfirm:

- working tree clean;
- exact same release-prep commit checked out;
- `eggup-archive 0.1.2` still absent;
- core 0.1.2 visible.

Then:

~~~bash
cargo publish -p eggup-archive --locked
~~~

After success, verify exact registry visibility and metadata.

### 6.6 Registry-only external smoke

Use a fresh temporary directory outside the Eggup workspace and without `[patch]`, path, or git overrides.

The fixture MUST depend on:

~~~toml
eggup-core = "=0.1.2"
eggup-archive = "=0.1.2"
~~~

Exercise the representative M001d flow:

`verified archive -> PersistedExtraction::into_bound_sources -> BoundExtraction -> BoundSources -> InstallPlan::prepare_with_bound_sources`.

At minimum run both tar.gz and zip positive flows plus one declared-member mismatch/rejection case.

The dependency tree MUST show both crates resolving from crates.io, not the local workspace.

### 6.7 Tag and GitHub Release

After both crates are registry-visible and the registry-only smoke passes:

- create `v0.1.2` pointing **exactly** at the release-prep commit used for publication;
- push the tag;
- create GitHub Release `0.1.2` for that tag.

Release notes MUST state:

- crates.io publication set: `eggup-core 0.1.2`, `eggup-archive 0.1.2`;
- other workspace crates were **not** published as 0.1.2;
- Rust MSRV 1.89;
- checksum integrity does not imply authenticity/signature verification;
- major 0.1.2 additions: object-bound archive handoff and bounded archive extraction;
- Egress adoption is now unblocked for consumer-side implementation.

Do not move or recreate the tag after publication.

## 7. Ordered work packages

### A — Preflight

Verify exact version absence, archive name availability/ownership, core publish authority, clean head, credential hygiene.

### B — Release-prep source commit

Cut root/archive changelogs to 0.1.2 without falsely publishing unrelated workspace changes.

### C — Exact-commit requalification

Run local clean qualification and hosted CI. Record release-prep SHA and package inventories.

### D — Core publication

Publish core 0.1.2; wait for and verify exact registry visibility.

### E — Archive publication

Publish archive 0.1.2 from the same commit; verify exact registry visibility.

### F — Registry-only consumer proof

Resolve both packages from crates.io in a clean external fixture and exercise the bound-source flow.

### G — Tag/release/closure

Create/push `v0.1.2`, create GitHub release, record closure, update Eggup planning, then mark Egress Delivery M003/M006 dependency gate satisfied.

## 8. Failure, retry, partial-publication, and irreversibility semantics

Publication is not transactional across two crates.

### Before core publication

Any failed preflight or qualification is fully reversible. Do not publish anything.

### Core upload accepted, archive not yet published

Core 0.1.2 is now immutable registry state.

If archive publication fails due to a transient registry/network error:

- verify whether 0.1.2 was actually accepted before retrying;
- retry only from the exact same release-prep commit/artifact if the version is still absent.

If archive publication exposes a source/metadata defect requiring source changes:

- STOP;
- do not move/create `v0.1.2` as though the pair completed;
- write a corrective release plan;
- do not overwrite/reuse an already accepted version;
- decide the follow-up version explicitly (normally a new patch for the affected package/release set).

### Archive upload accepted

Both package versions are immutable publication evidence. Subsequent defects use a new version; yanking is a separate explicit maintainer action and does not delete published bytes.

### Registry/index lag

Do not republish merely because search/docs pages lag. Verify exact version state through more than one registry-aware mechanism before retrying.

## 9. Compatibility and migration

No Rust API migration is introduced by M009; it publishes already-qualified additive 0.1.2 APIs.

Existing 0.1.1 consumers remain valid.

Egress may move from immutable-revision qualification to registry dependencies only after M009 closure verifies both exact versions.

Other Eggup crates remain at their existing published versions until separately planned.

## 10. Required tests and operational checks

Before real publication:

- clean working tree;
- full Stable fmt/clippy/tests/docs;
- Rust 1.89 check;
- macOS hosted suite;
- Windows hosted suite including `eggup-eggpack` adapter runtime;
- core/archive package and publish dry-runs;
- package file-list inspection;
- exact version-absence/name/ownership preflight.

After publication:

- exact crates.io visibility for both versions;
- registry-only external dependency resolution;
- tar.gz + zip bound-source success;
- mismatch rejection;
- dependency tree proves no local/git/path override;
- Git tag points at exact publication commit;
- GitHub release points at same tag;
- release notes accurately identify only two published crates.

## 11. Verification commands

Representative commands; do not record secrets:

~~~bash
git status --porcelain
git rev-parse HEAD

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked

cargo package -p eggup-core --locked
cargo publish -p eggup-core --dry-run --locked
cargo package -p eggup-archive --locked
cargo publish -p eggup-archive --dry-run --locked

# irreversible; maintainer-authorized only
cargo publish -p eggup-core --locked
# verify exact registry visibility before continuing
cargo publish -p eggup-archive --locked

git rev-parse v0.1.2
git diff --check
~~~

For exact registry checks, use crates.io/Cargo mechanisms that identify the requested version, not fuzzy search alone.

## 12. Documentation and planning updates

Before publication:

- root `CHANGELOG.md`;
- `crates/eggup-archive/CHANGELOG.md`.

After publication:

- `plans/closure/verified-update-core/009-status.md`;
- `plans/subsystems/verified-update-core-roadmap.md`;
- `plans/subsystems/consumer-adoption-roadmap.md`;
- `plans/registry.md`;
- Egress `plans/subsystems/delivery-roadmap.md`;
- Egress `plans/registry.md`;
- Egress canonical `docs/ROADMAP.md`.

Egress updates are status-only: Delivery M003 becomes ready; no consumer production implementation belongs to M009.

## 13. Acceptance criteria

M009 closes only when:

- release-prep commit is clean and hosted-green;
- `eggup-core 0.1.2` is published and registry-visible;
- `eggup-archive 0.1.2` is published and registry-visible;
- both came from the exact same recorded release-prep commit;
- registry-only external smoke passes for both formats and rejection case;
- no unrelated Eggup crate was published;
- `v0.1.2` points to the exact publication commit;
- GitHub Release `0.1.2` exists and accurately identifies the publication subset;
- closure evidence contains no credential/token material;
- Eggup M006 and Egress Delivery M003 are updated from publication-blocked to ready.

## 14. Stop conditions

STOP before core publication if:

- `eggup-archive` exact name/ownership availability is ambiguous;
- either 0.1.2 version already exists unexpectedly;
- release-prep CI is not fully green;
- package/dry-run differs materially from M008a;
- working tree is dirty;
- credentials/authorization are uncertain;
- changelog/release notes imply unrelated 0.1.2 crates will be published.

STOP after a partial publication if:

- a source/metadata change would be required to publish the second crate;
- exact version state is ambiguous;
- registry-only smoke resolves unexpected sources/versions.

Do not solve a stop by moving tags, overwriting versions, leaking credentials, or publishing additional crates.

## 15. Closure evidence required

Record:

- release-prep commit SHA;
- preflight version/name/ownership results without secrets;
- hosted CI run ID;
- exact prepublish package/dry-run command results;
- package file inventories/sizes;
- exact timestamps/results for core and archive publication;
- exact crates.io version identifiers/links;
- registry-only fixture manifest, dependency tree, and test results;
- `v0.1.2` tag target SHA;
- GitHub Release identifier;
- changelog/release-note diff;
- list of crates intentionally **not** published;
- Egress gate reconciliation commits;
- unresolved findings by severity.

## 16. Handoff notes

Treat this as a two-object release transaction with an irreversible first commit, not as ordinary code deployment.

The strongest safety rule is: prove the new archive crate name and both package artifacts before publishing core. Once crates.io accepts core 0.1.2, there is no rollback to an unpublished state.

Do not automate this milestone as part of normal CI. If future releases justify automated publication, design that separately rather than expanding M009.
