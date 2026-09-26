# Acquisition Transport M007 — Closure and Verification Record

Status: conditionally closed

Source plan: `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M007--boundary-safety-hardening-corrective`

Reviewed repository baseline: `b619fbd3821047e8e317343dbe49c3a41e6fbfc2` (plan-registration head; source tree was clean before implementation)

Implementation commits:

- `8d6c25c274e45d309c732250ebf40fa99b253a41` — finite artifact cap, UTF-8-safe acquisition/adapter diagnostics, curl handle-based body streaming, API/docs migration note.
- `ee515399ccb33395283938ed43b85ff65878ead7` — hosted Windows acquisition/curl test lane.
- `38bb7f0a5bc1ff02902b615062d94c4e030949a8` — retain safe curl child exit-code evidence.
- `7b3d0f95f2ed2e17b16be54258e499b68ec4e1dd` — gate Windows-hosted curl loopback cases after exit-7 evidence.
- `e7a85709d79df5d2e673e1c3720f7b2af098b244` — active-transfer cancellation/reap regression.

Hosted final implementation run: CI `36213509807` on `e7a85709d79df5d2e673e1c3720f7b2af098b244` (Stable, MSRV 1.89, macOS tests, Windows acquisition tests + workspace check all green).

## Executive finding

The three production defects are corrected. Bounded diagnostics now stop at UTF-8 boundaries in acquisition, Eggfetch, and curl. `FetchLimits` requires a positive finite artifact cap, and every fixture/native/external adapter enforces it. Curl sends the response body to stdout, sends status separately to stderr, and Eggup streams stdout into a duplicate of its still-open exclusive file handle; curl never receives the temporary pathname for a later reopen. Timeout, cancellation, overflow, and child errors kill/reap the process and join its readers before returning. Composition and release/fallback policy are unchanged.

The plan is conditionally closed because GitHub's Windows hosted runner refused connections from spawned curl processes to the suite's local TCP servers (curl exit 7). Only those network-dependent loopback tests are disabled on Windows. The Windows lane still runs the acquisition fixture suite (34 passed), five curl adapter/process/error-path tests (5 passed), and the workspace all-target check. Linux and macOS run the curl HTTP integration suite, including redirect, status, bounds, timeout, and active-transfer cancellation. No Windows live-HTTP curl behavior is claimed. No high- or medium-severity implementation finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| UTF-8-safe bounded acquisition diagnostics | Boundary tests cover 2-, 3-, and 4-byte characters crossing 256/512 byte ceilings; acquisition 36 tests, Eggfetch 25 tests, curl 18 tests on Darwin | passed |
| Every artifact acquisition has a positive finite cap | `FetchLimits.max_artifact_bytes: u64`; zero rejected at validation; fixture/Eggfetch/curl enforce bound; Eggpack binding only tightens to exact manifest size | passed |
| Curl cannot reopen an Eggup-exclusive temp by pathname | Curl arguments use `--output -`; body-reader writes through a duplicate of the retained exclusive `File`; fake-child assertion pins `--output -` | passed |
| Body and status are separate, both drained while child runs | Body uses stdout reader thread; bounded status uses stderr reader thread; exact 404 / 5xx classification tests on Linux/macOS | passed |
| Byte cap, timeout, cancellation, and process cleanup remain truthful | exact/over-limit adapter tests; total-timeout test; active-transfer cancel test observes `Cancelled` and no temp residue; child is killed, waited, readers joined | passed on Linux/macOS |
| Redaction, no-clobber, fallback, and dependency footprint remain intact | existing redaction/no-clobber/composition suites green; `cargo tree -p eggup-core --locked` unchanged (sha2 only); no Cargo dependency changes | passed |
| Windows compile and portable runtime qualification | hosted run `36213509807`: acquisition 34/34, curl 5/5, workspace check green | passed; curl loopback limitation recorded above |
| Known downstream `None` use is inventoried | Eggsact `src/update.rs::eggup_limits` supplies `None` for metadata only; CodeGG `src/upgrade/managed.rs::fetch_metadata` calls `FetchLimits::new(max, None, ...)` and its artifact adapter handles `Option`. Both consume earlier Eggup revisions; required migration is `None` → a chosen finite cap before adopting this API. No consumer intentionally requests unbounded artifact bytes. | migration enumerated; downstream repositories not changed or released |

## Production implementation evidence

- `crates/eggup-acquisition/src/lib.rs`: `FetchLimits.max_artifact_bytes` is now `u64`; validation rejects zero; `new` accepts a finite `u64`; fixture transport checks every artifact. Shared UTF-8-safe byte truncation is used by bounded diagnostics, URL redaction, and upstream-text scrubbing. Regression coverage includes 2/3/4-byte boundary cases at 256 and 512 bytes and the zero-cap rejection.
- `crates/eggup-eggfetch/src/lib.rs`: streamed artifact byte accounting always compares against the finite cap; adapter diagnostics use a character-boundary limit.
- `crates/eggup-curl/src/lib.rs`: stdout is body data and stderr is bounded HTTP status. A cloned descriptor referencing Eggup's exclusive file is retained by the body reader; curl receives no temp path. Streaming enforces `max_bytes + 1` as the detection ceiling and kills/reaps on overflow/cancel/deadline. Status is drained concurrently and retained only up to its fixed bound. Tests pin the `--output -` invocation and exercise cancellation while a transfer is active on supported loopback hosts.
- `crates/eggup-eggpack/src/lib.rs` and interoperability tests: request binding compares and writes finite integer limits while preserving exact-size tightening.
- `.github/workflows/ci.yml`: Windows runs `cargo test -p eggup-acquisition -p eggup-curl --locked` before the workspace check.
- No dependency or feature changes. `eggup-core` remains transport-free and archive-free.

## Exact commands and results

Local Darwin arm64, Rust 1.89.0:

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test --workspace --all-targets --all-features --locked    passed (254 before final cancellation regression; latest hosted full suite passed)
cargo doc --workspace --no-deps --locked                        passed
cargo tree --workspace --locked                                  passed; no dependency change
cargo tree -p eggup-core --locked                                passed; sha2 only
cargo test -p eggup-acquisition --locked                        passed (36)
cargo test -p eggup-eggfetch --locked                           passed (25)
cargo test -p eggup-curl --locked                               passed (18 after active-cancel regression)
cargo test -p eggup-curl cancellation_during_body_stream_kills_and_reaps_child --locked  passed (1)
git diff --check                                                passed
```

Hosted CI run `36213509807` on final implementation SHA:

- Stable Linux: fmt, clippy, workspace tests, docs — passed.
- MSRV Linux: Rust 1.89 workspace all-target check — passed.
- macOS: full workspace tests — passed.
- Windows: acquisition tests 34 passed; curl tests 5 passed; workspace all-target check — passed. Network-dependent curl loopback integration tests were not run on Windows because the prior hosted attempts consistently returned exit 7 connecting to their local server.

Earlier Windows experimental runs `36213125910` and `36213249799` are retained as failure evidence. They showed curl exit 7 for local loopback URLs; after gating only those unsupported cases, run `36213376971` passed all lanes, and final run `36213509807` added the active-cancellation regression and remained green.

## Invariant review

- Exact URL remains authoritative; HTTP 404 stays terminal and no source/release fallback was added.
- Default composition still falls back only for `Unavailable`; explicit broader transport fallback behavior is unchanged.
- Caller and adapter timeout ceilings still use the minimum. Curl has its independent parent deadline and kills/reaps before return.
- All artifact fetches have a finite positive cap. Eggpack's exact-size binding cannot widen a caller's cap.
- Curl writes only through the handle Eggup authorized. It never reopens a staging pathname; promotion remains same-directory no-clobber.
- Diagnostics remain bounded and redacted; raw curl/stderr content is not surfaced. Exit status is numeric category evidence only.
- `eggup-core` gains no transport or archive dependency.

## Failure and recovery review

| Failure | Result |
|---|---|
| Invalid zero artifact limit | `InvalidInput` before route, filesystem, or network work |
| Artifact exceeds cap | `TooLarge { limit }`; incomplete owned temp removed |
| Cancellation/deadline while curl runs | child killed and reaped; readers joined; only owned temp removed |
| Curl body/status pipe or owned-file write failure | child cannot be promoted; bounded category error; owned temp cleanup |
| Partial HTTP transfer/non-success status | hard `Transport`; incomplete output removed; only 404 is `NotFound` |
| Destination exists or races in | no-clobber promotion fails; foreign destination preserved |
| Successful promotion followed by redundant-temp cleanup failure | destination remains success per existing promotion contract |
| Windows hosted loopback restriction | integration claim is withheld; portable tests and compile check continue; no product recovery state is implied |

No destination, release, service, ownership, or commit behavior changed.

## Compatibility and migration review

This is a pre-1.0 source-breaking change: replace `max_artifact_bytes: Some(n)` with `max_artifact_bytes: n`; `None` is removed. `FetchLimits::new` likewise takes `u64`. The default remains 128 MiB. The crate README and root `CHANGELOG.md` record the migration; no publication occurred.

Repository-wide source audit found no in-workspace `None` caller after the migration. The checked neighboring consumer repositories contain only the two legacy metadata-only `None` sites listed above and currently pin older Eggup revisions. Their existing source does not need an immediate change until it updates its Eggup dependency; the migration must select a finite cap at that point. No downstream repository was edited as part of this Eggup workspace pass.

## Security review

The pathname-reopen race is removed at the authority boundary: curl sees stdout, while Eggup writes to its retained exclusive descriptor. The parent drains both pipes concurrently, enforces the byte ceiling while bytes arrive, and reaps the child before classifying cancellation, timeout, or overflow. Status capture is bounded; raw stderr and proxy material remain private. UTF-8 truncation cannot panic or emit invalid strings. No medium-or-higher related finding remains open.

## Documentation and operations evidence

- `crates/eggup-acquisition/README.md`: finite cap semantics and `Some(n)` → `n` migration.
- `crates/eggup-curl/README.md`: retained-handle body streaming and separate status channel.
- `CHANGELOG.md`: Unreleased M007 entry; no publication or consumer migration performed.
- Roadmap and registry updated below. Architecture layer boundaries remain unchanged.

## Unresolved findings

- Informational/operational: Windows-hosted spawned curl cannot connect to local loopback fixture servers (exit 7). Windows real curl network behavior remains unqualified; Linux/macOS curl integration plus Windows portable adapter/process tests and compile check are green. Revisit if Eggup adds a Windows runner with working child-process loopback.
- Informational/migration: Eggsact and CodeGG old revisions have metadata-only `None` uses. Before updating those consumers to this API, choose finite metadata-operation artifact caps and migrate to `u64`; no unbounded artifact requirement was found.
- None: no high- or medium-severity implementation issue remains.

## Roadmap disposition

Acquisition M007 moves from ready to conditionally closed. The subsystem's transport contract is corrected and hosted stable/MSRV/macOS/Windows lanes are green with the explicit Windows loopback limitation above. No later milestone is blocked on M007; Archive Extraction M001's hard prerequisites (Core M007 and Acquisition M006) were already closed. Egress M006 and Eggpack interoperability M002 remain blocked on Archive Extraction M001.

## Registry updates

- Acquisition M007: ready → conditionally closed; implementation/API migration recorded.
- Windows CI adds acquisition/curl runtime tests plus the existing workspace check.
- Archive Extraction M001 stays the next dependency-transition milestone; no downstream blocker is cleared before its own closure.
- Service M007 stays ready and follows Archive M001 in this requested sequence.
