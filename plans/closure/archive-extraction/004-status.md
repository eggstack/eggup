# Archive Extraction M004 — Closure and Verification Record

Status: **closed**; `eggup-archive 0.1.3` published 2026-10-06

Source plan: `plans/implementation/archive-extraction/004-eggup-archive-0.1.3-correctness-publication.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md`

Publication source commit: `bd43683`
Hosted run (that exact commit): `37527706900` — Stable ✅ MSRV ✅ macOS ✅ Windows ✅

## Publication record

| Field | Value |
|---|---|
| Registry version | `eggup-archive 0.1.3` |
| Checksum | `a28af652ec11b5309eb7b628d0c875262be2c491a7fb3c5d3fb6c50eabe9824c` |
| Locally built `.crate` sha256 | identical |
| Package | 7 files, 151.9 KiB unpacked / 30.2 KiB compressed |
| Yanked | no |
| Runtime dependency on `eggup-core` | **none**; dev-only path dependency preserved and package-safe |

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | `0.1.3` absent; publish authority confirmed | registry `max_version 0.1.2`; crate owned | pass |
| 2 | Crate/root 0.1.3 changelog cut | `crates/eggup-archive/CHANGELOG.md`, root `CHANGELOG.md` | pass |
| 3 | Package API unchanged from `0.1.2` | no source change under this milestone; changelog-only | pass |
| 4 | Windows device aliases rejected | fixture rejects `COM0`, `LPT0`, `COM1`, `LPT9`, `CONIN$`, `CONOUT$`, `CLOCK$` | pass |
| 5 | Near-miss names still valid | fixture accepts `com10`, `com11`, `lpt10`, `console`, `auxiliary`, `clock` | pass |
| 6 | Hostile entries rejected on the real archive path | fixture writes **raw ustar headers** (the `tar` crate refuses to encode `..`, so the header block is hand-assembled) and verbatim zip names; every traversal name rejected as `InvalidPath`; nothing escapes the output parent | pass |
| 7 | `residue_path()` is truthful | fixture asserts cleanup either removes the private root **or** fails with `CleanupFailed` and a `residue_path()` that genuinely exists on disk — it may never report "cleaned" while bytes remain | pass |
| 8 | Both archive formats positive path | tar.gz and zip extraction verified against the published crate, byte-exact with matching SHA-256 | pass |
| 9 | Native Windows reserved-device validation executed | hosted Windows lane (run `37527706900`) runs the archive test set | pass |
| 10 | Full hosted qualification | run `37527706900`, all four lanes | pass |
| 11 | Registry-only fixtures | no `[patch]`/path/Git; lockfile entirely registry sources | pass |
| 12 | No unrelated crate published in this milestone | one upload | pass |
| 13 | Release notes appended, tag not moved | `0.1.3` notes include this package and checksum | pass |
| 14 | Eggpack M005 archive dependency unblocked | `eggup-eggpack 0.1.3` published against `eggup-archive =0.1.3` | pass |

## Detection-gap evidence

```text
eggup-archive 0.1.2 -> 5 passed, 1 FAILED
    windows_device_aliases_are_rejected ... FAILED (`COM0` was accepted)
eggup-archive 0.1.3 -> 6 passed, 0 failed
```

## Invariant review

- Exact allowlist extraction still required.
- No traversal, link, or special-file extraction; regular files only.
- Finite member / aggregate / entry / path bounds unchanged.
- Handle-relative writes and handle-bound cleanup unchanged.
- SHA-256 remains integrity evidence only — **no authenticity claim**, no
  signature support, and none implied by this publication.
- No live-installation mutation: the crate extracts into a caller-owned output
  parent and never touches a live installation.
- No source edits under this publication milestone.

## Unresolved findings

| Finding | Severity | Disposition |
|---|---|---|
| None identified | — | — |

No `medium-or-higher` defect remains open for `eggup-archive`.