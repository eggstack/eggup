# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `889a234cbe7f461d92def3df45c83c06a7d257e5`

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
| `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md` | core owns local deployment mechanism; transport, service, distribution, and consumer release policy remain separate |
| `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet is the mutation unit; rollback and post-commit policy are explicit |
| `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md` | integrity/authenticity/candidate identity are distinct; core has no mandatory transport |

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M006 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M003 | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M002 | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap | M001 | `plans/closure/distribution-bootstrap/001-status.md` |
| Consumer adoption | M001 eggsact + M002 stegoeggo | `plans/closure/consumer-adoption/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.0 crates remain:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

`eggup-dist` exists in the workspace but remains unpublished.

Existing simple consumers:

- eggsact at `eggstack/eggsact@576f4b0`;
- stegoeggo at `eggstack/stegoeggo@10d8448`.

## Post-closure review findings

The core remains qualified with no newly identified medium-or-higher defect.

Three corrective gates were identified after reviewing implementation SHA `889a234c`.

### Acquisition

M003 fixed request timeout use, temp-file ownership, no-clobber promotion, and diagnostic redaction, but two narrower contract gaps remain:

- `FetchLimits` fields are public, so direct struct literals can bypass `FetchLimits::new` validation unless transports revalidate at entry;
- `__promote_no_clobber` can create a complete destination and then return ordinary `Err` if redundant temp unlink fails.

Tracked by acquisition M004.

### Service

M002 established useful Unix adapters, but post-closure review found:

- launchd restart can return `completed=true` after an incomplete stop/start subtransition;
- caller timeout is not consistently an end-to-end wall-clock budget;
- production manager execution uses bare program names through ambient PATH and inherits the full environment;
- `ServiceSpec.config` is not faithfully represented in systemd/launchd ownership observations.

Tracked by service M003.

### Distribution

M001 established TOML schema v1, but:

- distinct bundle/archive declarations can collapse to duplicate expanded asset, sidecar, or install names;
- malformed brace/template grammar is not always rejected until expansion.

Tracked by distribution M002.

These findings supersede the prior registry statement that eggsearch/validators were planning-ready.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | qualified 0.1.0 | broader bundle/platform evidence later |
| Acquisition transport | active corrective | M004 validated-limits/promotion-state corrective |
| Service lifecycle | active corrective | M003 Unix adapter correctness/security corrective |
| Distribution/bootstrap | active corrective | M002 schema uniqueness/template corrective |
| Consumer adoption | simple tier closed | blocked on current acquisition/service correctives |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Acquisition transport | M004 | **ready for handoff** | `plans/implementation/acquisition-transport/004-validated-limits-and-promotion-state-corrective.md` | M003 closed + post-closure review |
| Service lifecycle | M003 | **ready for handoff** | `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md` | M002 closed + post-closure review |
| Distribution/bootstrap | M002 | **ready for handoff** | `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md` | M001 closed + post-closure review |

The three corrective passes are independent enough to execute in parallel.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M005 lightweight/curl adapter | deferred/evidence-driven | corrected M004 footprint evidence, especially Gregg |
| Service lifecycle | M004 Windows SCM | blocked / plan intentionally unwritten | service M003 corrective |
| Service lifecycle | M005 update-lifecycle integration | blocked / plan intentionally unwritten | service M003 + M004 + core |
| Distribution/bootstrap | M003 validators | blocked / plan intentionally unwritten | distribution M002 corrective |
| Distribution/bootstrap | M004 generators/adoptions | blocked / plan intentionally unwritten | distribution M003 |
| Consumer adoption | M003 eggsearch | blocked / plan intentionally unwritten | acquisition M004 + service M003 |
| Consumer adoption | M004 Gregg | blocked / plan intentionally unwritten | acquisition M004 + service M003 + corrected-path footprint decision |
| Consumer adoption | M005 CodeGG | later / blocked | mature bundle core + distribution M003 evidence |
| Consumer adoption | M006 Egress | later / blocked | archive contract + distribution M003 evidence |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M004 corrective -----------+
                                       +--> eggsearch M003 planning
service M003 corrective ---------------+
                 |
                 +-------------------------> Gregg corrected-path measurement/planning
                                            (plus acquisition M004)

distribution M002 corrective -----------> distribution M003 validators
                                           |
                                           +--> later CodeGG/Egress evidence
```

No new service-aware/updater-bearing consumer migration should begin until its relevant corrective gates close.

## Current project state

- Rust baseline: 1.89.
- Core: verified multi-artifact transaction/ownership/rollback boundary remains qualified.
- Acquisition: M003 implementation exists; M004 corrective is the active safety/contract gate.
- Service: M002 Unix adapters exist; M003 corrective is the active service-adoption gate.
- Distribution: TOML v1 crate exists; M002 corrective is the validator gate.
- Consumer adoption: eggsact/stegoeggo remain the only completed consumers.
- Release process: manual crates.io publication only.
- Previously planned 0.1.1 publication must wait until acquisition/service corrective disposition is known; never publish known-contract defects merely to preserve the old version plan.

## Next handoff

Ready now, any order:

- `plans/implementation/acquisition-transport/004-validated-limits-and-promotion-state-corrective.md`
- `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md`
- `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md`

After each corrective:

1. create the matching closure record with the actual implementation SHA;
2. update the source subsystem roadmap;
3. update this registry;
4. keep downstream work blocked if any medium-or-higher issue remains;
5. only then author the newly dependency-ready downstream plan.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
