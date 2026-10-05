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
| `windows-check` | windows-latest | **runs tests**, then a workspace `cargo check` — see below |

`windows-check` is *not* compile-only. It runs six `cargo test` steps
(`ci.yml:48-53`) covering the platform-sensitive subset, then
`cargo check --workspace --all-targets --locked` (`ci.yml:54`):

1. `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked`
2. `cargo test -p eggup-eggpack --all-targets --all-features --locked`
3. three named `eggup-service --lib` UTF-8/control-character diagnostic-safety
   tests (`diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges`,
   `service_errors_output_and_permission_remediation_stay_utf8_bounded`,
   `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free`)
4. `cargo test -p eggup-service --lib windows_scm::tests --locked`

A full `cargo test --workspace` is deliberately *not* run on Windows; crates with
no Windows-specific behavior are covered by the closing workspace `cargo check`.

## Known coverage gaps — check these before claiming evidence

- `cargo tree` runs nowhere in CI; the boundary check is local-only.
- `msrv` and `windows-check` both omit `--all-features`. The three
  `eggup-transport-footprint` binaries are behind `required-features`, so **no
  MSRV lane and no Windows lane ever compiles them**. Only `stable` and `macos`
  build non-default features.
- clippy and fmt run on Linux only — a `#[cfg(windows)]` or `#[cfg(target_os = "macos")]`
  clippy violation is caught by no lane.
- Windows test coverage is the curated subset above; a Windows failure in an
  untargeted crate's tests would not be observed.

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
