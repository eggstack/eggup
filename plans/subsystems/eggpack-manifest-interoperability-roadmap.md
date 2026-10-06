# Eggpack Manifest Interoperability Roadmap

Status: active

Long-term references:

- plans/002-long-term-roadmap.md#phase-9--eggpack-authority-cutover-and-manifest-interoperability
- plans/001-terminology-and-domain-model.md#7-release-plan
- plans/001-terminology-and-domain-model.md#8-install-plan
- plans/001-terminology-and-domain-model.md#11-integrity-evidence

Applicable ADRs:

- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md
- plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md

## 1. Purpose and ownership boundary

Own the optional consumer-side translation from a stable Eggpack ReleaseManifest into explicit Eggup acquisition and deployment inputs.

This subsystem does not restore producer authority to Eggup. Eggpack remains authoritative for release contracts, final manifests, artifact construction, bootstrap generation, release CI, staging/publication, and producer provenance. Eggup remains authoritative for acquisition mechanics, local verification, candidate preparation, mutation, rollback/recovery, receipts, and service lifecycle. Applications remain authoritative for release selection, exact artifact origins, installation roots, ownership policy, permissions policy, and product-specific validation.

eggup-core, eggup-acquisition, and eggup-service MUST remain usable without Eggpack.

## 2. Work classification

### Invariants

- the adapter depends only on the lightweight eggpack-manifest consumer crate, never eggpack-core, eggpack-contract, bootstrap, or CI crates;
- target selection is exact canonical matching;
- manifest ProductId/ReleaseId remain opaque;
- SHA-256 remains integrity evidence, not authenticity;
- exact artifact size remains an acquisition/post-fetch constraint;
- caller policy supplies exact URLs and installation root;
- manifest install names become relative member identities/destinations only; they do not grant ownership or replacement authority;
- direct/bundle mapping preserves every artifact/install relationship;
- bundle projection is complete and one transactionally coherent ArtifactSet;
- archive acquisition/member evidence is preserved, but archive extraction remains a separate consumer-owned/qualified boundary;
- no release selection, mirror fallback, version ordering, service policy, elevation, or publication enters the adapter.

### Capabilities

- parse/accept one validated ReleaseManifest v1;
- select one exact canonical target;
- project direct/bundle artifact requirements into bounded Eggup acquisition inputs;
- enforce exact URL-map completeness without constructing origins;
- verify exact post-fetch file size before creating deployment members;
- attach manifest SHA-256 as Eggup integrity requirements;
- produce one coherent ArtifactSet for direct/bundle layouts with caller-supplied permissions intent;
- expose archive acquisition/member facts with an explicit extraction-required state.

### Infrastructure

- optional eggup-eggpack crate;
- pinned eggpack-manifest dependency;
- copied/pinned cross-repository compatibility fixtures with upstream provenance and SHA-256 identities;
- adapter-specific typed errors;
- cross-repo compatibility tests.

### Polish

- package/publish promotion after eggpack-manifest has a suitable published version;
- higher-level convenience helpers only after real consumers demonstrate stable policy.

## 3. Non-goals

- producer release generation;
- DistributionContract parsing;
- build/qualification/CI integration;
- release/version discovery;
- GitHub API integration;
- base-URL joining or mirror choice;
- network transport implementation;
- archive extraction;
- service lifecycle;
- ownership inference;
- authenticity/signature policy;
- mandatory Eggpack dependency for Eggup users.

## 4. Current state

Eggup distribution M004 removed the historical producer-side eggup-dist crate. Eggup's core/acquisition/service layers are independently qualified and remain producer-neutral.

Eggpack ReleaseManifest v1 is implemented and its consumer interface is now corrected/qualified:

- Eggpack manifest M001/M001a and M002 are closed;
- Eggpack interoperability M001a implementation: 8d9264b3c224f3f05a061f4038b0f328b1c5c95e;
- corrected closure in `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41`: `plans/closure/eggup-interoperability/001a-status.md` in that repository;
- hosted CI run: 35998756491, all required lanes passed;
- corrected CodeGG bundle projection SHA-256: f0b227442e2a2a28dc4e2100301233f2698703d15251b6f2459e6387df6c53ce.

The previous Eggup planning block on Eggpack M001a is satisfied. Eggup adapter M001 then implemented and closed historically, but post-closure review found an incomplete adapter regression matrix and an out-of-scope service-manifest edit in the implementation commit. Corrective M001a has since closed with the full regression matrix and service packageability reconciliation. M003 bounded adapter/API qualification is also complete.

The producer-side stop recorded by the first M003 execution pass has now been resolved externally. Eggpack Ecosystem M001 and Eggsact Distribution M005 closed on the real `v1.2.7` release: `eggstack/eggsact: release/eggpack/distribution.toml` is the producer authority for the live unversioned `eggsact-{target}[.exe]` names, and the generated release staged and the maintainer published `release-manifest.json` alongside the binaries, checksum sidecars, and installers.

A current consumer-contract refresh identified one generic adapter authority gap before Eggsact can adopt that path: the adapter's materializers used manifest `install` directly as the deployment destination, while the Eggpack consumer contract explicitly says the manifest does not select destinations or replacement authorization. M003a closed that gap with an additive caller-bound destination seam (implementation `39ff626`, hosted run `36890986000` green on all lanes; see `plans/closure/eggpack-manifest-interoperability/003a-status.md`). M003 Eggsact real-consumer adoption has now closed on that seam (consumer `eggstack/eggsact@65c916b`, hosted CI run `36902758482` + drift run `36902758396` green; see `plans/closure/eggpack-manifest-interoperability/003-status.md`); the producer gate and bounded JSON parse/project work remain satisfied.

## 5. Target architecture

~~~text
application release policy
       |
       +--> exact ReleaseManifest bytes
       |          |
       |          v
       |    eggpack-manifest
       |          |
       |          v
       |    eggup-eggpack adapter
       |      /              \
       |     /                \
       | direct/bundle       archive facts
       |     |                  |
       |     v                  +--> qualified consumer extraction [later]
       | acquisition requirements
       |     |
       +--> exact caller URLs
             |
             v
      eggup-acquisition
             |
             v
      acquired local files
             |
      exact-size gate + caller permissions
             |
             v
        ArtifactSet
             |
       caller InstallPlan root/
       ownership/validation policy
             |
             v
          eggup-core
~~~

Dependency direction:

~~~text
eggpack-manifest ---> eggup-eggpack <--- eggup-acquisition
                              |
                              v
                         eggup-core

eggup-core/acquisition/service -X-> eggpack-manifest
~~~

## 6. Dependency graph

### Hard

- Eggpack Interop M001a closure: satisfied at eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41;
- current Eggup eggup-core ArtifactSet/InstallPlan/integrity interface;
- current Eggup eggup-acquisition exact-request/FetchLimits interface.

### Interface

- corrected Eggpack direct/bundle/archive fixtures and projection semantics.

### Operational

- `eggup-eggpack` was an unpublished leaf adapter pinning `eggpack-manifest` by immutable Git revision until M004. M003 used immutable revisions for consumer qualification; registry/package promotion was the separate M004 decision. Eggpack published `eggpack-manifest 0.1.0` on 2026-10-02 with published bytes identical to that qualified Git source, and M004 made the switch: the adapter now resolves `eggpack-manifest = "=0.1.0"` from crates.io and is itself published as `eggup-eggpack 0.1.2` (see M004 below).

### Current M003 dependency

- M003a caller-bound destination policy corrective: closed (implementation `39ff626`, hosted run `36890986000`); M003 Eggsact real-consumer adoption: closed (consumer `eggstack/eggsact@65c916b`, hosted CI run `36902758482` + drift run `36902758396` green; see `plans/closure/eggpack-manifest-interoperability/003-status.md`).

### Soft

- Phase 10 archive extraction work is not needed for M001 direct/bundle support.

## 7. Milestones

### M001 — ReleaseManifest v1 direct/bundle adapter

Create eggup-eggpack as an optional leaf adapter. Project exact target release evidence, build bounded acquisition inputs from caller-supplied exact URLs, verify exact acquired sizes, and construct direct/bundle ArtifactSets with manifest SHA-256 evidence and caller-supplied permissions intent. Preserve archive facts but return/retain an explicit extraction-required boundary.

Implementation plan: plans/implementation/eggpack-manifest-interoperability/001-release-manifest-v1-adapter.md.

### M001a — Adapter qualification and closure hardening

Expand the adapter compatibility/negative matrix, mechanically compare adapter output to the copied direct/bundle/archive projection fixtures, reconcile historical service packageability scope, and re-close before consumer adoption.

Implementation plan: `plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md`.

### M002 — Archive extraction handoff

Status: implementation landed; closure qualification corrective M002a closed via hosted run `36477024102` (Stable clippy/tests/docs + MSRV + macOS + Windows adapter runtime all green). Original plan: `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`; closure: `plans/closure/eggpack-manifest-interoperability/002-status.md` (M002a addendum); corrective: `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md` → `plans/closure/eggpack-manifest-interoperability/002a-status.md`. Archive M001d remains closed and provides the required object-bound handoff. No Eggpack producer schema/code change was required.

`ManifestProjection::Archive` evidence now connects to the qualified local extraction layer and then to ArtifactSet construction through five typed adapter helpers (`archive_format_for_name`, `validate_acquired_archive`, `archive_plan_for`, `core_plan_for_archive`, `bind_archive_members`). Archive format dependencies stay out of `eggup-core` (still `sha2`-only) and `eggup-eggpack` itself performs no extraction, commit, or cleanup.

### M003a — Caller-bound destination policy corrective

Status: closed (implementation `39ff626`; closure `plans/closure/eggpack-manifest-interoperability/003a-status.md`; hosted run `36890986000` green on Stable/MSRV/macOS/Windows).

Implementation plan: `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md`.

Corrected the adapter authority seam so ReleaseManifest `install` remains producer logical/default install identity while the consuming application binds the exact relative deployment destination. Added caller-bound direct/bundle materialization (`materialize_artifact_set_with_destinations`) and the equivalent archive plan path (`core_plan_for_archive_with_destinations`) plus a pure `default_destinations` helper, retained existing manifest-default helpers as compatibility wrappers over one implementation, and proved exact destination-map failure behavior without weakening artifact/member size, digest, relationship, permission, or ownership boundaries.

M003a did not implement Eggsact networking/fallback behavior and did not publish the adapter. Its closure is what re-enabled the M003 Eggsact consumer implementation.

### M003 — Eggsact real-consumer manifest adoption

Status: closed (consumer `eggstack/eggsact@65c916b`; closure `plans/closure/eggpack-manifest-interoperability/003-status.md`; hosted CI run `36902758482` + drift run `36902758396` green).

Selected consumer: Eggsact, the already-qualified direct single-binary Eggup adopter.

Implementation plan: plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md.

Adopt the adapter in Eggsact's real updater path so producer-valid ReleaseManifest evidence can replace duplicated manifest-to-update mapping while Eggsact retains release/version/origin/fallback/install/candidate policy.

The bounded JSON parse/project API and its tests are implemented in Eggup. The former producer-evidence gate is now satisfied by Eggpack Ecosystem M001 / Eggsact Distribution M005: the live producer contract preserves the existing unversioned artifact names and `v1.2.7` published `release-manifest.json`. M003 landed on the M003a caller-bound seam against current repository baselines (Eggup pin `e336b32`, Eggsact `65c916b` rebased over the M005a closure); Eggsact-owned release/origin/fallback/destination policy is preserved and `release-manifest.json` is treated as producer-established evidence rather than an Eggup naming decision.

### M004a — Package/API promotion readiness preflight

Status: closed (see `plans/closure/eggpack-manifest-interoperability/004a-status.md`; readiness preflight complete 2026-10-01).

Implementation plan: `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md` (implemented).

M004a proved the minimum registry-resolvable publication graph without publishing anything: `eggpack-manifest 0.1.0` is absent from crates.io (Eggpack-owned prerequisite); `eggup-core`/`eggup-archive 0.1.2` are published; `eggup-acquisition 0.1.2` and `eggup-eggfetch 0.1.2` are absent and both required (the latter proven by an `E0308` incompatibility between published `eggfetch 0.1.1` source and the `0.1.2` seam); the minimum Eggup publication set is `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` after the Eggpack prerequisite. No public API/version adjustment is required; adapter package-metadata gaps are polish.

That external prerequisite is now concretely registered in Eggpack as Release Manifest M003: `eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md` (initial plan registration `b8cfce59fa8da76f5563f25812c00dd4d22c90a0`; Eggpack active registry reconciliation `338ba64233f6a1d19f28608f5ed8194610fd851f`). Eggpack M003 owns source-identity/package qualification, explicit manual publication of only `eggpack-manifest 0.1.0`, and registry-only post-publication proof.

**That prerequisite is now satisfied.** Eggpack Release Manifest M003 closed: `eggpack-manifest 0.1.0` was published to crates.io on 2026-10-02 from Eggpack `8d661e4eb9da1806e5d7c7606939d24e9aceb2c0` (checksum `2a08f24b05e9652878dd49145cdc3cbd38c7a76032d7b01a5fe1535d9446b629`, non-yanked, tag `eggpack-manifest-v0.1.0`, Eggpack hosted run `37064833069` green, closure at `eggstack/eggpack: plans/closure/release-manifest/003-status.md`). M004a's registry audit no longer observes the crate as absent. The published `src/lib.rs` is byte-identical to the consumer-qualified Git source at Eggup's pin, so no Eggup-side semantic requalification of the parser is implied by the promotion; the compatibility-incident branch is not triggered. Only `eggpack-manifest` was published — no other Eggpack package became a prerequisite.

M004a published no crates and mutated no committed package graph. Its closure is the readiness gate for M004 proper.

### M004 — Package/API promotion

Status: **closed**. Implementation plan: `plans/implementation/eggpack-manifest-interoperability/004-registry-package-api-promotion-and-publication.md`; closure: `plans/closure/eggpack-manifest-interoperability/004-status.md`. Publication source `02a1d32` (release-prep `e548b64`; `crates/` and `Cargo.lock` byte-identical between them), hosted run `37090397398` green on all four lanes.

M004 published `eggup-acquisition 0.1.2` (checksum `0b01deb8…f170`), `eggup-eggfetch 0.1.2` (`2e483152…3528`), and `eggup-eggpack 0.1.2` (`9dbfdfb7…b13fef`) to crates.io on 2026-10-02, in that order. The full workspace 0.1.2 registry set is now `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack`; core/archive were not republished. The adapter resolves `eggpack-manifest` from the registry at `=0.1.0`, whose published `src/lib.rs` is byte-identical to the source it previously pinned by Git revision, so no requalification was needed. Two external registry-only graphs — adapter-only and Eggsact-shaped — resolve with one registry source per package, no `git+` or path edge, and pass smoke including the adapter's exact-size and fail-closed checks.

The publication order was proven load-bearing rather than assumed: `cargo package`/`cargo publish` resolve the packaged manifest's dependencies from crates.io, so `eggup-eggfetch 0.1.2` could not verify before `eggup-acquisition 0.1.2` was visible, and `eggup-eggpack 0.1.2` could not produce a package at all. The plan was amended to record this before any upload.

The former first gate — closure of Eggpack Release Manifest M003 (`eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md`) with exact `eggpack-manifest =0.1.0` registry resolution — is **satisfied** (Eggpack published `0.1.0` on 2026-10-02 from `8d661e4`, checksum `2a08f24b…b629`, closure `eggstack/eggpack: plans/closure/release-manifest/003-status.md`). The published `src/lib.rs` is byte-identical to the consumer-qualified Git source, so no adapter requalification is required.

Pre-publication audit, now discharged by M004 (historical — the versions below are published; see the M004 status line above): a 2026-10-02 registry re-audit against the M004a-proven set showed: `eggup-acquisition 0.1.2` still absent (max `0.1.1`); `eggup-eggfetch 0.1.2` still absent (max `0.1.1`), with the `E0308` incompatibility against published `eggfetch 0.1.1` still in force; `eggup-eggpack 0.1.2` still absent (crate unpublished); `eggup-core 0.1.2` and `eggup-archive 0.1.2` still published and non-yanked. So M004 published `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` in dependency order.

M004's production change is the mechanical promotion M004a identified: remove `publish = false` from `eggup-eggpack`, replace the immutable-Git `eggpack-manifest` edge with `eggpack-manifest = "=0.1.0"`, backfill the five missing package-metadata fields, correct the now-false "unpublished / Git-pinned" README and CHANGELOG statements, and refresh the yanked `yoke-derive 0.8.3` lockfile entry to `0.8.4`. No `src/` change, no public API change, and no version change — the workspace is already `0.1.2` and `eggup-eggpack`'s `=0.1.2` pins to the already-published `eggup-core`/`eggup-archive`.

`v0.1.1` and `v0.1.2` are immutable and are not moved. `git diff v0.1.2..HEAD -- crates/eggup-acquisition crates/eggup-eggfetch` is empty, so the two lower crates publish exactly the tagged content; `eggup-eggpack` has changed since that tag by M003a and is therefore not reproducible from it. Per the plan, the three crates are recorded by appending to the existing GitHub Release `0.1.2` notes, and each crates.io version carries its own exact publication commit in `.cargo_vcs_info.json`.

Eggpack owes M004 nothing further; do not wait on it. At the time of M004, `eggup-curl`, `eggup-service`, and `eggup-transport-footprint` were all unpublished; since then `eggup-curl 0.1.2` has been published (Acquisition M009, 2026-10-04), and `eggup-service` is published at `0.1.0`/`0.1.1` with its `0.1.2` still unauthorized, while `eggup-transport-footprint` remains `publish = false`. Eggsact's Git→registry migration stays separately authorized in `eggstack/eggsact`.

### M005 — eggup-eggpack 0.1.3 security/correctness publication

Status: blocked on Acquisition M010, Core M013, and Archive M004.

Implementation plan: `plans/implementation/eggpack-manifest-interoperability/005-eggup-eggpack-0.1.3-security-publication.md`.

Publish the existing member-identity cross-check that fixes the positional archive
binding defect in registry 0.1.2, and move the adapter's exact Eggup dependency
pins coherently to `=0.1.3`. No adapter policy/API change.

## 8. Cross-cutting requirements

- Rust 1.89 baseline;
- forbid unsafe code;
- bound all strings/collections through upstream manifest/parser contracts and adapter maps;
- no raw credential-bearing URL text in adapter diagnostics;
- no hidden I/O in projection;
- exact URL set and exact acquired-file set fail closed;
- deterministic projection ordering;
- copied compatibility fixtures carry upstream commit/blob/hash provenance;
- no new transport/TLS stack.

## 9. Verification strategy

- corrected Eggpack direct/bundle/archive fixtures copied byte-for-byte with provenance;
- direct and three-member CodeGG bundle positive mapping;
- missing/extra/crossed URL/acquired-file cases;
- wrong canonical target;
- exact-size mismatch;
- SHA-256 byte propagation;
- caller permissions preservation;
- archive extraction-required behavior;
- unknown schema and malformed manifest rejection through eggpack-manifest;
- dependency-tree proof that only the adapter depends on Eggpack;
- full workspace stable + Rust 1.89 + macOS + Windows CI.

## 10. Risks and decision points

The main risk is turning the adapter into a new release-policy or archive framework. Keep it as a translation layer.

A second risk was premature publication coupling. M001 pinned an immutable Git revision and kept the adapter unpublished for that reason. `eggpack-manifest` is now published, and M004 has discharged the coupling risk it created: `eggup-acquisition`/`eggup-eggfetch`/`eggup-eggpack` 0.1.2 are published in dependency order with exact pins intact. Do not vendor or reimplement the Manifest parser to avoid a constraint.

## 11. Completion definition

The subsystem is mature when at least one real Eggup consumer can consume Eggpack ReleaseManifest v1 through the adapter with less duplicated mapping, while:

- lower Eggup layers remain Eggpack-independent;
- application release/origin/install/ownership policy remains external;
- direct/bundle transactions preserve exact evidence;
- archive extraction is handled only through a separately qualified boundary;
- non-Eggpack releases remain fully supported.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 ReleaseManifest v1 direct/bundle adapter | closed (historical) | plans/implementation/eggpack-manifest-interoperability/001-release-manifest-v1-adapter.md | plans/closure/eggpack-manifest-interoperability/001-status.md | post-closure qualification/scope findings tracked by M001a |
| M001a adapter qualification + closure hardening | closed | plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md | plans/closure/eggpack-manifest-interoperability/001a-status.md | — |
| M002 archive extraction handoff | closed with hosted qualification | `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md` | `plans/closure/eggpack-manifest-interoperability/002-status.md` (M002a addendum) | — |
| M002a hosted qualification + clippy corrective | closed | `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md` | `plans/closure/eggpack-manifest-interoperability/002a-status.md` | — (was: current head failed Stable clippy in run `36463041223`; superseded by green run `36477024102`) |
| M003a caller-bound destination policy corrective | closed | `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md` | `plans/closure/eggpack-manifest-interoperability/003a-status.md` | — (was: hard dependency for M003; closed via `39ff626` + hosted run `36890986000`) |
| M003 Eggsact real-consumer manifest adoption | closed | plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md | plans/closure/eggpack-manifest-interoperability/003-status.md | — (closed via consumer `65c916b` + hosted CI `36902758482` and drift `36902758396`) |
| M004a package/API promotion readiness preflight | closed | `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md` | `plans/closure/eggpack-manifest-interoperability/004a-status.md` | M004 has since closed on the proven set below; M004a itself published nothing; its Eggpack-owned item was satisfied — `eggpack-manifest 0.1.0` published 2026-10-02 |
| M004 package/API promotion | closed | `plans/implementation/eggpack-manifest-interoperability/004-registry-package-api-promotion-and-publication.md` | `plans/closure/eggpack-manifest-interoperability/004-status.md` | published `eggup-acquisition`/`eggup-eggfetch`/`eggup-eggpack` 0.1.2 from `02a1d32`; hosted run `37090397398` green; registry-only adapter-only and Eggsact-shaped proofs green; `v0.1.1`/`v0.1.2` unmoved |
| M005 eggup-eggpack 0.1.3 security/correctness publication | closed 2026-10-06 | `plans/implementation/eggpack-manifest-interoperability/005-eggup-eggpack-0.1.3-security-publication.md` | `plans/closure/eggpack-manifest-interoperability/005-status.md` | published from `bd43683` (run `37527706900` green); member-identity cross-check ships; the `eggup-eggpack 0.1.2` acquisition-pin caveat is now closed |
