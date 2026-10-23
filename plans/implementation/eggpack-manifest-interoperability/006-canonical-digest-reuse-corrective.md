# Eggpack Manifest Interoperability Milestone 006 — canonical digest reuse corrective

Status: conditionally closed 2026-10-23; implemented and green on the native macOS lane. Closure record: `plans/closure/eggpack-manifest-interoperability/006-status.md`. The remaining condition is the outstanding hosted qualification: Linux, Windows, the hosted CI lanes, and the full-workspace local gate were **not run**.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Original closed work affected:

- `plans/implementation/eggpack-manifest-interoperability/001-release-manifest-v1-adapter.md`
- `plans/closure/eggpack-manifest-interoperability/001-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **9** (registered at baseline `c480541`
  alongside this plan): "`eggup-eggpack` re-implements core's file-digest routine and carries a
  direct `sha2` dependency for that one call site." The finding is assessed **medium** severity
  and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Primary class: polish

## 1. Objective

Leave exactly one file-digest implementation in the workspace.

`eggup-eggpack` hashes the acquired archive with an inline SHA-256 loop and therefore carries a direct `sha2` runtime dependency whose only consumer is that loop — while already depending on `eggup-core`, which exports the identical capability as `eggup_core::hash_file`.

Required outcome: the adapter delegates the digest to `eggup_core::hash_file`; `sha2 = "0.10.9"` is removed from `crates/eggup-eggpack/Cargo.toml` `[dependencies]`; the symlink rejection and exact-size check remain and remain ordered **before** the digest computation; the digest value, the error variant, and the error payload are unchanged.

No behavior change is intended. This is de-duplication and dependency-surface work, not a correctness milestone.

## 2. Why this milestone is ready

The defect is fully characterized by reading the source; it needs no reproduction and no failure report. The duplicate is one function in one file (`crates/eggup-eggpack/src/lib.rs:499-521`); the reusable implementation is public and already published (`crates/eggup-core/src/integrity.rs:64-80`, re-exported at `crates/eggup-core/src/lib.rs:38-40`); the dependency to remove is one manifest line (`crates/eggup-eggpack/Cargo.toml:24`). The subsystem has no open milestone — the roadmap status table (`plans/subsystems/eggpack-manifest-interoperability-roadmap.md:301-310`) ends at M005, closed 2026-10-06. Scope is bounded to one call site in one crate plus one manifest line.

Severity: **medium**.

- Not a correctness defect: both implementations are correct and produce the same SHA-256 for the same bytes; chunk size does not affect the digest.
- Not a security defect: the symlink rejection and exact-size check are unaffected by the digest routine, and the digest comparison still fails closed.
- It is a supply-chain and maintenance cost on a *published* crate: a redundant direct dependency edge, a second version-resolution obligation in the adapter's own manifest, and a duplicated implementation that can drift from core's (a future change to `hash_file`'s chunking or read-error reporting would not automatically apply at the adapter's call site).
- Honest limitation: **no failing production report exists.** Current behavior is CORRECT. The issue is redundancy, not a bug.

This milestone alone does **not** satisfy the repository's "medium-or-higher" blocking gate on its own — a closed milestone needs a corrective for every open finding at that level, and this one does not retire the other findings recorded in `plans/registry.md` § "Post-0.1.3 open findings". It is taken because it is cheap, because it removes a dependency edge from an already-published crate, and because it is implementable against the already-pinned `eggup-core = "=0.1.3"` with no republication cascade.

## 3. Current implementation evidence

**Adapter, `crates/eggup-eggpack/src/lib.rs`** (`validate_acquired_archive`):

- `:485` — `use sha2::{Digest, Sha256};` — the **only** `sha2` import in non-test code in the crate.
- `:486` — function-scoped `use std::io::Read;`.
- `:499-500` — `fs::symlink_metadata(...)`; failure → `InvalidAcquiredFile`.
- `:501-503` — `if !meta.file_type().is_file() || meta.len() != exact_size` → `InvalidAcquiredFile`. This is the symlink rejection and the exact-size check.
- `:504-505` — `fs::File::open(...)`; failure → `InvalidAcquiredFile`.
- `:506-517` — inline streaming SHA-256 loop, 16 KiB chunk (`:507`).
- `:518-519` — `if digest != sha256 { return Err(AdapterError::InvalidAcquiredFile(...)) }`.

**Core, `crates/eggup-core/src/integrity.rs`**:

- `:64-65` — `pub fn hash_file(path: &Path) -> Result<[u8; 32]>`, documented as streaming without buffering the complete artifact.
- `:66-67` — `File::open(path)` → `Error::io("opening file for hashing", source)`.
- `:69` — 64 KiB buffer; `:70-78` — same loop shape, read failure → `Error::io("reading file for hashing", source)`; `:79` — `Ok(...)`.

**Side by side.**

| Property | Adapter (`:499-521`) | Core (`integrity.rs:64-80`) |
|---|---|---|
| Algorithm / streaming / non-buffering | SHA-256 / yes / yes | SHA-256 / yes / yes |
| Chunk size | 16 KiB (`:507`) | 64 KiB (`:69`) |
| Opens the path and follows symlinks | yes (`:504`) | yes (`:67`) |
| Digest result for the same bytes | identical | identical |

The only differences are the chunk size and the error type. SHA-256 over a byte stream is chunk-independent, so the digest value is unaffected.

**The dependency edge, `crates/eggup-eggpack/Cargo.toml`:**

- `:20-22` — `eggup-core` / `eggup-archive` / `eggup-acquisition`, each `=0.1.3` with a local path. `:23` — `eggpack-manifest = "=0.1.0"`.
- `:24` — `sha2 = "0.10.9"` ← **the redundant direct runtime edge**.
- `:32` — `sha2 = "0.10"` in `[dev-dependencies]` ← **separate**, used by the integration tests at `crates/eggup-eggpack/tests/archive_handoff.rs:72`.

Single call site: `validate_acquired_archive` is the only place the adapter hashes bytes itself (`architecture/eggpack-adapter.md:146`).

**Removable without a new core version.** `eggup_core::hash_file` is exported by the **published** `eggup-core 0.1.3`: crates.io registry source `eggup-core-0.1.3/src/lib.rs:38-40` re-exports `hash_file`, and `eggup-core-0.1.3/src/integrity.rs:64-80` is byte-identical to the local file. `hash_file` also existed in the published `eggup-core 0.1.2` (`plans/closure/verified-update-core/008-package-fixture/vendor/eggup-core-0.1.2/src/integrity.rs:65`), so the pinned `=0.1.3` already carries it. **No new core version is required and no pin cascade follows.**

**Baseline `cargo tree -p eggup-eggpack --locked -i sha2`:**

```text
sha2 v0.10.9
├── eggup-archive v0.1.3
│   └── eggup-eggpack v0.1.3
├── eggup-core v0.1.3
│   └── eggup-eggpack v0.1.3
└── eggup-eggpack v0.1.3
[dev-dependencies]
└── eggup-eggpack v0.1.3
```

Two honest consequences. The resolved package set **still contains `sha2 v0.10.9`** after M006, because `eggup-core` (`crates/eggup-core/Cargo.toml`: `sha2 = "0.10"`) and `eggup-archive` (`crates/eggup-archive/Cargo.toml`: `sha2 = "0.10.9"`) depend on it directly — M006 removes the adapter's **direct** edge, not the package from the graph. And the benefit is therefore smaller than "removing a dependency" suggests: what is removed is one direct edge to audit in the published manifest, one direct version-resolution obligation, and one duplicated implementation that can drift.

## 4. Invariants that must not regress

1. **Digest value byte-identical.** The adapter's SHA-256 for a given file equals `eggup_core::hash_file`'s output and equals the manifest digest.
2. **Symlink rejection remains and remains ordered before the digest.** A symlink is rejected at `crates/eggup-eggpack/src/lib.rs:501-503`, before any open or read. `hash_file` follows symlinks when it opens, so this guard is the only link rejection in the path; it must not be moved, weakened, or merged into the hash call.
3. **Exact-size check remains and remains ordered before the digest**, at `:501-503`.
4. **Digest mismatch fails closed.** Mismatch returns `AdapterError::InvalidAcquiredFile(artifact_name)`; never downgraded, warned, or retried.
5. **SHA-256 remains integrity evidence only.** No authenticity, signature, provenance, or trust claim is introduced; the adapter stays on the integrity rung (`architecture/eggpack-adapter.md:659-663`).
6. **The adapter's policy surface does not widen.** No URL/origin, install-root, ownership, permission, release-selection, service, or authenticity policy enters the adapter.
7. **The exact pins are unchanged**: `eggup-core`, `eggup-archive`, `eggup-acquisition` stay at `=0.1.3`; `eggpack-manifest` stays at `=0.1.0`.
8. **`eggup-eggpack` remains the only crate that touches producer types.** No new Eggpack edge may enter `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-archive`, `eggup-service`, or `eggup-transport-footprint`.
9. **The crate keeps `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`** (`crates/eggup-eggpack/src/lib.rs:1-2`); the workspace keeps `unsafe_code = "deny"` and clippy `all = warn`.
10. **No artifact mutation.** `validate_acquired_archive` stays read-only.

## 5. Scope

### In scope

- Replace the inline digest loop in `validate_acquired_archive` with `eggup_core::hash_file`.
- Apply and document the required error mapping at that call site.
- Preserve (and prove) the symlink and exact-size checks and their ordering before the digest computation.
- Remove `sha2 = "0.10.9"` from `crates/eggup-eggpack/Cargo.toml` `[dependencies]`.
- Record the dev-dependency decision explicitly (it stays — §6.5).
- Add the regression fixtures in §10; update `crates/eggup-eggpack/CHANGELOG.md` (`Unreleased`) and `architecture/eggpack-adapter.md`.
- Full local gate plus hosted-lane qualification.

### Explicitly out of scope

- **No change to the `=0.1.3` pins** on `eggup-core`, `eggup-archive`, or `eggup-acquisition`. No new core version.
- **No authenticity or signature work.** No trust establishment, no producer provenance, no signature verification.
- **No policy change.** URL/origin construction, install-root selection, ownership, permissions, release discovery, mirror fallback, version ordering, service policy, and elevation remain caller-owned and out of the adapter.
- **No republication.** M006 publishes nothing and does not move the `v0.1.3` tag.
- **No change to `eggpack-manifest`.** The `=0.1.0` registry edge owned by the Eggpack producer repository is untouched.
- **No change to the extraction handoff.** `archive_plan_for`, `bind_archive_members`, `BoundSources`, and `eggup-archive` extraction semantics are untouched.
- **No refactor of the rest of `eggup-eggpack`.** Projection, request binding, destination binding, and materialization are untouched.
- **No narrowing of the adapter's error surface** (§6.2, §9).
- **No change to `eggup-core`.** `hash_file` is consumed as published.
- **No new rule in `docs/`.** `docs/` is non-normative; it points at the contracts and never introduces a rule.

## 6. Required production changes

### 6.1 Use the core-provided digest routine

`crates/eggup-eggpack/src/lib.rs:499-521` calls `eggup_core::hash_file(acquired_archive_path)` in place of the inline loop. The `use sha2::{Digest, Sha256};` at `:485` (and the function-scoped `use std::io::Read;` at `:486`, if now unused) are removed. The digest comparison at `:518-519` stays in the adapter — the adapter compares against the manifest digest; core does not perform that comparison for the adapter.

### 6.2 Required error mapping — the collapse is preserved

`eggup_core::hash_file` returns `eggup_core::Result<[u8; 32]>` and reports failures as `Error::Io { operation, source }` with `operation` = `"opening file for hashing"` or `"reading file for hashing"` (`crates/eggup-core/src/error.rs:16-22`, `:133`).

**Required mapping.** Every digest-step failure maps to `AdapterError::InvalidAcquiredFile(artifact_name.clone())`: open failure (`Error::Io { operation: "opening file for hashing", .. }`), read failure (`Error::Io { operation: "reading file for hashing", .. }`), and digest mismatch (no core error). Rationale the implementer must record:

- The baseline collapses open, read, and digest-mismatch into one variant. Preserving it makes M006 a **zero-observable-change** refactor: identical variant, payload, and ordering.
- `InvalidAcquiredFile` is already documented to cover a missing path (`architecture/eggpack-adapter.md:108`; error table `:626-630`), so an open failure is within its documented contract.
- The `io::Error` source **cannot** be surfaced through the adapter's error type today: `AdapterError::InvalidAcquiredFile(String)` carries only an artifact name, `AdapterError` implements `std::error::Error` with no `source()` chain (`crates/eggup-eggpack/src/lib.rs:58`), and core's `Error::io` constructor is `pub(crate)` (`crates/eggup-core/src/error.rs:133`). Narrowing to `AdapterError::Eggup(String)` would forward only the `Display` text and would still drop the `io::Error` source — narrowing buys a different label, not better diagnostics.
- Narrowing is a separate, later decision (§9, §14) and must not be smuggled in here.

The implementer must verify the mapping is applied to the core error and not bypassed with `?` into a different variant.

### 6.3 Ordering preserved

The `symlink_metadata` guard at `:499-503` must remain textually and logically **before** the `hash_file` call. `hash_file` follows symlinks when it opens, so the guard is the only link rejection in this path: a symlink must be rejected without its target ever being opened or read.

### 6.4 Buffer size

The two implementations used 16 KiB and 64 KiB chunks. SHA-256 over a byte stream is chunk-independent, so the difference has **no effect on the digest value**; the retained chunk size is now core's 64 KiB (`crates/eggup-core/src/integrity.rs:69`). The adapter must not reintroduce a chunk-size constant, and its rustdoc must not claim a chunk size it no longer controls. `architecture/eggpack-adapter.md:667` ("streamed in 16 KiB chunks") must be corrected.

### 6.5 Manifest changes

- **Remove** `sha2 = "0.10.9"` from `crates/eggup-eggpack/Cargo.toml` `[dependencies]` (`:24`).
- **Keep** `sha2 = "0.10"` in `[dev-dependencies]` (`:32`). It is used by `crates/eggup-eggpack/tests/archive_handoff.rs:72` to compute fixture digests independently of the crate under test — that independence is deliberate test design, not the same redundancy. It is a dev-dependency, so it creates no runtime supply-chain edge. Removing it would require the fixtures to obtain digests another way; that is a **separate decision with its own evidence**, not authorized here. Record it as an observation.
- **Unchanged**: `eggup-core = "=0.1.3"`, `eggup-archive = "=0.1.3"`, `eggup-acquisition = "=0.1.3"`, `eggpack-manifest = "=0.1.0"`.
- `Cargo.lock` may need regeneration; the closure must record the lock diff and confirm the resolved `sha2` version is unchanged (`0.10.9`).

## 7. Ordered work packages

Each package is independently verifiable and leaves the tree compiling and testable.

- **WP1 — Replace the inline digest with the core routine.** Rewrite `crates/eggup-eggpack/src/lib.rs:499-521` to call `eggup_core::hash_file`; remove the `sha2` import at `:485` (and `:486` if unused); keep the comparison at `:518-519`. *Verify:* `cargo test -p eggup-eggpack --locked` green; the diff shows no change to the guard at `:499-503`.
- **WP2 — Apply and document the required error mapping.** Map `hash_file`'s `Err` to `AdapterError::InvalidAcquiredFile(artifact_name.clone())` per §6.2, without `?`-short-circuiting into another variant; record the mapping in the function's rustdoc. *Verify:* the open-failure fixture (§10.6) asserts the exact variant.
- **WP3 — Verify the symlink/size ordering.** Confirm the guard at `:501-503` precedes the `hash_file` call in the final source; add the ordering fixtures (§10.2–§10.4). *Verify:* the FIFO-ordering fixture completes rather than blocking; the symlink and size-mismatch fixtures reject with `InvalidAcquiredFile`.
- **WP4 — Remove the direct `sha2` runtime dependency; confirm the dev-dependency decision.** Delete `crates/eggup-eggpack/Cargo.toml:24`; keep `:32` with the recorded reason; regenerate the lockfile if needed. *Verify:* `cargo tree -p eggup-eggpack --locked -i sha2` no longer lists `eggup-eggpack` as a direct runtime dependent of `sha2`; the crate builds with no `sha2` import.
- **WP5 — Dependency-tree verification.** Capture `cargo tree -p eggup-eggpack --locked` and `cargo tree -p eggup-eggpack --locked -i sha2` before and after and diff them. *Verify:* the only change is the removed direct `sha2` edge; the `eggup-core`/`eggup-archive`/`eggup-acquisition`/`eggpack-manifest` edges are byte-identical; no package enters or leaves the graph.
- **WP6 — Tests, changelog, and docs.** Add the §10 fixtures (including the red-first manifest assertion); update `crates/eggup-eggpack/CHANGELOG.md` `Unreleased` and `architecture/eggpack-adapter.md` per §12. *Verify:* §11 commands green locally; hosted lanes recorded per lane.

## 8. Failure, cancellation, restart, and contention semantics

`validate_acquired_archive` is a read-only continuity gate: it mutates nothing, stages nothing, and has no rollback path.

| Condition | Reported as | Ordered at |
|---|---|---|
| Acquired path is relative | `InvalidAcquiredFile(artifact_name)` | `:496-498` |
| `symlink_metadata` fails (missing path) | `InvalidAcquiredFile(artifact_name)` | `:499-500` |
| Not a regular file (symlink, directory, FIFO, device) | `InvalidAcquiredFile(artifact_name)` | `:501-503` |
| Size ≠ manifest exact size | `InvalidAcquiredFile(artifact_name)` | `:501-503` |
| Open failure (`Error::Io { operation: "opening file for hashing", .. }`) | `InvalidAcquiredFile(artifact_name)` | after the guard |
| Read failure (`Error::Io { operation: "reading file for hashing", .. }`) | `InvalidAcquiredFile(artifact_name)` | after the guard |
| Digest ≠ manifest digest | `InvalidAcquiredFile(artifact_name)` | last step |

**Distinguishability:** under the required mapping (§6.2) all six conditions report the **same** variant and the **same** payload. They are not distinguishable from the returned error value. This is unchanged from the baseline and is recorded as a deliberate, documented limitation, not an oversight.

- **Digest mismatch fails closed.** It returns an error. It is never downgraded to a warning, never retried against an alternate source, never passed on as "unverified", and never turned into a success with a degraded flag.
- **A file that changes between the size check and the hash read is still caught.** The size check is a fast pre-filter, not a trust decision; the authoritative comparison is the digest at `:518-519`. A concurrent writer that changes content after `:501-503` is caught by the digest comparison and fails closed. A writer that changes content *and* produces bytes hashing to the manifest digest is, by construction, indistinguishable from the legitimate artifact — a property of SHA-256 integrity evidence, not of this implementation.
- **TOCTOU between the guard and the open is pre-existing and unchanged.** `symlink_metadata` (`:499`) and the open (`:504`, or inside `hash_file`) are not atomic: a path replaced by a symlink in that window is followed and the target would be hashed. M006 neither introduces nor closes this window. Closing it requires `O_NOFOLLOW`-style handle-relative opening — a capability `eggup-core` does not expose on this path today (`eggup-archive` uses `fs_at` for its own handle-relative work; core does not). Record as a **low**-severity observation carried forward; fixing it is a different milestone and is **not** authorized here.
- **Cancellation and restart:** none. The function performs no mutation, so there is nothing to roll back and no partial state to reconcile. A caller that aborts between the guard and the hash simply retries; the digest comparison remains authoritative.
- **Contention:** there is no lock. Concurrent readers are harmless; concurrent writers are resolved by the digest comparison, not by a lock.

## 9. Compatibility and migration

- **`eggup-eggpack 0.1.3` is published** (2026-10-06, source `bd43683`, hosted run `37527706900`, checksum `952e88b8…19c5` — `plans/registry.md:83`). The published crate carries the direct `sha2` edge and the inline digest loop.
- **Removing the direct dependency changes the published manifest.** The evidence is the `cargo tree -p eggup-eggpack --locked` diff: the adapter's direct `sha2` edge disappears. The *resolved package set* does not change — `sha2 v0.10.9` remains via `eggup-core` and `eggup-archive` (§3). The published manifest's runtime dependency list shortens by one entry; the transitive closure is unchanged.
- **Error surface.** With the required mapping (§6.2) the published and post-M006 error surfaces are **identical**: same variants, payloads, ordering, and fail-closed behavior. If the mapping were instead narrowed — e.g. routing open/read failures to `AdapterError::Eggup(String)` so that "the acquired file is invalid" is distinguishable from "the file could not be read" — the consequence would be observable to any consumer matching `AdapterError::InvalidAcquiredFile(_)` for an unreadable acquired path. `AdapterError` is `#[non_exhaustive]` (`crates/eggup-eggpack/src/lib.rs:20`), so *adding* a variant would not break downstream matches, but *changing which variant is produced* is a behavior change on a published crate and must be carried by a republication milestone. Narrowing is **out of scope for M006** and is a candidate for the intentional 0.2/1.0 API boundary alongside the deferred error-surface work already recorded in `plans/registry.md` § "Post-0.1.3 open findings" (entry 2).
- **No core republication and no pin cascade.** `eggup_core::hash_file` is exported by the **already-pinned** `eggup-core 0.1.3` (`crates/eggup-eggpack/Cargo.toml:20`); the published registry source for that version carries it (§3). The `=0.1.3` pins therefore remain valid and unchanged, and the usual "a seam fix needs a separate adapter republication to reach it" cascade does **not** apply.
- **Republication of `eggup-eggpack` itself is a separate milestone.** The manifest change is real but has no consumer-visible behavior difference and no consumer migration requirement. It must not be published under M006.
- **The `v0.1.3` tag must not move.** `v0.1.1`, `v0.1.2`, and `v0.1.3` are immutable publication markers.
- **Consumer migration: none.** No public signature changes, no behavior changes, no dependency changes for consumers.

## 10. Required tests

New fixtures belong in `crates/eggup-eggpack/tests/`. Each states what it proves and whether it can be red-first.

1. **Known artifact, digest unchanged.** A fixture file with fixed content whose SHA-256 is a known constant (e.g. the digest of `b"abc"`). Assert `validate_acquired_archive` returns `Ok(())` when the projection declares the exact size and digest, and that the digest compared equals `eggup_core::hash_file`'s output for the same file. Proves: the digest value is unchanged by the routine swap. *Pass-to-pass.*
2. **Symlinked archive rejected before hashing.** The symlink target has the exact expected size and digest, so the *only* reason for rejection is that the acquired path is a symlink. Assert `Err(AdapterError::InvalidAcquiredFile(_))`. Proves: the symlink rejection is effective and the link target's matching digest is not accepted. *Unix-only* (`#[cfg(unix)]`): Windows symlink creation needs privileges CI does not grant, so that lane records this fixture as **not run**.
3. **Ordering proof (Unix-only).** The symlink target is a FIFO. If the implementation opened the path before rejecting the symlink, the open would block. Run the call on a worker thread and require completion within a bounded wait, so a blocked call fails the test rather than hanging the lane. Proves: the guard precedes any open. *Unix-only.*
4. **Size mismatch rejected before hashing.** A regular file whose content digest **matches** the manifest digest, with the projection declaring `exact_size` one byte larger than the real size. The digest would match, so the *only* reason for rejection is the size check. Assert `Err(AdapterError::InvalidAcquiredFile(_))`. Proves: the size check is effective and precedes the digest comparison. Cross-platform.
5. **Digest mismatch still fails closed.** A regular file with the exact declared size but a digest the manifest does not declare. Assert `Err(AdapterError::InvalidAcquiredFile(_))`. Cross-platform.
6. **Error mapping for open failure (Unix-only).** A regular file with mode `000`: `symlink_metadata` succeeds and the size check passes, but `hash_file`'s `File::open` fails with `EACCES`. Assert `Err(AdapterError::InvalidAcquiredFile(_))` — the core error is mapped to the adapter's variant, not leaked as `AdapterError::Eggup` or a panic. On Windows a mode-`000` file is still readable, so this fixture is Unix-only; that lane records it as **not run**.
7. **Read-failure mapping.** No deterministic read-failure fixture exists today (a truncated file yields EOF, not an error; a directory is rejected earlier by the type check). Prove the mapping through fixture 6, which exercises the same `Err(core_error) → InvalidAcquiredFile` branch, and record the read path's evidence as source inspection of `crates/eggup-core/src/integrity.rs:70-78`. State this limitation in the closure; do not claim a read-failure fixture that does not exist.
8. **Red-first: no direct `sha2` runtime dependency.** An integration test reads `env!("CARGO_MANIFEST_DIR")/Cargo.toml`, parses `[dependencies]`, and asserts no `sha2` key is present. **This fixture FAILS at the pre-M006 baseline** (`crates/eggup-eggpack/Cargo.toml:24` is present) and passes after M006. Cross-platform.

**Honest caveat — which fixtures can be red-first.** Fixtures 1–7 are pass-to-pass: they assert behavior that is correct both before and after M006, so they will **pass** against the pre-M006 implementation. They are regression guards against the refactor moving or weakening the ordering and the fail-closed path — not red-first proofs. The **only** red-first fixture is number 8. The primary discriminating evidence for M006 is therefore (a) the fixture-8 red-first result and (b) the `cargo tree` before/after diff showing the direct edge removed and nothing else changed. M006 does not produce a red test proving the *behavior* changed, because the behavior must not change.

Existing fixtures that must remain green and unmodified in expectation: `crates/eggup-eggpack/tests/interoperability.rs`, `crates/eggup-eggpack/tests/archive_handoff.rs`, `crates/eggup-eggpack/tests/caller_destinations.rs`.

## 11. Required verification commands

Run and record, verbatim:

```text
cargo fmt --all -- --check
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test -p eggup-eggpack --locked interoperability
cargo test -p eggup-eggpack --locked --test digest_reuse
cargo clippy -p eggup-eggpack --all-targets --locked -- -D warnings
cargo doc -p eggup-eggpack --no-deps --locked
cargo tree -p eggup-eggpack --locked
cargo tree -p eggup-eggpack --locked -i sha2
cargo package -p eggup-eggpack --locked --allow-dirty
cargo test --workspace --all-targets --all-features --locked
./scripts/check-local.sh
```

Dependency-edge before/after diff, at the baseline and at the implementation SHA:

```text
git diff -- crates/eggup-eggpack/Cargo.toml Cargo.lock
cargo tree -p eggup-eggpack --locked > /tmp/m006-tree-after.txt
cargo tree -p eggup-eggpack --locked -i sha2 > /tmp/m006-sha2-after.txt
```

Red-first proof against the pre-M006 baseline:

```text
git worktree add ../eggup-m006-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
cp crates/eggup-eggpack/tests/digest_reuse.rs ../eggup-m006-baseline/crates/eggup-eggpack/tests/digest_reuse.rs
cd ../eggup-m006-baseline && cargo test -p eggup-eggpack --locked --test digest_reuse
```

Expected result at the baseline: the fixture-8 manifest assertion **FAILS** (the direct `sha2` dependency is present at `Cargo.toml:24`); fixtures 1–7 **PASS** (they assert preserved behavior). The implementer must record the actual output, not this expectation.

Hosted lanes, recorded per lane and never inferred across platforms: `stable`, `msrv`, `service-systemd`, `macos`, `windows-check` (lane names per `.github/workflows/ci.yml:11`, `:25`, `:34`, `:53`, `:64`). A lane that was not run is recorded as **not run**. Unix-only fixtures (§10.2, §10.3, §10.6) are **not run** on the `windows-check` lane.

## 12. Documentation updates

- `crates/eggup-eggpack/CHANGELOG.md` — add an entry under `Unreleased`: the adapter now uses `eggup_core::hash_file` for the archive digest; the direct `sha2` runtime dependency is removed; the `sha2` dev-dependency remains for test fixtures; **no published release and no consumer migration**. Preserve the existing no-publication disclaimer in that section (`crates/eggup-eggpack/CHANGELOG.md:35-38`).
- `architecture/eggpack-adapter.md` — the normative adapter contract: dependency table `:73-77` (remove the `sha2` row; correct the three Eggup pin rows, which still read `=0.1.2` while the manifest is `=0.1.3` — `crates/eggup-eggpack/Cargo.toml:20-22`); dev-dependency paragraph `:85`; the "runtime dependencies only in 0.1.2" sentence `:90`; the public-surface row for `validate_acquired_archive` `:146` ("the only place the adapter hashes bytes itself" is no longer true); the verification-ladder row `:667` ("streamed in 16 KiB chunks" is no longer the adapter's behavior); the error-mapping rows `:626-630` (record that open/read/mismatch remain collapsed into `InvalidAcquiredFile` and that the routine is now core-provided); and the §8 TOCTOU observation between the guard and the open.
- `crates/eggup-eggpack/README.md` — only if it states the crate's dependency set; otherwise no change.
- **Not required:** `docs/` is non-normative and must not restate the rule. `docs/crates.md:94-95` already lists the adapter's dependencies as `eggup-core`, `eggup-archive`, `eggup-acquisition`, plus `eggpack-manifest` — that list becomes **accurate** once M006 removes the direct `sha2` edge, so no `docs/` change is required. The stale `=0.1.2` pin text at `docs/crates.md:117` is pre-existing drift not introduced by M006; record it as a **low**-severity observation for the docs-hygiene workstream rather than fixing it here.
- The root `CHANGELOG.md` aggregates published releases; M006 publishes nothing, so no root changelog entry is required unless a later republication milestone adds one.

## 13. Acceptance criteria

M006 closes only when all of the following are checkable:

1. `crates/eggup-eggpack/src/lib.rs` contains no `sha2` import in non-test code, and the digest is computed by `eggup_core::hash_file`.
2. `crates/eggup-eggpack/Cargo.toml` `[dependencies]` contains no `sha2` entry; the `sha2` dev-dependency remains, with the reason recorded.
3. `cargo tree -p eggup-eggpack --locked -i sha2` no longer lists `eggup-eggpack` as a **direct** runtime dependent of `sha2`; the resolved `sha2` version is still `0.10.9`; no package enters or leaves the graph.
4. The `=0.1.3` pins on `eggup-core`, `eggup-archive`, and `eggup-acquisition`, and the `=0.1.0` pin on `eggpack-manifest`, are byte-identical to the baseline.
5. The `symlink_metadata` guard (`crates/eggup-eggpack/src/lib.rs:499-503`) still precedes the digest computation, and the FIFO-ordering fixture proves it.
6. A symlinked acquired archive, a size-mismatched archive, and a digest-mismatched archive each fail closed with `AdapterError::InvalidAcquiredFile`.
7. The open-failure fixture asserts the core error is mapped to `AdapterError::InvalidAcquiredFile`, not leaked as another variant.
8. The digest for a known artifact equals the known constant and equals `eggup_core::hash_file`'s output.
9. The red-first manifest fixture fails at the baseline and passes after M006.
10. `AdapterError`'s variants, payloads, and behavior are unchanged.
11. No authenticity, signature, provenance, or trust claim is introduced; SHA-256 remains integrity evidence only.
12. The adapter's policy surface is unchanged.
13. `eggup-eggpack` remains the only crate with an `eggpack-manifest` edge; `cargo tree -p eggup-core`, `-p eggup-acquisition`, and `-p eggup-service` show no Eggpack edge.
14. The crate still compiles under `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, workspace `unsafe_code = "deny"`, and clippy `all = warn` with `-D warnings`.
15. `./scripts/check-local.sh` is green on the local machine.
16. Hosted lanes `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` are recorded individually; any lane not run is recorded as **not run**.
17. Nothing was published, and the `v0.1.3` tag was not moved.

## 14. Stop conditions

Stop and escalate rather than improvising if any of these hold:

- `eggup_core::hash_file` turns out **not** to be exported at the pinned `=0.1.3` version — i.e. the registry source for `eggup-core 0.1.3` lacks `pub use integrity::hash_file` (contradicting `crates/eggup-core/src/lib.rs:38-40` and §3). The milestone would then require a core republication and the pin cascade that follows, which is a different milestone.
- The error-mapping change turns out to be a **breaking public change** that cannot be made source-compatible at the `0.1.3` surface — e.g. narrowing `InvalidAcquiredFile` behavior for unreadable paths would alter what a published consumer observes. Narrowing must then be sequenced at the intentional 0.2/1.0 API boundary, and M006 must keep the collapse.
- Removing the dependency would change the published manifest in a way the release process cannot carry — e.g. if `Cargo.lock` cannot be regenerated without changing the resolved `sha2` version or another published crate's graph.
- The `hash_file` call cannot be placed after the `symlink_metadata` guard without weakening the symlink rejection. The guard wins; the milestone is abandoned rather than the guard weakened.
- `eggup_core::hash_file` is found to differ from the adapter's inline loop in digest result for any input. The inline loop is then not a duplicate and the finding must be re-characterized.
- Any evidence that the change requires touching `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggpack-manifest`, or the extraction handoff. M006 owns one crate's manifest and one call site.
- The local gate or a hosted lane fails for a reason M006 did not cause. Record the failure honestly; do not close on a partial run.

## 15. Closure evidence required

The closure record is `plans/closure/eggpack-manifest-interoperability/006-status.md` and must satisfy `plans/closure/README.md`.

- **Requirement-to-evidence matrix** covering every acceptance criterion in §13, each mapped to a command result or a source citation.
- **Exact commands with results**, from §11, verbatim, with pass/fail and counts. No paraphrase.
- **The `cargo tree` before/after diff** as the key evidence: both `cargo tree -p eggup-eggpack --locked` and `cargo tree -p eggup-eggpack --locked -i sha2`, before and after, plus the `git diff` of `crates/eggup-eggpack/Cargo.toml` and `Cargo.lock`.
- **The red-first baseline result**: the §11 `git worktree` run at `c4805411f74b85158e0ae5d0a219a32c4209d98e`, with actual output showing fixture 8 failing and fixtures 1–7 passing, and the honest statement that the pass-to-pass fixtures cannot be red-first.
- **Separate Linux, macOS, and Windows results** for the hosted lanes `stable`, `msrv`, `service-systemd`, `macos`, `windows-check`. A lane not run is **not run**, never inferred. Unix-only fixtures are **not run** on `windows-check`.
- **Invariant review** against §4, item by item, with citations.
- **Security review**, with symlink-ordering preservation as the security-critical item: the guard at `:501-503` still precedes the open, the FIFO fixture proves it, and the §8 TOCTOU observation is recorded as a **low**-severity carried-forward finding with a reason.
- **Compatibility review** against §9: published `eggup-eggpack 0.1.3` is unchanged for consumers; the published manifest's direct dependency list shortens by one entry; the transitive closure is unchanged; no republication occurred; the `v0.1.3` tag did not move.
- **Docs/operations evidence**: the changelog entry, the `architecture/eggpack-adapter.md` edits, and the explicit statement that no `docs/` change was required.
- **Unresolved findings with severity**, each with its reason.
- **Roadmap disposition** and **registry updates**, recorded in the closure and applied by the closing worker (M006 does not edit `plans/registry.md` or the roadmap itself).

## 16. Handoff notes

- **What a separate republication milestone would require.** M006 leaves `eggup-eggpack 0.1.3` as the published baseline; the shorter dependency list and the delegated digest routine reach registry consumers only when the adapter is published again. That milestone would need: a version bump (the workspace version is shared at `Cargo.toml:15`), a per-crate and root changelog cut, an exact-pin review for `eggup-core`/`eggup-archive`/`eggup-acquisition`/`eggpack-manifest`, a `cargo package` + `cargo publish --dry-run` pass, full hosted qualification, and a GitHub Release note append **without moving the `v0.1.3` tag**. It must also decide whether the deferred error-surface narrowing (§9) rides along — which it must not, because that is a separate behavior change on a published crate.
- **Why no pin cascade is needed.** `eggup_core::hash_file` is exported by the already-pinned `eggup-core 0.1.3` (verified in the published registry source, §3). The `=0.1.3` pins remain valid, so no core republication is required and no adapter republication is forced to carry a seam fix. The usual "a seam fix needs a separate adapter republication to reach it" cascade does not apply.
- **What this plan does NOT authorize.** No publication of any crate and no movement of `v0.1.1`/`v0.1.2`/`v0.1.3`; no change to `eggup-core`, `eggup-archive`, `eggup-acquisition`, or `eggpack-manifest`; no narrowing of the adapter's error surface and no new `AdapterError` variant; no change to the `sha2` dev-dependency; no authenticity, signature, provenance, or trust work; no policy, destination, ownership, permission, release-selection, or service change in the adapter; no change to the extraction handoff; no fix for the §8 TOCTOU observation between the guard and the open (a separate milestone with its own design); and no edits by M006 to `plans/registry.md` or the subsystem roadmap, which are applied by the closing worker as part of the closure.
