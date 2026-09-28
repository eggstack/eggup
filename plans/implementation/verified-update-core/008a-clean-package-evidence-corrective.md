# Verified Update Core M008a — Clean Package and Current-Head Qualification Evidence Corrective

Status: implemented; closed by `plans/closure/verified-update-core/008a-status.md` (exact clean-tree package/dry-run + fixture + green hosted head)

Repository baseline: `f708e45ab82a35df740704f1af36a7d2275794cf`

Original plan:

- `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md`

Original closure:

- `plans/closure/verified-update-core/008-status.md`

Hard dependency:

- `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md`

Primary class: polish/corrective

## 1. Objective

Reconcile Core M008's package-qualification closure with the exact commands required by its source plan and with a green final repository head.

M008a does not redesign or re-version the 0.1.2 package boundary. It replaces provisional/local evidence with clean, reproducible package evidence from the corrected repository state.

## 2. Readiness and dependencies

M008a is blocked until M002a closes because current head `f708e45ab82a35df740704f1af36a7d2275794cf` fails Stable clippy and therefore is not a valid final package/closure baseline.

Once M002a restores a green current head, M008a is dependency-ready.

## 3. Current evidence and gaps

M008 source plan Section 11 required:

~~~text
cargo package -p eggup-core --locked
cargo publish -p eggup-core --dry-run --locked
cargo package -p eggup-archive --locked
cargo publish -p eggup-archive --dry-run --locked
~~~

The closure records successful qualification using `--allow-dirty` because package/version/docs edits were still uncommitted during local execution.

That evidence is useful but does not prove the exact clean-tree commands required by the plan.

The closure also reused M001d hosted qualification for some platform claims, while current final head `f708e45` is red due the M002 clippy regression.

## 4. Invariants that must not regress

- workspace/package version remains 0.1.2 unless a separate versioning issue requires change;
- no automatic publication;
- `eggup-core` package remains transport/archive independent;
- `eggup-archive` remains optional and independently packageable;
- M001d public APIs remain additive and unchanged;
- manual publish order remains core before archive;
- no git/path dependency is accepted as the final downstream Egress package solution;
- Rust 1.89 remains supported.

## 5. Scope and non-scope

### In scope

- rerun exact clean package commands from a committed, clean post-M002a tree;
- rerun exact clean `cargo publish --dry-run` commands;
- inspect package contents/dependency trees again;
- rerun the external package fixture if packaged tarballs change;
- bind M008 closure to the fresh green current-head hosted matrix;
- amend closure/registry/roadmap evidence.

### Out of scope

- actual crates.io publication;
- another version bump unless the clean commands prove 0.1.2 cannot be published;
- changing M001d APIs;
- Egress code changes;
- Eggpack producer changes.

## 6. Required production/package changes

No production change is expected.

If clean package verification fails because the current path-only archive dev-dependency cannot be represented correctly in the packaged crate, stop and write a package-metadata corrective; do not restore `--allow-dirty` as the closure path.

Package artifacts/fixture may be regenerated solely to reflect the committed corrected source.

## 7. Ordered work packages

1. Begin from the green committed M002a head with no working-tree modifications.
2. Verify `git status --porcelain` is empty.
3. Run exact core package and dry-run commands without `--allow-dirty`.
4. Run exact archive package and dry-run commands without `--allow-dirty`.
5. Inspect packaged file lists and dependency trees.
6. Re-run the clean external package fixture against the produced tarballs.
7. Confirm the fresh hosted matrix is green on Stable/MSRV/macOS/Windows.
8. Amend M008 closure and registry.

## 8. Failure, restart, cancellation, and contention semantics

No runtime semantics.

A failed package/dry-run does not authorize publication or dependency workarounds. Preserve the failure and stop for a corrective if package metadata is wrong.

## 9. Compatibility and migration

No public API migration is expected.

0.1.2 remains unpublished until a separate maintainer action.

## 10. Required tests

- exact package/dry-run commands on a clean tree;
- clean external fixture tar.gz + zip + missing-member tests;
- dependency tree checks;
- full current-head hosted matrix inherited from M002a or a later exact head;
- Rust 1.89 all-target check.

## 11. Verification commands

~~~bash
test -z "$(git status --porcelain)"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-core --locked
cargo publish -p eggup-core --dry-run --locked
cargo package -p eggup-archive --locked
cargo publish -p eggup-archive --dry-run --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-archive --locked
cargo doc --workspace --no-deps --locked
git diff --check
~~~

Then rerun the archived external fixture against the newly created package artifacts.

## 12. Documentation updates

- `plans/closure/verified-update-core/008-status.md` addendum or corrected evidence section;
- `plans/subsystems/verified-update-core-roadmap.md`;
- `plans/registry.md`.

## 13. Acceptance criteria

M008a closes when the exact clean-tree package/dry-run commands pass, the package fixture passes against those artifacts, and the final repository head has a green hosted matrix.

## 14. Stop conditions

Stop if:

- clean-tree package/dry-run differs materially from the recorded `--allow-dirty` result;
- 0.1.2 requires another version bump;
- package dependency resolution requires publishing unrelated Eggup crates;
- any current-head hosted lane fails after M002a.

## 15. Closure evidence required

Record:

- exact clean baseline SHA;
- `git status --porcelain` evidence;
- package file counts/sizes;
- publish dry-run results without `--allow-dirty`;
- dependency trees;
- package fixture result;
- hosted workflow run ID;
- confirmation that no publication occurred;
- remaining findings.

## 16. Handoff notes

This corrective exists to make closure evidence match the original plan exactly. Do not conflate "publication-ready" with "published"; crates.io publication remains a separate maintainer action.
