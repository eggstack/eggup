# Acquisition Transport Roadmap

Status: active corrective; M001-M003 closed, M004 corrective ready for handoff

Long-term references:

- `plans/000-long-term-specification.md#42-acquisition-adapters`
- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

## 1. Purpose and ownership boundary

This subsystem owns bounded byte acquisition adapters. It does not choose releases, versions, fallback sources, destinations, or commit policy.

## 2. Work classification

### Invariants

- core remains transport-free;
- adapters never silently choose fallback;
- redirects/timeouts/proxies/TLS behavior are explicit;
- downloaded artifacts are data, not shell code to execute;
- error output redacts credential-bearing URL material.

### Capabilities

- fetch bounded metadata;
- stream artifacts to an Eggup-owned stage;
- classify not-found versus hard failure;
- run deterministic local fixture tests.

### Infrastructure

- small acquisition trait/seam;
- eggup-eggfetch;
- test transport;
- optional lightweight adapter if justified.

### Polish

- progress events;
- richer error categories;
- footprint measurement.

## 3. Non-goals

- release discovery policy;
- SemVer selection;
- Cargo fallback;
- mirrors selected from environment by default;
- automatic retries unless explicitly designed later.

## 4. Current state

M001 and M002 are closed. `eggup-acquisition 0.1.0` and `eggup-eggfetch 0.1.0` are published, and both eggsact and stegoeggo use the shared acquisition path.

M003 closed the original timeout/temp-file/redaction findings and the corrected path remains green in eggsact and stegoeggo.

Post-closure review then found two narrower correctness gaps tracked by M004:

- `FetchLimits::new` validates limits, but the struct fields remain public, so direct struct literals can bypass validation unless each transport revalidates at its trust boundary;
- no-clobber promotion hard-links the complete temp to `dest` and then returns `Err` if unlinking the redundant temp fails, which can report ordinary failure after a complete destination already exists.

Broader updater-bearing adoption is gated on M004. Gregg still provides evidence for a possible lightweight transport after this corrective, but that remains optional and footprint-driven.

## 5. Target architecture

`eggup-eggfetch` should depend on a stable Eggfetch version with a deliberately narrow feature set.

The acquisition interface should represent operations such as bounded small-body fetch and streamed file download, returning typed status without encoding release policy.

A test adapter/local fixture path must exist so correctness never requires public GitHub.

## 6. Dependency graph

```text
verified-update-core M005 corrected public boundary
          |
          v
M001 acquisition seam + deterministic test adapter
          |
          v
M002 eggup-eggfetch
          |
          +--> first adopters eggsact/stegoeggo [closed]
          |
          v
M003 contract/temp-file hardening corrective [closed]
          |
          v
M004 validated-limits/promotion-state corrective
          |
          +--> broader updater-bearing adoption
          |
          `--> optional M005 lightweight/curl adapter (evidence-driven)
```

## 7. Milestones

### M001 — Acquisition seam and fixture transport

Class: infrastructure.

Hard dependency: verified-update-core M005 corrective closure.

Plan: `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md`.

Exit: transport contract can write an acquired artifact into the corrected staging boundary without knowing release policy.

### M002 — Eggfetch acquisition adapter

Class: capability/infrastructure.

Plan: `plans/implementation/acquisition-transport/002-eggfetch-adapter.md`.

Hard dependency: M001 closure.

Exit: strict HTTPS/redirect/proxy/timeout behavior is locally tested and dependency impact measured.

### M003 — Acquisition contract and temporary-file hardening corrective

Class: invariant/corrective.

Plan: `plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md`.

Correct per-request timeout enforcement, exclusive/private temporary files, no-clobber promotion, cleanup ownership, and diagnostic redaction before broader adoption.

### M004 — Validated limits and promotion-state corrective

Class: invariant/corrective.

Plan: `plans/implementation/acquisition-transport/004-validated-limits-and-promotion-state-corrective.md`.

Require boundary revalidation of public `FetchLimits` values and truthful post-promotion terminal state before broader updater-bearing adoption.

### M005 — Lightweight acquisition adapter

Class: optional infrastructure.

Create only if Gregg or another real consumer still demonstrates a footprint requirement after the M004-corrected Eggfetch path is remeasured.

## 8. Cross-cutting requirements

Transport adapters must preserve bounded body/output behavior, redaction, cancellation cleanup, and partial-file cleanup. Proxy behavior must be explicit rather than inherited accidentally.

## 9. Verification strategy

Local HTTP fixtures for redirect, downgrade, 404, 5xx, timeout, truncated body, oversized metadata, proxy misconfiguration, and interrupted download.

## 10. Risks and decision points

Eggfetch version/feature choice may materially affect binary size. Measure rather than assume. Do not fork Eggfetch policy into Eggup without an upstream-level need.

## 11. Completion definition

The subsystem's primary path is complete when two real consumers remain green on the corrected M004 acquisition contract, core remains transport-neutral, and no medium-or-higher acquisition safety/contract issue remains. The lightweight M005 adapter is optional and evidence-driven.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md` | `plans/closure/acquisition-transport/001-status.md` | — |
| M002 | closed; post-closure findings fed M003 | `plans/implementation/acquisition-transport/002-eggfetch-adapter.md` | `plans/closure/acquisition-transport/002-status.md` | — |
| M003 | closed; post-closure findings feed M004 | `plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md` | `plans/closure/acquisition-transport/003-status.md` | — |
| M004 | **ready for handoff** | `plans/implementation/acquisition-transport/004-validated-limits-and-promotion-state-corrective.md` | — | — |
| M005 | deferred/evidence-driven | — | — | corrected-path footprint evidence |
