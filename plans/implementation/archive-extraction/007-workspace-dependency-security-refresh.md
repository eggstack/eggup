# Archive Extraction Milestone 007 — Workspace dependency security refresh

Status: implemented

Repository baseline: `c789bef156aa09baaf9b349744dac12de2534a0a`

Source roadmaps:
- Primary: `plans/subsystems/archive-extraction-roadmap.md`
- Affected acquisition boundary: `plans/subsystems/acquisition-transport-roadmap.md`
- Affected Eggpack adapter boundary: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: infrastructure

## 1. Objective

Refresh the workspace dependency lock and the direct dependency requirements that block security fixes, while preserving the workspace Rust 1.89 floor, crate boundaries, archive extraction contract, Eggpack exact-pin policy, and public APIs. This is an implementation and qualification milestone, not a crate publication or release authorization.

## 2. Why this milestone is ready

At the reviewed baseline, `cargo deny check advisories` reported ten RustSec records in the locked graph: four for `rustls 0.23.45`, one for `rustls-webpki 0.103.15`, one for `webpki-roots 1.0.9`, and four for `zip 6.0.0`. RustSec reports the ZIP issues fixed in `zip >=9.0.0`; current crates.io metadata lists `zip 9.6.1`, whose declared Rust version is 1.88. The workspace floor is 1.89.

`cargo update --dry-run --verbose` resolved 49 package updates within existing version requirements, including patched rustls, rustls-webpki, and webpki-roots versions. It could not cross the `zip` major-version requirement or the exact `eggpack-manifest = "=0.1.0"` pin. `eggpack-manifest 0.1.1` is available. The `sha2 0.10.9` requirement is current within the 0.10 line; `sha2 0.11` is a separate major upgrade and is not needed to resolve the findings.

The ZIP findings concern general-purpose extraction helpers. Eggup's archive implementation iterates with `ZipArchive::by_index`, validates entry kinds and paths, and writes allowlisted members through its own bounded writer; it does not call `ZipArchive::extract()` or `ZipStreamReader::extract()`. The findings still justify updating the archive dependency, but closure must preserve and test Eggup's explicit extraction boundary rather than adopting convenience extraction.

## 3. Current implementation evidence

- Workspace package floor is Rust 1.89 in `Cargo.toml`.
- `crates/eggup-archive/Cargo.toml` currently requires `zip = "6.0.0"` at runtime.
- `crates/eggup-eggpack/Cargo.toml` currently exact-pins `eggpack-manifest = "=0.1.0"` and uses `zip = "6.0"` as a dev-dependency.
- `Cargo.lock` currently resolves `rustls 0.23.45`, `rustls-webpki 0.103.15`, `webpki-roots 1.0.9`, `zip 6.0.0`, `eggfetch-core 0.2.0`, and `eggpack-manifest 0.1.0`.
- `crates/eggup-archive/src/lib.rs:1036-1084` contains explicit ZIP entry iteration and an entry-kind predicate. `write_declared_member` fails closed for a declared non-regular entry and drains undeclared/non-regular entries under the configured budget.
- The Eggfetch TLS graph is owned by `eggup-eggfetch`; the adapter remains separate from `eggup-curl` and from the zero-dependency acquisition seam.
- `eggup-eggpack` is the only crate allowed to depend on `eggpack-manifest`; it exact-pins its in-workspace dependencies.
- The dependency audit baseline and findings are recorded in the parent conversation and can be reproduced with the commands in §11.

## 4. Invariants that must not regress

- `eggup-core` gains no transport, archive, service-manager, or Eggpack dependency; its `sha2` dependency remains on the declared 0.10 line.
- `eggup-acquisition` remains dependency-free.
- `eggup-eggfetch` remains independent of `eggup-curl`; transport composition stays outside adapters.
- `eggup-archive` remains local, bounded, allowlisted, regular-file-only extraction. Do not use ZIP convenience extraction APIs, preserve archive metadata, or change ownership/authority semantics.
- `eggup-eggpack` remains the sole Eggpack producer-type adapter, remains optional, and retains exact in-workspace dependency pins at `=0.2.0`.
- SHA-256 remains integrity evidence only. This work introduces no authenticity or signature claims.
- Rust 1.89 remains the minimum supported toolchain. Do not weaken limits, suppress advisories, or silently accept a changed public API to make an upgrade compile.

## 5. Scope

### In scope

- Update `Cargo.lock` to the latest compatible versions allowed by the workspace manifests.
- Raise both ZIP requirements to `9.6.1` and migrate source/tests only as required by the ZIP 6-to-9 API change, while preserving current behavior and feature selection.
- Raise the `eggpack-manifest` exact pin to `=0.1.1`, update the corresponding pin assertion, and verify the existing adapter API and behavior.
- Update active dependency facts, changelog `Unreleased` sections, the root aggregate changelog, and plan/roadmap/registry records.
- Re-run advisory, package, stable, Rust 1.89, macOS, and Windows checks where the environment permits; report missing hosted lanes explicitly.

### Explicitly out of scope

- Publishing any crate, changing workspace or crate package versions, moving tags, or creating a GitHub release.
- Committing or pushing changes without separate explicit user authorization. The user later authorized the M007 commit/push on 2026-11-17 to trigger hosted qualification; see §16.
- Changing public APIs, extraction policy, TLS features, fallback behavior, or dependency boundaries.
- Upgrading `sha2` across its 0.10-to-0.11 major boundary or `eggfetch-core` across 0.2-to-0.3 without a separate compatibility rationale.
- Adding a new cargo-deny policy file, CI job, dependency bot, or release automation.
- Editing historical plans, closure records, or changelog release sections to make past dependency state appear different.

## 6. Required production changes

- Refresh the lockfile using Cargo's latest compatible resolution after manifest edits. Review all resulting package additions, removals, and versions, not only direct dependencies.
- Set the runtime ZIP requirement in `crates/eggup-archive/Cargo.toml` and the ZIP test requirement in `crates/eggup-eggpack/Cargo.toml` to `9.6.1`, retaining `default-features = false` and the existing `deflate-flate2` feature.
- Set `eggpack-manifest` in `crates/eggup-eggpack/Cargo.toml` to the exact version `=0.1.1`.
- Update the manifest pin assertion and any source or tests required by the new ZIP API. Keep the resulting extraction algorithm explicit and bounded.
- Update current dependency references in `AGENTS.md`, `architecture/overview.md`, `architecture/tooling-governance.md`, `architecture/eggpack-adapter.md`, and `crates/eggup-eggpack/README.md` where they describe the active pin/version. Leave references in historical plan/closure and release sections unchanged.
- Record the unpublished dependency changes in `crates/eggup-archive/CHANGELOG.md`, `crates/eggup-eggpack/CHANGELOG.md`, and the root `CHANGELOG.md`, with clear no-publication/no-migration language.

## 7. Ordered work packages

1. Reconfirm the clean baseline, dependency graph, active package pins, and source call paths; preserve the initial RustSec findings as audit evidence.
2. Author this plan and register Archive Extraction M007 as active before implementation.
3. Update the two manifests, migrate any necessary source/test callsites, and resolve `Cargo.lock` with Cargo.
4. Review `cargo tree` for all workspace and platform-gated edges. Confirm the lockfile has no obsolete vulnerable versions and no forbidden dependency edge.
5. Run advisory, package, local, MSRV, macOS, and Windows verification; investigate any failure without weakening the contracts.
6. Update current documentation and `Unreleased` changelog entries, then write the closure record and reconcile the archive roadmap and registry. Do not publish; commit/push only under separate explicit user authorization (received 2026-11-17 for the M007 qualification run).

## 8. Failure, cancellation, restart, and contention semantics

This milestone changes build-time dependency resolution and contains no live-installation mutation, service operation, external updater process, or network transport action at runtime. Cargo resolution or compilation failure is a failed work package, not a reason to relax a version requirement or suppress a RustSec record. If the ZIP API migration, Eggpack 0.1.1 API, or any candidate dependency violates the current behavior or Rust 1.89 floor, stop and report the blocker before selecting another version. Revert only files changed by this milestone if abandoning it; do not discard unrelated user changes. Package publication is not part of this plan. Git operations were initially out of scope; see §16 for the later, narrowly scoped authorization to commit and push M007 to trigger hosted qualification.

## 9. Compatibility and migration

The intended change is private dependency maintenance, with no public Eggup API change and no workspace MSRV increase. The ZIP major upgrade requires source and test compatibility review; the archive crate must retain its explicit entry iteration, no-clobber writes, allowlist, finite budgets, and symlink/special-file rejection. The Eggpack parser pin remains exact and changes only from 0.1.0 to 0.1.1. A downstream user receives none of these manifest changes until a separate, manually authorized publication milestone; this plan does not change the already-published `0.2.0` packages or claim that they are fixed.

## 10. Required tests

- Existing archive ZIP hostile-entry, path, duplicate, regular-file, size, digest, and cleanup tests continue to pass on macOS and Windows.
- Existing Eggpack manifest projection and compatibility tests pass with `eggpack-manifest 0.1.1`; update the exact-pin assertion to match the new requirement.
- Run the complete workspace test/clippy/doc gate with all features.
- Verify packaged `eggup-archive` and `eggup-eggpack` manifests with Cargo package checks.
- `cargo deny check advisories` reports no unresolved applicable advisories. Any residual finding must be investigated and recorded with reachability and severity; do not silence it.

## 11. Required verification commands

Run these after the implementation unless noted:

```sh
cargo update
cargo deny check advisories
cargo tree --workspace --locked
cargo tree --workspace --locked --target x86_64-pc-windows-msvc --depth 1
cargo package -p eggup-archive --locked
cargo package -p eggup-eggpack --locked
./scripts/check-local.sh
cargo +1.89.0 check --workspace --all-targets --locked
cargo test -p eggup-archive --all-targets --locked
cargo test -p eggup-eggpack --all-targets --all-features --locked
git diff --check
```

Platform qualification must use the repository's CI commands, not inferred cross-compilation:

```sh
cargo test --workspace --all-targets --all-features --locked
cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test --workspace --all-targets --locked --no-fail-fast
```

The first command is the stable/macOS workspace test. The next three are Windows-lane steps relevant to the changed crates. A hosted lane not run must be recorded as `not run`. `./scripts/check-local.sh` also runs `cargo fmt`, Clippy, workspace tests, docs, `cargo tree`, and `scripts/check-docs-truth.sh` in the documented order.

## 12. Documentation updates

- Update only current dependency facts and references to the active exact pin; retain historical release and plan evidence verbatim.
- Add concise `Unreleased` notes to both affected crate changelogs and the root aggregate. State that these entries are unpublished, package versions remain 0.2.0, and no publication or consumer migration is authorized here.
- Add the M007 row and status to `plans/subsystems/archive-extraction-roadmap.md`; update the active control rows in `plans/registry.md` with evidence and any unrun platform lane.
- Do not add a new policy to `docs/`; user-facing API and behavior remain unchanged.

## 13. Acceptance criteria

- The locked graph resolves current compatible versions, including the TLS fixes; the direct ZIP requirements resolve to 9.6.1 and Eggpack resolves the exact parser pin at 0.1.1.
- `cargo deny check advisories` has no unresolved applicable findings.
- The workspace dependency boundaries and target-gated graph remain correct.
- ZIP and Eggpack package/test checks pass without public API or behavior changes.
- The local gate and Rust 1.89 check pass. Hosted results are reported separately; missing lanes are `not run`, not inferred.
- Current documentation and changelogs agree with the manifests. No package version, publication, or tag is changed. The user's later authorization for the M007 commit/push is recorded in §16.

## 14. Stop conditions

Stop if `zip 9.6.1` or `eggpack-manifest 0.1.1` breaks behavior/API in a way that cannot be corrected without widening scope; if any new dependency violates the 1.89 floor or a crate boundary; if the lockfile retains an applicable advisory; or if package, local, or platform tests fail. Do not claim full cross-platform closure without actual hosted evidence. Publication, package version bumps, and release ordering require separate plans and explicit authorization.

## 15. Closure evidence required

The closure record must include the exact reviewed baseline, all changed dependency requirements, a summary of lockfile updates, the final `cargo tree` boundary review, advisory results and applicability notes, package checks, exact commands and results, stable/macOS/MSRV/Windows results separately, compatibility and migration assessment, current documentation/changelog evidence, unresolved findings with severity, and whether changes remain uncommitted. Record unrun hosted lanes as `not run`. The closure status must be `conditionally closed` if required hosted qualification is unavailable.

## 16. Handoff notes

The implementation started from clean `main` at `c789bef156aa09baaf9b349744dac12de2534a0a`. At plan creation, the caller authorized only plan creation and local dependency changes, so the implementation was initially left uncommitted and unpublished. **Post-plan authorization (2026-11-17):** the caller separately authorized committing and pushing M007 to trigger hosted qualification. Commit `fe763189e4cfe1e0e6d6c62d431717d82b69f9f5` was pushed to `origin/main`; hosted CI run [44022660351](https://github.com/eggstack/eggup/actions/runs/44022660351) passed all five lanes. No package publication was authorized. A later release task must check current registry state, bump package versions, reconcile Eggpack's exact in-workspace pins, qualify registry-only package graphs, and obtain separate manual publication authorization.
