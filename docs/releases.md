# Versions and upgrades

This page was checked against exact-version crates.io API reads on 2026-11-22. Core, Archive, Acquisition,
Eggfetch, and Eggpack are at `0.2.2` and unyanked; Curl and Service remain at `0.2.0`.
Core M022 published only `eggup-core 0.2.2` from source SHA
`cdfe0bf0452d64c50da00a9e21c62ad2a0140bff` (checksum
`a0b0dba6035ec7d403ddf0298f1021a5bd544764f9bb4e3011cb31b2dacd614b`). Archive M009 published only
`eggup-archive 0.2.2` from source SHA `4e8779ed37ca35f3f11857dbcc4aa008e386bf21` (checksum
`67ab697e3af41b3cd3d1ed140f821c68739ec446d2a7782fa26ca4960edc882b`). Acquisition Transport M023
published `eggup-acquisition 0.2.2` (checksum
`4000935b1d4d384eaba066c9742e70f91d5cb3dd12618ac3aa79d654e2aab865`) and `eggup-eggfetch 0.2.2`
(checksum `f683e0c3b377a1713a5af4a64257f785da24f17771a927c07508b0d73ad35df1`) from source SHA
`cf6fcc1706f353a0c5e4c4d3e0071c482993f974`; registry archives and Cargo provenance match.
Post-publication hosted run `44985695980` attempt 2 passed all five lanes. Eggpack M010 then published
only `eggup-eggpack 0.2.2` from source SHA `c9a7ac0af7ea3f0a090d19fe1fffa817235f0081` (crates.io
SHA-256 `f24df78cf088e665dd69db1b319d7168007175ebc1ec5d2df76a069e31242e49`). The downloaded registry
archive matches the qualified local package byte-for-byte and Cargo provenance records that source SHA.
The normalized manifest exact-pins Core, Archive, and Acquisition at `=0.2.2`, plus
`eggpack-manifest =0.1.2`; the independent registry-only consumer fixture passes. Post-publication
hosted run `44988995059` attempt 2 passed all five lanes on the publication SHA. The immutable 0.2.2
archive also carries the pre-publication README and CHANGELOG from that source SHA: the README says the
source update was not yet published, and the CHANGELOG leaves M010 under Unreleased. The repository copies are
now reconciled; correcting the already-published archive docs would require a future Eggpack version. This
documentation-only limitation does not affect API, runtime behavior, or dependency resolution. No tags, GitHub Releases, or PRs were changed. See `plans/closure/verified-update-core/022-status.md`,
`plans/closure/archive-extraction/009-status.md`,
`plans/closure/acquisition-transport/023-status.md`, and
`plans/closure/eggpack-manifest-interoperability/010-status.md`.

Archive M008 and Acquisition M019 separately published `eggup-archive 0.2.1` and
`eggup-acquisition 0.2.1` from `5a04f68f60c8c3ad6392036fa314ed2749fa572c`. Eggpack M009
published `eggup-eggpack 0.2.1` from `ccb2bfcbbe7645541d43bb4f5fcb801339c319ed`
(crates.io SHA-256 `492271d54d9529fcef74bcf354466c1971e102c261e7ceac696ea3746fded795`).
Eggpack's published 0.2.1 package exact-pins Core, Archive, and Acquisition at `=0.2.1`
and eggpack-manifest at `=0.1.2`; its API and behavior are unchanged. Acquisition
Transport M020 published `eggup-eggfetch 0.2.1` from source commit
`2a33cfe4712dfb88a823614ee6f6a439ecdf792e` with checksum
`2ca53436f6f12ca3ae0a9d194e81a6af6bf04fadf2d90b49d866359ab9eab2dc`; it updates the
implementation dependency to `eggfetch-core 0.3.0` without a public API or
acquisition-policy change. M021 later moved only the workspace source to eggfetch-core
0.4.0; the published 0.2.1 artifact remains on 0.3.0. The `v0.2.1` tag and GitHub
Release still point at Core's 0.2.1 source commit; M022 did not move them. The immutable
Core 0.2.0 package lacks the contract documents, and Eggpack 0.2.0 remains unchanged.
Check crates.io again when choosing a version, since registry state can change
independently of this repository.

| Crate | Latest published version |
|---|---|
| [`eggup-core`](https://crates.io/crates/eggup-core) | `0.2.2` |
| [`eggup-acquisition`](https://crates.io/crates/eggup-acquisition) | `0.2.2` |
| [`eggup-eggfetch`](https://crates.io/crates/eggup-eggfetch) | `0.2.2` |
| [`eggup-curl`](https://crates.io/crates/eggup-curl) | `0.2.0` |
| [`eggup-archive`](https://crates.io/crates/eggup-archive) | `0.2.2` |
| [`eggup-eggpack`](https://crates.io/crates/eggup-eggpack) | `0.2.2` |
| [`eggup-service`](https://crates.io/crates/eggup-service) | `0.2.0` |

## Exact-pin behavior

The workspace crates generally use caret requirements for their Eggup
dependencies. Published `eggup-eggpack 0.2.2` pins eggup-core, eggup-archive,
and eggup-acquisition exactly to `=0.2.2`, plus eggpack-manifest `=0.1.2`.
The immutable published `eggup-eggpack 0.2.1` retains `=0.2.1` Eggup pins
and eggpack-manifest `=0.1.2`; published `eggup-eggpack 0.2.0` retains its
original exact pins to the 0.2.0 Eggup set and eggpack-manifest 0.1.0. A later
exact-pin update will not flow through an older immutable adapter; consumers
must select a new Eggpack publication to receive a later dependency set. The
current workspace-source pins are also `=0.2.2`. M016's contract documents
are published in Core 0.2.1; the immutable Eggpack 0.2.0 package still resolves
Core 0.2.0 and does not carry those documents. Check the resolved dependency
graph if you depend on `eggup-eggpack`.

## Earlier-version limitations

The following issues affect consumers pinned to the listed older releases. The
`0.2.0` line includes the fixes.

| Crate and affected version | Limitation corrected in `0.2.0` |
|---|---|
| `eggup-core 0.1.3` | `run_bounded` could exceed its deadline when a descendant held an output pipe open, and bounded stdin was unavailable. |
| `eggup-acquisition 0.1.2` | A composed fetch could exceed the caller's total timeout when trying a fallback transport. |
| `eggup-service 0.1.3` | A successful crontab stdin write could be reported as a failure. |
| `eggup-eggpack 0.1.2` | Archive members were bound positionally without checking member identity. |

Consult the relevant crate changelog for details. Registry versions are
immutable, so consumers who must remain on one of these older releases retain
its limitation.

## Migrating to `0.2.0`

`0.2.0` includes source-breaking changes. Review each crate's changelog before
upgrading. The main changes are:

- **`eggup-acquisition`:** `AcquisitionError::Io(String)` became
  `AcquisitionError::Io { operation, source }`. The error no longer implements
  `Clone`, `PartialEq`, or `Eq`.
- **`eggup-service`:** `ServiceError::Manager(detail)` became
  `ServiceError::Manager { kind, detail, os_code }`. `LifecycleUpdateReceipt`
  also gained the `engine_policy` field, which must be included in struct
  literals.
- **`eggup-core`:** `FailureCategory::RetainedEvidence` was removed. Code that
  names this variant must be updated.

There are also behavior changes to review:

- Cron managed blocks must include the spec's absolute executable path.
- `eggup-archive` cleanup can now succeed. Handle both `Ok` and `Err` results.

For detailed release notes, see the crate changelogs. The
[crate guide](crates.md) explains which crates fit each use case.
