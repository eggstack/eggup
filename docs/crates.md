# Choosing a crate

Eight crates, and you probably need one or two. All share the workspace version
`0.1.2`; only their registry state differs.

## The short version

| I want to… | Depend on |
|---|---|
| Stage, verify, and atomically replace local files | `eggup-core` |
| Fetch bytes over HTTP | `eggup-acquisition` + one adapter |
| Extract a verified `.tar.gz` / `.zip` | `eggup-archive` |
| Register or control a system service | `eggup-service` |
| Read an Eggpack ReleaseManifest v1 | `eggup-eggpack` |

## The crates

### `eggup-core` — the local transaction engine

The only crate most consumers need. Owns staging, SHA-256 verification, bounded
candidate validation, destination-ownership revalidation, locking, replacement,
rollback, and receipts.

- **Dependencies:** `sha2` only.
- **Status:** published at `0.1.2`.
- **Boundary:** transport, service managers, and Eggpack types stay out. If you
  need those, take a separate crate.

Start with [quickstart.md](quickstart.md).

### `eggup-acquisition` — the transport-neutral seam

Defines `AcquisitionTransport` plus the bounds, cancellation, and redaction
rules every adapter honours. Includes `FixtureTransport` (deterministic, for
tests) and `ComposedTransport` (try several transports in order).

- **Dependencies:** **none** — not even `sha2`. This is what lets both adapters
  and the footprint fixtures depend on it without dragging in an HTTP stack.
- **Status:** published at `0.1.2`.
- **You need an adapter too.** The seam defines the trait; it does not speak
  HTTP.

### `eggup-eggfetch` — native HTTP adapter

An in-process HTTP/1 + Rustls stack, with no external binary and no ambient
system trust beyond the platform root store.

- **Dependencies:** `eggup-acquisition`, `eggfetch-core`, `tokio`,
  `futures-util`.
- **Status:** published at `0.1.2`.
- **Trade-off:** the largest dependency footprint in the workspace — roughly 89
  transitive crates exist to provide TLS and async.

### `eggup-curl` — external-curl adapter

Shells out to a `curl` binary you name explicitly.

- **Dependencies:** `eggup-acquisition` only. No embedded HTTP or TLS stack.
- **Status:** published at `0.1.2`.
- **Trade-off:** a much smaller link footprint, at the cost of requiring `curl`
  on the host and trusting that binary.

### `eggup-archive` — bounded local extraction

Extracts explicitly declared, regular files from an **already verified**
tar.gz or zip, into a private staging directory with finite decompression
budgets and no-clobber writes. Never mutates a live installation.

- **Dependencies:** `flate2`, `fs_at`, `sha2`, `tar`, `zip`.
- **Status:** published at `0.1.2`.
- **Boundary:** you still own the trust decision. This crate will not tell you
  an archive is *authentic*, only that it extracted as declared.

### `eggup-service` — manager-neutral lifecycle

Service registration and lifecycle across systemd, launchd, cron, and the
Windows SCM, with allowlisted command paths, cleared environments, and
bounded output. No shell, no `sc.exe`, no ambient `PATH`, no automatic
elevation.

- **Dependencies:** `eggup-core`, `windows-args` (plus `windows-service` on
  Windows only).
- **Status:** **published but lagging** — crates.io carries `0.1.0` and `0.1.1`;
  the `0.1.2` working tree has never been published. Pin deliberately, and read
  [`crates/eggup-service/CHANGELOG.md`](../crates/eggup-service/CHANGELOG.md)
  for what is and is not on the registry.

### `eggup-eggpack` — optional ReleaseManifest adapter

Translates an Eggpack ReleaseManifest v1 into caller-bound acquisition requests
and deployment members. Optional: the core, acquisition, and service crates have
no Eggpack knowledge.

- **Dependencies:** `eggup-core`, `eggup-archive`, `eggup-acquisition`, plus
  `eggpack-manifest` from the registry.
- **Status:** published at `0.1.2`.
- **Boundary:** archive manifests stop at "extraction required". Release
  discovery, install roots, ownership, permissions, and authenticity policy stay
  yours.

### `eggup-transport-footprint` — measurement fixtures

Not a dependency of anything. It measures which transport stack a consumer
actually links, via three `required-features` binaries (`curl_only`,
`eggfetch_only`, `dual`).

- **Status:** `publish = false` by design. You cannot depend on it from
  crates.io; use it from a path dependency in this workspace only.

## Two rules that are easy to break

**The adapters must not know about each other.** `eggup-eggfetch` and
`eggup-curl` both implement `AcquisitionTransport` and neither may reference the
other. Composition belongs in the seam. Nothing enforces this automatically —
`cargo tree --workspace` in review is the check.

**`eggup-eggpack` pins its dependencies exactly** (`=0.1.2`). Every other crate
uses a caret requirement. So a fix published to a caret consumer is inherited
automatically, but the same fix cannot reach `eggup-eggpack 0.1.2` consumers
without a separate adapter republication. See [releases.md](releases.md).

## More

- [quickstart.md](quickstart.md) — a working first transaction.
- [`architecture/overview.md`](../architecture/overview.md) — the full
  dependency graph and module map.
- [`architecture/tooling-governance.md`](../architecture/tooling-governance.md) —
  the boundary rules and how they are reviewed.
