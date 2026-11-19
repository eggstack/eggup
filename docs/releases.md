# Versions and upgrades

This page was checked against crates.io on 2026-11-18. The latest published
version is **0.2.1** for `eggup-core` and **0.2.0** for the other six library
crates. M016 published only Core `0.2.1` from source commit
`b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; all eight workspace crates still
share source version `0.2.1`. The `v0.2.1` tag and GitHub Release point at that
source. The immutable Core `0.2.0` package remains unchanged and lacks the
three contract documents. Check crates.io again when choosing a version, since
registry state can change independently of this repository.

| Crate | Latest published version |
|---|---|
| [`eggup-core`](https://crates.io/crates/eggup-core) | `0.2.1` |
| [`eggup-acquisition`](https://crates.io/crates/eggup-acquisition) | `0.2.0` |
| [`eggup-eggfetch`](https://crates.io/crates/eggup-eggfetch) | `0.2.0` |
| [`eggup-curl`](https://crates.io/crates/eggup-curl) | `0.2.0` |
| [`eggup-archive`](https://crates.io/crates/eggup-archive) | `0.2.0` |
| [`eggup-eggpack`](https://crates.io/crates/eggup-eggpack) | `0.2.0` |
| [`eggup-service`](https://crates.io/crates/eggup-service) | `0.2.0` |

`eggup-transport-footprint` is not published. The earlier `0.1.x` releases
remain available. Because Eggup is pre-1.0, a `0.1` requirement does not accept
`0.2.1`; update your dependency requirement when upgrading.

## Exact-pin behavior

The workspace crates generally use caret requirements for their Eggup
dependencies. The published `eggup-eggpack 0.2.0` pins its Eggup dependencies exactly to
`0.2.0`, so a later fix to `eggup-core`, `eggup-archive`, or
`eggup-acquisition` will not reach that published adapter until it is
republished. The local workspace pins are aligned to `=0.2.1`; this does not
change the published adapter. The M016 contract documents are available to direct Core
consumers resolving `eggup-core 0.2.1`, not through the immutable
`eggup-eggpack 0.2.0` package. Check the resolved dependency graph if you
depend on `eggup-eggpack`.

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
