# Planning / Closure Hygiene Corrective C013 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md`

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Roadmap: `plans/registry.md` — this workstream has no subsystem roadmap (see the plan's "Source roadmap" section; `plans/subsystems/` holds exactly seven roadmaps and none owns documentation reconciliation). The registry is the roadmap equivalent.

Implementation commits: none — this milestone's changes are uncommitted in the working tree at the time of writing; the change set is listed under *Requirement-to-evidence matrix* row R-14 and verified by a path-scoped `git diff`.

## Executive finding

C013 closes open finding 11 (medium) and reconciles seven documentation-truth findings (A–G) without changing production source, dependencies, or publication state.

- **A — the root `CHANGELOG.md` contradicted itself.** The `## Unreleased` section asserted `Published baselines are 0.1.2` while the `0.1.3` section above it recorded six crates published at `0.1.3`, and thirteen bullets re-listed work the `0.1.3` section records as published. The header now names the current published baseline (`0.1.3` for `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, `eggup-eggpack`, `eggup-service`; `0.1.2` for `eggup-curl`; `publish = false` for `eggup-transport-footprint`), and `Unreleased` holds only genuinely unpublished work.
- **B — stale version claims.** `docs/crates.md`, `docs/quickstart.md`, `docs/releases.md`, `README.md`, `AGENTS.md`, `architecture/overview.md`, `architecture/tooling-governance.md`, and two skill files carried `0.1.2`/`=0.1.2` claims that the manifests and the registry contradict. All now name the current state; the one legitimate `eggup-curl` `0.1.2` claim and the one legitimate history reference survive.
- **C — contradictory live status in the registry.** Acquisition M010 was `closed` in the authoritative table and `ready` in the second table; the execution graph said `[READY]`; Gregg M004 was `blocked` in one table and `authorable` in another. Each milestone now resolves to exactly one status class.
- **D — the "Planned / blocked work" table was 95 % closure history.** It now carries three rows — Gregg M004, EggPool M007, and the authenticity/signatures ADR gate — and the 16 duplicated `(subsystem, milestone)` keys and the double Eggpack M004 row are gone.
- **E — the registry was not scannable.** The 4,000-character cell, the 3,267-character cell, the 2,560-character cell and five others are gone. The registry's longest line is now **481 bytes**, the ceiling recorded below.
- **F — `architecture/transport-footprint.md` overstated what the build CI proves.** The claim that `required-features` "makes an accidental cross-link a build error" is replaced with the accurate statement, and the CI-coverage table gains the row that says plainly that **no CI job builds the isolated single-adapter configurations**. The measured counts 3 / 92 / 93 are unchanged.
- **G — `eggup-core`'s contract docs are excluded from the published crate.** **Deferred**, per §6.6 of the plan: it is a manifest change with no consumer effect until a republication, so it is recorded as an open drift row owned by Verified Update Core M016, and no `crates/*/Cargo.toml` was touched.

## Requirement-to-evidence matrix

One row per acceptance criterion in the plan's §13. "Command" is the exact command whose observed output is the evidence.

| §13 | Requirement | Command | Observed | Result |
|---|---|---|---|---|
| 1 | Root `Unreleased` holds no entry the `0.1.3` section records as published, and its header states the published baseline | `python3` changelog check (script assertion 4) | `changelog Unreleased conflicts: 0`, exit 0; `CHANGELOG.md:224-234` header names `0.1.3` for six crates and `0.1.2` for `eggup-curl` | passed |
| 2 | `docs/crates.md` states `0.1.3` for the workspace and for every crate published at `0.1.3`; `:59` unchanged | `grep -n 'Status:' docs/crates.md` | `0.1.3` at `:25`, `:39`, `:50`, `:70`, `:83`, `:97`; `0.1.2` still at `:59` (`eggup-curl`) | passed |
| 3 | `docs/quickstart.md:18` and `README.md:18` name `eggup-core = "0.1.3"` | `sed -n '18p' docs/quickstart.md README.md` | `eggup-core = "0.1.3"` in both | passed |
| 4 | `docs/releases.md`'s limitations section describes the versions actually published; the historical `0.1.2` references are unchanged | `sed -n '30p;37p;59p;70p;81p' docs/releases.md` | `0.1.3` / `2026-10-09` / M011 / `=0.1.3` / the three rewritten limitation bullets; `docs/releases.md:16-23` and `:31` untouched | passed |
| 5 | `README.md`'s publication-state paragraph has no duplicated crate list and states `eggup-service 0.1.3` correctly | `sed -n '128,131p' README.md` | one crate list; "`eggup-service` was published later, on 2026-10-09" | passed |
| 6 | No `=0.1.2` pin claim survives outside the allowlisted historical rows | `grep -rn '=0\.1\.2' AGENTS.md architecture/overview.md architecture/tooling-governance.md docs/` | one hit: `docs/releases.md:47` (the allowlisted `eggup-eggpack 0.1.2` history reference) | passed |
| 7 | One status per milestone across the whole registry | plan §11 duplicate-row python snippet | `conflicting milestone states: 0`, exit 0 | passed |
| 8 | The second milestone table holds only open/authorable/future rows; the duplicates are gone; the authoritative table is named | `sed -n '/^## Open, authorable, and future work/,/^## Immediate/p' plans/registry.md` | three rows (Gregg M004, EggPool M007, Authenticity/signatures); `plans/registry.md:383` names the authoritative table | passed |
| 9 | The registry's longest line is at or below the recorded ceiling; no 4,000-character cell remains | `awk '{ print length($0)": "NR }' plans/registry.md \| sort -rn \| head -8` | `481: 634` … — maximum 481, ceiling 481 | passed |
| 10 | `architecture/transport-footprint.md` no longer claims the `required-features` split makes a cross-link a build error | `sed -n '177p;210p' architecture/transport-footprint.md` | the accurate guarantee at `:177`; the "No CI job builds the isolated single-adapter configurations" row at `:210` | passed |
| 11 | The transport-footprint counts still read 3 / 92 / 93 | the three §11 `cargo tree … --offline --prefix none` commands | `3`, `92`, `93` | passed |
| 12 | `crates/eggup-core/Cargo.toml` unchanged; the package still omits `crates/eggup-core/docs/`; the drift table names M016 | `cargo package -p eggup-core --list --allow-dirty --offline`; `git diff --name-only -- crates/` | 24 entries, none under `docs/`; no `crates/` path in this milestone's diff; the new Open drift row names Verified Update Core M016 | passed |
| 13 | No drift-table row contradicts the corrected surfaces; `:752` corrected; `:770` closed and removed | `sed -n '/^### Resolved/,/^### Open/p' architecture/tooling-governance.md` | the `Unreleased` aggregation row corrected in place; the registry-duplication Open row is gone; two new Open rows added | passed |
| 14 | `scripts/check-docs-truth.sh` exists, is wired into the gate, fails on the baseline and passes on the corrected tree; §4–§5 describes it | `./scripts/check-docs-truth.sh` in the baseline worktree and in this tree | baseline: 39 failures, exit 1; corrected tree: `documentation-truth checks passed`, exit 0 | passed |
| 15 | No `src/` byte changed, no `Cargo.lock`, no dependency added, nothing published, no tag moved | path-scoped `git diff --name-only` + `git diff --check` | 14 tracked paths, none under `crates/`; no `Cargo.toml`/`Cargo.lock`; whitespace clean | passed |
| 16 | `docs/` contains no normative rule introduced by this milestone | script assertion 3 | the three targeted rule constructs in `docs/quickstart.md` are accompanied by the contract links at `docs/quickstart.md:207-210`; no failure | passed |
| 17 | No closure record deleted; superseded predecessors marked historical/superseded rather than removed | `git diff --name-only -- plans/closure/` | empty — no closure record touched | passed |

## Per-file before/after evidence

Before-text is from the baseline commit `c480541` (`git show HEAD:<file>`); after-text is from the corrected tree. Line references are `file:line` on each side.

| File | Before (baseline) | After (corrected) |
|---|---|---|
| `CHANGELOG.md:224-227` | "The workspace-wide bug-audit fixes below are recorded here as well as in the per-crate changelogs, because they are **not** on the registry. Published baselines are `0.1.2` for all seven published crates. No publication milestone authorizes these entries yet." | `CHANGELOG.md:224-234` — "The entries below are **unpublished** work. The current published baselines are `0.1.3` for `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, `eggup-eggpack` and `eggup-service`; `0.1.2` for `eggup-curl`; `eggup-transport-footprint` is `publish = false`." |
| `CHANGELOG.md:233-366` | 13 bullets re-listing published work (core M010/M011, the Eggpack security fix, the core/archive/eggfetch/acquisition audit fixes, Eggpack M002, Acquisition M008/M007/M005, Service M007/M006) | removed; `CHANGELOG.md:235`, `:257`, `:276`, `:296`, `:324` hold the five genuinely unpublished entries, each still marked `**Unreleased**` / `**Unpublished**` / `**Not published by this milestone**` |
| `docs/crates.md:3-4` | "All share the workspace version `0.1.2`; only their registry state differs." | `docs/crates.md:3` — "The workspace version is `0.1.3`; only their registry state differs." |
| `docs/crates.md:25` | "- **Status:** published at `0.1.2`." | `docs/crates.md:25` — "- **Status:** published at `0.1.3`." (the same edit at `:39`, `:50`, `:70`, `:96`→`:97`) |
| `docs/crates.md:83-84` | "- **Status:** published at `0.1.2` — published later than the rest of the `0.1.2` set, from a different source commit than the shared `v0.1.2` tag." | `docs/crates.md:83-86` — "- **Status:** published at `0.1.3` (2026-10-09) — published later than the rest of the `0.1.3` set, from a different source commit than the shared `v0.1.3` tag, which was likewise not moved." |
| `docs/crates.md:117` | "**`eggup-eggpack` pins its dependencies exactly** (`=0.1.2`). Every other crate uses a caret requirement. So a fix published to a caret consumer is inherited automatically, but the same fix cannot reach `eggup-eggpack 0.1.2` consumers without a separate adapter republication." | `docs/crates.md:118-122` — "(`=0.1.3`, per `crates/eggup-eggpack/Cargo.toml`) … `eggup-eggpack 0.1.3` — which is published — carries its own exact pins and needs a separate adapter republication to inherit a later seam fix." |
| `docs/quickstart.md:18` | `eggup-core = "0.1.2"` | `docs/quickstart.md:18` — `eggup-core = "0.1.3"` |
| `docs/releases.md:30` | "`eggup-service` was published last (2026-10-05, Service Lifecycle M009)." | `docs/releases.md:30` — "`eggup-service` was published last (2026-10-09, Service Lifecycle M011, at `0.1.3`)." (the `v0.1.2` tag sentence at `:31` unchanged) |
| `docs/releases.md:37` | "(`=0.1.2`). Every other crate uses a caret requirement (`^0.1.0`)." | `docs/releases.md:37` — "(`=0.1.3`). Every other crate uses a caret requirement (`^0.1.0`)." |
| `docs/releases.md:43-45` | "- The same fix **cannot** reach you through `eggup-eggpack 0.1.2`. That crate resolves its own dependencies at exactly `0.1.2`, so it needs a separate republication before it inherits a seam fix." | `docs/releases.md:43-48` — "- The same fix reaches you through `eggup-eggpack 0.1.3`, which resolves its own dependencies at exactly `0.1.3` and therefore carries the `0.1.3` seam fixes. A seam fix published after `0.1.3` still needs a separate adapter republication …" |
| `docs/releases.md:57-65` | "- **Published `eggup-eggpack 0.1.2` contains a security fix that is not yet published.** … the fix is implemented and green but not released" | `docs/releases.md:59-68` — "- **`eggup-eggpack 0.1.2` carries the positional-binding defect.** … The fix shipped in `eggup-eggpack 0.1.3`; see the `0.1.3` section of `crates/eggup-eggpack/CHANGELOG.md`." |
| `docs/releases.md:67-76` | "- **A composed fetch in `eggup-acquisition 0.1.2` can overrun the caller's deadline.** … the fix … is implemented and green but not published; it is the payload of the open Acquisition M010 milestone" | `docs/releases.md:70-79` — "… The fix … shipped as `eggup-acquisition 0.1.3` on 2026-10-06 (Acquisition Transport M010), and `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.2` inherit it automatically on the next resolve." |
| `docs/releases.md:80-83` | "- **Published `eggup-core 0.1.2` predates the `Error::Injected` addition.** That variant is only in the unpublished working tree …" | `docs/releases.md:81-86` — "- **`Error::Injected` exists in no packaged build.** The variant is compiled only for the crate's own tests, so it is absent from every published `eggup-core` version, `0.1.2` and `0.1.3` alike." |
| `README.md:18` | `eggup-core = "0.1.2"` | `README.md:18` — `eggup-core = "0.1.3"` |
| `README.md:128-132` | "`eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, / `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, and / `eggup-eggpack` are published at `0.1.3`; `eggup-curl` and `eggup-service` are published at `0.1.2`. Service M010's `0.1.3` correction is still in M011 release qualification and has not been published." | `README.md:128-135` — one crate list; "`eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack`, and `eggup-service` are published at `0.1.3`; `eggup-curl` is published at `0.1.2`. `eggup-service` was published later, on 2026-10-09, from a different source commit than the shared `v0.1.3` tag…" |
| `README.md:139` | "for the limitations you inherit at `0.1.2`." | `README.md:136-137` — "for the limitations each published version carries." |
| `AGENTS.md:22` | "published (`=0.1.2`), depends on `core` + `archive` + `acquisition` (each pinned `=0.1.2`, local paths) + `eggpack-manifest = "=0.1.0"` from the registry + `sha2 = "0.10.9"`." | `AGENTS.md:22` — "published (`=0.1.3`), depends on `core` + `archive` + `acquisition` (each pinned `=0.1.3`, local paths) + `eggpack-manifest = "=0.1.0"` from the registry; its exact dependency list is owned by `crates/eggup-eggpack/Cargo.toml`." (the `sha2` clause is dropped because the manifest no longer declares it — Eggpack M006) |
| `AGENTS.md:99` | "Note that `eggup-eggpack` pins `=0.1.2`, so a seam fix needs a separate adapter republication to reach it." | `AGENTS.md:99` — "Note that `eggup-eggpack` pins its Eggup dependencies exactly (see `crates/eggup-eggpack/Cargo.toml`), so a seam fix needs a separate adapter republication to reach it." |
| `architecture/overview.md:75` | "7. **`eggup-eggpack`** → `eggup-core` + `eggup-archive` + `eggup-acquisition` (`=0.1.2`) + `eggpack-manifest =0.1.0` + `sha2`. Leaf producer adapter." | `architecture/overview.md:75` — "7. **`eggup-eggpack`** → `eggup-core` + `eggup-archive` + `eggup-acquisition` (`=0.1.3`) + `eggpack-manifest =0.1.0`. Leaf producer adapter." |
| `architecture/tooling-governance.md:58` | `version = "0.1.2"` | `architecture/tooling-governance.md:58` — `version = "0.1.3"` |
| `architecture/tooling-governance.md:118-120` | "requirements (`=0.1.2`) … even though the workspace version is now `0.1.2`" | `architecture/tooling-governance.md:118-120` — "requirements (`=0.1.3`) … even though the workspace version is now `0.1.3`" |
| `architecture/tooling-governance.md:659` | "(`eggup-eggpack` → `=0.1.2`) are accounted for …" | `architecture/tooling-governance.md:686` — "(`eggup-eggpack` → `=0.1.3`) are accounted for …" |
| `architecture/tooling-governance.md:697` | "on `eggpack-manifest` and to pin `=0.1.2` exactly." | `architecture/tooling-governance.md:724` — "on `eggpack-manifest` and to pin `=0.1.3` exactly." |
| `architecture/transport-footprint.md:54-56` | "\| `eggup-acquisition` \| `0.1.2` (path) \| The seam, and the only unconditional dependency \|" (+ the `eggup-curl` and `eggup-eggfetch` rows) | `architecture/transport-footprint.md:54-56` — "\| `eggup-acquisition` \| `\"0.1.0\"` (path) \| `0.1.3` \| The seam, and the only unconditional dependency \|" (+ the two adapter rows), with the manifest cited at `:58-60` |
| `architecture/transport-footprint.md:169` | "\| Adapters do not know each other \| `curl_only` links no Eggfetch code, and vice versa; the `required-features` split makes an accidental cross-link a build error, not a review finding \|" | `architecture/transport-footprint.md:177` — "`required-features` stops a binary from *building* when its adapter's feature is off. It does **not** stop a cross-link when both features are on … 'links no Eggfetch code' is a meaningful claim only in the single-adapter configurations, and those are the ones CI never builds" |
| `architecture/transport-footprint.md:195-201` | CI-coverage table with five rows, none about the isolated configurations | `architecture/transport-footprint.md:203-211` — the table gains "\| Any lane \| `--no-default-features --features curl` or `--features eggfetch` \| **No CI job builds the isolated single-adapter configurations** \|", the asymmetry note is retained, and the missing subsystem roadmap is named |
| `architecture/transport-footprint.md:93-94` | "Measured on this workspace at commit `ae9b47a` (macOS, `debug` profile, exact commands in [Verification](#verification)):" | `architecture/transport-footprint.md:93-97` — the same sentence plus "The workspace version advanced to `0.1.3` after that measurement; the counts below are the dated capture and are not re-derived." The 3 / 92 / 93 rows at `:106-108` are **byte-identical** |
| `architecture/tooling-governance.md:204-225` (§4) | "the whole gate — 9 lines … 5 of them `cargo` commands"; the gate block had five commands; the per-line table ended at line 8 | `architecture/tooling-governance.md:204-227` — "the whole gate — 10 lines … 5 of them `cargo` commands"; the block gains `./scripts/check-docs-truth.sh`; the per-line table gains row 9; a new §4.1 documents the check |
| `architecture/tooling-governance.md:344-353` (§5.2) | six coverage-gap rows | `architecture/tooling-governance.md:346-355` — a seventh row: "The documentation-truth check runs in the local gate only" |
| `architecture/tooling-governance.md:752` (Resolved) | "Added aggregation entries for eggpack, core …; the eggpack entry states that `0.1.2` consumers do not have the fix" | corrected in place — "…Those entries were then removed again when the 0.1.3 train published the same work, and the root `Unreleased` header no longer claims an unpublished baseline: it now names the current published baseline (`0.1.3` for six crates, `0.1.2` for `eggup-curl`) and holds only genuinely unpublished work" |
| `architecture/tooling-governance.md:770` (Open) | the registry-duplication row, with the stale counts "40 of its 44 rows" | **removed**; its resolution is this closure record. Two new Open rows replace it: the transport-footprint CI-coverage gap and the `eggup-core` contract-doc packaging gap (M016) |
| `architecture/tooling-governance.md:767`, `:769` (Open) | "while `[workspace.package] version` is `0.1.2`" / "while the workspace is at `0.1.2`" | "while `[workspace.package] version` is `0.1.3`" / "while the workspace is at `0.1.3`" — the rows **remain open**; the manifest literals are unchanged |
| `plans/registry.md:238` | "\| Acquisition transport \| M010 eggup-acquisition 0.1.3 publication \| **ready** \| …" | **removed** from the second table; the authoritative table's row (`plans/registry.md:398`) reads "closed 2026-10-06" |
| `plans/registry.md:266` | "\| Consumer adoption \| M004 Gregg \| blocked; plan intentionally unwritten \| …" | `plans/registry.md:462` — "\| Consumer adoption \| M004 Gregg \| authorable; plan intentionally unwritten \| Every versioned handoff it was gated on is published …" |
| `plans/registry.md:270` | "\| Consumer adoption \| M007 EggPool selective \| blocked/evidence-driven; plan intentionally unwritten \| …" | `plans/registry.md:463` — "\| Consumer adoption \| M007 EggPool selective \| authorable; plan intentionally unwritten \| Core M012/M013 are closed and `eggup-core 0.1.3` is published …" |
| `plans/registry.md:367` | "`-> M010 eggup-acquisition 0.1.3 publication [READY; ships the 0b8cb98 seam fixes]" | `plans/registry.md:558` — "[CLOSED 2026-10-06; shipped the 0b8cb98 seam fixes]" |
| `plans/registry.md:369` | "`-> eggup-eggpack adapter republication [NAMED FOLLOW-ON; =0.1.2 pin blocks inheritance]" | `plans/registry.md:560` — "[RETIRED; eggup-eggpack 0.1.3 published 2026-10-06 and inherits the seam fixes]" |
| `plans/registry.md:392` | "- Consumer adoption: Gregg M004 remains intentionally unwritten until Acquisition M010 and Core M012/M013 close. …" | `plans/registry.md:612-614` — "Gregg M004 and EggPool M007 are both **authorable** now — every versioned handoff they were gated on is published … so both plans are intentionally unwritten pending an authoring decision, not blocked." |
| `plans/registry.md:393` | "- Archive extraction: … M004 is registered to publish the already-implemented 0.1.3 correctness fixes once workspace 0.1.3 exists." | `plans/registry.md:614` — "M004 published the already-implemented `0.1.3` correctness fixes as `eggup-archive 0.1.3` on 2026-10-06, so that gate is discharged." |
| `plans/registry.md:233-273` (table) | "## Planned / blocked work", 37 rows, 35 of them closed or historical, `Eggpack M004` twice | `plans/registry.md:455-464` — "## Open, authorable, and future work", three rows; the 16 duplicated `(subsystem, milestone)` keys and the double Eggpack M004 row are gone |
| `plans/registry.md` (no such line at baseline) | — | `plans/registry.md:383` — "**`Dependency-ready implementation work` is the authoritative milestone-status table.**" |
| `plans/registry.md:15`, `:49`, `:328`, `:432`, `:584`, `:594` | 2,560 / 1,144 / 1,051 / 4,000 / 3,267 / 1,888-byte cells of closed-history prose | compressed to 464 / 476 / 410 / 480 / 474 / 480 bytes, each ending in a pointer at the closure record that owns the detail. No fact was deleted; the detail was relocated to the closure records that already hold it |
| `scripts/check-local.sh` | five `cargo` commands | `scripts/check-local.sh:9` — `./scripts/check-docs-truth.sh` (one added line; the five commands are byte-identical) |
| `scripts/check-docs-truth.sh` | did not exist | new file, `chmod +x` |
| `.opencode/skills/release-workflow/SKILL.md:22,32,37,46,50-51` | "`eggup-acquisition` 0.1.3, the current open milestone" / "`=0.1.2`" / "`eggup-eggpack 0.1.2` consumers" / "(`0.1.2` → `0.1.3`)" / "published through `0.1.2`; the M010 correction is the unpublished `0.1.3` candidate" | "published 2026-10-06" / "`=0.1.3`" / "`eggup-eggpack 0.1.3` consumers" / "(`0.1.3` → `0.1.4`)" / "published through `0.1.3` (M011, published 2026-10-09)" |
| `.opencode/skills/docs-hygiene/SKILL.md:64-65` | "`eggup-service 0.1.2` is published; the corrected `0.1.3` service candidate is not yet published." | "`eggup-service 0.1.3` is published (2026-10-09); `eggup-curl` is published only at `0.1.2`." |
| `.opencode/skills/verify-workflow/SKILL.md:14-26` | the local-gate block listed five `cargo` commands only | the block gains `./scripts/check-docs-truth.sh  # documentation agrees with itself, the manifests, and the registry`, plus a paragraph naming what it asserts and that no CI lane runs it |

## Production implementation evidence

None, by design. This milestone changes documentation, the planning registry, and the local gate only.

Zero-production-delta proof:

```sh
git diff --name-only c4805411f74b85158e0ae5d0a219a32c4209d98e..HEAD
```

→ **empty**. No commit has landed since the baseline; this session committed nothing, so the whole change set is uncommitted in the working tree.

```sh
git diff --name-only -- CHANGELOG.md README.md AGENTS.md docs/ \
  architecture/overview.md architecture/tooling-governance.md \
  architecture/transport-footprint.md plans/registry.md scripts/check-local.sh \
  .opencode/skills/
```

→ 14 tracked paths: `AGENTS.md`, `CHANGELOG.md`, `README.md`, the three `docs/` files, three `architecture/` deep dives, `plans/registry.md`, `scripts/check-local.sh`, and three skill files. **No `crates/` path, no `Cargo.toml`, no `Cargo.lock`.** `scripts/check-docs-truth.sh` is new and untracked.

```sh
git diff --check -- <the same paths>
```

→ clean; no whitespace errors.

The whole-tree `git diff --name-only` is **not** used as evidence: five other milestones are in flight in the same working tree and their `crates/**/src/**` edits are not this milestone's. The path-scoped check above is the honest equivalent; the unscoped tree-wide check is owned by the parent session.

- No file under `crates/` was edited.
- No dependency was added, removed, or re-versioned.
- No `Cargo.lock` change, no feature change, no public API change.
- Nothing was published, re-published, or tagged.

The only executable artifact is `scripts/check-docs-truth.sh`, a read-only, network-free, root-free, cargo-free checker wired into the local gate as its sixth command.

## Exact commands run and results

### Baseline-failure proof (assertion: the script must fail on the pre-change tree)

Mechanism: `git worktree add` at a unique `/tmp` path, with the new script copied into the baseline tree (the baseline predates the script, so the script is the only file that had to be introduced). The main working tree's git state was not mutated; the worktree was removed afterwards with `git worktree remove --force`.

```sh
BASE=/tmp/eggup-c013-base-$$
git worktree add "$BASE" HEAD        # HEAD is c480541, the plan's baseline
cp scripts/check-docs-truth.sh "$BASE/scripts/check-docs-truth.sh"
cd "$BASE" && ./scripts/check-docs-truth.sh
```

Observed output (excerpt; the full run printed 39 failures):

```
note 7: skipped — pass --baseline <ref> to scope the no-production-delta check
FAIL 2: docs/crates.md: a 'workspace version' paragraph names 0.1.2
FAIL 2: docs/crates.md:25: 'published at `0.1.2`' is not the single legitimate eggup-curl occurrence: - **Status:** published at `0.1.2`.
... (7 such hits)
FAIL 2: docs/releases.md: '=0.1.2' survives outside the allowlisted history reference
FAIL 2: AGENTS.md: '=0.1.2' survives outside the allowlisted history reference
FAIL 2: architecture/overview.md: '=0.1.2' survives outside the allowlisted history reference
FAIL 2: architecture/tooling-governance.md: '=0.1.2' survives outside the allowlisted history reference
FAIL 4: root Unreleased header still asserts: Published baselines are `0.1.2`
FAIL 4: root Unreleased header still asserts: No publication milestone authorizes these entries yet
FAIL 4: published-with-no-state-marker: eggup-core | - `eggup-core` gains current-executable transaction parity (Verified U
FAIL 4: published-as-unpublished: eggup-core | - Core audit fixes (unpublished): bound-source staging dropped the exe
... (13 bullet conflicts in total)
FAIL 5: milestone ('Acquisition transport', 'M010 eggup-acquisition 0.1.3 publication') resolves to several status classes: [(185, 'closed'), (238, 'ready')]
FAIL 5: registry:238: cross-surface contradiction 'M010 eggup-acquisition 0\.1\.3 publication[^|]*\|[^|]*ready'
FAIL 5: registry:367: cross-surface contradiction 'M010 eggup-acquisition 0\.1\.3 publication \[READY'
FAIL 5: registry:266: cross-surface contradiction 'M004 Gregg.*blocked'
FAIL 5: registry:270: cross-surface contradiction 'M007 EggPool selective.*blocked'
FAIL 5: registry:392: cross-surface contradiction 'remains intentionally unwritten until Acquisition M010'
FAIL 5: registry:393: cross-surface contradiction 'once workspace 0\.1\.3 exists'
FAIL 5: registry does not name the 'Dependency-ready implementation work' table as authoritative
documentation-truth checks failed: 39
exit=1
```

The six registry hits land on exactly the lines the plan's §11 names (`:238`, `:367`, `:266`, `:270`, `:392`, `:393`) and the duplicate-row conflict lands on `:185`/`:238` — the plan's stated baseline signature.

### Corrected tree

```sh
./scripts/check-docs-truth.sh
```

```
note 7: skipped — pass --baseline <ref> to scope the no-production-delta check
documentation-truth checks passed
exit=0
```

### §11 version-truth assertions

```sh
grep -rnE '0\.1\.[0-9]+' docs/ README.md AGENTS.md | grep -vE '0\.1\.(0|1|2|3)\b'
```
→ empty (exit 1). No version literal outside the known set.

```sh
grep -rn 'workspace version' docs/ README.md AGENTS.md
```
→ one hit: `docs/crates.md:3` — "The workspace version is `0.1.3`;" — every hit names `0.1.3`.

```sh
grep -rn '=0\.1\.2' AGENTS.md architecture/overview.md architecture/tooling-governance.md docs/
```
→ one hit: `docs/releases.md:47` — "which exact-pinned `=0.1.2`, never received them." — the allowlisted history reference, in the paragraph that names `eggup-eggpack 0.1.2`.

```sh
grep -n 'published at `0\.1\.2`' docs/crates.md
```
→ exactly one hit, `docs/crates.md:59` (`eggup-curl`), which is correct and survives.

### §11 registry assertions

Duplicate-row check (the plan's exact snippet) → `conflicting milestone states: 0`, exit 0.

The six targeted cross-surface greps → each empty:

| Pattern | Result |
|---|---|
| `M010 eggup-acquisition 0\.1\.3 publication[^|]*\|[^|]*ready` | empty |
| `M010 eggup-acquisition 0\.1\.3 publication \[READY` | empty |
| `M004 Gregg.*blocked` | empty |
| `M007 EggPool selective.*blocked` | empty |
| `remains intentionally unwritten until Acquisition M010` | empty |
| `once workspace 0\.1\.3 exists` | empty |

`grep -nE 'authoritative' plans/registry.md` → `plans/registry.md:383` names the `Dependency-ready implementation work` table as authoritative.

### §11 registry readability

```sh
awk '{ print length($0)": "NR }' plans/registry.md | sort -rn | head -8
```

```
481: 634
480: 621
480: 609
480: 453
480: 405
478: 73
478: 624
478: 615
```

**Recorded ceiling: 481 bytes per line.** It is set at the maximum line length of `architecture/overview.md` (481), so the registry is no less scannable than its closest peer; the stricter peer `plans/002-long-term-roadmap.md` (315) is not reachable for a table row that carries a milestone name, a status, and a record path. Measured on the pre-change registry the longest line was **4,000 bytes** at the `Dependency-ready` table's trailing narrative.

### §11 transport-footprint counts (unchanged)

```sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l     # 3
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l  # 92
cargo tree -p eggup-transport-footprint --all-features --locked --offline --prefix none | sort -u | grep -v '(\*)$' | wc -l                                # 93
```

`--offline` was added because this session has no network; the resolved graphs are identical to the plan's commands.

### §11 package contents (Finding G evidence)

```sh
cargo package -p eggup-core --list --allow-dirty --offline
```

→ 24 entries: `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, six `examples/*.rs`, and `src/*.rs`. **No `docs/` entry** — `docs/domain.md`, `docs/verification.md`, and `docs/transaction.md` are absent, exactly as at the baseline. The manifest is unchanged.

### §11 no production delta (path-scoped to this milestone's change set)

```sh
git diff --name-only -- CHANGELOG.md README.md AGENTS.md docs/ architecture/overview.md \
  architecture/tooling-governance.md architecture/transport-footprint.md plans/registry.md \
  scripts/check-local.sh .opencode/skills/
```

→ 14 tracked paths, all documentation, script, registry, or skill files. **No `crates/` path, no `Cargo.toml`, no `Cargo.lock`.** `git diff --check` over the same set → clean.

The whole-tree `git diff --name-only` is **not** used as evidence here: five other milestones are in flight in the same working tree and their `crates/**/src/**` edits are not this milestone's. The path-scoped check above is the honest equivalent, and the unscoped tree-wide check is owned by the parent session.

## Invariant review

Each invariant of the plan's §4, checked item by item, with its evidence.

| §4 invariant | Evidence | Result |
|---|---|---|
| `docs/` stays **non-normative**: it points at contracts, it does not restate them; no new normative rule under `docs/` | script assertion 3 passes; every `docs/` edit points at its owner (`crates/*/Cargo.toml`, `docs/releases.md`, `crates/eggup-core/docs/`, `architecture/`); no sentence states a rule whose owner is `architecture/` or `crates/eggup-core/docs/` | held |
| No rule outside `architecture/` and `crates/eggup-core/docs/`; `README.md` and `AGENTS.md` stay thin indexes | `README.md` and `AGENTS.md` were edited only in version/pin literals; `AGENTS.md:99` now points at `crates/eggup-eggpack/Cargo.toml` instead of restating the pin | held |
| `plans/000`–`003` and the accepted ADRs are not touched | path-scoped `git diff --name-only` shows no path under `plans/000`-`003` or `plans/adrs/` | held |
| The transport-footprint figures **3 / 92 / 93** are unchanged; the dated capture is not re-derived | `architecture/transport-footprint.md:106-108` byte-identical to the baseline; the three §11 `cargo tree` commands reproduce 3 / 92 / 93 | held |
| The registry remains the **active control surface** and keeps the "active/ready work" discipline | the second milestone table is narrowed to three open rows; the authoritative table is named at `plans/registry.md:383`; the longest line is 481 bytes | held |
| The root `CHANGELOG.md` and the per-crate changelogs stay consistent: the root aggregates, it does not replace | the root `Unreleased` now holds only genuinely unpublished work, each naming the published version that still carries the defect; **no per-crate changelog was edited** | held |
| Published-version facts are sourced from `plans/registry.md` and `docs/releases.md`, never invented | every corrected literal is derivable from `crates/*/Cargo.toml`, `plans/registry.md`, or the recorded crates.io verification in `docs/releases.md:3-8`; no new version was introduced | held |
| The per-crate changelog `Unreleased` sections were reviewed and are correct; the defect is in the **root** changelog only; correct per-crate files must not be churned | `crates/eggup-{core,archive,service,acquisition,curl,eggfetch,eggpack}/CHANGELOG.md` were read; none was edited | held |
| No closure record is deleted; superseded predecessors keep their records and are marked historical/superseded | `git diff --name-only -- plans/closure/` is empty; the registry's historical rows remain, compressed to pointers at their records | held |
| No `src/` byte changes, no dependency added, no version published, no tag moved | path-scoped diff: 14 tracked paths, none under `crates/`; no `Cargo.toml`/`Cargo.lock`; `cargo package --list` unchanged | held |

## Failure / rollback / recovery review

Not applicable in the runtime sense: there is no runtime behavior, no state machine, and no lock. The failure semantics of the milestone are the failure semantics of a text check.

- The plan's §8 precedence rule was applied in this order: `crates/*/Cargo.toml` for dependency pins and versions → `plans/registry.md` and `docs/releases.md` for publication state → crates.io (via the recorded `docs/releases.md:3-8` verification) for what is actually on the registry → `docs/` for the consumer-facing rendering.
- The plan's retained-closure-record rule was applied: no closure record was edited or deleted, and every compressed registry cell points at the record that owns the detail.
- `scripts/check-docs-truth.sh` is read-only, network-free, root-free, and never invokes `cargo`. It writes nothing; a failure is a non-zero exit with a per-assertion report and no partial state.
- It is wired into `scripts/check-local.sh` as the sixth command, after the five existing commands, which are unmodified. `set -euo pipefail` means the first failing command aborts the gate.
- Every assertion **fails closed**: a violation is reported and the script exits non-zero. No assertion silently passes on an unreadable file — an unreadable surface raises, which also exits non-zero.
- Rollback is `git revert` of the documentation commit, or `git checkout` of the affected file; nothing outside the working tree is touched. No tag, release, or registry state is involved.
- Assertion 6 (the transport-footprint counts) is deliberately **not** in the script, because that would make it invoke `cargo`; it stays a milestone verification command.
- Assertion 7 (no production source changed) is milestone-scoped by nature, so it runs only when a baseline ref is supplied (`--baseline <ref>`); the gate invocation runs assertions 1–5.

## Compatibility and migration review

None required. No consumer action is forced by a documentation correction.

- A consumer who pinned `0.1.2` remains validly pinned. The corrected text only stops recommending a superseded baseline.
- `docs/crates.md` and `docs/quickstart.md` now name `0.1.3`, so a consumer who copies the snippet resolves the current published baseline.
- `docs/releases.md`'s limitations section now describes the versions actually published: the Eggpack member-identity fix **did** reach `eggup-eggpack 0.1.3`; the composed-fetch budget fix **did** ship in `eggup-acquisition 0.1.3`; `Error::Injected` exists in no packaged build at all.
- `README.md`'s publication-state paragraph is de-duplicated and states `eggup-service 0.1.3` correctly.

## Security review

No security-relevant behavior changed. One security *statement* was corrected: `docs/releases.md` previously told readers that a published security fix in `eggup-eggpack` was unshipped. It is published in `eggup-eggpack 0.1.3`; the residual limitation is that `eggup-eggpack 0.1.2` consumers still carry the defect, which the corrected text now says.

The root `CHANGELOG.md` no longer tells a consumer that five published crates are unpublished.

## Documentation / operations evidence

| Surface | Owner of the fact | What changed |
|---|---|---|
| `CHANGELOG.md` (root) | the root aggregates; per-crate changelogs own per-crate release notes | `Unreleased` header now names the published baseline; 13 published re-listings removed; the 5 genuinely unpublished entries kept |
| `docs/crates.md` | `crates/*/Cargo.toml` and `docs/releases.md` | version claims corrected to `0.1.3`; `eggup-curl`'s `0.1.2` kept; the exact pin now `=0.1.3` and pointed at its owner |
| `docs/quickstart.md` | `crates/eggup-core/Cargo.toml` | dependency snippet `0.1.2` → `0.1.3` |
| `docs/releases.md` | crates.io, verified per its own §3-8 | limitations rewritten to the versions actually published |
| `README.md` | `crates/*/Cargo.toml` and `plans/registry.md` | snippet corrected; publication-state paragraph de-duplicated and corrected |
| `AGENTS.md` | `crates/eggup-eggpack/Cargo.toml` owns the pin | `=0.1.2` → `=0.1.3`; the pin is referenced to its owner rather than restated twice |
| `architecture/overview.md` | `crates/*/Cargo.toml` | eggpack dependency line corrected to `=0.1.3` |
| `architecture/tooling-governance.md` | the deep dive owns the gate description and the drift table | §4 gate table updated for the added line; §4.1 added; §5 coverage-gap row added; drift rows `:752` corrected in place, `:770` closed and removed, `:767`/`:769` premise corrected, two new Open rows added |
| `architecture/transport-footprint.md` | `crates/eggup-transport-footprint/Cargo.toml` + the three bin sources | version table corrected to the manifest's literals; the build-guarantee claim corrected; the CI-coverage table gains the isolated single-adapter row; the measured 3 / 92 / 93 block untouched and annotated as the dated `ae9b47a` capture |
| `plans/registry.md` | the registry owns open work, blockers, and next transitions | one status per milestone; authoritative table named; second table narrowed to 3 open rows; long cells compressed to pointers |
| `scripts/check-docs-truth.sh` (new), `scripts/check-local.sh` | the gate owns verification commands | the sixth gate command; five existing commands unmodified |
| `.opencode/skills/{release-workflow,docs-hygiene,verify-workflow}/SKILL.md` | the skills summarize normative sources | falsified version literals corrected; the local-gate listing updated for the added command |
| `crates/eggup-*/CHANGELOG.md` | per-crate release notes | **not edited** — reviewed and correct |
| `plans/000`–`003`, `plans/adrs/` | normative | **not edited** |

## Unresolved findings with severity

| ID | Severity | Finding | Owner |
|---|---|---|---|
| G | medium | `crates/eggup-core/Cargo.toml` excludes `docs/`, so the authoritative contract docs never reach crates.io consumers | **Verified Update Core M016 — `eggup-core` contract-doc packaging** (`plans/subsystems/verified-update-core-roadmap.md`); recorded as an open drift row |
| F-CI | medium | No CI job builds the isolated single-adapter footprint configurations; the crate has no `plans/subsystems/` roadmap, so no roadmap row owns the gap | Follow-on of planning C013; its prerequisite is authoring `plans/subsystems/transport-footprint-roadmap.md`; recorded as an open drift row |
| PIN | medium | `crates/eggup-transport-footprint/Cargo.toml:16,24,29` and `crates/eggup-service/Cargo.toml:24` still require their peers at caret `0.1.0` while the workspace is at `0.1.3` | Already tracked at `architecture/tooling-governance.md:767`/`:769`; those rows were corrected to the `0.1.3` premise and **remain open**, because the manifest literals are unchanged and the remedy is a pin-style milestone |
| C013-S1 | low | The registry's `Dependency-ready` table carries a milestone's *closure-record* path only for the six rows whose plan+closure paths would have exceeded the 481-byte ceiling; the closure record names its plan, so no pointer is lost | none — accepted trade-off of the ceiling |
| C013-S2 | low | The root `CHANGELOG.md` `Unreleased` aggregates the core/archive/service unpublished work but not the acquisition/curl/eggfetch/eggpack unpublished work that the per-crate changelogs hold | none — the root aggregates, it does not have to be exhaustive; the per-crate changelogs remain the owners |

**No medium-or-higher contradiction remains in the touched active surfaces.** The two medium rows above are recorded as open drift rows with named owners, which is the plan's prescribed outcome for Findings F's CI half and G.

## Per-platform results

This milestone is **documentation-only**. It changes no code, no manifest, no lockfile, no dependency, and no feature, so no platform lane applies to the change itself. No platform result is inferred from another.

| Lane | Result | Note |
|---|---|---|
| Linux (`stable`, `msrv`, `service-systemd`) | **not run** | no hosted runner is reachable from this machine |
| macOS (`macos`) | **not run** | the implementation machine is macOS, but the milestone adds no runtime behavior and the plan's §11 commands for this change are text checks, which were run here |
| Windows (`windows-check`) | **not run** | as above |
| Local whole-tree gate `./scripts/check-local.sh` | **not run** | owned by the parent session; the tree is mid-flight with five other milestones' concurrent source changes, so the whole-tree gate would fail for reasons unrelated to this milestone. This milestone's own gate command was run: `./scripts/check-docs-truth.sh` → exit 0 |
| `cargo tree` transport-footprint counts | **passed** | 3 / 92 / 93, platform-independent resolution counts, run on macOS |
| `cargo package -p eggup-core --list` | **passed** | no `docs/` entry, as at the baseline |

Assertions 1–5 and 7 of the new script are platform-independent text checks and are recorded once, not per platform. A lane not run is recorded as **not run**, never omitted and never inferred.

## Roadmap disposition

- No subsystem roadmap advances: this workstream has none by design, and the registry is the roadmap equivalent.
- `plans/registry.md` records C013 as **closed 2026-10-23**, open finding 11 as closed with its closure pointer, and the "Recently closed foundation" track as `C001-C013 all closed`.
- Two follow-ons are named and owned outside this milestone: **Verified Update Core M016** (Finding G) and the transport-footprint CI-coverage milestone (Finding F's CI half).
- No open milestone's status was changed except C013's own.

## Registry updates

- `plans/registry.md` — C013 row status `ready for handoff 2026-10-23` → `closed 2026-10-23`; open finding 11 marked closed with the closure pointer; the "Post-0.1.3 open findings" intro notes which findings have closed; the "Recently closed foundation" track reads `C001-C013 all closed`; the `Dependency-ready implementation work` section names itself authoritative; the second milestone table renamed to "Open, authorable, and future work" and narrowed to three open rows; the six cross-surface contradictions reconciled; the `Current project state` and `Next handoff` prose restated in the present tense for Acquisition M010, Gregg M004, EggPool M007, and the Archive M004 clause; the 2026-10-23 additions preserved, with long cells compressed to their closure-record pointers.
- `architecture/tooling-governance.md` §"Known doc/code drift" — the Resolved row on the root `Unreleased` aggregation corrected in place; the Open row on the registry's duplicated tables closed and removed (its resolution is this record); the two pin-style Open rows corrected to the `0.1.3` workspace version and left open; two new Open rows added for the transport-footprint CI-coverage gap and the `eggup-core` contract-doc packaging gap.

## Disposition

Closed. C013 reconciles Eggup's documentation with `plans/registry.md` and with what is actually published on crates.io, and leaves the repository with a sixth local gate command that fails when that agreement breaks.

The change publishes nothing, changes no production source, and adds no dependency. The two medium findings that this milestone could not close by a documentation edit — Finding G and the transport-footprint CI-coverage gap — are recorded as open drift rows with named owners, exactly as the plan's §6.6 and §6.7 require.
