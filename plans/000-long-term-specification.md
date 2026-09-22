# Eggup Long-Term Specification

Status: normative end-state specification

This document defines the durable product boundary and invariants for Eggup. Ordinary implementation work MUST conform to it and MUST NOT weaken it for convenience.

## 1. Product definition

Eggup is a Rust-native deployment substrate for Eggstack applications and independently reusable Rust programs.

Its primary purpose is to eliminate duplicated security-sensitive machinery around:

- verified artifact acquisition;
- bounded download and staging;
- checksum and authenticity verification;
- candidate identity validation;
- installation ownership checks;
- atomic or recoverable multi-artifact replacement;
- update locking and stale-lock recovery;
- rollback and recovery;
- install/uninstall primitives;
- optional service-manager registration and lifecycle coordination;
- release/install contract validation.

Eggup is not a centralized release policy engine. Applications retain authority over what version should be installed, which release source is authoritative, which artifacts form one release, which fallback mechanisms are acceptable, and how application-specific migrations or health checks behave.

## 2. Primary goals

Eggup MUST make the safe path reusable enough that Eggstack projects no longer need to independently implement the same updater, installer, rollback, ownership, and service-lifecycle mechanics.

The end state should allow a consumer to express an exact intended deployment and then delegate the dangerous mechanics to Eggup.

A representative conceptual flow is:

```text
consumer release policy
        |
        v
resolved install/update plan
        |
        v
artifact acquisition
        |
        v
integrity/authenticity verification
        |
        v
candidate validation
        |
        v
prepared transaction
        |
        v
lifecycle quiescence when required
        |
        v
ownership revalidation
        |
        v
transactional commit
        |
        v
post-install verification
        |
        v
lifecycle restoration
        |
        v
receipt / rollback disposition
```

## 3. Mechanism versus policy

The central architectural rule is:

**Eggup owns deployment mechanism. The consumer owns deployment policy.**

Eggup core MUST NOT hard-code:

- GitHub as the only release authority;
- crates.io as the only version authority;
- SemVer as the only version language;
- a particular Eggstack repository;
- a particular binary name;
- Cargo fallback as universal behavior;
- automatic source compilation;
- systemd, launchd, cron, or Windows SCM policy;
- a particular health endpoint;
- consumer CLI wording or numeric exit codes;
- application database/config migrations;
- package-manager transitions such as pip, pipx, uv, Homebrew, apt, or similar;
- release cadence or publication automation.

Adapters MAY implement concrete release sources or transports, but the core transaction engine MUST remain policy-neutral.

## 4. Crate and layer model

The intended workspace is layered.

### 4.1 eggup-core

`eggup-core` owns the smallest security-sensitive reusable substrate:

- artifact and artifact-set domain types;
- staging directories/files and permissions;
- bounded command execution required by candidate verification;
- digest/integrity primitives;
- candidate validation interfaces;
- destination ownership and replaceability checks;
- update/install transaction state;
- update locks and stale-lock recovery;
- backups, commit, rollback, and cleanup;
- install/update receipts;
- uninstall-safe path primitives where generic;
- deterministic test seams and failure injection.

It MUST NOT depend on an HTTP/TLS stack, a service manager, GitHub API types, crates.io API types, or application crates.

### 4.2 acquisition adapters

Transport adapters, initially `eggup-eggfetch`, own network acquisition.

They MAY provide:

- bounded HTTPS metadata fetch;
- bounded streaming artifact download;
- redirect policy;
- proxy policy;
- TLS/root policy;
- status classification;
- URL redaction;
- test transport injection.

The transport layer MUST return bytes/files and typed transport results. It MUST NOT decide whether a 404 implies Cargo fallback, whether GitHub latest is authoritative, or what version should be selected.

A lightweight external-curl adapter MAY exist if measured binary-footprint constraints justify it, but it MUST be explicit and optional.

### 4.3 eggup-service

`eggup-service` owns reusable service-manager mechanics:

- manager detection;
- registration inspection;
- exact executable ownership classification;
- install/register;
- uninstall/unregister;
- start/stop/restart;
- bounded state-transition waits;
- generic manager-neutral state types;
- systemd, launchd, cron/watchdog, and Windows SCM adapters where supported.

It MUST remain independent from release acquisition and artifact selection.

Consumer-specific service definitions, command arguments, hardening policy, health payload semantics, and privilege decisions remain supplied by the consumer.

### 4.4 eggup-dist

`eggup-dist` owns development/release-time distribution-contract machinery:

- machine-readable target and asset declarations;
- artifact-name/checksum-name derivation;
- release-contract validation;
- installer template generation or conformance validation;
- release asset completeness checks;
- bootstrap installer test fixtures;
- drift detection between runtime updater and bootstrap installer contracts.

It is tooling, not a runtime dependency required by every consumer.

### 4.5 eggup facade

A thin `eggup` facade crate MAY be introduced after the lower-level contracts stabilize. It MUST NOT force heavy optional dependencies into consumers that need only core transaction primitives.

## 5. Transaction model

Eggup MUST treat installation and update as explicit transactions.

A transaction proceeds through typed phases conceptually equivalent to:

```text
Planned
  -> Acquired
  -> IntegrityVerified
  -> CandidateVerified
  -> Prepared
  -> LifecycleQuiesced        [optional]
  -> DestinationRevalidated
  -> BackedUp
  -> Committing
  -> Installed
  -> PostInstallVerified
  -> LifecycleRestored        [optional]
  -> Committed
```

Failures MUST carry enough phase information to determine whether:

- no mutation occurred;
- rollback was attempted;
- rollback succeeded;
- a verified new installation remains in place;
- manual recovery is required.

## 6. Multi-artifact installation is fundamental

The core model MUST support one logical release containing multiple files.

A single binary is an artifact set of size one.

Multi-artifact support is required for consumers such as:

- a primary executable plus helper executable;
- compatibility sibling binaries;
- managed sidecar binaries;
- release notices or required runtime data;
- future platform-specific helper sets.

The transaction contract MUST prevent a successful result from leaving a mixed old/new artifact set.

No consumer should need a second bespoke rollback implementation merely because it contains two or three binaries.

## 7. Verification model

Eggup distinguishes integrity, authenticity, and candidate identity.

### 7.1 Integrity

Integrity proves that acquired bytes match declared bytes.

Initial supported mechanisms SHOULD include SHA-256 sidecars and checksum manifests.

Integrity verification MUST occur before an artifact is executed or committed.

### 7.2 Authenticity

Authenticity proves that integrity metadata or artifacts are authorized by a trusted publisher identity.

Initial releases MAY support `AuthenticityPolicy::None` while relying on HTTPS plus release-host security, matching current Eggstack practice.

The API MUST NOT misrepresent a checksum sidecar downloaded beside an artifact as an independent signature.

The model MUST leave room for signatures, pinned public keys, Sigstore-compatible attestations, or other provenance systems later.

### 7.3 Candidate identity

A candidate MAY be executed only after integrity verification.

Execution MUST be bounded in time and output, with controlled environment inheritance.

Candidate validators MUST be capable of proving:

- expected program identity;
- exact or policy-approved version;
- artifact-set consistency;
- consumer-specific compatibility checks.

## 8. Acquisition safety

All network acquisition MUST be bounded.

Adapters MUST provide explicit:

- connect deadline;
- total deadline;
- redirect limit;
- response size or streaming bounds;
- HTTPS downgrade policy;
- proxy behavior;
- URL/credential redaction behavior.

Unexpected transport failures MUST NOT silently trigger a different installation source.

Fallback is consumer policy and MUST occur only for explicitly classified conditions.

## 9. Filesystem safety

Eggup MUST:

- stage privately;
- avoid predictable world-writable temporary filenames;
- prefer same-filesystem commit staging when practical;
- verify destination writability before destructive mutation;
- revalidate destination identity immediately before commit;
- reject unsafe file types;
- treat symlink and hard-link behavior deliberately;
- preserve intended executable permissions;
- clean abandoned stage files where ownership is provable;
- fsync/flush critical state where required by the documented durability contract.

Eggup MUST NOT invoke `sudo`, UAC elevation, or privilege prompts internally.

It may return exact remediation/elevation guidance for the caller to present.

## 10. Ownership model

Destructive operations require evidence that the target belongs to the intended installation.

Generic ownership classifications are:

```text
Absent
Owned
Foreign
Unknown
```

`Foreign` and `Unknown` MUST fail closed for destructive operations unless an explicitly separate takeover operation is designed later.

Path basename matching alone is insufficient ownership evidence for service registrations or sibling artifacts.

## 11. Locking and contention

Only one mutating transaction may own a given installation root at a time.

The core MUST support:

- exclusive update/install lock creation;
- lock records bounded in size;
- installation identity in lock records;
- stale-lock recovery only when staleness can be proven;
- failure-closed behavior when ownership is ambiguous;
- cleanup on ordinary completion.

Process existence alone is not sufficient if PID reuse can cause ambiguity; platform-specific stronger identity SHOULD be used where practical.

## 12. Rollback model

Rollback is a first-class result, not an error-message afterthought.

The core MUST support policy equivalent to:

```rust
enum PostCommitFailurePolicy {
    KeepInstalled,
    RollBack,
}
```

This is necessary because current consumers differ legitimately:

- some retain a fully verified newly installed binary when service restart fails;
- others restore the previous binary if restart or post-install verification fails.

The transaction engine provides the mechanism. The consumer chooses the policy before mutation begins.

If rollback fails, Eggup MUST preserve recovery evidence and return a distinct high-severity result.

## 13. Service lifecycle

Service lifecycle is optional and separate from core artifact replacement.

A service-aware consumer may:

1. snapshot lifecycle and ownership;
2. fully acquire and verify candidate artifacts;
3. quiesce an owned service only when required;
4. commit;
5. perform post-install checks;
6. restart only if the pre-update policy requires it;
7. apply the configured post-commit failure policy.

A stopped service MUST remain stopped unless the consumer explicitly requests otherwise.

A foreign or ambiguous service registration MUST not be stopped, rewritten, or deleted.

## 14. Bootstrap installers

A bootstrap installer necessarily runs before the Rust library exists. Eggup therefore cannot eliminate shell/PowerShell bootstrap code solely through a runtime crate.

Instead, Eggup SHOULD provide one authoritative distribution contract from which bootstrap installers are generated or validated.

Bootstrap installers MUST:

- use fixed or validated release origins;
- map OS/architecture deterministically;
- download into private temporary space;
- verify integrity before execution or commit;
- validate candidate identity when practical;
- avoid automatic privilege escalation;
- fail closed on unsupported targets;
- preserve an existing installation on verification failure;
- perform multi-file updates transactionally when the release is a bundle.

Installer scripts MUST NOT become the normal in-process self-update implementation when native Eggup replacement is available.

## 15. Version and release authority

Eggup core treats release identifiers as consumer-owned opaque values except where an adapter explicitly provides parsing.

This allows:

- SemVer consumers;
- PEP-440-like consumers;
- GitHub-release-authority consumers;
- crates.io-authority consumers;
- exact pinned releases;
- custom catalogs.

A future generic version helper crate MAY exist, but no global version semantics are part of the transaction engine.

## 16. Cargo/source fallback

Cargo fallback is optional policy.

If supported, the fallback MUST be:

- explicitly enabled by the consumer;
- triggered only by enumerated conditions;
- exact-version when a version is resolved;
- staged outside the live destination;
- candidate-verified before commit;
- bounded in execution time and output.

Checksum mismatch, candidate identity mismatch, TLS failure, unexpected HTTP status, or malformed metadata MUST NOT silently fall back to source installation.

## 17. Uninstall

Generic uninstall machinery may share:

- exact current-executable resolution;
- installation-root ownership checks;
- path equivalence;
- writable-parent preflight;
- safe self-deletion where platform semantics permit;
- transactionally removing an owned artifact set.

Service deregistration, configuration/data deletion, package-manager ownership, and consumer cleanup policy remain explicit caller decisions.

Uninstall MUST preserve foreign and ambiguous artifacts.

## 18. Security model

The supported threat model includes:

- corrupted downloads;
- mismatched checksum metadata;
- malicious or compromised mirrors when origin policy is weak;
- archive path traversal;
- untrusted candidate output;
- destination path races;
- stale locks;
- service-name collisions;
- foreign service registrations;
- partial multi-file commit;
- process interruption;
- oversized metadata or process output;
- credentials embedded in URLs or environment;
- symlink/hard-link confusion.

Eggup does not claim to defend against a fully compromised privileged operating system or a publisher signing key that the configured authenticity policy explicitly trusts.

## 19. Dependency and footprint policy

`eggup-core` SHOULD remain small enough for SBC and daemon consumers.

It MUST avoid an HTTP/TLS dependency.

Dependencies with unsafe code, native libraries, or broad transitive graphs require explicit review.

The project baseline is Rust 1.89 unless an accepted ADR changes it.

Feature flags MUST NOT create a hidden maximal dependency set.

## 20. Platform scope

Initial target families are:

- Linux x86_64 GNU;
- Linux aarch64 GNU;
- macOS x86_64;
- macOS aarch64;
- Windows x86_64 MSVC.

Additional targets may be supported by core filesystem primitives before prebuilt release assets exist.

Service-manager support is platform-conditional and must be tested independently.

## 21. Observability

All major operations SHOULD expose structured progress/events suitable for CLI, TUI, JSON, or logs without forcing presentation into core.

Events MUST NOT contain credentials or unbounded command/network output.

Errors MUST retain stable machine-readable categories even when display text evolves.

## 22. Consumer adoption

No subsystem is considered proven solely by unit tests inside Eggup.

At least two independent real consumers MUST adopt a reusable contract before that contract is considered mature.

Initial intended adoption sequence:

1. eggsact and stegoeggo for native Eggfetch single-binary update;
2. eggsearch for updater plus later lifecycle integration;
3. gregg to replace `gregg-update` while retaining lightweight transport if needed;
4. CodeGG for verified multi-artifact self-update;
5. eggress for archive/two-binary transaction;
6. EggPool for selected low-level transaction/ownership primitives only where its richer provenance model remains intact.

## 23. Public API stability

Pre-1.0 releases may refine APIs, but changes MUST remain documented and consumer migrations deliberate.

Security-sensitive behavior MUST NOT change through undocumented default changes.

Public enums representing safety policy SHOULD be non-exhaustive where future extension is expected.

## 24. Testing requirements

The project MUST provide deterministic tests for:

- single-artifact commit;
- multi-artifact commit;
- each commit-phase failure with restoration;
- rollback failure;
- checksum mismatch;
- wrong candidate identity/version;
- command timeout/output bound;
- unwritable destination;
- symlink/hard-link rejection;
- lock contention and stale-lock behavior;
- cross-device staging behavior where supported;
- interruption/recovery seams;
- Windows running-image replacement semantics;
- service ownership and lifecycle state machines;
- installer/updater asset-contract drift.

Network tests MUST use local fixtures/test transports by default. Public internet access is not a correctness prerequisite.

## 25. Release philosophy

Eggup's own release process SHOULD remain simple and manually controlled initially.

Ordinary CI proves correctness. Publication is a separate maintainer action.

The project MUST support `cargo package` / `cargo publish --dry-run` qualification for crates intended for external consumers before declaring their package boundary stable.

## 26. Non-goals

Eggup is not:

- a general package manager;
- an OS package repository;
- a daemon supervisor;
- a release-hosting service;
- a deployment orchestrator for fleets;
- a configuration migration framework;
- an application auto-update scheduler;
- a replacement for systemd/launchd/SCM;
- a source-build system;
- an excuse to standardize unrelated application release policies.

Those capabilities may integrate with Eggup, but they do not belong in its core ownership boundary.

## 27. End-state completion criteria

Eggup reaches its intended mature state when:

- core verified multi-artifact transactions are stable and independently packaged;
- transport is pluggable and Eggfetch has a supported adapter;
- service lifecycle is separately reusable with ownership-safe semantics;
- bootstrap installer contracts share authoritative target/asset metadata with runtime consumers;
- CodeGG's generic updater blocker is resolved without importing Gregg-specific assumptions;
- at least two Eggstack projects have deleted their bespoke updater implementations in favor of Eggup;
- at least one multi-artifact consumer uses the transaction engine;
- rollback, contention, Windows, and service-lifecycle failure paths have platform evidence;
- integrity and authenticity semantics are accurately documented;
- no consumer is forced to adopt an unwanted HTTP stack, service manager, or release authority.
