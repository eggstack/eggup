# Eggpack Manifest Interoperability M006 — Closure and Verification Record

Status: **conditionally closed** — the milestone's own scope is complete and green on
macOS, and every requirement in the plan's §§6, 10, and 13 is either implemented and
evidenced or explicitly recorded below as deferred with a reason. The remaining
conditions are **outstanding qualification** (F-1): the hosted Linux and Windows lanes
and the full-workspace local gate were not run from this host, and `cargo fmt --all
-- --check` currently fails on a file owned by a concurrent writer (F-2), not on any
file this milestone owns.

Source plan: `plans/implementation/eggpack-manifest-interoperability/006-canonical-digest-reuse-corrective.md`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Reviewed repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`
(branch `plans/service-m010-failed-unit-recovery`; `HEAD` is identical to the baseline
SHA, so the baseline worktree used for the red-first proof is the committed
pre-M006 state of this crate)

Implementation commit(s): **working tree, uncommitted — no commit was made.** The
changes are in `crates/eggup-eggpack/src/lib.rs`,
`crates/eggup-eggpack/Cargo.toml`, `crates/eggup-eggpack/tests/digest_reuse.rs`
(new), `crates/eggup-eggpack/README.md`, `crates/eggup-eggpack/CHANGELOG.md`,
`architecture/eggpack-adapter.md`, the `Status:` header of the source plan
`plans/implementation/eggpack-manifest-interoperability/006-canonical-digest-reuse-corrective.md`,
and this closure record. **`Cargo.lock` is byte-identical to the baseline**
(see *Dependency-edge evidence*). `crates/eggup-eggpack/tests/archive_handoff.rs`
was already modified in the working tree by the parent session (the archive-M005
cleanup-contract correction) and was **preserved verbatim** by this milestone. No
publication, no version bump, no tag move, and no edit to `plans/registry.md` or
the subsystem roadmap — those are the closing parent session's files.

Finding of record: `plans/registry.md` § "Post-0.1.3 open findings" entry **9**
("`eggup-eggpack` re-implements core's file-digest routine and carries a direct
`sha2` dependency for that one call site"), assessed **medium** severity. This
milestone is the corrective for that finding.

## Executive finding

**Before.** `eggup-eggpack` computed the acquired archive's SHA-256 with an inline
streaming loop in `validate_acquired_archive` (`crates/eggup-eggpack/src/lib.rs:504-517`
at the baseline), and therefore carried a direct `sha2 = "0.10.9"` runtime dependency
whose **only** consumer in the crate was that loop — while the crate already depended
on `eggup-core`, which exports the identical capability as `eggup_core::hash_file`.
Two implementations of one routine, one redundant direct dependency edge on an
already-published crate, and a second version-resolution obligation in the adapter's
own manifest.

**After.** Exactly one file-digest implementation exists in the workspace: the adapter
delegates to `eggup_core::hash_file`. The direct `sha2` runtime edge is gone from
`crates/eggup-eggpack/Cargo.toml`; `sha2` remains a **dev-dependency only**. The
symlink rejection and the exact-size check remain, and remain **textually and
logically before** the digest computation — proven, not asserted, by a FIFO-ordering
fixture that would block the lane if the guard moved. The digest value, the error
variant, and the error payload are **unchanged**: open failure, read failure, and
digest mismatch all still report `AdapterError::InvalidAcquiredFile(artifact_name)`.

No behavior change was intended and none was introduced. The resolved package set did
not change either: `sha2 v0.10.9` remains in the graph through `eggup-core` and
`eggup-archive`, and `Cargo.lock` is byte-identical to the baseline.

## Requirement → evidence matrix

### §13 acceptance criteria

| # | Acceptance criterion | Evidence | Result |
|---|---|---|---|
| 1 | No `sha2` import in non-test code; digest computed by `eggup_core::hash_file` | `crates/eggup-eggpack/src/lib.rs:9` imports `hash_file` from `eggup_core`; `:522-523` is the only digest call; `grep -n "sha2" crates/eggup-eggpack/src/lib.rs` matches only `sha256` field names, never the crate | pass |
| 2 | `[dependencies]` contains no `sha2`; the dev-dependency remains, reason recorded | `crates/eggup-eggpack/Cargo.toml:19-23` (no `sha2`); `:31` `sha2 = "0.10"` under `[dev-dependencies]`; reason in `crates/eggup-eggpack/CHANGELOG.md` `Unreleased` and `architecture/eggpack-adapter.md` § "Position in the workspace" | pass |
| 3 | `cargo tree -p eggup-eggpack --locked -i sha2` no longer lists the adapter as a **direct runtime** dependent; resolved version still `0.10.9`; no package enters or leaves | see *Dependency-edge evidence* — before/after diff removes exactly one runtime edge; `sha2 v0.10.9` unchanged; the full tree diff is 3 lines | pass |
| 4 | The `=0.1.3` / `=0.1.0` pins are byte-identical to the baseline | `git diff -- crates/eggup-eggpack/Cargo.toml` touches exactly one deleted line (`sha2 = "0.10.9"`); the four pin lines are context, unchanged | pass |
| 5 | The `symlink_metadata` guard precedes the digest computation; the FIFO fixture proves it | guard at `lib.rs:513-517`, digest at `lib.rs:522-523`; fixture `symlink_guard_precedes_the_open_so_a_fifo_target_never_blocks` (`tests/digest_reuse.rs`), Unix-only, bounded 10 s wait | pass |
| 6 | Symlinked / size-mismatched / digest-mismatched archives each fail closed with `AdapterError::InvalidAcquiredFile` | fixtures `symlinked_acquired_archive_is_rejected_even_when_the_target_matches` (Unix), `size_mismatch_is_rejected_before_the_digest_comparison`, `digest_mismatch_still_fails_closed` | pass |
| 7 | The open-failure fixture asserts the core error is mapped to `InvalidAcquiredFile`, not leaked as another variant | fixture `digest_open_failure_is_mapped_to_invalid_acquired_file` (Unix); it **ran** (no skip message) — see *Per-platform results* | pass |
| 8 | The digest for a known artifact equals the known constant and equals `eggup_core::hash_file`'s output | fixture `known_artifact_digest_matches_the_published_constant_and_the_core_routine`: `b"abc"` → `ba7816bf…15ad`, compared against `hash_file(&path)` | pass |
| 9 | The red-first manifest fixture fails at the baseline and passes after M006 | see *Red-first proof* | pass |
| 10 | `AdapterError`'s variants, payloads, and behavior are unchanged | `crates/eggup-eggpack/src/lib.rs:18-58` (enum + `Display` + `std::error::Error`) untouched by the diff; the diff adds only rustdoc and swaps the digest computation | pass |
| 11 | No authenticity, signature, provenance, or trust claim; SHA-256 remains integrity evidence only | no authenticity vocabulary added; the crate's own boundary test `adapter_source_claims_no_producer_or_service_authority` (`tests/interoperability.rs`) still passes; `architecture/eggpack-adapter.md` § "The verification ladder" still states the integrity rung only | pass |
| 12 | The adapter's policy surface is unchanged | no URL/origin, install-root, ownership, permission, release-selection, service, or authenticity code touched; the diff is confined to one function body | pass |
| 13 | `eggup-eggpack` remains the only crate with an `eggpack-manifest` edge | `cargo tree --workspace --locked --offline -i eggpack-manifest` → `eggpack-manifest v0.1.0 └── eggup-eggpack v0.1.3` only; `cargo tree -p <crate>` for `eggup-core`, `eggup-acquisition`, `eggup-service`, `eggup-archive`, `eggup-eggfetch`, `eggup-curl` shows **no** eggpack edge | pass |
| 14 | `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, workspace `unsafe_code = "deny"`, clippy `all = warn` with `-D warnings` | crate attributes untouched (`lib.rs:1-2`); clippy clean with and without `--all-features`; `cargo doc -p eggup-eggpack --no-deps --locked --offline` builds with no warnings | pass |
| 15 | `./scripts/check-local.sh` green on the local machine | **NOT RUN** by this session — the session's instruction scoped testing to the adapter package, and the full workspace gate is the parent session's. The adapter-scoped equivalent is green (see *Exact commands run*). See F-1. | **not run** |
| 16 | Hosted lanes `stable`, `msrv`, `service-systemd`, `macos`, `windows-check` recorded individually | see *Per-platform results* — **all not run** from this host. | **not run** |
| 17 | Nothing published; the `v0.1.3` tag not moved | no `cargo publish`, no `git tag`, no version bump; `crates/eggup-eggpack/Cargo.toml` still `version.workspace = true` (`0.1.3`); `cargo package` produced a local artifact only | pass |

### §6 required production changes

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 6.1 | Use the core-provided digest routine; drop the `sha2` and `std::io::Read` imports; keep the comparison in the adapter | `lib.rs:9` (`hash_file` imported from `eggup_core`); `:522-523` the digest call; `:524-526` the adapter's own comparison against the manifest digest | pass |
| 6.2 | Error mapping: open failure, read failure, and digest mismatch all map to `AdapterError::InvalidAcquiredFile(artifact_name.clone())`; **not** bypassed with `?` into another variant; rationale recorded | `lib.rs:522-523` uses `.map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?` — the core error is consumed by the closure, never converted into `AdapterError::Eggup`. Rationale recorded in the function's `# Errors` rustdoc (`lib.rs:482-496`) and in `architecture/eggpack-adapter.md` § "Invariants and failure modes" (third property). Narrowing was **not** smuggled in: `AdapterError`'s variants are byte-identical to the baseline | pass |
| 6.3 | Ordering preserved: the `symlink_metadata` guard stays textually and logically before the `hash_file` call | `lib.rs:513-514` (`symlink_metadata`), `:515-517` (type + size guard), `:522` (`hash_file`) — the guard is above the call in the source; fixture `symlink_guard_precedes_the_open_so_a_fifo_target_never_blocks` proves it behaviorally | pass |
| 6.4 | No chunk-size constant reintroduced; no rustdoc claim about a chunk size the adapter no longer controls; `architecture/eggpack-adapter.md` "streamed in 16 KiB chunks" corrected | `grep -n "16 \* 1024\|KiB" crates/eggup-eggpack/src/lib.rs` → no match; the adapter rustdoc says only "bounded streaming reads"; `architecture/eggpack-adapter.md` verification-ladder row now reads "streamed through `eggup_core::hash_file` (`lib.rs:522-523`)" | pass |
| 6.5 | Remove `sha2 = "0.10.9"` from `[dependencies]`; keep `sha2 = "0.10"` in `[dev-dependencies]`; pins unchanged; record the lock diff and confirm the resolved `sha2` version is unchanged | `crates/eggup-eggpack/Cargo.toml:19-23` and `:31`; `git diff -- Cargo.lock` is **empty**; resolved version `sha2 v0.10.9` (unchanged) | pass |

### §10 required fixtures

| # | Required fixture | Test name | Implementation | Result |
|---|---|---|---|---|
| 1 | Known artifact, digest unchanged (pass-to-pass) | `known_artifact_digest_matches_the_published_constant_and_the_core_routine` | `tests/digest_reuse.rs` | pass |
| 2 | Symlinked archive rejected before hashing (Unix-only) | `symlinked_acquired_archive_is_rejected_even_when_the_target_matches` | `tests/digest_reuse.rs`, `#[cfg(unix)]` | pass (Unix-gated) |
| 3 | Ordering proof with a FIFO target (Unix-only) | `symlink_guard_precedes_the_open_so_a_fifo_target_never_blocks` | `tests/digest_reuse.rs`, `#[cfg(unix)]`; worker thread + 10 s bounded wait | pass (Unix-gated) |
| 4 | Size mismatch rejected before hashing (cross-platform) | `size_mismatch_is_rejected_before_the_digest_comparison` | `tests/digest_reuse.rs` | pass |
| 5 | Digest mismatch still fails closed (cross-platform) | `digest_mismatch_still_fails_closed` | `tests/digest_reuse.rs` | pass |
| 6 | Error mapping for open failure (Unix-only) | `digest_open_failure_is_mapped_to_invalid_acquired_file` | `tests/digest_reuse.rs`, `#[cfg(unix)]`; mode-`000` file | pass — **ran** on this host (uid 503, non-root); the skip branch did not trigger |
| 7 | Read-failure mapping | `core_read_failure_is_reported_as_an_io_error_the_adapter_can_map` | `tests/digest_reuse.rs` — source inspection of `crates/eggup-core/src/integrity.rs:78-80` (the read failure is mapped to `Error::io("reading file for hashing", source)`), asserted when the sibling core source tree is present. **No deterministic read-failure fixture exists** (a truncated file yields EOF, not an error; a directory is rejected earlier by the type check), so no such fixture is claimed. The same `Err(core_error) → InvalidAcquiredFile` branch is exercised by fixture 6 | pass, with the stated limitation |
| 8 | Red-first: no direct `sha2` runtime dependency | `the_adapter_manifest_carries_no_direct_runtime_sha2_dependency` | `tests/digest_reuse.rs`; reads `env!("CARGO_MANIFEST_DIR")/Cargo.toml` and scans the `[dependencies]` table | pass — **and FAILS at the baseline**, see *Red-first proof* |
| — | Existing fixtures remain green and unmodified in expectation | `tests/interoperability.rs` (28), `tests/archive_handoff.rs` (13), `tests/caller_destinations.rs` (15), inline (7) | all green; `tests/archive_handoff.rs` was **not** modified by this milestone (its archive-M005 cleanup-contract correction was already applied by the parent session and was preserved verbatim) | pass |

**Honest caveat, as the plan requires.** Fixtures 1–7 are pass-to-pass: they assert
behavior that is correct both before and after M006, so they **pass** against the
pre-M006 implementation. They are regression guards, not red-first proofs. The
**only** red-first fixture is number 8. M006 does not and cannot produce a red test
proving the *behavior* changed, because the behavior must not change.

## Production implementation evidence

### `validate_acquired_archive` — before and after

Before (`crates/eggup-eggpack/src/lib.rs:481-522` at the baseline):

```rust
    use sha2::{Digest, Sha256};
    use std::io::Read;
    …
    let meta = fs::symlink_metadata(acquired_archive_path)
        .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
    if !meta.file_type().is_file() || meta.len() != exact_size {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    let mut file = fs::File::open(acquired_archive_path)
        .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
    let mut hasher = Sha256::new();
    let mut chunk = [0u8; 16 * 1024];
    loop {
        let read = file
            .read(&mut chunk)
            .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
        if read == 0 {
            break;
        }
        hasher.update(&chunk[..read]);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    if digest != sha256 {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    Ok(())
```

After (`crates/eggup-eggpack/src/lib.rs:513-527`):

```rust
    let meta = fs::symlink_metadata(acquired_archive_path)
        .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
    if !meta.file_type().is_file() || meta.len() != exact_size {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    // The guard above runs first: `hash_file` follows symlinks when it opens
    // the path, so this is the only link rejection on the path. The digest
    // step's own failures collapse into the same variant and payload as the
    // mismatch below, preserving the adapter's published error surface.
    let digest: [u8; 32] = hash_file(acquired_archive_path)
        .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
    if digest != sha256 {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    Ok(())
```

Four ordered facts to hold onto:

1. **The guard is above the digest call, textually and logically.** `symlink_metadata`
   at `:513-514`, the type + size guard at `:515-517`, the digest at `:522`.
2. **The error mapping is applied, not bypassed.** `hash_file` returns
   `eggup_core::Result<[u8; 32]>`; the `.map_err(|_| …)?` closure consumes the core
   error and produces `AdapterError::InvalidAcquiredFile(artifact_name.clone())`. There
   is no `?` on the core `Result` that could fall through into `AdapterError::Eggup`
   (the `From`/`?` conversion path that `AdapterError` does not implement for core
   errors — `AdapterError` has no `From<eggup_core::Error>` impl at all).
3. **The adapter still owns the comparison.** `hash_file` returns a digest; the
   comparison against the manifest digest stays in the adapter at `:524-526`.
4. **No chunk-size constant was reintroduced.** The adapter does not control the
   chunk size any more; core's 64 KiB buffer applies. SHA-256 over a byte stream is
   chunk-independent, so the digest value is unaffected.

### Dependency-edge evidence (the key evidence)

**`git diff -- crates/eggup-eggpack/Cargo.toml`**

```diff
--- a/crates/eggup-eggpack/Cargo.toml
+++ b/crates/eggup-eggpack/Cargo.toml
@@ -21,7 +21,6 @@ eggup-core = { version = "=0.1.3", path = "../eggup-core" }
 eggup-archive = { version = "=0.1.3", path = "../eggup-archive" }
 eggup-acquisition = { version = "=0.1.3", path = "../eggup-acquisition" }
 eggpack-manifest = "=0.1.0"
-sha2 = "0.10.9"

 [dev-dependencies]
 serde = { version = "1", features = ["derive"] }
```

One deleted line. The `=0.1.3` / `=0.1.0` pins and the `[dev-dependencies]` table are
context, unchanged.

**`git diff -- Cargo.lock`** — **empty.** `Cargo.lock` lists a package's dev-dependencies
in the same `dependencies` array as its runtime dependencies, and `sha2` remains a
dev-dependency of `eggup-eggpack`, so removing the *runtime* edge does not change the
lockfile at all. No regeneration was needed, no package entered or left the graph, and
the resolved `sha2` version is unchanged at `0.10.9`.

**`cargo tree -p eggup-eggpack --locked --offline` — before**

```text
├── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
│   └── sha2 v0.10.9 (*)
└── sha2 v0.10.9 (*)
[dev-dependencies]
├── flate2 v1.1.10 (*)
├── serde v1.0.229 (*)
├── serde_json v1.0.151 (*)
├── sha2 v0.10.9 (*)
├── tar v0.4.46 (*)
└── zip v6.0.0 (*)
```

**`cargo tree -p eggup-eggpack --locked --offline` — after**

```text
└── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    └── sha2 v0.10.9 (*)
[dev-dependencies]
├── flate2 v1.1.10 (*)
├── serde v1.0.229 (*)
├── serde_json v1.0.151 (*)
├── sha2 v0.10.9 (*)
├── tar v0.4.46 (*)
└── zip v6.0.0 (*)
```

Full-tree diff (`diff tree-before.txt tree-after.txt`) — three lines, nothing else:

```text
63,65c63,64
< ├── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
< │   └── sha2 v0.10.9 (*)
< └── sha2 v0.10.9 (*)
---
> └── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
>     └── sha2 v0.10.9 (*)
```

**`cargo tree -p eggup-eggpack --locked --offline -i sha2` — before**

```text
sha2 v0.10.9
├── eggup-archive v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-archive)
│   └── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
├── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
│   └── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
└── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
[dev-dependencies]
└── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
```

**`cargo tree -p eggup-eggpack --locked --offline -i sha2` — after**

```text
sha2 v0.10.9
├── eggup-archive v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-archive)
│   └── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
└── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    └── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
[dev-dependencies]
└── eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
```

The direct runtime dependent of `sha2` is gone; the resolved version is still
`0.10.9`; no package enters or leaves the graph.

**Publishable-manifest confirmation.** `cargo package -p eggup-eggpack --locked
--allow-dirty --offline` succeeded, and the packaged crate's normalized manifest
lists exactly four runtime dependencies — `eggpack-manifest`, `eggup-acquisition`,
`eggup-archive`, `eggup-core` — with `sha2` present only under `[dev-dependencies]`.

## Exact commands run, with results

Every cargo invocation used `--offline` (no network is reachable from this machine).
Commands are recorded in the order they were run.

```text
$ cargo test -p eggup-eggpack --all-targets --all-features --locked --offline
     Running unittests src/lib.rs
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/archive_handoff.rs
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/caller_destinations.rs
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/digest_reuse.rs
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/interoperability.rs
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Total: **71 passed, 0 failed** (7 inline + 13 + 15 + 8 + 28).

```text
$ cargo test -p eggup-eggpack --locked --offline interoperability
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.00s
```

Note: the plan's §11 literal form filters by **test name**, not by binary name, so it
selects 0 tests (no adapter test's name contains "interoperability"). The meaningful
form was therefore also run:

```text
$ cargo test -p eggup-eggpack --locked --offline --test interoperability
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
$ cargo test -p eggup-eggpack --locked --offline --test digest_reuse
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```text
$ cargo clippy -p eggup-eggpack --all-targets --locked --offline -- -D warnings
    Checking eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.88s

$ cargo clippy -p eggup-eggpack --all-targets --all-features --locked --offline -- -D warnings
    Checking eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

```text
$ cargo doc -p eggup-eggpack --no-deps --locked --offline
   Documenting eggup-eggpack v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggpack)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s
    Generated /Users/davidbowman/projects/eggup/target/doc/eggup_eggpack/index.html
  (no warnings)
```

```text
$ cargo package -p eggup-eggpack --locked --allow-dirty --offline
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.84s
EXIT=0
```

```text
$ cargo fmt -p eggup-eggpack -- --check
(clean — no output)
```

```text
$ cargo fmt --all -- --check
Diff in /Users/davidbowman/projects/eggup/crates/eggup-acquisition/tests/typed_io_contract.rs
Diff in /Users/davidbowman/projects/eggup/crates/eggup-curl/src/lib.rs
Diff in /Users/davidbowman/projects/eggup/crates/eggup-eggfetch/tests/typed_io_contract.rs
exit=1
```

**This workspace-wide fmt failure is not caused by M006.** Every reported file is
owned by another concurrent writer, and each appeared in `git status` *after* this
session's first inspection (which showed all three clean). The set is a moving
target, because those writers are still working. The `eggup-eggpack` package is
fmt-clean (`cargo fmt -p eggup-eggpack -- --check` is silent). Recorded as F-2;
**not fixed here**, because those files are outside this milestone's ownership.

Commands from §11 **not run** by this session, and why:

- `cargo test --workspace --all-targets --all-features --locked` — **not run.** The
  session's instruction scoped testing to the adapter package and forbade attempting
  the whole workspace suite; the parent session runs the full gate once at the end.
  The adapter's own suite is green (71/71), and no crate outside
  `crates/eggup-eggpack/` was modified by this milestone, so no cross-crate
  regression is possible from these edits.
- `./scripts/check-local.sh` — **not run**, same reason (see F-1). Its adapter-relevant
  components (fmt for this package, clippy `-D warnings`, tests, doc) were run
  individually and are green, except the workspace fmt, which fails only in
  `crates/eggup-curl` (F-2).
- `cargo tree --workspace --locked` — not run in full; the per-crate trees the plan
  names (`-p eggup-eggpack`, and `-i sha2`) were run, plus `-p` trees for the six
  other crates to prove no Eggpack edge leaked.

## Red-first proof against the pre-M006 baseline

Mechanism: **`git worktree add /tmp/eggup-eggpack-m006-base-77648
c4805411f74b85158e0ae5d0a219a32c4209d98e`**, the new test file copied into that
worktree, the focused test run there, then
`git worktree remove --force /tmp/eggup-eggpack-m006-base-77648`. The worktree route
succeeded; the file-swap fallback was not needed. No git state was mutated: `git stash`,
`git checkout <path>`, `git restore`, `git commit`, `git push`, `git tag`, `git clean`,
and `git reset` were **not** run, and no branch was created or switched. The main
working tree's git state was not touched; `git worktree list` afterwards shows only the
primary worktree.

The new file **compiles** against the pre-M006 baseline and **fails** there — a
compile failure would not have been acceptable evidence.

```text
$ cd /tmp/eggup-eggpack-m006-base-77648
$ grep -n "sha2" crates/eggup-eggpack/Cargo.toml
24:sha2 = "0.10.9"
32:sha2 = "0.10"

$ cargo test -p eggup-eggpack --locked --offline --test digest_reuse
running 8 tests
test core_read_failure_is_reported_as_an_io_error_the_adapter_can_map ... ok
test the_adapter_manifest_carries_no_direct_runtime_sha2_dependency ... FAILED
test digest_open_failure_is_mapped_to_invalid_acquired_file ... ok
test digest_mismatch_still_fails_closed ... ok
test size_mismatch_is_rejected_before_the_digest_comparison ... ok
test known_artifact_digest_matches_the_published_constant_and_the_core_routine ... ok
test symlinked_acquired_archive_is_rejected_even_when_the_target_matches ... ok
test symlink_guard_precedes_the_open_so_a_fifo_target_never_blocks ... ok

failures:

---- the_adapter_manifest_carries_no_direct_runtime_sha2_dependency stdout ----

thread 'the_adapter_manifest_carries_no_direct_runtime_sha2_dependency' panicked at crates/eggup-eggpack/tests/digest_reuse.rs:304:5:
the adapter must not declare a direct runtime `sha2` dependency; found ["eggup-core", "eggup-archive", "eggup-acquisition", "eggpack-manifest", "sha2"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p eggup-eggpack --test digest_reuse`
EXIT_CODE=101
```

Exactly the plan's expectation: the fixture-8 manifest assertion **fails** at the
baseline (the direct `sha2` dependency is present at `Cargo.toml:24`) and fixtures 1–7
**pass**, because they assert preserved behavior. The pass-to-pass fixtures cannot be
red-first, and this record does not claim otherwise.

## Per-platform results

| Platform | Result |
|---|---|
| **macOS (darwin arm64, this machine)** | **RUN** — native. `cargo test -p eggup-eggpack --all-targets --all-features --locked --offline`: 7 inline + 13 + 15 + 8 + 28 integration = **71 tests passed, 0 failed**. Clippy (with and without `--all-features`) clean under `-D warnings`; `cargo doc` clean; `cargo fmt -p eggup-eggpack -- --check` clean; `cargo package` succeeded. The Unix-only fixtures — symlink rejection, FIFO ordering, and the mode-`000` open-failure mapping — **executed** here (the open-failure fixture ran, it did not take its skip branch; the test runner's uid is 503, non-root). |
| **Linux** | **NOT RUN** — no Linux host is reachable from this machine. |
| **Windows** | **NOT RUN** — no Windows host is reachable from this machine. The three Unix-only fixtures (`symlinked_acquired_archive_is_rejected_even_when_the_target_matches`, `symlink_guard_precedes_the_open_so_a_fifo_target_never_blocks`, `digest_open_failure_is_mapped_to_invalid_acquired_file`) are `#[cfg(unix)]` and therefore **not run** there. |
| **Hosted CI lanes** (`stable`, `msrv`, `service-systemd`, `macos`, `windows-check`) | **NOT RUN** — no hosted runner is reachable from this machine. Each lane is recorded individually as not run: `stable` **not run**; `msrv` **not run**; `service-systemd` **not run**; `macos` **not run**; `windows-check` **not run**. |

No platform result is inferred from another platform. The Unix-only fixtures are
recorded as **not run** on the Windows lane, as the plan requires.

## Invariant review (§4, item by item)

All ten §4 invariants are in the matrix above; all **pass**. Specifically:

1. **Digest value byte-identical.** The adapter's SHA-256 for a given file equals
   `eggup_core::hash_file`'s output and equals the manifest digest. Both routines
   are SHA-256 over the same byte stream; the chunk size (16 KiB → core's 64 KiB)
   does not affect the digest. Fixture 1 asserts the published `b"abc"` constant
   against `hash_file` and requires the adapter to accept the file.
2. **Symlink rejection remains and remains ordered before the digest.** The guard at
   `lib.rs:513-517` precedes the digest at `:522`; `hash_file` follows symlinks when
   it opens, so the guard is still the only link rejection in the path. The FIFO
   fixture proves the ordering behaviorally.
3. **Exact-size check remains and remains ordered before the digest.** Same guard,
   same ordering; fixture 4 proves it (the digest matches; only the size is wrong).
4. **Digest mismatch fails closed.** `lib.rs:524-526` returns
   `AdapterError::InvalidAcquiredFile(artifact_name)` — never downgraded, warned,
   retried, or passed as "unverified". Fixture 5 proves it.
5. **SHA-256 remains integrity evidence only.** No authenticity, signature, provenance,
   or trust claim was introduced; the crate's own textual boundary test still passes.
6. **The adapter's policy surface does not widen.** No URL/origin, install-root,
   ownership, permission, release-selection, service, or authenticity policy entered
   the adapter. The production diff is confined to one function body.
7. **The exact pins are unchanged.** `eggup-core`, `eggup-archive`, and
   `eggup-acquisition` remain `=0.1.3`; `eggpack-manifest` remains `=0.1.0`. The
   manifest diff is one deleted line.
8. **`eggup-eggpack` remains the only crate that touches producer types.**
   `cargo tree --workspace -i eggpack-manifest` shows only the adapter; the
   per-crate trees for the other six crates show no Eggpack edge.
9. **`#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` kept.** The crate
   attributes are untouched; the new rustdoc block is present; clippy `-D warnings`
   and `cargo doc` are clean.
10. **No artifact mutation.** `validate_acquired_archive` remains read-only: it
    stats, hashes, and compares. No file is created, written, or removed.

## Failure, rollback, and recovery review

`validate_acquired_archive` remains a read-only continuity gate: it mutates nothing,
stages nothing, and has no rollback path. The ordered failure table from the plan's §8
holds unchanged, with the digest-step rows now sourced from the core routine:

| Condition | Reported as | Ordered at |
|---|---|---|
| Acquired path is relative | `InvalidAcquiredFile(artifact_name)` | `lib.rs:510-512` |
| `symlink_metadata` fails (missing path) | `InvalidAcquiredFile(artifact_name)` | `lib.rs:513-514` |
| Not a regular file (symlink, directory, FIFO, device) | `InvalidAcquiredFile(artifact_name)` | `lib.rs:515-517` |
| Size ≠ manifest exact size | `InvalidAcquiredFile(artifact_name)` | `lib.rs:515-517` |
| Digest step's open failure | `InvalidAcquiredFile(artifact_name)` | `lib.rs:522-523` |
| Digest step's read failure | `InvalidAcquiredFile(artifact_name)` | `lib.rs:522-523` |
| Digest ≠ manifest digest | `InvalidAcquiredFile(artifact_name)` | `lib.rs:524-526` |

**Distinguishability is unchanged and remains a documented limitation.** All seven
conditions report the same variant and the same payload; they are not distinguishable
from the returned error value. That is exactly the baseline behavior, preserved
deliberately (§6.2). Narrowing it would change what a published consumer observes.

**Cancellation, restart, and contention** are unchanged: the function performs no
mutation, so there is nothing to roll back; there is no lock; concurrent readers are
harmless; a concurrent writer is resolved by the digest comparison at `:524-526`, not
by a lock. A writer that changes content *and* produces bytes hashing to the manifest
digest is, by construction, indistinguishable from the legitimate artifact — a
property of SHA-256 integrity evidence, not of this implementation.

## Security review

**Symlink-ordering preservation is the security-critical item, and it holds.**

- The guard at `lib.rs:513-517` still precedes any open of the acquired path.
- `hash_file` follows symlinks when it opens, so the adapter's `symlink_metadata`
  guard remains the **only** link rejection in this path. It was not moved, not
  weakened, and not merged into the hash call.
- The FIFO fixture proves the ordering behaviorally: a symlink whose target is a FIFO
  is rejected immediately, where an implementation that opened the path first would
  block forever. The fixture runs the call on a worker thread behind a 10-second
  bounded wait, so a regression fails the test rather than hanging the lane.
- The size check shares the same guard and therefore also precedes the digest;
  fixture 4 proves it with a file whose digest *does* match.
- The digest comparison at `:524-526` remains the authoritative check and fails
  closed.
- No `unsafe_code` was introduced, and no new I/O surface was opened.

**TOCTOU between the guard and the open — carried forward, low severity.** The
`symlink_metadata` (`:513`) and the open inside `hash_file` (`:522`) are separate
pathname operations and are not atomic: a path replaced by a symlink in that window is
followed and the target is hashed. M006 neither introduces nor closes this window —
the baseline had the same window between `symlink_metadata` (`:499`) and
`fs::File::open` (`:504`). Closing it requires `O_NOFOLLOW`-style handle-relative
opening, which `eggup-core` does not expose on this path today (`eggup-archive` uses
`fs_at` for its own handle-relative work; core does not). The finding is recorded in
`architecture/eggpack-adapter.md` § "The symlink guard, and the TOCTOU window it does
not close" as a **low**-severity carried-forward item. Fixing it is a separate
milestone with its own design and is **not** authorized here.

No other security property changed. SHA-256 remains integrity evidence only; the
adapter asserts no authenticity.

## Compatibility and migration review (§9)

- **`eggup-eggpack 0.1.3` is published** (2026-10-06, source `bd43683`, hosted run
  `37527706900`, checksum `952e88b8…19c5` — `plans/registry.md:83`). The published
  crate carries the direct `sha2` edge and the inline digest loop.
- **The published manifest is unchanged until the crate is published again.** The
  published `0.1.3` artifact still declares `sha2 = "0.10.9"`. This milestone
  publishes nothing.
- **The next publication's runtime dependency list shortens by one entry.** The
  packaged manifest from this tree lists exactly `eggpack-manifest`,
  `eggup-acquisition`, `eggup-archive`, `eggup-core` — one entry shorter than
  `0.1.3`'s.
- **The transitive closure is unchanged.** `sha2 v0.10.9` remains in the graph via
  `eggup-core` and `eggup-archive`; no package enters or leaves the graph; the
  lockfile is byte-identical.
- **The error surface is identical.** Same variants, same payloads, same ordering,
  same fail-closed behavior (§6.2). No `AdapterError` variant was added, removed, or
  rerouted.
- **No core republication and no pin cascade.** `eggup_core::hash_file` is exported
  by the already-pinned `eggup-core 0.1.3`; the `=0.1.3` pins remain valid and
  unchanged. The "a seam fix needs a separate adapter republication to reach it"
  cascade does not apply, because the seam fix was not needed.
- **Republication of `eggup-eggpack` is a separate milestone.** The manifest change
  is real but has no consumer-visible behavior difference and no consumer migration
  requirement. It must not be published under M006. The `v0.1.1`/`v0.1.2`/`v0.1.3`
  tags did not move.
- **Consumer migration: none.** No public signature changed, no behavior changed, no
  dependency changed for consumers.

## Documentation and operations evidence

- **Changelog.** `crates/eggup-eggpack/CHANGELOG.md` `Unreleased` gained an entry
  recording: the adapter now uses `eggup_core::hash_file`; the direct `sha2` runtime
  dependency is removed; the `sha2` dev-dependency is retained for test fixtures; the
  digest value, error variant, and error payload are unchanged; the symlink and size
  guards remain ahead of the digest; the published `0.1.3` manifest still carries the
  direct edge and the next publication would ship a list one entry shorter with an
  unchanged transitive closure; **no published release and no consumer migration**.
  The existing no-publication disclaimer (`0.1.3` is the published baseline; no
  publication milestone authorizes these entries) is preserved verbatim above the
  entry.
- **Crate README.** `crates/eggup-eggpack/README.md` now states the crate's runtime
  dependency set (the three exact-pinned Eggup crates plus `eggpack-manifest`), that
  the digest is computed by `eggup_core::hash_file`, that the crate declares no direct
  `sha2` runtime dependency, and that `sha2` remains a dev-dependency only. The
  no-migration / integrity-only framing is unchanged.
- **Normative architecture.** `architecture/eggpack-adapter.md` was updated per the
  plan's §12: the dependency table (the `sha2` row removed; the three Eggup pin rows
  corrected from `=0.1.2` to `=0.1.3`); a new paragraph stating that the adapter has
  no direct `sha2` runtime dependency and that the published manifest's list shortens
  by one entry; the dev-dependency paragraph, now recording *why* `sha2` stays; the
  "runtime dependencies only in 0.1.2" sentence, corrected (`eggup-archive` only); the
  public-surface row for `validate_acquired_archive` (it no longer hashes bytes
  itself); the "Verified here versus merely reported" bullet; the verification-ladder
  row ("streamed in 16 KiB chunks" → "streamed through `eggup_core::hash_file`"); the
  error-mapping rows (the digest step's open and read failures now have their own
  rows, mapped at `lib.rs:522-523`); a third property in § "Invariants and failure
  modes" recording the deliberate collapse; a new § "The symlink guard, and the TOCTOU
  window it does not close" section with the ordering proof and the carried-forward
  low-severity finding; the "Published-crate status" paragraph (published at `0.1.3`,
  and the M006 compatibility consequence); the "Testing approach" counts and table
  (71 tests across four integration files, including `tests/digest_reuse.rs`) and a
  new `### digest_reuse.rs` subsection; the cross-reference row for the closure
  records; and the "Known doc/code drift" section, which now records the two drifts
  this milestone did **not** fix because they belong to other files.
  Every `lib.rs:NNN` citation in the document was re-checked against the new source
  and shifted where the edit moved the line.
- **`docs/` — no change required, as the plan states.** `docs/` is non-normative.
  `docs/crates.md:94-95` already lists the adapter's dependencies as `eggup-core`,
  `eggup-archive`, `eggup-acquisition`, plus `eggpack-manifest` — that list becomes
  **accurate** once M006 removes the direct `sha2` edge. The stale `=0.1.2` pin text
  at `docs/crates.md:117` is pre-existing drift not introduced here; it is recorded
  as a **low**-severity observation for the docs-hygiene workstream (F-5) rather than
  fixed here, because `docs/` is outside this milestone's file ownership.
- **Root `CHANGELOG.md` — no entry required.** The root changelog aggregates published
  releases; M006 publishes nothing.
- **`plans/registry.md` and the subsystem roadmap — not edited here.** They are the
  parent session's files. The required updates are stated under *Roadmap disposition
  and registry updates* below.

## Unresolved findings, with severity

| ID | Finding | Severity | Reason / disposition |
|---|---|---|---|
| F-1 | The hosted Linux and Windows lanes (`stable`, `msrv`, `service-systemd`, `macos`, `windows-check`) and the full-workspace local gate (`./scripts/check-local.sh`, `cargo test --workspace`) were **not run** from this host | **medium** (qualification gap, not a defect) | No Linux/Windows host and no hosted runner is reachable from this machine, and the session's instruction scoped testing to the adapter package. The adapter-scoped suite (71/71), clippy `-D warnings`, doc, fmt, and `cargo package` are all green. The parent session owns the full gate. **This is the reason the milestone is conditionally closed.** |
| F-2 | `cargo fmt --all -- --check` fails on `crates/eggup-curl/src/lib.rs`, `crates/eggup-acquisition/tests/typed_io_contract.rs`, and `crates/eggup-eggfetch/tests/typed_io_contract.rs` | **low** (not caused by M006) | All three files are owned by other concurrent writers and were already modified in the working tree by those writers; they were clean at this session's first inspection. `cargo fmt -p eggup-eggpack -- --check` is silent. Not fixed here — outside this milestone's ownership. |
| F-3 | The TOCTOU window between the `symlink_metadata` guard and the open inside `hash_file` | **low** (carried forward, unchanged from the baseline) | Pre-existing and unchanged by M006. Closing it needs `O_NOFOLLOW`-style handle-relative opening, which `eggup-core` does not expose on this path. A separate milestone with its own design. Recorded in `architecture/eggpack-adapter.md`. |
| F-4 | The digest step's open and read failures are not distinguishable from a digest mismatch in the returned error | **low** (deliberate, documented) | Preserved deliberately per §6.2 to keep M006 a zero-observable-change refactor on a published crate. `AdapterError` carries only the artifact name and has no `source()` chain, so the underlying `io::Error` could not be surfaced through the adapter's error type even by adding a variant. Narrowing is a candidate for the intentional 0.2/1.0 API boundary and must not ride along with a republication. |
| F-5 | `docs/crates.md:117` still carries `=0.1.2` for the Eggup pins, and `architecture/overview.md`'s `849 / 63` test figure is now `855 / 71` | **low** (pre-existing / adjacent drift) | `docs/` is non-normative and outside this milestone's ownership; `architecture/overview.md` is owned by another writer. Both are recorded in `architecture/eggpack-adapter.md` § "Known doc/code drift" for the owning workstreams. |
| F-6 | The `sha2` dev-dependency remains | **none** (a recorded observation, not a defect) | The plan's §6.5 keeps it: the integration fixtures compute digests independently of the crate under test, which is deliberate test design rather than the runtime redundancy that was removed. A dev-dependency creates no runtime supply-chain edge. Removing it would require the fixtures to obtain digests another way — a separate decision with its own evidence, not authorized here. |

No **medium-or-higher** finding remains open **in M006's own scope**. F-1 is a
qualification gap that the parent session closes with the full workspace gate and the
hosted lanes.

## Roadmap disposition and registry updates

**M006 does not edit `plans/registry.md` or the subsystem roadmap** — those are the
closing parent session's files. The updates that the parent session should apply are:

1. `plans/registry.md` § "Post-0.1.3 open findings" — **entry 9** ("`eggup-eggpack`
   re-implements core's file-digest routine and carries a direct `sha2` dependency
   for that one call site", assessed **medium**) is **closed** by this milestone, with
   the closure record
   `plans/closure/eggpack-manifest-interoperability/006-status.md` as its evidence.
   The finding's honest framing must be preserved: it was a redundancy and
   maintenance/supply-chain cost, **not** a correctness or security defect, and no
   failing production report ever existed.
2. `plans/registry.md` § "Post-0.1.3 open findings" — **entry 2** (the deferred
   adapter error-surface narrowing) is **unchanged** and remains open: M006
   deliberately kept the collapse (§6.2). It remains a candidate for the intentional
   0.2/1.0 API boundary and must not ride along with a republication.
3. `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` — the status table
   gains a row for **M006** ("canonical digest reuse corrective"), status
   **closed** (or **conditionally closed** if the parent session prefers that wording
   until the hosted lanes are green), with a link to
   `plans/closure/eggpack-manifest-interoperability/006-status.md`.
4. No root `CHANGELOG.md` entry — M006 publishes nothing.

## Handoff notes for the next milestone

- **What a separate republication milestone would require.** M006 leaves
  `eggup-eggpack 0.1.3` as the published baseline; the shorter dependency list and
  the delegated digest routine reach registry consumers only when the adapter is
  published again. That milestone would need: a version bump (the workspace version
  is shared at `Cargo.toml:15`), a per-crate and root changelog cut, an exact-pin
  review for `eggup-core`/`eggup-archive`/`eggup-acquisition`/`eggpack-manifest`, a
  `cargo package` + `cargo publish --dry-run` pass, full hosted qualification, and a
  GitHub Release note append **without moving the `v0.1.3` tag**. It must also decide
  whether the deferred error-surface narrowing rides along — which it must not,
  because that is a separate behavior change on a published crate.
- **Why no pin cascade was needed.** `eggup_core::hash_file` is exported by the
  already-pinned `eggup-core 0.1.3`. The `=0.1.3` pins remain valid, so no core
  republication was required and no adapter republication is forced to carry a seam
  fix.
- **What this milestone did NOT authorize, and did not do.** No publication of any
  crate and no movement of `v0.1.1`/`v0.1.2`/`v0.1.3`; no change to `eggup-core`,
  `eggup-archive`, `eggup-acquisition`, or `eggpack-manifest`; no narrowing of the
  adapter's error surface and no new `AdapterError` variant; no change to the `sha2`
  dev-dependency; no authenticity, signature, provenance, or trust work; no policy,
  destination, ownership, permission, release-selection, or service change in the
  adapter; no change to the extraction handoff; no fix for the TOCTOU observation
  (F-3); and no edits to `plans/registry.md` or the subsystem roadmap.
