# ADR-0005: Local archive extraction safety contract

Status: accepted

Date: 2026-09-26

Decision owners: project maintainers

Related specifications:

- `plans/000-long-term-specification.md#6-multi-artifact-installation-is-fundamental`
- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#18-security-model`
- `plans/002-long-term-roadmap.md#phase-10--eggress-archivebundle-convergence`

Affected roadmaps:

- archive extraction;
- consumer adoption;
- Eggpack manifest interoperability.

## Context

Eggup already owns verified local deployment while Eggpack owns producer-side archive construction and release-member declarations. CodeGG proved that a consumer can verify a complete archive, perform strict consumer-owned extraction, and then use Eggup for the resulting multi-artifact transaction. That evidence did not create a reusable extraction contract.

Two roadmap branches now depend on such a contract:

- Consumer Adoption M006 needs reusable archive/member handling before Egress can replace its two-binary updater without carrying a second generic rollback/extraction implementation.
- Eggpack Manifest Interoperability M002 needs a qualified boundary that turns `ManifestProjection::Archive` member evidence into local files suitable for `ArtifactSet` construction.

The generic archive crates provide useful primitives but are not sufficient as the Eggup contract by themselves. Current `tar` documentation recommends `Entry::unpack_in` for untrusted archives but that API may overwrite an existing destination. Current `zip` APIs provide `enclosed_name` path validation, while convenience extraction may overwrite existing files and may materialize symbolic links. Eggup requires stricter no-clobber, allowlisted, bounded semantics.

## Decision

Eggup will add archive extraction as an optional consumer-side layer outside `eggup-core`.

The durable rule is:

**An archive may become deployment input only through a bounded, allowlisted extraction plan over already verified archive bytes. Extraction never grants live-destination mutation authority.**

### Layering

The intended dependency direction is:

```text
producer archive/member evidence
          |
          v
consumer verifies acquired archive bytes
          |
          v
optional eggup archive extraction layer
          |
          v
owner-private extracted regular files + exact member evidence
          |
          v
ArtifactSet / InstallPlan
          |
          v
eggup-core transaction
```

`eggup-core` remains archive-format independent.

### Input authority

The caller must provide:

- the exact local archive path;
- archive format or a format-specific adapter selected explicitly;
- the exact member allowlist to materialize;
- each member's expected relative source path;
- the intended installed member identity/destination;
- per-member exact size and/or integrity evidence when available;
- a finite per-member byte bound;
- a finite aggregate decompressed-byte bound;
- a finite entry-count bound.

Release selection, artifact origin, archive construction, version ordering, and fallback remain outside the extraction layer.

### Archive precondition

The extraction layer MUST NOT be the first verifier of the archive artifact.

The caller must supply evidence that the acquired archive itself has already passed the required integrity gate. Format parsing may still validate internal archive checksums/structure, but successful parsing is not a substitute for the external archive integrity requirement.

### Path rules

The generic contract accepts only normalized relative member paths.

It MUST reject:

- absolute paths;
- parent traversal;
- platform path prefixes/drives;
- NUL/control-path ambiguity;
- empty terminal names;
- duplicate normalized members;
- path aliases that would map two declared members to one extraction destination;
- archive entries that do not exactly correspond to the declared member path when extracting that member.

The first generic implementation may deliberately restrict member names to portable UTF-8 paths. Supporting arbitrary platform-native/non-UTF-8 archive names requires a later explicit contract.

### File-type rules

The initial generic contract materializes regular files only.

It MUST reject archive members represented as:

- symbolic links;
- hard links;
- devices;
- FIFOs;
- sockets;
- other special entries.

Directories may be created only as owner-private extraction infrastructure required for declared regular members. Archive metadata does not grant permission to create arbitrary filesystem objects.

### No-clobber and race rules

Extraction occurs under an Eggup-owned private temporary root that did not exist before the operation.

The implementation MUST:

- create the root exclusively;
- keep it owner-private;
- never extract directly into a live installation root;
- refuse to overwrite an existing extraction member;
- reject a preexisting/raced-in path;
- avoid following archive-created or foreign symlinks;
- clean only paths owned by the extraction operation;
- return extracted local files for later Eggup staging/transaction use.

Archive extraction success alone never authorizes replacement of an existing installation.

### Resource bounds

All extraction is finite.

The contract MUST enforce:

- declared archive/member count bounds before unbounded allocation;
- per-member decompressed byte bounds while streaming;
- aggregate decompressed byte bounds while streaming;
- bounded path/name lengths;
- bounded diagnostic text;
- cancellation/check opportunities sufficient to stop maliciously large inputs without waiting for the full archive.

Compressed size alone is not a decompression bound.

### Integrity continuity

When exact member digest/size evidence is available, the extraction layer verifies the extracted bytes before returning success.

The output must carry enough evidence to construct `ArtifactMember` values without losing the producer/consumer binding. The later `eggup-core` stage still performs its normal integrity verification and locked staged-byte revalidation.

### Extra archive entries

The generic contract does not silently install undeclared entries.

The implementation may skip undeclared entries while scanning only if doing so remains bounded and cannot affect declared-member extraction. A stricter caller mode may reject extra entries. No undeclared entry may be materialized.

### Formats

The first implementation should cover the formats required by concrete consumers rather than create a generic plugin framework.

Egress supplies current evidence for:

- `.tar.gz` on Linux/macOS;
- `.zip` on Windows.

Format-specific dependencies belong in the optional archive crate/layer, not `eggup-core`.

## Consequences

This creates one reusable consumer-side safety boundary for Egress and Eggpack archive interoperability while preserving the Eggpack/Eggup producer-consumer split.

The cost is a new optional dependency surface and a deliberately narrower extraction feature set than general-purpose archive tools. That restriction is intentional: updater archives need predictable deployment bytes, not restoration of arbitrary filesystem metadata.

## Verification

The extraction contract is qualified only with negative tests for:

- `../` and absolute paths;
- Windows-prefix/path alias cases;
- duplicate members;
- symlink/hardlink/special entries;
- existing/raced-in output paths;
- oversized member;
- aggregate decompression overflow;
- excessive entry count;
- truncated/corrupt archives;
- missing declared member;
- wrong exact size;
- digest mismatch;
- cancellation/cleanup;
- tar.gz and zip fixtures on relevant platforms.

At least one real archive consumer must adopt the contract before it is considered mature.

## Non-goals

This ADR does not authorize:

- archive construction;
- producer packaging policy;
- installer generation;
- release discovery;
- authenticity/signature policy;
- extracting arbitrary metadata/ownership/xattrs;
- live destination mutation during extraction;
- a general-purpose archive library.

## Supersession

A future ADR may broaden supported archive entry types or trust semantics, but it must preserve the no-live-mutation, bounded-resource, and explicit-authority properties unless this ADR is explicitly superseded.
