---
name: release-workflow
description: Eggup manual crates.io publication — milestone-gated, dependency-ordered, never from CI, and the exact-pin cascade that forces multi-crate releases
---

# Eggup release workflow

**CI never publishes.** No job in `.github/workflows/ci.yml` runs `cargo publish`,
and `permissions` is `contents: read` only. Every release is a manual,
plan-gated action. Deep dive:
[`architecture/tooling-governance.md`](../../../architecture/tooling-governance.md) §8.

## Publication is a milestone, not a side effect

Do not publish because code changed. Publication requires its own plan under
`plans/implementation/<subsystem>/NNN-*-publication.md` and a closure record
(see the `planning-workflow` skill).

Precedent: Verified Core M009 (`eggup-core` → `eggup-archive` 0.1.2),
Eggpack Interop M004 (`eggup-acquisition` → `eggup-eggfetch` → `eggup-eggpack`
0.1.2), Acquisition M009 (`eggup-curl` 0.1.2), Acquisition M010
(`eggup-acquisition` 0.1.3, the current open milestone).

## Order matters: publish dependencies first

A crate is published only after every in-workspace dependency it names is
available on the registry. M004 proved this load-bearing: the three crates had
to ship in seam → adapter → leaf order.

## The exact-pin cascade

`eggup-eggpack` pins its Eggup dependencies exactly (`=0.1.2`). Every other
crate uses a caret requirement. Consequence:

- A fix published to a caret consumer (`eggup-curl`, `eggup-eggfetch`,
  `eggup-service`) is inherited automatically on the next resolve.
- The same fix **cannot** reach `eggup-eggpack 0.1.2` consumers. The adapter
  needs a **separate republication** to inherit a seam fix.

So a single seam fix can become a two-publication event. Name that follow-on
explicitly in the plan and in the registry, or it will be discovered late.

## Never

- Do not re-publish a version that already exists on crates.io. A fix needs a new
  version (`0.1.2` → `0.1.3`).
- Do not move `v0.1.1` / `v0.1.2` tags. Extend release notes instead.
- Do not publish `eggup-transport-footprint` — it is `publish = false`.
- Do not publish `eggup-service` without explicit maintainer authorization for
  Service M011. It is published through `0.1.2`; the M010 correction is the
  unpublished `0.1.3` candidate.
- Do not add release CI. Manual publication is a recorded decision.

## Qualification before publishing

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo package -p <crate> --locked     # resolves against the registry, not path deps
```

`cargo package` is the check that matters most: it proves the crate builds
against the **registry-downloaded** dependency rather than the in-tree path
dependency. A publication milestone should also record a registry-only fixture
result — one registry source per package, no git or path edge.

`plans/003-planning-process.md` §16 requires package dry-run as part of
boundary qualification.

## After publishing

1. Write the closure record with the publication source SHA, checksums, and the
   registry-only proof.
2. Update `plans/registry.md`: publication set, and any downstream republication
   that the exact pin now requires.
3. Update the subsystem roadmap status table.
4. Update the crate `CHANGELOG.md` — move entries out of `Unreleased` under the
   published version, and drop the no-publication disclaimer for that release.
5. Update the release table in
   [`architecture/tooling-governance.md`](../../../architecture/tooling-governance.md) §8
   and the publication paragraph in
   [overview.md](../../../architecture/overview.md).
