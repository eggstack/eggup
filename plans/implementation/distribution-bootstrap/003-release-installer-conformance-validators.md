# Distribution and Bootstrap Milestone 003 — Release and Installer Conformance Validators

Status: ready for handoff

Repository baseline: `4495df6241b3fac9e396553727cf8d3d497ff3cd`

Source roadmap:

- `plans/subsystems/distribution-bootstrap-roadmap.md`

Primary class: capability / infrastructure

## 1. Objective

Turn corrected DistributionContract v1 into a deterministic conformance engine that can verify release inventories, archive-member inventories, and consumer-supplied bootstrap/runtime mapping observations without fetching releases, extracting archives, or parsing shell/PowerShell source.

M003 should make drift detectable. It must not become an installer generator or a GitHub-release client.

## 2. Readiness and dependencies

Hard dependencies are closed:

- distribution M001 schema;
- distribution M002 strict template/uniqueness corrective.

The v1 contract is now safe to use as the expected-name authority.

Interface evidence already exists for three layout classes:

- simple direct binary;
- CodeGG-like multi-file bundle;
- Egress-like archive with multiple required members.

M004 generators/adoptions remains blocked until this validator surface closes.

## 3. Current evidence

`eggup-dist` currently provides:

- strict TOML v1 parsing;
- deterministic target/alias resolution;
- direct/bundle/archive expansion;
- strict small template grammar;
- traversal-free literal archive member sources;
- portable release/install filename uniqueness.

It deliberately has no:

- release inventory type;
- conformance report;
- archive observed-member validator;
- runtime/bootstrap observation format;
- CLI;
- network/archive/shell execution.

Real consumers duplicate target/asset/checksum tables between Rust runtime code, release workflows, and shell/PowerShell installers. M003 should provide a machine-readable seam those consumer-specific tests can feed.

## 4. Invariants

- validators are pure with respect to network, subprocesses, archive extraction, and installation;
- the DistributionContract remains the expected-state authority;
- observed inventories are data supplied by callers/tests, not discovered through hidden I/O;
- validators never guess targets, versions, aliases, archive members, or filenames;
- missing required files/mappings are hard conformance failures;
- unrelated release extras are not automatically failures unless the caller explicitly requests exact-set validation;
- duplicate observed names fail rather than collapse;
- portable ASCII-case collision rules match M002;
- archive member paths are normalized/validated with the same safety rules as schema members;
- reports are deterministic, bounded, and suitable for CI;
- no shell/PowerShell parser is introduced;
- integrity/authenticity semantics remain distinct: this milestone checks naming/presence/mapping, not checksum bytes or signatures.

## 5. Scope and non-scope

### In scope

- typed release inventory;
- expected-release-file derivation from an expanded target;
- missing/duplicate/unexpected release-file reporting;
- caller-selectable extras policy;
- typed archive member inventory and required-member conformance;
- typed observed runtime/bootstrap mapping model;
- mapping conformance against target/asset/sidecar/install/member expectations;
- deterministic conformance report/error taxonomy;
- serialization format for observation fixtures (prefer TOML/JSON only if needed by CI);
- small `eggup-dist` CLI only if it materially improves CI use and stays a thin wrapper around pure library functions;
- golden fixtures for simple/CodeGG/Egress layouts;
- package/MSRV qualification.

### Out of scope

- HTTP/GitHub/crates.io API calls;
- reading a live GitHub release;
- SHA-256 computation or verification;
- archive extraction/listing by spawning tar/zip;
- parsing arbitrary shell/PowerShell/Rust source;
- installer generation;
- installing files;
- signing/authenticity;
- version selection/order;
- release publication;
- consumer adoption itself.

## 6. Required production changes

### A. Expected release file model

Add a typed API that derives all required release-side filenames for one concrete `target_or_alias + version`.

For direct:

- asset;
- checksum sidecar.

For bundle:

- each asset;
- each checksum sidecar.

For archive:

- archive asset;
- checksum sidecar.

Return stable logical labels for diagnostics, not only raw strings.

Reuse M002 collision rules; this API must not create a second name-expansion implementation.

### B. ReleaseInventory

Introduce a bounded observed inventory type constructed from caller-supplied names.

Construction MUST:

- reject empty/overlong/control/separator-containing names;
- reject exact and ASCII-case duplicate entries;
- retain deterministic order or sort deterministically.

No paths/directories are accepted; release assets are flat names.

### C. Release conformance

Provide pure validation:

`validate_release_inventory(contract, target, version, inventory, policy)` or equivalent.

At minimum report:

- missing required expected files;
- duplicate/invalid observed names at construction;
- unexpected extras when exact-set policy is requested.

Extras policy should be deliberately small:

- `AllowExtras` default for real releases that also contain source archives/signatures/docs;
- `Exact` for tightly controlled fixture/release checks.

Do not create a general filtering language.

### D. ArchiveMemberInventory

Define caller-supplied observed member paths for one archive.

Apply the same path safety normalization as schema member sources:

- relative;
- forward slashes;
- no absolute/drive prefix;
- no empty, `.`, or `..` components;
- no duplicate normalized/case-colliding entries according to the documented portable rule.

The validator checks that every required schema member source is present.

Extras MAY be allowed by policy; never infer install destination from an undeclared member.

M003 does not open archives. Consumer/release tooling passes the listing in.

### E. Runtime/bootstrap observation model

Define a small product-independent conformance observation that represents what a runtime updater/bootstrap installer claims for one target.

It should carry only contract-relevant facts, for example:

- target/triple or alias used;
- release asset filenames;
- checksum sidecar filenames;
- install names;
- archive member source -> install-name pairs where applicable.

It MUST NOT carry:

- URLs;
- package-manager fallback;
- privilege commands;
- shell snippets;
- service policy.

Prefer one typed `ObservedTargetMapping` that can be serialized by consumer tests.

### F. Mapping conformance

Validate an observation against the contract expansion.

Require exact agreement for declared contract facts:

- canonical target;
- asset set;
- sidecar set;
- install names;
- archive required source/install mapping.

No nearest-target/alias fallback.

For bootstrap/runtime drift testing, the consumer is responsible for producing observations from its own code/script fixtures. Eggup compares data; it does not parse languages.

### G. Report model

Add a deterministic structured report.

Suggested categories:

- MissingReleaseFile;
- UnexpectedReleaseFile;
- TargetMismatch;
- AssetMismatch;
- SidecarMismatch;
- InstallNameMismatch;
- MissingArchiveMember;
- UnexpectedArchiveMember (exact policy only);
- ArchiveMappingMismatch.

A successful report contains no findings.

Findings must be bounded and sorted deterministically so CI golden output is stable.

Prefer report + convenience `is_conformant` / `into_result` rather than one opaque string error.

### H. Fixture format

If a serialized observation fixture is useful, choose one simple format and document it.

Preferred:

- TOML for hand-reviewed checked-in fixtures; or
- JSON if consumer-generated CI output is materially simpler.

Do not support multiple formats in M003 unless there is real consumer evidence.

### I. Optional thin CLI

A CLI is optional, not mandatory.

Add it only if it can remain a thin wrapper such as:

```text
eggup-dist check-release --contract ... --target ... --version ... --inventory ...
eggup-dist check-mapping --contract ... --observation ...
```

The CLI MUST read local files/stdin only. No network, archive subprocess, shell execution, or release discovery.

If a library API plus integration tests provide the required value with less surface, defer CLI to M004.

## 7. Ordered work packages

A. Define expected-file and conformance report types.

B. Implement `ReleaseInventory` and release validation.

C. Implement `ArchiveMemberInventory` and member validation.

D. Implement observed target mapping + conformance.

E. Add simple direct, CodeGG bundle, and Egress archive positive/negative fixtures.

F. Decide and document whether a thin local-only CLI is justified; implement only if evidence is clear.

G. Package/MSRV/docs qualification and dependency audit.

H. Update roadmap/registry/closure.

## 8. Failure, restart, and contention semantics

This is deterministic release-time tooling.

On invalid input or conformance mismatch:

- return structured findings/error;
- perform no mutation;
- do not rewrite observations/contracts;
- do not fetch missing data;
- do not infer a different target/version.

No restart/contention semantics apply.

Large untrusted fixture/inventory input must be bounded in entry count and string lengths.

## 9. Compatibility and migration

`eggup-dist` remains unpublished at the current baseline, so additive M003 APIs may be designed cleanly.

Do not change valid schema-v1 interpretation.

M003 should be additive over M002.

If validating a real CodeGG/Egress layout requires altering schema semantics, stop and determine whether that is a schema defect/corrective rather than hiding it in validator policy.

## 10. Required tests

Release inventory:

- simple direct complete;
- missing asset;
- missing sidecar;
- extra asset allowed;
- extra asset rejected in Exact mode;
- duplicate exact/case-only observed names;
- invalid flat filename;
- bundle all required;
- bundle one member missing;
- archive asset + sidecar.

Archive inventory:

- all required members;
- missing member;
- extra member allowed/exact;
- traversal/absolute/backslash rejected;
- duplicate exact/case-only member;
- nested valid member path.

Mapping:

- exact direct mapping;
- wrong target;
- alias resolves to canonical target;
- wrong asset;
- wrong sidecar;
- wrong install name;
- CodeGG multi-entry exact mapping;
- Egress archive source/install pair mismatch;
- deterministic finding ordering.

Boundaries:

- large inventory count rejected;
- overlong labels rejected;
- no runtime crate dependency on `eggup-dist`;
- no network/process dependency introduced.

## 11. Verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-dist --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo package -p eggup-dist --locked --allow-dirty
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-dist --all-targets --locked
./scripts/check-local.sh
```

Hosted Linux/macOS/Windows-check must remain green.

## 12. Documentation updates

Update:

- `crates/eggup-dist/README.md`;
- public rustdoc/examples;
- `CHANGELOG.md`;
- distribution roadmap;
- registry.

Document clearly how consumer scripts/runtime code produce observations without Eggup parsing their source.

## 13. Acceptance criteria

M003 closes only when:

- required release-file completeness is deterministically validated for all three layout classes;
- archive required-member presence is validated without extraction ownership leaking into Eggup;
- runtime/bootstrap observations can be compared exactly to corrected schema v1;
- reports are typed, bounded, deterministic;
- extras policy is explicit and small;
- no shell/source parser, network client, extractor, or generator is introduced;
- valid M001/M002 fixtures remain green;
- package/MSRV/hosted CI passes;
- no medium-or-higher conformance ambiguity remains.

## 14. Stop conditions

Stop and write a schema/ADR corrective if:

- real consumer mappings cannot be represented without parsing arbitrary code;
- expected release names cannot be derived uniquely from v1;
- archive conformance requires Eggup to own extraction safety/execution;
- consumers require a general policy/filter DSL;
- validator semantics would change release/version authority.

## 15. Closure evidence required

Record:

- implementation SHA;
- public API inventory;
- release validation matrix;
- archive validation matrix;
- mapping validation matrix;
- fixture/golden outputs;
- extras-policy decision;
- CLI decision and rationale;
- dependency/package/MSRV results;
- proof runtime crates remain independent;
- hosted CI;
- unresolved findings.

## 16. Handoff notes

M003 closure unlocks:

- distribution M004 generator/templates/adoptions;
- stronger CodeGG M005 planning evidence;
- stronger Egress M006 planning evidence.

Do not author M004 generators until M003 closure confirms the observation/report model is sufficient.
