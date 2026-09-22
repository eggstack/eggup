# Eggup

Eggup is the planned shared library for safe, verified local updates of one or
more application artifacts. Its core owns local staging, verification,
destination ownership, locking, replacement, rollback, and recovery evidence.
Transport, release discovery, service management, installers, and consumer
policy remain outside the core boundary.

The repository is currently in foundation stage. `eggup-core` has a Rust 1.89
workspace and deterministic test support, but it does not yet claim production
updater capability. Work is sequenced by the [implementation plans](plans/implementation/README.md)
and their [closure records](plans/closure/README.md).

See [architecture/overview.md](architecture/overview.md) for the ownership
map and canonical planning references.

