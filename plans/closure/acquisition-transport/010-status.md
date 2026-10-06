# Acquisition Transport M010 — Closure and Verification Record

Status: **closed**; `eggup-acquisition 0.1.3` published 2026-10-06

Source plan: `plans/implementation/acquisition-transport/010-eggup-acquisition-0.1.3-publication.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`

## Executive finding

The acquisition seam's published correctness defect is closed. `eggup-acquisition
0.1.3` is on crates.io, unyanked, and carries both `0b8cb98` fixes that the
published `0.1.2` lacked. Because `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.2`
keep caret-compatible requirements, **every caret consumer now receives the fix
automatically** on the next resolve — verified below, not assumed.

One gap deliberately remains open and is stated plainly: the published
`eggup-eggpack 0.1.2` exact-pins `eggup-acquisition = "=0.1.2"` and therefore
still composes against the defective seam. Closing that needs an adapter
republication, not a transport change.

## Publication record

| Field | Value |
|---|---|
| Publication source commit | `bb8fe41219e310a0f0780b68f32e459642837972` |
| Hosted run (that exact commit) | `37525705132` — green on Stable/MSRV/macOS/Windows |
| Registry version | `eggup-acquisition 0.1.3` |
| Registry checksum | `71ce1a0d9fd4e07d2921d5d6e8e7bf9c2da481be492ef9712ef4f104dda285a7` |
| Locally built `.crate` sha256 | `71ce1a0d9fd4e07d2921d5d6e8e7bf9c2da481be492ef9712ef4f104dda285a7` — **identical** |
| Package size | 7 files, 87.0 KiB unpacked / 19,669 bytes `.crate` |
| Yanked | no |
| Published at | `2026-10-06T20:32:19Z` |
| Declared MSRV | `1.89` |

The packaged `.cargo_vcs_info.json` records `sha1 = bb8fe41…`, so the published
bytes provably originate from the CI-qualified commit.

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | `0.1.3` absent before publication | crates.io API showed `max_version = 0.1.2` pre-upload | pass |
| 2 | Workspace version → `0.1.3` | `Cargo.toml` `[workspace.package]`; all 7 crates inherit | pass |
| 3 | `eggup-eggpack` re-pinned to `=0.1.3` | `crates/eggup-eggpack/Cargo.toml` | pass |
| 4 | Workspace resolves; no unintended dep change | `Cargo.lock` diff is **version-only**, 8 insertions / 8 deletions, zero dependency-set change | pass |
| 5 | Zero `crates/eggup-acquisition/src/` change | `git diff 0b8cb98 -- crates/eggup-acquisition/src/` empty; `git status` shows no `src/` file touched | pass |
| 6 | Packaged manifest still dependency-free | packaged `Cargo.toml` has an empty `[dependencies]`; `cargo tree -p eggup-acquisition` prints only the crate itself | pass |
| 7 | Local gate green | `./scripts/check-local.sh` exit 0, **381 tests** | pass |
| 8 | MSRV green | `cargo +1.89.0 test --workspace --all-targets --locked` exit 0, 381 tests | pass |
| 9 | Hosted matrix green on publication commit | run `37525705132`: Stable ✅ MSRV ✅ macOS ✅ Windows ✅ | pass |
| 10 | Registry-visible, unyanked | crates.io API `0.1.3`, `yanked: false` | pass |
| 11 | Registry checksum == locally built `.crate` | both `71ce1a0d…a285a7` | pass |
| 12 | Registry-only fixture passes | 4/4 against published `=0.1.3`, lockfile `source = registry+…` with matching checksum | pass |
| 13 | Composed-transport fix reproduced from published artifact | fixture asserts the secondary receives a **strictly smaller** `total_timeout`, and `connect_timeout <= total_timeout` | pass |
| 14 | Exhausted budget never starts a doomed attempt | fixture asserts typed `Timeout` and `secondary.calls() == 0` | pass |
| 15 | Caret consumers float onto the fix | `eggup-curl 0.1.2` → `eggup-acquisition v0.1.3`; same for `eggup-eggfetch 0.1.2` | pass |
| 16 | No Git/path source in the resolved graph | every `source =` in the caret fixture lock is the crates.io registry | pass |
| 17 | Only `eggup-acquisition` published | no other crate was uploaded this milestone | pass |
| 18 | `v0.1.1`/`v0.1.2` and release `0.1.2` unmoved | no tag or release operation performed | pass |

## Detection-gap evidence (the fixture is discriminating, not decorative)

The registry-only fixture was run against the **published defective `0.1.2`** as
a control. It fails there and passes against `0.1.3`:

```text
eggup-acquisition =0.1.2  -> 2 passed, 2 FAILED
    secondary_receives_only_the_remaining_budget ... FAILED
    exhausted_budget_reports_timeout_without_invoking_the_secondary ... FAILED
eggup-acquisition =0.1.3  -> 4 passed, 0 failed
```

A fixture that passed against both would prove nothing.

## Invariant review

- `FetchLimits` field types, defaults, and validation messages are **unchanged**;
  semver-compatible within `0.1.x`, so no consumer migration is required.
- The part file is still created exclusively, owner-private (`0600` on Unix),
  never symlink-followed, promoted with race-safe no-clobber semantics. The fix
  only removes the candidate on the *securing* error path; the `0600` hardening
  itself is untouched.
- Transport fallback remains **not** release/source fallback: an exact
  `NotFound` stays terminal for the requested URL.
- Diagnostic redaction unchanged; no raw upstream, command-line, or credential
  material is embedded.
- Cleanup ownership unchanged: only an owned temp is ever removed, and a complete
  destination is never reported as an ordinary failure.
- No Eggup or producer dependency; the seam remains independently usable.

## The eggpack caveat — still open, stated truthfully

| Crate | Requirement on `eggup-acquisition` | Inherits `0.1.3`? |
|---|---|---|
| `eggup-curl 0.1.2` | `^0.1.0` | **yes** (verified) |
| `eggup-eggfetch 0.1.2` | `^0.1.0` | **yes** (verified) |
| `eggup-eggpack 0.1.2` | `=0.1.2` | **no** |

`eggup-eggpack 0.1.2` therefore keeps composing two transports against the
defective seam. Consumers of that adapter must pin `eggup-acquisition 0.1.3`
themselves or tolerate the doubled composed window until `eggup-eggpack 0.1.3`
publishes. Recorded in the root `CHANGELOG.md` under both `0.1.2` and `0.1.3`.

## Unresolved findings

| Finding | Severity | Disposition |
|---|---|---|
| `eggup-eggpack 0.1.2` does not receive the seam fix | **medium** | Open by design. Tracked as Eggpack M005; requires adapter republication, not a transport change. Not a reason to widen M010. |
| `eggup-curl` sub-second-deadline tests are load-sensitive on a busy host | **low** | Pre-existing and **not** caused by this milestone (reproduced at pristine HEAD under load). Two tests, `build_curl_args_passes_sub_second_deadlines_to_fake_curl` and `sub_second_total_timeout_kills_child_promptly`, carry sub-second wall-clock budgets that a loaded machine can exceed. Hosted CI passes both. Out of M010 scope (`eggup-curl` source was explicitly excluded); worth a corrective. |

No `medium-or-higher` defect remains open for the acquisition seam itself.

## Disposition

**Closed.** The seam's published correctness defect is fixed for every caret
consumer and the `eggup-eggpack 0.1.2` exception is documented at every surface
it can mislead a reader. Acquisition M011 (eggfetch 0.1.3) and Archive M004 are
unblocked.