# Domain and preparation contract

`InstallPlan` names a product, opaque release, exact installation root, and a
non-empty `ArtifactSet`. Every member has an independent `MemberId`, an
acquired absolute local source, and a normalized destination relative to the
installation root. Duplicate identities, duplicate normalized destinations,
path traversal, absolute destinations, control characters, source symlinks,
non-regular files, and sources inside the installation root are rejected.

`InstallPlan::prepare` copies inputs into an Eggup-owned temporary stage and
returns `PreparedTransaction`. Preparation does not create or replace any live
destination. Dropping the prepared transaction removes only its private stage.
The type intentionally has no commit operation yet; commit and rollback are
introduced by the next transaction milestone.

Integrity and authenticity fields are declarations only at this layer. A
declared requirement is not evidence that verification passed.

