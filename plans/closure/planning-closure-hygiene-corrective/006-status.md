# Planning / Closure Hygiene Corrective C006 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/planning-closure-hygiene-corrective/006-m001d-readiness-and-bound-source-contract-reconciliation.md`

Related runtime work:

- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md` (corrected by this pass; ready for handoff)
- `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`
- `plans/closure/archive-extraction/001c-status.md` (Section 14 stop; continued by M001d)
- `plans/closure/planning-closure-hygiene-corrective/005-status.md` (prior gate reconciliation)

Reviewed repository baseline: `f463aa79398a0b1893eb117dc639d2f262fe1b86` (pre-implementation head)

Implementation commits:

- This closure batch (docs-only; the commit SHA is visible in `git log` at commit time — built directly on `f463aa7`).
- Referenced (not re-implemented here): M001c write authority `09c953fe1fe512717584bb3e5509e79da894b35a`; M001c/C005 closure and M001d registration `f7a18e4762bd0549349eaa6f014b0649f36594c4`.

Hosted qualification: not required — C006 changes no runtime source, Cargo manifests, Cargo.lock, workflow logic, consumer repos, release artifacts, or package versions (docs-only per source plan Section 11). Runtime qualification is carried over: M001c hosted write-authority run `36257884083` passed Stable/MSRV/macOS/Windows, and current-head CI run `36257992801` also passed at documentation head `0d2f1f0`; every commit between `0d2f1f0` and the reviewed baseline `f463aa7` is planning/registry/roadmap text only.

## Executive finding

C006 corrects all four planning defects in the M001d implementation plan before runtime work begins, and returns M001d to ready-for-handoff:

```text
M001 -> M001a historical -> M001b cleanup [CLOSED]
                                  |
                                  v
                         M001c write half [IMPLEMENTED] + handoff [STOPPED]
                                  |
                                  v
                         C006 planning correction [CLOSED by this record]
                                  |
                                  v
                         M001d handle-backed handoff [READY, corrected]
                                  |
                      +-----------+-----------+
                      v                       v
               Egress M006              Eggpack M002
                 [BLOCKED]                 [BLOCKED]
```

Concretely:

1. M001d baseline `1388a02...` (predating M001c `09c953f`, the M001c Section 14 stop, hosted run `36257884083`, and doc head `0d2f1f0`) is rebased to `f463aa7`, with the carried runtime qualification recorded in the plan.
2. The permitted-but-unsound `root handle + member name -> later open_at(root, name) -> stage` shape is removed from the accepted authority shapes and explicitly forbidden as final authority; only object-bound shapes qualify (moved open member object, immediate object-bound copy into core-owned stage, or a proven equivalent with no name lookup after the handoff boundary).
3. Section 9 no longer implies removing/deprecating the path-source constructor: `ArtifactMember::new(id, source_path, destination)` and its semantics are preserved for non-archive callers, and M001d is additive (a narrow bound-source type/constructor/helper plus auto-trait accounting).
4. Readable-handle creation (`read(true)` at atomic create-new/no-follow time, or transfer from the already-open readable object — never reopen by name), cursor ownership (`File::try_clone` shared-offset semantics, single-owner move plus flush/seek-to-zero, non-zero-cursor test), and handle lifetime/cleanup ordering (transfer → consume/close → cleanup may empty root; deterministic ownership; Windows cleanup-after-consume tests; no pathname-recursive fallback) are now explicit, with the post-extraction member-entry replacement race required alongside root rename/replacement races.

## Requirement-to-evidence matrix

| Requirement (source plan Section 10 / 13) | Evidence | Result |
|---|---|---|
| M001d baseline is corrected | plan header now records baseline `f463aa7` plus carried qualification (`09c953f`, `f7a18e4`, runs `36257884083` + `36257992801`); Section 2 hard dependencies updated | passed |
| M001d no longer treats root-handle + later-name reopen as object-bound | Section 3 target flow + Section 6.1 forbid `open_at(root, name)` as final authority with the member-entry-replacement rationale; only shapes A/B/C qualify | passed |
| M001d explicitly preserves ordinary path-source APIs | rewritten Section 9: `ArtifactMember::new` kept, no migration/deprecation for non-archive consumers, additive seam preferred, auto-trait (`Clone`/`Eq`/`Sync`) accounting required | passed |
| M001d specifies readable authority without reopening a name | new Section 6.1a: `read(true)` at creation or object-bound transfer, flush + seek to zero, never reopen by path/name | passed |
| M001d documents `File::try_clone` shared-cursor behavior | new Section 6.1b: shared file-description semantics, single-owner move, clone only with proof, no unsafe cloneable API | passed |
| M001d requires seek-to-zero/full-byte staging evidence | Section 6.1b + Section 10: non-zero cursor before staging, full file staged from byte zero | passed |
| M001d defines handle lifetime/cleanup ordering | new Section 6.1c + Section 6.2/6.4: transfer → consume/close → cleanup ordering, deterministic ownership, Windows tests, no pathname fallback | passed |
| M001d requires member-entry replacement-after-extraction race coverage | Section 6.3 + Section 10 + Sections 13/15: 5-step deterministic race (extract, retain object, replace entry, stage, prove owned bytes + foreign untouched) | passed |
| Egress/Eggpack remain blocked | registry (top metadata, tables, graph, state, handoff), archive roadmap M002/M003 rows, unchanged consumer/Eggpack roadmaps all say blocked on M001d | passed |
| No runtime/Cargo/workflow file changes in C006 | `git diff --name-only f463aa7` lists only the five planning files below; grep for runtime extensions confirms zero delta | passed |

## Production implementation evidence

None — C006 is documentation/evidence reconciliation only. Explicit zero-runtime-delta statement: no `.rs` file, `Cargo.toml`, `Cargo.lock`, workflow, consumer repo, release artifact, or package version is modified by this batch.

Exact files changed by this batch:

- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md` (corrected: baseline, Sections 2/3/4/6.1–6.4/9/10/12/13/15/16, status → ready for handoff)
- `plans/implementation/planning-closure-hygiene-corrective/006-m001d-readiness-and-bound-source-contract-reconciliation.md` (status only)
- `plans/subsystems/archive-extraction-roadmap.md` (M001d blocked-on-C006 → ready)
- `plans/registry.md` (C006 ready → closed; M001d blocked → ready; heads/metadata/graph/state/handoff reconciled)
- `plans/closure/planning-closure-hygiene-corrective/006-status.md` (this record)

Consumer (`consumer-adoption-roadmap.md`) and Eggpack (`eggpack-manifest-interoperability-roadmap.md`) roadmaps required no edits: both already gate on Archive M001d with accurate M001c-stop wording, and the source plan (Section 12) calls for edits only if that wording is stale.

## Exact commands and results

Documentation-only verification (C006 source plan Section 11):

```text
git diff --check                                                passed (exit 0, no whitespace errors)
git diff --name-only f463aa7..HEAD                              lists only the five planning files above (four tracked modifications + new `006-status.md`)
rg -n "001d|M001d|bound source|open_at|try_clone|cursor|Egress M006|Eggpack.*M002" plans/   inspected; corrected M001d carries the new authority/compat/cursor/lifetime language, all active gates say M001d ready with Egress M006 / Eggpack M002 blocked on M001d, no stale blocked-on-C006 remains on any active surface
```

A Cargo check is not required for C006 because it is documentation/planning only (source plan Section 11). No new runtime matrix is required; the carried M001c write-authority matrices are recorded in `plans/closure/archive-extraction/001c-status.md` (local green; hosted run `36257884083` green).

## Invariant review

- Historical closure evidence stays historical: M001b/M001c/M008 records are untouched; only M001d's forward-looking plan text is corrected.
- M001c remains represented as implemented-write + stopped-handoff continued by corrected-ready M001d — never silently folded, never claimed closed.
- Downstream archive consumers stay blocked until M001d closes with hosted qualification.
- `eggup-core` archive-format independence, SHA-256-as-integrity-only, no-unsafe/MSRV 1.89, and foreign-preserving cleanup postures are restated unchanged in the corrected plan.
- Registry stays compact and defers detail to linked plans.

## Failure/restart/contention review

C006 has no runtime failure/restart semantics. The corrected M001d plan now makes the future semantics explicit per source plan Section 8: handle/read/seek failure fails closed; failed stage copy never reopens the path as fallback; foreign replacements are never removed; cleanup waits through ownership sequencing rather than retry loops; private residue is preferred to unsafe fallback; no hidden bound-to-path downgrade. Planning risk reviewed: if the M001d implementation later discovers the additive seam cannot coexist with existing path consumers, or Windows cannot retain/transfer the same opened member object under Rust 1.89, the plan's Section 14 stop conditions already require a broader ADR/API plan instead of a name-reopen workaround.

## Compatibility and migration review

No API, dependency, binary, package, or consumer behavior change in C006 itself. The corrected M001d posture is additive: existing path-source constructors stay, the bound-source path is new and archive-scoped, and any `File`-ownership auto-trait effects must be recorded and confined to the new path. No consumer migration is authored or performed; Egress M006 and Eggpack M002 re-gate on M001d closure unchanged.

## Security review

No runtime attack surface changes. The security-relevant effect is prevention: the plan can no longer be implemented as a root-handle + later-name reopen that a concurrent actor could defeat by replacing the member entry after extraction — the exact confusion the stale plan permitted is now forbidden text. With Egress M006 and Eggpack M002 still gated on M001d closure, no consumer can integrate against the weaker authority in the meantime.

## Documentation and operations evidence

- Corrected M001d plan, C006 plan status, archive roadmap, registry, and this closure record updated as listed above.
- Per-platform results: not applicable (docs-only; no platform-sensitive behavior changed). Windows-specific acceptance evidence is now a required M001d implementation deliverable (Section 6.1c/10), not a C006 deliverable.

## Unresolved findings

- None for C006. Residuals owned elsewhere: the M001c medium handoff finding (stale recorded pathname) → M001d (corrected, ready); M001c hosted write-authority evidence → carried (`36257884083`); M008/M001b → closed with no residuals above informational.

## Roadmap disposition

Planning/closure hygiene C006 moves from ready to closed. Future-plan triage (unblocking review):

- **Archive M001d**: blocked on C006 → **ready for handoff** (unblocked by this closure; corrected plan committed here).
- **Consumer Adoption M006 Egress**: **remains blocked** on M001d closure + green hosted qualification. No implementation plan exists yet (`—` in registry); authoring it now would violate the M001d gate, so no status change.
- **Eggpack Interoperability M002**: **remains blocked** on M001d closure + green hosted qualification. No implementation plan exists yet; same gate applies, so no status change.
- **Gregg M004**: writable but intentionally unwritten per separate authoring decision; C006 does not change its prerequisites or authorize migration, so no status change.
- **Eggpack M003/M004**: independently blocked on producer-owned conventions; unaffected by C006, so no status change.
- No other plan becomes dependency-ready as a result of C006.

## Registry updates

- C006: ready → closed.
- M001d: blocked on C006 → ready (corrected; M001c stop remains the runtime predecessor).
- Egress M006 / Eggpack M002: remain blocked on M001d (no change — still gated on runtime closure, not on planning).
- Top metadata: planning head `f463aa7` + this batch; runtime/doc head `0d2f1f0` with runs `36257884083` / `36257992801`; C005 + C006 closed.
- Execution graph, subsystem table, current-state, and next-handoff rows reconciled to the closed-C006/ready-M001d graph above.
