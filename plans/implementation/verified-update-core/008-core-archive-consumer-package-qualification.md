# Verified Update Core Milestone 008 — Core/Archive Consumer Package Qualification

Status: implemented; closed by `plans/closure/verified-update-core/008-status.md`

Repository baseline: `413a35a7da32ea22ef337caefc35d3619154794e`

Source roadmaps:

- `plans/subsystems/verified-update-core-roadmap.md`
- `plans/subsystems/archive-extraction-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#23-public-api-stability`
- `plans/000-long-term-specification.md#25-release-philosophy`

Primary class: infrastructure / polish

## 1. Objective

Qualify a crates.io-usable versioned package boundary containing the post-M001d `eggup-core` bound-source API and `eggup-archive` for downstream Egress adoption.

This milestone prepares packages and records a deterministic publish order. It does not publish automatically.

## 2. Why this milestone is ready

M001d is closed at implementation `18d83de` with hosted run `36335233644` green on Stable, Rust 1.89, macOS, and Windows. Current head is also green via run `36335617284`.

The currently published 0.1.1 set predates M001d and does not include `eggup-archive`. A versioned package boundary is therefore the remaining distribution prerequisite for a publishable Egress dependency.

## 3. Current implementation evidence

- `eggup-core` exposes additive `BoundSources` and `InstallPlan::prepare_with_bound_sources`.
- `eggup-archive` exposes `BoundMember`, `BoundExtraction`, `DeferredCleanup`, and `PersistedExtraction::into_bound_sources`.
- both crates remain Rust 1.89 and first-party unsafe-free;
- `eggup-core` runtime dependency remains `sha2` only;
- `eggup-archive` has no runtime dependency on `eggup-core` (core is currently a dev-dependency);
- changelogs mark M001b/M001c/M001d as unpublished.

## 4. Invariants that must not regress

- no automatic crates.io publication;
- no producer/installer generation enters Eggup;
- no network/service dependency enters core;
- archive extraction remains optional;
- M001d object-bound authority is unchanged;
- package contents/docs match actual security semantics;
- Rust 1.89 remains the package MSRV;
- package versioning must not make already-published 0.1.1 consumers silently incompatible.

## 5. Scope

### In scope

- choose and record the next compatible package version (0.1.2 is the expected patch candidate unless execution evidence requires another choice);
- update package/version metadata and exact internal version pins required for a coherent workspace;
- qualify `eggup-core` and `eggup-archive` with `cargo package` and `cargo publish --dry-run`;
- inspect generated crate contents;
- verify docs/changelogs expose the M001d authority model;
- record a manual publish order with core before archive;
- verify the package graph from a clean temporary consumer.

### Explicitly out of scope

- publishing without a separate maintainer action;
- publishing unrelated Eggup crates merely to keep a cosmetic lockstep;
- Egress code changes;
- Eggpack producer changes;
- API redesign.

## 6. Required production/package changes

Use the smallest versioning change that lets a clean external consumer depend on the qualified core/archive APIs from crates.io.

If workspace-wide version inheritance is retained, update exact local dependency requirements consistently so the workspace still resolves. Do not silently broaden exact requirements that were deliberately exact.

Package metadata for `eggup-archive` must include externally useful homepage/documentation/keywords/categories/excludes if currently missing and must not imply authenticity/signature support.

The package verification consumer must compile a minimal flow using:

`PersistedExtraction::into_bound_sources -> BoundExtraction -> BoundSources -> InstallPlan::prepare_with_bound_sources`.

## 7. Ordered work packages

A. Decide/record version and update internal exact pins.

B. Audit public docs/changelogs for core/archive M001d APIs.

C. Run package/dry-run qualification for core then archive.

D. Inspect crate contents and dependency trees.

E. Build a clean external fixture consumer against the packaged/versioned crates.

F. Record manual publish order and downstream Egress gate.

## 8. Failure, restart, cancellation, and contention semantics

No runtime transaction semantics change.

A failed package/dry-run leaves repository source untouched except intentional version/docs changes. Publication is never retried automatically.

## 9. Compatibility and migration

The M001d API is additive. Existing path-source consumers remain source-compatible.

If version/pin changes imply a breaking dependency constraint for an existing published consumer, stop and write a versioning corrective instead of forcing a patch release.

## 10. Required tests

- full workspace tests;
- core bound-source tests;
- archive bound-source tests;
- Rust 1.89 all-target check;
- package and publish dry-run for core/archive;
- clean external consumer compile/test from packaged/versioned crates;
- Linux/macOS/Windows hosted CI.

## 11. Required verification commands

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-core --locked
cargo publish -p eggup-core --dry-run --locked
cargo package -p eggup-archive --locked
cargo publish -p eggup-archive --dry-run --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-archive --locked
cargo doc --workspace --no-deps --locked
git diff --check
~~~

## 12. Documentation updates

- root and archive changelogs;
- core/archive README/rustdoc if package audit finds drift;
- verified-core and archive roadmaps;
- registry;
- closure record `plans/closure/verified-update-core/008-status.md`.

## 13. Acceptance criteria

- a versioned crates.io-compatible core/archive package pair is publication-ready;
- package/dry-run succeeds;
- clean external consumer proves the M001d flow;
- no git/path dependency is required by a downstream published Egress crate;
- publish order is explicit;
- no publication occurs automatically.

## 14. Stop conditions

Stop if a patch version would be semantically incompatible, package verification requires publishing unrelated crates, a clean consumer cannot express the M001d flow, or package changes expose a medium-or-higher authority defect.

## 15. Closure evidence required

Record exact version decision, manifest diffs, package file lists, dry-run output, dependency trees, clean-consumer fixture result, hosted matrix, unresolved findings, and the explicit manual publication gate.

## 16. Handoff notes

This is the package prerequisite for Egress adoption. Runtime consumer work may be prototyped against an immutable Eggup revision, but no merged/publishable Egress dependency may rely on git/path sources.
