# ADR-0002: Multi-artifact transaction and explicit rollback semantics

Status: accepted

Date: 2026-09-22

Decision owners: project maintainers

Related specification sections:

- `plans/000-long-term-specification.md#5-transaction-model`
- `plans/000-long-term-specification.md#6-multi-artifact-installation-is-fundamental`
- `plans/000-long-term-specification.md#12-rollback-model`

Affected roadmaps:

- `plans/subsystems/verified-update-core-roadmap.md`
- `plans/subsystems/consumer-adoption-roadmap.md`

## Context

Current Eggstack consumers span single executables, two-binary release units, and multi-runfile bundles. Several already implement their own backup/restore choreography.

Designing Eggup around one running executable would force bundle consumers to rebuild transaction semantics outside the library.

Current consumers also differ on post-commit service failure: some retain the verified new binary, while EggPool can restore the old binary.

## Decision drivers

- one transaction model for single and multi-file installs;
- no mixed-version successful outcome;
- reusable rollback;
- explicit post-commit policy rather than accidental behavior;
- deterministic fault-injection testing.

## Considered options

### Option A — single-executable self-replace API

Rejected as the primary model.

### Option B — directory-level swap only

Insufficient for installations whose files or manager assets cannot be represented as one directory rename.

### Option C — artifact-set transaction with per-member destinations and backups

Selected.

## Decision

The core unit of mutation is an ArtifactSet containing one or more ArtifactMembers.

The transaction prepares all members before live mutation, acquires one mutation lock for the installation domain, backs up all required existing members, commits the complete set, performs post-install verification, and only then finalizes backup removal according to configured policy.

Single-binary self-update is an ArtifactSet of one member.

Post-commit failures use an explicit policy:

```rust
enum PostCommitFailurePolicy {
    KeepInstalled,
    RollBack,
}
```

Rollback failure yields a distinct RecoveryRequired terminal state.

## Consequences

### Positive

- CodeGG and Egress become first-class use cases;
- single-binary consumers do not need different safety code;
- failure injection can test one state machine;
- service restart differences remain caller policy.

### Negative

- core implementation is more involved than a simple self-replace call;
- backup/commit rules require careful Windows and cross-filesystem qualification.

## Compatibility and migration

Adapters may initially expose convenience helpers for one-member transactions. Consumers migrate without changing visible release layout.

## Security and reliability implications

Commit must revalidate live destinations immediately before destructive rename/copy. Rollback evidence must not be discarded if restoration is incomplete.

## Verification

Fault-injection coverage must exercise failure before backup, after each backup, after each member commit, during post-install validation, and during rollback.

## Supersession

None.
