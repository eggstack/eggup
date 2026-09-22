# Eggup architecture overview

Eggup separates policy from local update mechanism. A consumer resolves a
release and acquires local inputs; `eggup-core` will later validate, stage, and
commit an explicitly described artifact set. Network transport, authenticity
trust choices, release ordering, service lifecycle, and bootstrap installers
are separate layers.

The durable requirements and decisions are maintained in:

- [long-term specification](../plans/000-long-term-specification.md)
- [terminology and domain model](../plans/001-terminology-and-domain-model.md)
- [verified update core roadmap](../plans/subsystems/verified-update-core-roadmap.md)
- [layered ownership ADR](../plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md)
- [transaction and rollback ADR](../plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md)
- [verification and transport ADR](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)

Current workspace layers:

```text
consumer/release policy  ->  acquisition/transport  ->  eggup-core
                                                       local mechanics
```

`eggup-core` implements validated plans, owner-private staging (0700/0600),
SHA-256 integrity verification, bounded candidate execution, explicit
`Absent | Owned | Foreign | Unknown` ownership proofs, locked ownership and
staged-digest revalidation, mutation locking with inspection-only stale
handling, and commit/rollback with structured failure reports. No live
destination parent is created automatically, and no authenticity claim is
made. Commit/rollback mechanics are documented in
the [transaction contract](../crates/eggup-core/docs/transaction.md); service
and transport layers remain outside the crate. Verification ordering and
bounded candidate execution are defined in the [verification contract](../crates/eggup-core/docs/verification.md).
Domain preparation rules are defined in the [domain contract](../crates/eggup-core/docs/domain.md).
