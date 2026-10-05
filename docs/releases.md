# Releases, pinning, and upgrading

Eggup is published to crates.io. **crates.io is the source of truth** for what
exists; the table below is a convenience snapshot recorded in
[`architecture/tooling-governance.md`](../architecture/tooling-governance.md) §8
and verified against the workspace manifests, not against the registry.

## Published state

All crates share the source version `0.1.2`. Only the registry differs.

| Crate | On crates.io | Notes |
|---|---|---|
| `eggup-core` | `0.1.0`, `0.1.1`, `0.1.2` | |
| `eggup-acquisition` | `0.1.0`, `0.1.1`, `0.1.2` | |
| `eggup-eggfetch` | `0.1.0`, `0.1.1`, `0.1.2` | |
| `eggup-curl` | `0.1.2` | first publication was `0.1.2` |
| `eggup-archive` | `0.1.2` | first publication was `0.1.2` |
| `eggup-eggpack` | `0.1.2` | first publication was `0.1.2` |
| `eggup-service` | `0.1.0`, `0.1.1` | **lags** — its `0.1.2` has never been published |
| `eggup-transport-footprint` | — | `publish = false`; not on the registry at all |

Two consequences worth planning around:

- **`eggup-service` is published but behind.** A dependency on `"0.1.1"` resolves
  today; a dependency on `"0.1.2"` does not. If you take a caret requirement you
  will silently get `0.1.1`, which predates the fixes in its `Unreleased`
  section. Read
  [`crates/eggup-service/CHANGELOG.md`](../crates/eggup-service/CHANGELOG.md)
  before assuming you have current behaviour.
- **`eggup-transport-footprint` cannot be depended on from crates.io.** It exists
  to measure footprint inside this workspace via a path dependency.

## Pinning: the exact-pin cascade

`eggup-eggpack` pins its Eggup dependencies with exact requirements
(`=0.1.2`). Every other crate uses a caret requirement (`^0.1.0`).

The practical effect:

- A fix published to a **caret** consumer (`eggup-curl`, `eggup-eggfetch`,
  `eggup-service`) reaches you on the next resolve, with no action.
- The same fix **cannot** reach you through `eggup-eggpack 0.1.2`. That crate
  resolves its own dependencies at exactly `0.1.2`, so it needs a separate
  republication before it inherits a seam fix.

So one seam fix can become two publication events. If you depend on
`eggup-eggpack`, check the registry before assuming a published fix is in your
dependency graph.

## Known limitations in currently published versions

These are real properties of the versions you can resolve today, not roadmap
items. Each is tracked upstream in
[`plans/registry.md`](../plans/registry.md).

- **Published `eggup-eggpack 0.1.2` contains a security fix that is not yet
  published.** Archive members were bound positionally with no identity
  cross-check, so a `BoundExtraction` from a different plan — or reused after
  member reordering — could commit bytes under another member's declared
  identity. That substitution is digest-invisible when two declared members
  share identical content. The fix is implemented and green but not released;
  see the `Unreleased` section of
  [`crates/eggup-eggpack/CHANGELOG.md`](../crates/eggup-eggpack/CHANGELOG.md).
- **Published `eggup-acquisition 0.1.2` does not validate every `FetchLimits`
  value at the transport boundary.** A composed transport can therefore run
  roughly twice the documented `total_timeout`. This is the specific payload of
  the open Acquisition M010 milestone (`eggup-acquisition 0.1.3`);
  `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.2` inherit the fix automatically
  once it lands.
- **Published `eggup-core 0.1.2` predates the `Error::Injected` addition.** That
  variant is only in the unpublished working tree, so a downstream exhaustive
  `match` on `Error` compiles against the published version without needing the
  extra arm yet.

## Upgrading

Eggup is pre-1.0, so a minor bump can break you. Before upgrading:

1. Read the target crate's `CHANGELOG.md`, especially the `Unreleased` section
   and any migration note.
2. Re-run the gate locally if you are working in this workspace:
   `./scripts/check-local.sh`.
3. Check whether the change reaches you through an exact-pinned crate
   (`eggup-eggpack`) — if so, upgrading is not sufficient on its own.

A note on the `Error` enum specifically: it is **not**
`#[non_exhaustive]`, so adding a variant is a breaking change for an exhaustive
`match`. `AcquisitionError` and `AdapterError` *are* `#[non_exhaustive]`, so
adding to those is additive.

## Release process

Publication is a deliberate, milestone-gated action. CI never publishes — there
is no `cargo publish` step and no registry credential in
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml). Releases are manual,
and published tags are never moved; release notes are extended instead.

The full process, including dependency ordering and the evidence a publication
milestone must record, is in
[`architecture/tooling-governance.md`](../architecture/tooling-governance.md) §8.
