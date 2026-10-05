# Changelog

## Unreleased

Nothing in this section has been published. `0.1.2` remains the published
baseline on crates.io, and no publication milestone authorizes these entries.
No consumer migration is required — no public API changed, and correct
declaration-order pairing is unaffected. Note that a published crate
(`0.1.2`) carries the positional-binding defect fixed below, so a consumer
resolving `eggup-eggpack 0.1.2` from crates.io does not have this fix until a
new version is published.

- **Security: archive members were bound positionally, with no identity
  cross-check.** `bind_archive_members` paired bound members to declared members
  by declaration order and checked only that the counts matched. A
  `BoundExtraction` produced from a different `ArchivePlan`, or reused after a
  re-projection that reordered members, would therefore commit each member's
  bytes under another member's declared identity. That substitution is
  digest-invisible whenever two declared members happen to share identical
  content, which is exactly when it is most dangerous. Each bound member's own
  recorded `source_path` and `output_name` are now cross-checked against the
  declared member it would be bound to, and a mismatch fails closed with
  `AdapterError::MapMismatch`. The correct declaration-order pairing is
  unaffected. No API change.

## 0.1.2 — 2026-10-02

First crates.io publication of this crate
(`plans/closure/eggpack-manifest-interoperability/004-status.md`). Published after
`eggup-acquisition 0.1.2` and `eggup-eggfetch 0.1.2`, in that dependency order.

- Registry promotion (M004): the immutable-Git `eggpack-manifest` dependency is
  replaced by the registry dependency `eggpack-manifest = "=0.1.0"`, and
  `publish = false` is removed. The published crate's `src/lib.rs` is
  byte-identical to the source at the previously pinned revision
  `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, so the promotion is
  byte-faithful rather than merely version-compatible; no requalification was
  needed. The Eggup edges stay exactly pinned at the published set:
  `eggup-core`, `eggup-archive`, and `eggup-acquisition` each at `=0.1.2`.
- Archive extraction handoff (M002): `archive_format_for_name`,
  `validate_acquired_archive`, `archive_plan_for`, `core_plan_for_archive`, and
  `bind_archive_members` translate `ManifestProjection::Archive` into a
  validated `eggup_archive::ArchivePlan`, a core `InstallPlan`, and
  `BoundSources` for `InstallPlan::prepare_with_bound_sources`, with exact member
  size/digest facts and fail-closed format/count/permission mapping. New
  `AdapterError::UnsupportedArchiveFormat` variant; the error enum is
  `#[non_exhaustive]`, so this is additive. New runtime dependencies are
  `eggup-archive =0.1.2` and `sha2 0.10.9`; `eggup-core` remains
  archive/Eggpack independent. The adapter performs no extraction, commit, or
  cleanup itself — I/O orchestration and deferred `DeferredCleanup` stay
  caller-owned. Direct and bundle APIs are unchanged, and
  `materialize_artifact_set` still returns `ArchiveExtractionRequired` for
  archive projections.
- Caller-bound destinations (M003a): `materialize_artifact_set_with_destinations`,
  `core_plan_for_archive_with_destinations`, and
  `ManifestProjection::default_destinations`. Manifest `install` values are
  producer default identity only and do not authorize replacement of a
  filesystem destination; missing, extra, or colliding destinations fail closed.
  Replacement authorization remains downstream in `eggup-core` ownership
  verification at commit time.
- Package metadata: `homepage`, `documentation`, `authors`, `keywords`, and
  `categories`.
- Unchanged boundaries: the adapter still selects no release, verifies no
  authenticity, and authorizes no filesystem destination. SHA-256 is integrity
  evidence only.

Eggsact's own Git-to-registry consumer migration is authorized separately in
`eggstack/eggsact` and is not claimed here.
