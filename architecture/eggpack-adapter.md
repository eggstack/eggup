# Eggpack Adapter — Deep-Dive Review (`crates/eggup-eggpack`)

Scope: `crates/eggup-eggpack/src/lib.rs`, `README.md`, `Cargo.toml`,
`tests/interoperability.rs`, `tests/archive_handoff.rs`,
`tests/caller_destinations.rs`, `tests/fixtures/*.json` + `README.md`.

Symbols are referenced by name rather than by absolute line number; the crate is
under active change and line anchors in this document would rot.

## 1. Purpose and layer boundary

`eggup-eggpack` is an **optional, published leaf adapter** from Eggpack
`ReleaseManifest` v1 to caller-owned Eggup acquisition/deployment inputs. It is
published to crates.io (`eggup-eggpack 0.1.2`, published 2026-10-02; see
`plans/closure/eggpack-manifest-interoperability/004-status.md`).

Per `README.md` and the crate-level rustdoc:

- Projects a manifest to Eggup inputs; does **not** fetch, install, extract,
  verify authenticity, choose URLs, choose installation roots, set ownership,
  choose permissions, or define release policy.
- SHA-256 values are **integrity evidence only** (`IntegrityRequirement::Sha256`),
  not authenticity.
- Archives are **extraction-required**: facts are preserved, installability is refused.

Dependency direction (from `Cargo.toml`):

- Depends on `eggup-core = "=0.1.2"`, `eggup-archive = "=0.1.2"`, and
  `eggup-acquisition = "=0.1.2"` (local paths, exact pins to the published set).
- Depends on `eggpack-manifest = "=0.1.0"` from the **registry** — no Git,
  branch, tag, or path edge. The published crate's `src/lib.rs` is byte-identical
  to the source at the previously pinned revision
  `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, which is retained only as fixture
  provenance in `tests/` (`PROMOTED_EGGPACK_REV`), not as a dependency source.
- Also depends on `sha2 0.10.9` (runtime archive digest check).
- No `publish = false`; the key was removed at registry promotion.
- Dev-dependencies only: `serde` (derive), `serde_json`, `flate2`, `tar`, `zip`.
- Lower Eggup crates remain independent of Eggpack; only this leaf crate may
  import `eggpack_manifest::{ArtifactForm, ReleaseManifest, MAX_DOCUMENT_BYTES}`.

Caller-owned responsibilities:

| Concern | Owner |
|---|---|
| Manifest byte acquisition, URL selection | Caller |
| `AcquisitionRequest` construction (URL may contain credentials; adapter preserves it byte-for-byte, never logs it) | Caller |
| `FetchLimits` baseline (timeouts, metadata cap, artifact cap) | Caller |
| Acquired-file staging (absolute regular files, exact sizes) | Caller |
| `PermissionsIntent` per `MemberId` | Caller |
| Exact destination map per `MemberId` (manifest `install` is a producer default, not an authorization) | Caller |
| I/O orchestration of extraction, commit, and deferred `DeferredCleanup` | Caller |
| Installation roots, ownership, release policy, authenticity/signature | Caller / outer layers |
| Archive extraction / qualification | Caller-executed via the `eggup-archive` handoff (see §3a); the crate supplies plans and bound sources, never extraction |

No-I/O guarantee: `project` / `project_json` perform validation + projection only.
`bind_requests` performs no fetch. `materialize_artifact_set` performs only
`symlink_metadata` + size/type checks, then pure `ArtifactMember` / `ArtifactSet`
construction. No `reqwest`/`hyper`/`tokio`, no decompression (`flate2`), no
`chmod`/`chown`/`sudo`, no service managers, no mirror/fallback/latest logic —
enforced by the authority-token test
`adapter_source_claims_no_producer_or_service_authority`
(`tests/interoperability.rs`).

Safety/lint posture: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.

## 2. Types

### `AdapterError`

`#[non_exhaustive]`, `Debug`, `Display + std::error::Error`. Details never contain
URLs or file contents:

- `InvalidManifest` — manifest or Eggup identity invalid; also covers
  oversize input, invalid UTF-8, malformed JSON, unsupported schema, bad
  `sha256_bytes()`, bad `MemberId`/`ProductId`/`ReleaseId` conversions where
  applicable. Display: `"invalid manifest or identity"`.
- `TargetNotFound` — exact canonical target absent. Display:
  `"canonical target not found"`.
- `MapMismatch(&'static str)` — a map differs from the required set in length or
  keys. The static tag is one of `"acquisition request"`, `"acquired path"`,
  `"destinations"`, `"permissions"`, or `"archive bound members"`.
- `CallerLimitTooSmall(String)` — caller artifact limit lower than manifest exact
  size; carries artifact name only.
- `InvalidAcquiredFile(String)` — acquired path relative, missing, linked,
  non-regular, wrong size, or archive digest-mismatched; carries artifact name
  only.
- `ArchiveExtractionRequired` — archive member bytes cannot become installable
  without qualified extraction.
- `UnsupportedArchiveFormat` — the caller-selected archive file name is not an
  accepted `.tar.gz` / `.tgz` / `.zip` suffix.
- `Eggup(String)` — lower-layer construction failure (`FetchLimits::validate`,
  `MemberId`/`ProductId`/`ReleaseId`/`ArtifactMember`/`ArtifactSet` errors);
  carries the lower-layer diagnostic string.

Leak discipline is tested: oversized/invalid/secret-bearing inputs all map to
`InvalidManifest` with no `"private"` / `"super-secret"` / `"token=hidden"` in
`Display + Debug`; the `MapMismatch` diagnostic must not contain URL text.

### `MAX_MANIFEST_BYTES`

```rust
pub const MAX_MANIFEST_BYTES: usize = MAX_DOCUMENT_BYTES;
```

Re-export of Eggpack's published document-size bound. Enforced pre-parse in
`project_json` (`input.len() > MAX_MANIFEST_BYTES` → `InvalidManifest`).
At-limit input (padded to exactly `MAX_MANIFEST_BYTES`) is accepted in the unit
test; `MAX + 1` is rejected.

### `ArtifactRequirement`

One manifest artifact paired with its exact installed-member relationship:

- `artifact_name: String` — exact map key only.
- `exact_size: u64` — exact expected byte count.
- `sha256: [u8; 32]` — manifest SHA-256 bytes (decoded via `sha256_bytes()`).
- `member_id: MemberId` — from manifest `install` string.
- `destination: String` — flat relative install destination (same `install` string).

### `ArchiveMemberRequirement`

Preserved evidence only; **not verified extracted files**:

- `source: String` — normalized archive source path.
- `install: String` — flat installation identity.
- `exact_size: u64` — from `member.bytes.size`.
- `sha256: [u8; 32]` — from `member.bytes.sha256_bytes()`.

### `ManifestProjection`

```rust
pub enum ManifestProjection {
    Installable { product: ProductId, release: ReleaseId, target: String,
                  artifacts: Vec<ArtifactRequirement> },
    Archive { product: ProductId, release: ReleaseId, target: String,
              artifact_name: String, exact_size: u64, sha256: [u8; 32],
              members: Vec<ArchiveMemberRequirement> },
}
```

- `Installable` covers Direct (1 artifact) and Bundle (N artifacts, manifest order).
- `Archive` retains one archive artifact + expected member relationships.
- Private `installable()` helper returns
  `(product, release, artifacts)` or `Err(ArchiveExtractionRequired)`. Both
  `materialize_artifact_set` and `install_ids` go through it, so archives can
  never reach `ArtifactSet` construction via the generic path.

### `PlannedAcquisition`

Exact request + manifest evidence for a transport:

- `artifact_name`, `request: AcquisitionRequest` (exact unmodified caller request),
  `limits: FetchLimits` (baseline with `max_artifact_bytes` tightened to manifest
  size), `exact_size`, `sha256`, `member_id`.

## 3. Functions

### `project(&ReleaseManifest, canonical_target: &str)`

Pure, no I/O:

1. `manifest.validate()` → `InvalidManifest` on failure.
2. `manifest.target(canonical_target)` → `TargetNotFound` on failure. Exact match
   only; no alias/nearest fallback.
3. `ProductId::new(manifest.product_id)` / `ReleaseId::new(manifest.release_id)`
   → `Eggup(_)` on failure.
4. Match on `target.form`:
   - `Direct { artifact, install }` → single-element `Installable`.
   - `Bundle { entries }` → N-element `Installable`, preserving entry order;
     each entry maps `artifact.name/size/sha256_bytes()` + `MemberId::new(install)`
     + `destination = install`.
   - `Archive { artifact, members }` → `Archive` with decoded archive digest and
     per-member `ArchiveMemberRequirement`s.
5. Any `sha256_bytes()` decode failure → `InvalidManifest`.

### `project_json(&[u8], canonical_target: &str)`

Bounded JSON entry point. Consumers that fetch manifest bytes call this; holders
of a parsed manifest call `project`:

1. Length check against `MAX_MANIFEST_BYTES` → `InvalidManifest`.
2. `std::str::from_utf8` → `InvalidManifest` (no input echoed).
3. `ReleaseManifest::from_json` (delegates parsing/schema validation to
   `eggpack-manifest`) → `InvalidManifest` (parser diagnostics discarded).
4. `project(&manifest, canonical_target)`.

### `ManifestProjection::bind_requests(HashMap<String, AcquisitionRequest>, FetchLimits)`

- Validates baseline via `FetchLimits::validate()` → `Eggup(_)` on failure.
- Requirement set: `Installable.artifacts`, or for `Archive` a single synthetic
  requirement keyed by `artifact_name` with `MemberId::new(artifact_name)`.
- Exact-map check: lengths equal **and** every requirement name present
  (missing or extra → `MapMismatch("acquisition request")`).
- Per requirement: if `baseline.max_artifact_bytes < exact_size` →
  `CallerLimitTooSmall(name)`.
- Output per artifact: `PlannedAcquisition` with the **unmodified** caller
  `request` and `limits = FetchLimits { max_artifact_bytes: exact_size,
  ..baseline }`. `max_artifact_bytes` is a plain positive `u64` on the 0.1.2
  acquisition seam, so the cap is always set to the exact manifest size.
  Timeouts and `max_metadata_bytes` are preserved, never widened. Never widens a
  caller cap upward — only tightens or errors.

### `ManifestProjection::materialize_artifact_set(HashMap<String, PathBuf>, HashMap<MemberId, PermissionsIntent>)`

All-or-nothing Direct/Bundle `ArtifactSet` construction. This is a compatibility
wrapper: it builds the manifest-default destination map via
`ManifestProjection::default_destinations()` and delegates to
`materialize_artifact_set_with_destinations`. There is one materialization
implementation.

1. `installable()` → archives immediately fail with `ArchiveExtractionRequired`.
2. `acquired` map must match requirement names exactly (length + keys) or
   `MapMismatch("acquired path")`.
3. `destinations` map must match requirement `member_id`s exactly or
   `MapMismatch("destinations")`; see the caller-owned destination-policy note
   below.
4. `permissions` map must match requirement `member_id`s exactly or
   `MapMismatch("permissions")`. Permissions are explicit caller intent; no
   defaults are invented.
5. Per requirement:
   - Path must be absolute, else `InvalidAcquiredFile(name)`.
   - `fs::symlink_metadata` (does **not** follow symlinks) must succeed and
     `file_type().is_file()` must hold — rejects symlinks, directories, and
     missing files — and `meta.len() == exact_size`, else
     `InvalidAcquiredFile(name)`.
   - Destination comes from the caller's `destinations` map keyed by `member_id`.
6. Duplicate member destinations fail with `MapMismatch("destinations")`.
7. `ArtifactMember::new(member_id, path, destination)` → `Eggup(_)` on failure;
   `.with_permissions(intent)` + `.with_integrity(Sha256(sha256))`.
8. `ArtifactSet::new(members)` → `Eggup(_)` on failure (e.g. duplicate IDs,
   empty set cannot occur here since Installable always has ≥1 artifact).

Content hashing is **not** performed here; the SHA-256 digest is attached as an
`IntegrityRequirement` for downstream verification at install time.

### `ManifestProjection::materialize_artifact_set_with_destinations(acquired, destinations, permissions)`

The caller-destination-policy entry point, and the single implementation that
`materialize_artifact_set` delegates to. Identical to the wrapper above except
that the exact relative destination of every member is supplied by the caller
through an exact `MemberId -> destination` map instead of the manifest default.

Manifest `install` values are **producer default identity only**. They do not
authorize replacement of a filesystem destination. A consumer with its own
destination policy — for example, updating the exact running executable in
place, including a renamed basename — calls this method directly. Missing, extra,
or colliding destinations fail closed (`MapMismatch("destinations")`) before any
set is returned; an invalid destination fails through `eggup-core` destination
validation into `Eggup(_)`. Member identity, exact size, SHA-256 evidence,
acquired-file relationships, and permission intent are unchanged.

Replacement authorization stays outside this helper: it records where bytes
should be installed and never proves control of a live destination. That proof
is `eggup-core` ownership verification at commit time.

### `install_ids(&ManifestProjection)`

Returns `(&ProductId, &ReleaseId)` for installable projections; archives fail
with `ArchiveExtractionRequired` via `installable()`.

## 3a. M002 archive extraction handoff (`eggup-archive`)

Before the M002 handoff existed, an archive projection was a dead end: the crate
could prove facts about the archive and its declared members, but had no way to
hand those facts to a qualified extractor and then to core staging. M002 added a
runtime `eggup-archive = "=0.1.2"` dependency plus a `sha2 0.10.9` dependency and
a set of free functions that translate `ManifestProjection::Archive` into the
`eggup-archive` and `eggup-core` types the caller needs. Direct and bundle APIs
are unchanged.

The documented caller sequence (from the crate-level `eggup-archive` handoff
comment) is:

1. `project` / `project_json` → `ManifestProjection::Archive`.
2. `bind_requests` → one exact `PlannedAcquisition` for the archive artifact.
3. `validate_acquired_archive(&projection, path)` — prove path/size/digest
   continuity **before** extraction, so no unverified byte is ever extracted.
4. `archive_format_for_name(name)` → `ArchiveFormat` (fail-closed suffix
   classification; the caller still owns artifact selection).
5. `archive_plan_for(&projection, format, path, limits)` → `eggup_archive::ArchivePlan`.
6. `eggup_archive::extract` (caller-owned).
7. `core_plan_for_archive` (or `core_plan_for_archive_with_destinations`) → core
   `InstallPlan`, while the advisory extraction path is still reachable.
8. `extracted.persist().into_bound_sources()?.into_members()` → `Vec<BoundMember>`.
9. `bind_archive_members(&projection, bound_members)` → `BoundSources`.
10. `InstallPlan::prepare_with_bound_sources`.

The helpers in detail:

- **`archive_format_for_name(file_name) -> Result<ArchiveFormat, AdapterError>`** —
  accepts only `.tar.gz`, `.tgz`, and `.zip` (ASCII case-insensitive); anything
  else is `UnsupportedArchiveFormat`. It never fetches, lists, or ranks releases.
- **`validate_acquired_archive(&projection, path)`** — the same continuity gate
  `materialize_artifact_set` applies to direct/bundle files, moved ahead of
  extraction: absolute path, `symlink_metadata` regular file (never a symlink,
  directory, or missing path), `meta.len() == exact_size`, and a bounded streaming
  SHA-256 equal to the manifest archive digest. Failures are
  `InvalidAcquiredFile(artifact_name)`. No network, no alternate-source retry, no
  provenance claim.
- **`archive_plan_for(&projection, format, path, limits) -> Result<ArchivePlan, _>`** —
  each declared member maps `source -> ArchiveMember::source_path` and
  `install -> ArchiveMember::output_name`, with `expected_size` / `expected_sha256`
  from the projection so the extractor can verify bytes against the manifest during
  extraction. It performs no I/O and does **not** re-validate the archive bytes —
  the caller must already have run `validate_acquired_archive`.
- **`core_plan_for_archive(...)` / `core_plan_for_archive_with_destinations(...)`** —
  build the core `InstallPlan` from the projection's archive members, `extraction_root`,
  `destination_root`, destinations, and permissions. The first is a compatibility
  wrapper that uses `default_destinations()`; the second is the caller-destination
  form. The returned plan's `BoundSources` are still empty — bound members are
  transferred separately by `bind_archive_members`. When a bound handle is
  supplied, advisory paths are never opened during staging; the recorded path
  satisfies `ArtifactMember::new` validation and is diagnostics only.
- **`bind_archive_members(&projection, bound_members) -> Result<BoundSources, _>`** —
  identity is the projection's `install` string converted into a `MemberId`; bound
  members are paired to the projected archive members in declaration order.
  Mismatched counts fail closed with `MapMismatch("archive bound members")`.
  Non-archive projections fail with `ArchiveExtractionRequired`.

Boundary: the adapter performs **no** extraction, commit, or cleanup itself. I/O
orchestration and the deferred `eggup-archive::DeferredCleanup` stay caller-owned.
The destination-policy boundary is identical to the direct/bundle case — the
caller owns URL selection, install-root and destination selection, ownership,
permission, and authenticity policy, and the crate stops at extraction-required
evidence. Coverage for this handoff is in `tests/archive_handoff.rs` and
`tests/caller_destinations.rs`.

## 4. Direct / Bundle / Archive + canonical-target exactness

- **Direct** (`direct-manifest.json`: `eggsact` 1.2.6, targets
  `aarch64-apple-darwin` + `x86_64-unknown-linux-gnu`, each size 3,
  `sha256 ba7816bf…15ad`, `install "eggsact"`): projects to 1-element
  `Installable`; `transaction_group "one-artifact-set"` in
  `projection-direct.json`.
- **Bundle** (`bundle-manifest.json`: `codegg` 2.4.0, Linux-only, 3 entries —
  `codegg` size 3, `codegg-helper` size 4, `codegg-manifest-2.4.0.json` size 5,
  distinct digests/installs): projects to 3-element `Installable` in manifest
  order; determinism asserted by `project == project` and order-vs-fixture
  comparison. Must not contain cross-product entries (e.g. no `eggsact`-prefixed
  names). `transaction_group "one-artifact-set"`.
- **Archive** (`archive-manifest.json`: `egress` 3.1.0, artifact
  `egress-3.1.0-x86_64-unknown-linux-gnu.tar.gz` size 3 + 2 members
  `egress` size 3 / `bin/egress-helper` size 4): projects to `Archive` with member
  evidence; fixture declares `"extraction_required": true` and
  `transaction_group "one-artifact-set-after-consumer-owned-extraction"`.
  `bind_requests` still works (single archive request, tightened limit), but
  `materialize_artifact_set` / `install_ids` always return
  `ArchiveExtractionRequired`.
- **Canonical-target exactness**: `manifest.target(canonical_target)` is an exact
  lookup. Alias `linux-x64` → `TargetNotFound` (unit + `negative_01`);
  `x86_64-pc-windows-gnu` (from `wrong-target.json`) and `aarch64-apple-darwin`
  against the Linux-only bundle → `TargetNotFound` (`negative_02`). No alias
  resolution, no nearest-target fallback.

## 5. Tests and fixtures

### Unit tests (inline `mod tests`, 7 tests)

- Bounded projection: fixture under `MAX_MANIFEST_BYTES`; space-padded at-limit
  input still projects.
- Invalid documents (oversized, invalid UTF-8, malformed JSON containing a fake
  secret URL, unsupported `schema_version` 999) all → `InvalidManifest` with no
  secret leakage.
- Alias target rejected; corrected fixtures project (Direct → Installable,
  Archive → Archive, Bundle → 3-artifact Installable); archive `bind_requests`
  with empty map → `MapMismatch`.
- Request binding preserves URL byte-for-byte and tightens cap; `exact - 1`
  caller cap → `CallerLimitTooSmall`.
- Materialization: exact-size file (`b"abc"`, size 3) OK; rewritten wrong-size
  file → `InvalidAcquiredFile`.

### Interoperability matrix (`tests/interoperability.rs`)

- Fixtures copied byte-for-byte from Eggpack rev `678bbf0…e41` (M001a closure);
  SHA-256 table in `tests/fixtures/README.md`.
- Test-only strict structs (`deny_unknown_fields`: `ReleaseIdentity`,
  `InstallableUnit`, `DirectProjection`, `BundleProjection`,
  `ArchiveAcquisitionUnit`, `ArchiveProjectedMember`, `ArchiveProjection`) —
  evidence only, never exported.
- Positive (6): direct-vs-fixture match; direct binding preserves URL/timeouts/
  metadata bound + tightens cap; direct materialization yields single member with
  manifest digest + caller permissions; bundle 3-member match + no `eggsact`
  entries + deterministic re-projection; bundle exact maps → 3-member set with
  per-member digests; archive facts preserved + single-request binding tightened
  + materialization blocked.
- Negative 01–20 (plan §7): alias/unknown target; unknown schema (strict parse
  fails; structurally parsed still `InvalidManifest` on `validate`); missing/
  extra acquisition request; caller limit too small; bounds never widened;
  missing/extra acquired artifact; missing/extra permission; relative path;
  symlink (skipped on privilege-constrained Windows); directory; missing file;
  wrong size; archive materialization blocked; bundle cardinality == 3;
  substituted `eggsact` member fails; crossed member/install pairing fails.
- Authority/dependency guards: `adapter_pins_registry_eggpack_manifest_and_no_producer_crates`
  asserts `Cargo.toml` carries `eggpack-manifest = "=0.1.0"`, contains no
  `git =`/`rev =`/`branch =`/`tag =`/`eggstack/eggpack.git` VCS edge, keeps the
  exact `=0.1.2` pins on `eggup-core` / `eggup-archive` / `eggup-acquisition`, and
  does not mention `eggpack-core`/`eggpack-contract`/`eggpack-bootstrap`.
  `adapter_source_claims_no_producer_or_service_authority` asserts production
  source (pre-`#[cfg(test)]`) contains no service/producer authority tokens
  (`reqwest`, `tokio`, `mirror`, `install_root`, `chown`, `signature`,
  `decompress`, `std::process`, …).

### Handoff and destination tests

- `tests/archive_handoff.rs` — M002 archive extraction handoff: format
  classification, pre-extraction continuity validation, `ArchivePlan`
  construction, `InstallPlan` construction, and `BoundSources` transfer.
- `tests/caller_destinations.rs` — M003a caller-bound destination seam:
  `default_destinations`, `materialize_artifact_set_with_destinations`, and
  `core_plan_for_archive_with_destinations`, including fail-closed missing, extra,
  and colliding destination cases.

### Fixture inventory (`tests/fixtures/`)

| File | Role |
|---|---|
| `direct-manifest.json` | `eggsact` 1.2.6 Direct, 2 targets |
| `bundle-manifest.json` | `codegg` 2.4.0 Bundle, 3 Linux entries |
| `archive-manifest.json` | `egress` 3.1.0 Archive, 1 artifact + 2 members |
| `projection-direct.json` / `projection-bundle.json` / `projection-archive.json` | Expected selected-target/identity/units/transaction-group evidence |
| `unknown-schema.json` | `schema_version` 2 rejection case |
| `wrong-target.json` | `{"selected_target":"x86_64-pc-windows-gnu",…}` unknown-target probe |
| `README.md` | Provenance + SHA-256 table |

## 6. Review checklist

1. **Leaf direction**: no `eggpack-*` imports outside `eggup-eggpack`; no producer
   crates (`eggpack-core/-contract/-bootstrap`) in `Cargo.toml`; `eggpack-manifest`
   stays an exact registry dependency with no Git or path edge.
2. **No authority creep**: no URL/root/ownership/permission/policy/authenticity/
   fetch/extract/decompress/service-manager code in production source; caller
   supplies requests, limits, paths, permissions, and destinations.
3. **Exactness**: canonical-target lookup exact; all four maps (requests, paths,
   destinations, permissions) exact-matched; sizes exact (`meta.len() == exact_size`);
   bundle order deterministic; digest pairing bidirectional.
4. **Archive separation**: `Archive` never reaches `ArtifactSet` through the
   direct/bundle path; `installable()` is the single gate for
   `materialize_artifact_set*` and `install_ids`. The M002 handoff is the only
   archive-to-`ArtifactSet` path, and it goes through the caller-owned
   `eggup-archive` extraction sequence.
5. **File validation**: absolute-only paths, `symlink_metadata` (no following),
   regular-file check, missing/dir/symlink/size/archive-digest failures map to
   `InvalidAcquiredFile(name)` without leaking paths/URLs.
6. **Limit tightening**: `max_artifact_bytes` only ever set to `exact_size` or
   rejected; timeouts/metadata bound passed through unchanged.
7. **Error hygiene**: `InvalidManifest`/`MapMismatch` carry no input/URL content;
   `Eggup(_)` wraps lower-layer diagnostics only; `Display` strings stable and
   secret-free (covered by leak assertions).
8. **Bound enforcement**: `MAX_MANIFEST_BYTES == MAX_DOCUMENT_BYTES` checked
   before UTF-8/JSON parsing; at-limit accepted, `MAX+1` rejected.
9. **Fixture fidelity**: fixtures byte-identical to the pinned Eggpack rev (verify via
   `README.md` SHA-256 table); projection JSON compared with strict
   `deny_unknown_fields` structs; bundle cardinality 3, no cross-product entries.
10. **Digest handling**: `sha256_bytes()` failures → `InvalidManifest`; digests
    attached as `IntegrityRequirement::Sha256`, never treated as authenticity.
