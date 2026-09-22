# Consumer Adoption and Compatibility Roadmap

Status: active planning; first adoption blocked on corrected core qualification and Eggfetch adapter

Long-term references:

- `plans/000-long-term-specification.md#22-consumer-adoption`
- `plans/002-long-term-roadmap.md#phase-5--first-real-consumer-adoption`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

## 1. Purpose and ownership boundary

This workstream proves Eggup against real Eggstack consumers and records compatibility requirements that cannot be discovered from an isolated library.

Eggup owns generic fixes revealed by adoption. Consumer repositories own their adapters, release policy, CLI behavior, and application migration semantics.

## 2. Work classification

### Invariants

- adoption deletes duplicated generic machinery rather than adding a wrapper around it;
- consumer release authority/version semantics stay consumer-owned;
- no consumer gains a second heavy HTTP/TLS stack without justification;
- existing safety behavior is preserved or intentionally strengthened;
- migration is independently reversible until the new path qualifies.

### Capabilities

- single-binary native verified update;
- service-aware update;
- multi-artifact bundle update;
- archive/two-binary update;
- selective low-level reuse in complex hybrid installers.

### Infrastructure

- consumer adapters and compatibility fixtures;
- versioned Eggup dependencies;
- release test seams.

### Polish

- migration docs;
- common examples;
- compatibility matrix.

## 3. Non-goals

- forcing all repos to identical commands;
- synchronizing release versions;
- making Eggup a monorepo dependency;
- importing consumer-specific migrations into Eggup.

## 4. Current-state matrix

| Consumer | Current pattern | Eggup value | Special constraint |
|---|---|---|---|
| eggsact | Eggfetch + single binary + checksum/Cargo fallback | first simple adopter | preserve explicit fallback conditions |
| stegoeggo | Eggfetch + single binary | second simple adopter | Rust 1.89 baseline |
| eggsearch | Eggfetch + updater + service lifecycle | core + service adopter | health/manager semantics |
| Gregg | shared gregg-update + service lifecycle | replace local shared crate | transport footprint; Cargo fallback |
| CodeGG | check-only update; verified bundle installer | resolve generic updater blocker | 3-runfile bundle |
| Egress | GitHub authority + archive + 2 binaries | multi-artifact/archive proof | no Cargo fallback |
| EggPool | rich provenance + package-manager transitions | selective primitives | PEP-440-like/version/provenance policy |

## 5. Target architecture

Each consumer contains only a thin adapter translating its release policy into Eggup domain types and mapping Eggup outcomes into its CLI/lifecycle presentation.

## 6. Dependency graph

```text
core M005 corrective + M006 qualification + transport M002
            |
            +--> M001 eggsact
            |
            `--> M002 stegoeggo
                    |
                    v
          generic API correction/qualification
                    |
          +---------+---------+
          |                   |
          v                   v
    M003 eggsearch       M004 Gregg
          |
          +------------------------+
                                   v
                             M005 CodeGG
                                   |
                                   v
                             M006 Egress

M007 EggPool selective adoption is independent/evidence-driven after core maturity.
```

## 7. Milestones

### M001 — eggsact adoption

Plan: `plans/implementation/consumer-adoption/001-eggsact-first-adoption.md`.

Hard dependencies: verified-update-core M006 and acquisition-transport M002.

Delete local generic updater mechanics in favor of Eggup core + Eggfetch while retaining eggsact release/fallback policy.

### M002 — stegoeggo adoption

Plan: `plans/implementation/consumer-adoption/002-stegoeggo-second-adoption.md`.

Hard dependency: M001 eggsact adoption closure.

Second independent single-binary consumer; use it to identify accidental eggsact assumptions.

### M003 — eggsearch adoption

Adopt core first, then service substrate after service roadmap qualifies.

### M004 — Gregg adoption

Replace `gregg-update` with Eggup mechanics while allowing measured lightweight transport.

### M005 — CodeGG bundle adoption

Resolve CodeGG M005 external interface blocker with native verified multi-runfile replacement.

### M006 — Egress archive/pair adoption

Prove archive/member and two-binary transactional semantics.

### M007 — EggPool selective adoption

Adopt only low-level primitives that reduce ownership without weakening provenance/package-manager logic.

## 8. Cross-cutting requirements

Every adoption records pre/post dependency tree, binary size where relevant, removed duplicated lines/modules, behavior differences, fixture coverage, and rollback path.

## 9. Verification strategy

Run each consumer's existing release/update/install tests plus focused Eggup compatibility tests. Do not claim generic maturity until at least two independent consumers close.

## 10. Risks and decision points

API instability discovered during first adoption should be fixed before broad migration. Avoid compatibility shims in Eggup that merely encode one consumer's historical quirks.

## 11. Completion definition

The roadmap closes when simple, service-aware, and multi-artifact consumers use Eggup and the remaining exceptions are explicit rather than accidental duplication.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 eggsact | closed | `plans/implementation/consumer-adoption/001-eggsact-first-adoption.md` | `plans/closure/consumer-adoption/001-status.md` | — |
| M002 stegoeggo | closed | `plans/implementation/consumer-adoption/002-stegoeggo-second-adoption.md` | `plans/closure/consumer-adoption/002-status.md` | — |
| M003 eggsearch | planned | — | — | first adoptions, service roadmap |
| M004 Gregg | planned | — | — | first adoptions |
| M005 CodeGG | planned | — | — | mature multi-artifact core |
| M006 Egress | planned | — | — | core/archive contract |
| M007 EggPool | deferred/evidence-driven | — | — | core maturity |
