# Releases, pinning, and upgrading

Eggup is published to crates.io. **crates.io is the source of truth** for what
exists. The table below was rechecked against crates.io with `cargo info` on **2026-10-23**
and matches the workspace manifests; re-check it when a release milestone
closes, since registry state is the one fact here that changes without a
commit. See also
[`architecture/tooling-governance.md`](../architecture/tooling-governance.md) §8.

## Published state

All workspace crates currently declare source version `0.2.0`. Only the registry differs.

| Crate | On crates.io | Notes |
|---|---|---|
| `eggup-core` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0` | latest published `0.2.0` |
| `eggup-acquisition` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0` | latest published `0.2.0`; source-breaking |
| `eggup-eggfetch` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0` | latest published `0.2.0` |
| `eggup-curl` | `0.1.2`, `0.2.0` | first publication was `0.1.2`; `0.1.3` was never published |
| `eggup-archive` | `0.1.2`, `0.1.3`, `0.2.0` | latest published `0.2.0` |
| `eggup-eggpack` | `0.1.2`, `0.1.3`, `0.2.0` | latest published `0.2.0` |
| `eggup-service` | `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0` | latest published `0.2.0` |
| `eggup-transport-footprint` | — | `publish = false`; not on the registry at all |

One consequence worth planning around:

- **`eggup-transport-footprint` cannot be depended on from crates.io.** It exists
  to measure footprint inside this workspace via a path dependency.

The `0.2.0` line is one lockstep train published from a single source commit.
`eggup-acquisition`'s next release had to be source-breaking, and the workspace
shares one version, so all seven library crates moved together. Nothing was
withdrawn: the `0.1.x` versions are still on crates.io, and the `v0.1.1`,
`v0.1.2`, and `v0.1.3` tags were not moved. `eggup-curl` skipped `0.1.3`
entirely — its next release after `0.1.2` is `0.2.0`.

## Pinning: the exact-pin cascade

`eggup-eggpack` pins its Eggup dependencies with exact requirements
(`=0.2.0`). Every other crate uses a caret requirement (`^0.2`).

The practical effect:

- A fix published to a **caret** consumer (`eggup-curl`, `eggup-eggfetch`,
  `eggup-service`) reaches you on the next resolve, with no action.
- The same fix reaches you through `eggup-eggpack 0.2.0`, which resolves its own
  dependencies at exactly `0.2.0` and therefore carries the `0.2.0` seam fixes.
  A seam fix published after `0.2.0` still needs a separate adapter
  republication before `eggup-eggpack` inherits it; `eggup-eggpack 0.1.3`,
  which exact-pinned `=0.1.3`, never received them, and `eggup-eggpack 0.1.2`,
  which exact-pinned `=0.1.2`, never received them either.

So one seam fix can become two publication events. If you depend on
`eggup-eggpack`, check the registry before assuming a published fix is in your
dependency graph.

A caret requirement does not cross a major boundary. `^0.1.0` does not accept
`0.2.0`, so when the seam went to `0.2.0` the three caret consumers had their
requirements widened in the same change set. A consumer that depends on both
the seam and an adapter inherits the seam's source break only after the
adapter's own requirement is widened — which is why the adapter's published
version, not the seam's, is what you actually resolve.

## Known limitations in currently published versions

These are real properties of the versions you can resolve today, not roadmap
items. Each is tracked upstream in
[`plans/registry.md`](../plans/registry.md).

The `0.2.0` line carries the fixes for every defect listed against the
`0.1.x` line below. A consumer who resolves today gets `0.2.0` and does not
carry them; a consumer still **pinned** to `0.1.x` does.

- **`eggup-eggpack 0.1.2` carries the positional-binding defect.** Archive members
  were bound positionally with no identity cross-check, so a `BoundExtraction` from
  a different plan — or reused after member reordering — could commit bytes under
  another member's declared identity. That substitution is digest-invisible when
  two declared members share identical content. Verified against the published
  source: `0.1.2`'s `bind_archive_members` pairs members with
  `members.iter().zip(...)` and checks only the count, while `0.1.3` additionally
  compares each bound member's `source_path`/`output_name` against the declared
  member and fails closed with `AdapterError::MapMismatch`. The fix shipped in
  `eggup-eggpack 0.1.3` and is carried by `0.2.0`; see the `0.2.0` section of
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
- **`eggup-service 0.1.3` reports a failed `crontab -` write as a failure even
  when the write succeeded.** The crate's own bounded executor consumed the
  stdin writer's verdict with `try_recv`, discarding a successful write and then
  reading the now-empty channel as a failure. A consumer pinned to
  `eggup-service 0.1.3` receives a typed `Spawn` failure where a successful
  install was intended. The fix is carried by `eggup-service 0.2.0`; see the
  `0.2.0` section of
  [`crates/eggup-service/CHANGELOG.md`](../crates/eggup-service/CHANGELOG.md).
- **`eggup-core 0.1.3` has no bounded stdin and an unbounded candidate
  deadline.** `run_bounded` in the published `0.1.3` evaluated its documented
  wall-clock deadline only while the direct child was still alive, so a
  descendant that inherited the output pipe could pin the call indefinitely, and
  there is no `CommandSpec::stdin_data` or `MAX_STDIN_BYTES` at all. A consumer
  pinned to `eggup-core 0.1.3` carries both. The fixes are carried by
  `eggup-core 0.2.0`.
- **`Error::Injected` exists in no packaged build.** The variant is compiled only
  for the crate's own tests, so it is absent from every published `eggup-core`
  version, `0.1.2`, `0.1.3`, and `0.2.0` alike. All three publish exactly the
  same seven `Error` variants, so a downstream exhaustive `match` on `Error`
  compiles against any of them without needing an extra arm.

## Upgrading

Eggup is pre-1.0, so a minor bump can break you. Before upgrading:

1. Read the target crate's `CHANGELOG.md`, especially the `Unreleased` section
   and any migration note.
2. Re-run the gate locally if you are working in this workspace:
   `./scripts/check-local.sh`.
3. Check whether the change reaches you through an exact-pinned crate
   (`eggup-eggpack`) — if so, upgrading is not sufficient on its own.

### Upgrading to `0.2.0`

`0.2.0` is source-breaking in three places. Nothing else in the line is.

- **`eggup-acquisition`: `AcquisitionError::Io(String)` became
  `AcquisitionError::Io { operation, source }`.** Match the struct-shaped
  variant. `AcquisitionError` also no longer derives `Clone`, `PartialEq`, or
  `Eq`, because `std::io::Error` implements none of them — a consumer that
  cloned or compared the error must compare something else. Its `Display`
  rendering is bounded at 512 bytes.
- **`eggup-service`: `ServiceError::Manager(detail)` became
  `ServiceError::Manager { kind, detail, os_code }`.** Match the struct form
  and branch on `kind` (`ManagerFailureKind`) rather than on the message text.
  `ServiceError::manager_kind()` and `ServiceError::manager_os_code()` are the
  programmatic discriminators.
- **`eggup-service`: `LifecycleUpdateReceipt` gained the public field
  `engine_policy`.** The struct has no `Default` impl, so a consumer that
  constructs it with a **struct literal** must add the field and stops
  compiling until it does. A consumer that only reads the receipt's fields is
  unaffected.

Two behavioural changes are worth reading about even though they do not stop
you compiling:

- **`eggup-service` cron now requires the managed block body to embed the
  absolute executable path of the spec.** A `desired_block` that omits
  `spec.executable()` now receives `ServiceError::InvalidInput` from `inspect`,
  `install`, and `uninstall`, where it previously received success (`install`)
  or a normal result.
- **`eggup-archive`'s `cleanup()` success path is now reachable.** A caller
  that wrote `cleanup().unwrap_err()` will panic and must match on the result
  instead; capture the residual path from `root()` before the consuming call.

`eggup-core` is additive at `0.2.0`: `Error` keeps exactly the same seven
variants, so an exhaustive `match` on it compiles unchanged. `MAX_STDIN_BYTES`
and `CommandSpec::stdin_data` are added, and `run_bounded`'s outcome
classification changed — the deadline now has priority over the stdin writer's
verdict.

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
**2026-10-23**: every run cited in `plans/` resolves and its head SHA matches
the commit the closure record names, including the runs deliberately cited as
failures. `v0.1.2` is an annotated tag pointing at `e8e07eb5`, the M009
publication source, `v0.1.3` points at `bd43683`, and GitHub releases `0.1.1`,
`0.1.2`, and `0.1.3` all exist. `v0.2.0` is created only after the
registry-only proofs pass, pointing at the exact publication source commit, and
the existing tags are never moved.

The full process, including dependency ordering and the evidence a publication
milestone must record, is in
[`architecture/tooling-governance.md`](../architecture/tooling-governance.md) §8.
