# Eggup Planning Registry

Status: active

Last implementation/closure baseline reviewed: `4c8c0e05a01f8d22fe50b4d616621ee6c32cc9d1` (post-M004a roadmap registration head; pre-C011/M004a closure baseline)

Latest planning/closure head: `e937c3f` (C011 + M004a closed; see `plans/closure/planning-closure-hygiene-corrective/011-status.md` and `plans/closure/eggpack-manifest-interoperability/004a-status.md`)

Latest reviewed pre-C003 planning/status baseline: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` (M003 producer-gate execution/status record; historical)

Latest planning registration head: `adb2e7c9ad02a28adccf4a430225cc11619c67e7` (C011 + M004a registered; superseded by this closure pass)

Newest qualified Eggup runtime delta with fully green hosted qualification is M003a implementation `39ff62602e39b14b31f5a8f154905d3343034db7` (feat(eggpack): caller-bound destination binding; hosted run `36890986000` green on Stable/MSRV/macOS/Windows). No runtime commit landed after `39ff626` (`git log 39ff626..HEAD -- crates/` empty; `e336b32`/`538e3e5` are docs/closure commits). M003 Eggsact real-consumer adoption is closed at consumer `eggstack/eggsact@65c916ba3b0f02916203ec1aad09d7e5c023c278` on Eggup pin `e336b32` (hosted CI run `36902758482` + drift run `36902758396` green). The M009 release-prep head `e8e07eb538d0eef18ea4cb4ace3bb905c316da72` (run `36487099388` green) remains the provenance for the published `eggup-core`/`eggup-archive 0.1.2` pair, not the latest qualified runtime. Code qualification was established at `9a5500e` (M002a clippy + Windows-adapter head; run `36477024102` green, superseding failed run `36463041223`); Core M008a reran the exact clean-tree package/publish-dry-run evidence on the post-M002a tree with no material delta. Planning-hygiene C009 corrected the stale Consumer M006 claim: current `eggstack/eggress@03134f8` has both the updater surface and registered Delivery M003 plan. Eggup M009 published the compatible 0.1.2 pair on 2026-09-28; Egress M006 subsequently closed on 2026-09-29 after Egress Delivery M003 landed with hosted Linux/macOS/Windows updater evidence. Eggpack's own interoperability roadmap still reports M003 as ready-to-resume with no manifest-publication milestone (cross-repo drift, Eggpack-owned follow-up; not evidence M003 is open).

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

`eggup-core` must remain usable without Eggpack. The optional manifest adapter translates stable Eggpack release evidence into Eggup deployment inputs without importing producer build/CI machinery. Eggpack interoperability M001a is closed at `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41`; Eggup adapter M001 is historically closed and M001a qualification/closure hardening is closed at `plans/closure/eggpack-manifest-interoperability/001a-status.md`. Bounded Eggup adapter/API qualification for M003 is complete. The producer-evidence gate that previously stopped real Eggsact adoption has been resolved externally: Eggpack Ecosystem M001 and Eggsact Distribution M005 closed on the real `v1.2.7` producer contract (`release/eggpack/distribution.toml` authority for the unversioned asset names; `release-manifest.json` published alongside the binaries, checksum sidecars, and installers); the consumer-path half of M003 has now closed via consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32` (hosted CI run `36902758482` + drift run `36902758396` green; see `plans/closure/eggpack-manifest-interoperability/003-status.md`).

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M007 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M006 closed; M007 conditionally closed | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M008 closed | `plans/closure/service-lifecycle/` |
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

Corrective work identified after reviewing implementation SHA `889a234c` was closed sequentially. A 2026-09-26 review at code baseline `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` opened acquisition M007 and service M007; Acquisition M007 is conditionally closed and Service M007 is closed. Archive Extraction M001 is historical. A follow-up at `ea51fe12...` implemented Acquisition M008 and Archive M001a. Review of head `4fb9d8ca...` then found M001a still had a cleanup TOCTOU and stable-Windows compilation defect; Archive M001b corrected both and closed at `0573996` with green hosted run `36222536670`, which also closed M008 hosted qualification. Review of documentation head `d46d35e...` then found a distinct archive materialization-authority gap. M001c implemented handle-relative write authority (`09c953f`) and stopped under Section 14 on path-handoff truthfulness. M001d subsequently implemented the object-bound source handoff and closed that stop with final hosted run `36335233644`; M001b/M001c historical closure evidence remains valid.

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
| Verified update core | M001-M009 closed; 0.1.2 core/archive pair published to crates.io | — |
| Acquisition transport | M001-M006 closed; M007 conditionally closed; M008 closed | — |
| Service lifecycle | M001-M008 closed | — |
| Archive extraction | M001/M001a historical; M001b closed; M001c historical predecessor with stop resolved by M001d; M001d closed | — |
| Distribution/bootstrap | archived/transferred; M001-M004 closed | no further Eggup producer work |
| Eggpack manifest interoperability | M001/M001a/M002/M002a/M003a/M003/M004a closed; bounded JSON adapter qualification, producer convention gate, caller-bound destination seam, real-consumer adoption, and package/API promotion readiness satisfied | M004 blocked: first close registered Eggpack Release Manifest M003 (`plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md`) with exact `eggpack-manifest =0.1.0` registry resolution, then Eggup `acquisition`/`eggfetch`/`eggpack` 0.1.2 |
| Consumer adoption | simple, eggsearch, CodeGG M005, and Egress M006 closed; C009 evidence reconciliation closed | Gregg M004 separately unwritten (intentionally); EggPool M007 deferred |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Planning/closure hygiene corrective | C011 post-M003 closure + registration reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/011-post-m003-closure-and-registration-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/011-status.md` | docs-only; M003a/M003 closures authoritative; M004a closed concurrently |
| Eggpack manifest interoperability | M004a package/API promotion readiness preflight | closed | `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md`; `plans/closure/eggpack-manifest-interoperability/004a-status.md` | M003a/M003 closed; qualification only, no publication occurred; M004 blocked on proven prerequisites |
| Archive extraction | M001c handle-relative member materialization corrective | historical predecessor; stop resolved | `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`; `plans/closure/archive-extraction/001c-status.md` | resolved by M001d closure |
| Service lifecycle | M008 OperationDeadline test determinism corrective | closed | `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`; `plans/closure/service-lifecycle/008-status.md` | hosted run `36332823865` green on all lanes (supersedes `36261671380`); M001-M007 closed |
| Planning/closure hygiene corrective | C007 C006 head + current-CI reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/007-c006-head-and-current-ci-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/007-status.md` | docs-only; reconciled C006 head, current-CI evidence, M008 registration, M001d gate |
| Planning/closure hygiene corrective | C006 M001d readiness + bound-source contract reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/006-m001d-readiness-and-bound-source-contract-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/006-status.md` | docs-only; corrected M001d contract |
| Archive extraction | M001d handle-backed source handoff | closed | `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`; `plans/closure/archive-extraction/001d-status.md` | hosted run `36335233644` green on all lanes (supersedes `36334772510`); M001c stop closed |
| Planning/closure hygiene corrective | C008 post-M001d status/handoff reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/008-post-m001d-status-and-handoff-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/008-status.md` | docs-only; active M001c stop wording reconciled |
| Eggpack manifest interoperability | M002a hosted qualification + clippy corrective | closed | `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md`; `plans/closure/eggpack-manifest-interoperability/002a-status.md` | hosted run `36477024102` green on all lanes (supersedes failed `36463041223`); direct Windows adapter runtime evidence collected |
| Planning/closure hygiene corrective | C009 Consumer M006 Egress evidence reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/009-m006-egress-evidence-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/009-status.md` | docs-only; Egress updater + Delivery M003 evidence confirmed at `03134f8`; M006 record corrected |
| Planning/closure hygiene corrective | C010 post-Egress/M003 active-status reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/010-post-egress-and-m003-readiness-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/010-status.md` | docs-only; reconciled archive-roadmap Egress closure, registry M003 producer-gate resolution, and M003 plan stale dependency prose |
| Verified update core | M008a clean package/current-head evidence corrective | closed | `plans/implementation/verified-update-core/008a-clean-package-evidence-corrective.md`; `plans/closure/verified-update-core/008a-status.md` | exact clean-tree package/dry-run + fixture green; no material delta from provisional evidence |
| Verified update core | M009 eggup-core / eggup-archive 0.1.2 publication | closed 2026-09-28 | `plans/implementation/verified-update-core/009-core-archive-0.1.2-publication.md`; `plans/closure/verified-update-core/009-status.md` | core then archive published from `e8e07eb`; registry-only smoke 3/3; `v0.1.2` + GitHub Release `0.1.2`; manual only, no release CI added |
| Verified update core | M008 core/archive consumer package qualification | closed with clean evidence (M008a); publication gate satisfied by M009 | `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md`; `plans/closure/verified-update-core/008-status.md` (M008a addendum) | — |
| Eggpack manifest interoperability | M002 archive extraction handoff | closed with hosted qualification (M002a) | `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`; `plans/closure/eggpack-manifest-interoperability/002-status.md` (M002a addendum) | — |
| Consumer adoption | M006 Egress archive/pair adoption | closed 2026-09-29 | `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`; `plans/closure/consumer-adoption/006-status.md` (+ M003-landed addendum) | Egress Delivery M003 landed on registry 0.1.2 pair with hosted Linux/macOS/Windows updater evidence (run `36639694985`) |
| Planning/closure hygiene corrective | C005 post-M001b materialization gate reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/005-post-m001b-materialization-gate-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/005-status.md` | docs-only; reconciled M001c stop + M001d gate |
| Archive extraction | M001b handle-bound cleanup + Windows portability corrective | closed | `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md`; `plans/closure/archive-extraction/001b-status.md` | hosted run `36222536670` green on all lanes |
| Acquisition transport | M008 sub-second deadline truthfulness corrective | closed | `plans/implementation/acquisition-transport/008-subsecond-deadline-truthfulness-corrective.md`; `plans/closure/acquisition-transport/008-status.md` | hosted run `36222536670` reached Windows portable tests |
| Archive extraction | M001a owned-root cleanup authority corrective | historical; superseded | `plans/implementation/archive-extraction/001a-owned-root-cleanup-authority-corrective.md`; `plans/closure/archive-extraction/001a-status.md` | superseded by M001b |
| Planning/closure hygiene corrective | C004 post-M007/M001 status + baseline reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/004-post-m007-m001-status-baseline-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/004-status.md` | docs-only; reconcile the new corrective gates and stale status text |
| Acquisition transport | M007 boundary safety hardening corrective | conditionally closed | `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`; `plans/closure/acquisition-transport/007-status.md` | M001-M006 closed; review baseline `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` |
| Service lifecycle | M007 UTF-8-safe bounded diagnostics corrective | closed | `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md`; `plans/closure/service-lifecycle/007-status.md` | Hosted Stable/MSRV/macOS/Windows qualification `36215858056` |
| Archive extraction | M001 bounded allowlisted extraction contract | closed | `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`; `plans/closure/archive-extraction/001-status.md` | hosted Linux/macOS/Windows qualification `36214688691` |
| Consumer adoption | M006 Egress archive/pair adoption | closed 2026-09-29 (dedupe: see active row above) | `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`; `plans/closure/consumer-adoption/006-status.md` | Egress Delivery M003 landed; no git/path dependency in Egress workspace |
| Eggpack manifest interoperability | M002 archive extraction handoff | closed with hosted qualification (M002a; dedupe: see active row above) | `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`; `plans/closure/eggpack-manifest-interoperability/002-status.md` | — |
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
| Eggpack manifest interoperability | M003a caller-bound destination policy corrective | closed | `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md`; `plans/closure/eggpack-manifest-interoperability/003a-status.md` | implementation `39ff626`, hosted run `36890986000` green on all lanes |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | closed | `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`; `plans/closure/eggpack-manifest-interoperability/003-status.md` | consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32`; hosted CI run `36902758482` + drift run `36902758396` green |
| Eggpack manifest interoperability | M004 package/API promotion | blocked on M004a-proven prerequisites | roadmap milestone; no implementation plan until ready | first gate is registered `eggstack/eggpack` Release Manifest M003 (`plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md`) closing with exact `eggpack-manifest =0.1.0` registry resolution; then Eggup-owned `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` (core/archive 0.1.2 already published); see `plans/closure/eggpack-manifest-interoperability/004a-status.md` |

CodeGG M005, Verified Update Core M007, and Service Lifecycle M005-M008 remain closed. Acquisition M008 is closed via hosted run `36222536670`. Archive M001b remains closed for handle-bound cleanup and stable-Windows portability. Archive M001c is a historical write-authority predecessor whose Section 14 handoff stop was closed by M001d. Service M008 closed the deadline-test determinism gap with green hosted run `36332823865`. Archive M001d is closed with green hosted run `36335233644` (superseding `36334772510`). Verified Core M008 package qualification is closed with exact clean-tree evidence (M008a) and published by M009 on 2026-09-28 (`eggup-core 0.1.2` then `eggup-archive 0.1.2`, `v0.1.2` + GitHub Release `0.1.2`; see `plans/closure/verified-update-core/009-status.md`), and Eggpack Interop M002 archive handoff is closed with full hosted qualification (M002a). Consumer Adoption M006 is closed 2026-09-29 (Egress Delivery M003 landed on the registry 0.1.2 pair with hosted updater evidence; M003-landed addendum recorded). Planning/closure hygiene C001-C011 are closed (C009 reconciled the M006 evidence; C010 reconciled the post-Egress active-state picture and refreshed the M003 readiness surfaces; C011 reconciled the post-M003 header/registration state and closed the M004a transition). Both Gregg-reference feature milestones are closed with green hosted qualification, and M005/M006 closure evidence is reconciled. Gregg M004 prerequisites are satisfied but its plan remains intentionally unwritten; no Gregg migration is authorized. Eggpack Interop M001a is closed with the full adapter regression matrix. M003 bounded parse/project qualification is complete, and the prior producer-owned artifact/manifest convention blocker is resolved by Eggpack Ecosystem M001 / Eggsact M005 `v1.2.7`. Current contract review isolated one generic destination-authority gap: M003a closed it (implementation `39ff626`, hosted run `36890986000` green on all lanes; see `plans/closure/eggpack-manifest-interoperability/003a-status.md`) and M003 Eggsact real-consumer adoption has now closed on that seam (consumer `eggstack/eggsact@65c916b`, hosted CI run `36902758482` + drift run `36902758396` green; see `plans/closure/eggpack-manifest-interoperability/003-status.md`). M004a package/API promotion readiness preflight is closed (see `plans/closure/eggpack-manifest-interoperability/004a-status.md`): it proved the minimum registry publication set is Eggpack-owned `eggpack-manifest 0.1.0`, then Eggup-owned `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` (a new `eggup-eggfetch 0.1.2` publication IS required — `E0308` incompatibility proven). The external producer prerequisite is now registered as Eggpack Release Manifest M003 at `eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md` (initial registration `b8cfce59`, active registry reconciliation `338ba642`). M004 proper remains blocked until every required external registry version is actually resolvable. Planning/closure hygiene C011 is closed (see `plans/closure/planning-closure-hygiene-corrective/011-status.md`). There is no dependency-ready producer-distribution implementation work in Eggup; that subsystem is archived/transferred.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Planning/closure hygiene corrective | C011 post-M003 closure + registration reconciliation | closed | docs-only; registry header/current-state bookkeeping reconciled, M004a transition registered and closed |
| Eggpack manifest interoperability | M004a package/API promotion readiness preflight | closed | qualification only, no publication occurred; minimum publication graph proven (see closure) |
| Eggpack manifest interoperability | M004 package/API promotion | blocked | M004a closed; first gate is registered Eggpack Release Manifest M003 publication plan (`eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md`) reaching closure with exact `eggpack-manifest =0.1.0`; then Eggup-owned `acquisition`/`eggfetch`/`eggpack` 0.1.2 in dependency order; no M004 implementation plan yet |
| Eggpack manifest interoperability | M003a caller-bound destination policy corrective | closed | `plans/closure/eggpack-manifest-interoperability/003a-status.md` (implementation `39ff626`, run `36890986000` green) |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | closed | consumer `eggstack/eggsact@65c916b` (Eggup pin `e336b32`); `plans/closure/eggpack-manifest-interoperability/003-status.md` (hosted CI `36902758482` + drift `36902758396` green) |
| Planning/closure hygiene corrective | C010 post-Egress/M003 active-status reconciliation | closed | docs-only; reconciled archive-roadmap Egress closure, registry M003 producer-gate resolution, and M003 plan stale dependency prose; `plans/implementation/planning-closure-hygiene-corrective/010-post-egress-and-m003-readiness-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/010-status.md` |
| Archive extraction | M001c handle-relative member materialization | historical predecessor; handoff stop resolved by M001d | `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`; `plans/closure/archive-extraction/001c-status.md` |
| Service lifecycle | M008 OperationDeadline test determinism | closed | `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`; `plans/closure/service-lifecycle/008-status.md` |
| Planning/closure hygiene corrective | C007 C006 head + current-CI reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/007-c006-head-and-current-ci-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/007-status.md` |
| Planning/closure hygiene corrective | C006 M001d readiness + bound-source contract reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/006-m001d-readiness-and-bound-source-contract-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/006-status.md` |
| Archive extraction | M001d handle-backed source handoff | closed | `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`; `plans/closure/archive-extraction/001d-status.md` (run `36335233644`) |
| Planning/closure hygiene corrective | C005 post-M001b materialization gate reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/005-post-m001b-materialization-gate-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/005-status.md` |
| Archive extraction | M001b handle-bound cleanup + Windows portability | closed | `plans/closure/archive-extraction/001b-status.md` (run `36222536670`) |
| Acquisition transport | M008 sub-second deadline truthfulness | closed | `plans/closure/acquisition-transport/008-status.md` (run `36222536670`) |
| Archive extraction | M001a owned-root cleanup authority | historical; superseded | `plans/closure/archive-extraction/001a-status.md` |
| Planning/closure hygiene corrective | C004 post-M007/M001 reconciliation | closed historically | `plans/closure/planning-closure-hygiene-corrective/004-status.md` |
| Acquisition transport | M007 boundary safety hardening | conditionally closed | `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md` |
| Service lifecycle | M007 UTF-8-safe bounded diagnostics | closed | `plans/closure/service-lifecycle/007-status.md` |
| Archive extraction | M001 bounded allowlisted extraction | closed historically; M001a superseded by M001b | `plans/closure/archive-extraction/001-status.md` |
| Acquisition transport | M005 curl adapter + explicit transport composition | closed | `plans/closure/acquisition-transport/005-status.md` |
| Planning/closure hygiene corrective | C001 post-batch status/evidence reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/001-status.md` |
| Planning/closure hygiene corrective | C002 final commit/baseline bookkeeping | closed | `plans/closure/planning-closure-hygiene-corrective/002-status.md` |
| Planning/closure hygiene corrective | C003 M003 registry/closure reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/003-status.md` |
| Service lifecycle | M005 prepared-transaction lifecycle integration | closed | `plans/closure/service-lifecycle/005-status.md` |
| Service lifecycle | M006 daemon update disposition/reference qualification | closed | `plans/closure/service-lifecycle/006-status.md` |
| Distribution/bootstrap | M004 retire `eggup-dist` | closed | `plans/closure/distribution-bootstrap/004-status.md` |
| Consumer adoption | M004 Gregg | writable; plan intentionally unwritten | prerequisites satisfied (acquisition M005/M006 + service M006 closed, hosted matrix green); no migration authorized |
| Consumer adoption | M005 CodeGG | closed | `plans/closure/consumer-adoption/005-status.md`; producer release mapping remains application/Eggpack-owned |
| Verified update core | M009 core/archive 0.1.2 publication | closed 2026-09-28 | `plans/closure/verified-update-core/009-status.md` (pair published, smoke 3/3, `v0.1.2` + Release `0.1.2`) |
| Consumer adoption | M006 Egress | closed 2026-09-29; M003-landed addendum recorded | — |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Eggpack manifest interoperability | M002 archive extraction handoff | closed with hosted qualification (M002a) | M002a green run `36477024102` (Stable clippy/tests/docs + MSRV + macOS + Windows adapter runtime) |
| Eggpack manifest interoperability | M004 package/API promotion | blocked | M004a closed; registered Eggpack Release Manifest M003 must first publish/prove exact `eggpack-manifest =0.1.0`, then Eggup-owned `acquisition`/`eggfetch`/`eggpack` 0.1.2 follow (see M004a closure) |
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
                     v
               M003a caller-bound destination seam [CLOSED via 39ff626, run 36890986000]
                      |
                      `--> M003 real-consumer adoption [CLOSED via eggsact 65c916b, CI run 36902758482 + drift 36902758396]
                               |
                               `--> M004a readiness preflight [CLOSED; see 004a-status.md]
                                        |
                                        `--> M004 package/API promotion [BLOCKED: Eggpack Release Manifest M003 plan registered for eggpack-manifest 0.1.0; then Eggup acquisition/eggfetch/eggpack 0.1.2]

core M006 + acquisition M004 [closed] --> CodeGG M005 managed bundle [CLOSED]
                                            |
                                            `--> producer release mapping stays in application/Eggpack; strict archive extraction stays CodeGG-owned

Archive Extraction M001 [CLOSED; run 36214688691]
               |
               v
Archive M001a cleanup authority [HISTORICAL; SUPERSEDED by M001b]
               |
               v
Archive M001b handle-bound cleanup + Windows portability [CLOSED; run 36222536670]
               |
               v
Archive M001c handle-relative member materialization [HISTORICAL PREDECESSOR; HANDOFF STOP RESOLVED BY M001d]
                |
                v
Planning C006 M001d readiness correction [CLOSED]
                |
                v
Service M008 deadline-test determinism [CLOSED via run 36332823865] -- green repo baseline restored
                |
                +--> Planning C007 current-CI/head reconciliation [CLOSED]
                |
                v
Archive M001d handle-backed source handoff [CLOSED via run 36335233644]
                |
                +--> Eggpack interoperability M002 [CLOSED with hosted qualification via M002a, run 36477024102]
                |         |
                |         `--> M002a hosted qualification/clippy corrective [CLOSED via run 36477024102]
                |                   |
                |                   `--> Core M008a clean package/current-head evidence [CLOSED; exact clean-tree evidence, no delta]
                |                             |
                |                             `--> Core M009 manual 0.1.2 publication [CLOSED 2026-09-28]
                |                                       |
                |                                       `--> Egress consumer M006 [CLOSED 2026-09-29 via Delivery M003]
                |
                `--> Planning C009 M006 Egress evidence reconciliation [CLOSED; docs-only]

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

review baseline ee1476 -> acquisition M007 boundary safety [CONDITIONALLY CLOSED; hosted run 36213509807]
                       `-> service M007 UTF-8 diagnostics [CLOSED; hosted run 36215858056]

review baseline ea51fe -> acquisition M008 deadline truthfulness [CLOSED via run 36222536670]
                       `-> archive M001a cleanup authority [HISTORICAL; SUPERSEDED by M001b]

implementation 0573996 -> archive M001b cleanup/Windows corrective [CLOSED]
                      `-> hosted run 36222536670 [STABLE/MSRV/MACOS/WINDOWS ALL PASSED]

Gregg read-only reference evidence -> acquisition M005 curl/composition [CLOSED]
                                  \-> service M006 disposition/revalidation [CLOSED]
                                                |
                                                v
                                acquisition M006 Windows/closure corrective [CLOSED via run 36176009068]
                                                |
                                                `-> Gregg M004 [WRITABLE; plan intentionally unwritten]
```

The earlier 2026-09-26 review opened acquisition M007 and service M007; those are conditionally closed and closed respectively. Archive Extraction M001 closed after hosted run 36214688691. The follow-up review opened Acquisition M008 and Archive M001a plus docs-only C004. Archive M001b then closed the remaining cleanup-authority TOCTOU and stable-Windows compilation defect with green hosted run `36222536670`, which also closed M008 hosted qualification. Post-M001b review found the separate materialization write-authority gap; M001c implemented the handle-relative write half (`09c953f`) and stopped under Section 14 on path-handoff truthfulness. M001d closed that stop with the object-bound source seam and green hosted run `36335233644`. C008 reconciled the historical M001c status. Eggpack interoperability M002 and Core M008 closed fully in this batch: M002a fixed the Stable clippy regression and added direct Windows adapter runtime evidence (green run `36477024102`, superseding failed `36463041223`), and M008a reran the exact clean-tree package/publish-dry-run evidence with no material delta. Planning-hygiene C009 corrected the stale Egress evidence in the M006 execution record: current `eggstack/eggress@03134f8` contains both the updater surface and Delivery M003. Verified Core M009 closed the publication gate on 2026-09-28 (registry-visible `eggup-core 0.1.2` + `eggup-archive 0.1.2`, smoke 3/3, `v0.1.2` + Release `0.1.2`; see `plans/closure/verified-update-core/009-status.md`), and Egress Delivery M003 subsequently landed Egress consumer M006 on 2026-09-29 with hosted Linux/macOS/Windows updater evidence (run `36639694985`; see `plans/closure/consumer-adoption/006-status.md` M003-landed addendum). Eggsearch M003, distribution M003/M004, service M004-M006, CodeGG M005, Verified Update Core M007, and Acquisition M005/M006 have reviewed closure evidence. Planning/closure hygiene C001-C003 and C008 are closed; C009 is closed (docs-only M006 evidence reconciliation); C010 is closed (docs-only post-Egress active-state reconciliation that refreshed the M003 readiness surfaces); C011 is closed (docs-only post-M003 header/registration reconciliation; see `plans/closure/planning-closure-hygiene-corrective/011-status.md`). Both Gregg-reference upstream mechanisms are closed with green hosted qualification; Gregg consumer migration is writable but not scheduled (plan intentionally unwritten). Eggpack manifest adapter/API work is structurally implemented and qualified, and adapter M001a has closed with the full regression matrix and truthful service packageability reconciliation. Historical M001 remains closed evidence; lower Eggup crates remain Eggpack-independent. M003 bounded JSON parse/project work is qualified and the former producer convention gate is resolved by Eggpack Ecosystem M001 / Eggsact M005 `v1.2.7`; M003 real-consumer adoption is closed (consumer `65c916b`), M004a readiness preflight is closed with the minimum publication set proven, and M004 promotion is blocked on the M004a-proven prerequisites: registered Eggpack Release Manifest M003 must first close with `eggpack-manifest =0.1.0` registry-resolvable, then Eggup owns `acquisition`/`eggfetch`/`eggpack` 0.1.2. No Eggup installer-generator replacement is authorized.

## Current project state

- Rust baseline: 1.89.
- Core: M001-M009 are closed. M009 published `eggup-core 0.1.2` then `eggup-archive 0.1.2` to crates.io on 2026-09-28 (registry-only smoke 3/3, `v0.1.2`, GitHub Release `0.1.2`); publication stayed manual with no release CI added. No automatic publication is authorized.
- Acquisition: M001-M006 are closed; M007 is conditionally closed. M008 sub-second deadline truthfulness is closed via hosted run `36222536670`, which executed the Windows portable acquisition/curl tests after Archive M001b restored the Windows lane.
- Service: M001-M008 remain closed. Service M008 replaced the scheduler-sensitive macOS unit proof with deterministic injected-`Instant` arithmetic without changing lifecycle/runtime deadline semantics; hosted run `36332823865` is green on all lanes (supersedes `36261671380`). See `plans/closure/service-lifecycle/008-status.md`.
- Distribution: M001-M003 remain historical predecessor evidence; M004 removed the producer crate after Eggpack Contract M002 closure. The subsystem is archived/transferred to Eggpack.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg M004 prerequisites are satisfied and the milestone remains intentionally unwritten. Egress M006 is closed 2026-09-29 (Egress Delivery M003 landed on the registry 0.1.2 pair; M003-landed addendum in `plans/closure/consumer-adoption/006-status.md`).
- Archive extraction: ADR-0005/M001 are historical foundation; M001a is superseded; M001b is closed; M001c is a historical write-authority predecessor whose Section 14 stop was resolved by M001d; M001d is closed with green run `36335233644`.
- Eggpack interoperability: M001/M001a remain closed. M002 archive handoff is closed with full hosted qualification via M002a (green run `36477024102`: Stable clippy/tests/docs + MSRV + macOS + Windows adapter runtime). M003 bounded JSON adapter qualification and the former producer convention gate are satisfied. M003a caller-bound destination corrective is closed (implementation `39ff626`, green run `36890986000`); M003 Eggsact real-consumer adoption is closed (consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32`; hosted CI run `36902758482` + drift run `36902758396` green). M004 promotion is blocked on the M004a-proven prerequisites: Eggpack Release Manifest M003 is registered at `eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md` and must first close with exact `eggpack-manifest =0.1.0` registry resolution; then Eggup owns `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2`; see `plans/closure/eggpack-manifest-interoperability/004a-status.md`.
- Release process: manual crates.io publication only.
- The lockstep 0.1.1 patch is published (seam-then-adapter order) now that the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until downstream adoption moves; no publication is implicit in these plans.

## Next handoff

Egress consumer M006 is closed (Delivery M003 landed 2026-09-29 against the published 0.1.2 pair with hosted updater evidence). No other 0.1.2 workspace crate was published. Gregg M004 remains intentionally unwritten. Eggpack Interop M003 Eggsact runtime manifest adoption is closed (consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32`; hosted CI `36902758482` + drift `36902758396` green); producer evidence came from Eggpack Ecosystem M001 / Eggsact M005 `v1.2.7`, and the bounded JSON adapter path plus the caller-bound destination seam are qualified in production use. C011 post-M003 closure reconciliation and the M004a package/API promotion readiness preflight are both closed (see `plans/closure/planning-closure-hygiene-corrective/011-status.md` and `plans/closure/eggpack-manifest-interoperability/004a-status.md`). Eggpack has now registered the producer publication handoff as Release Manifest M003 at `plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md`. M004 proper remains blocked until that plan closes with exact `eggpack-manifest =0.1.0` registry resolution and Eggup then publishes its own `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` chain; no M004 implementation plan exists yet.

Do not author or implement an Eggup installer generator. Keep producer behavior in Eggpack and archive extraction outside the adapter. M004a is closed (qualification/package-simulation only; nothing published); do not author M004 until every M004a-proven external registry version is actually resolvable. While `eggpack-manifest` remains absent, the publication handoff is already routed to registered Eggpack Release Manifest M003 (`eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md`); do not duplicate producer ownership in Eggup.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
