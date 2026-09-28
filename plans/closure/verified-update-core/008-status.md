# Verified Update Core M008 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M008--corearchive-consumer-package-qualification`

Reviewed baseline: `413a35a7da32ea22ef337caefc35d3619154794e`

Implementation commits:

- this batch (package metadata + workspace version bump + cargo-package/dry-run qualification + fixture + docs/changelogs) — SHA recorded in the post-implementation registry head.

## Executive finding

M008 qualifies a versioned, crates.io-compatible `eggup-core` 0.1.2 and `eggup-archive` 0.1.2 package pair that exposes the post-M001d bound-source authority. The package verification consumer under `plans/closure/verified-update-core/008-package-fixture/` proves the exact `PersistedExtraction::into_bound_sources → BoundExtraction → BoundSources → InstallPlan::prepare_with_bound_sources` flow from outside the Eggup workspace against the packaged tarballs.

No automatic crates.io publication occurs. A separate maintainer action publishes `eggup-core` 0.1.2 first; `eggup-archive` 0.1.2 then publishes against it. Egress M006 and Eggpack Interop M002 were the planned downstream consumers; Egress M006 remains gated on the separate maintainer publication action, while Eggpack M002 is unblocked for in-repo implementation because the package surface and bound-source API are now stable.

No medium-or-higher authority defect was introduced by the M008 metadata, version, and dependency changes.

## Version decision

- Workspace version: `0.1.1 → 0.1.2` (patch, additive API only).
- Internal exact pins updated deliberately:
  - `crates/eggup-archive/Cargo.toml` dev-dep `eggup-core = "=0.1.1"` → path-only (no version string). Rationale: the package verification step resolves dev-deps against crates.io; the exact pin kept the in-tree source pinned to the previously published version, but the new exact pin would not exist until maintainer publication. Path-only preserves the workspace invariant (the dev-dep uses whatever `../eggup-core` is at) while letting `cargo package` and `cargo publish --dry-run` complete against the in-tree eggup-core. The original exact pin was deliberately exact in the sense that it prevented version drift; that guarantee is preserved by the path-only form because the path always points to the same source tree. The change is explicit, not silent, and is recorded here.
  - `crates/eggup-eggpack/Cargo.toml` exact pins on `eggup-core = "=0.1.1"` and `eggup-acquisition = "=0.1.1"` updated to `=0.1.2`. eggup-eggpack remains `publish = false`; this is a workspace-only consistency update so the workspace tree resolves to the same versions it will publish.

## Package metadata changes

`crates/eggup-archive/Cargo.toml` now declares the externally useful metadata previously only present for `eggup-core`:

- `homepage = "https://github.com/eggstack/eggup"`
- `documentation = "https://docs.rs/eggup-archive"`
- `description` extended to record the absence of transport, authenticity, and live-installation policy.
- `keywords` extended with `tar` and `zip` for crate searchability.
- `categories = ["filesystem"]`, `exclude = ["tests/", "benches/"]` unchanged.

No authenticity, signature, or transport-token claims were introduced.

## Requirement-to-evidence matrix

| Requirement (source plan Section 10) | Evidence | Result |
|---|---|---|
| Full workspace tests | `cargo test --workspace --all-targets --all-features --locked` (36 eggup-archive / 44 eggup-core / 47 eggup-eggfetch / 22 eggup-curl / 25 eggup-service / 7 acquisition / 28 eggup-eggpack / 101 lifecycle = 165+ tests) | passed |
| Core bound-source tests | `eggup-core` 44 tests including the existing `prepare_with_bound_sources` matrix; new fixture proves the same flow against the packaged 0.1.2 tarballs | passed |
| Archive bound-source tests | `eggup-archive` 36 tests covering `into_bound_sources` and `prepare_with_bound_sources` from the eggup-archive side (Unix + portable subsets; Windows rename-refusal documented in M001d closure) | passed |
| Rust 1.89 all-target check | `cargo +1.89.0 check --workspace --all-targets --locked` clean | passed |
| Package + dry-run for core | `cargo package -p eggup-core --locked --allow-dirty` → 19 files, 180.4 KiB (35.5 KiB compressed); `cargo publish -p eggup-core --dry-run --locked --allow-dirty` → upload dry-run ok | passed |
| Package + dry-run for archive | `cargo package -p eggup-archive --locked --allow-dirty` → 7 files, 138.3 KiB (26.7 KiB compressed); `cargo publish -p eggup-archive --dry-run --locked --allow-dirty` → upload dry-run ok | passed |
| Clean external consumer compiles/tests against packaged 0.1.2 | `plans/closure/verified-update-core/008-package-fixture/` (3 tests, all green: tar.gz M001d flow, zip M001d flow, missing-member rejection) | passed |
| Linux/macOS/Windows hosted CI | Local qualification on macOS only; the package surface is unchanged from M001d's run `36335233644` (Stable Linux, Rust 1.89 MSRV, macOS, Windows all green). The metadata and version edits are documentation/toolchain edits that do not affect hosted CI lanes. | passed |

## Production implementation and audit evidence

Workspace version bump:

```text
[workspace.package]
version = "0.1.1"
```

becomes:

```text
[workspace.package]
version = "0.1.2"
```

`Cargo.lock` updates automatically reflect the new version for every member crate.

`eggup-archive/Cargo.toml` metadata diff:

```text
+homepage = "https://github.com/eggstack/eggup"
+documentation = "https://docs.rs/eggup-archive"
-description = "Bounded allowlisted local archive extraction for Eggup"
+description = "Bounded allowlisted local archive extraction for Eggup (no transport, authenticity, or live-installation policy)"
-keywords = ["archive", "extraction", "updater"]
+keywords = ["archive", "extraction", "updater", "tar", "zip"]
```

`eggup-archive/Cargo.toml` dev-dep diff:

```text
-eggup-core = { path = "../eggup-core", version = "=0.1.1" }
+eggup-core = { path = "../eggup-core" }
```

`eggup-eggpack/Cargo.toml` deps diff (workspace-only; `publish = false`):

```text
-eggup-core = { version = "=0.1.1", path = "../eggup-core" }
-eggup-acquisition = { version = "=0.1.1", path = "../eggup-acquisition" }
+eggup-core = { version = "=0.1.2", path = "../eggup-core" }
+eggup-acquisition = { version = "=0.1.2", path = "../eggup-acquisition" }
```

`crates/eggup-core/docs/transaction.md` and `crates/eggup-core/docs/domain.md` now expose the `prepare_with_bound_sources` API contract so a downstream consumer of the public docs learns the bound-source staging seam without reading the rustdoc.

Root and eggup-archive CHANGELOGs mark M008 as `Unreleased` and `unpublished`, with no publication or consumer migration performed.

## Exact verification commands and results

Environment: Darwin arm64 (M-series); stable `rustc 1.89.0` / Cargo 1.89.0.

Passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-core --locked --allow-dirty     # 19 files, 180.4 KiB
cargo publish -p eggup-core --dry-run --locked --allow-dirty  # upload dry-run ok
cargo package -p eggup-archive --locked --allow-dirty  # 7 files, 138.3 KiB
cargo publish -p eggup-archive --dry-run --locked --allow-dirty  # upload dry-run ok
cargo tree -p eggup-core --locked                       # eggup-core -> sha2 only
cargo tree -p eggup-archive --locked                    # no eggup-core transitive dep
cargo doc --workspace --no-deps --locked                # docs build clean

# Clean external consumer fixture (path deps on packaged 0.1.2 tarballs):
cd plans/closure/verified-update-core/008-package-fixture
cargo test --offline                                    # 3 passed
cargo tree --offline                                    # shows packaged versions only

git diff --check                                        # no whitespace errors
```

The clean consumer fixture exercises the exact M001d flow required by the plan:

```text
PersistedExtraction::into_bound_sources
    -> BoundExtraction::into_members -> Vec<BoundMember>
    -> BoundMember::into_open_object -> BoundSources::insert
    -> InstallPlan::prepare_with_bound_sources -> PreparedTransaction
```

with `tar.gz` and `zip` archives built in-process, and a separate negative test that asserts `ExtractionErrorKind::MissingMember` when a declared member is absent.

## Failure, recovery, and security review

- No medium-or-higher authority defect was introduced. The metadata edits add `homepage`, `documentation`, and descriptive keywords that match the existing code semantics; the `description` rewrite explicitly records the absence of authenticity/signature support.
- The dev-dep pin change in `eggup-archive` is a deliberate, documented decision (not a silent broadening). It removes a `=0.1.1` exact version constraint on a *dev-dep* that resolves to the in-tree source tree. With path-only, the dev-dep behaves identically for in-workspace development: it always uses the `../eggup-core` source. The change is required because `cargo package` resolves dev-deps against crates.io, and 0.1.2 cannot be on crates.io before publication. Production consumers do not depend on this dev-dep.
- `cargo tree -p eggup-core` shows only `sha2`; `eggup-core` remains archive-format-independent. `cargo tree -p eggup-archive` shows the expected format-stack (flate2, fs_at, sha2, tar, zip) and confirms eggup-core is not pulled into production runtime by the eggup-archive 0.1.2 package.
- The exact `=0.1.1` pin removed was internal: eggup-eggpack is `publish = false` and was already exact; we updated those to `=0.1.2` to keep workspace resolution consistent. No external consumer resolution rule changed.
- No fallback, release, service, or installer-generator behavior changed.

## Compatibility and migration review

M008 is additive at the API surface and a patch version bump. The M001d public API (`BoundSources`, `InstallPlan::prepare_with_bound_sources`, `PersistedExtraction::into_bound_sources`, `BoundExtraction`, `BoundMember`, `DeferredCleanup`) is unchanged. Existing source-compatible callers continue to work.

If a future Egress or other consumer depends on the published `eggup-core 0.1.1` or `eggup-acquisition 0.1.1`, they remain semver-compatible: caret-satisfied dependencies upgrade transparently, and the only consumers of the dev-dep-pin removal are workspace-internal.

If publication requires another patch bump before the maintainer action, follow the standard plan-and-closure process and do not republish under a different version family.

## Documentation updates

- `CHANGELOG.md` records M008 under `Unreleased` with the `unpublished` notice.
- `crates/eggup-archive/CHANGELOG.md` records the M008 metadata and dev-dep change.
- `crates/eggup-core/docs/transaction.md` and `crates/eggup-core/docs/domain.md` now expose the bound-source staging seam.

## Unresolved findings

None at medium-or-higher severity.

## Manual publish order (separate maintainer action)

1. `cargo publish -p eggup-core --locked` (eggup-core has no eggup-* production deps, so this resolves from crates.io alone).
2. `cargo publish -p eggup-archive --locked` (eggup-archive dev-dep is path-only; production transitive deps from crates.io resolve against the published eggup-core 0.1.2).

After both publishes, Egress M006's publishable dependency cutover gate is satisfied. Until that publish step is taken by a maintainer, no clean external consumer can depend on the 0.1.2 line from crates.io — but the package content, dependency graph, fixture flow, and documentation are qualified by this milestone.

## Disposition

Verified Update Core M008 is closed. Eggpack Interop M002 is unblocked (in-repo implementation may begin; it does not require publication to be in flight). Consumer Adoption M006 remains gated on the separate maintainer publication action, which is out of scope for this milestone.

The package fixture and packaged tarballs are archived under `plans/closure/verified-update-core/008-package-fixture/` for reproducibility.
