# eggup-archive

Optional local extraction for already verified tar.gz and zip files. Callers
declare the exact archive members and output names, and provide finite entry,
path, per-member, aggregate decompressed-byte, and archive-file bounds.

The extractor writes only declared regular files into one exclusive private
directory. It uses exclusive file creation, never overwrites, hashes while
streaming, checks expected size/digest facts when supplied, and removes its
owned incomplete directory after failure. Dropping an unused extraction result
cleans it; `persist()` transfers cleanup responsibility to the caller.

This crate performs no network access, authenticity policy, external process
invocation, live installation mutation, metadata restoration, or service
lifecycle work. Extracted files are inputs for a later caller-owned
`ArtifactSet`; extraction does not provide destination ownership. Flushing makes
completed files visible to later reads, but crash durability is not promised.

`eggup-core` remains independent of archive formats.
