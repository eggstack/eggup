# Acquisition Transport Milestone 008 — Sub-Second Deadline Truthfulness Corrective

Status: ready for handoff

Repository baseline: `ea51fe12a7c9120028b727eb5e40411e9b10f8e2`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Original work corrected by this pass:

- `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`
- `plans/closure/acquisition-transport/007-status.md`

Long-term requirements:

- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/000-long-term-specification.md#18-security-model`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant/corrective

## 1. Objective

Make the external curl adapter honor caller-provided sub-second connect and total deadlines without silently rounding them upward to whole seconds.

M007 correctly removed the pathname reopen race and made artifact byte bounds finite, but `eggup-curl` still serializes effective curl deadlines through `ceil_secs(Duration)`. A request-level connect ceiling such as 100 ms is therefore passed to curl as one second. This contradicts `FetchLimits::effective` documentation and the acquisition invariant that an adapter may tighten but never extend a caller deadline.

The corrective also removes the associated one-second timeout-attribution slack while preserving all M007 process cleanup, redaction, byte-bound, status-classification, and fallback semantics.

## 2. Why this milestone is ready

The defect is directly visible on current main:

- `build_curl_args` calls `ceil_secs(eff_connect)` and `ceil_secs(eff_total)`;
- `ceil_secs` rounds every positive fractional second upward;
- `classify_curl_result` then permits an additional one-second connect-phase attribution window because of that rounding.

Curl's `--connect-timeout` and `--max-time` accept decimal seconds using a dot decimal separator, so whole-second rounding is not required by the external tool.

M007 is otherwise conditionally closed with its three production defects corrected. This plan does not reopen the Windows hosted loopback limitation.

## 3. Current implementation evidence

At baseline `ea51fe12a7c9120028b727eb5e40411e9b10f8e2`:

- `FetchLimits::effective` computes `min(request, adapter ceiling)`;
- its public documentation says an adapter may never silently extend a caller deadline;
- the parent process enforces the effective total wall-clock deadline independently;
- curl is passed integer whole-second values produced by `ceil_secs`;
- connect-phase enforcement is therefore looser than the effective caller value for every sub-second connect deadline;
- exit-code 28 classification compensates with `eff_connect + 1 second`.

No release/source fallback, transport composition, or archive/service behavior is involved.

## 4. Invariants that must not regress

- effective connect and total deadlines are never greater than caller-provided ceilings;
- adapter ceilings may only tighten caller ceilings;
- parent total-wall-clock enforcement remains authoritative;
- curl cancellation/timeout kills and reaps the child before return;
- body/status reader threads are joined before terminal return;
- exact 404 remains `NotFound`;
- 5xx/TLS/connect/partial failures remain hard failures;
- artifact byte limits remain finite and independently enforced by Eggup;
- curl receives no Eggup temporary pathname;
- diagnostics remain bounded/redacted;
- composition fallback semantics remain unchanged;
- no shell execution or ambient curl config is introduced.

## 5. Scope

### In scope

- replace whole-second ceiling serialization for curl deadlines;
- use locale-independent decimal seconds accepted by curl;
- preserve exact or conservatively non-extending `Duration` semantics;
- remove `ceil_secs` if no longer used;
- remove the artificial one-second timeout-attribution slack;
- add deterministic argument-construction tests for sub-second and fractional-second values;
- add supported-host runtime timeout tests proving sub-second connect/total behavior where deterministic;
- preserve the current Windows loopback limitation honestly;
- update curl docs, acquisition roadmap, registry, and M007 closure addendum/relationship as appropriate;
- write M008 closure evidence.

### Explicitly out of scope

- changing the `FetchLimits` public API;
- changing default deadline values;
- retry policy;
- release/source fallback;
- changing curl executable discovery;
- changing proxy policy;
- changing body streaming/file-handle ownership;
- Windows hosted loopback infrastructure repair;
- archive/service changes;
- package publication.

## 6. Required production changes

### 6.1 Decimal duration serialization

Replace `ceil_secs` with a locale-independent serializer for positive Rust `Duration` values.

Required properties:

- use `.` as decimal separator regardless of host locale;
- represent whole seconds without unnecessary fractional text;
- represent sub-second values without rounding upward beyond the input duration;
- support nanosecond-resolution Rust durations or a deliberately documented lower precision that truncates/tightens rather than extends;
- never emit zero for a positive duration merely because of precision truncation;
- produce text accepted by curl on supported platforms.

A decimal formatter built from integer seconds + fractional nanoseconds is preferred over locale-sensitive floating-point formatting.

Examples that must remain semantically at or below the caller ceiling:

~~~text
100 ms   -> 0.1
250 ms   -> 0.25
1.5 s    -> 1.5
2 s      -> 2
~~~

Exact textual spelling may differ if equivalently non-extending.

### 6.2 Curl arguments

Pass the serialized effective durations to:

- `--connect-timeout`;
- `--max-time`.

The parent process must continue to enforce `eff_total` using its Rust `Instant` deadline.

Do not add a larger minimum such as one second. If curl cannot meaningfully represent a positive duration this small, fail validation or tighten to the smallest nonzero representable duration rather than widening the caller ceiling.

### 6.3 Timeout phase evidence

Remove the current one-second classification slack.

Exit code 28 may still need bounded phase attribution because curl does not expose a distinct connect-timeout exit code. The classification rule may use a small scheduler/polling tolerance solely to label an already-enforced timeout, but that tolerance MUST NOT extend the actual child lifetime or the arguments passed to curl.

Prefer one of:

1. deterministic phase evidence from already available curl timing/write-out data if it can be captured boundedly and reliably on timeout; or
2. elapsed-time attribution using a tolerance derived from `POLL_INTERVAL` plus a small documented scheduling margin.

Do not preserve a one-second semantic allowance.

## 7. Ordered work packages

1. Add unit tests pinning current whole-second widening as a failing case.
2. Implement locale-independent non-extending duration serialization.
3. Replace `ceil_secs` in curl argument construction.
4. Remove one-second timeout-attribution slack.
5. Add argument tests for sub-second, fractional, and whole-second deadlines.
6. Add deterministic supported-host runtime timeout tests where feasible.
7. Re-run M007 curl cancellation/body/status/no-clobber/redaction/fallback suites.
8. Run full workspace/MSRV/platform qualification.
9. Update docs/roadmap/registry and write M008 closure evidence.

## 8. Failure, cancellation, restart, and contention semantics

No new retry or recovery state is introduced.

A deadline expiry still:

- kills the child if it is running;
- waits/reaps it;
- joins body/status readers;
- removes only the owned incomplete temporary artifact;
- returns a typed timeout without transport fallback.

If a sub-second duration cannot be represented without widening, reject it or tighten it. Never silently extend it.

## 9. Compatibility and migration

No public API signature change is expected.

Callers using sub-second deadlines receive stricter, now-truthful behavior. This is a correctness change: connections that previously received up to approximately one second may now time out at the requested ceiling.

Whole-second callers should observe no meaningful behavior change.

No consumer repository migration is required.

## 10. Required tests

At minimum:

- 100 ms serializes to a nonzero value <= 100 ms;
- 250 ms serializes <= 250 ms;
- 1.5 s serializes <= 1.5 s;
- whole-second values remain exact;
- no locale-dependent comma decimal separator;
- effective adapter ceiling remains `min(request, adapter)`;
- generated `--connect-timeout` and `--max-time` arguments do not exceed effective durations;
- supported-host sub-second total timeout kills/reaps child;
- supported-host sub-second connect timeout does not wait approximately one whole second;
- exact 404/5xx/status behavior remains unchanged;
- cancellation and oversize tests remain green;
- no temp residue after timeout;
- Windows portable process/argument tests compile and run even if live loopback remains unavailable.

## 11. Required verification commands

~~~text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-curl --all-targets --locked
cargo test -p eggup-acquisition --all-targets --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh
git diff --check
~~~

Run hosted Stable/MSRV/macOS/Windows lanes. Preserve the existing truthful distinction between Windows portable curl tests and unavailable hosted live-loopback evidence.

## 12. Documentation updates

Update:

- `crates/eggup-curl/README.md`;
- root and/or crate changelog Unreleased section;
- acquisition roadmap;
- registry;
- M007 closure addendum only if useful to cross-reference the discovered follow-up;
- M008 closure record.

## 13. Acceptance criteria

M008 closes only when:

- curl deadline arguments never exceed effective caller/adapter durations;
- sub-second connect deadlines are no longer widened to one second;
- `ceil_secs` and the one-second attribution workaround are gone or demonstrably no longer capable of deadline widening;
- timeout/cancel child-reap semantics remain green;
- M007 body/status/file-handle and finite-byte-bound behavior remains intact;
- hosted platform qualification is green within the already documented Windows loopback scope;
- no medium-or-higher acquisition timing finding remains open.

## 14. Stop conditions

Stop and write a different corrective/ADR if:

- supported curl versions on Eggup's platform floor cannot accept decimal deadline values;
- exact deadline semantics require shell invocation or locale mutation;
- a proposed solution widens the caller duration to gain reliability;
- phase attribution requires unbounded/raw curl diagnostics;
- fixing this requires changing the public `FetchLimits` model.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- before/after curl argument examples;
- exact duration serialization rules;
- proof that no positive input is widened;
- timeout classification rule and tolerance, if any;
- supported-host sub-second runtime evidence;
- M007 regression results;
- Stable/MSRV/macOS/Windows results;
- unresolved findings by severity.

## 16. Handoff notes

This is a narrow truthfulness corrective. Do not rework M007's body streaming architecture.

The key invariant is simple: `FetchLimits::effective` returns a ceiling, not a suggestion. The external adapter must never turn a smaller caller duration into a larger curl duration.
