# Planning / Closure Hygiene Corrective C009 — Consumer M006 Egress Evidence Reconciliation

Status: implemented; closed by `plans/closure/planning-closure-hygiene-corrective/009-status.md`

Repository baseline: `f708e45ab82a35df740704f1af36a7d2275794cf`

Affected records:

- `plans/closure/consumer-adoption/006-status.md`
- `plans/subsystems/consumer-adoption-roadmap.md`
- `plans/registry.md`
- `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`

External source of truth:

- `eggstack/eggress@03134f8ce476e4935dde7ae81d7ddb12b924bc7e`
- `plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md`
- `crates/eggress-cli/src/update/mod.rs`
- `crates/eggress-cli/src/update/install.rs`

Primary class: polish/corrective

## 1. Objective

Correct Consumer M006's blocked execution record so it reflects the actual current Egress repository.

The record correctly identifies the unpublished Eggup 0.1.2 package gate, but incorrectly states that the inspected Egress checkout has no updater surface or mirrored delivery plan.

Current `eggstack/eggress/main` has both.

## 2. Readiness and dependencies

C009 is dependency-ready and docs-only.

It does not depend on M002a or M008a because the factual cross-repo correction is independent. M006 remains blocked on package publication regardless.

## 3. Current evidence and detection gap

Current Eggup M006 record says:

- no Egress updater surface exists;
- no mirrored Egress Delivery M003 plan exists.

Current Egress evidence contradicts that:

- `crates/eggress-cli/src/update/mod.rs` contains the real self-update flow;
- `crates/eggress-cli/src/update/install.rs` contains `extract_archive`, `replace_pair`, and `copy_or_rename`;
- `plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md` is registered;
- Egress delivery roadmap, registry, and canonical `docs/ROADMAP.md` register Delivery M003;
- Egress M003 is itself blocked only on compatible versioned Eggup core/archive registry packages.

The detection gap was a stale or incomplete consumer checkout during M006 execution review.

## 4. Invariants that must not regress

- M006 remains blocked until compatible 0.1.2 registry packages are actually available;
- do not imply Eggup can publish crates automatically;
- do not modify Egress from this corrective;
- do not reopen M001d/M008/M002 runtime architecture;
- preserve Egress-owned release/checksum/version/CLI policy;
- no Gregg migration is authorized.

## 5. Scope and non-scope

### In scope

- correct M006 blocked execution record;
- correct consumer roadmap and registry;
- record exact Egress commit/path evidence;
- reduce M006 blockers to the real remaining package-publication gate plus any later evidence discovered by Egress implementation.

### Out of scope

- implementing Egress Delivery M003;
- publishing Eggup 0.1.2;
- changing Egress code/plans;
- changing M006 runtime contract.

## 6. Required planning changes

Update M006 records to state:

- consumer updater surface exists;
- consumer-owned Delivery M003 exists and is registered;
- local duplicated mechanisms remain visible and are the intended migration target;
- 0.1.2 is qualified but unpublished;
- final Egress dependency cutover remains blocked until `eggup-core` and `eggup-archive` compatible 0.1.2 registry versions are available.

Do not call M006 "closed"; the existing file under `plans/closure/consumer-adoption/006-status.md` is an execution/block record, not successful closure.

## 7. Ordered work packages

1. Amend M006 execution record with exact Egress evidence.
2. Amend consumer adoption roadmap status/graph/table.
3. Amend registry current state, planned/blocked table, graph, and next handoff.
4. Verify Egress plan path and updater files remain present at the recorded commit.
5. Write C009 closure record.

## 8. Failure, restart, cancellation, and contention semantics

No runtime semantics.

If Egress changed concurrently, refetch and record the actual current commit rather than preserving stale evidence.

## 9. Compatibility and migration

No API or consumer migration.

## 10. Required checks

- no active Eggup planning text claims Egress lacks updater surface;
- no active Eggup planning text claims Egress lacks Delivery M003;
- M006 remains blocked on publication;
- exact Egress commit/path evidence is recorded;
- C009 diff is planning-only.

## 11. Verification commands

~~~text
git diff --check
rg -n "M006|Egress|updater surface|Delivery M003|0.1.2|publication" plans/
git diff --name-only <baseline>..HEAD
~~~

## 12. Documentation updates

- `plans/closure/consumer-adoption/006-status.md`;
- `plans/subsystems/consumer-adoption-roadmap.md`;
- `plans/registry.md`;
- `plans/closure/planning-closure-hygiene-corrective/009-status.md`.

## 13. Acceptance criteria

C009 closes when all active Eggup planning agrees that Egress has an updater and registered Delivery M003, while M006 remains blocked solely on the explicit versioned-package publication gate unless new concrete implementation evidence identifies another blocker.

## 14. Stop conditions

Stop if current Egress main no longer contains the cited plan/updater surface or if the package gate has already been satisfied by publication.

## 15. Closure evidence required

Record:

- Eggup docs commit;
- exact Egress commit;
- cited Egress paths;
- before/after blocker text;
- zero-runtime-delta file list.

## 16. Handoff notes

This corrective removes a false blocker; it does not remove the real operational gate. After M002a/M008a restore green package qualification and a maintainer publishes the compatible 0.1.2 pair, Egress Delivery M003 becomes the consumer-side implementation handoff.
