# Acquisition Transport Roadmap

Status: active planning; implementation blocked on verified-update-core M005 corrective closure

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

Eggstack has two relevant transport families:

- eggsact, stegoeggo, eggsearch, and CodeGG already use Eggfetch or are natural Eggfetch consumers;
- Gregg deliberately retained external curl after measuring substantial binary-size growth from an Eggfetch/TLS adoption experiment.

The subsystem must support both realities.

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
          +--> consumer adoption eggsact/stegoeggo
          |
          `--> optional M003 lightweight/curl adapter (evidence-driven)
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

### M003 — Lightweight acquisition adapter

Class: optional infrastructure.

Create only if Gregg or another real consumer still demonstrates a footprint requirement after Eggfetch integration is remeasured.

## 8. Cross-cutting requirements

Transport adapters must preserve bounded body/output behavior, redaction, cancellation cleanup, and partial-file cleanup. Proxy behavior must be explicit rather than inherited accidentally.

## 9. Verification strategy

Local HTTP fixtures for redirect, downgrade, 404, 5xx, timeout, truncated body, oversized metadata, proxy misconfiguration, and interrupted download.

## 10. Risks and decision points

Eggfetch version/feature choice may materially affect binary size. Measure rather than assume. Do not fork Eggfetch policy into Eggup without an upstream-level need.

## 11. Completion definition

Two real consumers successfully acquire artifacts through the same adapter while core remains transport-neutral.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | blocked | `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md` | — | core M005 |
| M002 | blocked | `plans/implementation/acquisition-transport/002-eggfetch-adapter.md` | — | transport M001 |
| M003 | deferred/evidence-driven | — | — | real footprint evidence |
