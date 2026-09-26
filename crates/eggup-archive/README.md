# eggup-archive

Optional local extraction for already verified tar.gz and zip files. Callers
declare the exact archive members and output names, and provide finite entry,
path, per-member, aggregate decompressed-byte, and archive-file bounds.

The extractor writes only declared regular files into one exclusive private
directory. It uses exclusive file creation, never overwrites, hashes while
streaming, checks expected size/digest facts when supplied, and empties its
owned incomplete directory after failure. Dropping an unused extraction result
best-effort empties it; `persist()` transfers cleanup responsibility (and the
retained directory handle) to the caller.

Member materialization is authorized by the same retained directory handle,
not by the recorded root pathname: tar and zip handlers share one
`ExtractionScope` authority object and create each declared file with
`fs_at::OpenOptions` (write + create-new + no-follow, `0600` on Unix) via
`open_at(root_handle, output_name)`. The returned member `File` is the
authoritative target for streaming, hashing, flushing, and validation. A root
rename/replacement before or between member writes therefore cannot redirect
bytes into a foreign directory, symlink, or reparse point.

Cleanup is authorized by a retained directory handle, not by a pathname
check: the extraction root is created via `mkdir_at` from the parent handle
(mode `0700` on Unix) and the returned `File` handle is carried through
`DirectoryGuard` into `PersistedExtraction`. Recursive content deletion uses
only `fs_at` handle-relative operations (`read_dir`, `open_dir_at`,
`unlink_at`, `rmdir_at`); symlinks and reparse points are unlinked, never
followed. No recursive operation traverses the replacement pathname, so a
foreign directory, file, symlink, or reparse point installed after any handle
operation cannot be recursively deleted. No object-bound root unlink exists
on all supported platforms, so explicit cleanup empties only the owned tree
and reports the now-empty directory as `CleanupFailed` residue instead of
recursively touching the pathname; drop best-effort empties the same way and
never falls back to `fs::remove_dir_all(path)`.

`ExtractedArchive` and `PersistedExtraction` retain a `std::fs::File` handle
and are therefore `Send` but not `Sync`.

Known limit (Archive M001c Section 14 stop, continued by M001d): the recorded
`ExtractedMember::path()` (`root.join(output_name)`) is handoff evidence
only. After a mid-extraction root rename the bytes safely remain in the
handle-owned directory while the recorded pathname dangles into the foreign
replacement, and no stable cross-platform identity proof exists to rebind it
(Windows stable has no file index; portable identity documents false-positive
equality). A handle-backed source handoff is required before archive
consumers integrate; Egress M006 and Eggpack M002 stay blocked on M001d.

This crate performs no network access, authenticity policy, external process
invocation, live installation mutation, metadata restoration, or service
lifecycle work. Extracted files are inputs for a later caller-owned
`ArtifactSet`; extraction does not provide destination ownership. Flushing makes
completed files visible to later reads, but crash durability is not promised.

`eggup-core` remains independent of archive formats.
