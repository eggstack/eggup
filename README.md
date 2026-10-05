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
made.

## The eight crates

| Crate | Role |
|---|---|
| [`eggup-core`](crates/eggup-core) | Policy-neutral local transaction mechanics. Sole dependency: `sha2` |
| [`eggup-acquisition`](crates/eggup-acquisition) | Transport-neutral seam: `AcquisitionTransport`, bounds resolution, private temp + no-clobber promotion, fixture transport, and cross-transport composition. **Zero dependencies** |
| [`eggup-eggfetch`](crates/eggup-eggfetch) | Native HTTP adapter (`eggfetch-core` + tokio) |
| [`eggup-curl`](crates/eggup-curl) | External-`curl` adapter with no embedded HTTP/TLS stack |
| [`eggup-archive`](crates/eggup-archive) | Bounded allowlisted tar.gz/zip extraction of already verified archives |
| [`eggup-eggpack`](crates/eggup-eggpack) | Optional Eggpack ReleaseManifest v1 adapter (leaf) |
| [`eggup-service`](crates/eggup-service) | Manager-neutral service lifecycle: systemd, launchd, cron, Windows SCM |
| `eggup-transport-footprint` | Non-published fixtures measuring which transport stack a consumer links |

`eggup-curl` and `eggup-eggfetch` are interchangeable: both implement the same
`AcquisitionTransport` trait and neither may know the other exists. Transport
composition lives in the seam, never in an adapter. Choosing between them is a
footprint and trust decision made by the caller.

## Optional adapters

The optional `eggup-eggpack` leaf crate translates Eggpack ReleaseManifest v1
into caller-bound acquisition requests and direct/bundle deployment members.
It does not add an Eggpack dependency to core, acquisition, or service. Archive
manifests remain extraction-required; URL, install-root, ownership, permission,
release, and authenticity policy remain caller-owned.

The optional `eggup-archive` crate extracts explicitly declared regular files
from already verified tar.gz and zip archives into a private bounded staging
directory. It never changes the live installation, and `eggup-core` remains
independent of archive formats.

## Where to read next

- [architecture/overview.md](architecture/overview.md) — the ownership map,
  module map, cross-cutting invariants, and the deep-dive index.
- [architecture/tooling-governance.md](architecture/tooling-governance.md) — the
  local verification gate, the CI matrix, the planning process, and the release
  process.
- [AGENTS.md](AGENTS.md) — the index for coding agents working in this workspace.
- [plans/implementation/README.md](plans/implementation/README.md) and
  [plans/closure/README.md](plans/closure/README.md) — how work is planned and
  how it is proven complete.

Work is sequenced by the [implementation plans](plans/implementation/README.md)
and their [closure records](plans/closure/README.md).

## Publication state

`eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`,
`eggup-eggpack`, and `eggup-curl` are published on crates.io at `0.1.2`.
`eggup-service` is published but lags at `0.1.1`, so its `0.1.2` has never been
published. `eggup-transport-footprint` is `publish = false` by design. CI never
publishes; releases are manual and milestone-gated.
