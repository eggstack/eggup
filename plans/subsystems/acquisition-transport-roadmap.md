# Acquisition Transport Roadmap

Status: M001-M006 closed; M007 boundary-safety corrective ready

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
- lightweight external-curl adapter;
- explicit caller-selected transport composition/fallback policy.

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

M004 is closed. Read-only review of `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` supplied the real-consumer evidence previously required for M005. M005 is closed and fully cross-platform qualified with corrective M006: hosted run `36169295410` exposed a Windows all-targets test portability failure in `eggup-curl`, corrected by `1c601f2`, with green hosted matrix `36176009068`. Gregg remains untouched.

A 2026-09-26 post-closure audit opened M007. Three boundary defects remain: fixed-byte Unicode diagnostic truncation can panic at a non-character boundary; `FetchLimits::max_artifact_bytes = None` permits an unbounded artifact-byte state contrary to the normative bounded-acquisition contract; and the curl adapter closes its exclusively-created temp before curl reopens the pathname, weakening the intended pathname-race guarantee. M007 is ready and must close before another transport-dependent consumer migration is treated as fully qualified.

## 5. Target architecture

`eggup-eggfetch` should depend on a stable Eggfetch version with a deliberately narrow feature set. `eggup-curl` should provide the complementary external-process path without embedding an HTTP/TLS implementation.

The acquisition interface should represent operations such as bounded small-body fetch and streamed file download, returning typed status without encoding release policy. A caller-selected composition layer may choose curl, Eggfetch, or a preferred order; transport fallback must remain distinct from release/source fallback, and exact `NotFound` is terminal for the requested URL.

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
           `--> M005 curl adapter + explicit transport composition [closed; qualified with M006]
                     |
                     v
               M006 Windows portability/qualification corrective [closed]
                     |
                     v
               M007 boundary safety hardening [READY]
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

### M005 — Curl adapter and explicit transport composition

Class: infrastructure/capability.

Plan: `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`.

Hard dependency: M004 closure.

Use Gregg only as read-only behavioral evidence for a mature bounded external-curl path. Add `eggup-curl` plus caller-selected curl/Eggfetch/preferred-fallback composition. Preserve exact `NotFound` as terminal for the requested URL; default composition falls back only when the preferred transport is unavailable, while broader transport-error fallback requires explicit caller policy. Record curl-only, Eggfetch-only, and dual-transport footprint evidence. No Gregg migration occurs in M005.

### M006 — M005 Windows portability and cross-closure qualification corrective

Class: invariant/corrective.

Plan: `plans/implementation/acquisition-transport/006-m005-windows-portability-and-cross-closure-qualification-corrective.md`.

Status: closed; see `plans/closure/acquisition-transport/006-status.md`.

Corrects the Windows `--all-targets` failure caused by Unix-only `PermissionsExt` test support in `eggup-curl`, cleans the Windows-only core import warnings, requires a green hosted matrix, and reconciles M005/M006 closure evidence plus stale roadmap/registry state. Gregg remains read-only and consumer M004 remains unwritten.

### M007 — Boundary safety hardening corrective

Class: invariant/corrective.

Plan: `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`.

Status: ready.

Close UTF-8-unsafe diagnostic truncation, require a finite artifact byte budget, and remove the curl exclusive-temp pathname reopen race by retaining Eggup-owned output authority while streaming the child body. Preserve existing fallback, redaction, no-clobber, timeout, cancellation, and footprint semantics.

## 8. Cross-cutting requirements

Transport adapters must preserve bounded body/output behavior, redaction, cancellation cleanup, and partial-file cleanup. Proxy behavior must be explicit rather than inherited accidentally. Transport fallback is never release/source fallback: `NotFound`, verification failure, cancellation, size-limit failure, and staging/promotion failure do not silently select another source.

## 9. Verification strategy

Local HTTP fixtures for redirect, downgrade, 404, 5xx, timeout, truncated body, oversized metadata, proxy misconfiguration, and interrupted download.

## 10. Risks and decision points

Eggfetch version/feature choice may materially affect binary size. Measure rather than assume. Do not fork Eggfetch policy into Eggup without an upstream-level need.

## 11. Completion definition

The subsystem's primary path is complete when M007 closes, the corrected native Eggfetch path and lightweight curl path both satisfy the common bounded acquisition contract, caller-selected composition is qualified, core remains transport-neutral, and no medium-or-higher acquisition safety/contract issue remains.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md` | `plans/closure/acquisition-transport/001-status.md` | — |
| M002 | closed; post-closure findings fed M003 | `plans/implementation/acquisition-transport/002-eggfetch-adapter.md` | `plans/closure/acquisition-transport/002-status.md` | — |
| M003 | closed; post-closure findings feed M004 | `plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md` | `plans/closure/acquisition-transport/003-status.md` | — |
| M004 | closed | `plans/implementation/acquisition-transport/004-validated-limits-and-promotion-state-corrective.md` | `plans/closure/acquisition-transport/004-status.md` | — |
| M005 | closed; qualified with M006 corrective | `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md` | `plans/closure/acquisition-transport/005-status.md` | — |
| M006 | closed | `plans/implementation/acquisition-transport/006-m005-windows-portability-and-cross-closure-qualification-corrective.md` | `plans/closure/acquisition-transport/006-status.md` | — |
| M007 | ready | `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md` | — | — |
