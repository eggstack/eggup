# Eggpack Manifest Interoperability M003a — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Reviewed repository baseline (pre-implementation): `c721638ab1c08adae379450e08b929419d2786cc`

Implementation commit: `39ff62602e39b14b31f5a8f154905d3343034db7` — feat(eggpack): add caller-bound destination binding (M003a)

Plan-authoring baseline from the source plan: `229b61c920f54b50b7c953b060e54ca2673a201b`. `crates/` is byte-identical between `229b61c` and the pre-implementation head `c721638` (verified via `git diff 229b61c c721638 -- crates/`, empty), so the work-package-1 refresh found no code drift; only `plans/` registration text moved.

External baselines (unchanged, per source plan):

- producer: `eggstack/eggpack@56ed7e747fd39e4d6a32a9f1fe3e09dd44355069`
- consumer: `eggstack/eggsact@f1352101dab748066c788e65e21e1bf303cfe995`
- pinned manifest interface: `eggpack-manifest v0.1.0` at `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`

## Executive finding

M003a closes with an additive, fully qualified destination-authority corrective. Manifest `install` remains producer default identity; the application binds the exact relative deployment destination through two new caller-bound entry points plus one pure default-map helper, while the two pre-existing helpers remain behavior-compatible wrappers over a single implementation. All 14 required test classes plus duplicate/colliding-destination negatives are automated (15 new tests), all prior adapter regressions still pass unchanged, lower crates remain Eggpack-free, and hosted qualification is green on all four lanes (run `36890986000`). No medium-or-higher finding remains. M003 Eggsact consumer adoption is unblocked and returns to ready.

## Public API delta

Additive only; no signature changed, no API removed, no version bump (`eggup-eggpack` stays `publish = false`, unpublished):

- `ManifestProjection::default_destinations(&self) -> Result<HashMap<MemberId, String>, AdapterError>` — pure, no I/O; manifest-default map for installable and archive projections.
- `ManifestProjection::materialize_artifact_set_with_destinations(acquired, destinations, permissions) -> Result<ArtifactSet, AdapterError>` — direct/bundle caller-bound materialization; holds the single materialization implementation.
- `core_plan_for_archive_with_destinations(projection, extraction_root, destination_root, destinations, permissions) -> Result<InstallPlan, AdapterError>` — archive caller-bound plan construction; holds the single archive plan implementation.
- `materialize_artifact_set` — unchanged signature; now builds the default map and delegates.
- `core_plan_for_archive` — unchanged signature; now builds the default map and delegates.

## Requirement-to-evidence matrix

| Requirement (source plan §6/§10/§13) | Evidence | Result |
|---|---|---|
| direct/bundle caller-bound materialization (§6.1) | `materialize_artifact_set_with_destinations`; `direct_caller_destination_differs_from_manifest_install` binds `renamed-executable` while `install` stays `eggsact`; `bundle_caller_destinations_bind_every_member_independently` binds three distinct destinations | passed |
| archive caller-bound plan path (§6.2) | `core_plan_for_archive_with_destinations`; `archive_caller_destinations_preserve_source_and_member_evidence` proves caller destinations with manifest-derived advisory sources (`extraction_root.join(install)`), member identity, size facts, and digests | passed |
| default-destination helper (§6.3) | `default_destinations`; `default_destinations_match_manifest_install_identity` covers installable (direct + 3-member bundle) and archive projections; pure, no I/O | passed |
| compatibility wrappers, one implementation (§6.1/§6.2) | `direct_wrapper_matches_prior_manifest_default_behavior` and `archive_wrapper_matches_prior_manifest_default_behavior` assert equality between wrapper output and explicit-default caller-bound output; old logic moved, not duplicated (single loop per path in `src/lib.rs`) | passed |
| exact destination-map validation (§4, §6) | `missing_destination_key_fails_closed`, `extra_destination_key_fails_closed`, `colliding_destinations_fail_closed` (exact duplicates plus `./shared` vs `shared` normalized collision), `archive_destination_map_failures_fail_closed` (missing/extra/colliding) — all `MapMismatch("destinations")` before any set/plan is returned | passed |
| invalid destinations through core validation (§6.1.7, §10.6) | `invalid_destinations_are_rejected_by_core_validation` (`/absolute`, `../escape-root`, empty, `nested/../escape-root`) and archive absolute/traversal cases — all `Eggup(_)` from `ArtifactMember::new` normalization | passed |
| identity/size/digest unchanged (§4, §10.7/§10.8) | custom-destination members keep manifest `MemberId` and `IntegrityRequirement::Sha256` bytes; `destination_change_preserves_size_gate_and_digest_evidence` proves a wrong-size file still fails with a valid custom destination | passed |
| permission/acquired maps independent (§10.9/§10.10) | `permission_and_acquired_maps_stay_exact_and_independent`: complete destinations do not excuse missing permission/acquired entries; acquired files stay keyed by manifest artifact name, not destination | passed |
| archive evidence preserved (§10.11) | archive test asserts `plan.destination(id) == destination_root.join(custom)`, `member.source() == extraction_root.join(install)`, integrity equals manifest member SHA-256, install root unchanged | passed |
| diagnostics carry no content/secrets (§6.1.8, §10.14) | `destination_diagnostics_carry_no_credential_or_content_material` plus extra-key assertion that the destination value is not echoed; `MapMismatch` carries a static kind only | passed |
| archive boundary preserved (§4, §10) | `archive_projection_rejects_direct_caller_bound_materialization` (`ArchiveExtractionRequired` through the new direct entry point); installable projections rejected by the archive entry point | passed |
| M001a/M002 regressions intact (§7.7) | `tests/interoperability.rs` 28/28 and `tests/archive_handoff.rs` 11/11 green unchanged; `adapter_source_claims_no_producer_or_service_authority` still passes with the new rustdoc (no new authority token) | passed |
| lower crates Eggpack-free (§4) | `cargo tree -p eggup-core/-acquisition/-service/-archive` contain zero `eggpack` edges; `cargo tree -p eggup-eggpack` shows only the pinned `eggpack-manifest` | passed |
| README/rustdoc (§12) | crate README gains the default-vs-caller-bound destination section; rustdoc on all three new public APIs documents the authority split and fail-closed map contract | passed |
| M003 handoff text (§5, §7.10) | M003 plan status returns to ready with the M003a path named as the required materialization seam; roadmap + registry M003 rows unblocked (see Roadmap disposition) | passed |

## Production implementation evidence

`crates/eggup-eggpack/src/lib.rs` (+145/−10):

- `default_destinations` method after the private `installable()` gate; installable arms map `member_id -> destination`, archive arms map `MemberId::new(install) -> install`.
- `materialize_artifact_set_with_destinations` validates installable form, then exact acquired/destinations/permissions maps, then per-member absolute/regular/exact-size acquired-file checks (unchanged), constructing each `ArtifactMember` with the caller destination; normalized-duplicate destinations are rejected via a `HashSet` over `member.destination()` before `ArtifactSet::new`.
- `materialize_artifact_set` delegates through `default_destinations`.
- `core_plan_for_archive_with_destinations` validates archive form, exact destinations/permissions maps, builds advisory `extraction_root.join(manifest install)` sources with caller destinations, rejects normalized-duplicate destinations, then `ArtifactSet` + `InstallPlan`.
- `core_plan_for_archive` delegates through `default_destinations`.

Test-only addition: `crates/eggup-eggpack/tests/caller_destinations.rs` (15 tests, see matrix). No `Cargo.toml` change: no new dependency, no version bump, pin unchanged.

## Exact verification commands and results

Local environment: Darwin arm64; stable toolchain; implementation head `39ff626`.

Passed:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
  lib unit tests:              7 passed
  tests/archive_handoff.rs:   11 passed
  tests/caller_destinations.rs: 15 passed (new)
  tests/interoperability.rs:  28 passed
cargo test -p eggup-core --all-targets --all-features --locked      (47 lib + suites green per-package)
cargo test -p eggup-archive --all-targets --all-features --locked   (green per-package)
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-eggpack --all-targets --locked           (7 + 11 + 15 + 28 passed)
cargo tree -p eggup-eggpack --locked    (only eggpack-manifest at pinned rev)
cargo tree -p eggup-core --locked       (sha2 only, zero eggpack edges)
cargo tree -p eggup-acquisition --locked (zero eggpack edges)
cargo tree -p eggup-service --locked    (zero eggpack edges)
cargo tree -p eggup-archive --locked    (zero eggpack edges)
./scripts/check-local.sh                 (fmt/clippy/eggpack/core/archive/doc lanes clean)
git diff --check
```

Hosted run [`36890986000`](https://github.com/eggstack/eggup/actions/runs/36890986000) on implementation head `39ff626` (push, `main`): all four lanes success.

```text
Stable checks (fmt + clippy + workspace tests + doc): success
MSRV check (1.89.0 cargo check --workspace --all-targets --locked): success
macOS tests (cargo test --workspace --all-targets --all-features --locked): success
Windows (archive/acquisition/curl + eggup-eggpack 7+11+15+28 + service lib + workspace check): success
```

The hosted full-workspace runs (Linux stable, macOS) are green including the new `caller_destinations` suite; Windows runs the full adapter suite (7 + 11 + 15 + 28) green, so the new destination tests hold on Windows path semantics as well.

## Local sandbox timing note (informational, not a finding against this plan)

Two timing-sensitive suites flake under full-workspace parallel load in this sandbox and fail identically on the pristine pre-implementation head, so they are unrelated to this change:

- `eggup-core` bounded-runner tests (`bounded_runner_kills_timeouts_and_limits_output`, occasionally `bounded_candidates_are_exact_and_environment_is_cleared`, `cross_member_identity_requires_bundle_agreement`) — pass per-package, fail under workspace-wide load.
- `eggup-curl` `build_curl_args_passes_sub_second_deadlines_to_fake_curl` — fails on pristine HEAD as well.

Hosted Linux/macOS full-workspace runs are green, which is the qualifying evidence. No action taken; recorded here so a future reader does not mistake sandbox load flakes for an M003a regression.

## Invariant review

- Eggpack remains authoritative for artifact identity, relationships, exact size, and SHA-256: projection code untouched; caller destinations never flow into `artifact_name`, `exact_size`, `sha256`, `member_id`, or acquisition binding.
- Application owns installation root and exact relative destination: new maps are caller-supplied; `InstallPlan` root selection unchanged and still caller-owned.
- Manifest `install` remains default identity only: wrappers preserve it; nothing in the adapter treats it as replacement authorization.
- Ownership verification stays downstream in `eggup-core`: no verifier, elevation, or inference added; new rustdoc states the helper never proves control of a live destination.
- Dependency boundary: only `eggup-eggpack` touches `eggpack-manifest`; no new Eggpack edge in `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, or `eggup-service` (tree evidence above); no producer build/CI crate anywhere.
- No release/origin/fallback/authenticity/service policy entered the adapter: the authority-token regression test passes unchanged with the new code; manual review confirms maps-only additions.
- Rust 1.89 holds (MSRV lane green); `#![forbid(unsafe_code)]` untouched, no unsafe added.

## Failure/rollback/recovery review

Projection and destination-map construction remain pure. Materialization stays metadata-read-only until downstream Eggup staging; a destination-map failure returns `Err` before any `ArtifactSet` is built and performs no destination mutation. Archive plan construction stays pure with no extraction or commit. Incomplete maps (missing key, wrong key with equal length) and extra keys fail before construction; colliding destinations fail after member validation but before `ArtifactSet::new`, so no colliding set can reach `InstallPlan`. Stop conditions in source plan §14 did not trigger: no ReleaseManifest change was needed, `eggup-core` represents caller destinations without modification, and member identity separates cleanly from destination.

## Compatibility and migration review

Additive before 1.0 per plan §9. Both existing helpers keep their signatures and manifest-default semantics, proven byte-for-value by the wrapper-equivalence tests (`ArtifactSet`/`InstallPlan` equality). Existing consumers that accept manifest defaults need no migration. M003 Eggsact must use the caller-bound variants and bind the selected member to the exact basename of `env::current_exe()` under its existing installation root, preserving canonical and renamed update-in-place behavior. No registry publication results from M003a.

## Security review

Destination maps are exact: length plus per-member key presence checked for all three maps; normalized collisions rejected. Destination strings pass through existing Eggup `normalize_relative_path` validation (absolute, traversal, empty, control-character, and non-file forms rejected as bounded `Eggup` errors). Exact acquired-file maps, exact-size checks, SHA-256 propagation, symlink rejection, and permission binding are byte-identical to the prior implementation (shared code path). Archive bound-source semantics untouched. Diagnostics name only artifact/member identities and static map kinds; tests assert no URL, credential, destination-value, or file-content leakage.

## Documentation/operations evidence

- `crates/eggup-eggpack/README.md`: default-vs-caller-bound destination section added.
- Rustdoc on `default_destinations`, `materialize_artifact_set_with_destinations`, `core_plan_for_archive_with_destinations`, plus delegation notes on both wrappers; `cargo doc` clean.
- No `Cargo.toml`/`Cargo.lock` delta: no dependency, version, or pin change. Package impact is nil (`publish = false` unchanged).
- This closure record; source plan status flipped to implemented/closed (see below).
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` and `plans/registry.md` updated: M003a closed, M003 ready (see below).
- M003 implementation plan dependency text updated from blocked-on-M003a to ready (see below).
- Historical M001/M001a/M002 closures untouched.

## Unresolved findings

None at medium or higher. One informational note (sandbox timing flakes, pre-existing, hosted-green) is recorded above and requires no corrective.

## Roadmap disposition

M003a is closed. Its closure is the sole hard prerequisite recorded for M003, so M003 Eggsact real-consumer manifest adoption moves from blocked-only-on-M003a back to ready: resume the already-registered plan against then-current Eggup/Eggsact baselines, confirm the `release/eggpack/distribution.toml` plus `release-manifest.json` convention is unchanged, and use the qualified caller-bound materialization seam. M003a does not close M003 itself. M004 package/API promotion remains blocked until M003 real adoption closes and a publishable `eggpack-manifest` version exists.

## Registry updates

- M003a row: ready → closed (implementation `39ff626`, hosted run `36890986000` green on all lanes).
- M003 row: blocked only on M003a → ready (dependency satisfied; consumer production edits may resume).
- Execution graph: M003a → M003 edge now traversable; M004 still blocked on M003.
