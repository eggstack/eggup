# Planning/Closure Hygiene Corrective 015 — architecture anchor and claim corrective

Status: implemented

Repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`

Source roadmap:
- `plans/registry.md` — the active control surface. This workstream has **no subsystem roadmap**; `plans/subsystems/` holds the seven subsystem roadmaps and none owns architecture deep dives. The planning-workflow skill defines `planning-closure-hygiene-corrective` as "the docs-only workstream for reconciling the registry, roadmaps, and closure records against reality; it has no subsystem roadmap" (`.opencode/skills/planning-workflow/SKILL.md:88-90`). No subsystem roadmap advances here.
- `plans/003-planning-process.md` — the normative process this milestone executes (registry rules §13, baselines §14, verification honesty §15).
- `architecture/tooling-governance.md` §6 (process) and §7 (doc ownership) — the deep dive owning the doc-change chain.

Original closed work affected:
- `plans/implementation/planning-closure-hygiene-corrective/014-documentation-truth-corrective.md` — the sibling documentation-truth milestone authored concurrently by another agent. This milestone does not edit it; it follows the same doc-ownership chain.
- `plans/closure/service-lifecycle/013-status.md` — the service M013 closure whose `ManagerFailureKind` taxonomy replaced the removed `permission_hint`, which this milestone's deep-dive correction names.

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: polish

## 1. Objective

Re-derive every `path:line` anchor in the five architecture deep dives whose source crates are **not** being edited concurrently, and correct every substantive claim in them that the code contradicts.

**Why this gap exists.** A prior corrective pass re-derived 376 stale `path:line` citations across four architecture deep dives — `core-transaction.md`, `curl-adapter.md`, `acquisition.md`, `eggfetch-adapter.md`. It did **not** cover the other five. An audit then found the remaining drift, including claims that are now false rather than merely mis-anchored. This milestone closes the gap for the deep dives whose source crates are stable.

**Scope, exactly five files.** `architecture/service-lifecycle.md`, `architecture/eggpack-adapter.md`, `architecture/archive-extraction.md`, `architecture/eggfetch-adapter.md`, `architecture/overview.md`. Their source crates (`crates/eggup-service/src/**`, `crates/eggup-eggpack/src/**`, `crates/eggup-archive/src/**`, `crates/eggup-eggfetch/src/**`) are not being modified by any other agent, so an anchor derived here will not go stale under this milestone.

This milestone changes no production source, adds no dependency, publishes nothing, and moves no git state.

## 2. Why this milestone is ready

Every finding below was characterized by reading the files at the baseline above, not inferred, and each was re-verified at implementation time against the current tree.

**Severity, stated explicitly.** The confirmed defects are **medium**: a normative deep dive documents a removed public API (`permission_hint`) and a removed version pin (`=0.1.2`), and states test and line-count figures that the code contradicts. A reader following `architecture/service-lifecycle.md` would look for an API that no longer exists and would draw a wrong conclusion about the crate's published version and its dependency pins. The remaining anchor drift is **low**: an off-by-N line reference costs a reader one grep.

**Honest scoping of the harm.** These are documentation-truth defects. There is no failing production report, no consumer incident, no hosted-run failure, and no runtime consequence: no code path, lock, receipt, or installed artifact is affected. The severity is about decision-correctness for a reader.

**The detection gap is known and specific.** `scripts/check-docs-truth.sh` asserts version literals, publication claims, and manifest agreement across `docs/`, `README.md`, `AGENTS.md`, and `architecture/*.md`. It does **not** resolve `path:line` anchors, so a deep dive that points at the wrong line passes the gate unchanged. The prior corrective pass repaired 376 anchors by hand; nothing prevents the remaining ones from drifting again.

**A convention this milestone must respect.** These deep dives cite files by **bare basename with crate scope**: `lib.rs:495` means line 495 of the crate the section is about, not the root `Cargo.toml` or some other crate's `lib.rs`. A citation is only wrong if it does not resolve **in its own crate's scope**. Already-verified-correct examples that must not be "fixed": `architecture/curl-adapter.md:6` `Cargo.toml:23-24` (→ `crates/eggup-curl/Cargo.toml`) and `architecture/curl-adapter.md:990` `README.md:20` (→ `crates/eggup-curl/README.md`).

## 3. Current implementation evidence

- `architecture/service-lifecycle.md` — the deepest drift of the five. 270 citation occurrences, **168 distinct anchors stale**. The public-surface table, the ownership-rules table, the invariants table, the three adapter sections, and the testing inventory were all written against `crates/eggup-service/src/lib.rs` at ~4.5k lines; the file is now 5 536 lines after service M012/M013/M014 landed. The removed `permission_hint` API is documented in two places.
- `architecture/eggpack-adapter.md` — 151 citation occurrences, **7 distinct anchors stale**, plus two substantive falsehoods: the header claims publication at `0.1.2` and the body claims the Eggup edges stay `=0.1.2`, while the registry cache holds `eggup-eggpack-0.1.3` and `crates/eggup-eggpack/Cargo.toml:20-22` pins `=0.1.3`. A drift row records `docs/crates.md:117` as still carrying `=0.1.2`; that line has been repaired and now carries the `cargo tree --workspace` review rule.
- `architecture/archive-extraction.md` — 56 citation occurrences, **4 distinct anchors stale**, plus a stale header figure (`3512 lines, 51 `#[test]` functions` against 3 717 and 52).
- `architecture/eggfetch-adapter.md` — 216 citation occurrences, **0 stale**. This is one of the four deep dives the prior corrective pass already repaired; this milestone re-derived all 216 independently and found no drift.
- `architecture/overview.md` — no `path:line` anchors at all (it is an index). It carries **8 stale LOC/test figures**, **4 substantive falsehoods** about `eggup-core`'s dependency set, and **3 stale source-layout lists** (the `eggup-service` module list omits `commit_engine.rs` entirely; `eggup-core` omits `failure_taxonomy_tests.rs` and two integration test files; `eggup-eggpack` omits `digest_reuse.rs`).
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/eggup-eggpack-0.1.3/` — the local registry cache, which confirms the published version without network access.

## 4. Invariants that must not regress

1. **No production source changes.** This is a documentation-only milestone. No `crates/**/src/**` file, no crate manifest, and no `Cargo.lock` may change.
2. **The documentation-truth gate still passes.** `./scripts/check-docs-truth.sh` must exit 0 against the corrected tree. A correction that trips assertion 8 (publication claim vs. the crate's own CHANGELOG) or assertion 2 (a `=0.1.2` claim) is a defect in the correction.
3. **No false "fixes".** A citation that already resolves correctly in its own crate's scope must not be changed. The count of changed anchors is reported, and every changed anchor must have been re-derived by symbol.
4. **No git state mutation.** No commit, stash, checkout, restore, reset, clean, tag, or push from this milestone. The parent owns all commits.
5. **No edit outside the owned paths.** The five architecture files and this milestone's own plan and closure only.
6. **Cross-crate anchors are flagged, not silently relied on.** A `crates/eggup-core/...` or `crates/eggup-acquisition/...` anchor inside an owned file may move when the concurrent core/acquisition agents land; each such anchor is listed in the closure's cross-crate watch list.

## 5. Scope

### In scope

- `architecture/service-lifecycle.md`
- `architecture/eggpack-adapter.md`
- `architecture/archive-extraction.md`
- `architecture/eggfetch-adapter.md`
- `architecture/overview.md`
- `plans/implementation/planning-closure-hygiene-corrective/015-architecture-anchor-corrective.md` (this plan)
- `plans/closure/planning-closure-hygiene-corrective/015-status.md` (the closure record)

### Explicitly out of scope

- `architecture/acquisition.md`, `architecture/curl-adapter.md`, `architecture/core-transaction.md`, `architecture/transport-footprint.md`, `architecture/tooling-governance.md` — owned by other agents; their sources are being edited concurrently. Their stale citations are reported as a ready-to-paste appendix in the closure for the parent to apply after their sources settle.
- `plans/registry.md`, `plans/subsystems/**` — the parent owns them; this milestone supplies a ready-to-paste registry row and roadmap row.
- The root `CHANGELOG.md`, `AGENTS.md`, `docs/**`, `.opencode/skills/**`, `.github/**`, `scripts/**`, `crates/**` — no edits.
- Any code change. If the audit finds that a code change is required, this milestone reports it instead of making it.

## 6. Required production changes

None. The milestone is documentation-only by construction.

The audit found no defect that requires a code change. One observation is reported for the parent rather than acted on: `crates/eggup-service/src/lib.rs:3126` — `validate_marker`'s control-character check already rejects `\n` and `\r`, so the `marker.contains(['\n', '\r'])` arm at `:3129` is unreachable. It is harmless, and the deep dive already records it as a known imprecision. No code change is proposed.

## 7. Ordered work packages

1. **WP1 — Establish the citation convention and the resolution method.** Build a resolver that maps every cited basename to a file inside the owning crate, falling back to explicit repo-relative paths. Confirm the two already-verified-correct examples (`curl-adapter.md:6`, `curl-adapter.md:990`) resolve as the convention describes, so the resolver is not over-eager.
2. **WP2 — Re-derive `architecture/service-lifecycle.md`.** Work by symbol, not by line: for each citation, find the symbol the sentence names, look up its true current line, rewrite the citation. Cover the public-surface table, the ownership-rules table, the disposition and lifecycle sections, the invariants table, the adapter sections, and the testing inventory.
3. **WP3 — Re-derive `architecture/eggpack-adapter.md`.** Same method, plus the two version claims and the repaired-`docs/crates.md` drift row.
4. **WP4 — Re-derive `architecture/archive-extraction.md`.** Same method, plus the header LOC/test figure.
5. **WP5 — Re-derive `architecture/eggfetch-adapter.md`.** Same method. Expected outcome: no change; the independent re-derivation is the evidence.
6. **WP6 — Correct `architecture/overview.md`.** LOC/test figures, the `eggup-core` dependency claims, and the three source-layout lists.
7. **WP7 — Verify.** Re-resolve every citation in the five files and confirm no anchor is blank, out of range, or comment-only where the sentence claims code. Run the documentation-truth gate.
8. **WP8 — Report.** Produce the closure record with the requirement→evidence matrix, per-platform results, the cross-crate watch list, and the ready-to-paste appendix for the files this milestone does not own.

## 8. Failure, cancellation, restart, and contention semantics

Not applicable. This milestone mutates no artifact, spawns no process, takes no lock, and changes no runtime behavior. Its only failure mode is a wrong correction, which §13 makes checkable: every changed anchor must be re-derivable by symbol from the current tree, and the two-class check in WP7 must find no surviving blank, out-of-range, or comment-only anchor in a sentence that claims code.

**Concurrency note.** Four crates were being edited by other agents while this milestone ran: `eggup-core` (`src/candidate.rs`), `eggup-curl` (`src/lib.rs`), `eggup-acquisition` (tests + CHANGELOG), and `eggup-service` (CHANGELOG only). `crates/eggup-service/src/**` — the crate this milestone's largest correction depends on — was **not** touched. The volatile figures are confined to `architecture/overview.md`'s `eggup-core` and `eggup-curl` rows and to the cross-crate anchors listed in the closure; both are explicitly flagged.

## 9. Compatibility and migration

None. No public API, manifest, version, dependency pin, tag, or published artifact is touched. The `0.1.2` → `0.1.3` corrections in the eggpack deep dive are corrections of a **description** of an already-published crate, not a republish: `eggup-eggpack 0.1.3` is already on crates.io and already pins `=0.1.3` in its published manifest.

## 10. Required tests

No new tests. This milestone adds no code and no test.

The verification is documentary: an independent anchor re-derivation (WP2–WP6), the blank/out-of-range/comment-only sweep (WP7), and the existing documentation-truth gate.

## 11. Required verification commands

```sh
# The documentation-truth gate (assertion 2, 8, 9 over the architecture surfaces).
./scripts/check-docs-truth.sh

# The repository's full gate — not required by this milestone, because it
# changes no source; it is listed so the parent can attribute any failure to
# the concurrent agents' in-flight edits rather than to this milestone.
./scripts/check-local.sh
```

Per-platform results are reported separately in the closure, and anything not run
says "not run" — never inferred.

## 12. Documentation updates

The five owned architecture files, plus this plan and the closure record. No other documentation surface is edited.

The doc-change chain is followed: a deep dive owns the fact it states; `architecture/overview.md` is an index and points at the deep dives rather than restating their contracts; `docs/` is non-normative and is not touched.

## 13. Acceptance criteria

1. Every `path:line` citation in the five owned files resolves in its own crate's scope to the symbol or construct the sentence names.
2. No citation in the five owned files is blank, out of range, or points at a comment where the sentence claims code — except where the sentence explicitly names a doc comment, a crate attribute, or a comment.
3. The removed `permission_hint` API is no longer documented anywhere in `architecture/service-lifecycle.md`; the row names what exists (`ManagerFailureKind::Permission`).
4. `architecture/eggpack-adapter.md` states `0.1.3` for publication and `=0.1.3` for the Eggup pins, and its `docs/crates.md` drift row reflects that the drift is repaired.
5. `architecture/overview.md` carries no claim that `eggup-core` depends on nothing but `sha2`; the Windows-only `self-replace` edge is stated where the dependency set is stated.
6. `./scripts/check-docs-truth.sh` exits 0.
7. The closure record states the number of anchors changed per file, quotes every substantive falsehood before and after, reports per-platform results honestly, and lists the cross-crate anchors that may need re-derivation.
8. No file outside the owned paths is modified.

## 14. Stop conditions

- Stop and report if a source crate this milestone anchors against (`crates/eggup-service/src/**`, `crates/eggup-eggpack/src/**`, `crates/eggup-archive/src/**`, `crates/eggup-eggfetch/src/**`) begins changing under another agent: every derived anchor would be stale.
- Stop and report if `scripts/check-docs-truth.sh` fails for a reason attributable to this milestone's corrections, rather than to a concurrent agent's in-flight edits.
- Stop and report if a correction would require a production change; it is reported instead.
- Do not edit a file owned by another agent, even if the correction is obvious.

## 15. Closure evidence required

- The requirement→evidence matrix (requirement | evidence | result).
- The exact commands run, with their real output.
- Per-platform results reported separately: Linux, macOS, Windows — each on its own line, with "not run" where a platform was not exercised.
- The number of anchors changed per file, and the count of anchors verified as already correct.
- Every substantive falsehood quoted before and after.
- The cross-crate watch list: anchors that point into a crate whose source is being edited concurrently.
- The ready-to-paste appendix for `architecture/acquisition.md`, `architecture/curl-adapter.md`, and `architecture/core-transaction.md`, with the true current line for each stale citation.
- The ready-to-paste registry row (≤ 481 bytes) and roadmap row.

## 16. Handoff notes

- **The parent applies the appendix.** The stale citations in `architecture/acquisition.md`, `architecture/curl-adapter.md`, and `architecture/core-transaction.md` are reported, not fixed, because their source crates are in flight. The appendix gives the true current line for each.
- **The parent applies the registry and roadmap rows.** This milestone does not edit `plans/registry.md` or `plans/subsystems/**`.
- **Two `architecture/overview.md` figures are volatile.** `eggup-core` and `eggup-curl` were being edited concurrently; their LOC figures are a snapshot and must be re-derived once those edits land. The command is in the closure.
- **`crates/eggup-core/src/candidate.rs` anchors are cross-crate.** The three anchors this milestone set in `architecture/service-lifecycle.md` point into a file under concurrent edit and must be re-verified by the parent.
