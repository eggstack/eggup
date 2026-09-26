# eggup-archive

Optional local extraction for already verified tar.gz and zip files. Callers
declare the exact archive members and output names, and provide finite entry,
path, per-member, aggregate decompressed-byte, and archive-file bounds.

The extractor writes only declared regular files into one exclusive private
directory. It uses exclusive file creation, never overwrites, hashes while
streaming, checks expected size/digest facts when supplied, and removes its
owned incomplete directory after failure. Dropping an unused extraction result
cleans it; `persist()` transfers cleanup responsibility (and the captured
identity evidence) to the caller.

Cleanup is authorized by retained filesystem identity rather than a bare
pathname: the captured `(dev, ino)` on Unix or `file_index` on Windows is
revalidated before any recursive deletion. A foreign directory or symlink
that occupies the original pathname after rename/replace cannot be
recursively deleted; cleanup fails closed with residue evidence. Drop
cleanup uses the same identity-checked primitive and never falls back to
`fs::remove_dir_all(path)`.

This crate performs no network access, authenticity policy, external process
invocation, live installation mutation, metadata restoration, or service
lifecycle work. Extracted files are inputs for a later caller-owned
`ArtifactSet`; extraction does not provide destination ownership. Flushing makes
completed files visible to later reads, but crash durability is not promised.

`eggup-core` remains independent of archive formats.
