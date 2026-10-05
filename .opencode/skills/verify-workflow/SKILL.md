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
```

`cargo tree` always exits 0 on a resolvable graph. Read its output: the
dependency-boundary rules (core has only `sha2`; acquisition has zero deps;
the two adapters never reference each other) are enforced by **human review
only** — nothing in the script or CI fails when a boundary is crossed.

## Focused runs

- `cargo test -p eggup-core`, `cargo test -p eggup-core <name>`
- `cargo clippy -p <crate> --all-targets --locked -- -D warnings`

## CI lanes (`.github/workflows/ci.yml`) — what local does NOT cover

| Job | Runner | What it adds |
|---|---|---|
| `stable` | ubuntu | fmt + clippy + full `cargo test` + `cargo doc`; the only lane running clippy, fmt, and doc |
| `msrv` | ubuntu, pinned `1.89.0` | `cargo check --workspace --all-targets --locked` — compile-only, no clippy, no tests |
| `macos` | macos-latest | full `cargo test`; the only macOS evidence |
| `windows-check` | windows-latest | **runs tests**, ending in a full `cargo test --workspace --all-targets` — see below |

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
5. `cargo test -p eggup-core --test current_executable --locked` — replaces a
   real running Windows image from a child process, both `KeepInstalled` and
   `RollBack`. A Windows claim about self-update is **only** evidenced here.
6. `cargo test -p eggup-core --test stale_lock_recovery --locked` — exercises
   the claim/race fixtures natively; `rename` semantics differ per platform, so
   compile-only evidence is not sufficient for M011.
7. `cargo test --workspace --all-targets --locked`

Step 7 exists because the per-crate steps cannot catch a regression in an
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
- `msrv` and `windows-check` both omit `--all-features`. The three
  `eggup-transport-footprint` binaries are behind `required-features`, so **no
  MSRV lane and no Windows lane ever compiles them**. Only `stable` and `macos`
  build non-default features.
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
