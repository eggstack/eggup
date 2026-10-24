# Planning/Closure Hygiene Corrective 014 — documentation-truth corrective

Status: implemented

Repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`

Source roadmap:
- `plans/registry.md` — the active control surface. This workstream has **no subsystem roadmap**; `plans/subsystems/` holds the seven subsystem roadmaps and none owns documentation reconciliation. The planning-workflow skill defines `planning-closure-hygiene-corrective` as "the docs-only workstream for reconciling the registry, roadmaps, and closure records against reality; it has no subsystem roadmap" (`.opencode/skills/planning-workflow/SKILL.md:88-90`). No subsystem roadmap advances here.
- `plans/003-planning-process.md` — the normative process this milestone executes (registry rules §13, baselines §14, verification honesty §15).
- `architecture/tooling-governance.md` §6 (process) and §7 (doc ownership) — the deep dive owning the doc-change chain.

Original closed work affected:
- `plans/closure/transport-footprint/001-status.md` — the M001 closure whose `ci.yml:` anchors this milestone re-anchors.
- `plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md` — the previous documentation-truth pass; its gate claims this milestone corrects.

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: polish

## 1. Objective

Make the documentation-truth gate actually check documentation truth, and make the deep dives that describe this workspace describe the workspace as it is.

Two defects dominate. **D1:** `scripts/check-docs-truth.sh` is described in three places as comparing the docs "with the manifests", but it opens only `docs/*`, `README.md`, `AGENTS.md`, `CHANGELOG.md`, and `plans/registry.md`. **No `Cargo.toml` is ever read**, so no manifest agreement is enforced at all; `architecture/*.md` — where most drift actually lives — is outside its coverage entirely; its registry assertion skips every table row with fewer than four cells, so the registry's three-column "Recently closed foundation" table is invisible to it; and its crate-scan assertion matches `crates/**/src/**` only, so a new `crates/*/build.rs` escapes it. The gate's own scope statement therefore overstates what it does, and the overstatement is repeated in `architecture/tooling-governance.md` and `.opencode/skills/verify-workflow/SKILL.md`.

**D2–D5:** `architecture/transport-footprint.md` contradicts the branch's own work in five places (the CI coverage table, the "gap is still open" claim, the "no subsystem roadmap" claim, the "zero tests" claim, and the `v0.1.2` version labels in its dependency-tree snippets); the §5 CI matrix in `architecture/tooling-governance.md` names `cargo package -p eggup-service --locked` for the `stable` and `macos` lanes when the workflow runs `--no-verify`; two `ci.yml:` anchors in that same file are stale; and nine of twelve `ci.yml:` anchors in `plans/closure/transport-footprint/001-status.md` resolve to unrelated content after commit `b9885fa` shifted the workflow's lines.

This milestone changes no production source, adds no dependency, publishes nothing, and moves no git state.

## 2. Why this milestone is ready

Every finding below was characterized by reading the files at the baseline above, not inferred, and each was re-verified at implementation time against the current tree.

**Severity, stated explicitly.** D1 and D2 are **medium**: a contributor is told the gate enforces something it does not, and therefore trusts a green gate that means less than the gate's own documentation claims; and a contributor is told the isolated transport configurations are never built in CI, which is false on the branch's own workflow. D3 and D4 are **low**: a stale command string and two stale line anchors in a reference table. D5 is **low** but is a closure-integrity defect: a closure record that cites the wrong lines is not evidence.

**Honest scoping of the harm.** These are documentation-truth defects. There is no failing production report, no consumer incident, no hosted-run failure, and no runtime consequence: no code path, lock, receipt, or installed artifact is affected. The severity is about decision-correctness for a reader.

**The detection gap is known and specific.** `architecture/tooling-governance.md` §4.1 enumerates the gate's assertions; the gate's scope sentence and the `verify-workflow` skill both promise manifest agreement that the script never performs. The gate was closed on compilation and a green run, not on a demonstrated negative control.

## 3. Current implementation evidence

- `scripts/check-docs-truth.sh` — the gate. It reads `docs/*.md`, `README.md`, `AGENTS.md`, `CHANGELOG.md`, and `plans/registry.md`; it reads **no** `Cargo.toml`. Assertion 1 checks version literals on `SURFACES`; assertion 3 checks five hard-coded normative constructs in `docs/*` only; assertion 4 checks the root changelog's `Unreleased` disclaimer; assertion 5 checks the registry, skipping rows with `len(cells) < 4`; assertion 7 scans `crates/**/src/**` for `unsafe`, `TODO`, `FIXME`, and `dbg!`.
- `architecture/tooling-governance.md:227` and `.opencode/skills/verify-workflow/SKILL.md:29-31` — both describe the gate as comparing the docs "with the manifests".
- `architecture/transport-footprint.md:190`, `:210`, `:212`, `:222`, `:5`, `:115-127` — the five contradicting statements and the stale version labels.
- `architecture/tooling-governance.md:282`, `:285`, `:580`, `:777` — the stale CI matrix rows and anchors.
- `plans/closure/transport-footprint/001-status.md` — twelve `ci.yml:` anchors, nine of them stale.
- `.github/workflows/ci.yml` — the authoritative workflow: the three `stable` footprint builds, the three `msrv` footprint checks, the two `macos` footprint builds, the `service-systemd` job, the `windows-check` job and its ten test steps, and the four `cargo package` steps.

## 4. Invariants that must not regress

1. **The gate still fails closed.** A false statement in any checked surface must produce a non-zero exit. The extension may not add a mode that skips failures.
2. **The gate stays dependency-free and fast.** POSIX shell plus `python3` only; no network; no new tool; a full run must stay in the sub-second range.
3. **No false positives on the current tree.** Every added assertion must pass against the repository as it stands at closing time, including the registry repair the parent session is applying concurrently.
4. **The dependency counts 3 / 92 / 93 are not touched.** They reproduce exactly under the method in `plans/closure/transport-footprint/001-status.md` §13.7 and are out of scope here.
5. **No git state mutation.** No commit, stash, checkout, restore, reset, clean, tag, or push from this milestone. The parent owns all commits.
6. **No production source change.** `crates/**` is read-only for this milestone.

## 5. Scope

### In scope

- `scripts/check-docs-truth.sh` — extend the gate: read the crate manifests, extend coverage to `architecture/*.md`, stop skipping three-cell table rows, cover `build.rs` alongside `src/**`, keep every existing assertion working.
- `architecture/transport-footprint.md` — correct D2's five contradictions and rewrite the CI coverage table to the current truth.
- `architecture/tooling-governance.md` — correct D3's CI matrix rows, D4's stale anchors, and the gate's scope statement.
- `plans/closure/transport-footprint/001-status.md` — re-anchor every `ci.yml:` citation (D5).
- `plans/implementation/planning-closure-hygiene-corrective/014-documentation-truth-corrective.md` — this plan.
- `plans/closure/planning-closure-hygiene-corrective/014-status.md` — the closure record and the ready-to-paste appendix for files this milestone does not own.

### Explicitly out of scope

- `plans/registry.md` — the parent owns it; this milestone does not touch it.
- `docs/**`, `AGENTS.md`, `.opencode/skills/**`, `.github/**`, `scripts/check-local.sh`, `crates/**`, the root `CHANGELOG.md`, `plans/subsystems/**` — owned by others or by the parent. Corrections needed there are delivered as a ready-to-paste appendix, not applied.
- Any change to the dependency counts, the dependency-tree structure, or the `Cargo.toml` files themselves.
- Any change to the `stable`/`msrv`/`macos`/`windows-check` lane structure. The workflow is read as the source of truth, never edited.
- `architecture/*.md` other than `transport-footprint.md` and `tooling-governance.md`.
- Publishing, tagging, releasing, or any network operation.

## 6. Required production changes

There are none. This milestone is documentation plus one shell script. `crates/**`, `.github/**`, and every manifest are untouched.

## 7. Ordered work packages

1. **WP1 — Extend the gate.** In `scripts/check-docs-truth.sh`:
   - **WP1.1** Add a manifest reader: the root `Cargo.toml` `[workspace.package] version`, and for each `crates/*/Cargo.toml` the crate name, its version (literal or `version.workspace = true`), its `publish` flag, and its declared dependency edges (regular, dev, build, target-specific, optional). The reader is a deliberately minimal parser over the fields the gate checks — not a general TOML implementation — so the gate keeps zero dependencies and one behaviour on every interpreter.
   - **WP1.2** Extend assertion 1's surface list to `architecture/*.md`, and derive the accepted version set from the manifests unioned with the registry-known set, so a version the manifests declare is always accepted and a version nobody declares is not.
   - **WP1.3** Extend assertion 3's scope to `architecture/*.md` and widen its rule-construct list, exempting the owning deep dive so the owner is not required to link itself.
   - **WP1.4** Change assertion 5 to include three-cell rows, choosing the status cell from the table's own column count rather than a fixed index.
   - **WP1.5** Extend assertion 7 to `crates/*/build.rs` as well as `crates/**/src/**`.
   - **WP1.6** Add assertion 8 (manifest agreement): workspace-version claims; published-version claims against the crate's own `CHANGELOG.md` released headings and the manifest's `publish` flag; and dependency claims as a subset of the manifest's declared edges.
2. **WP2 — Red-first proof.** In a `/tmp` copy of the tree, show the pre-fix script exiting 0 on the demonstrated false negatives and the post-fix script exiting non-zero on the same mutations, and exiting 0 on the unmodified tree.
3. **WP3 — Correct `architecture/transport-footprint.md`** (D2).
4. **WP4 — Correct `architecture/tooling-governance.md`** (D3, D4, and the gate scope statement).
5. **WP5 — Re-anchor `plans/closure/transport-footprint/001-status.md`** (D5).
6. **WP6 — Write the closure record** with the requirement→evidence matrix, exact commands, per-platform results, and the ready-to-paste appendix.

## 8. Failure, cancellation, restart, and contention semantics

- **Gate failure semantics.** Every assertion prints one `FAIL <file>:<line>: <what>` line naming the surface, the line, and the disagreement, and the script exits non-zero after all assertions have run — a run reports every defect it can find, not just the first. A check that cannot decide (an ambiguous subject, a crate with no changelog, a version claim naming two crates at once) is **skipped, never guessed**; a skip is reported as a `note`, not a failure.
- **Read-only.** The gate writes nothing. It creates no temporary file inside the repository.
- **Cancellation.** The gate is a single non-interactive process; there is no partial state to unwind. Interrupting it leaves the tree untouched.
- **Restart.** Re-running is idempotent; the gate has no memory between runs.
- **Contention.** The parent session is repairing `plans/registry.md` and `docs/crates.md` concurrently. The gate reads those files at run time and never writes them, so a concurrent repair can only change the gate's verdict, never corrupt it. The final verification run is recorded with the tree state it observed, and any failure attributable to a concurrent edit is reported as such rather than silently suppressed.

## 9. Compatibility and migration

None for consumers. The gate's exit status is unchanged in meaning (0 = documentation agrees, non-zero = it does not); it is only **stricter**, and it is stricter in both directions of false negatives and false positives. No published crate, tag, manifest, or dependency graph moves. The workspace version stays `0.1.3`.

## 10. Required tests

The gate's own assertions are the tests. The milestone additionally requires:

- a **red-first proof**: the pre-fix script exits 0 on at least one demonstrated false negative and the post-fix script exits non-zero on the same mutation, in a `/tmp` copy, with the repository untouched;
- a **green proof**: the post-fix script exits 0 on the unmodified tree;
- a **regression proof**: every pre-existing assertion still fires on its own mutation (spot-checked), so the extension did not silently disable existing coverage.

No new Rust test is required: no production code changes.

## 11. Required verification commands

```sh
./scripts/check-docs-truth.sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l
```

The three counts must remain **3 / 92 / 93**. `./scripts/check-local.sh` is the workspace gate and is run by the parent session; this milestone runs the scoped commands above and records the full-gate result only if it is run.

## 12. Documentation updates

- `architecture/transport-footprint.md` — D2's five corrections and the CI coverage table.
- `architecture/tooling-governance.md` — D3's CI matrix rows, D4's anchors, and the gate's scope sentence in §4.1.
- `.opencode/skills/verify-workflow/SKILL.md` — the gate's scope sentence, delivered as a ready-to-paste appendix (the file is owned by the parent).
- `AGENTS.md` and `.opencode/skills/docs-hygiene/SKILL.md` — the Windows-lane step count, delivered as a ready-to-paste appendix (the parent had already applied the "nine"→"ten" correction when this milestone reached implementation; the appendix records the applied text and any residual).

## 13. Acceptance criteria

1. `scripts/check-docs-truth.sh` reads at least one `Cargo.toml` and enforces workspace-version, published-version, and dependency agreement.
2. `scripts/check-docs-truth.sh` covers `architecture/*.md` for version literals and for normative-rule owner links.
3. `scripts/check-docs-truth.sh` no longer skips three-cell table rows.
4. `scripts/check-docs-truth.sh` matches `crates/*/build.rs` as well as `crates/**/src/**`.
5. Every pre-existing assertion still fires on its own mutation.
6. The red-first proof is recorded with exact commands and outputs.
7. `architecture/transport-footprint.md` no longer contains any of D2's five contradictions, and its dependency counts still read 3 / 92 / 93.
8. `architecture/tooling-governance.md`'s `stable` and `macos` CI matrix rows name the commands the workflow actually runs and state the deferral reason the other rows state.
9. Every `ci.yml:` line number written by this milestone is verified against the current `.github/workflows/ci.yml`.
10. Every `ci.yml:` citation in `plans/closure/transport-footprint/001-status.md` resolves to the content it claims.
11. The closure record states per-platform results separately, with "not run" for anything not run.
12. The closure record carries the ready-to-paste appendix verbatim.

## 14. Stop conditions

Stop and report to the parent if any of the following occurs:

- a new assertion cannot be made to pass against the current tree **without** weakening an existing assertion;
- the red-first proof cannot be demonstrated for any of the five demonstrated false negatives;
- the registry repair concurrent with this milestone makes the gate's verdict depend on an in-flight edit (report the verdict and the edit);
- any `ci.yml` line number cannot be confirmed against the current workflow file;
- the dependency counts 3 / 92 / 93 stop reproducing.

## 15. Closure evidence required

- The red-first proof transcript (pre-fix exit 0, post-fix exit non-zero, unmodified post-fix exit 0).
- The requirement→evidence matrix with one row per acceptance criterion.
- The exact commands run, with their real outputs.
- Per-platform results reported separately for Linux, macOS, and Windows, with "not run" where applicable.
- The dependency-count reproduction, with the exact commands.
- The ready-to-paste appendix, verbatim.

## 16. Handoff notes

- The parent owns `plans/registry.md`; the registry's "Planning/closure hygiene corrective" row must advance to `C001-C014` when this milestone closes. That edit is **not** performed here.
- The parent owns `AGENTS.md` and `.opencode/skills/**`; the appendix in the closure record carries the exact replacement text for anything still outstanding there.
- The gate is stricter after this milestone. A green run now means more than it did at the baseline; if a future change trips a new assertion, that is the gate doing its job, not a regression.
- No commit, tag, or publication is performed by this milestone.
