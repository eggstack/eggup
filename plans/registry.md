# Eggup Planning Registry

Status: active

Last implementation/closure baseline reviewed: `db5b3121f17f92029a47c389e68a464bc4478b27` (Acquisition Transport M009 eggup-curl 0.1.2 published from `b485228` with hosted run `37223895075` green; runtime/package baseline is now the M009 publication at `b485228`, superseding the M004 publication at `02a1d32` whose closure remains `ea1f1c5`)

Latest planning/closure head: `d9cefc0960e86965ca8ac95aebc1b22febf64bab` (C012 source plan marked closed; closure record `af62ede4`; superseded for the acquisition subsystem by the M009 closure record `plans/closure/acquisition-transport/009-status.md` registered below)

Latest reviewed pre-C003 planning/status baseline: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` (M003 producer-gate execution/status record; historical)

Latest planning registration head: `d71d75af90ab9d5913238de16ffeb562576ef4fe` (C012 closure state registered in the active control surface)

Newest qualified Eggup runtime delta with fully green hosted qualification is M003a implementation `39ff62602e39b14b31f5a8f154905d3343034db7` (feat(eggpack): caller-bound destination binding; hosted run `36890986000` green on Stable/MSRV/macOS/Windows). Runtime commits have landed after `39ff626`: `git log 39ff626..HEAD -- crates/` returns `e548b64` (eggpack 0.1.2 registry promotion), `0b8cb98` (workspace bug-audit fixes across `eggup-acquisition`/`eggup-archive`/`eggup-core`), `b485228` (eggup-curl 0.1.2 changelog cut), and `4a95e4d` (docs reconciliation touching `crates/eggup-service/CHANGELOG.md` only). The two `0b8cb98` acquisition fixes are implemented and green but **not yet published** — Acquisition M010 republishes the seam as `eggup-acquisition 0.1.3` to carry them. M003 Eggsact real-consumer adoption is closed at consumer `eggstack/eggsact@65c916ba3b0f02916203ec1aad09d7e5c023c278` on Eggup pin `e336b32` (hosted CI run `36902758482` + drift run `36902758396` green). The M009 release-prep head `e8e07eb538d0eef18ea4cb4ace3bb905c316da72` (run `36487099388` green) remains the provenance for the published `eggup-core`/`eggup-archive 0.1.2` pair, not the latest qualified runtime. Code qualification was established at `9a5500e` (M002a clippy + Windows-adapter head; run `36477024102` green, superseding failed run `36463041223`); Core M008a reran the exact clean-tree package/publish-dry-run evidence on the post-M002a tree with no material delta. Planning-hygiene C009 corrected the stale Consumer M006 claim: current `eggstack/eggress@03134f8` has both the updater surface and registered Delivery M003 plan. Eggup M009 published the compatible 0.1.2 pair on 2026-09-28; Egress M006 subsequently closed on 2026-09-29 after Egress Delivery M003 landed with hosted Linux/macOS/Windows updater evidence. The prior Eggpack cross-repo drift is now reconciled: Eggpack interoperability M003 records the downstream closure, and Eggpack Release Manifest M003 is closed: `eggpack-manifest 0.1.0` is published to crates.io from Eggpack `8d661e4` (checksum `2a08f24b…b629`, tag `eggpack-manifest-v0.1.0`, closure `eggstack/eggpack: plans/closure/release-manifest/003-status.md`).

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
| Verified update core | M001-M007 closed; M008/M008a package qualification closed; M009 published the `eggup-core`/`eggup-archive 0.1.2` pair 2026-09-28 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M006 closed; M007 conditionally closed; M008 closed; M009 published `eggup-curl 0.1.2` and closed 2026-10-04; **M010 is the one open milestone** | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M008 closed | `plans/closure/service-lifecycle/` |
| Archive extraction | M001 foundation historical; M001a superseded; M001b closed; M001c historical predecessor (stop resolved by M001d); M001d closed | `plans/closure/archive-extraction/` |
| Eggpack manifest interoperability | M001/M001a/M002/M002a/M003a/M003/M004a/M004 all closed; M004 published `eggup-acquisition`/`eggup-eggfetch`/`eggup-eggpack 0.1.2` | `plans/closure/eggpack-manifest-interoperability/` |
| Distribution/bootstrap (archived/transferred) | M001-M004 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch + M005 CodeGG + M006 Egress (closed 2026-09-29) | `plans/closure/consumer-adoption/` |
| Planning/closure hygiene corrective | C001-C012 all closed (docs-only reconciliation track) | `plans/closure/planning-closure-hygiene-corrective/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.1 crates (lockstep patch superseding 0.1.0):

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

Published `0.1.2` set on crates.io (six crates, in three steps): `eggup-core`, `eggup-archive` (2026-09-28); `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack` (2026-10-02); `eggup-curl` (2026-10-04). `eggup-service` is published but **lags the workspace**: crates.io carries `0.1.0`/`0.1.1` only, so its current `0.1.2` has never been published and no publication milestone authorizes it. `eggup-transport-footprint` is `publish = false` by design and has no registry version.

`eggup-dist` was unpublished and existed only as migration predecessor evidence. It was removed after Eggpack Contract M002 qualified the complete M003 behavior; see `plans/closure/distribution-bootstrap/004-status.md`.

Existing simple consumers:

- eggsact at `eggstack/eggsact@65c916b` (current; first adopted at `576f4b0`);
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

Eggsearch M003, Service M004, Verified Update Core M007, Service M005, Acquisition M005/M006, and Service M006 are closed. A fresh 2026-10-05 parity review against `eggstack/gregg@1aac89f1a82fcd347066379b55105e2ff6bfe770` and `eggstack/eggpool@fe3c308df1450cd3216f50abc160a0a9f426f8d9` found two still-normative Core gaps before either downstream migration can be claimed regression-free: current-executable transaction parity/Windows running-image rollback (Core M010) and proof-authorized stale-lock recovery (Core M011). Service M009 is also required to publish the already-qualified service substrate. Gregg M004 and EggPool M007 therefore remain intentionally unwritten/blocked.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | M001-M009 closed; 0.1.2 core/archive pair published to crates.io | M010 current-executable parity + M011 proof-authorized stale-lock recovery (both ready) |
| Acquisition transport | M001-M006 closed; M007 conditionally closed; M008 closed; M009 published `eggup-curl 0.1.2` and closed | M010 `eggup-acquisition 0.1.3` publication (release-order blocked on Service M009) |
| Service lifecycle | M001-M008 closed | M009 `eggup-service 0.1.2` publication (ready; precedes workspace 0.1.3 bump) |
| Archive extraction | M001/M001a historical; M001b closed; M001c historical predecessor with stop resolved by M001d; M001d closed | — |
| Distribution/bootstrap | archived/transferred; M001-M004 closed | no further Eggup producer work |
| Eggpack manifest interoperability | M001/M001a/M002/M002a/M003a/M003/M004a/M004 all closed | nothing outstanding in this subsystem; the producer schema and the adapter are both registry-resolvable and a downstream migration is executable |
| Consumer adoption | simple, eggsearch, CodeGG M005, and Egress M006 closed; C009 evidence reconciliation closed | Gregg M004 + EggPool M007 intentionally unwritten/blocked on new Core substrate gates |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Verified update core | M010 current-executable transaction parity corrective | ready | `plans/implementation/verified-update-core/010-current-executable-transaction-parity-corrective.md` | Core M009 closed; current Gregg/EggPool reference evidence |
| Verified update core | M011 proof-authorized stale-lock recovery | ready | `plans/implementation/verified-update-core/011-proof-authorized-stale-lock-recovery.md` | Core M009 closed; independent of M010, serialize Core edits |
| Service lifecycle | M009 eggup-service 0.1.2 publication | ready | `plans/implementation/service-lifecycle/009-eggup-service-0.1.2-publication.md` | M001-M008 closed; must precede Acquisition M010 workspace version bump |
| Acquisition transport | M009 eggup-curl 0.1.2 publication | closed 2026-10-04 | `plans/implementation/acquisition-transport/009-eggup-curl-0.1.2-publication.md`; `plans/closure/acquisition-transport/009-status.md` | published from `b485228`, hosted run `37223895075` green on all lanes; registry-only fixture 6/6 with registry-source lockfile; no `src/` change; `v0.1.1`/`v0.1.2` unmoved, release `0.1.2` notes extended; M007 Windows live-loopback limitation retained |
| Acquisition transport | M010 eggup-acquisition 0.1.3 publication | blocked (release order) | `plans/implementation/acquisition-transport/010-eggup-acquisition-0.1.3-publication.md` | Service M009 must close first; M009 curl publication closed; the two `0b8cb98` acquisition fixes are implemented and green, so this is publication-only. `eggup-eggpack 0.1.2` pins `=0.1.2` and needs a separate adapter republication to inherit the fix |
| Planning/closure hygiene corrective | C012 post-M004 roadmap + registry reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/012-post-m004-roadmap-and-registry-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/012-status.md` | docs-only; Phase 9/10 + post-M004 registry/archive active surfaces reconciled |
| Planning/closure hygiene corrective | C011 post-M003 closure + registration reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/011-post-m003-closure-and-registration-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/011-status.md` | docs-only; M003a/M003 closures authoritative; M004a closed concurrently |
| Eggpack manifest interoperability | M004a package/API promotion readiness preflight | closed | `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md`; `plans/closure/eggpack-manifest-interoperability/004a-status.md` | M003a/M003 closed; qualification only, no publication occurred; M004 has since closed (see M004 row below) |
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
| Eggpack manifest interoperability | M004 package/API promotion | closed | `plans/implementation/eggpack-manifest-interoperability/004-registry-package-api-promotion-and-publication.md`; `plans/closure/eggpack-manifest-interoperability/004-status.md` | published `eggup-acquisition` 0.1.2 (`0b01deb8…f170`), `eggup-eggfetch` 0.1.2 (`2e483152…3528`), `eggup-eggpack` 0.1.2 (`9dbfdfb7…b13fef`) from `02a1d32`; hosted run `37090397398` green; adapter-only and Eggsact-shaped registry-only proofs green; publication order proven load-bearing; `v0.1.1`/`v0.1.2` unmoved, release `0.1.2` notes extended |

CodeGG M005, Verified Update Core M007, and Service Lifecycle M005-M008 remain closed. Acquisition M008 is closed via hosted run `36222536670`. Archive M001b remains closed for handle-bound cleanup and stable-Windows portability. Archive M001c is a historical write-authority predecessor whose Section 14 handoff stop was closed by M001d. Service M008 closed the deadline-test determinism gap with green hosted run `36332823865`. Archive M001d is closed with green hosted run `36335233644` (superseding `36334772510`). Verified Core M008 package qualification is closed with exact clean-tree evidence (M008a) and published by M009 on 2026-09-28 (`eggup-core 0.1.2` then `eggup-archive 0.1.2`, `v0.1.2` + GitHub Release `0.1.2`; see `plans/closure/verified-update-core/009-status.md`), and Eggpack Interop M002 archive handoff is closed with full hosted qualification (M002a). Consumer Adoption M006 is closed 2026-09-29 (Egress Delivery M003 landed on the registry 0.1.2 pair with hosted updater evidence; M003-landed addendum recorded). Planning/closure hygiene C001-C012 are closed (C009 reconciled the M006 evidence; C010 reconciled the post-Egress active-state picture and refreshed the M003 readiness surfaces; C011 reconciled the post-M003 header/registration state and closed the M004a transition). Both Gregg-reference feature milestones are closed with green hosted qualification, and M005/M006 closure evidence is reconciled. Gregg M004 prerequisites are satisfied but its plan remains intentionally unwritten; no Gregg migration is authorized. Eggpack Interop M001a is closed with the full adapter regression matrix. M003 bounded parse/project qualification is complete, and the prior producer-owned artifact/manifest convention blocker is resolved by Eggpack Ecosystem M001 / Eggsact M005 `v1.2.7`. Current contract review isolated one generic destination-authority gap: M003a closed it (implementation `39ff626`, hosted run `36890986000` green on all lanes; see `plans/closure/eggpack-manifest-interoperability/003a-status.md`) and M003 Eggsact real-consumer adoption has now closed on that seam (consumer `eggstack/eggsact@65c916b`, hosted CI run `36902758482` + drift run `36902758396` green; see `plans/closure/eggpack-manifest-interoperability/003-status.md`). M004a package/API promotion readiness preflight is closed (see `plans/closure/eggpack-manifest-interoperability/004a-status.md`): it proved the minimum registry publication set is Eggpack-owned `eggpack-manifest 0.1.0`, then Eggup-owned `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` (a new `eggup-eggfetch 0.1.2` publication IS required — `E0308` incompatibility proven). The external producer prerequisite is registered as Eggpack Release Manifest M003 at `eggstack/eggpack: plans/implementation/release-manifest/003-eggpack-manifest-0.1.0-publication-and-compatibility-baseline.md` (initial registration `b8cfce59`, active registry reconciliation `338ba642`) and is now **closed**: `eggpack-manifest 0.1.0` was published to crates.io on 2026-10-02 from Eggpack `8d661e4eb9da1806e5d7c7606939d24e9aceb2c0` with checksum `2a08f24b05e9652878dd49145cdc3cbd38c7a76032d7b01a5fe1535d9446b629`, non-yanked, tag `eggpack-manifest-v0.1.0`, Eggpack hosted run `37064833069` green, closure `eggstack/eggpack: plans/closure/release-manifest/003-status.md`. A 2026-10-02 re-audit of the M004a-proven set confirms the published `src/lib.rs` is byte-identical to the consumer-qualified Git source, so the M004a compatibility-incident branch is not triggered. M004 proper has since closed: `eggup-acquisition`/`eggup-eggfetch`/`eggup-eggpack` 0.1.2 were published in dependency order from `02a1d32` (see `plans/closure/eggpack-manifest-interoperability/004-status.md`). Planning/closure hygiene C011 is closed (see `plans/closure/planning-closure-hygiene-corrective/011-status.md`). There is no dependency-ready producer-distribution implementation work in Eggup; that subsystem is archived/transferred.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker / evidence |
|---|---|---|---|
| Acquisition transport | M009 eggup-curl 0.1.2 publication | closed 2026-10-04 | `plans/closure/acquisition-transport/009-status.md` — published from `b485228`, hosted run `37223895075` green on all lanes; registry-only fixture 6/6 with registry-source lockfile; no `src/` change; `v0.1.1`/`v0.1.2` unmoved, release `0.1.2` notes extended; M007 Windows live-loopback limitation retained |
| Acquisition transport | M010 eggup-acquisition 0.1.3 publication | **ready** | `plans/implementation/acquisition-transport/010-eggup-acquisition-0.1.3-publication.md` — ready; M009 closed; the two `0b8cb98` acquisition fixes are implemented and green, so this is publication-only. `eggup-eggpack 0.1.2` pins `=0.1.2` and needs a separate adapter republication to inherit the fix |
| Planning/closure hygiene corrective | C012 post-M004 roadmap + registry reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/012-status.md`; no runtime/Cargo/workflow delta |
| Planning/closure hygiene corrective | C011 post-M003 closure + registration reconciliation | closed | docs-only; registry header/current-state bookkeeping reconciled, M004a transition registered and closed |
| Eggpack manifest interoperability | M004a package/API promotion readiness preflight | closed | qualification only, no publication occurred; minimum publication graph proven (see closure); its Eggpack-owned item is now satisfied — `eggpack-manifest 0.1.0` published 2026-10-02 |
| Eggpack manifest interoperability | M004 package/API promotion | closed | M004a closed; Eggpack Release Manifest M003 published `eggpack-manifest 0.1.0`; M004 published `eggup-acquisition`/`eggup-eggfetch`/`eggup-eggpack` 0.1.2 in order from `02a1d32`, with adapter-only and Eggsact-shaped registry-only proofs green; see `plans/closure/eggpack-manifest-interoperability/004-status.md` |
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
| Eggpack manifest interoperability | M004 package/API promotion | closed | M004a closed; Eggpack Release Manifest M003 published `eggpack-manifest 0.1.0`; M004 then published `acquisition`/`eggfetch`/`eggpack` 0.1.2 from `02a1d32` (see M004 closure) |
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
                                        `--> M004 package/API promotion [CLOSED: acquisition/eggfetch/eggpack 0.1.2 published]

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
                       `-> M009 eggup-curl 0.1.2 publication [CLOSED 2026-10-04; published from b485228]
                            |
                            +-> cargo-cleanme Phase 10 M010C lightweight updater [UNBLOCKED; consumer-owned]
                            |
                            `-> M010 eggup-acquisition 0.1.3 publication [READY; ships the 0b8cb98 seam fixes]
                                     |
                                     `-> eggup-eggpack adapter republication [NAMED FOLLOW-ON; =0.1.2 pin blocks inheritance]

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

The earlier 2026-09-26 review opened acquisition M007 and service M007; those are conditionally closed and closed respectively. Archive Extraction M001 closed after hosted run 36214688691. The follow-up review opened Acquisition M008 and Archive M001a plus docs-only C004. Archive M001b then closed the remaining cleanup-authority TOCTOU and stable-Windows compilation defect with green hosted run `36222536670`, which also closed M008 hosted qualification. Post-M001b review found the separate materialization write-authority gap; M001c implemented the handle-relative write half (`09c953f`) and stopped under Section 14 on path-handoff truthfulness. M001d closed that stop with the object-bound source seam and green hosted run `36335233644`. C008 reconciled the historical M001c status. Eggpack interoperability M002 and Core M008 closed fully in this batch: M002a fixed the Stable clippy regression and added direct Windows adapter runtime evidence (green run `36477024102`, superseding failed `36463041223`), and M008a reran the exact clean-tree package/publish-dry-run evidence with no material delta. Planning-hygiene C009 corrected the stale Egress evidence in the M006 execution record: current `eggstack/eggress@03134f8` contains both the updater surface and Delivery M003. Verified Core M009 closed the publication gate on 2026-09-28 (registry-visible `eggup-core 0.1.2` + `eggup-archive 0.1.2`, smoke 3/3, `v0.1.2` + Release `0.1.2`; see `plans/closure/verified-update-core/009-status.md`), and Egress Delivery M003 subsequently landed Egress consumer M006 on 2026-09-29 with hosted Linux/macOS/Windows updater evidence (run `36639694985`; see `plans/closure/consumer-adoption/006-status.md` M003-landed addendum). Eggsearch M003, distribution M003/M004, service M004-M006, CodeGG M005, Verified Update Core M007, and Acquisition M005/M006 have reviewed closure evidence. Planning/closure hygiene C001-C003 and C008 are closed; C009 is closed (docs-only M006 evidence reconciliation); C010 is closed (docs-only post-Egress active-state reconciliation that refreshed the M003 readiness surfaces); C011 is closed (docs-only post-M003 header/registration reconciliation; see `plans/closure/planning-closure-hygiene-corrective/011-status.md`). Both Gregg-reference upstream mechanisms are closed with green hosted qualification; Gregg consumer migration is writable but not scheduled (plan intentionally unwritten). Eggpack manifest adapter/API work is structurally implemented and qualified, and adapter M001a has closed with the full regression matrix and truthful service packageability reconciliation. Historical M001 remains closed evidence; lower Eggup crates remain Eggpack-independent. M003 bounded JSON parse/project work is qualified and the former producer convention gate is resolved by Eggpack Ecosystem M001 / Eggsact M005 `v1.2.7`; M003 real-consumer adoption is closed (consumer `65c916b`), M004a readiness preflight is closed with the minimum publication set proven, and M004 promotion has since closed: the Eggup-owned `acquisition`/`eggfetch`/`eggpack` 0.1.2 chain was published in dependency order from `02a1d32` (see `plans/closure/eggpack-manifest-interoperability/004-status.md`). No Eggup installer-generator replacement is authorized.

## Current project state

- Rust baseline: 1.89.
- Core: M001-M009 are closed and the 0.1.2 core/archive pair remains published. Current consumer-parity research registers M010 (current-executable authority + Windows running-image rollback) and M011 (proof-authorized stale-lock recovery) as ready. A later Core package/publication milestone is intentionally unwritten until both close.
- Acquisition: M001-M006 are closed; M007 is conditionally closed. M008 sub-second deadline truthfulness is closed via hosted run `36222536670`, which executed the Windows portable acquisition/curl tests after Archive M001b restored the Windows lane. **M009 closed 2026-10-04**: `eggup-curl 0.1.2` was published from release-prep commit `b485228` (checksum `79df200f…b465`, 17193 bytes) after hosted run `37223895075` passed Stable/MSRV/macOS/Windows. `cargo package` compiled the package against the registry-downloaded `eggup-acquisition 0.1.2`, and an external registry-only fixture passed 6/6 with one registry source per package and no git or path entry. No `src/` byte changed, no tag was moved, and the M007 Windows live-loopback limitation is retained rather than upgraded. M010 is now the next dependency-ready milestone: it republishes the seam as `eggup-acquisition 0.1.3` carrying the two `0b8cb98` fixes that published `0.1.2` lacks.
- Service: M001-M008 remain closed. M009 is ready to publish `eggup-service 0.1.2` from the already-qualified service substrate; it should close before Acquisition M010 moves the workspace version to 0.1.3.
- Distribution: M001-M003 remain historical predecessor evidence; M004 removed the producer crate after Eggpack Contract M002 closure. The subsystem is archived/transferred to Eggpack.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg M004 is intentionally unwritten and blocked on Core M010/M011, Service M009, Acquisition M010, and a later versioned Core handoff. EggPool M007 is likewise unwritten and targets only the standalone-Rust transaction after Core M010/M011; package-manager/provenance paths stay downstream. Egress M006 is closed 2026-09-29 (Egress Delivery M003 landed on the registry 0.1.2 pair; M003-landed addendum in `plans/closure/consumer-adoption/006-status.md`).
- Archive extraction: ADR-0005/M001 are historical foundation; M001a is superseded; M001b is closed; M001c is a historical write-authority predecessor whose Section 14 stop was resolved by M001d; M001d is closed with green run `36335233644`.
- Eggpack interoperability: M001/M001a remain closed. M002 archive handoff is closed with full hosted qualification via M002a (green run `36477024102`: Stable clippy/tests/docs + MSRV + macOS + Windows adapter runtime). M003 bounded JSON adapter qualification and the former producer convention gate are satisfied. M003a caller-bound destination corrective is closed (implementation `39ff626`, green run `36890986000`); M003 Eggsact real-consumer adoption is closed (consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32`; hosted CI run `36902758482` + drift run `36902758396` green). M004 promotion is **closed** at `plans/closure/eggpack-manifest-interoperability/004-status.md`: `eggup-acquisition 0.1.2`, `eggup-eggfetch 0.1.2`, and `eggup-eggpack 0.1.2` were published in that order from publication source `02a1d32` (hosted run `37090397398` green on all four lanes). The adapter now resolves the Eggpack-owned `eggpack-manifest =0.1.0` from crates.io — published 2026-10-02 from `eggstack/eggpack@8d661e4`, checksum `2a08f24b…b629`, published `src/lib.rs` byte-identical to the consumer-qualified Git source (`eggstack/eggpack: plans/closure/release-manifest/003-status.md`). Two external registry-only graphs resolve with one registry source per package and no git/path edge and pass smoke. `eggup-curl` was subsequently published as `0.1.2` on 2026-10-04 by Acquisition M009 (the M004a graph proof correctly excluded it from M004's authorization, and it was promoted under its own plan); `eggup-service` (published only at `0.1.0`/`0.1.1`, so its `0.1.2` was never published) and `eggup-transport-footprint` (`publish = false`) were both excluded from M004a's publication authorization; `v0.1.1`/`v0.1.2` were not moved and release `0.1.2` notes were extended. Eggsact's own Git-to-registry migration is now executable but remains Eggsact-owned and separately authorized.
- Release process: manual crates.io publication only.
- The lockstep 0.1.1 patch is published (seam-then-adapter order) now that the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until downstream adoption moves; no publication is implicit in these plans.

## Next handoff

Acquisition M009 closed on 2026-10-04: `eggup-curl 0.1.2` is published from `b485228` with a registry-only external proof, so the cargo-cleanme Phase 10 M010C lightweight updater path is unblocked and cargo-cleanme retains release/version/fallback/install/self-update policy. The current substrate handoff has three dependency-ready items: **Core M010** (current-executable parity), **Core M011** (proof-authorized stale-lock recovery), and **Service M009** (`eggup-service 0.1.2` publication). Service M009 should execute before **Acquisition M010**, because M010 advances the shared workspace version to 0.1.3; Acquisition M010 is therefore release-order blocked until M009 closes. Published `0.1.2` still hands a composed transport the caller's full budget, so a composed fetch can run roughly twice the documented `total_timeout` until `0.1.3` exists; `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.2` inherit the fix automatically through their `^0.1.0` requirements, while `eggup-eggpack 0.1.2` pins `=0.1.2` and needs a separate adapter republication. Egress consumer M006 is closed (Delivery M003 landed 2026-09-29 against the published core/archive 0.1.2 pair with hosted updater evidence). The currently published Eggup 0.1.2 set is `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack`, and `eggup-curl`; `eggup-service` (crates.io carries `0.1.0`/`0.1.1`; its `0.1.2` is unpublished and outside every publication authorization to date) and `eggup-transport-footprint` (`publish = false`) are both outside the `0.1.2` publication set. Gregg M004 remains intentionally unwritten and blocked on Core M010/M011, Service M009, Acquisition M010, and the later versioned Core handoff. Eggpack Interop M003 Eggsact runtime manifest adoption is closed (consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32`; hosted CI `36902758482` + drift `36902758396` green); producer evidence came from Eggpack Ecosystem M001 / Eggsact M005 `v1.2.7`, and the bounded JSON adapter path plus the caller-bound destination seam are qualified in production use. C011 post-M003 closure reconciliation and the M004a package/API promotion readiness preflight are both closed (see `plans/closure/planning-closure-hygiene-corrective/011-status.md` and `plans/closure/eggpack-manifest-interoperability/004a-status.md`). Eggpack closed the producer publication handoff as Release Manifest M003 (`eggstack/eggpack: plans/closure/release-manifest/003-status.md`) and Eggup M004 then published `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` from `02a1d32`, closing this seam end to end on 2026-10-02 (`plans/closure/eggpack-manifest-interoperability/004-status.md`). Eggsact's Git-to-registry migration remains Eggsact-owned and separately authorized. Acquisition M009 is closed and Acquisition M010 is now the only newly authorized Eggup package-promotion handoff.

Do not author or implement an Eggup installer generator. Keep producer behavior in Eggpack and archive extraction outside the adapter. M004a is closed (qualification/package-simulation only; nothing published); M004 is implemented, qualified, and closed; do not re-open it or re-publish its `0.1.2` versions. The `eggpack-manifest` producer item was discharged by Eggpack Release Manifest M003 and consumed by M004's registry pin. Do not duplicate producer ownership in Eggup. M009 is likewise closed: do not re-publish `eggup-curl 0.1.2` or any other published `0.1.2` version, and do not move `v0.1.1`/`v0.1.2`. M010 publishes a new `0.1.3` version of the seam only; it does not re-open M009 or M004.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
