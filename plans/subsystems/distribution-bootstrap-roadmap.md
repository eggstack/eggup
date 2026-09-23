# Distribution and Bootstrap Roadmap

Status: migration-only; M001-M003 closed historical predecessor, M004 blocked on Eggpack Contract M002 closure

Long-term references:

- `plans/000-long-term-specification.md#44-producer-manifest-interoperability`
- `plans/000-long-term-specification.md#14-bootstrap-installers`
- `plans/002-long-term-roadmap.md#phase-9--eggpack-authority-cutover-and-manifest-interoperability`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

## 1. Purpose and ownership boundary

This roadmap now exists only to preserve Eggup's qualified predecessor evidence and complete authority transfer to Eggpack.

Eggup distribution M001-M003 proved the portable DistributionContract v1 and deterministic conformance surface while Eggup was the only shared update/distribution project. Eggpack now owns the producer domain. No new producer capability is authorized here.

The durable split is:

```text
Eggpack: contract -> conformance -> build/qualify -> package -> manifest -> installer/CI/release
Eggup:   acquire -> verify -> validate candidate -> local transaction -> service lifecycle -> receipt
Application: release/version/origin/fallback policy
```

## 2. Historical capabilities

Closed Eggup predecessor work includes:

- DistributionContract schema v1;
- strict template grammar and target/filename collision rules;
- direct, bundle, and archive layout expansion;
- expected release-file derivation;
- bounded release inventories;
- bounded archive-member inventories;
- observed target mappings;
- deterministic conformance reports.

These capabilities are migration evidence for Eggpack, not a basis for further Eggup producer work.

## 3. Explicitly transferred work

The following belong exclusively to Eggpack after ADR-0004:

- DistributionContract evolution;
- release/archive/mapping conformance;
- target/asset/checksum/install naming authority;
- package/archive/bundle construction;
- final release digests and ReleaseManifest production;
- bootstrap-installer generation/conformance;
- generated release CI;
- release staging/publication;
- producer provenance/SBOM/attestation generation.

Eggup MUST NOT author a replacement M004 generator milestone.

## 4. Current state evidence

Eggup distribution milestones:

- M001 implementation `889a234cbe7f461d92def3df45c83c06a7d257e5`;
- M002 corrective `0a68f29fce44adf5f12d79f1b440a2c08aca9cb7`;
- M003 implementation `9941c58d7039410c728860f9e4e382881d4ccf54`;
- M003 closure `4169c8021b447fe73c8ee3ea71a80a535c940f54`.

`eggup-dist` remains unpublished. Runtime Eggup crates do not depend on it.

Eggpack Contract M001 is closed. Eggpack Contract M002 is responsible for porting and independently qualifying the full closed M003 conformance behavior.

## 5. Target architecture

```text
                producer
                  |
                  v
          eggpack-contract
                  |
            eggpack-manifest
                  |
          release hosting
                  |
==================|==================
                  |
              consumer
                  |
     application release policy
                  |
             acquisition
                  |
      optional eggup-eggpack
                  |
             eggup-core
                  |
       local install / rollback
```

There is no active `eggup-dist` node in the end state.

## 6. Dependency graph

```text
Eggup distribution M001 [closed]
          |
          v
Eggup distribution M002 [closed]
          |
          v
Eggup distribution M003 [closed]
          |
          +-----------------------------+
                                        |
                                        v
                          Eggpack Contract M001 [closed]
                                        |
                                        v
                          Eggpack Contract M002 [required]
                                        |
                                        v
                          Eggup distribution M004
                          retire duplicate authority
                                        |
                                        v
                                  subsystem archived

Eggpack ReleaseManifest v1 [future]
              |
              v
optional Eggup manifest-consumer adapter [separate future roadmap/plan]
```

## 7. Milestones

### M001 — Versioned DistributionContract schema

Closed historical predecessor.

Plan: `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md`.

### M002 — Schema uniqueness and template grammar corrective

Closed historical predecessor.

Plan: `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md`.

### M003 — Release/archive/mapping conformance validators

Closed terminal producer-side implementation in Eggup.

Plan: `plans/implementation/distribution-bootstrap/003-release-installer-conformance-validators.md`.

Closure: `plans/closure/distribution-bootstrap/003-status.md`.

### M004 — Retire Eggup producer authority

Plan: `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`.

Status: blocked.

Hard dependency: Eggpack Contract M002 must close with equivalent schema-v1/conformance behavior and direct/bundle/archive evidence.

Objective: remove the unpublished `eggup-dist` workspace member, preserve historical evidence, and archive this subsystem.

This milestone does not generate installers or add producer functionality.

## 8. Cross-cutting requirements

- Do not delete predecessor evidence before Eggpack qualification is recorded.
- Do not evolve schema-v1 independently in Eggup.
- Do not create another runtime target/asset authority in Eggup.
- A future Eggup manifest adapter must depend only on a narrow stable producer data contract, never Eggpack build/CI machinery.
- Archive extraction for local installation remains an Eggup deployment concern even though archive construction/member declaration belongs to Eggpack.
- Eggpack computes release digests; Eggup verifies acquired bytes. Both may use SHA-256 without sharing policy ownership.

## 9. Verification strategy

M004 requires:

- exact Eggpack M002 closure reference;
- predecessor-to-Eggpack public behavior/fixture comparison;
- dependency search for `eggup-dist`;
- full remaining Eggup stable/MSRV/package/doc qualification;
- proof that active Eggup planning no longer authorizes producer distribution work.

## 10. Risks and decision points

The primary risk is deleting the predecessor before Eggpack M002 proves equivalence. The inverse risk is leaving both implementations active long enough for semantic drift.

Therefore `eggup-dist` is frozen during migration: correctness/security fixes only, mirrored or accounted for in Eggpack before cutover.

## 11. Completion definition

This subsystem is complete when Eggpack is the sole active owner of DistributionContract/conformance, `eggup-dist` is removed from the Eggup workspace, historical evidence remains traceable, and all future producer release/installer work is planned in Eggpack.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed / historical | `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md` | `plans/closure/distribution-bootstrap/001-status.md` | — |
| M002 | closed / historical | `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md` | `plans/closure/distribution-bootstrap/002-status.md` | — |
| M003 | closed / terminal predecessor | `plans/implementation/distribution-bootstrap/003-release-installer-conformance-validators.md` | `plans/closure/distribution-bootstrap/003-status.md` | — |
| M004 | blocked / retirement | `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md` | — | Eggpack Contract M002 closure |
