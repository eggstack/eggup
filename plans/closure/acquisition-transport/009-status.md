# Acquisition Transport M009 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/009-eggup-curl-0.1.2-publication.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M009--eggup-curl-012-publication`

Reviewed repository baseline (plan): `6a3e1e931b4976deebb0da5ca28f6462e899671e`

Release-prep commit (publication source): `b485228d1edebe27e2dd691310c7c68f60d97d8d`
("release: cut 0.1.2 changelog entries for eggup-curl")

Predecessor runtime commit in the publication source: `0b8cb983058432f7a5a592b8b98bedac1339d4e6`
("fix: address workspace bug audit findings across all crates")

Hosted CI for the publication source: run
[`37223895075`](https://github.com/eggstack/eggup/actions/runs/37223895075) on
`b485228` — `Stable checks`, `MSRV check`, `macOS tests`, and `Windows archive,
acquisition, and service tests and check` all **success**.

Publication: `cargo publish -p eggup-curl --locked` from a clean tree with no
`--allow-dirty` → `Published eggup-curl v0.1.2 at registry 'crates-io'`.

Tag and release disposition: no tag was created or moved. `refs/tags/v0.1.2`
remained tag object `19e891490bf5cc40aa88f87474dfd206cb17bc2e` →
`e8e07eb538d0eef18ea4cb4ace3bb905c316da72` and `refs/tags/v0.1.1` remained
`881c95ff069d3d465a282cb6a495ba6fcb70cb6f` before and after, verified against the
remote. The existing GitHub Release `0.1.2` notes were **extended, not
replaced**. See "Tag and release disposition" below.

## Executive finding

M009 published `eggup-curl 0.1.2` to crates.io from the exact clean release-prep
commit `b485228`, and the crate is now visible on the registry
(https://crates.io/crates/eggup-curl/0.1.2, version id `3402311`, created
`2026-10-04T18:21:37.642594Z`, `yanked: false`, checksum
`79df200f5080c1f11c8d5a908ba8d73d44083ad13b17ae5576a8ad8cc52ab465`, size
`17193` bytes, `rust_version 1.89`, published by `dbowm91`). The local
`.crate` checksum and size match the registry exactly.

The published manifest carries exactly one dependency, `eggup-acquisition ^0.1.0`,
resolved from the registry. There is no Git, path, or branch edge anywhere in
the package. `cargo package` verified the crate by compiling it against the
registry-downloaded `eggup-acquisition 0.1.2`, which is the strongest available
proof that the packaged graph is registry-resolvable.

A fresh external fixture crate outside the workspace, declaring only
`eggup-curl = "=0.1.2"` and `eggup-acquisition = "=0.1.2"` with no `[patch]`,
path, or Git override, resolved both from crates.io and passed 6/6. Its
`Cargo.lock` shows `source = "registry+https://github.com/rust-lang/crates.io-index"`
for both crates with checksums matching the registry, and contains no `git+` or
path entry.

No transport semantics changed. The only repository changes for this milestone
were two changelog files and a `.gitignore` entry; not one file under
`crates/*/src/` was touched, and no `Cargo.toml` version, dependency
requirement, or workflow was edited. The crate was already carrying the
`0b8cb98` bug-audit fixes, so this publication also delivered the redirect
downgrade and coincident-ceiling timeout-phase corrections that `0b8cb98`
introduced.

The M007 Windows hosted live-loopback limitation is retained verbatim and is not
upgraded anywhere. The M005–M008 transport contract is unchanged.

One medium finding is carried forward rather than silently absorbed: the
published `eggup-acquisition 0.1.2` predates two `0b8cb98` acquisition fixes, so
the composed-transport deadline defect is still reachable for registry consumers
today. That defect is pre-existing and is **not** introduced by this publication.
It is now recorded in the root `CHANGELOG.md`, the GitHub Release `0.1.2` notes,
and this record, and it is planned as M010.

## Requirement-to-evidence matrix

| Requirement (source plan §) | Evidence | Result |
|---|---|---|
| §4.1 inspect manifest metadata and package contents | `crates/eggup-curl/Cargo.toml`; `cargo package --list` → 8 files | passed |
| §4.2 README/license included | packaged list contains `README.md` and `CHANGELOG.md`; `license = "MIT"` SPDX in the normalized manifest and in the registry API record; matches the already-published core/archive convention | passed |
| §4.3 path deps become registry-resolvable | packaged `Cargo.toml` shows `[dependencies.eggup-acquisition] version = "0.1.0"` with the path rewritten away; registry dependency record shows a single `eggup-acquisition ^0.1.0` normal edge | passed |
| §4.4 resolves against published `eggup-acquisition 0.1.2` | `cargo package` verify step downloaded and compiled `eggup-acquisition v0.1.2` from the registry | passed |
| §4.5 no unpublished workspace crate becomes a registry dep | the crate has exactly one dependency and it is published; no `eggup-core`/`eggup-archive`/`eggup-eggpack` edge | passed |
| §4.6 inspect `cargo package --locked` output | 8 files, `69.3KiB` packaged, `16.8KiB` compressed; file list recorded above | passed |
| §4.7 build/test from packaged source | `cargo package` verification compiled the packaged crate; external fixture exercised the published artifact | passed |
| §4.8 `cargo publish --dry-run` | upload dry-run ok, exit 0, from a clean tree | passed |
| §4 metadata-only adjustment kept behavior-neutral | **no manifest edit was needed at all**; the existing `^0.1.0` caret requirement was kept deliberately, see "Metadata decision" below | passed |
| §5 Stable gate | `cargo +stable fmt/clippy/test/doc` all clean, exit 0, 20 test-result lines all `ok` | passed |
| §5 MSRV gate | `cargo +1.89.0 check/test --workspace --all-targets --locked` clean; the active default toolchain is `1.89.0`; hosted `MSRV check` success | passed |
| §5 macOS lane | local `cargo test --workspace --all-targets --all-features` green on Darwin; hosted `macOS tests` success | passed |
| §5 Windows lane | hosted `Windows archive, acquisition, and service tests and check` success, which runs `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked` | passed |
| §5 no failure for the historical Windows loopback case | publication proceeded; the limitation is retained and no Windows live-HTTP claim is made | passed |
| §6 publish only `eggup-curl 0.1.2` | one `cargo publish` invocation; registry still shows no `eggup-service` or `eggup-transport-footprint` | passed |
| §6 registry-only external fixture | 6/6 passing, lockfile registry-only, checksums match | passed |
| §6 package graph has no Git/path source | lockfile has no `git+` or `path` entry; `cargo tree` shows only the two registry crates | passed |
| §6 tag naming follows existing convention | no per-crate tag created; the repo's actual convention is a single shared `v0.1.2` tag plus extended release notes | passed (see below) |
| §6 do not move existing `v0.1.2` tags or republish others | remote tag refs byte-identical before and after; no other crate uploaded | passed |
| §7 cargo-cleanme downstream note | recorded in §7 of the release notes and in the "Downstream handoff" section below | passed |
| §8 mark M009 closed in the roadmap | `plans/subsystems/acquisition-transport-roadmap.md` updated | passed |
| §8 add publication evidence to the registry | `plans/registry.md` updated | passed |
| §8 README updated only where it enumerates published crates | `crates/eggup-curl/README.md` does not enumerate published versions and asserts nothing false; left unchanged | passed |
| §8 keep M005–M008 closure records unchanged | no historical closure record was edited | passed |
| §9 Windows live-loopback limitation stated truthfully | stated in the crate changelog, root changelog, release notes, and this record | passed |
| §10 no stop condition fired | all five stop conditions reviewed negative, see below | passed |

## Package-readiness audit detail

Packaged file list, `cargo package -p eggup-curl --locked --list`:

```text
.cargo_vcs_info.json
CHANGELOG.md
Cargo.lock
Cargo.toml
Cargo.toml.orig
README.md
examples/curl_fetch.rs
src/lib.rs
```

Package size: 8 files, `69.3KiB` packaged, `16.8KiB` compressed, `.crate`
`17193` bytes.

Normalized packaged manifest dependency section:

```text
[dependencies.eggup-acquisition]
version = "0.1.0"
```

No `[dependencies]` on anything else, no `[dev-dependencies]`, no
`[build-dependencies]`. `license = "MIT"`, `repository =
"https://github.com/eggstack/eggup"`, `documentation =
"https://docs.rs/eggup-curl"`, `homepage = "https://github.com/eggstack/eggup"`,
`readme = "README.md"`, `rust-version = "1.89"`, `edition = "2021"`.

`.cargo_vcs_info.json` inside the published artifact:

```json
{ "git": { "sha1": "b485228d1edebe27e2dd691310c7c68f60d97d8d" },
  "path_in_vcs": "crates/eggup-curl" }
```

## Metadata decision

The plan allowed a metadata-only adjustment "if Cargo requires" one. None was
required, and one specific non-change was deliberate.

`eggup-curl` requires `eggup-acquisition = { version = "0.1.0", path =
"../eggup-acquisition" }`. The caret `0.1.0` was **kept** rather than tightened
to `=0.1.2` (the form `eggup-eggpack` uses). Reason: the caret requirement is
what lets an existing `eggup-curl 0.1.2` consumer automatically pick up a future
`eggup-acquisition 0.1.3` carrying the correctness fixes recorded below. Pinning
would have frozen the defect into every consumer's lockfile. This choice is
load-bearing for the M010 handoff and is why M010 could plan around an automatic
fix path for curl and eggfetch consumers.

## Publication evidence

Registry API, `GET /api/v1/crates/eggup-curl/0.1.2`:

| Field | Value |
|---|---|
| id | `3402311` |
| crate | `eggup-curl` |
| num | `0.1.2` |
| created_at | `2026-10-04T18:21:37.642594Z` |
| yanked | `false` |
| checksum | `79df200f5080c1f11c8d5a908ba8d73d44083ad13b17ae5576a8ad8cc52ab465` |
| crate_size | `17193` |
| rust_version | `1.89` |
| license | `MIT` |
| published_by | `dbowm91` |
| audit action | `publish` by `dbowm91` at `2026-10-04T18:21:37.642594Z` |

Local artifact cross-check, `shasum -a 256 target/package/eggup-curl-0.1.2.crate`
→ `79df200f5080c1f11c8d5a908ba8d73d44083ad13b17ae5576a8ad8cc52ab465`, size
`17193`. Exact match, so the published bytes are the reviewed bytes.

Registry dependency record, `GET /api/v1/crates/eggup-curl/0.1.2/dependencies`:
one `normal` dependency, `eggup-acquisition ^0.1.0`, `optional: false`. No git,
path, or branch source.

Preflight, all recorded before the upload:

```text
GET /api/v1/crates/eggup-curl      -> 404   # crate name was free
GET /api/v1/crates/eggup-curl/0.1.2 -> 404  # version absent, no clobber
eggup-acquisition 0.1.2 -> yanked False, rust_version 1.89,
                           checksum 0b01deb80e7b…f170 (16732 bytes)
```

`eggup-service` and `eggup-transport-footprint` were confirmed still absent from
the registry and were not touched. No other `cargo publish` invocation was made.

## Registry-only external fixture

Fixture: `eggup-curl-registry-smoke`, a crate outside the Eggup workspace in
scratch space. Manifest declares only:

```text
eggup-curl = "=0.1.2"
eggup-acquisition = "=0.1.2"
```

No `[patch]`, no path, no git override, no workspace inheritance.

```text
cargo test
  # Downloading crates ...
  #  Downloaded eggup-curl v0.1.2
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured

m009_public_api_is_constructible                        ok
m009_discover_curl_executable_is_typed                 ok
m009_deadline_ceilings_are_min_of_request_and_adapter  ok
m009_missing_executable_is_unavailable_not_a_panic     ok
m009_exact_404_is_not_found_and_200_is_ordinary_success ok
m009_cancellation_is_typed                             ok
```

`Cargo.lock` source evidence:

```text
name = "eggup-acquisition"  version = "0.1.2"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "0b01deb80e7b67a73348eb8fe9d8f55db4188e3fd10435b1828f98b96f6ef170"
name = "eggup-curl"         version = "0.1.2"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "79df200f5080c1f11c8d5a908ba8d73d44083ad13b17ae5576a8ad8cc52ab465"
```

`grep -nE "git\+|source = \"git|path = " Cargo.lock` → no match. The
`eggup-curl` checksum in the lockfile equals the registry checksum, and the
`eggup-acquisition` checksum equals the value the registry reported during
preflight.

`cargo tree`:

```text
eggup-curl-registry-smoke v0.0.0 (/private/tmp/eggup-curl-registry-smoke)
├── eggup-acquisition v0.1.2
└── eggup-curl v0.1.2
    └── eggup-acquisition v0.1.2
```

What the 6 tests actually prove, and why the fixture is deterministic:

- `m009_public_api_is_constructible` — `CurlTransport::with_executable`,
  `executable()`, `config()`, and the `proxy` setter are reachable from an
  external crate. `CurlProxy` is deliberately not `PartialEq`, so the proxy
  assertion is a `matches!` rather than an equality, and the protocol assertion
  compares a sorted set because order is not a contract.
- `m009_discover_curl_executable_is_typed` — discovery yields either a real
  non-empty path or a typed `AcquisitionError::Unavailable`, never a panic.
- `m009_deadline_ceilings_are_min_of_request_and_adapter` — the M008 seam is
  observable externally: adapter ceilings of `10 s`/`120 s` under
  `FetchLimits::default()`, a sub-second caller ceiling of `100 ms`/`250 ms`
  passing through unmodified, and a looser caller ceiling of `30 s`/`300 s`
  clamped down to the adapter ceiling.
- `m009_missing_executable_is_unavailable_not_a_panic` — an unspawnable
  executable path produces `Unavailable`, which is the composition-safe
  classification M005 requires.
- `m009_exact_404_is_not_found_and_200_is_ordinary_success` — end-to-end
  through the real `curl` process against a loopback `TcpListener` the test
  itself owns: `200` returns the exact body, exact `404` becomes
  `FetchOutcome::NotFound` and is never a transport error, and an artifact
  stream is promoted into caller-owned staging with the exact bytes. No public
  internet access is involved.
- `m009_cancellation_is_typed` — a pre-cancelled flag returns
  `AcquisitionError::Cancelled` before any process work.

The fixture was pre-validated against local paths before publication, which is
how four real API-misuse errors in the first draft (a three-argument
`FetchLimits::new`, a `FetchOutcome::Ok` variant that does not exist, a
`PartialEq` on `CurlProxy`, and an order-dependent protocol assertion) were
caught before the post-publication registry run rather than after it.

## Stop-condition review (all negative, as required)

- packaged manifest requires an unpublished dependency — **no**: its only
  dependency is `eggup-acquisition`, published at `0.1.2`, and the package
  verification step downloaded it from the registry;
- registry `eggup-acquisition` does not satisfy the packaged graph — **no**:
  `cargo package` compiled the package against the registry copy, and the
  external fixture built and ran against it;
- current-host tests reveal a semantic regression from M005–M008 — **no**: the
  full workspace gate is green on both the `1.89.0` and `stable` toolchains and
  all four hosted lanes are green;
- package contents omit required source/license/readme material — **no**: 8
  files including `src/lib.rs`, `README.md`, `CHANGELOG.md`, and the example,
  with an SPDX `MIT` license matching the already-published crates;
- publication would require widening transport semantics or claiming unsupported
  Windows live-network evidence — **no**: zero `src/` bytes changed and no
  Windows live-HTTP claim appears anywhere.

Because no stop condition fired, no corrective/package-readiness plan was
required for the publication itself.

## Production implementation and audit evidence

Zero `crates/*/src/` changes. The complete repository delta for this milestone
is three files:

- `crates/eggup-curl/CHANGELOG.md` — the "this crate has never been published"
  notice was replaced by a real `## 0.1.2 — 2026-10-04` first-publication
  section carrying the M005–M008 qualified behavior, the two `0b8cb98` fixes,
  and the platform note. The old `## 0.1.2` "source version only, not published"
  section was removed because it became false.
- `CHANGELOG.md` — the 0.1.2 publication set was completed with `eggup-curl`
  (now three steps: 2026-09-28 / 2026-10-02 / 2026-10-04); the M005, M007, and
  M008 `Unreleased` entries were relabelled to point at the release that
  actually shipped them, the same correction M004 applied to its own entry; a
  published-content caveat for `eggup-acquisition 0.1.2` was added; and the
  "not published" line was narrowed to `eggup-service` and
  `eggup-transport-footprint`.
- `.gitignore` — added `.DS_Store`. Two untracked macOS Finder metadata files
  were present in the worktree and would have made `cargo publish` refuse the
  upload without `--allow-dirty`. Ignoring them is the durable fix and is
  behavior-neutral; the alternative was deleting regenerating files. This is the
  only non-changelog change and it is recorded here rather than presented as a
  release-prep content change.

No manifest, `Cargo.lock`, workflow, or version change. `git diff` over
`crates/ Cargo.toml Cargo.lock .github/` between `0b8cb98` and `b485228` shows
only `crates/eggup-curl/CHANGELOG.md`.

## Exact verification commands and results

Local environment: Darwin, `rustc 1.89.0` / Cargo 1.89.0 as the active default
(which is also the MSRV), plus the `stable` toolchain. All run on `b485228` with
an empty `git status --porcelain`.

```text
./scripts/check-local.sh                                # exit 0
  # cargo fmt --all -- --check                          # clean
  # cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  # clean
  # cargo test --workspace --all-targets --all-features --locked   # all suites ok, 0 failures
  # cargo doc --workspace --no-deps --locked            # clean
  # cargo tree --workspace --locked                     # review-only

cargo +stable fmt --all -- --check                      # clean
cargo +stable clippy --workspace --all-targets --all-features --locked -- -D warnings
                                                        # Finished, clean
cargo +stable test --workspace --all-targets --all-features --locked
                                                        # 20 "test result: ok" lines, 0 failures
cargo +stable doc --workspace --no-deps --locked        # clean

cargo package -p eggup-curl --locked                    # 8 files; verify compiled vs registry acquisition 0.1.2
cargo package -p eggup-curl --locked --list             # file list recorded above
cargo publish -p eggup-curl --locked --dry-run          # "Uploading … warning: aborting upload due to dry run", exit 0
shasum -a 256 target/package/eggup-curl-0.1.2.crate     # 79df200f…b465  (matches registry)
git diff --check                                       # clean
```

The MSRV lane is satisfied twice over: the active default toolchain *is*
`1.89.0`, so the `check-local.sh` run above is an MSRV run, and the hosted
`MSRV check` job independently ran `cargo check --workspace --all-targets
--locked` on `1.89.0`.

Irreversible, maintainer-authorized, from clean tree `b485228` with CI green:

```text
cargo publish -p eggup-curl --locked
  # Uploaded eggup-curl v0.1.2 to registry `crates-io`
  # Published eggup-curl v0.1.2 at registry `crates-io`
```

Registry-only fixture: see the dedicated section above. `cargo test` 6/6,
`cargo tree` registry-only, lockfile checksums matching the registry.

## Per-platform qualification

Reported separately, with no inference between platforms.

| Platform | Evidence | Scope of the claim |
|---|---|---|
| Linux | hosted `Stable checks` + `MSRV check` on run `37223895075`; curl network-dependent integration tests run in this lane | live local HTTP/curl evidence carried |
| macOS | hosted `macOS tests` on run `37223895075`; local full workspace test run on Darwin | live local HTTP/curl evidence carried |
| Windows | hosted `Windows archive, acquisition, and service tests and check` on run `37223895075`, which runs `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked` and `cargo check --workspace --all-targets --locked` | **portable adapter, process, and error-path fixtures only**; the hosted runner refused the historical spawned-curl loopback case, so **no Windows live HTTP/curl result is claimed** |

The M007 conditional-closure limitation is retained exactly as recorded and is
not upgraded by this publication. Publication was explicitly not failed because
the hosted Windows environment still refuses the loopback case, per plan §5.

## Invariant review

- **No transport semantics changed.** Zero `src/` bytes. The published adapter is
  the same qualified adapter M005–M008 produced, plus the two already-merged
  `0b8cb98` fixes.
- Release-selection, version, and fallback policy remain caller-owned. The
  adapter still fetches exact caller-selected URLs and holds no mirror,
  release, or version knowledge.
- `NotFound` remains data for the exact URL, never a transport fallback
  decision. Proven externally by
  `m009_exact_404_is_not_found_and_200_is_ordinary_success`.
- TLS/5xx/timeout/cancellation/size/staging failures remain hard acquisition
  failures. Proven externally by the `Unavailable` and `Cancelled` assertions
  and by the unchanged in-crate suite.
- `eggup-core` remains transport-neutral and Eggpack-independent. `eggup-curl`
  has no `eggup-core` dependency at all.
- SHA-256 remains integrity evidence only. No authenticity, signature, or
  producer-trust claim appears in the manifest, changelog, README, release
  notes, or this record.
- No Eggpack, release-discovery, self-update, or service policy was added.
- Publication did not imply the Windows live-loopback gap had been closed. The
  limitation is stated in four places.
- Cargo-cleanme's dependency is unblocked, and cargo-cleanme retains
  release/version/fallback/install/self-update policy. No Gregg migration is
  authorized or implied by this publication.

## Failure/rollback/recovery review

Publication is a single non-transactional upload, so partial-state semantics
apply. The plan's stop conditions were all reviewed negative before the upload,
and the upload was accepted on the first attempt.

- Before the upload, every step was reversible: no `--allow-dirty`, clean tree
  at `b485228`, hosted CI green, registry preflight proving the name was free
  and `0.1.2` absent.
- After acceptance at `2026-10-04T18:21:37.642594Z`, `eggup-curl 0.1.2` is
  immutable registry state. Cargo's post-upload availability wait succeeded and
  the version was confirmed visible through the API before any next step.
- No yank was performed and none is warranted: the published bytes are the
  reviewed bytes, the checksum matches, and the crate is dependency-free beyond
  a published seam.
- No registry/index-lag republish was needed. A second upload of the same
  version would be rejected by the registry and is never a recovery strategy.
- If a future defect is found in the published `0.1.2`, the only correct remedy
  is a new version through the normal plan process, exactly as M010 is doing for
  the acquisition seam.

## Compatibility and migration review

No Rust API migration. The publication exposes the already-qualified additive
`0.1.2` API unchanged. Existing consumers of a local checkout see no behavioral
difference. The new registry package is a strictly additive option: nothing that
resolved before fails to resolve now, and no prior version was yanked.

`eggup-curl 0.1.2` requires `eggup-acquisition ^0.1.0`, so a fresh resolve picks
the published `0.1.2` seam today and will pick up a future `0.1.3` automatically.
That caret is deliberately not tightened; see "Metadata decision".

No consumer migration was performed and none is authorized by this milestone.
`eggstack/eggsact` and other existing consumers are untouched.

## Security review

No new trust boundary, dependency, or audit surface. The package has one
dependency, already published, and the audit above enumerates all 8 packaged
files. `cargo package` compiled the package against the registry copy of that
dependency rather than a local path, so the packaged graph is proven, not
assumed.

The redirect-downgrade fix shipped in this version closes a real cleartext
downgrade path: an `https` request could previously follow a `302 Location:
http://…` with no error and no diagnostic, and promote the cleartext body as an
ordinary success. `--proto-redir` is now narrowed per request.

The crates.io credential was read from the existing local cargo credentials file.
It appears in no command output, commit, CI log, release note, or this record,
and was never echoed or logged. Publisher authority is established rather than
assumed: the registry audit trail records `publish` by `dbowm91`, matching the
sole owner of the pre-existing `eggup-core`, `eggup-archive`,
`eggup-acquisition`, and `eggup-eggfetch` packages.

The external fixture manifest contains only public version requirements and lives
outside the repository; it was not committed.

## Tag and release disposition

The plan suggested a per-package tag such as `eggup-curl-v0.1.2` "if used
elsewhere in Eggup". It is not. Eggup's actual convention, established by core
M009 and confirmed by interop M004, is a single shared `v0.1.2` tag for the
publication wave plus **extended** release notes, with per-crate auditability
carried by each crate's `.cargo_vcs_info.json`. Creating a new per-crate tag
would have invented a convention the repository does not use, and moving
`v0.1.2` is forbidden by the plan and by the immutability of an existing release.

So: no tag was created and none was moved.

```text
refs/tags/v0.1.1 -> 881c95ff069d3d465a282cb6a495ba6fcb70cb6f   (unchanged)
refs/tags/v0.1.2 -> 19e891490bf5cc40aa88f87474dfd206cb17bc2e
                    -> e8e07eb538d0eef18ea4cb4ace3bb905c316da72   (unchanged)
```

Verified against the remote before and after the release edit.

The GitHub Release `0.1.2` notes were extended, not replaced, to add step 3, the
completed publication set, the `eggup-curl` contract summary, the platform note,
the `eggup-acquisition 0.1.2` published-content caveat, the third external
fixture, and the downstream cargo-cleanme handoff. The release remains
non-draft, non-prerelease, and attached to `v0.1.2`.

## Source-tag honesty

Because `0b8cb98` landed after the `v0.1.2` tag, **no** published crate is
reproducible from that tag any more. This was verified per crate and the release
notes were corrected accordingly; the previous release notes still claimed
`eggup-acquisition` and `eggup-eggfetch` "still match that tag exactly", which
stopped being true on 2026-10-04. Leaving that text would have been a false
claim on a public release page.

| Crate | Reproducible from `v0.1.2`? | Own publication commit |
|---|---|---|
| `eggup-core` | no — changed since tag | `e8e07eb` |
| `eggup-archive` | no — changed since tag | `e8e07eb` |
| `eggup-acquisition` | no — changed since tag | `02a1d32` |
| `eggup-eggfetch` | no — changed since tag | `02a1d32` |
| `eggup-eggpack` | no — changed since tag | `02a1d32` |
| `eggup-curl` | no — changed since tag | `b485228` |

The release notes now state that the tag must not be treated as a source
reference for any published artifact, and list each crate's own publication
commit. Per-crate auditability rests on `.cargo_vcs_info.json`, which is
authoritative and was read back out of the published `eggup-curl` artifact.

## Documentation/operations evidence

- `crates/eggup-curl/CHANGELOG.md` — `0.1.2` publication cut.
- root `CHANGELOG.md` — completed 0.1.2 set, `eggup-curl` entry, M005/M007/M008
  relabelling, `eggup-acquisition 0.1.2` published-content caveat.
- GitHub Release `0.1.2` notes — extended with step 3, the curl contract, the
  platform note, the acquisition caveat, and the corrected source-tag statement.
- `plans/closure/acquisition-transport/009-status.md` — this record.
- `plans/implementation/acquisition-transport/010-eggup-acquisition-0.1.3-publication.md`
  — the follow-on milestone authored from the finding below.
- `plans/subsystems/acquisition-transport-roadmap.md` and `plans/registry.md` —
  M009 closed, M010 registered, published set reconciled.
- No historical closure record for M001–M008, or any other subsystem, was edited.

## Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| medium | Published `eggup-acquisition 0.1.2` (from `02a1d32`) does not contain the two `0b8cb98` acquisition fixes. `ComposedTransport` hands the secondary adapter the caller's full `FetchLimits`, so one composed fetch can run for roughly twice the caller's documented `total_timeout`; and the owned `.part` file leaks when the permission-hardening step fails. | Reachable today by any registry consumer composing two transports. Pre-existing; **not** introduced by M009. Recorded in the root `CHANGELOG.md`, the Release `0.1.2` notes, and here. Planned as M010. `eggup-curl` consumers inherit the fix automatically through the caret requirement. |
| medium | `eggup-eggpack 0.1.2` pins `eggup-acquisition = "=0.1.2"`, so it will **not** inherit an `eggup-acquisition 0.1.3` and keeps composing against the defective seam. | Discovered while planning M010. The exact pin is intentional (M004 used it to stop the adapter graph floating onto an unreviewed seam), so the fix is an adapter republication, not a pin relaxation. Explicitly named as the M010 follow-on and as a stop condition in M010 §14. |
| low | Every published crate has moved past the `v0.1.2` Git tag, so the tag is no longer a source reference for any of them. | Recorded in the release notes with the per-crate publication commits. Tags are immutable and were not moved. |
| low | `.gitignore` gained `.DS_Store` purely so `cargo publish` could run without `--allow-dirty`. | Behavior-neutral hygiene. Recorded rather than presented as release-prep content. |
| low | docs.rs builds for the new version are an automatic registry side effect. | No action required. |
| informational | The registry-only fixture lives in scratch space outside the repository. | Its manifest, lockfile evidence, test list, and results are recorded above. Not committed. |

No medium-or-higher acquisition transport defect remains. The two medium
findings are published-state gaps in `eggup-acquisition`, not defects in
`eggup-curl`, and both are now planned.

## Downstream handoff

cargo-cleanme Phase 10 M010C is unblocked: it can now depend on
`eggup-curl = "0.1.2"` from crates.io with no Git or path dependency and no copy
of Gregg's local updater machinery. The package is small (17 KB) and has a
single seam dependency, so a curl-only consumer still carries no embedded HTTP/TLS
stack.

cargo-cleanme retains, and this milestone does not take: release and version
selection, GitHub origin, exact URLs, fallback policy, install destination, and
self-update CLI semantics.

Gregg consumer M004 remains intentionally unwritten. This publication does not
authorize or imply a Gregg migration.

`eggup-service` and `eggup-transport-footprint` remain unpublished, and no
publication of either is authorized by this milestone.

## Roadmap disposition

Acquisition Transport M009 is **closed**. The subsystem's mechanism path remains
complete through M008 and the curl path is now registry-consumable, which is
exactly the package-promotion gate M009 existed to satisfy. No transport
semantics were reopened.

M010 (`eggup-acquisition 0.1.3` publication) is **ready** and is the next
dependency-ready milestone in this subsystem. It is a publication milestone for
already-implemented, already-tested defect fixes, not new feature work.

## Registry updates

- Acquisition M009 row: ready → **closed** (this record; `eggup-curl 0.1.2`
  published, registry-only proof 6/6, no tag moved, release `0.1.2` notes
  extended).
- Acquisition M010 row: newly **registered** and **ready**
  (`plans/implementation/acquisition-transport/010-eggup-acquisition-0.1.3-publication.md`).
- Published 0.1.2 set: `eggup-core`, `eggup-archive`, `eggup-acquisition`,
  `eggup-eggfetch`, `eggup-eggpack`, `eggup-curl`.
- Acquisition transport next milestone: M010.
- Execution graph: M009 → M010 edge is traversable now; cargo-cleanme Phase 10
  M010C is unblocked externally and remains cargo-cleanme-owned.
