# Eggpack Manifest Interoperability Milestone 002 — Archive Extraction Handoff

Status: implemented; closed by `plans/closure/eggpack-manifest-interoperability/002-status.md`

Repository baseline: `413a35a7da32ea22ef337caefc35d3619154794e`

Source roadmap:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Hard dependencies:

- Eggpack interoperability M001/M001a closed;
- Archive M001d closed via run `36335233644`.

Primary class: capability / interface

## 1. Objective

Connect `ManifestProjection::Archive` to the qualified Eggup archive extraction + object-bound core staging path without adding archive behavior to `eggup-core` or producer authority to Eggup.

## 2. Why this milestone is ready

The adapter already preserves exact archive artifact size/digest and exact member source/install/size/digest facts. M001d now provides the missing safe local extraction-to-staging boundary.

No Eggpack producer schema change is required.

## 3. Current implementation evidence

`eggup-eggpack` currently:

- projects `ArtifactForm::Archive` to `ManifestProjection::Archive`;
- binds exactly one archive acquisition request with manifest size/digest;
- intentionally returns `ArchiveExtractionRequired` for direct ArtifactSet materialization;
- preserves member source/install/size/digest evidence.

`eggup-archive` now supports bounded allowlisted tar.gz/zip extraction and bound-source handoff.

## 4. Invariants that must not regress

- consumer selects/authorizes release and target;
- manifest does not grant authenticity by itself;
- archive bytes are integrity-verified before extraction;
- archive member allowlist comes exactly from manifest projection;
- extracted bytes are verified against manifest member size/digest;
- staged bytes come from bound objects, never advisory paths;
- core remains archive/Eggpack independent;
- producer build/CI/publication machinery remains outside Eggup.

## 5. Scope

### In scope

- an adapter/helper translating `ManifestProjection::Archive` into an `eggup_archive::ArchivePlan`;
- explicit format input from consumer-selected artifact metadata/name, with fail-closed supported-format mapping;
- member destination/permissions mapping for core plan construction;
- exact bound-source staging composition;
- deterministic direct/archive fixture tests.

### Out of scope

- fetching manifests/releases;
- choosing latest versions;
- adding new Eggpack wire fields;
- arbitrary archive formats;
- Egress-specific names/policy;
- publishing packages automatically.

## 6. Required production changes

Add a narrow archive handoff API in `eggup-eggpack` or a leaf helper module that:

1. accepts an archive projection plus caller-owned acquired archive path and finite extraction limits;
2. validates the acquired archive path/size/integrity continuity;
3. builds an `ArchivePlan` whose declared members exactly match projection source/install/size/digest;
4. performs no extraction itself if keeping I/O orchestration caller-owned is cleaner, but returns all typed inputs needed to call `eggup_archive::extract`;
5. builds the core `ArtifactSet/InstallPlan` while advisory paths are still valid;
6. transfers open member objects into `BoundSources`;
7. stages with `prepare_with_bound_sources`;
8. returns/threads deferred cleanup responsibility explicitly.

Prefer typed composition over a monolithic convenience method if that keeps authority/lifetimes obvious.

## 7. Ordered work packages

A. Define archive projection-to-plan types/API.

B. Map exact manifest member evidence into archive declarations and core identities.

C. Compose extraction, plan construction, bound handoff, and cleanup in tests/examples.

D. Add malformed/mismatch/unsupported-format negatives.

E. Run full adapter/archive/core regression matrix.

## 8. Failure, restart, cancellation, and contention semantics

Any mismatch fails before live mutation. Extraction/staging failure never falls back to advisory paths. Cleanup residue remains explicit under M001b/M001d semantics.

No network retry/release fallback is introduced.

## 9. Compatibility and migration

Existing direct/bundle adapter APIs remain unchanged. `ArchiveExtractionRequired` may remain for callers using `materialize_artifact_set`; the new archive-specific path is additive.

## 10. Required tests

- exact tar.gz and zip archive fixture projection;
- missing/extra/crossed member failure;
- wrong member size/digest failure;
- wrong archive size/digest failure;
- unsupported format failure;
- advisory member-entry replacement after core plan construction still stages original bound bytes;
- cleanup-after-consume;
- direct/bundle regression unchanged;
- Rust 1.89 and hosted Windows archive runtime tests.

## 11. Required verification commands

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test -p eggup-archive --all-targets --all-features --locked
cargo test -p eggup-core --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked
git diff --check
~~~

## 12. Documentation updates

- `eggup-eggpack` README/rustdoc;
- architecture archive/interoperability docs;
- Eggpack interoperability roadmap and registry;
- closure `plans/closure/eggpack-manifest-interoperability/002-status.md`.

## 13. Acceptance criteria

A Manifest v1 archive projection can flow through verified acquisition evidence, bounded extraction, bound-source core staging, and deferred cleanup with no pathname re-resolution after handoff and no producer dependency in core.

## 14. Stop conditions

Stop if the current manifest lacks evidence required by ADR-0005/M001d, if a producer schema change becomes necessary, or if the clean API requires archive dependencies in `eggup-core`.

## 15. Closure evidence required

Record public API diff, fixture matrix, dependency tree, direct/bundle non-regression, bound-source race evidence, hosted platform matrix, and confirmation that Eggpack producer code/schema was unchanged.

## 16. Handoff notes

Construct the core plan while advisory paths are still valid, then treat them as diagnostics only and stage exclusively from the bound objects. A pre-plan namespace attack may fail closed; it must never substitute staged bytes.
