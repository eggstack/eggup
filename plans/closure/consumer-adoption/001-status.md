# Consumer Adoption M001 — Closure and Verification Record (Eggsact First Adoption)

Status: closed

Source plan: `plans/implementation/consumer-adoption/001-eggsact-first-adoption.md`

Source roadmap: `plans/subsystems/consumer-adoption-roadmap.md#M001--eggsact-adoption`

Eggup repository baseline at handoff: publication commit `db2e122` (`eggup-acquisition/core/service/eggfetch 0.1.0` on crates.io).
Consumer implementation commit: `eggstack/eggsact@576f4b0` (`feat: adopt Eggup verified update`).

The plan was refreshed against eggsact HEAD `7938d9b` before handoff (release metadata path, target table, fallback classifier, candidate output, updater tests, and Cargo features were reinspected; findings in Consumer-call mapping below).

## Implementation commits/PRs

- Consumer: `eggstack/eggsact@576f4b0` — 3 files (`Cargo.toml`, `Cargo.lock`, `src/update.rs`; +493/−104).
- Eggup side: no Eggup code change was required (the qualified 0.1.0 API sufficed).
- No PR was required; direct push to consumer `main` per local workflow.

## Executive finding

M001 is complete. Eggsact's duplicated single-binary staging/integrity/candidate/replacement machinery is materially deleted and replaced with `eggup-core 0.1.0` + `eggup-eggfetch 0.1.0`, while eggsact-owned release selection and Cargo fallback policy are preserved. Behavior parity holds for the fallback classifier, checksum fatality, and CLI contract; safety is intentionally strengthened (backup/rollback, ownership proof, locked revalidation) with no high/medium migration defect remaining.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Add Eggup dependencies | `eggsact/Cargo.toml`: `eggup-acquisition/core/eggfetch 0.1.0` (versioned, crates.io); `eggfetch-core` + `futures-util` moved to dev-deps (test harness only) | passed |
| Translate release result into Eggup acquisition + InstallPlan | `prepare_candidate_async` → `eggup_download`/`eggup_get_text` (spawn_blocking); `commit_candidate` builds `InstallPlan{product eggsact, release latest, root=current.parent, member main}` with `Sha256` + `Executable` intent | passed |
| Existing-destination ownership verifier from known identity/path | `CurrentExeVerifier`: `Owned` iff canonical destination == canonical `current_exe` and regular file; else `Foreign`/`Unknown`/`Absent`; `DenyCreate` (replacement-only) | passed |
| Map transaction outcomes into CLI | `Committed` → updated message; `RolledBack` → error with phase/cause, old version preserved; `RecoveryRequired` → high-severity error with evidence path + retained lock | passed |
| Delete superseded generic code | Deleted: `sha256_file`, `run_bounded`, `wait_bounded`, Unix `replace_current` (copy/chmod/rename); structural test `eggup_generic_machinery_is_deleted_not_wrapped` guards | passed |
| Retain target/release/Cargo fallback policy | `target_for_host`, asset URLs, `cargo_candidate`, 404/unsupported-host fallback intact; `eggup_fallback_classifier_is_preserved` | passed |
| No second HTTP/TLS stack | Prod tree: one `eggfetch-core 0.2.0` via `eggup-eggfetch`; direct use dev-only | passed |
| No fetched shell execution | No `Command::new("curl")` (existing test); Eggup never executes downloads | passed |
| CLI/output contract preserved | `already current`, `updated …`, staged-Windows messages unchanged | passed |
| Updater failure preserves old exe or RecoveryRequired | Unix Eggup rollback restores; Windows staged script unchanged after validation | passed |
| Compatibility fixtures/tests | 6 new `eggup_*` tests + all 34 pre-existing update tests green | passed |
| Dependency/size measurement | Below | passed |

## Before/after updater module inventory

| Item | Before (`7938d9b`, 1451 lines) | After (`576f4b0`, 1807 lines) |
|---|---|---|
| Transport (prod) | direct `eggfetch_core::Client` (`build_update_client`, `get_small_text`, `download_to`) | `eggup-eggfetch` strict via `eggup_transport()` + `eggup_get_text`/`eggup_download` (spawn_blocking) |
| Hashing | local `sha256_file` (sha2 streaming) | `eggup_hash` → `eggup_core::hash_file` |
| Candidate exec | local `run_bounded`/`wait_bounded` (poll/kill, inherited env) | Eggup `ExactIdentityValidator` (`--version`, 10 s, cleared env, bounded output, empty-stderr rule) |
| Replacement (Unix) | copy/chmod/rename, no backup | Eggup transaction (lock, ownership revalidation, staged revalidation, backup, rollback, receipt) |
| Replacement (Windows) | PowerShell staged script | Unchanged script, now fed by Eggup-validated staged executable |
| Ownership | none (implicit path trust) | `CurrentExeVerifier` + `DenyCreate` |
| Staging | `unique_temp_dir` only | Retained for acquisition; Eggup adds private verified stage |
| Release policy | crates.io → GitHub asset → sidecar → `--version` → replace; Cargo on 404/unsupported | Unchanged |
| Test-only harness | — | Old direct-eggfetch path retained under `#[cfg(test)]` for policy tests (same 0.2.0, dev-deps) |

Net +356 lines: ~−60 generic deleted, ~+150 Eggup glue, ~+200 adapter + compatibility tests.

## Deleted duplicated code summary

- `sha256_file` (14 lines) → `eggup_core::hash_file`.
- `run_bounded` + `wait_bounded` (35 lines) → Eggup bounded validators.
- Unix `replace_current` (12 lines) → Eggup commit with backup/rollback.
- Direct-eggfetch prod path (`build_update_client`, `get_small_text`, `download_to` prod use) → Eggup seam; functions retained `#[cfg(test)]` as policy harness.

## Behavior matrix

| Case | Before | After | Parity |
|---|---|---|---|
| Already current/no-op | message, no mutation (consumer-owned) | unchanged | exact |
| Exact asset success | download → checksum → `--version` → rename | download → sidecar → Eggup verify/validate/commit | equivalent, strengthened (lock/backup/rollback) |
| Checksum mismatch | hard error, no fallback | hard error pre-commit, no fallback | exact |
| Candidate wrong identity/version | hard error | validator rejection, no commit | equivalent (stderr must now be empty — stricter) |
| 404 asset | Cargo fallback | Cargo fallback, then Eggup path for built binary | exact |
| 5xx/TLS/timeout | hard error, no fallback | hard error via seam, no fallback | exact |
| Ownership failure (non-self path) | replaced anyway (implicit trust) | denied closed (`Foreign`/`Unknown`) | intentionally strengthened |
| Rollback | none (failed rename could leave adjacent file) | verified rollback or `RecoveryRequired` with evidence | strengthened |
| Recovery-required | n/a | lock + backup retained, actionable message | new capability |
| Unsupported target | Cargo-only | Cargo-only | exact |
| Stderr on `--version` | ignored | must be empty (validator rule) | stricter; `--version` is stdout-only |

## Dependency and binary-size comparison

Prod dependency delta: +`eggup-acquisition/core/eggfetch 0.1.0`; −direct `eggfetch-core`/`futures-util` (now dev-only, same versions). Single intended HTTP/TLS stack (`eggfetch-core 0.2.0` via `eggup-eggfetch`).

Release binary (`--bin eggsact`, same profile): before `11,101,552` bytes → after `11,292,992` bytes (**+191,440, +1.7%**). Increase is the Eggup core/seam/adapter code with a shared transport stack; no second TLS stack was added.

## Full eggsact verification

- `cargo fmt --all -- --check`: clean (after `cargo fmt`).
- `cargo clippy --all-targets --all-features`: zero warnings (old-path harness gated `#[cfg(test)]`, contract helpers documented).
- `cargo test --bins`: 40/40 (34 pre-existing + 6 new `eggup_*`).
- Full `cargo test --workspace --all-targets`: 643 + 40 + 3392 + 51 passed, 0 failed.
- `cargo build --release --bin eggsact`: green (size above).

## Eggup version/API used

`eggup-core 0.1.0`, `eggup-eggfetch 0.1.0` (`eggup-acquisition 0.1.0` transitively). Only public APIs: `InstallPlan/ArtifactSet/ArtifactMember`, `IntegrityRequirement`, `ExactIdentityValidator`, `CommitOwnership/AbsentPolicy`, `CurrentExeVerifier→OwnershipVerifier`, `TransactionDisposition`, `AcquisitionRequest/FetchLimits/FetchOutcome/CancelFlag`, `EggfetchConfig/Transport/ProxyDecision`. No private modules used.

## Generic defect discovered

None requiring an Eggup corrective. Two observations (accepted, no API change):
1. Sync seam inside an async consumer needs `spawn_blocking` (documented in code); no seam change required.
2. Windows running-image replacement still needs the staged script after validation (known M005 limitation); Eggup validated the bytes, the script moves them. No Eggup API gap — a future Windows-commit primitive remains roadmap work, not a defect.

## Unresolved findings with severity

- Low / accepted: `--version` stderr must now be empty (validator rule); eggsact `--version` is stdout-only, no behavior impact.
- Low / accepted: Symlinked install parents now fail closed with an actionable error (previously followed); intentional hardening.
- Informational: Old direct-eggfetch path retained `#[cfg(test)]` as policy harness; future cleanup may port it to seam helpers.
- None: No high/medium migration defect remains.

## Disposition

Adoption M001 is closed. Eggsact proves the corrected core + Eggfetch adapter on a real single-binary consumer with release authority and fallback policy intact. Unblocks adoption M002 (stegoeggo second adoption).
