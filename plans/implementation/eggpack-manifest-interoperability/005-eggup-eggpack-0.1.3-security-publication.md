# Eggpack Manifest Interoperability Milestone 005 — eggup-eggpack 0.1.3 security/correctness publication

Status: blocked on Acquisition M010, Core M013, and Archive M002

Repository baseline: `44d7fdf6d31d67a2e2c8b0f62aa4f00ec365daeb`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Primary class: security/correctness publication

## 1. Objective

Publish `eggup-eggpack 0.1.3` so registry consumers receive the already-implemented archive-member identity cross-check and the adapter's exact Eggup dependency pins move coherently to the 0.1.3 set.

Published 0.1.2 binds archive members positionally and does not cross-check each bound member's recorded `source_path`/`output_name` against the declared member it is about to represent. A mismatched/reordered `BoundExtraction` can therefore bind bytes under another member identity when content digests happen to match.

The current source fails closed with `AdapterError::MapMismatch`.

## 2. Hard dependencies

Before package/dry-run:

- `eggup-acquisition 0.1.3` published by Acquisition M010;
- `eggup-core 0.1.3` published by Core M013;
- `eggup-archive 0.1.3` published by Archive M002;
- `eggpack-manifest 0.1.0` remains published/non-yanked.

These exact versions are load-bearing because `eggup-eggpack` intentionally exact-pins its Eggup dependency set.

## 3. Required manifest state

The 0.1.3 package must exact-pin:

```toml
eggup-core = "=0.1.3"
eggup-archive = "=0.1.3"
eggup-acquisition = "=0.1.3"
eggpack-manifest = "=0.1.0"
```

No Git/path source may appear in the packaged registry graph.

## 4. Invariants

- adapter selects no release;
- adapter verifies no authenticity;
- manifest install identity does not authorize a filesystem destination;
- caller-bound destination APIs remain unchanged;
- archive extraction remains outside the adapter;
- exact size/digest/member relationship checks remain fail-closed;
- no new transport/TLS dependency;
- no public API change.

## 5. Required tests

- correct declaration-order archive binding still succeeds;
- bound extraction from a different ArchivePlan fails MapMismatch;
- reordered member projection fails MapMismatch;
- identical-content members still cannot substitute identities;
- direct/bundle mapping unchanged;
- caller-bound destination negative matrix unchanged;
- registry-only adapter-only fixture;
- registry-only Eggsact-shaped fixture;
- dependency tree has only registry sources.

## 6. Ordered work

1. Verify all exact 0.1.3 dependencies visible.
2. Re-pin manifest to exact 0.1.3 set if not already done by Acquisition M010.
3. Cut crate/root changelog.
4. Package/dry-run from clean source.
5. Full Stable/MSRV/macOS/Windows qualification.
6. Publish only eggup-eggpack 0.1.3.
7. Run adapter-only and Eggsact-shaped registry fixtures.
8. Append GitHub Release 0.1.3 notes without moving tag.
9. Write closure and mark the published positional-binding defect resolved.

## 7. Verification

```text
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-eggpack --locked
cargo publish -p eggup-eggpack --dry-run --locked
```

## 8. Acceptance criteria

- 0.1.3 registry-visible/non-yanked;
- exact dependency pins resolve from crates.io;
- mismatched/reordered bound members fail closed even with identical content;
- existing valid direct/bundle/archive flows remain green;
- both registry-only fixtures pass;
- hosted matrix green;
- no unrelated crate published.

## 9. Stop conditions

Stop if any dependency 0.1.3 is absent, package resolution uses path/Git, the security fix requires API change, or valid declaration-order binding behavior changes.

## 10. Closure evidence

Publication SHA/checksum, dependency graph, hosted run, mismatch fixture results, registry-only fixture lockfiles/tests, and Release-note append.
