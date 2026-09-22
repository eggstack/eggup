# Domain and preparation contract

`InstallPlan` names a product, opaque release, exact installation root, and a
non-empty `ArtifactSet`. Every member has an independent `MemberId`, an
acquired absolute local source, and a normalized destination relative to the
installation root. Duplicate identities, duplicate normalized destinations,
path traversal, absolute destinations, control characters, source symlinks,
non-regular files, and sources inside the installation root are rejected.

`InstallPlan::prepare` copies inputs into an Eggup-owned private stage
(0700 directories, 0600 files, 0700 for executable intent mapped privately)
and returns `PreparedTransaction`. Preparation does not create or replace any
live destination. Dropping the prepared transaction removes only its own
stage after verifying stage ownership (expected prefix, real directory, no
symlink, recorded parent).

Members declare an `IntegrityRequirement`, but the declaration is not evidence.
Only `Sha256` members can be verified and committed; `None` is observable
during preparation but rejected by validation and by the final staged-digest
revalidation. There is no authenticity field: checksum evidence is distinct
from publisher trust, and no signature claim is made.

Destination ownership is never stored in the plan. At commit time the caller
supplies a synchronous `OwnershipVerifier` returning
`Absent | Owned | Foreign | Unknown` plus an `AbsentPolicy` authorizing (or
forbidding) creation of absent destinations. Generic helpers are
`AbsentOnlyVerifier` (create-only), `ExactDigestVerifier` (prior-content
proof), and the test-oriented `ExistingAsOwnedVerifier`. No universal
executable-identity rule is invented.

