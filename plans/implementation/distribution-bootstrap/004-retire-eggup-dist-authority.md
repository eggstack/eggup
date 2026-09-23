# Distribution and Bootstrap Milestone 004 — Retire Eggup Producer Authority

Status: implemented (closed; see `plans/closure/distribution-bootstrap/004-status.md`)

Repository baseline: `d894ae63a8963914e545a6d93dc3db92b998138c`

Source roadmap:
- `plans/subsystems/distribution-bootstrap-roadmap.md`

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Primary class: architecture / cleanup

External dependency:
- repository: `eggstack/eggpack`
- planning baseline at authoring: `e3452263225fa1ea262e03b557f40b395e6a52d8`
- reoriented Eggpack M002 plan commit: `9c71f6230e0cac25041dbe85af53c4d7d75a3ac1`
- required closure: Eggpack Contract and Conformance M002
- predecessor Eggup implementation: distribution M003 implementation `9941c58d7039410c728860f9e4e382881d4ccf54`, closure `4169c8021b447fe73c8ee3ea71a80a535c940f54`

## 1. Objective

Remove the unpublished `eggup-dist` producer-side contract/conformance implementation from Eggup after Eggpack independently qualifies the full closed M003 behavior. Leave Eggup with consumer deployment, acquisition, service lifecycle, and optional future manifest-consumer responsibilities only.

## 2. Original blocker and resolution

At authoring, Eggpack Contract M002 still needed to port and close the conformance surface present in Eggup M003. This blocker was resolved by Eggpack Contract M002 closure at `82f799f3d971b2999ac14c2d8fc1b965370e0f58`; its predecessor-to-Eggpack matrix covers the complete API and direct/bundle/archive fixture surface. See the closure record for reviewed equivalence evidence.

Deleting `eggup-dist` before that closure would discard the only qualified implementation of those behaviors.

## 3. Current implementation evidence

Eggup distribution M001-M003 are closed. `eggup-dist` is unpublished and runtime Eggup crates do not depend on it. M003 is therefore migration evidence rather than a public runtime compatibility commitment.

No M004 installer-generator implementation is authorized in Eggup.

## 4. Invariants that must not regress

- No qualified schema-v1 or conformance behavior is lost during the cutover.
- Historical Eggup plans/closure records remain intact for provenance.
- `eggup-core`, acquisition, and service crates gain no Eggpack build-time dependency.
- Eggup remains able to install explicitly described non-Eggpack artifacts.
- Producer-side target naming, release inventories, installer mapping, packaging, CI, and publication have one active authority: Eggpack.
- Checksums remain integrity evidence, not authenticity.
- Consumer release/version/fallback policy remains application-owned.

## 5. Scope

### In scope

- verify Eggpack Contract M002 closure against the Eggup M003 public behavior/test matrix;
- remove `crates/eggup-dist` from the active workspace and delete its production source after evidence is recorded;
- update workspace/package metadata and local qualification scripts that explicitly include `eggup-dist`;
- remove active planning references that authorize future Eggup distribution/bootstrap producer work;
- retain historical implementation/closure records and add migration pointers to Eggpack;
- update README/architecture references only where they imply Eggup owns producer tooling;
- verify no active consumer or published Eggup crate depends on `eggup-dist`.

### Explicitly out of scope

- implementing Eggpack M002;
- installer generation;
- ReleaseManifest implementation;
- build/package/CI orchestration;
- release publication;
- adding `eggup-eggpack`;
- changing Eggup transaction, acquisition, service, or release-selection semantics.

## 6. Required production changes

1. Remove `crates/eggup-dist` from workspace membership.
2. Remove the crate source once the migration comparison is captured in closure evidence.
3. Adjust scripts/tests/docs that assume the crate is part of the active workspace.
4. Do not replace it with another Eggup distribution crate.

## 7. Ordered work packages

1. Record the exact Eggpack M002 closure SHA and public API/fixture equivalence evidence.
2. Search Eggup and known consumers for active `eggup-dist` dependency/imports.
3. Stop if any runtime/published consumer dependency is found and write a narrow migration corrective.
4. Remove the workspace member and crate.
5. Update canonical/active documentation while preserving historical plans and closures.
6. Run full stable/MSRV/package/doc qualification for remaining Eggup crates.
7. Write M004 closure and mark the distribution/bootstrap subsystem archived/transferred to Eggpack.

## 8. Failure, cancellation, restart, and contention semantics

This milestone is repository cleanup and does not alter runtime mutation semantics. If equivalence evidence is incomplete, stop before deletion. Partial documentation cleanup must not be used to claim authority transfer complete while the duplicate production crate remains active.

## 9. Compatibility and migration

Because `eggup-dist` is unpublished, no crates.io compatibility promise is expected. Git/path consumers must nevertheless be searched before deletion. Historical schema-v1 documents remain valid under Eggpack's imported contract.

Any future runtime consumption of Eggpack release metadata is a separate optional adapter milestone and must not restore producer ownership to Eggup.

## 10. Required tests

- full remaining Eggup workspace stable/MSRV tests;
- package qualification for published Eggup crates;
- static search showing no active `eggup-dist` dependency;
- Eggpack M002 closure/golden equivalence reference;
- documentation link/path checks where available.

## 11. Required verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh
git grep -n "eggup-dist"
git diff --check
```

Package the externally published crates as required by the repository qualification script.

## 12. Documentation updates

- long-term specification;
- terminology/domain model;
- long-term roadmap;
- distribution/bootstrap roadmap;
- registry;
- README/architecture only where active ownership wording requires correction;
- M004 closure record.

## 13. Acceptance criteria

M004 closes only when Eggpack M002 is independently closed with equivalent predecessor behavior, no active runtime/published consumer depends on `eggup-dist`, the crate is absent from the Eggup workspace, active Eggup planning contains no future producer distribution/bootstrap milestones, historical evidence remains traceable, and the remaining workspace passes qualification.

## 14. Stop conditions

Stop if Eggpack M002 changes schema-v1 meaning, omits a closed M003 behavior, lacks required direct/bundle/archive evidence, or an active consumer still depends on `eggup-dist`.

## 15. Closure evidence required

Record Eggpack M002 closure SHA, predecessor-to-Eggpack behavior matrix, dependency search, deleted paths/workspace diff, verification commands/results, package results, documentation migration, unresolved findings, and final authority disposition.

## 16. Handoff notes

Eggpack Contract M002 is closed, so this plan was authorized and executed. This was the final Eggup distribution/bootstrap milestone; it retires the subsystem rather than extending it.
