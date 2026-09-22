# Verified Update Core M004 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M004--integrity-and-candidate-validation`

Reviewed repository baseline: `b9e5448`

## Implementation commits/PRs

- `527d12f` (`feat: add integrity and candidate validation`)
- `d4f3459` (`fix: enforce verified candidate execution boundary`)
- No PR was required for this local implementation pass.

## Executive finding

M004 is complete. Eggup now streams native SHA-256, parses strict single-entry
sidecars with exact optional filename binding, and enforces the public phase
sequence `PreparedTransaction -> VerifiedTransaction -> ValidatedTransaction ->
commit`. Candidate commands use literal argv, cleared inherited environment,
null stdin, controlled working directory, bounded output, and timeout
kill/reap. Integrity is distinct from authenticity; no signing or trust policy
was invented.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Native streaming SHA-256 | `hash_file` and known `abc` vector | passed |
| Strict sidecar parsing | one-entry, malformed, multi-entry, case, and filename tests | passed |
| Exact filename binding | wrong filename rejection in sidecar test | passed |
| Integrity mismatch hard failure | `integrity_verification_precedes_candidate_validation` | passed |
| Integrity/authenticity distinction | `IntegrityRequirement` and no signing implementation | passed |
| Candidate phase separation | only `ValidatedTransaction` exposes public commit | passed |
| Literal argv/no shell interpolation | `CommandSpec` and exact validator | passed |
| Timeout kill/reap | bounded timeout test | passed |
| Output bound | bounded overflow test | passed |
| Environment minimization | cleared-environment test returns `unset` | passed |
| Exact identity/version output | strict validator test | passed |
| Cross-member agreement | two-member agreement test | passed |
| Custom validator composition | `CandidateValidator` and `AllValidators` API | passed |

## Production implementation evidence

`integrity.rs` hashes files in 64 KiB reads and rejects malformed or ambiguous
sidecars. `candidate.rs` runs child processes with `env_clear`, null stdin,
literal arguments, a configured current directory, per-stream output caps, and
deadline enforcement. `VerifiedTransaction::validate` refuses to invoke any
candidate validator unless every member has verified declared SHA-256 evidence;
an absent integrity declaration therefore cannot unlock candidate execution.

The only runtime dependency added is `sha2` 0.10.9, justified by native local
hashing and isolated from transport, service, and consumer dependencies.

## Exact commands run and results

Environment: `Darwin 25.6.0 x86_64`, Rust 1.89.0.

```text
rustc +1.89.0 --version
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
./scripts/check-local.sh
cargo +1.89.0 package --workspace --locked --allow-dirty
```

The final suite passed 21/21 tests. Package verification completed
successfully. Native Windows and Linux evidence was not run and is not
inferred.

## Invariant review

- Integrity must pass before candidate validators run.
- A `None` integrity declaration is visible as `NotRequired` but cannot reach
  candidate execution or the public commit operation.
- Candidate output and execution time are bounded, and timed-out children are
  killed and reaped before a result is returned.
- Candidate identity remains consumer-supplied and no release ordering or
  fallback policy is selected.

## Failure/recovery review

Verification failure drops only disposable prepared stage state and never
mutates live destinations. Candidate timeout and output overflow terminate the
direct child and return structured failure state. Commit/rollback remains the
M003 synchronous state machine; no new live mutation path was introduced.

## Compatibility and migration review

This is pre-consumer API qualification. M004 changes the previously internal
M003 commit path into an intentionally ordered public API: consumers must call
`verify_integrity` and `validate` before `commit`. No existing Eggup consumer
requires migration, and the phase change is documented for future adopters.

## Security review

No network or service-manager dependency was added. Sidecar ambiguity,
malformed digests, wrong filenames, wrong bytes, wrong identity, nonzero exit,
timeouts, output overflow, and inherited environment secrets are covered by
negative evidence. No shell command string is constructed by the core API.

## Documentation/operations evidence

`crates/eggup-core/docs/verification.md` documents integrity versus
authenticity, phase ordering, command bounds, and custom validator composition.
The architecture overview and changelog link the new capability. The package
dry-run and ordinary CI/local verification remain clean.

## Unresolved findings with severity

- Medium / accepted scope: authenticity signatures and attestations are not
  implemented; `AuthenticityRequirement` remains an explicit policy
  descriptor. A future trust ADR must define them.
- Informational: native Windows process and permission evidence was not
  available in this environment; platform qualification remains future work.
- Informational: M005 package qualification and consumer adoption are not part
  of these four plans.

## Disposition and roadmap transition

M004 is closed. Verified update core M001-M004 are now closed. M005 is
dependency-unblocked for implementation-plan authoring, while acquisition
transport and service-lifecycle plan authoring can also proceed from the closed
M002/M003 interfaces. Consumer adoption remains blocked on transport work.

