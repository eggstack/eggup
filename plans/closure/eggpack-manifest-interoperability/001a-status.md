# Eggpack Manifest Interoperability M001a Closure

Status: closed

Corrective implementation commit: `19935ec3610a5238af33a9d4f05a14925ceac25c`

Corrective plan: `plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md`

Historical M001 implementation: `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef`

Historical M001 closure: `plans/closure/eggpack-manifest-interoperability/001-status.md`

Service packageability repair: `84076a066e498f2e4ec4ed5472af4359c0b93af7`

External Eggpack baseline: `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41` (unchanged; Eggpack M001a remains closed there)

## Production-code delta

None. All 28 new interoperability regressions passed against the existing
adapter on the first full run (after two test-harness-only fixes: a missing
dev-only `serde` derive dependency and a `HashMap` key-removal helper).
Per the corrective plan's production-code policy, the adapter was not
refactored for style.

Test-only changes at the implementation commit:

- new `crates/eggup-eggpack/tests/interoperability.rs` (28 tests);
- `crates/eggup-eggpack/Cargo.toml` gains a dev-only `serde` derive
  dependency for the strict test-only projection structs;
- `Cargo.lock` records the resulting dev-dependency edge (serde was already
  in the lockfile via other crates).

## Requirement-to-evidence matrix

| Corrective requirement | Evidence |
|---|---|
| Actual copied direct/bundle/archive fixtures checked against adapter output | `direct/bundle/archive_positive_*` tests load `tests/fixtures/*.json` via `include_str!` and compare every artifact/install field against strict test-only `DirectProjection`/`BundleProjection`/`ArchiveProjection` structs (`deny_unknown_fields`) |
| Direct positive matrix | `direct_positive_matches_projection_fixture`: exact Linux target, product/release identities, one artifact with exact name/size/digest/member/destination, `one-artifact-set` group |
| Caller URL byte fidelity + limit tightening | `direct_request_binding_preserves_url_and_tightens_limits`: credential-bearing URL retained byte-for-byte; `max_artifact_bytes` tightened to exact size; connect/total timeouts and metadata bound preserved |
| Direct materialization + digest propagation | `direct_materialization_yields_single_member_with_manifest_digest`: one `ArtifactSet` member, integrity equals exact manifest SHA-256 bytes, explicit `Executable` intent preserved |
| Bundle positive matrix (corrected three-member CodeGG) | `bundle_positive_matches_corrected_three_member_fixture`: exactly three requirements, every relation paired both directions, deterministic order equal to fixture order, distinct member identities, no `eggsact` entry; `bundle_exact_maps_produce_three_member_set`: exact three-request binding and three-member `ArtifactSet` with per-member digests |
| Archive projection/request/boundary | `archive_positive_preserves_facts_and_blocks_materialization`: artifact name/size/digest and member source/install/size/digest match fixture and manifest; single-request binding tightens the limit; generic `materialize_artifact_set` returns `ArchiveExtractionRequired`; fixture `extraction_required = true` retained |
| Negative cases 1-3 (targets, schema) | `negative_01` alias `linux-x64` -> `TargetNotFound`; `negative_02` unknown canonical target from checked-in `wrong-target.json` plus macOS-against-bundle -> `TargetNotFound`; `negative_03` `unknown-schema.json` rejected by `from_json` and by `project` as `InvalidManifest` |
| Negative cases 4-7 (request maps, caller limits) | `negative_04` missing request -> map mismatch; `negative_05` extra request -> map mismatch with no URL text in diagnostics; `negative_06` caller cap below exact size -> `CallerLimitTooSmall`; `negative_07` timeouts/metadata never widened, unbounded caller cap tightened to exact size |
| Negative cases 8-11 (acquired/permission maps) | `negative_08` missing bundle artifact; `negative_09` extra acquired file; `negative_10` missing permission; `negative_11` extra permission — all map mismatch |
| Negative cases 12-16 (filesystem rejection) | `negative_12` relative path; `negative_13` symlink (unix symlink exercised; Windows path isolates creation behind `symlink_file` with a privilege-skip notice); `negative_14` directory; `negative_15` missing file; `negative_16` wrong exact size — all `InvalidAcquiredFile` |
| Negative case 17 (archive boundary) | `negative_17_archive_materialization_requires_extraction` |
| Negative cases 18-20 (bundle fidelity) | `negative_18` cardinality is three; `negative_19` substituted `eggsact` member fails comparison; `negative_20` crossed destination/member relationship fails comparison |
| Lower crates remain Eggpack-independent | `cargo tree -p eggup-core`, `-p eggup-acquisition`, `-p eggup-service` contain zero `eggpack` edges (recorded below) |
| Only adapter depends on Eggpack, pinned revision | `cargo tree -p eggup-eggpack` shows only `eggpack-manifest` at rev `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`; no `eggpack-core`/`eggpack-contract`/`eggpack-bootstrap`; automated pin/producer-crate test included |
| Authority review | `adapter_source_claims_no_producer_or_service_authority` scans production source for 23 producer/service authority tokens (supplement to manual review); manual review confirms no release discovery, origin construction, extraction, install-root, ownership, service, or authenticity policy |
| Historical service-scope reconciliation | Section below; `cargo package -p eggup-service --locked --no-verify --allow-dirty` succeeds at this SHA (`Packaged 8 files, 271.3KiB`); `cargo tree -p eggup-service` shows no Eggpack edge |
| No unresolved medium-or-higher finding | None remains (see unresolved-findings section) |

## Verification evidence

Passed on a clean worktree at the implementation commit
`19935ec3610a5238af33a9d4f05a14925ceac25c`
(stable toolchain `1.98.1`; workspace MSRV remains `1.89`):

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test -p eggup-eggpack --all-targets --all-features --locked` — 4 unit + 28 integration passed
- `cargo test -p eggup-core --all-targets --all-features --locked` — passed
- `cargo test -p eggup-acquisition --all-targets --all-features --locked` — passed
- `cargo test --workspace --all-targets --all-features --locked` — all suites passed, 0 failed
- `cargo doc --workspace --no-deps --locked`
- `cargo tree -p eggup-eggpack --locked` (only `eggpack-manifest`; dev-only `serde`/`serde_json` under `[dev-dependencies]`)
- `cargo tree -p eggup-core --locked` — no `eggpack` edge
- `cargo tree -p eggup-acquisition --locked` — no `eggpack` edge
- `cargo tree -p eggup-service --locked` — `eggup-core` + `windows-args` only, no `eggpack` edge
- `cargo package -p eggup-service --locked --no-verify --allow-dirty` — `Packaged 8 files, 271.3KiB (51.6KiB compressed)`
- `cargo +1.89.0 check --workspace --all-targets --locked`
- `cargo +1.89.0 test -p eggup-eggpack --all-targets --locked` — 4 + 28 passed
- `./scripts/check-local.sh` — exit 0
- `git diff --check`

`cargo package -p eggup-eggpack` remains non-qualifying while
`eggpack-manifest` is unpublished and the adapter is `publish = false`; that
known packaging constraint is not a corrective failure.

Hosted CI run [36014508645](https://github.com/eggstack/eggup/actions/runs/36014508645) passed all four lanes on closure HEAD `56b9740`: stable fmt/clippy/workspace tests/docs, Rust 1.89 check, macOS workspace tests, and Windows workspace check.

## Copied fixture provenance

Fixtures under `crates/eggup-eggpack/tests/fixtures/` are unchanged from M001
and match `tests/fixtures/README.md` (upstream Eggpack `678bbf04...`):

| File | SHA-256 |
|---|---|
| archive-manifest.json | `723f1ed30b6bc7dae802ff54ac57afd6c8c8ea6be0e5bcc05c30ef56d4f647d1` |
| bundle-manifest.json | `440e871d6cd481bbf1a19ae0711ad9f438b3144315e939a91e66ee3b63b828cc` |
| direct-manifest.json | `04b1993f8ed1de1a4f19819f643831d10ecf87e32c783ab83259fd6cd55947f2` |
| projection-archive.json | `c8a2a6e33bc4336cd65cd9b0c4d94c581ba574967f111663b2e70498a7cc8e46` |
| projection-bundle.json | `f0b227442e2a2a28dc4e2100301233f2698703d15251b6f2459e6387df6c53ce` |
| projection-direct.json | `63cb46479a78479929c9bcd2040268435095ee2cc49506bdb1f2c93a04be3760` |
| unknown-schema.json | `3527fa83397409981974e498087f2e8e932215ed0115c22793b891d5d917160e` |
| wrong-target.json | `cd5dd514216a45aea4a2a83b98208bb15c159b8b05bc61441647eae4e39f18f2` |

## Historical service-scope reconciliation

- M001 implementation `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef` added a
  path-only `eggup-service -> eggup-core` dependency (`eggup-core = { path =
  "../eggup-core" }` with no `version`), which belonged to the separately
  planned Service M005 workstream and made normal service package
  construction invalid until repaired.
- Service M005 commit `84076a066e498f2e4ec4ed5472af4359c0b93af7` added the
  required `version = "0.1.0"`.
- Service M005 closure (`plans/closure/service-lifecycle/005-status.md`)
  qualified package construction and hosted CI.
- Current main (`crates/eggup-service/Cargo.toml`) declares
  `eggup-core = { version = "0.1.0", path = "../eggup-core" }`; the
  re-run `cargo package -p eggup-service --locked --no-verify --allow-dirty`
  above succeeds. No open service packageability defect is attributable to
  the historical M001 edit, and this corrective makes no service production
  change.

## Invariant, recovery, and compatibility review

Adapter projection/request binding remain pure; materialization remains
metadata-read-only and all-or-nothing. Tests clean all temporary fixture
files/directories via an RAII `TempDir` guard. No network, service mutation,
release publication, or live installation is performed. SHA-256 remains
integrity evidence, not authenticity. No public API or wire-format change;
the immutable Eggpack pin is unchanged. M001 historical closure remains
intact as the record of what was verified at the time; this record
supplements it.

## Security review

Adapter errors name only manifest artifact/member identities; the extra-
request diagnostic test asserts no URL text leaks. Caller byte limits are
only ever tightened. Filesystem inputs must be absolute regular non-symlink
files of exact expected size. No new transport, privilege, ownership, or
authenticity authority enters the adapter.

## Docs/operations evidence

- `crates/eggup-eggpack/README.md` and crate rustdoc already document the
  optional adapter, archive boundary, caller-owned policy, and
  integrity-not-authenticity semantics; unchanged by this corrective.
- Historical M001 closure annotated with an M001a closure pointer (this file).
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` and
  `plans/registry.md` updated: M001a closed, M003 real-consumer adoption
  returned to ready for plan authoring.
- Reciprocal Eggpack state: no change required. Eggpack interoperability
  M001a remains closed at `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41`;
  no Eggpack production code was touched.

## Unresolved findings

None. No medium-or-higher adapter correctness, security, or qualification
finding remains. Platform note (informational only): the symlink rejection
case runs fully on unix; on Windows hosts where symlink creation is
privilege-constrained, that single case prints a skip notice instead of
failing, while all other filesystem rejection cases still execute there.

## Disposition

- M001a is closed with the full regression matrix automated and the
  historical service packageability interaction truthfully recorded.
- M002 remains blocked on the long-term Phase 10 safe extraction contract.
- M003 real-consumer adoption returns to `ready for plan authoring`, which
  must select a real consumer currently owning duplicated
  manifest-to-update mapping.
- M004 remains planned pending real consumer adoption and a publishable
  upstream `eggpack-manifest` version.
