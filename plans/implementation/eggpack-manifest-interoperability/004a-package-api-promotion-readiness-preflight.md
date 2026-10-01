# Eggpack Manifest Interoperability Milestone 004a — Package/API Promotion Readiness Preflight

Status: implemented; closed by `plans/closure/eggpack-manifest-interoperability/004a-status.md` (readiness preflight complete 2026-10-01; M004 remains blocked on Eggpack-owned `eggpack-manifest 0.1.0` + Eggup-owned `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2`; no publication occurred)

Repository baseline: `538e3e5605cf3c315c10e5be200c8896de7379b1`

Source roadmap:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Predecessor closure:

- M003a: `plans/closure/eggpack-manifest-interoperability/003a-status.md`
- M003: `plans/closure/eggpack-manifest-interoperability/003-status.md`

Long-term references:

- `plans/002-long-term-roadmap.md#phase-9--eggpack-authority-cutover-and-manifest-interoperability`
- `plans/003-planning-process.md`

Applicable ADRs:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

External producer baseline reviewed during authoring:

- `eggstack/eggpack@0413e806945cfdd31e16bd266b0318efb3afa498`
- `eggpack-manifest` source package version `0.1.0`, with package metadata present and no `publish = false` flag;
- Eggpack's own interoperability roadmap is stale relative to Eggup M003 closure and does not currently register a manifest-publication milestone.

Primary class: infrastructure / package qualification / cross-repository readiness

## 1. Objective

Determine the exact, minimum, registry-resolvable package/API promotion path for `eggup-eggpack` after successful real-consumer M003 adoption, without publishing anything and without assuming lockstep publication of unrelated Eggup crates.

M003 intentionally qualified the adapter and Eggsact using immutable Git revisions because `eggup-eggpack` is unpublished and its lightweight Eggpack schema dependency is Git-pinned. M004 is the promotion decision that may move this seam to crates.io.

Before M004 can be safely authored, M004a must answer mechanically:

1. Is `eggpack-manifest 0.1.0` packageable and/or already registry-resolvable at the exact required version?
2. Which Eggup dependencies of `eggup-eggpack 0.1.2` are actually available on the registry?
3. What is the minimum additional Eggup publication set required for `cargo package` / registry-only resolution of `eggup-eggpack`?
4. Does a registry-only Eggsact consumer resolve one coherent Eggup source graph without Git/path dependencies?
5. Is a new `eggup-eggfetch 0.1.2` publication required for downstream coherence, or can the existing published adapter version remain compatible?
6. Are any public API/version adjustments required before promotion?
7. Which producer-side publication action belongs in Eggpack rather than Eggup?

M004a produces evidence and a precise M004 dependency graph. It does not publish crates.

## 2. Readiness

Hard dependencies:

- M003a closed: satisfied.
- M003 real-consumer adoption closed: satisfied.
- `eggup-core 0.1.2` / `eggup-archive 0.1.2` publication evidence: satisfied by M009.

Interface dependencies:

- Eggpack ReleaseManifest v1 schema and `eggpack-manifest 0.1.0` Git interface are stable for the qualified consumer path.
- Current M003 consumer proof uses Eggup revision `e336b32` and Eggpack manifest revision `678bbf04`.

Operational dependencies are the subject of this preflight and therefore are not blockers to M004a itself.

## 3. Current package evidence

At the Eggup baseline:

- workspace version is `0.1.2`;
- `eggup-eggpack` is `publish = false`;
- `eggup-eggpack` direct dependencies are:
  - `eggup-core = =0.1.2` (path);
  - `eggup-archive = =0.1.2` (path);
  - `eggup-acquisition = =0.1.2` (path);
  - `eggpack-manifest = =0.1.0` from immutable Git rev `678bbf04...`;
  - `sha2 = 0.10.9`.
- `eggup-core 0.1.2` and `eggup-archive 0.1.2` were published in M009.
- the active registry states no other 0.1.2 workspace crate was published.
- `eggup-eggfetch` source is workspace 0.1.2 and depends on `eggup-acquisition` with a semver `0.1.0` requirement plus a local path; this may or may not require a new registry publication for a downstream registry-only consumer and must be tested rather than assumed.
- `eggup-acquisition` has no runtime dependencies and is independently packageable in principle, but its current 0.1.2 registry availability must be verified.

At the Eggpack baseline:

- `eggpack-manifest` is workspace version `0.1.0`;
- its Cargo metadata includes repository/homepage/docs/readme/authors/keywords/categories;
- it has no `publish = false`;
- it depends only on `serde` + `serde_json`;
- the current Eggpack planning surface does not yet establish an explicit crates.io publication milestone for it.

At the M003 consumer:

- Eggsact pins `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, and `eggup-eggpack` to one immutable Eggup Git revision;
- `eggpack-manifest` appears only transitively under `eggup-eggpack`;
- this graph is qualified but deliberately not suitable for a public Eggsact release.

## 4. Invariants

### Ownership

- Eggpack owns whether/when `eggpack-manifest` is published.
- Eggup owns whether/when `eggup-eggpack` and Eggup support crates are published.
- Eggsact owns whether/when it migrates from Git qualification pins to registry dependencies.
- M004a MUST NOT publish, yank, tag, or release anything.

### Package coherence

- A promoted `eggup-eggpack` must be buildable by a clean external registry consumer with no workspace/path/Git requirement.
- The registry graph must contain one compatible source identity per Eggup package.
- Do not publish unrelated crates solely for lockstep aesthetics.
- Do not leave a published adapter depending on unavailable exact versions.
- Do not change public API merely to make `cargo package` pass unless a genuine package/API defect is identified and separately planned.

### Architecture

- `eggup-core`, `eggup-acquisition`, `eggup-archive`, and `eggup-service` remain Eggpack-independent.
- `eggup-eggpack` depends only on the lightweight schema crate on the Eggpack side.
- Publication does not convert integrity evidence into authenticity.
- No producer build/CI/bootstrap crate enters the adapter graph.

## 5. Scope

### In scope

- query/verify actual registry availability for exact dependency versions;
- `cargo package --list` and `cargo package --no-verify` dry-runs where meaningful;
- temporary/scratch manifest rewrites to model the prospective registry graph, without committing them;
- create a minimal external scratch consumer for registry-only resolution;
- determine whether `eggup-acquisition 0.1.2` must publish;
- determine whether `eggup-eggfetch 0.1.2` must publish for Eggsact's eventual registry-only migration;
- confirm whether `eggpack-manifest 0.1.0` requires an Eggpack publication handoff;
- identify exact package ordering if M004 later becomes ready;
- assess adapter public API/docs metadata for publication suitability;
- measure packaged contents and dependency footprint;
- update roadmap/registry with the evidence-driven M004 gate;
- write closure record.

### Out of scope

- changing `publish = false`;
- publishing any package;
- modifying Eggpack;
- migrating Eggsact dependencies;
- version bumps;
- tags/releases;
- broad API redesign;
- automatic release CI;
- authenticity/signing;
- publication of service/curl/transport-footprint crates unless mechanically required and separately justified.

## 6. Required preflight work

### 6.1 Registry availability matrix

Record exact registry state for:

- `eggpack-manifest 0.1.0`;
- `eggup-core 0.1.2`;
- `eggup-archive 0.1.2`;
- `eggup-acquisition 0.1.2`;
- current published `eggup-eggfetch` versions;
- any other package that enters the candidate graph.

Distinguish:

- source packageable;
- package present on crates.io;
- exact version resolvable;
- package yanked/unavailable;
- local-only.

Do not infer registry presence from Cargo metadata.

### 6.2 Adapter package graph simulation

In a scratch tree only:

1. remove/override `publish = false`;
2. replace the Git `eggpack-manifest` dependency with exact registry `=0.1.0` only if that version is actually resolvable;
3. preserve exact Eggup dependency versions initially;
4. run package metadata/list/dry-run;
5. record every unresolved package/version and every Cargo packaging warning/error;
6. do not commit the scratch edits.

If `eggpack-manifest` is not on the registry, stop that branch cleanly and record the producer-side prerequisite; do not vendor or duplicate it.

### 6.3 Minimum Eggup publication set

Using the graph simulation, determine the minimum Eggup publication set required before `eggup-eggpack` can publish.

Expected candidates are:

- `eggup-acquisition 0.1.2` if absent;
- `eggup-eggpack 0.1.2` itself after prerequisites;
- possibly no others because core/archive 0.1.2 are already published.

Do not assume this expected set is correct; prove it.

### 6.4 Registry-only consumer simulation

Create a temporary external Cargo project outside the workspace that models the M003 consumer dependency surface.

Test two levels:

A. Adapter-only:
- registry `eggup-eggpack` candidate plus its transitive dependencies;
- no path/Git source.

B. Eggsact-shaped:
- registry `eggup-core`;
- registry `eggup-acquisition`;
- registry `eggup-eggfetch`;
- registry `eggup-eggpack`;
- no Git/path Eggup dependencies;
- one source identity per package.

If the current published `eggup-eggfetch` can resolve coherently against the required acquisition/core versions and preserves the M003-used API, a new `eggup-eggfetch 0.1.2` publication is not automatically required. If it cannot, record the exact reason and add it to the M004 publication set.

No Eggsact source modification is required for this simulation.

### 6.5 API/package review

Review `eggup-eggpack` for publication-facing quality:

- package metadata: homepage/documentation/repository/readme/authors/keywords/categories as appropriate;
- public rustdoc completeness;
- no accidental test fixtures/secrets/large files in package contents;
- no workspace-only assumptions;
- no Git/path dependencies in the candidate manifest;
- MSRV 1.89;
- license/readme files included;
- dependency surface limited to intended crates.

A missing metadata field is polish, not automatically a blocker unless crates.io/package validation requires it.

### 6.6 Eggpack producer handoff

If `eggpack-manifest 0.1.0` is not already registry-resolvable:

- record a hard external prerequisite owned by Eggpack;
- specify the exact package/version required;
- link the current Eggpack source/revision and the fact that M003 proved the Git-pinned schema;
- recommend a small Eggpack manifest-publication plan in the Eggpack repository;
- do not author or execute that producer publication from Eggup unless separately instructed.

The stale Eggpack interoperability roadmap should also be called out for status reconciliation because it still reports M003 as ready to resume.

## 7. Ordered work packages

1. Capture exact Eggup/Eggpack heads and package manifests.
2. Verify crates.io/registry availability for every candidate exact version.
3. Run package-list/dry-run on currently publishable Eggup dependencies.
4. Perform scratch adapter registry-dependency rewrite and package simulation.
5. Derive the minimum Eggup publication set.
6. Build adapter-only external registry consumer simulation.
7. Build Eggsact-shaped external registry consumer simulation.
8. Determine whether current published `eggup-eggfetch` is sufficient or a 0.1.2 publication is required.
9. Review adapter package metadata/rustdoc/contents/MSRV.
10. Record producer-side `eggpack-manifest` prerequisite precisely.
11. Update roadmap/registry with evidence-based M004 readiness state.
12. Write `plans/closure/eggpack-manifest-interoperability/004a-status.md`.
13. Only if every hard registry prerequisite is resolved may a separate M004 publication implementation plan be authored.

## 8. Failure / restart semantics

M004a mutates no published state.

Scratch manifests/projects must be isolated and removed after evidence collection.

A package dry-run failure is evidence, not something to bypass with `--allow-dirty` or dependency vendoring unless the plan explicitly calls for that exact diagnostic command.

If registry state changes during execution, record the timestamp/version state and rerun the affected resolution test.

If another process publishes one of the required versions, refresh the matrix before closure.

## 9. Compatibility and migration

No consumer migration occurs in M004a.

The Git-pinned M003 Eggsact qualification remains valid evidence and must not be rewritten as a registry release.

The eventual M004 plan should preserve the M003 runtime/API behavior byte-for-byte unless this preflight discovers a separately justified package/API corrective.

The eventual downstream Eggsact registry migration should be separately authorized after M004 publication; do not combine consumer migration with package publication unless a later plan explicitly scopes both.

## 10. Required verification

Run applicable commands and record exact results:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo doc -p eggup-eggpack --no-deps --locked
cargo +1.89.0 check -p eggup-eggpack --all-targets --locked
cargo package -p eggup-core --locked --no-verify
cargo package -p eggup-archive --locked --no-verify
cargo package -p eggup-acquisition --locked --no-verify
cargo package -p eggup-eggfetch --locked --no-verify
cargo tree -p eggup-eggpack --locked
git diff --check
```

For `eggup-eggpack`, run `cargo package --list` and package dry-run only in the scratch candidate manifest because the committed package remains intentionally `publish = false` and Git-pinned until M004.

Also record:

- exact registry queries/results;
- scratch adapter package result;
- adapter-only external consumer resolution;
- Eggsact-shaped external consumer resolution;
- resulting one-source-per-package tree.

No `cargo publish` command is authorized.

## 11. Documentation updates

At closure update:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`;
- `plans/registry.md`;
- `plans/closure/eggpack-manifest-interoperability/004a-status.md`.

If an Eggpack prerequisite is confirmed, record the exact cross-repo handoff needed but do not edit Eggpack from this milestone.

## 12. Acceptance criteria

M004a closes only when:

- every exact package/version in the proposed registry graph has a documented registry state;
- the minimum Eggup publication set is proven, not assumed;
- whether `eggup-eggfetch 0.1.2` is required is answered by an external consumer resolution test;
- the adapter's scratch registry-only package graph either resolves cleanly or has a precise external blocker;
- a registry-only Eggsact-shaped graph is demonstrated as far as currently published packages permit;
- no hidden Git/path dependency remains in the proposed final graph;
- public API/package metadata review is complete;
- Eggpack-owned publication work is clearly separated from Eggup-owned work;
- no package was published;
- roadmap/registry state tells the truth about whether M004 proper is ready.

## 13. Stop conditions

Stop and write a separate corrective if:

- publication requires a breaking adapter/core API change;
- a lower Eggup crate would need an Eggpack dependency;
- the adapter requires producer build/CI crates rather than `eggpack-manifest` alone;
- registry resolution requires weakening exact M003 integrity/destination/fallback semantics;
- the existing 0.1.2 core/archive publication is incompatible with the candidate adapter;
- semver/source duplication cannot be resolved with ordinary dependency versioning.

If the only blocker is an unpublished `eggpack-manifest 0.1.0`, close M004a with M004 blocked on that explicit Eggpack-owned publication handoff; do not treat the preflight as failed.

## 14. Closure evidence required

Record:

- exact Eggup/Eggpack baselines;
- registry availability matrix;
- package-list/dry-run results;
- scratch candidate manifest diff;
- minimum Eggup publication set;
- adapter-only external consumer Cargo tree;
- Eggsact-shaped external consumer Cargo tree;
- `eggup-eggfetch` publication decision and evidence;
- API/package metadata findings;
- MSRV/docs/tests;
- cross-repo Eggpack prerequisite;
- unresolved findings by severity;
- explicit M004 readiness disposition.

## 15. Handoff notes

Do not author M004 as a publication plan until this preflight closes.

The expected likely path is:

`Eggpack publishes/resolves eggpack-manifest 0.1.0` → `Eggup publishes any missing exact support crate(s)` → `Eggup publishes eggup-eggpack` → `separate Eggsact registry-dependency migration/qualification`.

M004a exists specifically to verify that expected path and minimize the publication set before any irreversible registry action.
