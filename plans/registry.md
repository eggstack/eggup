# Eggup Planning Registry

Status: active

Last implementation/closure baseline reviewed: `e7a85709d79df5d2e673e1c3720f7b2af098b244` (Acquisition M007 implementation head; hosted matrix green on run `36213509807`; Windows loopback curl evidence unavailable as recorded in M007 closure)

Latest reviewed pre-C003 planning/status baseline: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` (M003 producer-gate execution/status record)

Latest planning registration head: `9f99e9cbff7d65f6dce773c1977dd3c91274dc9f` (ADR-0005 + archive extraction M001 + acquisition/service M007 registration; planning-only, implementation baselines remain `ee1476ef2a9e0d5569d6dc2469e780a4435cc426`)

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
| `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md` | Local archive extraction is optional Eggup deployment machinery: already-verified archives, explicit member allowlists, finite decompression budgets, regular-file-only/no-clobber private extraction, no live mutation |

## Producer/consumer ownership guard

Eggup owns consumer-side acquisition, verification, candidate validation, local extraction required for installation, ownership/staging/locking/commit/rollback/recovery, install receipts, and service lifecycle. Producer release contracts, release conformance, package/archive construction, final release manifests, bootstrap-installer generation, generated release CI, staging/publication, and producer provenance belong to `eggstack/eggpack`.

`eggup-core` must remain usable without Eggpack. The optional manifest adapter translates stable Eggpack release evidence into Eggup deployment inputs without importing producer build/CI machinery. Eggpack interoperability M001a is closed at `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41`; Eggup adapter M001 is historically closed and M001a qualification/closure hardening is closed at `plans/closure/eggpack-manifest-interoperability/001a-status.md`. Bounded Eggup adapter/API qualification for M003 is complete, but real Eggsact adoption is blocked on producer-owned live artifact mapping and ReleaseManifest publication/addressing; resume M003 only after that evidence exists.

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M007 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M006 closed; M007 conditionally closed | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M006 closed; hosted evidence reconciled via acquisition M006 | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap (archived/transferred) | M001-M004 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch | `plans/closure/consumer-adoption/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.1 crates (lockstep patch superseding 0.1.0):

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

Corrective work identified after reviewing implementation SHA `889a234c` was closed sequentially. A new 2026-09-26 review at code baseline `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` opened acquisition M007 and service M007. Acquisition M007 is conditionally closed at `plans/closure/acquisition-transport/007-status.md`; Service M007 remains ready. The same review promoted the long-standing Phase 10 archive blocker into accepted ADR-0005 plus Archive Extraction M001.

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

Eggsearch M003, Service M004, Verified Update Core M007, Service M005, Acquisition M005/M006, and Service M006 are closed. Read-only review of `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` justified both upstream milestones without authorizing a Gregg migration. Gregg consumer M004 prerequisites are satisfied (green hosted matrix on corrective run `36176009068`); its plan remains intentionally unwritten pending a separate authoring decision.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | M001-M007 closed/qualified | — |
| Acquisition transport | M001-M006 closed; M007 conditionally closed | boundary-safety corrective |
| Service lifecycle | M001-M006 closed; M007 ready | UTF-8-safe bounded-diagnostics corrective (next requested plan) |
| Archive extraction | M001 closed; handoffs ready to author | Egress and Eggpack archive handoff plans |
| Distribution/bootstrap | archived/transferred; M001-M004 closed | no further Eggup producer work |
| Eggpack manifest interoperability | M001/M001a closed; M002 ready to author; M003 waits on producer convention | author archive extraction handoff plan |
| Consumer adoption | simple, eggsearch, and CodeGG M005 closed; Egress M006 ready to author | write Egress archive adoption plan; Gregg M004 separately writable/unwritten |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Acquisition transport | M007 boundary safety hardening corrective | conditionally closed | `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`; `plans/closure/acquisition-transport/007-status.md` | M001-M006 closed; review baseline `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` |
| Service lifecycle | M007 UTF-8-safe bounded diagnostics corrective | ready | `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md` | M001-M006 closed; review baseline `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` |
| Archive extraction | M001 bounded allowlisted extraction contract | closed | `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`; `plans/closure/archive-extraction/001-status.md` | hosted Linux/macOS/Windows qualification `36214688691` |
| Consumer adoption | M006 Egress archive/pair adoption | ready to author | — | archive extraction M001 closed; write a bounded consumer plan |
| Eggpack manifest interoperability | M002 archive extraction handoff | ready to author | — | archive extraction M001 closed; write a bounded handoff plan |
| Planning/closure hygiene corrective | C001 post-batch status and evidence reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/001-status.md` | CodeGG M005 + core M007 closed |
| Planning/closure hygiene corrective | C002 C001 commit + registry baseline reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/002-c001-commit-and-registry-baseline-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/002-status.md` | — |
| Planning/closure hygiene corrective | C003 M003 registry + closure reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/003-m003-registry-and-closure-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/003-status.md` | bounded M003 execution/status record at `da1b4a8048bf863e6a653c25f1ba56bc42f4531b`; docs/evidence only |
| Verified update core | M007 post-commit policy / deferred finalization | closed | `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`; `plans/closure/verified-update-core/007-status.md` | — |
| Consumer adoption | M005 CodeGG managed-runfile bundle | closed | `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md` | `plans/closure/consumer-adoption/005-status.md` |
| Service lifecycle | M005 prepared-transaction lifecycle integration | closed | `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`; `plans/closure/service-lifecycle/005-status.md` | — |
| Acquisition transport | M005 curl adapter + explicit transport composition | closed/qualified with M006 | `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`; `plans/closure/acquisition-transport/005-status.md` (+ M006 addendum) | — |
| Acquisition transport | M006 M005 Windows portability + cross-closure qualification corrective | closed | `plans/implementation/acquisition-transport/006-m005-windows-portability-and-cross-closure-qualification-corrective.md`; `plans/closure/acquisition-transport/006-status.md` | green hosted matrix `36176009068` |
| Service lifecycle | M006 daemon update disposition + reference qualification | closed; evidence reconciled | `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`; `plans/closure/service-lifecycle/006-status.md` (+ M006 addendum) | — |
| Distribution/bootstrap | M004 retirement | closed | `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`; `plans/closure/distribution-bootstrap/004-status.md` | — |
| Eggpack manifest interoperability | M001 ReleaseManifest v1 direct/bundle adapter | closed (historical) | `plans/implementation/eggpack-manifest-interoperability/001-release-manifest-v1-adapter.md` | `plans/closure/eggpack-manifest-interoperability/001-status.md`; post-closure findings tracked by M001a |
| Eggpack manifest interoperability | M001a adapter qualification + closure hardening | closed | `plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md` | `plans/closure/eggpack-manifest-interoperability/001a-status.md`; full regression matrix qualified |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | blocked after bounded adapter/API qualification | `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`; `plans/closure/eggpack-manifest-interoperability/003-status.md` | Eggpack/Eggsact must establish producer-owned live artifact mapping and ReleaseManifest publication/addressing convention |
| Eggpack manifest interoperability | M004 package/API promotion | blocked | roadmap milestone; no implementation plan until ready | M003 adoption must close and eggpack-manifest needs a publishable version |

CodeGG M005, Verified Update Core M007, Service Lifecycle M005/M006, and Acquisition M005/M007 are closed (Acquisition M007 conditionally, due the recorded Windows loopback limit). Archive Extraction M001 is closed; Service Lifecycle M007 remains ready and is the next requested implementation. Archive M001 unblocks authoring Consumer Adoption M006 Egress and Eggpack Interoperability M002; neither migration is implemented or yet planned. Planning/closure hygiene C001, C002, and C003 are closed. Both Gregg-reference feature milestones are closed with green hosted qualification, and M005/M006 closure evidence is reconciled. Gregg M004 prerequisites are satisfied but its plan remains intentionally unwritten; no Gregg migration is authorized. Eggpack Interop M001a is closed with the full adapter regression matrix. M003 bounded parse/project qualification is complete, while real Eggsact updater integration is blocked on producer-owned artifact and manifest conventions; see `plans/closure/eggpack-manifest-interoperability/003-status.md`. M004 package/API promotion remains blocked. There is no dependency-ready producer-distribution implementation work in Eggup; that subsystem is archived/transferred.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M007 boundary safety hardening | conditionally closed | `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md` |
| Service lifecycle | M007 UTF-8-safe bounded diagnostics | ready | `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md` |
| Archive extraction | M001 bounded allowlisted extraction | closed | `plans/closure/archive-extraction/001-status.md` |
| Acquisition transport | M005 curl adapter + explicit transport composition | closed | `plans/closure/acquisition-transport/005-status.md` |
| Planning/closure hygiene corrective | C001 post-batch status/evidence reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/001-status.md` |
| Planning/closure hygiene corrective | C002 final commit/baseline bookkeeping | closed | `plans/closure/planning-closure-hygiene-corrective/002-status.md` |
| Planning/closure hygiene corrective | C003 M003 registry/closure reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/003-status.md` |
| Service lifecycle | M005 prepared-transaction lifecycle integration | closed | `plans/closure/service-lifecycle/005-status.md` |
| Service lifecycle | M006 daemon update disposition/reference qualification | closed | `plans/closure/service-lifecycle/006-status.md` |
| Distribution/bootstrap | M004 retire `eggup-dist` | closed | `plans/closure/distribution-bootstrap/004-status.md` |
| Consumer adoption | M004 Gregg | writable; plan intentionally unwritten | prerequisites satisfied (acquisition M005/M006 + service M006 closed, hosted matrix green); no migration authorized |
| Consumer adoption | M005 CodeGG | closed | `plans/closure/consumer-adoption/005-status.md`; producer release mapping remains application/Eggpack-owned |
| Consumer adoption | M006 Egress | ready to author | archive extraction M001 closed; bounded plan not yet written |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Eggpack manifest interoperability | M002 archive extraction handoff | ready to author | archive extraction M001 closed; bounded plan not yet written |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | blocked after bounded API qualification | producer-owned live artifact mapping and ReleaseManifest publication/addressing convention are absent; see `plans/closure/eggpack-manifest-interoperability/003-status.md` |
| Eggpack manifest interoperability | M004 package/API promotion | blocked | M003 real adoption closure + publishable eggpack-manifest version |
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
CodeGG M005 [closed] ---------------------------------------------+--> planning/closure hygiene C001 [CLOSED] --> C003 [CLOSED]
                                                                           |
                                                                           `--> service M005 implementation [CLOSED]

distribution M003 [closed/frozen] ---> Eggpack Contract M002 [closed]
                                              |
                                              `--> distribution M004 retirement [closed; subsystem archived]

Eggpack Interop M001a [CLOSED; corrected fixtures]
            |
            `--> eggup-eggpack adapter M001 [CLOSED HISTORICALLY]
                    |
                    v
              adapter M001a corrective [CLOSED]
                    |
                    v
              M003 bounded parse/API qualification [DONE]
                    |
                    `--> M003 real-consumer adoption [BLOCKED: producer artifact + manifest convention]
                              |
                              `--> M004 package/API promotion [BLOCKED: M003 + publishable upstream]

core M006 + acquisition M004 [closed] --> CodeGG M005 managed bundle [CLOSED]
                                            |
                                            `--> producer release mapping stays in application/Eggpack; strict archive extraction stays CodeGG-owned

Archive Extraction M001 [CLOSED; run 36214688691] --> Egress consumer M006 [READY TO AUTHOR]
                                                 `--> Eggpack interoperability M002 [READY TO AUTHOR]

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

review baseline ee1476 -> acquisition M007 boundary safety [CONDITIONALLY CLOSED; hosted run 36213509807]
                       `-> service M007 UTF-8 diagnostics [READY]

Gregg read-only reference evidence -> acquisition M005 curl/composition [CLOSED]
                                  \-> service M006 disposition/revalidation [CLOSED]
                                                |
                                                v
                                acquisition M006 Windows/closure corrective [CLOSED via run 36176009068]
                                                |
                                                `-> Gregg M004 [WRITABLE; plan intentionally unwritten]
```

The 2026-09-26 review opened acquisition M007 and service M007. Acquisition M007 is conditionally closed at `plans/closure/acquisition-transport/007-status.md`; Service M007 remains ready. Archive Extraction M001 closed at `plans/closure/archive-extraction/001-status.md` after hosted run 36214688691. Egress M006 and Eggpack interoperability M002 are ready for separate plan authoring; neither consumer change is part of M001. Eggsearch M003, distribution M003/M004, service M004/M005/M006, CodeGG M005, Verified Update Core M007, and Acquisition M005/M006 have reviewed closure evidence. Planning/closure hygiene C001-C003 are closed. Both Gregg-reference upstream mechanisms are closed with green hosted qualification; Gregg consumer migration is writable but not scheduled (plan intentionally unwritten). Eggpack manifest adapter/API work is structurally implemented and qualified, and adapter M001a has closed with the full regression matrix and truthful service packageability reconciliation. Historical M001 remains closed evidence; lower Eggup crates remain Eggpack-independent. M003 bounded JSON parse/project work is qualified, but the real Eggsact updater integration is blocked on producer convention evidence; M004 promotion is also blocked. No Eggup installer-generator replacement is authorized.

## Current project state

- Rust baseline: 1.89.
- Core: M001-M007 are qualified; M007 adds the accepted post-commit `KeepInstalled | RollBack` boundary without service coupling. Closure: `plans/closure/verified-update-core/007-status.md`.
- Acquisition: M001-M006 are closed; M007 is conditionally closed. Diagnostics are UTF-8 safe, artifact caps are mandatory finite values, and curl writes through Eggup's retained file handle. Windows acquisition fixtures and curl process tests pass; hosted curl loopback connections are unavailable, so Windows live-HTTP behavior is not claimed.
- Service: M001-M006 are closed; M007 is ready to make bounded diagnostics UTF-8 safe without changing the five-disposition/lifecycle semantics.
- Distribution: M001-M003 remain historical predecessor evidence; M004 removed the producer crate after Eggpack Contract M002 closure. The subsystem is archived/transferred to Eggpack.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg M004 prerequisites are satisfied (acquisition M005/M006 + service M006 closed, hosted matrix green) and the milestone is writable, but its plan remains intentionally unwritten pending a separate authoring decision.
- Archive extraction: ADR-0005 and M001 are closed; the bounded, allowlisted, regular-file-only tar.gz/zip extraction layer lives outside `eggup-core`. CodeGG remains historical consumer-owned extraction evidence; Egress M006 and Eggpack interoperability M002 are ready to author separate integration plans.
- Eggpack interoperability: M001 adapter/M001a qualification remain closed. M002 archive handoff is ready to author against the closed Archive Extraction M001 contract. M003 real Eggsact adoption remains separately blocked on producer-owned manifest publication/addressing; M004 promotion remains blocked behind M003 plus a publishable upstream crate.
- Release process: manual crates.io publication only.
- The lockstep 0.1.1 patch is published (seam-then-adapter order) now that the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until downstream adoption moves; no publication is implicit in these plans.

## Next handoff

Immediate handoff work is Service M007, following closure of Archive Extraction M001. Acquisition M007 is conditionally closed. Archive M001 made Egress M006 and Eggpack interoperability M002 ready for their own plan-authoring steps. Gregg M004 remains separately writable but intentionally unwritten; do not modify Gregg as part of these upstream passes. Eggsact manifest adoption remains blocked on Eggpack/Eggsact producer convention evidence.

Do not author or implement an Eggup installer generator. Keep producer behavior in Eggpack and archive extraction outside the adapter. Resume interoperability M003 only after producer-owned evidence resolves its gate.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
