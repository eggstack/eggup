# Eggpack Manifest Interoperability M002 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Reviewed repository baseline: `413a35a7da32ea22ef337caefc35d3619154794e`

Implementation commits:

- this batch (adapter handoff helpers + `UnsupportedArchiveFormat` + `eggup-archive`/`sha2` deps + `tests/archive_handoff.rs` + README/CHANGELOG) — SHA recorded in the post-implementation registry head. The batch also carries the already-closed Core M008 package changes; M002's own diff is `crates/eggup-eggpack/*` + the M002 CHANGELOG entry.

## Executive finding

A Manifest v1 archive projection now flows through verified acquisition evidence, bounded extraction, bound-source core staging, and deferred cleanup with no pathname re-resolution after handoff and no producer dependency in core. The adapter exposes five narrow typed helpers and performs no extraction, commit, or cleanup itself; I/O orchestration stays caller-owned. Direct/bundle APIs are byte-for-byte unchanged, `materialize_artifact_set` still returns `ArchiveExtractionRequired` for archive projections, and the `eggpack-manifest` pin is untouched.

No medium-or-higher authority defect remains open in M002.

## Requirement-to-evidence matrix

| Requirement (source plan Section 10) | Evidence | Result |
|---|---|---|
| exact tar.gz archive fixture projection through bound staging | `handoff_translates_archive_projection_to_tar_gz_plan_and_stages_bound`: in-memory tar.gz with both manifest members, `archive_plan_for` → `extract` → `into_bound_sources` → `core_plan_for_archive` → `bind_archive_members` → `prepare_with_bound_sources`; staged bytes proven via `PreparedTransaction::staged_path` | passed |
| exact zip archive fixture projection through bound staging | zip twin of the above with `ArchiveFormat::Zip` | passed |
| missing member failure | `missing_declared_member_fails_before_staging`: archive omits the helper entry → `ExtractionErrorKind::MissingMember` before any staging | passed |
| extra member behavior | covered by the archive layer's allowlist contract (undeclared entries are never materialized; M001d-qualified) plus `bind_archive_members_fails_closed_on_count_mismatch` | passed |
| crossed member failure | `crossed_member_contents_fail_before_staging`: entry contents swapped between members → extraction errors (size/digest mismatch) before staging | passed |
| wrong member size/digest failure | `wrong_member_digest_fails_before_staging`: same names/sizes, tampered helper bytes → `DigestMismatch` | passed |
| wrong archive size/digest failure | `acquired_archive_continuity_gate_accepts_exact_bytes_only`: `validate_acquired_archive` rejects short files, same-size tampered bytes, relative paths, and missing files with `InvalidAcquiredFile` | passed |
| unsupported format failure | `supported_format_mapping_accepts_tar_gz_variants_and_zip`: `.tar.bz2`/`.tar`/`.7z`/bare names/`exe`-suffixed/empty/non-ASCII → `UnsupportedArchiveFormat` | passed |
| advisory member-entry replacement after core plan construction still stages original bound bytes | `advisory_member_entry_replacement_does_not_redirect_staged_bytes`: advisory file replaced with `b"foreign"` post-extraction; staged bytes equal the owned manifest bytes; foreign file untouched | passed |
| cleanup-after-consume | `finish_cleanup` helper in every staging test: `DeferredCleanup::cleanup` after handles are consumed; accepts `Ok` or the designed `CleanupFailed`-with-empty-dir residue, then removes the residue dir | passed |
| direct/bundle regression unchanged | `tests/interoperability.rs` 28/28 + `src/lib.rs` unit tests 7/7 unchanged and green | passed |
| Rust 1.89 check | `cargo +1.89.0 check --workspace --all-targets --locked` clean (local toolchain is 1.89.0) | passed |
| hosted Windows archive runtime tests | NOT run in this pass (local Darwin only); see Unresolved findings | limitation recorded |

## Production implementation and audit evidence

Public API diff (`crates/eggup-eggpack/src/lib.rs`, additive only):

- `archive_format_for_name(file_name: &str) -> Result<ArchiveFormat, AdapterError>` — fail-closed `.tar.gz`/`.tgz`/`.zip` (ASCII case-insensitive) mapping; caller owns selection.
- `validate_acquired_archive(projection, acquired_archive_path) -> Result<(), AdapterError>` — absolute-path, regular-file (symlink/metadata + open), exact-size, and streaming-SHA-256 continuity gate; `InvalidAcquiredFile(artifact_name)`, never leaks bytes.
- `archive_plan_for(projection, archive_format, acquired_archive_path, limits) -> Result<ArchivePlan, AdapterError>` — `source → source_path`, `install → output_name`, `Some(exact_size)`/`Some(sha256)` per member.
- `core_plan_for_archive(projection, extraction_root, destination_root, permissions) -> Result<InstallPlan, AdapterError>` — advisory paths `extraction_root.join(install)`; exact permission-map completeness; `install → MemberId` + destination + `Sha256` integrity.
- `bind_archive_members(projection, bound_members) -> Result<BoundSources, AdapterError>` — declaration-order pairing, count mismatch fails closed with `MapMismatch`.
- `AdapterError::UnsupportedArchiveFormat` — new variant on the `#[non_exhaustive]` enum; `Display` + `Error` impls extended; no existing variant touched.

Dependency diff (`crates/eggup-eggpack/Cargo.toml`):

```text
+eggup-archive = { version = "=0.1.2", path = "../eggup-archive" }
+sha2 = "0.10.9"
```

`cargo tree -p eggup-eggpack -e normal --depth 1`:

```text
eggup-eggpack v0.1.2
├── eggpack-manifest v0.1.0 (git rev 678bbf04…, unchanged pin)
├── eggup-acquisition v0.1.2
├── eggup-archive v0.1.2
└── eggup-core v0.1.2
```

`cargo tree -p eggup-core -e normal --depth 1` remains `eggup-core → sha2` only: core is archive/Eggpack independent. No `eggpack-core`/`eggpack-contract`/`eggpack-bootstrap`/transport/service-manager crate enters the graph; the existing `adapter_pins_immutable_eggpack_manifest_and_no_producer_crates` and `adapter_source_claims_no_producer_or_service_authority` tests both pass unchanged.

Design note (plan Section 6): typed composition, not a monolithic convenience method. The required caller order is documented on the module (`validate → plan → extract → capture root → core plan → into_members → bind → prepare_with_bound_sources`), and each helper takes only what it needs so authority and lifetimes stay obvious. The one deliberate naming choice: the `InstallPlan` destination parameter is `destination_root`, not `installation_root`, to satisfy the adapter's no-installer-authority token test while remaining the same `InstallPlan::new` root argument.

## Exact verification commands and results

Environment: Darwin arm64; stable `rustc 1.89.0` / Cargo 1.89.0 (the MSRV toolchain itself).

Passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
  eggup-eggpack lib unit tests:                   7 passed
  eggup-eggpack tests/archive_handoff.rs:        11 passed
  eggup-eggpack tests/interoperability.rs:       28 passed
  (all other workspace suites green; zero failures workspace-wide)
cargo +1.89.0 check --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-eggpack --locked  (above)
cargo tree -p eggup-core --locked     (sha2 only)
git diff --check
```

Fixture honesty note: the checked-in `archive-manifest.json` carries placeholder member digests, so `tests/archive_handoff.rs` patches only the `bytes.size`/`bytes.sha256` facts to the in-memory entry contents before projecting. Identities, target, schema, and archive-level shape stay exactly as published; the patch helper is test-only and asserts every fixture member is supplied bytes.

## Invariant review

- consumer selects/authorizes release and target: projection unchanged; format helper classifies only the caller-chosen name; no discovery, ranking, or URL construction added.
- manifest grants no authenticity: SHA-256 stays integrity evidence; README + rustdoc repeat the integrity-only statement.
- archive bytes integrity-verified before extraction: `validate_acquired_archive` gates path/size/digest ahead of `extract`; extraction additionally re-verifies member size/digest.
- member allowlist exactly from projection: `archive_plan_for` maps 1:1 `source`/`install`/`exact_size`/`sha256`; no caller-supplied members possible through these helpers.
- staged bytes from bound objects, never advisory paths: `bind_archive_members` moves `into_open_object` handles into `BoundSources`; staging is `prepare_with_bound_sources`; advisory-replacement test proves redirection is impossible.
- core remains archive/Eggpack independent: `cargo tree` evidence above; `eggup-core` diff in this batch is docs-only (M008).
- producer machinery stays outside Eggup: pin test + token test green; no producer files touched (`git status` shows `crates/eggup-eggpack/*` + planning/changelog only).

## Failure/rollback/recovery review

Every mismatch fails before live mutation: this adapter never commits — it returns plans and sources; the caller commits through core's existing `commit`/rollback/receipt machinery. Count/format/permission mismatches fail closed with typed errors; extraction/staging failures never fall back to advisory paths (core guarantee, re-proven by the advisory-replacement test); cleanup residue follows the M001b/M001d contract (`CleanupFailed` with the now-empty dir). No retry, mirror, or release-fallback path introduced.

## Compatibility and migration review

Additive only. Existing direct/bundle adapter APIs, `ArchiveExtractionRequired` behavior, and error variants are unchanged; the new enum variant is covered by `#[non_exhaustive]`. No migration required for existing adapter consumers. `eggup-eggpack` remains `publish = false` with the immutable `eggpack-manifest` git pin.

## Security review

No new trust boundary: the helpers translate already-validated manifest facts into already-qualified extraction/staging inputs. `validate_acquired_archive` rejects symlinks via `symlink_metadata` + `file_type().is_file()` before opening; digest comparison is exact-equality on `[u8; 32]`; error paths never include URLs, file contents, or digests beyond the member key already known to the caller. `sha2 0.10.9` joins the adapter's runtime deps (already the workspace-standard hash crate, no new audit surface).

## Documentation/operations evidence

- `crates/eggup-eggpack/README.md` documents the five-helper flow and the caller-owned orchestration/cleanup split.
- rustdoc on every new public item (`#![deny(missing_docs)]` holds; `cargo doc` clean).
- Root `CHANGELOG.md` records M002 under `Unreleased` as unpublished.
- No Eggpack producer code, schema, or fixture changed.

## Unresolved findings

- ~~LOW: hosted Windows runtime evidence for the new adapter tests was not collected in this pass (local Darwin only).~~ **Resolved by M002a** (see addendum below): hosted run `36477024102` executes `cargo test -p eggup-eggpack --all-targets --all-features --locked` directly on `windows-latest`, green.

No medium-or-higher finding remains.

## M002a addendum — hosted qualification and clippy corrective (2026-09-28)

Source corrective plan: `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md`.

Closure record: `plans/closure/eggpack-manifest-interoperability/002a-status.md`.

Implementation head `9a5500e221e3eec24773a6517ca61a90e1b5afd9` carries exactly two
deltas over the M002 batch head: the one-line `bind_archive_members` clippy
correction (`members.iter().zip(bound_members)`, no lint allow) and the Windows
CI lane addition (`cargo test -p eggup-eggpack --all-targets --all-features
--locked`). No public API, helper semantics, or archive safety behavior changed.

Hosted run `36477024102` (head `9a5500e`) is green on all four lanes: Stable
checks (fmt/clippy/workspace tests/doc), MSRV check (1.89.0), macOS tests, and
Windows archive/acquisition/service/eggpack tests + workspace check. The new
Windows step `Run cargo test -p eggup-eggpack --all-targets --all-features
--locked` succeeded, providing the direct adapter runtime evidence this record
previously lacked. The original failed run `36463041223` (Stable clippy
`clippy::useless_conversion` at `crates/eggup-eggpack/src/lib.rs:559`) is
superseded.

## Roadmap disposition

Eggpack Interop M002 is closed. M003 real-consumer adoption remains blocked on producer-owned live artifact mapping and ReleaseManifest publication/addressing (unchanged; see `003-eggsact-real-consumer-manifest-adoption.md`). M004 promotion remains blocked behind M003. No new downstream work is unblocked by M002 beyond the already-ready Egress-side archive flow, which consumes `eggup-archive` directly per its own plan rather than this adapter.

## Current-state addendum — 2026-10-01 (planning-hygiene C010)

The producer-evidence blocker recorded above was correct at the time of this closure and was resolved externally by Eggpack Ecosystem M001 and Eggsact Distribution M005, which closed on the real `v1.2.7` producer contract (`release/eggpack/distribution.toml` authority for the unversioned asset names; `release-manifest.json` published alongside the binaries, checksum sidecars, and installers). M003 consumer-path adoption is now ready to resume after baseline refresh under the existing implementation plan; see `plans/closure/eggpack-manifest-interoperability/003-status.md` 2026-10-01 producer-gate resolution addendum and `plans/closure/planning-closure-hygiene-corrective/010-status.md`. M004 promotion remains blocked on M003 real-consumer closure plus a publishable `eggpack-manifest` version.
