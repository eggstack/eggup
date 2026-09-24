# Eggpack Manifest Interoperability Milestone 001 — ReleaseManifest v1 Direct/Bundle Adapter

Status: ready for handoff

Repository baseline: dfad557f7405beac569c4e3368f67e5a2e501002

Source roadmap:

- plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Long-term requirements:

- plans/002-long-term-roadmap.md#phase-9--eggpack-authority-cutover-and-manifest-interoperability
- plans/001-terminology-and-domain-model.md#7-release-plan
- plans/001-terminology-and-domain-model.md#8-install-plan
- plans/001-terminology-and-domain-model.md#11-integrity-evidence

Applicable ADRs:

- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md

External interface baseline:

- Eggpack corrected planning/closure baseline: eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41
- Eggpack interoperability M001a implementation: 8d9264b3c224f3f05a061f4038b0f328b1c5c95e
- Eggpack M001a closure: plans/closure/eggup-interoperability/001a-status.md in eggstack/eggpack
- Eggpack hosted CI: 35998756491
- corrected projection-bundle.json SHA-256: f0b227442e2a2a28dc4e2100301233f2698703d15251b6f2459e6387df6c53ce

Primary class: infrastructure / capability / cross-repository compatibility

## 1. Objective

Add an optional eggup-eggpack adapter crate that consumes Eggpack ReleaseManifest v1 and translates one exact canonical target into explicit Eggup acquisition and deployment inputs without moving producer or application policy into Eggup.

M001 must support direct and bundle layouts through the complete translation boundary:

ReleaseManifest -> exact target projection -> caller-bound exact AcquisitionRequest values with manifest-tightened byte limits -> explicitly acquired local files -> exact-size validation -> Eggup ArtifactSet carrying manifest SHA-256 integrity requirements and caller-supplied permissions intent.

Archive manifests must be parsed and projected, including archive artifact and member evidence, but must remain explicitly extraction-required. M001 must not invent a generic archive extractor or silently construct an ArtifactSet from archive bytes.

The adapter must remain optional. Existing Eggup users and all lower Eggup crates must remain fully usable without Eggpack.

## 2. Why this milestone is ready

The prior cross-repository gate is closed.

Eggpack Interop M001a corrected the CodeGG bundle projection and added relationship-aware direct/bundle/archive fixture validation. Its closure states that no unresolved medium-or-higher interoperability evidence defect remains and explicitly unblocks Eggup adapter plan authoring.

Eggup already exposes the required consumer-side primitives:

- eggup-acquisition::AcquisitionRequest for exact caller-selected HTTP(S) URLs;
- eggup-acquisition::FetchLimits for caller byte/time bounds;
- eggup-core::ProductId and ReleaseId as opaque validated identities;
- eggup-core::MemberId;
- eggup-core::IntegrityRequirement::Sha256([u8; 32]);
- eggup-core::ArtifactMember and ArtifactSet;
- eggup-core::InstallPlan with caller-owned installation root;
- existing ownership, validation, staging, commit, rollback, and receipt machinery downstream.

No producer-side implementation remains in Eggup. ADR-0004 assigns optional manifest-to-deployment translation to Eggup, so this work is on the correct side of the ownership boundary.

## 3. Current implementation evidence

At the repository baseline:

- workspace crates are eggup-core, eggup-acquisition, eggup-eggfetch, and eggup-service;
- eggup-core has no Eggpack dependency;
- eggup-acquisition has no Eggpack dependency;
- AcquisitionRequest stores one exact caller-provided URL and does no discovery/fallback;
- FetchLimits validates explicit byte/time bounds;
- ArtifactMember attaches a local source path, normalized relative destination, permissions intent, and integrity requirement;
- ArtifactSet is non-empty with unique member identities;
- InstallPlan owns the installation root and rejects invalid/local-source aliasing;
- no eggup-eggpack crate exists;
- no Eggup code parses ReleaseManifest v1.

Eggpack's corrected consumer fixture baseline includes direct Eggsact, three-member CodeGG bundle, and Egress archive examples. The corrected CodeGG bundle contains all three artifact/install relationships and is mechanically regression-tested upstream.

## 4. Invariants that must not regress

### Ownership and dependency invariants

- eggup-core MUST NOT depend on Eggpack.
- eggup-acquisition MUST NOT depend on Eggpack.
- eggup-service MUST NOT depend on Eggpack.
- only eggup-eggpack may depend on eggpack-manifest.
- eggup-eggpack MUST NOT depend on eggpack-core, eggpack-contract, eggpack-bootstrap, generated-CI code, or producer build types.
- non-Eggpack consumers remain first-class Eggup users.

### Release-policy invariants

- the adapter never discovers latest releases;
- the adapter never orders versions;
- the adapter never constructs GitHub URLs, base URLs, mirrors, or fallback sources;
- callers provide exact AcquisitionRequest values after their own release/origin policy;
- target selection is an exact canonical triple, never an alias or nearest-target guess;
- source_revision remains producer evidence and is not interpreted as update policy.

### Deployment-policy invariants

- manifest install names become relative member identities/destinations only;
- the manifest does not supply or authorize the installation root;
- the manifest does not prove ownership of an existing destination;
- permissions/executable intent are caller supplied because Manifest v1 does not encode them;
- product-specific candidate validation remains caller owned;
- service lifecycle remains outside this adapter.

### Evidence invariants

- exact manifest size tightens acquisition limits and is rechecked against the acquired local regular file;
- SHA-256 bytes are propagated exactly into IntegrityRequirement::Sha256;
- SHA-256 remains integrity evidence, not authenticity;
- direct/bundle relationships are relationship-preserving, not independent filename/install sets;
- bundle projection is all-or-nothing and maps to one coherent ArtifactSet;
- archive projection preserves archive and member facts but cannot become an ArtifactSet without a separate qualified extraction seam.

### Diagnostic/security invariants

- adapter errors do not echo raw exact URLs or credentials;
- no arbitrary file contents enter diagnostics;
- projection performs no network or process I/O;
- local-file materialization rejects missing, non-regular, symlink, relative, wrong-size, missing-member, and extra-member inputs before ArtifactSet success.

## 5. Scope

### In scope

- add workspace crate crates/eggup-eggpack;
- mark it publish = false for M001;
- depend on eggpack-manifest from an immutable exact Eggpack Git revision;
- depend on local eggup-core and eggup-acquisition;
- strict ReleaseManifest target projection;
- direct/bundle installable projections;
- archive projection with explicit extraction-required semantics;
- exact caller AcquisitionRequest binding;
- manifest-derived per-artifact FetchLimits tightening;
- exact acquired-file set and byte-size validation;
- caller-supplied member PermissionsIntent;
- construction of direct/bundle ArtifactSet values with SHA-256 integrity evidence;
- copied compatibility fixtures pinned to the corrected Eggpack baseline;
- typed bounded adapter errors;
- dependency-direction and cross-repo fixture tests;
- README/rustdoc and closure evidence.

### Explicitly out of scope

- release discovery;
- GitHub API or repository metadata access;
- URL joining/base origin construction;
- HTTP/TLS transport implementation;
- invoking fetches inside projection;
- InstallPlan root selection;
- ownership verification;
- candidate execution/identity checks;
- commit/rollback orchestration;
- service lifecycle;
- archive extraction;
- package construction;
- producer ReleaseManifest generation;
- authenticity/signature verification;
- crates.io publication of eggup-eggpack;
- changing the published 0.1.0 Eggup crates.

## 6. Required production changes

### A. Add optional eggup-eggpack crate

Add crates/eggup-eggpack to the workspace.

Package constraints:

- Rust 1.89;
- edition 2021;
- unsafe forbidden;
- missing docs denied;
- publish = false for M001;
- no default feature coupling into other Eggup crates.

Dependencies should be limited to:

- eggup-core via workspace/local path;
- eggup-acquisition via workspace/local path;
- eggpack-manifest from exact Git revision 678bbf04f5a02827003a1d9ab83ba4f0e6360e41;
- Serde only if needed for test/provenance fixtures, preferably dev-only.

Do not depend on Eggpack's producer crates.

The exact Git dependency is an operational bridge while eggpack-manifest is unpublished. Do not use a branch/tag/floating Git reference. Do not vendor or copy the parser.

### B. Projection model

Define a small consumer-side projection model equivalent in purpose to:

~~~text
ManifestProjection
  - Installable(InstallableProjection)   # direct or bundle
  - Archive(ArchiveProjection)           # acquisition/member facts; extraction required

InstallableProjection
  product: eggup_core::ProductId
  release: eggup_core::ReleaseId
  target: canonical triple
  artifacts: ordered ArtifactRequirement[]

ArtifactRequirement
  artifact_name
  exact_size
  sha256 [u8; 32]
  member_id: eggup_core::MemberId
  relative_destination

ArchiveProjection
  product/release/target
  archive artifact name/exact_size/sha256
  expected members: source/install/exact_size/sha256
  extraction_required: structural state, not caller-toggleable
~~~

Rust names may differ, but the ownership boundary must remain.

Direct produces one ArtifactRequirement. Bundle produces exactly one requirement per manifest bundle entry. Archive does not expose a direct conversion to ArtifactSet in M001.

Projection order must be deterministic and preserve the manifest's validated ordering.

### C. Exact target selection

Provide a pure entry point equivalent to:

~~~text
project(manifest: &ReleaseManifest, canonical_target: &str)
    -> Result<ManifestProjection, AdapterError>
~~~

Requirements:

1. call/require manifest validation;
2. select exactly the canonical target using the manifest's exact-target helper;
3. convert product_id/release_id through Eggup ProductId/ReleaseId constructors;
4. convert SHA-256 through eggpack-manifest's validated byte helper;
5. convert install identity through MemberId;
6. preserve direct/bundle/archive relationships;
7. return typed failure on target absence or malformed identity;
8. perform no filesystem/network/process I/O.

### D. Caller-owned exact acquisition requests

Do not accept a base URL.

For InstallableProjection, provide a binding API that accepts an exact map keyed by manifest artifact name:

~~~text
artifact_name -> eggup_acquisition::AcquisitionRequest
~~~

The map must contain every projected artifact exactly once and no extras.

For each artifact derive an effective FetchLimits value from a caller-provided baseline:

- validate the caller limits;
- if caller max_artifact_bytes is Some(n) and n < manifest exact_size, fail rather than widening policy;
- otherwise set max_artifact_bytes to Some(manifest exact_size);
- preserve caller connect/total timeouts and metadata bound;
- never widen any caller bound.

Return deterministic PlannedAcquisition values pairing:

- artifact identity;
- exact request;
- tightened limits;
- exact expected size/digest/member identity.

Do not perform the fetch.

ArchiveProjection may expose the same single-archive request binding behavior, but it must remain marked extraction-required and must not provide member ArtifactSet construction.

### E. Acquired-file materialization

For direct/bundle only, provide a post-acquisition conversion equivalent to:

~~~text
materialize_artifact_set(
    acquired: artifact_name -> absolute local path,
    permissions: member_id -> PermissionsIntent
) -> Result<ArtifactSet, AdapterError>
~~~

Before constructing ArtifactSet:

- require exact artifact-name set equality;
- require exact permissions/member set equality;
- reject relative paths;
- use symlink_metadata and reject symlinks/non-regular objects;
- require local file size == manifest exact_size;
- never infer permissions;
- do not recompute or relabel authenticity.

Construct each ArtifactMember with:

- manifest-derived MemberId;
- caller-acquired local path;
- manifest-derived flat relative destination;
- caller-supplied PermissionsIntent;
- IntegrityRequirement::Sha256(manifest digest).

ArtifactSet creation must remain all-or-nothing.

Do not construct InstallPlan automatically in M001. The caller must explicitly provide the installation root and may then combine projection product/release with the returned ArtifactSet through eggup-core::InstallPlan::new. This keeps root/install authorization outside the manifest adapter.

### F. Archive boundary

ArchiveProjection must expose:

- archive artifact acquisition requirement;
- exact archive size/digest;
- expected member source/install/size/digest relationships.

It must not:

- extract;
- accept arbitrary archive entries;
- map member source paths directly to live destinations;
- create ArtifactSet from archive bytes;
- claim that the archive contains the expected member bytes merely because the manifest says so.

If a generic materialization entry point receives archive form, return a typed ArchiveExtractionRequired error or use the type system so that operation is unavailable.

### G. Error model

Define a bounded AdapterError with variants sufficient to distinguish:

- invalid manifest/identity;
- target not found;
- unsupported operation/layout;
- missing/extra acquisition request;
- caller acquisition bound below manifest size;
- missing/extra permissions policy;
- missing/extra acquired path;
- relative/non-regular/symlink acquired path;
- exact-size mismatch;
- archive extraction required;
- Eggup core/acquisition construction failure.

Errors may name manifest artifact/member identities. They must not print raw URLs, secrets, or arbitrary file contents. Avoid embedding absolute local paths where an artifact/member identity is sufficient.

## 7. Ordered work packages

1. Add eggup-eggpack crate with immutable eggpack-manifest Git dependency and publish=false.
2. Copy the corrected Eggpack interoperability fixtures into an Eggup test fixture directory with a provenance README recording upstream commit and SHA-256 identities.
3. Implement typed projection/error models and exact canonical target selection.
4. Implement direct projection.
5. Implement complete relationship-preserving bundle projection.
6. Implement archive facts/extraction-required projection.
7. Implement exact AcquisitionRequest binding and manifest-tightened FetchLimits.
8. Implement direct/bundle acquired-file exact-size validation plus caller-permissions ArtifactSet construction.
9. Add cross-repo positive/negative fixture matrix.
10. Add dependency/source scans proving no lower-layer Eggpack dependency and no producer/network authority.
11. Update crate/root docs, subsystem roadmap, registry, and closure evidence.
12. Observe all hosted CI lanes before closure.

## 8. Failure, cancellation, restart, and contention semantics

Projection and request binding are pure and synchronous.

No cancellation or retry semantics exist because the adapter does not execute network operations.

Acquired-file materialization is read-only:

- it reads metadata only;
- it does not copy, rename, delete, chmod, or mutate acquired files;
- failure returns no ArtifactSet;
- rerunning against unchanged inputs is deterministic.

The eventual Eggup prepare/commit path retains its own transaction, locking, rollback, recovery, and cancellation semantics. M001 must not wrap or weaken them.

Concurrent mutation of acquired files after the adapter's exact-size metadata check remains possible until eggup-core stages/verifies them. This is acceptable because the adapter does not claim candidate verification; the core's staging/integrity path remains authoritative. Document this boundary rather than adding a second transaction layer.

## 9. Compatibility and migration

This is additive.

Existing Eggup crates and consumers do not gain an Eggpack dependency.

eggup-eggpack is optional and unpublished in M001. Consumers may continue using their existing release mapping.

The adapter pins eggpack-manifest to exact Eggpack revision 678bbf04f5a02827003a1d9ab83ba4f0e6360e41. A future M004/package-promotion milestone may switch to a registry dependency only after eggpack-manifest is published and compatibility is qualified.

Do not copy ReleaseManifest structs/parser into Eggup. Cross-repo fixture copies are test compatibility evidence only.

## 10. Required tests

### Upstream compatibility fixtures

Copy at minimum the corrected Eggpack M001a fixtures:

- direct-manifest.json;
- projection-direct.json;
- bundle-manifest.json;
- projection-bundle.json;
- archive-manifest.json;
- projection-archive.json;
- unknown-schema.json;
- wrong-target.json.

Record upstream baseline and fixture SHA-256 values. In particular, projection-bundle.json must have corrected hash:

f0b227442e2a2a28dc4e2100301233f2698703d15251b6f2459e6387df6c53ce

### Positive tests

- direct Eggsact target projects product/release/one artifact/member/digest exactly;
- CodeGG bundle projects all three entries exactly and one coherent group;
- bundle order is deterministic;
- archive projection exposes one archive artifact plus exact member facts and extraction-required state;
- ProductId/ReleaseId remain opaque;
- exact AcquisitionRequest map binds without URL rewriting;
- caller FetchLimits are tightened to each exact artifact size;
- direct acquired file with exact size and explicit permissions produces one-member ArtifactSet;
- CodeGG three-file acquired set with exact sizes and explicit per-member permissions produces one ArtifactSet;
- SHA-256 bytes in ArtifactMember integrity equal manifest digest bytes;
- caller can subsequently create InstallPlan by supplying an installation root, without adapter-owned root policy.

### Negative tests

- alias/nearest target such as linux-x64 fails when not a canonical manifest target;
- unknown target fails;
- unknown Manifest schema fails through eggpack-manifest;
- missing acquisition request fails;
- extra acquisition request fails;
- caller max_artifact_bytes smaller than manifest size fails;
- adapter never widens timeouts/bounds;
- missing acquired bundle member fails;
- extra acquired file mapping fails;
- missing permissions entry fails;
- extra permissions entry fails;
- relative source path fails;
- symlink source fails;
- directory/non-regular source fails;
- exact-size mismatch fails;
- archive cannot produce an ArtifactSet in M001;
- corrected three-member CodeGG bundle cannot regress to the old two-unit/eggsact-substitution shape;
- dependency tree proves eggup-core/acquisition/service do not depend on eggpack-manifest;
- source scan proves adapter has no GitHub/release-discovery/base-origin/archive-extraction/service code.

## 11. Required verification commands

Run and record:

~~~bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test -p eggup-core --all-targets --all-features --locked
cargo test -p eggup-acquisition --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-eggpack --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-acquisition --locked
cargo tree -p eggup-service --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-eggpack --all-targets --locked
./scripts/check-local.sh
git diff --check
~~~

Because eggup-eggpack is intentionally publish=false and uses an immutable Git dependency on the currently unpublished eggpack-manifest crate, M001 does not claim crates.io package qualification for the adapter. Existing publishable Eggup packages must remain packageable under their current release process.

Hosted CI must pass:

- Linux stable including fmt/check/test/clippy/docs;
- Linux Rust 1.89;
- macOS workspace check/test;
- Windows workspace check/test.

If the current CI does not fetch exact Git dependencies on all lanes, fix the CI/repository configuration narrowly rather than replacing the immutable dependency with a floating source.

## 12. Documentation updates

Add/update:

- crates/eggup-eggpack/README.md;
- crate-level rustdoc;
- root README workspace/crate overview;
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md;
- plans/registry.md;
- closure plans/closure/eggpack-manifest-interoperability/001-status.md.

Document prominently:

- adapter is optional;
- direct/bundle supported in M001;
- archive extraction deferred;
- exact URLs are caller-owned;
- installation root/ownership/permissions/validation remain caller-owned;
- integrity is not authenticity;
- Git dependency/publish=false is temporary operational packaging state, not architectural coupling.

## 13. Acceptance criteria

M001 closes only when:

- eggup-eggpack exists as an optional leaf adapter;
- lower Eggup crates remain free of Eggpack dependencies;
- eggpack-manifest is pinned to an immutable corrected upstream revision;
- direct and corrected three-entry CodeGG bundle fixtures project exactly;
- exact caller AcquisitionRequest maps are complete and unmodified;
- FetchLimits never exceed caller policy and are tightened to manifest exact sizes;
- exact acquired-file size is checked before ArtifactSet construction;
- SHA-256 evidence reaches IntegrityRequirement::Sha256 exactly;
- caller permissions are required rather than inferred;
- direct/bundle ArtifactSet construction is all-or-nothing;
- archive projection remains explicitly extraction-required;
- no release/origin/install-root/ownership/service/authenticity policy is introduced;
- stable/MSRV/macOS/Windows CI passes;
- no unresolved medium-or-higher adapter correctness/security finding remains.

## 14. Stop conditions

Stop and prepare a new ADR/plan if:

- consuming ReleaseManifest requires eggpack-core or another producer crate;
- the adapter must choose releases, mirrors, origins, or installation roots;
- Manifest v1 lacks enough relationship data for exact direct/bundle translation;
- direct/bundle mapping requires inference from filenames instead of manifest relationships;
- archive support cannot remain cleanly separated from extraction policy;
- eggup-core must gain an Eggpack dependency;
- publication constraints would require vendoring/reimplementing the Manifest parser;
- a trust/authenticity decision is required for the adapter to function.

## 15. Closure evidence required

Record:

- exact Eggup implementation SHA(s);
- exact Eggpack dependency revision and M001a closure baseline;
- eggup-eggpack Cargo dependency tree;
- proof lower Eggup crates remain Eggpack-independent;
- copied fixture inventory and SHA-256/provenance;
- direct/bundle/archive projection matrix;
- exact AcquisitionRequest/FetchLimits behavior;
- acquired-file size/materialization matrix;
- permissions-policy evidence;
- archive extraction-required evidence;
- full positive/negative tests;
- stable/Rust1.89/macOS/Windows hosted CI;
- docs;
- unresolved findings and severity;
- disposition on M002 archive handoff and M003 real-consumer adoption readiness.

## 16. Handoff notes

This milestone is intentionally a thin adapter, not a new updater framework.

Do not modify Eggpack under this plan.

Do not add producer behavior to Eggup.

Do not implement archive extraction just to make the archive fixture installable.

Do not publish eggup-eggpack under M001.

Service Lifecycle M005 and planning-hygiene C002 are independent Eggup work and may proceed in parallel; they do not change this adapter's hard dependencies.
