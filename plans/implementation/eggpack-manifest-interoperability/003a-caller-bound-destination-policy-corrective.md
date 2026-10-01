# Eggpack Manifest Interoperability Milestone 003a — Caller-Bound Destination Policy Corrective

Status: implemented; closed by `plans/closure/eggpack-manifest-interoperability/003a-status.md` (implementation `39ff626`, hosted run `36890986000` green on all lanes)

Repository baseline: `229b61c920f54b50b7c953b060e54ca2673a201b`

Source milestone:

- `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`

Source roadmap:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Long-term references:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md#8-install-plan`
- `plans/001-terminology-and-domain-model.md#17-destination`
- `plans/002-long-term-roadmap.md#phase-9--eggpack-authority-cutover-and-manifest-interoperability`

Applicable ADRs:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

External interface evidence:

- `eggstack/eggpack: architecture/eggup-manifest-consumer-v1.md` states that ReleaseManifest does not select destinations or replacement authorization and that `install` is a flat target-local identity/default relative basename.
- Current reviewed producer head: `eggstack/eggpack@56ed7e747fd39e4d6a32a9f1fe3e09dd44355069`.
- Current reviewed Eggsact head: `eggstack/eggsact@f1352101dab748066c788e65e21e1bf303cfe995`.
- Eggsact's normal self-update still replaces `env::current_exe()`; M005a changed deterministic Windows linker behavior only and did not change updater destination semantics.

Primary class: invariant / compatibility corrective

## 1. Objective

Correct the optional `eggup-eggpack` adapter so manifest-provided install identity remains producer evidence while the consuming application can bind the exact relative deployment destination authorized by its installation policy.

The current direct/bundle materialization API uses the ReleaseManifest `install` value directly as the `ArtifactMember` destination. The archive handoff similarly uses each manifest member `install` value directly when constructing the core `InstallPlan`.

That is safe for canonical installs whose executable basename matches the producer default, but it is stricter than the documented authority boundary: Eggpack explicitly does not select destinations or replacement authorization, and ADR-0004 assigns installation policy to the application.

M003 requires this distinction because current Eggsact updates the exact running executable path. A renamed but otherwise valid installation must not silently be retargeted to `<current parent>/eggsact`, nor should Eggsact need to bypass the adapter's evidence mapping to preserve its current destination policy.

M003a establishes an additive caller-bound destination seam before the real Eggsact manifest path lands.

## 2. Readiness and dependencies

Hard dependencies are already closed:

- Eggpack ReleaseManifest M001/M001a/M002;
- Eggpack interoperability M001a;
- Eggup adapter M001/M001a;
- Eggup archive handoff M002/M002a;
- Eggup planning reconciliation C010.

Interface dependency is stable:

- Eggpack's consumer contract defines `install` as target-local install identity/default basename, not destination/root/ownership authority.

Operational dependency:

- none. M003a is local adapter API work and qualification only.

M003 real-consumer adoption becomes dependency-ready again only after M003a closes.

## 3. Current evidence

At the repository baseline:

- `ManifestProjection::Installable` exposes each `ArtifactRequirement` with manifest artifact name, exact size, digest, `MemberId`, and manifest-derived `destination`;
- `ManifestProjection::materialize_artifact_set` accepts exact acquired-path and permission maps but no caller destination map;
- it creates `ArtifactMember` values using the manifest-derived destination directly;
- `core_plan_for_archive` likewise constructs each archive member's destination from manifest `install`;
- `InstallPlan` itself already supports arbitrary validated relative destinations and caller-selected installation roots;
- no lower Eggup-core change is required to represent caller-owned destinations;
- current compatibility tests prove manifest relationship/size/digest preservation but do not prove that destination authorization remains caller-owned when it differs from the producer default.

At current Eggsact:

- the destination is the exact `env::current_exe()`;
- the installation root is `current_exe.parent()`;
- ownership is proven against that exact current executable;
- the existing updater therefore supports canonical installer names and manually renamed executable basenames;
- the producer manifest's canonical direct install identity remains `eggsact` / `eggsact.exe`.

Without M003a, the manifest path would either regress renamed-in-place update behavior or force Eggsact to reconstruct deployment members outside the adapter, defeating the purpose of M003.

## 4. Invariants

### Authority

- Eggpack remains authoritative for artifact identity, relationships, exact byte size, and SHA-256 evidence.
- The application remains authoritative for installation root and exact relative destination.
- Manifest `install` remains a target-local logical/default install identity; it does not authorize replacement of a filesystem destination.
- Destination binding must not alter product, release, target, artifact, size, digest, or bundle relationship evidence.
- Ownership verification remains downstream in `eggup-core`; destination binding does not prove ownership.

### Dependency boundary

- only `eggup-eggpack` may depend on `eggpack-manifest`;
- no new Eggpack edge may enter `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, or `eggup-service`;
- no producer build/CI/contract crate may enter runtime consumers.

### Compatibility

- existing public materialization helpers remain behavior-compatible for consumers that accept manifest default destinations;
- the corrective should be additive before 1.0 rather than forcing unrelated consumers to migrate immediately;
- direct, bundle, and archive paths must expose the same authority model;
- no release/origin/fallback/authenticity/service policy enters the adapter.

### Security

- caller destination maps must be exact: missing, extra, duplicate/colliding, or invalid destinations fail closed;
- destination strings remain subject to existing Eggup normalized-relative-destination validation;
- absolute paths, traversal, separators/invalid forms rejected by the existing core destination contract must remain rejected;
- destination binding must not weaken exact acquired-file maps, exact-size checks, SHA-256 propagation, symlink rejection, permission binding, or archive bound-source semantics.

## 5. Scope

### In scope

- add an additive caller-bound destination materialization entry point for direct/bundle projections;
- add the equivalent caller-bound destination entry point for archive `InstallPlan` construction;
- preserve current methods as compatibility wrappers using the manifest default destination map;
- exact destination-map validation keyed by stable projection member identity;
- regression tests proving producer identity/evidence and caller destination policy remain separate;
- README/API documentation for default versus caller-bound destinations;
- full adapter/workspace stable/MSRV/hosted qualification;
- update M003 handoff text to use the new caller-bound path.

### Out of scope

- changing ReleaseManifest v1;
- changing Eggpack producer contracts;
- changing `eggup-core::ArtifactMember` or `InstallPlan` unless an independently generic defect is found;
- release discovery, URL construction, target aliasing, fallback policy, candidate policy, ownership inference, service policy, or authenticity;
- publishing `eggup-eggpack` or `eggpack-manifest`;
- implementing the Eggsact manifest network path itself;
- removing existing adapter APIs.

## 6. Required production changes

### 6.1 Direct/bundle destination binding

Add an API equivalent to:

~~~text
ManifestProjection::materialize_artifact_set_with_destinations(
    acquired: HashMap<String, PathBuf>,
    destinations: HashMap<MemberId, String>,
    permissions: HashMap<MemberId, PermissionsIntent>,
) -> Result<ArtifactSet, AdapterError>
~~~

Exact naming may differ, but the contract must be explicit.

Requirements:

1. only `ManifestProjection::Installable` is accepted;
2. `acquired`, `destinations`, and `permissions` must each contain exactly the required projected member set;
3. artifact filename -> acquired path relationship remains producer-manifest-derived;
4. member identity remains the projection's stable `MemberId`;
5. destination comes only from the caller destination map;
6. exact size, regular-file/non-symlink, SHA-256 requirement, and permission behavior remain unchanged;
7. invalid caller destinations fail through existing Eggup validation and are mapped to bounded adapter errors;
8. no URL or filesystem content is echoed in new diagnostics.

Retain `materialize_artifact_set` as a compatibility wrapper that constructs the destination map from each manifest `install` default and delegates to the new path. There must be one implementation of the actual materialization logic.

### 6.2 Archive destination binding

Add an equivalent caller-bound destination variant for `core_plan_for_archive`.

Requirements:

- archive member identity/source/member digest facts remain manifest-derived;
- caller destination map must exactly match the projected member IDs;
- advisory extraction source paths remain derived from the extraction root and manifest member relationship;
- destination strings come from caller policy;
- `BoundSources` and object-bound extraction handoff remain unchanged;
- retain the existing `core_plan_for_archive` as a compatibility wrapper using manifest default destinations.

Do not modify extraction authority or cleanup semantics.

### 6.3 Default-destination helper

If useful to avoid duplicated wrapper logic, add a small pure helper that returns the manifest-default `MemberId -> destination` map for installable/archive projections.

It must not be required for callers that bind their own destinations and must perform no I/O.

## 7. Ordered work packages

1. Re-read current `eggup-eggpack` code and tests at handoff SHA.
2. Add failing tests first for a caller destination differing from manifest `install`.
3. Add exact destination-map negative tests.
4. Implement direct/bundle caller-bound materialization.
5. Refactor the current direct/bundle helper into a compatibility wrapper.
6. Implement the archive caller-bound `InstallPlan` path and compatibility wrapper.
7. Re-run all M001a/M002 adapter regressions.
8. Confirm lower-crate dependency trees remain Eggpack-free.
9. Measure public API/doc/package impact.
10. Write M003a closure and update M003/roadmap/registry from blocked-on-M003a back to ready if qualification is clean.

## 8. Failure, restart, and contention semantics

Projection and destination-map construction remain pure.

Materialization remains metadata-read-only until normal Eggup staging begins downstream. A destination-map failure performs no destination mutation.

Archive plan construction remains pure and performs no extraction or commit.

If a destination map is incomplete or contains an extra key, fail before any `ArtifactSet` or `InstallPlan` is returned.

If current code changes concurrently, refresh the baseline and preserve any newer compatibility API rather than force this exact method spelling.

## 9. Compatibility and migration

This is an additive API corrective.

Existing consumers using manifest default destinations continue to use the current helpers with unchanged semantics.

M003 Eggsact should use the caller-bound variant and bind the selected member to the exact basename of `env::current_exe()` under its existing installation root. That preserves the current update-in-place contract for canonical and renamed executable basenames while keeping ownership verification against the exact running executable.

No registry publication is authorized by M003a.

## 10. Required tests

At minimum add focused tests for:

1. direct projection with manifest default destination through compatibility wrapper;
2. direct projection with caller destination different from manifest `install`;
3. bundle projection with independent caller destinations for every member;
4. missing destination key;
5. extra destination key;
6. invalid/traversal/absolute destination rejected by core validation;
7. destination change does not alter member identity;
8. destination change does not alter expected size or SHA-256;
9. permission map remains exact and independent from destination map;
10. acquired-file map remains exact and independent from destination map;
11. archive plan uses caller destinations while preserving manifest source/member evidence;
12. archive destination missing/extra/invalid failures;
13. old helpers produce byte-for-byte/equality-equivalent deployment inputs to their prior manifest-default behavior where comparable;
14. diagnostics do not include source file contents or credential-bearing material.

Re-run all current `eggup-eggpack` tests.

## 11. Verification commands

Run and record:

~~~text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test -p eggup-core --all-targets --all-features --locked
cargo test -p eggup-archive --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-eggpack --all-targets --locked
cargo tree -p eggup-eggpack --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-acquisition --locked
cargo tree -p eggup-service --locked
./scripts/check-local.sh
git diff --check
~~~

Hosted Linux stable/MSRV, macOS, and Windows adapter/workspace evidence must be recorded truthfully. A platform lane that cannot run the relevant tests is not inferred from another platform.

## 12. Documentation updates

Required:

- `crates/eggup-eggpack/README.md`;
- public rustdoc for all new APIs;
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`;
- `plans/registry.md`;
- M003 implementation plan dependency/readiness text;
- new `plans/closure/eggpack-manifest-interoperability/003a-status.md` at closure.

Do not rewrite historical M001/M001a/M002 closure evidence.

## 13. Acceptance criteria

M003a closes only when:

- direct/bundle callers can bind exact relative destinations independently of manifest default install names;
- archive callers have the same authority model;
- current helpers remain compatible wrappers;
- manifest artifact/member identity, size, digest, and relationships remain authoritative and unchanged;
- destination maps are exact and fail closed;
- lower Eggup crates remain Eggpack-independent;
- no release/origin/ownership/authenticity/service policy enters the adapter;
- full stable/MSRV/hosted qualification is green or any platform limitation is explicitly recorded;
- no medium-or-higher generic adapter defect remains.

Closure of M003a re-enables the already-registered M003 Eggsact consumer implementation. It does not close M003 itself.

## 14. Stop conditions

Stop and write a broader corrective/ADR only if:

- preserving application-owned destination policy requires changing ReleaseManifest v1;
- `eggup-core` cannot represent caller destinations without a breaking core redesign;
- member identity and deployment destination cannot be separated without losing bundle/archive relationship safety;
- fixing the direct path would require leaving the archive path with contradictory authority semantics;
- the implementation would require destination or ownership inference inside the adapter.

If Eggsact-specific behavior alone is the problem and the generic adapter already provides caller binding after refresh, close M003a as no-production-delta qualification rather than adding redundant API.

## 15. Closure evidence required

Record:

- exact implementation SHA;
- exact pre-implementation baseline;
- public API delta;
- direct/bundle/archive requirement-to-evidence matrix;
- custom-destination and exact-map negative evidence;
- compatibility-wrapper evidence;
- dependency trees proving only `eggup-eggpack` depends on Eggpack;
- stable/Rust 1.89/macOS/Windows results and run IDs;
- docs/package impact;
- unresolved findings with severity;
- explicit M003 readiness disposition.

## 16. Handoff notes

M003a is a prerequisite corrective discovered while refreshing M003 against the real Eggsact updater.

Do not implement the Eggsact network/fallback path in this pass. Once M003a closes, resume `003-eggsact-real-consumer-manifest-adoption.md` against the then-current Eggsact head.

For Eggsact, caller destination binding must preserve the existing exact-current-executable update policy. The manifest may choose the producer artifact and provide default install identity; it must not become replacement authorization.
