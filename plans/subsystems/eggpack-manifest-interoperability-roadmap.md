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

The previous Eggup planning block on Eggpack M001a is satisfied. Eggup adapter M001 then implemented and closed historically, but post-closure review found an incomplete adapter regression matrix and an out-of-scope service-manifest edit in the implementation commit. Corrective M001a has since closed with the full regression matrix and service packageability reconciliation. M003 bounded adapter/API qualification is complete, but real-consumer adoption is blocked: Eggsact's pinned release workflow publishes unversioned `eggsact-{target}` artifacts plus `.sha256` sidecars and no ReleaseManifest; the current Eggpack Eggsact fixture uses version-qualified artifacts; and no adopted Eggsact manifest publication convention or producer contract is present. See `plans/closure/eggpack-manifest-interoperability/003-status.md`. Eggup must not invent producer naming or silently change release assets.

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

- eggpack-manifest is currently an unpublished 0.1.0 workspace crate. Initial adapter qualification may use an exact Git revision and remain publish = false; publication promotion is a separate milestone/action.

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

Only after Phase 10 establishes the generic/safe archive extraction contract, connect archive member evidence to qualified extracted local files and ArtifactSet construction.

### M003 — Eggsact real-consumer manifest adoption

Selected consumer: Eggsact, the already-qualified direct single-binary Eggup adopter.

Implementation plan: plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md.

Adopt the adapter in Eggsact's real updater path so producer-valid ReleaseManifest evidence can replace duplicated manifest-to-update mapping while Eggsact retains release/version/origin/fallback/install/candidate policy.

The producer-evidence gate was inspected and remains unmet. The bounded JSON parse/project API and its tests are implemented in Eggup, but M003 is not closed: the normal Eggsact updater has no manifest path, and producer-owned artifact/manifest publication conventions remain unestablished. Resume only after Eggpack/Eggsact evidence resolves both contracts. M003 must not change release naming or invent a manifest filename inside Eggup.

### M004 — Package/API promotion

If adoption justifies the crate and eggpack-manifest has a publishable stable version, replace the immutable Git dependency with a compatible registry dependency, qualify packaging, and decide whether eggup-eggpack should join Eggup's published lockstep crates. M004 remains blocked until M003 real-consumer adoption closes and a publishable eggpack-manifest version exists.

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

A second risk is premature publication coupling while eggpack-manifest is unpublished. M001 therefore pins an immutable Git revision and keeps the adapter unpublished. Do not vendor or reimplement the Manifest parser to avoid this constraint.

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
| M002 archive extraction handoff | blocked | — | — | Phase 10 archive extraction contract |
| M003 Eggsact real-consumer manifest adoption | blocked after bounded adapter/API qualification | plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md | plans/closure/eggpack-manifest-interoperability/003-status.md | producer-owned Eggsact artifact mapping and ReleaseManifest publication convention are absent |
| M004 package/API promotion | blocked | — | — | M003 real adoption must close and eggpack-manifest must have a publishable version |
