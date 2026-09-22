# Changelog

## Unreleased

- Added the Rust 1.89 `eggup-core` workspace foundation and deterministic test
  fixture support.
- Documented the transport-neutral ownership boundary. No production updater
  behavior is included yet.
- Added validated private staging, synchronous commit/rollback receipts, native
  SHA-256 integrity checks, and bounded candidate validation phases.
- M005 corrective (breaking pre-1.0): canonical `Absent | Owned | Foreign |
  Unknown` ownership with consumer verifier and absent-create policy; no
  automatic destination-parent creation; staged-digest revalidation under lock;
  removed unenforced authenticity API; renamed cleanup disposition to
  `CleanupDisposition`; structured failure phase/category/member reports with
  real recovery paths; owner-private stage/backup/lock permissions with
  collision-resistant names and ownership-checked cleanup; truthful
  fail-closed lock inspection with no automatic stale removal; checksum-only
  docs.
