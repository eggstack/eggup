# Service Lifecycle Milestone 008 — OperationDeadline Test Determinism Corrective

Status: implemented (see `plans/closure/service-lifecycle/008-status.md`)

Repository baseline: `510111fa67bafd0e188dad744eed4d224a015b76`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Original closed work affected:

- `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md`
- `plans/closure/service-lifecycle/003-status.md`

Related later closure evidence:

- `plans/closure/service-lifecycle/007-status.md`

Observed hosted failure:

- workflow run `36261671380`
- failing job: macOS tests
- failing test: `unix_tests::transition_deadline_rejects_zero_and_only_shrinks`

Primary class: invariant/corrective

## 1. Objective

Restore a green hosted baseline by removing scheduler-sensitive wall-clock timing from the private `OperationDeadline` unit test without weakening production deadline semantics.

Current production semantics are intentionally strict:

- zero or overlong transition budgets are rejected;
- one absolute `Instant` represents the operation deadline;
- every manager command receives only the remaining duration;
- once the absolute deadline is exhausted, `command_timeout()` fails closed with `ServiceError::Manager("transition deadline exhausted before manager command")`;
- no later command may receive a larger timeout than an earlier point in the same operation.

The current test attempts to prove the shrinking property by creating an 80 ms deadline, reading it once, sleeping 10 ms, then unconditionally unwrapping a second remaining timeout. On hosted macOS run `36261671380`, scheduler/runtime delay consumed the full 80 ms before the second read, so the production helper correctly returned the exhaustion error and the test failed.

M008 must make that proof deterministic. It must not widen deadlines, add tolerance to production expiry, retry an exhausted command, or turn exhaustion into success.

## 2. Readiness and dependencies

M008 is dependency-ready.

Evidence:

- current head `510111fa67bafd0e188dad744eed4d224a015b76` is docs-only relative to the previously green runtime head;
- Stable Linux passed fmt, clippy, full workspace tests, and docs in run `36261671380`;
- Rust 1.89 MSRV passed in that run;
- Windows archive/acquisition/curl/service tests and workspace check passed;
- only macOS failed, at the single timing-sensitive service test;
- the same production service source passed prior hosted macOS matrices;
- no runtime source changed between the prior green run and `510111f`.

The failure therefore demonstrates nondeterministic test construction, not evidence that the production absolute-deadline contract should change.

## 3. Current implementation evidence

Production helper:

~~~rust
#[derive(Debug, Clone, Copy)]
struct OperationDeadline(Instant);

impl OperationDeadline {
    fn new(timeout: Duration) -> Result<Self, ServiceError> {
        if timeout.is_zero() || timeout > MAX_TRANSITION_TIMEOUT {
            return Err(ServiceError::invalid("transition timeout out of range"));
        }
        Ok(Self(Instant::now() + timeout))
    }

    fn remaining(self) -> Option<Duration> {
        self.0
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
    }

    fn command_timeout(self) -> Result<Duration, ServiceError> {
        self.remaining().ok_or_else(|| {
            ServiceError::manager("transition deadline exhausted before manager command")
        })
    }
}
~~~

Flaky test:

~~~rust
let deadline = OperationDeadline::new(Duration::from_millis(80)).unwrap();
let first = deadline.command_timeout().unwrap();
std::thread::sleep(Duration::from_millis(10));
let second = deadline.command_timeout().unwrap();
assert!(second < first);
assert!(second <= Duration::from_millis(70));
~~~

Hosted macOS result:

~~~text
called Result::unwrap() on an Err value:
Manager("transition deadline exhausted before manager command")
~~~

That error is valid production behavior if the host is descheduled long enough.

## 4. Invariants that must not regress

- one absolute operation deadline governs the whole manager transition;
- manager command timeouts only shrink;
- expired deadlines fail before a new manager command starts;
- zero timeout remains invalid;
- `MAX_TRANSITION_TIMEOUT` remains enforced;
- no production scheduling slack/tolerance is added;
- no exhausted deadline is renewed or extended;
- systemd/launchd/cron/SCM lifecycle behavior is unchanged;
- public API is unchanged;
- service error text/bounds remain unchanged;
- Rust 1.89 remains supported;
- no new dependency;
- no first-party unsafe.

## 5. Scope and non-scope

### In scope

- make `OperationDeadline` time calculations deterministically testable with an explicit `Instant`;
- preserve production wrappers using `Instant::now()`;
- replace the wall-clock sleep test with exact injected-time assertions;
- prove pre-deadline shrink and at/after-deadline exhaustion;
- rerun all service/workspace tests;
- obtain fresh Stable/MSRV/macOS/Windows hosted evidence;
- reconcile service roadmap and registry after qualification.

### Explicitly out of scope

- changing transition timeout defaults;
- changing manager polling cadence;
- adding production grace periods;
- changing error categories;
- changing service lifecycle/disposition logic;
- changing archive M001d runtime implementation;
- Gregg migration;
- unrelated CI optimization.

## 6. Required production/test changes

### 6.1 Deterministic internal time seam

Refactor the private helper so time arithmetic can be evaluated at a supplied `Instant`.

Preferred shape:

~~~rust
impl OperationDeadline {
    fn new(timeout: Duration) -> Result<Self, ServiceError> {
        Self::new_at(Instant::now(), timeout)
    }

    fn new_at(now: Instant, timeout: Duration) -> Result<Self, ServiceError> {
        // same validation
        Ok(Self(now + timeout))
    }

    fn remaining(self) -> Option<Duration> {
        self.remaining_at(Instant::now())
    }

    fn remaining_at(self, now: Instant) -> Option<Duration> {
        self.0.checked_duration_since(now).filter(|d| !d.is_zero())
    }

    fn command_timeout(self) -> Result<Duration, ServiceError> {
        self.command_timeout_at(Instant::now())
    }

    fn command_timeout_at(self, now: Instant) -> Result<Duration, ServiceError> {
        self.remaining_at(now).ok_or_else(/* existing error */)
    }
}
~~~

Equivalent private factoring is acceptable, but:

- public API must not change;
- production callers must continue to use the real monotonic clock;
- tests must not depend on sleeping to prove arithmetic;
- no fake-clock framework/dependency is justified.

### 6.2 Deterministic unit assertions

Replace the scheduler-sensitive test with an explicit start instant:

~~~text
start
  + 0 ms  -> remaining exactly 80 ms
  +10 ms  -> remaining exactly 70 ms
  +79 ms  -> remaining exactly 1 ms
  +80 ms  -> exhausted/error
  +81 ms  -> exhausted/error
~~~

Also retain zero-timeout rejection and over-max rejection coverage.

The test should prove monotonic shrink by arithmetic, not elapsed wall-clock timing.

### 6.3 Preserve runtime integration coverage

Existing manager transition tests must remain unchanged unless a concrete independent defect is found. M008 is not permission to replace actual timeout/process tests with fake time.

## 7. Ordered work packages

1. Add deterministic `Instant`-parameterized private helper(s).
2. Keep existing production wrappers and error behavior byte-for-byte where practical.
3. Rewrite only the flaky `transition_deadline_rejects_zero_and_only_shrinks` test to use injected instants.
4. Add exact boundary assertions for 1 ms remaining and expiry.
5. Run the focused service test repeatedly locally.
6. Run full service/workspace Stable + Rust 1.89 checks.
7. Push and require a fresh hosted Stable/MSRV/macOS/Windows matrix.
8. Record closure with the original failed run `36261671380` and the new green run.
9. Reconcile roadmap/registry and rebase/unblock Archive M001d only after the baseline is green.

## 8. Failure, restart, cancellation, and contention semantics

No production failure semantics may change.

An exhausted deadline remains a hard pre-command manager error. M008 must not retry, renew, or silently clamp an exhausted deadline to a positive duration.

Test infrastructure must not introduce sleeps, polling, or retries that can hide a real deadline failure.

## 9. Compatibility and migration

No public API or consumer migration.

All new helpers remain private to `eggup-service`. No dependency or feature changes.

## 10. Required tests

At minimum:

- zero transition budget rejected;
- budget above `MAX_TRANSITION_TIMEOUT` rejected;
- deterministic exact remaining value at creation;
- deterministic exact shrink at multiple supplied instants;
- exactly-at-deadline returns exhaustion;
- after-deadline returns exhaustion;
- existing manager deadline tests remain green;
- full `eggup-service` suite green;
- full workspace all-target/all-feature suite green;
- Rust 1.89 all-target check green;
- macOS hosted lane executes and passes the formerly flaky test;
- Windows service tests remain green.

No test in this corrective may require a short real-time sleep to prove the `OperationDeadline` arithmetic invariant.

## 11. Required verification commands

~~~text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --lib transition_deadline_rejects_zero_and_only_shrinks --locked
cargo test -p eggup-service --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
git diff --check
~~~

Recommended local repeat check for the focused deterministic test:

~~~text
for i in 1 2 3 4 5 6 7 8 9 10; do
  cargo test -p eggup-service --lib transition_deadline_rejects_zero_and_only_shrinks --locked || exit 1
done
~~~

Hosted closure requires:

- Stable Linux full checks;
- Rust 1.89 MSRV all-target check;
- macOS full workspace tests;
- Windows archive/acquisition/curl/service tests + workspace all-target check.

M008 MUST NOT close on a rerun of the old flaky test without a deterministic test correction.

## 12. Documentation updates

Update:

- `plans/subsystems/service-lifecycle-roadmap.md`;
- `plans/registry.md`;
- `plans/closure/service-lifecycle/008-status.md`.

A changelog entry is optional because runtime behavior/public API does not change; if added, describe this as test/qualification hardening, not a service behavior change.

## 13. Acceptance criteria

M008 closes only when:

- `OperationDeadline` production semantics are unchanged;
- the shrinking/expiry unit proof no longer depends on scheduler timing;
- exact injected-time boundary assertions pass;
- the original macOS failure is recorded as superseded evidence;
- a fresh hosted Stable/MSRV/macOS/Windows matrix is green;
- current main is again green;
- no medium-or-higher service finding remains open;
- Archive M001d is rebased to the new green head before runtime handoff.

## 14. Stop conditions

Stop and write a broader service corrective if:

- deterministic injected-time tests reveal production arithmetic is wrong;
- any manager transition can start after the absolute deadline;
- fixing the test requires widening runtime deadlines;
- a platform-specific `Instant` behavior invalidates the current absolute-deadline model;
- hosted macOS continues failing after the sleep-based assertion is removed.

Do not classify a genuine runtime failure as test flakiness merely to restore CI.

## 15. Closure evidence required

Record:

- implementation SHA;
- exact private helper diff;
- proof public API/dependencies are unchanged;
- original failure run `36261671380`;
- focused deterministic test results;
- full local command results;
- new hosted workflow run ID and per-platform job conclusions;
- service roadmap disposition;
- Archive M001d rebase/unblock SHA;
- unresolved findings by severity.

## 16. Handoff notes

Keep this corrective small.

The intended proof is not “80 ms usually lasts long enough.” The intended proof is: for a fixed absolute deadline and explicit monotonic observation instant, remaining time is exactly the positive difference and becomes an error at exhaustion.

Do not change production timeout policy to make a unit test pass.
