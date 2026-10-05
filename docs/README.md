# Eggup documentation

User-facing documentation for adopting Eggup as a library. This directory is
about **how to use it**. The design rationale lives in
[`architecture/`](../architecture/overview.md) and the authoritative contracts
live in [`crates/eggup-core/docs/`](../crates/eggup-core/docs/domain.md) — read
those before treating any summary here as normative.

| Document | Read it when |
|---|---|
| [quickstart.md](quickstart.md) | You are starting from zero and want a verified, runnable walkthrough |
| [crates.md](crates.md) | You know the shape and need to decide which crate(s) to depend on |
| [releases.md](releases.md) | You are shipping, pinning, or upgrading a published Eggup version |

For the internals of any single crate, follow the deep-dive index in
[`architecture/overview.md`](../architecture/overview.md#deep-dive-index-review-entry-points).

## What is authoritative

This directory never owns a normative claim. If a statement here disagrees with
the source, with `crates/eggup-core/docs/`, or with `architecture/`, those win
and this directory is the thing that is wrong — file it in the
[Known doc/code drift](../architecture/tooling-governance.md#known-doccode-drift)
table rather than editing the owner.

Specifically:

- **Public API surface** — the source, not these pages. Names, signatures, and
  enum variants are checked against `crates/*/src/`.
- **Contract semantics** (ownership, integrity, receipts) —
  `crates/eggup-core/docs/{domain,verification,transaction}.md`.
- **Why the workspace is layered this way** — `architecture/`.
- **Which versions exist on crates.io** — crates.io, not this directory.
