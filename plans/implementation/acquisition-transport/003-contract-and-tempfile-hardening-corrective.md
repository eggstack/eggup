# Acquisition Transport Milestone 003 — Contract and Temporary-File Hardening Corrective

Status: implemented (closed; see `plans/closure/acquisition-transport/003-status.md`)

Repository baseline: `8f6ce48cda5bdeb593939077bcca452cdd5f2800`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Corrects post-closure findings from:

- `plans/closure/acquisition-transport/001-status.md`
- `plans/closure/acquisition-transport/002-status.md`

Consumer evidence:

- `plans/closure/consumer-adoption/001-status.md`
- `plans/closure/consumer-adoption/002-status.md`

Long-term requirements:

- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant / corrective

## 1. Objective

Reconcile the public acquisition contract with the actual Eggfetch adapter and harden artifact temporary-file creation/promotion before service-aware or bundle consumers adopt Eggup.

The corrective has three concrete goals:

1. make caller-supplied `FetchLimits` time bounds authoritative for every fetch, subject only to stricter adapter ceilings;
2. replace predictable/open-existing temporary-file creation with exclusive, owner-private transaction-owned files and no-clobber promotion semantics;
3. prove that upstream transport/proxy errors cannot reintroduce credential-bearing URLs or proxy secrets into diagnostics after Eggup redaction.

## 2. Readiness and dependencies

M001 and M002 are closed and two independent consumers use the published 0.1.0 seam.

This plan is ready now because the discrepancies are visible in the public 0.1.0 API and implementation:

- `FetchLimits::{connect_timeout,total_timeout}` are part of every seam call;
- `eggup-eggfetch` currently builds one client from `EggfetchConfig` and does not apply the per-call timeout values;
- fixture and Eggfetch artifact writers create generated sibling names with ordinary create/truncate semantics;
- generic upstream error strings are embedded after only the request URL itself is redacted.

No core/service work is required to implement this corrective.

## 3. Current evidence

At the baseline:

- `FetchLimits::new` validates positive timeouts but does not require connect <= total;
- metadata and artifact fetches use request byte limits correctly;
- `EggfetchConfig` owns 10 s connect / 120 s total defaults;
- `EggfetchTransport::fetch_metadata` and `fetch_artifact` execute through the prebuilt client and therefore use adapter timeouts, not caller limits;
- the source comments currently say per-fetch bounds are authoritative, creating a contract/implementation mismatch;
- fixture transport uses `File::create` for `.eggup-acquire-*.part`;
- Eggfetch transport uses `tokio::fs::File::create` for `.eggup-eggfetch-<pid>-<nanos>.part`;
- both later rename the temporary sibling to the requested destination;
- Unix and Windows differ when rename targets already exist;
- upstream `eggfetch_core::Error` display strings are appended to Eggup diagnostics without proof that they are already credential-safe.

The current consumers happen to configure compatible timeout values, so this was not exposed by their migrations.

## 4. Invariants that must not regress

- the exact caller-selected URL remains the only acquisition target;
- no adapter chooses fallback or release policy;
- request byte and time limits are never weakened by adapter defaults;
- adapter-level configuration MAY impose stricter ceilings;
- no partial artifact is promoted as success;
- temporary acquisition state is exclusively created and owned by the current operation;
- no unexpected existing destination is overwritten as a side effect of acquisition;
- cleanup never removes a file whose ownership cannot be proven;
- secrets in URL userinfo, query strings, or proxy configuration never appear in returned/displayed diagnostics;
- `eggup-core` remains free of transport dependencies;
- no unsafe code or implicit privilege escalation.

## 5. Scope

### In scope

- `FetchLimits` time-bound semantics;
- effective timeout derivation in `eggup-eggfetch`;
- connect/total relationship validation;
- deterministic timeout fixture coverage;
- exclusive temporary sibling creation;
- Unix owner-private temp-file permissions;
- no-clobber artifact destination policy;
- cleanup ownership checks;
- cross-platform promotion behavior;
- diagnostic redaction of upstream transport/proxy error details;
- package/docs/changelog/compatibility evidence;
- migration notes for existing eggsact/stegoeggo users.

### Explicitly out of scope

- retries;
- mirrors;
- release discovery;
- new HTTP protocol versions;
- changing TLS roots;
- changing proxy default from explicit/disabled;
- lightweight/curl adapter;
- async public acquisition trait redesign;
- core transaction changes;
- automatic crates.io publication.

## 6. Required production changes

### A. Authoritative effective timeout model

Define and document one rule:

```text
effective connect timeout = min(request connect, adapter connect ceiling)
effective total timeout   = min(request total, adapter total ceiling)
```

A stricter adapter is allowed; it may never silently extend a caller deadline.

Require:

- non-zero connect and total values;
- connect <= total, unless Eggfetch semantics prove a different relationship is required and that decision is documented;
- the effective total deadline covers response headers plus body streaming;
- both metadata and artifact operations use the same derivation.

Prefer an Eggfetch request-level timeout override if 0.2.x exposes one.

If Eggfetch only supports client-level timeouts, build/cache a client keyed by the effective timeout policy or construct a bounded short-lived client. Do not fake enforcement with a post-hoc elapsed-time check after the network operation already exceeded the caller's deadline.

Do not introduce a second timeout system that races Eggfetch cancellation unless it is required and tested.

### B. Timeout error truthfulness

A caller deadline must produce `AcquisitionError::Timeout`, not an arbitrary transport error when Eggfetch exposes enough information to classify it.

Tests must prove both directions:

- request stricter than adapter;
- adapter stricter than request.

Connect-phase testing may use an injectable seam if deterministic real-network connect timeout is impractical. Total-body timeout must be exercised against a local fixture.

### C. Exclusive temporary artifact creation

Replace ordinary `File::create` / `tokio::fs::File::create` on generated sibling names.

The creation primitive MUST:

- fail if the candidate path already exists;
- retry with a fresh name on collision within a small bound;
- never follow a precreated symlink;
- use owner-private permissions on Unix, normally `0600`;
- keep the file in the exact destination parent so final promotion stays on the same filesystem;
- record enough identity to remove only the operation's own temporary file.

A PID/timestamp/nonce name is acceptable only when combined with `create_new`; unpredictability alone is not ownership.

### D. No-clobber destination contract

Acquisition is staging, not live replacement.

Define `fetch_artifact` to require that its destination be absent at promotion time. An unexpected existing destination must return an explicit error rather than relying on platform-specific rename-overwrite behavior.

Implement a race-safe no-clobber promotion strategy.

If std-only cross-platform primitives cannot provide the documented guarantee, stop and evaluate either:

- changing the seam to accept an already exclusively-created destination/file handle; or
- a small, well-audited dependency that provides no-replace persist semantics.

Do not silently fall back to "pre-check then rename" if another process can win the race.

### E. Partial-failure cleanup

Cancellation, timeout, body-limit overflow, early disconnect, write failure, and failed promotion must:

- leave no successful destination;
- remove only the owned temp file;
- preserve unexpected foreign files;
- report cleanup failure separately if evidence must remain.

Do not use broad prefix-based cleanup of unrelated siblings.

### F. Diagnostic redaction

Audit every path that incorporates:

- request URL;
- Eggfetch error display/debug text;
- proxy parse/routing error;
- redirect target;
- TLS error context.

Returned `AcquisitionError` strings must not expose:

- URL username/password;
- query parameters;
- proxy credentials.

Prefer structured/category text over echoing upstream errors when upstream redaction cannot be proven.

Add regression strings containing sentinel secrets and assert they are absent from `Display` and `Debug`-adjacent public diagnostics used by callers.

### G. Existing consumer compatibility

Eggsact and stegoeggo already use explicit `FetchLimits` and `EggfetchConfig`.

Document whether the corrected effective timeout changes their actual runtime bounds.

Run their updater-focused suites against path dependencies or a locally patched package before release qualification.

No consumer workaround should be added for a generic acquisition defect.

## 7. Ordered work packages

A. Specify and test effective timeout semantics in `eggup-acquisition`.

B. Apply those semantics in `eggup-eggfetch` using real Eggfetch timeout enforcement.

C. Build a shared internal exclusive-temp/no-clobber promotion helper usable by fixture and Eggfetch implementations without leaking policy into core.

D. Add cancellation/cleanup/collision/race regression tests.

E. Redaction audit and secret-sentinel tests.

F. Run downstream eggsact/stegoeggo compatibility.

G. Update package docs/changelog and determine patch-release requirements without publishing automatically.

## 8. Failure, cancellation, restart, and contention semantics

- cancellation before creation: no filesystem state;
- cancellation during streaming: owned temp removed, destination absent;
- timeout during streaming: same;
- temp-name collision: retry without touching the existing entry;
- destination appears before promotion: fail no-clobber and preserve both the foreign destination and owned temp cleanup semantics;
- cleanup failure: return an actionable I/O/recovery error and preserve only evidence whose ownership is known;
- process crash: ordinary filesystem temp residue may remain; no crash journal is claimed. Any future scavenger requires separate ownership design.

## 9. Compatibility and migration

This is a bug-fix corrective to a published 0.1.0 contract.

Before closure:

- determine whether package versioning remains lockstep across all workspace crates or only acquisition crates are released;
- record exact semver/version decision;
- do not republish 0.1.0;
- if a corrected package is published later, update eggsact/stegoeggo through their normal dependency process.

Avoid changing method signatures unless no-clobber promotion cannot be made truthful with the existing path-based seam.

A signature change triggers explicit migration notes and requalification of both adopters.

## 10. Required tests

### Timeout contract

- request 50 ms total / adapter 5 s -> request bound wins;
- request 5 s / adapter 50 ms -> adapter bound wins;
- connect > total rejected or explicitly normalized according to documented policy;
- metadata body stall;
- artifact body stall after partial data;
- timeout never maps to NotFound/fallback.

### Temporary-file ownership

- precreate predicted/candidate temp path -> no truncation/follow;
- symlink collision -> no target modification;
- repeated collisions -> bounded failure;
- Unix mode is owner-private;
- cancellation removes only owned temp;
- early disconnect removes only owned temp.

### Destination no-clobber

- destination absent -> success;
- destination exists before call -> hard failure;
- destination appears during fetch before promotion -> hard failure, existing bytes preserved;
- behavior equivalent on Linux/macOS/Windows fixture logic.

### Redaction

- request URL userinfo sentinel absent from error;
- query token sentinel absent;
- proxy password sentinel absent;
- redirect/error sentinel absent if upstream error contains URL text.

### Downstream

- eggsact updater-focused suite;
- stegoeggo updater-focused suite;
- no release/fallback behavior change.

## 11. Required verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-acquisition --locked
cargo tree -p eggup-eggfetch --locked
cargo package -p eggup-acquisition --locked --allow-dirty
cargo package -p eggup-eggfetch --locked --allow-dirty
./scripts/check-local.sh
```

Run Rust 1.89 variants for check/clippy/test/doc. Record hosted stable/MSRV/macOS/Windows-check results.

Run the two current consumer updater test sets against the corrective.

## 12. Documentation updates

Update:

- `crates/eggup-acquisition/README.md`;
- `crates/eggup-eggfetch/README.md`;
- seam rustdoc;
- `CHANGELOG.md`;
- acquisition roadmap and registry.

State the effective timeout formula and no-clobber destination requirement explicitly.

## 13. Acceptance criteria

M003 closes only when:

- every request time limit is actually enforced or tightened, never extended;
- fixture and Eggfetch adapters obey the same timeout contract;
- acquisition temporary files are exclusive and private;
- a preexisting or raced-in destination cannot be overwritten;
- partial failures do not promote artifacts;
- diagnostics do not expose credential sentinel values;
- eggsact/stegoeggo compatibility stays green;
- core remains transport-free;
- no medium-or-higher acquisition issue remains.

## 14. Stop conditions

Stop and write an ADR or narrower design plan if:

- truthful no-clobber promotion requires a breaking acquisition-seam redesign;
- Eggfetch cannot enforce per-request/effective deadlines without an upstream change;
- a platform needs unsafe/FFI solely for rename-no-replace semantics;
- redaction requires guessing about opaque upstream strings rather than structuring errors;
- the corrective materially changes release/fallback authority.

## 15. Closure evidence required

The closure record must include:

- before/after timeout semantics;
- exact effective-timeout tests;
- temp collision/symlink/no-clobber matrix;
- Unix permissions evidence;
- redaction sentinel matrix;
- downstream eggsact/stegoeggo results;
- package/version decision;
- dependency tree;
- hosted CI results;
- unresolved findings and severity.

## 16. Handoff notes

This is the primary next handoff and a gate for additional updater-bearing consumer migrations.

Do not implement the optional lightweight/curl adapter in this milestone. The roadmap moves that evidence-driven work to M004.
