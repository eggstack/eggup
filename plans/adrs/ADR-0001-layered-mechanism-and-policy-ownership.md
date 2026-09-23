# ADR-0001: Layered mechanism and policy ownership

Status: accepted; distribution ownership superseded by ADR-0004

Date: 2026-09-22

Decision owners: project maintainers

Related specification sections:

- `plans/000-long-term-specification.md#3-mechanism-versus-policy`
- `plans/000-long-term-specification.md#4-crate-and-layer-model`

Affected roadmaps:

- all initial subsystem roadmaps.

## Context

Eggstack currently contains multiple independently developed updater/install/service implementations. They share staging, checksums, candidate validation, replacement, rollback, target mapping, and service lifecycle patterns, but differ materially in release authority, version semantics, fallback policy, bundle shape, HTTP footprint, and restart behavior.

A library that centralizes those application policies would either become consumer-specific or force incorrect uniformity.

## Decision drivers

- remove duplicated security-sensitive mechanics;
- preserve consumer-specific release semantics;
- avoid forcing an HTTP/TLS stack into small binaries;
- allow service lifecycle to evolve independently from artifact transactions;
- keep CodeGG's generic-updater dependency free of Gregg-specific constants;
- support both single-binary and bundle consumers.

## Considered options

### Option A — one monolithic updater crate

One crate would own GitHub/crates.io discovery, HTTP, replacement, services, and installer behavior.

Rejected because it couples unrelated dependencies and embeds policy.

### Option B — one core crate with a large feature matrix

A single crate could gate HTTP and service managers behind features.

Rejected as the primary architecture because feature unification can still increase consumer dependency surfaces and makes ownership less explicit.

### Option C — layered workspace

Use a small policy-neutral core plus optional transport, service, and distribution crates.

Selected.

## Decision

Eggup will use a layered workspace.

`eggup-core` owns verified artifact transaction mechanics only.

Network acquisition is supplied by separate adapters, initially `eggup-eggfetch`.

Service manager operations live in `eggup-service`.

Release/bootstrap contract tooling was initially placed in `eggup-dist`. ADR-0004 supersedes that portion of this decision: producer-side distribution contracts, conformance, packaging, bootstrap generation, release manifests, CI, and publication now belong to Eggpack. `eggup-dist` is migration-only predecessor evidence pending retirement.

Consumer applications retain release authority, version ordering, fallback decisions, CLI presentation, database/config migrations, and application-specific health semantics.

A thin `eggup` facade may be added after lower-level APIs stabilize.

## Consequences

### Positive

- low dependency floor for core consumers;
- Gregg can retain lightweight acquisition;
- eggsact/stegoeggo/codegg can share Eggfetch;
- service code does not contaminate CLI-only consumers;
- consumer policy differences remain explicit;
- easier package and security review.

### Negative

- more crates and publication ordering;
- adapters require small consumer glue;
- some concepts cannot be hidden behind one convenience function.

### Neutral/deferred

A facade crate can later simplify common usage without changing ownership.

## Compatibility and migration

Existing consumers migrate incrementally. No consumer must change release authority merely to adopt Eggup.

## Security and reliability implications

The separation prevents hidden network fallback and privilege/service behavior inside the transaction engine. Security review can focus independently on transport, filesystem mutation, and manager ownership.

## Verification

- dependency-tree tests/inspection showing no HTTP/TLS or service-manager dependency in eggup-core;
- two consumers with differing release policy adopt the same core;
- Gregg-class lightweight transport remains possible.

## Supersession

ADR-0004 supersedes only the producer distribution/bootstrap ownership portion of this ADR. The layered Eggup core/acquisition/service and consumer-policy decisions remain authoritative.
