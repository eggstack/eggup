# Eggup

Eggup is the shared library for safe, verified local updates of one or more
application artifacts. Its core owns local staging, SHA-256 integrity
verification, bounded candidate validation, destination ownership
revalidation, locking, replacement, rollback, and recovery evidence. A caller
that needs a post-install check can use `ValidatedTransaction::commit_with_post_commit`
to retain rollback evidence through one check and choose `KeepInstalled` or
`RollBack` on failure.
Transport, release discovery, service management, installers, and consumer
policy remain outside the core boundary.

`eggup-core` implements a verified local transaction
(`PreparedTransaction -> VerifiedTransaction -> ValidatedTransaction ->
commit`) with explicit `Absent | Owned | Foreign | Unknown` destination
ownership, owner-private transaction state, staged-digest revalidation under
lock, structured failure reports, and truthful fail-closed lock semantics.
Integrity is checksum evidence only; no authenticity or signature claim is
made. Work is sequenced by the [implementation plans](plans/implementation/README.md)
and their [closure records](plans/closure/README.md).

The optional `eggup-eggpack` leaf crate translates Eggpack ReleaseManifest v1
into caller-bound acquisition requests and direct/bundle deployment members.
It does not add an Eggpack dependency to core, acquisition, or service. Archive
manifests remain extraction-required; URL, install-root, ownership, permission,
release, and authenticity policy remain caller-owned.

The optional `eggup-archive` crate extracts explicitly declared regular files
from already verified tar.gz and zip archives into a private bounded staging
directory. It never changes the live installation, and `eggup-core` remains
independent of archive formats.

See [architecture/overview.md](architecture/overview.md) for the ownership
map and canonical planning references.
