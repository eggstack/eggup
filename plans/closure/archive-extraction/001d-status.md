# Archive Extraction M001d — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md#M001d--Handle-backed-source-handoff`

Original work continued by this pass:

- `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`
- `plans/closure/archive-extraction/001c-status.md` (Section 14 stop record)

Reviewed source baseline: `0b0cdaafb308e4b27b4ecfa6b7144e90c84a488b` (M008 green head; source-plan baseline). Work began from the M008 closure head `421ad4afc3b0bd700707d6b50fdf327c7295d908`; every commit between the plan baseline and the work head is planning/registry/roadmap text only.

Implementation commits:

- `09064a34896eaeb131b8b9c5ff787e9613d0e563` — handle-backed source handoff: retained open member objects, `BoundMember`/`BoundExtraction`/`DeferredCleanup`, core `BoundSources` + `prepare_with_bound_sources`, deterministic races, docs/changelogs.
- `18d83decd0c894279dd5e5d89407e1423f839d99` — qualify M001d races for Windows hold-open rename refusal + newer-clippy fix (see Hosted evidence).
- This closure batch (docs/planning only: closure record + roadmap/registry downstream updates; head recorded in registry as the M001d closure head).

Hosted final qualification: CI run [36335233644](https://github.com/eggstack/eggup/actions/runs/36335233644) on `18d83de`; Stable Linux, Rust 1.89 MSRV, macOS, and Windows all passed (conclusion `success` on all four jobs).

Superseded red run (honest history): first implementation head `09064a3` got run [36334772510](https://github.com/eggstack/eggup/actions/runs/36334772510) — Stable clippy failed on a newer `useless_conversion` lint (`zip(handles.into_iter())`) unknown to the local toolchain, and Windows failed 8 rename tests with `Access is denied`. Root cause (both Windows pre-existing M001b/M001c races and 2 new races): M001d retains member objects open from creation, and Windows refuses directory renames while objects inside are held open. That refusal is fail-safe — the rename attack itself is blocked by the OS — not a contract gap: create-new/no-follow, bound-source staging, and cleanup-after-consume all pass on Windows. The fix commit qualifies the races (see Failure review) without touching production semantics. No genuine runtime failure was misclassified; the red run is superseded, not hidden.

## Executive finding

No staged byte is obtained by re-resolving a recorded extraction pathname or by reopening a member name after the handoff boundary. Tar and zip bound sources resolve only through the already-open member object: readable authority is established on the same object at creation (read + write via the same atomic handle-relative create-new/no-follow operation), the handoff rewinds it to byte zero, and staging rewinds again before copying, so a non-zero cursor can never truncate staged bytes. The existing path-source `ArtifactMember::new` API and semantics are preserved for non-archive callers.

M001d is closed. No medium-or-higher archive authority finding remains open. Egress M006 and Eggpack M002 return to ready-to-author (see Roadmap disposition).

## Requirement-to-evidence matrix

| Requirement (source plan Section 10) | Evidence | Result |
|---|---|---|
| tar/zip root rename before first bound-source write; bound reads owned; foreign untouched | `bound_tar_root_rename_before_first_write_reads_owned_bytes`, zip twin (hook seq 0, portable; both green on Windows too) | passed |
| two-member replacement between writes; both bound sources owned | `bound_two_member_replacement_between_writes_reads_owned_bytes` (Unix-gated; hook seq 1 rename refused on Windows, see Failure review) | passed on Linux/macOS |
| post-extraction member-entry replacement before staging; staging consumes original bytes; replacement untouched | `bound_tar_member_entry_replacement_after_handoff_stages_owned_bytes`, zip twin, `bound_two_member_replacement_after_handoff_stages_owned_bytes` (remove+write, portable; green on Windows) | passed on all lanes |
| post-handoff root replacement before staging; staging consumes owned bytes | `bound_post_handoff_root_replacement_stages_owned_bytes` (Unix-gated; same Windows reason) | passed on Linux/macOS |
| non-zero cursor before staging; full file from byte zero | `bound_nonzero_cursor_stages_full_file_from_byte_zero` (advances shared cursor post-handoff, portable; green on Windows) | passed on all lanes |
| raced existing output + symlink/reparse output fail closed, targets untouched | pre-existing `raced_existing_output_file_in_owned_root_fails_without_clobber`, `raced_output_symlink_...` (unix), `windows_reparse_...` (windows) — untouched, green | passed |
| Unix `0600` preserved on bound sources | `bound_sources_preserve_owner_private_mode_and_exact_evidence` (advisory + staged mode `0600`) | passed |
| size/digest evidence exact | bound evidence asserts (`bytes_written`, `sha256` vs `Sha256::digest`) in entry-replacement + mode tests | passed |
| all M001/M001a/M001b/M001c suites green | full archive suite 44/44 Linux/macOS; M001b race setups converted to pre-write hook renames with all assertions unchanged (see Failure review) | passed |
| stable Windows bound-source tests incl. cleanup-after-consume | Windows lane: archive 37/37 incl. 6 portable bound tests + migrated consumer test, every staging test ending in `DeferredCleanup::cleanup` after handles close | passed |
| Rust 1.89 all-target check | `cargo +1.89.0 check --workspace --all-targets --locked` + hosted MSRV job | passed |
| `eggup-core` archive-format independent | `cargo tree -p eggup-core`: only `sha2`; no `tar`/`zip`/`flate2`/`fs_at` | passed |
| no medium-or-higher authority finding open | reviews below; Windows rename refusal is fail-safe, not a gap | passed |

## Production implementation and audit evidence

Chosen shape: **C (equivalent object-bound source)** — moved open member objects (A-like ownership) staged through an additive prepare variant (B-like copy timing). Rationale recorded: verbatim A (`ArtifactMember::from_handle` storing `File` in `ArtifactMember`) would strip `Clone`/`PartialEq`/`Eq` from `ArtifactMember`/`ArtifactSet`/`InstallPlan` — broad breakage against Section 9. Verbatim B (extraction copies into core staging) needs an archive→core runtime dependency or staging-layout duplication — unjustified. The chosen seam adds zero dependencies and leaves every existing type, derive, and constructor untouched.

Before/after staging call graph:

```text
BEFORE (path re-resolution):
extract -> ExtractedMember{path: root.join(name)} -> consumer maps path
  -> ArtifactMember::new(id, source_path, dest) -> InstallPlan::prepare
  -> fs::copy(member.source(), staged)            # re-opens the NAME

AFTER (object-bound):
extract (handle-relative create read+write) -> retained open File
  -> persist (handles move, authority kept) -> into_bound_sources
  -> BoundMember{handle} -> consumer maps advisory_path (diagnostics only)
  -> ArtifactMember::new(id, advisory, dest) [UNCHANGED] -> InstallPlan::new
  -> into_members -> BoundSources{id: File} -> prepare_with_bound_sources
  -> stage_bound_source: seek(0) + io::copy(handle, staged)  # no lookup
  -> handles consumed/closed -> DeferredCleanup::cleanup (handle-only)
```

Exact private helper diff: `git diff 421ad4a..18d83de -- crates/eggup-core/src crates/eggup-archive/src` (plus docs). Public API diff:

- `eggup-core` (additive only): `BoundSources::new/insert`, `InstallPlan::prepare_with_bound_sources`. `ArtifactMember`, `ArtifactSet`, `InstallPlan` construction, derives, and `prepare()` byte-for-byte unchanged.
- `eggup-archive` (additive only): `BoundMember` (evidence accessors, `advisory_path`, `handle_mut`, `into_open_object`), `BoundExtraction::members/members_mut/root/into_members/cleanup`, `DeferredCleanup::root/cleanup`, `PersistedExtraction::into_bound_sources`. `ExtractedMember` (incl. `Clone`/`PartialEq`/`Eq`), `ExtractedArchive`, `PersistedExtraction::{root,members,cleanup}` unchanged; `ExtractedMember::path()` documented advisory/non-authoritative.
- The one in-tree consumer test (`extracted_file_can_be_prepared_as_an_ordinary_core_artifact`) migrated to the bound flow; all other pre-existing tests assert identical behavior (M001b race setups converted per Failure review, assertions unchanged).

Readable-handle/cursor evidence: `create_private_file_at` adds `read(true)` to the same atomic `open_at` create-new/no-follow call; `into_bound_sources` flushes + seeks each object to zero; `stage_bound_source` seeks to zero again before copying; the cursor test moves the shared cursor post-handoff and stages full bytes. No `try_clone` anywhere in the bound path.

Handle lifetime/cleanup ordering evidence: compile-time moves only (`persist` → `into_bound_sources` → `into_members` → `BoundSources` → staging consumes handles → `DeferredCleanup::cleanup`). Every staging test closes/consumes handles before cleanup; Windows hosted tests exercise cleanup-after-consume (member-entry + cursor + migrated tests green on Windows). No pathname-recursive fallback exists on any cleanup path; `remove_owned_root_via_handle` semantics unchanged (empties via handle, reports `CleanupFailed` residue).

No-clobber/no-follow evidence: unchanged `create_new(true)` + `follow(false)` creation (raced-output tests green); staged bound destinations are created `0600` on Unix before `apply_permissions`, matching the `0600`-source path flow end state.

Dependency/footprint delta: zero new dependencies (`cargo tree -p eggup-archive` unchanged: `flate2`, `fs_at 0.2.1`, `sha2`, `tar`, `zip`; `cargo tree -p eggup-core`: `sha2` only). No first-party unsafe (`forbid(unsafe_code)` holds). Rust 1.89 holds (no new language features; `Option::is_some_and` 1.70+, `File::rewind`-equivalent `seek` long stable).

## Exact commands and results

Local Darwin arm64 (on `18d83de`):

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test -p eggup-core --lib bound_source --locked           passed (4/4)
cargo test -p eggup-archive --lib bound_ --locked              passed (9/9)
cargo test -p eggup-archive --lib --locked                     passed (44/44)
cargo test --workspace --all-targets --all-features --locked    passed (all suites green)
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked           passed
./scripts/check-local.sh                                         exit 0 (18 ok suites, zero failures/errors)
cargo tree -p eggup-archive --locked / -p eggup-core            reviewed, deltas zero (above)
git diff --check                                                passed
```

Hosted final run 36335233644 (on `18d83de`):

- Stable checks: success (fmt, clippy incl. the newer lint, full workspace tests — archive 44/44, core 47/47 — docs).
- MSRV check: success (Rust 1.89 workspace all-target check).
- macOS tests: success (full workspace; Unix-gated rename races `... ok`).
- Windows archive/acquisition/service tests and check: success (archive 37/37 incl. 6 portable bound tests + migrated consumer test with cleanup-after-consume tails; service diagnostics/SCM; workspace check).

## Invariant review

- Declared members materialize only beneath the exact extraction-root object (handle-relative creation unchanged; M001c write semantics unchanged).
- Member creation remains exclusive/no-clobber, no-follow, owner-private (`0600` Unix, now with read authority on the same object).
- Resource, path, entry-count, decompression, size, and digest bounds from M001 unchanged.
- M001b handle-relative cleanup semantics unchanged (same `remove_owned_root_via_handle`; race setups converted, assertions identical).
- No successful result exposes an authoritative pathname: `ExtractedMember::path()` and `BoundMember::advisory_path()` are documented diagnostics-only.
- No staged archive byte is obtained by reopening a member name after the extracted member object could have been replaced (member-entry races prove it on all lanes).
- Ordinary path-source API preserved for non-archive callers; no unrelated migration required.
- Handle/cursor ownership deterministic (move-only types, dual rewind).
- Cleanup failure remains fail-closed, never mutates foreign state, no retry loops, private residue acceptable.
- SHA-256 remains integrity evidence, not ownership/authenticity evidence.
- `eggup-core` remains archive-format independent.
- No first-party `unsafe`; Rust 1.89 MSRV; no Egress/Eggpack integration in M001d.

## Failure/restart/contention review

Extraction remains non-resumable; retries use a fresh private root. Handle authority loss (flush/seek failure at handoff) fails closed with `Io` and truthful residue; a failed stage copy (`rewind`/`create`/`copy`) fails closed with `Io` and never reopens the path as fallback; unknown bound identities and leftover handles fail closed with `Invalid`; foreign state is never removed; cleanup waits through ownership sequencing (stage → close → cleanup) rather than retry loops; no hidden bound-to-path downgrade (`prepare()` on advisory paths is the caller's explicit path-source choice, documented as non-archive).

Windows hold-open rename refusal (observed, fail-safe): while member objects are held open, Windows denies directory renames with `Access is denied`. The rename attack is therefore blocked by the OS itself on that platform; Unix proves redirect resistance by simulation, Windows proves no-reresolution by member-entry races plus OS refusal. This is not a Section 14 stop: `create_new`/`no-follow`/bound-source/cleanup semantics are fully preserved on Windows (all green); no contract weakening was required — only race-simulation conditioning. Pre-write-hook M001c races and all non-rename suites run unmodified on Windows.

Stop-condition check (source plan Section 14): no unsafe/FFI needed; no false-positive file identity used as authority (objects, not identities); no digest-equals-foreign trust; no new dependency; Windows preserves equivalent semantics per above. No further ADR/plan required.

## Compatibility and migration review

Additive API change by design. `ArtifactMember::new(id, source_path, destination)` and its path-source semantics are preserved for ordinary already-acquired local artifacts — existing simple consumers migrate nothing. The bound-source path is archive-scoped: new types `BoundMember`/`BoundExtraction`/`DeferredCleanup` (archive) and `BoundSources`/`prepare_with_bound_sources` (core).

Auto-trait accounting: the new handle-owning types are move-only (`File` is not `Clone`; cursors are shared, so no assumed-independent clones) and therefore not `Clone`/`PartialEq`/`Eq`; `Debug` is derived. Existing `ExtractedMember` (`Clone`/`PartialEq`/`Eq`), `ArtifactMember`/`ArtifactSet`/`InstallPlan` (all derives) are unaffected — effects confined to the new bound-source path per Section 9. Crate README/CHANGELOG posture (`Send` but not `Sync` handle owners) extended consistently to the new types.

No Egress/Eggpack consumer migration occurs in M001d; they re-gate on M001d closure (see Roadmap disposition). New public items carry rustdoc stating the authority model (object-bound member object; pathname- and member-name-advisory) and cleanup responsibility transfer.

## Security review

The exact confusion the stale plan permitted (root-handle + later-name reopen acquiring a replacement object) is now impossible on the bound path: staging performs zero name lookups after the handoff boundary, proven by deterministic member-entry replacement races on every hosted lane with the foreign replacement verified untouched. The recorded path cannot silently resolve to foreign state because it is never opened. Failed copies never fall back to path re-resolution. No new attack surface (no network, no elevation, no new parsing, no unsafe); no secret flow. No medium-or-higher finding remains open.

## Documentation and operations evidence

- `crates/eggup-archive/README.md` (read authority, handoff flow, advisory-path doctrine, ordering, gates), `crates/eggup-archive/CHANGELOG.md` + root `CHANGELOG.md` (`Unreleased` M001d entries, no-publication/no-migration disclaimers), `architecture/archive-extraction.md` (ownership/cleanup/handoff section) updated in the implementation commits.
- Per-platform results: Linux Stable (full gate), Rust 1.89 MSRV (check), macOS (full workspace incl. Unix rename races), Windows (archive/curl/service suites incl. bound member-entry/cursor/cleanup-after-consume tests). No platform inferred from another.

## Unresolved findings

- None: no high- or medium-severity archive issue remains. Informational: registry `Latest planning registration head` lags the closure-batch head by one commit per standing norm; the next batch records it. Informational: Windows refuses directory renames while member objects are held open (fail-safe; see Failure review) — rename-with-held-object races are Unix-gated with Windows coverage via member-entry races plus OS refusal.

## Roadmap disposition and downstream unblock

Archive Extraction M001d moves from ready to closed; the subsystem completes its M001b/M001c/M001d corrective chain with green hosted qualification. Future-plan triage (unblocking review):

- **Consumer Adoption M006 Egress**: blocked on M001d → **writable; plan unwritten** (unblocked by this closure; M001d closed `18d83de` with hosted run `36335233644`). Authoring the Egress plan is newly permitted, not performed here.
- **Eggpack Interoperability M002**: blocked on M001d → **writable; plan unwritten** (same gate satisfaction; archive-side handoff now qualified).
- **Gregg M004**: writable but intentionally unwritten per separate authoring decision; M001d does not change its prerequisites or authorize migration, so no status change.
- **Eggpack M003/M004**: independently blocked on producer-owned conventions; unaffected by M001d, so no status change.
- No other plan becomes dependency-ready as a result of M001d. Both requested follow-on states (Egress/Eggpack writability) are recorded; no further eligible-plan implementation is in scope for this batch.

## Registry updates

- Archive Extraction M001d: ready → closed (hosted run `36335233644` green; supersedes `36334772510`).
- Consumer Adoption M006 Egress: blocked → writable, plan unwritten (M001d gate satisfied).
- Eggpack Interoperability M002: blocked → writable, plan unwritten (M001d gate satisfied).
- Archive extraction roadmap: M001/M001a historical; M001b closed; M001c write half implemented with Section 14 stop closed by M001d; M001d closed; M002/M003 archive-side ready-to-author.
- Top metadata: current head green (`36335233644` all lanes); M001d closed; Egress/Eggpack writable.
- Execution graph, subsystem table, current-state, and next-handoff rows reconciled to the closed-M001d / writable-consumer graph.
