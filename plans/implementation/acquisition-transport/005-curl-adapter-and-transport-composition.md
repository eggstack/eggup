# Acquisition Transport Milestone 005 — Curl Adapter and Explicit Transport Composition

Status: ready for handoff

Repository baseline: `881c95ff069d3d465a282cb6a495ba6fcb70cb6f`

Reference implementation reviewed: `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` (`crates/gregg-update/src/exec.rs`). Gregg is reference/test evidence only; this milestone MUST NOT modify or depend on Gregg.

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#42-acquisition-adapters`
- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Primary class: infrastructure

## 1. Objective

Add a lightweight external-`curl` acquisition adapter and an explicit caller-selected transport-composition boundary so a consumer may choose curl only, Eggfetch only, curl preferred with Eggfetch fallback, or Eggfetch preferred with curl fallback.

The transport layer MUST continue to fetch only exact caller-selected URLs. It MUST NOT select releases, versions, mirrors, Cargo/source fallback, install destinations, or update policy.

The mature bounded `curl` machinery in Gregg is the behavioral reference for process execution, timeout/reaping, HTTP status classification, and partial-output cleanup. Eggup's existing acquisition invariants remain authoritative where they are stricter.

## 2. Why this milestone is ready

Acquisition M001-M004 are closed. The seam now has exact URL requests, validated byte/time limits at every trust boundary, typed `Success | NotFound`, private temporary-file staging, no-clobber promotion, cancellation cleanup, diagnostic URL redaction, and a qualified Eggfetch adapter.

The previous M005 gate asked for a real consumer demonstrating that an embedded HTTP/TLS stack can be undesirable. Gregg provides that evidence at the reviewed reference commit: its updater deliberately uses a bounded external `curl` process, and Eggup's consumer roadmap already records transport footprint as a Gregg constraint.

No Gregg migration is required to implement or qualify this adapter.

## 3. Current implementation evidence

At the Eggup baseline:

- `eggup-acquisition::AcquisitionTransport` is transport-neutral and receives exact caller-selected URLs;
- `FetchLimits` expresses metadata/artifact byte ceilings plus connection/total time bounds;
- `FetchOutcome::NotFound` is data and does not trigger fallback inside a transport;
- the fixture transport and `eggup-eggfetch` already qualify cleanup/redaction/cancellation semantics.

At the Gregg reference baseline, `gregg-update/src/exec.rs` demonstrates production behavior worth upstreaming:

- discover `curl`/`curl.exe`;
- invoke it directly without a shell or `sudo`;
- bound request time and file size;
- kill and reap a hung process;
- capture HTTP status from the same artifact request rather than issuing a second probe;
- distinguish exact HTTP 404 from TLS/5xx/timeout/spawn failure;
- remove partial output on failure.

Gregg writes directly to its own staging destination and has Gregg-specific error/output policy. Those pieces MUST be adapted to Eggup's stricter temporary-file, no-clobber, cancellation, and redaction contracts rather than copied verbatim.

## 4. Invariants that must not regress

- `eggup-core` remains transport-free.
- Downloaded bytes are data and are never executed by the acquisition layer.
- No shell, `sudo`, privilege escalation, package manager, or release-selection command is invoked.
- Exact HTTP 404 remains `FetchOutcome::NotFound`; it never silently selects another release/source.
- A transport chain never retries `NotFound` against another transport for the same exact URL.
- Integrity/signature/candidate validation failures are outside transport fallback and are never retried through another transport.
- Existing destination files are never overwritten.
- Partial downloads never become promoted artifacts.
- All subprocess output, wall time, connection time, redirects, and artifact bytes remain bounded.
- Cancellation kills/reaps the owned curl child before return.
- Credential-bearing URL material is redacted from errors and diagnostics.
- Proxy behavior is explicit rather than accidentally inherited.
- Consumers can choose curl without linking Eggfetch, and Eggfetch without linking curl.

## 5. Scope

### In scope

- a new workspace crate, expected to be named `eggup-curl`, implementing `AcquisitionTransport`;
- explicit construction from a caller-supplied curl executable path plus an opt-in discovery helper;
- bounded direct-process execution with kill/reap on timeout and cancellation;
- metadata capture and artifact acquisition;
- exact HTTP status classification from the request that transfers the body;
- reuse or extraction of Eggup-owned private-temp/no-clobber promotion helpers where necessary;
- a typed transport-unavailable condition if required for safe composition;
- a transport composition helper in `eggup-acquisition` (or a comparably narrow sibling surface) with caller-selected preferred order and fallback policy;
- footprint evidence for curl-only, Eggfetch-only, and dual-transport representative binaries;
- deterministic local fixture/process tests requiring no public GitHub access;
- documentation and package qualification.

### Explicitly out of scope

- modifying, depending on, or migrating `eggstack/gregg`;
- release discovery or SemVer/latest selection;
- an `update [VERSION]` CLI contract;
- Cargo fallback after a missing binary;
- retrying different URLs, mirrors, channels, or releases;
- libcurl bindings;
- producer packaging/release-manifest generation;
- changing `eggup-core`;
- executing downloaded content;
- hidden retries.

## 6. Required production changes

### 6.1 `eggup-curl`

Implement `AcquisitionTransport` without an embedded HTTP/TLS library.

Executable selection must be explicit. A caller-provided executable path is the strongest path. A convenience discovery function MAY search `PATH`, but discovery must be opt-in and return bounded non-secret evidence.

For each request:

- validate `FetchLimits` at the adapter boundary;
- pass explicit connect/total timeout controls to curl;
- maintain an independent parent-process wall deadline and cancellation check;
- cap captured stdout/stderr;
- use explicit redirect and protocol policy;
- make proxy policy an explicit adapter configuration;
- download artifacts only to an Eggup-owned private temporary sibling;
- enforce artifact byte ceilings both through curl where supported and by Eggup-side validation;
- promote with the existing no-clobber terminal-state contract only after a complete successful transfer.

Do not issue a separate HEAD/status probe for artifact existence.

### 6.2 Transport availability/error classification

If the current acquisition error model cannot distinguish “adapter unavailable” from “adapter attempted the request and failed,” add a non-exhaustive typed category such as `Unavailable`.

Do not encode curl command lines, raw proxy credentials, or unredacted URLs in public errors.

### 6.3 Caller-selected composition

Provide a small composition type rather than embedding fallback into either concrete adapter.

The composition contract must represent preferred transport order, fallback on adapter unavailability, and optional explicitly requested fallback on ordinary transport failure.

Regardless of policy:

- `NotFound` is terminal for the exact URL;
- invalid input, cancellation, byte-limit violation, and local staging/promotion failure are terminal;
- verification/candidate failures occur above this layer and are never transport-fallback inputs.

The default composition policy should fall back only when the preferred adapter is unavailable. Broader retry on transport failure requires an explicit caller choice.

### 6.4 Footprint fixture

Add a small non-published test/example/fixture surface producing comparable release binaries for acquisition seam + curl, acquisition seam + Eggfetch, and acquisition seam + both/composition.

Record stripped release sizes and dependency trees in closure evidence. Measurement is evidence, not a permanent threshold unless a later ADR establishes one.

## 7. Ordered work packages

1. Add failing composition-contract tests, especially terminal `NotFound`, cancellation, and fallback call counts.
2. Add/adjust typed unavailable classification without changing existing `NotFound` semantics.
3. Factor only the minimum reusable private-temp/promotion machinery needed by a second production adapter.
4. Implement curl executable selection/discovery with deterministic test doubles.
5. Implement bounded metadata capture.
6. Implement artifact download to owned temporary storage and no-clobber promotion.
7. Add explicit redirect/proxy/protocol configuration and negative tests.
8. Implement preferred/fallback composition.
9. Add footprint fixtures and record measurements.
10. Run package/docs/MSRV/platform qualification and update closure evidence.

## 8. Failure, cancellation, restart, and contention semantics

There is no restart/service behavior in this milestone.

A cancelled or timed-out curl process must be killed and reaped before return. Owned partial files must be removed where safe; a destination promoted successfully must never later be reported as an ordinary pre-promotion failure.

A destination race remains a no-clobber failure. The adapter must not delete a destination it did not create.

Fallback must be decided from typed outcomes, not by parsing human-readable errors.

## 9. Compatibility and migration

Existing `eggup-eggfetch` consumers require no migration. This is an additive transport option.

The `AcquisitionTransport` contract should remain source-compatible where practical. Adding a variant to a `#[non_exhaustive]` error is acceptable if required.

No downstream repository is changed. Gregg is a read-only behavioral reference only.

## 10. Required tests

At minimum:

- curl executable absent/discovery failure and spawn failure;
- metadata 2xx/404/5xx;
- artifact 2xx/404/5xx and TLS-like process failure;
- truncated transfer;
- metadata and artifact size overflow;
- connect timeout, total timeout, and parent wall timeout;
- cancellation with child kill/reap;
- redirect allowed/rejected cases;
- proxy disabled versus explicitly enabled behavior;
- temp-file permissions/ownership where testable;
- no-clobber destination race;
- cleanup before promotion and truthful post-promotion result;
- URL/query credential redaction;
- preferred-order call counts;
- fallback on unavailable;
- default policy does not retry ordinary transport failures;
- explicitly broad policy may retry eligible transport failures;
- no policy retries `NotFound`, cancellation, too-large, invalid input, or staging/promotion failure;
- Eggfetch-only build does not link curl adapter;
- curl-only build does not link Eggfetch.

## 11. Required verification commands

Run and record at least:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo doc --workspace --no-deps
cargo check --workspace --all-targets --all-features
cargo package -p eggup-acquisition --allow-dirty
cargo package -p eggup-curl --allow-dirty
cargo package -p eggup-eggfetch --allow-dirty
```

Also run the repository's Rust 1.89/MSRV and hosted platform lanes. Record release binary sizes and dependency trees for the three footprint fixtures.

## 12. Documentation updates

Update the acquisition roadmap, crate map if it enumerates adapters, `eggup-acquisition` composition semantics, `eggup-curl` docs, consumer guidance, and security notes for executable discovery/proxies/redirects/subprocess bounds.

State prominently that transport fallback is not release/source fallback.

## 13. Acceptance criteria

M005 closes only when:

- curl acquisition implements the existing seam without embedding release policy;
- curl-only consumption does not require Eggfetch or an embedded HTTP/TLS stack;
- Eggfetch-only consumption does not require curl;
- preferred/fallback composition is explicit and deterministic;
- exact 404 remains terminal `NotFound`;
- default fallback occurs only for adapter unavailability;
- broader transport-error fallback is opt-in;
- subprocesses are bounded, cancelled children are reaped, and output is bounded;
- Eggup temp/no-clobber/redaction semantics are preserved;
- deterministic tests require no public network;
- footprint evidence for all three configurations is recorded;
- Rust 1.89, package/docs, and hosted CI pass;
- no medium-or-higher correctness/security finding remains open.

## 14. Stop conditions

Stop and write a corrective/ADR if:

- curl requires shell execution or implicit elevation;
- reliable cancellation cannot kill/reap the child;
- exact 404 cannot be distinguished without a second network request;
- composition requires interpreting human-readable errors;
- proxy/redirect semantics cannot be made explicit;
- safe no-clobber promotion would require weakening M004;
- a proposed change puts release/version/Cargo fallback policy into the transport;
- a producer packaging/release concern is discovered.

## 15. Closure evidence required

The closure record must include implementation commit(s), exact public API changes, contract-test/call-count evidence, timeout/cancellation/reaping evidence, redaction/no-clobber evidence, curl-only/Eggfetch-only/dual dependency trees and stripped sizes, Rust 1.89/hosted platform/package/doc evidence, security findings, and confirmation that Gregg was not modified or depended on.

## 16. Handoff notes

Use `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` only to understand/test mature curl behavior. Do not import Gregg-specific release naming, crates.io authority, Cargo fallback, CLI text, or product identifiers.

The key distinction is:

```text
transport fallback: curl <-> Eggfetch for the same exact URL
release/source fallback: binary -> Cargo/other release source
```

M005 implements only the first.
