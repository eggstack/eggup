# Acquisition Transport Milestone 001 — Acquisition Seam and Deterministic Fixture Transport

Status: implemented (closed; see `plans/closure/acquisition-transport/001-status.md`)

Repository baseline for planning: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#42-acquisition-adapters`
- `plans/000-long-term-specification.md#8-acquisition-safety`

Applicable ADRs:

- ADR-0001
- ADR-0003

Primary class: infrastructure

## 1. Objective

Define the smallest transport-neutral acquisition contract that can stream an artifact into Eggup-owned staging without selecting release, fallback, version, destination, or service policy.

## 2. Why this milestone is blocked

The seam should be written against the corrected stage/ownership API produced by core M005.

Do not bind transport to the known-defective pre-corrective API.

## 3. Invariants

- `eggup-core` remains transport-free;
- transport never decides release/fallback policy;
- transport never executes downloaded content;
- metadata bodies are bounded;
- artifacts stream to a file/sink rather than requiring full buffering;
- partial downloads are cleaned or remain clearly transaction-owned;
- not-found is distinguishable from hard transport failure;
- URLs/secrets are redacted in diagnostics;
- correctness tests use deterministic local fixtures.

## 4. Scope

### In scope

- new transport package boundary if needed, or a transport trait owned by `eggup-eggfetch` support code without contaminating core;
- typed request descriptors;
- typed `Success | NotFound | Failure` status;
- bounded small-body fetch;
- streamed file fetch;
- cancellation/drop cleanup semantics;
- deterministic in-memory/local-fixture transport;
- progress callback/event shape only if required by a real consumer and kept presentation-neutral.

### Explicitly out of scope

- Eggfetch implementation itself;
- GitHub/crates.io metadata policy;
- retries;
- Cargo fallback;
- mirrors;
- service lifecycle;
- latest-version selection.

## 5. Required design

Prefer a synchronous or async shape based on Eggfetch's native API and consumer needs, but do not force async into `eggup-core`.

The seam must allow an adapter to:

- receive an exact URL/source selected by caller policy;
- enforce caller-provided byte/time bounds;
- write to a transaction-owned destination;
- return status/metadata needed by the caller;
- avoid exposing transport-specific response types through the public generic contract.

If the abstraction becomes more complex than the two real required operations—bounded metadata and streamed artifact—stop and simplify.

## 6. Ordered work packages

A. Consumer census against eggsact/stegoeggo/eggsearch current download calls.

B. Minimal request/result model.

C. Deterministic fixture transport.

D. Partial-download and bound tests.

E. Package/API docs.

## 7. Failure semantics

- NotFound is data, not fallback;
- TLS/proxy/timeout/5xx/malformed response are hard failures;
- partial artifact is not promoted to verified candidate state;
- caller cancellation cleans owned incomplete output where safe.

## 8. Required tests

- exact body success;
- streamed artifact success;
- metadata limit;
- artifact limit if configured;
- not-found classification;
- timeout/cancellation abstraction;
- partial write failure;
- redaction;
- no fallback invocation.

## 9. Acceptance criteria

The seam is small enough that an Eggfetch implementation and a lightweight implementation could both satisfy it without policy leakage.

## 10. Stop conditions

Stop if the contract requires GitHub JSON types, SemVer, Cargo, service manager, or a second staging model independent of core.

## 11. Closure evidence required

- API surface;
- consumer-call mapping;
- fixture tests;
- dependency tree;
- proof core manifest unchanged with respect to transport.
