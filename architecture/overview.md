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

Only the workspace and crate boundary are implemented in the foundation
milestone. The domain/preparation milestone now also validates explicit plans
and copies local inputs into private stage state; no live installation, service,
or network behavior is present. Commit/rollback mechanics are documented in
the [transaction contract](../crates/eggup-core/docs/transaction.md); service
and transport layers remain outside the crate.
