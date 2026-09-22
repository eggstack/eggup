# Distribution and Bootstrap M002 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md`

Source roadmap: `plans/subsystems/distribution-bootstrap-roadmap.md#M002--schema-uniqueness-and-template-grammar-corrective`

Reviewed repository baseline: `889a234cbe7f461d92def3df45c83c06a7d257e5`

Implementation commit: `0a68f29fce44adf5f12d79f1b440a2c08aca9cb7` (`fix: validate distribution templates and names`).

## Executive finding

M002 is complete. Schema v1 now parses file-name templates with one strict
grammar at contract validation time, and expansion rejects every collision
within release-file and install-name namespaces. Collision keys use
locale-independent ASCII lowercasing, so exact and ASCII-case-only collisions
are rejected portably. Valid direct, CodeGG-like bundle, and Egress-like
archive fixtures still expand deterministically. `eggup-dist` remains a
release-time crate with no runtime dependency from core/acquisition/service.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Strict parse-time template grammar | `parse_template`; malformed unmatched, stray, empty, nested/double brace, whitespace, literal-space, unknown, and context-invalid `{asset}` cases | passed |
| Repeated valid placeholders | `repeated_known_placeholders_remain_valid` | passed |
| Bundle release asset uniqueness | exact raw duplicate rejected during parse; `{version}` expansion collision rejected | passed |
| Sidecar uniqueness | constant bundle sidecar collision rejected | passed |
| Cross-collision between sidecar and asset | bundle sidecar `first` colliding with another expanded asset rejected | passed |
| Direct and archive asset/sidecar difference | direct same-name output and archive `{asset}` sidecar rejected | passed |
| Install-name uniqueness | duplicate bundle/archive names rejected during parse; version-dependent bundle collision rejected during expansion | passed |
| Archive install-name collision | exact raw collision and post-expansion ASCII case-only collision rejected | passed |
| Portable collision behavior | release asset/sidecar case-only collision plus bundle/archive install case-only collision rejected | passed |
| Typed bounded collision error | `DistError::NameCollision` reports `NameNamespace` and bounded logical field labels, not arbitrary filenames | passed |
| Expansion final authority | `DistributionContract::expand` validates all expanded names before returning `ExpandedTarget` | passed |
| M001 fixture and serialization regression | simple direct, CodeGG bundle, Egress archive, alias lookup, traversal and deterministic round-trip tests | passed |
| Runtime dependency boundary | no dependency from `eggup-core`, acquisition, or service to `eggup-dist` | passed |

## Production implementation evidence

- `parse_template` accepts only safe ASCII filename literals and exact
  placeholders `{product}`, `{version}`, `{target}`, `{alias}`, with `{asset}`
  limited to checksum-sidecar context. It rejects malformed braces, nesting,
  double-brace forms, malformed placeholder names, unsupported syntax, and
  unknown/context-invalid placeholders during contract construction.
- `DistributionContract::expand` constructs complete release and install
  namespaces, applies ASCII case folding, and returns no `ExpandedTarget` if
  any two logical fields collide.
- Bundle assets and install templates, plus archive install templates, receive
  cheap raw duplicate checks during construction. Expansion-time checks remain
  authoritative for placeholder-dependent collisions.
- `NameCollision` is typed with `NameNamespace::{ReleaseFiles, InstallNames}`
  and bounded logical field labels. No arbitrary input filename is embedded
  in this error.
- No filesystem, network, extraction, installer generation, or runtime
  dependency was added.

## Exact commands and results

Environment: Darwin 25.6.0 arm64; stable `rustc 1.98.1`; installed MSRV
`1.89.0`.

Passed:

```text
cargo fmt --all
cargo test -p eggup-dist --locked
cargo +1.89.0 test -p eggup-dist --locked
cargo clippy -p eggup-dist --all-targets --all-features --locked -- -D warnings
cargo doc -p eggup-dist --no-deps --locked
cargo package -p eggup-dist --locked --allow-dirty
git diff --check
```

The stable and Rust 1.89 crate suites each reported 24 unit tests and 4
fixture tests passing, 0 failed. Package verification passed (Cargo warned
that its `tests/fixtures.rs` integration target is not included in the package;
the library package itself compiled successfully).

An initial stable test run caught an incorrect test fixture expectation for
which `bundle_with` argument represented install names; the test was corrected
and the full stable suite rerun successfully. No production behavior change
was needed for that test correction.

Hosted Linux/macOS/Windows CI was not available before push. The push for this
work wave will trigger repository CI; its result is recorded in the final
handoff if accessible. Native platform behavior is not applicable to this
deterministic parse/expand-only crate.

## Invariant review

- A concrete target/version expansion never contains ambiguous release or
  install filenames.
- Sidecars share a release namespace with assets and cannot alias them.
- ASCII case collisions fail independently of host filesystem case rules;
  no locale-sensitive or Unicode normalization was introduced.
- Unknown/malformed template syntax cannot be deferred to a later expansion.
- Expansion remains pure and all-or-nothing; no fallback, rename, or
  deduplication is performed.
- Product version stays opaque subject to existing filesystem safety rules.
- Runtime crates remain independent of `eggup-dist`.

## Failure and recovery review

Malformed grammar returns `InvalidInput` or `UnknownPlaceholder` during
contract parse. Namespace collisions return `NameCollision` either during
cheap raw duplicate validation or during expansion. Both failures return no
partial expanded contract and perform no external mutation. Valid existing
fixtures preserve their names and deterministic serialization.

## Compatibility and migration review

`eggup-dist` remains unpublished and no consumer has adopted schema v1, so
the schema was tightened without a version bump. Existing valid M001 fixtures
are unchanged. Contracts that depended on malformed template forms or
colliding names now fail closed; none of the real simple/CodeGG/Egress fixture
layouts does so. No consumer migration or publication occurred.

## Security review

Templates remain a small data substitution grammar, not a shell or general
template language. Literal unsafe syntax is rejected; all output remains a
flat filename. Collision checks prevent downstream validators/installers from
choosing an ambiguous winner. No extraction or execution behavior was added.

## Documentation and operations evidence

- `crates/eggup-dist/README.md` documents grammar, allowed literal characters,
  uniqueness namespaces, and ASCII case-fold policy.
- `DistributionContract::expand`, `DistError::NameCollision`, and
  `NameNamespace` have public rustdoc.
- `CHANGELOG.md`, the distribution roadmap, registry, and closure index are
  updated.
- `cargo doc` and package verification pass.

## Unresolved findings

- Informational: hosted CI is pending push.
- None: no medium-or-higher schema ambiguity or template-grammar issue remains.

## Disposition and roadmap transition

M002 is closed. Distribution M003 release and installer validators are now
dependency-ready for implementation-plan authoring. Distribution M004
generators/adoptions remains blocked on M003 validator evidence. CodeGG/Egress
adoption planning remains later work on that M003 evidence; this corrective
does not start consumer migration.

## Registry updates

- Distribution M002 → closed.
- Distribution M003 validators → unblocked; author the bounded implementation
  plan against corrected schema v1.
- Distribution M004 generators/adoptions → remains blocked on M003.
