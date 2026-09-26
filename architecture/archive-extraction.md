# Archive Extraction — Deep Dive

`eggup-archive` is an optional leaf crate for extracting explicitly declared
regular files from already verified local tar.gz and zip archives. It owns
member path authorization, decompression limits, private output files, streamed
hashing, and cleanup of the extraction root. The caller still owns acquisition,
archive integrity/authenticity policy, install naming, permissions, service
lifecycle, and the later `ArtifactSet`/transaction.

## Operation boundary

```text
caller verifies local archive and chooses policy
                |
                v
ArchivePlan { format, declared members, finite limits }
                |
                v
parse entries -> compare exact normalized paths -> regular files only
                |
                v
private root + handle-relative create_new 0600 files + streaming bounds + SHA-256
                |
                v
ExtractedArchive evidence -> caller prepares ArtifactSet/core transaction
```

The archive crate has no Eggpack, acquisition, service-manager, or network
dependency. Its production dependencies are `tar`, `flate2`, `zip`,
`fs_at`, and `sha2`; tar xattr and zip's optional compression/crypto features
are disabled except deflate decoding, and `fs_at` optional `log` /
`workaround-procmon` features are disabled. `eggup-core` does not depend on
this crate or any archive-format package.

## Path and entry contract

- Archive names must be UTF-8, relative, slash-separated paths. Empty, rooted,
  drive-prefixed, backslash, control, dot, parent, repeated-separator, and
  overlong paths fail closed; no dangerous path is rewritten into a safe one.
- Output names are single portable ASCII filenames. Windows device names,
  separators, control characters, trailing dot/space, and reserved punctuation
  are rejected. Duplicate source names and case-insensitive output aliases are
  rejected by plan construction.
- Only declared tar regular entries (type `0` or NUL) and regular zip entries
  can be materialized. Links, directories, devices, FIFOs, sparse/unknown types,
  and other special entries are never written. Undeclared regular entries are
  drained under the same per-member and aggregate decompressed-byte limits.
- Every entry path is bounded and duplicate normalized archive paths are
  rejected. Success requires each declared path exactly once.

## Bounds and evidence

`ArchiveLimits` bounds compressed archive size, entry count, path bytes,
individual decompressed member bytes, and total decompressed bytes. All limits
must be positive and finite. The conservative defaults are 512 MiB compressed,
10,000 entries, 1 KiB path strings, 256 MiB per member, and 512 MiB total.
Consumers should choose limits from their own release contract rather than
treating defaults as release policy.

Data is copied in fixed 16 KiB chunks. The extractor counts bytes before each
write, updates SHA-256 in the same loop, flushes each completed output, then
checks exact size/digest facts when supplied. A success result contains paths,
byte counts, and hashes in caller declaration order. Flush supports immediate
subsequent reads; extraction makes no crash-durability or restart guarantee.

## Ownership, cleanup, and handoff

The selected output parent must already be a real directory. Extraction creates
one exclusive sibling root (`mkdir_at` from the parent handle, `0700` on Unix)
and creates each declared member file relative to the retained root handle
(`fs_at` write + create-new + no-follow via `open_at`, `0600` on Unix; one
shared `ExtractionScope` authority object for tar and zip). It never
creates install-root parents or writes into the live installation. The root
`File` handle is retained through `DirectoryGuard` into `PersistedExtraction`;
recursive cleanup deletes only through that handle via `fs_at` `*at`
operations and never recursively traverses the original pathname. Because no
portable object-bound root unlink exists, explicit cleanup empties only the
owned tree and reports the now-empty directory as `CleanupFailed` residue;
drop best-effort empties the same way. Extraction failure preserves the
original error category and attaches the empty-residue path.

Known limit (M001c Section 14 stop, continued by M001d): the recorded member
pathname is handoff evidence only, not write authority. A mid-extraction root
rename leaves bytes in the handle-owned directory while the recorded path
dangles into the replacement, and no stable cross-platform proof rebinds it;
the handle-backed source handoff (M001d) must land before Egress M006 or
Eggpack M002 integrate.

`ExtractedArchive` cleans itself on drop, which is safe for unused results.
After the caller has prepared/copied the members into a later transaction, it
can call `persist()` to transfer cleanup responsibility and explicitly empty
the retained root when no longer needed. Neither state proves destination
ownership or grants permission to commit.

## Failure semantics

Malformed/truncated streams, unsafe/duplicate paths, non-regular declared
members, missing declarations, archive/entry/member/aggregate overages, exact
size mismatch, digest mismatch, and I/O errors fail before live mutation. The
caller can retry into a fresh exclusive root; extraction is not resumable.
Diagnostics expose bounded categories, not archive contents or arbitrary
upstream text. Raw operating-system errors are not returned.

See `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md` and
`plans/closure/archive-extraction/001-status.md` for the normative contract and
M001 qualification evidence.
