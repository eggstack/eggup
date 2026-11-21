# Acquisition Transport Milestone 021 — eggfetch-core 0.4.0 dependency update

Status: active (workspace implementation; no publication)

Repository baseline: 9800d55fc7b7701747b4d4002af3f8e9e766eeab

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md (classification only)

Primary class: infrastructure

## 1. Objective

Update the sole workspace consumer of eggfetch-core—eggup-eggfetch—from the 0.3 line to 0.4.0. Preserve the four explicit features and the adapter’s public API and network policy. Update its upstream-version diagnostic, lockfile, tests, and source documentation. Do not change any Eggup package version and do not publish.

## 2. Why this milestone is ready

- M020 is closed; eggup-eggfetch 0.2.1 and the other selective 0.2.1 releases are already published.
- crates.io exposes a non-yanked eggfetch-core 0.4.0 release, Rust 1.89, published by dbowm91; cargo info eggfetch-core@0.4.0 resolves it.
- The upstream 0.4.0 changelog describes additive request-builder/API additions and changes the implication of the proxy feature from http1 to native-http1. This adapter enables http1 explicitly as well as proxy, so the feature-set change does not remove the selected HTTP/1 policy bundle.
- The declared Rust MSRV remains 1.89, matching this workspace’s floor.

## 3. Current implementation evidence

- Reviewed baseline 9800d55fc7b7701747b4d4002af3f8e9e766eeab is clean and matches origin/main.
- crates/eggup-eggfetch/Cargo.toml declares eggfetch-core = 0.3.0, default-features = false, and exactly http1, tls-rustls, tls-native-roots, and proxy.
- Cargo.lock resolves eggfetch-core 0.3.0.
- EggfetchTransport::eggfetch_version() and its test expect the 0.3.0 diagnostic.
- Adapter call sites use Client, HttpVersionPolicy, ProxyEnvironment, RedirectPolicy, Timeout, the request builder, response handling, and typed upstream errors; focused and workspace tests are the compatibility evidence.

## 4. Invariants that must not regress

- Keep the Eggup public API and its transport-neutral acquisition seam unchanged.
- Preserve HTTP/1-only operation, explicit Rustls/native-root and proxy choices, strict redirect/downgrade behavior, bounded connect/total deadlines and bodies, redaction, streaming cleanup, and no automatic retry behavior.
- Retain default-features = false and the same four explicit Eggfetch features. Do not add eggup-curl or broaden another crate’s dependency graph.
- Keep all Eggup workspace package versions at 0.2.1; this is a source-only dependency candidate.
- Do not publish, yank, create/move tags, or alter GitHub Releases. The user explicitly authorized commit/push to qualify and close this milestone; keep publication separate.

## 5. Scope

### In scope

- Change only the eggup-eggfetch Eggfetch requirement to 0.4.0 using existing Cargo caret semantics.
- Resolve the lockfile to eggfetch-core 0.4.0 and any corresponding transitive updates.
- Update the upstream-version diagnostic and its assertion.
- Update the Eggfetch changelog and owning architecture/dependency summaries so they distinguish the source candidate from the immutable published 0.2.1 artifact.
- Run the focused adapter checks and the workspace local gate; review the resolved dependency tree and docs truth.

### Explicitly out of scope

- Changing the versions of eggup-core, eggup-acquisition, eggup-curl, eggup-archive, eggup-service, eggup-eggpack, eggup-eggfetch, or eggup-transport-footprint.
- Publishing eggup-eggfetch or any other crate, creating a registry-only fixture, or editing release notes/tags.
- Any adapter API, timeout, redirect, proxy, retry, body-size, error-mapping, or transport-composition redesign.

## 6. Required production changes

Set the manifest requirement to 0.4.0 while preserving its explicit feature list and disabled defaults. Regenerate Cargo.lock from the registry. Change the diagnostic string and its expected value to 0.4.0. Make no other production-code changes unless compilation demonstrates a required source adaptation; stop rather than weakening an invariant.

## 7. Ordered work packages

1. Register M021 in the acquisition roadmap and active registry before changing production files.
2. Update the dependency manifest and resolve/inspect the lockfile; update the source diagnostic and its test.
3. Update the crate changelog and current-source architecture summaries, preserving the historical/published 0.3.0 claim for Eggfetch 0.2.1.
4. Run focused adapter tests and clippy, docs truth, the full local gate, and manual dependency-boundary review.
5. Commit and push the locally qualified implementation candidate as explicitly requested; wait for Stable, MSRV, service-systemd, macOS, and Windows-check to pass on that exact implementation SHA. Do not publish.
6. Write closure evidence only after the hosted results are read back; leave the milestone open if any required lane fails or is missing.

## 8. Failure, cancellation, restart, and contention semantics

This change performs no installation or other live-state mutation. A resolver/network failure must not be worked around with an unreviewed path/Git dependency. A compile or test failure blocks completion. If 0.4.0 requires broadening features, weakening an existing policy, or changing the adapter contract, stop and report the incompatibility. No registry publication is authorized by this plan.

## 9. Compatibility and migration

Eggfetch-core is pre-1.0, so 0.3 to 0.4 is a semver-minor boundary that may contain breaking changes; qualification must cover the actual adapter call sites and behavior tests. Upstream 0.4.0 retains Rust 1.89 and documents no removal of the APIs used here. Eggup’s public API and package versions remain unchanged. The already-published eggup-eggfetch 0.2.1 remains immutable and continues to declare the 0.3 requirement; consumers receive no change until a separately authorized release.

## 10. Required tests

- eggup-eggfetch unit/integration tests covering configuration, redirects, proxy handling, status/error mapping, bounded metadata, streamed artifacts, deadlines, redaction, and cleanup.
- Workspace all-target/all-feature tests and clippy against the resolved 0.4.0 graph.
- Review the resolved feature tree to confirm default-features = false, the four explicitly selected features, and no forbidden adapter cross-edge.
- Documentation-truth check and full local verification gate.

## 11. Required verification commands

- cargo update -p eggfetch-core --precise 0.4.0
- cargo test -p eggup-eggfetch --locked
- cargo clippy -p eggup-eggfetch --all-targets --locked -- -D warnings
- cargo tree -p eggup-eggfetch --locked
- cargo tree --workspace --locked
- ./scripts/check-docs-truth.sh
- ./scripts/check-local.sh
- git diff --check
- `gh run list --commit <implementation-sha> --limit 10`
- `gh run watch <push-run-id> --exit-status --interval 15`

Inspect the Cargo tree manually; resolution success is not an automated dependency-boundary proof. Hosted CI must be read back on the exact implementation SHA.

## 12. Documentation updates

Update crates/eggup-eggfetch/CHANGELOG.md, architecture/eggfetch-adapter.md, the current Eggfetch dependency entries in architecture/overview.md and architecture/curl-adapter.md, and the acquisition milestone/status summaries in architecture/tooling-governance.md, plans/subsystems/acquisition-transport-roadmap.md, and plans/registry.md. Keep M020 publication records and user-facing registry-release facts unchanged. Do not edit docs/releases.md to claim a new package release.

## 13. Acceptance criteria

- The only direct workspace eggfetch-core consumer requires 0.4.0 and keeps default-features = false plus the same four features.
- The lockfile resolves eggfetch-core 0.4.0; focused adapter tests and clippy pass.
- The upstream-version diagnostic and test report 0.4.0.
- The full local gate and docs-truth check pass; the dependency tree preserves workspace boundaries.
- No Eggup package version, published artifact, release/tag, or unrelated production crate changes.

## 14. Stop conditions

Stop if the registry/index cannot resolve 0.4.0, if an API/feature incompatibility requires a policy or public-contract change, if any required test/gate fails, if the dependency tree crosses a forbidden boundary, or if the worktree reveals unrelated changes that cannot be safely preserved. Do not publish or alter tags/releases. The user-authorized push is only for the locally qualified candidate and hosted closure evidence.

## 15. Closure evidence required

Record the exact reviewed and implementation SHAs, changed files, focused test/clippy output, full local-gate and docs-truth results, and manual dependency-tree findings. State macOS, Linux, Windows, and hosted CI evidence separately; mark anything not run as not run. No registry artifact, provenance, reverse-dependency, tag, or release evidence is expected because publication is out of scope. Do not mark closed if required evidence is absent.

## 16. Handoff notes

The current published adapter remains eggup-eggfetch 0.2.1 with eggfetch-core ^0.3.0. This milestone only updates workspace source to 0.4.0. Any later publication needs its own authorized release scope, package-version decision, exact-SHA hosted qualification, and registry verification. The user explicitly authorized commit/push for hosted qualification and closure; do not publish.
