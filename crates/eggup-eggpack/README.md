# eggup-eggpack

Optional adapter from Eggpack ReleaseManifest v1 to caller-owned Eggup acquisition and deployment inputs.

Direct and bundle layouts can be projected, bound to exact caller URLs, and materialized as an `ArtifactSet` after exact-size checks. Archive layouts expose evidence but remain extraction-required. The adapter does not choose URLs, installation roots, ownership, permissions, release policy, or authenticity. SHA-256 is integrity evidence only.

The crate is unpublished and pins the currently unpublished `eggpack-manifest` crate to an immutable Eggpack revision. Lower Eggup crates remain independent of Eggpack.
