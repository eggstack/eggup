# ADR-0003: Verification layers and transport neutrality

Status: accepted

Date: 2026-09-22

Decision owners: project maintainers

Related specification sections:

- `plans/000-long-term-specification.md#7-verification-model`
- `plans/000-long-term-specification.md#8-acquisition-safety`

Affected roadmaps:

- `plans/subsystems/verified-update-core-roadmap.md`
- `plans/subsystems/acquisition-transport-roadmap.md`

## Context

Existing Eggstack updaters mix transport, checksum verification, release-policy fallback, and executable validation. Most newer consumers use Eggfetch, while Gregg measured an unacceptable footprint increase from adopting a TLS stack and intentionally retained external curl.

Checksum sidecars currently provide corruption/mismatch detection but are not independent publisher signatures.

## Decision drivers

- avoid mandatory HTTP/TLS dependencies in core;
- permit Eggfetch-native acquisition where already present;
- preserve lightweight transport options;
- prevent transport failures from silently changing release source;
- accurately distinguish integrity from authenticity.

## Decision

Eggup core receives already acquired inputs or an abstract acquisition result; it does not depend on HTTP.

`eggup-eggfetch` is the preferred native Rust transport adapter for consumers already carrying or accepting Eggfetch.

Other adapters may exist when justified.

Transport returns typed success/not-found/hard-failure results but does not select fallback behavior.

Verification is layered:

1. integrity verification;
2. optional authenticity verification;
3. candidate identity/compatibility verification.

A checksum sidecar from the same release location is integrity evidence, not an independent authenticity guarantee.

## Consequences

- consumer fallback remains explicit;
- core stays small;
- error taxonomy must preserve acquisition classification across adapters;
- authenticity can be added later without redefining checksum semantics.

## Compatibility and migration

Consumers can initially preserve existing checksum-only behavior and explicitly configure no independent authenticity policy.

## Security and reliability implications

Unverified candidate bytes must never execute. HTTPS downgrade, redirect, proxy, and timeout policy belong to each transport adapter and must be explicit.

## Verification

- core dependency graph has no HTTP/TLS stack;
- Eggfetch adapter uses bounded local fixture tests;
- transport 404 can be distinguished from TLS/5xx/malformed response;
- checksum mismatch cannot trigger source fallback inside Eggup.

## Supersession

None.
