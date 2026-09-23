# ADR-0004: Eggpack producer / Eggup consumer distribution boundary

Status: accepted

Date: 2026-09-22

Decision owners: project maintainers

Related specifications:

- `plans/000-long-term-specification.md#3-mechanism-versus-policy`
- `plans/000-long-term-specification.md#4-crate-and-layer-model`
- `eggstack/eggpack:plans/000-long-term-specification.md#4-producerconsumer-ownership-rule`

Affected roadmaps:

- distribution/bootstrap;
- consumer adoption;
- future Eggpack interoperability.

## Context

Eggup originally included `eggup-dist` so release-layout schema and bootstrap/runtime mapping drift could be addressed while the deployment substrate was being established. That work reached a useful, qualified predecessor state through distribution M003: schema v1, strict template/collision rules, expected release inventories, archive-member inventories, observed mappings, and deterministic conformance reports.

Eggpack now exists specifically to own developer-side release construction and release evidence. Keeping future contract, conformance, installer-generation, packaging, manifest, and CI work in both repositories would create two authorities for the same producer domain.

## Decision

The durable ownership rule is:

**Eggpack owns producer release contracts, release construction, and release evidence. Eggup owns consumer-machine deployment mechanism. Applications own release-selection and installation policy.**

### Eggpack owns

- DistributionContract schema and target/artifact naming authority;
- release/archive/mapping conformance;
- build and qualification planning;
- package/archive/bundle construction;
- final artifact size/digest production;
- ReleaseManifest formats and generation;
- bootstrap-installer generation/conformance;
- generated release CI;
- release staging/publication adapters;
- producer provenance/SBOM/attestation generation.

### Eggup owns

- bounded artifact and metadata acquisition;
- integrity/authenticity verification of acquired inputs;
- candidate identity validation;
- safe archive extraction when extraction is part of local installation;
- destination ownership and replacement checks;
- local staging, locking, commit, rollback, recovery, and install receipts;
- service-manager registration and update lifecycle coordination;
- optional narrow adapters that translate producer manifests into Eggup deployment inputs.

### Applications own

- release authority and version ordering;
- whether/when to update;
- release-origin policy;
- fallback policy;
- privilege decisions;
- application migrations and health semantics.

## Eggup-dist disposition

`eggup-dist` M003 is the terminal Eggup producer-side implementation. It is frozen except for correctness/security fixes needed to preserve migration evidence.

Eggpack Contract M002 must port and independently qualify the closed M003 conformance surface rather than redesigning or independently re-deriving it.

After Eggpack Contract M002 closes with equivalent schema/conformance behavior, Eggup distribution M004 removes `eggup-dist` from the active workspace and supersedes future Eggup distribution/bootstrap work. Historical plans and closure records remain for provenance.

No Eggup installer-generator milestone follows M003.

## Interoperability seam

The preferred producer/consumer seam is a small, versioned Eggpack release manifest:

```text
Eggpack producer
  contract -> build -> qualify -> finalize -> ReleaseManifest
                                                |
                                                v
application release policy -> acquire -> optional eggup-eggpack adapter
                                                |
                                                v
                                       Eggup ArtifactSet
                                                |
                                                v
                              verify -> install -> rollback/receipt
```

`eggup-core` MUST NOT depend on Eggpack. A future optional `eggup-eggpack` adapter MAY depend on a small manifest-only Eggpack crate, but MUST NOT pull Eggpack build/CI tooling into runtime consumers.

Eggup must remain usable with non-Eggpack release systems, and Eggpack must remain usable for products that do not use Eggup.

## Archive boundary

Archive construction and release-member declaration are producer concerns owned by Eggpack.

Archive extraction onto a consumer machine is a deployment concern owned by Eggup or a consumer-supplied extraction adapter. Producer tooling MUST NOT own live destination mutation.

## Checksum boundary

Eggpack computes and records digests over finalized release bytes.

Eggup computes/verifies digests over acquired bytes against caller-supplied or manifest-supplied expected evidence.

This deliberate duplication of the hash primitive does not create competing policy authority.

## Bootstrap installers

Bootstrap installers are producer-generated release artifacts and therefore belong to Eggpack. They may contain the minimal consumer-side mechanics required before an Eggup-linked application exists, but they are not a second general update engine.

Normal in-process update after installation should use Eggup.

## Compatibility and migration

1. Preserve all existing Eggup M001-M003 distribution closure evidence.
2. Port the closed M003 public behavior/tests into Eggpack Contract M002.
3. Close Eggpack M002 with differential/golden evidence against the Eggup predecessor.
4. Execute Eggup distribution M004 to remove the unpublished `eggup-dist` workspace member and update canonical docs.
5. Continue installer generation, manifests, build/package, CI, and producer adoption only in Eggpack.
6. Add Eggup manifest interoperability only after Eggpack ReleaseManifest v1 is stable enough to consume.

## Consequences

The repositories have one authority per lifecycle domain. Eggup's runtime dependency surface stays small; Eggpack can evolve build/release tooling without contaminating deployment code; cross-repository compatibility is concentrated in versioned data contracts rather than mutual core dependencies.

The cost is an explicit migration/cutover milestone and cross-repository compatibility tests.

## Verification

The boundary is considered enforced when:

- Eggpack owns the only active DistributionContract/conformance implementation;
- `eggup-dist` is absent from the active Eggup workspace;
- no Eggup roadmap authorizes installer generation, build/package orchestration, release CI, or publication;
- `eggup-core` has no Eggpack dependency;
- an eventual Eggup adapter can consume an Eggpack manifest without importing producer build machinery;
- Eggup still supports explicitly described non-Eggpack artifact sets.

## Supersession

This ADR supersedes the distribution-ownership portion of ADR-0001. ADR-0001 remains authoritative for Eggup's core/acquisition/service layering and consumer-policy separation.
