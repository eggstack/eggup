# Verified Update Core M013 — Closure and Verification Record

Status: **closed**; `eggup-core 0.1.3` published 2026-10-06; `v0.1.3` + release created

Source plan: `plans/implementation/verified-update-core/013-eggup-core-0.1.3-publication.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`

Release-prep commit: `bd43683`
Hosted run (that exact commit): `37527706900` — Stable ✅ MSRV ✅ macOS ✅ Windows ✅

## Publication record

| Field | Value |
|---|---|
| Registry version | `eggup-core 0.1.3` |
| Checksum | `161b244bc474fa24815ebd81db8cda46e0f752dd4b84983e454239f89fd789ad` |
| Locally built `.crate` sha256 | identical |
| Package | 22 files, 265.9 KiB unpacked / 58.2 KiB compressed |
| Yanked | no |
| Declared MSRV | `1.89` |
| Tag | `v0.1.3` → `bd43683` (newly created; `v0.1.1`/`v0.1.2` **not moved**) |
| Release | `0.1.3` created with per-package publication SHAs |

## Package boundary (M013 §3)

- Package inventory: 22 files; no path/Git dependency edge anywhere.
- Unix/macOS dependency surface remains `sha2` only.
- The **Windows-only** `self-replace` edge is present and correctly target-gated:
  ```toml
  [dependencies.sha2]
  version = "0.10"
  [target."cfg(windows)".dependencies.self-replace]
  version = "1.5"
  ```
- Root/core changelogs distinguish published `0.1.3` content from
  still-unpublished sibling crates at each point in the sequence.
- Only `eggup-core` was published under M013's own boundary; archive, eggfetch,
  eggpack, curl, service, and transport-footprint were published under their own
  milestones.

## Registry-only proof (M013 §4)

**Core direct fixture** — `eggup-core = "=0.1.3"`, registry-only, 5/5:

- ordinary one-member transaction committing to a real `Committed` receipt with
  the new bytes live;
- current-executable planning API (M010): a bound `CurrentExecutable` produces a
  one-member plan whose destination is the running image, whose
  `stage_placement()` is `InsideInstallationRoot`, and which carries the
  current-executable proof;
- stale-lock observation/verifier API (M011): bounded `LockObservation` carrying
  exact record bytes; default `acquire` refuses a proven-stale record and leaves
  it untouched; `acquire_with_recovery` displaces it only on `ProvenStale`;
- exhaustive `match` over the legacy `Error` variants with **no wildcard arm**;
- typed `RecoveryError` match, `evidence()` naming a real path, `Core` chaining.

**Service transitive fixture** — depends **only** on `eggup-service = "=0.1.2"`,
no `[patch]`, path, or Git override. Its lockfile resolves:

```text
eggup-service v0.1.2
└── eggup-core  v0.1.3  (registry+https://github.com/rust-lang/crates.io-index,
                         checksum 161b244b…)
```

This is the release blocker M013 §2 named: the published caret edge does not
become a broken graph when Core 0.1.3 appears. Representative M006
disposition/lifecycle usage compiles and runs against that graph.

## Compatibility

`Error` keeps **exactly** the seven variants `0.1.2` published. Proven three ways:

1. M012's byte-identical Fixture A compiles and runs against both `0.1.2` and the
   candidate, and **fails** against the pre-M012 tree.
2. M013's registry-only direct fixture repeats the exhaustive match against the
   published `0.1.3`.
3. `eggup-service 0.1.2` resolves and runs against it.

One intentional source-visible addition exists and is documented rather than
hidden: `CleanupDisposition::DeferredToProcessExit` on a closed enum, required by
M010 so a kept-installed Windows self-update does not report its still-mapped old
generation as either cleaned or stranded. Full reasoning and consumer-impact
evidence are in `plans/closure/verified-update-core/012-status.md`.

## Tag and release truthfulness

`v0.1.3` points at `bd43683`, the Core/Archive/Eggfetch publication source.
`eggup-acquisition 0.1.3` published earlier from `bb8fe41`, an ancestor of the
tag, because four publications in this train depend on it existing on the
registry. The release notes state each package's exact publication SHA and
checksum, and the tag annotation repeats it. No existing tag was moved and no
existing release was replaced.

## Consumer-unblock reconciliation

- **Gregg M004** — authorable. Acquisition 0.1.3, Core 0.1.3, and the versioned
  Core handoff all exist.
- **EggPool M007** — authorable immediately on this closure; Core 0.1.3 is
  published and registry-proven.

## Unresolved findings

| Finding | Severity | Disposition |
|---|---|---|
| `CleanupDisposition::DeferredToProcessExit` breaks exhaustive matches on that one enum | **low** | Deliberate; M010 requires it; zero `eggstack` consumers reference the type. Documented in changelog and release notes. Revisit at the 0.2/1.0 boundary. |
| `eggup-curl` sub-second-deadline tests are load-sensitive on a busy host | **low** | Pre-existing, outside the Core publication surface, hosted CI green. Worth a corrective. |

No `medium-or-higher` defect remains open. **Publication boundary respected:** no
unrelated crate was published here, and the registry state is immutable from
here — any corrective uses a new patch version, never an overwrite or a retraction.