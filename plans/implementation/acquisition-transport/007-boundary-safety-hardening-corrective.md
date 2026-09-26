# Acquisition Transport Milestone 007 — Boundary Safety Hardening Corrective

Status: implemented

Repository baseline: `ee1476ef2a9e0d5569d6dc2469e780a4435cc426`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Original work corrected by this pass:

- acquisition M001-M006 contracts and closure evidence, especially M003-M006 hardening.

Long-term requirements:

- `plans/000-long-term-specification.md#8-acquisition-safety`
- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`
- `plans/000-long-term-specification.md#19-dependency-and-footprint-policy`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Primary class: invariant/corrective

## 1. Objective

Close three post-M006 acquisition-boundary defects before broader consumer migration:

1. fixed-byte diagnostic truncation can panic when a UTF-8 code point straddles the limit;
2. `FetchLimits::max_artifact_bytes = None` permits an effectively unbounded artifact byte count despite the normative "all network acquisition is bounded" requirement;
3. `eggup-curl` exclusively creates an output file, closes it, then passes the pathname to curl for reopen/truncate, reintroducing a pathname race that the exclusive-open step was intended to avoid.

Preserve existing release/fallback policy boundaries and curl-vs-Eggfetch footprint choice.

## 2. Why this corrective is ready

The defects are directly visible at the reviewed main baseline.

Rust's `String::truncate` panics if the requested byte length is not a character boundary. Acquisition/curl/eggfetch diagnostic helpers use fixed byte truncation on arbitrary strings.

The curl adapter currently creates a private exclusive temp, drops the file handle, and invokes curl with `--output <path>`. A foreign actor with sufficient directory mutation authority can replace the directory entry between close and reopen. The native Eggfetch adapter keeps the exclusive file handle and therefore does not share this gap.

Current curl supports routing `--write-out` output to stderr, allowing an implementation where response-body bytes are streamed over stdout into Eggup's already-open owned file while bounded status data is captured separately. Exact implementation may differ, but M007 must preserve the open-handle/no-clobber property.

`FetchLimits` already has validation at every transport boundary, so tightening the finite-artifact contract is localized and can be migrated deliberately.

## 3. Current implementation evidence

Affected production helpers include:

- `eggup-acquisition::bound_detail`, `redact_url`, and hidden scrub helpers;
- `eggup-eggfetch::bound`;
- `eggup-curl::bound`.

`FetchLimits.max_artifact_bytes` is currently `Option<u64>`; adapters skip byte-count enforcement when it is `None`.

`eggup-curl` metadata and artifact paths both pre-create an exclusive temp and then close it before curl opens the pathname.

M005/M006 transport composition and Windows qualification otherwise remain valid and must not be reopened unnecessarily.

## 4. Invariants that must not regress

- exact caller URL remains authoritative;
- `NotFound` remains terminal for the exact URL;
- no release/source fallback is added;
- default composition falls back only on `Unavailable`;
- all metadata/artifact acquisition is bounded in time and bytes;
- curl and Eggfetch enforce the same caller byte/time ceilings;
- downloaded bytes never execute in the transport;
- output files are exclusive, owner-private, and never clobber existing paths;
- cancellation/timeout kills and reaps curl before return;
- raw curl/proxy diagnostics do not leak credentials;
- curl-only consumers do not gain Eggfetch/TLS dependencies;
- lower core remains transport-free.

## 5. Scope

### In scope

- central UTF-8-safe bounded-string helper semantics across acquisition crates;
- regression tests with multibyte boundary-straddling diagnostics;
- make finite artifact byte budget mandatory at validated transport entry;
- migrate known Eggup consumers/tests/examples using `None`;
- preserve a convenient finite default;
- close the curl path-reopen race for metadata and artifacts;
- stream curl body into an already-open Eggup-owned file or an equivalently strong handle-preserving design;
- capture/parse HTTP status without buffering response bodies in memory;
- independently enforce byte limit while streaming even if curl's own size option is retained as defense-in-depth;
- bounded status/control output;
- cross-platform tests including Windows;
- public API migration notes if `FetchLimits` changes.

### Explicitly out of scope

- release discovery;
- automatic retries;
- new fallback sources;
- Gregg migration;
- archive extraction;
- service lifecycle;
- authenticity/signatures;
- unrelated Eggfetch feature changes;
- asynchronous public acquisition API;
- publishing a release.

## 6. Required production changes

### 6.1 UTF-8-safe bounded diagnostics

Replace direct fixed-byte `String::truncate(N)` on arbitrary text with a helper that:

- returns text no larger than the configured byte bound;
- backs up to a valid UTF-8 boundary;
- never panics;
- preserves existing redaction-before-display rules.

Audit all acquisition/curl/eggfetch production truncation, including URL redaction and upstream-text scrub defense-in-depth.

### 6.2 Finite artifact budget

Remove or reject the unbounded artifact state.

Preferred pre-1.0 direction:

- make the validated `FetchLimits` artifact bound a finite `u64` rather than `Option<u64>`;
- keep a finite default;
- make fields private if doing so can be migrated cleanly, with constructors/accessors preserving explicit validation.

If changing field privacy is too large for one corrective, it may remain public temporarily, but `None`/zero-equivalent unbounded behavior must not survive M007.

Audit eggsact, stegoeggo, eggsearch, CodeGG integration fixtures, examples, and `eggup-eggpack::bind_requests`. Record whether any downstream intentionally relied on unbounded artifact bytes.

### 6.3 Curl body/status separation

Do not give curl a pathname that Eggup has exclusively created and then relinquished.

Preferred architecture:

1. Eggup creates and retains an exclusive owner-private output file handle.
2. Curl writes response body to stdout.
3. Parent continuously drains stdout into the already-open file while:
   - enforcing caller byte cap;
   - checking cancellation;
   - enforcing total deadline;
   - killing/reaping child on failure.
4. HTTP status is emitted on a separate bounded channel using curl write-out routing or an equivalent deterministic mechanism.
5. Curl's ordinary raw error output remains suppressed or is never incorporated into returned diagnostics.
6. Successful body/status completion flushes the file before no-clobber promotion.

The implementation must avoid stdout/stderr pipe deadlock by draining required child streams while the process is running rather than only after exit.

Retain protocol, redirect, proxy, `--disable`, timeout, and environment-clearing policy.

### 6.4 Metadata

Metadata may stream into the same retained temporary-file mechanism before a bounded read into memory. Do not regress the metadata memory cap.

### 6.5 Promotion

Keep existing no-clobber promotion semantics. Successful destination creation remains the commit point; redundant temp-link cleanup remains best effort after commit.

## 7. Ordered work packages

1. Add UTF-8 boundary regression tests and safe bounded-text helper.
2. Audit/replace acquisition/eggfetch/curl truncation sites.
3. Add finite-artifact-budget negative tests at seam and adapters.
4. Migrate `FetchLimits` API/fixtures/known consumers to a mandatory finite bound.
5. Add a deterministic test reproducing pathname replacement between exclusive creation and external writer open, or otherwise pin the old vulnerability structurally.
6. Refactor curl execution to stream response bytes through a retained Eggup-owned file handle.
7. Add bounded concurrent status/control capture.
8. Re-run timeout/cancel/404/5xx/TLS/proxy/oversize/no-clobber tests.
9. Run Windows runtime tests for curl fixture path where supported.
10. Record dependency/footprint regression.
11. Update docs/roadmap/registry and closure.

## 8. Failure, cancellation, restart, and contention semantics

A byte-limit exceed, cancellation, timeout, child failure, malformed status, or stream/write error must kill/reap the curl child before return and remove only the owned incomplete temp.

A destination that already exists or races in is preserved.

No failure after a successful no-clobber promotion may be reported as an ordinary fetch failure merely because redundant temp cleanup failed.

Transport fallback semantics remain unchanged.

## 9. Compatibility and migration

A finite-artifact API change is source-breaking but allowed pre-1.0 only with explicit migration notes.

Known consumers should normally use their current finite caps or the existing finite default. Do not silently choose an enormous sentinel for callers that intentionally supplied `None`; record and resolve each such use.

`eggup-eggpack::bind_requests` should continue tightening a caller cap to the exact manifest size and must never widen it.

## 10. Required tests

At minimum:

- multibyte UTF-8 at 255/256/511/512 byte diagnostic boundaries;
- long credential-bearing URL remains redacted and does not panic;
- artifact fetch with no finite cap cannot be constructed/accepted;
- exact bound success and bound+1 failure for Eggfetch and curl;
- curl retained-handle body streaming;
- curl status capture separated from body;
- status/control capture bound;
- cancellation during stream kills/reaps child;
- total timeout during stream kills/reaps child;
- 404 remains `NotFound`;
- 5xx remains hard failure;
- partial body remains hard failure;
- existing/raced destination preserved;
- no-clobber promotion remains truthful;
- proxy credentials/raw curl output absent from diagnostics;
- composed fallback matrix unchanged;
- Windows compile + runtime fixture coverage where supported.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo test -p eggup-acquisition --locked
cargo test -p eggup-eggfetch --locked
cargo test -p eggup-curl --locked
cargo tree --workspace --locked
./scripts/check-local.sh
git diff --check
```

Run hosted Stable/MSRV/macOS/Windows. Windows evidence should execute portable acquisition tests rather than relying exclusively on `cargo check`.

## 12. Documentation updates

Update:

- acquisition README;
- eggfetch README if public limit semantics change;
- curl README with retained-handle streaming semantics;
- acquisition architecture deep dive;
- public migration note for `FetchLimits`;
- source roadmap;
- registry;
- closure record.

## 13. Acceptance criteria

M007 closes only when:

- arbitrary Unicode diagnostics cannot panic at fixed bounds;
- every artifact fetch has a finite byte cap;
- curl never relinquishes an exclusive pathname and then asks curl to reopen it for body output;
- curl body streaming is independently byte bounded;
- timeout/cancellation kill and reap before return;
- no-clobber/redaction/fallback semantics remain unchanged;
- known consumer migrations are enumerated;
- hosted cross-platform matrix is green;
- no high- or medium-severity acquisition finding remains open.

## 14. Stop conditions

Stop and split/ADR if:

- a mandatory finite cap requires introducing release/source policy into acquisition;
- preserving curl portability requires shell execution;
- curl body/status separation cannot be implemented without unbounded pipe buffering;
- fixing the race would require platform-specific unsafe code;
- downstream code genuinely needs unbounded network artifacts;
- the change materially alters transport-fallback policy.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- affected truncation-site audit;
- old/new `FetchLimits` API and downstream migration inventory;
- pathname-race regression evidence;
- curl process/pipe lifecycle tests;
- exact byte-bound matrix;
- redaction negative tests;
- Stable/MSRV/macOS/Windows results;
- dependency/footprint deltas;
- unresolved findings by severity.

## 16. Handoff notes

Treat the curl file-handle issue as a filesystem authority problem, not merely a temp-name problem. The fix should make the external adapter's staging guarantee as close as practical to the native Eggfetch path: Eggup retains the file it authorized and writes only through that handle.
