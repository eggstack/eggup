# Consumer Adoption and Compatibility Roadmap

Status: active; simple and eggsearch tiers closed, CodeGG M005 ready, Gregg remains footprint-gated

Long-term references:

- `plans/000-long-term-specification.md#22-consumer-adoption`
- `plans/002-long-term-roadmap.md#phase-5--first-real-consumer-adoption`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

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
 acquisition M004      service M003
 corrective            corrective
          \                   /
           +--------+--------+
                    |
          +---------+---------+
          |                   |
          v                   v
    M003 eggsearch       M004 Gregg
       [closed]        [blocked independently]

qualified core M006 + acquisition M004
                    |
                    v
              M005 CodeGG [ready]
                    |
                    +--> multi-artifact consumer evidence

M006 Egress remains independently blocked on the generic consumer-side archive extraction/update contract.

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

Plan: `plans/implementation/consumer-adoption/003-eggsearch-service-aware-adoption.md`.

Adopt corrected core/acquisition plus the Unix service substrate after acquisition M004 and service M003 closure. Preserve eggsearch-owned release/Cargo fallback, health, cron watchdog, and Windows-specific behavior until the corresponding shared layers are qualified.

Closed by `plans/closure/consumer-adoption/003-status.md`. Eggsearch uses the immutable Eggup revision for its shared updater and Unix manager paths; Windows self-replacement/SCM remain consumer-owned. The measured release binary increase is 12.6%; Gregg's separate footprint gate remains open.

### M004 — Gregg adoption

Replace `gregg-update` with Eggup mechanics after remeasuring the acquisition-M004-corrected Eggfetch path against Gregg's footprint budget. Create lightweight acquisition M005 only if the evidence still justifies it. Detailed adoption plan waits for that decision.

### M005 — CodeGG managed-runfile bundle adoption

Plan: `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md`.

Status: ready.

Hard dependencies: verified-update-core M006 and acquisition M004 are closed.

Consumer evidence: `dbowm91/codegg` currently has a check-only self-update path because its earlier dependency-security M005 correctly refused to copy a Gregg-specific updater. Its prebuilt release is one verified `.tar.gz` managed bundle containing `codegg`, `codegg-sandbox-helper`, and `codegg-eggsearch` plus an optional fixed notice.

Adopt Eggup for bounded acquisition contracts and the multi-artifact local transaction. Keep release/version/target policy and strict archive extraction in CodeGG. The archive is verified before extraction; member digests derived from that verified snapshot preserve integrity continuity through Eggup staging/commit. Preserve CodeGG's current Eggfetch/WebPKI trust profile rather than blindly enabling a broader adapter feature set.

This milestone does not depend on core M007 because CodeGG requires immediate verified bundle replacement, not service/post-install rollback orchestration. It may execute in parallel with M007.

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
| M003 eggsearch | closed | `plans/implementation/consumer-adoption/003-eggsearch-service-aware-adoption.md` | `plans/closure/consumer-adoption/003-status.md` | — |
| M004 Gregg | blocked / plan intentionally unwritten | — | — | corrected-path footprint decision |
| M005 CodeGG | ready | `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md` | — | verified-update-core M006 + acquisition M004 closed |
| M006 Egress | blocked | — | — | archive update/extraction transaction contract |
| M007 EggPool | deferred/evidence-driven | — | — | core maturity |
