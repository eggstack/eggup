# eggup-eggpack

Optional adapter from Eggpack ReleaseManifest v1 to caller-owned Eggup acquisition and deployment inputs.

Direct and bundle layouts can be projected, bound to exact caller URLs, and materialized as an `ArtifactSet` after exact-size checks. Archive layouts flow through the qualified extraction handoff: `validate_acquired_archive` proves archive path/size/digest continuity, `archive_format_for_name` maps the caller-selected artifact name to a supported format (fail-closed), `archive_plan_for` builds the `eggup_archive::ArchivePlan` with exact member size/digest facts, `core_plan_for_archive` builds the core `InstallPlan` while advisory paths are valid, and `bind_archive_members` transfers open member objects into `BoundSources` for `InstallPlan::prepare_with_bound_sources`. The helpers perform no extraction, commit, or cleanup themselves; I/O orchestration and deferred `DeferredCleanup` stay caller-owned. The adapter does not choose URLs, installation roots, ownership, permissions, release policy, or authenticity. SHA-256 is integrity evidence only.

Consumers that fetch manifest bytes should call `project_json(&bytes, canonical_target)`. This applies Eggpack's published document-size bound, delegates UTF-8/schema parsing to `eggpack-manifest`, and returns sanitized adapter errors. Consumers that already hold a parsed `ReleaseManifest` may continue to call `project`.

The crate is unpublished and pins the currently unpublished `eggpack-manifest` crate to an immutable Eggpack revision. Lower Eggup crates remain independent of Eggpack.
