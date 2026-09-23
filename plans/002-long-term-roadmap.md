# Eggup Long-Term Implementation Roadmap

Status: execution roadmap for `plans/000-long-term-specification.md`

Terminology: `plans/001-terminology-and-domain-model.md`

This roadmap is dependency-ordered, not calendar-ordered. Each phase must leave Eggup independently testable and must not require downstream consumer migration to prove an earlier internal contract unless that phase explicitly names adoption evidence.

## Cross-phase execution rules

Every phase MUST:

1. preserve the mechanism-versus-policy boundary;
2. keep `eggup-core` free of HTTP/TLS and service-manager dependencies;
3. verify all candidate bytes before live mutation;
4. preserve existing deployments on pre-commit failure;
5. maintain typed rollback/recovery outcomes after commit starts;
6. avoid implicit privilege escalation;
7. use deterministic local fixtures for correctness tests;
8. bound network, subprocess, and captured-output resources;
9. document dependency/feature impact for consumers;
10. add closure evidence before dependent work is considered ready;
11. keep producer release contracts, packaging, bootstrap generation, release CI, and publication in Eggpack.

## Phase 0 — Repository and contract foundation

### Objective

Create the Rust workspace, planning/architecture surfaces, baseline safety policy, and deterministic test harness needed before security-sensitive implementation begins.

### Deliverables

- Rust 1.89 workspace;
- `eggup-core` package skeleton;
- workspace package/dependency/lint ownership;
- `#![forbid(unsafe_code)]` or stricter equivalent for first-party core code unless an accepted ADR later creates a narrowly isolated exception;
- typed error/event skeleton;
- test-support module for temporary installation roots and deterministic failure injection;
- local verification script or documented equivalent;
- minimal ordinary CI for Linux plus compile/check coverage appropriate to the initial scope;
- package metadata sufficient for eventual `cargo package` qualification.

### Exit criteria

- workspace builds and tests on the declared MSRV;
- no network/service-manager dependency is present in `eggup-core`;
- architecture guards make the core boundary mechanically visible;
- first implementation plan closes with a clean package baseline.

## Phase 1 — Core deployment domain and transaction preparation

### Objective

Implement policy-neutral domain types and the prepared-transaction boundary without yet performing live replacement.

### Deliverables

- ProductId/ReleaseId validated wrappers;
- ArtifactSet and ArtifactMember;
- InstallPlan;
- IntegrityEvidence representation;
- CandidateValidator interface;
- Ownership classification;
- transaction phase/error taxonomy;
- private staging primitives;
- destination preflight and path normalization.

### Exit criteria

- single- and multi-member plans can be validated without mutation;
- unsafe/duplicate/escaping destinations fail closed;
- staged files are owner-private;
- test fixtures prove the same domain can represent eggsact-like single binaries and CodeGG/Egress-like bundles.

## Phase 2 — Transactional commit, lock, rollback, and recovery

### Objective

Make live mutation safe for one or many files.

### Deliverables

- MutationLock;
- stale-lock proof rules;
- backup set;
- same-filesystem commit path;
- controlled cross-device staging transfer where supported;
- multi-member commit;
- deterministic rollback;
- rollback-failure/RecoveryRequired result;
- cleanup rules;
- transaction receipt.

### Exit criteria

- every injected commit-phase failure has deterministic old/new/recovery outcome evidence;
- successful multi-file commit never reports success with mixed generations;
- lock contention is deterministic;
- pre-commit failures do not alter live destinations.

## Phase 3 — Integrity and candidate validation

### Objective

Centralize the verification behavior duplicated across Eggstack.

### Deliverables

- native SHA-256 implementation;
- sidecar/manifest parser with exact-file binding;
- bounded candidate command runner;
- exact identity/version validator helpers;
- environment-clearing/redaction defaults;
- composable custom validator interface.

### Exit criteria

- checksum mismatch and malformed evidence fail before execution;
- wrong program/version fails before commit;
- timeout and output overflow terminate and reap children;
- artifact-set validators can require cross-member version agreement.

## Phase 4 — Acquisition abstraction and Eggfetch adapter

### Objective

Provide native bounded HTTP acquisition without coupling the core to a transport.

### Deliverables

- transport-neutral Fetcher interface or equivalent seam outside core;
- `eggup-eggfetch` adapter;
- explicit timeout/redirect/proxy/root behavior;
- streamed-to-file artifact path;
- bounded small-metadata path;
- typed HTTP 404 versus hard failure;
- local HTTP fixture qualification.

### Exit criteria

- core compiles without Eggfetch;
- Eggfetch adapter supports eggsact/stegoeggo-class downloads;
- no transport condition silently chooses a fallback;
- consumers can inject a different transport.

## Phase 5 — First real consumer adoption

### Objective

Prove core/updater contracts against at least two similar existing implementations before expanding architecture.

### Initial consumers

- eggsact;
- stegoeggo.

### Required evidence

- consumer adapters retain their current release/version policy;
- bespoke checksum/stage/replacement code is deleted rather than wrapped;
- updater behavior and failure semantics are preserved or intentionally documented;
- dependency and binary-size impact are measured;
- no second HTTP/TLS stack is introduced.

### Exit criteria

- two independently released consumers use the same Eggup core/update contract;
- issues found during adoption are resolved in Eggup rather than copied locally where generic.

## Phase 6 — Service lifecycle substrate

### Objective

Extract reusable ownership-safe manager mechanics from eggsearch/greggd patterns.

### Deliverables

- `eggup-service`;
- manager-neutral registration/state interfaces;
- ownership classification;
- systemd adapter;
- launchd adapter;
- cron/watchdog adapter;
- Windows SCM adapter;
- bounded transition waits;
- optional HealthProbe seam;
- lifecycle snapshot and restore orchestration.

### Exit criteria

- destructive manager operations require Owned state;
- foreign/unknown registrations are preserved;
- stopped services remain stopped by default;
- Linux/macOS/Windows manager state machines have platform or deterministic adapter evidence.

## Phase 7 — eggsearch and Gregg convergence

### Objective

Replace mature duplicated updater/service machinery while preserving each product's policy and footprint requirements.

### eggsearch

- adopt Eggup core + Eggfetch adapter;
- later adopt service substrate where parity is proven;
- preserve startup health semantics.

### Gregg

- replace `gregg-update`;
- allow lightweight curl acquisition if measured Eggfetch footprint remains unacceptable;
- preserve Cargo fallback and exact service lifecycle semantics.

### Exit criteria

- no copied generic update helper remains in either consumer;
- service policy remains consumer-owned;
- Gregg is not forced to adopt a heavier transport.

## Phase 8 — Multi-artifact CodeGG adoption

### Objective

Resolve CodeGG's documented generic-updater blocker.

### Deliverables

- CodeGG ReleasePlan adapter;
- transactional bundle update for `codegg`, `codegg-sandbox-helper`, and `codegg-eggsearch`;
- native Eggfetch acquisition;
- checksum manifest verification;
- bundle member identity/version checks;
- atomic/recoverable commit;
- removal of check-only limitation for supported prebuilt installations.

### Exit criteria

- normal `codegg upgrade` no longer requires manual network-fetched shell execution for in-place updates;
- failure leaves a coherent prior installation or typed RecoveryRequired state;
- fresh-install bootstrap script remains independently supported.

## Phase 9 — Eggpack authority cutover and manifest interoperability

### Objective

End Eggup's temporary ownership of producer-side distribution contracts and establish a narrow producer/consumer seam with Eggpack.

### Deliverables

- preserve Eggup distribution M001-M003 as predecessor/closure evidence;
- freeze `eggup-dist` except for migration-critical correctness fixes;
- require Eggpack Contract M002 to port and independently qualify the closed Eggup M003 conformance behavior;
- execute Eggup distribution M004 to remove `eggup-dist` after that Eggpack closure (completed; see `plans/closure/distribution-bootstrap/004-status.md`);
- keep installer generation, release conformance, package construction, release manifests, CI, and publication in Eggpack;
- after Eggpack ReleaseManifest v1 stabilizes, optionally add a small `eggup-eggpack` manifest-consumer adapter.

### Exit criteria

- Eggup has no active producer distribution crate; the historical distribution roadmap is archived/transferred to Eggpack;
- Eggpack is the sole active authority for DistributionContract/conformance;
- `eggup-core` has no Eggpack dependency;
- any future manifest adapter translates producer evidence into explicit Eggup deployment inputs without deciding update policy;
- Eggup remains usable with non-Eggpack releases.

## Phase 10 — Eggress archive/bundle convergence

### Objective

Prove archive extraction and two-binary transaction support.

### Deliverables

- safe archive adapter or consumer-supplied extraction contract;
- archive traversal guards;
- `eggress` + `pproxy` cross-member verification;
- transactional pair replacement through Eggup.

### Exit criteria

- Egress removes bespoke pair rollback code;
- no successful result permits mismatched pair versions.

## Phase 11 — EggPool selective adoption

### Objective

Reuse only the parts that genuinely simplify EggPool without weakening its richer provenance/package-manager transition model.

Candidate shared areas:

- mutation lock mechanics;
- destination ownership helpers;
- staged executable validation;
- backup/rollback primitives;
- structured transaction outcomes.

Explicitly retained in EggPool:

- Python/Rust release-era compatibility;
- install provenance;
- uv/pipx/pip transitions;
- database/config compatibility checks;
- EggPool-specific rollback policy.

### Exit criteria

Adoption occurs only where net code/security ownership improves. No full-migration requirement exists.

## Phase 12 — Authenticity and supply-chain hardening

### Objective

Add optional independent authenticity verification after the core deployment model is proven.

Potential work:

- detached signatures;
- pinned publisher keys;
- GitHub/Sigstore-style attestations;
- provenance receipts;
- key rotation model;
- offline verification.

This phase requires its own ADR before selecting a durable trust standard.

## Phase 13 — Public API stabilization

### Objective

Stabilize the independently consumed crate surface toward 1.0.

### Deliverables

- semver policy;
- feature compatibility matrix;
- MSRV policy;
- migration guide;
- package/release procedure;
- fuzz/property coverage for parsers and transaction state;
- published API review.

### Exit criteria

- consumer migrations no longer require internal module access;
- security-sensitive defaults are documented;
- package dry-runs and supported platform CI are green;
- no known high-severity correctness or recovery defect remains.
