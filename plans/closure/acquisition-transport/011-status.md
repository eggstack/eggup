# Acquisition Transport M011 — Closure and Verification Record

Status: **closed**; `eggup-eggfetch 0.1.3` published 2026-10-06

Source plan: `plans/implementation/acquisition-transport/011-eggup-eggfetch-0.1.3-correctness-publication.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`

Publication source commit: `bd43683`
Hosted run (that exact commit): `37527706900` — Stable ✅ MSRV ✅ macOS ✅ Windows ✅

## Publication record

| Field | Value |
|---|---|
| Registry version | `eggup-eggfetch 0.1.3` |
| Checksum | `e2bdffe9c66edd94631d91821c9aa24dcc87d17bf39d615315423f5bffe56ada` |
| Locally built `.crate` sha256 | identical |
| Package | 8 files, 82.1 KiB unpacked / 21.8 KiB compressed |
| Yanked | no |
| Resolved seam | `eggup-acquisition 0.1.3` (caret requirement, registry-only lockfile) |

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | `0.1.3` absent before publication; ownership confirmed | registry showed `max_version 0.1.2`; crate already owned | pass |
| 2 | No source change beyond the already-qualified audit fixes | three fixes verified present in `src/lib.rs` before publishing: `.map_err(... Unavailable)` runtime path (L245-248), `thread_local!` runtime (L196-232), `TooLarge { limit: <real bound> }` (L314/322/427) | pass |
| 3 | Crate/root 0.1.3 changelog cut | `crates/eggup-eggfetch/CHANGELOG.md`, root `CHANGELOG.md` | pass |
| 4 | No public API / transport-policy change | `Error`-surface and `FetchLimits` untouched; no release/proxy/redirect policy moved | pass |
| 5 | Packaged graph resolves registry `eggup-acquisition 0.1.3` | registry-only fixture lockfile, all `source = registry+…` | pass |
| 6 | Runtime-construction failure is typed, not a panic | fixture passes against `0.1.3`; **aborts the process** against `0.1.2` (`fatal runtime error: thread local panicked on drop, aborting`) | pass |
| 7 | `TooLarge.limit` equals the enforced bound | fixture asserts `limit == 256` for a `max_metadata_bytes: 256` fetch and `limit == 512` for a `max_artifact_bytes: 512` fetch; also asserts no part file survives | pass |
| 8 | `EggfetchTransport` still shareable across threads | compile-time `assert_send_sync::<EggfetchTransport>()` plus 4 threads × 3 concurrent fetches on one shared `Arc` | pass |
| 9 | Full hosted qualification | run `37527706900`, all four lanes | pass |
| 10 | Registry-only fixture, no path/Git override | fixture manifest has no `[patch]`; lockfile entirely registry sources | pass |
| 11 | Only `eggup-eggfetch` published in this milestone | one upload | pass |
| 12 | Release notes appended without moving the tag | `0.1.3` release notes include this package and its checksum | pass |

## Detection-gap evidence

The registry-only fixture was run against the published `0.1.2` as a control:

```text
eggup-eggfetch 0.1.2 -> transport test ok, TooLarge tests ok,
                      runtime test FAILED with a process abort
                      ("fatal runtime error: thread local panicked on drop")
eggup-eggfetch 0.1.3 -> 4 passed, 0 failed
```

This is the sharpest evidence in the 0.1.3 train: on `0.1.2` the defect is not a
wrong value, it is the process dying from inside a fallible API.

## The deterministic seam for "no panic"

The plan asks for proof "through the existing deterministic seam". `block_on` is
private, so the fixture reproduces the failure externally by exploiting thread-local
teardown ordering, which is deterministic rather than timing-dependent:

1. A fixture-owned `thread_local!` is initialized **first**.
2. The adapter is then used on that thread, initializing **its** runtime
   thread-local.
3. Thread exit destroys thread-locals in **reverse** initialization order, so the
   adapter's runtime is already gone when the fixture's destructor runs.
4. The destructor calls `fetch_metadata` and reports the outcome over a channel.

Pre-fix that path reached `.expect()` and aborted. Post-fix it returns a typed
`Unavailable`. The fixture asserts the report arrived at all, and that the message
contains neither a panic nor an unwrap.

## Honest limit on one claim

The `TooLarge { limit: 0 }` fix is verified **in-crate** (the `map_fetch_error`
unit test asserts a real bound is threaded through), not externally. The
artifact-path test passes against `0.1.2` as well, because `0.1.2` already passed
the bound at the *streaming* check and only omitted it at the
`DecodedBodyTooLarge` call sites — which the public `EggfetchConfig` gives no way
to induce. Recorded as **not externally reachable** rather than claimed as proven.

## Invariant review

- HTTPS/proxy/redirect policy unchanged; explicit finite connect/total/artifact
  bounds unchanged.
- `EggfetchTransport` remains shareable across threads.
- No new runtime or API feature.
- Diagnostic redaction and bounded-error behaviour unchanged.
- The adapter selects no release and infers no version.

## Unresolved findings

| Finding | Severity | Disposition |
|---|---|---|
| `TooLarge.limit` fix not externally reproducible | **informational** | In-crate test covers it; public config cannot reach the affected branch. No action. |
| `eggup-curl` sub-second-deadline tests are load-sensitive on a busy host | **low** | Pre-existing, out of M011 scope, hosted CI passes. Worth a corrective. |

No `medium-or-higher` defect remains open for `eggup-egfetch`. Archive M004 and
Eggpack M005 were unblocked by the acquisition 0.1.3 + workspace 0.1.3 state and
have since closed.