# Verified Update Core Milestone 013 — eggup-core 0.1.3 publication and registry handoff

Status: closed 2026-10-06; `eggup-core 0.1.3` published from `bd43683`, `v0.1.3` + release created

Repository baseline: `44d7fdf6d31d67a2e2c8b0f62aa4f00ec365daeb`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`

Primary class: package promotion / downstream handoff

Hard dependencies:

- Core M010/M011 closed and hosted-green.
- Core M012 closed with a compatibility-preserving package candidate.
- Acquisition M010 closed, which establishes workspace source version 0.1.3 and publishes `eggup-acquisition 0.1.3`.
- Exact-current-head Stable/MSRV/macOS/Windows qualification green.

## 1. Objective

Publish `eggup-core 0.1.3` containing:

- current-executable transaction parity from M010;
- proof-authorized stale-lock recovery from M011;
- the earlier unpublished Core audit fixes;
- the M012 public-error compatibility correction.

This is the versioned Core handoff required before Gregg M004 or EggPool M007 is authored.

## 2. Publication boundary

Publish **only** `eggup-core 0.1.3` in this milestone.

Do not publish `eggup-archive`, `eggup-eggfetch`, `eggup-eggpack`, `eggup-curl`, or `eggup-service` here. Their publication work is independently registered.

The existing `eggup-service 0.1.2` requirement `eggup-core ^0.1.0` means a fresh resolve can select Core 0.1.3. That compatibility is therefore a release blocker, not an optional smoke.

## 3. Pre-publication gates

- exact `eggup-core 0.1.3` absent from crates.io;
- package ownership/publish authority confirmed;
- workspace version exactly 0.1.3;
- M012 compatibility fixture green;
- no Git/path dependencies in the packaged manifest;
- Windows package includes only the intended target-gated `self-replace` dependency;
- Unix/macOS package dependency surface remains sha2-only;
- root/core changelogs accurately distinguish published 0.1.3 content from still-unpublished sibling crates.

## 4. Registry-only proof

After publication, create fresh external fixtures with no patch/path/Git overrides.

### Core direct fixture

Depend on `eggup-core = "=0.1.3"` and exercise:

- ordinary one-member transaction;
- current-executable planning API compile/use;
- stale-lock observation/verifier API compile/use;
- exhaustive match over the legacy `Error` variants;
- typed recovery-specific error match.

### Service transitive fixture

Depend only on `eggup-service = "=0.1.2"`.

Require the generated lockfile to resolve `eggup-core 0.1.3` from crates.io, then compile/run representative service disposition/lifecycle usage.

This proves the published caret edge does not become a broken graph when Core 0.1.3 appears.

## 5. Tag and GitHub release semantics

Do not move `v0.1.1` or `v0.1.2`.

If `v0.1.3` does not yet exist, create it only after Core 0.1.3 is registry-visible and the registry-only proofs pass, pointing at the exact Core publication source commit.

Acquisition M010 may have published `eggup-acquisition 0.1.3` from an earlier commit. Release notes must state each package's exact publication source SHA rather than implying every 0.1.3 package byte originated at the tag commit.

Later Archive/Eggfetch/Eggpack 0.1.3 publications append to the same release notes without moving the tag.

## 6. Ordered work packages

1. Confirm M012 + Acquisition M010 closure.
2. Cut root/core 0.1.3 changelog entries.
3. Run exact clean-tree local/package/dry-run qualification.
4. Push release-prep commit and require green hosted matrix.
5. Publish `eggup-core 0.1.3` manually.
6. Verify crates.io metadata/checksum/source commit.
7. Run direct and service-transitive registry-only fixtures.
8. Create `v0.1.3` + GitHub Release if absent, with package-source truthfulness.
9. Write M013 closure.
10. Reconcile consumer roadmap: Gregg M004 becomes authorable once Acquisition M010 is closed; EggPool M007 becomes authorable immediately on M013 closure.

## 7. Required verification

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
cargo package -p eggup-core --locked
cargo package -p eggup-core --list --locked
cargo publish -p eggup-core --dry-run --locked
```

Then, maintainer-authorized only:

```text
cargo publish -p eggup-core --locked
```

## 8. Acceptance criteria

- Core 0.1.3 visible/non-yanked with recorded checksum and exact publication SHA.
- Registry package exposes M010/M011 APIs and M012-compatible Error surface.
- Registry-only direct fixture green.
- Registry-only `eggup-service 0.1.2 -> eggup-core 0.1.3` graph green.
- Stable/MSRV/macOS/Windows matrix green on publication source.
- No unrelated crate published.
- Tag/release notes are truthful about per-package publication source commits.
- Gregg/EggPool authoring gates updated.

## 9. Stop conditions

Stop before upload if any compatibility fixture fails, package API differs materially from M012 closure, service 0.1.2 cannot resolve/compile against Core 0.1.3, hosted matrix is not green, or publication would require moving an existing tag.

After an accepted upload, use a new patch version for any corrective; never overwrite registry state.

## 10. Closure evidence required

Record release-prep SHA, hosted run id, package inventory/dependency tree, crates.io checksum/metadata, direct fixture lock/test result, service-transitive fixture lock/test result, tag/release state, and consumer-unblock reconciliation.
