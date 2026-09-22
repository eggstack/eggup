# Distribution and Bootstrap Milestone 001 — Versioned DistributionContract Schema

Status: implemented (closed; see `plans/closure/distribution-bootstrap/001-status.md`)

Repository baseline: `8f6ce48cda5bdeb593939077bcca452cdd5f2800`

Source roadmap:

- `plans/subsystems/distribution-bootstrap-roadmap.md`

Consumer evidence:

- two completed Eggup single-binary adopters: eggsact and stegoeggo;
- eggsearch release/bootstrap layout;
- CodeGG multi-runfile installer layout;
- Egress archive/two-binary release layout.

Long-term requirements:

- `plans/000-long-term-specification.md#14-bootstrap-installers`
- `plans/000-long-term-specification.md#42-eggup-dist`

Applicable ADR:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: infrastructure / invariant

## 1. Objective

Create the first versioned, machine-readable `DistributionContract` model and parser/validator for target, asset, checksum, archive/member, and install-name mappings.

The contract is release-time tooling. It does not fetch releases, publish packages, render complete installers, or become a runtime dependency of ordinary consumers.

M001 is schema/model work only: prove one contract can express the real single-binary, bundle, and archive layouts already present across Eggstack.

## 2. Readiness and dependencies

The roadmap required first consumer evidence. That gate is satisfied:

- eggsact uses Eggup 0.1.0;
- stegoeggo independently uses the same core/acquisition contracts;
- no consumer-specific leakage was required.

CodeGG and Egress already supply the multi-member/archive shapes necessary to avoid designing the schema only around single executables.

No service adapter or acquisition corrective is a hard dependency because this milestone does not perform network or live installation.

## 3. Current evidence

Release/install policy is currently duplicated across runtime code, shell/PowerShell installers, and CI:

- target triples and release-platform aliases;
- binary names;
- archive names;
- checksum sidecar names;
- archive member names;
- install-time executable names;
- bundle membership.

The known layout classes are:

1. single direct binary + sidecar checksum;
2. multiple direct runfiles forming one logical release;
3. archive containing multiple required binaries;
4. platform-specific asset aliases/names.

The schema must capture these without deciding release version order, GitHub authority, Cargo fallback, service policy, or installation directories.

## 4. Invariants that must not regress

- schema version is explicit;
- target mappings are deterministic;
- unsupported targets fail instead of guessing;
- one logical artifact/member has one unambiguous source mapping per target;
- archive member paths cannot escape extraction roots;
- checksum naming is explicit/derivable without claiming authenticity;
- product/release version semantics remain caller-owned;
- URLs/hosting remain outside the contract;
- installer privilege/fallback policy remains consumer-owned;
- `eggup-dist` is not a required runtime dependency of `eggup-core`.

## 5. Scope

### In scope

- new `crates/eggup-dist` development/release-time crate;
- schema version 1;
- typed Rust model;
- one checked-in human-readable serialization format;
- parse + structural validation;
- deterministic template/placeholder expansion for names;
- target and alias mappings;
- direct-binary asset form;
- multi-direct-member/bundle form;
- archive asset + required member form;
- checksum sidecar naming;
- install/display member names;
- representative fixtures for eggsact/stegoeggo, CodeGG, and Egress-style layouts;
- schema compatibility/version error handling;
- package/docs tests.

### Explicitly out of scope

- downloading release assets;
- querying GitHub/crates.io;
- release version selection;
- checksum computation/verification;
- archive extraction;
- actual installer generation;
- shell/PowerShell execution;
- release publication;
- signing/authenticity;
- service definitions;
- automatic consumer migration.

## 6. Required production changes

### A. Crate boundary

Add `eggup-dist` as a workspace crate.

It may depend on serialization/parser crates because it is tooling, but keep the graph proportionate.

It must not become a dependency of `eggup-core`, `eggup-acquisition`, or `eggup-service`.

### B. Schema version

Every serialized document must begin with an explicit schema version, initially 1.

Unknown future major schema versions fail with a typed unsupported-version error.

Within schema v1, optional fields must have deterministic defaults; do not silently reinterpret unknown fields if the chosen parser can reject them.

### C. Product identity

Represent an opaque product id and optional human-readable metadata only as needed for validation.

Do not encode SemVer rules, tag prefixes, "latest" authority, GitHub owner/repo, or package-manager fallback into the core schema.

Version is an input to expansion, not a parsed/ordered value.

### D. Target model

Each supported target entry should express:

- canonical Rust target triple;
- zero or more consumer/platform aliases;
- asset mapping;
- required artifact members;
- optional install/display name metadata.

Aliases must be globally unambiguous within the contract.

Reject duplicate triples and duplicate aliases mapping to different targets.

### E. Asset forms

Schema v1 must represent at least:

#### Direct asset

One release asset corresponds to one member.

#### Direct bundle

Multiple release assets together form one logical release.

#### Archive

One release asset contains multiple required members.

Do not create an overly generic recursive package language.

A small tagged enum such as `direct | bundle | archive` is preferable.

### F. Name templates

Support a deliberately small placeholder vocabulary grounded in current releases, for example:

- `{product}`;
- `{version}`;
- `{target}`;
- `{alias}` only if unambiguous and necessary.

Reject unknown placeholders.

Do not embed shell expressions, conditionals, arbitrary formatting code, or environment expansion.

### G. Checksum contract

Represent checksum sidecar/manifest naming separately from the artifact.

Initial schema should support exact SHA-256 sidecar names/templates.

The model must call this integrity metadata, never signature/authenticity.

### H. Archive member safety

For archive forms, member source paths must be relative, normalized, and traversal-free.

Reject:

- absolute paths;
- `..`;
- platform prefix/root components;
- empty member names;
- duplicate normalized member paths.

M001 does not extract archives; it validates the contract that later extraction must obey.

### I. Representative fixtures

Add canonical test fixtures modeling:

- a simple direct binary release;
- a CodeGG-like three-member logical bundle;
- an Egress-like archive or paired-binary release.

Use real naming patterns where stable, but keep product-specific fixtures under tests/examples rather than generic production constants.

### J. Serialization choice

Choose one checked-in format that is readable in repository review and stable for generators/CI.

TOML is preferred if it expresses the required v1 model cleanly; JSON is acceptable if TOML creates ambiguity for tagged asset forms.

Record the decision in the closure record. An ADR is not required unless the format creates a cross-version compatibility constraint beyond the v1 parser contract.

## 7. Ordered work packages

A. Census current release layouts and freeze v1 requirements.

B. Define typed schema model and error taxonomy.

C. Implement parser + strict structural validation.

D. Implement safe deterministic name-template expansion.

E. Implement direct/bundle/archive validation.

F. Add representative fixtures and round-trip/golden tests.

G. Package/docs/MSRV qualification.

## 8. Failure, cancellation, restart, and contention semantics

This milestone performs no network or live mutation.

Parsing/validation is synchronous and deterministic.

Invalid input:

- returns typed validation errors;
- never partially rewrites the source file;
- never attempts fallback to inferred targets/assets;
- never performs filesystem extraction.

If a later caller asks for a target/alias absent from the contract, return unsupported-target rather than guessing a nearby architecture.

## 9. Compatibility and migration

New crate; no consumer migration.

Schema v1 becomes a versioned file contract once a consumer checks one in. Therefore:

- field names and semantics require care;
- future incompatible changes use a new schema version or explicit migration;
- M001 should avoid fields that merely mirror one product's current script layout.

No existing installer is replaced in this milestone.

## 10. Required tests

- parse minimal valid v1;
- unknown schema version;
- duplicate target;
- duplicate/colliding alias;
- unsupported target lookup;
- unknown placeholder;
- missing placeholder input;
- direct single-member expansion;
- multi-direct bundle expansion;
- archive asset expansion;
- archive member absolute path;
- archive member traversal;
- duplicate normalized archive members;
- checksum sidecar naming;
- product version string treated opaquely;
- real-layout fixtures for simple, CodeGG-like, Egress-like releases;
- deterministic serialized/golden output if round-trip writing is exposed.

## 11. Required verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-dist --locked
cargo package -p eggup-dist --locked --allow-dirty
./scripts/check-local.sh
```

Run Rust 1.89 variants.

No network-dependent test is permitted for schema qualification.

## 12. Documentation updates

Add:

- `crates/eggup-dist/README.md`;
- schema-v1 reference;
- examples for direct, bundle, and archive layouts;
- explicit non-goals around version/release/hosting/install policy;
- roadmap and registry updates;
- changelog entry.

## 13. Acceptance criteria

M001 closes when:

- one strict versioned schema expresses all three required layout classes;
- target/alias mappings are unambiguous;
- unsafe archive member paths fail;
- name expansion is small and deterministic;
- checksum metadata is represented truthfully;
- fixtures prove the model is not single-consumer-specific;
- no runtime Eggup crate depends on `eggup-dist`;
- package/MSRV/CI pass;
- M002 release/installer conformance validators can be planned against a stable v1 model.

## 14. Stop conditions

Stop and write an ADR or revise the roadmap if:

- real CodeGG/Egress layouts require arbitrary scripting in the schema;
- archive semantics require implementing extraction to define the model;
- product release authority/version ordering becomes necessary to express assets;
- target aliases cannot be made deterministic;
- serialization format limitations force a materially different public data model.

## 15. Closure evidence required

- v1 schema reference;
- representative fixture inventory;
- target/alias collision matrix;
- archive-path negative tests;
- package/dependency tree;
- proof runtime crates do not depend on dist;
- exact verification commands/results;
- serialization-format decision;
- unresolved findings.

## 16. Handoff notes

This milestone is safe to run in parallel with service M002 and after/beside acquisition M003 because it is release-time modeling only.

Do not generate installers or validate live GitHub releases yet; those belong to distribution M002.
