# Choose a crate

Most consumers need `eggup-core` and, depending on their workflow, one or more
adapters.

## By use case

| Use case | Crates |
|---|---|
| Stage, verify, and replace local files | `eggup-core` |
| Fetch over HTTP | `eggup-acquisition` plus `eggup-eggfetch` or `eggup-curl` |
| Extract a tar.gz or zip archive | `eggup-archive` |
| Manage a system service | `eggup-service` |
| Read an Eggpack ReleaseManifest v1 | `eggup-eggpack` |

## Crate roles and boundaries

| Crate | Role and boundary |
|---|---|
| [`eggup-core`](https://docs.rs/eggup-core) | Local staging, SHA-256 integrity checks, candidate validation, ownership checks, replacement, rollback, and receipts. No transport or service-manager policy. |
| [`eggup-acquisition`](https://docs.rs/eggup-acquisition) | Transport-neutral acquisition interface, fetch limits, fixtures, and composition. It does not perform HTTP requests and has no dependencies. |
| [`eggup-eggfetch`](https://docs.rs/eggup-eggfetch) | Native HTTP adapter with Rustls. Use with `eggup-acquisition`; it does not select releases or install destinations. |
| [`eggup-curl`](https://docs.rs/eggup-curl) | Adapter that runs an explicitly configured `curl` executable. It has no embedded HTTP or TLS stack, and requires trusting and providing that executable. |
| [`eggup-archive`](https://docs.rs/eggup-archive) | Bounded, allowlisted local extraction of tar.gz and zip files. It does not provide transport, authenticity checks, or live-installation policy. |
| [`eggup-service`](https://docs.rs/eggup-service) | Service registration and lifecycle adapters for systemd, launchd, cron, and Windows SCM. It does not elevate privileges automatically. |
| [`eggup-eggpack`](https://docs.rs/eggup-eggpack) | Optional adapter from Eggpack ReleaseManifest v1 to acquisition and deployment inputs. Release selection, destination choice, ownership, and authenticity remain caller policy. |
| `eggup-transport-footprint` | Workspace-only fixtures for measuring transport dependencies. It is not published and cannot be used as a crates.io dependency. |

The HTTP adapters are alternatives. Depend on `eggup-acquisition` and choose
`eggup-eggfetch` for an in-process HTTP stack or `eggup-curl` to use a host
`curl` binary. Composition is provided by the acquisition crate, not by either
adapter.

`eggup-eggpack` pins its Eggup dependencies exactly. A later fix to one of those
crates will not reach an existing `eggup-eggpack` release until the adapter is
republished. See [versions and upgrades](releases.md).

For a first local transaction, start with the [quickstart](quickstart.md).
