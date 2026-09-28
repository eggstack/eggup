# Changelog

## 0.1.2 — 2026-09-28

First crates.io publication of `eggup-archive`. Integrity is SHA-256
checksum evidence only; no authenticity or signature claims.

- Bounded allowlisted extraction for verified local tar.gz and zip archives.
- Handle-backed source handoff (M001d): `PersistedExtraction::into_bound_sources()`
  converts each extracted member into a `BoundMember` owning its already-open
  readable object (created read + write via the same atomic handle-relative
  create-new/no-follow operation, rewound to byte zero at handoff), and
  `BoundExtraction::into_members()` splits staging sources from deferred
  `DeferredCleanup`. The matching `eggup-core` seam (`BoundSources` +
  `InstallPlan::prepare_with_bound_sources`) stages each object from byte zero
  with no pathname or member-name lookup after the handoff boundary, so a
  member-entry replacement or root rename cannot redirect staged bytes and the
  foreign replacement stays untouched. The recorded `ExtractedMember::path()`
  is advisory diagnostics only. Member handles are single-owner and move-only
  (no assumed-independent clones); supported order is stage, then
  close/consume handles, then handle-authorized cleanup, which empties only
  the owned tree with the usual `CleanupFailed` residue. A failed bound stage
  copy never reopens the recorded path as fallback, and leftover bound handles
  for unknown members fail closed. The ordinary path-source
  `ArtifactMember::new` API and semantics are preserved for non-archive
  callers. Deterministic tar/zip rename/replacement races (pre/between/post
  writes plus post-handoff member-entry and root replacement), non-zero-cursor
  staging, Unix `0600`, and exact size/digest evidence are covered by tests.
- Handle-relative member materialization (M001c write half): tar and zip
  declared-member files are created through the retained extraction-root
  handle (`fs_at::OpenOptions` write + create-new + no-follow via `open_at`,
  `0600` on Unix) instead of `root.join(output_name)` + pathname creation.
  Both format handlers share one `ExtractionScope` authority object; the
  returned member `File` is the authoritative streaming/hashing target.
  Deterministic rename/replacement races prove foreign directories,
  symlinks, and reparse points stay untouched.
- Handle-bound cleanup corrective (M001b): replace pathname-authorized
  recursive cleanup with retained directory authority via `fs_at 0.2.1`
  (default features disabled). The extraction root is created with
  `mkdir_at` from the parent handle and its `File` handle is carried through
  `DirectoryGuard` into `PersistedExtraction`; content deletion uses only
  handle-relative `read_dir`/`open_dir_at`/`unlink_at`/`rmdir_at` and never
  recursively traverses the original pathname. This removes the stable
  Windows dependency on nightly `MetadataExt::file_index()` and closes the
  M001a check-then-`remove_dir_all` time-of-check/time-of-use window. No
  portable object-bound root unlink exists, so explicit cleanup empties only
  the owned tree and returns `CleanupFailed` with the now-empty directory as
  residue; drop best-effort empties the same way and never falls back to
  `fs::remove_dir_all(path)`. Extraction failure preserves its original
  category and attaches the empty-residue path. `ExtractedArchive` and
  `PersistedExtraction` are now `Send` but not `Sync` due to the retained
  handle.
- Package metadata: explicit `homepage` and `documentation` fields,
  broadened keywords (`archive`, `extraction`, `updater`, `tar`, `zip`),
  and a description that records the absence of authenticity or signature
  support. The dev-dependency on `eggup-core` is path-only (no version
  constraint) so `cargo package`/`cargo publish --dry-run` for
  `eggup-archive` can verify against the in-tree `eggup-core` before
  `eggup-core 0.1.2` is on crates.io.
- No first-party unsafe; Rust 1.89 holds.
