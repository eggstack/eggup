# Verified Update Core Milestone 004 — Integrity and Candidate Validation

Status: implemented

Repository baseline for planning: `f5765a5`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md#M004--integrity-and-candidate-validation`

Long-term requirements:

- `plans/000-long-term-specification.md#7-verification-model`
- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/001-terminology-and-domain-model.md#11-integrity-evidence`
- `plans/001-terminology-and-domain-model.md#13-candidate`

Applicable ADR:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: capability / invariant

## 1. Objective

Implement local integrity and bounded candidate verification so a staged file cannot become a commit candidate until its declared integrity and consumer-supplied identity requirements pass.

## 2. Why this milestone is not yet ready

Hard dependency: M002 closure, because verification must attach to the actual ArtifactMember/PreparedTransaction model rather than invent parallel structures. M002 is closed at `adaada4`; the latest reconciled implementation baseline is M003 at `f5765a5`.

M003 may proceed in parallel after M002, but final core qualification requires both.

## 3. Current implementation evidence

Current Eggstack patterns repeatedly implement SHA-256 sidecar parsing, executable-bit handling, bounded `version` command execution, exact program/version output parsing, and cross-member version agreement.

Some implementations use external hash tools; others use Rust `sha2`. Eggup should provide native local hashing to avoid external tool variability.

## 4. Invariants that must not regress

- integrity verification precedes candidate execution;
- checksum sidecar is not described as independent authenticity;
- malformed/mismatched integrity evidence is hard failure;
- verification failure never invokes fallback;
- candidate subprocesses are time/output bounded and reaped;
- sensitive inherited environment is minimized;
- candidate identity logic remains consumer-configurable.

## 5. Scope

### In scope

- SHA-256 file hashing;
- strict sidecar/manifest parser;
- exact filename binding where manifest format includes names;
- integrity result type;
- explicit authenticity-policy placeholder/type with `None` support but no signing implementation;
- bounded command runner;
- command environment policy;
- exact program/version validator helper;
- cross-member agreement helper;
- custom CandidateValidator interface;
- verification state attached to staged members/prepared transaction.

### Explicitly out of scope

- network fetch;
- signature/attestation verification;
- release-version ordering;
- Cargo fallback;
- archive extraction;
- service health checks;
- live commit if M003 separately owns it.

## 6. Required production changes

Use native SHA-256.

Sidecar parsing must reject ambiguous multi-digest input unless an explicit manifest parser resolves the requested filename.

Candidate execution should support:

- argv vector, never shell interpolation;
- deadline;
- maximum stdout/stderr;
- null stdin;
- controlled current directory;
- environment clear by default with explicit allowlist if required;
- kill/reap on timeout;
- structured exit/output result.

Provide a validator for conventional exact output such as `program X.Y.Z` without making SemVer parsing mandatory. Exact expected release text should be consumer supplied.

Cross-member validator should allow Egress/CodeGG-like bundles to assert all member identities are mutually compatible.

## 7. Ordered work packages

A. IntegrityEvidence and SHA-256 implementation.

B. Sidecar/manifest parser and malformed-input tests.

C. Bounded command runner.

D. Exact identity/version validator.

E. Composable custom and cross-member validators.

F. Integrate verification state into PreparedTransaction without live mutation.

## 8. Failure, cancellation, restart, and contention semantics

Verification failure destroys only disposable stage if the caller chooses; it never mutates live state.

Timeout must kill and reap the child before returning.

Output overflow should stop retaining bytes and terminate/fail according to documented behavior; do not allow unbounded memory.

## 9. Compatibility and migration

Initial consumers can preserve checksum-only authenticity policy explicitly. Do not force signing.

## 10. Required tests

- known SHA vectors/files;
- malformed, uppercase/lowercase, whitespace, wrong filename, wrong digest sidecars;
- candidate exits nonzero;
- candidate hangs;
- stdout/stderr overflow;
- candidate wrong program;
- wrong version;
- extra unexpected output under strict validator;
- environment secret not inherited by default;
- two/three-member agreement/disagreement;
- unverified candidate cannot reach executable validator through normal API.

## 11. Required verification commands

Focused tests plus M001 broad suite. Consider fuzz/property tests for sidecar parsing if small and useful, but do not add a fuzz framework merely for this milestone unless justified.

## 12. Documentation updates

Document integrity versus authenticity precisely and provide a safe validator example.

## 13. Acceptance criteria

A staged artifact becomes a validated candidate only after integrity passes, and candidate execution is bounded and consumer-policy-neutral.

## 14. Stop conditions

Stop if:

- implementation requires network knowledge;
- signature trust selection is required to complete checksum semantics;
- validator design requires hard-coded SemVer/product names;
- subprocess behavior cannot be bounded on a supported platform.

## 15. Closure evidence required

- parser negative-test matrix;
- timeout/reap evidence;
- output-bound evidence;
- environment-inheritance evidence;
- exact identity/version tests;
- cross-member validation;
- broad verification.

## 16. Handoff notes

Do not introduce an `AuthenticityVerified` claim when policy is `None`. The API should make the distinction visible.
