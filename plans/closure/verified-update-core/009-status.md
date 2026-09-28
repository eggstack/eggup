# Verified Update Core M009 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/009-core-archive-0.1.2-publication.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M009--eggup-core--eggup-archive-012-publication`

Reviewed repository baseline (plan): `71eb9bdf44cb739adc3701b00310f58430fa7bd1`

Release-prep commit (publication source): `e8e07eb538d0eef18ea4cb4ace3bb905c316da72`
("release: cut 0.1.2 changelog entries for eggup-core and eggup-archive")

Implementation commits:

- `e8e07eb` — release-prep changelog cut (root `CHANGELOG.md` 0.1.2 section +
  `crates/eggup-archive/CHANGELOG.md` 0.1.2 section). Only commit between the
  prior head (`2011bb6`, plans-only) and publication; no `crates/` source,
  manifest, workflow, or version edits.
- tag `v0.1.2` (tag object `19e891490bf5cc40aa88f87474dfd206cb17bc2e`) →
  exactly `e8e07eb` (verified via `git rev-parse "v0.1.2^{commit}"`).
- GitHub Release `0.1.2` → tag `v0.1.2`
  (`https://github.com/eggstack/eggup/releases/tag/v0.1.2`).

## Executive finding

M009 published the already-qualified 0.1.2 pair in the manual order the plan
requires — `eggup-core 0.1.2` (2026-09-28T21:53:21Z) then `eggup-archive`
0.1.2 (2026-09-28T21:53:39Z), both from the exact clean release-prep commit
`e8e07eb` with no `--allow-dirty`. Exact crates.io visibility was verified
through the API for both versions, and a fresh registry-only external
fixture resolved both `=0.1.2` crates from crates.io (lockfile
`source = "registry+https://github.com/rust-lang/crates.io-index"`, checksums
matching the registry) and passed the tar.gz + zip bound-source flows plus
the declared-member mismatch rejection (3/3). `v0.1.2` and GitHub Release
`0.1.2` point at the publication commit and name exactly the two published
crates. No unrelated Eggup crate was published. No credential material
appears in any command output, commit, or this record. M009 is closed;
Consumer M006 / Egress Delivery M003 is now executable.

## Requirement-to-evidence matrix

| Requirement (source plan §13) | Evidence | Result |
|---|---|---|
| release-prep commit clean and hosted-green | `e8e07eb`; `git status --porcelain` empty before both uploads; hosted run `36487099388` (Stable checks / MSRV check / macOS tests / Windows archive-acquisition-service — all success) | passed |
| `eggup-core 0.1.2` published and registry-visible | `cargo publish -p eggup-core --locked` → `Published eggup-core v0.1.2`; API `GET /api/v1/crates/eggup-core/0.1.2` → version id `3351376`, `created_at 2026-09-28T21:53:21Z`, `yanked: false`, checksum `0f44129c…cc8e6`, size 36327 bytes (matches local `.crate`), `rust_version 1.89`, `published_by dbowm91`; https://crates.io/crates/eggup-core/0.1.2 | passed |
| `eggup-archive 0.1.2` published and registry-visible | reconfirmed clean tree + same commit + 0.1.2 absent + core visible before upload; `cargo publish -p eggup-archive --locked` → `Published eggup-archive v0.1.2`; API `GET /api/v1/crates/eggup-archive/0.1.2` → version id `3351378`, `created_at 2026-09-28T21:53:39Z`, `yanked: false`, checksum `758e9564…1c8292`, size 27102 bytes (matches local `.crate`), `rust_version 1.89`, `published_by dbowm91`; https://crates.io/crates/eggup-archive/0.1.2 | passed |
| both from the exact same recorded release-prep commit | both uploads ran with HEAD `e8e07eb` on a clean tree; package verification re-ran on that commit (see commands) | passed |
| registry-only external smoke (both formats + rejection) | temp crate `eggup-registry-consumer-smoke` outside the workspace, deps `eggup-core = "=0.1.2"` + `eggup-archive = "=0.1.2"`, no `[patch]`/path/git overrides; `cargo test` → `m009_registry_flow_tar_gz`, `m009_registry_flow_zip`, `m009_registry_tar_gz_mismatch_is_rejected` — 3 passed | passed |
| no unrelated Eggup crate published | only the two `cargo publish -p …` invocations above; `eggup-acquisition`, `eggup-eggfetch`, `eggup-curl`, `eggup-service`, `eggup-transport-footprint` untouched on the registry; `eggup-eggpack` remains `publish = false` | passed |
| `v0.1.2` points to the exact publication commit | `git rev-parse "v0.1.2^{commit}"` → `e8e07eb…`; tag pushed to `origin`; tag never moved | passed |
| GitHub Release `0.1.2` exists, accurate subset | release `0.1.2` (not draft/prerelease) → tag `v0.1.2`, notes name the two-crate publication set, MSRV 1.89, checksum-only integrity, and the Egress unblock | passed |
| closure evidence contains no credential/token material | all commands below print no secrets; credentials file never displayed | passed |
| Eggup M006 and Egress Delivery M003 updated to ready | this closure + roadmap/registry reconciliation (see Roadmap disposition); Egress-side status-only files updated in `eggstack/eggress` | passed |

## Production implementation and audit evidence

No production change in M009: zero `crates/` source, manifest, workflow, or
version edits. The release-prep commit touches only the two changelogs:

- root `CHANGELOG.md`: new `## 0.1.2 — 2026-09-28` section naming the exact
  crates.io publication set (`eggup-core` + `eggup-archive`), stating the
  remaining 0.1.2-versioned workspace crates were **not** published, and
  recording `eggup-eggpack` as `publish = false`; acquisition/service/curl/
  eggpack-adapter work stays under `## Unreleased`.
- `crates/eggup-archive/CHANGELOG.md`: `Unreleased` archive entries cut into
  `## 0.1.2 — 2026-09-28` (bounded allowlisted extraction, M001d handoff,
  M001c write half, M001b cleanup, package metadata, no-unsafe/MSRV notes);
  the "not published" notice removed. M001b/M001c/M001d safety semantics and
  the M008 package qualification are preserved verbatim from closure
  evidence; no historical closure record was rewritten.

Preflight (before any irreversible upload, no secrets recorded):

- `eggup-core 0.1.2` absent: registry API showed only 0.1.0/0.1.1.
- `eggup-archive` name free: `GET /api/v1/crates/eggup-archive` → 404, and
  `eggup-archive 0.1.2` → 404 immediately before its upload.
- core publish authority: maintainer `dbowm91` already owned `eggup-core`
  (0.1.0/0.1.1 audit); valid crates.io credential present in the local cargo
  credentials file (contents never printed, committed, or logged).
- `main` clean at `2011bb6` (plans-only deltas over the M008a baseline);
  plan lineage verified (`71eb9bd..HEAD` = M009 registration commits only).

Prepublish package evidence on `e8e07eb` (no `--allow-dirty`; no material
delta from M008a — core still 19 files, archive still 7 files; the only
packaged-content change is the archive CHANGELOG 0.1.2 cut, which is why
requalification was required):

- core: `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`,
  `README.md`, 5 examples, 10 `src/` files (19 total); `.crate` 36327 bytes.
- archive: `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`,
  `Cargo.toml.orig`, `README.md`, `src/lib.rs` (7 total); `.crate` 27102 bytes.
- `cargo tree -p eggup-core` → `sha2` only; `cargo tree -p eggup-archive` →
  format stack (flate2/fs_at/sha2/tar/zip), no `eggup-core` production dep.

Stop-condition review (all negative, as required):

- no version/name/ownership ambiguity before core upload;
- no unexpected pre-existing 0.1.2 version;
- release-prep CI fully green before publication;
- package/dry-run shows no material delta from M008a;
- working tree clean at both uploads;
- changelog implies no unrelated 0.1.2 publication;
- no partial-publication retry was needed (both uploads accepted first try;
  cargo's post-upload availability wait succeeded for each).

## Exact verification commands and results

Local environment: Darwin arm64; stable `rustc 1.89.0` / Cargo 1.89.0
(== MSRV). All run on `e8e07eb` with an empty `git status --porcelain`:

```text
cargo fmt --all -- --check                                              # clean
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  # clean
cargo test --workspace --all-targets --all-features --locked            # all suites green, 0 failures
  # 36 archive / 44 core / 47 eggfetch / 22 curl / 25 service /
  # 7 eggpack-unit + 11 archive_handoff + 28 interoperability / 101 lifecycle
cargo +1.89.0 check --workspace --all-targets --locked                  # clean
cargo doc --workspace --no-deps --locked                                # clean
cargo package -p eggup-core --locked                                    # 19 files, .crate 36327 bytes
cargo publish -p eggup-core --dry-run --locked                          # upload dry-run ok
cargo package -p eggup-archive --locked                                 # 7 files, .crate 27102 bytes
cargo publish -p eggup-archive --dry-run --locked                       # upload dry-run ok
cargo package -p eggup-core --list --locked                             # 19 files (list above)
cargo package -p eggup-archive --list --locked                          # 7 files (list above)
git diff --check                                                        # clean
```

Irreversible (maintainer-authorized; commit `e8e07eb`, clean tree, CI green):

```text
cargo publish -p eggup-core --locked      # Published eggup-core v0.1.2 (2026-09-28T21:53:21Z)
# API verified eggup-core 0.1.2 visible (id 3351376) before continuing
cargo publish -p eggup-archive --locked   # Published eggup-archive v0.1.2 (2026-09-28T21:53:39Z)
```

Registry-only smoke (temp dir outside the workspace; manifest depends on
`eggup-core = "=0.1.2"` and `eggup-archive = "=0.1.2"` with no `[patch]`,
path, or git overrides):

```text
cargo test      # 3 passed (tar.gz flow, zip flow, mismatch rejection)
cargo tree      # eggup-archive v0.1.2 + eggup-core v0.1.2, no local paths
# Cargo.lock: both crates source = "registry+https://github.com/rust-lang/crates.io-index"
#   with checksums 0f44129c…cc8e6 / 758e9564…1c8292 matching the registry API
```

Tag and release:

```text
git tag -a v0.1.2 -m "…" e8e07eb…   # tag object 19e891490b…
git rev-parse "v0.1.2^{commit}"         # e8e07eb… (exact publication commit)
git push origin v0.1.2
gh release create v0.1.2 --title "0.1.2" --notes-file …  # https://github.com/eggstack/eggup/releases/tag/v0.1.2
```

Hosted matrix for the release-prep commit:

- run `36487099388` (head `e8e07eb`, push): Stable checks success,
  MSRV check success, macOS tests success, Windows archive/acquisition/
  service tests + check success.

Confirmation of manual-only publication: no workflow file was changed or
added; CI ran only its normal check lanes on the release-prep commit.

## Invariant review

- publication was a deliberate maintainer action, never a CI side effect;
- published exactly the artifacts qualified from the release-prep commit;
- no `--allow-dirty` on any real publish command;
- no version overwrite: both 0.1.2 versions were proven absent first;
- core published (21:53:21Z) before archive (21:53:39Z);
- archive name/ownership availability proven (404) before the core upload;
- no unrelated 0.1.2 workspace crate published;
- no authenticity/signature claims in package metadata or release notes;
- Rust 1.89 support documented in metadata, changelogs, and release notes;
- tag and GitHub release identify the exact publication source commit;
- release notes distinguish the workspace source version from the two
  actually published crates;
- Egress is marked ready only now that both registry packages resolve from
  crates.io without local/git/path overrides (smoke lockfile evidence).

## Failure/rollback/recovery review

Publication is not transactional across two crates; the plan's partial-state
semantics applied:

- before core publication: all preflight/qualification reversible — passed,
  so publication proceeded;
- core accepted, archive pending: core 0.1.2 became immutable registry
  state at 21:53:21Z. Archive preconditions re-verified (clean tree, same
  commit, 0.1.2 still absent, core visible); no defect surfaced, so no
  corrective plan was needed;
- archive accepted 21:53:39Z: both versions immutable. No yank performed;
  any future defect takes a new version through the normal plan process.
- no registry/index-lag republish: cargo's availability wait plus two
  independent API reads confirmed exact version state before each next step.

## Compatibility and migration review

No Rust API migration: M009 publishes the already-qualified additive 0.1.2
APIs unchanged. Existing 0.1.1 consumers remain valid (caret requirements
keep resolving; no yank of prior versions). Egress may now move from
immutable-revision qualification to registry dependencies. Other Eggup
crates remain at their existing published versions until separately planned.

## Security review

No new trust boundary, dependency, or audit surface: no source change at
all. Package contents inspected (inventories above). No authenticity or
signature support claimed anywhere. Registry credential was used from the
existing local cargo credentials file; it appears in no command output,
commit, CI log, or this record. The smoke fixture manifest contains only
public version requirements.

## Documentation/operations evidence

- root `CHANGELOG.md` + `crates/eggup-archive/CHANGELOG.md` 0.1.2 cut
  (release-prep commit `e8e07eb`);
- GitHub Release `0.1.2` notes (publication set, MSRV, checksum-only
  integrity, Egress unblock);
- this closure record;
- roadmap + registry reconciliation (see below); no historical record
  rewritten.

## Unresolved findings

None at medium-or-higher severity. Notes (informational):

- docs.rs builds for the two new versions are automatic registry side
  effects and were not gated by the plan; no action required.
- The registry-only smoke fixture lives in scratch temp space outside the
  repo; its manifest, lockfile evidence, and results are recorded above.

## Roadmap disposition

Verified Update Core M009 is closed. No further core publication work is
required for the Egress M006 dependency cutover: the §14 merge gate
("Package M008 published in a crates.io-usable form") is satisfied by the
two registry-visible 0.1.2 versions proven above. Consumer Adoption M006
moves from publication-blocked to executable; implementation remains
Egress-owned under Egress Delivery M003 (no consumer code belongs to M009).

## Registry updates

- Verified Core M009 row: ready for maintainer handoff → closed (pair
  published, tag + release recorded here).
- Consumer M006 row: blocked on M009 publication → executable (gate
  satisfied; Egress Delivery M003 owns implementation).
- Execution graph: M009 → M006 edge now traversable.
