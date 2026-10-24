# Planning/Closure Hygiene Corrective 014 — documentation-truth corrective

Status: **closed** (documentation-only; no platform lane applies to the change itself — the scoped local gate is green against the committed baseline, and the working tree is red for exactly one reason this milestone does not own, recorded as U5; the workspace-wide cargo gate is reported below with the concurrent-edit failure that stopped it)

Source plan: `plans/implementation/planning-closure-hygiene-corrective/014-documentation-truth-corrective.md`

Source roadmap: **none.** `planning-closure-hygiene-corrective` has no subsystem
roadmap by design (`.opencode/skills/planning-workflow/SKILL.md:88-90`). The
planning-workflow skill defines this workstream as "the docs-only workstream for
reconciling the registry, roadmaps, and closure records against reality; it has no
subsystem roadmap". `plans/registry.md` plus the normative process documents are
the roadmap equivalent. No subsystem roadmap advances in this milestone.

Reviewed repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`
(`docs(closure): record the hosted qualification of the second milestone batch`)

Implementation commits/PRs: **none — the parent session owns every commit.** This
milestone produced working-tree changes only and no git state was mutated: no
`git stash`, `git checkout`, `git restore`, `git commit`, `git push`, `git tag`,
`git clean`, or `git reset` was run against the workspace repository. The
throwaway git repository created for the red-first proof lives entirely in
`/tmp/eggup-redproof` and is not this repository.

Long-term requirements: `plans/000-long-term-specification.md`,
`plans/001-terminology-and-domain-model.md`. Applicable ADR:
`plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`.

## Executive finding

The documentation-truth gate was a gate in name only, and the deep dives that
describe this workspace described a workspace that no longer exists. Both are now
correct, and the correction is proven red-first rather than asserted.

Five defects were in scope. **D1:** `scripts/check-docs-truth.sh` was described in
three places as comparing the docs "with the manifests", but it read **no**
`Cargo.toml` at all, covered **none** of `architecture/*.md`, skipped every
registry table row with fewer than four cells, and matched `crates/**/src/**` only —
so a new `crates/*/build.rs` was invisible to it. **D2:**
`architecture/transport-footprint.md` contradicted the branch's own work in five
places: it said CI builds none of the isolated single-adapter configurations (CI
builds them in three lanes), that the gap "is still open" (M001 closed it), that
the crate has no subsystem roadmap (the roadmap exists), that it has "3 binaries,
0 tests" (it has a 2-test guard), and it printed its dependency tree at `v0.1.2`
(the crates are `v0.1.3`). **D3:** the §5 CI matrix in
`architecture/tooling-governance.md` still named `cargo package -p eggup-service
--locked` for the `stable` and `macos` lanes when the workflow runs
`--no-verify`, and neither row mentioned the transport-footprint steps the branch
added. **D4:** two `ci.yml` anchors in that file were stale. **D5:** nine of twelve
`ci.yml` anchors in `plans/closure/transport-footprint/001-status.md` resolved to
unrelated content after commit `b9885fa` shifted the workflow's lines.

The gate now reads the manifests, the per-crate changelogs, and
`architecture/*.md`; it fails on four of the five demonstrated false negatives;
and every pre-existing assertion still fires on its own mutation.

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | The gate reads the crate manifests | `scripts/check-docs-truth.sh` — `parse_manifest()` reads the root `Cargo.toml` `[workspace.package] version` and each `crates/*/Cargo.toml` (name, version, `publish`, dependency edges); `CRATES` is built from it | **pass** |
| 2 | Published-version claims are checked | Assertion 8b: a "published at `V`" claim is checked against the crate's own `CHANGELOG.md` released headings and the manifest's `publish` flag. Red-first: mutation (a) → `FAIL 8: docs/crates.md:59: eggup-curl is claimed as published at 0.1.3, which its CHANGELOG.md does not record as released (released: ['0.1.2'])` | **pass** |
| 3 | Dependency claims are checked | Assertion 8c: the `docs/crates.md` `**Dependencies:**` bullets, the `architecture/tooling-governance.md` §2.4 `Dependencies` column, and the `architecture/overview.md` module map are each checked as a **subset** of the manifest's declared edges | **pass** |
| 4 | Workspace-version claims are checked | Assertion 8a: a "workspace version … `V`" claim must name the manifest's `[workspace.package] version` | **pass** |
| 5 | Coverage extends to `architecture/*.md` | Assertion 1 now runs over `ALL_SURFACES` (= `docs/` + `README.md` + `AGENTS.md` + `architecture/*.md`); assertion 3 runs over `docs/` **and** `architecture/*.md`, with the owning deep dive exempt; assertion 9 runs over the same set plus the root `CHANGELOG.md`. Red-first: mutation (d) → `FAIL 1: architecture/overview.md:41: version literal 0.1.9 outside the known set` | **pass** |
| 6 | Three-cell registry rows are no longer skipped | Assertion 5 now reads every table row with ≥ 3 cells; the header is detected only when the next line is the separator **and** the previous line is not a table row, so a data row can never be mistaken for a header and dropped | **pass** |
| 7 | `crates/*/build.rs` is covered with `crates/**/src/**` | Assertion 7 matches `path.endswith("/build.rs")` as well as `"/src/" in path`, and it now also reads `git ls-files --others --exclude-standard` — a file that was just created is untracked until it is added, and `git diff` does not list untracked files. Red-first: mutation (e) → `FAIL 7: production source changed: crates/eggup-acquisition/build.rs` | **pass** |
| 8 | A false normative rule inside the owning deep dive is caught | Assertion 9 (invariant polarity): a tracked invariant has one true polarity. Red-first: mutation (c) → `FAIL 9: architecture/core-transaction.md:554: states the opposite of a tracked invariant: a member declared NotRequired always fails; there is no escape hatch` | **pass** |
| 9 | Every pre-existing assertion still fires | Regression proof A1, A2, A3, A4, A5, A7 — the pre-fix and post-fix scripts produce the **same** `FAIL` line for each mutation | **pass** |
| 10 | The gate still passes on the unmodified tree | The extended gate → exit 0 against a clean export of the committed baseline (`git archive HEAD`) and in the `/tmp` copy. On the **working tree** it is red for one reason only — a concurrent root-`CHANGELOG.md` edit tripping the pre-existing assertion 4 (U5), reproduced identically by the pre-fix script | **pass** |
| 11 | D2 — `architecture/transport-footprint.md` no longer contradicts the branch | Header now reads "45 bin source lines total, 3 binaries … `tests/cross_link_guard.rs` (a 2-test source-level guard)"; the two tree snippets read `v0.1.3`; the invariant row says CI builds the single-adapter configurations "as their own steps"; §"Testing approach" states the two tests and their names; the CI table names the three `stable` builds (`ci.yml:26-28`), the three `msrv` checks (`ci.yml:54-56`), and the two `macos` builds (`ci.yml:101-102`); "The gap this crate exists to close is **closed**"; the roadmap `plans/subsystems/transport-footprint-roadmap.md` is linked | **pass** |
| 12 | D2 — the dependency counts are unchanged | 3 / 92 / 93 reproduce exactly (§"Exact commands run"); only the version labels were corrected | **pass** |
| 13 | D3 — the §5 CI matrix names the commands the workflow runs | `stable` names the three footprint builds and `cargo package … --locked --no-verify`; `msrv` names the three footprint checks; `macos` names the two footprint builds and `cargo package … --locked --no-verify`; each states "The package step's registry-resolved compile is deferred; see the packaging note below", the same wording the other rows use | **pass** |
| 14 | D4 — the stale `ci.yml` anchors are corrected | `ci.yml:34-80` → `ci.yml:70-87`; the Resolved-drift row now reads "**ten** test steps at `ci.yml:122-142`" | **pass** |
| 15 | Every `ci.yml:` line number written is verified against the current file | Each was read from `.github/workflows/ci.yml` at the time of writing; the set is listed in §"Anchors verified against the current workflow" below | **pass** |
| 16 | D5 — every `ci.yml:` citation in the transport-footprint closure record is re-anchored | `plans/closure/transport-footprint/001-status.md` — twelve citations re-anchored, listed in that record's new "C014 re-anchoring addendum" | **pass** |
| 17 | The closure record's own Windows step count is corrected | The parent-session addendum described hosted run `40062299630` (commit `b9885fa`) as "nine targeted `cargo test` steps". `git show b9885fa:.github/workflows/ci.yml` shows **ten** `cargo test` steps in that lane — the live SCM fixture at `ci.yml:134` was part of that commit. Corrected to **ten** in both places | **pass** |
| 18 | The plan uses the full 16-section template | `plans/implementation/planning-closure-hygiene-corrective/014-documentation-truth-corrective.md` — §1–§16, SHA baseline, non-goals, failure semantics, verification commands | **pass** |
| 19 | The gate stays dependency-free, POSIX-portable, and fast | `bash` + `python3` only; no `tomllib` (a deliberately minimal manifest reader keeps one behaviour on every interpreter); no network; a full run completes in well under a second | **pass** |
| 20 | The registry is updated | **Not done here — the parent owns `plans/registry.md`.** The registry's "Planning/closure hygiene corrective" row must advance from `C001-C013 all closed` to `C001-C014 all closed` | **parent action** |

## Production implementation evidence

There is none, by design. No file under `crates/**`, no manifest, no
`Cargo.lock`, no workflow file, and no git state was changed by this milestone.
The only code changed is `scripts/check-docs-truth.sh`, and it changes no
production behaviour of any crate.

The gate's own extension is the production surface of this milestone:

| Change | Where |
|---|---|
| Minimal manifest reader (`parse_manifest`), crate table (`CRATES`), per-crate released-version reader (`published_versions`) | `scripts/check-docs-truth.sh` — new block after `walk()` |
| `ALL_SURFACES` = `docs/` + `README.md` + `AGENTS.md` + `architecture/*.md` | `scripts/check-docs-truth.sh` |
| Assertion 1 extended to `architecture/*.md`; the known-version set is the registry history **unioned with the manifest's declared version**; a literal the same clause disclaims is not a claim | `scripts/check-docs-truth.sh` |
| Assertion 3 extended to `architecture/*.md`, two constructs added (`never inferred`, `integrity evidence only`), the owning deep dive exempt | `scripts/check-docs-truth.sh` |
| Assertion 5 reads ≥ 3-cell rows; header detection cannot drop a data row | `scripts/check-docs-truth.sh` |
| Assertion 7 matches `crates/*/build.rs` and reads untracked files | `scripts/check-docs-truth.sh` |
| Assertion 8 (new): workspace-version claims, published-version claims, dependency claims | `scripts/check-docs-truth.sh` |
| Assertion 9 (new): invariant polarity | `scripts/check-docs-truth.sh` |

## Exact commands run and results

### Scoped verification (this milestone)

```sh
./scripts/check-docs-truth.sh
# -> "documentation-truth checks passed", exit 0
```

**Two distinct states must be reported, because the working tree moved under
this milestone while it ran.**

**(a) Against the committed baseline — green.** The extended gate was run
against a clean export of the baseline commit, with this milestone's script in
place:

```sh
rm -rf /tmp/head-copy && mkdir /tmp/head-copy
git archive HEAD | tar -x -C /tmp/head-copy
cp scripts/check-docs-truth.sh /tmp/head-copy/scripts/check-docs-truth-post.sh
cd /tmp/head-copy && ./scripts/check-docs-truth-post.sh
# -> "documentation-truth checks passed", exit 0
```

This is the load-bearing result: **the extension introduces no false positive on
the committed tree.** It was green on every run during implementation, including
after each of this milestone's own edits to `architecture/*.md`.

**(b) Against the working tree at the end of this milestone — red, from a
concurrent edit this milestone does not own.** Another writer added a 41-line
bullet to the root `CHANGELOG.md`'s `Unreleased` section (the Acquisition
Transport M016 publication-coupling entry) while this milestone was running. That
bullet makes the **pre-existing** assertion 4 fail:

```sh
./scripts/check-docs-truth.sh
# -> FAIL 4: published-as-unpublished: eggup-acquisition, eggup-eggfetch, eggup-eggpack |
#      - The two transport adapters record their publication coupling with th
# -> documentation-truth checks failed: 1     (exit 1)
```

The **pre-fix** script (`git show HEAD:scripts/check-docs-truth.sh`) fails with
the **identical** line on the same tree, so this milestone's extension did not
cause it. The bullet itself is substantively correct — it says the *seam's next
release* is unpublished ("capabilities … that exist only in the unpublished
seam", `eggup-acquisition 0.2.0`), and says nothing false about the published
`0.1.3`. Assertion 4 cannot tell the two apart: it matches a crate **name**
anywhere in a bullet and then rejects any occurrence of "unpublished" in that
bullet, without requiring the word to be about that crate's published version.
See U5. The root `CHANGELOG.md` is not this milestone's file, and the
pre-existing assertion is not this milestone's to loosen, so both were left
untouched and the finding is reported instead.

```sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --offline --prefix none \
  | sort -u | grep -v '(\*)$' | wc -l          # -> 3
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --offline --prefix none \
  | sort -u | grep -v '(\*)$' | wc -l          # -> 92
cargo tree -p eggup-transport-footprint --no-default-features --features curl,eggfetch --locked --offline --prefix none \
  | sort -u | grep -v '(\*)$' | wc -l          # -> 93
```

All three counts reproduce the recorded **3 / 92 / 93** exactly, under the method
recorded in `plans/closure/transport-footprint/001-status.md` §13.7. The counts
were **not** changed by this milestone.

### Red-first proof

Run entirely in `/tmp`; the workspace repository was never modified. The
pre-fix script is `git show HEAD:scripts/check-docs-truth.sh` (the committed
baseline); the post-fix script is the working-tree version. Each script was
placed inside the copy's `scripts/` directory, because the script derives its
root from its own location.

```sh
bash /tmp/proof.sh
```

| Mutation | Pre-fix | Post-fix | The assertion that fires |
|---|---|---|---|
| (a) `docs/crates.md` claims `eggup-curl` is published at `0.1.3` | **exit 0** | **exit 1** | `FAIL 8: docs/crates.md:59: eggup-curl is claimed as published at 0.1.3, which its CHANGELOG.md does not record as released (released: ['0.1.2'])` |
| (b) `docs/quickstart.md` gains a new normative rule | exit 0 | exit 0 | **not detected — see U1** |
| (c) `architecture/core-transaction.md` claims an escape hatch exists | **exit 0** | **exit 1** | `FAIL 9: architecture/core-transaction.md:554: states the opposite of a tracked invariant: a member declared NotRequired always fails; there is no escape hatch` |
| (d) `architecture/overview.md` gains a bogus `0.1.9` literal | **exit 0** | **exit 1** | `FAIL 1: architecture/overview.md:41: version literal 0.1.9 outside the known set ['0.1.0', '0.1.1', '0.1.2', '0.1.3']` |
| (e) a new `crates/eggup-acquisition/build.rs` | **exit 0** | **exit 1** | `FAIL 7: production source changed: crates/eggup-acquisition/build.rs` |
| unmodified tree | exit 0 | exit 0 | — |

### Regression proof — every pre-existing assertion still fires

```sh
bash /tmp/regression.sh
```

| Mutation | Assertion | Pre-fix | Post-fix |
|---|---|---|---|
| a bogus `0.1.9` in `docs/crates.md` | 1 | `FAIL 1: docs/crates.md:134: version literal 0.1.9 …` | **same line** |
| a second `published at \`0.1.2\`` in `docs/crates.md` | 2 | `FAIL 2: docs/crates.md:60: … is not the single legitimate eggup-curl occurrence` | **same lines** |
| a rule construct in a new `docs/rule-leak.md` with no owner link | 3 | `FAIL 3: docs/rule-leak.md: states a normative rule (no escape hatch) without linking its owner …` | **same line** |
| a published crate with no state marker in the root `Unreleased` | 4 | `FAIL 4: published-with-no-state-marker: eggup-core …` | **same line** |
| one milestone key with two status classes in `plans/registry.md` | 5 | `FAIL 5: milestone ('Drift probe', 'Drift probe M001') resolves to several status classes: [(532, 'closed'), (533, 'blocked')]` | **same line** |
| a changed `crates/eggup-core/src/lib.rs` | 7 | `FAIL 7: production source changed: crates/eggup-core/src/lib.rs` | **same line** |

### Workspace-wide cargo gate

```sh
CARGO_NET_OFFLINE=true ./scripts/check-local.sh
# -> exit 1, at the FIRST command, `cargo fmt --all -- --check`
```

**The failure is not in this milestone's change set.** The gate stopped on
formatting differences in `crates/eggup-core/src/candidate.rs` — a file this
milestone never touched, and which another writer is editing concurrently in the
working tree (`git diff --stat` shows `193 insertions(+), 31 deletions(-)` there).
Because `scripts/check-local.sh` runs under `set -euo pipefail`, the remaining
commands (clippy, test, doc, `cargo tree`, and the documentation-truth check) did
not run in that invocation. The documentation-truth check was run separately and
is green (§"Scoped verification" above). The parent session owns the workspace
cargo gate and should re-run it once the concurrent Rust edits are formatted.

## Per-platform results

| Platform | Result | Notes |
|---|---|---|
| **Linux** | **not run** | No Linux host was available in this session. The gate is platform-independent text analysis; the `stable`, `msrv`, and `service-systemd` lanes were not executed here. Nothing is inferred from the macOS run. |
| **macOS** (this session's host) | **pass** | The extended gate → exit 0 against a clean export of the committed baseline (`git archive HEAD`), and on every run during implementation. The three `cargo tree` counts → 3 / 92 / 93. The **working tree** at the end of this milestone is red for one reason only — a concurrent root-`CHANGELOG.md` edit that trips the pre-existing assertion 4 (U5); the pre-fix script fails identically on it. |
| **Windows** | **not run** | No Windows host was available. The `windows-check` lane was not executed, and no claim is made about it. |

## Invariant review

| Invariant | Result | Evidence |
|---|---|---|
| The gate still fails closed, and reports every defect it can find | **intact** | Every assertion appends to a failure list; all failures print before a non-zero exit. A check that cannot decide is reported as a `note`, never as a pass. |
| The gate stays dependency-free and fast | **intact** | `bash` + `python3` only; no `tomllib`; no network; sub-second runtime. |
| No false positives on the committed tree | **intact** | The extended gate → exit 0 against a clean export of the committed baseline, and after each of this milestone's own edits to `architecture/*.md`. The one red on the working tree is a concurrent edit tripping the **pre-existing** assertion 4 (U5), reproduced identically by the pre-fix script. |
| Dependency counts 3 / 92 / 93 are untouched | **intact** | Reproduced exactly (§"Exact commands run"). |
| No git state mutated | **intact** | No `git stash`/`checkout`/`restore`/`commit`/`push`/`tag`/`clean`/`reset` against the workspace repository. The throwaway repository for mutation (e) is in `/tmp` only. |
| No production source change | **intact** | `git status` shows no modification under `crates/*/src/**` by this milestone. |
| The gate is not weakened by the extension | **intact** | The regression proof shows identical `FAIL` lines pre-fix and post-fix for assertions 1, 2, 3, 4, 5, and 7. |

## Failure, rollback, and recovery review

The gate writes nothing, so there is nothing to roll back. A failure prints one
`FAIL <assertion>: <detail>` line per defect and exits 1; the exit code is the
only state. Re-running is idempotent.

Two failure modes were found and fixed **during** this milestone, and both are
recorded here rather than hidden:

1. The first three-cell registry reader treated a data row that happened to be
   followed by a separator as a table header and **dropped** it, which silently
   disabled assertion 5 for a malformed table. Caught by the regression proof
   (A5), fixed by requiring that a header's previous line is not itself a table
   row, and by never dropping a non-header row.
2. Assertion 7 could not see an **untracked** file, so the new-`build.rs` case —
   the exact case the milestone set out to cover — was invisible until the file
   was staged. Fixed by adding `git ls-files --others --exclude-standard` to the
   change set.

## Compatibility and migration review

None. No published crate, tag, manifest, dependency graph, or consumer-facing
behaviour changed. The workspace version stays `0.1.3`. The gate's exit status
keeps its meaning; it is only **stricter**. A future change that trips a new
assertion is the gate doing its job.

## Security review

No security surface is touched. The gate adds no network call, no credential
handling, and no privilege requirement. The manifest reader reads files the
repository already contains and parses only the fields the gate checks; it adds
no dependency and does not execute anything from the manifests. The gate
remains read-only and root-free.

## Documentation and operations evidence

- `architecture/tooling-governance.md` §4.1 — the assertion table now has rows
  1-5, 7, 8, and 9, each stating what it actually checks; the paragraph after
  the table states what the gate **does not decide**.
- `architecture/tooling-governance.md` §5 — the `stable`, `msrv`, `macos`, and
  `windows-check` rows name the commands the workflow runs.
- `architecture/tooling-governance.md` §5.2 and §5.3 — the `--all-features`
  asymmetry rows now distinguish "the workspace check passes no `--all-features`"
  from "the lane builds none of the feature-gated binaries", which is now true
  only of Windows.
- `architecture/transport-footprint.md` — D2's five corrections.
- `plans/closure/transport-footprint/001-status.md` — D5's re-anchoring, with a
  dated addendum that records what was changed and what was deliberately **not**
  changed (the historical YAML-parse step counts).
- `.opencode/skills/verify-workflow/SKILL.md` — the gate's scope sentence and
  the Windows lane's step count and enumeration are **not** corrected here; the
  exact replacement text is in the appendix below, for the parent to apply.

## Anchors verified against the current workflow

Every `ci.yml:` line number written by this milestone, read from
`.github/workflows/ci.yml` at the current baseline:

| What | Lines |
|---|---|
| `stable` footprint builds (`curl`, `eggfetch`, `curl,eggfetch`) | 26, 27, 28 (comment 22-25) |
| `msrv` footprint checks (`curl`, `eggfetch`, `curl,eggfetch`) | 54, 55, 56 (comment 50-53); toolchain 48; workspace check 49 |
| `macos` footprint builds (`curl`, `eggfetch`) | 101, 102 (comment 98-100); workspace `--all-features` test 97 |
| `service-systemd` job | 70-87 |
| `windows-check` job | 116-154 |
| `windows-check` test steps (**ten**) | 122, 123, 124, 125, 126, 127, 134, 138, 139, 142 |
| `cargo package -p eggup-service --locked --no-verify` steps (**four**) | 41 (stable), 68 (msrv), 114 (macos), 154 (windows-check) |

Job starts: `stable` 11, `msrv` 43, `service-systemd` 70, `macos` 89,
`windows-check` 116. Workflow triggers `ci.yml:3-6`; permissions `ci.yml:7-8`.

## Unresolved findings, with severity

| # | Finding | Severity | Status |
|---|---|---|---|
| U1 | **A normative rule added to a file that already cites its owning contract is not detected.** Mutation (b) — a new rule in `docs/quickstart.md` — still exits 0. Assertion 3 asks whether a *file* links the owner, not whether a *line* was added. A paragraph-level or sentence-level owner requirement was evaluated and **rejected**: the repository's own consumer surfaces cite the owner in a different section (`docs/quickstart.md` cites it at "Where to go next", `README.md` at "Documentation"), so a stricter rule would fail on the current tree. Detecting an *added* rule needs a documentation-diff mode, which the script does not have. | **low** | **Open** — a bounded follow-on: add a `--baseline` documentation-delta assertion that flags an added normative construct in a non-owner file whose added hunk carries no owner citation. Recorded, not closed. |
| U2 | `architecture/tooling-governance.md` §2.4 describes `eggup-core`'s dependencies as `sha2 = "0.10"` with the boundary rule "Sole dep. No transport, no service manager, no Eggpack." The manifest also declares `self-replace` under `[target.'cfg(windows)'.dependencies]`. The row is therefore incomplete, though its **rule** ("no transport, no service manager, no Eggpack") is still true. The parent is repairing the same fact in `docs/crates.md` concurrently. | **low** | **Open — parent-owned.** Not changed here, to avoid a concurrent edit of the same table. |
| U3 | Six other closure records and plans still say "nine separate `cargo test` steps" for the Windows lane (`plans/closure/verified-update-core/014,015,017-status.md`, `plans/closure/archive-extraction/005-status.md`, `plans/closure/acquisition-transport/014-status.md`, `plans/implementation/transport-footprint/001-…-corrective.md`). The lane has had ten steps since commit `b9885fa`. | **low** | **Open — not owned by this milestone.** The transport-footprint closure record, which this milestone does own, is corrected. |
| U4 | The `verify-workflow` skill's Windows lane enumeration still omits the live SCM fixture step, so its list of steps does not add up to the "ten" its own table row claims. | **low** | **Open — appendix below.** |
| U5 | **Assertion 4 (pre-existing) false-positives on a correct bullet.** The root `CHANGELOG.md`'s `Unreleased` bullet added concurrently by another writer says the seam's *next* release is unpublished ("capabilities … that exist only in the unpublished seam", `eggup-acquisition 0.2.0`). Assertion 4 matches a crate **name** anywhere in the bullet and then rejects any occurrence of "unpublished" in it, without requiring the word to be about that crate's published version, so the bullet is rejected. The pre-fix script fails identically, so this is pre-existing behaviour and not a regression from this milestone. | **low** | **Open — parent-owned.** Two fixes are available and neither is this milestone's to apply: (i) reword the concurrent bullet so the word "unpublished" is not in the same bullet as the crate names, or (ii) scope assertion 4's "unpublished" test to the clause that names the crate's **published** version (`eggup-X 0.1.3`) rather than to the whole bullet. Option (ii) keeps the assertion's power — it still rejects "`eggup-core 0.1.3` is unpublished" — and is the recommended fix, but it changes a pre-existing assertion, which this milestone was scoped not to do. |

## Roadmap disposition

No subsystem roadmap advances: this workstream has none, and no crate's roadmap
row changes. The transport-footprint roadmap's M001 row is unaffected — this
milestone corrects the deep dive that M001's closure pointed at, not the roadmap.

## Registry updates

**The parent session owns `plans/registry.md`.** Nothing was edited here. One
edit is required when this milestone closes: the "Recently closed foundation"
table's row

> | Planning/closure hygiene corrective | C001-C013 all closed (docs-only reconciliation track) | `plans/closure/planning-closure-hygiene-corrective/` |

must become

> | Planning/closure hygiene corrective | C001-C014 all closed (docs-only reconciliation track) | `plans/closure/planning-closure-hygiene-corrective/` |

## Ready-to-paste appendix — files this milestone does not own

Each block below is **verbatim** replacement text. Apply only what is marked
outstanding; items marked *already applied* are recorded for traceability, not
for action.

### A1 — `AGENTS.md`

*Already applied by the parent session, correctly:* the Windows-lane test-step
count now reads **ten**. The current sentence is:

> CI (`.github/workflows/ci.yml`) adds what the script lacks: `stable` (fmt/clippy/test/doc plus the packaged-service step on Linux; its registry-resolved compile is deferred), `msrv` (workspace `cargo check` and the packaged-service step on 1.89.0; its registry-resolved compile is deferred), `service-systemd` (registry-only published 0.1.2 negative controls, current real-systemd fixtures, and published registry-only 0.1.3 positive quiescence and ownership controls), `macos` (full `cargo test` plus the packaged-service step; its registry-resolved compile is deferred), and `windows-check` — which runs ten `cargo test` steps, then `cargo package -p eggup-service --locked --no-verify`. Windows tests cover acquisition/archive/curl, eggpack, three named service diagnostic tests, SCM, the live SCM fixture, Core `current_executable` and `stale_lock_recovery`, then the full workspace. Core M010/M011 self-replacement and lock-race fixtures execute natively, not merely compile. Neither `msrv` nor `windows-check` passes `--all-features`, so the feature-gated footprint binaries are built only by `stable` and `macos`. CI never publishes; releases are manual.

**Outstanding A1.1 — the last sentence of that paragraph is now false.** The
`msrv` lane checks all three feature-gated footprint configurations by dedicated
steps (`ci.yml:54-56`). Replace **only** this sentence:

> Neither `msrv` nor `windows-check` passes `--all-features`, so the feature-gated footprint binaries are built only by `stable` and `macos`.

with:

> Neither `msrv` nor `windows-check` passes `--all-features` on its workspace check, but `msrv` checks the three feature-gated footprint configurations by dedicated steps, so the footprint binaries are built by `stable` and `macos` and checked at the 1.89 floor by `msrv`; only `windows-check` builds none of them.

**Outstanding A1.2 — the workspace-layout bullet is incomplete.** The transport-footprint
M001 closure asked for this bullet to name the guard test and the CI coverage.
Replace **only** this bullet:

> - `crates/eggup-transport-footprint` — non-published (`publish = false`) footprint fixtures: `curl_only` / `eggfetch_only` / `dual` binaries behind `required-features`.

with:

> - `crates/eggup-transport-footprint` — non-published (`publish = false`) footprint fixtures: `curl_only` / `eggfetch_only` / `dual` binaries behind `required-features`, with a source-level linkage-discipline guard in `tests/cross_link_guard.rs`. The isolated single-adapter configurations are built by `stable` and `macos` and checked at the MSRV floor by `msrv`; `windows-check` builds none of them.

### A2 — `.opencode/skills/docs-hygiene/SKILL.md`

*Nothing outstanding.* The parent session already applied both corrections, and
both are correct as they stand:

> 1. **CI lane understatement.** Windows runs ten `cargo test` steps
>    (`ci.yml:122-142`) plus the packaged-service step, whose registry-resolved
>    compile is deferred. There is also a privileged Linux `service-systemd`
>    lane. Count commands, not bullets: three named `eggup-service` diagnostic
>    tests are three steps. The MSRV lane runs a workspace check and the
>    packaged-service step; it does not run tests, and that step's
>    registry-resolved compile is deferred too.

> 9. **Counting steps, not bullets.** A CI lane that runs three separately-named
>    tests is three steps, not one. `windows-check` is ten `cargo test` steps
>    (`ci.yml:122-142`): one combined three-crate run, eggpack, three named
>    service tests, SCM, the live SCM fixture, two Core integration fixtures, and
>    the full workspace run. It then packages the service crate. Count the
>    `- run:` lines.

### A3 — `.opencode/skills/verify-workflow/SKILL.md`

**Outstanding A3.1 — the gate's scope sentence.** The gate's coverage now
includes `architecture/*.md` and the per-crate changelogs. Replace this
paragraph:

> `scripts/check-docs-truth.sh` is the only gate command that does not invoke
> `cargo`. It is read-only and network-free, and it asserts that the version and
> status claims in `docs/`, `README.md`, `AGENTS.md`, the root `CHANGELOG.md`, and
> `plans/registry.md` agree with each other and with the crate manifests. It is
> wired into the local gate only — no CI lane runs it — so a pull request is not
> checked for documentation drift until a lane is added.

with:

> `scripts/check-docs-truth.sh` is the only gate command that does not invoke
> `cargo`. It is read-only and network-free, and it asserts that the version and
> status claims in `docs/`, `README.md`, `AGENTS.md`, the root `CHANGELOG.md`,
> `architecture/*.md`, and `plans/registry.md` agree with each other and with the
> crate manifests and the per-crate changelogs. It is wired into the local gate
> only — no CI lane runs it — so a pull request is not checked for documentation
> drift until a lane is added.

**Outstanding A3.2 — the Windows lane enumeration is missing one step.** The
enumeration lists nine `cargo test` steps; the lane runs ten. Replace the whole
enumeration block with:

> 1. `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked`
> 2. `cargo test -p eggup-eggpack --all-targets --all-features --locked`
> 3. three named `eggup-service --lib` UTF-8/control-character diagnostic-safety
>    tests (`diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges`,
>    `service_errors_output_and_permission_remediation_stay_utf8_bounded`,
>    `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free`)
> 4. `cargo test -p eggup-service --lib windows_scm::tests --locked`
> 5. `cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture`
>    — the live SCM fixture: it registers a disposable service through the
>    crate's public API and drives it against the genuine Service Control
>    Manager. It needs a hosted Windows runner with administrator rights and
>    records a skip reason on stdout when those rights are absent, which is why
>    it runs with `--nocapture`.
> 6. `cargo test -p eggup-core --test current_executable --locked` — replaces a
>    real running Windows image from a child process, both `KeepInstalled` and
>    `RollBack`. A Windows claim about self-update is **only** evidenced here.
> 7. `cargo test -p eggup-core --test stale_lock_recovery --locked` — exercises
>    the claim/race fixtures natively; `rename` semantics differ per platform, so
>    compile-only evidence is not sufficient for M011.
> 8. `cargo test --workspace --all-targets --locked --no-fail-fast`
> 9. `cargo package -p eggup-service --locked --no-verify` — packages the service crate on
>    Windows. Its registry-resolved compile is **deferred**: the source line uses core
>    capabilities (Core M014, M017) the published `eggup-core 0.1.3` does not carry, so the
>    strict form cannot pass until the `eggup-core` republication lands. The step still
>    validates the manifest and the packaged file list; the workspace build, test, and
>    clippy steps cover the compile against the workspace path dependency

…and change the sentence that follows the enumeration from

> Step 7 exists because the per-crate steps cannot catch a regression in an
> untargeted crate. It passes `--no-fail-fast` deliberately: this lane is the only
> Windows test coverage, so one failing target must not hide the state of the rest.

to

> Step 8 exists because the per-crate steps cannot catch a regression in an
> untargeted crate. It passes `--no-fail-fast` deliberately: this lane is the only
> Windows test coverage, so one failing target must not hide the state of the rest.

**Outstanding A3.3 — the coverage-gap bullet is now false for `msrv`.** Replace
this bullet:

> - `msrv` and `windows-check` both omit `--all-features`. The three
>   `eggup-transport-footprint` binaries are behind `required-features`, so **no
>   MSRV lane and no Windows lane ever compiles them**. Only `stable` and `macos`
>   build non-default features.

with:

> - `msrv` and `windows-check` both omit `--all-features` **on the workspace
>   check**. The three `eggup-transport-footprint` binaries are behind
>   `required-features`, so a lane has to name them: `msrv` checks all three
>   configurations by dedicated steps (`ci.yml:54-56`), and `stable` and `macos`
>   build them. **No Windows lane ever compiles them** — that asymmetry is an
>   open, deliberate decision recorded in the transport-footprint closure.

**Outstanding A3.4 — the CI-lane table omits the footprint steps.** Replace these
three rows:

> | `stable` | ubuntu | fmt + clippy + full `cargo test` + `cargo doc` + the packaged-service step; the only lane running clippy, fmt, and doc. That step's registry-resolved compile is deferred |
> | `msrv` | ubuntu, pinned `1.89.0` | workspace `cargo check` and the packaged-service step at the MSRV (registry-resolved compile deferred) |
> | `macos` | macos-latest | full `cargo test` + the packaged-service step; the only macOS evidence. That step's registry-resolved compile is deferred |

with:

> | `stable` | ubuntu | fmt + clippy + full `cargo test` + `cargo doc` + the three isolated `eggup-transport-footprint` builds (`ci.yml:26-28`) + the packaged-service step; the only lane running clippy, fmt, and doc. That step's registry-resolved compile is deferred |
> | `msrv` | ubuntu, pinned `1.89.0` | workspace `cargo check` + the three isolated `eggup-transport-footprint` checks (`ci.yml:54-56`) + the packaged-service step at the MSRV (registry-resolved compile deferred) |
> | `macos` | macos-latest | full `cargo test` + the two isolated `eggup-transport-footprint` builds (`ci.yml:101-102`) + the packaged-service step; the only macOS evidence. That step's registry-resolved compile is deferred |

### A4 — `architecture/tooling-governance.md`

**Nothing outstanding.** This milestone owns the file and applied every change
in it.

### A5 — `docs/**`

**Nothing outstanding.** No file under `docs/` was edited by this milestone; the
gate reads them, and the only `docs/` change in the working tree is the parent's
concurrent `self-replace` repair in `docs/crates.md`.

### A6 — `plans/registry.md`

**Nothing outstanding here.** The parent owns the file. The single required edit
is the `C001-C013` → `C001-C014` row, given verbatim in §"Registry updates"
above.
