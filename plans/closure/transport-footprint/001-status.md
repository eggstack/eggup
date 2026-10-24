# Transport Footprint Milestone 001 — isolated-configuration coverage corrective

Status: closed (pending the parent session's hosted CI qualification)

Source plan:
- plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md

Source roadmap:
- plans/subsystems/transport-footprint-roadmap.md

Reviewed repository baseline: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`
(branch `plans/corrective-batch-2026-10-23`)

Implementation commits / PRs: **none.** The parent session owns all git
mutations; this milestone produced working-tree changes only. No commit, tag,
branch, stash, restore, reset, clean, or publication was made. The negative
controls were executed in a scratch copy of the repository **outside** the
working tree (see "Negative-control evidence" below), so the repository's
tracked sources were never mutated.

Finding of record (closed by this milestone):
- `plans/registry.md` §"Post-0.1.3 open findings" — entry **12** (**medium**):
  the `eggup-transport-footprint` crate's central claim was verified by no
  machine, because every CI lane built only the dual configuration or no
  configuration at all.

## Executive finding

Before this milestone the `eggup-transport-footprint` crate's entire reason
for existing — that a consumer of `eggup-acquisition` can link exactly one
transport stack — was verified by **no machine**. The crate declares three
`required-features`-gated binaries (`curl_only`, `eggfetch_only`, `dual`), but
the five CI lanes built only `--all-features` (the dual configuration) or no
features at all, and the local gate did the same. The one configuration the
crate exists to represent was the one configuration nothing built. A plain
cross-link would therefore have been invisible to every lane.

This milestone closes that gap in two halves, because the isolated builds
alone are not a complete guard (plan §3.6, empirically re-confirmed below):

1. **CI now builds the isolated configurations.** `stable` builds all three,
   `msrv` checks all three at the 1.89.0 floor, and `macos` builds the two
   isolated ones. `windows-check` and `service-systemd` are **unmodified**.
2. **A crate-side guard now rejects a sibling-adapter reference in every
   configuration**, including a `#[cfg(feature = "...")]`-gated one that the
   isolated builds provably cannot see. The guard is
   `crates/eggup-transport-footprint/tests/cross_link_guard.rs`, an
   integration-test target with no `required-features`, so it is built and run
   in **every** feature configuration including `default = []`. It uses `std`
   only and adds no dependency, so `Cargo.lock` does not change.

Both guards were **observed to fail** on deliberately introduced defects, in a
scratch copy of the repository outside the working tree. The discriminating
case — a `#[cfg]`-gated cross-link — produced **green isolated builds and a
red guard**, which is exactly the hole the CI-only approach leaves open.

The recorded dependency counts **3 / 92 / 93 reproduce exactly**, before and
after. The crate remains `publish = false`, no manifest changed, no published
crate's dependency graph changed, and no consumer migration is required.

## Requirement → evidence matrix

One row per acceptance criterion in the plan's §13. All commands were run from
`/Users/davidbowman/projects/eggup` with `--offline` (no network), on **macOS
(darwin 25.6.0, arm64), Rust 1.89.0 / cargo 1.89.0 / clippy 0.1.89** — the
workspace MSRV toolchain itself, so the local runs are MSRV-equivalent.
Per-platform reporting is in the next section; no platform is inferred from
another.

| # | Requirement (plan §) | Evidence (file:line) | Command and observed result |
|---|---|---|---|
| 1 | The `stable` lane builds all three configurations with `--locked`, as three separate steps (§13.1) | `.github/workflows/ci.yml:26`, `:27`, `:28` (comment at `:22-25`) | `cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline` → **exit 0**; `--features eggfetch` → **exit 0**; `--features curl,eggfetch` → **exit 0**. YAML parse check (`python3 -c "import yaml; ..."`) → `stable -> 10 steps`, the three `cargo build -p eggup-transport-footprint` runs listed in order |
| 2 | The `msrv` lane checks all three configurations at 1.89.0 with `--locked`, as three separate steps (§13.2) | `.github/workflows/ci.yml:43`, `:44`, `:45` (comment at `:39-42`); toolchain pinned at `:37` (`dtolnay/rust-toolchain@1.89.0`) | `cargo check -p eggup-transport-footprint --no-default-features --features curl --locked --offline` → **exit 0**; `--features eggfetch` → **exit 0**; `--features curl,eggfetch` → **exit 0**. YAML parse check → `msrv -> 7 steps`, the three `cargo check -p eggup-transport-footprint` runs listed in order |
| 3 | The `macos` lane builds the curl-only and eggfetch-only configurations with `--locked`, as two separate steps (§13.3) | `.github/workflows/ci.yml:79`, `:80` (comment at `:76-78`); the dual configuration is already built by the lane's `--all-features` step at `:75` | `cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline` → **exit 0**; `--features eggfetch` → **exit 0**. YAML parse check → `macos -> 6 steps`, the two `cargo build -p eggup-transport-footprint` runs listed |
| 4 | The `windows-check` lane and the `service-systemd` lane are unmodified; Windows binary coverage is recorded as `not run` (§13.4) | `.github/workflows/ci.yml:48-65` (`service-systemd`) and `:83-110` (`windows-check`) — byte-identical to `20ac770` | `git diff .github/workflows/ci.yml` → the diff hunks are at `@@ -19,6 +19,13 @@` (stable), `@@ -29,6 +36,13 @@` (msrv), `@@ -59,6 +73,11 @@` (macos) only; no hunk touches `windows-check` or `service-systemd`. YAML parse check → `service-systemd -> 9 steps`, `windows-check -> 13 steps`, neither containing an `eggup-transport-footprint` run |
| 5 | `crates/eggup-transport-footprint/tests/cross_link_guard.rs` exists and contains exactly the two guards of §10.1, using `std` only, with no `unsafe` block (§13.5) | The file, 381 lines. T1 at `:65-93`; T2 at `:230-381`. `use std::fs;` at `:29`, `use std::path::{Path, PathBuf};` at `:30`. No `unsafe` token in the file | `grep -c unsafe crates/eggup-transport-footprint/tests/cross_link_guard.rs` → **0**; the only two `use` statements are `std::fs` (`:29`) and `std::path` (`:30`). `cargo clippy -p eggup-transport-footprint --all-targets --all-features --locked --offline -- -D warnings` → **exit 0**, no warnings (`unsafe_code = "deny"` is inherited from the workspace lint table, `[workspace.lints.rust] unsafe_code = "deny"` in the root `Cargo.toml:21-22`) |
| 6 | `cargo test -p eggup-transport-footprint --all-targets --locked --offline` reports **2 passed**, and the `no targets matched` warning is gone (§13.6) | The test target at `crates/eggup-transport-footprint/tests/cross_link_guard.rs`; the crate's manifest has no `[[test]]` entry — the `tests/` target is auto-discovered and carries no `required-features` | `cargo test -p eggup-transport-footprint --all-targets --locked --offline` → `running 2 tests` / `test manifest_keeps_the_shape_that_produces_the_recorded_counts ... ok` / `test single_adapter_fixtures_never_reference_the_sibling_adapter ... ok` / `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. The baseline warning was reproduced and is now gone: with the `tests/` directory absent, the same command printed `warning: target filter \`all-targets\` specified, but no targets matched; this is a no-op`; with it present, the warning does not appear |
| 7 | The three dependency-count commands print **3**, **92**, **93**, in that order (§13.7) | `crates/eggup-transport-footprint/Cargo.toml:15-31` — unchanged by this milestone | `cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none \| sort -u \| grep -v '(\*)$' \| wc -l` → **3**; `--features eggfetch` → **92**; `--features curl,eggfetch` → **93**. Identical before and after the change set — see "Dependency-count verification" below |
| 8 | NC-1, NC-2, and NC-3 were each executed, each failed in the expected way with the exact output captured, and the tree was restored afterwards (§13.8) | Scratch copy at `/tmp/eggup-tf-nc-67387`, outside the repository; negative-control evidence section below | NC-1: isolated build **failed** with `error[E0433]: failed to resolve: use of unresolved module or unlinked crate \`eggup_eggfetch\``, dual build **succeeded** (exit 0), guard **failed**. NC-2: isolated build **succeeded** (exit 0), guard **failed**. NC-3: guard **failed** naming the missing feature. Restore verified by `diff -r` (scratch crate identical to the repository crate) and by `git status --short crates/eggup-transport-footprint/` showing only the new untracked `tests/` path |
| 9 | `git diff --name-only` shows no path under `crates/*/src/**`, no `Cargo.toml`, and no `Cargo.lock`; `git diff --check` is clean (§13.9) | See "Production implementation evidence" below | `git diff --check` → **exit 0**, no output. `git diff --name-only -- Cargo.lock` → **empty**. `git diff --name-only -- 'crates/*/Cargo.toml'` → **empty**. `git diff --name-only -- crates/eggup-transport-footprint/` → **empty** (no tracked footprint-crate path changed) |
| 10 | `cargo fmt -p eggup-transport-footprint -- --check` exits 0, and the three crate-scoped clippy runs exit 0 with `-D warnings` (§13.10) | Plan §11.1 | `cargo fmt -p eggup-transport-footprint -- --check` → **exit 0**, no output. `cargo clippy -p eggup-transport-footprint --all-targets --all-features --locked --offline -- -D warnings` → **exit 0**. `… --all-targets --no-default-features --features curl --locked --offline -- -D warnings` → **exit 0**. `… --all-targets --no-default-features --features eggfetch --locked --offline -- -D warnings` → **exit 0**. No warnings emitted in any of the four runs |
| 11 | The workspace-wide gate (§11.2) is green on the parent session's run, accompanied by a read of the three scoped `cargo tree` runs (§13.11) | **Parent session's responsibility** — plan §11.2 states these are not the implementing session's, because a parallel milestone may have in-flight edits in the same tree | **not run** by this milestone. The three scoped `cargo tree` runs this milestone *did* produce are recorded in "Dependency-count verification" below, so the parent's review step has them. The parent must still run the workspace-wide gate and record its output |
| 12 | The documentation changes in §12.2 are applied by the parent session, and the accurate cross-link claim at `architecture/transport-footprint.md:177` survives unmodified in substance (§13.12) | **Parent-owned.** Current anchors recorded in "Documentation / operations evidence" below | **not applied** by this milestone — `architecture/**`, `AGENTS.md`, `plans/registry.md`, `plans/subsystems/README.md`, the root `CHANGELOG.md`, and `plans/003-planning-process.md` are parent-owned in this batch (plan §12.1, §14 stop condition 10). The intended prose is drafted below under "Registry updates" and "Roadmap disposition". `architecture/transport-footprint.md:177` was **read and not modified** by this milestone |
| 13 | No publication, no tag movement, no version bump, and no `publish = false` → `publish = true` change occurred (§13.13) | `crates/eggup-transport-footprint/Cargo.toml:8` — unchanged (`publish = false`); `crates/eggup-transport-footprint/Cargo.toml:3` (`version.workspace = true`) — unchanged | `git diff --name-only -- 'crates/*/Cargo.toml'` → **empty**. `git tag --list` → unchanged by this milestone (no tag command was run at all). No `cargo publish`, no `cargo package` of this crate, no version edit |
| 14 | The closure record exists at `plans/closure/transport-footprint/001-status.md` and satisfies §15 (§13.14) | This file | `ls plans/closure/transport-footprint/001-status.md` → the file exists; its sections cover every item plan §15 requires |

## Production implementation evidence

The change set is **two files** plus this closure record and the plan's
`Status:` header.

| Change | Location | Notes |
|---|---|---|
| New guard test target | `crates/eggup-transport-footprint/tests/cross_link_guard.rs` (new, 381 lines) | T1 at `:65-93`, T2 at `:230-381`. `std` only (`:29-30`). Auto-discovered integration test; **no** manifest entry and **no** dev-dependency were added. The test target carries no `required-features`, which is what makes it run in every configuration including `default = []` |
| `stable` lane: three isolated-configuration builds | `.github/workflows/ci.yml:26-28` (comment `:22-25`) | `cargo build` with `--no-default-features --features curl` / `eggfetch` / `curl,eggfetch`, each `--locked`, each its own step |
| `msrv` lane: three isolated-configuration checks | `.github/workflows/ci.yml:43-45` (comment `:39-42`) | `cargo check`, matching the lane's existing type-check style; `--locked` |
| `macos` lane: two isolated-configuration builds | `.github/workflows/ci.yml:79-80` (comment `:76-78`) | `cargo build` with `--features curl` and `--features eggfetch`, `--locked`. The dual configuration is already covered by the lane's existing `--all-features` step at `:75` |
| Plan status header | `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md:3` | `Status: ready for handoff` → `Status: implemented` |
| Plan factual corrections | same file, §3.6 (after the three-row table), §8.2 S2 row, §10.2 NC-3 | Three observations recorded below under "Plan corrections". No decision was reversed and no section was restructured |

### Change-set proof

```text
$ git diff --name-only -- Cargo.lock
(empty)

$ git diff --name-only -- 'crates/*/Cargo.toml'
(empty)

$ git diff --name-only -- crates/eggup-transport-footprint/
(empty)

$ git status --short --untracked-files=all crates/eggup-transport-footprint/
?? crates/eggup-transport-footprint/tests/cross_link_guard.rs

$ git diff --check
(exit 0, no output)
```

**This milestone's tracked-file diff is exactly `.github/workflows/ci.yml`.**
Its only other file is the new, untracked
`crates/eggup-transport-footprint/tests/cross_link_guard.rs`.

**No `crates/*/src/**` path, no `Cargo.toml`, and no `Cargo.lock` changed.**
`crates/eggup-transport-footprint/src/bin/curl_only.rs`,
`src/bin/eggfetch_only.rs`, and `src/bin/dual.rs` are byte-identical to
`20ac770` — the negative controls that touched them ran in a scratch copy
outside the repository, and the repository's tracked sources were never
mutated. `git diff --name-only -- crates/eggup-transport-footprint/` returns
empty, which is the proof.

**Concurrent-writer note.** The working tree also carries uncommitted work
owned by **other, concurrent milestones** — `crates/eggup-core/src/candidate.rs`
and `crates/eggup-core/src/lib.rs`, `crates/eggup-curl/src/lib.rs`,
`crates/eggup-service/src/lib.rs` (which appeared *during* this milestone's
run, confirming a live concurrent writer), four `architecture/*.md` files,
the root `CHANGELOG.md`, `plans/registry.md`, and three `plans/subsystems/*`
roadmaps. Those files were **not** touched, edited, staged, or depended on by
this milestone. They are the reason every verification command here is
crate-scoped and `--offline`, and the reason the workspace-wide gate was left
to the parent session.

## Negative-control evidence

**Method.** Every control ran in a scratch copy of the repository at
`/tmp/eggup-tf-nc-67387` — created with `rsync -a --exclude target --exclude
.git` from the working tree, built with a separate
`CARGO_TARGET_DIR=/tmp/eggup-tf-nc-target`, and run with `--locked --offline`.
**The repository's own working tree was never mutated**, so a concurrent
writer could not observe a deliberate cross-link. The scratch copy was
restored and re-verified afterwards, and then discarded.

The scratch copy's baseline was itself verified first: with the guard file
present, `cargo test -p eggup-transport-footprint --all-targets --locked
--offline` → `2 passed; 0 failed`.

### NC-1 — the isolated build catches a plain cross-link

**Mutation** (scratch `crates/eggup-transport-footprint/src/bin/curl_only.rs`),
in two forms so that both name-resolution error codes the plan's §8.2 S2
signature can produce were observed:

*Form A — a `use` statement:*

```rust
use eggup_curl::{CurlConfig, CurlTransport};
#[allow(unused_imports)]
use eggup_eggfetch::EggfetchTransport;
```

```text
$ cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline
   Compiling eggup-transport-footprint v0.1.3 (/private/tmp/eggup-tf-nc-67387/crates/eggup-transport-footprint)
error[E0432]: unresolved import `eggup_eggfetch`
 --> crates/eggup-transport-footprint/src/bin/curl_only.rs:6:5
  |
6 | use eggup_eggfetch::EggfetchTransport;
  |     ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `eggup_eggfetch`
  |
  = help: if you wanted to use a crate named `eggup_eggfetch`, use `cargo add eggup_eggfetch` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `eggup-transport-footprint` (bin "curl_only") due to 1 previous error
curl-only exit=101
```

*Form B — a path expression:*

```rust
fn main() {
    let _ = std::any::type_name::<eggup_eggfetch::EggfetchTransport>();
```

```text
$ cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `eggup_eggfetch`
 --> crates/eggup-transport-footprint/src/bin/curl_only.rs:7:35
  |
7 |     let _ = std::any::type_name::<eggup_eggfetch::EggfetchTransport>();
  |                                   ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `eggup_eggfetch`
  |
  = help: if you wanted to use a crate named `eggup_eggfetch`, use `cargo add eggup_eggfetch` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `eggup-transport-footprint` (bin "curl_only") due to 1 previous error
curl-only exit=101
```

**The dual build succeeds on the same tree** — the empirical disproof of the
old "cross-link is a build error" claim:

```text
$ cargo build -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline
   Compiling eggup-eggfetch v0.1.3 (/private/tmp/eggup-tf-nc-67387/crates/eggup-eggfetch)
   Compiling eggup-transport-footprint v0.1.3 (/private/tmp/eggup-tf-nc-67387/crates/eggup-transport-footprint)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.54s
dual exit=0
```

**The guard also catches it**, in the same configuration:

```text
$ cargo test -p eggup-transport-footprint --all-targets --locked --offline
running 2 tests
test manifest_keeps_the_shape_that_produces_the_recorded_counts ... ok
test single_adapter_fixtures_never_reference_the_sibling_adapter ... FAILED

---- single_adapter_fixtures_never_reference_the_sibling_adapter stdout ----
thread 'single_adapter_fixtures_never_reference_the_sibling_adapter' panicked at crates/eggup-transport-footprint/tests/cross_link_guard.rs:88:5:
cross-link guard failed for eggup-transport-footprint:
  - src/bin/curl_only.rs references eggup_eggfetch; the single-adapter fixture must link one adapter only

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### NC-2 — the guard catches a `#[cfg]`-gated cross-link the isolated build cannot see

**This is the discriminating control.** **Mutation** (scratch
`src/bin/curl_only.rs`):

```rust
use eggup_curl::{CurlConfig, CurlTransport};
#[cfg(feature = "eggfetch")]
use eggup_eggfetch::EggfetchTransport;
```

**The isolated builds are GREEN** — the `cfg` strips the reference before name
resolution:

```text
$ cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline
   Compiling eggup-transport-footprint v0.1.3 (/private/tmp/eggup-tf-nc-67387/crates/eggup-transport-footprint)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
curl-only exit=0

$ cargo build -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline
   Compiling eggup-transport-footprint v0.1.3 (/private/tmp/eggup-tf-nc-67387/crates/eggup-transport-footprint)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.58s
eggfetch-only exit=0
```

**The guard is RED**, naming the file and the token:

```text
$ cargo test -p eggup-transport-footprint --all-targets --locked --offline
running 2 tests
test manifest_keeps_the_shape_that_produces_the_recorded_counts ... ok
test single_adapter_fixtures_never_reference_the_sibling_adapter ... FAILED

---- single_adapter_fixtures_never_reference_the_sibling_adapter stdout ----
thread 'single_adapter_fixtures_never_reference_the_sibling_adapter' panicked at crates/eggup-transport-footprint/tests/cross_link_guard.rs:88:5:
cross-link guard failed for eggup-transport-footprint:
  - src/bin/curl_only.rs references eggup_eggfetch; the single-adapter fixture must link one adapter only

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Green isolated builds + a red guard** is the plan's §8.2 **S3** signature,
and it is the reason the milestone is CI **plus** a crate-side guard rather
than CI alone.

### NC-3 — the manifest guard catches a dropped feature

**Mutation** (scratch `crates/eggup-transport-footprint/Cargo.toml`): the
`curl = ["dep:eggup-curl"]` feature line was deleted.

The first attempt showed that the control must be constructed so that the
package stays **buildable**, or the guard test never runs. Two cargo
behaviors were observed and are recorded in the plan's §10.2:

- Cargo treats a `required-features` entry naming a missing feature as a
  **warning**, not an error:
  `warning: invalid feature \`curl\` in required-features of target \`dual\`: \`curl\` is not present in [features] section`.
- `src/bin/*.rs` are **auto-discovered**, so deleting the `[[bin]]` entry for
  `curl_only` does not remove the target — it becomes an auto-discovered binary
  with **no** `required-features`, which is then built in every configuration
  and fails to compile (`error[E0432]: unresolved import \`eggup_curl\``)
  before the guard test can run.

With the auto-discovered bin moved aside so only the guard target builds, T2
fails and names the missing feature:

```text
$ cargo test -p eggup-transport-footprint --all-targets --locked --offline
running 2 tests
test manifest_keeps_the_shape_that_produces_the_recorded_counts ... FAILED
test single_adapter_fixtures_never_reference_the_sibling_adapter ... FAILED

---- manifest_keeps_the_shape_that_produces_the_recorded_counts stdout ----
thread 'manifest_keeps_the_shape_that_produces_the_recorded_counts' panicked at crates/eggup-transport-footprint/tests/cross_link_guard.rs:376:5:
manifest-shape guard failed for eggup-transport-footprint:
  - Cargo.toml [features] must declare curl; it is absent
  - Cargo.toml must declare exactly three [[bin]] targets; found 2
  - Cargo.toml must declare a [[bin]] named curl_only

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

A second, more surgical variant was also run — the feature body emptied to
`curl = []`, with the `[[bin]]` entry left intact — and T2 caught the wrong
feature body:

```text
---- manifest_keeps_the_shape_that_produces_the_recorded_counts stdout ----
thread 'manifest_keeps_the_shape_that_produces_the_recorded_counts' panicked at crates/eggup-transport-footprint/tests/cross_link_guard.rs:376:5:
manifest-shape guard failed for eggup-transport-footprint:
  - Cargo.toml [features] curl must be ["dep:eggup-curl"]; found []
```

### Restore verification

```text
$ diff -r /Users/davidbowman/projects/eggup/crates/eggup-transport-footprint \
         /tmp/eggup-tf-nc-67387/crates/eggup-transport-footprint
(no output — the scratch crate is identical to the repository crate)

$ cd /tmp/eggup-tf-nc-67387 && cargo test -p eggup-transport-footprint --all-targets --locked --offline
running 2 tests
test manifest_keeps_the_shape_that_produces_the_recorded_counts ... ok
test single_adapter_fixtures_never_reference_the_sibling_adapter ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cd /Users/davidbowman/projects/eggup && git status --short crates/eggup-transport-footprint/
?? crates/eggup-transport-footprint/tests/
```

The guard passes again after the restore, and the repository's working tree
contains **no** deliberate cross-link — the only footprint-crate path present
is the new untracked `tests/` directory.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` with `--offline`
(no network) on **macOS (darwin 25.6.0, arm64)**, toolchain **rustc 1.89.0 /
cargo 1.89.0 / clippy 0.1.89** — the workspace MSRV itself.

| # | Command | Result |
|---|---|---|
| 1 | `cargo fmt -p eggup-transport-footprint -- --check` | **exit 0**, no output |
| 2 | `cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.02s` |
| 3 | `cargo build -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.13s` |
| 4 | `cargo build -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.04s` |
| 5 | `cargo clippy -p eggup-transport-footprint --all-targets --all-features --locked --offline -- -D warnings` | **exit 0** — `Finished \`dev\` profile … in 1.55s`; no warnings |
| 6 | `cargo clippy -p eggup-transport-footprint --all-targets --no-default-features --features curl --locked --offline -- -D warnings` | **exit 0** — `Finished \`dev\` profile … in 0.14s`; no warnings |
| 7 | `cargo clippy -p eggup-transport-footprint --all-targets --no-default-features --features eggfetch --locked --offline -- -D warnings` | **exit 0** — `Finished \`dev\` profile … in 0.17s`; no warnings |
| 8 | `cargo test -p eggup-transport-footprint --all-targets --locked --offline` | **exit 0** — `running 2 tests` → `ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| 9 | `cargo test -p eggup-transport-footprint --all-targets --all-features --locked --offline` | **exit 0** — `ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| 10 | `cargo check -p eggup-transport-footprint --no-default-features --features curl --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.20s` |
| 11 | `cargo check -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.10s` |
| 12 | `cargo check -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.12s` |
| 13 | `cargo tree -p eggup-transport-footprint --locked --offline --depth 1` | `eggup-transport-footprint v0.1.3 (…)` → `└── eggup-acquisition v0.1.3 (…)` — **one unconditional dependency**, no `eggup-core`/`eggup-service`/`eggup-archive`/`eggup-eggpack` |
| 14 | `cargo tree -p eggup-curl --locked --offline --depth 1` | `eggup-curl v0.1.3` → `└── eggup-acquisition v0.1.3` — **one dependency**, no `eggup-eggfetch` |
| 15 | `cargo tree -p eggup-eggfetch --locked --offline --depth 1` | `eggup-eggfetch v0.1.3` → `eggfetch-core`, `eggup-acquisition`, `futures-util`, `tokio` — **no `eggup-curl`** |
| 16 | `python3 -c "import yaml; …"` (CI workflow parse) | **exit 0** — `stable -> 10 steps`, `msrv -> 7 steps`, `service-systemd -> 9 steps`, `macos -> 6 steps`, `windows-check -> 13 steps`; the eight new `eggup-transport-footprint` runs listed, three in `stable`, three in `msrv`, two in `macos`, **none** in `service-systemd` or `windows-check` |
| 17 | `git diff --check` | **exit 0**, no output |
| 18 | `git status --short`, `git diff --name-only` | See "Change-set proof" and the concurrent-writer note |

**The workspace-wide gate (`./scripts/check-local.sh`, and the §11.2 commands)
was NOT run by this milestone.** It is the parent session's responsibility
(plan §11.2), because a parallel milestone has in-flight edits in the same
tree. It is recorded here as **not run**, never as green.

### Test counts

| Target | Tests | Result |
|---|---|---|
| `tests/cross_link_guard.rs` | 2 (T1 `single_adapter_fixtures_never_reference_the_sibling_adapter`, T2 `manifest_keeps_the_shape_that_produces_the_recorded_counts`) | 2 passed, 0 failed |
| **Total** | **2** | **2 passed, 0 failed** |

The crate went from **zero tests** (and a `no targets matched` warning) to
**two guard tests** that run in every feature configuration.

### Dependency-count verification

The counting method is the one recorded by planning C013 (unique tree lines,
`(*)` dedup markers excluded, `--locked --offline`):

```sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
```

| Configuration | Count **before** the change set | Count **after** the change set | Changed? |
|---|---|---|---|
| `--no-default-features --features curl` | **3** | **3** | **no** |
| `--no-default-features --features eggfetch` | **92** | **92** | **no** |
| `--no-default-features --features curl,eggfetch` | **93** | **93** | **no** |

**The recorded counts 3 / 92 / 93 did not change.** The "before" column was
measured at the baseline, before any file in this milestone was written; the
"after" column was measured after the guard file and the CI edits were in
place. The counts are a property of the manifest, which this milestone did not
touch — which is exactly what T2 pins.

## Per-platform results

Reported separately, as required. **No platform is inferred from another**
(`plans/closure/README.md`; `architecture/tooling-governance.md` §5.2).

| Platform / lane | Footprint **binaries** built | Guard **test** executed | Dependency counts claimed | Status |
|---|---|---|---|---|
| **macOS** (darwin 25.6.0, arm64; local, rustc 1.89.0) | **all three** configurations built — curl-only, eggfetch-only, dual (`cargo build`, commands 2-4 above) **plus** all three `cargo check`ed (10-12) | **executed** — `2 passed; 0 failed` (command 8), and again under `--all-features` (command 9) | **3 / 92 / 93** — measured on this platform, before and after | **Run — green.** This is the platform on which the recorded counts were captured. |
| **Linux** (`stable`) | **not run** by this milestone — the hosted `stable` lane will build all three (`ci.yml:26-28`) | **not run** — the hosted lane's `cargo test --workspace --all-targets --all-features --locked` will run it | **not claimed** — the counts were not measured on Linux | **Not run.** No Linux host was available in this session. Nothing is inferred from the macOS run. The hosted lane is the parent's qualification. |
| **Linux** (`msrv`, 1.89.0) | **not run** — the hosted `msrv` lane will `cargo check` all three (`ci.yml:43-45`) | **not run** — the `msrv` lane runs `cargo check`, not `cargo test`, so the guard is *compiled* there but not *executed* | **not claimed** — the counts are a resolution result, not a toolchain result | **Not run.** No Linux host was available in this session. |
| **Linux** (`service-systemd`) | **not run** — the lane never builds the workspace and was **not modified** | **not run** — the lane never builds the workspace | **not claimed** | **Not run**, and **unchanged by design** (plan §6.3). |
| **macOS** hosted (`macos` lane) | **not run** — the hosted lane will build curl-only and eggfetch-only (`ci.yml:79-80`); the dual configuration is already covered by the lane's `--all-features` step | **not run** — the hosted lane's `cargo test --workspace --all-targets --all-features --locked` will run it | **not claimed** from the hosted run; the local macOS run above is the measured capture | **Not run** locally as a hosted lane. The local macOS run is reported above and is **not** a substitute for the hosted qualification. |
| **Windows** (`windows-check`) | **none — `not run`**, before this milestone and after it. The lane was **not modified** (plan §6.3: the crate has no `#[cfg]` attribute and no platform-specific code, so a Windows build proves nothing about the property being guarded) | **not run** — the hosted lane's `cargo test --workspace --all-targets --locked --no-fail-fast` (`ci.yml:109`) *will* execute it, because the new test target has no `required-features` and the guard is platform-independent by construction (it reads text files) | **not claimed** — the transitive crate set of `eggup-eggfetch` is platform-dependent, and the recorded 3 / 92 / 93 are the macOS capture | **Not run.** No Windows host was available in this session. **No coverage was removed**: the status quo is preserved and stated, not inferred. |

## Invariant review

Plan §4's eleven invariants, each marked intact or breached.

| # | Invariant | Result | Evidence |
|---|---|---|---|
| 1 | `publish = false` stays at `Cargo.toml:8` | **Intact** | `crates/eggup-transport-footprint/Cargo.toml:8` still reads `publish = false`; `git diff --name-only -- 'crates/*/Cargo.toml'` → empty; T2 asserts it (`cross_link_guard.rs:237-250`) and passes |
| 2 | The dependency counts **3 / 92 / 93** reproduce exactly under the §3.5 method | **Intact** | See "Dependency-count verification" — 3 / 92 / 93 before and after |
| 3 | The `required-features` split survives: `curl_only` requires `curl`, `eggfetch_only` requires `eggfetch`, `dual` requires both, `default` stays empty | **Intact** | `crates/eggup-transport-footprint/Cargo.toml:33-46` unchanged; T2 asserts all four conditions (`cross_link_guard.rs:299-356`) and passes; the three isolated builds and checks in commands 2-4 and 10-12 prove the split is live |
| 4 | The crate gains no dependency on `eggup-core`, `eggup-service`, `eggup-archive`, or `eggup-eggpack`; `eggup-acquisition` stays the only unconditional dependency; `eggup-curl` and `eggup-eggfetch` stay optional | **Intact** | `cargo tree -p eggup-transport-footprint --locked --offline --depth 1` → exactly `eggup-acquisition`; T2 asserts it (`cross_link_guard.rs:252-278`, `:358-374`) and passes; `Cargo.lock` unchanged |
| 5 | `eggup-curl` does not learn that `eggup-eggfetch` exists, and `eggup-eggfetch` does not learn that `eggup-curl` exists | **Intact** | `cargo tree -p eggup-curl --locked --offline --depth 1` → only `eggup-acquisition`; `cargo tree -p eggup-eggfetch --locked --offline --depth 1` → no `eggup-curl`. Neither adapter crate was touched by this milestone (`git diff --name-only -- crates/eggup-curl/ crates/eggup-eggfetch/` shows only `crates/eggup-curl/src/lib.rs`, modified by the **concurrent** Acquisition M014 milestone) |
| 6 | The three binaries perform no measurement, select no transport, and assert no transport policy; `curl_only` keeps its hardcoded `/usr/bin/curl` fixture value | **Intact** | `crates/eggup-transport-footprint/src/bin/*.rs` are byte-identical to `20ac770`; `git diff --name-only -- crates/eggup-transport-footprint/` → empty |
| 7 | No published crate's dependency graph changes; `eggup-eggpack`'s exact `=0.1.3` pins are untouched | **Intact** | `git diff --name-only -- Cargo.lock` → empty; `git diff --name-only -- 'crates/*/Cargo.toml'` → empty. The new test adds no dependency and therefore no lockfile edge |
| 8 | `Cargo.lock` does not change; the new test uses only `std` | **Intact** | `git diff --name-only -- Cargo.lock` → empty; the test file's only imports are `std::fs` and `std::path` (`cross_link_guard.rs:29-30`) |
| 9 | The local gate `scripts/check-local.sh` keeps its shape | **Intact** | `scripts/check-local.sh` was read and **not** modified (`git diff --name-only -- scripts/` → empty). The guard runs inside the gate's existing `cargo test --workspace --all-targets --all-features --locked` step, so local coverage is preserved with no new gate line |
| 10 | The accurate cross-link claim from `architecture/transport-footprint.md:177` survives; the old overclaim does not return | **Intact** | `architecture/transport-footprint.md` was read and **not** modified by this milestone. The old overclaim is not present in any file this milestone wrote: the guard's module doc states the `required-features` limitation explicitly (`cross_link_guard.rs:5-9`) |
| 11 | `unsafe_code = "deny"` holds in the new test file through the workspace lint table; the new test contains no `unsafe` block | **Intact** | No `unsafe` token in `crates/eggup-transport-footprint/tests/cross_link_guard.rs`; all four clippy runs pass with `-D warnings`, and the workspace lint table (`[workspace.lints.rust] unsafe_code = "deny"`, root `Cargo.toml:21-22`) is inherited through the crate's `[lints] workspace = true` (`crates/eggup-transport-footprint/Cargo.toml:12-13`) |

## Failure, rollback, and recovery review

The plan's §8.2 three signatures, with the observed texts.

| Signature | Observed in this milestone? | Evidence |
|---|---|---|
| **S1 — feature-resolution failure.** Cargo rejects the *requested* feature before compiling anything; the message names a **feature**; all three footprint steps fail identically at the resolver stage. | **Not observed.** No feature was renamed or misspelled by this milestone, and no such failure was induced. It is a **manifest/configuration defect** signature and is recorded as a discriminator only. | The S1 discriminator is nevertheless **partially exercised**: NC-3 induced a feature-level defect and observed the cargo behavior — `warning: invalid feature \`curl\` in required-features of target \`dual\`: \`curl\` is not present in [features] section` — which is a *warning*, not a resolver error, in cargo 1.89. That observation is recorded in the plan's §10.2 |
| **S2 — plain cross-link.** The feature resolves; the build fails with a name-resolution error naming the **sibling adapter**; exactly one footprint step fails; the **dual** step succeeds. | **Observed**, twice, in two cross-link forms. | `error[E0432]: unresolved import \`eggup_eggfetch\`` (use form) and `error[E0433]: failed to resolve: use of unresolved module or unlinked crate \`eggup_eggfetch\`` (path form), while the dual build returned `dual exit=0`. See NC-1 above |
| **S3 — cfg-gated cross-link.** The isolated builds **all succeed**, and the guard test **fails** with a message naming the offending file and the forbidden token. | **Observed.** | NC-2: `curl-only exit=0`, `eggfetch-only exit=0`, and `test single_adapter_fixtures_never_reference_the_sibling_adapter ... FAILED` with `src/bin/curl_only.rs references eggup_eggfetch; the single-adapter fixture must link one adapter only` |

**Discriminator, as observed:** a resolver error naming a *feature* is a
manifest defect; a resolver-success / name-resolution error naming the
*sibling adapter* is a plain cross-link (S2); green isolated builds plus a
red guard is a cfg-gated cross-link (S3).

**Failure semantics of the guard itself.** The guard is deterministic,
hermetic, and offline: it reads two text files from `CARGO_MANIFEST_DIR` and
compares them. It has no timing, no network, no process spawn, and no
filesystem writes. It cannot flake. Its failure message names the file path
and the forbidden token, so a failure is actionable without opening the file.

**CI failure semantics.** A failing footprint step fails its lane; GitHub
Actions stops the job at the first failing step, so the lane is red and its
remaining steps do not run. There is no `continue-on-error`, no `|| true`, and
no warning-only mode. The three configurations are three **separate** steps
per lane, so the CI UI names the failing configuration rather than hiding it
in a loop.

**Cancellation, restart, and contention.** There is nothing to cancel,
restart, or arbitrate: the milestone changes no runtime behavior, holds no
lock, spawns no process, writes no file outside the repository working tree,
and touches no shared resource. `cargo build` and `cargo test` invocations are
independent and re-runnable. The one shared resource is the repository working
tree, which is why every verification command here is crate-scoped and
`--offline`, and why the negative controls ran in a scratch copy.

**Rollback.** Reverting this milestone is a two-file revert: delete
`crates/eggup-transport-footprint/tests/cross_link_guard.rs` and drop the
eight CI steps. Nothing else depends on either.

## Compatibility and migration review

- **No public API change.** The crate has no library target; nothing can
  depend on it. Nothing depends on it today either.
- **No consumer impact.** The crate is `publish = false` with **no registry
  version**. It is not a dependency of any published crate.
- **No manifest change, no dependency change, no `Cargo.lock` change.** The
  new test target needs no manifest entry (Cargo auto-discovers `tests/*.rs`)
  and no dev-dependency (it uses `std` only).
- **No version bump.** The crate inherits `version.workspace = true` (`0.1.3`)
  and stays unpublished.
- **No migration.** There is nothing for a consumer to migrate.
- **Publication status: unchanged.** `publish = false` stays `publish =
  false`. No `cargo publish`, no `cargo package` of this crate, no tag
  movement, no crates.io write of any kind. CI never publishes.
- **Developer-visible change.** A developer running `cargo test -p
  eggup-transport-footprint` now gets **two guard tests** instead of the
  `no targets matched` warning. A developer running the full local gate sees
  the guard tests run inside the existing `cargo test --workspace
  --all-targets --all-features --locked` step. The gate's **commands are
  unchanged**.

## Security review

| Question | Finding |
|---|---|
| Does the new file contain `unsafe` code? | **No.** `unsafe_code = "deny"` is inherited through the workspace lint table and `[lints] workspace = true`; no `unsafe` token appears in `crates/eggup-transport-footprint/tests/cross_link_guard.rs`; all four clippy runs pass with `-D warnings` |
| Does the guard perform I/O outside the repository? | **No.** It reads exactly two text files under `CARGO_MANIFEST_DIR` — `src/bin/curl_only.rs`, `src/bin/eggfetch_only.rs`, `src/bin/dual.rs`, and `Cargo.toml` — and writes nothing |
| Does the guard reach the network, spawn a process, or use credentials? | **No.** No network access, no `Command`, no environment lookup beyond `env!("CARGO_MANIFEST_DIR")`, which is a compile-time constant. It is deterministic, hermetic, and offline, and cannot flake |
| Does the guard give a false sense of coverage? | **No.** It is explicitly **source-level**, and its module doc (`cross_link_guard.rs:19-21`) says so: the binaries are never executed, they perform no measurement, and nothing here asserts throughput, latency, reliability, or binary size |
| Does anything claim authenticity, provenance, or publisher trust? | **No.** The guard inspects linkage and manifest shape only. SHA-256 remains integrity evidence only; no signature or authenticity claim is introduced anywhere |
| Does the CI change introduce a supply-chain surface? | **No.** The added steps run `cargo build`/`cargo check` against the workspace's own pinned lockfile with `--locked`. They add no action, no tool, no permission, and no download beyond what the existing steps already fetch |
| Are the added CI steps a new privilege or a new trigger? | **No.** No new job, no changed command, no changed timeout, no changed trigger, no changed `permissions` block. The workflow's `permissions: contents: read` is unchanged |

## Documentation / operations evidence

### Documents changed by this milestone

| Document | What changed |
|---|---|
| `crates/eggup-transport-footprint/tests/cross_link_guard.rs` | **New.** 381 lines. The module doc states the claim (`:3-9`), the `required-features` limitation (`:5-9`), the two guards (`:11-17`), the source-level nature of the checks (`:19-21`), and the `std`-only / `CARGO_MANIFEST_DIR` / no-`required-features` properties (`:23-27`). T1 at `:65-93`; T2 at `:230-381`; helpers: `crate_root` `:37`, `read_text` `:42`, `strip_comment` `:102`, `parse_tables` `:124`, `table` `:164`, `unquote` `:172`, `value` `:184`, `normalize_list` `:194`, `declared_dependencies` `:201` |
| `.github/workflows/ci.yml` | `stable` gains three `cargo build` steps (`:26-28`, comment `:22-25`); `msrv` gains three `cargo check` steps (`:43-45`, comment `:39-42`); `macos` gains two `cargo build` steps (`:79-80`, comment `:76-78`). `service-systemd` and `windows-check` are **unchanged**; no job, command, timeout, trigger, or permission was changed |
| `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md` | `Status:` header `:3` → `implemented`. Three **factual** corrections recorded (below); no decision reversed, no section restructured |
| `plans/closure/transport-footprint/001-status.md` | **New.** This record |

### Plan corrections (factual only, recorded here so they are not lost)

1. **§3.6 and §8.2 (S2 row) — the cross-link error code depends on the
   cross-link's form.** A `use`-form cross-link fails with
   `error[E0432]: unresolved import`; a path-expression cross-link fails with
   `error[E0433]: failed to resolve: use of unresolved module or unlinked
   crate`. The explanatory text is identical. §6.2 already said
   "(`E0433`/`E0432`)"; §8.2's S2 row has been widened to match, and a
   refinement paragraph was added under §3.6. **No decision changed.**
2. **§10.2 (NC-3) — dropping the `curl` feature is not cargo-neutral.** Cargo
   treats a `required-features` entry naming a missing feature as a
   **warning**, and `src/bin/*.rs` are **auto-discovered**, so deleting the
   `[[bin]]` entry does not remove the target — it becomes a bin with no
   `required-features` that fails to compile before the guard can run. An
   implementation note was added to §10.2 recording both behaviors and both
   control forms that do reach T2. **No decision changed.**
3. **§8.2 (S1 row) — a missing feature is a cargo *warning*, not a resolver
   error, in cargo 1.89.** This is folded into correction 2; the S1 row's
   description of a *requested* feature that does not exist remains accurate
   and was not changed.

### Documents deliberately NOT changed (parent-owned in this batch)

`architecture/transport-footprint.md`, `architecture/tooling-governance.md`,
`architecture/overview.md`, `AGENTS.md`, `plans/registry.md`,
`plans/subsystems/README.md`,
`plans/subsystems/transport-footprint-roadmap.md`, the root `CHANGELOG.md`,
`plans/003-planning-process.md`, `scripts/check-local.sh`,
`crates/eggup-transport-footprint/Cargo.toml`,
`crates/eggup-transport-footprint/src/**`, `Cargo.lock`, `docs/**`, and every
other crate's `Cargo.toml` and `src/**`.

Their current anchors, so the parent can locate them:

| Document | Current state (at `20ac770` + this milestone) | Parent action |
|---|---|---|
| `architecture/transport-footprint.md:190-193` | "There is **no test coverage, by design**. The crate has zero `#[test]` functions…" | Replace with: the crate has **one guard test file** and **zero behavioral tests**; the binaries are still not executed at runtime and still perform no measurement |
| `architecture/transport-footprint.md:210` | `\| Any lane \| --no-default-features --features curl or --features eggfetch \| **No CI job builds the isolated single-adapter configurations** \|` | Replace with the post-M001 truth: the isolated configurations are built by `stable`, `msrv`, and `macos`; `windows-check` builds **none** of the three binaries, before and after, and that is recorded as `not run` |
| `architecture/transport-footprint.md:212-220` | "The last row is the gap this crate exists to close and it is still open…" | Update: the gap is closed by M001; the narrow platform asymmetry on the **binaries** remains and is recorded as `not run` for Windows |
| `architecture/transport-footprint.md:222-226` | "This crate also has **no subsystem roadmap**: there is no `plans/subsystems/transport-footprint-roadmap.md`…" | Update: the roadmap exists and M001 closed the CI-coverage half |
| `architecture/transport-footprint.md:177` | The C013-corrected cross-link claim — "…those are the ones CI never builds (see [Testing approach](#testing-approach))" | **Keep the claim** and extend it with what CI now proves: the isolated configurations are CI-built, so the single-adapter binaries' linkage is machine-checked, and the guard extends that to every configuration. **The old overclaim must not return** |
| `architecture/transport-footprint.md:228-251` (Verification) | The existing command block | Add the three CI configuration commands and the two guard-test commands |
| `architecture/tooling-governance.md` §5 CI matrix and §5.2 / §5.3 | The `--all-features` asymmetry rows | Update: `msrv` and `windows-check` still do not pass `--all-features`, but the footprint crate's three configurations are now built by dedicated steps in `stable`, `msrv`, and `macos` |
| `architecture/tooling-governance.md` §"Known doc/code drift" | The **Open** row naming this milestone (`:797` per plan §12.2) | **Remove** the Open row and record the resolution in the **Resolved** table |
| `architecture/tooling-governance.md` §6.1 inventory | "Subsystem roadmaps \| 7" | → 8, and the subsystem list gains `transport-footprint` |
| `plans/subsystems/README.md` | The subsystem list | Add `transport-footprint-roadmap.md` |
| `plans/registry.md` | Finding 12 open; no transport-footprint rows | Apply the prose under "Registry updates" below |
| `plans/subsystems/transport-footprint-roadmap.md` §12 (`:413`) | `\| M001 isolated-configuration coverage corrective \| open; ready for handoff; … \|` | Apply the prose under "Roadmap disposition" below |
| `AGENTS.md` §"Verify" (CI paragraph) | "Neither `msrv` nor `windows-check` passes `--all-features`, so the feature-gated footprint binaries are built only by `stable` and `macos`" | Update: `msrv` builds them by dedicated steps, and the footprint crate has a guard test |
| `AGENTS.md` §"Workspace layout" | The `eggup-transport-footprint` bullet | The bullet gains the guard test and the CI coverage |
| Root `CHANGELOG.md` §`Unreleased` | — | Apply the bullet under "Registry updates" below |
| `plans/003-planning-process.md` §17 | The active-roadmap enumeration names six subsystems | **The parent's decision** — see "Registry updates" below. Recommendation: **add it** |

## Unresolved findings

The repository's rule: **"medium-or-higher" is the blocking-severity
threshold**. Any such issue still open forces a new corrective before the
milestone closes.

| ID | Finding | Severity | Status |
|---|---|---|---|
| M001-UF-1 | The `windows-check` lane builds **none** of the three footprint **binaries** — before this milestone and after it | **low** | **Open by decision, not by omission** (plan §6.3). The crate contains no `#[cfg]` attribute, no platform-specific code, and no OS-varying path; the property being guarded is dependency resolution and linkage, which the manifest and lockfile govern, not the target platform. No coverage was removed. A Windows-parity build is a bounded, **deliberately unwritten** follow-on candidate in the roadmap's §7, not a corrective this milestone owes. The lane's **guard-test** coverage *does* run — the new test target has no `required-features` and the lane's `cargo test --workspace --all-targets --locked --no-fail-fast` (`ci.yml:109`) executes it — but it must be **observed** in the hosted run, not assumed |
| M001-UF-2 | The recorded dependency counts 3 / 92 / 93 are a **macOS** capture; the transitive crate set of `eggup-eggfetch` is platform-dependent, so no Linux or Windows count is claimed | **low** | **Open by decision.** The counts are a resolution result against the current registry state, not a toolchain or platform invariant. §15's per-platform rule forbids claiming a count for a platform that did not run the commands, and none is claimed |
| M001-UF-3 | `plans/003-planning-process.md` §17 enumerates the active Eggup roadmaps and lists six; `transport footprint` is now a seventh | **low** | **Open, parent-owned decision** (plan §12.4, §16.6). The enumeration is a factual index of the active roadmaps and an incomplete index is itself drift; the file's own §6.2 rule protects normative *decisions*, not factual indices. **Recommendation: add it.** If the parent declines, this residual stands as an open low-severity documentation item owned by a later planning milestone, and the roadmap's §12 M001 row must state it. The file is normative and was **not** edited by this milestone |
| M001-UF-4 | The plan's §8.2 S2 row named only `E0433`; a `use`-form cross-link actually fails with `E0432` | **low** | **Corrected in this milestone.** The plan's §3.6 and §8.2 now name both codes and the exact texts, and the closure records both observed outputs. No behavior or decision changed |
| M001-UF-5 | The plan's §10.2 NC-3 assumed dropping a feature leaves the package buildable; cargo's auto-discovery of `src/bin/*.rs` makes that false in the naive form | **low** | **Corrected in this milestone.** The plan's §10.2 now records both cargo behaviors and the two control forms that do reach T2. No behavior or decision changed |

**No finding is open at medium or higher.** **No follow-on corrective is
required for this milestone.** Registry finding 12 (**medium**) is closed by
this milestone.

## Roadmap disposition

**The parent session owns `plans/subsystems/transport-footprint-roadmap.md`.**
This milestone does not edit it. The intended entry is drafted here for the
parent to apply.

**§12 status row** — replace the M001 row at
`plans/subsystems/transport-footprint-roadmap.md:413` with:

> | M001 isolated-configuration coverage corrective | closed <date>; hosted-qualified (<run>) | `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md` | `plans/closure/transport-footprint/001-status.md` | Closes registry open finding 12 (**medium**). All three configurations are built by CI; the linkage guard is observed to discriminate. Counts 3 / 92 / 93 unchanged; `publish = false` preserved |

**Header status line** — `plans/subsystems/transport-footprint-roadmap.md:3`
currently reads:

> Status: active; M001 open — `001-isolated-configuration-coverage-corrective.md`, ready for handoff; no closed milestones; the crate has never been published and is not publishable by design (`publish = false`)

The parent's replacement should record M001 as closed and keep the
"never been published / not publishable by design" clause.

**§11 completion definition** — the roadmap's §11 items 1-7 are satisfied for
the coverage-and-guard half by this milestone. Items **4** and **5** are
documentation obligations carried as evidence by this closure record and
discharged by the parent's `architecture/transport-footprint.md` edits; the
parent should confirm them when applying the §12.2 prose.

## Registry updates

**The parent session owns `plans/registry.md`,
`plans/subsystems/transport-footprint-roadmap.md`, and the root
`CHANGELOG.md`.** None of them was edited by this milestone. The intended
prose for each is drafted below for the parent to apply.

### `plans/registry.md` — finding 12

Replace the closing paragraph of the §"Post-0.1.3 open findings" entry 12
with:

> **Closed by Transport Footprint M001**
> (`plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md`;
> closure `plans/closure/transport-footprint/001-status.md`). The isolated
> single-adapter configurations are now built by CI — `stable` and `macos`
> build them with `cargo build`, `msrv` checks them at the 1.89 floor — and a
> linkage-discipline guard rejects a sibling-adapter reference in every
> configuration. The crate's dependency counts 3 / 92 / 93 still reproduce
> exactly. The `windows-check` lane still builds none of the three binaries,
> before and after; that is recorded as `not run`, not inferred.

### `plans/registry.md` — "Active subsystem roadmaps" table

New row:

> | Transport footprint | M001 open; the subsystem roadmap is authored and registers M001 as its only milestone | M001 isolated-configuration coverage corrective — closes registry finding 12 (medium); `ready for handoff` |

…which the parent should update to the closed form once the hosted run lands.

### `plans/registry.md` — "Dependency-ready implementation work" table

New row:

> | Transport footprint | M001 isolated-configuration coverage corrective | closed <date>; hosted-qualified (<run>) | `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md`; `plans/closure/transport-footprint/001-status.md` | Closes open finding 12 (**medium**). CI builds the curl-only, eggfetch-only, and both-adapters configurations; a linkage-discipline guard rejects a sibling-adapter reference in every configuration, with a discriminating negative control. Counts 3 / 92 / 93 unchanged. `publish = false`; unpublished; no tag moved |

### Root `CHANGELOG.md` §`Unreleased`

> - `eggup-transport-footprint` gains CI coverage for the configurations it
>   exists to represent (Transport Footprint M001): the crate's central claim
>   — that a consumer can link exactly one transport stack — was verified by
>   no machine, because every lane built only the dual configuration. The
>   curl-only, eggfetch-only, and both-adapters configurations are now built
>   by the `stable` and `macos` lanes and checked at the 1.89 MSRV floor, and
>   a linkage-discipline guard rejects a sibling-adapter reference in every
>   configuration, with a negative control that shows the guard
>   discriminates. The recorded dependency counts 3 / 92 / 93 still reproduce
>   exactly. The crate remains `publish = false` with no registry version; no
>   published crate's dependency graph changed, and no consumer migration is
>   required.

### `plans/003-planning-process.md` §17 — the parent's decision

**Recommendation: add `transport footprint` to the active-roadmap
enumeration.** The enumeration is a factual index of the active roadmaps, and
the file's own §6.2 rule protects normative *decisions*, not factual indices;
an incomplete index is itself drift. If the parent declines, M001-UF-3 stands
as an open **low**-severity documentation item owned by a later planning
milestone, and the roadmap's §12 M001 row must state it.

## Implementation status

The plan's `Status:` header was updated from `ready for handoff` to
`implemented` once this record existed, per `plans/implementation/README.md`
and the workflow in `architecture/tooling-governance.md` §6.

No "Parent-session verification addendum" section appears in this record: that
section is added by the parent session after the hosted CI qualification.
