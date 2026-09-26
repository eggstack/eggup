# Archive Extraction Roadmap

Status: active; M001/M001a historical; M001b handle-bound cleanup + Windows portability corrective ready

Long-term references:

- `plans/000-long-term-specification.md#6-multi-artifact-installation-is-fundamental`
- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`
- `plans/002-long-term-roadmap.md#phase-10--eggress-archivebundle-convergence`

Applicable ADRs:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

## 1. Purpose and ownership boundary

This subsystem owns safe local materialization of explicitly declared regular-file members from an already verified archive into an Eggup-owned private extraction root.

It does not own archive construction, release naming, release discovery, artifact origin, authenticity policy, live installation mutation, or service lifecycle.

## 2. Work classification

### Invariants

- `eggup-core` remains archive-format independent;
- archive bytes are integrity-verified before extraction is trusted as deployment input;
- only caller-declared members may be materialized;
- extraction never writes into the live installation root;
- traversal, link, special-file, alias, and overwrite cases fail closed;
- per-member, aggregate decompressed-size, entry-count, path, and diagnostic bounds are finite;
- extracted member size/digest evidence is verified before success;
- cleanup removes only operation-owned extraction state.

### Capabilities

- safely extract an allowlisted regular file from tar.gz;
- safely extract an allowlisted regular file from zip;
- materialize multiple declared members under one bounded extraction operation;
- return exact local paths/evidence suitable for `ArtifactSet` construction;
- support the two-binary Egress archive model;
- later consume Eggpack `ManifestProjection::Archive` member evidence.

### Infrastructure

- optional archive crate/layer;
- archive plan/member domain types;
- private extraction root;
- bounded streaming copy/hash helpers;
- format-specific readers;
- deterministic malicious archive fixtures.

### Polish

- extraction progress events;
- richer archive diagnostics;
- additional formats only with consumer evidence.

## 3. Non-goals

- general-purpose archive restoration;
- preservation of archive ownership/xattrs/devices/links;
- producer package construction;
- live destination replacement;
- service restart;
- release fallback;
- automatic format guessing from untrusted filenames.

## 4. Current evidence

CodeGG M005 is closed using CodeGG-owned strict extraction of one verified archive before Eggup multi-artifact commit. That proves the transaction seam but intentionally leaves extraction outside Eggup.

Egress currently publishes one version-aligned archive containing `eggress` and `pproxy`: tar.gz on Linux/macOS and zip on Windows. Its updater verifies the archive SHA-256, extracts both staged executables, verifies both versions, then replaces the pair as one release unit. This is the concrete Phase 10 consumer pattern.

Eggpack `eggup-eggpack` already preserves archive artifact size/digest plus declared member source/install/size/digest facts, but returns `ArchiveExtractionRequired`. With M001 closed, interoperability M002 can define the handoff without adding archive dependencies to lower layers.

Current generic archive APIs are useful primitives but do not by themselves satisfy Eggup's contract. Tar's safer `unpack_in` path can overwrite existing output; Zip's `enclosed_name` helps contain paths, while convenience extraction can overwrite files and support links. Eggup needs explicit iteration, allowlisting, no-clobber, and bounds.

## 5. Target architecture

```text
verified local archive
       |
       v
ArchivePlan + declared members + finite budgets
       |
       v
eggup archive layer
  - parse format
  - scan bounded entries
  - exact allowlist match
  - regular-file-only
  - stream to exclusive private files
  - enforce size/aggregate limits
  - hash while writing
       |
       v
ExtractedArtifactSet evidence
       |
       +--> consumer maps to ArtifactSet
       |
       +--> eggup-eggpack M002 adapter handoff
       |
       v
eggup-core prepare/verify/validate/commit
```

## 6. Dependency graph

```text
core M007 [closed] + acquisition M006 [closed]
              |
              v
archive M001 bounded allowlisted extraction [CLOSED]
              |
              v
archive M001a owned-root cleanup authority [HISTORICAL; SUPERSEDED]
              |
              v
archive M001b handle-bound cleanup + Windows portability [READY]
              |
              +--> consumer adoption M006 Egress [BLOCKED]
              |
              `--> Eggpack interoperability M002 archive handoff [BLOCKED]
```

## 7. Milestones

### M001 — Bounded allowlisted extraction contract

Plan: `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`.

Implement the optional extraction layer with tar.gz + zip evidence, regular-file-only semantics, private/no-clobber extraction, per-member and aggregate bounds, exact member size/digest verification, deterministic malicious fixtures, and no `eggup-core` dependency growth beyond ordinary domain consumption.

### M001a — Owned-root cleanup authority corrective

Plan: `plans/implementation/archive-extraction/001a-owned-root-cleanup-authority-corrective.md`.

Status: implemented historically but superseded by M001b; see `plans/closure/archive-extraction/001a-status.md`.

M001a added identity revalidation before recursive cleanup, but current review found the remaining check→delete TOCTOU and a stable-Windows compile failure around `MetadataExt::file_index()`. M001b is now the authoritative cleanup corrective.

### M001b — Handle-bound cleanup and Windows portability corrective

Plan: `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md`.

Status: ready.

Replace pathname-authorized recursive cleanup with retained directory authority/capability, repair stable-Windows portability, add deterministic after-check replacement-race tests, and require a fresh full hosted matrix before re-unblocking consumers.

### M002 — Egress real-consumer archive/pair adoption

Blocked until M001b closes. This roadmap does not authorize or replace the separate consumer-adoption plan.

Adopt the generic extraction output plus Eggup's existing multi-artifact transaction in Egress while preserving Egress-owned release/version/origin/candidate/CLI policy. Delete duplicated generic extraction/rollback machinery only after parity is qualified.

### M003 — Eggpack archive projection handoff

Blocked until M001b closes. This roadmap does not authorize or replace the separate Eggpack interoperability plan.

Connect `ManifestProjection::Archive` member evidence to the extraction contract without putting archive policy or producer authority into lower Eggup layers.

The existing Eggpack interoperability roadmap may retain its historical M002 numbering; this roadmap's M003 describes the archive subsystem side of that integration only.

## 8. Cross-cutting requirements

- Rust 1.89 baseline;
- no unsafe code in first-party implementation;
- no hidden shell/external extractor by default;
- no implicit privilege escalation;
- no unbounded decompression;
- diagnostics remain bounded and UTF-8 safe;
- platform path interpretation is fail-closed;
- dependency and binary-size impact is measured.

## 9. Verification strategy

Use generated/local fixtures, never public release archives as the correctness prerequisite.

Required malicious classes include traversal, absolute/prefix paths, duplicate/alias paths, links, devices/special files, oversized decompression, excessive entries, malformed/truncated archives, extra members, missing members, size/digest mismatch, raced output, and cleanup after failure.

Run tar.gz coverage on Unix and zip coverage on Unix plus Windows. Hosted Windows must execute archive tests rather than compile-only if the format implementation is portable.

## 10. Risks and decision points

The largest risk is accidentally turning Eggup into a general archive library. Keep the contract declaration-driven and regular-file-only.

The second risk is trusting convenience extraction APIs whose overwrite/link behavior is broader than Eggup's updater threat model. Iterate entries explicitly and apply Eggup's own no-clobber/bounds.

The third risk is dependency/footprint growth. Keep the crate optional and format features separable if measurement justifies it.

## 11. Completion definition

The subsystem is mature when M001b has closed the cleanup-authority and Windows-portability invariants with a green hosted matrix, Egress has removed its duplicated generic archive/pair update mechanics, and Eggpack archive evidence can flow through the same extraction boundary without adding archive code to `eggup-core`.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 bounded allowlisted extraction | closed historically | `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md` | `plans/closure/archive-extraction/001-status.md` | — |
| M001a owned-root cleanup authority | historical; superseded by M001b | `plans/implementation/archive-extraction/001a-owned-root-cleanup-authority-corrective.md` | `plans/closure/archive-extraction/001a-status.md` | remaining TOCTOU + Windows compile defect |
| M001b handle-bound cleanup + Windows portability | ready | `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md` | — | — |
| M002 Egress adoption | blocked | — | — | M001b closure |
| M003 Eggpack archive handoff | blocked | — | — | M001b closure |
