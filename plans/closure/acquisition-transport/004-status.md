# Acquisition Transport M004 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/004-validated-limits-and-promotion-state-corrective.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M004--validated-limits-and-promotion-state-corrective`

Reviewed repository baseline: `889a234cbe7f461d92def3df45c83c06a7d257e5`

Implementation commit: `6d101dace56f60082ec984f1e553383ed8da3da5` (`fix: validate acquisition limits and promotion state`).

## Executive finding

M004 is complete. `FetchLimits::validate` is the single rule set used by the
constructor and every fixture/Eggfetch metadata and artifact entry point.
Invalid public-field literals now fail before route lookup, filesystem work,
or network I/O. Successful no-clobber hard-link creation is the promotion
commit point; failure to remove the redundant temp link is best-effort cleanup
and no longer returns ordinary failure with a complete destination present.
The 0.1.x public fields and transport signatures remain compatible. No
medium-or-higher acquisition contract issue remains.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| One authoritative validation rule | `FetchLimits::validate`; `FetchLimits::new` delegates to it | passed |
| Metadata range and timeout relationships | direct-literal matrix covers zero/over-16-MiB metadata, zero connect/total, and connect greater than total | passed |
| Fixture validates before route lookup | invalid values against an unregistered URL return `InvalidInput`, not route failure | passed |
| Eggfetch validates before network/filesystem work | invalid values against an unreachable URL return `InvalidInput`; artifact destination and directory remain untouched | passed |
| Valid direct literals remain accepted | `limits.validate()` and constructor success asserted; existing successful transport cases remain green | passed |
| Promotion commit point is truthful | `__promote_no_clobber_with` ignores injected post-link temp cleanup failure; destination and owned temp both contain complete bytes and result is success | passed |
| Existing/raced destination preservation | M003 `existing_destination_is_a_hard_no_clobber_failure` and `promote_no_clobber_preserves_raced_destination` remain green | passed |
| No ordinary pre-promotion error after commit | post-link injected cleanup regression returns `Ok`; failure-path regression tests assert destination absent | passed |
| eggsact updater regression | copied consumer, local crate patches, `cargo test --bin eggsact update::tests`: 25 passed, 0 failed | passed |
| stegoeggo regression | full copied consumer test run reported 649 passed across four suites; updater-specific `cargo check --tests` against patched local crates passed | passed, with native test-link limit below |
| Runtime/core boundary | changes touch acquisition and Eggfetch only; `eggup-core` remains transport-free | passed |

## Production implementation evidence

- `FetchLimits::validate` enforces `max_metadata_bytes` in `1..=16 MiB`,
  non-zero connect/total deadlines, and `connect <= total`.
- Both `FixtureTransport` methods and both `EggfetchTransport` methods call
  validation at entry. Public `EggfetchConfig::effective_timeouts` documents
  its validated-input precondition.
- Public fields remain available for 0.1.x source compatibility; invalid
  values created with struct literals now fail closed at transport boundaries.
- `__promote_no_clobber` treats successful hard-link creation as commit and
  ignores redundant temp-link cleanup failure. The private injectable helper
  gives deterministic coverage without a public filesystem abstraction.
- README and rustdoc explain validation and post-promotion cleanup debt.

## Exact commands and results

Environment: Darwin 25.6.0 arm64; stable `rustc 1.98.1`; installed MSRV
`1.89.0`.

Passed:

```text
cargo fmt --all
cargo test -p eggup-acquisition -p eggup-eggfetch --locked
cargo +1.89.0 test -p eggup-acquisition -p eggup-eggfetch --locked
cargo package -p eggup-acquisition --locked --allow-dirty
cargo package -p eggup-eggfetch --locked --allow-dirty --no-verify
./scripts/check-local.sh
git diff --check
```

The package verifier `cargo package -p eggup-eggfetch --locked --allow-dirty`
was also attempted with verification enabled and failed because Cargo
compiled the packaged adapter against the already-published
`eggup-acquisition 0.1.0`, which lacks the new API. Packaging without isolated
verification passed; workspace check/test/doc and the local patched consumer
checks use the corrected source. This is release ordering evidence, not a
production compile failure.

Consumer commands used temporary copies so their lockfiles and worktrees were
not modified:

```text
cargo test --bin eggsact update::tests --config patch.crates-io.eggup-acquisition.path=... --config patch.crates-io.eggup-eggfetch.path=...
cargo check --tests --manifest-path stegoeggo-cli/Cargo.toml --config patch.crates-io.eggup-acquisition.path=... --config patch.crates-io.eggup-eggfetch.path=...
```

The stegoeggo updater test binary could not be linked in this environment:
Apple clang crashed while linking `zerofrom-derive` (signal 11). The broader
stegoeggo test invocation reported 649 passing tests, but the runner required
interrupting a lingering `rtk` wrapper after its pass summary. The focused
consumer test compilation passed. These limits are reported rather than
treated as native test-link evidence.

Hosted Linux/macOS/Windows CI was not available before push. The push for this
work wave will trigger the repository's hosted CI; its result is recorded in
the final handoff if accessible.

## Invariant review

- Invalid caller limits cannot be legitimized by adapter ceilings.
- Validation occurs before potentially blocking route lookup and before
  network or filesystem work.
- The destination is absent on ordinary pre-promotion error; foreign/raced-in
  destinations remain untouched.
- Once complete bytes are linked to the destination, cleanup debt cannot
  produce a contradictory ordinary failure.
- Temp cleanup remains scoped to the operation-owned path.
- No unsafe code, FFI, retry, fallback, or core transport dependency was added.

## Failure and recovery review

| State/event | Result |
|---|---|
| Invalid limits | `InvalidInput`; no route/network/filesystem activity |
| Hard-link fails | ordinary error; destination remains absent or foreign destination remains intact |
| Destination exists or races in | hard failure; foreign destination preserved; owned temp cleanup remains best-effort |
| Hard-link succeeds and temp unlink succeeds | success; destination complete; no temp residue |
| Hard-link succeeds and temp unlink fails | success; destination complete; owned temp residue may remain |
| Process stops after link before cleanup | both names may remain and reference the complete artifact; no scavenger is claimed |

## Compatibility and migration review

No signature or field-visibility break was introduced. Valid eggsact and
stegoeggo request values remain valid. Invalid struct literals now fail before
I/O by design. The existing workspace release note still identifies 0.1.1
as the next lockstep patch candidate; no version was rewritten and no crate
was published. Publication remains a separately directed release action.

## Security review

Direct struct literals cannot bypass bounds; all untrusted byte/time limits
are checked at the adapter boundary. Promotion remains no-clobber and does
not use unsafe/FFI. Post-commit cleanup does not remove a valid destination or
misreport its state. Existing diagnostic redaction, cancellation, size, and
timeout regressions passed in the acquisition/Eggfetch suites.

## Documentation and operations evidence

- Acquisition and Eggfetch READMEs document validation at transport entry.
- Rustdoc records the public-field compatibility rule and helper precondition.
- `CHANGELOG.md` records M004 behavior and no-publication disposition.
- Acquisition roadmap and registry now show M004 closed and downstream gate
  effects.

## Unresolved findings

- Informational: isolated Eggfetch package verification uses the published
  0.1.0 seam; packaged compile verification must follow the planned seam-then-
  adapter release order or use a local patch.
- Informational: native stegoeggo updater test linking hit an Apple clang
  crash; its test targets compile successfully and the broader suite reported
  passing tests.
- Informational: hosted CI is pending push.
- None: no medium-or-higher acquisition correctness or security finding.

## Disposition and roadmap transition

M004 is closed. The acquisition corrective gate is removed for broader
updater-bearing consumer work. Eggsearch planning still depends on service
M003; Gregg planning still depends on service M003 and an updated corrected-
path footprint decision. Optional lightweight acquisition M005 remains
deferred until that footprint measurement demonstrates a real requirement.

## Registry updates

- Acquisition M004 → closed.
- Consumer adoption: acquisition gate closed; service M003 remains the
  applicable blocker until its closure.
- Optional acquisition M005: remains deferred/evidence-driven; no
  implementation plan is ready until corrected Eggfetch footprint evidence
  is collected.
