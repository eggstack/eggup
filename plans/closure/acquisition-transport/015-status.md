# Closure record — acquisition-transport M015

## Status

**Conditionally closed.**

The defect the milestone targets is fixed and proven on the native macOS lane,
and the new diagnostic assertion is proven to **fail** against the pre-M015
behavior. The condition is the same one the sibling acquisition M012 and M013
closures carry: the **Linux** and **Windows** lanes and the hosted CI matrix
were **not run** at this stage, and the parent session owns the whole-tree
gate. Everything not run is recorded as **not run**, never inferred from the
macOS lane.

Three deviations are recorded below with explicit severity:

- **D-1 (low)** — the crate changelog entry was placed at the **top** of the
  `Unreleased` section rather than appended after M012's entry, because every
  changelog in this workspace orders `Unreleased` **newest-first** (M013 sits
  above M012 here; M015 above M014 above M012 in `eggup-core`). M012's and
  M013's entries were **not** touched. The parent session may move the block if
  it prefers a literal append.
- **D-2 (low)** — this milestone's own insertions shifted
  `crates/eggup-curl/src/lib.rs` by +30 lines before the test module and +119
  after it, so `architecture/curl-adapter.md`'s `file:line` references were
  **mechanically re-aligned** to keep the deep dive truthful. Seven
  **pre-existing** stale references found during that pass were corrected at the
  same time (documentation only, no behavior).
- **D-3 (low)** — this milestone **narrows** registry finding 1 rather than
  closing it; see *Registry updates* and UF-1.

No publication was performed, no tag was moved, and no published version was
bumped. `eggup-curl` remains published at **`0.1.2`**, which still carries the
cause-free spawn detail.

## Source plan and roadmap

- Plan: `plans/implementation/acquisition-transport/015-spawn-failure-diagnosability-corrective.md`
- Roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Finding of record: `plans/registry.md` § "Post-0.1.3 open findings" — item
  **1** (the `eggup-curl` sub-second-deadline tests are load-sensitive on a
  busy host). M015 **narrows** item 1; it does not close it.

The plan file's `Status:` header was updated from `active` to
`conditionally closed` and now points at this record.

## Reviewed repository baseline

- The plan's reviewed baseline is `c4805411f74b85158e0ae5d0a219a32c4209d98e`
  on branch `plans/corrective-batch-2026-10-23`.
- The working tree at the time of implementation was at `87cab50` — **two
  commits ahead** of the plan's baseline (`ab90688` "fix(corrective): close nine
  audit milestones across four subsystems", `87cab50` "fix(archive): make the
  M005 member-write guard line-ending portable"). Neither touches the spawn
  site.
- The defect is present at **both** revisions. At the plan's baseline the site
  read `AcquisitionError::Unavailable(bound(format!("curl spawn failed for
  {redacted}")))` (`crates/eggup-curl/src/lib.rs:604`); at `87cab50` it reads
  `AcquisitionError::__adapter_unavailable(format!("curl spawn failed for
  {redacted}"))` (`crates/eggup-curl/src/lib.rs:602`). M013 retyped the
  constructor; the **cause-discarding `map_err(|_| …)` is identical** in both,
  so the defect survives M013 untouched.
- The pre-M015 baseline was materialised with `git worktree add --detach` at a
  **unique** path (`/tmp/eggup-acq-m015-base-34704`), never by mutating the main
  working tree's git state. The worktree was removed with
  `git worktree remove --force` afterwards.

## Implementation commits / PRs

**None.** No commit was created, no branch was created or switched, and no tag
was moved. The work exists as uncommitted changes in the working tree, in the
files listed under *Files touched* below, on top of `87cab50`.

The concurrently modified files `crates/eggup-service/tests/**` visible in
`git status` are **another agent's** work and were neither touched nor
reverted by this milestone.

## Executive finding

The spawn-failure diagnostic in `eggup-curl` now carries the OS cause while
keeping every other contract exactly where it was.

- **The defect.** `cmd.spawn()`'s `io::Error` was discarded by a
  `map_err(|_| …)`, so a missing executable (`ENOENT`), a non-executable file
  (`EACCES`), a path-component error (`ENOTDIR`), and a transient resource
  failure (`EAGAIN`/`ENOMEM`) all rendered the identical, cause-free string
  `curl spawn failed for {redacted}`.
- **The hosted evidence.** Run `40004604594` passed the `Stable`
  (ubuntu-latest) lane; run `40005168760` **failed** it on identical code,
  with the exact panic text
  `Unavailable("curl spawn failed for https://example.com/x")`. The failure was
  intermittent and environment-level, and **the diagnostic gave no way to
  attribute it**. That is the defect this milestone fixes.
- **The fix.** The mapping now preserves the cause inside the `Unavailable`
  detail, in the exact documented format
  `curl spawn failed for {redacted}: {kind}[ (os error {code})]`.
- **What did not change.** The variant is still `AcquisitionError::Unavailable`,
  so the `ComposedTransport` fallback contract is untouched —
  `Unavailable` still falls back under `CompositionPolicy::UnavailableOnly`,
  and `Io` (terminal under every policy) is **not** used for a spawn failure.
  The URL redaction is unchanged, the detail stays bounded by the seam's
  shared 512-byte UTF-8-safe rule, no dependency changed, no public API
  changed, and nothing was published.
- **The cause segment carries no free-form OS text.** Only the `ErrorKind`
  discriminant and the numeric raw OS code are rendered; the OS-provided
  message is never rendered. An OS message is free-form, platform-specific,
  and can echo a path, so rendering it would risk carrying caller-supplied
  material into a diagnostic this crate's redaction rule governs.

## Requirement → evidence matrix

| Req | Requirement | Evidence | Command / result |
|---|---|---|---|
| §6.1 | The `io::Error` is no longer discarded; the detail names the `ErrorKind` | `crates/eggup-curl/src/lib.rs:631-633` (mapping) and `:439-445` (`spawn_failure_detail`) | `cargo test -p eggup-curl --locked --offline` → `spawn_failure_detail_names_the_os_cause_and_stays_unavailable ... ok` |
| §6.1 | The detail names the raw OS code when the error carries one | `crates/eggup-curl/src/lib.rs:441-444` | same test asserts `os error {code}` with the code observed independently on this platform (macOS: `NotFound (os error 2)`) |
| §6.1 | The exact documented format is pinned and cannot drift | the test's exact-equality assertion against `curl spawn failed for {redacted}: {kind}[ (os error {code})]` | same test → `ok` |
| §6.1 | The established `curl spawn failed for ` prefix is preserved | the test's `starts_with` assertion | same test → `ok` |
| §4 / §6 | The variant is still `Unavailable` and `is_unavailable()` is true | the test's first two assertions | same test → `ok` |
| §4 / §5 | The existing contract test is **unchanged** and still passes | `crates/eggup-curl/src/lib.rs:1083-1106` (`missing_executable_and_spawn_failure_are_unavailable`), byte-identical to its pre-M015 form | `cargo test -p eggup-curl --locked --offline` → `missing_executable_and_spawn_failure_are_unavailable ... ok` |
| §4 | The fallback contract is unchanged: `Unavailable` still falls back | `composition_with_fixture_falls_back_from_unavailable_curl` (pre-existing, untouched) | same run → `... ok` |
| §4 | `Io` is **not** used for a spawn failure | the variant asserted is `Unavailable`; no `__adapter_io` call was added to the spawn path | `grep -n "spawn_failure_detail\|__adapter_io" crates/eggup-curl/src/lib.rs` → the spawn path uses only `__adapter_unavailable` |
| §4 | The URL redaction is preserved and the cause adds no URL material | the test asserts the userinfo / query / fragment sentinels are absent and that `<redacted>` is present | same test → `ok` |
| §4 | The detail stays bounded | the test asserts `detail.len() <= 512`; the seam's `bound_detail` applies the cap | same test → `ok` |
| §6.2 | The cause segment carries no free-form OS text | `crates/eggup-curl/src/lib.rs:439-445` renders only `{:?}` of the `ErrorKind` and the numeric code | read-back of the helper; no `to_string()` of the `io::Error` anywhere in the helper |
| §11 | The scoped gate is green | fmt, clippy, test, doc | see *Exact commands run and results* |
| §11 | The new assertion **fails** against the pre-M015 behavior | see *Baseline failure proof* | verbatim failure below |
| §12 | `architecture/curl-adapter.md` §7.2 states the new format and the reason | `architecture/curl-adapter.md:364-407` | read-back |
| §12 | §7.2 records the "no free-form OS text" rule | `architecture/curl-adapter.md:410-417` | read-back |
| §12 | The §10 message table row shows the new format | `architecture/curl-adapter.md:625` | read-back |
| §12 | The §11 failure-mode row no longer implies a cause-free detail | `architecture/curl-adapter.md:653` | read-back |
| §12 | The crate changelog has a new `Unreleased` entry and M012/M013's entries are intact | `crates/eggup-curl/CHANGELOG.md:8-40` (M015), `:42` (M013), `:83` (M012) | read-back |
| §12 | The deep dive's `file:line` references still resolve | every `lib.rs:N` reference and every §12 test-table line re-resolved against the source | a script re-checked all 28 test-table rows: **0 mismatches**; all `lib.rs:` references land on the construct they name |
| §13 | The plan's `Status:` header is set to the closed state and points at this record | `plans/implementation/acquisition-transport/015-spawn-failure-diagnosability-corrective.md:3` | read-back |
| §13 | No republication, no version bump, no tag movement, no dependency change | no `cargo publish`, no `git tag`, no manifest edited | `git status --short` shows no manifest change; `git tag` list unchanged |
| §16 | Registry finding 1 is recorded as **narrowed**, with the hosted evidence | see *Registry updates* | prose for the parent to apply |
| §15 | Per-platform results reported separately | see *Per-platform results* | macOS run; Linux and Windows **not run** |

## Production implementation evidence

### The mapping

`crates/eggup-curl/src/lib.rs:631-633`:

```rust
let mut child = cmd.spawn().map_err(|e| {
    AcquisitionError::__adapter_unavailable(spawn_failure_detail(&redacted, &e))
})?;
```

Before (identical at `c480541` and `87cab50`, modulo the constructor M013
retyped):

```rust
let mut child = cmd.spawn().map_err(|_| {
    AcquisitionError::__adapter_unavailable(format!("curl spawn failed for {redacted}"))
})?;
```

### The helper

`crates/eggup-curl/src/lib.rs:439-445`:

```rust
fn spawn_failure_detail(redacted: &str, error: &std::io::Error) -> String {
    let kind = format!("{:?}", error.kind());
    match error.raw_os_error() {
        Some(code) => format!("curl spawn failed for {redacted}: {kind} (os error {code})"),
        None => format!("curl spawn failed for {redacted}: {kind}"),
    }
}
```

It is **private** (no `pub`), takes the **already redacted** string, and never
receives the raw URL. It renders only the `ErrorKind` discriminant and the
numeric raw OS code.

### The rendered detail, observed

On macOS (darwin/arm64), a fetch through a transport whose executable does not
exist renders exactly:

```text
curl spawn failed for https://<redacted>@example.com/x?<redacted>: NotFound (os error 2)
```

`NotFound` and the raw code `2` are the platform's own values for a missing
executable; they were observed **independently** of the adapter by the test's
own `Command::new(<missing path>).spawn()` call, so the assertion is not
circular with the adapter's rendering and does not hard-code an errno.

### The new test

`crates/eggup-curl/src/lib.rs:1126-1195`
(`spawn_failure_detail_names_the_os_cause_and_stays_unavailable`) proves, in
order:

1. the failure still classifies as `Unavailable` and `is_unavailable()` is true;
2. the detail names the `ErrorKind` observed for that spawn;
3. the detail carries the raw OS code when the platform produced one, and no
   `os error` segment when it did not;
4. the detail keeps the established `curl spawn failed for ` prefix;
5. the detail **equals** the documented format exactly;
6. the userinfo, query, and fragment sentinels are absent and `<redacted>` is
   present;
7. the detail is ≤ 512 bytes.

It is deterministic by construction: no network, no wall-clock budget, no
`PATH` dependency, and no real `curl` required.

## Exact commands run and results

All cargo invocations used `--offline`; no network was used.

```text
cargo fmt --all -- --check
# → exit 0 (no diff)

cargo clippy -p eggup-curl --all-targets --all-features --locked --offline -- -D warnings
# → Finished `dev` profile ... (no warnings, exit 0)

cargo test -p eggup-curl --locked --offline
# → lib target:  27 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
# → tests/assertion_guard.rs: 3 passed; 0 failed
# → doc-tests:    0 passed
#   incl. tests::spawn_failure_detail_names_the_os_cause_and_stays_unavailable ... ok
#        tests::missing_executable_and_spawn_failure_are_unavailable ... ok
#        tests::composition_with_fixture_falls_back_from_unavailable_curl ... ok

cargo doc -p eggup-curl --no-deps --locked --offline
# → Finished `dev` profile ... Generated /Users/davidbowman/projects/eggup/target/doc/eggup_curl/index.html (exit 0)
```

The one **ignored** test is the pre-existing
`connect_phase_timeout_is_typed_end_to_end`, which carries a recorded reason
and is reported as *ignored*, never as *passed*. It is unrelated to this
milestone.

**Not run, by design:** `cargo tree` (a review step, not a gate) and the
whole-tree commands (the parent session owns them — see the addendum). No
manifest was edited, so the dependency graph is unchanged by construction:
`eggup-curl` still depends on `eggup-acquisition` alone.

## Baseline failure proof

**Mechanism.** A detached worktree at a unique path, with the M015 source
copied in and **only the diagnostic change** reverted inside the copy. The
main working tree's git state was never mutated.

```text
$ git worktree add --detach /tmp/eggup-acq-m015-base-34704 HEAD
Preparing worktree (detached HEAD 87cab50)
HEAD is now at 87cab50 fix(archive): make the M005 member-write guard line-ending portable

$ cp crates/eggup-curl/src/lib.rs \
      /tmp/eggup-acq-m015-base-34704/crates/eggup-curl/src/lib.rs
```

Inside the **copy only**, the mapping was restored to the cause-discarding form
and the `spawn_failure_detail` helper was removed:

```text
$ cd /tmp/eggup-acq-m015-base-34704
$ grep -n "curl spawn failed" crates/eggup-curl/src/lib.rs
602:        AcquisitionError::__adapter_unavailable(format!("curl spawn failed for {redacted}"))
```

The new test was then run against that reverted copy:

```text
$ cargo test -p eggup-curl --locked --offline \
      spawn_failure_detail_names_the_os_cause_and_stays_unavailable

running 1 test
test tests::spawn_failure_detail_names_the_os_cause_and_stays_unavailable ... FAILED

failures:

---- tests::spawn_failure_detail_names_the_os_cause_and_stays_unavailable stdout ----

thread 'tests::spawn_failure_detail_names_the_os_cause_and_stays_unavailable' panicked at crates/eggup-curl/src/lib.rs:1126:9:
detail does not name the ErrorKind NotFound: curl spawn failed for https://<redacted>@example.com/x?<redacted>
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    tests::spawn_failure_detail_names_the_os_cause_and_stays_unavailable

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p eggup-curl --lib`
```

**Observed failure:** the assertion *"detail does not name the ErrorKind
NotFound"* fires against the cause-free detail
`curl spawn failed for https://<redacted>@example.com/x?<redacted>` — exactly
the pre-M015 rendering. The test therefore **discriminates**: it passes on the
M015 tree and fails on the pre-M015 behavior.

The worktree was removed afterwards:

```text
$ git worktree remove --force /tmp/eggup-acq-m015-base-34704
$ git worktree list
/Users/davidbowman/projects/eggup 87cab50 [plans/corrective-batch-2026-10-23]
```

No `stash`, `checkout <path>`, `restore`, `commit`, `push`, `tag`, `clean`, or
`reset` was run at any point, and no branch was created or switched.

## Invariant review

| §4 invariant | Result | Evidence |
|---|---|---|
| The variant does not change — a spawn failure is `Unavailable` | **held** | the mapping constructs `__adapter_unavailable`; the test asserts `Unavailable` + `is_unavailable()`; the pre-existing contract test is byte-identical and green |
| The fallback contract does not change — `Unavailable` still falls back under `UnavailableOnly`; `Io` is terminal and unused here | **held** | `composition_with_fixture_falls_back_from_unavailable_curl` green; the seam's `should_fallback` and `CompositionPolicy` are untouched (`crates/eggup-acquisition/**` not modified) |
| URL redaction is preserved | **held** | `request.redacted()` is still the only URL source; the helper receives the redacted string |
| Credential material stays out | **held** | the test asserts the userinfo / query / fragment sentinels are absent from the detail |
| The detail stays bounded (512 bytes, UTF-8 boundary) | **held** | the seam's `bound_detail` applies the cap; the test asserts `len() <= 512`; the longest realistic detail is ≈ 308 bytes |
| No dependency change | **held** | no manifest edited; `std::io::ErrorKind` / `raw_os_error()` are in `std` |
| No public API change | **held** | the helper is private; no variant, signature, field, or feature changed |
| No authenticity / signature / publisher-trust claim | **held** | the detail carries only a kernel error classification and a numeric code; no doc, string, or changelog line claims authenticity |
| Process lifecycle unchanged | **held** | the spawn site still precedes any child; every kill/reap/join/teardown path is untouched; no child is created on a spawn failure |
| Rust 1.89 MSRV | **held** | `ErrorKind`'s `Debug` and `raw_os_error()` are stable well below 1.89; no new syntax or API |
| `unsafe_code` denied / `missing_docs` denied | **held** | the crate keeps `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`; `cargo doc` is clean and clippy runs with `-D warnings` |

## Failure / rollback / recovery review

- **A spawn failure still yields `Unavailable`** — the classification is
  unchanged, so `ComposedTransport` still falls back to the secondary transport
  under the default policy. The added evidence changes **what is reported**,
  never **what is decided**.
- **No residue.** The failure happens before any child process exists; the
  owned temp is removed by the existing `Guard` and the artifact destination is
  never created. The pre-existing contract test asserts `!dest.exists()` and is
  unchanged.
- **No new control flow.** Nothing branches on the cause: the adapter does not
  retry, does not re-select an executable, and does not escalate to another
  transport on its own.
- **Cancellation and timeouts are untouched.** A cancellation observed before
  the spawn still returns `Cancelled` and never reaches the spawn site; a
  spawn failure still returns `Unavailable`, not `Timeout`.
- **Rollback.** There is nothing to roll back: the change is one mapping
  expression and one private helper. Reverting it restores the exact pre-M015
  rendering, which the baseline proof demonstrates.
- **Concurrency.** The adapter holds no shared mutable state on this path, and
  `AcquisitionError` remains `Send + Sync` (the detail is a `String`).

## Compatibility and migration review

- **Published version in scope:** `eggup-curl 0.1.2` (crates.io, 2026-10-04,
  `plans/closure/acquisition-transport/009-status.md`). The working tree
  carries the workspace version `0.1.3`, which is **not published** for this
  crate.
- **The public API is unchanged.** No signature, field, variant, constructor,
  or feature was added, removed, or retyped. `AcquisitionError::Unavailable(String)`
  is unchanged.
- **The classification is unchanged.** Every consumer that matches on the
  variant keeps working, and the fallback policy keeps working.
- **Only the detail string changes.** A consumer matching the established
  `curl spawn failed for ` prefix still matches. A consumer asserting the
  *entire* detail equals `curl spawn failed for {redacted}` would now see a
  longer string. The crate's published contract does not promise a byte-exact
  detail, and this is a diagnostic-text change, not a behavior change.
- **No consumer migration is required.**
- **The published artifact is unchanged.** `eggup-curl 0.1.2` on crates.io
  **still carries the cause-free detail**. This milestone publishes nothing,
  bumps no published version, and moves no tag. A republication of
  `eggup-curl` — and only then — would carry the fix to consumers; that is a
  **separate milestone** and is **not** authorized here.
- **No pin cascade.** `crates/eggup-eggpack/Cargo.toml` depends on
  `eggup-core = "=0.1.3"`, `eggup-archive = "=0.1.3"`, and
  `eggup-acquisition = "=0.1.3"` — **not** on `eggup-curl`. This change
  therefore has no consequence for the eggpack pin chain.

## Security review

- **The cause segment cannot carry caller material.** It is derived only from
  `io::Error::kind()` (a closed enumeration) and the numeric raw OS code. The
  OS-provided message text is **never** rendered, so no path, URL, or
  credential can enter the diagnostic through the error message. This was a
  deliberate design choice, not an omission.
- **The URL redaction is unchanged** and the cause segment adds no URL
  material: the userinfo / query / fragment sentinels are asserted absent, and
  the redacted form is asserted present.
- **Bounded.** The detail is truncated by the seam's shared 512-byte UTF-8-safe
  rule. Truncation would drop the **tail** (the cause segment) rather than the
  redaction — the correct order of loss.
- **No authenticity, signature, or publisher-trust claim** appears in the
  detail, the architecture record, the changelog, or this closure record.
- **No new failure mode.** The helper allocates one `String` and performs no
  I/O; it cannot panic.

## Documentation / operations evidence

| Document | Change | Where |
|---|---|---|
| `architecture/curl-adapter.md` §7.2 | New detail format; the "why the cause is now preserved" paragraph; the "no free-form OS text" rule; the `Unavailable`-classification paragraph now names the helper | `:364-417` |
| `architecture/curl-adapter.md` §10 | The message table row for the spawn failure now shows the new format and names the `ErrorKind` + raw OS code | `:624` |
| `architecture/curl-adapter.md` §11 | The failure-mode row for "Executable missing / not executable / spawn failure" now states the outcome is still `Unavailable` (never `Transport`, never `Io`) **and** that the detail names the OS cause | `:653` |
| `architecture/curl-adapter.md` §11 | The review checklist's diagnostics bullet no longer says "numeric category only" | `:678` |
| `architecture/curl-adapter.md` §12 | Test count 26 → 28; a row for the new test; the whole document's `file:line` references re-aligned | `:683`, `:728`, and throughout |
| `crates/eggup-curl/CHANGELOG.md` | New `Unreleased` entry for M015, with the no-publication / no-migration disclaimer; M012's and M013's entries untouched | `:8-45` (M015), `:47` (M013), `:88` (M012) |
| `plans/implementation/acquisition-transport/015-…md` | `Status:` header set to the closed state | `:3` |
| `plans/closure/acquisition-transport/015-status.md` | This record | — |

**Pre-existing drift corrected in passing (D-2, documentation only).** The
re-alignment pass surfaced seven references that were already wrong at `87cab50`
and pointed at unrelated code: `credential_material_is_redacted`
(`lib.rs:1746` → `:1974`), `artifact_temp_is_owner_private_and_no_clobber`
(`lib.rs:1698` → `:1926`), and five §11 "test `lib.rs:…`" cells that pointed
at the wrong test (`:1569`→`:1797`, `:1121`→`:1285`, `:1182`→`:1395`,
`:1531`→`:1740`, `:1591`→`:1819`). Two further references were corrected to
the construct they name: the second `child.wait()` (`:690`→`:716`) and the
post-join byte-count `TooLarge` site (`:725-728`→`:731-733`). No behavior
changed; `scripts/check-docs-truth.sh` does not validate `file:line` offsets,
so this was a truthfulness pass, not a gate fix.

## Unresolved findings with severity

- **UF-1 (medium, carried forward) — registry finding 1 is narrowed, not
  closed.** M015 makes the **spawn-failure** mode of the finding *diagnosable*.
  It does **not** make the adapter's behavior deterministic under host resource
  pressure: an intermittent spawn failure can still occur on a loaded host, and
  the fix makes it **attributable**, not **prevented**. The finding's
  *timeout-coupling* mode was closed separately by acquisition M012
  (`plans/closure/acquisition-transport/012-status.md`). The finding must
  therefore be recorded as **partially addressed**, not closed.
- **UF-2 (low) — platform qualification outstanding.** Linux, Windows, and
  every hosted CI lane (`stable`, `msrv`, `service-systemd`, `macos`,
  `windows-check`) were **not run** from this implementation machine, and the
  whole-tree gate was not run by this milestone. The parent session owns both.
- **UF-3 (low) — the fix is unpublished.** The published `eggup-curl 0.1.2`
  artifact keeps the cause-free detail. Consumers do not receive the
  improvement until a republication milestone ships it. M015 authorizes no
  publication.
- **UF-4 (low) — residual reference precision.** After the re-alignment, a few
  `architecture/curl-adapter.md` references remain approximate rather than
  exact (for example the `:679-685` cell in the `TooLarge` row points at the
  overflow teardown rather than at the flag). They were already approximate at
  `87cab50`; the document's own "Known doc/code drift" section records the
  general condition. No behavior is affected.
- **D-1 (low, deviation)** — the changelog entry sits at the **top** of
  `Unreleased` (workspace newest-first convention) rather than appended after
  M012's entry. M012's and M013's entries were not rewritten. The parent may
  move the block if it prefers a literal append.

## Roadmap disposition

**`plans/subsystems/acquisition-transport-roadmap.md`** — the parent session
owns this file. It must record M015 as implemented and conditionally closed on
the native macOS lane, with the Linux / Windows / hosted qualification
outstanding, alongside M012 (conditionally closed) and M013.

## Registry updates

**Not applied by this milestone.** `plans/registry.md`, the root
`CHANGELOG.md`, and the subsystem roadmap are the parent session's to edit.
The exact text the parent should apply:

1. **`plans/registry.md` § "Post-0.1.3 open findings", item 1** — append to the
   existing entry (do not renumber it, and do **not** mark it closed):

   > **Re-observed as hosted evidence 2026-10-23.** Hosted run `40004604594`
   > passed the `Stable` (ubuntu-latest) lane; run `40005168760` **failed** it
   > on identical code, with the exact panic text
   > `Unavailable("curl spawn failed for https://example.com/x")`. The failure
   > was intermittent and environment-level, and the adapter's diagnostic
   > **discarded the `io::Error`**, so a missing executable, a non-executable
   > file, and a transient resource failure were indistinguishable and the
   > failure could not be attributed. **Acquisition M015 narrows this finding
   > rather than closing it** — the spawn-failure mode is now diagnosable (the
   > `Unavailable` detail names the `ErrorKind` and the raw OS code), and the
   > timeout-coupling mode was closed by M012. The residual exposure is that an
   > intermittent spawn failure can still occur on a loaded host; it is now
   > attributable, not prevented.

2. **`plans/registry.md` § "Active subsystem roadmaps"**, Acquisition transport
   row — replace the status/next-milestone cells with:

   > M001-M011 closed; acquisition/eggfetch 0.1.3 published, curl 0.1.2
   > published; M012 and M013 implemented; M015 implemented and conditionally
   > closed 2026-10-23 | M015 spawn-failure diagnosability corrective
   > (narrows open finding 1) — conditionally closed on the native macOS lane;
   > Linux, Windows, and the hosted CI lanes were not run. No republication:
   > `eggup-curl 0.1.2` still carries the cause-free detail.

3. **`plans/registry.md` § "Dependency-ready implementation work"** — add a row:

   > | Acquisition transport | M015 spawn-failure diagnosability corrective |
   > conditionally closed 2026-10-23 (macOS only) |
   > `plans/implementation/acquisition-transport/015-spawn-failure-diagnosability-corrective.md`;
   > `plans/closure/acquisition-transport/015-status.md` | Narrows open finding
   > 1 (does not close it). The `Unavailable` detail now names the
   > `io::ErrorKind` and the raw OS code; the variant, the fallback contract,
   > and the redaction are unchanged. `eggup-curl 0.1.2` keeps the cause-free
   > detail until a republication milestone; no pin cascade, because
   > `eggup-eggpack` does not depend on `eggup-curl`. Linux/Windows/hosted CI
   > **not run** |

4. **Root `CHANGELOG.md` `Unreleased`** — add the aggregate entry for M015,
   matching `crates/eggup-curl/CHANGELOG.md`: the spawn-failure diagnostic
   now carries the OS cause; the variant and the fallback contract are
   unchanged; the fix is unpublished, the published `eggup-curl 0.1.2` still
   carries the cause-free detail, and no publication milestone authorizes it.

## Per-platform results

Reported separately. A lane not run is recorded as **not run** and is never
inferred from another lane.

| Platform | Result | Detail |
|---|---|---|
| **macOS (darwin/arm64)** | **run** | `cargo fmt --all -- --check` → exit 0; `cargo clippy -p eggup-curl --all-targets --all-features --locked --offline -- -D warnings` → exit 0; `cargo test -p eggup-curl --locked --offline` → 27 passed, 1 ignored, 0 failed (+ 3 guard tests passed); `cargo doc -p eggup-curl --no-deps --locked --offline` → exit 0; baseline-failure proof → the new test FAILS against the pre-M015 behavior with the verbatim panic recorded above |
| **Linux** | **not run** | no Linux host was reachable from this implementation machine; no Linux result is claimed or inferred from the macOS lane |
| **Windows** | **not run** | no Windows host was reachable from this implementation machine; no Windows result is claimed or inferred. The new test is **not** `#[cfg]`-gated, so it is expected to run on the Windows lane, but that expectation is **not** evidence |
| **Hosted CI lanes** (`stable`, `msrv`, `service-systemd`, `macos`, `windows-check`) | **not run** | the parent session owns the hosted lanes; the hosted evidence cited in this record (`40004604594`, `40005168760`) was supplied by the parent session and was not reproduced from this machine |

## Parent-session verification addendum (2026-10-23)

This record lists the whole-tree commands and the hosted CI lanes as
deliberately **not run** at this stage, because the parent session owns them.
Nothing in this milestone should be read as a claim about them.

- `./scripts/check-local.sh` — **not run** by this milestone.
- `cargo test --workspace --all-targets --all-features --locked --offline` —
  **not run** by this milestone.
- `cargo clippy --workspace --all-targets --all-features --locked --offline --
  -D warnings` — **not run** by this milestone.
- `cargo doc --workspace --no-deps --locked --offline` — **not run** by this
  milestone.
- `cargo tree --workspace --locked` — **not run** by this milestone. No
  manifest was edited, so the dependency graph is unchanged by construction:
  `eggup-curl` still depends on `eggup-acquisition` alone.
- Hosted lanes `stable`, `msrv`, `service-systemd`, `macos`, `windows-check` —
  **not run** by this milestone.

What **was** run by this milestone, on the native macOS lane, is listed in
*Exact commands run and results* and is scoped to `eggup-curl`: fmt, the
scoped clippy, the crate's own test suite (unit + integration guard), the
crate's own rustdoc, and the baseline-failure proof in a detached worktree.

The parent session should append its own results here when it runs the
whole-tree gate and the hosted lanes, per the convention the sibling closure
records already follow.

---

## Parent-session verification addendum (2026-10-23, hosted qualification)

**What this addendum supersedes.** Seven statements earlier in this record are
overtaken by the evidence below. They are left in place only as the record of the
state at the time of writing; the section each one lives in is named so that no
two contradictory statements remain in this file.

1. § "Status" — "the **Linux** and **Windows** lanes and the hosted CI matrix
   were **not run** at this stage, and the parent session owns the whole-tree
   gate. Everything not run is recorded as **not run**, never inferred from the
   macOS lane." The hosted lanes **have now run** and the whole-tree gate **has
   now run**. That condition is **discharged**. The macOS lane's result is still
   never inferred from another lane.
2. § "Requirement → evidence matrix", row **§15** — "Per-platform results
   reported separately … macOS run; Linux and Windows **not run**". Replaced by
   the per-platform results below.
3. § "Per-platform results" — the **Linux** row (**not run**), the **Windows**
   row (**not run**, "the new test is **not** `#[cfg]`-gated, so it is expected
   to run on the Windows lane, but that expectation is **not** evidence"), and
   the **Hosted CI lanes** row (**not run**). All three are replaced by the
   per-platform results below; the Windows expectation is now **evidence**.
4. § "Unresolved findings with severity" → **UF-2 (low)** — "platform
   qualification outstanding. Linux, Windows, and every hosted CI lane … were
   **not run** from this implementation machine, and the whole-tree gate was not
   run by this milestone. The parent session owns both." That condition is
   **now discharged**. UF-2's text and its **low** severity are retained above
   unchanged and are **not** rewritten — only the satisfaction of its condition
   is recorded here.
5. § "Roadmap disposition" — "It must record M015 as implemented and
   **conditionally closed** on the native macOS lane, with the Linux / Windows /
   hosted qualification **outstanding**". The milestone is now **closed** and
   hosted-qualified; the parent must record it as closed, not conditionally
   closed with qualification outstanding.
6. § "Registry updates" items **2** and **3** — the proposed registry text that
   says "Linux, Windows, and the hosted CI lanes were not run" and "Linux/
   Windows/hosted CI **not run**". That wording is **superseded** and must not
   be applied: the hosted lanes ran green. Item **1** (finding 1 re-observed as
   hosted evidence, **narrowed** not closed) remains correct and is extended by
   this addendum.
7. § "Source plan and roadmap" — "The plan file's `Status:` header was updated
   from `active` to `conditionally closed` and now points at this record." The
   plan header now reads **closed 2026-10-23** and still points at this record;
   that header change is what discharged the condition above.

Also superseded: the earlier § "Parent-session verification addendum
(2026-10-23)" section above, whose bullet list records the whole-tree commands
and the hosted lanes as **not run** by this milestone. Those bullets were true
about *this milestone's own* scope; as a statement about the qualification
status they are overtaken by the results below.

### Hosted qualification — CI run `40008039843`

Branch `plans/corrective-batch-2026-10-23`, commit `0cb2c53`, push-triggered,
workflow `CI` (`.github/workflows/ci.yml`). **All five hosted lanes are green.**

| Lane | Conclusion | Evidence |
|---|---|---|
| `Stable checks` (ubuntu-latest) | **success** | fmt + clippy `-D warnings` + `cargo test --workspace --all-targets --all-features --locked` → **483 passed, 0 failed, 1 ignored, 44 test binaries** + `cargo doc --workspace --no-deps --locked` + `cargo package -p eggup-service --locked` |
| `MSRV check` (ubuntu-latest, Rust 1.89.0) | **success** | `cargo check --workspace --all-targets --locked` on 1.89.0 |
| `macOS tests` | **success** | **483 passed, 0 failed, 1 ignored, 44 test binaries** |
| `Windows archive, acquisition, and service tests and check` (windows-latest) | **success** | **598 passed, 0 failed, 0 ignored, 63 test binaries** |
| `Linux systemd service qualification` (ubuntu-latest) | **success** | **9 passed, 0 failed, 0 ignored, 7 test binaries** |

The one ignored test in the `Stable checks` and `macOS tests` lanes is the
pre-existing, environment-conditional curl connect-timeout test, with its skip
reason recorded in the test source. It is **not** a new ignore introduced by this
batch.

The Windows lane's larger totals (**598 passed / 63 test binaries**, against
**483 / 44** on Linux and macOS) are the documented **platform-set difference**,
not a coverage gap: the `#[cfg(not(windows))]` curl tests are excluded on the
Windows lane, while the Windows-only SCM tests do not exist on the Unix lanes.

### How the hosted lanes reached green

The acquisition lanes were **not** straightforwardly green, and the intermediate
evidence is the motivation for this milestone. It is recorded here so the green
result below is not read as a first-attempt pass:

- **Run `40004604594`** (first push of this batch): the **Windows** lane failed
  on a CRLF-fragile test anchor in
  `crates/eggup-archive/tests/m005_member_write_parity.rs`. The **Stable** lane
  **passed**.
- **Run `40005168760`** (second push): the **Windows** lane failed on 13
  Unix-shaped fixtures in `crates/eggup-service/tests/` plus one CRLF-fragile
  rustdoc anchor in `crates/eggup-service/tests/errors_doc_sections.rs`. All
  were fixed. The **Stable** lane then failed with exactly one test:

  ```text
  test tests::build_curl_args_passes_sub_second_deadlines_to_fake_curl ... FAILED
  thread '...' panicked at crates/eggup-curl/src/lib.rs:1609:14:
  fake curl completes well inside the 30s budget: Unavailable("curl spawn failed for https://example.com/x")
  ```

  The failure was a **spawn** failure, not a timeout, and it was
  **intermittent** — identical code passed the same lane in run `40004604594`.
  It was not diagnosable because `Command::spawn()`'s `io::Error` was
  discarded, making a missing executable, a non-executable file, and a transient
  resource exhaustion indistinguishable. That is the defect this milestone
  fixes: the `Unavailable` detail now reads
  `curl spawn failed for {redacted}: {kind}[ (os error {code})]`, rendering
  **only** the `ErrorKind` discriminant and the numeric OS code, so no path,
  URL, or credential material can leak. The variant stays `Unavailable`, the
  composition fallback contract is untouched, and the existing contract test
  `missing_executable_and_spawn_failure_are_unavailable` is byte-identical and
  still passes.
- **Run `40008039843`** (third push): **green on every lane**, including the
  `Stable` lane that had just failed.

**M015's objective — make the spawn cause attributable — is achieved and
verified on every platform.** The new diagnostic assertion
(`spawn_failure_detail_names_the_os_cause_and_stays_unavailable`) is **not**
`#[cfg]`-gated, and it now runs and passes on Linux, macOS, and Windows: the
Windows-lane expectation recorded above has become evidence.

### Local whole-tree gate

Run by the parent session on the same tree:

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 483 passed; 0 failed; 44 test binaries
```

### Per-platform results

Reported separately; no platform's result is inferred from another's.

| Platform | Result | Evidence |
|---|---|---|
| **Linux** | **run — green** | hosted `Stable checks` lane (fmt, clippy `-D warnings`, workspace test → **483 passed, 0 failed, 1 ignored, 44 test binaries**, `cargo doc`, `cargo package -p eggup-service --locked`) and hosted `Linux systemd service qualification` lane (→ **9 passed, 0 failed, 0 ignored, 7 test binaries**) of CI run `40008039843` |
| **macOS** | **run — green** | hosted `macOS tests` lane → **483 passed, 0 failed, 1 ignored, 44 test binaries**; plus this record's own native macOS lane and the local whole-tree gate above |
| **Windows** | **run — green** | hosted `Windows archive, acquisition, and service tests and check` lane → **598 passed, 0 failed, 0 ignored, 63 test binaries** |

### Severity reconciliation — UF-1 against registry finding 1

This addendum records a difference in severity assessment **openly**, and does
**not** rewrite the record's own assessment. M015's **UF-1 (medium, carried
forward)** entry above is the implementer's assessment and is left exactly as
written — it is **not** deleted, softened, or silently downgraded here.

The two assessments describe **the same finding**:

- **M015's UF-1** is the residual of **registry finding 1** (§ "Post-0.1.3 open
  findings", item 1). It is an **inherited, already-registered** finding, not a
  defect M015 introduced. M015 inherited it, narrowed it, and carried it
  forward.
- The **registry** assesses finding 1 as **low** — "Findings 1 and 2 are
  **low**-severity and block nothing."

**The reconciliation.** The registry records finding 1 as **open and partially
addressed**: M012 narrowed the timeout-coupling mode, M015 narrowed the
spawn-failure mode by making the cause attributable, and **neither closes it**.
The residual exposure is unchanged in kind — an intermittent spawn failure under
host resource pressure can still occur; it is now **attributable, not
prevented**. M015 therefore **narrows** finding 1 rather than closing it, and
the milestone's own "medium, carried forward" label remains visible above as
written. The two severity labels differ in wording only; the registry's **low**
assessment is the one that governs the finding's standing, and the difference
is recorded here rather than by editing the implementer's entry.

### Compatibility consequence

**Unchanged, and carried forward honestly.** None of these milestones published,
none moved a tag, and the published crates still carry every defect these
milestones fixed. `eggup-curl` remains published at **`0.1.2`** and still
carries the cause-free spawn detail; `eggup-acquisition` remains published at
**`0.1.3`** and still carries the flattened `AcquisitionError::Io(String)` shape
that acquisition M013 fixed. The hosted qualification proves the **source**
line, not the published crates; a republication milestone is the only way these
fixes reach consumers. There is **no pin cascade** from this milestone, because
`eggup-eggpack` does not depend on `eggup-curl`.
