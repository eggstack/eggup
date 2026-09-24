# eggup-eggpack

Optional adapter from Eggpack ReleaseManifest v1 to caller-owned Eggup acquisition and deployment inputs.

Direct and bundle layouts can be projected, bound to exact caller URLs, and materialized as an `ArtifactSet` after exact-size checks. Archive layouts expose evidence but remain extraction-required. The adapter does not choose URLs, installation roots, ownership, permissions, release policy, or authenticity. SHA-256 is integrity evidence only.

Consumers that fetch manifest bytes should call `project_json(&bytes, canonical_target)`. This applies Eggpack's published document-size bound, delegates UTF-8/schema parsing to `eggpack-manifest`, and returns sanitized adapter errors. Consumers that already hold a parsed `ReleaseManifest` may continue to call `project`.

The crate is unpublished and pins the currently unpublished `eggpack-manifest` crate to an immutable Eggpack revision. Lower Eggup crates remain independent of Eggpack.
