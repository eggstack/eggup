# Consumer Adoption M003 — Closure and Verification Record (Eggsearch)

Status: closed

Source plan: `plans/implementation/consumer-adoption/003-eggsearch-service-aware-adoption.md`

Source roadmap: `plans/subsystems/consumer-adoption-roadmap.md#M003--eggsearch-adoption`

Eggup baseline: `cf5b3d3819c168eb2dbf841daa8332f3eb28c915`.
Consumer baseline: `eggstack/eggsearch@68ae2fa5457c3fb8fa335851d60d0dce3e98aa08`.
Consumer implementation: `eggstack/eggsearch@0edff1b729cddb48cd88c0e5025a5b2b3eeea6c1` on pushed branch `codex/eggsearch-m003`.
No Eggup production code changed in this milestone. No Eggup package publication or consumer release was performed.

## Executive finding

Eggsearch M003 is complete. Its updater now uses the immutable Eggup core/acquisition/Eggfetch revision for bounded release acquisition, selected-filename SHA-256 verification, candidate identity validation, and the Unix commit transaction. Its systemd, launchd, and cron registration mechanics use `eggup-service`. Release selection, registry policy, exact fallback conditions, service rendering, health checks, manager selection, cron watchdog/PID policy, and CLI wording remain consumer-owned.

The release binary grew 2,361,344 bytes (12.6%) from the refreshed baseline. This is material and should be considered in future Eggsearch product packaging; it is recorded rather than treated as a generic Eggup defect. No medium-or-higher migration defect was found. Windows manager registration and running-image commit remain consumer-owned and were not qualified on Windows in this pass.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Use corrected Eggup updater path | Exact Eggup git revision `cf5b3d3819c168eb2dbf841daa8332f3eb28c915` for `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, and `eggup-service`; locked tree retains one `eggfetch-core` transport | passed |
| Preserve release policy and fallback | crates.io stable-version discovery and GitHub target mapping remain consumer-owned; only unsupported target or typed exact-asset `NotFound` can enter Cargo fallback; all other download, integrity, validation, ownership, and commit errors remain hard failures | passed by code review and updater tests |
| Bound acquisition and verify selected artifact | Eggup transport keeps 20 s connect/total timeout policy, 10 redirects, environment proxy behavior, and 128 MiB artifact cap; sidecar is parsed by Eggup and bound to the selected asset filename | passed |
| Validate candidate identity before commit | Eggup `ExactIdentityValidator` requires exact `eggsearch <version>\n`, successful `--version`, bounded output, and timeout; checksum and wrong-version negative tests prove no destination replacement | passed |
| Preserve ownership and recovery semantics | Current executable canonical-path verifier feeds Eggup `CommitOwnership`; shared receipt outcomes map to updater results/errors with recovery evidence retained | passed by code review and full suite |
| Use shared Unix manager mechanics | `SystemdManager`, `LaunchdManager`, and `CronManager` inspect and mutate registrations; Eggsearch retains definition rendering, manager choice, health, privilege messaging, and CLI mapping | passed |
| Preserve cron entries and migrate legacy owned line | Shared managed-block merge preserves unrelated bytes; only one exact legacy eggsearch command is migrated after a reread check; foreign or duplicate legacy lines fail closed | passed by implementation and tests |
| Preserve service lifecycle and health policy | Existing update snapshot/restart path uses shared manager state; stopped registrations stay stopped; application health remains separate from manager ownership; cron process watchdog/PID identity remains local | passed by full suite and code review |
| Avoid second transport stack and delete duplicated mechanics | `eggfetch-core` remains one production transport dependency; local SHA-256, bounded candidate execution, staging/commit and generic manager operation helpers were removed where Eggup covers them | passed |
| Record dependency and executable size impact | Before/after dependency tree and same-profile release binary measurements below | passed |
| Document Windows scope truthfully | Windows `self-replace` remains a narrow local commit shim after shared acquisition/integrity/identity validation; Windows SCM registration remains consumer-owned | passed; native Windows behavior not verified |

## Before/after ownership and footprint

| Measure | Baseline `68ae2fa` | Consumer commit `0edff1b` | Change |
|---|---:|---:|---:|
| `src/update.rs` lines | 1,146 | 1,124 | -22 |
| `src/startup.rs` lines | 1,541 | 1,692 | +151 |
| Production source diff across those modules | — | 532 insertions / 403 deletions | +129 net lines |
| Eggsearch release binary, default release profile | 18,777,728 bytes | 21,139,072 bytes | +2,361,344 bytes (+12.6%) |

The updater module is slightly smaller after replacing the generic implementation. Startup gained adapter and legacy cron migration glue; it grew because existing consumer policy and health behavior stayed local while manager state was translated to the shared service contract. The overall diff deletes 403 production lines and adds 532, including tests and adapter code.

The baseline and migrated binaries were both built with `cargo build --locked --release` on this host/profile. The consumer tree has the four direct Eggup crates pinned to one immutable revision. `eggfetch-core 0.2.0` is shared by Eggsearch and `eggup-eggfetch`, not duplicated. `sha2` is dev-only; `self-replace` is Windows-target-only. The package still has Eggsearch's existing production HTTP stack.

## Failure, recovery, and security review

- Only exact selected-asset HTTP 404/typed `NotFound` and unsupported target classification permit Cargo fallback. Timeout, TLS, 5xx, malformed sidecar, filename mismatch, checksum mismatch, identity mismatch, ownership conflict, and commit failure do not fall back.
- The selected release file is staged under Eggup acquisition bounds, then checked against a filename-bound SHA-256 sidecar before any candidate process runs.
- Commit uses Eggup's prepared one-member install transaction, exact destination ownership verifier, executable permission intent on Unix, and typed receipt mapping. Rolled-back and recovery-required results retain their distinct updater outcomes.
- systemd/launchd Foreign or Unknown ownership is surfaced as conflict before manager mutation. Health state is not used as ownership evidence. No privilege escalation is hidden in the adapter.
- Cron block merging retains unrelated crontab content. Legacy migration requires a unique exact old marker/command and refuses foreign or ambiguous entries; a reread detects concurrent edits before writing.
- Windows commit is not represented as Eggup transaction support: the consumer keeps `self-replace` only after shared candidate checks. Windows service registration remains on the existing consumer path.
- No high- or medium-severity generic Eggup finding was identified. Accepted limitation: Windows runtime/SCM behavior and hosted CI were unavailable in this pass. The Windows cross-target check below was blocked in dependency build tooling before Eggsearch code was checked.

## Exact verification commands and results

Environment: Darwin x86_64; stable `rustc 1.98.1` / Cargo 1.98.1; Rust 1.89.0 installed.

Passed:

```text
cargo fmt --all -- --check                         # via make check
cargo clippy --locked --all-targets --all-features -- -D warnings # via make check
cargo check --locked --no-default-features         # via make check
cargo test --locked --all-features                 # via make check: 3,240 passed; integration targets and doctests passed
./packaging/check-repo-hygiene.sh                   # via make check
./packaging/check-contract.sh                       # via make check
make check                                          # exit 0
cargo test --locked --all-features update::tests::checksum_mismatch_and_candidate_mismatch_stop_before_replacement -- --nocapture # 1 passed
cargo +1.89.0 check --locked --all-targets --all-features # passed
cargo build --locked --release                     # baseline and migrated build passed
cargo tree --locked -e normal --depth 2            # dependency review passed
git diff --check                                   # passed
```

The plan's exact default-feature `cargo +1.89.0 check --locked --all-targets` command failed because pre-existing Eggsearch integration targets import the `mock` module and `fetch_disabled_state` while their required feature is disabled. Re-running with `--all-features` passed. This is an existing feature-gating issue outside M003's changed paths; it remains recorded for follow-up rather than being hidden.

Attempted Windows cross-target check:

```text
cargo check --locked --all-targets --all-features --target x86_64-pc-windows-msvc
```

Blocked before Eggsearch target code was checked: the local macOS C toolchain lacks Windows/MSVC headers; dependency `ring` failed to find `assert.h`. No hosted CI run or native Windows runtime was available. Windows evidence is therefore **not run/blocked**, not inferred from Unix tests.

An early version of the wrong-candidate test placed its source inside the destination's install root and correctly received Eggup's source-containment error. The fixture was moved to a separate temporary directory; the targeted identity-negative test then passed. No production change was needed for that finding.

## Compatibility and migration review

- Release authority, SemVer policy, target selection, URL construction, exact Cargo command, and CLI error wording remain in Eggsearch.
- Eggsearch keeps `eggfetch-core` because it is used by providers and other product HTTP paths; adoption adds no second HTTP/TLS stack.
- The manager adapter receives exact executable, argv, and config identity. Service definitions remain rendered by Eggsearch.
- Existing systemd/launchd state and restart behavior are delegated to the shared service managers. Cron supervision, PID/start-token checks, and health probes remain consumer-owned.
- Windows self-replacement and SCM registration are explicitly outside the adopted shared boundary and remain unchanged in behavior by design.
- Eggup crates are pinned to an immutable git commit because the corrected crates.io patch was not published. Any later crates.io consumer release must separately move to a published patch; no floating branch dependency is present.

## Disposition and downstream readiness

Consumer Adoption M003 is closed. The Eggsearch branch is pushed at `0edff1b729cddb48cd88c0e5025a5b2b3eeea6c1`; no pull request or hosted CI result is claimed. The consumer-adoption roadmap is updated.

This closure does not unblock Gregg M004: the corrected-path footprint decision is still outstanding, and Eggsearch's +12.6% binary-size result is not Gregg's measurement. Distribution M003 and Service M004 were already independently dependency-ready; the sequential batch proceeds to Distribution M003 next. CodeGG/Egress remain gated on distribution M003 validator evidence and their own multi-artifact/archive requirements.
