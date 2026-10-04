# M010 — eggup-acquisition 0.1.3 publication (workspace 0.1.3 source version)

Status: ready

Repository baseline: `b485228d1edebe27e2dd691310c7c68f60d97d8d`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`

Primary class: package promotion / correctness-delivery

Hard dependencies:

- Acquisition M005-M008 closed (transport semantics qualified and unchanged).
- Acquisition M009 closed — `eggup-curl 0.1.2` is published from `b485228` with
  a registry-only external proof (`plans/closure/acquisition-transport/009-status.md`).
- `0b8cb98` landed the two acquisition defect fixes this milestone publishes.
- `eggup-acquisition 0.1.2` is published from `02a1d32`; `0.1.3` is absent and
  `eggup-acquisition` is already owned, so no name or ownership gate applies.

Downstream trigger: every registry consumer of the acquisition seam that
composes two transports is currently reachable for the doubled-total-deadline
defect described in §3.

## 1. Objective

Publish `eggup-acquisition 0.1.3` carrying the two `0b8cb98` correctness fixes to
crates.io, so the composed-transport deadline defect and the owned-`.part` leak
stop being reachable for registry consumers.

This is a publication milestone for already-implemented and already-tested
defect fixes. It introduces no new transport feature and must not alter any
bounded acquisition, staging, promotion, or redaction semantics.

## 2. Why this milestone is ready

Both fixes are implemented, in the local tree, and covered by the current
workspace gate. Nothing needs to be written; this milestone decides the version
bump, the pin consequences, and the publication order.

## 3. Current implementation evidence

`0b8cb98` ("fix: address workspace bug audit findings across all crates") landed
two acquisition changes that are **not** in the published `0.1.2`:

1. **Owned `.part` file leak.** `stage_bound_source` created the part file and
   then hardened its permissions. The cleanup guard is only armed by the caller
   *after* that call returns `Ok`, so a failure while reading permissions or
   while calling `set_permissions` left the created part file on disk with no
   owner able to remove it. Both failure paths now drop the handle and remove the
   part file before returning the error. The `0600` stage hardening itself is
   unchanged.

2. **`ComposedTransport` restarted the caller's total budget.** The composed
   transport passed the caller's original `FetchLimits` to the secondary adapter,
   so one composed fetch could run for roughly twice the caller's documented
   `total_timeout` — the connect phase is part of the total wall-clock budget.
   The fallback now receives only the remaining budget, with `connect_timeout`
   clamped alongside it to preserve `connect_timeout <= total_timeout`. An
   already-exhausted budget is reported as a `Timeout` instead of starting a
   doomed attempt.

Defect 2 is reachable today by any consumer that composes two transports from
the registry, because the defective code is what `eggup-acquisition 0.1.2`
ships. It is **not** introduced by M009, and it is recorded in the root
`CHANGELOG.md` published-content caveat and in the GitHub Release `0.1.2` notes
as of `b485228`.

Local evidence that the published artifact lacks the fixes: the published
`eggup-acquisition 0.1.2` `src/lib.rs` (`.cargo_vcs_info.json` sha
`02a1d32931be29cc3d8980833643b2cd822f2d28`) differs from the workspace source
specifically at the permission-hardening error arms and the absence of
`ComposedTransport::remaining_limits`.

## 4. Invariants that must not regress

- `eggup-core` stays free of transport and Eggpack dependencies.
- SHA-256 remains integrity evidence only; no authenticity or signature claim.
- Public `FetchLimits` field types, defaults, and validation messages are
  unchanged; this is not a `0.1.x` breaking change and must not become one.
- Transport fallback is still not release/source fallback; exact `NotFound`
  stays terminal for the requested URL.
- The part file is still created exclusively, owner-private (`0600` on Unix),
  never symlink-followed, and promoted with race-safe no-clobber semantics.
- Diagnostic redaction is unchanged; no raw upstream, command-line, or
  credential material is ever embedded.
- Cleanup ownership is unchanged: only an owned temp is ever removed, and a
  complete destination is never reported as an ordinary failure.
- No change to `eggup-curl`, `eggup-eggfetch`, `eggup-service`,
  `eggup-archive`, `eggup-core`, or `eggup-eggpack` transport behavior.

## 5. Scope

### In scope

- The workspace source version bump `0.1.2` → `0.1.3`.
- The `eggup-eggpack` exact-pin consequence that bump forces (see §6).
- Changelog cut for the `0.1.3` publication.
- `cargo package` / `cargo publish --dry-run` / `cargo publish` for
  `eggup-acquisition` only.
- Registry-only external proof, including a composed-transport regression test
  that pins the remaining-budget behavior from outside the crate.
- Roadmap, registry, and closure bookkeeping.

### Explicitly out of scope

- Publishing `eggup-core`, `eggup-archive`, `eggup-curl`, `eggup-eggfetch`,
  `eggup-eggpack`, `eggup-service`, or `eggup-transport-footprint`. Their
  already-published `0.1.2` versions are immutable and are not republished.
- Any new transport feature, retry policy, mirror policy, or release policy.
- Editing the already-implemented fixes. If they are wrong, that is a
  corrective against this milestone, not an expansion of it.
- Gregg M004, which remains intentionally unwritten.
- Phase 12 authenticity work, which still requires an ADR.
- Any change to `.github/workflows/ci.yml`. Publication stays manual.

## 6. Required production changes

### 6.1 Workspace version bump

`[workspace.package] version` in the root `Cargo.toml` moves `0.1.2` → `0.1.3`.
Every crate inherits it, so the local tree claims `0.1.3` for all eight crates
while five remain published at `0.1.2`. `Cargo.lock` is regenerated.

This is unavoidable rather than optional: `version.workspace = true` means
`eggup-acquisition` cannot be published at `0.1.3` without moving the shared
version.

### 6.2 The `eggup-eggpack` exact-pin consequence

`eggup-eggpack` pins `eggup-core`, `eggup-archive`, and `eggup-acquisition` to
`=0.1.2`. After the bump its local `=0.1.2` requirements no longer match the
local `0.1.3` versions and the workspace stops resolving. The pin topology as
published today is:

| Crate | Requirement on `eggup-acquisition` | Inherits `0.1.3`? |
|---|---|---|
| `eggup-curl 0.1.2` | `^0.1.0` | yes |
| `eggup-eggfetch 0.1.2` | `^0.1.0` | yes |
| `eggup-eggpack 0.1.2` | `=0.1.2` | **no** |
| `eggup-transport-footprint` | `^0.1.0` | local only, unpublished |

So the seam fix reaches curl and eggfetch consumers automatically the moment
`eggup-acquisition 0.1.3` exists, but the published Eggpack adapter does **not**
receive it. This milestone must:

- re-pin `eggup-eggpack`'s three `=0.1.2` requirements to `=0.1.3` so the
  workspace resolves, and
- state explicitly, in this plan and in the closure record, that the published
  `eggup-eggpack 0.1.2` therefore keeps composing against the defective
  acquisition until `eggup-eggpack 0.1.3` is published.

Republishing `eggup-eggpack` is **not** part of this milestone; it is a separate
follow-on. Until then, adapter consumers that compose two transports must pin
`eggup-acquisition 0.1.3` themselves or tolerate the doubled composed window.

The `=0.1.2` pins themselves are intentional and are not criticized here: they
were introduced in M004 so the adapter graph could not silently float onto an
unreviewed acquisition. Keeping them and paying for an adapter republication is
the correct trade.

### 6.3 Changelog cut

- Root `CHANGELOG.md`: a new `## 0.1.3` section recording that only
  `eggup-acquisition 0.1.3` is published, naming both fixes, and repeating the
  `eggup-eggpack 0.1.2` caveat.
- `crates/eggup-acquisition/CHANGELOG.md`: move the two `0b8cb98` entries out of
  `## Unreleased` into `## 0.1.3 — <date>`, and move the M007 finite-bounds note
  that still reads "unpublished corrective" into a historical statement, since
  that work did ship in `0.1.2`.
- Historical `0.1.2` text is corrected in place with the reason stated. Nothing
  is erased to conceal a defect.

### 6.4 No source change

Zero edits to any `crates/*/src/` file. If the gate reveals a needed source
change, stop under §14 and open a corrective.

## 7. Ordered work packages

1. Capture HEAD, confirm `0.1.3` absent on the registry, and record the pin
   topology from §6.2.
2. Bump `[workspace.package] version` to `0.1.3`.
3. Re-pin `eggup-eggpack`'s three `=0.1.2` requirements to `=0.1.3`.
4. Regenerate `Cargo.lock`; confirm no unintended dependency change.
5. Run the full gate and the Stable/MSRV/macOS/Windows hosted matrix on the bump
   commit.
6. Cut the changelog entries.
7. `cargo package -p eggup-acquisition --locked`; inspect the packaged manifest
   and confirm it still has zero `[dependencies]`.
8. `cargo publish -p eggup-acquisition --locked --dry-run` from a clean tree.
9. Publish `eggup-acquisition 0.1.3`; verify via the crates.io API.
10. Build the external registry-only fixture: a composed-transport test that
    proves the remaining-budget behavior against the published `0.1.3`, plus a
    check that `Cargo.lock` resolves `eggup-curl`/`eggup-eggfetch` onto `0.1.3`
    with no Git or path source.
11. Write the closure record; mark this plan closed.
12. Update the roadmap status table and `plans/registry.md`.

## 8. Failure, cancellation, restart, and contention semantics

No new runtime semantics are introduced; the published code is the code already
in `0b8cb98`.

Publication is a single non-transactional upload. Partial-state semantics:

- before the upload, every step is reversible;
- after acceptance, `0.1.3` is immutable registry state. The defect is fixed for
  new resolves the moment the version is visible, because `^0.1.0` consumers
  select the highest compatible version automatically. No consumer republish or
  yank is required;
- if the upload is accepted but a later verification step fails, do **not**
  yank. Record the failure, open a corrective for the follow-on `0.1.4`, and
  state the published defect truthfully in the root `CHANGELOG.md` and the
  release notes.

If the registry/index lags, wait for visibility rather than republishing. A
second upload of the same version is rejected by the registry and is never a
recovery strategy.

## 9. Compatibility and migration

- No Rust API change. Both fixes are internal to existing error paths and to
  `ComposedTransport`'s internal budget derivation. `FetchLimits` field types,
  defaults, and validation are untouched, so this is semver-compatible within
  `0.1.x`.
- Existing `0.1.2` consumers keep resolving; no prior version is yanked.
- `^0.1.0` consumers (`eggup-curl`, `eggup-eggfetch`) pick up `0.1.3`
  automatically on the next resolve. Nothing to change on their side.
- `eggup-eggpack 0.1.2` does **not** move, because it pins `=0.1.2`. Adapter
  consumers that compose two transports need an explicit
  `eggup-acquisition 0.1.3` until `eggup-eggpack 0.1.3` is published.
- The `eggup-acquisition` seam stays usable with no Eggup or producer
  dependency, and keeps zero runtime dependencies.

## 10. Required tests

The fixes are already covered in the local tree; the new obligation is that the
**published artifact** reproduces them, not that new tests are written.

- Existing workspace tests must stay green unmodified across the version bump.
- New external registry-only fixture test: with a `FetchLimits::default()` and a
  primary transport that fails after a known delay, the secondary transport must
  observe a total timeout strictly less than the caller's `total_timeout`, and a
  call whose budget is already exhausted must return `Timeout` without invoking
  the secondary transport at all.
- New external registry-only fixture test: a fetch whose permission-hardening
  step fails must leave no part file behind.

The fixture lives outside the workspace with registry-only requirements and no
`[patch]`, path, or Git override.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
cargo tree -p eggup-acquisition --locked          # must remain empty (zero deps)
cargo package -p eggup-acquisition --locked
cargo publish -p eggup-acquisition --locked --dry-run
```

Plus the hosted `Stable checks`, `MSRV check`, `macOS tests`, and
`Windows archive, acquisition, and service tests and check` lanes on the
publication commit, and the external fixture's `cargo test` and `cargo tree`.

Platform truth to preserve: live local HTTP evidence is carried by the Linux and
macOS lanes; the Windows lane carries the portable acquisition/curl fixtures
only, and no Windows live-HTTP result is claimed.

## 12. Documentation updates

- Root `CHANGELOG.md`: new `## 0.1.3` section; the published-content caveat for
  `0.1.2` stays, annotated as resolved by `0.1.3` for `^0.1.0` consumers and
  still open for the pinned adapter.
- `crates/eggup-acquisition/CHANGELOG.md`: the `0.1.3` cut.
- `crates/eggup-curl/CHANGELOG.md` and `crates/eggup-eggpack/CHANGELOG.md`: no
  edit. Their published `0.1.2` content is immutable and this milestone does not
  change it.
- GitHub Release `0.1.2` notes: append, do not replace, that `0.1.3` exists and
  that the caveat is now resolved for caret consumers. Do not move `v0.1.2` or
  `v0.1.1`, and do not move or delete the `0.1.2` release.
- This crate's `README.md` needs no change; it does not enumerate published
  versions.
- No historical closure record is rewritten.

## 13. Acceptance criteria

- `eggup-acquisition 0.1.3` is visible on crates.io, unyanked, with a recorded
  checksum, size, publisher, and timestamp.
- The packaged manifest still has zero `[dependencies]`, and the workspace
  resolves with `eggup-eggpack` re-pinned to `=0.1.3`.
- Registry-only external resolution and build succeed with no Git or path source
  for any Eggup crate, and the composed-transport regression test passes against
  the published version.
- Current Stable/MSRV/macOS/Windows qualification is green on the publication
  commit.
- No `crates/*/src/` file changed; only manifests, `Cargo.lock`, and docs.
- Only `eggup-acquisition` was published; no other crate's `0.1.2` was
  republished or yanked.
- The `eggup-eggpack 0.1.2` pin caveat is stated truthfully in the changelog,
  the release notes, and the closure record.
- The Windows live-HTTP limitation is stated truthfully and not upgraded.

## 14. Stop conditions

Stop and open a separate corrective if:

- the version bump forces any `src/` change, or any change to a public API
  shape, `FetchLimits` field type, default, or validation message;
- `eggup-acquisition`'s packaged manifest is no longer dependency-free;
- publishing would require widening any bounded acquisition, staging, promotion,
  or redaction semantics;
- the external fixture cannot reproduce the remaining-budget or cleanup behavior
  from the published artifact;
- the hosted matrix is not green on the publication commit;
- the `eggup-eggpack` re-pin appears to require republishing the adapter to
  deliver the fix to adapter consumers — that is the follow-on milestone, not a
  reason to widen this one.

A stop produces a corrective/package-readiness plan rather than a silent change
to transport policy.

## 15. Closure evidence required

- exact publication source commit and its hosted run id;
- `cargo package` file list, size, and checksum, plus the crates.io API record;
- publish dry-run result and the publish command result;
- the external fixture's `Cargo.lock` showing registry sources and matching
  checksums, plus its test output;
- the recorded pin topology and the explicit `eggup-eggpack 0.1.2` caveat;
- proof that no `src/` file changed;
- confirmation that `v0.1.1`/`v0.1.2` and release `0.1.2` were not moved;
- per-platform qualification results, with the Windows limitation retained;
- unresolved findings, severitized.

## 16. Handoff notes

After M010, the acquisition seam's published correctness defect is closed for
every caret consumer. The remaining known gap is the `eggup-eggpack 0.1.2`
exact pin, which needs an adapter republication to pick up `0.1.3`; that is the
next candidate milestone and should be authored only as a deliberate decision,
not folded into a transport milestone.

Publishing `eggup-service` remains unauthorized. Gregg M004 remains
intentionally unwritten. Phase 12 authenticity still requires an ADR, and Phase
13 remains future API stabilization.
