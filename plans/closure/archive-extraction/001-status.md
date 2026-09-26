# Archive Extraction M001 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md#M001--bounded-allowlisted-extraction-contract`

Reviewed repository baseline: `b619fbd3821047e8e317343dbe49c3a41e6fbfc2` (plan-registration head; clean before the requested implementation batch)

Implementation commit: `230f7682274112c9ec1edb5b7b7d9eddc906fe7c` — bounded allowlisted archive extractor, crate/workspace documentation, dependency lock, and Windows runtime test lane.

Hosted qualification: CI run [36214688691](https://github.com/eggstack/eggup/actions/runs/36214688691) on the implementation commit; stable Linux, Rust 1.89, macOS tests, Windows archive/acquisition runtime tests, and Windows workspace check all passed.

## Executive finding

The optional `eggup-archive` leaf crate now extracts explicit regular-file members from already verified local tar.gz and zip archives. It uses declaration-driven slash paths, finite archive/entry/path/member/aggregate limits, streamed SHA-256 and optional exact size/digest checks, exclusive private output creation, deterministic output order, and cleanup ownership guards. It never invokes an external extractor, performs network access, or mutates the live installation. `eggup-core` continues to depend only on SHA-256 support and does not acquire archive-format dependencies.

The hosted Windows test lane executes the archive suite, including a two-member zip fixture. No high- or medium-severity extraction issue remains open. M001 is closed.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| tar.gz and zip allowlisted regular-file extraction | `valid_tar_gz_extracts_allowlisted_members_in_declaration_order`; `valid_zip_extracts_members_and_returns_digest_evidence` (two members) | passed on Linux, macOS, Windows |
| deterministic declaration order and exact required-member completion | two-member format tests; missing/duplicate archive entry tests | passed |
| traversal, rooted, drive, backslash, dot, repeated-separator, overlong, and output-name ambiguity rejection | `archive_paths_reject_traversal_absolute_prefix_alias_and_overlong_names`; `output_names_reject_windows_reserved_or_ambiguous_names` | passed |
| duplicate paths, links, and special-file refusal | duplicate archive-member, tar link/FIFO/device, and zip symlink fixtures | passed |
| no-clobber and owner-private extraction | exclusive output preservation and Unix 0700 root/0600 file mode tests | passed; Unix mode assertions skipped on Windows |
| finite compressed, entry, per-member, aggregate, and path limits | archive-size/entry-count and per-member/aggregate overflow fixtures; all counters enforced during reads before writes | passed |
| exact size and SHA-256 evidence | positive fixture digest evidence and size/digest mismatch fixtures | passed |
| malformed/truncated archive and gzip footer rejection | corrupt/truncated tar.gz and zip tests, including truncated gzip trailer | passed |
| operation-owned cleanup and residue behavior | partial failure cleanup, drop cleanup, explicit persist/cleanup, and simulated replacement-path cleanup-failure test | passed; residue reported without deleting replacement |
| ordinary core handoff | extracted file is passed as an ordinary `ArtifactMember` to `eggup-core` prepare | passed; no archive dependency added to core |
| portable runtime coverage | hosted run 36214688691: Linux archive tests 18/18, macOS 18/18, Windows 17/17 (Unix-only permission test is cfg-gated) | passed |
| dependency/MSRV/quality qualification | locked dependency trees, local Rust 1.89 check, full gate, hosted stable/MSRV/macOS/Windows | passed |

## Production implementation evidence

- `crates/eggup-archive/src/lib.rs`: validated finite `ArchiveLimits`, `ArchiveMember`, and `ArchivePlan`; UTF-8 relative slash-path validation; ASCII portable one-component output names; explicit tar entry-type checks and zip regular-file checks; fixed 16 KiB streaming with limits and SHA-256; exclusive private root and file creation; explicit cleanup/persist ownership.
- tar iteration drains and validates the gzip trailer after tar end markers, rejecting nonzero trailing data and bounding accepted zero padding. ZIP entries are read by index and materialized only after path, allowlist, and regular-file checks.
- `crates/eggup-archive/README.md`, crate `CHANGELOG.md`, and `architecture/archive-extraction.md` document caller-owned archive verification, flush-only durability, cleanup, and the non-live-install boundary. Root README and architecture inventories include the optional crate.
- `.github/workflows/ci.yml` runs `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked` on Windows before the workspace check.

## Exact commands and results

Local Darwin arm64:

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test --workspace --all-targets --all-features --locked    passed; archive 18/18
cargo doc --workspace --no-deps --locked                        passed
cargo check --workspace --all-targets --locked                   passed
cargo +1.89.0 check --workspace --all-targets --locked           passed
cargo tree --workspace --locked                                  passed
cargo tree -p eggup-core --locked                                passed; sha2 only
cargo tree -p eggup-archive --locked                             passed; versions/features recorded below
cargo test -p eggup-archive --locked                             passed; 18/18
./scripts/check-local.sh                                         passed
git diff --check                                                passed
```

Hosted CI 36214688691:

- Stable Linux: format, clippy, full workspace tests (archive 18/18), and docs passed.
- MSRV Linux: Rust 1.89 workspace all-target check passed.
- macOS: full workspace tests passed (archive 18/18).
- Windows: acquisition/curl/archive portable tests passed (archive 17/17); workspace all-target check passed. Archive suite includes runtime zip decode/extract fixtures; only Unix permission-mode assertions are excluded.

The final documentation/closure edits are in a follow-up commit; implementation code and CI workflow were fully tested at the implementation commit above. `git diff --check` and formatting/doc gates are rerun before the closure commit is pushed.

## Dependency and footprint review

Pinned lockfile selections: `flate2 1.1.10` with `default-features = false, features = ["rust_backend"]`; `tar 0.4.46` with default features disabled; `zip 6.0.0` with default features disabled and only `deflate-flate2`; and `sha2 0.10.9`. The crate has no feature switches: tar.gz and zip are both always available when a consumer opts into this separate crate, with no safety-semantic changes under `--all-features`.

`cargo tree -p eggup-archive -e normal --prefix none --locked` reports 25 unique package entries in the normal dependency closure; the corresponding `eggup-core` normal closure reports 12 entries (`sha2` and its transitive dependencies only). The archive-specific additions are `flate2`/miniz, `tar`/`filetime`, and `zip`/`indexmap`/`memchr` plus shared compression/checksum dependencies. No standalone binary exists in this workspace to measure a binary-size delta; the optional leaf boundary keeps that cost out of core and consumers that do not select this crate.

## Invariant review

- The source archive must be a regular local file under the compressed-size cap. Integrity/authenticity verification remains a caller precondition and is not asserted by this crate.
- Every observed entry is parsed within the count/path/decompressed-byte limits. Only exact declared regular members are materialized. Undeclared regular members are drained but never written.
- No path is sanitized into a different path. Output names are caller-selected, portable single components, and file creation is exclusive.
- Extraction output has no live destination ownership. It is handed to a later ordinary `ArtifactSet`/core preparation flow.
- SHA-256 is byte-integrity evidence only; no authenticity, crash durability, resume, restart journal, or live recovery claim is made.
- `eggup-core` has no archive-format dependency. No external process, shell, network, metadata restoration, links, or special files are involved in materialization.

## Failure and recovery review

| Failure | Result |
|---|---|
| Unsafe/ambiguous path, duplicate member, link/special declared member | typed failure before success; owned partial root is removed |
| Malformed or truncated tar.gz/zip, including gzip integrity trailer | typed malformed-archive failure; owned partial root is removed |
| Compressed, entry, per-member, or aggregate bound exceeded | typed finite-limit failure before excess bytes are written |
| Exact size or digest mismatch | typed verification failure; partial root cleanup attempted |
| Existing/raced output file | `create_new` failure; existing file is preserved |
| Cleanup of owned root fails | `CleanupFailed` plus residue path; no live-install recovery state is implied |
| Dropped unused result | best-effort removal of only the guarded operation root |
| Explicit persisted cleanup fails | returns residue evidence and preserves an unexpected replacement at the original path |

All failures precede core preparation and live mutation. Retry uses a new exclusive root; extraction is not resumable.

## Compatibility and migration review

This adds an optional unpublished workspace crate and does not change an existing public API or migrate consumers. No consumer is silently opted into archive dependencies. The crate `CHANGELOG.md` has an `Unreleased` entry and states that nothing has been published and no consumer migration is required. Egress remains consumer-owned until its own plan is authored and closed; `eggup-eggpack` still stops at extraction-required evidence pending a separate handoff plan.

## Security review

The boundary is deliberately local and declaration-driven. Unsafe traversal aliases, duplicate paths, link/special types, zip symlinks, nonportable output names, output clobbering, malformed streams, and decompression overages have negative tests. Archive parsing libraries are used for entry access only; Eggup creates and writes output handles itself. No medium-or-higher issue remains open.

## Documentation and operations evidence

- New crate README and `Unreleased` changelog entry.
- New `architecture/archive-extraction.md`, workspace inventory, architecture overview flow, and tooling-governance crate list.
- Archive roadmap, consumer/Eggpack dependency blockers, plan registry, and this closure record updated after successful hosted qualification.
- No publication, consumer change, or release occurred.

## Unresolved findings

- None: no high- or medium-severity extraction issue remains.
- Informational: this is intentionally not a general archive restoration library; UTF-8 slash-relative names, ASCII output filenames, tar.gz, zip, and regular files are the qualified surface. Adding formats, links, metadata restoration, or features needs separate consumer evidence and review.

## Roadmap disposition and downstream unblock

Archive M001 moves from ready to closed. This closes the only stated hard dependency for two future work items:

- Consumer Adoption M006 Egress becomes **ready to author**: its plan may now define integration of this extraction boundary with Eggup multi-artifact transactions, while preserving Egress release/version/origin/candidate/CLI policy. No Egress migration is performed here.
- Eggpack Interoperability M002 archive handoff becomes **ready to author**: its plan may now connect `ManifestProjection::Archive` evidence to `eggup-archive` and later `ArtifactSet` construction. No Eggpack adapter integration is performed here.

Service Lifecycle M007 was already ready and remains ready for the next requested plan. No newly unblocked work is a prerequisite for that implementation.

## Registry updates

- Archive Extraction M001: ready → closed.
- Consumer Adoption M006 Egress: blocked → ready to author; implementation plan remains unwritten.
- Eggpack Interoperability M002: blocked → ready to author; implementation plan remains unwritten.
- Service Lifecycle M007: remains ready and is the next plan in the requested sequence.
