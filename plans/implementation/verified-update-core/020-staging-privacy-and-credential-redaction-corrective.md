# Verified Update Core Milestone 020 — Staging privacy and credential redaction corrective

Status: ready for handoff

Repository baseline: `2516d9e08ad764cce2b5617c8d7f20f15c10e6d4`

Source roadmaps:
- Primary: `plans/subsystems/verified-update-core-roadmap.md`
- Affected acquisition boundary: `plans/subsystems/acquisition-transport-roadmap.md`

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant (hardening; no capability is added)

## 1. Objective

Make deployment-private filesystem state private **from creation** rather than by a
subsequent best-effort `chmod`, and stop the acquisition seam's diagnostic surfaces
from emitting credential-bearing material. Two defect families, from one read-only
audit:

- **F-A — creation-mode privacy (core).** The private stage directory, its
  intermediate directories, the backup root, and the mutation-lock record are created
  with umask-derived permissions and narrowed afterwards; the narrowing error is
  discarded. One lock-record creation site never narrows at all.
- **F-B — credential exposure through `Debug` (seam and adapters).**
  `AcquisitionRequest`, `CurlProxy::Custom`, and `ProxyDecision::Custom` derive
  `Debug` over raw URLs and proxy pairs, so `{:?}` prints what `redact_url` exists to
  hide. This contradicts a documented claim.

The audit also produced three items this milestone deliberately does **not** act on;
§5 records them with their reasons.

## 2. Why this milestone is ready

The findings come from a read-only security audit of the exact repository baseline
above: source reading of all eight crates plus `cargo audit` (0 vulnerabilities,
0 unmaintained, 0 yanked across 169 locked dependencies). No finding is above **low**
severity, so none forces a corrective on its own. Two of the five are actionable and
share one root-cause class — a *transition* in private state or in a diagnostic
surface that no existing assertion observes. The other three are recorded in §5 as
accepted residuals with their reasons, and this plan deliberately does not fix them.

The work is dependency-ready:

- No new dependency is required. Every fix uses `std` only: `DirBuilderExt::mode`
  and `OpenOptionsExt::mode` (`std::os::unix::fs`, both long-stable and well inside
  the Rust 1.89 floor), plus manual `Debug` impls.
- No public API changes: no signature, type, variant, field, or trait is added,
  removed, or retyped. `Debug` already exists on every type named here.
- No crate in this workspace has an unpublished change that this would entangle.
  `eggup-core`, `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch` are all
  published at `0.2.0`; these changes stay unpublished until a separately authorized
  train.
- The correct implementation pattern already exists in this workspace
  (`eggup-archive` and `eggup-acquisition`), so this is convergence, not invention.

### How the gaps went undetected

The M003 redaction corrective established `redact_url`, `__scrub_upstream_text`, and
category-only transport errors, and added
`redaction_sentinels_are_absent_from_diagnostics`
(`crates/eggup-acquisition/src/lib.rs:1777-1812`). That test asserts the sentinels are
absent from `Display` output and from `redact_url` results. It never formats a request
or a proxy decision with `{:?}`, so a derived `Debug` — which bypasses the redaction
helpers entirely — had no failing assertion. `architecture/acquisition.md` §8
accordingly lists three redaction mechanisms and calls category-only diagnostics "the
primary defense"; `Debug` appeared in neither the table nor the leak vectors.

For F-A, `transaction_owned_state_is_owner_private`
(`crates/eggup-core/src/lib.rs:1621-1660`) asserts the **final** mode of the stage
directory (`0700`), the staged file (`0600`), and a preserved lock record (`0600`).
Those assertions pass today because the post-hoc `set_permissions` succeeds. Nothing
observes the interval before it runs, and the `let _ =` on the call site makes a
failed narrowing indistinguishable from a successful one. The chain
`architecture/core-transaction.md:42-43` states "directory creation happens
exclusively inside the stage (`stage.rs:73-83`) and the backup root
(`transaction.rs:908`)" — accurate, but it describes *containment*, not the mode of
the created object, so the doc review that produced it could not surface the gap
either.

`lock.rs:579` (`restore_claim`) was missed even by the final-mode assertion: it
re-creates a displaced lock record with `create_new` and never chmods, so its mode is
umask-derived for the record's whole life, on the one path that runs precisely when a
foreign writer may be active.

## 3. Current implementation evidence

**F-A — creation modes**

- `crates/eggup-core/src/stage.rs:260` — `fs::create_dir(&path)` (umask-derived)
  followed by `:265` `let _ = fs::set_permissions(&path, 0o700)`. The retry loop
  treats `AlreadyExists` as a collision and retries with a new name.
- `crates/eggup-core/src/stage.rs:74-82` — `fs::create_dir_all(parent)` followed by
  `let _ = fs::set_permissions(parent, 0o700)`.
- `crates/eggup-core/src/stage.rs:125-140` — the bound-source destination is opened
  with default mode and then narrowed to `0600`; this one at least propagates the
  failure.
- `crates/eggup-core/src/stage.rs:97-103` — the path branch uses `fs::copy`, which
  *carries the source mode onto the destination* before `apply_permissions` narrows
  it, and `staged_is_executable` (`:297-302`) reads executable intent back from that
  copy.
- `crates/eggup-core/src/transaction.rs:922-929` — backup root `fs::create_dir` plus
  ignored `0o700` chmod; `:714` — `fs::create_dir_all` for nested backup parents with
  no mode control.
- `crates/eggup-core/src/lock.rs:419` — `OpenOptions::new().write(true).create_new(true)`
  with no mode; `:431` `let _ = fs::set_permissions(0o600)`. `:579` (`restore_claim`)
  — the same `create_new` with no mode and no narrowing at all.
- Reference implementations that already do this correctly:
  `crates/eggup-archive/src/lib.rs:1259-1286` (`mkdir_at` with `mode(0o700)` at
  creation), `:1305-1320` (`open_at` with `create_new(true).follow(false)` and
  `mode(0o600)`), and `crates/eggup-acquisition/src/lib.rs:503-560`
  (`__acquire_exclusive_temp`, which additionally re-reads the mode and fails closed
  if `0600` could not be enforced).

**F-B — `Debug`**

- `crates/eggup-acquisition/src/lib.rs:20-26` — the doc says "Query and credential
  material is redacted in all diagnostics"; the type derives `Debug` over `url:
  String`.
- `crates/eggup-curl/src/lib.rs:44-54` — `CurlProxy::Custom(Vec<(String, String)>)`
  derives `Debug`; `:57-58` `CurlConfig` derives `Debug` over it.
- `crates/eggup-eggfetch/src/lib.rs:27-37` — `ProxyDecision` mirrors that shape.
- `architecture/acquisition.md:419-451` — §8 "Redaction" lists three mechanisms and
  the "primary defense" argument. `architecture/curl-adapter.md:510-511` already
  accepts one exposure it cannot control (the argv in the process table), which is the
  closest existing statement of this boundary.
- Existing coverage: `redaction_sentinels_are_absent_from_diagnostics`
  (`eggup-acquisition/src/lib.rs:1777`), `diagnostics_are_redacted` (`:1349`),
  `credential_material_is_redacted` (`eggup-curl/src/lib.rs:2279`). All are
  `Display`- and message-oriented.

**Constraint inventory for the change** (verified against source, not assumed)

- The production sites in scope, verified against source:
  - `stage.rs:74` `create_dir_all` + `:81` ignored `0o700` chmod (intermediates);
  - `stage.rs:260` `create_dir` + `:265` ignored `0o700` chmod (stage root);
  - `stage.rs:97` `fs::copy` (source mode carried onto the destination) and
    `stage.rs:125-140` open with default mode + propagated `0o600` chmod;
  - `transaction.rs:714` `create_dir_all` (nested backup parents);
  - `transaction.rs:922` `create_dir` + `:927` ignored `0o700` chmod (backup root);
  - `lock.rs:419` `create_new` with no mode + `:431` ignored `0o600` chmod;
  - `lock.rs:579` `create_new` with no mode and no chmod at all.
- `stage.rs:332` (`apply_permissions`) and `:138` are *not* the same class: `:332`
  must remain (it is the one legitimate mode transition, widening `0600` to `0700`
  for executable intent) and already propagates its error.
- No test in the workspace formats an `AcquisitionRequest`, `CurlProxy`,
  `ProxyDecision`, `CurlConfig`, or `EggfetchConfig` with `{:?}`, so no existing
  assertion encodes the leaking output.
- The non-bound staging path (`fs::copy`) has **no** executable-intent test today:
  `bound_source_preserves_executable_intent_that_the_fresh_stage_masks`
  (`eggup-core/src/lib.rs:485`) and `bound_source_does_not_widen_a_non_executable_source`
  (`:541`) both use bound sources. This matters in §6.2 and §10.

## 4. Invariants that must not regress

- `eggup-core` gains no dependency. Its dependency list stays `sha2`, plus
  `self-replace` on Windows only. `eggup-acquisition` stays dependency-free. No
  `Cargo.lock` change is permitted by this milestone.
- No public API change anywhere: no item added, removed, renamed, or retyped; no
  variant or field change; no new public trait method.
- Commit, rollback, receipt, and lock **semantics** are unchanged: `RolledBack` and
  `RecoveryRequired` keep their current meanings, `acquire` still never recovers, and
  `restore_claim` still refuses to replace an occupied lock path.
- The stage root and backup root keep their existing uniqueness contract:
  `AlreadyExists` selects a new name; an existing directory is never adopted.
- Still no authenticity or signature claim anywhere. SHA-256 remains integrity
  evidence only, and this milestone does not change transport policy, cleartext
  allowance, or trust anchors.
- Rust 2021 and the Rust 1.89 MSRV. Every new API call is long-stable `std`.
- No version bump, publication, tag, or release-history change.
- Windows behavior is unchanged: every `mode` call is `cfg(unix)`-gated, and the
  Windows graph never had POSIX modes.
- Owner-private intent is preserved exactly: stage directories `0700`, staged files
  `0600` unless executable intent requires `0700`, lock records `0600`.

## 5. Scope

### In scope

- F-A: creation modes for the stage root, stage intermediates, backup root, nested
  backup parents, and both lock-record creation sites, with failure propagated
  instead of ignored.
- F-A: staged-file creation at `0600`, with executable intent read from the source
  rather than from the mode that `fs::copy` carries.
- F-B: manual `Debug` impls for `AcquisitionRequest`, `CurlProxy`, and
  `ProxyDecision`.
- A source-level guard for the creation-mode invariant, plus behavioural tests for
  the redaction and for the path staging branch.
- Doc, changelog, roadmap, and registry reconciliation per §12.

### Explicitly out of scope

- **Finding 3 — pathname-based staged and acquired-file proofs.** `revalidate_staged_under_lock`
  (`transaction.rs:851-908`) stats and re-opens by path, `hash_file`
  (`integrity.rs:72-87`) follows a symlink at open time, and
  `eggup-eggpack::validate_acquired_archive` (`eggpack/src/lib.rs:513-522`) documents
  that its `symlink_metadata` guard is the only link rejection. Closing this fully
  needs either `O_NOFOLLOW` (which core cannot name without `libc` or a hardcoded
  per-platform constant) or handle-relative hashing and `renameat` (which needs
  `fs_at`, forbidden in core by the dependency boundary). A handle-retained hash
  would fix only the hash and leave the rename by path, so it buys partial closure at
  real complexity cost. Impact is bounded on the composed path, and that bound is
  verified rather than assumed: `archive_plan_for` pins every declared member with
  `Some(m.exact_size)` and `Some(m.sha256)` (`crates/eggup-eggpack/src/lib.rs:552-556`)
  and every declared member must be present, so a replaced archive can only fail
  extraction, not substitute bytes. That is a property of the composed path, **not**
  of `eggup-archive` itself: `ArchiveMember::new` takes `Option`s by design
  (`crates/eggup-archive/src/lib.rs:98-114`), so an unpinned declaration is
  unprotected against source substitution — a caller obligation the crate's
  policy-neutral contract leaves open on purpose.
  **Decision: accepted residual.** This milestone removes its
  precondition (a directory others can write into) and records the residual in
  `architecture/core-transaction.md`. Revisit only if core is ever allowed an
  `*at` dependency.
- **Finding 5 — `ExistingAsOwnedVerifier`.** The rustdoc already states the type
  "exists only for deterministic tests and examples" and that production consumers
  SHOULD use `ExactDigestVerifier` (`domain.rs:233-241`);
  `architecture/core-transaction.md:293-296` calls it "fixture-grade
  trust-everything", and `docs/quickstart.md:47-50` tells readers not to use it in
  production. Hiding or renaming it would churn four examples, four test modules, and
  three docs for no security gain while removing a documented test helper.
  **Decision: accepted, no change.**
- The cleartext-HTTP allowance and the absence of an authenticity layer
  (`AcquisitionRequest::new` accepts `http://`; ADR-0003 owns the boundary). This is
  a consumer policy decision, not a defect in these crates.
- `eggup-archive`, `eggup-service`, and `eggup-eggpack` source. Archive is the
  reference implementation, not a target.
- Publication, version changes, CI workflow changes, and new dependencies.
- Any change to `plans/000`–`003` or to an accepted ADR.

## 6. Required production changes

### 6.1 Core staging and backup directories (F-A)

`stage.rs::create_stage_directory` (`:250-274`): replace `fs::create_dir(&path)` with
a **non-recursive** `fs::DirBuilder` carrying `mode(0o700)` on Unix, and delete the
`let _ = fs::set_permissions(...)` block. The builder must stay non-recursive:
`recursive(true)` returns `Ok(())` for an existing directory and would silently adopt
a squatter's directory in place of the `AlreadyExists` retry contract. Error mapping
stays `Error::io("creating private stage", source)`.

`stage.rs::copy_members` (`:70-108`): replace `fs::create_dir_all(parent)` plus the
ignored chmod with one recursive `DirBuilder` carrying `mode(0o700)`. Recursive mode
is correct *here*, because these are intermediates under a root this process just
created, and `DirBuilderExt::mode` applies to every component the builder creates.

`transaction.rs::create_backup_directory` (`:910-936`): same non-recursive treatment,
keeping `.eggup-backup-{pid}-{nonce}-{nanos}` naming, the `AlreadyExists` retry, and
`Error::io("creating backup set", source)`.

`transaction.rs::backup_members` (`:713-719`): replace `fs::create_dir_all` with the
recursive `mode(0o700)` builder.

### 6.2 Core staged files (F-A)

`stage.rs::stage_bound_source` (`:121-144`): set `.mode(0o600)` on the destination
`OpenOptions` (Unix only) and delete the now-redundant post-hoc
`fs::set_permissions(destination, 0o600)`. The `seek(0)` rewind, `create_new`
semantics, and both error strings stay unchanged.

`stage.rs::copy_members` path branch (`:96-104`): replace `fs::copy` with a new
`pub(crate) fn copy_into_private_stage(source: &Path, destination: &Path) -> Result<()>`
that opens the source read-only, opens the destination with
`write(true).create_new(true)` plus `mode(0o600)`, copies with `std::io::copy`, and on
any failure removes the partial destination before returning. Two deliberate
consequences, both required:

- `create_new(true)` replaces `fs::copy`'s truncate-an-existing-file behaviour. In a
  stage root this process created moments earlier, any pre-existing entry at a member
  path is hostile or injected, so failing closed is the correct change. It must be
  recorded in §8, not treated as an accident.
- Executable intent must be read from the **source** (`fs::metadata(member.source())`,
  which follows symlinks exactly as `fs::copy` did) *before* the destination exists,
  because the destination is now `0600` at creation. `staged_is_executable`
  (`:297-302`) becomes dead and is deleted; add a `source_is_executable` with the same
  `cfg(unix)` / `cfg(not(unix)) -> Ok(false)` shape as `bound_handle_is_executable`.

`apply_permissions` (`:309-338`) is unchanged: it is the one legitimate widening step
(`0600` → `0700` for executable intent, `0600` otherwise) and already propagates its
error.

### 6.3 Core lock record (F-A)

`lock.rs::create_new` (`:406-439`): build an `OpenOptions` with
`write(true).create_new(true)` plus `mode(0o600)` on Unix, then open. Delete the
`let _ = fs::set_permissions(&path, 0o600)` block. The
`AlreadyExists → Ok(None)` contention path is untouched.

`lock.rs::restore_claim` (`:576`): apply the same `mode(0o600)` at creation. This
site has no narrowing at all today, so a restored record is umask-derived for its
whole life. Nothing else in the restore protocol changes: it still uses `create_new`
so it can never replace an occupied lock path, and a partial write still returns
`RecoveryRequired` naming the claim.

### 6.4 Seam and adapter diagnostics (F-B)

`eggup-acquisition/src/lib.rs::AcquisitionRequest`: replace `#[derive(Debug, ...)]`'s
`Debug` with a manual impl that formats `self.redacted()`. The raw `url` field must
not reach the formatter. Keep `Clone, PartialEq, Eq`.

`eggup-curl/src/lib.rs::CurlProxy`: manual `Debug`. `Disabled` and `FromEnvironment`
print as today. `Custom(pairs)` must print each variable **name** with a redacted
value — `Custom([("HTTPS_PROXY", "<redacted>")])` — and never the value. Names are the
diagnosis-relevant part and are not secret; a value routinely is, because a proxy URL
can carry `user:password@`.

`eggup-eggfetch/src/lib.rs::ProxyDecision`: the same manual impl, same shape.

`CurlConfig` and `EggfetchConfig` keep `derive(Debug)`: their only credential-bearing
field is the proxy enum, which now redacts. Do not add redaction to types that hold
no credential material — an over-broad change would make ordinary debug output
useless without buying privacy.

## 7. Ordered work packages

1. **F-B first** — it is isolated, needs no behavioural reasoning about the
   transaction, and lands with its own tests (WP 6) so a late stop still leaves a
   complete, verified improvement.
2. **Stage root and intermediates** (§6.1, `stage.rs`) with the non-recursive
   constraint.
3. **Staged files** (§6.2), including the source-read executable intent and the
   `create_new` tightening. Re-run the full core suite here: this is the step most
   likely to expose an untested assumption.
4. **Backup root and nested parents** (§6.1, `transaction.rs`).
5. **Lock record** (§6.3, both sites).
6. **Tests** (§10), including the source-level guard.
7. **Docs and records** (§12), then the local gate and the Rust 1.89 check.

Work packages 2–5 are independent of 1 and of each other; only 6 depends on all of
them.

## 8. Failure, cancellation, restart, and contention semantics

- **Stage-root creation failure** stays a preparation `Err`, not a receipt: it happens
  before the mutation lock, so no receipt, disposition, or recovery path is involved.
  The operation string `"creating private stage"` is unchanged, so
  `FailureCategory::Filesystem` mapping is unchanged.
- **Directory narrowing is no longer a separate step**, so there is no longer a
  state in which the directory exists with umask permissions. If the creation cannot
  apply `0700`, creation fails; it does not continue permissively.
- **`AlreadyExists` still means collision.** `DirBuilder` non-recursive returns
  `AlreadyExists` for an existing path on Unix and Windows; the 32-attempt loops keep
  their current bound and their `Error::invalid` exhaustion message. A recursive
  builder at the stage or backup root would break this and is prohibited.
- **A pre-existing staged member now fails closed.** `create_new` returns
  `AlreadyExists`, surfaced as `Error::Io` from preparation; the `Stage` `Drop`
  cleanup still removes only a real `.eggup-stage-` directory whose parent matches,
  so no partial artifact survives.
- **Partial copy** removes the partial destination before returning `Err`; preparation
  then drops the stage as today.
- **Lock contention is unchanged**: `create_new` remains the only winner, a symlinked
  or existing record still yields `Ok(None) → Error::UpdateInProgress` from
  `acquire`, and `acquire_with_recovery` still requires an exact-record
  `ProvenStale` verdict. Changing the record's mode does not affect record bytes,
  `LockObservation::record()` equality, or the claim protocol.
- **`restore_claim`** keeps its rule that it never replaces an occupied path; only
  the mode of the record it materializes changes.
- **Cancellation/restart**: nothing here introduces a journal, resume point, or
  restart obligation. Crash at any point leaves either no object or an owner-private
  object, and the crate's existing "no crash journaling" boundary is unchanged.
- **`Debug`** has no failure semantics.

## 9. Compatibility and migration

- **Public API: unchanged.** No item, signature, variant, field, or trait changes.
  `Debug` was already implemented on all three types; only its output changes.
- **`Debug` output is not a stability contract** and is not covered by semver. State
  this in the changelog entry so a consumer that scraped `{:?}` knows it was never
  supported. No migration is required.
- **File modes: unchanged at rest.** Installed and staged artifacts keep `0700`/`0600`
  exactly as documented; only the transition and the failure path change. A consumer
  that observes modes through `staged_path()` sees the same final values.
- **No dependency, lockfile, MSRV, or Windows change.**
- **Versions stay at `0.2.0`.** The four affected crates are published; these changes
  are unpublished until a separately authorized train. Per-crate `CHANGELOG.md`
  `Unreleased` sections carry the entry plus the standing no-publication disclaimer,
  and the root aggregate `CHANGELOG.md` repeats it. `eggup-archive` is unchanged and
  gets no entry.

## 10. Required tests

Behavioural (new):

1. `acquisition_request_debug_never_prints_credentials` (`eggup-acquisition`) — a
   request with userinfo, query token, and fragment; assert `format!("{:?}")` contains
   none of the three sentinels and does contain `<redacted>`.
2. `curl_proxy_debug_never_prints_pair_values` (`eggup-curl`) — `CurlProxy::Custom`
   with a credential-shaped value; assert the value is absent from `{:?}` of both the
   enum and `CurlConfig`.
3. `proxy_decision_debug_never_prints_pair_values` (`eggup-eggfetch`) — the same for
   `ProxyDecision::Custom` and `EggfetchConfig`.
4. `path_staged_copy_preserves_executable_intent` (`eggup-core`) — a `0755` source
   with `PermissionsIntent::Preserve` staged through the **non-bound** path; assert the
   staged mode is `0700`. This closes the gap noted in §3: the branch has no such test
   today, so §6.2's change could otherwise silently drop the executable bit on every
   non-bound install.
5. `path_staged_copy_does_not_widen_a_non_executable_source` (`eggup-core`) — `0644`
   source, `Preserve`; staged `0600`.
6. `private_stage_copy_refuses_a_pre_existing_entry` (`eggup-core`) — call
   `copy_into_private_stage` with an existing destination; assert the error is
   `Error::Io` and the pre-existing content is unchanged.

Structural guard (new file `crates/eggup-core/tests/creation_mode_guard.rs`, `std`
only, `CARGO_MANIFEST_DIR`-anchored, modelled on
`crates/eggup-transport-footprint/tests/cross_link_guard.rs`):

7. `stage_and_backup_roots_set_their_mode_at_creation` — `stage.rs` and
   `transaction.rs` contain `DirBuilderExt` and `.mode(0o700)`; neither contains
   `fs::create_dir(` or `fs::create_dir_all(`; neither contains
   `let _ = fs::set_permissions(`.
8. `lock_record_sets_its_mode_at_creation` — `lock.rs` contains `OpenOptionsExt` and
   `.mode(0o600)`, and contains no `set_permissions` at all.
9. `staged_file_creation_sets_its_mode` — `stage.rs` contains
   `OpenOptionsExt`/`.mode(0o600)` and no `fs::copy(`.

The guard is a **source-level structural** check, not behavioural evidence: it cannot
observe a race, and it must be documented as such in its own module doc, the same way
`cross_link_guard.rs` states its limits. Its job is to keep the defect from returning
through a later refactor that the behavioural suite cannot see.

Retained (must keep passing unchanged):

- `transaction_owned_state_is_owner_private` (`eggup-core/src/lib.rs:1621-1660`) —
  final modes `0700`/`0600`/`0600`.
- `bound_source_preserves_executable_intent_that_the_fresh_stage_masks`
  (`:485`) and `bound_source_does_not_widen_a_non_executable_source` (`:541`).
- `redaction_sentinels_are_absent_from_diagnostics`,
  `diagnostics_are_redacted`, `credential_material_is_redacted`.

## 11. Required verification commands

```sh
cargo fmt --all
cargo fmt --all -- --check
git diff --check

cargo test -p eggup-core --all-targets --locked
cargo test -p eggup-acquisition --all-targets --locked
cargo test -p eggup-curl --all-targets --locked
cargo test -p eggup-eggfetch --all-targets --locked

./scripts/check-local.sh
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-docs-truth.sh
cargo audit
```

`./scripts/check-local.sh` already runs `cargo fmt --all -- --check`, workspace
clippy with `-D warnings`, workspace tests, `cargo doc`, `cargo tree`, and
`check-docs-truth.sh`; the focused commands above exist to localize a failure, not to
replace the gate. `check-docs-truth.sh` is listed separately because it is the one
check that can fail for a documentation-only edit. `cargo audit` is evidence of no new
advisory, and it must be run after the change because a dependency change here is a
stop condition (§14).

Hosted lanes are **not** run by these commands. A closure that has not run
`stable`, `msrv`, `macos`, `systemd`, and `windows-check` must report each as **not
run**, never infer it from local results. `windows-check` matters even though every
change is `cfg(unix)`: it proves the `cfg` split compiles.

## 12. Documentation updates

- `architecture/core-transaction.md` — the staging section (`:108-209`) must state
  that the stage root, its intermediates, and the backup root are created with their
  mode at creation and that a failure to apply it fails the operation; correct the
  `stage.rs:73-83` / `transaction.rs:908` anchors if line numbers move (they will);
  add the accepted residual for path-based revalidation from §5 to the ownership or
  staging section, naming what would be required to close it. `:42-43` currently
  describes directory creation as confined to two sites — it now needs the mode
  statement too.
- `architecture/acquisition.md` §8 "Redaction" — add `Debug` as a fourth mechanism
  row with its scope (`AcquisitionRequest`, `CurlProxy`, `ProxyDecision`), and extend
  the "three details a reviewer should know" list so the table and the prose agree
  (the section heading says "three mechanisms"; that word changes).
- `architecture/curl-adapter.md` — the diagnostics section must record that the
  process-table exposure (`:510-511`) and the `Debug` surface are now both explicit
  boundaries, and that the latter is closed.
- `architecture/eggfetch-adapter.md` §9 — add `Debug` to the redaction bullet list.
- `crates/{eggup-core,eggup-acquisition,eggup-curl,eggup-eggfetch}/CHANGELOG.md` —
  `Unreleased` entries with the no-publication/no-migration disclaimer; `eggup-core`
  and `eggup-acquisition` also carry the "`Debug` is not a stability contract" line.
- Root `CHANGELOG.md` — aggregate the four entries.
- `crates/*/README.md` — only where the crate's stated privacy or redaction claim is
  affected; do not restate rules that the contracts own.
- `plans/registry.md` finding 18 and the milestone row; the subsystem roadmap status
  table. Per the registry update rule, the registry carries the transition and the
  plan path, not this plan's detail.

Do **not** edit `plans/000`–`003` or an accepted ADR. `docs/` needs no change: the
verifier is untouched, so the quickstart's warning stays accurate as written.

## 13. Acceptance criteria

1. No production code path creates a private directory or file with umask-derived
   permissions: every site in §6 sets its mode at creation, and no
   `let _ = fs::set_permissions` remains in `stage.rs`, `transaction.rs`, or
   `lock.rs`.
2. `lock.rs::restore_claim` materializes its record at `0600`.
3. The stage root and backup root still refuse to adopt an existing directory, proven
   by the `AlreadyExists` retry paths being intact and by the guard in §10.7.
4. Executable intent on the non-bound staging path is preserved from the source and
   covered by §10.4–10.5.
5. `{:?}` of `AcquisitionRequest`, `CurlProxy`, and `ProxyDecision` contains no
   userinfo, query, fragment, or proxy value, proven by §10.1–10.3.
6. The retained tests in §10 pass unchanged; no existing assertion is weakened,
   deleted, or rewritten to accommodate the change.
7. `./scripts/check-local.sh` and `cargo +1.89.0 check --workspace --all-targets
   --locked` pass, and `cargo audit` reports no advisory.
8. `git diff` shows no `Cargo.toml`, `Cargo.lock`, version, or publication change.
9. Documentation in §12 is updated and `./scripts/check-docs-truth.sh` passes.

## 14. Stop conditions

Stop and report, without working around, if:

- a fix appears to need a new dependency, `libc`, a hardcoded platform constant, or
  any change to `Cargo.toml`/`Cargo.lock`;
- a fix appears to need a public API change, including a new public helper;
- the source-level guard cannot be written unambiguously (for example, a legitimate
  remaining `fs::create_dir` whose presence cannot be distinguished from the defect
  without fragile context matching);
- closing the staged-file gap would require the handle-based redesign excluded by §5
  — record it as a residual instead of attempting it here;
- a Windows behaviour change turns out to be required (it should not be: every change
  is `cfg(unix)`);
- any retained assertion in §10 must be modified to pass. That means the change
  altered semantics the milestone promised to preserve;
- staged-file staging or executable-intent behaviour turns out to differ from
  `fs::copy` in a case any existing contract covers. Report the difference rather than
  choosing silently.

## 15. Closure evidence required

- Requirement→evidence matrix mapping each of §13's nine criteria to a command result
  or a named test, at an exact commit.
- Per-platform results, each stated as run or **not run**: Linux (local gate), macOS,
  Windows (`windows-check`), Rust 1.89 (`msrv`), systemd lane. A lane not executed is
  not inferred from another.
- Hosted run link and conclusion, when a run is authorized. If not run, say so; a
  closure may be `conditionally closed` on hosted qualification, as M005/M012 were.
- The exact commands from §11 with their observed outcomes.
- Unchanged-surface evidence: `git diff --stat` showing no `Cargo.toml`/`Cargo.lock`
  and no version change; an explicit statement that no public item was added, removed,
  or retyped.
- The three non-actionable audit items (§5) recorded with their severity and the
  condition that would reopen each.
- Confirmation that `eggup-archive` was used as the reference and not modified.
- Registry and roadmap updated, with finding 18 pointing at this plan.

## 16. Handoff notes

- This milestone is unpublished by design. It changes four published crates; nothing
  reaches a consumer until a publication train is separately authorized, and commit or
  push requires its own explicit authorization.
- Finding provenance: the audit that produced F-A/F-B is a read-only review at
  `2516d9e`. Its three non-actionable items are recorded in §5 with reasons; do not
  "helpfully" fix finding 5 or the path-based proofs here.
- If a later milestone reopens the path-based residual, the natural home is a core
  `*at`-capable extraction boundary, which today belongs to `eggup-archive`. That is a
  boundary decision, not a patch.
- The guard file is deliberately narrow. If it starts needing context-sensitive
  matching to stay green, delete it and say so in the closure rather than growing it
  into a brittle second implementation.
- `stage.rs:138` and `stage.rs:332` look similar and are not: `:138` is a latent
  narrowing of a freshly created file and goes away; `:332` is the intended
  executable-intent widening and stays. A reviewer diff that removes both is wrong.
