# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `8d5fc12f7224145285f22d0975a7bb91e1e363ea`

Planning baseline for this implementation-plan batch: `66acd739f792437cb9fa1701b8fa456c4ba4c403`

Latest closure/planning baseline reviewed: `2cab1f97ef30fa347c2030da321462459672c521` (Verified Update Core M007 closure state; C001 corrective state lands with this registry update)

This file is the compact control surface for active Eggup planning. Detailed requirements live in the linked plans and roadmaps.

## Canonical documents

| Document | Status |
|---|---|
| `plans/000-long-term-specification.md` | normative |
| `plans/001-terminology-and-domain-model.md` | normative |
| `plans/002-long-term-roadmap.md` | active |
| `plans/003-planning-process.md` | normative |

## Accepted architecture decisions

| ADR | Decision |
|---|---|
| `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md` | core/acquisition/service remain layered; original producer-distribution ownership is superseded by ADR-0004 |
| `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet is the mutation unit; rollback and post-commit policy are explicit |
| `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md` | integrity/authenticity/candidate identity are distinct; core has no mandatory transport |
| `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md` | Eggpack owns producer release contracts/construction/evidence; Eggup owns local deployment; applications own release/install policy |

## Producer/consumer ownership guard

Eggup owns consumer-side acquisition, verification, candidate validation, local extraction required for installation, ownership/staging/locking/commit/rollback/recovery, install receipts, and service lifecycle. Producer release contracts, release conformance, package/archive construction, final release manifests, bootstrap-installer generation, generated release CI, staging/publication, and producer provenance belong to `eggstack/eggpack`.

`eggup-core` must remain usable without Eggpack. A future optional manifest adapter may translate stable Eggpack release evidence into Eggup deployment inputs without importing producer build/CI machinery. Adapter plan authoring is currently blocked on Eggpack interoperability corrective M001a (`eggstack/eggpack`, plan `plans/implementation/eggup-interoperability/001a-projection-fixture-consistency-corrective.md`, registered from commit `d70bdcb0e56e70afdd430095a4010fe8caccb22e`), because the historical M001 bundle projection fixture did not match its paired manifest.

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M006 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M004 | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M004 | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap (archived/transferred) | M001-M004 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch | `plans/closure/consumer-adoption/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.0 crates remain:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

`eggup-dist` was unpublished and existed only as migration predecessor evidence. It was removed after Eggpack Contract M002 qualified the complete M003 behavior; see `plans/closure/distribution-bootstrap/004-status.md`.

Existing simple consumers:

- eggsact at `eggstack/eggsact@576f4b0`;
- stegoeggo at `eggstack/stegoeggo@10d8448`.

## Post-closure review findings

The core remains qualified with no newly identified medium-or-higher defect.

Corrective work identified after reviewing implementation SHA `889a234c` has
been closed sequentially; closure records capture verification evidence.

### Acquisition

M004 closed the remaining M003 review findings. Public `FetchLimits` values
are validated at every transport boundary, and post-link temp cleanup failure
can no longer report ordinary failure after a complete destination exists.
Details and environment limits are in
`plans/closure/acquisition-transport/004-status.md`.

### Service

Service M003 closed the post-M002 restart, deadline, executable/environment,
and config-identity findings. See
`plans/closure/service-lifecycle/003-status.md` for evidence and platform
limits.

### Distribution

M001-M003 are closed predecessor work. M003 is the terminal Eggup producer-side implementation and provides the conformance behavior Eggpack Contract M002 ported and independently qualified. The implementation is `9941c58d7039410c728860f9e4e382881d4ccf54`; closure is `4169c8021b447fe73c8ee3ea71a80a535c940f54`.

Distribution M004 removed `eggup-dist` after Eggpack Contract M002 closed at `82f799f3d971b2999ac14c2d8fc1b965370e0f58`; see `plans/closure/distribution-bootstrap/004-status.md`. The subsystem is archived/transferred, with no active Eggup producer-distribution milestone.

Eggsearch M003 and service M004 Windows SCM are closed. Verified Update Core M007 has closed the deferred-finalization boundary required by ADR-0002. Service M005 is ready for plan authoring against `ValidatedTransaction::commit_with_post_commit`; Gregg remains gated on its corrected-path footprint evidence.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | M001-M007 closed/qualified | — |
| Acquisition transport | corrected | M004 closed; optional M005 footprint evidence |
| Service lifecycle | M004 Windows SCM closed | M005 ready for plan authoring |
| Distribution/bootstrap | archived/transferred; M001-M004 closed | no further Eggup producer work |
| Eggpack manifest interoperability | blocked cross-repo gate | optional adapter plan intentionally unwritten pending Eggpack M001a closure |
| Consumer adoption | simple, eggsearch, and CodeGG M005 closed | Gregg remains footprint-gated |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Planning/closure hygiene corrective | C001 post-batch status and evidence reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/001-status.md` | CodeGG M005 + core M007 closed |
| Planning/closure hygiene corrective | C002 C001 commit + registry baseline reconciliation | ready | `plans/implementation/planning-closure-hygiene-corrective/002-c001-commit-and-registry-baseline-reconciliation.md` | C001 closed; docs-only; no runtime blocker |
| Verified update core | M007 post-commit policy / deferred finalization | closed | `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`; `plans/closure/verified-update-core/007-status.md` | — |
| Consumer adoption | M005 CodeGG managed-runfile bundle | closed | `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md` | `plans/closure/consumer-adoption/005-status.md` |
| Service lifecycle | M005 update-lifecycle integration | ready for plan authoring | — | core M007 closed; C001 closure hygiene complete |
| Distribution/bootstrap | M004 retirement | closed | `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`; `plans/closure/distribution-bootstrap/004-status.md` | — |
| Eggpack manifest interoperability | optional `eggup-eggpack` adapter | blocked / plan intentionally unwritten | — | Eggpack Interop M001a corrective closure + corrected fixture baseline |

CodeGG M005 and Verified Update Core M007 are closed. Planning/closure hygiene C001 is closed. C002 is a narrow docs-only bookkeeping corrective for C001's exact landing SHA and registry baseline semantics; it does not block Service Lifecycle M005 API planning. Service Lifecycle M005 is dependency-ready against the qualified core seam and is now unblocked for plan authoring. There is no dependency-ready producer-distribution implementation work in Eggup; that subsystem is archived/transferred.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M005 lightweight/curl adapter | deferred/evidence-driven | corrected-path footprint evidence, especially Gregg |
| Planning/closure hygiene corrective | C001 post-batch status/evidence reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/001-status.md` |
| Planning/closure hygiene corrective | C002 final commit/baseline bookkeeping | ready | C001 exact landing SHA + registry baseline semantics |
| Service lifecycle | M005 update-lifecycle integration | ready for plan authoring | core M007 closed; C001 closure hygiene complete; C002 is docs-only and not a runtime blocker |
| Distribution/bootstrap | M004 retire `eggup-dist` | closed | `plans/closure/distribution-bootstrap/004-status.md` |
| Consumer adoption | M004 Gregg | blocked / plan intentionally unwritten | corrected-path footprint decision |
| Consumer adoption | M005 CodeGG | closed | `plans/closure/consumer-adoption/005-status.md`; producer release mapping remains application/Eggpack-owned |
| Consumer adoption | M006 Egress | blocked | archive update/extraction transaction contract |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Eggpack manifest interoperability | optional adapter | blocked / plan intentionally unwritten | Eggpack Interop M001a corrective closure and corrected direct/bundle/archive projection evidence |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M004 [closed] --+
                            +--> eggsearch M003 [closed]
service M003 [closed] ------+          |
                                       +--> service-aware adoption evidence recorded

service M003 [closed] ----------> service M004 Windows SCM [closed]
                                      |
                                      +--------------------------+
                                                                 |
core M006 [closed] --> core M007 post-commit policy [CLOSED] ---+
                                                                 |
CodeGG M005 [closed] ---------------------------------------------+--> planning/closure hygiene C001 [CLOSED]
                                                                           |
                                                                           `--> service M005 plan authoring [READY FOR AUTHORING]

distribution M003 [closed/frozen] ---> Eggpack Contract M002 [closed]
                                              |
                                              `--> distribution M004 retirement [closed; subsystem archived]

core M006 + acquisition M004 [closed] --> CodeGG M005 managed bundle [CLOSED]
                                            |
                                            `--> producer release mapping stays in application/Eggpack; strict archive extraction stays CodeGG-owned

Egress consumer adoption remains blocked on local archive extraction/update transaction semantics.

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

Gregg M004 remains separate: corrected Eggfetch footprint measurement -> adopt directly
                                                            \-> acquisition M005 only if justified
```

The runtime corrective gates are closed. Eggsearch M003, distribution M003/M004, service M004, CodeGG M005, and Verified Update Core M007 have reviewed closure evidence. Planning/closure hygiene C001 is closed after reconciling the remaining low-severity status/measurement drift. Service M005 is ready against the qualified core rollback seam and is unblocked for plan authoring. Future Eggpack manifest consumption is specifically blocked on Eggpack Interop M001a correcting and mechanically validating the direct/bundle/archive projection fixtures; no Eggup adapter plan should be authored before that closure. No Eggup installer-generator replacement is authorized.

## Current project state

- Rust baseline: 1.89.
- Core: M001-M007 are qualified; M007 adds the accepted post-commit `KeepInstalled | RollBack` boundary without service coupling. Closure: `plans/closure/verified-update-core/007-status.md`.
- Acquisition: M004 validation/promotion corrective is closed; optional M005 still needs corrected-path footprint evidence.
- Service: M004 Windows SCM is closed; M005 update-lifecycle integration is ready for plan authoring now that planning/closure hygiene C001 has closed.
- Distribution: M001-M003 remain historical predecessor evidence; M004 removed the producer crate after Eggpack Contract M002 closure. The subsystem is archived/transferred to Eggpack.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg still needs its own footprint decision.
- CodeGG M005 closed using CodeGG-owned strict extraction of the existing verified archive and Eggup's multi-artifact transaction; it did not recreate producer release authority in Eggup. Evidence: `plans/closure/consumer-adoption/005-status.md`. Egress remains separately blocked on a generic archive transaction/extraction contract.
- Release process: manual crates.io publication only.
- The previously selected lockstep 0.1.1 patch is now eligible for separate qualification/publication when directed because the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until that release exists; no publication is implicit in these plans.

## Next handoff

The requested implementation batch is complete: CodeGG M005 and Verified Update Core M007 are closed at `plans/closure/consumer-adoption/005-status.md` and `plans/closure/verified-update-core/007-status.md`, and planning/closure hygiene C001 is closed at `plans/closure/planning-closure-hygiene-corrective/001-status.md`. The immediate handoff is authoring Service Lifecycle M005 against the stabilized post-commit API.

Do not author or implement an Eggup installer generator. A future consumer manifest adapter remains a separate, optional milestone and is currently blocked on Eggpack Interop M001a closure and its corrected fixture baseline. Do not author the Eggup adapter implementation plan until that external gate closes.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
