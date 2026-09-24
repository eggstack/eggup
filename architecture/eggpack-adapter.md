# Eggpack Adapter — Deep-Dive Review (`crates/eggup-eggpack`)

Scope: `crates/eggup-eggpack/src/lib.rs` (492 lines), `README.md`, `Cargo.toml`,
`tests/interoperability.rs` (1018 lines), `tests/fixtures/*.json` + `README.md`.

## 1. Purpose and layer boundary

`eggup-eggpack` is an **optional, unpublished leaf adapter** from Eggpack
`ReleaseManifest` v1 to caller-owned Eggup acquisition/deployment inputs.

Per `README.md` and `src/lib.rs:3`:

- Projects a manifest to Eggup inputs; does **not** fetch, install, extract,
  verify authenticity, choose URLs, choose installation roots, set ownership,
  choose permissions, or define release policy.
- SHA-256 values are **integrity evidence only** (`IntegrityRequirement::Sha256`),
  not authenticity.
- Archives are **extraction-required**: facts are preserved, installability is refused.

Dependency direction (from `Cargo.toml:15-18`):

- Depends on `eggup-core`, `eggup-acquisition` (both `=0.1.1`, local paths).
- Depends on `eggpack-manifest` `=0.1.0` via git pin:
  `https://github.com/eggstack/eggpack.git`, rev
  `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, package `eggpack-manifest`.
- `publish = false`.
- Dev-dependencies only: `serde` (derive), `serde_json`.
- Lower Eggup crates remain independent of Eggpack; only this leaf crate may
  import `eggpack_manifest::{ArtifactForm, ReleaseManifest, MAX_DOCUMENT_BYTES}`
  (`src/lib.rs:5`).

Caller-owned responsibilities:

| Concern | Owner |
|---|---|
| Manifest byte acquisition, URL selection | Caller |
| `AcquisitionRequest` construction (URL may contain credentials; adapter preserves it byte-for-byte, never logs it) | Caller |
| `FetchLimits` baseline (timeouts, metadata cap, artifact cap) | Caller |
| Acquired-file staging (absolute regular files, exact sizes) | Caller |
| `PermissionsIntent` per `MemberId` | Caller |
| Installation roots, ownership, release policy, authenticity/signature | Caller / outer layers |
| Archive extraction / qualification | Separate boundary; explicitly out of scope |

No-I/O guarantee: `project` / `project_json` perform validation + projection only.
`bind_requests` performs no fetch. `materialize_artifact_set` performs only
`symlink_metadata` + size/type checks, then pure `ArtifactMember` / `ArtifactSet`
construction. No `reqwest`/`hyper`/`tokio`, no decompression (`flate2`), no
`chmod`/`chown`/`sudo`, no service managers, no mirror/fallback/latest logic —
enforced by the authority-token test
(`tests/interoperability.rs:979-1018`).

Safety/lint posture: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.

## 2. Types

### `AdapterError` (`src/lib.rs:16-48`)

`#[non_exhaustive]`, `Debug`, `Display + std::error::Error`. Details never contain
URLs or file contents:

- `InvalidManifest` — manifest or Eggup identity invalid; also covers
  oversize input, invalid UTF-8, malformed JSON, unsupported schema, bad
  `sha256_bytes()`, bad `MemberId`/`ProductId`/`ReleaseId` conversions where
  applicable. Display: `"invalid manifest or identity"`.
- `TargetNotFound` — exact canonical target absent. Display:
  `"canonical target not found"`.
- `MapMismatch(&'static str)` — `"acquisition request"`, `"acquired path"`, or
  `"permissions"` map differs from the required set in length or keys.
- `CallerLimitTooSmall(String)` — caller artifact limit lower than manifest exact
  size; carries artifact name only.
- `InvalidAcquiredFile(String)` — acquired path relative, missing, linked,
  non-regular, or wrong size; carries artifact name only.
- `ArchiveExtractionRequired` — archive member bytes cannot become installable
  without qualified extraction.
- `Eggup(String)` — lower-layer construction failure (`FetchLimits::validate`,
  `MemberId`/`ProductId`/`ReleaseId`/`ArtifactMember`/`ArtifactSet` errors);
  carries the lower-layer diagnostic string.

Leak discipline is tested: oversized/invalid/secret-bearing inputs all map to
`InvalidManifest` with no `"private"` / `"super-secret"` / `"token=hidden"` in
`Display + Debug` (`src/lib.rs:373-396`); `MapMismatch` diagnostic must not
contain URL text (`tests/interoperability.rs:658-659`).

### `MAX_MANIFEST_BYTES` (`src/lib.rs:51`)

```rust
pub const MAX_MANIFEST_BYTES: usize = MAX_DOCUMENT_BYTES;
```

Re-export of Eggpack's published document-size bound. Enforced pre-parse in
`project_json` (`input.len() > MAX_MANIFEST_BYTES` → `InvalidManifest`).
At-limit input (padded to exactly `MAX_MANIFEST_BYTES`) is accepted in the unit
test; `MAX + 1` is rejected.

### `ArtifactRequirement` (`src/lib.rs:55-66`)

One manifest artifact paired with its exact installed-member relationship:

- `artifact_name: String` — exact map key only.
- `exact_size: u64` — exact expected byte count.
- `sha256: [u8; 32]` — manifest SHA-256 bytes (decoded via `sha256_bytes()`).
- `member_id: MemberId` — from manifest `install` string.
- `destination: String` — flat relative install destination (same `install` string).

### `ArchiveMemberRequirement` (`src/lib.rs:70-79`)

Preserved evidence only; **not verified extracted files**:

- `source: String` — normalized archive source path.
- `install: String` — flat installation identity.
- `exact_size: u64` — from `member.bytes.size`.
- `sha256: [u8; 32]` — from `member.bytes.sha256_bytes()`.

### `ManifestProjection` (`src/lib.rs:83-112`)

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
- Private `installable()` helper (`src/lib.rs:115-127`) returns
  `(product, release, artifacts)` or `Err(ArchiveExtractionRequired)`. Both
  `materialize_artifact_set` and `install_ids` go through it, so archives can
  never reach `ArtifactSet` construction via the generic path.

### `PlannedAcquisition` (`src/lib.rs:232-245`)

Exact request + manifest evidence for a transport:

- `artifact_name`, `request: AcquisitionRequest` (exact unmodified caller request),
  `limits: FetchLimits` (baseline with `max_artifact_bytes` tightened to manifest
  size), `exact_size`, `sha256`, `member_id`.

## 3. Functions

### `project(&ReleaseManifest, canonical_target: &str)` (`src/lib.rs:248-325`)

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

### `project_json(&[u8], canonical_target: &str)` (`src/lib.rs:331-341`)

Bounded JSON entry point. Consumers that fetch manifest bytes call this; holders
of a parsed manifest call `project`:

1. Length check against `MAX_MANIFEST_BYTES` → `InvalidManifest`.
2. `std::str::from_utf8` → `InvalidManifest` (no input echoed).
3. `ReleaseManifest::from_json` (delegates parsing/schema validation to
   `eggpack-manifest`) → `InvalidManifest` (parser diagnostics discarded).
4. `project(&manifest, canonical_target)`.

### `ManifestProjection::bind_requests(HashMap<String, AcquisitionRequest>, FetchLimits)` (`src/lib.rs:130-187`)

- Validates baseline via `FetchLimits::validate()` → `Eggup(_)` on failure.
- Requirement set: `Installable.artifacts`, or for `Archive` a single synthetic
  requirement keyed by `artifact_name` with `MemberId::new(artifact_name)`.
- Exact-map check: lengths equal **and** every requirement name present
  (missing or extra → `MapMismatch("acquisition request")`).
- Per requirement: if `baseline.max_artifact_bytes < exact_size` →
  `CallerLimitTooSmall(name)`.
- Output per artifact: `PlannedAcquisition` with the **unmodified** caller
  `request` and `limits = FetchLimits { max_artifact_bytes: Some(exact_size),
  ..baseline }`. Timeouts and `max_metadata_bytes` are preserved, never widened;
  an unbounded (`None`) artifact cap is tightened to `Some(exact_size)`.
  Never widens a caller cap upward — only tightens or errors.

### `ManifestProjection::materialize_artifact_set(HashMap<String, PathBuf>, HashMap<MemberId, PermissionsIntent>)` (`src/lib.rs:190-227`)

All-or-nothing Direct/Bundle `ArtifactSet` construction:

1. `installable()` → archives immediately fail with `ArchiveExtractionRequired`.
2. `acquired` map must match requirement names exactly (length + keys) or
   `MapMismatch("acquired path")`.
3. `permissions` map must match requirement `member_id`s exactly or
   `MapMismatch("permissions")`. Permissions are explicit caller intent; no
   defaults are invented.
4. Per requirement:
   - Path must be absolute, else `InvalidAcquiredFile(name)`.
   - `fs::symlink_metadata` (does **not** follow symlinks) must succeed and
     `file_type().is_file()` must hold — rejects symlinks, directories, and
     missing files — and `meta.len() == exact_size`, else
     `InvalidAcquiredFile(name)`.
5. `ArtifactMember::new(member_id, path, &destination)` → `Eggup(_)` on failure;
   `.with_permissions(intent)` + `.with_integrity(Sha256(sha256))`.
6. `ArtifactSet::new(members)` → `Eggup(_)` on failure (e.g. duplicate IDs,
   empty set cannot occur here since Installable always has ≥1 artifact).

Content hashing is **not** performed here; the SHA-256 digest is attached as an
`IntegrityRequirement` for downstream verification at install time.

### `install_ids(&ManifestProjection)` (`src/lib.rs:344-349`)

Returns `(&ProductId, &ReleaseId)` for installable projections; archives fail
with `ArchiveExtractionRequired` via `installable()`.

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

### Unit tests (`src/lib.rs:351-492`, 7 tests)

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
- Authority/dependency guards: `Cargo.toml` must contain pinned rev and must not
  mention `eggpack-core`/`eggpack-contract`/`eggpack-bootstrap`; production
  source (pre-`#[cfg(test)]`) must not contain service/producer authority tokens
  (`reqwest`, `tokio`, `mirror`, `install_root`, `chown`, `signature`,
  `decompress`, `std::process`, …).

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
   crates (`eggpack-core/-contract/-bootstrap`) in `Cargo.toml`; pin rev unchanged.
2. **No authority creep**: no URL/root/ownership/permission/policy/authenticity/
   fetch/extract/decompress/service-manager code in production source; caller
   supplies requests, limits, paths, permissions.
3. **Exactness**: canonical-target lookup exact; all three maps (requests, paths,
   permissions) exact-matched; sizes exact (`meta.len() == exact_size`); bundle
   order deterministic; digest pairing bidirectional.
4. **Archive separation**: `Archive` never reaches `ArtifactSet`; `installable()`
   is the single gate for both `materialize_artifact_set` and `install_ids`.
5. **File validation**: absolute-only paths, `symlink_metadata` (no following),
   regular-file check, missing/dir/symlink/size failures map to
   `InvalidAcquiredFile(name)` without leaking paths/URLs.
6. **Limit tightening**: `max_artifact_bytes` only ever set to `Some(exact_size)`
   or rejected; timeouts/metadata bound passed through unchanged.
7. **Error hygiene**: `InvalidManifest`/`MapMismatch` carry no input/URL content;
   `Eggup(_)` wraps lower-layer diagnostics only; `Display` strings stable and
   secret-free (covered by leak assertions).
8. **Bound enforcement**: `MAX_MANIFEST_BYTES == MAX_DOCUMENT_BYTES` checked
   before UTF-8/JSON parsing; at-limit accepted, `MAX+1` rejected.
9. **Fixture fidelity**: fixtures byte-identical to pinned Eggpack rev (verify via
   `README.md` SHA-256 table); projection JSON compared with strict
   `deny_unknown_fields` structs; bundle cardinality 3, no cross-product entries.
10. **Digest handling**: `sha256_bytes()` failures → `InvalidManifest`; digests
    attached as `IntegrityRequirement::Sha256`, never treated as authenticity.
