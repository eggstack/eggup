---
name: docs-hygiene
description: Keep AGENTS.md, architecture/, skills, and crate docs truthful against the code — which doc owns which fact, and the drift classes this repo keeps hitting
---

# Eggup documentation hygiene

This repo's docs are load-bearing: `architecture/` is an index layer,
`crates/eggup-core/docs/` plus `plans/` are the contracts. Use this when you
change code, CI, the gate, or the plan system, and the prose that describes it
must move with it.

Deep dive: [`architecture/tooling-governance.md`](../../../architecture/tooling-governance.md)
§7 and the reviewer's checklist §9.

## Who owns which fact

| Fact | Owner | Never restated in |
|---|---|---|
| Verification commands | `scripts/check-local.sh` + `.github/workflows/ci.yml` | prose, skills |
| Lint layering | `Cargo.toml` `[workspace.lints]` + crate `lib.rs` attributes | prose |
| Dependency boundaries | `crates/*/Cargo.toml` | prose |
| Public API surface | source | `architecture/*.md` tables |
| Process, naming, templates | `plans/003-planning-process.md` + `plans/*/README.md` | skills |
| Open work and blockers | `plans/registry.md` | roadmaps, skills |
| Published versions | crates.io | any claim of "unpublished" |

## The chain to update when a fact changes

```text
code / script / workflow
  -> the owning doc (README, deep dive, plans/)
  -> AGENTS.md  (thin index; link to the section, never restate the rule)
  -> .opencode/skills/  (summary only; source of truth stays plans/ + architecture/)
  -> plans/registry.md + roadmap status, if open/closed work changed
```

**`AGENTS.md` is an index, not a second copy.** Each rule in it should name the
`architecture/` section that owns the detail. That is what keeps it short enough
to stay correct.

## Drift classes this repo keeps hitting

Each has occurred at least once — check for them explicitly:

1. **CI lane understatement.** Windows runs four `cargo test` invocations, not
   just `cargo check`. MSRV is compile-only. Never call `windows-check`
   "compile-only".
2. **Publication status.** "Unpublished" is wrong for `eggup-service` — it is on
   crates.io at `0.1.0`/`0.1.1` while its `0.1.2` was never published. Check
   crates.io rather than inferring from a `publish = false` key or a plan.
3. **Enum tables lagging the source.** `architecture/core-transaction.md` once
   described a whole earlier generation of `eggup-core`'s enums. When a variant
   is added, renamed, or removed, update every table that lists variants.
4. **Inverted behavior claims.** A doc once stated the core's `validate` accepted
   a member whose integrity was `NotRequired`; the code rejects it. When a doc
   describes a *gate*, confirm the gate's direction against the source.
5. **A universal rule with a real exception.** "Every crate has README +
   CHANGELOG" does not hold for `eggup-transport-footprint` (binary-only). State
   the exception instead of implying the rule is absolute.
6. **Stale index.** `plans/subsystems/README.md` once omitted a live subsystem
   roadmap. When adding a subsystem, add it to the index.
7. **Falsified git claim.** A registry line asserted
   `git log A..HEAD -- crates/` was empty when it returned commits. Re-run the
   command before repeating a state claim that depends on it.
8. **Broken markdown tables.** Copy-pasting a row between tables of different
   arity silently drops cells. Check column counts.

## Reporting

`architecture/tooling-governance.md` ends with a **Known doc/code drift** table.
Record residual drift there rather than silently leaving it — and remove rows as
they are fixed, so the table stays a live list rather than a historical log.

## Boundary

Doc hygiene is a `planning-closure-hygiene-corrective` workstream when it needs
its own milestone (see the `planning-workflow` skill). Fixing docs that your own
change invalidated is part of that change, not a separate milestone.
