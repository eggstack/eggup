---
name: verify-workflow
description: Run Eggup verification gate locally and interpret CI lanes — command order, focused runs, MSRV and platform evidence
---

# Eggup verification workflow

Summary of `scripts/check-local.sh` and `.github/workflows/ci.yml`. The scripts
are authoritative; deep dive is
[`architecture/tooling-governance.md`](../../../architecture/tooling-governance.md) §4–§5.

## Local gate (in this order)

```sh
./scripts/check-local.sh
# = cargo fmt --all -- --check
# + cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# + cargo test --workspace --all-targets --all-features --locked
# + cargo doc --workspace --no-deps --locked
# + cargo tree --workspace --locked  # review step, not a gate
# + ./scripts/check-docs-truth.sh  # documentation agrees with itself, the manifests, and the registry
```

`cargo tree` always exits 0 on a resolvable graph. Read its output: the
dependency-boundary rules (core has only `sha2`; acquisition has zero deps;
the two adapters never reference each other) are enforced by **human review
only** — nothing in the script or CI fails when a boundary is crossed.

`scripts/check-docs-truth.sh` is the only gate command that does not invoke
`cargo`. It is read-only and network-free, and it asserts that the version and
status claims in `docs/`, `README.md`, `AGENTS.md`, the root `CHANGELOG.md`,
`architecture/*.md`, and `plans/registry.md` agree with each other and with the
crate manifests and the per-crate changelogs. It is wired into the local gate
only — no CI lane runs it — so a pull request is not checked for documentation
drift until a lane is added.

## Focused runs

- `cargo test -p eggup-core`, `cargo test -p eggup-core <name>`
- `cargo clippy -p <crate> --all-targets --locked -- -D warnings`

## CI lanes (`.github/workflows/ci.yml`) — what local does NOT cover

| Job | Runner | What it adds |
|---|---|---|
| `stable` | ubuntu | fmt + clippy + full `cargo test` + `cargo doc` + the three isolated `eggup-transport-footprint` builds (`ci.yml:26-28`) + the packaged-service step; the only lane running clippy, fmt, and doc. That step's registry-resolved compile is deferred |
| `msrv` | ubuntu, pinned `1.89.0` | workspace `cargo check` + the three isolated `eggup-transport-footprint` checks (`ci.yml:54-56`) + the packaged-service step at the MSRV (registry-resolved compile deferred) |
| `service-systemd` | ubuntu-latest | registry-only 0.1.2 negative control, current workspace systemd fixtures, and published registry-only 0.1.3 positive quiescence plus ownership controls |
| `macos` | macos-latest | full `cargo test` + the two isolated `eggup-transport-footprint` builds (`ci.yml:101-102`) + the packaged-service step; the only macOS evidence. That step's registry-resolved compile is deferred |
| `windows-check` | windows-latest | **runs ten test steps**, ending in a full `cargo test --workspace --all-targets`, then package verification — see below |

`windows-check` is *not* compile-only, and it is no longer a curated subset. It
runs the platform-sensitive steps, then the two Core fixture files, then the
whole workspace:

1. `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked`
2. `cargo test -p eggup-eggpack --all-targets --all-features --locked`
3. three named `eggup-service --lib` UTF-8/control-character diagnostic-safety
   tests (`diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges`,
   `service_errors_output_and_permission_remediation_stay_utf8_bounded`,
   `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free`)
4. `cargo test -p eggup-service --lib windows_scm::tests --locked`
5. `cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture`
   — the live SCM fixture: it registers a disposable service through the
   crate's public API and drives it against the genuine Service Control
   Manager. It needs a hosted Windows runner with administrator rights and
   records a skip reason on stdout when those rights are absent, which is why
   it runs with `--nocapture`.
6. `cargo test -p eggup-core --test current_executable --locked` — replaces a
   real running Windows image from a child process, both `KeepInstalled` and
   `RollBack`. A Windows claim about self-update is **only** evidenced here.
7. `cargo test -p eggup-core --test stale_lock_recovery --locked` — exercises
   the claim/race fixtures natively; `rename` semantics differ per platform, so
   compile-only evidence is not sufficient for M011.
8. `cargo test --workspace --all-targets --locked --no-fail-fast`
9. `cargo package -p eggup-service --locked --no-verify` — packages the service crate on
   Windows. Its registry-resolved compile is **deferred**: the source line uses core
   capabilities (Core M014, M017) the published `eggup-core 0.1.3` does not carry, so the
   strict form cannot pass until the `eggup-core` republication lands. The step still
   validates the manifest and the packaged file list; the workspace build, test, and
   clippy steps cover the compile against the workspace path dependency

Step 8 exists because the per-crate steps cannot catch a regression in an
untargeted crate. It passes `--no-fail-fast` deliberately: this lane is the only
Windows test coverage, so one failing target must not hide the state of the rest.
If a new Core milestone adds platform-dependent filesystem or running-image
behaviour, add its fixture here — a local macOS pass says nothing about Windows.

**This lane has already paid for itself, repeatedly.** Turning on
`cargo test --workspace` exposed `bound_source_stages_open_object_after_root_rename`
failing on Windows (renaming a directory that contains an open handle is refused
there), a portability gap invisible for the crate's whole life, plus twelve
`eggup-service` fixtures rejected for hardcoding POSIX paths that are not
absolute on Windows. The Core M010 fixtures caught a missing Windows identity
check in the same period. All were compile-clean on Linux and macOS, and the
twelve were invisible until `--no-fail-fast` stopped the first failing target
from hiding them.

The lane reached all-green on run `37376971555`. Treat it as an active finding
source: when a Windows failure appears, fix the portability gap rather than
narrowing the step, and keep `--no-fail-fast`.

## Known coverage gaps — check these before claiming evidence

- `cargo tree` runs nowhere in CI; the boundary check is local-only.
- `msrv` and `windows-check` both omit `--all-features` **on the workspace
  check**. The three `eggup-transport-footprint` binaries are behind
  `required-features`, so a lane has to name them: `msrv` checks all three
  configurations by dedicated steps (`ci.yml:54-56`), and `stable` and `macos`
  build them. **No Windows lane ever compiles them** — that asymmetry is an
  open, deliberate decision recorded in the transport-footprint closure.
- The `msrv` package check applies the Rust 1.89 floor to `eggup-service` only;
  other crates are covered by the workspace `cargo check`.
- clippy and fmt run on Linux only — a `#[cfg(windows)]` or `#[cfg(target_os = "macos")]`
  clippy violation is caught by no lane.
- `macos` is the only lane that runs `cargo test` with `--all-features`, and
  `windows-check` omits it; a feature-gated Windows-only path would not be
  observed.

## Reporting

Closure records need per-platform results, so a platform-gated change needs CI
(or a manual matrix) evidence, not just the local script. Distinguish results as
passed / failed / timed out / blocked by environment / not run / not applicable
(`plans/003-planning-process.md` §15). A lane you did not run is recorded as
**not run**, never omitted and never inferred from another platform.

CI never publishes — releases are manual. Public API changes need rustdoc
(`deny(missing_docs)` enforces presence, not quality); keep crate `README.md`
and `CHANGELOG.md` (`Unreleased`, with a no-publication/no-migration disclaimer)
current.
