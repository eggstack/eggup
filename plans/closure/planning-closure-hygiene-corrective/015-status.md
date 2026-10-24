# Planning/Closure Hygiene Corrective 015 — architecture anchor and claim corrective

Status: closed

Source plan: [`plans/implementation/planning-closure-hygiene-corrective/015-architecture-anchor-corrective.md`](../implementation/planning-closure-hygiene-corrective/015-architecture-anchor-corrective.md)

Source roadmap: none — `planning-closure-hygiene-corrective` is the docs-only reconciliation track and has no subsystem roadmap (`plans/003-planning-process.md` §13; `.opencode/skills/planning-workflow/SKILL.md:88-90`). `plans/subsystems/` holds the seven subsystem roadmaps; none owns architecture deep dives. Registry disposition is recorded in §"Roadmap disposition" below.

Reviewed repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`

Implementation commits/PRs: **none from this milestone.** The parent owns all commits; this milestone mutated no git state (no commit, stash, checkout, restore, reset, clean, tag, or push). The changed files are staged in the working tree for the parent to commit:

| Path | Change |
|---|---|
| `architecture/service-lifecycle.md` | 168 `path:line` anchors re-derived; 2 substantive falsehoods corrected; 4 source-layout figures corrected |
| `architecture/eggpack-adapter.md` | 7 `path:line` anchors re-derived; 3 substantive falsehoods corrected |
| `architecture/archive-extraction.md` | 4 `path:line` anchors re-derived; 1 source-layout figure corrected |
| `architecture/eggfetch-adapter.md` | 0 changed — 216 anchors independently re-derived and confirmed correct |
| `architecture/overview.md` | 8 LOC/test figures, 4 dependency claims, 3 source-layout lists corrected |
| `plans/implementation/planning-closure-hygiene-corrective/015-architecture-anchor-corrective.md` | new (this milestone's plan) |
| `plans/closure/planning-closure-hygiene-corrective/015-status.md` | new (this closure record) |

## Executive finding

The audit was correct that the prior corrective pass covered four deep dives and left five. It understated the size of the remaining drift in one of them: `architecture/service-lifecycle.md` had **168 distinct stale anchors** — its entire public-surface table, ownership-rules table, invariants table, adapter sections and testing inventory still described `crates/eggup-service/src/lib.rs` at ~4.5k lines, while the file is now 5 536 lines after service M012/M013/M014 landed. Two of its statements were not merely mis-anchored but **false**: the deep dive documented `permission_hint`, an API that service M013 removed, and it carried `=0.1.2` version claims in the eggpack deep dive while the published crate is `0.1.3`.

`architecture/eggfetch-adapter.md` required **no correction at all**: all 216 of its anchors were independently re-derived and confirmed to resolve. That is a positive result, and it is recorded as one.

**No code change is required by this audit.** One observation is reported rather than acted on (§"Unresolved findings").

## Requirement → evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| R1 — Every `path:line` citation in the five owned files resolves in its own crate's scope to what the sentence claims | Independent re-derivation by **symbol**, then an automated sweep of all 702 citation occurrences: 0 blank, 0 out-of-range. The only comment-only hits are sentences that explicitly name a doc comment, a crate attribute, or a code comment | **pass** |
| R2 — Work by symbol, not by line | Each rewrite was located by grepping the crate's symbol index (definition site of the named item), not by adjusting a number. Example: `lib.rs:143` → `lib.rs:251` for `LifecycleSnapshot`, located via `grep -n "pub struct LifecycleSnapshot"`, not by scanning | **pass** |
| R3 — The removed `permission_hint` API is no longer documented | `grep -rn "permission_hint" crates/ architecture/` → 0 hits in `architecture/`. The two places that named it now name `ManagerFailureKind::Permission` (`lib.rs:339`) and the SCM bounded detail (`windows_scm.rs:279-288`) | **pass** |
| R4 — The eggpack deep dive states the published version and the pin version correctly | `ls -d ~/.cargo/registry/src/index.crates.io-*/eggup-eggpack-*` → `eggup-eggpack-0.1.3` only. `crates/eggup-eggpack/Cargo.toml:20-22` → `=0.1.3` on all three Eggup edges | **pass** |
| R5 — The `docs/crates.md` drift row is reconciled | `docs/crates.md:117` is the `cargo tree --workspace` review rule; `docs/crates.md:119-120` states `=0.1.3`. The drift row now records the drift as **repaired** | **pass** |
| R6 — `architecture/overview.md` states `eggup-core`'s dependency set truthfully | `crates/eggup-core/Cargo.toml` → `[dependencies] sha2 = "0.10.9"` plus `[target.'cfg(windows)'.dependencies] self-replace = "1.3.0"`. All three `sha2`-only claims corrected | **pass** |
| R7 — The documentation-truth gate passes | `./scripts/check-docs-truth.sh --baseline bf439f5…` → **`documentation-truth checks passed`, EXIT=0**, run against `HEAD` plus only this milestone's changes (isolation method in §"Commands run") | **pass** |
| R8 — No production source changed | The same isolated run's assertion 7 (no production-source delta against `bf439f5`) passed. `git diff --name-only` shows only `architecture/*.md` and `plans/**` for this milestone | **pass** |
| R9 — No file outside the owned paths was modified | `git diff --name-only` for this milestone lists exactly the five architecture files plus this plan and closure | **pass** |
| R10 — No git state mutated | No `git commit`/`stash`/`checkout`/`restore`/`reset`/`clean`/`tag`/`push` was run against the working repository. The only git operations were read-only (`git show`, `git ls-files`, `git diff`, `git status`, `git log`, `git cat-file`) | **pass** |
| R11 — Ready-to-paste appendix for the unowned files | §"Appendix A — ready-to-paste corrections for files this milestone does not own" | **pass** |
| R12 — Ready-to-paste registry and roadmap rows | §"Registry updates" and §"Roadmap disposition" | **pass** |
| R13 — Per-platform results reported separately | §"Per-platform results" | **pass** |

## Production implementation evidence

None. This milestone is documentation-only by construction: no `crates/**` file, no manifest, no `Cargo.lock`, and no script was changed. §"Commands run" records the isolated gate run whose assertion 7 proves the no-production-delta property.

## Exact commands run and results

Run on macOS 25.6.0 (arm64), Darwin, against the working tree at baseline `bf439f5` plus the concurrent agents' in-flight edits.

| # | Command | Result |
|---|---|---|
| 1 | `git status --short` / `git log --oneline -3` / `git branch --show-current` | Branch `plans/corrective-batch-2026-10-23`, HEAD `bf439f5`. Confirmed the concurrent agents' in-flight edits to `crates/eggup-core/src/candidate.rs`, `crates/eggup-core/src/transaction.rs`, `crates/eggup-curl/src/lib.rs`, `crates/eggup-acquisition/**`, `CHANGELOG.md`, `architecture/{curl-adapter,tooling-governance,transport-footprint}.md`, `plans/registry.md` |
| 2 | `ls -d ~/.cargo/registry/src/index.crates.io-*/eggup-eggpack-*` | `eggup-eggpack-0.1.3` — the published version, confirmed offline from the local registry cache |
| 3 | `grep -rn "permission_hint" crates/ architecture/ plans/` | 0 hits in `architecture/`. The only remaining hits are in the service M013 closure and CHANGELOGs, where the removal is *recorded* — correct |
| 4 | `grep -c "#\[test\]" crates/<crate>/src/*.rs crates/<crate>/tests/*.rs` per crate | Produced the test counts in `architecture/overview.md` |
| 5 | `cat crates/<crate>/src/*.rs \| wc -l` per crate | Produced the LOC counts in `architecture/overview.md` |
| 6 | `python3 /tmp/eggup-cites/symbols.py <crate>/src/lib.rs` | Built the per-crate symbol index used for every re-derivation |
| 7 | `python3 /tmp/eggup-cites/resolve.py <md> <crate>` | Resolved every citation to a file + line and printed the cited content, for all five owned files and the three unowned files |
| 8 | `python3 /tmp/eggup-cites/sweep.py …` (blank / out-of-range / comment-only classifier) | Final sweep over the five owned files: **`TOTAL_CITATIONS=702 BLANK=0 OUT_OF_RANGE=0`**. The residual comment-only hits are sentences that explicitly name a doc comment or a code comment, e.g. `architecture/service-lifecycle.md:698` → `crates/eggup-service/src/lib.rs:1516-1521` ("the comment at `lib.rs:1516-1521` records why") and `architecture/archive-extraction.md:248` → `crates/eggup-eggpack/src/lib.rs:458` (`archive_format_for_name`) |
| 9 | `./scripts/check-docs-truth.sh` (live working tree) | **EXIT=1** — `FAIL 4: published-as-unpublished` (the offending bullet is an **added** line in the root `CHANGELOG.md`, written by a concurrent agent for Acquisition Transport M016) and `FAIL 7: production source changed: crates/eggup-core/src/candidate.rs`, `crates/eggup-core/src/transaction.rs`, `crates/eggup-curl/src/lib.rs`. **Not attributable to this milestone** — see §"Attribution of the live-tree gate failure" |
| 10 | `./scripts/check-docs-truth.sh --baseline bf439f5…` (live working tree) | **EXIT=1** — same two failures, same attribution |
| 11 | Isolated gate: pristine `HEAD` tree copied to `/tmp/eggup-cites/iso`, then only this milestone's five architecture files + plan + closure overlaid, then the **working-tree** `scripts/check-docs-truth.sh` copied in; `bash scripts/check-docs-truth.sh --baseline bf439f5…` | **EXIT=0 — `documentation-truth checks passed`.** Assertion 7 (no production-source delta) also passed, proving this milestone changed no production source |
| 12 | `cargo fmt --all -- --check` | **Fails** — but only at `crates/eggup-core/src/candidate.rs:574` and `:1083`, a file under concurrent edit by another agent. No file in this milestone's scope is formatted (this milestone changed no Rust) |

### Attribution of the live-tree gate failure

Commands 9 and 10 fail on the live tree. The failure is **not** caused by this milestone:

- `FAIL 7` names three production source files. All three are under concurrent edit by other agents (`git status` shows them modified; `git diff` shows the edits). This milestone's diff contains no `crates/**` path.
- `FAIL 4` reads the root `CHANGELOG.md`'s `## Unreleased` section. The offending bullet is `- The two transport adapters record their publication coupling with the seam (Acquisition Transport M016): …`, and `git diff CHANGELOG.md | grep -c "^+.*publication coupling"` returns `1` — it is an **added** line in the concurrent agent's edit, not present at `bf439f5`. The root `CHANGELOG.md` is outside this milestone's scope.
- Command 11 isolates this milestone's changes on top of a pristine `HEAD` tree and the gate passes with **EXIT=0**.

### Attribution of the `cargo fmt` failure

`cargo fmt --all -- --check` reports diffs only in `crates/eggup-core/src/candidate.rs`. This milestone changed no Rust file; the failure belongs to the concurrent core agent.

## Invariant review

| Invariant | Result |
|---|---|
| No production source change | **held** — isolated gate assertion 7 passed; the milestone diff contains no `crates/**` path |
| Documentation-truth gate passes | **held** — EXIT=0 on `HEAD` + this milestone's changes (command 11) |
| No false "fixes" | **held** — `architecture/eggfetch-adapter.md` had 216 anchors re-derived and **0 changed**; the change count per file is reported and every changed anchor was located by symbol, not by adjusting a number |
| No git state mutation | **held** — no mutating git command was run against the working repository |
| No edit outside the owned paths | **held** — exactly the five architecture files plus this plan and closure |
| Cross-crate anchors flagged | **held** — see §"Cross-crate watch list" |

## Failure / rollback / recovery review

Not applicable in the runtime sense: this milestone mutates no artifact, spawns no process, and takes no lock. Its only failure mode is a wrong correction, and that is checkable: R1's sweep (command 8) found 0 blank, 0 out-of-range, and 0 unexplained comment-only anchors across all 702 citations in the five owned files.

**Rollback** is a plain revert of the seven files listed above; no state is created outside the working tree.

## Compatibility and migration review

None. No public API, manifest, version, dependency pin, tag, or published artifact was touched.

The `0.1.2` → `0.1.3` corrections in the eggpack deep dive describe an **already-published** crate: `eggup-eggpack 0.1.3` is on crates.io, and its published manifest (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/eggup-eggpack-0.1.3/Cargo.toml`) pins the Eggup edges at `=0.1.3` and carries a direct `sha2 = "0.10.9"` runtime dependency, which corroborates the deep dive's existing statement that "the published `0.1.3` manifest carries a direct `sha2` runtime dependency and the adapter's own inline digest loop". Nothing was republished and nothing needs a migration.

## Security review

No security-relevant change. No code, no dependency, no permission, no credential handling, and no trust boundary was touched. The deep dives continue to assert, and this milestone does not weaken, the boundary statements that SHA-256 is integrity evidence only with no authenticity or signature claim.

One correction is worth recording as a *security-relevant accuracy* fix: the eggpack deep dive previously claimed the adapter "delegates to `eggup_core::hash_file` instead of running its own inline SHA-256 loop" for the **published** `0.1.3`. The published artifact still contains the inline loop (`Sha256::new()` at line 506 of the published `src/lib.rs`). The deep dive's *existing* text on that point is correct — it already states that the published `0.1.3` "carries a direct `sha2` runtime dependency and the adapter's own inline digest loop" — and this milestone left it correct. The workspace-source claim about delegation was also already correct and was left alone. No security claim was added, removed, or weakened.

## Documentation / operations evidence

Files changed: the five architecture files, plus this plan and this closure. `docs/`, `AGENTS.md`, `CHANGELOG.md`, `.opencode/skills/**`, `.github/**`, and `scripts/**` were **not** edited by this milestone.

The doc-ownership chain was followed: each deep dive owns the fact it states, `architecture/overview.md` remains an index that points at the deep dives rather than restating their contracts, and `docs/` (non-normative) was not touched.

## Per-platform results

This milestone is documentation-only. Its verification is a static anchor re-derivation plus the documentation-truth gate, both of which are platform-independent; no platform-specific code path is exercised.

| Platform | Result | Notes |
|---|---|---|
| **Linux** | **not run** | No Linux host available in this session. The gate is a static text/manifest check with no platform branch; no Linux-specific claim was changed. The `architecture/service-lifecycle.md` Windows-SCM section was corrected by reading `crates/eggup-service/src/windows_scm.rs` only — **no Windows behavior was executed or observed** |
| **macOS** | **passed** | macOS 25.6.0 (arm64), Darwin. Commands 1–12 above were run here. `./scripts/check-docs-truth.sh --baseline bf439f5…` on `HEAD` + this milestone's changes: **EXIT=0, `documentation-truth checks passed`**. Anchor sweep: **702 citations, 0 blank, 0 out-of-range** |
| **Windows** | **not run** | No Windows host available in this session. The Windows-specific anchors corrected (`windows_scm.rs:205/218/230/248/1012/279-288/424-468/469-504/505-531/532-566/1530/1829/1857`) were derived by reading the source, not by running it. They are **source-derived, not behaviour-verified** |

## Unresolved findings with severity

| # | Finding | Severity | Disposition |
|---|---|---|---|
| 1 | `crates/eggup-service/src/lib.rs:3129` — `validate_marker`'s `marker.contains(['\n', '\r'])` arm is unreachable, because `marker.chars().any(char::is_control)` at `:3126` already rejects both. Harmless; the deep dive already records it under "Known doc/code drift" | low | **Reported, not fixed.** This milestone is documentation-only. No code change is proposed; the existing deep-dive note is accurate |
| 2 | `architecture/overview.md`'s `eggup-core` and `eggup-curl` LOC figures are a **snapshot taken while both crates were being edited concurrently** (`crates/eggup-core/src/candidate.rs` measured 7 155 lines, then 7 181 lines, within this session) | low | **Flagged.** The table carries a visible snapshot note. Re-derive with the command in §"Cross-crate watch list" once the concurrent agents land |
| 3 | `crates/eggup-core/src/candidate.rs`, `crates/eggup-core/src/transaction.rs`, `crates/eggup-curl/src/lib.rs`, `crates/eggup-acquisition/**`, the root `CHANGELOG.md`, and the three unowned architecture files are mid-edit by other agents, so the live-tree docs-truth gate fails on `FAIL 4` and `FAIL 7` | low (process) | **Not this milestone's.** Proven by the isolated run (command 11). The parent should re-run `./scripts/check-docs-truth.sh` after the concurrent agents land |
| 4 | `architecture/core-transaction.md`, `architecture/acquisition.md`, and `architecture/curl-adapter.md` still carry stale citations and stale version claims | medium | **Reported in §"Appendix A"**, not fixed — those files and their source crates are owned by other agents |

No medium-or-higher finding remains open **within this milestone's scope**.

## Roadmap disposition

This workstream has no subsystem roadmap. `plans/subsystems/` holds seven subsystem roadmaps, none of which owns the architecture deep dives; the registry's workstream row for `Planning/closure hygiene corrective` is the disposition surface.

**Ready-to-paste roadmap row** (the registry's workstream-summary table, `plans/registry.md:63`):

```markdown
| Planning/closure hygiene corrective | C001-C013 all closed (docs-only reconciliation track); C014 documentation-truth and C015 architecture-anchor correctives closed concurrently | `plans/closure/planning-closure-hygiene-corrective/` |
```

The parent should apply this **after** C014 has landed, adjusting the C001-C0xx range to the highest closed milestone at that point.

## Registry updates

**Ready-to-paste registry row** for the "Recently closed foundation" table (`plans/registry.md:506`):

```markdown
| Planning/closure hygiene corrective | C015 architecture anchor and claim corrective | closed 2026-10-24; hosted-qualification pending | `plans/closure/planning-closure-hygiene-corrective/015-status.md` | Docs-only: re-derived 179 stale `path:line` anchors in the five stable architecture deep dives; removed-API, `=0.1.2` pin, and `eggup-core` `self-replace` claims corrected |
```

Measured length: **379 bytes** (limit 481). The parent owns `plans/registry.md` and applies this row after C014's row.

**Not applied by this milestone:** `plans/registry.md` and `plans/subsystems/**` are outside this milestone's owned paths.

## Cross-crate watch list

These anchors live in files this milestone owns but point into crates whose source is being edited concurrently. Each was re-derived against the tree as it stood; each must be re-verified once the concurrent agent lands.

| Anchor | Points into | Current value |
|---|---|---|
| `architecture/service-lifecycle.md` — `crates/eggup-core/src/candidate.rs:333`, `:410`, `:513` | `crates/eggup-core/src/candidate.rs` (under concurrent edit) | The `"spawning bounded candidate"` / `"polling bounded candidate"` / `"writing bounded stdin"` error sites |
| `architecture/service-lifecycle.md` — `crates/eggup-core/src/candidate.rs:169` | `crates/eggup-core/src/candidate.rs` | `CommandOutput::output_limited` |
| `architecture/eggpack-adapter.md` — `crates/eggup-core/src/domain.rs:696-700` | `crates/eggup-core/src/domain.rs` | The "recorded source path is advisory diagnostics only" rustdoc |
| `architecture/eggfetch-adapter.md` — `crates/eggup-acquisition/src/lib.rs:143-151`, `:404-432` | `crates/eggup-acquisition/src/lib.rs` | `FetchLimits::effective`, `redact_url` |
| `architecture/overview.md` — `eggup-core` LOC `7181`, `eggup-curl` LOC `2532` | both crates under concurrent edit | Snapshot values; see §"Unresolved findings" #2 |

Re-derive command:

```sh
for c in eggup-core eggup-acquisition eggup-eggfetch eggup-curl eggup-service eggup-archive eggup-eggpack eggup-transport-footprint; do
  sl=$(cat crates/$c/src/*.rs crates/$c/src/bin/*.rs 2>/dev/null | wc -l | tr -d ' ')
  st=$(grep -h "#\[test\]" crates/$c/src/*.rs crates/$c/src/bin/*.rs 2>/dev/null | wc -l | tr -d ' ')
  tt=$(grep -h "#\[test\]" crates/$c/tests/*.rs 2>/dev/null | wc -l | tr -d ' ')
  printf "%-26s loc=%-6s src_tests=%-4s total_tests=%s\n" "$c" "$sl" "$st" "$((st+tt))"
done
```

## Appendix A — ready-to-paste corrections for files this milestone does not own

**These were found but NOT applied.** `architecture/acquisition.md`, `architecture/curl-adapter.md`, and `architecture/core-transaction.md` are owned by other agents and their source crates are being edited concurrently. Each entry gives the stale citation **verbatim** and the true current line, re-derived by symbol. Re-verify the `crates/eggup-core/**` and `crates/eggup-acquisition/**` entries after those agents land.

### A.1 `architecture/acquisition.md`

| # | Doc line | Stale citation (verbatim) | True current location |
|---|---|---|---|
| A1-1 | `:49` | `` | `eggup-eggpack` | `=0.1.2` exact pin + path | `` | **Substantive.** The pin is `=0.1.3` — `crates/eggup-eggpack/Cargo.toml:20-22` |
| A1-2 | `:735` | `lib.rs:437-445` (for `truncate_utf8_bytes`) | `crates/eggup-acquisition/src/lib.rs:384-392` (fn at `:384`) |
| A1-3 | `:736` | `lib.rs:429-431` (for `__adapter_bound_detail`) | `crates/eggup-acquisition/src/lib.rs:376-378` (fn at `:376`) |

A1-2 and A1-3 were already confirmed by the parent; this milestone re-derived them independently and agrees. A1-1 is a **new** finding.

### A.2 `architecture/curl-adapter.md`

| # | Doc line | Stale citation (verbatim) | True current location |
|---|---|---|---|
| A2-1 | `:137` | `lib.rs:337`, `lib.rs:408` (for "`config.validate()` runs at construction *and* again per fetch") | Per-fetch `self.config.validate()` is at `crates/eggup-curl/src/lib.rs:346` (metadata fetch) and `:417` (artifact fetch). The construction-time validates are at `:195` and `:252` |
| A2-2 | `:633` | `lib.rs:1046` (for `diagnostic_bound_never_splits_multibyte_code_points`) | `crates/eggup-curl/src/lib.rs:1055` |

### A.3 `architecture/core-transaction.md`

| # | Doc line | Stale citation (verbatim) | True current location |
|---|---|---|---|
| A3-1 | `:3` | `Status: current as of `eggup-core` 0.1.2` (`M009` closure, published to crates.io).` | **Substantive.** `eggup-core` is published at `0.1.3` — the local registry cache holds `eggup-core-0.1.3` |
| A3-2 | `:119` | `candidate.rs:278` (for `run_bounded`) | `crates/eggup-core/src/candidate.rs:304` |
| A3-3 | `:120` | `candidate.rs:525` (for `CandidateValidator`) | `crates/eggup-core/src/candidate.rs:643` |
| A3-4 | `:121` | `candidate.rs:540` (for `ExactIdentityValidator`) | `crates/eggup-core/src/candidate.rs:658` |
| A3-5 | `:122` | `candidate.rs:641` (for `CrossMemberAgreementValidator`) | `crates/eggup-core/src/candidate.rs:759` |
| A3-6 | `:123` | `candidate.rs:675` (for `AllValidators<'a>`) | `crates/eggup-core/src/candidate.rs:793` |
| A3-7 | `:124` | `candidate.rs:711` (for `ValidatedTransaction`) | `crates/eggup-core/src/candidate.rs:829` |
| A3-8 | `:240` | `candidate.rs:785` (for `commit`) | `crates/eggup-core/src/candidate.rs:903` |
| A3-9 | `:241` | `candidate.rs:813` (for `commit_with_post_commit`) | `crates/eggup-core/src/candidate.rs:931` |
| A3-10 | `:305` | `candidate.rs:278` (for `run_bounded`) | `crates/eggup-core/src/candidate.rs:304` |
| A3-11 | `:351` | `candidate.rs:540` (for `ExactIdentityValidator`) | `crates/eggup-core/src/candidate.rs:658` |
| A3-12 | `:351` | `candidate.rs:641` (for `CrossMemberAgreementValidator`) | `crates/eggup-core/src/candidate.rs:759` |
| A3-13 | `:353` | `candidate.rs:625-633` (for "Timeout and output-overflow are converted into `Error::CandidateExecution`") | `crates/eggup-core/src/candidate.rs:743-751` (the `!output.success()` check at `:743`, the `Error::CandidateExecution` construction at `:747`) |
| A3-14 | `:354` | `candidate.rs:681` (for `AllValidators::new()`) / `candidate.rs:695` (for `Default`) | `crates/eggup-core/src/candidate.rs:799` / `crates/eggup-core/src/candidate.rs:812` |
| A3-15 | `:558` | `candidate.rs:785` (for `commit`) | `crates/eggup-core/src/candidate.rs:903` |
| A3-16 | `:651` | `candidate.rs:945` (for `bounded_display`) | `crates/eggup-core/src/candidate.rs:1063` |
| A3-17 | `:652` | `candidate.rs:552-563` (for the post-commit panic catch) | `crates/eggup-core/src/transaction.rs:553` — the `catch_unwind` call. The enclosing `post_commit` block starts at `:552`. **The parent's figure of `:551` is two lines early**; `candidate.rs` has never contained a `catch_unwind` |
| A3-18 | `:731` | `candidate.rs:625-632` (for the validator's timeout/overflow → `CandidateExecution`) | `crates/eggup-core/src/candidate.rs:743-751` |

### A.4 Corrections deliberately NOT made

- `architecture/curl-adapter.md:7` and `:911` state that `eggup-curl` is published at `0.1.2`. **Correct** — the local registry cache holds only `eggup-curl-0.1.2`, and `plans/registry.md` records `eggup-curl` as the one crate still on `0.1.2`. Do not "correct" these.
- `architecture/acquisition.md:223`, `:600`, `:701` mention `0.1.2`. All three are **correct** in context (pre-`0.1.2` migration history and the published `eggup-curl 0.1.2` user-agent contract).
- `architecture/curl-adapter.md:6` `Cargo.toml:23-24` and `:990` `README.md:20` — **verified correct** in the curl crate's own scope. Do not "fix" them.
- `architecture/acquisition.md:97` (`lib.rs:300`, `304`, `329`), `:109` (`lib.rs:214-216`), `:139` (`lib.rs:434-439`), `:271`–`:274`, `:302`–`:304`, `:313` (`lib.rs:347-350`), `:356`–`:382`, `:422`–`:431` — all **verified correct** in the acquisition crate's own scope.

## Appendix B — substantive falsehoods corrected, quoted before and after

### B.1 `architecture/service-lifecycle.md`

**B1-1 — a removed public API was documented as if it existed** (doc lines 56 and 1217).

Before:

> | Privilege acquisition | never. Access denial returns a bounded hint, never an elevation flow (`lib.rs:1541`) |

> | Access denied by a manager | Bounded remediation hint, no elevation | `permission_hint`, `lib.rs:1541`; `scm_error_detail`, `windows_scm.rs:215` |

After:

> | Privilege acquisition | never. Access denial is a typed `ManagerFailureKind::Permission` with a bounded detail, never an elevation flow (`lib.rs:339`, `lib.rs:363`) |

> | Access denied by a manager | Typed `ManagerFailureKind::Permission` with a bounded detail; no elevation | `ManagerFailureKind::from_spawn_error`, `lib.rs:363`; `ManagerFailureKind::from_scm_os_code`, `lib.rs:378`; `scm_error_detail`, `windows_scm.rs:279-288` |

Why: service M013 replaced `permission_hint` with the typed `ManagerFailureKind` taxonomy. `crates/eggup-service/src/lib.rs` had 13 occurrences of `permission_hint` at baseline `c480541` and has **0** now. A normative deep dive must not name an API that does not exist.

**B1-2 — the service crate's source-layout figures were stale** (doc lines 19, 22, 23).

Before: `src/lib.rs` "~4.5k lines", `src/lifecycle_update.rs` "~1.1k lines", `src/windows_scm.rs` "~1.6k lines".
After: "~5.5k lines" (5 536), "~1.3k lines" (1 279), "~1.9k lines" (1 886).

**B1-3 — the test inventory understated the crate** (doc lines 1225 and the per-area table).

Before: "111 `#[test]` functions across the five source files".
After: "112 `#[test]` functions across the five source files" — the per-area breakdown was also re-derived (the old table's `lib.rs` sub-ranges no longer exist), and the integration-target counts were corrected: `errors_doc_sections.rs` 4 → **5**, `bounded_executor_consolidation.rs` 12 → **14**, and the previously unlisted `ownership_type_identity.rs` (0 tests, compile-time only) was added.

**B1-4 — `architecture/overview.md`'s `lifecycle_update.rs` module description was wrong** — the module list said `lifecycle_update.rs` did "definition writing + `commit_with_lifecycle`". Definition writing is `atomic_write_definition` in `lib.rs`; `lifecycle_update.rs` holds the lifecycle orchestration, policy, receipt, and error types. Fixed in `architecture/overview.md`'s service source-layout list.

### B.2 `architecture/eggpack-adapter.md`

**B2-1 — the published version was stated as `0.1.2`** (doc line 13).

Before: `| Status | Published to crates.io at `0.1.2` |`
After: `| Status | Published to crates.io at `0.1.3` |`

**B2-2 — the dependency pins were stated as `=0.1.2`** (doc line 258).

Before: "asserts all three Eggup edges stay `=0.1.2`"
After: "asserts all three Eggup edges stay `=0.1.3` — derived from the adapter's own `env!("CARGO_PKG_VERSION")` rather than a hardcoded literal, so the invariant is "pinned to *our* version" and survives the next workspace bump"

**B2-3 — a drift row asserted a repair that had already happened** (doc line ~888).

Before:

> - `docs/crates.md:117` still carries `=0.1.2` for the Eggup pins, which are `=0.1.3`. Pre-existing drift, not introduced here; `docs/` is non-normative.

After:

> - `docs/crates.md` previously carried `=0.1.2` for the Eggup pins, which are `=0.1.3`. That drift has been **repaired**: `docs/crates.md:119-120` now states `=0.1.3` and names `crates/eggup-eggpack/Cargo.toml` as the source of truth. `docs/crates.md:117` is the separate `cargo tree --workspace` review rule and never carried a version pin. `docs/` is non-normative, so this document does not restate its wording.

### B.3 `architecture/archive-extraction.md`

**B3-1 — the crate header's size and test count were stale** (doc line 4).

Before: "`crates/eggup-archive/src/lib.rs`, 3512 lines, 51 `#[test]` functions"
After: "`crates/eggup-archive/src/lib.rs`, 3717 lines, 52 `#[test]` functions"

### B.4 `architecture/overview.md`

**B4-1 — `eggup-core`'s dependency set was stated three times as "nothing but `sha2`"** (doc lines 37, 69, 204, and the dependency diagram).

Before: "and it depends on nothing but `sha2`." / "**`eggup-core`** → `sha2`. Policy-neutral local mechanics." / "Its only dependency is `sha2`." / "(sha2 only)"
After: "and it depends on nothing but `sha2` plus a **Windows-only** `self-replace` edge (deferred deletion of a replaced running image; the Unix and macOS graphs stay `sha2` only)." / "**`eggup-core`** → `sha2` (plus `self-replace` on Windows only). Policy-neutral local mechanics." / "Its only dependencies are `sha2` and the Windows-only `self-replace` edge." / "(sha2; self-replace on Windows only)"

Why: `crates/eggup-core/Cargo.toml` declares `[target.'cfg(windows)'.dependencies] self-replace = "1.3.0"`. The workspace `AGENTS.md` already states this precisely.

**B4-2 — the module map's LOC/test figures were stale for every crate** (doc lines 99–106).

Before: `eggup-core` 5650 / 54, `eggup-acquisition` 2182 / 38, `eggup-eggfetch` 1231 / 25, `eggup-curl` 1672 / 24, `eggup-service` 10101 / 104, `eggup-archive` 3512 / 51, `eggup-eggpack` 849 / 63, `eggup-transport-footprint` 45 / 0.
After: `eggup-core` 7181 / 110, `eggup-acquisition` 2229 / 50, `eggup-eggfetch` 1229 / 31, `eggup-curl` 2532 / 38, `eggup-service` 11700 / 174, `eggup-archive` 3717 / 59, `eggup-eggpack` 855 / 71, `eggup-transport-footprint` 45 / 2.

The legend was also made explicit — "`LOC` is source lines under the crate's `src/` directory; `tests` is the count of `#[test]` functions across the crate's `src/` **and** `tests/` directories" — because the old column mixed the two conventions (`eggup-acquisition` 38 and `eggup-eggfetch` 25 were src-only counts, while `eggup-eggpack` 63 was a total).

**B4-3 — three source-layout lists were incomplete** (doc lines 111–125).

Before: `eggup-core` "six runnable `examples/` plus two integration test files (`current_executable.rs`, `stale_lock_recovery.rs`)"; `eggup-service` "`lib.rs` (neutral model + Unix mechanics), `disposition.rs` (M006 runtime-authority barrier), `lifecycle_update.rs` (definition writing + `commit_with_lifecycle`), `windows_scm.rs` (native SCM)"; `eggup-eggpack` "single `lib.rs` plus three integration test files (`interoperability.rs`, `archive_handoff.rs`, `caller_destinations.rs`)".

After: `eggup-core` adds `failure_taxonomy_tests.rs` and names **four** integration test files (`candidate_deadline.rs`, `candidate_stdin.rs`, `current_executable.rs`, `stale_lock_recovery.rs`); `eggup-service` adds `commit_engine.rs` — "the single commit-and-rollback engine" — and corrects `lifecycle_update.rs`'s role; `eggup-eggpack` names **four** integration test files, adding `digest_reuse.rs`.

## Appendix C — anchors re-derived per file

| File | Citation occurrences (before → after) | Distinct anchors in the corrected file | Distinct anchors changed | Result |
|---|---|---|---|---|
| `architecture/service-lifecycle.md` | 270 → 278 | 225 | **168** | 168 stale anchors corrected |
| `architecture/eggpack-adapter.md` | 151 → 152 | 105 | **7** | 7 stale anchors corrected, plus 3 substantive falsehoods |
| `architecture/archive-extraction.md` | 56 → 56 | 50 | **4** | 4 stale anchors corrected, plus 1 stale figure |
| `architecture/eggfetch-adapter.md` | 216 → 216 | 158 | **0** | All 216 re-derived and confirmed correct — **no change made** |
| `architecture/overview.md` | 0 → 0 | 0 | **0** | No `path:line` anchors; 8 LOC/test figures, 4 dependency claims, 3 source-layout lists corrected |
| **Total** | **693 → 702** | | **179 anchors + 11 substantive claims/figures** | |

The final sweep (command 8) re-checked all 702 occurrences after the edits: **0 blank, 0 out of range**.

## Appendix D — anchors in files this milestone owns that point into crates under concurrent edit

See §"Cross-crate watch list" above. Each was re-derived and is recorded so the parent can re-verify it after the concurrent agents land.
