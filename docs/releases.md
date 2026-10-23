# Releases, pinning, and upgrading

Eggup is published to crates.io. **crates.io is the source of truth** for what
exists. The table below was rechecked against crates.io with `cargo info` on **2026-10-09**
and matches the workspace manifests; re-check it when a release milestone
closes, since registry state is the one fact here that changes without a
commit. See also
[`architecture/tooling-governance.md`](../architecture/tooling-governance.md) §8.

## Published state

All workspace crates currently declare source version `0.1.3`. Only the registry differs.

| Crate | On crates.io | Notes |
|---|---|---|
| `eggup-core` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3` | latest published `0.1.3` |
| `eggup-acquisition` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3` | latest published `0.1.3` |
| `eggup-eggfetch` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3` | latest published `0.1.3` |
| `eggup-curl` | `0.1.2` | first publication was `0.1.2` |
| `eggup-archive` | `0.1.2`, `0.1.3` | latest published `0.1.3` |
| `eggup-eggpack` | `0.1.2`, `0.1.3` | latest published `0.1.3` |
| `eggup-service` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3` | M011 published the M010 owned failed-systemd quiescence correction on 2026-10-09 |
| `eggup-transport-footprint` | — | `publish = false`; not on the registry at all |

One consequence worth planning around:

- **`eggup-transport-footprint` cannot be depended on from crates.io.** It exists
  to measure footprint inside this workspace via a path dependency.

`eggup-service` was published last (2026-10-09, Service Lifecycle M011, at
`0.1.3`). The shared `v0.1.2` tag still denotes the `eggup-core`/`eggup-archive`
publication source; it was not moved to point at the service publication.

## Pinning: the exact-pin cascade

`eggup-eggpack` pins its Eggup dependencies with exact requirements
(`=0.1.3`). Every other crate uses a caret requirement (`^0.1.0`).

The practical effect:

- A fix published to a **caret** consumer (`eggup-curl`, `eggup-eggfetch`,
  `eggup-service`) reaches you on the next resolve, with no action.
- The same fix reaches you through `eggup-eggpack 0.1.3`, which resolves its own
  dependencies at exactly `0.1.3` and therefore carries the `0.1.3` seam fixes.
  A seam fix published after `0.1.3` still needs a separate adapter
  republication before `eggup-eggpack` inherits it; `eggup-eggpack 0.1.2`,
  which exact-pinned `=0.1.2`, never received them.

So one seam fix can become two publication events. If you depend on
`eggup-eggpack`, check the registry before assuming a published fix is in your
dependency graph.

## Known limitations in currently published versions

These are real properties of the versions you can resolve today, not roadmap
items. Each is tracked upstream in
[`plans/registry.md`](../plans/registry.md).

- **`eggup-eggpack 0.1.2` carries the positional-binding defect.** Archive members
  were bound positionally with no identity cross-check, so a `BoundExtraction` from
  a different plan — or reused after member reordering — could commit bytes under
  another member's declared identity. That substitution is digest-invisible when
  two declared members share identical content. Verified against the published
  source: `0.1.2`'s `bind_archive_members` pairs members with
  `members.iter().zip(...)` and checks only the count, while `0.1.3` additionally
  compares each bound member's `source_path`/`output_name` against the declared
  member and fails closed with `AdapterError::MapMismatch`. The fix shipped in
  `eggup-eggpack 0.1.3`; see the `0.1.3` section of
  [`crates/eggup-eggpack/CHANGELOG.md`](../crates/eggup-eggpack/CHANGELOG.md).
- **A composed fetch in `eggup-acquisition 0.1.2` can overrun the caller's
  deadline.** `ComposedTransport` handed the caller's *original* `FetchLimits` to
  a fallback adapter, so one composed fetch could run for roughly twice the
  documented `total_timeout`. The fix (`remaining_limits`, which subtracts
  elapsed time and clamps `connect_timeout` to the remainder) shipped as
  `eggup-acquisition 0.1.3` on 2026-10-06 (Acquisition Transport M010), and
  `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.2` inherit it automatically on the
  next resolve. A consumer pinned to `eggup-acquisition 0.1.2` still carries the
  overrun. Note that `FetchLimits::validate` *is* already enforced at every
  transport boundary in the published `0.1.2` — the gap is the fallback budget,
  not validation.
- **`Error::Injected` exists in no packaged build.** The variant is compiled only
  for the crate's own tests, so it is absent from every published `eggup-core`
  version, `0.1.2` and `0.1.3` alike. Both publish exactly the same seven
  `Error` variants, so a downstream exhaustive `match` on `Error` compiles
  against either without needing an extra arm.

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

The published evidence for each milestone is a hosted CI run. Those run IDs,
the tags, and the release records were checked against the GitHub API on
**2026-10-05**: every run cited in `plans/` resolves and its head SHA matches
the commit the closure record names, including the runs deliberately cited as
failures. `v0.1.2` is an annotated tag pointing at `e8e07eb5`, the M009
publication source, and GitHub releases `0.1.1` and `0.1.2` both exist.

The full process, including dependency ordering and the evidence a publication
milestone must record, is in
[`architecture/tooling-governance.md`](../architecture/tooling-governance.md) §8.
