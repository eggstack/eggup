# Eggpack Manifest Interoperability Milestone 003 — Eggsact Real-Consumer Manifest Adoption

Status: blocked after bounded adapter/API qualification; real-consumer adoption is not closed (see `plans/closure/eggpack-manifest-interoperability/003-status.md`)

Eggup plan-authoring baseline: 77fe72a9f6e34e48e1f667d2869b05fd010c1462

Selected consumer baseline:

- eggstack/eggsact@576f4b0ac09238a42e5561c2da6da8ff4a47bce6

External producer/interface baseline:

- eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41
- Eggpack ReleaseManifest M001/M001a/M002: closed
- Eggpack interoperability M001a: closed
- Eggup adapter M001a: closed at 19935ec3610a5238af33a9d4f05a14925ceac25c

Source roadmap:

- plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Long-term requirements:

- plans/002-long-term-roadmap.md#phase-9--eggpack-authority-cutover-and-manifest-interoperability
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:

- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md

Primary class: capability / compatibility / cross-repository adoption

## 1. Objective

Use Eggsact as the first real consumer of the optional eggup-eggpack ReleaseManifest adapter.

The milestone must prove that a real updater can replace duplicated manifest-to-update mapping with this dependency direction:

~~~text
Eggsact release/version/origin policy
        |
        +--> exact ReleaseManifest URL chosen by Eggsact policy
        |          |
        |          v
        |     eggup-eggpack
        |          |
        |     exact target projection
        |          |
        +--> exact artifact URL(s) chosen under Eggsact release origin
                   |
                   v
          eggup-acquisition / eggup-eggfetch
                   |
                   v
          acquired exact local file
                   |
                   v
       eggup-eggpack ArtifactSet materialization
                   |
                   v
       Eggsact candidate/ownership policy
                   |
                   v
               eggup-core
          commit / rollback / receipt
~~~

Eggpack remains the producer authority for manifest contents and release artifact naming. Eggup remains the deployment mechanism. Eggsact remains the authority for version selection, release origin, fallback, exact supported target policy, installation destination, candidate identity, CLI behavior, and whether a release is eligible for update.

M003 is not a request to move Eggsact release policy into Eggup or to make Eggup a release-discovery client.

## 2. Why Eggsact is selected

Eggsact is the narrowest qualified real consumer for this milestone:

- Consumer Adoption M001 is already closed at eggstack/eggsact@576f4b0;
- it already uses eggup-core, eggup-acquisition, and eggup-eggfetch in the normal self-update path;
- it is a direct single-binary release, so M003 does not depend on the blocked archive-extraction milestone;
- its updater still owns target-to-asset naming, checksum-sidecar acquisition/parsing, and conversion of that evidence into Eggup ArtifactSet inputs;
- the corrected Eggpack/Eggup direct interoperability fixtures use Eggsact as the representative product;
- no service lifecycle or multi-artifact behavior is needed to prove the manifest seam.

Stegoeggo remains a useful second consumer later, but using it first would repeat the same direct-binary proof without the existing Eggsact interoperability fixture lineage.

CodeGG is not selected because its real release path is archive-based and the adapter intentionally refuses archive materialization before the generic extraction contract. Eggsearch adds service and target-diversity concerns unnecessary for this proof.

## 3. Execution gates and known producer-side mismatch

### 3.1 Required preflight before consumer production edits

Before changing Eggsact's normal update flow, inspect and record:

1. current Eggsact release workflow and actual release asset names;
2. the current Eggpack DistributionContract/ReleaseManifest producer mapping intended for Eggsact;
3. the producer-owned filename/location convention, if any, for the ReleaseManifest artifact itself;
4. whether a current Eggpack producer path can emit a manifest over Eggsact's existing release artifacts without renaming them;
5. whether the selected release/tag can expose that manifest at an exact caller-owned URL.

The copied M001/M001a direct fixture is compatibility evidence, not live naming authority. At this plan baseline it uses artifact names such as:

~~~text
eggsact-1.2.6-x86_64-unknown-linux-gnu
~~~

while the current Eggsact updater constructs live asset names such as:

~~~text
eggsact-x86_64-unknown-linux-gnu
~~~

M003 MUST NOT resolve that difference by silently changing Eggsact release asset names, by hard-coding the fixture convention into the updater, or by adding producer naming policy to Eggup.

If current Eggpack producer authority cannot describe the existing Eggsact release layout, stop the consumer cutover and route the producer-side correction/adoption work to Eggpack. Record the exact blocker. Do not duplicate a second contract in Eggsact or Eggup.

### 3.2 Manifest artifact availability gate

At plan authoring, Eggpack can construct ReleaseManifest v1 from explicitly supplied finalized files, but its broader CI/release staging and ecosystem-adoption work is not yet complete. M003 therefore must not pretend that public Eggsact releases already contain an Eggpack manifest.

Consumer/API qualification may proceed with deterministic locally served manifest-bearing release fixtures after the preflight contract is proven. The milestone may close only when the normal Eggsact updater contains the manifest path and that path is exercised end-to-end against producer-valid ReleaseManifest evidence.

If the normal updater cannot name an exact producer-owned manifest asset without inventing a new convention, leave M003 blocked after the API/fixture work rather than choosing a filename in Eggup.

### 3.3 Publication gate

eggup-eggpack and eggpack-manifest are currently unpublished. Any Eggsact qualification using them must use immutable Git revisions and MUST NOT publish a new Eggsact crate/release as part of M003.

Package/API promotion remains M004.

## 4. Current implementation evidence

At the Eggup baseline:

- eggup-eggpack is an unpublished leaf crate;
- it pins only eggpack-manifest from the corrected immutable Eggpack revision;
- lower Eggup crates have no Eggpack dependency;
- project(&ReleaseManifest, canonical_target) validates and exactly selects one canonical target;
- direct/bundle projections bind exact caller AcquisitionRequest values and tighten max_artifact_bytes to the manifest size;
- materialize_artifact_set requires exact acquired/permission maps, absolute regular non-symlink files, exact size, and preserves manifest SHA-256 bytes as Eggup integrity requirements;
- archive projection remains extraction-required;
- M001a qualified the direct/bundle/archive positive matrix and 20 negative classes.

One consumer-facing API gap is visible before adoption: project accepts an eggpack_manifest::ReleaseManifest value, while a consumer fetching JSON should not need a second direct dependency on eggpack-manifest merely to parse the document. M003 should close that leaf-boundary ergonomics gap in Eggup rather than teaching Eggsact about the upstream parser crate.

At the selected Eggsact baseline:

- crates.io chooses the latest stable release;
- target_for_host and asset_name own host/target/release policy;
- release_asset_url constructs the exact GitHub release origin;
- the updater fetches a .sha256 sidecar, parses the digest locally, downloads the binary, and then creates Eggup integrity/deployment inputs;
- only a genuine selected-asset 404 or unsupported target permits Cargo fallback;
- candidate identity, destination ownership, CLI behavior, and Windows running-image replacement are consumer-owned.

## 5. Invariants that must not regress

### Producer/consumer boundary

- Eggpack owns ReleaseManifest schema/content and producer artifact naming.
- Eggup MUST NOT generate a release manifest, choose a release, construct a repository origin, or publish assets.
- Eggsact owns release selection, exact manifest/artifact origin, target eligibility, fallback, install-root/destination, candidate identity, and CLI policy.
- eggup-core, eggup-acquisition, eggup-eggfetch, and eggup-service remain Eggpack-independent.
- Eggsact MUST NOT need a direct dependency on eggpack-core, eggpack-contract, eggpack-bootstrap, or Eggpack CI/build machinery.
- Prefer no direct Eggsact dependency on eggpack-manifest; parsing should remain behind eggup-eggpack.

### Trust and integrity

- ReleaseManifest SHA-256 is integrity evidence only, not authenticity.
- HTTPS/release-origin trust remains Eggsact transport policy.
- selected manifest product_id must equal Eggsact's expected product identity;
- selected manifest release_id must exactly equal the version already authorized by Eggsact;
- canonical target selection must be exact;
- malformed/oversized/unsupported-schema manifests fail closed;
- a present valid manifest referencing a missing artifact is an inconsistent release and MUST NOT silently downgrade to a legacy checksum path or Cargo fallback.

### Fallback

- unsupported host behavior remains unchanged;
- legacy manifest absence MAY use the existing sidecar path only as an explicit backwards-compatibility policy for releases that predate manifests;
- only an actual manifest NotFound result may enter that compatibility path;
- timeout, TLS, proxy, 5xx, malformed JSON, unsupported schema, wrong product/release/target, map mismatch, size mismatch, checksum mismatch, candidate mismatch, ownership conflict, or commit failure MUST NOT become manifest-absence fallback;
- once a valid manifest has been accepted, artifact NotFound is a hard incomplete-release failure.

### Deployment

- candidate bytes are still validated before commit;
- current-executable ownership proof remains required;
- checksum/size evidence flows through Eggup staging and locked revalidation;
- transaction outcomes remain Committed, RolledBack, or RecoveryRequired with the existing Eggsact CLI mapping;
- Windows running-image replacement scope remains unchanged unless separately planned.

## 6. Scope

### In Eggup

- add a bounded consumer-facing JSON parse/project helper to eggup-eggpack so downstream consumers do not need a direct eggpack-manifest dependency;
- expose the upstream manifest document byte bound through the adapter, or otherwise provide a stable adapter-owned bound for acquisition;
- add focused API tests proving malformed/oversized/unsupported manifests remain AdapterError::InvalidManifest without leaking raw document content;
- retain the existing project(&ReleaseManifest, ...) API for callers that already own a parsed manifest;
- update eggup-eggpack README/rustdoc with the intended real-consumer flow;
- do not modify lower Eggup crates unless a genuine generic defect is found.

### In Eggsact

- align all Eggup dependencies used by the manifest qualification path to the same immutable Eggup Git revision so Cargo does not create incompatible crates.io-vs-git Eggup type identities;
- add eggup-eggpack from that exact revision;
- do not add a direct eggpack-manifest dependency unless the Eggup parse/project helper proves technically impossible; if that occurs, stop and record the generic API deficiency instead;
- retain crates.io version-selection policy and current canonical target mapping;
- fetch the exact producer-owned manifest URL as bounded metadata using the existing Eggup/Eggfetch transport policy;
- validate/project it through eggup-eggpack;
- enforce expected Eggsact product id and exact selected release id before artifact acquisition;
- construct exact artifact URL(s) under the already-authorized Eggsact release origin using the manifest-provided artifact name;
- bind the exact AcquisitionRequest through the adapter so manifest size tightens the caller's existing artifact limit rather than widening it;
- acquire to an explicit local path using the existing Eggup acquisition transport;
- materialize the direct ArtifactSet through eggup-eggpack with caller-supplied Executable permission intent;
- preserve the existing Eggsact candidate validator, ownership verifier, InstallPlan root, transaction handling, and CLI mapping;
- retain the old sidecar path only for the explicit manifest-NotFound compatibility case while producer rollout is incomplete;
- materially delete manifest-path-local checksum parsing/mapping that is superseded once the adapter path is selected.

### Explicitly out of scope

- changing Eggsact public release asset naming merely to match old fixtures;
- generating or publishing ReleaseManifest files from Eggup;
- Eggpack CI/workflow implementation;
- bootstrap installer changes;
- release discovery inside eggup-eggpack;
- automatic mirror/base-URL construction;
- service lifecycle changes;
- archive extraction;
- CodeGG/Egress adoption;
- authenticity/signature policy;
- crates.io publication of eggup-eggpack or eggpack-manifest;
- publishing an Eggsact release with Git dependencies;
- removing legacy sidecar compatibility before producer adoption proves it is safe.

## 7. Required Eggup adapter changes

### 7.1 Bounded parse/project entry point

Add an API equivalent in behavior to:

~~~text
pub const MAX_MANIFEST_BYTES: usize = <upstream schema bound>;

pub fn project_json(
    input: &[u8],
    canonical_target: &str,
) -> Result<ManifestProjection, AdapterError>
~~~

Exact naming may differ.

Requirements:

1. reject input above the upstream ReleaseManifest bound before allocation-heavy parsing;
2. reject invalid UTF-8 as InvalidManifest;
3. delegate schema parsing/validation to eggpack-manifest::ReleaseManifest::from_json rather than reimplementing the schema;
4. delegate target projection to the existing project path;
5. never return parser messages containing raw JSON or caller URLs;
6. keep project(&ReleaseManifest, ...) public and behavior-compatible;
7. do not re-export producer build/contract types.

If re-exporting the leaf ReleaseManifest type is demonstrably cleaner than a parse helper, document the dependency consequence and stop for review before making Eggsact depend directly on eggpack-manifest. The default implementation direction is the adapter-owned parse helper.

### 7.2 Tests

Add focused tests for:

- valid direct fixture through JSON bytes;
- exact MAX_MANIFEST_BYTES boundary behavior;
- oversized document rejection;
- invalid UTF-8;
- malformed JSON;
- unsupported schema;
- wrong canonical target after successful parse;
- diagnostics that contain neither raw JSON fragments nor credential-bearing URL material.

Re-run the full M001a interoperability suite; M003 must not weaken any existing negative case.

## 8. Required Eggsact implementation sequence

### A. Refresh actual release evidence

Before code edits, capture the current Eggsact release workflow, target table, asset names, checksum sidecars, and any release fixtures. Compare these to the current Eggpack contract intended for Eggsact.

The implementation record must explicitly resolve the version-qualified-fixture versus live-unversioned-asset naming difference described in section 3.

### B. Dependency-source alignment

Because eggup-eggpack is unpublished, pin eggup-core, eggup-acquisition, eggup-eggfetch, and eggup-eggpack to one exact Eggup Git revision containing the M003 adapter helper.

Verify cargo tree contains one source identity for each Eggup package and no duplicate crates.io/git copy of eggup-core or eggup-acquisition.

No floating branch dependency.

### C. Manifest acquisition

Use the existing strict Eggfetch policy and a bounded metadata call.

The exact manifest URL is application policy and must come from the producer convention established in the preflight. Do not put release-origin construction into eggup-eggpack.

A manifest fetch outcome must distinguish:

- Success -> parse/project path;
- NotFound -> explicit legacy sidecar compatibility path;
- any other error -> hard failure.

### D. Product/release/target binding

After projection and before artifact acquisition:

- require ProductId == eggsact;
- require ReleaseId == the exact StableVersion already selected by Eggsact;
- require the current canonical target exactly;
- reject all mismatch as hard errors.

Do not allow the manifest to select a newer/different release than the one already authorized by Eggsact.

### E. Artifact request and acquisition

Use the manifest artifact name only after the release origin and release id have been authorized by Eggsact.

Construct the exact URL under that release origin, create AcquisitionRequest, and call ManifestProjection::bind_requests with Eggsact's existing timeout/body policy.

The adapter may tighten the artifact byte maximum to exact manifest size. It must never widen Eggsact's caller bound.

A manifest-backed artifact NotFound is hard failure.

### F. Materialization and transaction

After acquisition:

- pass the exact artifact-name -> absolute path map to materialize_artifact_set;
- supply explicit Executable permission intent;
- use install_ids/projection identities only after Eggsact's product/release equality checks;
- preserve ExactIdentityValidator policy for eggsact <version>;
- preserve CurrentExeVerifier and DenyCreate;
- preserve InstallPlan root as current executable parent;
- preserve Windows staged replacement limitations and messaging;
- preserve transaction disposition mapping.

### G. Legacy path isolation

Keep legacy .sha256 parsing/acquisition only behind the exact manifest-NotFound compatibility branch.

Add a structural test or code organization guard making it clear that a present-but-invalid manifest cannot enter the legacy path.

Do not duplicate the adapter's exact-size/digest mapping in the manifest branch.

## 9. Required verification

### Eggup

Run at minimum:

~~~text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree -p eggup-eggpack --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-acquisition --locked
cargo tree -p eggup-service --locked
./scripts/check-local.sh
git diff --check
~~~

Dependency evidence must still show no Eggpack edge below eggup-eggpack.

### Eggsact

Use its existing full verification plus focused manifest-path tests.

Required behavior matrix:

1. selected release already current -> no manifest/artifact download;
2. supported target + valid manifest + exact artifact -> successful update;
3. manifest product mismatch -> hard failure;
4. manifest release mismatch -> hard failure;
5. manifest target missing/wrong -> hard failure;
6. malformed/unsupported/oversized manifest -> hard failure;
7. manifest timeout/TLS/proxy/5xx -> hard failure;
8. manifest NotFound -> legacy sidecar path only;
9. valid manifest + artifact NotFound -> hard failure, no Cargo fallback;
10. exact-size mismatch -> failure before commit;
11. digest mismatch -> failure before candidate execution/commit;
12. wrong candidate identity/version -> failure before commit;
13. ownership conflict -> no mutation;
14. rollback and RecoveryRequired mapping remain distinct;
15. unsupported host -> existing Cargo fallback policy unchanged;
16. dependency tree has one Eggup source identity and no direct Eggpack producer crates.

Use a deterministic local HTTP fixture for manifest and artifact behavior. Public GitHub is not required for correctness tests.

Where available, run hosted Linux/macOS/Windows checks. Record native versus cross-check evidence truthfully.

## 10. Acceptance criteria

M003 closes only when all of the following are true:

- Eggsact's normal updater contains and exercises the eggup-eggpack manifest path;
- a producer-valid direct ReleaseManifest flows through real Eggsact update code to Eggup acquisition, ArtifactSet materialization, candidate validation, and commit;
- Eggsact does not directly parse/reimplement ReleaseManifest schema semantics;
- exact product/release/target policy remains in Eggsact;
- manifest-provided artifact identity/size/digest replace duplicate manifest-branch mapping;
- lower Eggup crates remain Eggpack-independent;
- a valid manifest cannot silently downgrade to legacy sidecar/Cargo fallback on any later failure;
- legacy compatibility is entered only on exact manifest NotFound while producer rollout requires it;
- the current live asset naming contract is preserved unless separately changed by an Eggpack producer plan;
- no archive, service, CI, bootstrap, publication, or authenticity authority enters the adapter;
- dependency and binary-size impact are measured;
- no high- or medium-severity generic defect remains.

A deterministic local end-to-end manifest-bearing release fixture is sufficient for correctness qualification. A public release is not required, but the normal updater path and exact producer convention must be real rather than test-only dead code.

## 11. Stop conditions

Stop and write the appropriate corrective or cross-repository plan if:

- Eggpack cannot describe Eggsact's existing release artifact naming without changing producer semantics;
- no producer-owned ReleaseManifest asset convention exists for the normal updater to address;
- Eggsact would need a direct dependency on eggpack-core, eggpack-contract, bootstrap, or CI machinery;
- Cargo creates duplicate incompatible Eggup source identities that cannot be eliminated by one immutable revision;
- consumer adoption requires release discovery/version ordering inside eggup-eggpack;
- a present invalid manifest would have to fall back silently to legacy evidence;
- a generic adapter defect would otherwise be worked around in Eggsact;
- implementation attempts to publish unpublished Git dependencies.

If a generic eggup-eggpack defect is found, correct and requalify Eggup first, then resume consumer adoption. If the issue is producer naming/manifest publication, route it to Eggpack rather than expanding Eggup authority.

## 12. Closure evidence required

The closure record must include:

- exact Eggup, Eggpack, and Eggsact SHAs;
- the resolved producer manifest artifact convention and evidence source;
- the actual Eggsact release asset naming comparison, including disposition of the historical version-qualified fixture mismatch;
- before/after Eggsact updater authority map;
- exact legacy-fallback truth table;
- focused end-to-end manifest fixture description;
- adapter API delta and M001a regression result;
- dependency trees proving one Eggup source identity and no producer-crate leakage;
- before/after updater code inventory;
- release binary-size delta using the same profile/toolchain;
- local and hosted verification results;
- any blocked platform evidence stated as blocked, not inferred;
- explicit confirmation that no public release/package publication occurred;
- unresolved findings by severity.

The closure must update this roadmap and plans/registry.md. If M003 closes cleanly, M004 package/API promotion becomes the next interoperability decision point, subject to a publishable eggpack-manifest version. M002 archive extraction remains independently blocked on the Phase 10 extraction contract.
