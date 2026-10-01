# Eggpack Manifest Interoperability Milestone 003 — Eggsact Real-Consumer Manifest Adoption

Status: implemented; closed 2026-10-01 — see `plans/closure/eggpack-manifest-interoperability/003-status.md` (consumer `eggstack/eggsact@65c916b`, hosted CI run `36902758482` + drift run `36902758396` green); M003a caller-bound destination policy corrective is closed; producer-evidence gate and bounded JSON adapter qualification remain satisfied

Eggup plan-authoring baseline: 229b61c920f54b50b7c953b060e54ca2673a201b (post-C010 M003 baseline refresh). M003a was subsequently registered at `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md`.

Historical selected consumer baseline for the bounded pass:

- eggstack/eggsact@576f4b0ac09238a42e5561c2da6da8ff4a47bce6

Historical producer/interface baseline for the bounded pass:

- eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41
- Eggpack ReleaseManifest M001/M001a/M002: closed
- Eggpack interoperability M001a: closed
- Eggup adapter M001a: closed at 19935ec3610a5238af33a9d4f05a14925ceac25c

Resume/reconciliation evidence reviewed 2026-10-01:

- eggstack/eggpack@56ed7e747fd39e4d6a32a9f1fe3e09dd44355069 — current reviewed producer/planning head; manifest schema crate remains unchanged from Eggup's immutable `678bbf04` pin;
- eggstack/eggsact@f1352101dab748066c788e65e21e1bf303cfe995 — current reviewed consumer head after M005a Windows deterministic-link implementation; updater destination/fallback semantics remain unchanged;
- `eggstack/eggsact: release/eggpack/distribution.toml` — producer authority for the five live unversioned asset names;
- Eggsact `v1.2.7` live release evidence — 15 staged/published assets including `release-manifest.json`;
- Eggup planning-hygiene C010 closed at `54c48dede5df87f91498806153b4cacbc321401c` (head refresh at `605fa8f7a50f5bd53f7f294e71290d53282f62e9`) — registry, archive-roadmap, M003 plan dependency prose, and Verified Update Core M008 blockers reconciled;
- M003 refresh then identified one generic adapter authority gap: the current materializer uses manifest `install` directly as the deployment destination even though Eggpack's consumer contract says the manifest does not select destinations. M003a owns the additive caller-bound destination seam and is the sole hard prerequisite before consumer edits.

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

## 3. Execution gates and producer-side reconciliation

### 3.1 Producer contract gate — satisfied 2026-10-01

The first bounded execution pass correctly stopped because the then-current Eggsact release workflow used live unversioned asset names while only version-qualified Eggpack fixtures existed, and no producer-owned manifest filename/publication convention had been adopted.

That condition is now resolved by producer-owned evidence:

1. Eggpack Ecosystem M001 is closed at `plans/closure/ecosystem-adoption/001-status.md` in `eggstack/eggpack`;
2. Eggsact Distribution M005 is closed at `plans/closure/distribution-update-release/005-status.md` in `eggstack/eggsact`;
3. `release/eggpack/distribution.toml` is now the producer authority for the existing public names `eggsact-{target}` / `eggsact-{target}.exe` and their sidecars;
4. the maintainer-authorized `v1.2.7` generated release staged and then published `release-manifest.json` alongside those exact artifacts.

M003 therefore MUST consume the established producer contract rather than the historical fixture naming. It still MUST NOT rename public assets, synthesize a second producer contract, or move release naming/origin policy into Eggup.

Before production edits, refresh the exact current Eggup and Eggsact SHAs and verify that the live DistributionContract and `release-manifest.json` convention remain unchanged. If they have materially changed, stop and re-review rather than mechanically applying this plan.

### 3.2 Manifest artifact availability gate — satisfied; consumer path still required

The producer-owned manifest asset convention is now `release-manifest.json`, proven by the real `v1.2.7` release pipeline and publication evidence. Eggsact may construct the exact manifest URL only after its existing policy has selected and authorized the release/tag and release origin.

This resolves the external availability gate; it does **not** close M003. The normal Eggsact updater still must:

- fetch the exact manifest as bounded metadata;
- distinguish manifest `NotFound` from all other acquisition failures;
- parse/project through `eggup-eggpack`;
- bind product/release/target identity to the release already selected by Eggsact;
- acquire the manifest-selected artifact under the already-authorized release origin;
- materialize and commit through the existing Eggup transaction path;
- retain the legacy sidecar path only for the explicit manifest-`NotFound` compatibility case;
- execute the complete failure/fallback/rollback matrix and hosted consumer verification.

A valid manifest whose selected artifact is absent remains an inconsistent-release hard failure; it MUST NOT become Cargo or legacy-sidecar fallback.

### 3.3 Package/API promotion gate

`eggup-eggpack` remains `publish = false` and currently pins the lightweight `eggpack-manifest` interface by immutable Git revision. M003 may use immutable revisions for consumer qualification, with all Eggup packages aligned to one source identity.

M003 does not authorize publication of `eggup-eggpack`, a new Eggsact release containing Git-only dependencies, or broader API stabilization. Package/API promotion remains M004.

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

`MAX_MANIFEST_BYTES` and `project_json(&[u8], canonical_target)` are already implemented and qualified in `eggup-eggpack`; Eggsact does not need a direct `eggpack-manifest` dependency. The remaining generic API gap is destination authority: direct/bundle materialization and archive plan construction currently use manifest `install` values directly as relative destinations. Eggpack's consumer contract defines those values as logical/default install identities and explicitly leaves destination/replacement authorization to the consumer. M003a corrects that boundary before Eggsact integration.

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

- close M003a caller-bound destination policy corrective before Eggsact production edits;
- retain the already-qualified bounded `project_json` / `MAX_MANIFEST_BYTES` API and its full regression matrix;
- expose caller-owned exact relative destination binding without weakening manifest artifact/member identity, size, digest, or relationship evidence;
- retain current manifest-default materialization helpers as compatibility wrappers;
- update eggup-eggpack README/rustdoc with the caller-owned destination distinction;
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

## 7. Required Eggup prerequisite — M003a

The bounded JSON parse/project work described by the original M003 pass is already implemented and qualified. Do not reimplement it.

Before Eggsact production edits, close:

- `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md`.

M003a must provide an additive caller-bound destination materialization seam for direct/bundle projections and the equivalent archive plan construction path while keeping current manifest-default helpers compatible.

For M003, the required consumer contract after M003a is:

1. manifest artifact name / exact size / SHA-256 / relationship remain producer evidence;
2. projected member identity remains stable;
3. Eggsact supplies the exact relative destination authorized by its current installation policy;
4. ownership verification still proves that exact current executable before mutation;
5. no destination or ownership inference moves into the adapter.

M003 must use the qualified M003a path rather than working around destination authority in Eggsact.

## 8. Required Eggsact implementation sequence

### A. Refresh actual release evidence

Before code edits, capture the current Eggsact release workflow, target table, asset names, checksum sidecars, and any release fixtures. Compare these to the current Eggpack contract intended for Eggsact.

The implementation record must explicitly resolve the version-qualified-fixture versus live-unversioned-asset naming difference described in section 3.

### B. Dependency-source alignment

Because eggup-eggpack is unpublished, pin eggup-core, eggup-acquisition, eggup-eggfetch, and eggup-eggpack to one exact Eggup Git revision containing the closed M003a adapter API.

Verify cargo tree contains one source identity for each Eggup package and no duplicate crates.io/git copy of eggup-core or eggup-acquisition.

No floating branch dependency.

### C. Manifest acquisition

Use the existing strict Eggfetch policy and a bounded metadata call. Current Eggup `FetchLimits` requires a finite positive `u64` artifact ceiling; do not carry forward Eggsact's historical `max_artifact_bytes: None` literal when aligning to the current Git revision. Use a finite Eggsact-owned ceiling (the current Eggup default 128 MiB is sufficient for the observed ~11–17 MiB v1.2.7 binaries) and allow `bind_requests` to tighten it to the manifest exact size.

The exact manifest URL is application policy and must come from the producer convention established in the preflight. Do not put release-origin construction into eggup-eggpack.

A manifest fetch outcome must distinguish structurally, not by parsing error strings:

- Success -> parse/project path;
- NotFound -> explicit legacy sidecar compatibility path;
- any other error -> hard failure.

If the current `eggup_get_text` helper erases metadata `NotFound` into a string error, introduce a narrow typed metadata helper for the updater rather than inspecting formatted text.

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

- pass the exact artifact-name -> absolute path map to the M003a caller-bound materialization API;
- bind the projected member to the exact basename of `env::current_exe()` under the existing `current_exe.parent()` installation root; this preserves canonical and renamed executable update-in-place behavior;
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
16. dependency tree has one Eggup source identity and no direct Eggpack producer crates;
17. renamed current-executable basename is updated in place rather than retargeted to the manifest default basename;
18. finite caller artifact ceiling is never widened and is tightened to manifest exact size.

Use a deterministic local HTTP fixture for manifest and artifact behavior. Public GitHub is not required for correctness tests.

Where available, run hosted Linux/macOS/Windows checks. Record native versus cross-check evidence truthfully.

## 10. Acceptance criteria

M003 closes only when all of the following are true:

- Eggsact's normal updater contains and exercises the eggup-eggpack manifest path;
- a producer-valid direct ReleaseManifest flows through real Eggsact update code to Eggup acquisition, M003a caller-bound ArtifactSet materialization, candidate validation, and commit;
- Eggsact does not directly parse/reimplement ReleaseManifest schema semantics;
- exact product/release/target policy remains in Eggsact;
- manifest-provided artifact identity/size/digest replace duplicate manifest-branch mapping while Eggsact remains authoritative for the exact deployment destination;
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
- M003a is not closed or a generic adapter defect would otherwise be worked around in Eggsact;
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
- M003a closure/API delta plus M001a/M002 regression result;
- dependency trees proving one Eggup source identity and no producer-crate leakage;
- before/after updater code inventory;
- release binary-size delta using the same profile/toolchain;
- local and hosted verification results;
- any blocked platform evidence stated as blocked, not inferred;
- explicit confirmation that no public release/package publication occurred;
- unresolved findings by severity.

The closure must update this roadmap and plans/registry.md. If M003 closes cleanly, M004 package/API promotion becomes the next interoperability decision point, subject to a publishable eggpack-manifest version. The historical statement that "M002 archive extraction remains independently blocked on the Phase 10 extraction contract" was correct at the time of the bounded pass and was resolved externally by Archive M001d closure, Eggpack Interop M002/M002a closure (hosted run `36477024102`), Core M009 publication (`eggup-core 0.1.2` + `eggup-archive 0.1.2`), and Egress consumer M006 closure (2026-09-29); the runtime archive handoff therefore no longer blocks the M003 consumer-path half.

### 12.1 Bounded-pass-still-holds verification (post-C010, plan-authoring baseline `605fa8f`)

The bounded-pass adapter/API implementation (`MAX_MANIFEST_BYTES`, `project_json`, focused tests) was re-verified intact on the post-C010 Eggup HEAD with the plan §9 commands on the local macOS ARM64 host:

- `cargo fmt --all -- --check` — clean.
- `cargo check --workspace --all-targets --locked` — clean.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — clean.
- `cargo test -p eggup-eggpack --all-targets --all-features --locked` — 7 + 11 + 28 = 46 passed (unit, focused, M001a interoperability).
- `cargo doc --workspace --no-deps --locked` — clean.
- `cargo +1.89.0 check --workspace --all-targets --locked` — clean (MSRV unchanged).
- `cargo tree -p eggup-eggpack --locked` — pins `eggpack-manifest v0.1.0` by immutable Git rev `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`; no other Eggpack edge; lower crates absent.
- `cargo tree -p eggup-core --locked` — `sha2` only.
- `cargo tree -p eggup-acquisition --locked` — no internal Eggup edge.
- `cargo tree -p eggup-service --locked` — depends only on `eggup-core`.

The dependency-boundary invariants in §5 hold unchanged: no Eggpack edge below `eggup-eggpack`, `eggup-core` remains `sha2`-only, `eggup-acquisition` remains transport-neutral, `eggup-service` is Eggpack-independent. The bounded JSON adapter work remains qualified, but the current consumer-contract review discovered the separately planned M003a destination-authority corrective. After M003a closes, the Eggsact cross-repo work in §8 becomes the remaining implementation gap.
