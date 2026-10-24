# Transport Footprint Roadmap

Status: active; M001 closed 2026-10-23 and hosted-qualified in run `40062299630` — `001-isolated-configuration-coverage-corrective.md`, closure at `plans/closure/transport-footprint/001-status.md`; no other milestone is registered; the crate has never been published and is not publishable by design (`publish = false`)

Long-term references:

- `plans/000-long-term-specification.md#4-crate-and-layer-model`
- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`
- `plans/000-long-term-specification.md#20-platform-scope`
- `plans/000-long-term-specification.md#24-testing-requirements`
- `plans/001-terminology-and-domain-model.md#10-fetcher`
- `plans/001-terminology-and-domain-model.md#36-core-boundary-rule`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

## 1. Purpose and ownership boundary

This subsystem owns the **build-time and link-time fixture crate**
`crates/eggup-transport-footprint`, whose purpose is to make one decision
measurable: which HTTP transport stack a consumer of `eggup-acquisition` links
into its binary.

It owns:

- the three fixture binaries — `curl_only`, `eggfetch_only`, `dual` — and the
  `required-features` split that makes an isolated single-adapter
  configuration representable;
- the crate's manifest shape: which dependencies are unconditional, which are
  optional, and which feature enables which optional edge;
- the CI coverage that proves the isolated configurations actually build;
- the linkage-discipline guard that keeps a single-adapter binary from
  referencing the sibling adapter.

It does **not** own:

- **transport policy.** Which adapter a consumer uses is the consumer's
  decision. This subsystem never selects one, never defaults one, and never
  ranks them.
- **acquisition semantics.** Redirect, timeout, proxy, TLS, redaction, and
  size-bound behavior belong to `eggup-acquisition` and the two adapters. This
  subsystem owns none of it and changes none of it.
- **publication.** The crate is `publish = false` by design
  (`crates/eggup-transport-footprint/Cargo.toml:8`); it has no registry
  version, and `plans/registry.md` records it as such. Nothing in this
  subsystem may make it publishable.
- **the real footprint measurement method.** The crate performs **no
  measurement in code**. Its three binaries construct a transport, construct a
  request, and print the configuration they resolve to
  (`crates/eggup-transport-footprint/src/bin/curl_only.rs:6-13`,
  `src/bin/eggfetch_only.rs:6-16`, `src/bin/dual.rs:5-16`). The measurable
  quantity is the *resolved dependency graph* and the *linked binary size*,
  both of which are observed externally. This roadmap therefore claims no
  measurement capability, no throughput number, no latency number, and no
  reliability number, and no milestone in it may introduce one without a
  separate decision.

## 2. Work classification

### Invariants

- the crate stays `publish = false` and keeps no registry version;
- the recorded dependency counts **3 / 92 / 93** (curl-only / eggfetch-only /
  both adapters) must not change;
- the `required-features` split exists so that an isolated single-adapter
  configuration is *representable*; removing it would delete the crate's only
  reason to exist;
- the crate must never gain a dependency on `eggup-core`, `eggup-service`,
  `eggup-archive`, or `eggup-eggpack`; its only unconditional dependency is
  `eggup-acquisition`, and its only optional dependencies are `eggup-curl`
  and `eggup-eggfetch`;
- `eggup-curl` must not learn that `eggup-eggfetch` exists, and
  `eggup-eggfetch` must not learn that `eggup-curl` exists;
- the binaries perform no measurement and assert no transport policy;
- SHA-256, where it appears at all, is integrity evidence only; no
  authenticity or signature claim is made by anything in this subsystem.

### Capabilities

- represent the three transport configurations as build targets;
- make the *cost* of the transport choice visible in a consumer's dependency
  graph and linked image;
- demonstrate that the seam owns redaction, bounds, and composition, and that
  the adapters own transport policy;
- keep the isolated single-adapter configurations built by CI, so the claim
  "this binary links this stack" is machine-checked rather than asserted;
- reject a cross-link between the two adapters in every configuration.

### Infrastructure

- the `eggup-transport-footprint` package: `Cargo.toml` plus
  `src/bin/{curl_only,eggfetch_only,dual}.rs`;
- the `curl` / `eggfetch` / `default` feature surface, with the optional
  dependencies declared in the conventional `dep:` form so no implicit
  `dep:`-prefixed feature is exposed;
- the CI steps that build the isolated configurations;
- the linkage-discipline guard test.

### Polish

- rustdoc on the three binaries (the only documentation surface the crate has
  — it has no `README.md` and no `CHANGELOG.md`);
- richer "what configuration did I just build" output from the binaries;
- an optional size-report helper, only if a consumer ever asks for one.

## 3. Non-goals

- choosing a transport for any consumer, or recommending one;
- making the seam or either adapter faster, smaller, or safer;
- adding a `lib.rs`, a public API, or a library target to the crate;
- adding a `README.md` or a `CHANGELOG.md` to the crate. It is binary-only and
  non-published, and is the sole documented exception to the library-crate
  rule (`architecture/tooling-governance.md` §3.4);
- publishing the crate, ever, under any tag or registry;
- measuring throughput, latency, redirect handling, proxy behavior, or TLS
  posture — those are adapter concerns and are not properties of a link graph;
- changing the local gate's shape. `scripts/check-local.sh` is a uniform,
  non-specialized 5-command gate; this subsystem does not add
  crate-specific lines to it;
- re-deciding the dependency pin style (`"0.1.0"` versus `=0.1.3`) that
  `architecture/tooling-governance.md` §"Known doc/code drift" carries as an
  open row for a separate workspace-wide milestone.

## 4. Current state evidence

Baseline: commit `20ac770`, branch `plans/corrective-batch-2026-10-23`.

The crate is **three binaries, 45 source lines, zero tests**:

| Binary | Path | `required-features` | Lines |
|---|---|---|---|
| `curl_only` | `crates/eggup-transport-footprint/src/bin/curl_only.rs` | `curl` | 13 |
| `eggfetch_only` | `crates/eggup-transport-footprint/src/bin/eggfetch_only.rs` | `eggfetch` | 16 |
| `dual` | `crates/eggup-transport-footprint/src/bin/dual.rs` | `curl`, `eggfetch` | 16 |

The manifest (`crates/eggup-transport-footprint/Cargo.toml`) declares
`publish = false` at `:8`, `[lints] workspace = true` at `:12-13`, the single
unconditional dependency at `:16`, the feature surface at `:18-21`, the two
optional adapter dependencies at `:23-31`, and the three `[[bin]]` targets with
their `required-features` at `:33-46`.

**What the crate proves today, and what it does not.** The crate's central
claim is that a consumer can build a fetching updater that links exactly one
transport stack. `architecture/transport-footprint.md` §"Invariants and what
this crate enforces" states that claim accurately as of planning C013:

> `required-features` stops a binary from *building* when its adapter's
> feature is off. It does **not** stop a cross-link when both features are on,
> because an enabled optional dependency becomes available to every target in
> the package.

That statement is correct and this roadmap does not contradict it. The
earlier wording — that the split "makes an accidental cross-link a build
error" — was inaccurate and was corrected at C013.

**The CI coverage gap is open.** `architecture/tooling-governance.md` §5.3 and
§"Known doc/code drift" record it: the five CI lanes are `stable`, `msrv`,
`service-systemd`, `macos`, and `windows-check`. Only `stable` and `macos`
pass `--all-features`; `msrv` and `windows-check` run
`cargo check`/`cargo test --workspace --all-targets --locked` with default
features only, and `default = []`. **No lane builds either isolated
single-adapter configuration**, and `msrv` and `windows-check` compile none of
the three binaries at all. The local gate
(`scripts/check-local.sh:5-6`) likewise builds only `--all-features`, i.e.
only the dual configuration. The configuration the crate exists to
represent is therefore the one configuration no machine builds.

**The dependency counts reproduce exactly.** Verified at `20ac770` with the
counting method recorded by planning C013
(`plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md` §11):

```sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
```

Observed: **3**, **92**, **93**. These are the figures published in
`architecture/transport-footprint.md` §"The decision this crate exists to
expose", and they must not change.

Two counting subtleties, recorded so a future reader does not "correct" the
figures: the `(*)` dedup markers `cargo tree --prefix none` emits on repeat
nodes must be excluded, or the counts read 118 and 119; and the counts are
*unique tree lines*, not unique crate names — `webpki-roots` resolves at two
versions (`0.26.11` and `1.0.9`) in the eggfetch graph, so a name-only count
would read 91 and 92. The C013 method is the recorded method.

**Registry finding 12** (`plans/registry.md`, §"Post-0.1.3 open findings", entry
12) is the open, registered, **medium**-severity finding this subsystem owns:
the crate's central claim is not verified by any CI job, and the crate had no
subsystem roadmap. The roadmap prerequisite is discharged by this document;
the CI-coverage half is owned by M001.

## 5. Target architecture

```text
consumer chooses a transport (never this subsystem)
        |
        v
eggup-acquisition  -- seam: redaction, bounds, composition
        |
        +--> eggup-curl       (optional; feature "curl")
        +--> eggup-eggfetch   (optional; feature "eggfetch")
        |
        v
eggup-transport-footprint  (publish = false; binary-only)
  |
  +--> curl_only      required-features = ["curl"]          -> seam + curl only
  +--> eggfetch_only  required-features = ["eggfetch"]      -> seam + Eggfetch only
  +--> dual           required-features = ["curl","eggfetch"] -> seam + both + composition
        |
        v
CI builds all three configurations on every lane that builds the crate
        |
        v
linkage-discipline guard rejects a sibling-adapter reference in any configuration
```

The *architecture* being targeted is not a code shape; the crate's code is
already minimal and correct for its purpose. The target is an **evidence
architecture**: three build configurations, three CI steps that build them,
and one guard that makes the isolation claim machine-checkable in every
configuration rather than in one.

## 6. Dependency graph

```text
eggup-acquisition (zero-dependency seam)
        |
        +--> eggup-curl (optional; feature "curl")
        +--> eggup-eggfetch (optional; feature "eggfetch")
        |
        v
eggup-transport-footprint (publish = false; binary-only)
        |
        v
M001 isolated-configuration coverage corrective [CLOSED 2026-10-23]
```

Dependency classes for M001:

| Dependency | Class | Note |
|---|---|---|
| `eggup-acquisition` seam contract | **interface** | Stable and published; M001 consumes it and changes it not at all |
| `eggup-curl` / `eggup-eggfetch` adapter contracts | **interface** | Published (`0.1.2` / `0.1.3`); M001 consumes their types and changes them not at all |
| Planning C013 closure | **hard** | Closed. It corrected the architecture claim and registered finding 12, naming this roadmap as the prerequisite |
| CI workflow edit (`.github/workflows/ci.yml`) | **soft** | The step may land independently of any other in-flight milestone; the workspace-wide gate the parent runs is the integration point |
| Workspace-wide gate run | **operational** | The parent session owns the workspace-wide run and the commit |

Nothing in this subsystem is **hard**-dependent on another subsystem's open
milestone.

## 7. Milestones

### M001 — Isolated-configuration coverage corrective

Plan: `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md`.

Status: **closed** 2026-10-23 and hosted-qualified in run `40062299630` — closure
`plans/closure/transport-footprint/001-status.md`.

Class: invariant (it closes a registered **medium** finding, registry entry 12).

Hard dependency: planning C013 closure (closed).

Close the CI-coverage half of registry finding 12. Build all three transport
configurations — curl-only, eggfetch-only, both adapters — in the CI lanes
that build the crate, and add a linkage-discipline guard that rejects a
sibling-adapter reference in **every** configuration, not only the isolated
ones. Pin the 3 / 92 / 93 counts as regression evidence. No production source
change, no manifest change, no dependency change, no publication.

### Next milestone disposition

**M001 is the only milestone registered in this subsystem.** There is no
authored M002 and none is proposed here. Two candidates are deliberately left
**unwritten** rather than blocked, because a milestone needs consumer
evidence to justify its existence:

- *Windows parity for the isolated configurations.* The `windows-check` lane
  builds none of the three footprint binaries today, and M001 leaves it that
  way by decision (the crate has no `#[cfg]` attribute and no platform-specific
  code, so a Windows build proves nothing about the property being guarded).
  If a maintainer wants platform parity for the *counts*, that is a
  separate decision, because the transitive crate set of `eggup-eggfetch` is
  platform-dependent and the recorded 92 / 93 were captured on macOS.
- *A size-report helper.* The crate performs no measurement in code. Adding one
  is a capability change, not a corrective, and needs consumer demand first.

Until a consumer or a maintainer asks for one of these, this subsystem's
active work is exactly M001.

## 8. Cross-cutting requirements

- **Storage.** None. The crate writes nothing at runtime; its binaries only
  print to stdout.
- **Protocol.** None. No network access, no HTTP semantics, no TLS posture.
  The crate constructs transports but never performs a fetch.
- **Security.** No unsafe code (`unsafe_code = "deny"` is inherited through
  `[lints] workspace = true` at `Cargo.toml:12-13`; the crate has no
  `lib.rs`, so it cannot carry the library-crate `#![forbid(unsafe_code)]` /
  `#![deny(missing_docs)]` attributes — `architecture/tooling-governance.md`
  §3.4). No credentials, no secrets, no URL material beyond the hard-coded
  `https://example.com/artifact` request the binaries print redacted.
  `curl_only` hardcodes `/usr/bin/curl` as a *fixture value*, not a discovery
  default.
- **Recovery.** None. The crate has no transaction, no lock, and no
  recoverable state. A build failure leaves a build failure.
- **Performance.** The crate's performance claim is the *size* of what it
  links. The recorded figures are a dated capture
  (`architecture/transport-footprint.md` §"The decision this crate exists to
  expose") and are not re-measured by this subsystem. The dependency counts
  **are** re-verified, because they are the property the crate's manifest
  shape controls.
- **Documentation.** The crate's documentation surface is the three binaries'
  module docs plus `architecture/transport-footprint.md`. The crate
  deliberately has no `README.md` and no `CHANGELOG.md`; the root
  `CHANGELOG.md` is where repository-level changes are recorded.
- **Toolchain.** Rust 1.89 (`rust-version.workspace = true`), edition 2021,
  resolver 2. The MSRV floor applies to the feature-gated binaries exactly as
  it applies to everything else, which is why M001 adds the isolated
  configurations to the `msrv` lane.

## 9. Verification strategy

Verification for this subsystem is **build-matrix and graph verification**,
not runtime testing. There is no behavior to test: the binaries perform no
measurement and no fetch.

The required evidence is:

1. **All three configurations build** — curl-only, eggfetch-only, both
   adapters — in every CI lane that builds the crate.
2. **The isolation claim is machine-checked** — a sibling-adapter reference in
   a single-adapter binary is rejected by a guard that is observed to
   *fail* when such a reference is introduced. An assertion never observed to
   fail has not been shown to work.
3. **The dependency counts reproduce** — 3 / 92 / 93, by the C013 counting
   method in §4, with `--locked`.
4. **The guard runs on every platform** — it is source-level and therefore
   platform-independent, and it executes wherever `cargo test --workspace
   --all-targets` runs, which includes the Windows lane.
5. **Per-platform honesty** — a platform that did not run is recorded as
   `not run` and never inferred (`plans/closure/README.md`;
   `architecture/tooling-governance.md` §5.2).

## 10. Risks and decision points

**The largest risk is mistaking a build for a measurement.** The crate's
name, its roadmap, and the sibling subsystem's "footprint measurement" polish
item all invite the claim that the crate measures something. It does not. Its
binaries print a resolved configuration; the measurable facts are the
dependency graph and the linked image, both observed externally. Any future
milestone that wants to *measure* must say so explicitly and separately.

**The second risk is a guard that cannot discriminate.** A guard that passes
on a tree with a cross-link proves nothing. M001 therefore requires a
negative control for every guard it adds, and requires the negative control's
failing output to be recorded.

**The third risk is over-claiming isolation.** `required-features` does not
make a cross-link a build error in the dual configuration. The accurate claim
is narrow and must stay narrow: the *isolated* configurations make a
single-adapter binary's linkage machine-checked; the guard extends that to
every configuration; nothing else does.

**The fourth risk is scope creep into the local gate or the workspace
dependency-pin style.** `scripts/check-local.sh` is uniform by design, and
the pin-style question is a workspace-wide milestone already registered as an
open drift row. Neither belongs to M001.

**Decision points.**

| Decision | Choice in M001 | Alternative | Why |
|---|---|---|---|
| Which lanes build the isolated configurations | `stable`, `msrv`, `macos` | also `windows-check` | The crate has no `#[cfg]` attribute and no platform-specific code; a Windows build adds cost to the scarcest lane and proves nothing about the guarded property. Windows coverage of the *binaries* stays `not run` — exactly as it is today — and is recorded, not inferred |
| CI-only, or CI plus a crate-side guard | **both** | CI-only | An isolated build catches a plain cross-link but not a `#[cfg]`-gated one; the guard test catches both |
| Pin the 3 / 92 / 93 counts as an automated test | **no** — verified as commands | a test that shells out to `cargo tree` | The counts are a property of a 92-crate resolution and would be brittle against upstream churn for no safety gain; the stable thing worth asserting is the manifest shape that produces them |
| Does the crate keep zero tests | **no** — one guard test file | keep zero tests | The crate's whole output is a claim; a claim with no check is the finding. The binaries still perform no measurement and are still not exercised at runtime |
| Change the local gate | **no** | add three crate-specific lines | The gate is uniform and non-specialized; the guard test runs in the gate's existing `cargo test` step, so local coverage is preserved |

## 11. Completion definition

The subsystem is complete when:

1. every CI lane that builds the crate builds all three configurations, or
   records explicitly that it does not and why;
2. a sibling-adapter reference in a single-adapter binary is rejected in
   **every** configuration, by a guard that has been observed to fail on a
   deliberately introduced cross-link;
3. the dependency counts 3 / 92 / 93 reproduce with the recorded counting
   method, and the manifest shape that produces them is asserted;
4. the architecture deep dive states the accurate claim — the corrected one
   from C013, extended with what CI now proves — and does not claim a
   measurement capability the crate does not have;
5. no milestone, document, or registry row claims that the crate measures
   throughput, latency, reliability, or binary size in code;
6. the crate remains `publish = false`, has no registry version, and no
   milestone in this subsystem publishes or tags anything;
7. no published crate's dependency graph changed as a result of this
   subsystem's work.

M001 satisfies 1–7 for the coverage and guard half. Items 4 and 5 are
documentation obligations that M001's closure must carry as evidence.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 isolated-configuration coverage corrective | closed 2026-10-23; hosted-qualified (run `40062299630`); closes registry open finding 12 (**medium**) | `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md` | `plans/closure/transport-footprint/001-status.md` | — |

**Next milestone.** M001 is the only milestone in this subsystem, and it is
closed. There is no blocked milestone and no unwritten-but-authorable
milestone. The two candidates in §7 are intentionally unwritten pending
consumer evidence, not blocked.

**M001-UF-3 — the roadmap enumeration in `plans/003-planning-process.md` §17.**
Severity: **low**. That file enumerates the active roadmaps and does not yet
name this one. `plans/003-planning-process.md` is a normative planning
document, and this repository's rule is that `plans/000`–`003` are not edited
in ordinary work, so the parent session did **not** amend the enumeration as
part of M001. The item is carried here and in the closure record as an open
low-severity documentation drift for a later planning milestone.
