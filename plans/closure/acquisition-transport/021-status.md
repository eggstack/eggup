# Acquisition Transport M021 Closure — eggfetch-core 0.4.0 dependency update

Status: closed 2026-11-21; hosted-qualified on all five CI lanes at implementation SHA 8af7db87cf188859ee8b3222aed8f4a6c9b74ed7; run https://github.com/eggstack/eggup/actions/runs/44879536834.

Source plan: plans/implementation/acquisition-transport/021-eggfetch-core-0.4.0-dependency-update.md
Source roadmap: plans/subsystems/acquisition-transport-roadmap.md

Reviewed baseline: 9800d55fc7b7701747b4d4002af3f8e9e766eeab (clean; matched origin/main)
Implementation commit: 8af7db87cf188859ee8b3222aed8f4a6c9b74ed7

## Executive finding

The sole workspace consumer, eggup-eggfetch, now requires eggfetch-core 0.4.0. Its explicit feature list and disabled defaults are unchanged, Cargo.lock resolves eggfetch-core and eggfetch-http-connect 0.4.0, and the adapter API and acquisition policy remain unchanged. The Eggup package version remains 0.2.1. No crate was published, no tag or GitHub Release was changed, and the already-published eggup-eggfetch 0.2.1 remains on eggfetch-core 0.3.0.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Upgrade only the sole direct consumer | crates/eggup-eggfetch/Cargo.toml:25; workspace cargo tree | Pass; other crate manifests unchanged |
| Preserve feature selection and policy | Manifest, cargo tree -e features -i eggfetch-core --locked, adapter suite | Pass; defaults remain disabled and the same four features are selected |
| Update source diagnostic and expectation | crates/eggup-eggfetch/src/lib.rs:215-220, :658-661 | Pass; both report 0.4.0 |
| Keep public API and Eggup package versions unchanged | Source diff, workspace manifests, full tests | Pass |
| Keep published state immutable | Registry-facing release docs and no publish/tag operation | Pass; published 0.2.1 remains on 0.3.0 |
| Qualify across supported hosted lanes | CI run 44879536834 on the exact implementation SHA | Pass; all five lanes green |

## Production implementation evidence

- crates/eggup-eggfetch/Cargo.toml selects version 0.4.0 with default-features=false and http1, tls-rustls, tls-native-roots, and proxy.
- Cargo.lock resolves eggfetch-core 0.4.0 and eggfetch-http-connect 0.4.0 from crates.io.
- EggfetchTransport::eggfetch_version and its assertion were updated to 0.4.0; no other production source changed.
- The resolved feature tree has no eggfetch-core default feature activation. The explicit http1 feature remains selected alongside proxy, preserving the HTTP/1 feature bundle after the upstream proxy-feature implication change.
- Workspace dependency review found no new cross-edge: eggup-eggfetch depends on eggup-acquisition and the external Eggfetch stack, not eggup-curl; other crate boundaries remain unchanged.

## Exact commands and results

- cargo update -p eggfetch-core --precise 0.4.0 — passed; updated eggfetch-core and eggfetch-http-connect only.
- cargo test -p eggup-eggfetch --locked — passed: 25 unit tests, 3 assertion-guard tests, 2 typed-I/O tests; 30 passed, 0 failed.
- cargo clippy -p eggup-eggfetch --all-targets --locked -- -D warnings — passed. Cargo emitted non-failing redundant-homepage manifest warnings.
- cargo tree -e features -i eggfetch-core --locked — passed and manually reviewed.
- cargo tree --workspace --locked — passed and manually reviewed for crate boundaries.
- ./scripts/check-docs-truth.sh — passed (also run by the full local gate).
- ./scripts/check-local.sh — passed on macOS at the implementation candidate; fmt, clippy, workspace tests, docs, dependency resolution, and docs-truth checks succeeded.
- git diff --check — passed before implementation commit.

## Hosted CI evidence

Run 44879536834 was triggered by push and completed successfully with head SHA 8af7db87cf188859ee8b3222aed8f4a6c9b74ed7. All five required lanes passed:

| Lane | Result |
|---|---|
| Stable checks | Passed |
| MSRV check (Rust 1.89.0) | Passed |
| Linux systemd service qualification | Passed |
| macOS tests | Passed |
| Windows archive, acquisition, and service tests/check | Passed |

Platform evidence is lane-specific: Linux stable tests and systemd fixtures passed; macOS full workspace tests passed; Windows all test steps and package verification passed; the Rust 1.89 workspace/footprint checks passed. No platform result is inferred from another.

## Invariant review

The transport-neutral acquisition seam and crate dependency boundaries are unchanged. HTTP/1, TLS roots, explicit proxy configuration, strict redirect policy, deadlines, bounded bodies, streaming cleanup, redaction, and error classification remain covered by the passing adapter suite. No retries, fallback, release selection, authenticity, or install policy was added.

## Failure, rollback, and recovery review

This dependency update performs no install, filesystem promotion, process execution, or other live-state mutation. It introduces no rollback or recovery path. Compile, test, and CI failures would have blocked closure; none occurred.

## Compatibility and migration review

The upstream 0.3-to-0.4 transition crosses a pre-1.0 minor boundary and was qualified against adapter call sites and behavior tests. The Eggup public API and all workspace package versions remain unchanged. No consumer migration is needed for workspace source; registry consumers still receive the immutable 0.2.1 artifact built against eggfetch-core 0.3.0. A future publication requires a separate release decision and qualification.

## Security review

No new secret handling, logging, network policy, or install authority was introduced. Existing redaction and fail-closed behavior tests passed.

## Documentation and operations evidence

The crate and root changelogs, adapter architecture, dependency comparison, workspace overview, governance table, roadmap, registry, and this closure record distinguish the workspace source candidate from the published artifact. Documentation-truth checks passed. No release notes, tags, registry artifacts, or consumer requirements were changed.

## Unresolved findings

No M021-specific residual finding remains. The pre-existing low-severity registry finding 1 concerning intermittent host-level spawn/resource failures remains open and unchanged; it is unrelated to this dependency update and was not represented as closed by M021.

## Roadmap disposition

M021 is closed and hosted-qualified. The workspace source uses eggfetch-core 0.4.0; no Eggup crate version was bumped and no package was published.

## Registry updates

The acquisition-transport roadmap and plans/registry.md record M021 as closed and link this record. Published-version tables remain unchanged.
