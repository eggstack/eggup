# Verified Update Core Milestone 006 — Core Package Qualification

Status: blocked on M005 closure

Repository baseline for planning: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`
- `plans/000-long-term-specification.md#23-public-api-stability`
- `plans/000-long-term-specification.md#25-release-philosophy`

Primary class: infrastructure / polish

## 1. Objective

Qualify the corrected `eggup-core` package as an independently consumable pre-1.0 Rust library before the first Eggstack adoption.

This milestone does not publish automatically. It proves package contents, API/documentation quality, MSRV, dependency surface, examples, and cross-platform compile posture.

## 2. Why this milestone is blocked

Hard dependency: M005 pre-qualification safety/API corrective.

Package qualification against the current M004 API would freeze known contract defects.

Before handoff, replace the baseline with the M005 closure SHA.

## 3. Invariants that must not regress

- no HTTP/TLS/service-manager dependency;
- no consumer-specific product/repository constants;
- Rust 1.89 MSRV unless separately changed;
- no unsafe first-party core code;
- no automatic publication;
- examples use only the supported verified/validated commit path;
- package metadata does not imply signature/authenticity support that is absent.

## 4. Scope

### In scope

- public API/rustdoc review;
- package manifest metadata;
- crates.io name availability check at execution time;
- `cargo package` and `cargo publish --dry-run`;
- package-content inspection;
- dependency tree and duplicate/security review;
- representative examples;
- compile/check lanes for supported desktop/server targets;
- MSRV proof;
- README/changelog version-state reconciliation;
- optional benchmark/size measurement for the library graph if meaningful.

### Explicitly out of scope

- actual crates.io publication unless separately directed;
- Eggfetch adapter;
- consumer migration;
- service manager;
- release automation;
- signature support.

## 5. Required production changes

### Public API review

Review every exported type/function for:

- stable terminology;
- clear ownership/precondition docs;
- failure semantics;
- no accidental consumer-specific policy;
- `#[non_exhaustive]` where appropriate for externally matched enums;
- no public test/fault-injection surface;
- no misleading "atomic" claims.

Add doctests/examples for:

1. one-member verified transaction;
2. multi-member verified transaction;
3. custom candidate validator;
4. ownership verifier;
5. interpreting Committed/RolledBack/RecoveryRequired.

### Package metadata

Ensure `description`, `readme`, `license`, `repository`, relevant keywords/categories, and package include/exclude behavior are correct.

Check current crates.io namespace before choosing a publication name. If `eggup-core` is unavailable, stop rather than silently publishing under an accidental name.

### Package content

Inspect the generated `.crate` contents. Do not package planning archives, local build outputs, secrets, test fixtures that are not useful to consumers, or repository-only CI assets unless intentionally included.

### Dependency/security review

Record:

- `cargo tree --workspace --locked`;
- direct/runtime dependencies;
- known advisory state using the repository's accepted security tooling if available;
- rationale for every non-std runtime dependency.

Do not add a scanner framework solely to satisfy this milestone unless the project adopts it as a durable maintenance choice.

### Platform qualification

At minimum:

- Linux x86_64 hosted tests;
- macOS native tests if available;
- Windows compile/test lane sufficient to exercise path/process APIs.

If Windows live replacement remains unsupported/unproven, document that clearly in package support status rather than claiming full Windows transaction support.

## 6. Ordered work packages

A. Public API/rustdoc audit.

B. Examples and doctests.

C. Manifest/package-content qualification.

D. MSRV/dependency/security review.

E. Cross-platform CI/check expansion.

F. Publication-readiness report without publishing.

## 7. Failure/recovery semantics

Examples must explicitly demonstrate that a rolled-back or recovery-required terminal result is not equivalent to success.

Documentation must state the exact limits of crash durability and stale-lock recovery.

## 8. Compatibility and migration

There are still no consumers. This is the final preferred point for pre-adoption breaking API cleanup.

Any breaking issue discovered during qualification should trigger a corrective plan rather than be papered over with compatibility aliases.

## 9. Required tests and verification

Run M005's full verification plus:

```bash
cargo +1.89.0 package -p eggup-core --locked
cargo +1.89.0 publish -p eggup-core --dry-run --locked
cargo +1.89.0 test --doc -p eggup-core --locked
```

Inspect the package file list and record platform CI.

## 10. Acceptance criteria

- package and publish dry-run succeed;
- crate name is verified available or an explicit naming decision is recorded;
- public docs/examples match actual safety semantics;
- dependency surface remains intentionally small;
- supported-platform claims match evidence;
- no known high/medium safety contract defect remains;
- first consumer may depend on a versioned package/API without using private modules.

## 11. Stop conditions

Stop if package naming is unavailable, cross-platform compile reveals architecture breakage, or qualification discovers another safety/API issue requiring correction.

## 12. Closure evidence required

- package file listing;
- dry-run outputs;
- public API review notes;
- direct/transitive dependency summary;
- platform matrix;
- MSRV result;
- crate-name availability result;
- unresolved limitations;
- recommendation on whether publication/adoption may proceed.
