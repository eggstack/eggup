# Eggpack Manifest Interoperability M005 — Closure and Verification Record

Status: **closed**; `eggup-eggpack 0.1.3` published 2026-10-06

Source plan: `plans/implementation/eggpack-manifest-interoperability/005-eggup-eggpack-0.1.3-security-publication.md`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Publication source commit: `bd43683`
Hosted run (that exact commit): `37527706900` — Stable ✅ MSRV ✅ macOS ✅ Windows ✅

## Executive finding

The positional-binding defect present in published `0.1.2` is closed.
`eggup-eggpack 0.1.3` cross-checks every bound archive member against the declared
member it is about to represent and fails closed with `AdapterError::MapMismatch`
on any mismatch.

This publication also **closes the `eggup-eggpack 0.1.2` caveat** that has been
open since Acquisition M010: because the adapter now exact-pins `=0.1.3`, adapter
consumers receive the acquisition composed-transport deadline fix automatically.

## Publication record

| Field | Value |
|---|---|
| Registry version | `eggup-eggpack 0.1.3` |
| Checksum | `952e88b87ffd7b88c7db246e52169542abff9b01771f1086c22e86b4739619c5` |
| Locally built `.crate` sha256 | identical |
| Yanked | no |

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | All exact `0.1.3` dependencies visible before packaging | `eggup-acquisition 0.1.3`, `eggup-core 0.1.3`, `eggup-archive 0.1.3` all registry-visible; `eggpack-manifest 0.1.0` still published and unyanked | pass |
| 2 | Manifest exact-pins the `0.1.3` set | packaged manifest: `eggup-core =0.1.3`, `eggup-archive =0.1.3`, `eggup-acquisition =0.1.3`, `eggpack-manifest =0.1.0` | pass |
| 3 | No Git/path source in the packaged registry graph | packaged `Cargo.toml` carries version-only dependency entries; no `path`/`git`/`rev`/`branch`/`tag` | pass |
| 4 | Correct declaration-order binding still succeeds | existing `direct_positive_matches_projection_fixture`, `bundle_positive_matches_corrected_three_member_fixture`, `archive_positive_preserves_facts_and_blocks_materialization` green | pass |
| 5 | Bound extraction from a different `ArchivePlan` fails `MapMismatch` | negative fixtures green | pass |
| 6 | Reordered member projection fails `MapMismatch` | negative fixtures green | pass |
| 7 | Identical-content members cannot substitute identities | `negative_19_substituted_eggsact_member_fails_comparison` green | pass |
| 8 | Caller-bound destination negative matrix unchanged | M003a matrix (`negative_01`…`negative_20`) green | pass |
| 9 | No public API change | no `src/` change under this milestone | pass |
| 10 | Full hosted qualification | run `37527706900`, all four lanes | pass |
| 11 | Only `eggup-eggpack` published | one upload | pass |
| 12 | Release notes appended, tag not moved | `0.1.3` notes include this package, its checksum, and the caveat closure | pass |

## Registry-only graph proof

A registry-only fixture depending on the complete published set resolves
coherently, with every Eggup edge on the registry:

```text
eggup-eggpack v0.1.3
├── eggup-acquisition v0.1.3   <- the seam fix now reaches adapter consumers
├── eggup-archive    v0.1.3
└── eggup-core       v0.1.3
```

The full seven-crate set (core, acquisition, eggfetch, archive, eggpack at
`0.1.3`; curl and service at `0.1.2`) builds together from the registry alone —
**158 packages, every one a registry source**, zero path or Git edges.

## Invariant review

- The adapter selects no release.
- The adapter verifies no authenticity.
- Manifest install identity does not authorize a filesystem destination.
- Caller-bound destination APIs unchanged.
- Archive extraction stays outside the adapter.
- Exact size/digest/member relationship checks remain fail-closed.
- No new transport or TLS dependency.

## Producer/consumer boundary (ADR-0004)

Unchanged. Producer release contracts, packaging, and provenance remain
Eggpack's; this crate translates a manifest into Eggup deployment inputs and stops
at extraction-required evidence. No producer authority was added.

## Unresolved findings

| Finding | Severity | Disposition |
|---|---|---|
| None identified for this milestone | — | — |

The `eggup-eggpack 0.1.2` pin caveat, previously carried as a medium finding, is
**resolved** by this publication.