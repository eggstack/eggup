# Versions and upgrades

This page was checked against crates.io on 2026-11-19. The latest published
version is **0.2.1** for `eggup-core`, `eggup-archive`, `eggup-acquisition`,
and `eggup-eggpack`; the other three library crates (`eggup-curl`,
`eggup-eggfetch`, and `eggup-service`) remain at **0.2.0**. Core M016
published `eggup-core 0.2.1` from
`b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; Archive M008 and Acquisition
M019 separately published `eggup-archive 0.2.1` and
`eggup-acquisition 0.2.1` from
`5a04f68f60c8c3ad6392036fa314ed2749fa572c`. Eggpack M009 published
`eggup-eggpack 0.2.1` from
`ccb2bfcbbe7645541d43bb4f5fcb801339c319ed` (crates.io SHA-256
`492271d54d9529fcef74bcf354466c1971e102c261e7ceac696ea3746fded795`).
Eggpack's published 0.2.1 package exact-pins Core, Archive, and Acquisition
at `=0.2.1` and eggpack-manifest at `=0.1.2`; its API and behavior are
unchanged. All eight workspace crates share source version `0.2.1`. The
`v0.2.1` tag and GitHub Release still point at Core's source commit; these
selective publications did not move the tag. The immutable Core `0.2.0`
package lacks the contract documents, and Eggpack `0.2.0` remains unchanged.
Check crates.io again when choosing a version, since registry state can change
independently of this repository.

| Crate | Latest published version |
|---|---|
| [`eggup-core`](https://crates.io/crates/eggup-core) | `0.2.1` |
| [`eggup-acquisition`](https://crates.io/crates/eggup-acquisition) | `0.2.1` |
| [`eggup-eggfetch`](https://crates.io/crates/eggup-eggfetch) | `0.2.0` |
| [`eggup-curl`](https://crates.io/crates/eggup-curl) | `0.2.0` |
| [`eggup-archive`](https://crates.io/crates/eggup-archive) | `0.2.1` |
| [`eggup-eggpack`](https://crates.io/crates/eggup-eggpack) | `0.2.1` |
| [`eggup-service`](https://crates.io/crates/eggup-service) | `0.2.0` |

## Exact-pin behavior

The workspace crates generally use caret requirements for their Eggup
dependencies. Published `eggup-eggpack 0.2.1` pins eggup-core, eggup-archive,
and eggup-acquisition exactly to `=0.2.1`, plus eggpack-manifest `=0.1.2`.
The immutable published `eggup-eggpack 0.2.0` retains its original exact pins
to the 0.2.0 Eggup set and eggpack-manifest 0.1.0. A later exact-pin update
will not flow through either immutable adapter version; consumers must select a
new Eggpack publication to receive a later dependency set. The local workspace
pins are aligned to `=0.2.1`. M016's contract documents are published in Core
0.2.1; the immutable Eggpack 0.2.0 package still resolves Core 0.2.0 and does
not carry those documents. Check the resolved dependency graph if you depend
on `eggup-eggpack`.

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
