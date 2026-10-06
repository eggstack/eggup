# Acquisition Transport Milestone 011 — eggup-eggfetch 0.1.3 correctness publication

Status: active; Acquisition M010 closed (`eggup-acquisition 0.1.3` published 2026-10-06)

Repository baseline: `44d7fdf6d31d67a2e2c8b0f62aa4f00ec365daeb`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`

Primary class: correctness publication

## 1. Objective

Publish `eggup-eggfetch 0.1.3` carrying the already-implemented audit fixes absent from 0.1.2:

- runtime construction failure returns `AcquisitionError::Unavailable` instead of panicking;
- one current-thread Tokio runtime is cached per calling thread without making the transport !Sync;
- `TooLarge` reports the actual enforced artifact limit instead of zero.

No public API change and no acquisition-policy change.

## 2. Dependencies

Hard dependency: Acquisition M010 publishes `eggup-acquisition 0.1.3` and establishes workspace version 0.1.3.

The packaged `eggup-eggfetch 0.1.3` caret dependency must resolve the published acquisition 0.1.3 from crates.io during dry-run/package proof.

## 3. Invariants

- no release/version/fallback-source policy enters the adapter;
- transport composition stays in acquisition/curl;
- HTTPS/proxy/redirect policy unchanged;
- explicit finite connect/total/artifact bounds unchanged;
- `EggfetchTransport` remains shareable across threads;
- no new runtime/API feature is added.

## 4. Required work

1. Confirm exact version absence/ownership.
2. Cut crate/root 0.1.3 changelog.
3. Package/dry-run against registry acquisition 0.1.3.
4. Prove no panic on runtime-construction failure through the existing deterministic seam.
5. Prove `TooLarge.limit` equals the enforced bound.
6. Run thread-sharing/auto-trait regression.
7. Full Stable/MSRV/macOS/Windows qualification.
8. Publish only eggup-eggfetch 0.1.3.
9. Registry-only external fixture, no path/Git overrides.
10. Append release notes and write closure.

## 5. Verification

```text
cargo test -p eggup-eggfetch --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-eggfetch --locked
cargo publish -p eggup-eggfetch --dry-run --locked
```

## 6. Acceptance criteria

Registry 0.1.3 visible/non-yanked; packaged dependency resolves registry acquisition 0.1.3; runtime-build failure is typed not panic; TooLarge carries real limit; thread-sharing contract preserved; hosted matrix green; no unrelated crate published.

## 7. Stop conditions

Any public API or transport-policy change, new fallback behavior, or source change beyond the already-qualified audit fixes stops the publication plan.

## 8. Closure evidence

Publication SHA/checksum, hosted run, package dependency tree, registry fixture lock/test output, and Release-note append.
