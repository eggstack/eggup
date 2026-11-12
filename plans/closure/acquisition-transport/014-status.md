# Acquisition Transport Milestone 014 — adapter capability parity corrective

Status: closed; hosted-qualified in CI run `40062299630` (commit `b9885fa`), all five lanes green

Source plan:
- plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Reviewed repository baseline: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`
(branch `plans/corrective-batch-2026-10-23`)

Implementation commits / PRs: **none.** The parent session owns all git
mutations; this milestone produced working-tree changes only. No commit, tag,
branch, stash, or publication was made.

Finding of record (closed by this milestone):
- `plans/closure/acquisition-transport/013-status.md` — UF-1 (medium): the
  `eggup-eggfetch` adapter exposed a caller-settable `User-Agent` and the
  `eggup-curl` adapter did not, so the two adapters were **not
  capability-parity**.
- `plans/registry.md` — dependency-ready implementation work, row "Acquisition
  transport | M014 adapter capability parity corrective".

## Executive finding

The `eggup-curl` adapter now exposes a caller-settable outbound `User-Agent`,
so a caller can set that header on **either** acquisition adapter for every
request the adapter makes. The capability gap M013 deferred is closed.

The control is placed on `CurlTransport` (a private field plus a fallible
constructor and a public accessor), **not** on `CurlConfig`: `CurlConfig`'s
fields are public and it is **not** `#[non_exhaustive]`, so adding a field
there would be source-breaking for every consumer that constructs it with a
struct literal, while `CurlTransport`'s fields are private and its only
construction path is a constructor, so a private field is not source-breaking.
The change is therefore **purely additive**: nothing is source-breaking for a
consumer of the published `eggup-curl 0.1.2`.

The value is validated before it can reach the wire and every rejection is
`AcquisitionError::InvalidInput` — never `Unavailable`, the one variant
`ComposedTransport` falls back on — so a malformed caller value can never
silently change which transport serves the request. The validation is not
redundant with curl's own behavior: `curl` does **not** sanitize
`--user-agent`, and a value containing `CR LF` was observed being written into
the request head verbatim, injecting an extra header.

The **default wire behavior of the published adapter is unchanged**: the
default transport emits no `--user-agent` element, so curl applies its own
default and the bytes an existing consumer sends are identical.

The parity achieved is **capability parity**, not placement parity and not
default-value parity. Both remaining differences are deliberate consequences
of the two adapters' separate publication histories, and both are documented.

## Requirement → evidence matrix

Every row names the file:line anchor that carries the requirement and the
exact command whose result proves it. All commands were run on **macOS
14/15 arm64 (darwin 25.6.0), Rust 1.89-compatible toolchain, `curl 8.7.1`
present on `PATH`**; per-platform reporting is in the next section.

| # | Requirement (plan §) | Evidence (file:line) | Command and result |
|---|---|---|---|
| 1 | A caller can set the outbound `User-Agent` for every request (§6.2) | `crates/eggup-curl/src/lib.rs:238-257` (`with_executable_and_user_agent`), `:275-277` (`user_agent()`), `:367` and `:448` (forwarded by both fetch paths) | `cargo test -p eggup-curl --locked --offline` → `user_agent_is_emitted_as_a_literal_argv_element_before_the_url ... ok`, `user_agent_reaches_the_wire_as_the_configured_header ... ok` |
| 2 | The accepted value is observable through a public accessor (§6.2) | `crates/eggup-curl/src/lib.rs:275-277` | `cargo test -p eggup-curl --locked --offline` → `user_agent_reaches_the_wire_as_the_configured_header ... ok` (asserts `user_agent() == Some("eggup-curl-test/1.0")`) |
| 3 | `CR`, `LF`, `NUL`, any other C0 control, `DEL`, and C1 are rejected as `InvalidInput` and **no argv is produced** (§6.3 R3–R5) | `crates/eggup-curl/src/lib.rs:286-327` (`validate_user_agent`), `:677-682` (the argv-build choke point) | `cargo test -p eggup-curl --locked --offline` → `user_agent_rejects_control_characters_without_producing_argv ... ok` |
| 4 | An empty value and an over-bound value are rejected as `InvalidInput`; a value exactly at the bound is accepted (§6.3 R1–R2) | `crates/eggup-curl/src/lib.rs:37` (`MAX_USER_AGENT_BYTES = 512`), `:288-296` | `cargo test -p eggup-curl --locked --offline` → `user_agent_rejects_empty_and_over_bound_values ... ok` |
| 5 | The rejection is terminal, never `Unavailable`, so a composed transport does not fall back (§6.3, §8) | `crates/eggup-curl/src/lib.rs:288-327` (every rejection is `InvalidInput`) | `cargo test -p eggup-curl --locked --offline` → `rejected_user_agent_is_a_terminal_invalid_input_not_unavailable ... ok` (asserts `!err.is_unavailable()` **and** that a composed transport surfaces the terminal failure instead of the fixture's success) |
| 6 | The value reaches curl as `--user-agent <value>`, **before** the `--` separator and the URL, as a literal argv element (§6.4) | `crates/eggup-curl/src/lib.rs:714-720` (the emission), `:725-726` (the `--` separator and the URL) | `cargo test -p eggup-curl --locked --offline` → `user_agent_is_emitted_as_a_literal_argv_element_before_the_url ... ok` (asserts the exact element after the flag, exactly one occurrence, `flag < separator < url`, and `url == args.len() - 1`) |
| 7 | The default transport's argv is unchanged and it sends no `User-Agent` (§6.5) | `crates/eggup-curl/src/lib.rs:191-207` (`with_executable` installs `user_agent: None` at `:205`), `:177` | `cargo test -p eggup-curl --locked --offline` → `default_transport_emits_no_user_agent_and_keeps_the_previous_argv ... ok` (asserts `user_agent() == None`, no `--user-agent` element, and the **element-for-element argv vector**) |
| 8 | The default path's argv is provably identical to the pre-change argv | `git diff` of `build_curl_args` against `20ac770`: the only changes are the added `user_agent: Option<&str>` parameter, the added validation block, and the added `if let Some(agent)` emission block — all of which are no-ops when `user_agent` is `None` | `git diff -U3 -- crates/eggup-curl/src/lib.rs` (read-only) + the element-for-element assertion of requirement 7 |
| 9 | No shell anywhere; the value is a literal argv element (§6.4) | `crates/eggup-curl/src/lib.rs:766-768` (`Command::new(executable).args(&args)` — unchanged) | `cargo clippy -p eggup-curl --all-targets --locked --offline -- -D warnings` → exit 0; and the argv assertions of requirement 6 |
| 10 | The dependency boundary holds: `eggup-curl` depends only on `eggup-acquisition` (§4.1, §11) | `crates/eggup-curl/Cargo.toml:23-24` (unchanged) | `cargo tree -p eggup-curl --locked --offline --depth 1` → `eggup-curl v0.1.3` → `└── eggup-acquisition v0.1.3` (one dependency, no `eggup-eggfetch`) |
| 11 | The seam's API and the eggfetch adapter are unmodified (§5) | `crates/eggup-acquisition/**` and `crates/eggup-eggfetch/**` absent from this milestone's change set | `git status --short` → only `crates/eggup-curl/**` and the three architecture documents are modified by this milestone |
| 12 | The exact grammar and bound are stated in the plan, the rustdoc, and the README (§6.3) | plan §6.3; `crates/eggup-curl/src/lib.rs:217-224` (rustdoc grammar), `:226-241` (the rejection list); `crates/eggup-curl/README.md` "Outbound `User-Agent`" | `cargo doc -p eggup-curl --no-deps --locked --offline` → exit 0, `Generated .../target/doc/eggup_curl/index.html` |
| 13 | The parity claim is defined as capability parity, not placement parity, and is recorded in all three architecture documents in the place the M013 note occupied (§6.6) | `architecture/acquisition.md` §11.1 (heading now "recorded, resolved by M014"); `architecture/curl-adapter.md` §13.1; `architecture/eggfetch-adapter.md` §12.1 (heading now "recorded, resolved by M014") | Document review; the M013 note was updated **in place** in each file, not appended beside it |
| 14 | The boundary consequence is stated: no mechanical cross-adapter parity test exists or is claimed (§6.7) | `architecture/acquisition.md` §11.1 "How parity is enforced"; `architecture/curl-adapter.md` §13.1 "How parity is enforced"; `architecture/eggfetch-adapter.md` §12.1 "How parity is enforced" | Document review |
| 15 | Existing tests are not weakened; the guard suite still passes (§4.8, §10) | `crates/eggup-curl/tests/assertion_guard.rs` (unchanged) | `cargo test -p eggup-curl --locked --offline` → `every_ignored_test_records_a_reason ... ok`, `crate_sources_have_no_vacuous_early_exit ... ok`, `required_behaviors_have_named_asserting_tests ... ok` |

## Production implementation evidence

| Change | Location | Notes |
|---|---|---|
| New public bound | `crates/eggup-curl/src/lib.rs:37` | `pub const MAX_USER_AGENT_BYTES: usize = 512;` |
| New private field | `crates/eggup-curl/src/lib.rs:177` | `user_agent: Option<String>` on `CurlTransport`; private, so not source-breaking |
| New fallible constructor | `crates/eggup-curl/src/lib.rs:238-257` | `with_executable_and_user_agent`; validates config, path, and the agent; `InvalidInput` on rejection; no transport is constructed then |
| New public accessor | `crates/eggup-curl/src/lib.rs:275-277` | `user_agent(&self) -> Option<&str>` |
| New private validator | `crates/eggup-curl/src/lib.rs:286-327` | `validate_user_agent`; implements the §6.3 grammar and bound |
| argv emission | `crates/eggup-curl/src/lib.rs:714-720` | `--user-agent` + the exact value, before `--output -` / `--write-out` / `--` / the URL |
| argv-build choke point | `crates/eggup-curl/src/lib.rs:677-682` | The value is re-validated before a single element is pushed, so a rejected value produces **no** argv |
| Plumbing | `crates/eggup-curl/src/lib.rs:367`, `:448`, `:671`, `:747`, `:758` | `self.user_agent.as_deref()` is forwarded by both fetch methods through `run_curl_to_file` into `build_curl_args` |

**Diff scope.** The production change is confined to
`crates/eggup-curl/src/lib.rs`. No dependency was added, no error variant was
added or retyped, no existing public signature was changed, removed, or
retyped, and no existing assertion was removed or weakened.

**Default-UA decision.** The adapter sends **no** `User-Agent` by default.
`CurlTransport::with_executable` produces a transport whose argv contains no
`--user-agent` element, so `curl` applies its own default (`User-Agent:
curl/<version>`). The compatibility consequence for a consumer of the published
`eggup-curl 0.1.2` is **none** — the default path's argv is byte-for-byte what
it was before this milestone. The alternative (defaulting to `eggup-curl`) was
rejected because it would have changed the observable `User-Agent` header of
every existing consumer of the published crate for no concrete benefit.

**Validation grammar and bound implemented.**

```text
user-agent = 1*( VCHAR / SP / obs-text )
VCHAR      = %x21-7E          ; printable ASCII, excluding SP
SP         = %x20            ; permitted only between two non-SP characters
obs-text   = %xA0-10FFFF     ; any non-ASCII scalar value (C1 excluded)
```

Bound: `MAX_USER_AGENT_BYTES = 512` bytes of the UTF-8 encoding. Rejections
(R1–R6 in the plan): empty; over the bound; any C0 control character (`CR`,
`LF`, `NUL`, `HTAB`, and the rest); `DEL` (`U+007F`); any C1 control character
(`U+0080`–`U+009F`); leading or trailing `SP` (`U+0020`). Every rejection is
`AcquisitionError::InvalidInput`.

**Empirical evidence gathered for the design** (local `curl 8.7.1`, macOS, with
a loopback listener; not a substitute for the crate's own tests):

- `curl --user-agent <value>` consumes the **next** argv element as the value
  unconditionally — a value beginning with `-` is not re-parsed as an option,
  and interior spaces and parentheses are transmitted unchanged.
- `curl` does **not** sanitize the value: a value containing `CR LF` was
  emitted into the request head verbatim, producing an injected
  `X-Injected: 1` header. This is why R3 is a rejection rule and not a
  formality.
- An interior NUL cannot be transmitted at all: `std::process::Command::spawn`
  fails with `io::ErrorKind::InvalidInput` ("nul byte found in provided data"),
  which this adapter maps to `Unavailable` — the one variant that falls back.
  This is why NUL is rejected at validation time with a terminal
  `InvalidInput` instead.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` with
`--offline` (no network). Platform: macOS (darwin 25.6.0, arm64), Rust
toolchain 1.89-compatible, `curl 8.7.1` on `PATH`.

| # | Command | Result |
|---|---|---|
| 1 | `cargo fmt -p eggup-curl -- --check` | **exit 0**, no output |
| 2 | `cargo clippy -p eggup-curl --all-targets --locked --offline -- -D warnings` | **exit 0** — `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.07s`; no warnings |
| 3 | `cargo test -p eggup-curl --locked --offline` | **exit 0** — `unittests src/lib.rs`: `running 34 tests` → `ok. 33 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`; `tests/assertion_guard.rs`: `running 3 tests` → `ok. 3 passed; 0 failed; 0 ignored`; `Doc-tests eggup_curl`: `running 0 tests` → `ok. 0 passed; 0 failed` |
| 4 | `cargo doc -p eggup-curl --no-deps --locked --offline` | **exit 0** — `Finished \`dev\` profile … in 0.25s`; `Generated .../target/doc/eggup_curl/index.html`; no rustdoc warnings |
| 5 | `cargo tree -p eggup-curl --locked --offline --depth 1` | `eggup-curl v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-curl)` / `└── eggup-acquisition v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-acquisition)` — **exactly one dependency**; no `eggup-eggfetch`, no HTTP/TLS stack, no `tokio` |

The workspace-wide gate (`./scripts/check-local.sh`) is the parent session's
responsibility and was **not** run by this milestone. `cargo tree` is a review
step, not a gate: it always exits 0 on a resolvable graph, and its value here
is the dependency-boundary evidence in row 10 of the matrix.

### Test counts

| Target | Tests | Result |
|---|---|---|
| `unittests src/lib.rs` | 34 (28 pre-existing + 6 added by M014) | 33 passed, 1 ignored (pre-existing, environment-conditional, reason recorded), 0 failed |
| `tests/assertion_guard.rs` | 3 | 3 passed, 0 failed |
| `Doc-tests eggup_curl` | 0 | 0 passed, 0 failed |
| **Total** | **37** | **36 passed, 1 ignored, 0 failed** |

### The six added tests

| Test | Line | Gate | Proves |
|---|---|---|---|
| `user_agent_is_emitted_as_a_literal_argv_element_before_the_url` | 2237 | none | the exact element after `--user-agent`, exactly one occurrence, before `--` and the URL; the URL stays last |
| `user_agent_rejects_control_characters_without_producing_argv` | 2283 | none | `CR`, `CR LF`, `LF`, `NUL`, `HTAB`, `DEL`, and two C1 characters are `InvalidInput` at the argv layer **and** at the constructor, and no argv is produced |
| `user_agent_rejects_empty_and_over_bound_values` | 2326 | none | empty rejected; `MAX_USER_AGENT_BYTES + 1` rejected; exactly `MAX_USER_AGENT_BYTES` accepted and transmitted verbatim |
| `default_transport_emits_no_user_agent_and_keeps_the_previous_argv` | 2383 | none | the default transport carries no user agent and its argv is the element-for-element previous vector |
| `rejected_user_agent_is_a_terminal_invalid_input_not_unavailable` | 2447 | none | the rejection is `InvalidInput` and not `Unavailable`; a `ComposedTransport` under `UnavailableOnly` surfaces the terminal failure instead of the fixture's success |
| `user_agent_reaches_the_wire_as_the_configured_header` | 2499 | `#[cfg(not(windows))]` | the peer receives exactly `User-Agent: eggup-curl-test/1.0` — proven on the wire, not only in argv |

## Per-platform results

Reported separately, as required. **No platform is inferred from another.**

| Platform | `cargo fmt --check` | `cargo clippy --all-targets -D warnings` | `cargo test` | `cargo doc` | Status |
|---|---|---|---|---|---|
| **macOS** (darwin 25.6.0, arm64) | pass (exit 0) | pass (exit 0, no warnings) | pass (36 passed, 1 ignored, 0 failed) | pass (exit 0) | **Run — green.** Includes the live-loopback `User-Agent` wire test, which requires `curl` on `PATH` (8.7.1 present). |
| **Linux** | not run | not run | not run | not run | **Not run.** No Linux host was available in this session. Nothing is inferred from the macOS run. |
| **Windows** | not run | not run | not run | not run | **Not run.** No Windows host was available in this session. Nothing is inferred from the macOS run. |

**Known platform difference carried forward, not introduced here.** The
`#[cfg(not(windows))]` gate on `user_agent_reaches_the_wire_as_the_configured_header`
is the same **known, deliberate** platform difference that already applies to
this crate's other live-loopback tests (hosted Windows runners refuse
connections from spawned curl to the suite's local servers, curl exit 7). The
gate is recorded at the point of use in the test module, the Windows lane
therefore runs a **smaller** test set, and **no Windows live-HTTP curl
behavior is claimed** anywhere. The Windows lane still runs the five
platform-neutral M014 tests (argv emission, the three rejection tests, and the
terminality test), so the new control's *policy* is exercised there; only the
on-the-wire assertion is Unix-only. The crate's existing Windows lane
(`cargo test` plus `cargo package -p eggup-service --locked`) is the parent
session's hosted-CI responsibility.

## Invariant review

| Invariant (plan §4) | Result | Evidence |
|---|---|---|
| 1. Transport neutrality — `eggup-curl` must not learn `eggup-eggfetch` exists | **Held** | No `eggup-eggfetch` reference anywhere in the change; `cargo tree` shows one dependency; the parity claim is enforced by documentation, not by a cross-import |
| 2. The seam's API is unchanged | **Held** | `crates/eggup-acquisition/**` not modified |
| 3. No shell | **Held** | `Command::args` unchanged (`lib.rs:764-768`); the value is one argv element |
| 4. URL isolation — `--` separator and URL keep their meaning | **Held** | `--user-agent` is emitted at `:714-720`, before `--output -` / `--write-out` / `--` (`:725`) / the URL (`:726`); asserted by test |
| 5. Fallback semantics unchanged | **Held** | No change to `ComposedTransport`, `CompositionPolicy`, or which failures fall back; the *new* rejection is deliberately terminal |
| 6. No authenticity claim | **Held** | A `User-Agent` is documented as an identity string only; no doc or error string claims publisher trust, signature validity, or provenance |
| 7. No invented timeouts; no new deadline semantics | **Held** | The new option does not touch `min(request, adapter)`; no timeout value was introduced |
| 8. Existing tests not weakened; guards still pass | **Held** | 3/3 guard tests pass; no existing assertion was removed or relaxed |
| 9. Published default wire behavior preserved | **Held** | The default argv is asserted element-for-element and the diff of `build_curl_args` shows the added code is a no-op when `user_agent` is `None` |

## Failure, rollback, and recovery review

- **Rejected value at construction.** `with_executable_and_user_agent` returns
  `Err(AcquisitionError::InvalidInput(...))`; no transport is constructed, no
  process is spawned, no file is created.
- **Rejected value at argv build.** `build_curl_args` returns
  `Err(AcquisitionError::InvalidInput(...))` before any element is pushed, so
  no argv exists and no child can be spawned with an invalid value. This is the
  single choke point every request path goes through, so the property is
  structural rather than per-call-site.
- **No fallback.** The rejection is `InvalidInput`, which the seam documents as
  terminal under every composition policy, so a `ComposedTransport` does not
  retry the request on another adapter. Asserted end-to-end at the composition
  layer in `rejected_user_agent_is_a_terminal_invalid_input_not_unavailable`.
- **Cancellation.** Unchanged. The `CancelFlag` is polled exactly where it was
  polled before; the new option adds no wait and no new blocking point.
- **Restart / retry.** Unchanged. One child process per request, as before; the
  `User-Agent` is a static property of the transport, so a retry sends the same
  value.
- **Contention.** Unchanged. No lock, no shared state, no new file.
  `CurlTransport` remains `Send`/`Sync` by virtue of its field types
  (`PathBuf`, `CurlConfig`, `Option<String>`).
- **Child failure.** Unchanged: exit-code and status classification (§8 of
  `architecture/curl-adapter.md`) is untouched.
- **Rollback.** Reverting the production change is a single-file revert; the
  new public items are additive, so removing them cannot break a consumer that
  never adopted them.

## Compatibility and migration review

### Exactly what is source-breaking for a consumer of the published `eggup-curl 0.1.2`

**Nothing.** The change is additive in full:

- `CurlConfig` is **unchanged** — no field added, no attribute added, so
  struct-literal construction in consumer code keeps compiling. It was
  deliberately **not** made `#[non_exhaustive]`, because that attribute alone
  would have been source-breaking.
- `CurlTransport` keeps every existing public item and gains a **private**
  field, which no external code can name, construct, or destructure.
  `with_executable` keeps its exact signature and behavior.
- The new items — `MAX_USER_AGENT_BYTES`, `with_executable_and_user_agent`,
  `user_agent()` — are additions. The only conceivable friction is a name
  collision with a consumer's own glob-imported `MAX_USER_AGENT_BYTES`, which
  is the ordinary friction of any additive API.
- No trait implementation was added or removed; no error variant was added or
  retyped; no dependency, feature, or default changed.

### Behavioral compatibility

- **Default (no `User-Agent` configured):** argv and wire behavior are
  unchanged. The bytes a consumer of the published crate sends are identical.
- **Opt-in (`User-Agent` configured):** a `--user-agent` element appears in the
  child's argv before `--` and the URL, and the peer observes the caller's
  `User-Agent` header.

### Publication status

**No republication was performed by this milestone.** `eggup-curl` remains
published at `0.1.2` on crates.io with its current shape; the `v0.1.2` tag is
not moved; no version is bumped; no crates.io write of any kind occurred. The
in-tree manifest inherits the workspace version `0.1.3`, which for this crate
was **never published** — that is the pre-existing repository state, unchanged
by this milestone. The published `0.1.2` artifact keeps whatever this
milestone changes only after an explicit, separately-governed republication
(the release workflow: manual, dependency-ordered, with the exact-pin
cascade). Because the change is additive, such a republication would be a
non-breaking `0.1.3`.

No other crate in the workspace pins `eggup-curl`
(`crates/eggup-eggpack/Cargo.toml` pins `eggup-core`, `eggup-archive`, and
`eggup-acquisition` only), so there is no pin cascade to plan.

### Migration

None required. Adopting the control is optional and additive.

## Security review

| Question | Finding |
|---|---|
| Can a caller-supplied value inject a header? | **No.** `CR` and `LF` are rejected (R3) before the value can reach the wire. This is a real defense, not a formality: `curl` does **not** sanitize `--user-agent`, and a `CR LF` value was observed being written into the request head verbatim, producing an injected `X-Injected: 1` header. |
| Can a caller-supplied value become an option or a second URL? | **No.** The value is one literal argv element immediately after `--user-agent`; `curl` consumes the next element as the value unconditionally. It is emitted **before** the `--` separator, so it can never be read as part of the URL. No shell is involved. |
| Can a rejected value cause a silent fallback to another transport? | **No.** Every rejection is `InvalidInput`, which is terminal under every composition policy. A NUL, had it reached the argv, would have made `spawn()` fail with `io::ErrorKind::InvalidInput` → `Unavailable` → fallback; the validation prevents that path from existing. |
| Can the value escape the header into the request body or the URL? | **No.** It is a single argv element with no control characters; no C0/C1 control character and no surrounding whitespace survives validation. |
| Does the value leak into diagnostics? | **No.** The value never enters an error string. The four rejection messages are fixed category text ("user agent must not be empty", "… exceeds the maximum length", "… must not contain control characters", "… must not begin or end with a space"), and they are bounded by the seam's shared 512-byte UTF-8-safe rule. The value itself is never echoed. |
| Does the control imply authenticity, publisher trust, or provenance? | **No.** No doc, rustdoc, or error string claims any such thing. A `User-Agent` is an identity string; SHA-256 remains integrity evidence only. |
| Is the bound adequate? | 512 bytes of UTF-8, the same magnitude the adapter already uses for bounded diagnostics, keeps the outbound header a bounded field value. |

## Documentation / operations evidence

| Document | What changed |
|---|---|
| `crates/eggup-curl/src/lib.rs` | Production change (§ "Production implementation evidence" above) plus the six new tests and the `serve_once_recording_head` harness. New public items carry rustdoc, including the exact accepted grammar and the rejection list. |
| `crates/eggup-curl/README.md` | New "Outbound `User-Agent`" section: how to set it, the exact accepted grammar, the 512-byte bound, the default (none sent; curl's own default applies), and the `InvalidInput`-is-terminal rule. A bullet in the feature list points at it. |
| `crates/eggup-curl/CHANGELOG.md` | New `## Unreleased` entry "Adapter capability parity: a caller-settable `User-Agent` (Acquisition Transport M014)" describing the added control, the validation rules, the unchanged default wire behavior, and the no-republication statement. **No published version section was touched.** The M013 entry's now-stale sentence "the adapter never gains a `User-Agent` surface" was qualified in place to "gains no `User-Agent` surface **in M013** (M014 later added one; see the M014 entry above)". |
| `architecture/acquisition.md` | §11.1 updated **in place**: the heading is now "recorded, resolved by M014"; the interim-difference table is retained and followed by the resolution, the capability-parity definition, the explicit "not placement parity" and "not default-value parity" statements, the validation/terminality rule, and the "how parity is enforced" boundary consequence. The M013 "Owner: deferred" paragraph is replaced by the resolution, not appended beside it. Separately, 8 `curl/src/lib.rs:…` anchors were re-aligned to the new line numbers, and the §11 review checklist gained the two `User-Agent` invariants. |
| `architecture/curl-adapter.md` | §3 public-surface table: `MAX_USER_AGENT_BYTES`, `with_executable_and_user_agent`, and `user_agent()` added; `CurlTransport`'s row now names the private field; a new API-surface note records **why** `CurlConfig` does not carry the value. §7.1: the argv block and a reviewer note describe the `--user-agent` element and its position. §9.2: the control is forwarded. §11: a new failure-mode row and two checklist items. §12: test count 28 → 34, the third harness described, six table rows added, platform-gating counts corrected. §13: a `User-Agent` row in the trade-offs table and a new §13.1 carrying the placement decision, the grammar and bound, the "curl does not sanitize" evidence, the `InvalidInput`-not-`Unavailable` rule, the unchanged default, the parity definition, and the boundary consequence. **Also: 145 `lib.rs:…` anchors and 28 test-table line anchors were re-aligned to the new line numbers**, and a pre-existing drift was corrected — §12's integration-suite table claimed a `tests/typed_io_contract.rs` suite with 2 tests that has **never existed in this crate** (it exists in `eggup-acquisition` and `eggup-eggfetch`); that row was removed and the platform-gating counts (11/9/7 = 27) were corrected to the true 17/10/7 = 34. |
| `architecture/eggfetch-adapter.md` | §12.1 updated **in place**: the heading is now "recorded, resolved by M014"; the M013 wording is retained as history and followed by the resolution, the placement and default-value statements, the validation/terminality rule, and the "how parity is enforced" boundary consequence. The §12 trade-off row that said the curl adapter had "no user-agent field" was updated in place to name the new control and to point at §12.1. **The eggfetch adapter's own API and behavior are unchanged** — only the record was updated. |
| `plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md` | New plan; `Status:` updated from `ready for handoff` to `implemented` (see "Implementation status" below). |
| `plans/closure/acquisition-transport/014-status.md` | This record. |

**Documents deliberately NOT changed** (owned by the parent session or another
milestone): `plans/registry.md`, `plans/subsystems/**`, the root `CHANGELOG.md`,
`architecture/overview.md`, `architecture/tooling-governance.md`,
`crates/eggup-acquisition/**`, `crates/eggup-eggfetch/**`,
`crates/eggup-core/**`, `crates/eggup-service/**`, `crates/eggup-archive/**`,
`crates/eggup-eggpack/**`, `scripts/**`, `.github/**`, `AGENTS.md`, `docs/**`.

**Working-tree note.** At the time this record was written the working tree
also carried changes owned by a **concurrent writer** — `crates/eggup-core/src/candidate.rs`
(modified), `plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md`
and `plans/subsystems/transport-footprint-roadmap.md` (untracked). Those files
were **not** touched, read for content, or depended on by this milestone, and
the tree was clean at this milestone's first inspection (`20ac770`). The
`eggup-curl` verification results above are unaffected by them: `eggup-curl`
depends only on `eggup-acquisition`, which does not depend on `eggup-core`.

## Unresolved findings

The repository's rule is that **"medium-or-higher" is the blocking-severity
threshold**: any such issue still open forces a new corrective before the
milestone closes.

| ID | Finding | Severity | Status |
|---|---|---|---|
| M014-UF-1 | The two adapters' `User-Agent` controls live in different types (`EggfetchConfig` vs `CurlTransport`) | **low** | **Closed by choice, not by change.** Placement parity is not achievable without a source break in `eggup-eggfetch 0.1.3`, which is published. Deliberate and documented in all three architecture documents. Recorded in the plan's §16 so it is not re-raised as a new finding. |
| M014-UF-2 | The two adapters' default `User-Agent` values still differ (`eggup-eggfetch` vs "none sent") | **low** | **Closed by choice, not by change.** Aligning them would change the observable wire behavior of published `eggup-curl 0.1.2` consumers for no concrete benefit. Deliberate and documented. |
| M014-UF-3 | The curl adapter's accepted-value grammar is stricter than the eggfetch adapter's (the HTTP header library's rules) | **low** | **Closed by choice, not by change.** The curl grammar is a strict subset of a well-formed HTTP field value, so any value both adapters accept behaves identically. Documented. |
| M014-UF-4 | §12 of `architecture/curl-adapter.md` claimed an integration suite (`tests/typed_io_contract.rs`, 2 tests) that has never existed in this crate, and under-counted the platform-gating split (11/9/7 = 27 instead of 17/10/7 = 34) | **low** | **Corrected in this milestone** while editing §12. The claimed file belongs to `eggup-acquisition` and `eggup-eggfetch`. Recorded here because it was a pre-existing doc/code drift, not something M014 introduced. |

**No finding is open at medium or higher.** No follow-on corrective is
required for this milestone. The plan's §16 names **M016** as the natural
candidate if the parent session later wants the additive control to reach
published consumers (a manual `eggup-curl 0.1.3` republication); that is a
separate, explicitly-governed act and is **not** performed here.

## Roadmap disposition

**The parent session owns `plans/subsystems/acquisition-transport-roadmap.md`.**
This milestone does not edit it. The intended entry is drafted here for the
parent to apply.

Intended roadmap status-table row:

> | Acquisition transport | M014 adapter capability parity corrective | implemented 2026-10-23; closure `plans/closure/acquisition-transport/014-status.md` | Closed M013's UF-1 (medium): `eggup-curl` now exposes a caller-settable outbound `User-Agent` (`CurlTransport::with_executable_and_user_agent`), so both acquisition adapters offer the same caller capability. Capability parity only — placement parity and default-value parity are explicitly **not** claimed and are documented as deliberate. The value is validated before the wire (R1-R6, 512-byte bound) and every rejection is `InvalidInput`, never `Unavailable`, so a malformed value can never trigger a transport fallback. The published `eggup-curl 0.1.2` default wire behavior is unchanged. No seam change, no eggfetch change, no dependency, no republication. |

Intended subsystem narrative addition (to the acquisition-transport section):

> **M014 (2026-10-23)** closed the adapter-capability-parity finding M013
> deferred as UF-1. The `eggup-curl` adapter gained a caller-settable outbound
> `User-Agent`; the control lives on `CurlTransport` rather than `CurlConfig`
> because `CurlConfig`'s public, non-`#[non_exhaustive]` fields make an added
> field source-breaking, while `CurlTransport`'s private fields do not. The
> value is validated against a stated grammar before it can reach the wire, and
> the rejection is the terminal `InvalidInput` variant so it can never become a
> transport fallback. The parity claim is **capability** parity: the eggfetch
> control stays in `EggfetchConfig` (published `0.1.3`) and the two default
> values remain different, deliberately, so the published curl adapter's wire
> behavior is unchanged. A mechanical cross-adapter parity test is impossible
> under transport neutrality; parity is enforced by per-adapter tests plus the
> documented contract in `architecture/acquisition.md` §11.1.

## Registry updates

**The parent session owns `plans/registry.md`, `plans/subsystems/acquisition-transport-roadmap.md`,
and the root `CHANGELOG.md`.** None of them was edited by this milestone. The
intended prose for each is drafted below for the parent to apply.

### `plans/registry.md` — dependency-ready implementation work table

Replace the existing M014 row with:

> | Acquisition transport | M014 adapter capability parity corrective | implemented 2026-10-23; closure `plans/closure/acquisition-transport/014-status.md` | Closes the adapter-capability-parity finding deferred by M013 as UF-1 (medium). `eggup-curl` now exposes a caller-settable outbound `User-Agent` (`CurlTransport::with_executable_and_user_agent`), so both adapters offer the same caller capability. Capability parity only; placement and default-value parity deliberately not claimed. Validation before the wire (R1-R6, 512-byte bound); every rejection is `InvalidInput`, never `Unavailable`. Published `eggup-curl 0.1.2` default wire behavior unchanged; no republication. |

### `plans/registry.md` — acquisition-transport subsystem summary row

The subsystem row (currently "Acquisition transport | M001-M011 closed; …") should
gain M014 to its closed list:

> Acquisition transport | M001-M011 closed; M009 published `eggup-curl 0.1.2` (2026-10-04), M010 published `eggup-acquisition 0.1.3` (2026-10-06), M011 published `eggup-eggfetch 0.1.3` (2026-10-06); M012 registered, `ready for handoff`; **M014 implemented 2026-10-23 (adapter capability parity corrective — M013's UF-1 closed; capability parity only; no republication)** | `plans/closure/acquisition-transport/` |

### Root `CHANGELOG.md`

The root changelog aggregates but does not replace the per-crate changelogs.
Intended entry:

> ### Acquisition transport — M014 adapter capability parity corrective
>
> `eggup-curl` gained a caller-settable outbound `User-Agent`, closing the
> adapter-capability-parity finding M013 deferred. The control lives on
> `CurlTransport` (private field, fallible constructor, public accessor), not on
> `CurlConfig`, whose public and non-`#[non_exhaustive]` fields would make an
> added field source-breaking. The value is validated before it can reach the
> wire and every rejection is the terminal `InvalidInput` variant, so a
> malformed value can never trigger a transport fallback. The published
> `eggup-curl 0.1.2` default wire behavior is unchanged; **no republication was
> performed**. See `crates/eggup-curl/CHANGELOG.md` (Unreleased) and
> `plans/closure/acquisition-transport/014-status.md`.

### `architecture/acquisition.md` and `architecture/curl-adapter.md` cross-links

Both documents already carry the M014 parity record in place. If the parent
prefers to add the closure record to the "Planning and closure" cross-reference
lists in `architecture/curl-adapter.md` §14, the intended line is:

> - [`plans/closure/acquisition-transport/014-status.md`](../plans/closure/acquisition-transport/014-status.md)
>   — the adapter capability parity record: the curl adapter's caller-settable
>   `User-Agent`, its validation grammar and bound, the placement decision, and
>   the unchanged published default.

## Implementation status

The plan's `Status:` header was updated from `ready for handoff` to
`implemented` once this record existed, per
`plans/implementation/README.md` and the workflow in
`architecture/tooling-governance.md` §6.

No "Parent-session verification addendum" section appears in this record: that
section is added by the parent session after the hosted CI qualification.

## Parent-session verification addendum (2026-10-24, hosted qualification)

**Hosted CI run `40062299630`** (commit `b9885fa`) — **all five lanes green**.

| Lane | Result | Evidence |
|---|---|---|
| `stable` (Linux) | **success** | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-targets --all-features --locked`, `cargo doc --workspace --no-deps --locked`, and the packaged-service step: **518 passed, 0 failed, 48 test binaries** |
| `msrv` (Linux, 1.89.0) | **success** | `cargo check --workspace --all-targets --locked` at the declared floor and the packaged-service step. This lane runs no tests |
| `macos` | **success** | full `cargo test` plus the packaged-service step: **518 passed, 0 failed, 48 test binaries** |
| `windows-check` | **success** | nine targeted `cargo test` steps plus the full workspace run: **637 passed, 0 failed, 68 test binaries** |
| `service-systemd` (Linux) | **success** | the real-systemd fixtures: **9 passed, 0 failed, 7 test binaries** |

The Windows lane's 637 against the other lanes' 518 is **not** a coverage
difference. That lane runs nine separate `cargo test` steps, so its aggregate
double-counts the binaries the other lanes run once, and it additionally runs
`--all-features` for `eggup-eggpack`. The per-step counts are the evidence; the
aggregate is a sum over steps, not a distinct-test count.

### What this supersedes

The platform table above records **Linux: not run** and **Windows: not run**,
and the note that hosted CI qualification is the parent session's
responsibility. **Both are superseded.** This milestone is now qualified on all
three platforms, and the hosted lanes are the authoritative evidence for Linux
and Windows.

The `eggup-curl` suite — including the new `User-Agent` control's tests and the
crate's `assertion_guard` — ran natively in the `stable` lane on Linux, in the
`macos` lane, and in the `windows-check` lane. The control's validation,
its `--user-agent` argv emission, and its terminal `InvalidInput` rejection
are therefore not macOS-only observations.

### Compatibility consequence, carried forward

Unchanged by the hosted qualification, and stated here so it is not lost:
`eggup-curl` is published at **`0.1.2`** and the published crate still carries
no `User-Agent` control. The `v0.1.2` tag is unmoved, no republication was
performed, and the `v0.1.3` republication milestone named by this plan's §16
remains the delivery path. The `eggup-acquisition 0.1.3` seam is unchanged.

## Superseded addendum (2026-10-24, Acquisition Transport M016)

**Acquisition Transport M016** — plan
`plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md`,
closure `plans/closure/acquisition-transport/016-status.md` — supersedes two
statements in this record. The record's own text is left in place, as a closed
closure's text must be; this addendum is the record of the correction.

### What this supersedes

| # | Superseded statement | Where | Corrected statement |
|---|---|---|---|
| S1 | "Because the change is additive, such a republication would be a non-breaking `0.1.3`." | § "Publication status" | **False once the seam's next release is published.** The seam's next release is `eggup-acquisition 0.2.0` and it is **source-breaking**; `0.2.0` does **not** satisfy the `^0.1.0` requirement this crate declares today, so the requirement **must widen to `0.2`** at publication time. The workspace is **version-lockstep**, so the seam's `0.2.0` release moves the workspace version to `0.2.0` and this crate's next publication is **`eggup-curl 0.2.0`, not `0.1.3`**. |
| S2 | `SP = %x20 ; permitted only between two non-SP characters` | § "Validation grammar and bound implemented" | The implementation permits a **run** of interior `SP` (only a leading and a trailing `SP` are refused). RFC 7230 §3.2 permits the run (`field-content = field-vchar [ 1*( SP / HTAB ) field-vchar ]`). M016 corrects the **documentation**; the accepted value set, the rejection set, and the argv emission are unchanged. |

### The consumer-facing consequence, stated once

A consumer that depends on the seam **directly** inherits the seam's source
break when it moves to `0.2`. A consumer of `eggup-curl` inherits it too,
once `eggup-curl` is republished with the widened requirement, because the
seam's `AcquisitionError` is part of the adapter's public surface
(`AcquisitionTransport::fetch` returns it). Such a consumer must update its
`AcquisitionError::Io` matchers and must stop cloning or comparing whole
`AcquisitionError` values.

What this milestone established remains true: the adapter's **own** API change
is additive and introduces no source break of its own.

### Unchanged

- **No republication was performed**, no tag was moved, and no published
  version was bumped by M016 either. `eggup-curl` remains published at
  **`0.1.2`**, `eggup-acquisition` at **`0.1.3`**.
- The published artifacts keep their current shape and behavior.
- The residual copies of the corrected `User-Agent` grammar in
  `crates/eggup-curl/README.md` and `architecture/curl-adapter.md` are owned by
  another writer and are reported in the M016 closure record.
