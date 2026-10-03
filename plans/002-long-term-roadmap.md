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
- deferred successful-commit finalization for one bounded caller post-install check;
- explicit post-commit `KeepInstalled | RollBack` policy;
- cleanup rules;
- transaction receipt.

### Exit criteria

- every injected commit-phase failure has deterministic old/new/recovery outcome evidence;
- successful multi-file commit never reports success with mixed generations;
- a coherent newly installed generation can remain rollback-capable through one caller-owned post-install verification step;
- post-commit rollback failure yields RecoveryRequired with retained evidence;
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
- lifecycle snapshot and restore orchestration composed with the core deferred-finalization/post-commit policy seam.

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

- CodeGG `InstallPlan` / `ArtifactSet` adapter;
- transactional bundle update for `codegg`, `codegg-sandbox-helper`, and `codegg-eggsearch`;
- bounded native acquisition through Eggup's acquisition seam while preserving CodeGG's existing Eggfetch trust profile;
- archive checksum-manifest verification before strict CodeGG-owned extraction;
- per-member integrity continuity plus product-specific bundle member identity/version checks;
- atomic/recoverable commit;
- removal of check-only limitation for supported prebuilt installations.

### Exit criteria

- normal `codegg upgrade` no longer requires manual network-fetched shell execution for in-place updates;
- failure leaves a coherent prior installation or typed RecoveryRequired state;
- fresh-install bootstrap script remains independently supported.

## Phase 9 — Eggpack authority cutover and manifest interoperability

Status: **complete**.

### Objective

End Eggup's temporary ownership of producer-side distribution contracts and establish a narrow producer/consumer seam with Eggpack.

### Deliverables

- preserve Eggup distribution M001-M003 as predecessor/closure evidence;
- retire `eggup-dist` after Eggpack Contract M002 independently qualified the transferred producer behavior (completed by Eggup Distribution M004; see `plans/closure/distribution-bootstrap/004-status.md`);
- keep installer generation, release conformance, package construction, release manifests, CI, and publication in Eggpack;
- provide the optional `eggup-eggpack` manifest-consumer adapter without importing producer build/CI machinery into lower Eggup layers;
- prove the seam through a real consumer and a registry-resolvable package graph.

Completion evidence:

- Eggpack ReleaseManifest v1 and the lightweight `eggpack-manifest 0.1.0` consumer crate are established and published;
- Eggup Interoperability M003 closed on real Eggsact consumption of manifest evidence while Eggsact retained release/origin/fallback/destination policy;
- M004 published `eggup-acquisition 0.1.2`, `eggup-eggfetch 0.1.2`, and `eggup-eggpack 0.1.2`, with registry-only adapter and Eggsact-shaped graphs qualified (see `plans/closure/eggpack-manifest-interoperability/004-status.md`);
- `eggup-core`, acquisition, archive, and service remain independent of Eggpack producer build/CI/bootstrap crates.

### Exit criteria

All satisfied:

- Eggup has no active producer distribution crate; the historical distribution roadmap is archived/transferred to Eggpack;
- Eggpack is the sole active authority for DistributionContract/conformance and producer release evidence;
- `eggup-core` has no Eggpack dependency;
- `eggup-eggpack` translates producer evidence into explicit Eggup deployment inputs without deciding update policy;
- Eggup remains usable with non-Eggpack releases.

## Phase 10 — Eggress archive/bundle convergence

Status: **complete**.

### Objective

Prove archive extraction and two-binary transaction support.

### Deliverables

- safe bounded archive extraction with traversal/link/special-file/no-clobber guards;
- handle-backed extracted-member handoff into Eggup transaction preparation;
- Eggpack archive-evidence handoff through the same qualified extraction boundary;
- `eggress` + `pproxy` staged cross-member/version verification;
- transactional pair replacement through Eggup.

Completion evidence:

- Archive M001d closed the handle-backed source handoff with hosted qualification;
- Eggpack Interoperability M002/M002a closed the archive projection/extraction handoff with hosted qualification;
- Consumer Adoption M006 closed through Egress Delivery M003 at `19e6dc7`: Egress removed its bespoke `replace_pair` backup/rollback and extraction helpers, retained only application-specific path/release policy, and moved the pair through bounded extraction, staged exact-version checks, and Eggup commit/rollback;
- Egress hosted updater/archive lanes passed on Linux, macOS, and Windows (run `36639694985`).

### Exit criteria

All satisfied:

- Egress removed bespoke pair rollback code;
- exact staged-version agreement plus sibling-pair policy prevents a successful mismatched pair, while Eggup provides the transaction/rollback disposition.

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
