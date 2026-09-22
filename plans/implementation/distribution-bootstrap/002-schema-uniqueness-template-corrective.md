# Distribution and Bootstrap Milestone 002 — Schema Uniqueness and Template Grammar Corrective

Status: ready for handoff

Repository baseline: `889a234cbe7f461d92def3df45c83c06a7d257e5`

Source roadmap:

- `plans/subsystems/distribution-bootstrap-roadmap.md`

Corrects post-closure findings from:

- `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md`
- `plans/closure/distribution-bootstrap/001-status.md`

Primary class: invariant / corrective

## 1. Objective

Tighten DistributionContract v1 before building release/installer validators on top of it.

M001 correctly validates target/alias identity and archive source traversal, but it does not yet guarantee that distinct bundle/archive declarations remain unambiguous after template expansion for a concrete target/version.

This corrective must:

1. reject duplicate/colliding expanded release asset names and checksum sidecars;
2. reject duplicate/colliding expanded install names;
3. validate template brace/placeholder grammar at contract parse time rather than deferring malformed forms until expansion.

## 2. Readiness and dependencies

Distribution M001 is closed and `eggup-dist` is still unpublished/unadopted.

This is the lowest-cost point to tighten v1 before any consumer checks a schema into a release workflow.

Release/installer conformance validators are blocked until this corrective closes.

## 3. Current evidence and detection gap

At the baseline:

- duplicate target triples and aliases are rejected;
- duplicate literal archive source paths are rejected;
- bundle entry `asset`/`install` templates are individually syntax-checked for allowed characters/placeholders;
- archive member install templates are individually checked;
- expansion simply pushes entries/members into vectors.

There is no post-expansion uniqueness check for:

- bundle `asset_file`;
- bundle `sidecar_file`;
- bundle `install_name`;
- archive member `install_name`;
- cross-namespace release filename collision such as one entry's asset equaling another entry's checksum sidecar.

Therefore a structurally valid contract can resolve to ambiguous release or install mappings for a particular version/alias.

`placeholders_in` also tolerates unmatched braces during initial validation; some malformed templates fail only after expansion.

Detection gap: M001 fixtures used naturally unique names and negative tests focused on target aliases and archive traversal, not expanded namespace collisions.

## 4. Invariants that must not regress

- one concrete target/version expansion has an unambiguous release-file namespace;
- one concrete target/version expansion has an unambiguous install-name namespace;
- every direct/bundle/archive member maps deterministically;
- checksum sidecars cannot alias another release asset/sidecar;
- archive source paths remain unique and traversal-free;
- malformed template grammar fails when the contract is parsed/validated, not only when a particular expansion happens;
- product version remains opaque except for filesystem safety;
- no network/extraction/installer generation is introduced;
- runtime crates remain independent of `eggup-dist`.

## 5. Scope and non-scope

### In scope

- strict template lexer/parser for the small v1 placeholder grammar;
- unmatched/empty/nested/stray brace rejection;
- expansion-time release namespace uniqueness;
- expansion-time install namespace uniqueness;
- collision error taxonomy;
- case/cross-platform collision policy;
- negative fixtures/tests;
- schema-v1 documentation clarification.

### Out of scope

- release HTTP lookup;
- actual checksum verification;
- archive extraction;
- installer generation;
- signing;
- target auto-detection;
- version ordering;
- schema v2;
- consumer migration/publication.

## 6. Required production changes

### A. Strict v1 template grammar

Replace ad-hoc placeholder scanning with one deterministic parser/validator for file-name templates.

Allowed forms:

- literal safe filename characters;
- exact placeholders `{product}`, `{version}`, `{target}`, `{alias}`;
- `{asset}` only in checksum-sidecar context.

Reject at parse time:

- unmatched `{`;
- stray `}`;
- empty `{}`;
- nested braces;
- whitespace/control characters in placeholder names;
- unknown placeholders;
- shell/env syntax;
- escaped/double-brace forms unless explicitly supported by v1 (preferred: reject).

Do not add a general templating language.

### B. Release-file namespace uniqueness

After expansion for one target/version/lookup alias, build the complete set of release-side file names.

For Direct:

- asset and checksum sidecar must differ.

For Bundle:

- every `asset_file` unique;
- every `sidecar_file` unique;
- no sidecar equals any asset;
- no duplicate release names across entries.

For Archive:

- archive file and checksum sidecar differ.

Return a typed/bounded collision error naming the conflicting logical fields without embedding arbitrary unbounded input.

### C. Install namespace uniqueness

For Bundle:

- all expanded `install_name` values must be unique.

For Archive:

- all expanded member `install_name` values must be unique.

Direct has one install name.

The contract must never require a downstream installer to decide which duplicate member "wins".

### D. Portable collision key

Define a deterministic v1 collision rule suitable for cross-platform release contracts.

Preferred conservative rule:

- exact-name collisions always fail;
- ASCII case-fold collisions also fail for release and install filenames.

This prevents contracts that are distinct on a case-sensitive Linux checkout but collide on common Windows/macOS filesystems.

Do not attempt locale-dependent Unicode case folding; current filename inputs are already constrained.

If maintainers choose target-specific case handling instead, document the rule and prove Windows targets reject case-insensitive collisions. The portable conservative rule is preferred for v1 simplicity.

### E. Expansion remains the final authority

Some collisions depend on `{version}`, `{target}`, or `{alias}`, so parse-time validation cannot prove all uniqueness.

`DistributionContract::expand` MUST run uniqueness checks before returning `ExpandedTarget`.

A caller must never receive an ambiguous expanded contract.

### F. Parse-time cheap duplicate checks

Where two raw templates are byte-identical in the same namespace, reject during contract construction as early feedback.

This is supplementary; expansion-time checks remain required.

## 7. Ordered work packages

A. Add strict template grammar parser/tests.

B. Add typed collision helper/error.

C. Enforce release namespace uniqueness during expansion.

D. Enforce install namespace uniqueness during expansion.

E. Add portable case-fold collision policy/tests.

F. Re-run simple/CodeGG/Egress fixtures and deterministic round-trip tests.

G. Update README/schema reference/changelog/roadmap/registry.

## 8. Failure, restart, and contention semantics

This crate remains pure deterministic parse/expand tooling.

Invalid grammar/collision:

- returns typed `DistError`;
- produces no partial `ExpandedTarget`;
- performs no filesystem/network mutation;
- never guesses/renames/deduplicates automatically.

No restart/contention semantics apply.

## 9. Compatibility and migration

`eggup-dist` is unpublished and no consumer has adopted schema v1, so tightening v1 is appropriate.

Do not bump to schema v2 solely for rejecting inputs that should have been invalid under M001's "unambiguous mapping" invariant.

Existing valid M001 fixtures must continue to parse/expand unchanged.

If a real current Eggstack release layout relies on a newly rejected collision, stop and inspect that layout rather than weakening uniqueness.

## 10. Required tests

### Template grammar

- unmatched opening brace;
- stray closing brace;
- empty placeholder;
- nested placeholder;
- unknown placeholder;
- `{asset}` outside sidecar;
- valid repeated known placeholders if semantically harmless;
- literal braces rejected.

### Release collisions

- bundle duplicate exact asset;
- bundle assets collide only after version expansion;
- two sidecars collide due constant sidecar template;
- one sidecar collides with another asset;
- direct asset == sidecar;
- archive asset == sidecar;
- ASCII-case-only collision.

### Install collisions

- bundle duplicate install name;
- bundle case-only install collision;
- archive members duplicate install name;
- archive members case-only install collision;
- collision appears only after placeholder expansion.

### Regression

- simple-direct fixture;
- CodeGG-like three-member fixture;
- Egress-like archive fixture;
- round-trip deterministic output;
- alias lookup;
- traversal negative tests.

## 11. Verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo package -p eggup-dist --locked --allow-dirty
./scripts/check-local.sh
```

Run Rust 1.89 and hosted CI.

## 12. Documentation updates

Update:

- `crates/eggup-dist/README.md`;
- schema-v1 reference;
- rustdoc for expansion and errors;
- `CHANGELOG.md`;
- distribution roadmap;
- registry.

Document:

- strict placeholder grammar;
- release filename uniqueness;
- install filename uniqueness;
- portable case-collision rule;
- uniqueness being checked after expansion.

## 13. Acceptance criteria

M002 closes only when:

- malformed template grammar fails at parse/validation time;
- no expanded release filename collision can be returned;
- no expanded install-name collision can be returned;
- sidecars cannot alias assets or each other;
- case-only portable collisions fail according to the documented rule;
- all M001 fixtures remain valid;
- runtime crates still do not depend on dist;
- CI/MSRV pass;
- no medium-or-higher schema ambiguity remains.

## 14. Stop conditions

Stop and revise the schema/ADR if:

- a real CodeGG/Egress layout cannot be represented without intentional filename collision;
- uniqueness requires target-filesystem behavior that cannot be expressed deterministically;
- strict grammar requires introducing general template escaping/conditionals;
- the correction would change release/version authority.

## 15. Closure evidence required

Record:

- exact implementation SHA;
- grammar-negative matrix;
- release collision matrix;
- install collision matrix;
- case-fold policy and tests;
- fixture regression results;
- package/dependency tree;
- hosted/MSRV CI;
- unresolved findings.

## 16. Handoff notes

This corrective is the hard gate before distribution release/installer validators.

Future distribution milestones are renumbered:

- validators -> M003;
- generators/two-consumer adoption -> M004.
