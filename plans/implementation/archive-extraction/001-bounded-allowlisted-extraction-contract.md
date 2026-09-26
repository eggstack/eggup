# Archive Extraction Milestone 001 — Bounded Allowlisted Extraction Contract

Status: implemented

Repository baseline: `ee1476ef2a9e0d5569d6dc2469e780a4435cc426`

Source roadmap:

- `plans/subsystems/archive-extraction-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md#phase-10--eggress-archivebundle-convergence`

Applicable ADRs:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: invariant/infrastructure

## 1. Objective

Create the first reusable Eggup local archive-extraction boundary: an optional Rust crate/layer that accepts an already verified local archive plus an explicit member plan and produces owner-private, verified regular files suitable for later `ArtifactSet` construction.

M001 must support the formats required by the concrete Phase 10 consumer evidence:

- tar.gz for Linux/macOS Egress releases;
- zip for Windows Egress releases.

It must not mutate the live installation.

## 2. Why this milestone is ready

All hard prerequisites are closed:

- verified-update-core M007 is closed;
- acquisition M006 is closed;
- CodeGG has qualified the "verify archive -> strict extract -> Eggup multi-artifact commit" architecture with extraction still consumer-owned;
- Egress supplies a real two-binary tar.gz/zip consumer pattern;
- `eggup-eggpack` already exposes exact archive/member evidence but intentionally stops at `ArchiveExtractionRequired`.

ADR-0005 now fixes the generic safety contract.

No producer-side Eggpack work is required to implement M001.

## 3. Current implementation evidence

Eggup has no archive crate in the workspace.

`eggup-core` accepts ordinary local source files through `ArtifactMember` and already provides private staging, integrity verification, candidate validation, locked revalidation, multi-member commit, rollback, and recovery.

`eggup-eggpack::ManifestProjection::Archive` preserves:

- archive artifact name;
- exact archive size;
- archive SHA-256;
- each member source path;
- install name;
- exact member size;
- member SHA-256.

Egress documents a release unit containing exactly `eggress` and `pproxy`, delivered as tar.gz on Linux/macOS and zip on Windows, with archive checksum and staged binary-version verification before replacement.

External library research confirms that generic helpers need additional Eggup constraints: tar's safer extraction API may overwrite an existing destination, and zip convenience extraction may overwrite and materialize links. M001 must iterate and enforce its own declaration/no-clobber contract.

## 4. Invariants that must not regress

- `eggup-core` gains no archive-format dependency.
- Extraction accepts local bytes only; it performs no network I/O.
- Caller release/version/origin policy remains external.
- The archive artifact must already have passed caller-required integrity verification.
- No undeclared entry is installed.
- Only regular files are materialized in M001.
- No extraction path can escape the owned root.
- Existing/raced-in extraction paths are never overwritten.
- Every operation has finite entry, path, per-member, aggregate-byte, and diagnostic bounds.
- Extracted bytes are verified against exact expected size/digest when supplied.
- Failure cleans only operation-owned files.
- Successful extraction still grants no live destination ownership.
- No external `tar`, `unzip`, shell, sudo, or platform package utility is invoked.

## 5. Scope

### In scope

- add an optional archive crate, tentatively `eggup-archive`;
- archive/member plan types with validated finite budgets;
- explicit `ArchiveFormat::{TarGz, Zip}` or equivalent;
- private extraction-root lifecycle;
- normalized portable relative-path validation;
- exact declared-member matching;
- regular-file-only enforcement;
- bounded tar.gz iteration;
- bounded zip iteration;
- streamed write + SHA-256 + exact-size validation;
- per-member and total decompressed byte enforcement;
- entry-count enforcement;
- no-clobber exclusive file creation;
- cleanup on all ordinary failure/cancellation paths;
- deterministic fixture builders/tests;
- output evidence suitable for later `ArtifactSet` construction;
- dependency/feature/footprint measurement.

### Explicitly out of scope

- Egress repository changes;
- `eggup-eggpack` integration;
- live install mutation;
- service lifecycle;
- producer archive generation;
- symlink/hardlink/device extraction;
- arbitrary file metadata restoration;
- authenticity/signature policy;
- format autodetection;
- rar/7z/deb/rpm or other formats;
- async API unless required by a concrete implementation constraint;
- generic extension/plugin framework.

## 6. Required production changes

### 6.1 Crate boundary

Add a leaf crate that depends on the smallest archive/hash/path support required and may depend on `eggup-core` only for stable domain/evidence types if that materially reduces duplication.

Prefer keeping format dependencies entirely out of `eggup-core`.

If tar.gz and zip dependency cost is materially different, feature-gate formats so consumers can build only what they need. Do not make `--all-features` silently change safety semantics.

### 6.2 Domain contract

Provide validated types equivalent to:

```text
ArchivePlan {
  archive_path,
  format,
  members,
  max_entries,
  max_total_uncompressed_bytes,
}

ArchiveMember {
  source_path,
  member_id/install identity,
  exact_size or finite max_size,
  expected_sha256 when required,
}

ExtractedMember {
  member identity,
  absolute local path,
  bytes,
  sha256,
}
```

Exact names may differ.

All public constructors must validate bounds. Do not expose public fields that permit invalid zero/unbounded budgets without boundary revalidation.

### 6.3 Path contract

Normalize/validate declared and observed archive names before comparing them.

Reject at minimum:

- empty path;
- absolute/rooted path;
- `..`;
- platform prefix/drive;
- NUL/control ambiguity;
- overlong path;
- duplicate normalized path;
- duplicate output member;
- observed path not exactly matching the declared normalized member.

Do not "sanitize" a dangerous name into a different safe name. Reject it.

### 6.4 Entry-type contract

Accept regular-file entries only for declared members.

Reject a declared member if the archive entry is a link or special file.

Undeclared entries may be scanned/skipped only within global entry/path/decompression-safe parsing limits and may never be materialized.

### 6.5 Private extraction

Create one exclusive owner-private operation root in caller-selected/private temporary storage.

For each declared member:

- create parent directories only under the owned root;
- ensure each parent remains a real directory;
- create the output file with no-clobber semantics;
- stream bytes into the already-open owned file;
- enforce member and aggregate byte budgets while writing;
- hash while writing;
- flush/sync according to the documented extraction durability contract;
- validate exact size and expected digest before marking the member complete.

Do not close an exclusive file and later ask an external/archive helper to reopen the same pathname.

### 6.6 Completion

Success requires every declared member exactly once.

Return extracted-member evidence in deterministic declaration order.

The caller owns the returned extraction root until it has constructed/prepared the subsequent Eggup transaction. Provide an explicit cleanup/guard ownership model so dropping an unused result cannot delete files after ownership has intentionally transferred.

## 7. Ordered work packages

1. Add crate skeleton, dependency review, and API-domain tests.
2. Implement finite budget/path/member validation.
3. Implement private extraction root and exclusive file creation.
4. Implement shared streamed copy/hash/budget accounting.
5. Implement tar.gz entry iteration with explicit regular-file checks.
6. Implement zip entry iteration with enclosed/normalized path checks and explicit regular-file checks.
7. Add malicious fixture matrix and failure cleanup tests.
8. Add output-to-`ArtifactSet` example without changing `eggup-core`.
9. Add cross-platform tests, including Windows runtime tests for zip.
10. Measure dependency tree and minimal/full feature footprint.
11. Update architecture/docs/roadmap/registry and write closure evidence.

## 8. Failure, cancellation, restart, and contention semantics

Extraction failure is pre-transaction and must not alter the live installation.

A failed extraction returns a typed phase/category and removes only its owned incomplete extraction root where safe.

If cleanup fails, report the residue path as bounded recovery evidence; do not claim the live installation requires recovery.

M001 need not provide resumable extraction. Retrying starts a new exclusive extraction root.

If cancellation is exposed, it must be cooperative but checked during archive iteration and streamed copy, not only between members.

Concurrent extractions use distinct exclusive roots and do not contend on the live Eggup mutation lock. The later transaction retains its existing lock semantics.

## 9. Compatibility and migration

No existing Eggup public API needs to break.

No existing consumer is migrated in M001.

`eggup-eggpack` remains unchanged and continues returning `ArchiveExtractionRequired`.

Egress remains unchanged until M001 closes and its adoption plan is separately authored.

## 10. Required tests

At minimum:

- valid tar.gz two-member extraction;
- valid zip two-member extraction;
- deterministic multi-member ordering;
- missing declared member;
- duplicate declared member;
- duplicate archive member;
- `../` path;
- absolute Unix path;
- Windows drive/prefix path;
- nested traversal alias;
- overlong path;
- symlink member;
- hardlink member;
- device/FIFO/special member where format permits;
- existing/raced-in output file preserved;
- exact-size mismatch;
- SHA-256 mismatch;
- per-member decompression overflow;
- aggregate decompression overflow;
- entry-count overflow;
- truncated/corrupt tar.gz;
- corrupt zip;
- undeclared extra entry not materialized;
- extraction failure cleans owned partials;
- cleanup failure produces residue evidence without live-install recovery claim;
- output can be consumed by an ordinary `ArtifactSet`/prepare fixture;
- tar/zip dependencies do not appear in `eggup-core`.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --workspace --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-archive --locked
./scripts/check-local.sh
git diff --check
```

Hosted evidence must include Linux stable/MSRV, macOS tests, and Windows runtime tests for the portable archive suite. Do not substitute Windows compile-only evidence for zip runtime claims.

## 12. Documentation updates

Update:

- workspace/root README crate inventory;
- `architecture/overview.md`;
- add archive architecture deep dive;
- archive crate README/rustdoc;
- `plans/subsystems/archive-extraction-roadmap.md`;
- consumer-adoption blocker text only after M001 closure;
- Eggpack interoperability blocker text only after M001 closure;
- `plans/registry.md`;
- closure record.

## 13. Acceptance criteria

M001 closes only when:

- tar.gz and zip declared regular members extract successfully under finite budgets;
- traversal/link/special/no-clobber/zip-bomb classes fail closed;
- all declared members are exact-size/digest verified before success;
- the live installation is untouched on every extraction failure;
- `eggup-core` remains archive-format independent;
- Windows has runtime zip evidence;
- dependency/footprint impact is recorded;
- no high- or medium-severity extraction issue remains;
- Egress M002 and Eggpack archive handoff become dependency-ready only after closure.

## 14. Stop conditions

Stop and write a corrective/ADR rather than silently widening scope if:

- supporting Egress requires symlink/hardlink installation;
- archive paths require non-UTF-8 semantics that cannot fit ADR-0005;
- a chosen archive crate cannot prevent overwrite/link behavior without unsafe workarounds;
- `eggup-core` would need archive dependencies;
- format handling requires external executables;
- decompression cannot be bounded during streaming;
- producer naming/package policy starts entering the extraction layer.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- exact archive dependency versions/features;
- dependency trees for `eggup-core` and `eggup-archive`;
- format fixture inventory;
- requirement-to-negative-test matrix;
- Linux/macOS/Windows results;
- per-member/aggregate bound evidence;
- cleanup/residue behavior;
- example `ArtifactSet` handoff;
- before/after footprint measurements;
- unresolved findings by severity.

## 16. Handoff notes

Implement the smallest declaration-driven extractor needed by Egress and Eggpack archive evidence. Do not optimize for general archive compatibility.

The strongest design property is that format libraries only parse/read entries; Eggup owns path authorization, exclusive output creation, streaming bounds, hashing, and cleanup.
