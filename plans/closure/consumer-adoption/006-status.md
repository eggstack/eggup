# Consumer Adoption M006 — Execution Status

Disposition: **closed** (consumer implementation landed in Egress; see M003-landed addendum below)

Source plan: `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`

Source roadmap: `plans/subsystems/consumer-adoption-roadmap.md`

Cross-repository owner: `eggstack/eggress: plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md` (per the Eggup plan; see Blocker evidence below).

This record closes the current bounded Eggup-side execution pass and preserves its evidence. It does not claim M006 acceptance: no Egress consumer code was changed, no Eggup dependency was cut over, and no publication was performed. Both omissions are required by the plan itself, not gaps in this pass.

## Eggup-side gate progress (the only in-repo M006 work)

The plan's sole Eggup-owned prerequisite was the versioned core/archive package boundary (Package M008). That gate is now qualified:

- Core M008 closed at `plans/closure/verified-update-core/008-status.md`: workspace `0.1.2`, `cargo package` + `cargo publish --dry-run` green for `eggup-core` and `eggup-archive`, clean external fixture proving the M001d bound-source flow, explicit manual publish order (core before archive).
- Eggpack Interop M002 closed at `plans/closure/eggpack-manifest-interoperability/002-status.md`: archive-projection → `ArchivePlan` → bound-source staging helpers qualified; direct Egress-side archive flow consumes `eggup-archive` per its own plan.

No Eggup production change belongs to M006, and none was made: the plan's handoff notes assign all consumer code changes to `eggstack/eggress`.

## Blocker evidence

One stop applies (C009 correction, 2026-09-28: the earlier second stop below was
based on a stale Egress checkout and is retracted):

1. **Publication gate (plan Section 14).** "Stop before dependency merge if Package M008 is not published in a crates.io-usable form." M008 is qualified but unpublished: publication is a separate maintainer action and this pass performs no publication. A merged/publishable Egress dependency therefore still cannot cut over. Pre-merge qualification against an immutable Eggup revision remains permitted by work package A.

~~2. **Consumer-premise absence in the available checkout.**~~ **Retracted by C009.**
The earlier revision of this record claimed the inspected Egress checkout
(`/Users/davidbowman/projects/eggress`, head `6b0edc2`) had no updater surface
and no mirrored delivery plan. That checkout was stale. Current
`eggstack/eggress` at `03134f8ce476e4935dde7ae81d7ddb12b924bc7e`
(`origin/main`, fetched 2026-09-28) contains both:

- updater surface: `crates/eggress-cli/src/update/mod.rs` (self-update flow),
  `crates/eggress-cli/src/update/install.rs` (`extract_archive`, `replace_pair`,
  `copy_or_rename`), plus `download.rs`, `target.rs`, `verify.rs`, `version.rs`;
- registered consumer-owned delivery plan:
  `plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md`
  (status: blocked on versioned Eggup core/archive package availability),
  registered in the Egress delivery roadmap, registry, and canonical
  `docs/ROADMAP.md`.

Work packages B–F therefore have a consumer surface to attach to in Egress;
implementation belongs to `eggstack/eggress` Delivery M003, gated only on the
publication stop above.

## Requirement disposition

| Requirement | Result / evidence |
|---|---|
| Versioned Eggup core/archive package availability (§2 gate) | Satisfied 2026-09-28 by M009 closure: `eggup-core 0.1.2` + `eggup-archive 0.1.2` registry-visible (see addendum below). Registry dependency cutover is now permitted. |
| Egress adapter / pair-transaction replacement (§6–§7, packages B–E) | Not started in Eggup (correct: consumer code belongs in Egress). Consumer surface exists at `eggstack/eggress@03134f8` (`crates/eggress-cli/src/update/`, Delivery M003 registered); implementation is Egress-owned. No Egress production source or dependency was changed from this repo. |
| Local helper deletion after parity (§5, package E) | Not applicable; nothing to delete. |
| Cross-platform/package/size qualification (§7F, §10–§11) | Not run; Egress verification commands belong to the consumer repo and require the consumer implementation to exist. |
| CLI exit-code / release-policy preservation (§4) | Not applicable; no consumer behavior changed. |
| Closure evidence: Eggup version, parity matrix, fault injection, package evidence (§15) | Partially satisfiable from the Eggup side (version `0.1.2`, M008 package evidence) but the consumer-owned rows cannot be produced here. |

## Invariant review

All §4 invariants hold vacuously: nothing in Egress was touched, so no release/checksum/version/CLI policy could regress. The Eggup-side M008/M002 changes that M006 depends on were reviewed in their own closures with no medium-or-higher finding.

## What unblocks M006

1. Maintainer publishes `eggup-core 0.1.2` then `eggup-archive 0.1.2` to crates.io (M008 manual order), satisfying the §14 merge gate.

Egress already registers its delivery plan (Delivery M003 at
`eggstack/eggress@03134f8`) and presents the updater surface
(`crates/eggress-cli/src/update/`) the Eggup plan contracts against; no
contract revision is needed on that axis (C009 correction 2026-09-28).

Until the publication gate holds, M006 remains planned/blocked. No further
corrective is opened: the remaining stop is an external gate, not an Eggup
defect.

## M009-closure addendum — publication gate satisfied (2026-09-28)

Verified Update Core M009 is closed (`plans/closure/verified-update-core/009-status.md`):

- `eggup-core 0.1.2` published 2026-09-28T21:53:21Z (registry id `3351376`,
  checksum `0f44129c…cc8e6`), then `eggup-archive 0.1.2` published
  2026-09-28T21:53:39Z (registry id `3351378`, checksum `758e9564…1c8292`),
  both from release-prep commit `e8e07eb`, tagged `v0.1.2` with GitHub
  Release `0.1.2`.
- A registry-only external fixture resolved both `=0.1.2` crates from
  crates.io (no path/git overrides) and passed the tar.gz + zip
  bound-source flows plus the mismatch rejection (3/3).

The §14 merge gate ("Package M008 published in a crates.io-usable form") is
therefore satisfied. M006 moves from publication-blocked to executable:
Egress Delivery M003 may now cut over to registry dependencies. M006 itself
is not closed — work packages B–F remain Egress-owned implementation, and
their evidence belongs to the consumer repo.

## M003-landed addendum — consumer adoption complete (2026-09-29)

Egress Delivery M003 is closed (`eggstack/eggress`
`plans/closure/delivery/003-status.md`, implementation head `19e6dc7`).
This satisfies every remaining M006 acceptance row from the Eggup side:

- **Registry cutover**: Egress `eggress-cli` depends on `eggup-core = "=0.1.2"`
  + `eggup-archive = "=0.1.2"` from crates.io (lockfile registry sources,
  M009 checksums); no git/path dependency in the publishable workspace.
- **Generic mechanics deleted**: `extract_archive` (tar/PowerShell shell-out),
  `replace_pair` (bespoke backup/rollback), `copy_or_rename`, and
  `make_executable` removed; only Eggress-specific path policy remains.
- **Bound-source flow**: verified archive → bounded extraction → object-bound
  sources → `prepare_with_bound_sources` → staged exact-version checks →
  Eggup commit/rollback with truthful disposition mapping; member
  size/digest expectations stay `None` per the release contract (no member
  manifest; whole-archive SHA-256 gate plus extraction-computed evidence).
- **Policy preserved**: GitHub authority, target mapping, checksum sidecars,
  exact staged-version agreement, CLI exit codes, sibling-pair requirement,
  0755 installed modes, no Cargo fallback, no elevation.
- **Fault evidence**: all-disposition mapping tests, pre-mutation failures
  with untouched installs, lock-contention failure, upstream fault-injection
  qualification; mid-commit e2e rollback unreachable via the public API
  (recorded upstream-consumer limitation, not an M006 gap).
- **Package/size**: clean-tree `cargo package -p eggress-cli` (45 files,
  verify green); release `eggress` +2.8% (+270,864 B), `pproxy` unchanged.
- **Qualification**: Rust 1.89; hosted run `36639694985` green on Rust smoke
  (full workspace) plus updater lanes on Linux, macOS, and Windows executing
  the updater/archive path.

M006 is closed. No further Eggup work is unblocked by this closure beyond
what M009 already opened; remaining Eggup plans keep their own gates
(Gregg M004 intentionally unwritten, Eggpack M003/M004 producer-blocked,
EggPool M007 deferred).

## Current-state addendum — 2026-10-01 (planning-hygiene C010)

The "Eggpack M003/M004 producer-blocked" line above recorded the active state when this M003-landed addendum was written (2026-09-29). The producer-evidence gate was correct at that time and was resolved externally on 2026-10-01 by Eggpack Ecosystem M001 and Eggsact Distribution M005, which closed on the real `v1.2.7` producer contract (`release/eggpack/distribution.toml` authority for the unversioned asset names; `release-manifest.json` published alongside the binaries, checksum sidecars, and installers). M003 consumer-path adoption is now ready to resume under the existing implementation plan after baseline refresh; M004 promotion remains blocked on M003 real-consumer closure plus a publishable `eggpack-manifest` version. See `plans/closure/planning-closure-hygiene-corrective/010-status.md` and the 2026-10-01 addendum in `plans/closure/eggpack-manifest-interoperability/003-status.md`.
