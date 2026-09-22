# Acquisition Transport Milestone 002 — Eggfetch Adapter

Status: ready for handoff (unblocked by transport M001 closure)

Repository baseline for planning: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Primary class: capability / infrastructure

## 1. Objective

Implement `eggup-eggfetch` as the preferred native Rust HTTP acquisition adapter for consumers already using Eggfetch, with explicit bounded network policy and no release/fallback authority.

## 2. Dependencies

Hard:

- acquisition transport M001 closure;
- corrected core M005.

Operational:

- select the current supported Eggfetch release at execution time and record its feature surface.

## 3. Invariants

- no Eggfetch dependency enters `eggup-core`;
- HTTP/1 + Rustls feature ownership is explicit and minimal;
- HTTPS downgrade is rejected;
- redirects are bounded;
- proxy behavior is explicit;
- connect/total deadlines are explicit;
- metadata is bounded in memory;
- artifact bodies stream to stage;
- 404 is typed distinctly but never triggers fallback inside the adapter;
- no decompression/cookies/retry features are enabled unless a consumer requirement proves them necessary.

## 4. Scope

- `crates/eggup-eggfetch`;
- exact Eggfetch dependency/features;
- adapter implementation for M001 seam;
- local HTTP fixture server;
- redirect/status/timeout/proxy/body-bound tests;
- package docs;
- dependency and binary-size measurements.

## 5. Required network policy

Model the stricter existing eggsact/stegoeggo posture:

- HTTP/1 only unless current Eggfetch architecture requires otherwise;
- Rustls;
- documented root-store choice;
- finite connect timeout;
- finite total timeout;
- finite redirect count;
- HTTPS -> HTTP downgrade rejection;
- explicit environment-proxy decision;
- invalid proxy configuration fails closed;
- bounded metadata;
- streamed binary;
- no automatic retry.

Exact timeout numbers may be configurable; defaults must be documented.

## 6. Ordered work packages

A. Pin/qualify Eggfetch version and features.

B. Metadata fetch.

C. Streamed artifact fetch.

D. Redirect/proxy/TLS/status policy.

E. Local fixture coverage.

F. Footprint/dependency measurement.

## 7. Failure semantics

No fallback on:

- checksum mismatch;
- TLS error;
- malformed metadata;
- timeout;
- 5xx;
- redirect-policy violation.

The adapter may return typed NotFound for exact 404; the consumer decides what that means.

## 8. Required tests

- 200 metadata;
- oversized metadata;
- streaming download;
- early disconnect/truncation;
- 404;
- 500;
- redirect loop;
- redirect limit;
- HTTPS downgrade policy through test seam;
- connect/total timeout;
- valid/invalid proxy configuration;
- output file cleanup.

## 9. Verification

Run workspace checks plus package-specific package/dry-run checks.

Record `cargo tree` for both `eggup-core` and `eggup-eggfetch` to prove transport isolation.

## 10. Acceptance criteria

- adapter satisfies M001 seam;
- no release-policy decisions are embedded;
- core remains transport-free;
- local fixture suite is green;
- dependency/size cost is recorded before adoption.

## 11. Stop conditions

Stop if Eggfetch cannot satisfy streaming/bounds without adding policy to core, or if its current API requires a materially different acquisition abstraction.

## 12. Closure evidence required

- exact Eggfetch version/features;
- fixture matrix;
- dependency trees;
- size measurement methodology/result;
- HTTP policy documentation;
- package qualification result.
