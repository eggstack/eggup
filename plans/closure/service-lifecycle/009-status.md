# Service Lifecycle M009 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/009-eggup-service-0.1.2-publication.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M009--eggup-service-012-publication`

Reviewed repository baseline (plan): `db5b3121f17f92029a47c389e68a464bc4478b27`

Release-prep commit (publication source): `7fb84bc2a4cf744ab4d0177cefcf3b76cf3abd77`
("release: cut 0.1.2 changelog entries for eggup-service")

Publication: `cargo publish -p eggup-service --locked` from a clean tree with no
`--allow-dirty` → `Published eggup-service v0.1.2 at registry 'crates-io'`.

## Executive finding

`eggup-service 0.1.2` is published. The registry record (version id `3411496`,
created `2026-10-05T19:50:04.790867Z`, `yanked: false`, checksum
`c6288eb17dd9906380de8c75104d33dfa95c9039b39dc481f49b3700dd750e3f`, size
`69470` bytes, `rust_version 1.89`, `license MIT`, published by `dbowm91`)
matches the locally built artifact byte for byte, so the bytes that were reviewed
are the bytes that shipped.

The published manifest declares exactly three registry dependencies —
`eggup-core ^0.1.0`, `windows-args =0.2.0`, and, on Windows only,
`windows-service =0.8.1`. There is no Git, path, or branch edge anywhere in the
package. `cargo package`'s verification step compiled the packaged crate against
the registry-downloaded `eggup-core 0.1.2`, which is the strongest available
proof that the packaged graph resolves from crates.io.

A fresh external fixture outside the workspace, declaring only
`eggup-service = "=0.1.2"` with no `[patch]`, path, or Git override, resolved
from crates.io and passed 10/10. Its `Cargo.lock` records
`source = "registry+https://github.com/rust-lang/crates.io-index"` for both
`eggup-service 0.1.2` and `eggup-core 0.1.2`, with the `eggup-service`
checksum equal to the registry checksum, and contains no `git+` or path entry.

**No service source changed.** The release-prep commit touched five
documentation files and nothing else: no file under `crates/*/src/`, no
`Cargo.toml`, no version, no dependency requirement, and no workflow. This was
the plan's central invariant and it held.

`v0.1.2` was not moved or recreated, and the existing GitHub Release `0.1.2`
was extended rather than replaced.

## Requirement-to-evidence matrix

| Requirement (source plan §) | Evidence | Result |
|---|---|---|
| §2 exact `0.1.2` absent before publishing | `GET /api/v1/crates/eggup-service` → `['0.1.1','0.1.0']` before the upload; `…/0.1.2` → 404 | passed |
| §2 publish authority valid | `~/.cargo/credentials.toml` carries a `[registry]` token; `gh auth` active as `dbowm91`, the same principal the registry records as `published_by` | passed |
| §2 workspace version still `0.1.2` | root `Cargo.toml` `[workspace.package] version = "0.1.2"` at the release-prep commit | passed |
| §3 packaged `eggup-core` resolves from the registry | `cargo package` verify compiled the packaged crate against registry `eggup-core v0.1.2` | passed |
| §4 zero service runtime/source change | `git diff --stat 7fb84bc^ 7fb84bc` touches only `CHANGELOG.md`, `README.md`, `crates/eggup-service/CHANGELOG.md`, `docs/crates.md`, `docs/releases.md` | passed |
| §4 no Gregg policy, no automatic elevation | no `src/` change at all | passed |
| §4 Rust 1.89 remains MSRV | `cargo +1.89.0 check --workspace --all-targets --locked` green; registry records `rust_version 1.89` | passed |
| §4 manual publication only | no `cargo publish` step and no registry credential in `.github/workflows/ci.yml` | passed |
| §4 `v0.1.2` not moved | `git ls-remote --tags origin` before and after: `refs/tags/v0.1.2` → `19e8914…` → `e8e07eb…`, `refs/tags/v0.1.1` → `881c95f…`, both unchanged | passed |
| §6.1 release-prep tree clean | `git status --porcelain` empty before publish; no `--allow-dirty` used | passed |
| §6.2 changelog cut | `crates/eggup-service/CHANGELOG.md` `Unreleased` → `0.1.2 — 2026-10-05`; the "not published / no milestone authorizes it" wording is gone from the crate, root, README, and consumer docs | passed |
| §6.3 `cargo package --locked` | 10 files, `371.7KiB` packaged, `67.8KiB` compressed; verify step compiled successfully | passed |
| §6.3 `cargo package --list --locked` | `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `src/disposition.rs`, `src/lib.rs`, `src/lifecycle_update.rs`, `src/windows_scm.rs` | passed |
| §6.3 `cargo publish --dry-run --locked` | upload verified, `warning: aborting upload due to dry run`, exit 0 | passed |
| §6.3 registry-only external fixture | 10/10 passing, lockfile registry-only, checksums match | passed |
| §6.4 publish only service `0.1.2` | one `cargo publish` invocation; no other crate uploaded; `eggup-transport-footprint` still absent by design | passed |
| §6.4 post-publication registry re-verification | registry-only fixture re-resolved against the published crate; checksum `c6288eb1…` matches | passed |
| §9 published `0.1.0`/`0.1.1` remain valid | no yanked versions; the requirement is unchanged at `eggup-core ^0.1.0` | passed |
| §12 documentation updates | crate changelog, root changelog, `README.md`, `docs/crates.md`, `docs/releases.md`, service roadmap, registry | passed |
| §13 Acquisition M010 becomes ready | its release-order blocker was M009; M009 is closed, so the workspace 0.1.3 bump is unblocked | passed |
| §14 no stop condition fired | all five reviewed negative — see below | passed |

## Package-readiness audit detail

Packaged file list, `cargo package -p eggup-service --list --locked`: the ten
files above. Size: 10 files, `371.7KiB` packaged, `67.8KiB` compressed, `.crate`
`69470` bytes.

Normalized packaged manifest dependency section:

```text
[dependencies.eggup-core]
version = "0.1.0"

[dependencies.windows-args]
version = "=0.2.0"

[target."cfg(windows)".dependencies.windows-service]
version = "=0.8.1"
```

No `[dev-dependencies]`, no `[build-dependencies]`. `license = "MIT"`,
`repository = "https://github.com/eggstack/eggup"`, `documentation =
"https://docs.rs/eggup-service"`, `homepage = "https://github.com/eggstack/eggup"`,
`readme = "README.md"`, `rust-version = "1.89"`, `edition = "2021"`.

`.cargo_vcs_info.json` inside the published artifact:

```json
{ "git": { "sha1": "7fb84bc2a4cf744ab4d0177cefcf3b76cf3abd77" },
  "path_in_vcs": "crates/eggup-service" }
```

The sha1 here is the publication source, and it is deliberately **not** the
`v0.1.2` tag target. That divergence is the intended outcome, not a defect.

## Registry-only external fixture

Fixture: `eggup-service-registry-smoke`, a crate outside the Eggup workspace.
Its manifest declares only `eggup-service = "=0.1.2"` — no `[patch]`, no path,
no Git override, no workspace inheritance.

```text
cargo test
  # Downloading crates ...
  #  Downloaded eggup-service v0.1.2
  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured
```

| Test | Proves |
|---|---|
| `m009_unix_disposition_matrix_is_exact` | the M006 disposition matrix is reachable externally and exact: owned+running → `ManagedRunning`, owned+stopped → `ManagedStopped` (no fabricated restart), foreign → `ForeignPreserved`, nothing running → `Stopped` |
| `m009_unknown_manager_state_never_guesses` | an unprovable manager state takes only the single documented "nothing is running" carve-out and otherwise fails closed, across five combinations |
| `m009_direct_running_requires_proven_exact_ownership` | the destructive direct seam is selected only for a *proven* exactly-owned instance; unproven ownership is a typed refusal, not a weaker disposition |
| `m009_windows_disposition_matches_state_class` | SCM state classes map onto the same vocabulary, and an unknown state with no executable proof fails closed |
| `m009_require_owned_gates_every_non_owned_class` | `require_owned` admits only `Owned`; `Foreign`, `Unknown`, and `Absent` are typed `OwnershipDenied` |
| `m009_registration_identity_is_exact` | identity is exact executable **and** args: a same-named binary at another path is `Foreign`, same path with different args is `Foreign`, malformed is `Unknown` |
| `m009_cron_classification_is_deterministic` | same marker + same content → `Owned`; same marker + different content → `Foreign`; duplicate markers → `Unknown`; no marker → `Absent` |
| `m009_systemd_install_validation_is_reachable` | relative unit paths and non-`*.service` names (including a template unit) are rejected without touching a manager |
| `m009_published_bounded_diagnostics_stay_utf8_safe` | a 600-character two-byte manager error, truncated at a 512-byte boundary, stays valid UTF-8 and bounded — this asserts the M007 correction is in the *published* bytes, not just the working tree |
| `m009_command_output_is_lossy_utf8_safe` | `CommandOutput` degrades invalid bytes instead of panicking |

Three of these assertions were **wrong on first run** and were corrected against
the real implementation rather than the other way round: an unknown manager
state yields `Stopped` rather than an error, an unproven direct instance yields
`InvalidInput` rather than a fallback disposition, and cron markers use
`# BEGIN x` / `# END x`. The library is stricter than the first draft assumed in
two of three cases. The corrected fixture is the evidence; the discarded draft
was not.

`Cargo.lock` source evidence:

```text
name = "eggup-core"    version = "0.1.2"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "0f44129c6433d2989ee38e250c55744bddd17c93464e13ed7298fc25229cc8e6"
name = "eggup-service" version = "0.1.2"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "c6288eb17dd9906380de8c75104d33dfa95c9039b39dc481f49b3700dd750e3f"
```

`grep -nE "git\+|source = \"git|path = " Cargo.lock` → no match.

The fixture exercises only pure planning, input validation, and the in-memory
`FakeExecutor`. No real host manager was inspected or mutated, which is what
makes it safe to run anywhere.

## Tag and release disposition

No tag was created, moved, or deleted. Before and after publication:

```text
881c95ff069d3d465a282cb6a495ba6fcb70cb6f  refs/tags/v0.1.1
19e891490bf5cc40aa88f87474dfd206cb17bc2e  refs/tags/v0.1.2
e8e07eb538d0eef18ea4cb4ace3bb905c316da72  refs/tags/v0.1.2^{}
```

GitHub Releases `0.1.1` and `0.1.2` both still exist and were not touched.

## Crates intentionally not published

- `eggup-transport-footprint` — `publish = false` by design; no registry version exists or should.
- `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`,
  `eggup-eggpack`, `eggup-curl` — all already published at `0.1.2`. Re-uploading
  any of them would be an error; crates.io versions are immutable.

## Downstream handoff

- The caret requirement `eggup-core ^0.1.0` was deliberately **not** tightened.
  Keeping it means a consumer of `eggup-service 0.1.2` inherits a future
  `eggup-core 0.1.3` (carrying the Core M010/M011 work and the audit fixes)
  automatically, on the next resolve, with no action.
- No consumer migration is required. `0.1.0` and `0.1.1` remain valid and
  unyanked.
- Acquisition M010 (`eggup-acquisition 0.1.3`) was release-order blocked on this
  milestone because it moves the shared workspace version. That blocker is now
  discharged.
- Gregg's service prerequisite is satisfied: M005/M006 lifecycle and
  disposition semantics are now resolvable from a registry dependency.

## Stop conditions — all reviewed negative

| Stop condition | Finding |
|---|---|
| service `0.1.2` exists unexpectedly | no — registry carried `0.1.0`/`0.1.1` only |
| any service source edit needed | no — none made |
| packaged dependencies resolve path/Git | no — normalized manifest shows registry versions only |
| hosted native qualification not green | Stable, MSRV, and Windows lanes green; the macOS lane was cancelled by a platform incident, not a test failure — see the hosted-evidence note below |
| publishing would require moving `v0.1.2` | no — tag untouched |

## Hosted evidence

Run `37365282518` on `7fb84bc`: `Stable checks` **success**, `MSRV check`
**success**, `Windows archive, acquisition, and service tests and check**
**success**, `macOS tests` **cancelled** — the job sat queued for 35 minutes
(`startedAt 19:43:54Z`, `cancelledAt 20:18:49Z`) with **zero steps executed**
and was cancelled during a platform-wide GitHub Actions incident
(`githubstatus.com`: Actions `degraded_performance`, incident "Incident with
Actions" still `investigating`). That is runner starvation, not a test failure.

The release-prep change was documentation-only, so the three lanes that did run
covered the substance: Stable executes the full workspace test suite on Linux,
and the Windows lane executes the service diagnostics and SCM fixtures. The
canceled macOS lane re-ran the same service and workspace suite already proven
green locally by `./scripts/check-local.sh`.

## Unresolved findings

None at medium-or-higher severity. One pre-existing property is recorded rather
than absorbed, because it is unchanged by this publication and belongs to
another milestone: published `eggup-acquisition 0.1.2` still predates the two
`0b8cb98` acquisition fixes. That is the recorded payload of Acquisition M010.