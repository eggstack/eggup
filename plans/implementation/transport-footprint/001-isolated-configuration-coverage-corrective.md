# Transport Footprint Milestone 001 — isolated-configuration coverage corrective

Status: implemented

Repository baseline: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`

Source roadmap:

- `plans/subsystems/transport-footprint-roadmap.md`

Finding of record:

- `plans/registry.md` §"Post-0.1.3 open findings" — entry **12**: "The `eggup-transport-footprint` crate's central claim is not verified by any CI job, and the crate has no subsystem roadmap." Registered at baseline `c480541` alongside planning C013, which corrected the architecture claim and recorded the residual CI-coverage gap as an open drift row in `architecture/tooling-governance.md` §"Known doc/code drift", naming the missing `plans/subsystems/transport-footprint-roadmap.md` as this milestone's prerequisite. The prerequisite is discharged: the roadmap is authored and is this plan's source roadmap. The finding is assessed **medium** severity and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant

## 1. Objective

Close registry finding 12 for `crates/eggup-transport-footprint` by making the
crate's central claim machine-checked.

The crate exists so that one fact is visible: which HTTP transport stack a
consumer of `eggup-acquisition` links into its binary. It expresses that fact
as three build configurations — `curl_only`, `eggfetch_only`, `dual` — behind
`required-features`. Today **no machine builds the isolated single-adapter
configurations at all**: the five CI lanes build only `--all-features` (the
dual configuration) or default features (nothing), and the local gate does the
same. The one configuration the crate exists to represent is the one
configuration nothing builds.

This milestone does four things, in this order:

1. **Build the isolated configurations in CI.** `curl-only`,
   `eggfetch-only`, and both-adapters are built by the lanes that build the
   crate, so "this binary links this stack" becomes a CI fact.
2. **Make the isolation claim hold in *every* configuration, not only the
   isolated ones.** A `required-features` split does *not* make an accidental
   cross-link a build error — under `--all-features` both optional
   dependencies are visible to every binary in the package, so a cross-link
   compiles. This milestone adds a guard that rejects a sibling-adapter
   reference in every configuration, including a `#[cfg]`-gated one that the
   isolated builds cannot see.
3. **Pin the recorded dependency counts** 3 / 92 / 93 as regression evidence,
   and pin the manifest shape that produces them.
4. **Correct the documentation** so that the crate's own claim, its CI
   coverage, and its test story are stated accurately — and so that nothing
   claims a measurement capability the crate does not have.

No production source change, no manifest change, no dependency change, no
`Cargo.lock` change, no publication, no tag movement, no version bump.

## 2. Why this milestone is ready

- Every defect is fully characterized by reading the repository at baseline
  `20ac770`. Nothing in the diagnosis depends on an unobserved runtime
  behavior.
- The prerequisite named by C013 — a `transport-footprint` subsystem roadmap —
  exists: `plans/subsystems/transport-footprint-roadmap.md` is this plan's
  source roadmap, and it registers M001 as the subsystem's only milestone.
- The scope is bounded to one non-published fixture crate, one CI workflow
  file, and one new test file. It touches no published crate's API, manifest,
  or dependency graph.
- The finding is **medium** severity. No production failure has been reported:
  no consumer has been harmed, because the crate is not published, is not a
  dependency of any crate, and is used by no consumer. The severity reflects
  the size of the unverified claim, not an observed outage.

Honest limitations, stated up front:

- **The crate performs no measurement in code.** Its three binaries construct
  a transport, construct a request, and print the configuration they resolve
  to (`crates/eggup-transport-footprint/src/bin/curl_only.rs:6-13`,
  `src/bin/eggfetch_only.rs:6-16`, `src/bin/dual.rs:5-16`). This milestone
  verifies **linkage** and **dependency resolution**. It does not add, verify,
  or claim any throughput, latency, reliability, or binary-size measurement.
- **The recorded size figures are not re-measured.** The binary sizes in
  `architecture/transport-footprint.md` are a dated capture from commit
  `ae9b47a` on macOS. This milestone re-verifies the *dependency counts*,
  which the manifest shape controls, and leaves the size figures alone.
- **The `windows-check` lane will still build none of the three binaries.**
  That is a deliberate decision with a recorded rationale (§6.3), not an
  oversight, and the closure must report the Windows binary coverage as
  `not run` — never inferred.
- **The crate's "zero tests" property changes.** See §10.1 for the decision
  and its justification.

## 3. Current implementation evidence

### 3.1 The manifest

`crates/eggup-transport-footprint/Cargo.toml` — 46 lines, the crate's whole
contract:

| Lines | Content |
|---|---|
| `:1-10` | `[package]`: `name`, the five inherited workspace keys (`:3-7`), **`publish = false` (`:8`)**, `description`, `authors` |
| `:12-13` | `[lints]` / `workspace = true` — the crate's only lint layer |
| `:15-16` | `[dependencies]` — **`eggup-acquisition` only**, `{ version = "0.1.0", path = "../eggup-acquisition" }` |
| `:18-21` | `[features]`: `curl = ["dep:eggup-curl"]`, `eggfetch = ["dep:eggup-eggfetch"]`, `default = []` |
| `:23-26` | `[dependencies.eggup-curl]` — `optional = true` |
| `:28-31` | `[dependencies.eggup-eggfetch]` — `optional = true` |
| `:33-36` | `[[bin]] curl_only`, `required-features = ["curl"]` |
| `:38-41` | `[[bin]] eggfetch_only`, `required-features = ["eggfetch"]` |
| `:43-46` | `[[bin]] dual`, `required-features = ["curl", "eggfetch"]` |

`default = []` at `:21` is load-bearing: with no features, **no binary target
matches** and Cargo reports `warning: target filter 'all-targets' specified,
but no targets matched; this is a no-op` (reproduced at `20ac770`; also
recorded at `architecture/transport-footprint.md:159-162`).

The crate directory contains **only** `Cargo.toml` and `src/bin/`. There is no
`lib.rs`, no `README.md`, no `CHANGELOG.md`, and no `tests/`. The crate is the
sole documented exception to the library-crate rule
(`architecture/tooling-governance.md` §3.4; `AGENTS.md` §"Workspace layout").

### 3.2 The three binaries — 45 source lines, zero tests

| Binary | Path | Lines | `required-features` | References |
|---|---|---|---|---|
| `curl_only` | `src/bin/curl_only.rs` | 13 | `curl` | `eggup_acquisition` (`:3`), `eggup_curl` (`:4`) |
| `eggfetch_only` | `src/bin/eggfetch_only.rs` | 16 | `eggfetch` | `eggup_acquisition` (`:3`), `eggup_eggfetch` (`:4`) |
| `dual` | `src/bin/dual.rs` | 16 | `curl`, `eggfetch` | `eggup_acquisition` (`:3`), `eggup_curl` (`:6`), `eggup_eggfetch` (`:12`) |

Total 13 + 16 + 16 = **45 source lines**. `grep -rn "#\[test\]"
crates/eggup-transport-footprint` returns **zero** hits; the crate has no
`tests/` directory and no dev-dependencies.

Each binary's module doc states the fixture's actual assertion
(`curl_only.rs:1`, `eggfetch_only.rs:1`, `dual.rs:1`):

```rust
//! Curl-only footprint binary: links seam + curl, never Eggfetch.
//! Eggfetch-only footprint binary: links seam + Eggfetch, never curl.
//! Dual-transport footprint binary: links seam + curl + Eggfetch + composition.
```

None of the three sources contains an `#[cfg]` attribute, an `unsafe` block,
or a platform-specific path. `curl_only.rs:7` hardcodes `/usr/bin/curl` as a
*fixture value* — a string, never opened at build time.

### 3.3 The corrected architecture claim (do not reintroduce the old one)

`architecture/transport-footprint.md:177`, corrected by planning C013, states
the guarantee accurately:

> `required-features` stops a binary from *building* when its adapter's
> feature is off. It does **not** stop a cross-link when both features are on,
> because an enabled optional dependency becomes available to every target in
> the package — so "links no Eggfetch code" is a meaningful claim only in the
> single-adapter configurations, and those are the ones CI never builds.

The pre-C013 wording — that the split "makes an accidental cross-link a build
error" — was **inaccurate** and must not return. The same section's
CI-coverage table ends with the row (`:210`):

> Any lane | `--no-default-features --features curl` or `--features eggfetch` |
> **No CI job builds the isolated single-adapter configurations**

### 3.4 The CI coverage, as it stands at `20ac770`

`.github/workflows/ci.yml` has five jobs. What each lane does with this crate:

| Lane | Command | Footprint binaries built |
|---|---|---|
| `stable` (`:19-23`) | `cargo fmt --all`; `cargo clippy --workspace --all-targets --all-features --locked`; `cargo test --workspace --all-targets --all-features --locked`; `cargo doc --workspace --no-deps --locked`; `cargo package -p eggup-service --locked` | **dual only** (`--all-features`) |
| `msrv` (`:30-32`) | `cargo check --workspace --all-targets --locked`; `cargo package -p eggup-service --locked` | **none** (no `--all-features`, `default = []`) |
| `service-systemd` (`:40-51`) | registry-only service fixtures and the privileged systemd fixture | **none** (never builds the workspace) |
| `macos` (`:61-62`) | `cargo test --workspace --all-targets --all-features --locked`; `cargo package -p eggup-service --locked` | **dual only** |
| `windows-check` (`:70-91`) | nine `cargo test` steps, then `cargo test --workspace --all-targets --locked --no-fail-fast`; `cargo package -p eggup-service --locked` | **none** (no `--all-features`) |

The local gate `scripts/check-local.sh:5-6` runs
`cargo clippy --workspace --all-targets --all-features --locked` and
`cargo test --workspace --all-targets --all-features --locked` — **dual
only**.

`architecture/tooling-governance.md` §5.3 records the same asymmetry as an
open coverage gap, and its §"Known doc/code drift" **Open** table carries the
row (`:797`) that names this milestone.

### 3.5 The dependency counts reproduce exactly

Verified at `20ac770` with the counting method recorded by planning C013
(`plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md` §11), with `--offline`:

```sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
```

Observed: **3**, **92**, **93** — identical to the figures published in
`architecture/transport-footprint.md:106-108` and in the registry's finding 12.

Two counting traps, recorded so a future reader does not "correct" the
figures:

- `cargo tree --prefix none` emits `(*)` markers on repeat nodes. They must be
  excluded, or the counts read **118** and **119**.
- The counts are unique tree **lines**, not unique crate **names**:
  `webpki-roots` resolves at two versions (`0.26.11` and `1.0.9`) in the
  eggfetch graph, so a name-only count reads **91** and **92**. The C013
  method is the recorded method.

### 3.6 What the mechanism actually does — three behaviors, verified

The cross-link question was settled empirically before this plan was written,
using a scratch package outside this repository that reproduces the crate's
structure (one optional dependency per feature, two `required-features`
bins). The three observed behaviors are the design basis for §6 and §8:

| Scenario | Observed result |
|---|---|
| A plain cross-link (`use` of the sibling crate) in the single-adapter binary, built in the **isolated** configuration | **Fails**: `error[E0433]: failed to resolve: use of unresolved module or unlinked crate \`beta\`` |
| The same cross-link, built in the **dual** configuration | **Succeeds** — compiles and links cleanly |
| A `#[cfg(feature = …)]`-gated cross-link in the single-adapter binary, built in the **isolated** configuration | **Succeeds** — the `cfg` strips the reference before name resolution, so the sibling crate is never resolved |

The third row is the reason the isolated builds alone are **not** a complete
guard, and the reason §6 requires a crate-side guard as well.

**Error-code refinement, recorded at implementation (NC-1).** The first row's
error code depends on the *form* of the cross-link, and both forms were
observed in this repository during M001's negative controls. A cross-link
written as an `use` statement fails with **`error[E0432]: unresolved import
\`eggup_eggfetch\`**, whose help line reads "use of unresolved module or
unlinked crate `eggup_eggfetch`". A cross-link written as a path expression
(`std::any::type_name::<eggup_eggfetch::EggfetchTransport>()`) fails with
**`error[E0433]: failed to resolve: use of unresolved module or unlinked crate
\`eggup_eggfetch\`**. The explanatory text is identical; only the code differs.
§6.2's "(`E0433`/`E0432`)" already names both; §8.2's S2 row has been
widened to match the observed reality.

## 4. Invariants that must not regress

1. `publish = false` stays at `crates/eggup-transport-footprint/Cargo.toml:8`.
   The crate has no registry version and must never acquire one.
2. The dependency counts **3 / 92 / 93** reproduce exactly under the §3.5
   method. Any change to them is a change to the recorded evidence in
   `architecture/transport-footprint.md:106-108` and in the registry.
3. The `required-features` split survives: `curl_only` requires `curl`,
   `eggfetch_only` requires `eggfetch`, `dual` requires both, and `default`
   stays empty. The split is what makes an isolated configuration
   representable; removing it deletes the crate's reason to exist.
4. The crate gains **no** dependency on `eggup-core`, `eggup-service`,
   `eggup-archive`, or `eggup-eggpack`. `eggup-acquisition` stays the only
   unconditional dependency; `eggup-curl` and `eggup-eggfetch` stay optional.
5. `eggup-curl` does not learn that `eggup-eggfetch` exists, and
   `eggup-eggfetch` does not learn that `eggup-curl` exists.
6. The three binaries perform no measurement, select no transport, and assert
   no transport policy. `curl_only` keeps its hardcoded `/usr/bin/curl`
   fixture value.
7. No published crate's dependency graph changes. In particular
   `eggup-eggpack`'s exact `=0.1.3` pins are untouched.
8. `Cargo.lock` does not change. The new test uses only `std`.
9. The local gate `scripts/check-local.sh` keeps its shape: five `cargo`
   commands, one script call, no conditionals, no per-crate specialization
   (`architecture/tooling-governance.md` §4).
10. The accurate cross-link claim from `architecture/transport-footprint.md:177`
    survives. The old overclaim does not return.
11. `unsafe_code = "deny"` holds in the new test file through the workspace
    lint table (`[lints] workspace = true`, `Cargo.toml:12-13`). The new test
    must not contain an `unsafe` block.

## 5. Scope

### In scope

- A new integration-test target for `eggup-transport-footprint`:
  `crates/eggup-transport-footprint/tests/cross_link_guard.rs`, containing the
  cross-link guard (§10.1) and the manifest-shape guard (§10.1).
- New CI steps in `.github/workflows/ci.yml` for the `stable`, `msrv`, and
  `macos` lanes, building or checking the three configurations (§6).
- Negative-control evidence for every guard (§10.2), including the recorded
  failing output.
- Re-verification of the 3 / 92 / 93 counts and of the manifest shape.
- The documentation edits enumerated in §12, split into the ones the
  implementing session owns and the ones the **parent session** owns.

### Explicitly out of scope

- **Any change to `crates/eggup-transport-footprint/src/bin/*.rs`.** The
  three binaries are correct for their purpose. They gain no measurement, no
  guard macro, no `compile_error!`, and no new output. (The negative controls
  in §10.2 edit them *temporarily* and restore them; §14 stops the milestone
  if the tree is not restored.)
- **Any change to `crates/eggup-transport-footprint/Cargo.toml`** — no new
  feature, no new dependency, no dev-dependency, no `[[test]]` section (an
  auto-discovered `tests/` target needs no manifest entry), no pin-style
  change. The pin-style question belongs to the workspace-wide milestone
  already recorded as an open drift row in
  `architecture/tooling-governance.md` §"Known doc/code drift".
- **Any change to `scripts/check-local.sh`.**
- **Any change to the `windows-check` lane** (rationale in §6.3).
- **Any change to a published crate, any crate's `Cargo.toml`, or
  `Cargo.lock`.**
- **Publication, tagging, version bumps, release notes.** The crate is
  `publish = false` and stays that way. CI never publishes.
- **Re-measuring the binary-size figures** in
  `architecture/transport-footprint.md`.
- **Adding a `README.md` or `CHANGELOG.md` to the crate.** It is the
  documented binary-only exception (`architecture/tooling-governance.md`
  §3.4); the crate's documentation surface is its three binaries' module docs
  plus the architecture deep dive.
- **Adding a library target, a `lib.rs`, or any public API.**
- **A real measurement capability.** No size reporting, no benchmark, no
  throughput probe.
- **The `plans/003-planning-process.md` §17 roadmap enumeration.** That file
  is normative and is not edited in ordinary work; see §12.4 and §16.
- **Writing the closure record.** `plans/closure/transport-footprint/001-status.md`
  is written after implementation, by the implementing session or the parent.

## 6. Required production changes

**This is not a CI-only change.** It is CI **plus** a crate-side guard, and
the justification is §3.6: an isolated build catches a plain cross-link but
not a `#[cfg]`-gated one, so the isolated builds alone leave a hole exactly
in the guarantee the crate exists to provide. The CI change makes the
isolated configurations real; the crate-side guard makes the isolation claim
hold in every configuration.

The production changes are:

### 6.1 The crate-side guard

Add exactly one file:
`crates/eggup-transport-footprint/tests/cross_link_guard.rs`.

It is an **integration test target**, not a `lib.rs` and not a new binary. It
has no `required-features`, so it is built and run in **every** feature
configuration, including `default = []` — which is exactly the property the
guard needs. It uses only `std`; it adds **no** dependency and therefore does
not change `Cargo.lock`. It resolves the crate root through
`env!("CARGO_MANIFEST_DIR")` so it does not depend on the working directory.

Its content is specified in §10.1 (T1, T2).

### 6.2 The CI steps

Add the following to `.github/workflows/ci.yml`.

**`stable` lane** (Linux; the reference platform; already runs clippy, fmt,
and doc). Insert after the existing
`cargo test --workspace --all-targets --all-features --locked` step, before
`cargo doc`:

```yaml
      # The isolated single-adapter configurations are the crate's entire
      # purpose. Without these steps, "curl_only links no Eggfetch code" is
      # never checked by any machine. Each configuration is its own step so a
      # failure names itself instead of hiding in a loop.
      - run: cargo build -p eggup-transport-footprint --no-default-features --features curl --locked
      - run: cargo build -p eggup-transport-footprint --no-default-features --features eggfetch --locked
      - run: cargo build -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked
```

**`msrv` lane** (toolchain `1.89.0`). Insert after the existing
`cargo check --workspace --all-targets --locked` step:

```yaml
      # `cargo check --workspace --all-targets --locked` enables no feature
      # (default = []), so it compiled none of the three footprint binaries.
      # The MSRV floor must apply to the feature-gated paths too, and the
      # dual configuration is where two independent adapter graphs meet.
      - run: cargo check -p eggup-transport-footprint --no-default-features --features curl --locked
      - run: cargo check -p eggup-transport-footprint --no-default-features --features eggfetch --locked
      - run: cargo check -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked
```

`cargo check` rather than `cargo build` matches the lane's existing style
(the lane is a type-check lane, and it does not run tests or clippy). It is
sufficient: the cross-link failure is a **name-resolution** error
(`E0433`/`E0432`), which `check` reports exactly as `build` does.

**`macos` lane**. Insert after the existing
`cargo test --workspace --all-targets --all-features --locked` step:

```yaml
      # macOS is the platform on which the published 3 / 92 / 93 counts were
      # captured, so the isolated configurations are built here as well. The
      # dual configuration is already built by the lane's --all-features step.
      - run: cargo build -p eggup-transport-footprint --no-default-features --features curl --locked
      - run: cargo build -p eggup-transport-footprint --no-default-features --features eggfetch --locked
```

`--locked` is mandatory in CI. `architecture/transport-footprint.md:250-251`
states why: without it the dependency counts are not reproducible and a silent
`Cargo.lock` update could change them.

### 6.3 Why these three lanes, and not `windows-check`

`stable` is the reference platform and already builds the crate. `msrv` is
the lane that closes a real hole: it compiles **none** of the three binaries
today, so the 1.89 floor has never been applied to the feature-gated paths —
including the dual configuration, which is the place two independent adapter
dependency graphs meet and therefore the place most likely to break on a
toolchain floor. `macos` is the platform on which the published counts were
captured, so building the isolated configurations there keeps those figures
re-derivable natively.

`windows-check` is **excluded, by decision**. The crate contains no `#[cfg]`
attribute, no platform-specific code, and no path that differs by OS; the
property being guarded is *dependency resolution and linkage*, which is
governed by the manifest and the lockfile, not by the target platform. Adding
three build steps to the repository's scarcest lane would cost CI time
without adding information about that property.

Consequences that must be recorded, not inferred:

- The Windows lane's coverage of the three footprint **binaries** is `not
  run` — before this milestone and after it. No coverage is removed; the
  status quo is preserved and stated.
- The Windows lane's coverage of the new **guard test** *does* run: the lane
  already executes `cargo test --workspace --all-targets --locked
  --no-fail-fast` (`ci.yml:90`), and the new test target has no
  `required-features`, so it is built and executed on Windows. The guard is
  platform-independent by construction (it reads text files), so this is a
  real, expected result — but it must still be *observed* and recorded, not
  assumed.
- The dependency counts 3 / 92 / 93 were captured on macOS. The transitive
  crate set of `eggup-eggfetch` is platform-dependent, so **no count is
  claimed for Windows or for Linux** beyond what the §3.5 commands observe
  on the platform that ran them. §15 requires per-platform reporting.

If a maintainer wants Windows parity for the binaries, that is a bounded
follow-on (three more `cargo build` steps) and is **not** M001's scope; the
roadmap records it as an intentionally unwritten candidate.

### 6.4 The guard's target: which configurations it protects

The guard is a source-level test, so it runs in every configuration and does
not depend on any feature being enabled. It protects:

- `curl_only` — must never reference `eggup_eggfetch`;
- `eggfetch_only` — must never reference `eggup_curl`;
- `dual` — the one place the two adapters meet, and therefore the one place
  referencing both is *correct*; the guard asserts that it does reference
  both, so that the crate cannot quietly stop exercising composition.

## 7. Ordered work packages

**WP1 — Roadmap (already complete).** `plans/subsystems/transport-footprint-roadmap.md`
is authored and is this plan's source roadmap. It registers M001 as the
subsystem's only milestone, records the invariants, and records the decision
table that §6 implements. Nothing further is required for it.

**WP2 — The guard test.** Create
`crates/eggup-transport-footprint/tests/cross_link_guard.rs` with T1 and T2
(§10.1). `std` only. No manifest change, no dependency, no `Cargo.lock`
change. Verify locally: `cargo test -p eggup-transport-footprint --all-targets
--locked --offline` must now report the guard tests as run (the
`no targets matched` warning at `20ac770` disappears).

**WP3 — The CI steps.** Add the `stable`, `msrv`, and `macos` steps from
§6.2 to `.github/workflows/ci.yml`, each with the explanatory comment given
there. Change nothing else in the workflow: no new job, no changed command,
no changed timeout, no changed trigger.

**WP4 — Negative controls.** Run NC-1, NC-2, and NC-3 (§10.2) and capture
the exact failing output of each, then restore the tree. The restore is
verified by `git status --short` showing only the intended files.

**WP5 — Local verification.** Run the §11 crate-scoped commands, record the
observed output, and hand the workspace-wide gate to the parent session
(§11.2).

**WP6 — Documentation.** Apply the doc edits the implementing session owns
(§12.2) and hand the parent-owned prose to the parent session (§16).

**WP7 — Closure.** The closure record is written to
`plans/closure/transport-footprint/001-status.md` after implementation, by
the implementing session or the parent, following `plans/closure/README.md`
and §15 of this plan.

## 8. Failure, cancellation, restart, and contention semantics

### 8.1 What a failing isolated build means

A failing footprint step **fails its lane**. GitHub Actions stops the job at
the first failing step, so the lane's remaining steps do not run and the
lane is red. This is intended: the isolated configuration is a correctness
gate, not a notice. There is no `continue-on-error`, no `|| true`, and no
warning-only mode.

Within a lane, the three configurations are **three separate steps**, not one
loop, so the CI UI names the failing configuration. A single looping step
would report one opaque failure and the reader would have to reconstruct which
configuration broke.

### 8.2 Distinguishing a missing feature combination from a cross-link regression

These are different defects and must not be conflated. Three distinct
signatures:

| Signature | Where it appears | How to read it |
|---|---|---|
| **S1 — feature-resolution failure.** Cargo rejects the *requested* feature before compiling anything. The message names a **feature** (e.g. `package 'eggup-transport-footprint' does not have the feature 'curl'` / `none of the selected packages contains these features: curl`). | **All three** footprint steps of the lane fail identically, and the failure is at resolver stage, not at compile stage. The `dual` step fails too. | **Manifest/configuration defect.** A feature was renamed, dropped, or misspelled; or the `--features` argument is wrong. Nothing about the sources is implicated. |
| **S2 — plain cross-link.** The feature resolves; the build fails with a name-resolution error naming the **sibling adapter** (`eggup_eggfetch` in `curl_only`, `eggup_curl` in `eggfetch_only`) — `error[E0433]: failed to resolve: use of unresolved module or unlinked crate` for a path-expression cross-link, `error[E0432]: unresolved import` for a `use`-form cross-link (see §3.6's refinement). | **Exactly one** footprint step fails — the isolated configuration of the offending binary. The **dual** step **succeeds**, because both optional dependencies are present there and the cross-link compiles. | **Cross-link regression, plain form.** §3.6 row 1. This is the failure the isolated build exists to catch, and it is precisely the case the old architecture claim got wrong. |
| **S3 — cfg-gated cross-link.** The isolated builds **all succeed**, and the guard test **fails** with a message naming the offending file and the forbidden token. | The guard test, in every configuration. | **Cross-link regression, `#[cfg]`-gated form.** §3.6 row 3. The isolated build cannot see this; the guard is the only thing that can. |

The discriminator, stated plainly: **a resolver error naming a feature is a
manifest defect; a resolver-success / name-resolution error naming the sibling
adapter is a cross-link regression; a guard failure with green isolated builds
is a cfg-gated cross-link.** The closure must record the exact error text for
each case, and NC-1 and NC-2 exist so that the S2 and S3 texts are
*observed*, not asserted.

### 8.3 Failure semantics of the guard itself

The guard test fails the `cargo test` step of the `stable` and `macos` lanes
and the `cargo test --workspace --all-targets --locked --no-fail-fast` step of
the `windows-check` lane. On the Windows lane that closing run uses
`--no-fail-fast` (`:90`), so the other Windows test targets still execute and
report, rather than one failing target hiding the state of the rest. The
guard's own failure message must name the file path and the forbidden token,
so the failure is actionable without opening the file.

The guard is deterministic, hermetic, and offline: it reads two text files
from `CARGO_MANIFEST_DIR` and compares them. It has no timing, no network, no
process spawn, and no filesystem writes. It cannot flake.

### 8.4 Cancellation, restart, and contention

There is nothing to cancel, restart, or arbitrate: the milestone changes no
runtime behavior, holds no lock, spawns no process, writes no file outside
the repository working tree, and touches no shared resource. `cargo build` and
`cargo test` invocations are independent and re-runnable; a cancelled or
failed CI job is re-run from scratch.

The one shared resource is the repository working tree: a parallel milestone's
in-flight edits are why §11 mandates **crate-scoped** `cargo fmt -p
eggup-transport-footprint -- --check` rather than `cargo fmt --all -- --check`,
and why §11 mandates `--offline` and crate-scoped `cargo` invocations
throughout.

## 9. Compatibility and migration

- **No public API change.** The crate has no library target; nothing can
  depend on it. Nothing depends on it today either — `grep -rn
  "eggup-transport-footprint" crates/*/Cargo.toml Cargo.toml` returns only the
  workspace member entry and the crate's own manifest.
- **No consumer impact.** The crate is `publish = false` with no registry
  version. It is not a dependency of any published crate.
- **No manifest change, no dependency change, no `Cargo.lock` change.** The
  new test target needs no manifest entry and no dev-dependency.
- **No version bump.** The crate inherits `version.workspace = true` (`0.1.3`)
  and stays unpublished.
- **No migration.** There is nothing for a consumer to migrate.
- **Developer-visible change.** A developer running
  `cargo test -p eggup-transport-footprint` now gets two guard tests instead
  of the `no targets matched` warning. A developer running the full local
  gate sees the guard tests run inside the existing
  `cargo test --workspace --all-targets --all-features --locked` step. The
  gate's *commands* are unchanged.

## 10. Required tests

### 10.1 The two guard tests

Both live in `crates/eggup-transport-footprint/tests/cross_link_guard.rs`.
They are **source-level guards**, not behavioral tests: the binaries are still
not executed, and still perform no measurement.

**T1 — cross-link guard.** Reads the three `src/bin/*.rs` files through
`env!("CARGO_MANIFEST_DIR")` and asserts:

| Source | Assertion |
|---|---|
| `src/bin/curl_only.rs` | contains **no** occurrence of the token `eggup_eggfetch` |
| `src/bin/eggfetch_only.rs` | contains **no** occurrence of the token `eggup_curl` |
| `src/bin/dual.rs` | contains **both** `eggup_curl` and `eggup_eggfetch` — `dual` is the one place the two adapters legitimately meet, and the guard asserts the composition fixture still exercises composition |

The token check must be over the **whole file text**, not over the lines that
survive a `cfg` evaluation — that is the whole point of T1 (§3.6 row 3). The
failure message must name the file and the forbidden token, e.g.
`curl_only.rs references eggup_eggfetch; the curl-only fixture must link the
curl adapter only`.

**T2 — manifest dependency-shape guard.** Reads
`crates/eggup-transport-footprint/Cargo.toml` and asserts, as **text**
(normalized lines; no TOML parser, because adding one would add a dependency
and change `Cargo.lock`):

| Assertion | Why it is a guard |
|---|---|
| `publish = false` is present | Invariant 1 |
| `eggup-acquisition` is the only **unconditional** dependency | Invariant 4 |
| `eggup-curl` and `eggup-eggfetch` are each declared `optional = true` | The optionality is what produces the 3 / 92 / 93 split |
| `curl = ["dep:eggup-curl"]`, `eggfetch = ["dep:eggup-eggfetch"]`, `default = []` | Invariant 3; a non-empty `default` would put the dual configuration into every default build |
| each `[[bin]]` carries the expected `required-features` | Invariant 3 |
| **no** dependency on `eggup-core`, `eggup-service`, `eggup-archive`, or `eggup-eggpack` | Invariant 4 |

T2 pins the **shape** that produces the counts, not the counts themselves.
That is deliberate and is the answer to "should the counts be an automated
test?": the counts are a property of a 92-crate resolution against the
current registry state, and `webpki-roots` alone resolves at two versions in
that graph. A hard-coded count assertion would break on an upstream release
that has nothing to do with this crate, and it would guard nothing that T2
does not already guard more stably. The counts are verified by the §11
commands and recorded in the closure (§15), which is where C013 put them.

### 10.2 Negative controls — required, and they must be observed to fail

An assertion never observed to fail has not been shown to work. Each of the
three controls below must be executed, its **failing output captured
verbatim**, and the tree restored afterwards.

**NC-1 — the isolated build catches a plain cross-link.** Temporarily add a
plain `use eggup_eggfetch::…;` reference to `src/bin/curl_only.rs`.
Expected: the curl-only build **fails** with
`error[E0433]: failed to resolve: use of unresolved module or unlinked crate
\`eggup_eggfetch\``, while the **dual** build **succeeds** — which is the
empirical proof of the §8.2 S2 signature and the empirical disproof of the
old "cross-link is a build error" claim. Restore the file.

**NC-2 — the guard catches a `#[cfg]`-gated cross-link that the isolated
build cannot see.** Temporarily add a
`#[cfg(feature = "eggfetch")]`-gated reference to `eggup_eggfetch` in
`src/bin/curl_only.rs`. Expected: the curl-only build **succeeds** (the
`cfg` strips the reference before name resolution) and the guard test
**fails**, naming the file and the token. This is the discriminating case,
and the reason T1 exists. Restore the file.

**NC-3 — the manifest guard catches a dropped feature.** Temporarily remove
or rename the `curl` feature in `Cargo.toml`. Expected: T2 **fails**, naming
the missing feature. Restore the file.

**NC-3 implementation note, recorded at implementation.** Dropping the `curl`
feature is **not** cargo-neutral, and a control that only edits the
`[features]` table will not reach T2. Two cargo behaviors were observed:

- Cargo treats a `required-features` entry naming a missing feature as a
  **warning**, not an error (`warning: invalid feature \`curl\` in
  required-features of target \`dual\`: \`curl\` is not present in [features]
  section`), so the `[[bin]]` entries do not by themselves stop the build.
- `src/bin/*.rs` are **auto-discovered** by Cargo, so deleting the `[[bin]]`
  entry for `curl_only` does **not** remove the target — it becomes an
  auto-discovered binary with **no** `required-features`, which is then built
  in every configuration and fails to compile (`error[E0432]: unresolved
  import \`eggup_curl\``) before the guard test can run.

To observe T2 failing, the control must leave the package buildable: keep the
`[[bin]] curl_only` entry and move `src/bin/curl_only.rs` aside in the control
copy, or drop the feature and the source file together. Both forms were
executed and both made T2 fail, naming the missing feature. See the closure
record's negative-control evidence.

Restore must be verified: after all three controls, `git status --short`
must show only the intended changes (the new test file, the workflow file, and
the doc files). If the tree is not restored, §14 stops the milestone.

### 10.3 What is deliberately not added

- No behavioral test of the three binaries. They construct a transport, build
  a request, and print; they perform no measurement and no fetch, and a test
  that asserts their stdout would assert an implementation detail.
- No test that shells out to `cargo tree`. The counts are verified by the
  §11 commands; a test that spawns cargo would be slow, network-sensitive,
  and would assert something upstream churn can break for no safety gain.
- No dependency-count test, for the reason given in §10.1.
- No benchmark, no size assertion, no measurement harness.

## 11. Required verification commands

### 11.1 Crate-scoped commands for the implementing session

Run from the repository root. **`--offline` everywhere.** Use the
**crate-scoped** `cargo fmt -p eggup-transport-footprint -- --check` — do
**not** use `cargo fmt --all -- --check`: a parallel milestone's in-flight
edits must not be able to make this milestone's verification fail spuriously.

```sh
# --- formatting (crate-scoped; NOT --all) ---
cargo fmt -p eggup-transport-footprint -- --check

# --- the three configurations must build ---
cargo build -p eggup-transport-footprint --no-default-features --features curl --locked --offline
cargo build -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline
cargo build -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline

# --- clippy over the crate in every configuration ---
cargo clippy -p eggup-transport-footprint --all-targets --all-features --locked --offline -- -D warnings
cargo clippy -p eggup-transport-footprint --all-targets --no-default-features --features curl --locked --offline -- -D warnings
cargo clippy -p eggup-transport-footprint --all-targets --no-default-features --features eggfetch --locked --offline -- -D warnings

# --- the guard tests ---
cargo test -p eggup-transport-footprint --all-targets --locked --offline

# --- dependency counts: expected 3, 92, 93 in that order ---
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l

# --- the change set is exactly what it should be ---
git status --short
git diff --name-only
git diff --check
```

Expected results: the three counts are `3`, `92`, `93`; the clippy runs are
clean with `-D warnings`; the guard tests report **2 passed**; `git diff
--name-only` shows only `.github/workflows/ci.yml`, the new test file, and the
doc files listed in §12 — no `crates/*/src/**`, no `Cargo.toml`, no
`Cargo.lock`.

### 11.2 Workspace-wide commands — **the parent session runs these**

These are **not** the implementing session's responsibility, because a
parallel milestone may have in-flight edits in the same tree. The parent
session runs them after the change set is complete:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree --workspace --locked          # review step, not a gate
./scripts/check-local.sh
```

Note for the parent's review step: `cargo tree --workspace --locked` is
**feature-blind** and does not surface the footprint crate's optional adapter
edges. The three scoped `cargo tree` runs in §11.1 are the ones that show
them, and the parent's review must read them.

## 12. Documentation updates

### 12.1 The rule

`architecture/**`, `plans/registry.md`, `AGENTS.md`, and the root
`CHANGELOG.md` are owned by the **parent session** in this batch. The
implementing session MUST NOT edit them; it hands the parent the prose in
§16. The one exception is the crate's own new test file.

### 12.2 The documentation changes required by this milestone

| File | Change | Owner |
|---|---|---|
| `architecture/transport-footprint.md` §"Testing approach" | Replace "There is **no test coverage, by design**" and "The crate has zero `#[test]` functions" with the accurate statement: the crate has **one guard test file** and **zero behavioral tests**; the binaries are still not executed at runtime and still perform no measurement. | parent |
| `architecture/transport-footprint.md` §"Testing approach" CI table | Replace the row "Any lane \| … \| **No CI job builds the isolated single-adapter configurations**" with the post-M001 truth: the isolated configurations are built by `stable`, `msrv`, and `macos`; `windows-check` builds none of the three binaries, before and after, and that is recorded as `not run`. | parent |
| `architecture/transport-footprint.md` §"Invariants and what this crate enforces", row "Adapters do not know each other" | Keep the C013-corrected claim at `:177` **and** extend it with what CI now proves: the isolated configurations are CI-built, so the single-adapter binaries' linkage is machine-checked; the guard extends that to every configuration. The old overclaim must not return. | parent |
| `architecture/transport-footprint.md` §"Verification" | Add the three CI configuration commands and the two guard-test commands. | parent |
| `architecture/tooling-governance.md` §5 CI matrix and §5.2 / §5.3 coverage gaps | Update the lane table and the `--all-features` asymmetry rows: `msrv` and `windows-check` still do not pass `--all-features`, but the footprint crate's three configurations are now built by dedicated steps in `stable`, `msrv`, and `macos`. | parent |
| `architecture/tooling-governance.md` §"Known doc/code drift" | **Remove** the Open row at `:797` — rows are removed as they are fixed — and record the resolution in the **Resolved** table. | parent |
| `architecture/tooling-governance.md` §6.1 inventory | "Subsystem roadmaps \| 7" → 8, and the subsystem list gains `transport-footprint`. | parent |
| `plans/subsystems/README.md` | Add `transport-footprint-roadmap.md` to the subsystem list. | parent |
| `plans/registry.md` | Finding 12 → closed; a row in "Active subsystem roadmaps"; a row in "Dependency-ready implementation work"; the closure record path. Prose in §16. | parent |
| `plans/subsystems/transport-footprint-roadmap.md` §12 status row | M001 → closed, with the closure path. Prose in §16. | parent |
| `AGENTS.md` §"Verify" (CI paragraph) | The sentence "Neither `msrv` nor `windows-check` passes `--all-features`, so the feature-gated footprint binaries are built only by `stable` and `macos`" is now incomplete: `msrv` builds them by dedicated steps, and the footprint crate has a guard test. | parent |
| `AGENTS.md` §"Workspace layout" | The `eggup-transport-footprint` bullet gains the guard test and the CI coverage. | parent |
| Root `CHANGELOG.md` §`Unreleased` | One bullet. Prose in §16. | parent |

### 12.3 What must **not** be written

- No claim that the crate measures throughput, latency, reliability, or
  binary size in code. It does not.
- No re-measurement of the size figures from commit `ae9b47a`.
- No restatement of the old "cross-link is a build error" claim.
- No claim that the Windows lane builds the footprint binaries.
- No claim of a Windows or Linux dependency count. The 3 / 92 / 93 figures are
  the recorded macOS/Linux-measured capture; §15 requires per-platform
  reporting.

### 12.4 A normative document this milestone does **not** touch

`plans/003-planning-process.md` §17 enumerates the active Eggup roadmaps
("1. verified update core; 2. acquisition transport; 3. service lifecycle;
4. archive extraction; 5. Eggpack manifest interoperability; 6. consumer
adoption and compatibility"). Adding a seventh (transport footprint) makes
that enumeration incomplete. `plans/000`–`003` are normative and are not
edited in ordinary work (`AGENTS.md` §"Planning governance"; the file's own
§6.2 rule), so **this milestone does not edit it**. The parent session must
decide, and §16 carries the recommendation: the enumeration is a factual
index of active roadmaps, so adding the row is a factual correction rather
than a policy change, and the alternative is to record the residual as an
open low-severity documentation item owned by a later planning milestone.

## 13. Acceptance criteria

Each criterion is checkable and is mapped to evidence in §15.

1. The `stable` lane builds all three configurations —
   `--no-default-features --features curl`,
   `--no-default-features --features eggfetch`, and
   `--no-default-features --features curl,eggfetch` — with `--locked`, as
   three separate steps.
2. The `msrv` lane checks all three configurations at toolchain `1.89.0`,
   with `--locked`, as three separate steps.
3. The `macos` lane builds the curl-only and eggfetch-only configurations,
   with `--locked`, as two separate steps.
4. The `windows-check` lane and the `service-systemd` lane are **unmodified**,
   and the closure records the Windows binary coverage as `not run` with the
   §6.3 reason.
5. `crates/eggup-transport-footprint/tests/cross_link_guard.rs` exists and
   contains exactly the two guards of §10.1, using `std` only, with no
   `unsafe` block.
6. `cargo test -p eggup-transport-footprint --all-targets --locked --offline`
   reports **2 passed**, and the `no targets matched` warning that
   `architecture/transport-footprint.md:159-162` records is gone.
7. The three dependency-count commands from §11.1 print **3**, **92**, and
   **93**, in that order.
8. NC-1, NC-2, and NC-3 were each executed, each failed in the expected way
   with the exact output captured, and the tree was restored afterwards
   (`git status --short` shows only the intended changes).
9. `git diff --name-only` shows no path under `crates/*/src/**`, no
   `Cargo.toml`, and no `Cargo.lock`; `git diff --check` is clean.
10. `cargo fmt -p eggup-transport-footprint -- --check` exits 0, and the
    three crate-scoped clippy runs (§11.1) exit 0 with `-D warnings`.
11. The workspace-wide gate (§11.2) is green on the parent session's run, and
    the parent's `cargo tree --workspace --locked` review is accompanied by a
    read of the three scoped `cargo tree` runs, confirming the optional
    adapter edges are still optional and that no published crate's graph
    changed.
12. The documentation changes in §12.2 are applied by the parent session, and
    the accurate cross-link claim at
    `architecture/transport-footprint.md:177` survives unmodified in substance.
13. No publication, no tag movement, no version bump, and no
    `publish = false` → `publish = true` change occurred.
14. The closure record exists at `plans/closure/transport-footprint/001-status.md`
    and satisfies §15.

## 14. Stop conditions

Stop and escalate to the parent session; do not improvise past any of these.

1. **The dependency counts do not reproduce** as 3 / 92 / 93 under the §11.1
   commands. A dependency change has slipped in; the milestone's premise is
   gone.
2. **Any published crate's dependency graph changed**, or `Cargo.lock`
   changed. The milestone is CI-and-guard only; a lockfile change means
   something outside its scope happened.
3. **A guard cannot be made to fail** on a deliberately introduced cross-link.
   An assertion never observed to fail is not evidence (§10.2). Fix the
   assertion before landing it.
4. **The isolated configurations cannot be built** on the `stable` lane for a
   reason outside the footprint crate — for example a transitive dependency
   failing to compile at the pinned lockfile. Do **not** update `Cargo.lock`
   to work around it; that is a different milestone. Stop and report.
5. **The tree is not restored** after a negative control. Stop, restore, and
   re-verify before continuing.
6. **A concurrent publication lands** while this milestone is in flight.
   Re-read `plans/registry.md` and `docs/releases.md`, re-derive the version
   claims, and re-run §11.1 before continuing.
7. **The request is to publish, tag, bump a version, or change
   `publish = false`.** Out of scope; the crate is non-published by design.
8. **The request is to modify the `windows-check` lane or the local gate
   `scripts/check-local.sh`.** Both are deliberately out of scope (§6.3, §5);
   a Windows-parity request is a separate, bounded follow-on.
9. **The request is to add a real measurement capability** (size reporting,
benchmarks, throughput probes). That is a capability milestone, not a
corrective, and it needs its own decision (§10.3).
10. **A request arrives to edit `architecture/**`, `plans/registry.md`,
    `AGENTS.md`, the root `CHANGELOG.md`, or
    `plans/003-planning-process.md`.** Those are parent-owned in this batch;
    hand over the prose in §16 instead.

## 15. Closure evidence required

The closure record is written to **`plans/closure/transport-footprint/001-status.md`**
and MUST contain, per `plans/closure/README.md`:

- **Status**, source plan, source roadmap, reviewed repository baseline
  (`20ac770dacfdcd3d82e734c22613bfccd36f9c09`), and implementation
  commit(s).
- **Executive finding** — one paragraph stating what was unverified and what
  is now verified.
- **Requirement → evidence matrix** — one row per acceptance criterion in
  §13, with the **exact command** and the **exact observed output** (not the
  expected output).
- **Production implementation evidence** — the change-set proof:
  `git diff --name-only` (only `.github/workflows/ci.yml`,
  `crates/eggup-transport-footprint/tests/cross_link_guard.rs`, and the doc
  files), the `git diff --check` result, and the explicit statement that no
  `crates/*/src/**` path, no `Cargo.toml`, and no `Cargo.lock` changed.
- **Negative-control evidence** — the verbatim failing output of NC-1, NC-2,
  and NC-3, and the post-restore `git status --short`.
- **Exact commands and results** — every command in §11.1, with its actual
  output, and the parent's §11.2 gate output. A command that was not run is
  recorded as `not run`, never inferred.
- **Invariant review** — §4's eleven invariants, each marked intact or
  breached, with the evidence.
- **Failure/recovery review** — the three signatures of §8.2, with the
  observed NC-1 text confirming S2 and NC-2 confirming S3, and an explicit
  statement that S1 was not observed in this milestone.
- **Compatibility and migration review** — no public API change, no consumer
  impact, no migration.
- **Security review** — `unsafe_code = "deny"` holds in the new test; no
  `unsafe` block; no network, no credentials, no file written outside the
  working tree.
- **Documentation/operations evidence** — the §12.2 changes, each with its
  before/after `file:line` citation.
- **Unresolved findings with severity** — including the Windows binary
  coverage recorded as `not run`, and the `plans/003-planning-process.md` §17
  enumeration residual if the parent does not amend it.
- **Roadmap disposition** — M001 closed, with the roadmap's §12 status row
  and §11 completion definition updated.
- **Registry updates** — finding 12 closed, the milestone row, the
  subsystem row.

### 15.1 Per-platform reporting rule

Report natively, per platform, and **never infer one platform from another**
(`plans/closure/README.md`; `architecture/tooling-governance.md` §5.2):

| Platform | Footprint **binaries** built | Guard **test** executed | Dependency counts claimed |
|---|---|---|---|
| Linux (`stable`) | all three configurations — required | expected to run inside `cargo test --workspace --all-targets --all-features --locked` | only if the §11.1 commands were run on Linux |
| Linux (`msrv`, 1.89.0) | all three configurations, `cargo check` | compiled (the lane runs `cargo check --workspace --all-targets`) | not claimed — the counts are a resolution result, not a toolchain result |
| macOS (`macos`) | curl-only and eggfetch-only; dual via `--all-features` | expected to run | only if the §11.1 commands were run on macOS |
| Windows (`windows-check`) | **none — `not run`**, with the §6.3 reason | expected to run inside the lane's `cargo test --workspace --all-targets --locked --no-fail-fast` | **not claimed** — the transitive set of `eggup-eggfetch` is platform-dependent, and the recorded 3 / 92 / 93 are the macOS/Linux capture |

A platform that did not run is recorded as `not run`. It is never reported
green by analogy, and no count is claimed for a platform that did not
produce it.

## 16. Handoff notes

### 16.1 Sequencing

M001 touches only `crates/eggup-transport-footprint/tests/**`,
`.github/workflows/ci.yml`, and (via the parent) documentation. It does not
touch any crate's source, manifest, or API, so it cannot conflict with an
in-flight milestone in another crate. It may land before, between, or after
them.

The one interaction to watch: `crates/eggup-curl` has an in-flight edit in
this batch. M001's `cargo clippy`/`cargo test` runs are **crate-scoped to
`eggup-transport-footprint`** but still compile `eggup-curl` as a dependency,
so a broken in-flight `eggup-curl` will fail M001's local verification. That
is a rebase/ordering matter, not a scope matter: the parent owns the ordering
and the commits.

### 16.2 Ordered work packages (restated for the implementer)

1. **WP1** — Roadmap: **done** (`plans/subsystems/transport-footprint-roadmap.md`).
2. **WP2** — Add `crates/eggup-transport-footprint/tests/cross_link_guard.rs`
   with T1 and T2.
3. **WP3** — Add the three lanes' steps to `.github/workflows/ci.yml` (§6.2).
4. **WP4** — Run NC-1, NC-2, NC-3; capture the failing output; restore.
5. **WP5** — Run the §11.1 crate-scoped verification; record the output.
6. **WP6** — Apply the implementing session's own doc edits (none outside the
   new test file — see §12.1) and hand the §12.2 prose to the parent.
7. **WP7** — Closure: `plans/closure/transport-footprint/001-status.md`.

### 16.3 Intended `plans/registry.md` prose (for the parent to apply)

**Finding 12**, in §"Post-0.1.3 open findings" — replace the entry's closing
paragraph with:

> **Closed by Transport Footprint M001**
> (`plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md`;
> closure `plans/closure/transport-footprint/001-status.md`). The isolated
> single-adapter configurations are now built by CI — `stable` and `macos`
> build them with `cargo build`, `msrv` checks them at the 1.89 floor — and a
> linkage-discipline guard rejects a sibling-adapter reference in every
> configuration. The crate's dependency counts 3 / 92 / 93 still reproduce
> exactly. The `windows-check` lane still builds none of the three binaries,
> before and after; that is recorded as `not run`, not inferred.

**"Active subsystem roadmaps" table** — new row:

| Transport footprint | M001 open; the subsystem roadmap is authored and registers M001 as its only milestone | M001 isolated-configuration coverage corrective — closes registry finding 12 (medium); `ready for handoff` |

**"Dependency-ready implementation work" table** — new row:

| Transport footprint | M001 isolated-configuration coverage corrective | closed <date>; hosted-qualified (<run>) | `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md`; `plans/closure/transport-footprint/001-status.md` | Closes open finding 12 (**medium**). CI builds the curl-only, eggfetch-only, and both-adapters configurations; a linkage-discipline guard rejects a sibling-adapter reference in every configuration, with a discriminating negative control. Counts 3 / 92 / 93 unchanged. `publish = false`; unpublished; no tag moved |

### 16.4 Intended roadmap status-row prose (for the parent to apply after implementation)

Replace the M001 row in `plans/subsystems/transport-footprint-roadmap.md` §12
with:

| M001 isolated-configuration coverage corrective | closed <date>; hosted-qualified (<run>) | `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md` | `plans/closure/transport-footprint/001-status.md` | Closes registry open finding 12 (**medium**). All three configurations are built by CI; the linkage guard is observed to discriminate. Counts 3 / 92 / 93 unchanged; `publish = false` preserved |

### 16.5 Intended root `CHANGELOG.md` bullet (for the parent to apply)

Add to the root `CHANGELOG.md` §`Unreleased`:

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

### 16.6 The `plans/003-planning-process.md` §17 residual

The parent must decide whether to add `transport footprint` to the active
roadmap enumeration in `plans/003-planning-process.md` §17. **Recommendation:
add it.** The enumeration is a factual index of the active roadmaps, and the
file's own §6.2 rule protects normative *decisions*, not factual indices; an
incomplete index is itself drift. If the parent declines, M001's closure
records the residual as an open **low**-severity documentation item owned by a
later planning milestone, and the roadmap's §12 M001 row states it.

### 16.7 What this plan does **not** authorize

- It does not authorize publishing, tagging, versioning, or changing
  `publish = false`.
- It does not authorize editing `crates/eggup-transport-footprint/src/bin/*.rs`
  or `crates/eggup-transport-footprint/Cargo.toml`.
- It does not authorize editing `scripts/check-local.sh` or the
  `windows-check` lane.
- It does not authorize a dependency, a `Cargo.lock` update, a new dev-dependency,
  or a TOML-parsing dependency in the guard test.
- It does not authorize editing `architecture/**`, `plans/registry.md`,
  `AGENTS.md`, the root `CHANGELOG.md`, `plans/subsystems/README.md`,
  `plans/003-planning-process.md`, or any other plan or closure record — those
  are the parent session's, and the prose is in §16.3–§16.6.
- It does not authorize a measurement capability, a benchmark, or a size
  assertion.
- It does not authorize claiming any dependency count for a platform that did
  not run the §11.1 commands.
