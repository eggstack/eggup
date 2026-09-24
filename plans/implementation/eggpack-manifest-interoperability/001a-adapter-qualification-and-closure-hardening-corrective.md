# Eggpack Manifest Interoperability Milestone 001a — Adapter Qualification and Closure-Hardening Corrective

Status: ready for handoff

Repository baseline: `3809241d637b2ee1f7f5e36170db10043f19a4a8`

Historical M001 implementation: `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef`

Historical M001 closure: `plans/closure/eggpack-manifest-interoperability/001-status.md`

Current service packageability repair: `84076a066e498f2e4ec4ed5472af4359c0b93af7`

External Eggpack baseline: `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41`

Source roadmap:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Applicable ADRs:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Primary class: corrective / qualification / closure evidence

## 1. Objective

Harden the closed `eggup-eggpack` M001 adapter with the negative and cross-repository compatibility matrix required by its original implementation plan, and reconcile closure history for the unrelated `eggup-service -> eggup-core` manifest edit that landed in the M001 implementation commit before Service M005 later repaired packageability.

This corrective is evidence-first. Do not redesign the adapter or broaden its authority. Production changes are allowed only when a new regression test demonstrates an actual correctness defect.

M003 real-consumer adoption is re-blocked until M001a closes.

## 2. Why this corrective is required

Post-closure review found two medium-severity qualification/closure issues.

### A. Adapter regression matrix is materially incomplete

The historical closure reports four `eggup-eggpack` tests. The M001 plan required substantially broader evidence, including:

- exact direct/bundle relationship fidelity;
- missing/extra acquisition requests;
- caller byte limit behavior;
- missing/extra acquired files;
- missing/extra permissions;
- relative/symlink/non-regular paths;
- exact-size mismatch;
- archive materialization refusal;
- unknown schema behavior;
- corrected CodeGG bundle regression protection;
- dependency-direction/source scans.

The production implementation appears to enforce most of these rules, but closure cannot claim them as qualified until the checked-in tests exercise them.

### B. Historical M001 implementation contained unrelated service manifest scope

Commit `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef` added a path-only `eggup-service -> eggup-core` dependency while implementing the adapter. That edit belonged to the separately planned Service M005 workstream and temporarily made normal service package construction invalid because the path dependency had no version.

Service M005 later repaired the dependency at `84076a066e498f2e4ec4ed5472af4359c0b93af7` and qualified `eggup-service` package construction. Current `main` is healthy, but the M001 closure must record this historical scope/packageability interaction rather than implying the implementation commit was isolated to adapter changes.

## 3. Affected invariants

The corrective must preserve and prove:

- only `eggup-eggpack` depends on `eggpack-manifest`;
- lower Eggup crates remain usable without Eggpack;
- `eggpack-manifest` remains pinned to immutable revision `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`;
- target selection is exact canonical matching;
- direct and bundle relationships preserve exact artifact/install pairing;
- the corrected three-member CodeGG bundle cannot regress to the old two-member/substituted shape;
- request maps, acquired-file maps, and permission maps are exact sets;
- caller acquisition limits are never widened;
- local materialization accepts only absolute regular non-symlink files with exact expected size;
- manifest SHA-256 bytes are propagated unchanged into `IntegrityRequirement::Sha256`;
- caller permissions remain explicit;
- archive form may bind the archive acquisition request but cannot materialize an `ArtifactSet`;
- no release/origin/install-root/ownership/service/authenticity policy enters the adapter;
- current `eggup-service` packageability remains qualified and the corrective does not modify service runtime behavior;
- M003 adoption remains blocked until corrective closure.

## 4. Scope

### In scope

- expand `eggup-eggpack` tests to cover the original M001 positive/negative matrix;
- exercise the actual copied Eggpack fixtures, including projection fixtures;
- add strict test-only comparison of adapter projections against `projection-direct.json`, `projection-bundle.json`, and `projection-archive.json`;
- validate exact request-map, acquired-map, and permissions-map behavior;
- validate filesystem input rejection cases;
- validate unknown schema and exact-target failures;
- validate dependency direction;
- document the historical out-of-scope service manifest edit and its later repair;
- re-run service package construction at current main as closure evidence;
- annotate historical M001 closure;
- write `plans/closure/eggpack-manifest-interoperability/001a-status.md` at closure;
- update roadmap/registry and reciprocal Eggpack planning state.

### Explicitly out of scope

- new adapter capabilities;
- release selection or URL construction;
- archive extraction;
- install-root or ownership policy;
- service lifecycle changes;
- changing `eggpack-manifest` revision;
- publishing `eggup-eggpack`;
- consumer adoption;
- authenticity/signatures;
- refactoring lower Eggup crates without a failing corrective test.

## 5. Required test architecture

Prefer moving the broad compatibility matrix into an integration test such as:

```text
crates/eggup-eggpack/tests/interoperability.rs
```

Keep unit tests for local implementation details only.

Test-only fixture projection structs are appropriate. They are not a production schema and must not be exported from the crate.

The tests MUST load the checked-in fixture files under `crates/eggup-eggpack/tests/fixtures/`, not reconstructed equivalents.

## 6. Required positive matrix

### A. Direct projection

Using `direct-manifest.json` and `projection-direct.json`:

- exact Linux canonical target resolves;
- product/release identities match;
- exactly one artifact requirement exists;
- artifact name, exact size, digest, member id, and relative destination match the projection fixture exactly;
- exact caller URL is retained byte-for-byte;
- baseline connect/total timeout and metadata bound are preserved;
- `max_artifact_bytes` is tightened to exact manifest size;
- an exact-size local file plus explicit permissions produces one `ArtifactSet` member;
- resulting member integrity equals exact manifest SHA-256 bytes.

### B. Bundle projection

Using `bundle-manifest.json` and corrected `projection-bundle.json`:

- exactly three requirements exist;
- every artifact/install relation matches the projection fixture;
- order is deterministic;
- exact request binding succeeds only for all three artifacts;
- exact acquired-file/permission maps produce one three-member `ArtifactSet`;
- CodeGG, helper, and manifest member identities/destinations remain distinct and correct;
- no entry from another product can appear.

### C. Archive projection

Using `archive-manifest.json` and `projection-archive.json`:

- archive artifact name/size/digest match;
- all member source/install/size/digest relationships match;
- archive request binding accepts exactly one archive request and tightens its limit;
- `materialize_artifact_set` returns `ArchiveExtractionRequired`;
- test evidence explicitly retains `extraction_required = true` semantics.

## 7. Required negative matrix

Add independent regressions for at least:

1. alias/non-canonical target -> `TargetNotFound`;
2. unknown canonical target -> `TargetNotFound`;
3. `unknown-schema.json` cannot project successfully;
4. missing acquisition request -> map mismatch;
5. extra acquisition request -> map mismatch;
6. caller `max_artifact_bytes` below exact manifest size -> `CallerLimitTooSmall`;
7. connect/total/metadata limits are not widened;
8. missing acquired bundle artifact -> map mismatch;
9. extra acquired artifact -> map mismatch;
10. missing permission entry -> map mismatch;
11. extra permission entry -> map mismatch;
12. relative acquired path -> invalid acquired file;
13. symlink acquired path -> invalid acquired file;
14. directory/non-regular acquired path -> invalid acquired file;
15. missing local file -> invalid acquired file;
16. wrong exact size -> invalid acquired file;
17. archive materialization -> extraction required;
18. corrected CodeGG bundle cardinality must be three;
19. substituted `eggsact` member in expected bundle relationship -> comparison failure;
20. crossed bundle destination/member relationship -> comparison failure.

On platforms where symlink creation is privilege- or policy-constrained, isolate the case behind an OS-appropriate helper and record any platform limitation. Do not silently omit the behavior from all CI.

## 8. Dependency and authority qualification

Add/retain automated or closure-time evidence that:

```text
cargo tree -p eggup-core
cargo tree -p eggup-acquisition
cargo tree -p eggup-service
```

contain no `eggpack-*` dependency.

`cargo tree -p eggup-eggpack` may contain `eggpack-manifest` only; it must not pull `eggpack-core`, `eggpack-contract`, or `eggpack-bootstrap`.

Perform a source scan of `crates/eggup-eggpack` for prohibited authority such as:

- GitHub release discovery;
- "latest" release lookup;
- base-origin joining/fallback;
- archive extraction;
- service control;
- ownership inference;
- elevation;
- signature/authenticity claims.

A simple literal scan is supplementary to code review; do not treat absence of strings as sole evidence.

## 9. Historical service-scope reconciliation

Do not revert or modify Service M005 behavior merely to clean history.

Instead, closure M001a must record:

- M001 implementation `5fbb6685...` included an out-of-scope path-only `eggup-service -> eggup-core` dependency;
- this made the service manifest unsuitable for normal package construction until repaired;
- Service M005 commit `84076a06...` added the required `version = "0.1.0"`;
- Service M005 closure qualified package construction and hosted CI;
- current main therefore has no open service packageability defect attributable to the historical M001 edit.

Re-run at current corrective implementation SHA:

```bash
cargo package -p eggup-service --locked --no-verify --allow-dirty
cargo tree -p eggup-service --locked
```

No further service production change is expected.

## 10. Production-code policy

Start by adding tests only.

If all required regressions pass against existing production code, do not refactor the adapter for style.

If a test exposes a correctness defect:

1. record the failing case;
2. make the smallest production fix;
3. keep authority boundaries unchanged;
4. add the regression;
5. describe the production delta explicitly in M001a closure.

If fixing a failure requires a new policy authority, archive extraction, lower-layer Eggpack dependency, or manifest wire change, stop and re-plan.

## 11. Ordered work packages

1. Add strict test-only projection fixture parser/comparator.
2. Add direct positive and negative matrix.
3. Add corrected three-member bundle positive and relationship-negative matrix.
4. Add archive projection/request/materialization-boundary matrix.
5. Add filesystem path/type/size rejection matrix.
6. Add dependency-direction and authority review evidence.
7. Run current service packageability qualification and record historical repair relationship.
8. Run full stable/MSRV/docs/workspace/hosted CI.
9. Annotate historical M001 closure and update roadmap/registry.
10. Write M001a closure and only then re-open M003 real-consumer plan authoring.

## 12. Failure, restart, and contention semantics

The corrective adds deterministic tests and documentation. Adapter projection/request binding remains pure, and local materialization remains metadata-read-only.

Tests must clean all temporary fixture files/directories they create.

No network, service mutation, release publication, or live installation is performed.

## 13. Compatibility and migration

No public API or wire-format change is expected.

M001 historical closure remains intact as the record of what was implemented and verified at the time. M001a supplements it with stronger qualification and truthful scope history.

If production code is unchanged, state that explicitly.

The existing immutable Eggpack dependency pin remains unchanged.

## 14. Required verification commands

Run and record:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test -p eggup-core --all-targets --all-features --locked
cargo test -p eggup-acquisition --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-eggpack --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-acquisition --locked
cargo tree -p eggup-service --locked
cargo package -p eggup-service --locked --no-verify --allow-dirty
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-eggpack --all-targets --locked
./scripts/check-local.sh
git diff --check
```

Hosted CI must pass Linux stable, Linux Rust 1.89, macOS, and Windows lanes.

`cargo package -p eggup-eggpack` remains non-qualifying while `eggpack-manifest` is unpublished and the adapter is `publish = false`; do not reinterpret that known packaging constraint as a corrective failure.

## 15. Documentation updates

Update:

- historical `plans/closure/eggpack-manifest-interoperability/001-status.md` with a post-closure corrective annotation;
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`;
- `plans/registry.md`;
- new `plans/closure/eggpack-manifest-interoperability/001a-status.md` at closure;
- reciprocal Eggpack interop roadmap/registry status.

Do not rewrite the historical M001 closure to pretend the original four-test qualification or service manifest scope did not occur.

## 16. Acceptance criteria

M001a closes only when:

- the actual copied direct/bundle/archive fixtures are checked against adapter output;
- the required positive/negative matrix above is automated;
- the corrected three-entry CodeGG bundle is regression-protected;
- exact map semantics and acquisition limit non-widening are qualified;
- filesystem path/type/size rejection is qualified;
- archive materialization remains blocked;
- lower Eggup crates remain Eggpack-independent;
- the historical service path-dependency/packageability interaction is documented and current packageability is re-verified;
- stable/MSRV/docs/hosted CI pass;
- no unresolved medium-or-higher adapter correctness/security/qualification finding remains.

Only then may M003 real-consumer adoption return to `ready for plan authoring`.

## 17. Stop conditions

Stop and prepare a new plan/ADR if:

- a required regression exposes an ambiguity in ReleaseManifest v1;
- direct/bundle mapping requires filename inference;
- lower Eggup crates must depend on Eggpack;
- archive extraction is required to close M001a;
- release/origin/install-root/ownership/authenticity policy must enter the adapter;
- service lifecycle behavior must change to repair the historical scope issue.

## 18. Closure evidence required

Record:

- exact corrective implementation/review SHA;
- production-code delta, including explicit "none" if tests/docs only;
- adapter test count and named matrix;
- copied fixture provenance/hashes;
- direct/bundle/archive comparison evidence;
- request/acquired/permissions exact-map evidence;
- filesystem rejection evidence;
- dependency trees;
- authority review;
- service packageability command/result and historical repair reference;
- stable/Rust1.89/macOS/Windows CI run;
- unresolved findings;
- explicit M003 readiness disposition.

## 19. Handoff notes

This is the sole corrective required before real-consumer adoption.

Do not add a second Eggup adapter plan, do not modify Eggpack production code, and do not broaden M001 into archive extraction or update policy.
