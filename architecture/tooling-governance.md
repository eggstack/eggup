# Tooling and Planning Governance — Deep Dive

The cross-cutting layer: what a change must satisfy before it is allowed to land.
Grounded in `Cargo.toml`, `scripts/check-local.sh`, `.github/workflows/ci.yml`,
and `plans/`. Factual as of the current tree.

## 1. Purpose

Every other deep dive in `architecture/` describes *what a subsystem does*. This
one describes *how a change is admitted*: the workspace manifest and its
dependency boundaries, the three lint layers, the single local verification
script, the four-job CI matrix, and the plan → implement → closure → registry
process that produces the evidence a reviewer checks.

The governing idea is separation of concerns applied to verification itself:

- `scripts/check-local.sh` is the **pre-push gate** — fast, single-platform, and
  the only place a `cargo tree` review happens.
- `.github/workflows/ci.yml` is the **admission gate** — it adds the toolchain
  floor and the two platforms the local script structurally cannot reach.
- `plans/` is the **evidence record** — it is where a change states its
  baseline, its non-goals, and its verification results, and where a reviewer
  reads whether the code and the claimed evidence agree.

A change is landed by satisfying all three. Passing the script without a plan or
closure record is a code change with no evidence trail; a plan with no green
lane is a claim.

## 2. The workspace manifest

`Cargo.toml` is 25 lines and is the only place workspace-wide policy is declared.

### 2.1 Members and resolver

`Cargo.toml:2` sets `resolver = "2"`. `Cargo.toml:3-12` lists **8 members**:

| Member | Published | Role |
|---|---|---|
| `crates/eggup-core` | yes | Policy-neutral transaction mechanics; sole dep `sha2` |
| `crates/eggup-acquisition` | yes | Transport-neutral seam; **zero dependencies** |
| `crates/eggup-eggfetch` | yes | Native HTTP adapter |
| `crates/eggup-curl` | yes | External-curl adapter |
| `crates/eggup-archive` | yes | Bounded allowlisted local extraction |
| `crates/eggup-eggpack` | yes | Optional Eggpack ReleaseManifest v1 adapter |
| `crates/eggup-service` | no (unpublished) | Manager-neutral service lifecycle |
| `crates/eggup-transport-footprint` | no (`publish = false`) | Footprint fixture binaries |

`resolver = "2"` is load-bearing rather than cosmetic: the workspace mixes
edition-2021 members and platform-gated target dependencies, and resolver 1
would unify features across the build graph, which would let an optional
dependency of one crate silently activate in another.

### 2.2 The five shared keys

`[workspace.package]` (`Cargo.toml:14-19`) declares exactly five keys:

```toml
version = "0.1.2"
edition = "2021"
rust-version = "1.89"
license = "MIT"
repository = "https://github.com/eggstack/eggup"
```

All 8 crates inherit all five via `*.workspace = true` (verified: every
`crates/*/Cargo.toml` contains five such keys) **and** opt into the lint table
with `[lints] workspace = true`. The count of `workspace = true` occurrences per
manifest is 6 for every crate — five shared keys plus the lint opt-in. A new
crate that adds a sixth `[workspace.package]` key, or omits the lint opt-in,
breaks a uniform invariant that currently holds across the workspace.

`rust-version = "1.89"` is what makes the `msrv` CI job meaningful; the pin is
duplicated as `dtolnay/rust-toolchain@1.89.0` in `.github/workflows/ci.yml:29`
and is **not** pinned locally — there is no `rust-toolchain.toml` in the
repository, so a local run uses whatever default toolchain the machine has.

### 2.3 New-crate requirements

A crate joining the workspace must, at minimum:

1. be added to `[workspace] members`;
2. inherit all five shared keys via `*.workspace = true`;
3. set `[lints] workspace = true`;
4. carry `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` in `lib.rs` if it
   is a library (all 7 library crates do — see §3.3);
5. carry a `README.md` and a `CHANGELOG.md` with an `Unreleased` section (all 7
   published crates plus the unpublished `eggup-service` do; the fixture crate
   is the sole exception, see §3.4).

### 2.4 Dependency boundaries

The dependency graph is the enforcement mechanism for the layered design, and
it is checked only by `cargo tree` — which is why §4 flags `cargo tree` as
review-critical.

| Crate | Dependencies | Boundary rule |
|---|---|---|
| `eggup-core` | `sha2 = "0.10"` | Sole dep. No transport, no service manager, no Eggpack |
| `eggup-acquisition` | *(none)* | Zero-dependency seam |
| `eggup-eggfetch` | `eggup-acquisition`, `eggfetch-core`, `tokio`, `futures-util` | Must not know `eggup-curl` exists |
| `eggup-curl` | `eggup-acquisition` | Must not know `eggup-eggfetch` exists; no embedded HTTP/TLS stack |
| `eggup-archive` | `flate2`, `fs_at`, `sha2`, `tar`, `zip` (+ dev-dep `eggup-core`) | No transport, no authenticity, no live-install policy |
| `eggup-service` | `eggup-core`, `windows-args` (+ `windows-service` on Windows) | Manager-neutral; `windows-service` is `cfg(windows)`-gated |
| `eggup-eggpack` | `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggpack-manifest = "=0.1.0"`, `sha2` | Only crate allowed to touch producer types |
| `eggup-transport-footprint` | `eggup-acquisition` (+ optional `eggup-curl` / `eggup-eggfetch`) | Composition lives in the seam, not in an adapter |

Two boundary properties worth naming explicitly because a manifest edit can
silently break them:

- `eggup-acquisition` has an **empty** `[dependencies]` table. It is the only
  member with no dependencies at all, which is what lets both adapters and the
  footprint fixtures depend on it without dragging in an HTTP stack.
- `eggup-archive` depends on `eggup-core` as a **dev-dependency only**
  (`crates/eggup-archive/Cargo.toml`). Core does not depend on archive; the
  arrow is one-way, and the test fixtures are what need the transaction types.

**Exact-pin asymmetry.** `eggup-eggpack` pins its in-workspace deps with exact
requirements (`=0.1.2`) and takes `eggpack-manifest` as `=0.1.0` from the
registry. Every other crate uses a caret requirement — and several still carry
the literal `"0.1.0"` even though the workspace version is now `0.1.2`
(`eggup-eggfetch` → `eggup-acquisition`, `eggup-curl` → `eggup-acquisition`,
`eggup-service` → `eggup-core`, and all three in
`eggup-transport-footprint`). These resolve correctly through the path
dependencies, so the build is not broken, but the literals are stale relative
to `[workspace.package] version`. The pin style also differs by crate for no
stated reason, which means a version bump has to be reasoned about per-crate
rather than applied uniformly.

## 3. Lint policy

Three layers, each catching something the others cannot.

### 3.1 Layer 1 — workspace lint table

```toml
[workspace.lints.rust]
unsafe_code = "deny"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
```

(`Cargo.toml:21-25`.) The `priority = -1` on clippy `all` places it below
crate-level lint config, so an individual crate can raise a lint to `deny`
without the workspace table overriding it. This table applies to all 8 crates
through the `[lints] workspace = true` opt-in.

`unsafe_code = "deny"` is workspace-wide, so it also covers the fixture crate
that carries no `lib.rs` and therefore no crate-level attribute.

### 3.2 Layer 2 — clippy escalation at the command line

Both `scripts/check-local.sh:5` and `.github/workflows/ci.yml:20` pass
`-- -D warnings` to clippy. The workspace table only sets `warn`; the
`-D warnings` flag is what converts warnings into build failures. This is the
only place the escalation happens, so the flag is load-bearing: removing it
would silently reduce clippy from a gate to advisory output.

The `all` group is enabled, so `clippy::all` covers the correctness and
suspicious lints. Notably, `clippy::pedantic` and `clippy::nursery` are **not**
enabled, and there is no `[workspace.lints.rust]` entry beyond `unsafe_code`.

### 3.3 Layer 3 — per-crate attributes

All 7 library crates open `src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
```

Verified at `crates/{eggup-acquisition,eggup-archive,eggup-core,eggup-curl,
eggup-eggfetch,eggup-eggpack,eggup-service}/src/lib.rs:1-2`.

`forbid` is stronger than `deny` and cannot be overridden by an inner attribute,
so this is a hard stop even if the workspace table were relaxed.
`deny(missing_docs)` is what makes `cargo doc` a meaningful publication gate:
every public item needs rustdoc, which is also a `plans/003-planning-process.md`
§16 requirement ("exported types require docs").

### 3.4 The exception: `eggup-transport-footprint`

`crates/eggup-transport-footprint/` contains only `Cargo.toml` and three
`src/bin/*.rs` files. There is no `lib.rs`, therefore:

- it cannot carry `#![forbid(unsafe_code)]` or `#![deny(missing_docs)]`, since
  those are crate-root attributes and it has no library crate root;
- it relies entirely on layer 1 (`[lints] workspace = true`, confirmed at
  `crates/eggup-transport-footprint/Cargo.toml:12-13`) for `unsafe_code = "deny"`;
- it has **no `README.md` and no `CHANGELOG.md`** — the only crate of the 8
  without them.

The consequence is bounded but real: a `missing_docs` violation in this crate
would not be caught, and its public-facing documentation surface is the
`architecture/transport-footprint.md` deep dive plus rustdoc on the binaries
rather than crate-level docs. This is a reasonable trade for a
non-published, binary-only fixture crate, but it means the "every crate has a
README and CHANGELOG" rule that AGENTS.md states does not hold uniformly, and
`crates/eggup-transport-footprint` is the reason.

## 4. The local gate

`scripts/check-local.sh` is the whole gate — 6 lines, no conditionals, no
argument handling, no per-crate specialization.

```sh
#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree --workspace --locked
```

| Line | Command | What it proves |
|---|---|---|
| 4 | `cargo fmt --all -- --check` | Formatting is canonical across every member |
| 5 | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | No clippy `all` warning anywhere, in tests/examples/benches too, with all features on |
| 6 | `cargo test --workspace --all-targets --all-features --locked` | Tests pass with the full feature union; `--all-targets` includes doc-test targets |
| 7 | `cargo doc --workspace --no-deps --locked` | `deny(missing_docs)` holds workspace-wide and rustdoc builds clean |
| 8 | `cargo tree --workspace --locked` | Prints the resolved graph for human review |

`set -euo pipefail` (line 2) means the script **is** a gate: the first failing
command aborts the script, and a non-`cargo` failure in a pipeline propagates.

### 4.1 The `cargo tree` caveat

`cargo tree` is a **review step, not an assertion**. It always exits 0 on a
resolvable graph — it prints. Its only failure mode is an unresolvable
dependency graph, which `cargo check` would already have caught. The comment in
`.opencode/skills/verify-workflow/SKILL.md` ("dependency-surface review, no
gate") states this correctly, and `AGENTS.md` agrees ("review-only, no gate").

This is a genuine hole in the enforcement model, and the specific consequence
is worth stating: **the dependency-boundary rules in §2.4 are enforced by human
review, not by any automated check.** Nothing in the script or in CI fails when
a transport dependency is added to `eggup-core` or when `eggup-curl` learns
about `eggup-eggfetch`. Clippy and tests stay green; only a reviewer reading
`cargo tree` output catches it. Since `cargo tree` is also the one command CI
never runs (§5.3), the boundary check is the step most likely to be skipped
under time pressure, and it is the one that protects the layered design.

## 5. The CI matrix

`.github/workflows/ci.yml` — triggers `push` and `pull_request` (`ci.yml:3-6`),
`permissions: contents: read` (`ci.yml:7-8`), four jobs.

| Job | Runner | Toolchain | Exact commands | What it uniquely proves |
|---|---|---|---|---|
| `stable` | `ubuntu-latest` | stable + `rustfmt`, `clippy` | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-targets --all-features --locked`; `cargo doc --workspace --no-deps --locked` | Linux is the reference platform. The only lane that runs clippy, fmt, and doc |
| `msrv` | `ubuntu-latest` | `1.89.0` (`ci.yml:29`) | `cargo check --workspace --all-targets --locked` | The code type-checks at the declared `rust-version` floor. Clippy and tests are not run here, so this is a compile check only |
| `macos` | `macos-latest` | stable | `cargo test --workspace --all-targets --all-features --locked` | The full test suite passes on a non-Linux Unix — the only evidence for macOS behavior |
| `windows-check` | `windows-latest` | stable | `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked`; `cargo test -p eggup-eggpack --all-targets --all-features --locked`; three `cargo test -p eggup-service --lib <name> --locked` steps; `cargo test -p eggup-service --lib windows_scm::tests --locked`; `cargo check --workspace --all-targets --locked` | Windows compiles the whole workspace and *runs* the platform-sensitive subset |

### 5.1 Why `windows-check` is targeted rather than a full workspace test run

A plain `cargo test --workspace --all-features` on `windows-latest` is not run.
The job instead names specific crates and specific tests. The selection reflects
where Windows-specific behavior actually lives:

- `eggup-acquisition`, `eggup-archive`, `eggup-curl` — filesystem and process
  behavior that differs from POSIX. `eggup-archive` in particular has
  `fs_at` handle-backed extraction with Windows-specific cleanup semantics
  (see [archive-extraction.md](archive-extraction.md)).
- `eggup-eggpack` — full `--all-targets --all-features` run of the adapter
  integration tests.
- `eggup-service` — the manager adapter crate. The final
  `cargo check --workspace --all-targets --locked` covers the rest at
  compile level, so crates with no Windows-specific behavior still get a
  Windows type-check without paying for a full test run.

The three individually-named `eggup-service --lib` tests are the
UTF-8-boundary and control-character safety tests:

- `diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges`
- `service_errors_output_and_permission_remediation_stay_utf8_bounded`
- `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free`

They are named individually rather than run as a module filter because they
are *diagnostic-safety* assertions, and they are meaningful on Windows
specifically: these tests exercise how failure detail is rendered into bounded
byte buffers. Windows surfaces those failures through the service control
manager and the console, where a diagnostic that is not valid UTF-8 or that
contains raw control characters produces a garbled or unusable message. Running
them on `windows-latest` confirms the bounded-diagnostic contract holds on the
platform where the output is most constrained, rather than inferring it from
the Linux run. They are cheap, deterministic, and platform-independent in
assertion, so naming them explicitly costs almost nothing while adding a
platform lane that a whole-workspace run would not guarantee.

`windows_scm::tests` then covers the native SCM adapter itself.

### 5.2 Coverage gaps — what CI does not catch

| Gap | Detail |
|---|---|
| No `cargo tree` anywhere in CI | The dependency-boundary review is local-only, and the local step is advisory (§4.1) |
| No `cargo check` with `--all-features` on the MSRV lane | `msrv` uses `cargo check --workspace --all-targets --locked` (`ci.yml:30`) — no `--all-features`. Feature-gated code is not compiled at the 1.89 floor, so an MSRV break that only appears under a non-default feature passes |
| No `cargo check` with `--all-features` on the Windows lane | `ci.yml:54` is the same command. The feature-gated footprint binaries are not built on Windows either |
| Clippy and fmt run on Linux only | A `#[cfg(windows)]` or `#[cfg(target_os = "macos")]` clippy violation is not caught by any lane — the Windows and macOS jobs run `test`/`check`, never `clippy` |
| `cargo doc` on Linux only | `deny(missing_docs)` is not re-verified per platform, which is acceptable since rustdoc is platform-independent |
| No full Windows workspace test run | Windows test coverage is a curated subset (§5.1); a Windows failure in an untargeted crate's tests would not be observed |

### 5.3 Coverage gaps — the `--all-features` asymmetry, concretely

The most consequential gap is specific and checkable.

`crates/eggup-transport-footprint/Cargo.toml:18-46` declares:

```toml
[features]
curl = ["dep:eggup-curl"]
eggfetch = ["dep:eggup-eggfetch"]
default = []

[[bin]]
name = "curl_only"
required-features = ["curl"]

[[bin]]
name = "eggfetch_only"
required-features = ["eggfetch"]

[[bin]]
name = "dual"
required-features = ["curl", "eggfetch"]
```

`default = []`, so the three footprint binaries are all feature-gated. The
consequences:

- **`msrv` job**: `cargo check --workspace --all-targets --locked` enables no
  features beyond defaults, so `curl_only`, `eggfetch_only`, and `dual` are not
  built at all. The MSRV floor is never applied to the composition binaries that
  link both adapters together — the code most likely to break on a toolchain
  floor, because it is the only place two independent adapter dependency
  graphs meet.
- **`windows-check` job**: the closing `cargo check --workspace --all-targets
  --locked` has the same gap. Combined with the fact that the Windows lane does
  not run `--all-features` on anything, **no Windows lane ever compiles the
  footprint binaries.** Windows coverage exists for the archives, the seam, the
  curl adapter, the eggpack adapter, and selected service tests — the footprint
  fixtures are Linux/macOS-only in practice, with no job recording that fact.
- Only `stable` and `macos` use `--all-features` and therefore actually compile
  these three binaries.

`cargo tree --workspace --locked` is likewise feature-blind, so the local gate
does not surface the footprint crates' optional adapter edges either.

## 6. Planning governance

Entry point `plans/README.md`. Normative process document:
`plans/003-planning-process.md` (17 sections). Active control surface:
`plans/registry.md`.

### 6.1 Inventory

| Artifact | Count | Location |
|---|---|---|
| Canonical long-term documents | 4 | `plans/000`–`003` |
| Accepted ADRs | 5 | `plans/adrs/ADR-0001`…`0005` |
| Subsystem roadmaps | 7 | `plans/subsystems/*-roadmap.md` |
| Implementation plans | 62 | `plans/implementation/<subsystem>/NNN-*.md` |
| Closure records | 61 | `plans/closure/<subsystem>/NNN-status.md` |
| Closure directories | 8 | 7 subsystems plus `planning-closure-hygiene-corrective` |

The subsystem set is: `verified-update-core`, `acquisition-transport`,
`archive-extraction`, `service-lifecycle`, `distribution-bootstrap`,
`consumer-adoption`, `eggpack-manifest-interoperability`. The eighth closure
directory, `planning-closure-hygiene-corrective`, is a docs-only workstream that
holds plans `001`–`012` and has no subsystem roadmap — it exists to reconcile
the registry and closure records against reality.

### 6.2 Normative documents and their mutability

`plans/000`–`003` are normative. `plans/registry.md:19-24` records `000`, `001`,
and `003` as `normative` and `002` (the long-term roadmap) as `active`. These
are not edited in ordinary work; a change to them is itself a milestone.

ADRs are immutable-by-succession. `plans/adrs/README.md:18` states the rule
directly: "Accepted ADRs are historical. Supersede rather than rewrite." The
lifecycle is `proposed → accepted → deprecated or superseded`
(`plans/adrs/README.md:14`). All 5 ADRs are currently accepted, and
ADR-0001 already records a partial supersession: its producer-distribution
ownership was superseded by ADR-0004 (`plans/registry.md:30`).

| ADR | Accepted decision |
|---|---|
| `ADR-0001-layered-mechanism-and-policy-ownership` | Core/acquisition/service stay layered; original producer-distribution ownership superseded by ADR-0004 |
| `ADR-0002-multi-artifact-transaction-and-rollback` | `ArtifactSet` is the mutation unit; rollback and post-commit policy are explicit |
| `ADR-0003-verification-layers-and-transport-neutrality` | Integrity / authenticity / candidate identity are distinct; core has no mandatory transport |
| `ADR-0004-eggpack-producer-eggup-consumer-boundary` | Eggpack owns producer release contracts and construction; Eggup owns local deployment; applications own release/install policy |
| `ADR-0005-local-archive-extraction-safety-contract` | Local extraction is optional deployment machinery: already-verified archives, explicit member allowlists, finite decompression budgets, regular-file-only no-clobber private extraction, no live mutation |

ADR-0004 is the classification gate: `plans/003-planning-process.md` §3 requires
any cross-repo release/update milestone to be classified against it before
Eggup work is authored. ADR-0005 is the archive safety contract that
[archive-extraction.md](archive-extraction.md) implements.

### 6.3 The 16-section implementation plan

`plans/implementation/README.md` defines a mandatory template. The 16 sections
are: Objective; Why this milestone is ready; Current implementation evidence;
Invariants that must not regress; Scope (In scope / Explicitly out of scope);
Required production changes; Ordered work packages; Failure, cancellation,
restart, and contention semantics; Compatibility and migration; Required tests;
Required verification commands; Documentation updates; Acceptance criteria;
Stop conditions; Closure evidence required; Handoff notes.

Four of these carry the governance weight:

- **Section 5, Explicitly out of scope** — the non-goals. Prevents scope
  creep into a neighbouring subsystem.
- **Section 8** — failure/cancellation/restart/contention semantics. This is
  where the "no invented timeouts" and "no crash-journaling claims" rules
  become checkable.
- **Section 11, Required verification commands** — the exact strings that
  section 5's results are measured against.
- **Section 3 / 14** — SHA baseline and stop conditions. `plans/003-planning-process.md`
  §14 requires exact SHAs rather than vague `main` references after bootstrap.

Naming is uniform: plan `NNN-short-title.md` paired with closure
`NNN-status.md` at the same number.

### 6.4 Closure records

`plans/closure/README.md` fixes the required structure. A closure record must
carry: status (closed / conditionally closed / corrective pass required /
blocked); source plan and roadmap; reviewed repository baseline;
implementation commits or PRs; executive finding; requirement-to-evidence
matrix; production implementation evidence; exact commands run and results;
invariant review; failure/rollback/recovery review; compatibility and migration
review; security review; documentation/operations evidence; unresolved findings
with severity; roadmap disposition; registry updates.

Two rules do the real work:

- "A milestone MUST NOT be marked closed on compilation or happy-path tests
  alone."
- "For platform-sensitive work, report native evidence separately by Linux,
  macOS, and Windows. Do not infer one platform from another."

This is what forces the §5.3 gap to matter: a closure record for a
feature-gated crate must report per-platform results, and the CI matrix does not
currently produce Windows or MSRV evidence for the footprint binaries.

`plans/003-planning-process.md` §15 ("Verification honesty") additionally
requires that results be distinguished as passed / failed / timed out / blocked
by environment / not run / not applicable. A closure that omits a lane must say
"not run", not leave it blank.

### 6.5 The registry as the active control surface

`plans/003-planning-process.md` §13 is explicit: "`plans/registry.md` is the
active control surface," and it should contain only active subsystem roadmaps,
ready/active plans, recently closed work, blocked work and blockers, and next
dependency transitions. "Do not duplicate plan detail into the registry."

The file matches that intent — canonical-doc table, accepted-ADR table with the
ADR-0001→0004 supersession note, a producer/consumer ownership guard, a
recently-closed foundation table, post-closure review findings, an active
roadmap table with next milestones, and a dependency-ready work table where each
row cites its plan, its closure record, and its evidence (hosted run numbers,
SHAs, or `docs-only`). The evidence-per-row format is what makes the registry
reviewable without reading 62 plans.

`distribution-bootstrap` appears in the registry as `archived/transferred` with
"no further Eggup producer work" — the roadmap-level expression of ADR-0004.

## 7. Skills vs. source of truth

`.opencode/skills/` contains exactly two skills, both `SKILL.md` with YAML
frontmatter:

- `planning-workflow` — "Eggup plan-to-closure workflow — normative docs,
  16-section implementation plans, closure evidence, registry updates"
- `verify-workflow` — "Run Eggup verification gate locally and interpret CI
  lanes — command order, focused runs, MSRV and platform evidence"

There is no `.skills/` directory. The skills are summaries: both restate the
command list and the process order that already exist in
`scripts/check-local.sh` and `plans/`. They add no new policy. That ordering is
deliberate — `plans/` plus `architecture/` remain the source of truth, so a
skill that drifts is corrected by rewriting the skill, not by treating it as
authoritative.

The consequence is that a skill *can* drift, and one currently has. The
`verify-workflow` skill describes the `windows-check` job as "`cargo check` on
`windows-latest` (compile-only)". The actual job runs seven steps including
four `cargo test` invocations (§5). `AGENTS.md` repeats the same
understatement, listing CI's Windows addition as "cargo check on Windows". Both
understate the Windows lane; the workflow file is authoritative.

## 8. Release process

**CI never publishes.** No job in `.github/workflows/ci.yml` contains a
`cargo publish`, and `permissions` is `contents: read` only (`ci.yml:7-8`) —
there is no `packages: write` or registry credential anywhere in the file.
Releases are manual, and this is recorded as a deliberate design decision in the
registry: the M009 core/archive publication row states "manual only, no release
CI added."

| Crate | Version | Publish status | Note |
|---|---|---|---|
| `eggup-core` | 0.1.2 | published | Sole dep `sha2`; transaction mechanics |
| `eggup-archive` | 0.1.2 | published | Bounded extraction |
| `eggup-acquisition` | 0.1.2 | published | Zero-dependency seam |
| `eggup-eggfetch` | 0.1.2 | published | Native HTTP adapter |
| `eggup-curl` | 0.1.2 | published | External-curl adapter |
| `eggup-eggpack` | 0.1.2 | published | Only crate touching producer types |
| `eggup-service` | 0.1.2 | **unpublished** | Manager-neutral lifecycle; no `publish = false` key, it is simply not published |
| `eggup-transport-footprint` | 0.1.2 | `publish = false` | Binary-only fixture crate, non-publishable by design |

All 8 crates share the workspace version `0.1.2`; only the publish status
differs. Publication is a milestone, not a side effect: the registry records
dedicated publication milestones (M009 for `eggup-curl`, M009 for the
core/archive pair, M010 for `eggup-acquisition 0.1.3` marked `ready`), and
`plans/003-planning-process.md` §16 requires package dry-run as part of
boundary qualification. The registry also records a downstream consequence
worth noting: `eggup-eggpack` pins `=0.1.2`, so it needs a separate
republication to inherit any `eggup-acquisition` fix — an exact-pin
requirement turning a single-crate fix into a multi-crate release event.

`CHANGELOG.md` discipline follows the same separation: every published crate
keeps an `Unreleased` section (7 of 8 crates have a CHANGELOG; the fixture
crate is the exception) carrying an explicit no-publication / no-migration
disclaimer while `0.1.2` remains the published baseline.

## 9. Reviewer's checklist

Ordered by how expensive a miss is.

1. **Is the change in scope?** Does it need a `plans/implementation/<subsystem>/NNN-*.md`
   plan first, or is it within an already-planned milestone?
2. **Does the plan have all 16 sections**, plus a SHA baseline (§3), explicit
   non-goals (§5), failure/restart/contention semantics (§8), and exact
   verification commands (§11)?
3. **Manifest review** — if `Cargo.toml` or any `crates/*/Cargo.toml` changed:
   all five shared keys inherited, `[lints] workspace = true` present, and the
   `cargo tree --workspace --locked` output shows no new cross-boundary edge?
   Specifically: still no transport/service/Eggpack dep in `eggup-core`, still
   zero deps in `eggup-acquisition`, still no cross-reference between
   `eggup-eggfetch` and `eggup-curl`?
4. **ADR-0004 classification** — is this work that belongs to Eggup, or producer
   work that belongs to Eggpack? `distribution-bootstrap` stays archived.
5. **Run `./scripts/check-local.sh`** and treat all 5 lines as gates, including
   reading the `cargo tree` output rather than skimming past it (§4.1).
6. **Feature-gated change?** If the change lives behind a feature flag, check
   which CI lane actually compiles it. Non-default features are built only by
   `stable` and `macos` (§5.3) — not by `msrv`, not on Windows.
7. **Platform-sensitive change?** Do not infer Windows or macOS behavior from a
   local Linux run. Confirm the relevant lane ran, and if the change is outside
   the curated Windows subset, treat Windows evidence as `not run` and say so.
8. **Lint layering intact?** `unsafe_code = deny` still in the workspace table,
   `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]` still on the library
   crate root, `-D warnings` still on the clippy command.
9. **Public API change?** Rustdoc present (`deny(missing_docs)` enforces
   presence, not quality); crate `README.md` and `CHANGELOG.md` `Unreleased`
   section updated with the no-publication/no-migration disclaimer.
10. **Closure record** — written to `plans/closure/<subsystem>/NNN-status.md`
    with a requirement→evidence matrix, exact commands and results, all reviews,
    severitized residuals, and separate Linux/macOS/Windows reporting. A
    missing lane is recorded as `not run`, never omitted.
11. **Registry updated** — status transitions, blockers, and next dependency
    transitions only; no plan detail duplicated. Roadmap status table reconciled.
12. **If publishing** — confirm it is a tracked publication milestone, that
    `cargo package`/dry-run was qualified, that downstream exact pins
    (`eggup-eggpack` → `=0.1.2`) are accounted for, and that CI still does not
    publish.

## 10. Cross-references

- [overview.md](overview.md) — birds-eye view, module map, cross-cutting
  invariants, and the deep-dive index.
- [core-transaction.md](core-transaction.md) — the crate whose sole `sha2`
  dependency makes the boundary in §2.4 checkable.
- [acquisition.md](acquisition.md) — the zero-dependency seam both adapters and
  the footprint fixtures depend on.
- [eggfetch-adapter.md](eggfetch-adapter.md), [curl-adapter.md](curl-adapter.md)
  — the two adapters that must remain mutually unaware.
- [archive-extraction.md](archive-extraction.md) — the crate whose Windows
  filesystem behavior motivates the targeted `windows-check` selection.
- [service-lifecycle.md](service-lifecycle.md) — the crate whose three
   UTF-8-boundary diagnostic tests are named individually on Windows.
- [transport-footprint.md](transport-footprint.md) — the feature-gated binaries
  that `msrv` and `windows-check` do not build (§5.3).
- [eggpack-adapter.md](eggpack-adapter.md) — the only crate allowed to depend
  on `eggpack-manifest` and to pin `=0.1.2` exactly.
- [ADR-0005](../plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md) —
  the archive safety contract; the ADR to read before touching
  `eggup-archive` boundaries.
- [ADR-0004](../plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md) —
  the classification gate for cross-repo work.
- [registry.md](../plans/registry.md) — the active control surface.
- [003-planning-process.md](../plans/003-planning-process.md) — the normative
  process document.
- [implementation/README.md](../plans/implementation/README.md) — the 16-section
  plan template.
- [closure/README.md](../plans/closure/README.md) — the required closure
  structure and the per-platform reporting rule.

## Known doc/code drift

Recorded rather than corrected, since the scripts and workflows are the
behavior and the prose is owned elsewhere.

| Location | Drift |
|---|---|
| `verify-workflow` skill, CI lanes table | Describes `windows-check` as `cargo check` only, "compile-only". The job runs four `cargo test` steps covering acquisition, archive, curl, eggpack, and selected `eggup-service` tests |
| `AGENTS.md`, CI section | Lists CI's Windows addition as "cargo check on Windows", likewise understating the targeted test run |
| `AGENTS.md`, planning governance | States "keep crate `README.md` + `CHANGELOG.md` current" as a general rule; `eggup-transport-footprint` has neither, and legitimately cannot carry `missing_docs` since it has no library root |
| `verify-workflow` skill | Says CI's `windows-check` "adds `cargo check` on `windows-latest`". It does not mention that the job, like `msrv`, omits `--all-features`, which is the concrete gap in §5.3 |
| `crates/eggup-transport-footprint/Cargo.toml:16,24,29` | Pins `"0.1.0"` for `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch` while `[workspace.package] version` is `0.1.2`. Resolves correctly via the caret requirement and the path dependencies, but the literals are stale |
