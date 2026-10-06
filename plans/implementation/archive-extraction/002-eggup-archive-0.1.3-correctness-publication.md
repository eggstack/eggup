# Archive Extraction Milestone 002 — eggup-archive 0.1.3 correctness publication

Status: blocked on Acquisition M010 workspace 0.1.3 bump

Repository baseline: `44d7fdf6d31d67a2e2c8b0f62aa4f00ec365daeb`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md`

Primary class: correctness publication

## 1. Objective

Publish `eggup-archive 0.1.3` carrying the already-implemented workspace-audit fixes that are absent from published 0.1.2:

- reject Windows device aliases `COM0`, `LPT0`, `CONIN$`, `CONOUT$`, and `CLOCK$`;
- make `residue_path()` truthful by promoting retained non-empty cleanup evidence to `CleanupFailed`;
- retain the hostile tar/zip entry tests on the actual untrusted archive path.

No public API change and no new extraction feature.

## 2. Dependencies and scope

Hard dependency: workspace version 0.1.3 established by Acquisition M010.

This milestone does not require Core M013 for runtime resolution because `eggup-archive` has no runtime dependency on Core. Its path-only Core dev dependency must remain package-safe exactly as it was for 0.1.2.

Publish only `eggup-archive 0.1.3`.

## 3. Invariants

- exact allowlist extraction remains required;
- no traversal/link/special-file extraction;
- finite member/aggregate/entry/path bounds;
- handle-relative writes and handle-bound cleanup unchanged;
- SHA-256 remains integrity, not authenticity;
- no live installation mutation;
- no source edits under this publication milestone unless qualification exposes a new defect.

## 4. Required work

1. Confirm 0.1.3 absent and publish authority.
2. Cut archive/root changelog 0.1.3 entries.
3. Package and inspect manifest/file list.
4. Full Stable/MSRV/macOS/Windows qualification.
5. Publish archive 0.1.3 manually.
6. Run registry-only tar.gz + zip positive and hostile-name fixtures.
7. Append package source/checksum to GitHub Release 0.1.3 if it exists; never move its tag.
8. Write closure and unblock Eggpack Interop M005's archive dependency.

## 5. Verification

```text
cargo test -p eggup-archive --all-targets --locked
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-archive --locked
cargo publish -p eggup-archive --dry-run --locked
```

Native Windows tests must execute the reserved-device validation.

## 6. Acceptance criteria

Published 0.1.3 is registry-visible, registry-only fixtures prove both archive formats and hostile-name rejection, package API is unchanged from 0.1.2, hosted matrix is green, and no unrelated crate is published.

## 7. Stop conditions

Any source/API change, new dependency, or extraction-semantics change stops this publication plan and opens a corrective.

## 8. Closure evidence

Exact publication SHA, hosted run, package inventory/checksum, crates.io record, registry fixture output, and Release-note append.
