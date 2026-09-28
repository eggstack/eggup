# Consumer Adoption Milestone 006 — Egress Archive/Pair Adoption

Status: executable; M009 publication gate satisfied 2026-09-28 (`eggup-core 0.1.2` + `eggup-archive 0.1.2` registry-visible; see `plans/closure/verified-update-core/009-status.md` and the M009-closure addendum in `plans/closure/consumer-adoption/006-status.md`). Implementation belongs to Egress Delivery M003.

Repository baseline: `413a35a7da32ea22ef337caefc35d3619154794e`

Source roadmap:

- `plans/subsystems/consumer-adoption-roadmap.md`

Cross-repository implementation owner:

- `eggstack/eggress: plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md` (registered in Egress delivery roadmap, registry, and canonical `docs/ROADMAP.md`; blocked on versioned Eggup core/archive package availability).

Primary class: capability / adoption

## 1. Objective

Replace Egress's duplicated generic archive extraction and two-binary backup/rollback mechanics with Eggup's qualified archive extraction and multi-artifact transaction while preserving Egress-owned release policy and CLI behavior.

## 2. Why this milestone is ready

Archive M001d is closed and cross-platform qualified. Egress's existing updater contract is concrete and stable.

The only distribution gate is a crates.io-usable Eggup core/archive version; Package M008 owns that prerequisite and no git/path dependency may become part of Egress's publishable release state.

## 3. Current Egress evidence

Egress currently owns:

- GitHub latest/tag authority and target asset naming;
- curl-based archive/checksum download;
- sidecar SHA-256 policy;
- external tar/PowerShell extraction;
- staged `eggress` + `pproxy` version checks;
- bespoke same-filesystem backup/replace/rollback pair transaction;
- explicit no-Cargo-fallback and no-elevation behavior.

## 4. Invariants that must not regress

- GitHub release/version/target/origin policy stays Egress-owned;
- checksum mismatch remains hard failure;
- both candidates must report the exact expected release and agree;
- no partial successful pair update;
- old pair remains usable on pre-commit failure and is restored when rollback succeeds;
- no automatic privilege escalation or source/Cargo fallback;
- CLI exit-code contract remains unchanged;
- updater remains offline-fixture testable.

## 5. Scope

### In scope

- map the verified archive into `eggup-archive`;
- create a two-member Eggup install plan for sibling `eggress`/`pproxy`;
- use bound-source preparation;
- preserve staged candidate version execution before commit;
- replace bespoke pair transaction with Eggup commit/rollback;
- delete local generic extraction/backup helpers after parity qualification;
- dependency/size measurement.

### Out of scope

- release discovery rewrite;
- switching from curl unless separately justified;
- Eggpack manifests as a requirement;
- service lifecycle;
- changing bootstrap installers;
- changing public version strings or CLI exit codes.

## 6. Required production changes

Keep Egress acquisition/release policy through checksum verification. After verified archive acquisition:

1. declare exactly two archive members for the current target with finite limits. Current Egress releases verify the whole archive SHA-256 and do not publish member manifests; use optional member size/digest expectations as `None` rather than synthesizing trust from extracted paths, then retain Eggup's computed extraction evidence;
2. extract via `eggup-archive`;
3. build the Eggup core plan while advisory paths are valid;
4. transfer member open objects to bound sources;
5. prepare the two-member transaction;
6. run Egress's exact staged-version validation against the prepared staged paths;
7. classify existing sibling destinations as owned under Egress's installation identity;
8. commit through Eggup transaction machinery;
9. clean extraction residue after handles are consumed;
10. map Eggup terminal outcomes into existing Egress messages/exit codes.

Do not retain a second rollback implementation after parity is proven.

## 7. Ordered work packages

A. Add versioned Eggup dependencies after Package M008 publication; immutable revision may be used only for pre-merge qualification.

B. Introduce an Egress adapter around archive declarations/core plan mapping.

C. Preserve candidate/version verification on Eggup stage paths.

D. Replace pair mutation with Eggup transaction.

E. Remove local extraction/rollback helpers proven redundant.

F. Qualify cross-platform behavior, packageability, dependency/size delta, and docs.

## 8. Failure, restart, cancellation, and contention semantics

Before commit, failures leave installed pair untouched. Commit failures use Eggup rollback/recovery outcomes; `RecoveryRequired` must surface as a hard runtime failure with retained evidence, never as success.

No update retry/fallback to another source is added.

## 9. Compatibility and migration

The command remains `eggress update`; release authority and sidecar format remain unchanged. Existing installations need no persistent data migration.

The dependency cutover must not make Egress crates.io packaging invalid.

## 10. Required tests

- existing offline success fixture;
- checksum mismatch leaves old pair untouched;
- one missing archive member;
- optional member digest/size mismatch fixtures when explicit expectations are supplied;
- staged Egress wrong version;
- staged pproxy wrong version/disagreement;
- injected first/second-member commit failures with rollback;
- RecoveryRequired mapping;
- member-entry replacement after plan construction cannot redirect bytes;
- Windows running-image replacement path;
- existing CLI exit-code tests;
- release/package smoke and binary-size comparison.

## 11. Verification commands

Use Egress's canonical verification policy, at minimum:

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p eggress-cli --locked
cargo test -p eggress-cli --test cli_exit_codes --locked
cargo test --workspace --locked
cargo +1.89.0 check --workspace --all-targets --locked
scripts/release-preflight.sh --check-versions-only
cargo package -p eggress-cli --locked
git diff --check
~~~

Do not use `--all-features` for the whole Egress workspace because it enables test-only insecure QUIC.

## 12. Documentation updates

- Egress updater/install/release docs in the consumer repo;
- Eggup consumer adoption roadmap/registry;
- cross-repo closure `plans/closure/consumer-adoption/006-status.md`.

## 13. Acceptance criteria

Egress updates the two binaries as one Eggup transaction, no longer shells out for archive extraction, no longer owns generic pair rollback, preserves release/checksum/version/CLI policy, remains packageable, and passes Linux/macOS/Windows qualification.

## 14. Stop conditions

Stop before dependency merge if Package M008 is not published in a crates.io-usable form. Stop if Windows running-image semantics regress, if Egress needs a product-specific change in Eggup core, or if binary/dependency growth is material without explicit acceptance.

## 15. Closure evidence required

Record Eggup version/revision, deleted local updater mechanics, behavior parity matrix, fault-injection outcomes, package/dry-run evidence, binary/dependency delta, hosted CI, and any retained Egress-specific updater code with rationale.

## 16. Handoff notes

The Eggup plan is the cross-repo adoption contract. Actual consumer code changes belong in `eggstack/eggress`; do not modify Gregg or Eggpack as part of this migration.
