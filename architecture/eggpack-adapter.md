# `eggup-eggpack` — Eggpack ReleaseManifest v1 adapter

Crate-level deep dive. Start at [overview.md](overview.md); authoritative
contracts live in the `plans/` documents and, for `eggup-core`, in
[`crates/eggup-core/docs/`](../crates/eggup-core/docs/domain.md). Where this
file and a contract disagree, the contract wins.

| | |
|---|---|
| Source | `crates/eggup-eggpack/src/lib.rs` (single file, 849 lines) |
| Tests | 7 inline `#[test]` + 3 integration files (56 tests) |
| Crate docs | [`README.md`](../crates/eggup-eggpack/README.md), [`CHANGELOG.md`](../crates/eggup-eggpack/CHANGELOG.md) |
| Status | Published to crates.io at `0.1.2` |
| Boundary ADR | [ADR-0004](../plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md) |

## Purpose and ownership boundary

The adapter translates one **Eggpack ReleaseManifest v1** document, for one
exact canonical target, into Eggup deployment inputs: acquisition requests, a
core `ArtifactSet`, an `eggup-archive::ArchivePlan`, and a core `InstallPlan`
with bound sources. It performs no transport, no extraction, no commit, and
no cleanup. It stops at **extraction-required evidence** — the point at which a
caller must use a qualified extractor.

[ADR-0004](../plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md)
sets the durable rule: *Eggpack owns producer release contracts, release
construction, and release evidence. Eggup owns consumer-machine deployment
mechanism. Applications own release-selection and installation policy.* The
adapter is the narrow seam ADR-0004 explicitly permits: an optional adapter
that translates a producer manifest into Eggup inputs, depending on a
manifest-only producer crate without pulling producer build or CI machinery
into runtime consumers.

### What the adapter owns

- Bounded manifest intake: document-size cap, UTF-8 check, strict schema
  validation, and hex-digest decoding.
- Exact canonical target selection. No alias resolution, no guessing.
- Translation of `install` identities into Eggup `MemberId`s, with Eggup
  identifier validation applied.
- Exact-map correspondence between manifest facts and caller-supplied maps
  (requests, acquired paths, destinations, permissions, bound members).
- Acquired-file continuity for the archive path: absolute path, regular file,
  exact size, streamed SHA-256 compared to the manifest digest.
- Construction of typed lower-layer inputs (`FetchLimits`, `ArtifactSet`,
  `ArchivePlan`, `InstallPlan`, `BoundSources`).

### What stays caller-owned, explicitly

| Caller-owned | Where the adapter stops |
|---|---|
| Release selection, version ordering, whether/when to update | `project` requires an already-chosen `canonical_target` (`lib.rs:333-335`) |
| Release-origin and fallback policy | `PlannedAcquisition.request` is the caller's `AcquisitionRequest`, passed through verbatim (`lib.rs:211`) |
| Archive artifact selection and format | `archive_format_for_name` classifies a name the caller already chose; it takes no projection argument (`lib.rs:458`) |
| Install-root / destination selection | Destinations and `destination_root` are caller parameters (`lib.rs:238-243`, `lib.rs:583-589`) |
| Replacement authorization | No helper proves control of a live destination; that is `eggup-core` ownership verification at commit |
| Ownership and privilege decisions | Not represented in any type here |
| Permissions intent | Caller supplies `PermissionsIntent` per member; the adapter only maps it |
| Trust establishment (who vouched for a digest) | No authenticity check exists; see [The verification ladder](#the-verification-ladder) |
| Extraction, persistence, deferred cleanup, commit | Documented caller order only (`lib.rs:429-450`) |

## Position in the workspace

`eggup-eggpack` is a **leaf adapter**: nothing in the workspace depends on it.
It sits above three Eggup crates plus the producer manifest schema and stops
there. It is the only crate permitted to know the producer domain
([overview.md](overview.md), boundary rules).

Runtime dependencies ([`Cargo.toml`](../crates/eggup-eggpack/Cargo.toml)):

| Dependency | Constraint | Source |
|---|---|---|
| `eggup-core` | `=0.1.2` | local path `../eggup-core` |
| `eggup-archive` | `=0.1.2` | local path `../eggup-archive` |
| `eggup-acquisition` | `=0.1.2` | local path `../eggup-acquisition` |
| `eggpack-manifest` | `=0.1.0` | crates.io registry |
| `sha2` | `0.10.9` | crates.io registry |

The three Eggup edges are exact-pinned so the published crate cannot silently
resolve a different published seam; the producer edge is registry-only so the
published crate has no VCS source requirement. Both facts are asserted as
tests, not just comments — see
[Testing approach](#testing-approach).

Dev-dependencies (`serde`, `serde_json`, `flate2`, `tar`, `zip`, `sha2`) exist
so integration tests can build real producer documents and real tar.gz/zip
bytes rather than asserting against hand-rolled fakes.

The crate is published (`publish = false` was removed in the 0.1.2 registry
promotion). `eggup-archive` and `sha2` became runtime dependencies only in
0.1.2, for the archive handoff; `eggup-core` remains Eggpack- and
archive-independent.

## Public surface

Every exported item. `file:line` refers to `crates/eggup-eggpack/src/lib.rs`
unless stated otherwise.

### Error type

| Item | Line | Role |
|---|---|---|
| `AdapterError` | `lib.rs:21` | Typed adapter failure; `#[non_exhaustive]`, `Display` at `lib.rs:40`, `std::error::Error` at `lib.rs:58`. Doc contract: details never contain URLs or file contents. |
| `::InvalidManifest` | `lib.rs:23` | Manifest or Eggup identity is invalid. |
| `::TargetNotFound` | `lib.rs:25` | Exact canonical target is absent. |
| `::MapMismatch(&'static str)` | `lib.rs:27` | A caller map differs from the required set. The `&'static str` names the map (`"acquisition request"`, `"acquired path"`, `"destinations"`, `"permissions"`, `"archive bound members"`, `"archive bound member identity"`). |
| `::CallerLimitTooSmall(String)` | `lib.rs:29` | Caller artifact cap is below the manifest exact size; carries the artifact name. |
| `::InvalidAcquiredFile(String)` | `lib.rs:31` | Acquired path is relative, missing, linked, non-regular, wrong-sized, or digest-mismatched; carries the artifact name. |
| `::ArchiveExtractionRequired` | `lib.rs:33` | The projection is archive-shaped and a direct/bundle-only helper was used (or vice versa). |
| `::UnsupportedArchiveFormat` | `lib.rs:35` | Caller-selected name does not name a supported archive format. |
| `::Eggup(String)` | `lib.rs:37` | Construction in a lower Eggup layer failed; forwards that layer's `Display`. |

### Constant

| Item | Line | Role |
|---|---|---|
| `MAX_MANIFEST_BYTES: usize` | `lib.rs:61` | Maximum accepted UTF-8 JSON document size, re-exported from Eggpack's `MAX_DOCUMENT_BYTES` (`eggpack-manifest 0.1.0` `src/lib.rs:12`, value `1_048_576`, i.e. 1 MiB). The adapter re-exports rather than re-derives the bound, so the two cannot drift. |

### Projection data types

| Item | Line | Role |
|---|---|---|
| `ArtifactRequirement` | `lib.rs:65` | One manifest artifact paired with its installed member relationship: `artifact_name`, `exact_size`, `sha256: [u8; 32]`, `member_id`, `destination`. `artifact_name` is used *only* as an exact map key, never as a path. |
| `ArchiveMemberRequirement` | `lib.rs:80` | Preserved archive member evidence: `source`, `install`, `exact_size`, `sha256: [u8; 32]`. Named "requirement" deliberately — these are declared facts, not verified extracted files. |
| `ManifestProjection` | `lib.rs:93` | Manifest projection for one exact canonical target. `Installable { product, release, target, artifacts }` (one artifact for `Direct`, N for `Bundle`) and `Archive { product, release, target, artifact_name, exact_size, sha256, members }`. `Clone + PartialEq + Eq`; no serde, no serialization, no producer type. |

### `ManifestProjection` methods

| Method | Line | Role |
|---|---|---|
| `default_destinations` | `lib.rs:147` | Pure, no I/O. `Installable`: projected `MemberId` → manifest `install`. `Archive`: `MemberId::new(install)` → `install`. Exposes producer default identity. |
| `bind_requests` | `lib.rs:165` | Binds every requirement to one caller-supplied `AcquisitionRequest` plus tightened limits; returns `Vec<PlannedAcquisition>`. Works for both variants. |
| `materialize_artifact_set_with_destinations` | `lib.rs:238` | Validates acquired files and builds one all-or-nothing `ArtifactSet` from caller-bound destinations. The single materialization implementation. |
| `materialize_artifact_set` | `lib.rs:298` | Compatibility wrapper: builds `default_destinations()` and delegates. `Installable` only. |
| `installable()` | `lib.rs:125` | **Private.** Destructures the `Installable` arm; returns `ArchiveExtractionRequired` for `Archive`. This is the routing gate. |

### Free functions and result type

| Item | Line | Role |
|---|---|---|
| `PlannedAcquisition` | `lib.rs:310` | Exact request plus manifest evidence to hand to a transport: `artifact_name`, `request`, `limits`, `exact_size`, `sha256`, `member_id`. |
| `project` | `lib.rs:326` | Validates a `ReleaseManifest` and projects one exact canonical target. No I/O. **The only public function that accepts a producer type.** |
| `project_json` | `lib.rs:409` | Bounded UTF-8 + strict-parse + project. Takes `&[u8]`, so no producer type is in its signature. Parser diagnostics are discarded. |
| `install_ids` | `lib.rs:422` | Returns `(&ProductId, &ReleaseId)` for an installable projection. |
| `archive_format_for_name` | `lib.rs:458` | Maps a caller-selected artifact file name to `ArchiveFormat` (`.tar.gz`/`.tgz` → `TarGz`, `.zip` → `Zip`). No I/O, no projection argument. |
| `validate_acquired_archive` | `lib.rs:481` | Continuity gate for a caller-acquired archive file: absolute, regular, exact size, streamed SHA-256 equal to the manifest digest. The only place the adapter hashes bytes itself. |
| `archive_plan_for` | `lib.rs:534` | Translates the archive member list into a validated `eggup_archive::ArchivePlan` with per-member expected size and digest. No I/O; does not re-validate the archive. |
| `core_plan_for_archive_with_destinations` | `lib.rs:583` | Builds the core `InstallPlan` with caller-bound destinations, recording advisory paths as `extraction_root.join(manifest install)`. Bound sources are still empty. |
| `core_plan_for_archive` | `lib.rs:650` | Compatibility wrapper: builds `default_destinations()` and delegates. The single archive plan implementation. |
| `bind_archive_members` | `lib.rs:681` | Pairs `BoundMember` handles to declared members by declaration order **with an identity cross-check**, returning `BoundSources`. |

## The projection boundary — the review focus

This is the section a reviewer should read first. The claim to verify is:
**no `eggpack_manifest` type appears in any public API other than
`project(&ReleaseManifest, …)`, and none appears past the projection boundary.**

### Why project instead of pass-through

`eggpack_manifest` is a *published producer schema*. If its types were passed
into `eggup-core` inputs directly, three things would follow, all undesirable:

1. Every downstream signature would carry a producer dependency, so a change to
   the producer schema becomes a breaking change to the deployment surface —
   and `eggup-core` would acquire an Eggpack edge, which ADR-0004 forbids.
2. Producer semantics would leak into deployment decisions. `ByteEvidence`
   with a *hex string* digest is a document representation; `IntegrityRequirement::Sha256([u8; 32])`
   is a deployment decision. Passing the former through would push a decode
   failure to commit time, deep inside the mutation core.
3. Producer *shape* would become load-bearing. `ArtifactForm::Bundle` entries
   versus `ManifestProjection::Installable.artifacts` are different groupings
   with the same deployment meaning; downstream code should not be forced to
   re-derive that equivalence.

Projection collapses all of this at one function. `project` is the whole
boundary.

### The translation

```text
eggpack_manifest 0.1.0   (registry; producer authority — untrusted input)
  ReleaseManifest { schema_version, product_id, release_id, targets[] }
  TargetRecord    { target: String, form: ArtifactForm }
  ArtifactForm    = Direct { artifact, install }
                  | Bundle { entries: [BundleRecord{artifact, install}] }
                  | Archive { artifact, members: [ArchiveMemberRecord{source, install, bytes}] }
  ArtifactRecord  { name: String, size: u64, sha256: String(hex) }
  ByteEvidence    { size: u64, sha256: String(hex) }
  constants       SCHEMA_V1 = 1, MAX_DOCUMENT_BYTES = 1_048_576
        |
        |  project(&ReleaseManifest, canonical_target)          lib.rs:326
        |    manifest.validate()                    -> InvalidManifest      :330
        |    manifest.target(canonical)  EXACT only -> TargetNotFound      :333
        |    ProductId::new / ReleaseId::new        -> Eggup(String)       :336,:338
        |    match &target.form { Direct | Bundle | Archive }               :341
        |      sha256_bytes(): hex -> [u8; 32]     -> InvalidManifest      :349,:368,:384,:395
        |      MemberId::new(install)               -> Eggup(String)       :352,:371
        v
ManifestProjection                                   lib.rs:93   (adapter-owned type)
  Installable { ProductId, ReleaseId, String, Vec<ArtifactRequirement> }
  Archive    { ProductId, ReleaseId, String, String, u64, [u8;32],
               Vec<ArchiveMemberRequirement> }
        |
        |  no producer type below this line
        +--> bind_requests                    -> Vec<PlannedAcquisition>  -> eggup-acquisition
        +--> materialize_artifact_set*        -> eggup_core::ArtifactSet
        +--> validate_acquired_archive        -> (hashes bytes here)       --+
        +--> archive_plan_for                 -> eggup_archive::ArchivePlan   |  eggup-archive
        +--> core_plan_for_archive*           -> eggup_core::InstallPlan      |
        +--> bind_archive_members             -> eggup_core::BoundSources   --+
```

Three translation facts worth checking during review:

- **Hex digests are decoded inside `project`, not downstream.** Every
  `sha256_bytes()` call site (`lib.rs:349`, `:368`, `:384`, `:395`) maps failure
  to `InvalidManifest`. By the time a `ManifestProjection` exists, all digests
  are `[u8; 32]` values that Eggup can compare. A projection can never carry an
  undecodable or half-validated digest.
- **Producer strings become Eggup-validated identities.** `ProductId::new`,
  `ReleaseId::new`, `MemberId::new` all run during projection, so a manifest
  whose identifiers Eggup would reject cannot produce a projection at all.
- **`Direct` and `Bundle` converge.** Both arms build
  `ManifestProjection::Installable` (`lib.rs:342-356`, `:357-377`), with
  `Direct` producing a one-element `artifacts` vector. The deployment layer
  therefore has one direct path, not two.

### Evidence that nothing escapes

Checked against the source, not inferred:

| Check | Result |
|---|---|
| `eggpack_manifest` imports in production code | Exactly one `use` statement, `lib.rs:5`: `{ArtifactForm, ReleaseForm…}` → `{ArtifactForm, ReleaseManifest, MAX_DOCUMENT_BYTES}` |
| `ReleaseManifest` in a public signature | Exactly one: `project(manifest: &ReleaseManifest, …)`, `lib.rs:327` |
| `ReleaseManifest` constructed internally | `lib.rs:417` inside `project_json`, never re-exported |
| Producer types in `ManifestProjection`'s definition | None — every field is `String`, `u64`, `[u8; 32]`, or an Eggup `ProductId`/`ReleaseId`/`MemberId` |
| Producer types in `ArtifactRequirement` / `ArchiveMemberRequirement` | None; all fields are primitives or `MemberId` |
| Producer types in `PlannedAcquisition` | None; fields are `String`, `AcquisitionRequest`, `FetchLimits`, `u64`, `[u8; 32]`, `MemberId` |
| Producer types in any archive helper | None; arguments and returns are `ManifestProjection`, `ArchiveFormat`, `Path`, `ArchiveLimits`, `ArchivePlan`, `InstallPlan`, `BoundSources` |

Two tests automate the parts that are easy to regress, and are worth reading
as the enforcement point:

- `adapter_pins_registry_eggpack_manifest_and_no_producer_crates`
  ([`tests/interoperability.rs:972`](../crates/eggup-eggpack/tests/interoperability.rs))
  asserts the manifest contains `eggpack-manifest = "=0.1.0"` and that it
  contains no `git =`, `rev =`, `branch =`, or `tag =` edge; asserts all three
  Eggup edges stay `=0.1.2`; and asserts `eggpack-core`, `eggpack-contract`,
  and `eggpack-bootstrap` appear nowhere.
- `adapter_source_claims_no_producer_or_service_authority`
  ([`tests/interoperability.rs:1019`](../crates/eggup-eggpack/tests/interoperability.rs))
  splits `src/lib.rs` at `#[cfg(test)]` and asserts 24 authority tokens are
  absent from production code: `api.github.com`, `reqwest`, `hyper`, `tokio`,
  `mirror`, `fallback`, `latest`, `base_url`, `base-origin`, `install_root`,
  `installation_root`, `ownership`, `chown`, `chmod`, `sudo`, `elevation`,
  `signature`, `authenticity`, `systemd`, `launchd`, `windows-service`,
  `decompress`, `flate2`, `std::process`.

The token list is a textual proxy, and the test says so: absence of the word
`authenticity` is not proof of the absence of an authenticity claim, and the
closure record also carries `cargo tree` and manual review as evidence. Note
that `decompress` and `flate2` being forbidden is a concrete expression of the
extraction boundary — the adapter must not decompress anything, only describe
what should be extracted.

One residual coupling worth naming: `project` accepts `&ReleaseManifest`, so a
caller holding only bytes must go through `project_json`, and a caller holding
a parsed manifest must depend on `eggpack-manifest` itself. That is
intentional and bounded — the dependency is a leaf, published, exactly pinned
one, and it is the only producer edge in the workspace.

## Request binding

`bind_requests` (`lib.rs:165`) turns each manifest artifact into exactly one
`PlannedAcquisition`. It is the only place the adapter produces transport
inputs.

Order of operations:

1. `baseline.validate()` (`lib.rs:170-172`) — the caller's `FetchLimits` must
   be self-consistent before anything else. Failure → `Eggup`. (`FetchLimits`
   fields are public for 0.1.x source compatibility, so each transport
   re-validates at its own I/O boundary; see
   [`acquisition.md`](acquisition.md).)
2. Build the requirement list from the projection (`lib.rs:173-197`).
   `Installable` contributes one requirement per `ArtifactRequirement`;
   `Archive` contributes exactly one requirement for the archive artifact
   itself.
3. Exact-map check (`lib.rs:198-202`): `requests.len() == requirements.len()`
   **and** every required name is present. Both missing and extra requests
   fail, with the same `MapMismatch("acquisition request")`.
4. Per-requirement clamp and emit.

### The clamp

```rust
if baseline.max_artifact_bytes < size {
    return Err(AdapterError::CallerLimitTooSmall(name.clone()));
}
Ok(PlannedAcquisition {
    artifact_name: name.clone(),
    request: requests.get(&name).expect("checked map").clone(),
    limits: FetchLimits {
        max_artifact_bytes: size,   // <- manifest exact size
        ..baseline                  // <- everything else from the caller
    },
    exact_size: size,
    sha256: sha,
    member_id: member,
})
```

`lib.rs:206-219`.

The effective artifact cap handed downstream is `min(caller cap, manifest
exact size)`, and the `CallerLimitTooSmall` guard is what keeps that a
*tightening* rather than a widening: if the caller's cap is already below the
declared size, the adapter refuses rather than silently raising the cap to
match. The manifest size is non-zero by producer validation, and
`FetchLimits::validate` rejects a zero cap, so the emitted cap is always
positive and always finite.

`..baseline` propagates `max_metadata_bytes`, `connect_timeout`, and
`total_timeout` **unchanged**. The adapter is narrowing one field and
inheriting the rest; it does not participate in the
`min(request, adapter)` timeout-ceiling resolution, which belongs to
[`acquisition.md`](acquisition.md) and the transport adapters. Test
`negative_07_timeouts_and_metadata_bounds_are_never_widened`
([`tests/interoperability.rs:691`](../crates/eggup-eggpack/tests/interoperability.rs))
pins all four fields.

### The URL is the caller's, untouched

`request` is cloned verbatim from the caller's map. The adapter never
constructs, rewrites, normalizes, or ranks a URL, and never reads a
release-origin hint out of the manifest. Unit test
`exact_request_binding_preserves_url_and_tightens_byte_limit`
(`lib.rs:797`) binds a request whose URL embeds credentials and a query token
and asserts field equality afterwards, which is the concrete statement that
URL policy is the caller's.

### `PlannedAcquisition.member_id` means two different things

For `Installable`, `member_id` is the installed member identity
(`MemberId::new(install)`). For `Archive`, `bind_requests` sets it to
`MemberId::new(artifact_name)` (`lib.rs:194`) — the *archive file* identity, not
any archive member identity. Archive member identities are derived later, in
`core_plan_for_archive_with_destinations` (`lib.rs:608`) and
`bind_archive_members` (`lib.rs:701`), from `MemberId::new(m.install)`.

This divergence is not exploitable, because `materialize_artifact_set*` is
gated to `Installable` and returns `ArchiveExtractionRequired` for an archive
projection before any map is read. But a caller that treats
`PlannedAcquisition` uniformly across both variants will hold a member id that
belongs to no other structure in the archive flow. Flagging it as a review
note, not a defect.

## Materialization

`materialize_artifact_set_with_destinations` (`lib.rs:238`) is the only place
an `ArtifactSet` is constructed. `materialize_artifact_set` (`lib.rs:298`) is
a four-line wrapper over it; there is exactly one implementation.

Order of operations, and what each step actually proves:

| Step | Line | Check | Proves |
|---|---|---|---|
| 1 | `lib.rs:244` | `installable()` | The projection is direct/bundle-shaped. Archive → `ArchiveExtractionRequired`, before any map is read. |
| 2 | `lib.rs:245-251` | `acquired` map length + per-key containment | Exactly the required artifact names, no missing, no extra → `MapMismatch("acquired path")`. |
| 3 | `lib.rs:252-258` | `destinations` map length + per-key containment | Exactly one destination per `MemberId` → `MapMismatch("destinations")`. |
| 4 | `lib.rs:259-263` | `permissions` map length + per-key containment | Exactly one `PermissionsIntent` per `MemberId` → `MapMismatch("permissions")`. |
| 5 | `lib.rs:267-269` | `path.is_absolute()` | The acquired path is absolute, not a working-directory-relative resolution. |
| 6 | `lib.rs:270-271` | `fs::symlink_metadata(path)` | `symlink_metadata` does **not** follow symlinks, so a symlink is observed as a symlink. Missing path fails here. |
| 7 | `lib.rs:272-274` | `meta.file_type().is_file()` and `meta.len() == exact_size` | A regular file (not a symlink, directory, FIFO, or device) of exactly the declared byte count. All failures → `InvalidAcquiredFile`. |
| 8 | `lib.rs:277-279` | `ArtifactMember::new(member_id, path, destination)` | Eggup destination validation — relative, contained, non-colliding. Failure → `Eggup`. |
| 9 | `lib.rs:280` | `IntegrityRequirement::Sha256(r.sha256)` | **Attaches** the expected digest. No hashing occurs here. |
| 10 | `lib.rs:283-288` | `HashSet` over `member.destination()` | No two members resolve to the same destination → `MapMismatch("destinations")`. |
| 11 | `lib.rs:289` | `ArtifactSet::new(members)` | Non-empty and no duplicate member identity. Failure → `Eggup`. |

### Verified here versus merely reported

This is the distinction a reviewer most often gets wrong:

- **Verified in the adapter:** absolute path, regular-file kind (symlink-safe),
  and exact byte size (steps 5-7).
- **Attached, not verified:** the SHA-256 digest. `materialize_artifact_set*`
  never opens the file for reading. It records the manifest digest as an
  `IntegrityRequirement`, and `eggup-core` compares it during
  `verify_integrity`. The adapter is a translator here, not a verifier.
- **Verified in the adapter, on the archive path only:** `validate_acquired_archive`
  does hash — streamed, 16 KiB chunks (`lib.rs:506-520`) — because the archive
  must be proven before *any* extraction happens. See
  [Archives: the extraction-required path](#archives-the-extraction-required-path).

The diagnostic is `InvalidAcquiredFile`, which is slightly broader than its
name: it also covers digest mismatch and wrong size, not only "invalid file"
shape. Tests cover the shapes individually:
`negative_12_relative_acquired_path_is_rejected`,
`negative_13_symlink_acquired_path_is_rejected`,
`negative_14_directory_acquired_path_is_rejected`,
`negative_15_missing_local_file_is_rejected`, and
`negative_16_wrong_exact_size_is_rejected`.

Note the three map checks (steps 2-4) are *length plus containment*, not set
equality by iteration. That is sufficient: equal length plus every required
key present implies no extra keys, since keys are unique within a `HashMap`.

## Destination binding

### `default_destinations` (`lib.rs:147`)

A pure helper with no I/O. `Installable` projects `member_id → destination`
(where `destination` is the manifest `install` value). `Archive` produces
`MemberId::new(install) → install` for each declared member.

### Why the caller owns destination selection

A manifest `install` value is **producer default identity**, not an
authorization. Nothing in a producer's document proves that the machine
reading it has any relationship to the path named by `install`, that the path
is not somebody else's file, or that overwriting it is intended. If the
adapter adopted `install` as the destination by default, then a manifest —
an untrusted input document — would select filesystem targets on the consumer
machine. That is release-and-install policy leaking into deployment
mechanism, which [ADR-0004](../plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md)
assigns to the application.

The API is shaped so the *safe* choice is the explicit one:

| Function | Destination source | Role |
|---|---|---|
| `materialize_artifact_set_with_destinations` | caller `HashMap<MemberId, String>` | The real implementation |
| `materialize_artifact_set` | `default_destinations()` | Compatibility wrapper preserving prior behavior |
| `core_plan_for_archive_with_destinations` | caller `HashMap<MemberId, String>` | The real implementation |
| `core_plan_for_archive` | `default_destinations()` | Compatibility wrapper preserving prior behavior |

The wrappers exist to avoid breaking callers that predate the destination-map
API; both delegate, so there is no second code path to audit. The README
frames the use case the explicit form exists for: a consumer updating the exact
running executable in place, including under a renamed basename, needs a
destination that differs from the manifest default.

### What the adapter does and does not decide

**Decides:** that the caller's destination map must be exactly the projected
member set, in count and in keys (`lib.rs:252-258`, `:599-604`); that no two
resolved destinations collide (`lib.rs:283-288`, `:623-628`); that each
destination string passes Eggup's own destination validation
(`ArtifactMember::new` at `lib.rs:277` and `:617`, failure → `Eggup`).

**Does not decide:** the destination root. `core_plan_for_archive_with_destinations`
takes `destination_root: &Path` from the caller and forwards it to
`InstallPlan::new` (`lib.rs:630`); Eggup validates the root. The adapter
performs no filesystem mutation, creates no parent directories, and never
inspects a live destination.

`default_destinations` is the one helper that reaches `MemberId::new` on
caller-visible data for the archive variant (`lib.rs:156`); an invalid
identity surfaces as `Eggup`.

## Archives: the extraction-required path

`eggup-archive` owns the extraction; the adapter only *describes* it. See
[archive-extraction.md](archive-extraction.md) for the extractor itself.

The required caller order is documented in a header comment at
`lib.rs:429-450` and enforced only by that documentation and by the integration
tests — no type signature forces the sequence.

```text
caller-selected archive artifact
  |
  1. validate_acquired_archive(&projection, &path)        lib.rs:481
  |     projection must be Archive, else ArchiveExtractionRequired
  |     absolute? -> symlink_metadata -> is_file? -> len == exact_size
  |     streamed SHA-256 == manifest archive digest
  |     => no unverified byte is ever handed to an extractor
  |
  2. archive_format_for_name(caller_name)                  lib.rs:458
  |     ".tar.gz" | ".tgz" -> TarGz ; ".zip" -> Zip ; else UnsupportedArchiveFormat
  |     no projection argument: classifies the name the caller already chose
  |
  3. archive_plan_for(&projection, format, &path, limits) lib.rs:534
  |     per member: ArchiveMember::new(source, install, Some(size), Some(sha256))
  |     => eggup_archive::ArchivePlan   [no I/O, no re-validation of the archive]
  |
  4. eggup_archive::extract(&plan)          <-- outside this crate
  |     allowlist, budgets, private root, streamed member evidence
  |
  5. captured_root = extracted.root()       <-- capture BEFORE persisting
  |
  6. core_plan_for_archive[_with_destinations](
  |        &projection, &captured_root, &destination_root, destinations, permissions)
  |     member id     = MemberId::new(manifest install)          lib.rs:608
  |     advisory path = captured_root.join(manifest install)     lib.rs:612
  |     integrity     = IntegrityRequirement::Sha256(m.sha256)  lib.rs:620
  |     => eggup_core::InstallPlan  with EMPTY BoundSources
  |
  7. extracted.persist().into_bound_sources()?.into_members()  -> Vec<BoundMember>
  |
  8. bind_archive_members(&projection, bound_members)          lib.rs:681
  |     count equality                       -> MapMismatch("archive bound members")
  |     per-pair identity cross-check        -> MapMismatch("archive bound member identity")
  |     => eggup_core::BoundSources keyed by MemberId::new(install)
  |
  9. InstallPlan::prepare_with_bound_sources(plan, sources)
```

### Why `core_plan_for_archive*` sits at step 6, before persistence

`ArtifactMember::new` requires a source path, and at step 6 the extracted
bytes do not yet have an open handle bound to them. The adapter therefore
records `extraction_root.join(manifest install)` as an **advisory** path
(`lib.rs:612`). The doc at `lib.rs:576-578` states the consequence precisely:
advisory paths are never opened during staging when a bound handle is
supplied; the string exists to satisfy `ArtifactMember::new`'s validation and
to support diagnostics. `eggup-core` documents the same rule independently
(`domain.rs:415-416`: the recorded source path is advisory diagnostics only
once a handle is supplied).

This is why the caller must capture `ExtractedArchive::root()`
(`eggup-archive` `lib.rs:328`) before `persist()` consumes the guard. After
persistence the caller holds a `PersistedExtraction`, and the adapter's
advisory path must still be meaningful for as long as the plan exists.

Test `advisory_member_entry_replacement_does_not_redirect_staged_bytes`
([`tests/archive_handoff.rs:586`](../crates/eggup-eggpack/tests/archive_handoff.rs))
exercises the exact attack this design forecloses: replacing the file at the
advisory path after extraction must not change which bytes get staged, because
staging reads the bound handle.

### The identity cross-check in `bind_archive_members`

`bind_archive_members` (`lib.rs:681`) pairs `BoundMember` handles to declared
members by declaration order, then verifies each pair's own recorded identity:

```rust
if bound.source_path() != declared.source || bound.output_name() != declared.install {
    return Err(AdapterError::MapMismatch("archive bound member identity"));
}
```

`lib.rs:698-700`. The rationale is in the doc comment at `lib.rs:675-680` and
in the `Unreleased` [CHANGELOG](../crates/eggup-eggpack/CHANGELOG.md) entry:
count-only pairing would let a `BoundExtraction` produced from a *different*
`ArchivePlan`, or reused after a re-projection that reordered members, commit
each member's bytes under another member's declared identity. That
substitution is digest-invisible precisely when two declared members share
identical content — the case where it matters most.

`archive_plan_for` is what makes the check sound: it sets
`source_path = declared.source` and `output_name = declared.install`
(`lib.rs:546-551`), so a correctly-produced extraction always carries exactly
the identities the projection declares. Tests:
`bind_archive_members_cross_checks_bound_identity_not_just_count`
(`archive_handoff.rs:482`) and
`bind_archive_members_rejects_an_extraction_of_a_different_projection`
(`archive_handoff.rs:554`).

### Two caller obligations the types do not enforce

Not defects — documented obligations with no compile-time enforcement, and the
two places most likely to be got wrong in review:

1. `validate_acquired_archive` is not required by any signature.
   `archive_plan_for` accepts no proof that it ran, and performs no I/O
   (`lib.rs:531-533`). A caller who skips it gets a valid `ArchivePlan` for
   unverified bytes; the failure then surfaces inside `eggup-archive::extract`
   as a per-member digest or size mismatch, which is later but not equivalent.
2. `archive_format_for_name` takes a name, not the projection
   (`lib.rs:458`), so the returned `ArchiveFormat` is never cross-checked
   against `projection.artifact_name`. A caller can classify one name and
   extract a different artifact's bytes. This is not a security boundary —
   step 1 already proved the bytes by digest, and `format` only selects the
   parser — but a mismatch is a caller's confusion to diagnose, and the
   adapter offers no help.

## Direct vs archive-required

Routing is decided in exactly one place: the `match &target.form` inside
`project` (`lib.rs:341-402`). It is a **producer-shape** decision, not a
capability or environment decision. The adapter never sniffs bytes, never
inspects a local filesystem to choose a path, and never upgrades or downgrades
a form.

| Input condition | Projection | Direct path | Archive path | Failure raised |
|---|---|---|---|---|
| `ArtifactForm::Direct { artifact, install }` | `Installable`, 1 artifact | `bind_requests` → 1 `PlannedAcquisition`; `materialize_artifact_set*` → `ArtifactSet` of 1; `install_ids` → ids | All archive helpers reject | — |
| `ArtifactForm::Bundle { entries }` (N entries) | `Installable`, N artifacts | `bind_requests` → N `PlannedAcquisition`; `materialize_artifact_set*` → `ArtifactSet` of N | All archive helpers reject | — |
| `ArtifactForm::Archive { artifact, members }` | `Archive` | `materialize_artifact_set*` and `install_ids` reject; `bind_requests` still works (1 request for the archive file) | `validate_acquired_archive` → `archive_plan_for` → `core_plan_for_archive*` → `bind_archive_members` | `ArchiveExtractionRequired` on the direct path |
| Exact canonical target absent | none | — | — | `TargetNotFound` |
| Alias string used instead of a canonical triple | none | — | — | `TargetNotFound` — no alias resolution |
| `schema_version != 1`, or any structural violation | none | — | — | `InvalidManifest` |

The asymmetry is deliberate and is the shape of the extraction boundary: the
direct helpers refuse archive projections (`lib.rs:135`, reached from `:244`
and `:425`), and the archive helpers refuse installable projections
(`lib.rs:542`, `:597`, `:687`). `bind_requests` and `default_destinations`
serve both variants, because an archive still needs exactly one acquisition
request for the archive file.

The routing gate is a private method, `installable()` (`lib.rs:125`), so the
rule is stated once rather than re-derived per helper.

## Invariants and failure modes

Every `AdapterError` variant, what triggers it, and where.

| Condition | Observable outcome | Enforced at |
|---|---|---|
| `manifest.validate()` fails — schema version, bounds, duplicate names, structural invariants | `InvalidManifest` | `lib.rs:330-332` |
| JSON input exceeds `MAX_MANIFEST_BYTES` (1 MiB) | `InvalidManifest` | `lib.rs:413-415` |
| Input is not valid UTF-8 | `InvalidManifest` | `lib.rs:416` |
| `ReleaseManifest::from_json` parse or schema failure | `InvalidManifest`; producer diagnostic discarded | `lib.rs:417` |
| Hex digest fails to decode (artifact or member) | `InvalidManifest` | `lib.rs:349`, `:368`, `:384`, `:395` |
| Exact canonical target absent, or an alias is used | `TargetNotFound` | `lib.rs:333-335` |
| Request map missing or has extra entries | `MapMismatch("acquisition request")` | `lib.rs:198-202` |
| Acquired-path map missing or has extra entries | `MapMismatch("acquired path")` | `lib.rs:245-251` |
| Destination map missing or has extra entries | `MapMismatch("destinations")` | `lib.rs:252-258`, `:599-604` |
| Permission map missing or has extra entries | `MapMismatch("permissions")` | `lib.rs:259-263`, `:602-604` |
| Two members resolve to the same destination | `MapMismatch("destinations")` | `lib.rs:283-288`, `:623-628` |
| Bound member count ≠ declared member count | `MapMismatch("archive bound members")` | `lib.rs:689-691` |
| Bound member `source_path`/`output_name` ≠ declared identity | `MapMismatch("archive bound member identity")` | `lib.rs:698-700` |
| Caller `max_artifact_bytes` < manifest exact size | `CallerLimitTooSmall(name)` | `lib.rs:206-208` |
| Acquired path is relative | `InvalidAcquiredFile(name)` | `lib.rs:267-269`, `:496-498` |
| Acquired path missing (`symlink_metadata` errors) | `InvalidAcquiredFile(name)` | `lib.rs:270-271`, `:499-500`, `:504-505`, `:509-511` |
| Acquired path is a symlink, directory, or other non-regular | `InvalidAcquiredFile(name)` | `lib.rs:272-274`, `:501-503` |
| Acquired file size ≠ declared exact size | `InvalidAcquiredFile(name)` | `lib.rs:272-274`, `:501-503` |
| Acquired archive SHA-256 ≠ manifest digest | `InvalidAcquiredFile(artifact_name)` | `lib.rs:518-520` |
| Installable-only helper used on an `Archive` projection | `ArchiveExtractionRequired` | `lib.rs:135` (from `:244`, `:425`) |
| Archive-only helper used on an `Installable` projection | `ArchiveExtractionRequired` | `lib.rs:494`, `:542`, `:597`, `:687` |
| Name is empty, non-ASCII, or not `.tar.gz`/`.tgz`/`.zip` | `UnsupportedArchiveFormat` | `lib.rs:459-469` |
| `FetchLimits::validate` fails on the baseline | `Eggup(detail)` | `lib.rs:170-172` |
| `MemberId::new` / `ProductId::new` / `ReleaseId::new` rejects an identifier | `Eggup(detail)` | `lib.rs:156`, `:195`, `:337`, `:339`, `:353`, `:372`, `:608`, `:701` |
| `ArtifactMember::new` rejects a destination | `Eggup(detail)` | `lib.rs:277-278`, `:617-618` |
| `ArtifactSet::new` rejects empty input or duplicate identity | `Eggup(detail)` | `lib.rs:289`, `:629` |
| `InstallPlan::new` rejects the root or the set | `Eggup(detail)` | `lib.rs:630-631` |
| `ArchiveMember::new` or `ArchivePlan::new` rejects a declaration or limits | `Eggup(detail)` | `lib.rs:552`, `:556` |

Two properties of this table that a reviewer should hold onto:

- **Fail-closed, not fail-open, and structurally.** Every mismatch between
  caller-supplied maps and manifest-derived requirements is a distinct named
  `MapMismatch`, and every check precedes any construction. There is no
  partial success: a bundle with two of three artifacts acquired produces an
  error, not a two-member set.
- **`Eggup(String)` is the one variant that forwards a lower-layer message.**
  Its content is bounded to Eggup's identifier and destination validation text
  — not URLs, not file contents, not manifest input. Test
  `destination_diagnostics_carry_no_credential_or_content_material`
  ([`tests/caller_destinations.rs:625`](../crates/eggup-eggpack/tests/caller_destinations.rs))
  asserts this for destinations; the projection-side equivalent is
  `bounded_json_projection_rejects_invalid_documents_without_leaking_input`
  (`lib.rs:734`), which feeds a document containing `https://user:super-secret@example.invalid/?token=hidden`
  and asserts the formatted `Display` + `Debug` output contains none of
  `private`, `super-secret`, or `token=hidden`.

## The verification ladder

The adapter sits on the **integrity rung only**
([ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md),
[overview.md](overview.md)). It never answers "who vouched for that digest?".

| Operation | What it does | Who performs the comparison |
|---|---|---|
| `validate_acquired_archive` | Hashes the acquired archive file, streamed in 16 KiB chunks | **The adapter**, against the manifest-declared digest (`lib.rs:506-520`) |
| `materialize_artifact_set*` | Attaches `IntegrityRequirement::Sha256` | `eggup-core`, at `verify_integrity` |
| `archive_plan_for` | Passes `Some(size)` and `Some(sha256)` per member | `eggup-archive`, during extraction |
| `bind_archive_members` | Compares identities, not digests | Identity only; no hash involved |

The rule to state plainly when reviewing this crate: **comparing a digest to a
manifest is an integrity check, not a trust decision.** It proves the acquired
bytes match a claim made in the document. It says nothing about whether the
document's author is who the caller believes it is. A manifest is untrusted
input unless the caller established otherwise — through whatever channel
produced the bytes, and whatever policy that channel implements.

That is why the crate's own boundary test forbids the words `signature` and
`authenticity` in production source
([`tests/interoperability.rs:1044`](../crates/eggup-eggpack/tests/interoperability.rs)).
The check is textual, and the test says it supplements rather than replaces
review, but it expresses the intended claim boundary: this crate has no
vocabulary for trust because it has no trust decisions to make.

Consequences a reviewer should confirm hold:

- Digest comparison uses `!=` on `[u8; 32]` (`lib.rs:518`), not a
  constant-time comparison. For a public hash of a public artifact, the timing
  profile carries no information the caller does not already have.
- The adapter never re-hashes a direct/bundle artifact. It is not a
  verification layer; do not let a future change make it one by adding hashing
  to `materialize_artifact_set*`, which would duplicate `eggup-core`'s job and
  blur which layer is authoritative.
- `eggup-archive` performs the same kind of work for members it extracts, and
  for the same reason: streamed SHA-256/size evidence so a caller can bind an
  extraction to a digest the producer published. Neither crate claims
  authenticity.

## Testing approach

63 tests total: 7 inline in `src/lib.rs:711-849`, 56 across three integration
files. The 3:1 ratio is the right shape for a conformance adapter — the inline
tests check the adapter's own arithmetic in isolation, while the integration
tests check composition against the *real* producer schema and the *real*
lower crates, which is where a translation bug actually lives.

| File | Lines | Tests | What it establishes |
|---|---|---|---|
| `src/lib.rs` inline | 839 | 7 | Boundary at exactly `MAX_MANIFEST_BYTES`; invalid-input redaction; exact-target selection and alias rejection; corrected-fixture projection across all three forms; request binding and cap tightening; size and permission gate on materialization. |
| `tests/interoperability.rs` | 1058 | 28 | Producer-schema conformance plus a numbered rejection matrix. |
| `tests/archive_handoff.rs` | 663 | 13 | The full eight-step archive chain against real `tar.gz` and `zip` bytes. |
| `tests/caller_destinations.rs` | 666 | 15 | Destination binding, defaults, wrapper equivalence, and failure closure. |

### `interoperability.rs`

Six positive tests compare each form's projection against a strictly-parsed
`#[serde(deny_unknown_fields)]` fixture (`direct_positive_matches_projection_fixture`,
`bundle_positive_matches_corrected_three_member_fixture`,
`archive_positive_preserves_facts_and_blocks_materialization`, and three
binding/materialization tests). Twenty numbered negative tests,
`negative_01_alias_target_is_rejected` through
`negative_20_crossed_bundle_relationship_fails_comparison`, each pin one
rejection rule by number, so a failure names the rule it broke. Two further
tests are the dependency- and authority-boundary checks described in
[The projection boundary](#the-projection-boundary--the-review-focus).

### `archive_handoff.rs`

Builds genuine archives with the `tar`, `zip`, and `flate2` dev-dependencies
rather than mocking, then runs the whole chain through
`eggup_archive::extract` and `eggup-core` prepare. The bulk of the file is
adversarial: crossed member contents, count mismatch, identity cross-check, an
extraction produced from a *different* projection, advisory-path entry
replacement, missing declared member, and wrong member digest. The identity
cases are the ones that would otherwise be digest-invisible.

### `caller_destinations.rs`

Fifteen tests covering defaults, wrapper-equivalence with prior behavior,
per-member destination independence for bundles, missing / extra / colliding
destination keys, rejection of invalid destinations by Eggup's own validation,
independence of the permission and acquired maps, archive-projection
destination failures, and diagnostic sanitization.

### Focused commands

```sh
cargo test -p eggup-eggpack
cargo test -p eggup-eggpack --test interoperability
cargo test -p eggup-eggpack --test archive_handoff
cargo test -p eggup-eggpack --test caller_destinations
cargo test -p eggup-eggpack <test_name>
cargo clippy -p eggup-eggpack --all-targets --locked -- -D warnings
```

Full workspace gate: [`scripts/check-local.sh`](../scripts/check-local.sh).
See [tooling-governance.md](tooling-governance.md).

## Cross-references

| Topic | Where |
|---|---|
| Workspace index, module map, boundary rules | [overview.md](overview.md) |
| Producer / consumer ownership rule | [ADR-0004](../plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md) |
| Verification layers, transport neutrality | [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) |
| Core state machine, ownership, commit | [core-transaction.md](core-transaction.md) |
| `FetchLimits`, `AcquisitionRequest`, transport seam, composition policy | [acquisition.md](acquisition.md) |
| Bounded allowlisted extraction, `ArchivePlan`, `BoundMember`, deferred cleanup | [archive-extraction.md](archive-extraction.md) |
| Service lifecycle composition (downstream, optional) | [service-lifecycle.md](service-lifecycle.md) |
| Lints, CI matrix, planning and closure process | [tooling-governance.md](tooling-governance.md) |
| Closure records: `001`, `001a`, `002`, `002a`, `003`, `003a`, `004`, `004a` | [`plans/closure/eggpack-manifest-interoperability/`](../plans/closure/eggpack-manifest-interoperability/) |
| Planning registry | [plans/registry.md](../plans/registry.md) |
| Crate README / CHANGELOG | [`crates/eggup-eggpack/`](../crates/eggup-eggpack/) |

**Published-crate status.** `eggup-eggpack` is published to crates.io at
`0.1.2` (`docs.rs/eggup-eggpack`). The published `src/lib.rs` is recorded in
the 0.1.2 CHANGELOG entry as byte-identical to the previously pinned revision
`678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, so the registry promotion is
byte-faithful rather than merely version-compatible. The `Unreleased`
CHANGELOG entry records the `bind_archive_members` identity cross-check as a
security fix with no API change. `AdapterError` is `#[non_exhaustive]`, so
adding variants remains source-compatible for downstream `match` callers.

## Known doc/code drift

Checked while reading; no drift found in the current tree. Specifically
verified against the code rather than assumed:

- The README's claim that `materialize_artifact_set` and `core_plan_for_archive`
  are compatibility wrappers over caller-bound variants is accurate
  (`lib.rs:292-305`, `:634-664`); there is one implementation of each.
- The 0.1.2 CHANGELOG's claim that `materialize_artifact_set` still returns
  `ArchiveExtractionRequired` for archive projections is accurate
  (`lib.rs:135` via `:244`; test `negative_17`).
- `overview.md`'s `849 / 63` figure is accurate: 849 source lines, 7 inline
  plus 28 + 13 + 15 integration tests.
- `MAX_MANIFEST_BYTES` is the producer's bound re-exported, not a
  separately-chosen value.

Two items that read as imprecision but are not drift, recorded here so a
future reviewer does not re-raise them:

- `AdapterError::InvalidAcquiredFile` covers digest mismatch and wrong size,
  which is broader than its name suggests (`lib.rs:272-274`, `:518-520`).
- `PlannedAcquisition.member_id` denotes an archive *file* identity for
  `Archive` projections and an *installed member* identity for `Installable`
  ones (`lib.rs:194` versus `:352`/`:371`). Not reachable across the routing
  gate, but not a single consistent meaning either.
