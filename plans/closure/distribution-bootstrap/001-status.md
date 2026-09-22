# Distribution and Bootstrap M001 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md`

Source roadmap: `plans/subsystems/distribution-bootstrap-roadmap.md#M001--versioned-distributioncontract-schema`

Reviewed repository baseline: `8f6ce48cda5bdeb593939077bcca452cdd5f2800` (plan baseline; implementation ran at `892d6cc` plus dist changes)

## Implementation commits/PRs

- M001 dist commit (this pass): `feat: add versioned distribution contract schema (M001)` (pending SHA; see git log)
- Prior: `892d6cc` (`planning: register post-adoption implementation wave`)
- No PR was required for this local implementation pass. No publication was performed.

## Executive finding

M001 is complete. One strict versioned schema expresses the three required
layout classes (single direct binary, multi-direct bundle, archive with
members) without release/version/hosting/install policy. Target/alias
mappings are unambiguous, archive member paths are traversal-free, name
expansion is small and deterministic, checksum metadata stays
integrity-only, and fixtures prove the model is not single-consumer-specific.
No runtime Eggup crate depends on `eggup-dist`.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| New `eggup-dist` workspace crate, tooling-only | `crates/eggup-dist`; workspace member; `serde` + `toml 0.8` only | passed |
| No runtime dependency on dist | `cargo tree -p eggup-core/acquisition/service` contain zero `eggup-dist` lines | passed |
| Explicit schema version 1 | `SCHEMA_V1 = 1`; `schema_version` required; `UnsupportedVersion{found}` for v2 | passed |
| Unknown fields rejected | `#[serde(deny_unknown_fields)]` on all raw shapes; `unknown_fields_are_rejected` | passed |
| Opaque product id, no SemVer/tag/latest/GitHub/fallback | `ProductIdentity{id}` validated as opaque; version is expansion input only, never ordered | passed |
| Canonical triple + aliases, unambiguous | `validate_triple`/`validate_alias`; duplicate triple/alias/triple-collision rejected; `resolve` fails `UnknownTarget` | passed |
| Direct asset form | `AssetForm::Direct`; `direct_single_member_expansion` + simple fixture | passed |
| Bundle form | `AssetForm::Bundle`; `multi_direct_bundle_expansion` + CodeGG fixture (3 entries) | passed |
| Archive form | `AssetForm::Archive`; `archive_asset_expansion` + Egress fixture (2 members) | passed |
| No generic recursive package language | Fixed `direct\|bundle\|archive` via explicit `kind` tables | passed |
| Small placeholder vocabulary | `{product}/{version}/{target}/{alias}` (+ `{asset}` for sidecars); `unknown_placeholder_is_rejected` | passed |
| No shell/conditionals/env expansion | Literal `replace` only; separators rejected; `unknown_fields` + template tests | passed |
| Checksum sidecar, integrity-only | `ChecksumSpec{sidecar}`; `checksum_sidecar_naming_is_explicit`; docs never claim authenticity | passed |
| Archive member safety | `validate_member_source` (relative, no `..`/abs/empty/drive/backslash, literals only); absolute/traversal/duplicate tests | passed |
| Representative fixtures | `simple-direct.toml`, `codegg-bundle.toml`, `egress-archive.toml` + integration tests | passed |
| Deterministic golden output | `deterministic_round_trip_golden` + `fixtures_round_trip_deterministically` | passed |
| Version opaque | `product_version_is_opaque_but_filesystem_safe` (dotted passthrough; `../`, `a/b`, empty rejected) | passed |
| Unsupported target fails, no guessing | `unsupported_target_lookup_fails_without_guessing` | passed |
| Missing `{alias}` input fails | `missing_alias_input_fails` (triple lookup fails, alias lookup succeeds) | passed |
| No network/extraction/installer generation | Sync parse/validate only; no `Command`, no net deps; `cargo tree -p eggup-dist` is `serde`+`toml` only | passed |

## V1 schema reference

```toml
schema_version = 1
[product]
id = "<opaque>"
[[targets]]
triple = "<canonical-triple>"
aliases = ["<alias>"]
[targets.asset]
kind = "direct|bundle|archive"
# direct: asset + install
# bundle: entries[] of asset + install
# archive: asset + members[] of source (literal) + install
[targets.checksum]
sidecar = "<template with {asset} allowed>"
```

Placeholders: `{product}`, `{version}`, `{target}`, `{alias}` everywhere;
`{asset}` additionally in `sidecar`. All names are flat file names (no `/`
or `\`). Archive `source` paths are literal relative `/`-separated paths.

## Representative fixture inventory

| Fixture | Layout class | Targets | Expansion proof |
|---|---|---|---|
| `simple-direct.toml` | single direct binary + sidecar (eggsact/stegoeggo-like) | linux-gnu + macos-arm64 | `simple_direct_fixture_expands` |
| `codegg-bundle.toml` | three-member logical bundle (CodeGG-like) | linux-gnu | `codegg_bundle_fixture_expands_three_members` |
| `egress-archive.toml` | archive with two required binaries (Egress-like) | linux-gnu + macos-arm64 | `egress_archive_fixture_expands_members` |

Product-specific names live under `tests/fixtures`, never as generic constants.

## Target/alias collision matrix

| Case | Evidence | Result |
|---|---|---|
| Duplicate triple | `duplicate_target_is_rejected` | passed |
| Duplicate alias to different targets | `duplicate_alias_is_rejected` | passed |
| Alias colliding with any triple | `alias_colliding_with_triple_is_rejected` | passed |
| Unknown triple/alias | `unsupported_target_lookup_fails_without_guessing` | passed |

## Archive-path negative tests

- Absolute (`/egress`): rejected.
- Traversal (`../evil`, `a/../../b`, `a/../b`, `bin/..`, `.`, `..`): rejected.
- Duplicate normalized members: rejected.
- Backslash, drive prefix, empty component, templates in source: rejected.

## Package/dependency tree

- `cargo tree -p eggup-dist`: 27 lines; `serde 1.0.229` + `toml 0.8.23` only (proportionate tooling graph).
- `cargo tree -p eggup-core/acquisition/service`: zero `eggup-dist` references (runtime independence proven).
- `cargo package -p eggup-dist --allow-dirty`: green (6 files, 48.1 KiB, 11.5 KiB compressed; `tests/` excluded by package rules with an informational warning, same as sibling crates).

## Proof runtime crates do not depend on dist

`cargo tree -p eggup-core --locked | grep -c eggup-dist` → 0 (same for
`eggup-acquisition`, `eggup-service`). `eggup-dist` is a workspace member but
appears in no runtime manifest.

## Exact verification commands/results

Environment: `Darwin 25.6.0 arm64`, stable `1.98.1`, MSRV `1.89.0`.

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-dist --locked
cargo package -p eggup-dist --locked --allow-dirty
./scripts/check-local.sh
```

MSRV variants (`cargo +1.89.0 …`) run for check/clippy/test/doc: green.

Results at closure: 18 `eggup-dist` unit + 4 fixture integration = 22 passed;
workspace totals 26 acquisition + 37 core + 23 eggfetch + 10 service + 22 dist
= 118 passed, 0 failed, on stable and 1.89. No network-dependent test exists
in `eggup-dist`.

## Serialization-format decision

**TOML** (via `toml 0.8`, `parse` + `display` features only). Readable in
repository review, stable for generators/CI, and clean for the tagged
`direct|bundle|archive` forms via explicit `kind` tables (no serde tag
ambiguity; manual `RawAsset{kind,…}` → `AssetForm` conversion with precise
errors). No ADR required: the format creates no cross-version constraint
beyond the v1 parser contract (future incompatibilities use a new schema
version). JSON was not needed.

## Invariant review

- Schema version explicit; future majors fail typed.
- Target mappings deterministic; unsupported fails, never guesses.
- One artifact/member has one unambiguous source mapping per target.
- Archive paths traversal-free (literals only).
- Checksum naming explicit/derivable, integrity-only.
- Product/version opaque; URLs/hosting outside contract.
- Privilege/fallback policy consumer-owned.
- `eggup-dist` not a runtime dependency.

## Failure/recovery review

Parse/validate is synchronous and deterministic with no network or live
mutation. Invalid input returns typed errors, never rewrites the source,
never falls back to inferred targets/assets, never extracts. Missing
target/alias returns `UnknownTarget`; missing template input returns
`MissingInput`; unsafe member returns `InvalidMemberPath`.

## Compatibility/migration review

New crate; no consumer migration. Schema v1 becomes a versioned file contract
once a consumer checks one in; field names/semantics were kept minimal to
avoid mirroring one product's script layout. No installer replaced.

## Security review

- No network, extraction, shell, or privilege surface in M001.
- Bounded inputs (64/128/256/512 chars); control characters rejected.
- Flat file names prevent directory creation; versions cannot escape via separators.
- Archive sources literal-only, preventing template-injected traversal.
- `#![forbid(unsafe_code)]`.

## Docs/operations evidence

- `crates/eggup-dist/README.md`: schema-v1 reference, asset forms, templates, target/archive/checksum rules, non-goals.
- Rustdoc on every public type/method; `cargo doc` clean on stable and 1.89 with `#![deny(missing_docs)]`.
- `CHANGELOG.md`: M001 entry + TOML decision.
- Fixtures checked in under `tests/fixtures` with real-layout naming.

## Unresolved findings with severity

- None (M001 scope). Validators (M002) and generators/adoptions (M003) explicitly deferred.
- Informational: `cargo package` notes `tests/fixtures.rs` excluded from the published crate (standard `exclude = ["tests/"]`, same as siblings).
- Informational: Hosted CI not run locally.

## Disposition and roadmap transition

M001 is closed. Unblocks distribution M002 (release/installer conformance
validators) planning against the stable v1 model. No service/acquisition
dependency was required and none is created.

## Registry updates

- Distribution M001 → closed.
- Distribution M002 → planning-ready (plan may now be authored).
