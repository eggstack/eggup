# Acquisition Transport Milestone 004 — Validated Limits and Promotion-State Corrective

Status: implemented (closed; see `plans/closure/acquisition-transport/004-status.md`)

Repository baseline: `889a234cbe7f461d92def3df45c83c06a7d257e5`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Corrects post-closure findings from:

- `plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md`
- `plans/closure/acquisition-transport/003-status.md`

Primary class: invariant / corrective

## 1. Objective

Close two remaining correctness gaps in the M003 acquisition contract before any additional updater-bearing consumer adopts Eggup:

1. make `FetchLimits` validation authoritative at every transport call even when callers bypass `FetchLimits::new` through the currently-public fields;
2. make artifact promotion terminal state truthful so an ordinary `Err` can never be returned after the destination has already become a valid promoted artifact.

This milestone is intentionally narrow. It does not redesign acquisition policy, add retries, or add a new transport.

## 2. Readiness and dependencies

Hard dependencies are closed:

- acquisition M001-M003;
- verified-update core M005-M006;
- eggsact/stegoeggo simple-adopter evidence.

M003 is functionally successful but its closure overstated two invariants:

- `FetchLimits::new` validates, but all fields remain public and both fixture/production transports accept a `FetchLimits` value directly;
- `__promote_no_clobber` creates `dest` by hard-linking the complete temp and then returns `Err` if removal of the temp link fails.

The current implementation HEAD is the baseline above.

## 3. Current evidence and detection gap

### A. Limits validation bypass

`FetchLimits` currently exposes:

- `pub max_metadata_bytes`;
- `pub max_artifact_bytes`;
- `pub connect_timeout`;
- `pub total_timeout`.

The constructor rejects zero/out-of-range metadata limits, zero timeouts, and `connect > total`, but a caller can construct invalid values directly.

`FetchLimits::effective` returns minima without validation, and `eggup-eggfetch` applies the resulting timeout directly.

Detection gap: M003 tests exercised values created through the constructor/default path and did not include invalid public-field literals at the transport boundary.

### B. Promotion terminal-state ambiguity

`__promote_no_clobber`:

1. hard-links `tmp -> dest`;
2. attempts `remove_file(tmp)`;
3. returns `Err(Io)` if temp unlink fails.

After step 1, `dest` already contains the complete artifact. Therefore the caller can receive an ordinary failure while observing a promoted destination.

Detection gap: M003 tested destination collision and successful promotion but did not fault-inject failure after the no-clobber link was established.

## 4. Invariants that must not regress

- every transport call validates byte/time limits before I/O;
- invalid limits fail before network or filesystem mutation;
- adapter ceilings may tighten valid request deadlines but never legitimize invalid caller input;
- `FetchOutcome::Success` means a complete artifact exists at the destination;
- an ordinary acquisition `Err` before promotion means the destination was not newly created by that operation;
- once a complete no-clobber destination has been created, cleanup of an auxiliary temp link must not convert that successful promotion into an ordinary pre-promotion-style error;
- foreign/preexisting destinations are never overwritten or removed;
- cleanup only touches state owned by the current operation;
- no unsafe/FFI is introduced solely to obtain no-replace rename semantics;
- core remains transport-free.

## 5. Scope and non-scope

### In scope

- one canonical `FetchLimits::validate` or equivalent checked boundary;
- transport-boundary validation in fixture and Eggfetch adapters;
- tests using direct struct literals to prove constructor bypass cannot weaken enforcement;
- documentation of public-field compatibility;
- promotion outcome semantics after hard-link success;
- fault-injectable post-link cleanup behavior;
- exact terminal-state tests;
- eggsact/stegoeggo regression qualification;
- version/release note reconciliation.

### Out of scope

- making `FetchLimits` fields private if that causes unnecessary 0.1.x source breakage;
- retries, mirrors, fallback;
- lightweight/curl transport;
- changing TLS/proxy/redirect policy;
- crash-journal/scavenger design;
- archive acquisition;
- publication to crates.io.

## 6. Required production changes

### A. Canonical limits validation

Add one reusable validation path on `FetchLimits`.

It MUST validate at least:

- `max_metadata_bytes > 0`;
- `max_metadata_bytes <= 16 MiB` or the currently documented seam maximum;
- `connect_timeout > 0`;
- `total_timeout > 0`;
- `connect_timeout <= total_timeout`;
- any artifact-size constraint introduced by existing docs/types.

`FetchLimits::new` MUST call the same validation logic rather than duplicate it.

### B. Validate at trust boundaries

Every `AcquisitionTransport::fetch_metadata` and `fetch_artifact` implementation in the workspace MUST reject invalid `FetchLimits` before:

- route lookup that can block;
- client request construction/sending;
- destination/temp creation;
- body streaming.

`EggfetchConfig::effective_timeouts` MUST NOT be the first place invalid caller timeouts are observed.

If a helper remains public, its documentation must state whether it assumes validated input or returns a checked result.

### C. Preserve 0.1.x compatibility deliberately

Preferred corrective:

- retain public `FetchLimits` fields for this patch line;
- add `validate(&self) -> Result<(), AcquisitionError>`;
- call it at every transport boundary.

Field privatization may be deferred to an explicit API-stabilization/breaking pass.

If implementation proves that public fields make the invariant unmaintainable, stop and document the required source break rather than silently changing it.

### D. Truthful promotion semantics

Define the point of no return precisely.

Once `hard_link(tmp, dest)` succeeds, `dest` is a complete artifact backed by the same inode/bytes as the fully-written temp. Failure to remove the redundant temp link is a cleanup-residue condition, not a failed acquisition.

Preferred behavior:

- promotion becomes committed at successful no-clobber link creation;
- temp-link removal is best-effort cleanup after commit;
- failure to remove the redundant temp MUST NOT return a normal acquisition error that implies no destination was created;
- the operation returns success with complete `ArtifactEvidence`;
- owned temp residue may remain and is documented as cleanup debt.

If maintainers require cleanup failure to be surfaced synchronously, introduce an explicit success-with-warning/recovery outcome that still states `destination_present=true`. Do not overload ordinary `Err`.

### E. Do not roll back a valid destination casually

Do not remove `dest` merely because temp unlink failed unless there is a provably race-safe ownership token showing the current operation still owns the exact destination link.

The simplest safe semantics are to keep the valid destination and treat redundant temp cleanup as post-promotion best effort.

### F. Fault injection seam

Make the post-link cleanup path deterministically testable without requiring filesystem permission tricks that vary by host.

A private promotion filesystem/test seam is acceptable.

Do not expose a broad public filesystem abstraction solely for tests.

## 7. Ordered work packages

A. Add canonical `FetchLimits` validation and direct-literal negative tests.

B. Apply validation at fixture and Eggfetch transport boundaries.

C. Refactor promotion helper around an explicit committed point.

D. Add deterministic post-link cleanup fault injection and state assertions.

E. Re-run M003 collision/cancellation/redaction tests.

F. Run eggsact/stegoeggo updater suites against the corrected path.

G. Update docs/changelog/closure evidence and version decision if needed.

## 8. Failure, restart, and contention semantics

Before successful no-clobber link:

- cancellation/error -> destination absent, owned temp cleaned best-effort.

Destination already exists/races in:

- hard failure;
- foreign destination preserved;
- owned temp cleaned best-effort.

After successful no-clobber link:

- destination is committed and complete;
- temp unlink success -> ordinary success, no residue;
- temp unlink failure -> success or typed success-with-warning, never ordinary failure-with-destination;
- no retry may overwrite the committed destination.

Process crash after link but before unlink may leave both names. Both names reference complete bytes. No crash scavenger is added here.

## 9. Compatibility and migration

No signature break is expected.

Existing eggsact/stegoeggo valid `FetchLimits` remain accepted.

If a downstream caller previously constructed invalid limits via struct literal, it now fails closed before I/O. Record this as intentional bug-fix behavior.

The current planned workspace 0.1.1 patch may include this corrective if publication has not occurred. If 0.1.1 has already been published by execution time, select the next patch version; never rewrite a published version.

## 10. Required tests

### Limits

- direct literal with metadata limit 0 -> rejected before transport work;
- direct literal above metadata maximum -> rejected;
- direct literal connect 0 -> rejected;
- direct literal total 0 -> rejected;
- direct literal connect > total -> rejected;
- valid literal still succeeds;
- constructor and `validate` produce consistent results;
- fixture and Eggfetch paths both reject the same invalid cases.

### Promotion

- hard-link failure leaves destination absent;
- preexisting destination preserved;
- raced-in destination preserved;
- link success + temp unlink success -> success/no residue;
- link success + injected temp unlink failure -> success (or explicit committed-warning state), destination bytes correct, temp residue owned/preserved;
- ordinary `Err` assertions verify destination absent unless the error type explicitly communicates committed/recovery state.

### Regression

- cancellation before promotion;
- artifact too large;
- body timeout;
- early disconnect;
- redaction sentinel tests;
- eggsact updater suite;
- stegoeggo updater suite.

## 11. Verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
./scripts/check-local.sh
```

Run Rust 1.89 checks/tests and hosted Linux/macOS/Windows lanes.

## 12. Documentation updates

Update:

- `crates/eggup-acquisition/README.md`;
- `crates/eggup-eggfetch/README.md`;
- `FetchLimits` rustdoc;
- promotion helper/seam rustdoc;
- `CHANGELOG.md`;
- acquisition roadmap;
- registry.

State explicitly that transport entry points revalidate public-field values.

State explicitly that successful no-clobber link creation commits a complete destination and redundant temp cleanup cannot retroactively make acquisition fail.

## 13. Acceptance criteria

M004 closes only when:

- invalid direct-literal `FetchLimits` cannot reach network/filesystem I/O;
- constructor and boundary validation share one rule set;
- no ordinary failure is returned after an operation has successfully created a complete destination;
- post-link cleanup failure has truthful observable semantics;
- all M003 safety tests remain green;
- eggsact/stegoeggo remain green;
- hosted CI passes;
- no medium-or-higher acquisition contract issue remains.

## 14. Stop conditions

Stop and write an ADR/API migration plan if:

- truthful promotion requires a breaking public result model;
- cross-platform no-clobber cannot retain the current hard-link guarantee;
- field privacy is required immediately and would break current published consumers;
- fixing promotion requires unsafe/FFI solely for this helper.

## 15. Closure evidence required

Record:

- exact implementation SHA;
- direct-literal invalid-limits matrix;
- proof validation occurs before I/O;
- promotion state table including injected unlink failure;
- destination/temp filesystem evidence;
- downstream eggsact/stegoeggo results;
- stable/MSRV/macOS/Windows CI;
- release-version disposition;
- unresolved findings.

## 16. Handoff notes

This corrective is a hard gate for new updater-bearing consumer migrations.

The optional lightweight acquisition milestone is renumbered to M005 and remains evidence-driven.
