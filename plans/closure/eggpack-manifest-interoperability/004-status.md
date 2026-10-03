# Eggpack Manifest Interoperability M004 — Registry Package/API Promotion and Publication Closure

Status: closed

Source plan: `plans/implementation/eggpack-manifest-interoperability/004-registry-package-api-promotion-and-publication.md`

Preflight: `plans/closure/eggpack-manifest-interoperability/004a-status.md`

Roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Plan-authoring baseline: `3f4e99e381b233bfd4be1a676218e9ba2cdce2d4`

Release-prep commit: `e548b64721aeb6e3eeeec370d0da3e63a06843e7` ("release: promote eggup-eggpack to the registry for the 0.1.2 publication set")

Publication source commit: `02a1d32931be29cc3d8980833643b2cd822f2d28`

Hosted CI: [run 37090397398](https://github.com/eggstack/eggup/actions/runs/37090397398) on `02a1d32`; `Stable checks`, `MSRV check`, `macOS tests`, and `Windows archive, acquisition, and service tests and check` all passed. `git diff e548b64..02a1d32 -- crates/ Cargo.lock` is empty, so the published bytes are identical to the release-prep commit's `crates/` content.

External prerequisite consumed: `eggpack-manifest 0.1.0` published 2026-10-02 from `eggstack/eggpack@8d661e4`, checksum `2a08f24b05e9652878dd49145cdc3cbd38c7a76032d7b01a5fe1535d9446b629`; closure `eggstack/eggpack: plans/closure/release-manifest/003-status.md`.

## Executive finding

The producer-manifest interoperability seam is now registry-consumable end to end. `eggup-eggpack 0.1.2` is published to crates.io and resolves `eggpack-manifest` from the registry at the exact version `0.1.0` with no Git or path edge, after `eggup-acquisition 0.1.2` and `eggup-eggfetch 0.1.2` were published ahead of it in that order.

The promotion is byte-faithful rather than merely version-compatible: the published `eggpack-manifest 0.1.0` `src/lib.rs` hashes to the same SHA-256 as the source at the revision the adapter previously pinned by Git, so M004a's compatibility-incident branch was not triggered and the adapter needed no semantic requalification. Only the dependency source and package metadata changed; there was no `src/` change, no public API change, and no version change.

Publication order was a hard mechanical constraint, discovered by execution rather than assumed. `cargo package` and `cargo publish` resolve the *packaged* manifest's dependencies from crates.io, so `eggup-eggfetch 0.1.2` could not pass its own package verification before `eggup-acquisition 0.1.2` was visible, and `eggup-eggpack 0.1.2` could not produce a package at all. This is the same `E0308` incoherence M004a proved from the other direction, now observed as a build failure in the correct direction.

Both registry-only external graphs resolve with exactly one registry source per package and no `git+` or path entry, and both pass smoke against the published artifacts, including the adapter's exact-size enforcement and fail-closed negatives. Exactly three crates were published. `v0.1.1` and `v0.1.2` were not moved.

## Published identity (immutable)

| Crate | Version | crates.io checksum | Size | Publisher | Published at |
|---|---|---|---|---|---|
| `eggup-acquisition` | 0.1.2 | `0b01deb80e7b67a73348eb8fe9d8f55db4188e3fd10435b1828f98b96f6ef170` | 16732 | `dbowm91` | 2026-10-03T02:38:50Z |
| `eggup-eggfetch` | 0.1.2 | `2e48315263c6d2419e35d201007e420f6414f72e42c4e0233e834c973d8f3528` | 19488 | `dbowm91` | 2026-10-03T02:39:10Z |
| `eggup-eggpack` | 0.1.2 | `9dbfdfb74b4faa88d6badaa6f24e71dada3c7322dd0e9877cebe8b0168b13fef` | 30543 | `dbowm91` | 2026-10-03T02:39:29Z |

All three are `yanked = false`, edition 2021, `rust-version` 1.89, license MIT, and owned solely by `dbowm91`. Each local `.crate` SHA-256 equals the registry checksum exactly.

The full workspace `0.1.2` registry set is now `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack`. The already-published `eggup-core 0.1.2` (`0f44129c…cc8e6`) and `eggup-archive 0.1.2` (`758e9564…1c8292`) were not republished; `git diff v0.1.2..HEAD -- crates/eggup-core crates/eggup-archive` is empty.

These versions are immutable. No defect may be corrected by overwriting them; a defect takes a new version through the normal plan and closure process.

## Production changes

The only production change is the mechanical promotion M004a identified, plus the metadata truthfulness it flagged. `git diff v0.1.2..HEAD -- crates/` touches exactly three files, none of them `src/`.

**`crates/eggup-eggpack/Cargo.toml`**

- removed `publish = false`;
- `eggpack-manifest`: immutable-Git edge at `rev 678bbf04f5a02827003a1d9ab83ba4f0e6360e41` → registry `eggpack-manifest = "=0.1.0"`;
- backfilled `homepage`, `documentation`, `authors`, `keywords`, and `categories` so the published metadata matches the other published Eggup crates (M004a recorded the omission as polish);
- extended the `description` to state the no-release-policy / no-authenticity / no-destination-selection boundary;
- the three exact `=0.1.2` Eggup pins and `sha2 0.10.9` are unchanged.

**`crates/eggup-eggpack/README.md`**

The final paragraph asserted the crate "is unpublished and pins the currently unpublished `eggpack-manifest` crate to an immutable Eggpack revision." Both halves are false once published, and the file is rendered on crates.io, so it now describes the registry dependency and restates the boundary that manifest evidence is producer-established input which selects no release, verifies no authenticity, and authorizes no destination.

**`crates/eggup-eggpack/tests/interoperability.rs`**

`adapter_pins_immutable_eggpack_manifest_and_no_producer_crates` asserted the manifest contains `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`. M004 changes exactly that, so the test failed by design and its invariant had to be restated. It is replaced by `adapter_pins_registry_eggpack_manifest_and_no_producer_crates`, which asserts the same purpose more strongly:

- the exact registry dependency `eggpack-manifest = "=0.1.0"` is present;
- no VCS source edge exists (`git =`, `rev =`, `branch =`, `tag =`, `eggstack/eggpack.git`);
- the three exact `=0.1.2` Eggup pins are retained, so the adapter cannot silently resolve a different published seam;
- the pre-existing no-producer-crates rule is retained.

Producer-source provenance (published checksum, publication commit, and byte-identity with `678bbf04`) is recorded next to the constant so a future pin change is reviewable in place. This is a restatement of a deliberately changed invariant, not a weakening: the superseded test would have *passed* a Git edge that `cargo publish` cannot use at all. The guard's logic was verified to pass on the promoted manifest and fail on the pre-promotion manifest.

**`Cargo.lock`**

`eggpack-manifest` now resolves from crates.io instead of the git source, and no `git+` source remains anywhere in the lockfile. The yanked `yoke-derive 0.8.3` entry was refreshed to `0.8.4` (M004a's info finding; `yoke-derive` is transitive via `eggfetch-core`, so this affects only the `eggup-eggfetch` graph).

**`CHANGELOG.md`**

The `## 0.1.2` section asserted that acquisition/eggfetch/curl/service/eggpack "were **not** published as 0.1.2" and that `eggup-eggpack` "remains `publish = false` by design." Completing a partially shipped workspace version makes those statements false, so the section now records the complete publication set with per-crate dates and the eggfetch coherence reason, explicitly framed as "2026-09-28, completed 2026-10-02." The `Unreleased` M002 entry, which described the adapter as unpublished, is relabelled to point at the 0.1.2 publication. Nothing was erased to conceal a defect; the earlier statements are corrected in place with the reason stated.

## Requirement-to-evidence matrix

| Requirement (plan §8) | Evidence | Result |
|---|---|---|
| Eggpack prerequisite satisfied, identity preserved | `eggpack-manifest 0.1.0` published; Eggpack closure `plans/closure/release-manifest/003-status.md`; published `src/lib.rs` SHA-256 equals the `678bbf04` source; no adapter requalification performed | passed |
| Exactly three crates published, in order | three `cargo publish` invocations; registry API verified after each; `eggup-curl`, `eggup-service`, `eggup-transport-footprint` never uploaded | passed |
| No republish of core/archive | `git diff v0.1.2..HEAD -- crates/eggup-core crates/eggup-archive` empty; no publish command for either; registry checksums unchanged from M009 | passed |
| Adapter manifest fully registry-based | packaged `eggup-eggpack` lockfile resolves all Eggup/Eggpack edges from `registry+https://github.com/rust-lang/crates.io-index`; zero `git+` entries; `=0.1.2` and `=0.1.0` pins intact | passed |
| Package metadata complete and truthful | packaged manifest carries `homepage`, `documentation`, `authors`, `keywords`, `categories`, `license = "MIT"`, `rust-version = "1.89"`; README and CHANGELOG no longer claim unpublished | passed |
| Clean qualification | `scripts/check-local.sh` exit 0; `cargo +1.89.0 check --workspace --all-targets --locked` clean; `cargo +1.89.0 test -p eggup-eggpack` 28 passed; `cargo doc --workspace --no-deps --locked` clean; 336 workspace tests across 20 suites; `cargo package` + `cargo publish --dry-run` green for all three from a clean tree; no `--allow-dirty` on any command | passed |
| Hosted qualification | run 37090397398 green on `02a1d32` across all four lanes | passed |
| Registry-only usability | adapter-only and Eggsact-shaped external projects resolved exact `=0.1.2` and passed smoke; no `git+`/`path` in either lockfile | passed |
| Tags untouched | `refs/tags/v0.1.2` = `19e891490bf5cc40aa88f87474dfd206cb17bc2e` → `e8e07eb538d0eef18ea4cb4ace3bb905c316da72` before and after; `v0.1.1` untouched; release notes extended, not replaced | passed |
| Bounded scope | no `eggup-curl`/`eggup-service`/`eggup-transport-footprint` publication; no Eggsact change; no publication workflow added; `eggup-transport-footprint` remains `publish = false` | passed |
| Closure traceability | per-crate commit, checksum, size, publisher, timestamp, commands, CI run, and the `eggup-eggpack`-not-in-tag caveat recorded here | passed |

## Publication ordering: proven, not assumed

M004a predicted the order was load-bearing. Execution showed it is a hard build constraint, and the plan was amended to record that before any upload.

Before `eggup-acquisition 0.1.2` was on the registry:

```text
cargo package -p eggup-eggfetch --locked
  → error[E0308]: mismatched types  src/lib.rs:385:34
      if written > max_artifact {   expected `u64`, found `Option<u64>`
  → error[E0308]: mismatched types  src/lib.rs:387:36
  → error: failed to verify package tarball

cargo package -p eggup-eggpack --locked
  → error: failed to select a version for the requirement `eggup-acquisition = "=0.1.2"`
    candidate versions found which didn't match: 0.1.1, 0.1.0
    (this failed even with --no-verify: dependency *resolution* of the packaged
     manifest fails, so no .crate could be produced at all)
```

The `eggup-eggfetch 0.1.2` source is migrated to the 0.1.2 finite `u64` `max_artifact_bytes` seam, while the only registry candidate (`eggup-acquisition 0.1.1`) types that field as `Option<u64>`. This is the incoherence M004a identified, now observed as a compile failure in the packaging step.

Immediately after `eggup-acquisition 0.1.2` became visible:

```text
cargo package -p eggup-eggfetch --locked
  → Downloaded eggup-acquisition v0.1.2
    Compiling eggup-acquisition v0.1.2
    Compiling eggup-eggfetch v0.1.2 (target/package/eggup-eggfetch-0.1.2)
    Finished `dev` profile … exit 0
```

and after `eggup-eggfetch 0.1.2`, `cargo package -p eggup-eggpack --locked` verified cleanly against registry `eggup-core 0.1.2` and `eggup-archive 0.1.2`. Full verification was obtained for every crate actually uploaded, and no verification was skipped for a published crate.

## Registry-only consumer proof

Two temporary projects outside the repository, each with only registry dependencies and no `[patch]`, path, or Git override. Both were removed after the run.

**Adapter-only** — `eggup-eggpack = "=0.1.2"`, 58 packages locked:

```text
[[package]]
name = "eggup-eggpack"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9dbfdfb74b4faa88d6badaa6f24e71dada3c7322dd0e9877cebe8b0168b13fef"
```

Smoke asserted, against the published artifact: bounded `project_json` parse and exact-target projection; the projected artifact name, exact size, `member_id`, and destination for `x86_64-unknown-linux-gnu`; decoded non-zero SHA-256 evidence; fail-closed alias (`linux-x64`) and unknown-target resolution; the M003a `default_destinations` seam; and negatives for unsupported schema version, malformed JSON, an 8 MiB oversized document, and a truncated document. A canary string in the input was checked to confirm adapter diagnostics do not echo input contents. It also passed a second run with `--offline`, proving resolution from registry cache rather than any local path.

```text
ok m004_registry_adapter_smoke product=eggsact release=1.2.6 target=x86_64-unknown-linux-gnu
ALL ADAPTER-ONLY REGISTRY SMOKE CHECKS PASSED
```

**Eggsact-shaped** — `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack` all `= "=0.1.2"`, 144 packages locked. Every Eggup and Eggpack package resolved to exactly one registry source:

| Package | Version | Source | Checksum |
|---|---|---|---|
| `eggpack-manifest` | 0.1.0 | crates.io index | `2a08f24b…b629` |
| `eggup-acquisition` | 0.1.2 | crates.io index | `0b01deb8…f170` |
| `eggup-archive` | 0.1.2 | crates.io index | `758e9564…1c8292` (unchanged, M009) |
| `eggup-core` | 0.1.2 | crates.io index | `0f44129c…cc8e6` (unchanged, M009) |
| `eggup-eggfetch` | 0.1.2 | crates.io index | `2e483152…3528` |
| `eggup-eggpack` | 0.1.2 | crates.io index | `9dbfdfb7…b13fef` |

`grep 'git+\|path = ' Cargo.lock` matched nothing. Smoke asserted: the published `FetchLimits::new` accepts a finite `u64` artifact bound and validates; `effective()` never extends a caller deadline; `AcquisitionRequest::redacted` hides the URL; `EggfetchConfig` composition (user agent, redirect bound) compiles against the 0.1.2 seam; `ProductId`/`ReleaseId` accept valid and reject empty input; `ArtifactSet` accepts one member and rejects an empty set; the published adapter projects the producer manifest and `materialize_artifact_set` succeeds against a real acquired file of exactly the declared 3 bytes; and writing a 4-byte file for the same manifest evidence **fails closed**, proving exact-size enforcement survived the promotion.

```text
ok m004_registry_eggsact_shaped_graph product=eggsact release=1.2.6
ALL EGGSACT-SHAPED REGISTRY SMOKE CHECKS PASSED
```

## Verification executed

```text
scripts/check-local.sh                                            # exit 0
  # cargo fmt --all -- --check
  # cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
  # cargo test --workspace --all-targets --all-features --locked   # 336 passed, 20 suites
  # cargo doc --workspace --no-deps --locked
  # cargo tree --workspace --locked
cargo +1.89.0 check --workspace --all-targets --locked            # Finished
cargo +1.89.0 test -p eggup-eggpack --all-targets --locked        # 28 passed
cargo test -p eggup-eggpack --all-targets --all-features --locked # 61 passed, 4 suites
cargo package -p eggup-acquisition --locked                       # 6 files, 16.3KiB compressed
cargo publish -p eggup-acquisition --locked --dry-run             # upload dry-run ok
cargo package -p eggup-eggfetch --locked --no-verify              # 7 files (contents proof, pre-publication)
cargo package -p eggup-eggfetch --locked                          # 7 files, verify ok (post-acquisition)
cargo publish -p eggup-eggfetch --locked --dry-run                # upload dry-run ok
cargo package -p eggup-eggpack --locked --list                    # 19 files
cargo package -p eggup-eggpack --locked                           # 19 files, verify ok (post-eggfetch)
cargo publish -p eggup-eggpack --locked --dry-run                 # upload dry-run ok
git status --porcelain                                            # empty at every upload
git diff --check                                                  # clean
```

Pre-publication gate, immediately before each upload: clean tree on `02a1d32`; `git diff e548b64..02a1d32 -- crates/ Cargo.lock` empty; the exact version proven absent (`eggup-acquisition 0.1.2` / `eggup-eggfetch 0.1.2` "does not have a version `0.1.2`", `eggup-eggpack 0.1.2` "does not exist"); owners of the two existing packages confirmed as `dbowm91`; crates.io credential read from the local credentials file and never printed, committed, or logged.

Manual uploads, in order, no `--allow-dirty`:

```text
cargo publish -p eggup-acquisition --locked   # Published eggup-acquisition v0.1.2
cargo publish -p eggup-eggfetch --locked      # Published eggup-eggfetch v0.1.2
cargo publish -p eggup-eggpack --locked       # Published eggup-eggpack v0.1.2
```

Confirmation of manual-only publication: no workflow file was changed or added; `.github/workflows/ci.yml` still declares `permissions: contents: read` and contains no publish or release job.

## Tag and release disposition

`v0.1.1` and `v0.1.2` are immutable and were not moved. `refs/tags/v0.1.2` remained tag object `19e891490bf5cc40aa88f87474dfd206cb17bc2e` → `e8e07eb538d0eef18ea4cb4ace3bb905c316da72` before and after, verified against the remote.

The existing GitHub Release `0.1.2` notes were **extended, not replaced**, to record both publication steps, the complete 0.1.2 set, the promotion, the ordering constraint, the verification, and the source-tag caveat. The release remains non-draft, non-prerelease, and attached to `v0.1.2`.

Source-tag honesty is recorded explicitly in the release notes: `git diff v0.1.2..HEAD -- crates/eggup-acquisition crates/eggup-eggfetch` is empty, so the two lower crates are exactly the tagged content, but `eggup-eggpack` has changed since that tag by the M003a caller-bound seam and is **not** reproducible from it. Each crates.io version carries its own publication commit in `.cargo_vcs_info.json`, so per-crate auditability does not depend on the tag. The release notes originally asserted that the other crates "were **not** published as 0.1.2" and that `eggup-eggpack` "remains `publish = false` by design"; leaving that text would have been a false claim on the release page.

## Invariant review

- Publication was a deliberate maintainer action, never a CI side effect; no publication automation was added.
- Published exactly the three authorized crates. `eggup-curl`, `eggup-service`, and `eggup-transport-footprint` were never uploaded, and `eggup-transport-footprint` remains `publish = false`.
- No `--allow-dirty` on any package or publish command; the tree was clean at every upload.
- No version overwrite: each exact `0.1.2` was proven absent immediately before its upload.
- Ordering respected and mechanically necessary: `acquisition` (02:38:50Z) before `eggfetch` (02:39:10Z) before `eggpack` (02:39:29Z), each predecessor registry-verified before the next began.
- Exact pins not weakened: `eggup-eggpack` keeps `=0.1.2` on core/archive/acquisition and `=0.1.0` on `eggpack-manifest`; a regression test enforces it.
- No producer crate reached the consumer graph: `eggup-core`, `eggup-acquisition`, and `eggup-archive` have no `eggpack` edge, and the adapter depends only on the lightweight schema crate. `eggpack-core`, `eggpack-contract`, and `eggpack-bootstrap` remain absent from the adapter manifest, enforced by test.
- Adapter authority unchanged: it selects no release, verifies no authenticity, and authorizes no filesystem destination. Manifest `install` remains producer default identity only, with the M003a caller-bound seam available for consumers with their own destination policy.
- Integrity remains SHA-256 checksum evidence only. No authenticity or signature claim appears in any manifest field, README, changelog entry, or release note.
- The producer schema is reached only through crates.io. No Git, branch, tag, or path edge survives anywhere in the adapter manifest, its packaged lockfile, or either external smoke lockfile.
- Dependency-direction and authority regressions remain gated: `adapter_source_claims_no_producer_or_service_authority` and the restatement above both pass.

## Failure, rollback, and recovery review

Publication is not transactional across three crates, so the plan's partial-state semantics applied:

- Before the first upload, all qualification was reversible and passed, so publication proceeded.
- Each crate was published only after its predecessor was proven registry-visible, and each upload was accepted on the first attempt. Cargo's post-upload availability wait plus an independent crates.io API read confirmed exact version state before starting the next crate.
- No partial-publication retry, no corrective plan, and no mid-execution defect arose.
- No yank was performed. If a defect is found in a published `0.1.2`, the published record is preserved and a new version is taken through the normal plan and closure process; `0.1.2` is never overwritten.

One stop-and-record event occurred before any upload, and is disclosed rather than smoothed over: `cargo package -p eggup-eggfetch` failed to verify. It was investigated, classified as the already-known `E0308` incoherence surfacing in the packaging step, recorded in the plan, and resolved by the planned ordering rather than by a code, API, or pin change. No invariant was relaxed to make it pass.

## Compatibility and migration review

No Rust API migration. The `0.1.2` adapter source is the M003a-qualified source unchanged; only its manifest dependency source, package metadata, and documentation changed. Existing `0.1.1` and `0.1.0` consumers of `eggup-acquisition` and `eggup-eggfetch` remain valid: no prior version was yanked, and their caret requirements keep resolving.

The Git-pinned M003 Eggsact qualification remains valid and untouched. Because the published `eggpack-manifest 0.1.0` is byte-identical to the pinned `678bbf04` source, a consumer may move from the Git pin to a registry `=0.1.0` requirement without semantic requalification. Eggsact's own migration is separately authorized in `eggstack/eggsact` and is **not** claimed here.

The yanked `yoke-derive 0.8.3` → `0.8.4` lockfile refresh affects only transitive resolution for the `eggup-eggfetch` graph; it is a patch-level, non-breaking change in a dependency of `eggfetch-core`.

## Security review

No new trust boundary, authority, or audit surface. The adapter gained registry metadata only. Package contents were inspected: `eggup-acquisition` 6 files, `eggup-eggfetch` 7 files, `eggup-eggpack` 19 files, all small source/fixture files with no secrets, no generated binaries, and no unrelated repository content. The `eggup-eggpack` fixtures are the adapter's existing compatibility JSON plus `fixtures/README.md` provenance.

The published `eggup-eggpack` README was corrected specifically because it is rendered on crates.io and asserted a publication state that is now false; an inaccurate claim on a registry page is a trust-surface defect, not cosmetic.

The crates.io credential was read from the existing local credentials file. It appears in no command output, commit, CI log, closure record, or release note, and was never echoed or logged. The credential was not committed to the repository or placed in CI configuration. Publisher authority is established rather than assumed: the crates.io audit trail records `publish` by `dbowm91` for all three versions, matching the sole owner of the pre-existing `eggup-core`, `eggup-archive`, `eggup-acquisition`, and `eggup-eggfetch` packages.

The external smokes performed no network fetch, no service-manager interaction, and no credential use. The only filesystem write in the Eggsact-shaped smoke was a self-owned temp file used to prove the adapter's exact-size check, removed at the end.

## Documentation and operations evidence

- root `CHANGELOG.md` `## 0.1.2` section completed, with per-crate publication dates and the eggfetch coherence reason; the `Unreleased` M002 entry relabelled to the publication.
- `crates/eggup-eggpack/README.md` final paragraph corrected to the published registry state.
- GitHub Release `0.1.2` notes extended with both publication steps, the promotion, the ordering constraint, verification, and the source-tag caveat.
- this closure record.
- plan amended to record the execution-proven ordering constraint before any upload.
- roadmap and registry reconciled below. No historical closure record was rewritten.

## Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| low | `eggup-curl`, `eggup-service`, and `eggup-transport-footprint` remain at workspace version `0.1.2` but are unpublished. A consumer that assumes a workspace version is a published version would be wrong. | Accepted and recorded in the CHANGELOG and release notes. M004a's graph proof showed no edge from them into the published adapter, so publishing them is not authorized by this milestone. A future publication needs its own plan. |
| low | `eggup-eggpack 0.1.2` is not reproducible from the `v0.1.2` Git tag, because M003a landed after that tag. | Accepted; tags are immutable and were not moved. The release notes state it explicitly, and per-crate `.cargo_vcs_info.json` preserves exact commit auditability. A tag that covers the adapter would require a new release version, not a tag move. |
| informational | `eggup-curl`'s `proxy_custom_env_is_explicit` test failed once during a cold full-workspace run and did not reproduce in three isolated runs or two subsequent full 336-test runs. It spawns a generated fake-curl script under a PID+nanos temp path. | Classified as a pre-existing flake in a crate outside this milestone's scope, which was neither modified nor published. Not investigated further here because it cannot affect a published artifact; carried for the next `eggup-curl` work. |
| informational | docs.rs had not yet produced build metadata for the three new versions at closure time. | Not gated by this milestone; docs.rs builds are an automatic registry side effect. Re-check later; a build failure would be a docs-infra signal, not an evidence or integrity defect. |
| informational | crates.io's crate-level summary for `eggup-eggpack` reports `license: null` while the version record and packaged manifest both report `MIT`. | Cosmetic crates.io API inconsistency. The published artifact is correct and the `LICENSE`-bearing manifest is what downstream tooling reads. |

No medium-or-higher finding remains open, and none reopens this milestone.

## Roadmap disposition

- M004a: closed (unchanged). Its proven publication set was executed exactly, in its proven order.
- M004 package/API promotion: **closed** by this record. The adapter is registry-resolvable against the published producer schema, the full 0.1.2 workspace set is consistent on crates.io, and both registry-only consumer graphs resolve and pass smoke.
- Eggup interoperability M003/M003a: unchanged, still closed. No corrective is triggered; the M003a caller-bound seam shipped in the published `0.1.2`.
- M002 archive handoff: closed historically; its adapter code shipped in this publication, which its closure record had described as unpublished. That historical record is left unrewritten, per the corrective discipline.

Dependency transitions unlocked:

- **Eggsact's Git→registry migration** is now executable against the registry: `eggup-eggpack =0.1.2` plus `eggpack-manifest =0.1.0` resolve with no Git or path edge. Implementation remains Eggsact-owned and separately authorized; this closure does not claim it.
- A consumer may now depend on the producer schema (`eggpack-manifest 0.1.0`) and the full Eggup 0.1.2 set without any Eggpack or Eggup Git checkout.

Explicitly **not** unlocked and **not** claimed:

- Publication of `eggup-curl`, `eggup-service`, or `eggup-transport-footprint`.
- Any authenticity, signing, or release-selection authority. None exists in these crates.
- Any new subsystem milestone. This seam's Eggpack-side work is complete; further work needs new evidence and a new plan.

## Registry updates

- M004a row: closed (unchanged), with its Eggpack-owned item now discharged.
- M004 row: active/authorized → **closed** (this record; three crates published, registry-only proof green, tag policy honoured).
- Dependency table: M004 now `closed`; the Eggup-owned publication chain is discharged.
- Planned/blocked table: M004 moved out of blocked; no new blocked item is created by this closure.
- Immediate execution graph: M004a → M004 edge now closed end to end, ending at the Eggsact-owned migration which Eggup does not own.
- Cross-repo mapping: the Eggpack prerequisite (`eggstack/eggpack@48ed13c` / `64cc845`, Release Manifest M003 closed) is recorded as consumed.
