# Eggup Terminology and Domain Model

Status: normative terminology

This document defines canonical Eggup language. Implementation code, documentation, ADRs, and consumer adapters SHOULD use these terms consistently.

## 1. Deployment

A **deployment** is the complete installed state owned by one consumer-defined product at one installation root.

A deployment may contain one or more artifacts and may optionally be associated with a service registration.

A deployment is not synonymous with one executable.

## 2. Product identity

A **ProductId** identifies the logical consumer product for diagnostics, receipts, locks, and ownership metadata.

It is consumer supplied. Eggup does not infer durable product identity from a filename.

Examples:

```text
eggsearch
stegoeggo
codegg
eggress
```

## 3. Release identifier

A **ReleaseId** is the consumer's exact identifier for one desired release.

Eggup core treats it as an opaque validated value.

Examples may be SemVer, PEP-440-like values, or another release vocabulary. Parsing and ordering are release-policy concerns outside the core transaction model.

## 4. Artifact

An **Artifact** is one immutable acquired input that participates in a deployment.

Examples:

- raw executable;
- release archive;
- checksum manifest;
- notice file;
- sidecar executable.

An acquired archive is not necessarily a committed deployment member. Extraction may produce several **ArtifactMembers**.

## 5. Artifact member

An **ArtifactMember** is one staged filesystem object that will be committed to a destination.

It has:

- logical member identity;
- staged path;
- destination path;
- expected file kind;
- executable/permission policy;
- validation state.

## 6. Artifact set

An **ArtifactSet** is the complete set of artifact members that form one transactionally consistent release unit.

A single binary is an artifact set with one member.

A successful commit guarantees that the set is not intentionally left in a mixed old/new state.

## 7. Release plan

A **ReleasePlan** is consumer policy output describing the exact release to acquire.

It may contain:

- ReleaseId;
- source descriptors;
- expected integrity evidence;
- artifact layout;
- fallback policy;
- target/platform selection.

The core does not discover a latest release by itself.

## 8. Install plan

An **InstallPlan** is the mutation-ready plan after acquisition/extraction policy has identified concrete artifact members and destinations.

It defines what the transaction engine is authorized to mutate.

## 9. Acquisition source

An **AcquisitionSource** describes where bytes come from.

Examples:

- HTTPS URL;
- local file fixture;
- injected byte stream;
- Cargo build output through a consumer adapter.

Source type alone does not imply trust.

## 10. Fetcher

A **Fetcher** is an adapter that retrieves metadata or artifacts under explicit network bounds.

A Fetcher has no authority to choose a release or fallback source.

## 11. Integrity evidence

**IntegrityEvidence** proves acquired bytes match an expected digest or manifest.

Initial forms include:

- SHA-256 digest;
- SHA-256 sidecar;
- checksum manifest.

Integrity evidence is not automatically authenticity evidence.

## 12. Authenticity evidence

**AuthenticityEvidence** binds an artifact or integrity statement to a trusted publishing identity.

Examples could include:

- detached signature;
- pinned public-key signature;
- Sigstore-compatible attestation.

`None` is a valid explicit policy for initial consumers.

## 13. Candidate

A **Candidate** is an integrity-verified staged member that may be executed or otherwise validated before commit.

No unverified download is a Candidate.

## 14. Candidate validator

A **CandidateValidator** verifies consumer-specific properties of staged content.

Typical properties:

- program identity;
- exact version;
- helper compatibility;
- config compatibility;
- artifact-set internal consistency.

A validator does not commit files.

## 15. Stage

A **Stage** is private temporary state owned by one transaction.

Staging is disposable until commit begins.

## 16. Installation root

An **InstallationRoot** is the directory boundary within which a deployment's committed members are expected to reside.

A deployment MAY span additional explicitly declared paths for service-manager files or configuration, but artifact transactions should remain narrowly scoped.

## 17. Destination

A **Destination** is one exact target path authorized by the InstallPlan.

Eggup does not search PATH for destinations during mutation.

## 18. Ownership

**Ownership** is evidence that a filesystem or manager object belongs to the intended deployment.

Canonical classification:

```rust
enum Ownership {
    Absent,
    Owned,
    Foreign,
    Unknown,
}
```

Destructive actions are permitted only for `Owned`, or `Absent` when creating a new object.

## 19. Transaction

A **Transaction** is the stateful operation that changes a deployment from one coherent version to another.

It owns staging, lock acquisition, destination preflight, backups, commit, rollback, cleanup, and the final receipt.

## 20. Prepared transaction

A **PreparedTransaction** has completed acquisition-independent validation and is ready to mutate destinations.

All candidate members are validated before preparation succeeds.

## 21. Commit

A **Commit** is the phase where staged members become the live deployment.

Commit begins with the first destructive mutation of a live destination.

## 22. Backup set

A **BackupSet** is transaction-owned recovery state containing every pre-existing live member required to restore the deployment.

Backups are not deleted until the configured commit/finalization policy allows it.

## 23. Rollback

**Rollback** restores the pre-transaction deployment after a failure occurring during or after commit.

Rollback has its own typed result.

## 24. Transaction receipt

A **TransactionReceipt** is the structured successful or recoverable terminal record.

It should identify:

- product;
- release;
- source category;
- committed members;
- previous deployment state when available;
- lifecycle action;
- rollback disposition;
- warnings.

A receipt is not necessarily persisted by Eggup; the consumer may choose persistence.

## 25. Mutation lock

A **MutationLock** provides exclusive ownership of one installation mutation domain.

The lock record carries bounded identity evidence allowing safe stale-lock decisions.

## 26. Lifecycle snapshot

A **LifecycleSnapshot** describes service/runtime state before mutation.

It may record:

- registration manager;
- ownership;
- running/stopped/transitioning state;
- health state;
- restart obligation.

The snapshot does not authorize mutation of foreign/unknown registrations.

## 27. Service manager

A **ServiceManager** is a platform adapter implementing registration and lifecycle operations.

Initial manager families:

- systemd;
- launchd;
- cron/watchdog fallback;
- Windows SCM.

## 28. Service specification

A **ServiceSpec** is consumer-supplied desired registration information.

It may include:

- service identity;
- executable path;
- arguments;
- environment;
- config path;
- restart policy;
- rendered unit/plist data or structured equivalent.

Eggup-service owns safe manager interaction, not consumer-specific daemon semantics.

## 29. Health probe

A **HealthProbe** is an optional consumer-supplied readiness/identity check.

It is separate from manager state.

A running manager entry is not sufficient proof that the intended application is healthy.

## 30. Distribution contract

A **DistributionContract** is producer-side machine-readable release-layout policy owned by Eggpack.

It may define supported target triples, asset/checksum/install names, archive member layout, and other portable release identity needed to keep release assets and bootstrap installers synchronized.

Eggup does not own or evolve this contract. An optional Eggup interoperability adapter may consume concrete producer evidence derived from it without making DistributionContract a runtime deployment authority.

## 31. Bootstrap installer

A **BootstrapInstaller** is the pre-application shell/PowerShell/program entry point used before any Eggup-linked binary exists.

Its generation and release-layout conformance belong to Eggpack. Product policy may still supply origin, privilege, and fallback choices.

It is not the preferred normal self-update path once native Eggup integration exists.

## 32. Release authority

A **ReleaseAuthority** answers which release is desired or current according to consumer policy.

Examples:

- crates.io stable versions;
- GitHub latest non-prerelease;
- a checked-in release catalog;
- an exact operator-specified version.

ReleaseAuthority is outside eggup-core.

## 33. Fallback policy

A **FallbackPolicy** determines whether acquisition may switch to another explicitly authorized source.

Fallback is policy, not a transport side effect.

## 34. Post-commit failure policy

**PostCommitFailurePolicy** determines what happens if a verified installation commits but later post-install or lifecycle work fails.

Canonical choices initially:

- `KeepInstalled`;
- `RollBack`.

## 35. Recovery-required state

A **RecoveryRequired** result means Eggup cannot prove either the old or new deployment is fully coherent and automatic rollback did not restore certainty.

This is a distinct terminal condition and should be treated as high severity by consumers.

## 36. Core boundary rule

When terminology is ambiguous, classify concepts by authority:

- release choice and application semantics belong to the consumer;
- safe artifact mutation belongs to eggup-core;
- byte transport belongs to acquisition adapters;
- manager operations belong to eggup-service;
- producer release contracts, bootstrap/release drift control, packaging, manifests, and release CI belong to Eggpack;
- optional producer-manifest translation into deployment inputs may live in a narrow Eggup adapter.
