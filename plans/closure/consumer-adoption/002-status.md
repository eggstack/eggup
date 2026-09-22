# Consumer Adoption M002 — Closure and Verification Record (Stegoeggo Second Adoption)

Status: closed

Source plan: `plans/implementation/consumer-adoption/002-stegoeggo-second-adoption.md`

Source roadmap: `plans/subsystems/consumer-adoption-roadmap.md#M002--stegoeggo-adoption`

Eggup repository baseline: qualified `0.1.0` (`eggup-core/acquisition/eggfetch/service` on crates.io; no Eggup code change in this milestone).
Consumer implementation commit: `eggstack/stegoeggo@10d8448` (`feat: adopt Eggup verified update`).
Eggsact comparison baseline: `eggstack/eggsact@576f4b0` (M001 closure).

The plan was refreshed against stegoeggo HEAD `9bc75b3` and the eggsact adoption closure before handoff.

## Executive finding

M002 is complete. Stegoeggo is the second independent single-binary consumer on the same Eggup core/acquisition contracts. No eggsact-specific policy leaked into Eggup: the only consumer deltas are release-policy configuration (bare `version` argv, `stegoeggo` prefix, 300 s/16 KiB candidate bounds, 60 s total timeout, 4 MB/8 KB/64 MB body limits, yanked-skipping registry policy, `self_replace` on Windows). No Eggup corrective was required; no local workaround duplicating security-sensitive machinery was added.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Replace local updater mechanics; use `eggup-eggfetch` | `stegoeggo-cli`: +`eggup-acquisition/core/eggfetch 0.1.0`; `eggfetch-core`/`sha2` → dev-deps; acquisition via `eggup_transport()` + `seam_download`/`eggup_get_bytes` | passed |
| Exact acquisition requests from existing release selection | `release_base_url`/`registry_url` (with `STEGOEGGO_*` overrides) → `{base}/download/v{latest}/{asset}` + `.sha256`; crates.io yanked-skipping registry retained | passed |
| Destination ownership proof | `CurrentExeVerifier` (canonical current-exe equality) + `DenyCreate` | passed |
| Map transaction outcomes | `Committed` → updated; `RolledBack` → cause-preserving error; `RecoveryRequired` → evidence-path error | passed |
| Delete superseded code | Prod no longer hashes/compares, probes writability, execs candidates, or replaces directly on Unix; `run_bounded` retained only for Cargo fallback policy | passed |
| Tests/docs updated | 7 new `eggup_*` tests; full cli suite green | passed |
| Dependency/size delta | Below | passed |
| Same Eggup contracts, no private modules | Only public Eggup APIs (same set as M001) | passed |
| No eggsact policy in Eggup | Zero Eggup changes; deltas confined to the consumer adapter (below) | passed |

## Before/after code ownership

| Item | Before (`9bc75b3`) | After (`10d8448`) |
|---|---|---|
| Transport (prod) | direct `eggfetch_core::Client` | `eggup-eggfetch` strict (`eggup_transport`, 10 s/60 s) |
| Registry/sidecar fetch | `fetch_to_file`/`download_required` (prod) | `eggup_get_bytes`/`seam_download` (prod); old fns `#[cfg(test)]` harness |
| Checksum | manual `verify_checksum` (sha2 compare) | Eggup integrity (`Sha256` requirement + locked revalidation); `parse_checksum` policy retained |
| Candidate exec | `candidate_version` + `run_bounded` (300 s, 16 KB, first-line prefix) | Eggup `ExactIdentityValidator` (`version` argv, `stegoeggo {latest}\n`, 300 s, 16 KB) |
| Preflight | `ensure_replaceable` (readonly/parent/writability probe) | Eggup ownership + containment under lock |
| Replacement (Unix) | `self_replace::self_replace` | Eggup commit (lock/backup/rollback/receipt) |
| Replacement (Windows) | `self_replace` | Eggup validate → `self_replace(staged)` (running-image limit preserved) |
| Cargo fallback | `cargo_fallback` on 404/unknown target | Unchanged (still uses a small cargo runner) |
| Deps (prod) | `sha2`, `self-replace`, `eggfetch-core` direct | Eggup trio; `self-replace` Windows-only; `sha2`/`eggfetch-core` dev-only |

## Comparison with eggsact adapter

| Concern | Eggsact (`576f4b0`) | Stegoeggo (`10d8448`) | Generic? |
|---|---|---|---|
| Core contract | prepare→verify→validate→commit | identical | yes |
| Seam | `fetch_metadata`/`fetch_artifact` + `NotFound`→fallback | identical | yes |
| Ownership | canonical-exe equality + `DenyCreate` | identical shape | yes |
| Validator argv | `--version` | `version` (bare) | caller policy (parameter, not fork) |
| Expected output | `eggsact {v}\n` | `stegoeggo {v}\n` | caller policy |
| Candidate bounds | 10 s / 64 KB default | 300 s / 16 KB | caller policy |
| Total timeout | 120 s | 60 s | caller policy |
| Body limits | 1 MiB / 64 KiB | 4 MB / 8 KB / 64 MB | caller policy |
| Windows replace | staged PowerShell script | `self_replace(staged)` | caller policy |
| Registry policy | `max_stable_version` | yanked-skipping max | caller policy |

No generic deficiency was found: every delta is configuration or pre-existing platform helper choice. No Eggup API change, no consumer-local security workaround.

## Verification matrix

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features`: zero warnings.
- `cargo test -p stegoeggo-cli`: 77 + 73 + 16 passed (7 new `eggup_*`).
- Full `cargo test --workspace`: all targets green (618 + 14 + 14 + 35 + 4 + 16 + 6 + 25 + 34 + 53 across workspace lines observed; 0 failed).
- `cargo build --release -p stegoeggo-cli`: green.
- Single stack: prod tree shows the Eggup trio with one `eggfetch-core 0.2.0` underneath; direct `eggfetch-core`/`sha2` are dev-only.

## Dependency/size measurement

Release binary (`stegoeggo`, same profile): before `3,438,640` → after `3,538,224` (**+99,584, +2.9%**). Increase is Eggup core/seam/adapter code on the shared transport stack.

## Genericity finding

Two independent consumers now share the Eggup core/acquisition contracts with neither requiring private modules nor leaking policy into Eggup. The `version`-vs-`--version` difference is handled by the existing validator parameters — the exact extensibility point the shared path was designed for.

## Recommendation on eggsearch/Gregg migrations

May begin per the roadmap (eggsearch: core first, then service substrate after service qualification; Gregg: replace `gregg-update` with measured lightweight transport). If migration surfaces a generic gap, write an Eggup corrective before adding local security-sensitive workarounds (per the plan's stop condition; not triggered here).

## Unresolved findings with severity

- Low / accepted: `--version`-style exact-output rule is stricter than first-line-prefix parsing (stegoeggo `version` output is single-line; no impact).
- Low / accepted: Symlinked install parents now fail closed (hardening).
- Informational: Direct-eggfetch helpers retained `#[cfg(test)]` as policy harness.
- None: No high/medium defect remains.

## Disposition

Adoption M002 is closed. The consumer-adoption roadmap's simple-consumer tier (two independent single-binary adopters) is complete. Remaining tiers (service-aware, bundle, archive, selective) proceed under their roadmap entries with the same corrective-before-workaround rule.
