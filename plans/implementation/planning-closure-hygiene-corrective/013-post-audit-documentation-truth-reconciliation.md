# Planning/Closure Hygiene Corrective 013 — post-audit documentation truth reconciliation

Status: closed 2026-10-23; see `plans/closure/planning-closure-hygiene-corrective/013-status.md` (documentation-only — no platform lane applies to the change itself; the whole-tree local gate and hosted run `40008039843` are green)

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:
- `plans/registry.md` — the active control surface. This workstream has **no subsystem roadmap**, and that is deliberate, not an omission. `plans/subsystems/` holds exactly seven roadmaps (`acquisition-transport`, `archive-extraction`, `consumer-adoption`, `distribution-bootstrap`, `eggpack-manifest-interoperability`, `service-lifecycle`, `verified-update-core`) and none owns documentation reconciliation. The planning-workflow skill defines `planning-closure-hygiene-corrective` as "the docs-only workstream for reconciling the registry, roadmaps, and closure records against reality; it has no subsystem roadmap" (`.opencode/skills/planning-workflow/SKILL.md:88-90`). No subsystem's roadmap advances here, so the registry plus the normative process documents are the roadmap equivalent.
- `plans/003-planning-process.md` — the normative process this milestone executes (registry rules §13, baselines §14, verification honesty §15).
- `architecture/tooling-governance.md` §6 (process) and §7 (doc ownership) — the deep dive owning the doc-change chain.

Original closed work affected:
- `plans/implementation/planning-closure-hygiene-corrective/012-post-m004-roadmap-and-registry-reconciliation.md`
- `plans/closure/planning-closure-hygiene-corrective/012-status.md`

Finding of record:
- `plans/registry.md` § "Post-0.1.3 open findings" — entry **11** (registered at baseline `c480541`
  alongside this plan): *"11. Root `CHANGELOG.md`, the consumer-facing `docs/`, `AGENTS.md`, the
  architecture deep dives, and this registry itself carry stale or contradictory status text after
  the 0.1.3 train."* The finding is assessed **medium** severity and is closed when this milestone
  closes.
  - **Numbering.** The section carried items **1** and **2** before this batch. The concurrent
    milestone plans written at this baseline originally collided on the numbers after them — two
    of them claimed **3** — and the registrar has since assigned unique numbers: core M014 holds
    **3**, archive M005 holds **4**, service M012 holds **5**, service M014 holds **6**, service M013
    holds **7**, acquisition M013 holds **8**, eggpack M006 holds **9**, and core M015 holds
    **10**. C013 therefore holds **11**. **Two entries may never share a number.**
  - The entry's surface list must name every surface WP1–WP3 actually corrects: an entry narrower
    than the change is itself drift. The registered text above already names the root changelog,
    `docs/`, `AGENTS.md`, the architecture deep dives, and the registry's own rows.

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: polish

## 1. Objective

Make Eggup's documentation agree with `plans/registry.md` and with what is actually published on crates.io.

The 0.1.3 train shipped crate-by-crate between 2026-10-06 and 2026-10-09 (`plans/registry.md:75-94`). The repository's *authoritative* surfaces tracked that correctly. The *derived* surfaces did not: the root changelog still describes published work as unpublished, the consumer-facing `docs/` still name `0.1.2` as current, and the registry asserts three different states for one milestone inside one file. This milestone changes no production source, adds no dependency, and publishes nothing. It reconciles text: (1) make the root `CHANGELOG.md` `Unreleased` section stop contradicting the `0.1.3` section above it and stop re-listing published work as unpublished; (2) make every version claim in `docs/`, `README.md`, `AGENTS.md` and `architecture/*.md` match the manifests and the registry; (3) make `plans/registry.md` assert exactly one state per milestone and say which table is authoritative; (4) stop the registry being 95% closed history and carrying 4,000-character table cells; (5) correct `architecture/transport-footprint.md`, which claims a build guarantee the build CI does not provide; (6) decide, explicitly and in writing, the fate of the `eggup-core` contract docs excluded from the published crate.

## 2. Why this milestone is ready

Every finding below was characterized by reading the files at the baseline above, not inferred. No runtime decision, re-implementation, or re-qualification is needed. Scope is documentation-only: no `src/` byte changes, no manifest change (§6.6), no new dependency, no republication. No open milestone exists in this workstream — C001–C012 are all closed (`plans/registry.md:62`, C012 row at `:190`).

**Severity, stated explicitly rather than assumed.** Findings A, B, C, F, G are **medium**. Findings D, E are **low**/quality.

- **Medium** means a consumer or contributor is told something false about the current published state, and acting on it produces a wrong outcome — pinning a superseded version, depending on a crate that is not published, or copying a status that contradicts the registry. A is medium because the root changelog tells a consumer five published crates are unpublished. B is medium because `docs/crates.md` tells a consumer to depend on `eggup-core 0.1.2` when `0.1.3` is the baseline, and because `docs/crates.md` and `docs/releases.md` disagree inside one directory. C is medium because the registry is the control surface and asserts three states for one milestone. F is medium because the deep dive states a build guarantee that does not hold. G is medium because the authoritative contracts cannot be read by the consumers they are authoritative for.
- **Low** means no false claim about published state, but the artifact fails its stated purpose — the registry is a control surface and cannot be scanned; its own rule says it should not be mostly history.

**Honest scoping of the harm.** These are documentation-truth defects. There is **no failing production report**, no consumer incident, no hosted-run failure, and **no runtime consequence**: no code path, lock, receipt, or installed artifact is affected by any finding here. The severity above is about *decision-correctness for a reader*, not runtime safety.

**The detection gap is known and specific.** The repository already tracks part of this class: `architecture/tooling-governance.md:752` is a **Resolved** drift row recording that the root `Unreleased` omitted the unpublished bug-audit fixes, and that the fix "added aggregation entries". The gap was closed by *adding* entries, and nobody removed them when the 0.1.3 train published the same work. The tracked drift was closed without re-verifying the surface it had just changed. That is why this is a corrective with regression evidence, not a fresh sweep.

## 3. Current implementation evidence

### 3.1 Finding A — the root `CHANGELOG.md` contradicts itself (medium)

`grep -n "^## " CHANGELOG.md` gives `:3` `## 0.1.2 — 2026-09-28, completed 2026-10-04`, `:122` `## 0.1.3 — 2026-10-06, in progress`, `:222` `## Unreleased`, `:368` `## 0.1.1`.

`CHANGELOG.md:224-227` opens `## Unreleased` with:

> "The workspace-wide bug-audit fixes below are recorded here as well as in the per-crate changelogs, because they are **not** on the registry. Published baselines are `0.1.2` for all seven published crates. No publication milestone authorizes these entries yet."

That is false on both halves. The `0.1.3` section directly above (`:122-131`) states 0.1.3 "is being published to crates.io crate-by-crate". The registry records `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, `eggup-eggpack` published at `0.1.3` on 2026-10-06 (`plans/registry.md:79-83`) and `eggup-service` at `0.1.3` on 2026-10-09 (`:89-91`).

`CHANGELOG.md:229-232` — the very next paragraph of the same section — contradicts its own header:

> "The two `0b8cb98` acquisition fixes that previously lived in this section are no longer here: they were published as `eggup-acquisition 0.1.3` on 2026-10-06 (Acquisition Transport M010) and are recorded in the `0.1.3` section above."

The section then re-lists work the `0.1.3` section already records as published, each described as unpublished: `:233-247` (Core M010 current-executable transaction), `:249-257` (Core M011 stale-lock recovery), `:259-268` (Eggpack Interop audit fix — **security, in a published crate**), `:270-276` (Core audit fixes), `:278-281` (Archive audit fixes), `:283-286` (Eggfetch audit fixes), `:288-297` (Acquisition audit fixes). One of those entries points the reader the wrong way: `:299` says "published in `eggup-eggpack 0.1.2` on 2026-10-02; see the 0.1.2 section above" for an item whose current home is the `0.1.3` section. Measured at the baseline, the `## Unreleased` section holds **13 bullets**, and **all 13** name a crate the `0.1.3` section records as published — 7 describe published work as "unpublished", and 6 name a published crate with no state marker at all.

### 3.2 Finding B — stale version claims across consumer-facing and index surfaces (medium)

`docs/` is explicitly non-normative (`docs/README.md`: "This directory never owns a normative claim"; `AGENTS.md`: "`docs/` is **non-normative**"), but it is the only surface a consumer reads without the rest of the tree, so a false version claim there is a real user-facing defect. `docs/crates.md` has **eleven** `0.1.2` occurrences — `:4`, `:25`, `:39`, `:50`, `:59`, `:70`, `:83`, `:84`, `:96`, `:117`, `:119`. Exactly **ten are stale**; `:59` is correct and must be left alone, because `eggup-curl` is still published only at `0.1.2` (`plans/registry.md:601`, `docs/releases.md:19`, `architecture/tooling-governance.md:601`).

The same class appears outside `docs/`, in surfaces that are **not** non-normative and therefore matter more. The full per-line sweep with the required outcome for each is §6.2; the defects are grouped as:

- **Stale current-version claims** — `docs/crates.md:3-4` (workspace version `0.1.2`; the workspace is `0.1.3`, `Cargo.toml` `[workspace.package] version = "0.1.3"`), `docs/quickstart.md:18` and `README.md:18` (`eggup-core = "0.1.2"`).
- **Stale published-state claims** — `docs/crates.md:25`, `:39`, `:50`, `:70`, `:83-84`, `:96` (crates published at `0.1.3`); `README.md:130-132` (`eggup-service` "published at `0.1.2`", and Service M010's `0.1.3` correction "has not been published" — it was published 2026-10-09 under M011); `README.md:128-129` (the published-crate list is **duplicated** on consecutive lines).
- **Stale pin claims** — `docs/crates.md:117` and `:119` (the exact pin is `=0.1.3` per `crates/eggup-eggpack/Cargo.toml`; and `eggup-eggpack 0.1.3` is published and carries the member-identity cross-check — `plans/registry.md:59`, `:189`); `AGENTS.md:22`, `:99`; `architecture/overview.md:75`; `architecture/tooling-governance.md:58`, `:118-120`, `:659`, `:697`.

`docs/releases.md` is **partly** correct and **partly** stale, and the plan must not conflate the two. Correct, leave alone: `:12` ("All workspace crates currently declare source version `0.1.3`"), the table at `:16-23`, and the historical tag/release references at `:31` and `:112-113`. Stale: `:30` (service published 2026-10-05 under M009; `0.1.3` shipped 2026-10-09 under M011); `:37` (the exact pin); `:43-45` ("cannot reach you through `eggup-eggpack 0.1.2` … needs a separate republication" — `eggup-eggpack 0.1.3` is published); `:57` ("contains a security fix that is not yet published" — it shipped in `0.1.3`); `:67-68` (points at the per-crate `Unreleased` section; the fix is in the published `0.1.3` section, and that per-crate section is correct); `:74` ("the payload of the open Acquisition M010 milestone" — M010 closed 2026-10-06); `:75-76` ("inherit it automatically once it lands" — it landed); `:80-83` ("Published `eggup-core 0.1.2` predates the `Error::Injected` addition. That variant is only in the unpublished working tree" — `Error::Injected` is `#[cfg(test)]`-only and exists in no packaged build at all; `crates/eggup-core/src/error.rs:43`, doc comment at `:36-41`).

### 3.3 Finding C — the registry asserts contradictory live status for the same milestones (medium)

Verified contradictions **inside** `plans/registry.md`.

| Milestone | Location | Text | State |
|---|---|---|---|
| Acquisition M010 | `:56` | "M001-M011 closed; … M010 published `eggup-acquisition 0.1.3` (2026-10-06) …" | closed |
| Acquisition M010 | `:185` | "closed 2026-10-06" | closed |
| Acquisition M010 | `:238` | "M010 eggup-acquisition 0.1.3 publication \| **ready**" | **ready** |
| Gregg M004 | `:266` | "Consumer adoption \| M004 Gregg \| blocked; plan intentionally unwritten" | **blocked** |
| Gregg M004 | `:159` | "…both are now authorable" | authorable |
| Gregg M004 | `:171` | "**Gregg M004 and EggPool M007 are now authorable**" | authorable |
| Gregg M004 | `:380` | "`-> Gregg M004 [WRITABLE; plan intentionally unwritten]" | writable |
| EggPool M007 | `:270` | "blocked/evidence-driven; plan intentionally unwritten" | **blocked** |
| EggPool M007 | `:171` | "…now authorable" | authorable |

A fourth M010 statement: `:367` — "`-> M010 eggup-acquisition 0.1.3 publication [READY; ships the 0b8cb98 seam fixes]". Also `:389` ("M010 is now the next dependency-ready milestone"), `:402` ("M010 publishes a new `0.1.3` version of the seam only"), and `:369` ("eggup-eggpack adapter republication [NAMED FOLLOW-ON; `=0.1.2` pin blocks inheritance]"), whose premise no longer holds.

**Stale gating clause:** `:392` — "Gregg M004 remains intentionally unwritten until Acquisition M010 and Core M012/M013 close". Both closed 2026-10-06 (`:159`). **Archive M004:** `:393` — "M004 is registered to publish the already-implemented 0.1.3 correctness fixes once workspace 0.1.3 exists"; `eggup-archive 0.1.3` was published 2026-10-06 (`:81`, `:58`).

### 3.4 Finding D — the "Planned / blocked work" table is not a planned-work table (low/quality)

`plans/registry.md:233-273` is titled "Planned / blocked work". Measured at the baseline: **37 rows** (`:237`–`:273`), of which **35 are closed or historical**; exactly two are not — `:270` (EggPool M007) and `:273` ("Authenticity/signatures | future | ADR required"). The registry's own rule at `:412-414`:

> "Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps."

A table that is 95% closure history contradicts the rule the same file states.

### 3.5 Finding E — the registry is not scannable (low/quality)

`plans/registry.md` is 414 lines / 63,234 bytes. Longest lines, measured: `:231` = 4,000 chars, `:383` = 3,267, `:15` = 2,560, `:394` = 1,888, `:49` = 1,144, `:136` = 1,051, `:389` = 948, `:402` = 702. For comparison `architecture/overview.md` is 255 lines with a 481-character maximum, and `plans/002-long-term-roadmap.md` is 373 lines with a 315-character maximum. The registry is the control surface meant to be scanned; a 4,000-character cell defeats that.

### 3.6 Finding F — the transport-footprint deep dive overstates what the build CI proves (medium)

`architecture/transport-footprint.md:169` claims:

> "Adapters do not know each other | `curl_only` links no Eggfetch code, and vice versa; the `required-features` split makes an accidental cross-link a build error, not a review finding"

That is inaccurate for the CI the repository runs. `crates/eggup-transport-footprint/Cargo.toml` declares `eggup-curl` and `eggup-eggfetch` as **optional** dependencies; once a feature is enabled the dependency is available to **every** target in the package. `required-features` only prevents a binary from *building* when its feature is off — it does not prevent `src/bin/curl_only.rs` from `use eggup_eggfetch::…` when both features are on.

The build invocations are fixed and never use a single-adapter feature set: `scripts/check-local.sh:5` is `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `.github/workflows/ci.yml:20`, `:21`, `:61`, `:71` are all `--all-features`. No `--no-default-features` and no `--features curl` / `--features eggfetch` appears anywhere in the workflow. Under `--all-features`, a cross-link compiles cleanly, and CI never builds the isolated single-adapter configurations that are the crate's entire purpose.

Additional facts about the same crate, verified: three binaries, 45 source lines, **zero** tests (`architecture/transport-footprint.md:4-7`, `:182-185`); the binaries print a config value and a redacted request and perform no measurement in code (`:14-17`); and `eggup-transport-footprint` has **no subsystem roadmap** — `plans/subsystems/` contains none for it.

The **measured numbers in that document still reproduce exactly** at the baseline: curl-only **3** unique crates, eggfetch-only **92**, dual **93**. They must not change. Note the counting method: `cargo tree --prefix none` emits `(*)` dedup markers on repeat nodes, so the naive `sort -u | wc -l` returns 118/119. The figures require the `(*)` lines excluded. Exact commands are in §11.

### 3.7 Finding G — `eggup-core`'s authoritative contracts are not shipped to consumers (medium)

`crates/eggup-core/Cargo.toml` declares `exclude = ["docs/", "tests/", "benches/"]`. `cargo package -p eggup-core --list --allow-dirty --offline` at the baseline returns:

```text
.cargo_vcs_info.json   CHANGELOG.md   Cargo.lock   Cargo.toml   Cargo.toml.orig   README.md
examples/custom_validator.rs   examples/multi_member.rs   examples/one_member.rs
examples/ownership.rs   examples/receipts.rs   examples/stale_lock_recovery.rs
src/candidate.rs   src/current_exe.rs   src/domain.rs   src/error.rs   src/integrity.rs
src/lib.rs   src/lock.rs   src/stage.rs   src/test_support.rs   src/transaction.rs
```

`crates/eggup-core/docs/domain.md`, `crates/eggup-core/docs/verification.md` and `crates/eggup-core/docs/transaction.md` are **absent**. `AGENTS.md` calls those three files the authoritative contracts, `architecture/overview.md` routes readers to them, and `docs/README.md` names them the owner of contract semantics. A consumer on crates.io or docs.rs cannot read them.

## 4. Invariants that must not regress

- `docs/` stays **non-normative**: it points at the contracts, it does not restate them. No new normative rule may be introduced under `docs/`.
- No rule may be introduced outside `architecture/` and `crates/eggup-core/docs/`. `README.md` and `AGENTS.md` stay thin indexes that link rather than restate.
- `plans/000`–`003` and the accepted ADRs are **not touched**. They are normative; changing them is itself a milestone (`.opencode/skills/planning-workflow/SKILL.md:98-99`).
- The measured transport-footprint figures **3 / 92 / 93** are unchanged. `architecture/transport-footprint.md:93-124` is a dated capture at commit `ae9b47a`; its counts are not re-derived, re-estimated, or re-rounded.
- The registry remains the **active control surface** and keeps the "active/ready work" discipline of `plans/registry.md:412-414`.
- The root `CHANGELOG.md` and the per-crate changelogs stay consistent: the root aggregates, it does not replace.
- Published-version facts are sourced from `plans/registry.md` and `docs/releases.md`, never invented. A version not in the registry is not asserted anywhere.
- The per-crate changelog `Unreleased` sections were individually reviewed and are **correct** — e.g. `crates/eggup-core/CHANGELOG.md:3` (`## 0.1.3 — 2026-10-06`) and `:26-28` ("Nothing in this section has been published. `0.1.3` is the published baseline on crates.io, and no publication milestone authorizes these entries."). The defect is in the **root** changelog only. The reconciliation must not churn correct per-crate files.
- No closure record is deleted. Superseded predecessors keep their closure records and are marked historical or superseded in the registry, never removed from the closure set (`.opencode/skills/planning-workflow/SKILL.md:96-97`).
- No `src/` byte changes. No dependency added. No version published. No tag moved.

## 5. Scope

### In scope

Root `CHANGELOG.md` `Unreleased` section (`:222` onward); `docs/crates.md`, `docs/quickstart.md`, `docs/releases.md`; `README.md` (`:18`, `:126-140`); `AGENTS.md` (`:22`, `:99`); `architecture/overview.md` (`:75`); `architecture/tooling-governance.md` (`:58`, `:118-120`, `:659`, `:697`, plus the drift table); `architecture/transport-footprint.md`; `plans/registry.md` (status reconciliation, table authority, table shape, readability, the new open-findings entry, the C013 row); `scripts/check-docs-truth.sh` (new) and one added invocation line in `scripts/check-local.sh`; the C013 closure record `plans/closure/planning-closure-hygiene-corrective/013-status.md`.

### Explicitly out of scope

- **No production source change.** No `crates/*/src/**` byte is touched.
- **No new dependency.** No crate manifest gains or changes a dependency; `Cargo.lock` is unchanged.
- **No edit to `plans/000`–`003` and no edit to any accepted ADR.** Those are normative; changing them is its own milestone.
- **No new normative rule introduced in `docs/`.**
- **No republication.** No `cargo publish`, no package upload, no tag move, no release-notes extension, no registry mutation.
- **No change to the transport-footprint crate's source, manifest, features, `required-features` split, or measured figures.**
- **No authoring of a new transport-footprint subsystem roadmap or milestone plan.** The CI-coverage gap is recorded as an open finding with a named follow-on; the roadmap decision is that follow-on's first step.
- **No change to the `eggup-eggpack` exact-pin style or the `=0.1.0` literals** in `crates/eggup-transport-footprint/Cargo.toml` and `crates/eggup-service/Cargo.toml` — already open drift rows (`architecture/tooling-governance.md:767`, `:769`) that want their own pin-style milestone.
- **No change to `crates/eggup-core/Cargo.toml`'s `exclude` list** — see §6.6.
- No correction of a closure record's historical text. Closure records are immutable evidence; only live status surfaces are reconciled.

## 6. Required documentation changes

No production changes, so §6 states the required **documentation** changes per target file, with the exact required outcome.

### 6.1 Root `CHANGELOG.md` — WP1

`CHANGELOG.md:224-227` must be **replaced**, not softened. Required outcome: the `## Unreleased` section states the **current** published baseline — `0.1.3` for `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, `eggup-eggpack` and `eggup-service`; `0.1.2` for `eggup-curl`; `eggup-transport-footprint` is `publish = false`. "No publication milestone authorizes these entries yet" is removed — it is false. The paragraphs `:233-297` re-listing published work as unpublished are **removed from `Unreleased`**, because the `0.1.3` section (`:122`) already records that work with its per-crate provenance; nothing is lost, since the `0.1.3` section and the per-crate changelogs retain it. The `:299` pointer ("see the 0.1.2 section above") is retargeted or removed with its bullet, and `:229-232` is reconciled so the section no longer contradicts itself. `Unreleased` ends up containing **only** genuinely unpublished work; if nothing is unpublished it says so explicitly rather than re-listing published work. The section order (`0.1.2` → `0.1.3` → `Unreleased` → `0.1.1`) is left as it is; this milestone does not reorder the changelog.

### 6.2 Version-truth sweep — WP2

Every entry below is a **fact correction**. None introduces a rule.

| File:line | Required outcome |
|---|---|
| `docs/crates.md:3-4` | workspace version → `0.1.3` |
| `docs/crates.md:25`, `:39`, `:50`, `:70`, `:96` | `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, `eggup-eggpack` → published at `0.1.3` |
| `docs/crates.md:59` | **LEAVE ALONE** — `eggup-curl` is published only at `0.1.2` |
| `docs/crates.md:83-84` | `eggup-service` → published at `0.1.3` on 2026-10-09 from `feb6ae5`; the `v0.1.2`-tag sentence is rewritten for the 0.1.3 era (the `v0.1.3` tag was likewise not moved — `plans/registry.md:92`) |
| `docs/crates.md:117` | exact pin → `=0.1.3` |
| `docs/crates.md:119` | "needs a separate adapter republication" corrected: `eggup-eggpack 0.1.3` is published and carries the member-identity cross-check |
| `docs/quickstart.md:18` | `eggup-core = "0.1.3"` |
| `docs/releases.md:30` | service publication → M011, `0.1.3`, 2026-10-09 |
| `docs/releases.md:37` | exact pin → `=0.1.3` |
| `docs/releases.md:43-45` | rewritten: the fix **did** reach `eggup-eggpack 0.1.3` consumers |
| `docs/releases.md:57`, `:67-68` | the member-identity fix is described as published in `0.1.3`; the pointer targets the published `0.1.3` section of the per-crate changelog |
| `docs/releases.md:74`, `:75-76` | Acquisition M010 is closed; the fix landed 2026-10-06 and `eggup-curl 0.1.2` / `eggup-eggfetch 0.1.2` inherit it now |
| `docs/releases.md:80-83` | rewritten to the actual state: `Error::Injected` is `#[cfg(test)]`-only and exists in no packaged build; `Error` carries the seven published `0.1.2` variants |
| `README.md:18` | `eggup-core = "0.1.3"` |
| `README.md:128-129` | the duplicated crate list is de-duplicated |
| `README.md:130-132` | `eggup-service` published at `0.1.3`; "has not been published" removed |
| `AGENTS.md:22`, `:99` | the `=0.1.2` pin claim → `=0.1.3`, referenced to its owner `crates/eggup-eggpack/Cargo.toml` rather than restated twice |
| `architecture/overview.md:75` | `=0.1.3` |
| `architecture/tooling-governance.md:58` | snippet shows `version = "0.1.3"` |
| `architecture/tooling-governance.md:118-120` | `=0.1.3`, and "the workspace version is now `0.1.3`" |
| `architecture/tooling-governance.md:659`, `:697` | `=0.1.3` |

**Must be left alone.** The `0.1.2` references that are correct history: the `docs/releases.md` table rows `:16-23`; the `v0.1.2` tag references at `docs/releases.md:31` and `:112-113`; the drift-table rows at `architecture/tooling-governance.md:752-758` that describe the 0.1.2-era state as it was then (except `:752`, corrected by WP7); and `docs/crates.md:59`. A blanket find-and-replace on `0.1.2` is the wrong tool and is forbidden by this section.

### 6.3 `plans/registry.md` — WP3 (status reconciliation and table authority)

**Decision, made explicit, as the drift table requires.** `architecture/tooling-governance.md:770` records that the two tables duplicate 16 `(subsystem, milestone)` rows and that 40 of 44 rows are closed records, and says deduplicating "wants an explicit decision and its own hygiene milestone rather than a docs pass". **This milestone is that hygiene milestone, and the decision is: the `Dependency-ready implementation work` table (`plans/registry.md:173-229`) is the authoritative milestone-status table**, because it carries each milestone's plan path and its closure-record path. **The `Planned / blocked work` table (`:233-273`) is narrowed to open, blocked, authorable, and future work only**, and every closed or historical row is removed from it. Measured duplication to eliminate: **16** duplicated `(subsystem, milestone)` keys, and `Eggpack M004` appearing **twice inside the `Planned / blocked work` table itself** (`:242` and `:272`).

Required outcomes: `:238` — M010's state becomes **closed 2026-10-06**, matching `:185` and `:56`; the `**ready**` cell and its stale "publication-only" prose are removed. `:185`, `:56`, `:367`, `:369`, `:389`, `:402` — the remaining M010 statements are reconciled to the single closed state; `:367`/`:389` lose the "READY" / "next dependency-ready milestone" framing; `:369`'s eggpack-republication follow-on is retired; `:402` is restated in the past tense. `:266` — Gregg M004 becomes **authorable; plan intentionally unwritten**, matching `:159`, `:171`, `:380`; the word "blocked" must not survive anywhere for Gregg M004. `:270` — EggPool M007 becomes **authorable; plan intentionally unwritten**, matching `:171`. `:392` — the stale gating clause is replaced with the current authorable state, citing the published handoffs. `:393` — the Archive M004 "once workspace 0.1.3 exists" clause is replaced with the published `eggup-archive 0.1.3` state. `:273` — the "Authenticity/signatures | future | ADR required" row **stays**: it is genuinely not-closed work and the ADR gate is real. The `Planned / blocked work` table's heading is renamed to match its narrowed content. The registry gains **one** sentence naming the authoritative table, so the next reader does not re-derive it.

**Deduplication of the 16 shared keys.** Each duplicated key appears in exactly one table after the change. Where the surviving row is a closed record it stays in the authoritative table with its closure-record path; where it is a superseded predecessor it is marked **historical / superseded** rather than deleted.

### 6.4 `plans/registry.md` — WP4 (readability pass)

The 4,000-character cell at `:231` is broken up; long closed-history prose moves out, because `plans/closure/<subsystem>/NNN-status.md` is where closure detail belongs and the remaining sentence must still say what the registry claims. The 3,267-character cell at `:383` and the 2,560-character cell at `:15` are reduced the same way. The 1,888-character cell at `:394`, the 1,144-character cell at `:49`, the 1,051-character cell at `:136`, the 948-character cell at `:389` and the 702-character cell at `:402` are reduced to scannable form, with detail moved to the closure record that owns it. **Target:** no registry table cell exceeds a length that keeps the file scannable against `architecture/overview.md` (max 481 chars) and `plans/002-long-term-roadmap.md` (max 315 chars); the exact ceiling is set in WP4 and recorded in the closure evidence, and a line that must stay long (the header's baseline bookkeeping) is shortened by pointing at its closure record, not by deleting evidence. **No closure record is deleted and no historical fact is removed** — only relocated to the closure record that already holds it, or compressed to a pointer.

### 6.5 `architecture/transport-footprint.md` — WP5

`:169` — the "build error, not a review finding" claim is replaced with the accurate statement: `required-features` prevents a binary from *building* when its adapter's feature is off, and does **not** prevent a cross-link when both features are on, because optional dependencies become available to every target in the package once enabled. The property it does guarantee is that the *single-adapter* configurations are the only ones where "links no Eggfetch code" is meaningful — and those configurations are not built by CI. `:52-56` — the version table is corrected to the manifest's actual literal requirement (`"0.1.0"`, path) and the current resolved workspace version `0.1.3`, cross-referenced to `crates/eggup-transport-footprint/Cargo.toml`; this is a doc-versus-manifest correction only, and the manifest is not changed. `:93-124` — the measured block is **left as the dated capture at commit `ae9b47a`**, with a one-line note that the workspace version advanced to `0.1.3` after the measurement and that the counts still reproduce at the current baseline via the §11 commands; the counts 3 / 92 / 93 are not re-derived or re-worded. `:192-207` — the CI-coverage table gains the missing row for the isolated single-adapter configurations, stating plainly that no CI job builds them, and the existing asymmetry note (MSRV and Windows do not build the bins) is retained. The crate's lack of a subsystem roadmap is stated in the deep dive, with the named follow-on that owns the CI-coverage gap.

### 6.6 Finding G — the `eggup-core` contract-doc packaging decision (WP6)

**Decision: DEFER.** The `exclude = ["docs/", …]` entry in `crates/eggup-core/Cargo.toml` is **not** changed by this milestone; the defect is recorded as an open finding with a named owner.

1. It is a **manifest** change, not a documentation edit. `crates/eggup-core/Cargo.toml` owns the published package's contents, and `architecture/tooling-governance.md:767-769` sets the precedent that a manifest change of this kind "wants a milestone, not a docs pass".
2. The manifest edit alone delivers **no consumer benefit**: consumers receive the shipped docs only when a new version is published. Landing it inside a milestone that forbids republication leaves a change with no consumer-visible effect until an unauthorized publication occurred — precisely the half-landed state that produced Finding A.
3. The purpose therefore requires a republication of `eggup-core` (a `0.1.4` or later), and publication is a tracked milestone with its own qualification and ordering obligations (`architecture/tooling-governance.md` §8; `.opencode/skills/release-workflow/SKILL.md`). A docs reconciliation must not smuggle a publication gate in behind a manifest edit.
4. Making the contract docs part of the published artifact also makes every later edit to them publication-relevant. That is a release-policy decision, not a documentation decision.

**Named owner:** **Verified Update Core M016 — `eggup-core` contract-doc packaging**, owned by `plans/subsystems/verified-update-core-roadmap.md` (the Core subsystem owns `crates/eggup-core/docs/`). It must (a) remove `"docs/"` from the `exclude` list, (b) qualify `cargo package -p eggup-core --list` to show `docs/domain.md`, `docs/verification.md` and `docs/transaction.md`, and (c) authorize and execute the republication that delivers them to consumers. C013 records the finding and the owner; it does not write M015's plan. The number **M016** is the next free one at the baseline — `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md` occupies M014 and `plans/implementation/verified-update-core/015-public-api-contract-truthfulness-corrective.md` occupies M015 — and if another milestone takes M016 first, the next free number is used; the name and the owner are what matter, not the digit.

### 6.7 Drift-table updates — WP7

`architecture/tooling-governance.md` § "Known doc/code drift" is the live list (`docs-hygiene` skill: "Record residual drift there rather than silently leaving it — and remove rows as they are fixed"). The **Resolved** row at `:752` is **corrected in place**, not left standing: its resolution text is now itself stale, because the entries it added were never removed after the 0.1.3 train. The corrected row records that the aggregation entries were added, then removed from `Unreleased` when the 0.1.3 train published the same work, and that the root `Unreleased` header no longer claims an unpublished baseline. The **Open** row at `:770` is **closed and removed** once WP3 lands, with its resolution recorded in the C013 closure record; that row's own counts are stale — it says "40 of its 44 rows", while the baseline table has **37** rows of which **35** are closed or historical — and the corrected row must not repeat the old counts. The Open rows at `:767` and `:769` are **corrected in place** where this milestone changes their premise (the workspace version is `0.1.3`, not `0.1.2`), and **remain open** as rows, because the manifest literals are unchanged. A **new Open row** is added for the transport-footprint CI-coverage gap (Finding F's CI half), naming the follow-on milestone and its prerequisite. A **new Open row** is added for the `eggup-core` contract-doc packaging gap (Finding G), naming Verified Update Core M016. Rows are **removed as they are fixed**, per the table's own instruction that it "stays a live list rather than a historical log".

### 6.8 No rule may leak into `docs/` — WP8

No sentence in `docs/` may state a rule whose owner is `architecture/` or `crates/eggup-core/docs/`. The existing rule statements in `docs/quickstart.md` (`:115-124`, `:128-135`) are the ones the check is written against; they already point at the contract links at `:207-210`. `docs/` corrections point at their owner: where a `docs/` file states a version, it points at `docs/releases.md` (which points at crates.io) rather than asserting an independent fact.

## 7. Ordered work packages

Each work package is independently verifiable. Order matters where a later package depends on an earlier one's output.

**WP1 — Root `CHANGELOG.md` truth.** Correct `:224-227`; remove the re-listed published entries (`:233-297`); retarget the `:299` pointer; reconcile `:229-232`. *Verifiable:* `Unreleased` contains no entry the `0.1.3` section records as published, and its header names the current published baseline.

**WP2 — Version-truth sweep.** Apply §6.2 line by line, including the `docs/crates.md:59` exclusion and the `docs/releases.md` history exclusions. *Verifiable:* the grep assertions in §11 return empty.

**WP3 — Registry status reconciliation and authoritative-table decision.** Apply §6.3: reconcile M010, Gregg M004, EggPool M007, the stale gating clause and the Archive M004 clause; narrow the `Planned / blocked work` table to open work only; eliminate the 16 duplicated keys and the double Eggpack M004 row; add the sentence naming the authoritative table. *Verifiable:* the §11 registry-status command reports `conflicting milestone states: 0`.

**WP4 — Registry readability pass.** Apply §6.4 to the eight over-long cells, relocating detail to the closure record that owns it. *Verifiable:* the max-line-length measurement in §11 is at or below the recorded ceiling.

**WP5 — Transport-footprint deep dive correction and follow-on naming.** Apply §6.5: correct `:169` and `:52-56`; annotate the dated measurement at `:93-124`; extend the CI-coverage table; state the missing roadmap and name the follow-on. *Verifiable:* the doc no longer claims a build guarantee the CI does not provide, and the counts are unchanged.

**WP6 — Finding G decision.** Apply §6.6: record the deferral in the drift table and name Verified Update Core M016 as owner. The manifest is not changed. *Verifiable:* `git diff --name-only` shows no `crates/*/Cargo.toml` change, and the drift table names the owner.

**WP7 — Drift-table update.** Apply §6.7: correct the stale Resolved row at `:752`; close and remove the Open row at `:770`; correct the premises of `:767`/`:769`; add the two new Open rows. *Verifiable:* the drift table contains no row whose claim contradicts the corrected surfaces.

**WP8 — Regression evidence and no-leak verification.** Create `scripts/check-docs-truth.sh`; add one invocation line to `scripts/check-local.sh`; update `architecture/tooling-governance.md` §4–§5 for the added gate line; run the full §11 command set, including the pre-change failure demonstration required by §10. *Verifiable:* the script fails on the baseline and passes on the corrected tree; `git diff --name-only` shows no `src/` file.

**Ordering:** WP1 → WP2 → WP3 → WP4 → WP5 → WP6 → WP7 → WP8. WP1–WP2 are independent of WP3–WP5; WP4 follows WP3 (the table narrowing changes which cells exist); WP7 follows WP1–WP6; WP8 follows everything.

## 8. Failure, cancellation, restart, and contention semantics

No runtime semantics, no artifact mutation, no external process behavior. The semantics that apply are those of the reconciliation itself.

**Precedence rule — the registry wins.** When two sources disagree about a milestone's status or a published version, `plans/registry.md` is authoritative, because it is the active control surface (`plans/registry.md:24`, `:412-414`). `docs/releases.md` is authoritative for *registry* state because it is the surface verified against crates.io (`docs/releases.md:3-8`), and crates.io is authoritative for what exists (`docs/README.md`: "Which versions exist on crates.io — crates.io, not this directory"). `crates/*/Cargo.toml` is authoritative for the workspace version and dependency pins. `architecture/` and `crates/eggup-core/docs/` are authoritative for rules. A correction never resolves a disagreement by inventing a third value; it resolves it by naming the owner and quoting it.

**A closed record still listed as planned is removed from the active table and retained in the closure record set.** Removing a *row* from the registry's active table is not deleting a closure record. No closure record may be deleted by this milestone. Superseded predecessors **keep** their closure records and are marked *historical* or *superseded* in the registry rather than deleted (`.opencode/skills/planning-workflow/SKILL.md:96-97`). The registry's "Recently closed foundation" table (`:51-62`) and the directories under `plans/closure/` remain the retained record set.

**Contradictory rows are not averaged.** If a milestone is `closed` in the authoritative table and `ready` in another, the `ready` row is corrected — not turned into a third value such as "mostly closed". A milestone is either open or it is not; the registry must not carry both.

**Restart semantics.** If the work is interrupted, the registry remains consistent: a milestone may be registered as `ready for handoff` before implementation and updated to `closed` only with a closure record in hand. If a later corrective reopens a milestone, it appends a dated addendum rather than rewriting historical evidence (`docs-hygiene` drift class 11).

**Contention.** If a concurrent change lands on `plans/registry.md`, `CHANGELOG.md`, `docs/`, `README.md`, `AGENTS.md` or `architecture/` while C013 is in flight, re-read the affected file, re-apply the reconciliation against the new text, and re-run the §11 checks. Do not carry a claim forward without re-reading its file. If the concurrent change is itself a publication, re-read the registry and `docs/releases.md` first, because a new published version invalidates the version claims C013 is correcting. The same rule governs the registry entry number: it is assigned at registration to the next free number, and no two entries share a number.

**Cancellation.** If the registry's duplicated rows cannot be reconciled without a decision only the maintainer can make (§14), the contradiction is **recorded** — written down as an open finding naming the specific rows — rather than half-reconciled. A half-reconciled registry is worse than a contradictory one that is honestly labelled.

**Fail-closed.** Every assertion in §10 is fail-closed: a check that cannot evaluate (a missing file, a grep that matches nothing because the file was renamed) is a **failure**, not a pass. Silence is not evidence.

## 9. Compatibility and migration

**No runtime behavior changes.** No `src/` byte changes, no dependency changes, no `Cargo.lock` change, no feature changes, no public API change.

**No published artifact changes.** Nothing is republished; no tag moves; no release notes extended. The crates on crates.io are exactly what they were.

**What consumers see after the change:** `docs/crates.md` and `docs/quickstart.md` name `0.1.3`, and `README.md`'s dependency snippet names `0.1.3`, so a consumer who copies the snippet gets the current published baseline instead of the previous one. `docs/releases.md`'s limitations section describes the versions actually published, so a reader is not told that a shipped security fix is still unshipped. The root `CHANGELOG.md` no longer tells a consumer that five published crates are unpublished. `README.md`'s publication-state paragraph is de-duplicated and states `eggup-service 0.1.3` correctly.

**Migration:** none required. No consumer action is forced by a documentation correction. A consumer who pinned `0.1.2` remains validly pinned; the corrected text only stops recommending a superseded baseline.

**The only packaging question is Finding G**, and §6.6 defers it. `cargo package -p eggup-core --list` output is therefore **unchanged** by this milestone, and `crates/eggup-core/docs/` remains excluded from the published crate until Verified Update Core M016 lands.

## 10. Required tests

**These are documentation assertions, not production tests.** No unit test, integration fixture, or benchmark is added, because no production behavior changes and `cargo test --workspace --all-targets --all-features --locked` cannot fail on a documentation defect. The "tests" for this milestone are checks that the documentation agrees with itself, with the manifests, and with the registry.

**Where they live.** A new script `scripts/check-docs-truth.sh` — read-only, `grep`/`python3`-based, no network, no cargo invocation — plus one added invocation line in `scripts/check-local.sh`. Rationale: the planning-workflow skill requires a corrective plan to "add regression evidence" (`.opencode/skills/planning-workflow/SKILL.md:42-47`), and the detection gap was exactly that tracked drift was closed without re-verifying. The five existing gate commands are **not** modified; the new line is appended. The corresponding `architecture/tooling-governance.md` §4–§5 update is part of WP8 (§12).

**The required assertions:**

1. **No `docs/` file claims a version the registry does not.** No `0.1.N` literal outside `{0.1.0, 0.1.1, 0.1.2, 0.1.3}` appears anywhere under `docs/`, `README.md` or `AGENTS.md`.
2. **No `docs/` file states the current version as `0.1.2`.** The strings `workspace version` + `0.1.2`, `published at \`0.1.2\`` (excluding the single legitimate `eggup-curl` occurrence), and `=0.1.2` (excluding the historical rows the sweep is forbidden to touch) must not survive.
3. **No normative rule language leaked into `docs/`.** The check is a targeted list of forbidden constructs, not a language model.
4. **The root `CHANGELOG.md` `Unreleased` section does not contradict the `0.1.3` section.** Neither false header sentence survives, and no bullet in `Unreleased` describes a crate published at `0.1.3` as unpublished or names one with no state marker.
5. **No milestone appears with two different statuses in `plans/registry.md`.** Every `(subsystem, milestone)` key resolves to exactly one status class across every milestone table, and the six cross-surface contradictions (row, execution graph, prose) return empty under the targeted greps.
6. **The transport-footprint counts still equal 3 / 92 / 93**, via the exact `cargo tree` commands in §11.
7. **No production source changed.** The change set contains no `crates/*/src/**` path and no `Cargo.lock` change.

**The check must be shown to FAIL on the pre-change baseline.** An assertion never observed to fail has not been shown to work. The closure evidence must include the failing run against the baseline and the passing run against the corrected tree, for the same script.

**Platform note.** Assertions 1–5 and 7 are platform-independent text checks, recorded once, not per platform. Assertion 6 is a `cargo tree` resolution check, also platform-independent in the counts it produces. Assertions 2 and 4 do not vary by operating system.

## 11. Required verification commands

Exact strings, run from the repository root at the baseline and at the corrected tree.

**Transport-footprint counts (assertion 6).** The `(*)` dedup markers that `cargo tree --prefix none` emits must be excluded; without that filter the counts read 118 and 119 and the *check* is wrong, not the tree.

```sh
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked --prefix none | sort -u | grep -v '(\*)$' | wc -l
cargo tree -p eggup-transport-footprint --all-features --locked --prefix none | sort -u | grep -v '(\*)$' | wc -l
```
Expected: `3`, `92`, `93`, in that order — the figures in `architecture/transport-footprint.md:98-100`, unchanged.

**Package contents (Finding G evidence — the manifest is unchanged, so this output must also be unchanged).**

```sh
cargo package -p eggup-core --list --allow-dirty --offline
```
Expected: the baseline list (§3.7), **without** `docs/domain.md`, `docs/verification.md`, `docs/transaction.md`. If those three appear, the `exclude` list was changed and the milestone has exceeded its scope.

**Workspace gate.**

```sh
cargo test --workspace --all-targets --all-features --locked
./scripts/check-local.sh
```

**Version-truth assertions (assertions 1–2).**

```sh
grep -rnE '0\.1\.[0-9]+' docs/ README.md AGENTS.md | grep -vE '0\.1\.(0|1|2|3)\b'
```
Expected: empty. Any hit is a version literal outside the known set.

```sh
grep -rn 'workspace version' docs/ README.md AGENTS.md
```
Expected: every hit names `0.1.3`.

```sh
grep -rn '=0\.1\.2' AGENTS.md architecture/overview.md architecture/tooling-governance.md docs/
```
Expected: empty, except the drift-table rows that legitimately describe the 0.1.2-era state (`architecture/tooling-governance.md:752-758`) and the `docs/releases.md` history references, which are excluded by an explicit allowlist inside the script.

```sh
grep -n 'published at `0\.1\.2`' docs/crates.md
```
Expected: exactly one hit, `docs/crates.md:59` (`eggup-curl`), which is **correct** and must survive.

**Registry status assertions (assertion 5).** Two layers, because the registry states status in tables *and* in prose and in the execution graph, and a table-only check cannot see the prose layer.

*(a) Duplicate-row check.* For each `(subsystem, milestone)` key, every row across every table in the file must classify the milestone identically, where a status cell is classified `conditional` if it contains "conditional", `closed` if it contains "closed", `historical` if it contains "historical" or "superseded", `authorable` if it contains "authorable" or "writable", `blocked` if it contains "blocked", `ready` if it contains "ready", and `other` otherwise.

```sh
python3 - <<'PY'
import sys
def classify(s):
    s = s.lower()
    if 'conditional' in s: return 'conditional'
    if 'closed' in s: return 'closed'
    if 'historical' in s or 'superseded' in s: return 'historical'
    if 'authorable' in s or 'writable' in s: return 'authorable'
    if 'blocked' in s: return 'blocked'
    if 'ready' in s: return 'ready'
    return 'other'
rows = {}
for i, line in enumerate(open('plans/registry.md'), 1):
    if not line.startswith('|'):
        continue
    cells = [c.strip() for c in line.strip().strip('|').split('|')]
    if len(cells) < 4 or set(''.join(cells)) <= set('-: ') or cells[0] == 'Subsystem':
        continue
    rows.setdefault((cells[0], cells[1]), []).append((i, classify(cells[2])))
bad = {k: v for k, v in rows.items() if len({c for _, c in v}) > 1}
for k, v in sorted(bad.items()):
    print('CONFLICT', k, v)
print('conflicting milestone states:', len(bad))
sys.exit(1 if bad else 0)
PY
```
Expected after WP3: `conflicting milestone states: 0` and exit status 0. On the baseline this reports exactly one conflict — `('Acquisition transport', 'M010 eggup-acquisition 0.1.3 publication')` at rows 185 (closed) and 238 (ready). It cannot see the prose and graph layers, which is why (b) exists.

*(b) Targeted cross-surface greps.* Each of the reconciled contradictions that lives outside a milestone row — the execution graph, the "Current project state" prose, and the milestone's own row — is asserted directly. Each command must return **empty** after WP3:

```sh
grep -nE 'M010 eggup-acquisition 0\.1\.3 publication[^|]*\|[^|]*ready' plans/registry.md
grep -nE 'M010 eggup-acquisition 0\.1\.3 publication \[READY' plans/registry.md
grep -nE 'M004 Gregg.*blocked' plans/registry.md
grep -nE 'M007 EggPool selective.*blocked' plans/registry.md
grep -nE 'remains intentionally unwritten until Acquisition M010' plans/registry.md
grep -nE 'once workspace 0\.1\.3 exists' plans/registry.md
```
On the baseline these hit `:238`, `:367`, `:266`, `:270`, `:392`, and `:393` respectively — the six contradictions recorded in §3.3. After WP3 every one is empty.

*(c) The authoritative-table statement.* The registry must name which table is authoritative:

```sh
grep -nE 'authoritative' plans/registry.md
```
Expected: at least one hit, in the sentence added by WP3, naming the `Dependency-ready implementation work` table.

**Root changelog contradiction assertion (assertion 4).** The `Unreleased` section is checked at bullet granularity, because the re-listed work is one bullet per milestone and a line-based check drowns in bullet continuation lines. A crate published at `0.1.3` is recognised by its literal name or by its subsystem alias (`eggup-service` ↔ "Service", and so on).

```sh
python3 - <<'PY'
import re, sys
ALIAS = {'eggup-core':'Core','eggup-acquisition':'Acquisition','eggup-eggfetch':'Eggfetch',
         'eggup-curl':'Curl','eggup-archive':'Archive','eggup-eggpack':'Eggpack','eggup-service':'Service'}
text = open('CHANGELOG.md').read()
head, unreleased = text.split('## Unreleased', 1)
unreleased = unreleased.split('\n## ', 1)[0]
norm = ' '.join(unreleased.split())
published = sorted(set(re.findall(r'`(eggup-[a-z]+) 0\.1\.3`', head)))
bad = []
for s in ('Published baselines are `0.1.2`', 'No publication milestone authorizes these entries yet'):
    if s in norm:
        bad.append('header: ' + s)
for b in [x for x in re.split(r'\n(?=- )', unreleased) if x.startswith('- ')]:
    names = [c for c in published if c in b or (ALIAS.get(c) and ALIAS[c] in b)]
    if not names:
        continue
    why = 'published-as-unpublished' if re.search(r'\bunpublished\b', b, re.I) else 'published-with-no-state-marker'
    bad.append(why + ': ' + ', '.join(names) + ' | ' + b.split('\n')[0][:60])
for x in bad:
    print('CONFLICT', x)
print('changelog Unreleased conflicts:', len(bad))
sys.exit(1 if bad else 0)
PY
```
Expected after WP1: `changelog Unreleased conflicts: 0` and exit status 0. On the baseline it reports **15** conflicts and exits 1: the two false header sentences, and all **13** bullets in the `Unreleased` section — 7 that describe published work as unpublished and 6 that name a published crate with no state marker at all.

**Registry readability assertion (Finding E).**

```sh
awk '{ print length($0)": "NR }' plans/registry.md | sort -rn | head -8
```
Expected: the maximum line length is at or below the ceiling recorded in the WP4 closure evidence, and `plans/registry.md:231` is no longer the longest line by a 4,000-character margin.

**No production delta (assertion 7).**

```sh
git diff --name-only <baseline>..HEAD
git diff --check
```
Expected: no `crates/*/src/**` path, no `Cargo.lock`, no `crates/*/Cargo.toml` (Finding G is deferred), and no whitespace errors.

**The regression script itself.**

```sh
./scripts/check-docs-truth.sh
```
Expected: exit status 0 on the corrected tree, and a **non-zero** exit status when run against the baseline tree (§10).

## 12. Documentation updates

The milestone **is** documentation, so this section states the doc-change chain explicitly per the `docs-hygiene` skill: *code/script/workflow → the owning doc → root README + `docs/` → `AGENTS.md` → skills → registry + roadmap status.*

| Touched file | Owning contract for the fact | Drift-table row |
|---|---|---|
| `CHANGELOG.md` | the root aggregates; per-crate changelogs own the per-crate release notes | `:752` corrected in place |
| `docs/crates.md` | `crates/*/Cargo.toml` (versions, pins) and `docs/releases.md` (registry state) | new row if residual remains |
| `docs/quickstart.md` | `crates/eggup-core/Cargo.toml`; the snippet stays a verbatim copy of `crates/eggup-core/examples/one_member.rs` | new row if residual remains |
| `docs/releases.md` | crates.io, verified per `docs/releases.md:3-8`; the prose points at `architecture/tooling-governance.md` §8 | new row if residual remains |
| `README.md` | `crates/*/Cargo.toml` and `plans/registry.md`; README points rather than restates | new row if residual remains |
| `AGENTS.md` | `crates/eggup-eggpack/Cargo.toml` owns the pin; AGENTS links to the deep dive | new row if residual remains |
| `architecture/overview.md` | `crates/*/Cargo.toml` | new row if residual remains |
| `architecture/tooling-governance.md` | the deep dive owns the drift table, the gate description (§4–§5), and the pin/cascade rules (§2.4, §8) | `:752` corrected, `:770` closed, `:767`/`:769` premise corrected, two new Open rows |
| `architecture/transport-footprint.md` | `crates/eggup-transport-footprint/Cargo.toml` + the three bin sources own the crate contract | new Open row for the CI-coverage gap |
| `plans/registry.md` | the registry owns open work, blockers, next transitions (`plans/registry.md:412-414`) | `:770` closed |
| `scripts/check-docs-truth.sh` (new) + `scripts/check-local.sh` | the gate owns verification commands; `architecture/tooling-governance.md` §4–§5 owns their rationale and coverage gaps | new row if residual remains |
| `plans/closure/planning-closure-hygiene-corrective/013-status.md` (new) | closure records own closure status | — |

**Chain rules stated for this milestone.** `docs/` edits **point at** contracts and do **not** restate them; no normative rule is introduced under `docs/`. `AGENTS.md` and `README.md` remain thin indexes that link to the owning doc rather than restating a version twice. The per-crate `CHANGELOG.md` files are **not** touched — they were individually reviewed and found correct, and churning correct per-crate files would create drift rather than remove it. `plans/000`–`003` and the accepted ADRs are **not** touched. The root `CHANGELOG.md` and the per-crate changelogs are reconciled together in the sense that the root's `Unreleased` is emptied of work the per-crate changelogs record as published — the root stops contradicting them rather than diverging from them. No skill file needs a change: `planning-workflow`, `docs-hygiene`, `verify-workflow` and `release-workflow` state no version literal this milestone falsifies; if any is found to, it is corrected under WP2 with the same precedence rule.

## 13. Acceptance criteria

C013 closes when **all** of the following are checkable:

1. `CHANGELOG.md`'s `## Unreleased` contains no entry describing work the `0.1.3` section records as published, and its header states the current published baseline rather than an unpublished one.
2. `docs/crates.md` states `0.1.3` as the workspace version and for every crate published at `0.1.3`; `docs/crates.md:59` (`eggup-curl` at `0.1.2`) is unchanged.
3. `docs/quickstart.md:18` and `README.md:18` name `eggup-core = "0.1.3"`.
4. `docs/releases.md`'s limitations section describes the versions actually published; the correct historical `0.1.2` references at `:16-23`, `:31`, `:112-113` are unchanged.
5. `README.md`'s publication-state paragraph contains no duplicated crate list and states `eggup-service 0.1.3` correctly.
6. No `=0.1.2` pin claim survives in `AGENTS.md`, `architecture/overview.md`, `architecture/tooling-governance.md` or `docs/`, except the allowlisted historical rows.
7. `plans/registry.md` states exactly one status for Acquisition M010, Gregg M004 and EggPool M007, and no milestone key resolves to two different status classes anywhere in the file.
8. The `Planned / blocked work` table contains only open, blocked, authorable or future rows; the 16 duplicated `(subsystem, milestone)` keys and the double `Eggpack M004` row are gone; the registry states which table is authoritative.
9. The registry's longest line is at or below the ceiling recorded in the WP4 closure evidence; no 4,000-character cell remains.
10. `architecture/transport-footprint.md` no longer claims that the `required-features` split makes an accidental cross-link a build error; it states the accurate guarantee and names the missing CI coverage and the follow-on that owns it.
11. The transport-footprint counts still read 3 / 92 / 93 from the §11 commands.
12. `crates/eggup-core/Cargo.toml` is unchanged, `cargo package -p eggup-core --list` still omits `crates/eggup-core/docs/`, and the drift table names Verified Update Core M016 as the owner of that gap.
13. The drift table has no row whose claim contradicts the corrected surfaces; the stale Resolved row at `:752` is corrected and the Open row at `:770` is closed and removed.
14. `scripts/check-docs-truth.sh` exists, is wired into `scripts/check-local.sh`, **fails on the baseline** and **passes on the corrected tree**, and `architecture/tooling-governance.md` §4–§5 describes the added gate line.
15. No file under `crates/*/src/**` changed; no `Cargo.lock` changed; no dependency was added; no version was published; no tag moved.
16. `docs/` contains no normative rule introduced by this milestone.
17. No closure record was deleted, and every superseded predecessor is marked historical or superseded in the registry rather than removed from the closure set.

## 14. Stop conditions

Stop and escalate to the maintainer — do not improvise a decision — if any of the following holds:

1. **The registry's duplicated rows cannot be reconciled without an explicit decision the maintainer must make.** §6.3 states the decision this plan makes (the `Dependency-ready implementation work` table is authoritative; the `Planned / blocked work` table is narrowed). If the maintainer rejects it, C013 stops and the contradiction is recorded as an open finding naming the exact conflicting rows.
2. **Finding G's manifest change turns out to require a repackage or a republication.** §6.6 defers it. If a reviewer concludes the `exclude` change must land for the milestone to be coherent, that is a scope change needing a publication milestone (Verified Update Core M016), not a docs pass. Stop rather than publish.
3. **A reconciliation would require editing `plans/000`–`003` or an accepted ADR.** Those are normative; changing them is its own milestone. Stop.
4. **A version claim cannot be resolved against the registry.** If a `0.1.N` literal is not derivable from `plans/registry.md`, `docs/releases.md`, `crates/*/Cargo.toml` or crates.io, it is not corrected by guessing. Stop and record it.
5. **The transport-footprint counts do not reproduce as 3 / 92 / 93** after the doc correction. Stop: that would mean the measured block or the tree changed, and the milestone may change neither.
6. **A per-crate `CHANGELOG.md` is found inconsistent** with the root after the root is corrected. The per-crate files were reviewed and found correct; if that review was wrong, WP1's scope changes and needs the maintainer to re-scope it.
7. **The regression script cannot be made to fail on the baseline.** An assertion never observed to fail is not evidence. Stop and fix the assertion before landing it.
8. **A concurrent publication lands** while C013 is in flight. Re-read the registry and `docs/releases.md`, re-derive the version claims, and re-run §11 before continuing.

## 15. Closure evidence required

`plans/closure/planning-closure-hygiene-corrective/013-status.md` must contain, per `plans/closure/README.md`:

- **Status**, source plan, source roadmap-equivalent (the registry plus `plans/003-planning-process.md`), reviewed repository baseline (`c4805411f74b85158e0ae5d0a219a32c4209d98e`), and implementation commit(s).
- **Executive finding** — one paragraph stating what was false and what is now true.
- **Requirement → evidence matrix** — one row per acceptance criterion in §13, with the exact command and the exact observed output.
- **Production implementation evidence** — for this milestone the **zero-production-delta proof** (`git diff --name-only <baseline>..HEAD` shows no `crates/*/src/**`, no `Cargo.lock`, no `crates/*/Cargo.toml`), plus the `git diff --check` result.
- **Exact commands and results** — every command in §11, with its actual output. For each, the **observed** value, not an expected one. A command that was not run is recorded as `not run`, never inferred.
- **Per-file before/after evidence** for every corrected claim in §6 — the before-text and the after-text, cited by `file:line` on both sides. This is the core of the closure record for a truth-reconciliation milestone: a claim without a before/after pair is not reconciled.
- **Platform reporting.** Every command used by this milestone is platform-independent: the `cargo tree` resolution counts, `cargo package --list`, `cargo test --workspace --all-targets --all-features --locked`, `./scripts/check-local.sh`, and the `grep`/`awk`/`python3` text checks all produce the same result on Linux, macOS and Windows. The closure record states this **once, plainly**, naming the commands and why they do not vary — it does **not** manufacture three identical platform sections, and it does **not** claim a Windows or Linux run that was not performed. The one output that could differ by platform is the *compiled* form of the `cargo tree`/`cargo package` commands; the counts asserted (3 / 92 / 93) are dependency-graph counts, not binary sizes, and the binary-size figures in `architecture/transport-footprint.md:98-100` are **not** re-measured by this milestone and are not asserted on any platform.
- **Invariant review** — §4 checked item by item, each with its evidence.
- **Failure / rollback / recovery review** — §8's precedence rule, the retained-closure-record rule, and the fail-closed behavior of each assertion.
- **Compatibility and migration review** — §9: no published artifact changed, no runtime behavior changed, the only packaging question (Finding G) deferred with its named owner.
- **Security review** — no authenticity, signature or trust claim is introduced, removed or altered by any correction. The Eggpack member-identity cross-check remains described as published in `0.1.3`, never as unpublished. The SHA-256-is-integrity-only framing (`AGENTS.md`, `architecture/overview.md`) is unchanged.
- **Documentation / operations evidence** — the doc-change chain table from §12, with the drift-table rows added and removed named.
- **Unresolved findings with severity** — every finding this milestone did **not** close, each with a severity and an owner. At minimum: the transport-footprint CI-coverage gap and the `eggup-core` contract-doc packaging gap, plus the `=0.1.0` pin literals already tracked at `architecture/tooling-governance.md:767`/`:769`.
- **Roadmap disposition** — this workstream has no subsystem roadmap; the record states that and names the registry entry and the two follow-on milestones instead.
- **Registry updates** — the C013 row, the new open-findings entry, and the resulting table shapes.

## 16. Handoff notes

**What C013 authorizes.** Documentation truth only: a corrected root changelog, a corrected version surface, a registry that states one status per milestone, a registry that can be scanned, a transport-footprint deep dive that states what the CI actually proves, and a drift table that is live.

**What C013 does NOT authorize.** No implementation of any kind — no `src/` change, no new dependency, no manifest change, no new feature, no new CI lane. No publication, repackage or tag movement. No authoring of the transport-footprint CI-coverage milestone and no creation of a `plans/subsystems/transport-footprint-roadmap.md`. No authoring of Verified Update Core M016 (Finding G). No edit to `plans/000`–`003` or to any accepted ADR. No new rule in `docs/`. No change to the measured transport-footprint figures 3 / 92 / 93. No deletion of a closure record. No edit to any per-crate `CHANGELOG.md` (all seven were reviewed and found correct).

**Named follow-on 1 — transport-footprint CI coverage.** The crate's entire purpose is the isolated single-adapter configurations, and **no CI job builds them**: `scripts/check-local.sh:5` and `.github/workflows/ci.yml:20`, `:21`, `:61`, `:71` use `--all-features` only, and no `--no-default-features --features curl` / `--features eggfetch` invocation exists anywhere in the workflow. The crate also has 3 binaries, 45 source lines and **zero** tests, and the binaries perform no measurement in code.

- **Prerequisite:** a **subsystem roadmap decision**. `eggup-transport-footprint` has no roadmap in `plans/subsystems/`, and the repo's rule is to create one "when ownership, dependencies, or closure criteria are meaningfully distinct" (`plans/subsystems/README.md`). Whether the footprint fixture earns its own roadmap, or is folded into `acquisition-transport` (whose seam it consumes) as a milestone there, is a maintainer decision that precedes any milestone authoring.
- **Proposed milestone, subject to that decision:** *Transport Footprint M001 — isolated single-adapter build coverage*, which would add the two `--no-default-features --features curl` / `--features eggfetch` builds to CI, add a measurement that lives in code rather than in a doc's prose table, and close the finding recorded by C013's drift-table row. C013 does not write it.

**Named follow-on 2 — `eggup-core` contract-doc packaging (Finding G).** **Verified Update Core M016 — `eggup-core` contract-doc packaging**, owned by `plans/subsystems/verified-update-core-roadmap.md`. It must remove `"docs/"` from `crates/eggup-core/Cargo.toml`'s `exclude` list, qualify `cargo package -p eggup-core --list` to show `crates/eggup-core/docs/{domain,verification,transaction}.md`, and authorize and execute the `eggup-core` republication that actually delivers those contracts to crates.io and docs.rs consumers. Until M015 lands, the authoritative contracts remain readable only from the repository. C013 records the finding and the owner; it does not write M015's plan.

**Standing rule for the next reader.** The registry is the control surface; `docs/` is non-normative and points at the contracts; crates.io decides what exists. When two of them disagree, the registry wins, and the disagreement is recorded rather than averaged away.
