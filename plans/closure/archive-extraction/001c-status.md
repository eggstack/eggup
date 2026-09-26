# Archive Extraction M001c — Closure and Verification Record

Status: blocked — Section 14 stop; write-authority half implemented, path-handoff half continued by M001d

Source plan: `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md#M001c--handle-relative-member-materialization-corrective`

Reviewed repository baseline: `1388a02356dfa01d72c63c97c1baca09ff6004a1` (pre-implementation head; source tree was clean before implementation)

Implementation commit: `09c953fe1fe512717584bb3e5509e79da894b35a` — handle-relative member materialization via retained root handle; shared `ExtractionScope` authority object; `fs_at` write/create-new/no-follow `create_private_file_at` with Unix `0600`; deterministic rename/replacement race seam plus tar/zip/before/between/symlink/raced-output tests; no new dependency.

Hosted qualification: green — CI run `36257884083` on the docs-batch head (implementation `09c953f` plus this closure batch): Stable Linux (fmt/clippy/full workspace tests/docs), Rust 1.89 MSRV all-target check, macOS full workspace tests, and Windows archive/acquisition/curl/service portable tests plus workspace check all passed. This qualifies the write-authority half; the Section 14 stop outcome is unchanged (it follows from stable-API analysis, now with hosted confirmation that the write half introduces no platform regression).

## Executive finding

M001c is stopped under its Section 14, not closed. The two halves have different outcomes:

**Implemented (write authority).** No declared-member write is authorized by joining an output filename to the root pathname anymore. Tar and zip materialization receive the retained root handle through one shared `ExtractionScope` object; member files are created with `fs_at::OpenOptions` write + create-new + no-follow (Unix mode `0600`) via `open_at(root_handle, output_name)`, and the returned member `File` is the authoritative target for the entire copy/hash/flush/validation sequence. Deterministic synchronous-hook races prove root renames/replacements before or between member writes cannot redirect bytes into foreign directories, symlinks are never followed, raced existing/symlink outputs fail closed without clobber, and foreign state stays byte-for-byte untouched. No new dependency was added (`fs_at 0.2.1` sufficed) and all prior M001/M001a/M001b suites stay green locally.

**Stopped (successful-handoff truthfulness, Section 6.4).** Handle-relative writes alone cannot make `ExtractedMember::path()` truthful: after a root rename the bytes safely remain in the handle-owned (renamed) directory while the recorded `root.join(output_name)` pathname dangles into the foreign replacement. The plan requires either (A) proving the recorded paths still resolve to the owned objects with a safe cross-platform mechanism, or (B) replacing path authority with a stronger handoff — and explicitly forbids marker files, random filenames, timestamps, digests, and ordinary canonicalize as ownership evidence. Determination per work package 8:

- Unix could compare `(dev, ino)` from stable `MetadataExt`, but Windows stable offers no file identity at all: `MetadataExt::file_index`, `volume_serial_number`, `number_of_links`, and `change_time` are nightly-only behind `windows_by_handle` (#63010; confirmed on current stable docs). M001b already removed the nightly `file_index` path to restore stable-Windows compilation.
- The only portable identity crate (`same-file`) explicitly documents false-positive equality for distinct files on supported filesystems (ReFS 128-bit IDs truncated to 64-bit `nFileIndex`; win.rs source comments), which is exactly the plan's Section 14 stop trigger for an identity mechanism used as ownership authority.
- A handle-path query (e.g. `GetFinalPathNameByHandle`) would need first-party unsafe/FFI or a new native dependency, both separately listed as stop triggers, and would still leave the post-handoff TOCTOU (consumer re-opens the pathname in core staging later) unaddressed.
- End-to-end safety therefore needs the Option B stronger contract: a handle-backed source from extraction into core staging that never re-resolves the recorded pathname.

Per Sections 9 and 14, M001c does not broaden `eggup-core` silently and does not weaken ADR-0005 to avoid the API decision. The required public handoff/API follow-up is authored as `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md` (ready for handoff). M001c remains blocked on M001d; Egress M006 and Eggpack M002 stay blocked (now via M001d).

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| No declared-member write authorized by `root.join(output_name)` | production `grep`: zero `root.join` outside `member_path()` handoff-evidence constructor; tar/zip call only `scope.create_member()` → `create_private_file_at(root_handle, name)` | passed |
| Tar and zip writes relative to retained root handle | `ExtractionScope { root_path, root_handle, write_seq, hook }` borrowed by both `extract_tar_gz` and `extract_zip`; no duplicated authority logic | passed |
| Member creation create-new/no-follow, owner-private | `create_private_file_at`: `write(Write)` + `create_new(true)` + `follow(false)` + `open_at`; Unix `mode(0o600)` via `fs_at::os::unix::OpenOptionsExt` | passed |
| Exact `fs_at` options recorded | see above; `ensure_rootless` still rejects rooted inputs; validated single-component `output_name` only | passed |
| Root replacement before first member cannot redirect (tar) | `tar_handle_relative_write_survives_root_rename_before_first_member`: hook renames + installs 3-file foreign dir at seq 0; owned `moved/app` holds `owned-bytes`, foreign holds only `keep` files, `root/app` absent | passed |
| Same race (zip) | `zip_handle_relative_write_survives_root_rename_before_first_member` mirrors tar | passed |
| Replacement between two members | `two_member_replacement_between_writes_stays_in_owned_root`: hook renames at seq 1; `moved/one` + `moved/two` hold `first`/`second`, foreign holds exactly one attacker file | passed |
| Symlink/reparse replacement not followed | `root_symlink_replacement_is_not_followed_by_materialization` (Unix symlink) and `windows_reparse_replacement_is_not_followed_by_materialization` (Windows best-effort symlink_dir with graceful fallback); link target keeps exactly its `keep` entry | passed Unix locally; Windows variant compiles, hosted run will execute it |
| Raced existing output fails, no clobber | `raced_existing_output_file_in_owned_root_fails_without_clobber`: hook writes `existing` at seq 0; extraction errs `Io`; residue never carries new bytes at raced name | passed |
| Raced output symlink fails, target untouched | `raced_output_symlink_in_owned_root_fails_and_target_untouched` (Unix): hook symlinks `app`→target; errs `Io`; target still `keep` | passed |
| Unix member mode `0600` | `extraction_root_and_files_are_owner_private` still green (handle creation preserves mode); root `0700` unchanged | passed |
| Size/digest evidence still exact | `exact_size_and_sha256_mismatches_fail`, `valid_*` digest assertions green | passed |
| Prior M001/M001a/M001b suites green | full `eggup-archive` suite 35/35 locally (29 pre-existing + 6 new); workspace all-target green | passed |
| `eggup-core` archive-free | `cargo tree -p eggup-core` shows `sha2` only | passed |
| No new dependency | `cargo tree -p eggup-archive` unchanged (`fs_at 0.2.1` only); `Cargo.toml`/`Cargo.lock` untouched | passed |
| Rust 1.89 supported | `cargo +1.89.0 check --workspace --all-targets --locked` passed locally | passed |
| Successful handoff provably bound (6.4A) | NOT satisfiable with stable cross-platform APIs — see stop analysis; recorded paths go stale on rename by construction (new race tests assert owned-bytes-via-moved-location while `root/app` is absent) | stopped → continued by M001d |
| Binding-loss produces failure, never stale success (6.4/6.5) | NOT implemented — without a sound binding proof, detection would itself be unsound; M001d owns fail-closed bound-source semantics | stopped → continued by M001d |
| Hosted Linux/MSRV/macOS/Windows matrix green | pending — local evidence green; hosted run to be recorded as supplement | pending |

## Production implementation evidence

- `crates/eggup-archive/src/lib.rs::create_private_file_at` — new handle-relative exclusive creator (`fs_at` write/create-new/no-follow, Unix `0600`, `open_at(root, name)`); the old pathname-authorized `create_private_file(path)` (`std OpenOptions::create_new(path)`) is deleted.
- `crates/eggup-archive/src/lib.rs::ExtractionScope` — one private root-authority object (`root_path`, `root_handle`, `write_seq`, `hook`) with `create_member()` / `member_path()` / `run_hook()`; tar and zip share it (no duplicated authority).
- `crates/eggup-archive/src/lib.rs::extract_with_hook_inner` — threads `&File` into both format handlers; production `extract` passes `None`.
- `crates/eggup-archive/src/lib.rs::extract_with_hook` (`#[cfg(test)]`) — mirrors `extract` with the deterministic hook for race tests.
- `crates/eggup-archive/src/lib.rs::extract_tar_gz` / `extract_zip` — hook runs synchronously before each declared-member creation; writes go to the handle-returned `File` for copy/hash/flush/validate; `member_path()` output is recorded as handoff evidence only with an explicit stale-path comment.
- Tests added (6 new; 35/35 green on Unix, Windows variant compiles for hosted lane):
  - tar/zip pre-first-member rename races, two-member between-writes race, Unix symlink race, Windows reparse race, owned-root raced-file race, owned-root raced-symlink race (Unix).
- Tests updated (1): `exclusive_output_creation_preserves_an_existing_file` now exercises `create_private_file_at` via a real private root.
- Dependency/footprint delta: zero — no `Cargo.toml`/`Cargo.lock` change; `cargo tree` identical before/after.

## Exact commands and results

Local Darwin arm64, Rust 1.89.0 (implementation commit `09c953f`):

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test -p eggup-archive --all-targets --all-features --locked  passed (35/35)
cargo test --workspace --all-targets --all-features --locked    passed (acquisition 36, archive 35, core 43, curl 22)
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked          passed
cargo tree -p eggup-archive --locked                             passed; unchanged (fs_at 0.2.1 only)
cargo tree -p eggup-core --locked                                passed; sha2 only
git diff --check                                                passed
```

Hosted CI run `36257884083` (docs-batch head containing implementation `09c953f`):

- Stable Linux: fmt/clippy/full workspace tests/docs passed.
- MSRV Linux: Rust 1.89 workspace all-target check passed.
- macOS: full workspace tests passed.
- Windows: archive/acquisition/curl/service portable tests plus workspace all-target check passed.

The Section 14 stop stands with hosted confirmation of the write half.

## Invariant review

- Writes operate only on the retained directory object; the replacement pathname is never opened for writing (proven by construction + races).
- Foreign directories (3-file, 1-file), symlinks, and Windows reparse points installed before or between writes are never written through and stay byte/entry-identical.
- Drop/explicit cleanup authority is untouched (same handle-relative `empty_dir_contents`, empty-residue fallback).
- All M001 traversal/link/special-file/no-clobber/decompression/digest invariants unchanged; `eggup-core` format-independent; no first-party `unsafe` (workspace deny clean); MSRV 1.89 preserved.
- Handoff invariant is explicitly NOT met: success can still record a stale pathname. This is the documented stop, not a silent regression — the new race tests pin the stale-path shape so M001d has a failing-to-passing target.

## Failure and recovery review

| Failure | Result |
|---|---|
| Root renamed/replaced before or between writes | bytes stay in owned (renamed) tree via handle; foreign untouched; extraction currently still reports success with a stale recorded path (documented gap owned by M001d, which will make binding loss fail closed) |
| Concurrent file at output name inside owned root | `create_new` fails closed (`Io`); no truncation/reopen retry; failure residue path attached; only owned tree cleaned via handle |
| Symlink/reparse at output name inside owned root | `follow(false)` + `create_new` refuses; target never opened for write |
| Extraction inner failure (bounds, digest, malformed) | original category preserved with empty-residue path (M001b semantics unchanged) |
| Cleanup after failure | handle-only emptying; foreign never removed; no indefinite loops |

## Compatibility and migration review

No public API change in M001c: `ArchivePlan`/`ArchiveMember` constructors, `ExtractedMember` fields, error categories, and `Send`-but-not-`Sync` handle-carrying guards are unchanged. The in-tree consumer test (`extracted_file_can_be_prepared_as_an_ordinary_core_artifact`) is unmodified and green. The required public handoff change is deliberately deferred to M001d (Section 9), where its API diff and migration will be recorded — M001c does not silently change `ExtractedMember::path()` semantics.

## Security review

The M001c write-redirection TOCTOU (`root.join(output_name)` + pathname `create_new` while a foreign directory can be installed at the old root pathname mid-extraction) is eliminated for the write path: every member byte flows through `open_at(root_handle, output_name)` with atomic create-new and no-follow where the filesystem supports it. SHA-256 remains integrity evidence only; no authenticity claim is added. The remaining medium finding is the handoff: a stale recorded pathname could direct a later path-based consumer into foreign state. It is recorded as an open medium authority finding owned by M001d (not hidden), with deterministic tests pinning the exact stale shape. No other medium-or-higher finding is introduced.

## Documentation and operations evidence

- `crates/eggup-archive/src/lib.rs` documents handle authority, exact `fs_at` options, the stale-handoff stop pointer, and the shared-scope design.
- Archive/user docs updates (README, architecture, changelogs, roadmaps, registry) land with the closure batch; see registry updates below.
- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md` authors the required follow-up (ready for handoff).
- M001c implementation plan status moves to `implemented` (work executed; closure is a Section 14 stop, not a close).

## Unresolved findings

- Medium: successful `ExtractedArchive` can record a stale/foreign `ExtractedMember::path()` after a mid-extraction root rename/replacement, and a later path-based staging open could follow it. Owner: Archive M001d (handle-backed source handoff). Not downgraded; Egress M006 and Eggpack M002 stay blocked.
- Informational: none pending — hosted write-authority matrix is green (`36257884083`).
- Informational: Windows reparse race coverage stays best-effort (privilege-gated symlink creation with graceful fallback), same posture as M001b.

## Roadmap disposition and downstream unblock

Archive M001c moves from ready to blocked (Section 14 stop; write half implemented, handoff continued by M001d). This unblocks exactly one plan:

- **Archive Extraction M001d handle-backed source handoff**: blocked → ready for handoff (new plan authored here).

It does NOT unblock (all remain blocked on M001d closure + green hosted qualification):

- **Consumer Adoption M006 Egress**: stays blocked (was blocked on M001c; now blocked on M001d).
- **Eggpack Interoperability M002 archive handoff**: stays blocked (was blocked on M001c; now blocked on M001d).

Archive M001/M001a/M001b, Acquisition M008, and all other closed milestones are unaffected. No Gregg migration is authorized. No Eggup installer-generator work is authorized.

## Registry updates

- Archive M001c: ready → blocked (Section 14 stop; implementation `09c953f`; continued by M001d).
- Archive M001d: — → ready (new handle-backed handoff plan).
- Consumer Adoption M006 Egress: blocked on M001c → blocked on M001d.
- Eggpack Interoperability M002: blocked on M001c → blocked on M001d.
- Planning/closure hygiene C005: ready (reconciles this updated graph; see its closure).
