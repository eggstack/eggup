# Consumer Adoption and Compatibility Roadmap

Status: active; existing adopters closed; the versioned Core handoff shipped 2026-10-06 (Acquisition M010, Core M012/M013), so Gregg M004 and EggPool M007 are now authorable and remain intentionally unwritten

Long-term references:

- `plans/000-long-term-specification.md#22-consumer-adoption`
- `plans/002-long-term-roadmap.md#phase-5--first-real-consumer-adoption`
- `plans/002-long-term-roadmap.md#phase-8--multi-artifact-codegg-adoption`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

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
| Gregg | shared gregg-update + mature daemon-update lifecycle | future replacement of local generic machinery | transport selection/footprint; Cargo fallback; exact executable/config runtime semantics |
| CodeGG | verified in-place managed-runfile update on supported Linux/macOS; manual fresh-install guidance elsewhere | first multi-artifact bundle adopter; generalized updater blocker resolved | 3-runfile bundle |
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
     M003 eggsearch       upstream M005 acquisition + M006 service
        [closed]              [closed; Gregg reference only]
                                       |
                                       v
                                 M004 Gregg
                [BLOCKED on Core M012/M013 + Acquisition M010;
                 plan intentionally unwritten]

qualified core M006 + acquisition M004
                    |
                    v
               M005 CodeGG [closed]
                    |
                    +--> multi-artifact consumer evidence

M006 Egress was unblocked by Archive Extraction M001d closure (`plans/closure/archive-extraction/001d-status.md`, hosted run `36335233644`) and its adoption plan is registered at `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`. Verified Core M009 closed the publication gate on 2026-09-28 (`eggup-core 0.1.2` + `eggup-archive 0.1.2` registry-visible; `plans/closure/verified-update-core/009-status.md`), and Egress Delivery M003 landed the adoption on 2026-09-29 with hosted Linux/macOS/Windows updater evidence — M006 is closed (see `plans/closure/consumer-adoption/006-status.md` M003-landed addendum). No git/path dependency entered Egress's publishable release state.

M007 EggPool selective adoption remains evidence-driven. Core M010/M011 are closed; its standalone-Rust branch becomes authorable only after Core M012 preserves the 0.1.x boundary and M013 publishes the versioned Core. uv/pipx/pip provenance and package-manager rollback remain EggPool-owned.
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

Closed by `plans/closure/consumer-adoption/003-status.md`. Eggsearch uses the immutable Eggup revision for its shared updater and Unix manager paths; Windows self-replacement/SCM remain consumer-owned. The measured release binary increase was 12.6%. Gregg's former footprint/transport prerequisite was subsequently resolved by acquisition M005/M006, and the versioned Core handoff (Acquisition M010, Core M012/M013) shipped 2026-10-06; Gregg M004 is authorable but remains intentionally unwritten.

### M004 — Gregg adoption

Do not execute the Gregg migration yet. Gregg was a read-only reference/test oracle for two upstream Eggup milestones:

- acquisition M005: external curl adapter + explicit curl/Eggfetch transport composition;
- service M006: managed/direct/stopped/foreign-preserved daemon update disposition and revalidation semantics.

The parity gaps themselves are now closed and hosted-green: Core M010/M011 and Service M009 are complete. The release boundary Gregg was waiting on is safe and versioned as of 2026-10-06: Core M012 preserved 0.1.x source compatibility (`Error` keeps the seven variants published in `0.1.2`), Acquisition M010 published the corrected acquisition seam, and Core M013 published the resulting Core as `eggup-core 0.1.3`. **Gregg M004 is authorable** and remains intentionally unwritten. Gregg-owned release/version/target/Cargo-fallback semantics remain downstream policy.

### M005 — CodeGG managed-runfile bundle adoption

Plan: `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md`.

Status: closed; see `plans/closure/consumer-adoption/005-status.md`.

Hard dependencies: verified-update-core M006 and acquisition M004 are closed.

Consumer evidence: CodeGG's check-only updater blocker is closed through its immutable-pinned Eggup adoption follow-up. Its prebuilt release is one verified `.tar.gz` managed bundle containing `codegg`, `codegg-sandbox-helper`, and `codegg-eggsearch` plus an optional fixed notice.

Adopt Eggup for bounded acquisition contracts and the multi-artifact local transaction. Keep release/version/target policy and strict archive extraction in CodeGG. The archive is verified before extraction; member digests derived from that verified snapshot preserve integrity continuity through Eggup staging/commit. Preserve CodeGG's current Eggfetch/WebPKI trust profile rather than blindly enabling a broader adapter feature set.

This milestone does not depend on core M007 because CodeGG requires immediate verified bundle replacement, not service/post-install rollback orchestration. It was completed first in the requested sequential batch; Verified Update Core M007 follows.

### M006 — Egress archive/pair adoption

Plan: `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`.

Status: closed 2026-09-29 by Egress Delivery M003 (registry `eggup-core`/`eggup-archive` 0.1.2 adopted, hosted Linux/macOS/Windows updater evidence; see `plans/closure/consumer-adoption/006-status.md` M003-landed addendum). The former stale cross-repo evidence was corrected by planning-hygiene C009 (closed).

Replace duplicated generic tar.gz/zip extraction and pair rollback with the qualified extraction boundary plus Eggup's existing multi-artifact transaction. Preserve Egress-owned GitHub release/version/origin, checksum, candidate-version, CLI, and provenance policy.

### M007 — EggPool selective adoption

Adopt only low-level primitives that reduce ownership without weakening provenance/package-manager logic.

The 2026-10-05 review of EggPool `fe3c308d` confirms that only the `StandaloneRust` local executable transaction is a clean Eggup target. The `UvTool`, `Pipx`, `PipEnvironment`, source-checkout, release catalog, PEP-440-like ordering, DB/config compatibility and package-manager rollback paths remain EggPool-owned. Core M010/M011 are closed, and the 0.1.x compatibility boundary closed with M012 and the versioned Core handoff published with M013 on 2026-10-06; **M007 is authorable** and remains intentionally unwritten.

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
| M004 Gregg | authorable; plan intentionally unwritten | — | — | Every prior gate is closed — Service M009, Core M010/M011/M012/M013, Acquisition M010 (`eggup-core 0.1.3` + `eggup-acquisition 0.1.3` published 2026-10-06) |
| M005 CodeGG | closed | `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md` | `plans/closure/consumer-adoption/005-status.md` | — |
| M006 Egress | closed 2026-09-29 | `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md` | `plans/closure/consumer-adoption/006-status.md` (+ M003-landed addendum) | — (Egress Delivery M003 landed) |
| M007 EggPool | authorable/evidence-driven; plan intentionally unwritten | — | — | Core M012 + M013 closed 2026-10-06; package-manager/provenance paths remain explicitly excluded |
