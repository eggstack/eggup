# Archive Extraction — Deep Dive

Review entry point for `eggup-archive`: a single-file crate
(`crates/eggup-archive/src/lib.rs`, 3512 lines, 51 `#[test]` functions) that
extracts explicitly declared regular files from **already verified** local
`tar.gz` and `zip` archives into one owner-private directory, then hands the
bytes to `eggup-core` staging through open file objects rather than
pathnames.

Read this together with [overview.md](overview.md) (index and terminology
contract) and the normative
[ADR-0005](../plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md).

## 1. Purpose and ownership boundary

The crate owns member path authorization, entry classification, decompression
budgets, exclusive private materialization, streamed size/SHA-256 evidence, and
residue cleanup for the extraction root it creates. Nothing else.

It never does any of the following, and there is no code path in the crate that
could:

| Not owned | Evidence |
|---|---|
| Fetch or acquire bytes | no transport dependency; `extract` opens a caller-supplied `PathBuf` only (`lib.rs:751`, `975`, `1043`) |
| Choose a source, URL, or release | no ranking, retry, or mirror logic; `ArchivePlan::new` takes one `archive_path` (`lib.rs:139`) |
| Claim authenticity | no signature or trust-root code; digests are SHA-256 only, `validate_expected` compares against caller facts (`lib.rs:1134`) |
| Select an install root or destination | no destination field on any type; the caller supplies `output_parent` (`lib.rs:753`) |
| Mutate a live installation | no parent creation for install roots, no writes outside the created child (`create_private_root`, `lib.rs:1259`) |
| Restore metadata, ownership, or xattrs | no `set_permissions`/chown/xattr calls; only `0600`/`0700` creation modes (`lib.rs:1274`, `1315`) |
| Run an external process or service lifecycle | zero such dependencies; `Cargo.toml` lists `tar`, `zip`, `flate2`, `fs_at`, `sha2` |

Extraction success proves only that the declared bytes were written and hashed.
It is not authorization to replace anything; destination ownership and
permission decisions stay in `eggup-core` and the caller.

## 2. Position in the workspace

`eggup-archive` sits between the caller and `eggup-core`, downstream of
acquisition and optional Eggpack projection:

```text
eggup-eggpack (optional)  ManifestProjection::Archive -> ArchivePlan
        |
        v
eggup-archive             extract() -> ExtractedArchive -> PersistedExtraction -> BoundExtraction
        |                  (declarations + budgets + private root + evidence)
        v
eggup-core                InstallPlan::prepare_with_bound_sources(BoundSources)
```

Dependencies are `tar`, `zip`, `flate2` (`rust_backend`), `fs_at`, `sha2` only.
`eggup-core` appears in `[dev-dependencies]` **only** and is referenced only
from the test module (`lib.rs:1515`, `3229`), to prove the handoff composes with
`InstallPlan`/`BoundSources` and stages bytes in a real transaction
(`extracted_file_can_be_prepared_as_an_ordinary_core_artifact`, `lib.rs:2830`).
It is not a runtime dependency, and `eggup-core` does not depend on this crate
or on any archive-format package. See
[core-transaction.md](core-transaction.md) and
[eggpack-adapter.md](eggpack-adapter.md).

## 3. Public surface

Every exported item in the crate, with its role. All line references are
`crates/eggup-archive/src/lib.rs`.

| Item | Role | Line |
|---|---|---|
| `ArchiveFormat` | `TarGz` \| `Zip`; format is a caller declaration, never sniffed | 29 |
| `ArchiveLimits` | Finite per-operation budget bundle (fields private, no accessors) | 38 |
| `ArchiveLimits::new` | Builds validated limits; rejects zero, `max_entries > 100_000`, `max_path_bytes > 4096`, `max_member_bytes > max_total_uncompressed_bytes` | 48 |
| `ArchiveLimits::strict_default` | Conservative starting point (see §5) | 76 |
| `ArchiveMember` | One declared `source_path` → `output_name` mapping plus optional expected size/digest | 89 |
| `ArchiveMember::new` | Validates both strings against a hardcoded 4096-byte cap | 98 |
| `ArchiveMember::source_path` | Exact normalized archive path that must appear | 117 |
| `ArchiveMember::output_name` | Single-component filename inside the private root | 122 |
| `ArchivePlan` | Archive path + format + allowlist + limits; opaque after construction (no getters) | 129 |
| `ArchivePlan::new` | Rejects empty/oversized allowlists, duplicate sources, case-insensitive duplicate outputs, over-budget declared sizes | 139 |
| `ExtractionErrorKind` | 14 stable failure categories (no OS error text) | 182 |
| `ExtractionError` | Failure carrier: `kind` plus optional `residue_path` | 219 |
| `ExtractionError::kind` | Stable category accessor | 259 |
| `ExtractionError::residue_path` | Private-root residue location; see §9 for its two meanings | 277 |
| `ExtractedMember` | Evidence for one extracted file: source, output, advisory `path`, `bytes_written`, `sha256` | 292 |
| `ExtractedArchive::root` | Owner-private root pathname (diagnostic) | 349 |
| `ExtractedArchive::members` | Evidence in **plan declaration order** | 354 |
| `ExtractedArchive::persist` | Disarms the drop guard and transfers cleanup to the caller | 360 |
| `ExtractedArchive::cleanup` | Explicit abort: closes member objects, then empties the owned root; `Ok` = emptied, `Err(CleanupFailed)` = not emptied (§9) | 397 |
| `PersistedExtraction::root` | Retained root pathname | 426 |
| `PersistedExtraction::members` | Same evidence, still in declaration order | 431 |
| `PersistedExtraction::cleanup` | Empties the retained root via its handle; same `Ok`/`Err(CleanupFailed)` contract (§9) | 460 |
| `PersistedExtraction::into_bound_sources` | Converts each member into an object-bound `BoundMember`, rewound to byte 0 | 486 |
| `BoundMember` | Evidence + already-open readable member object; deliberately not `Clone` | 540 |
| `BoundMember::advisory_path` | Recorded pathname, explicitly non-authoritative | 565 |
| `BoundMember::bytes_written` / `sha256` | Same evidence as `ExtractedMember` | 570 / 575 |
| `BoundMember::handle_mut` | Mutable borrow to prove cursor ownership | 582 |
| `BoundMember::into_open_object` | Releases the `File` for `eggup-core` `BoundSources` | 592 |
| `BoundExtraction::members` / `members_mut` | Bound members, declaration order | 612 / 618 |
| `BoundExtraction::root` | Retained root pathname | 623 |
| `BoundExtraction::into_members` | Splits staging sources from `DeferredCleanup` | 634 |
| `BoundExtraction::cleanup` | Abort without staging; same `Ok`/`Err(CleanupFailed)` contract (§9) | 670 |
| `DeferredCleanup::root` | Residue diagnostics | 701 |
| `DeferredCleanup::cleanup` | Empties the root once bound handles are closed; same `Ok`/`Err(CleanupFailed)` contract (§9) | 729 |
| `extract` | Entry point: `&ArchivePlan` + `output_parent` → `ExtractedArchive` | 751 |

Internal, not exported: `ExtractionScope` (797), `DirectoryGuard` (1461),
`write_declared_member` (916), `copy_bounded` (1086), `drain_bounded` (1126),
`validate_expected` (1134), `validate_archive_path` (1154),
`validate_output_name` (1192), `create_private_root` (1259),
`create_private_file_at` (1305), `empty_dir_contents` (1326),
`remove_owned_root_with_hook` (1439), and the `#[cfg(test)]` hook
`MaterializationHook` (789) / `extract_with_hook` (834).

`ExtractedArchive`, `PersistedExtraction`, `BoundMember`, `BoundExtraction`, and
`DeferredCleanup` each retain a `std::fs::File`, so they are `Send` and are
single-owner handle holders — `BoundMember` is intentionally not `Clone`,
because `File` cursors are shared across clones (`lib.rs:530-534`).

## 4. The allowlist model

`ArchivePlan` is a closed allowlist, not a filter. `members` is the complete
set of things that may ever be written; everything else in the archive is
counted and discarded.

- `ArchiveMember::new` (`lib.rs:96`) validates the declared pair up front
  against a hardcoded 4096-byte length cap, because the plan's
  `max_path_bytes` is not known yet.
- `ArchivePlan::new` (`lib.rs:145-170`) re-validates both strings against the
  plan's `max_path_bytes` and enforces three allowlist invariants: no empty
  allowlist, no count above `max_entries`, and no two members sharing a
  `source_path` or a case-insensitively equal `output_name`
  (`to_ascii_lowercase`, `lib.rs:155`). A declared `expected_size` is also
  rejected if it exceeds `max_member_bytes` or if the declared sizes sum above
  `max_total_uncompressed_bytes`.
- Selection is **exact string equality on the normalized path**
  (`plan.members.iter().find(|m| m.source_path == normalized)`, `lib.rs:925`,
  in the shared member-write helper). There is no glob, prefix, or suffix
  matching; a declared
  `source_path` that does not equal the archive's normalized path is simply
  never selected.

### `source_path` → `output_name`

`source_path` is the archive-internal, possibly nested, slash-separated path.
`output_name` is the flat, single-component filename the member gets in the
private root. `validate_output_name` (`lib.rs:1192`) rejects any `/`, `\`, or
`:`, so **nested archive layouts are flattened**: a member at `bin/tools/app`
must be declared as `output_name = "app"`. The extractor never creates
subdirectories. This is stricter than ADR-0005's allowance that "directories
may be created as owner-private extraction infrastructure"
([ADR-0005](../plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md)
§File-type rules) — directory entries in an archive are drained, never created.

`output_name` is also restricted to portable ASCII with no control characters,
no `<>|?*`, no trailing dot or space, and no Windows device name — `CON`, `PRN`,
`AUX`, `NUL`, `CLOCK$`, `CONIN$`, `CONOUT$`, and `COM0`–`COM9` / `LPT0`–`LPT9`
(the whole digit range, not just 1–9, per the comment at `lib.rs:1215`).

### Members that are not declared

Undeclared entries are **never materialized**. They are still read and charged:
`drain_bounded` (`lib.rs:1126`) routes through the same `copy_bounded` loop with
`output = None`, so an undeclared entry consumes the same per-member and
aggregate decompressed budgets it would have consumed if declared, and its
inflate cost is bounded. This is what stops a declared-member allowlist from
becoming a decompression-bomb bypass. Note the asymmetry: the crate implements
only the "skip extra entries" mode ADR-0005 permits; there is no stricter
caller mode that rejects extras.

Every entry path is validated (`validate_archive_path`, `lib.rs:1154`) before
any selection, and every normalized path is inserted into a `seen` set
(`lib.rs:997`, `1057`) so two archive entries that normalize to the same path
fail as `DuplicateArchiveMember`.

### Why `advisory_path` is advisory

`BoundMember::advisory_path` (`lib.rs:565`) is the old `ExtractedMember::path()`
retained for diagnostics. It is `root.join(output_name)` — a *recorded* name, not
an authorization. Because writes are authorized by the retained root handle
(`open_at`, `lib.rs:1318`), a root rename or member-entry replacement after
extraction can leave this path pointing into a foreign directory while the owned
bytes remain in the handle-owned object. Opening it would therefore re-introduce
exactly the pathname-resolution hazard the design removes, which is why
`into_open_object` (`lib.rs:592`) is the only byte-producing accessor. Tests
deliberately read owned bytes from the known moved location instead of the
returned path when they induce a rename (`lib.rs:2877-2880`).

## 5. Budgets and limits

`ArchiveLimits` (`lib.rs:38`) has five fields, all private and all with no
getters, so a caller can neither read back nor log the limits it is running
under. `ArchiveLimits::new` rejects a zero value for any bound,
`max_entries > 100_000`, `max_path_bytes > 4096`, and
`max_member_bytes > max_total_uncompressed_bytes`.

| Field | `strict_default()` | Bound to | Checked at |
|---|---|---|---|
| `max_archive_bytes` | 512 MiB (536 870 912) | compressed file length | `lib.rs:761` — once, from `symlink_metadata().len()` |
| `max_entries` | 10 000 | archive entry count | tar per iteration `983`; zip once against `archive.len()` `1047` |
| `max_path_bytes` | 1 024 | every entry and declared path | `996`, `1056` (entry), `152`–`153` (plan) |
| `max_member_bytes` | 256 MiB (268 435 456) | one member, decompressed | `1108` — per 16 KiB chunk, before the write |
| `max_total_uncompressed_bytes` | 512 MiB (536 870 912) | whole extraction, decompressed | `1111` — per 16 KiB chunk, before the write |

### Compressed vs decompressed accounting

These are two independent budgets and only one of them is streamed.

`max_archive_bytes` is a **pre-flight metadata check** on the archive path,
performed before the private root is even created (`lib.rs:761`). The handler
then reopens the file by path (`lib.rs:975`, `1043`) without re-checking, so the
compressed bound is advisory with respect to the bytes actually read; a file
swapped between the check and the read is not re-validated. The crate relies on
the caller's precondition here — the caller is required to have already
integrity-gated the artifact, and `eggup-eggpack::validate_acquired_archive`
(`crates/eggup-eggpack/src/lib.rs:481`) does exactly that size+SHA-256 gate
before any extraction.

`max_member_bytes` and `max_total_uncompressed_bytes` are the real budget and
are enforced **while streaming**, inside `copy_bounded` (`lib.rs:1102-1113`):
both counters use `checked_add` (overflow maps to `MemberTooLarge` /
`TotalTooLarge`) and both are compared against their limits *before*
`write_all` runs, so an over-budget member is never fully materialized. The
`total` counter is a single accumulator threaded through the whole extraction,
including drained undeclared and non-regular entries, so declared members
cannot borrow budget from undeclared ones or vice versa.

### Why a decompression budget is needed after digest verification

Verifying the compressed artifact's digest proves the bytes on disk are the
bytes the producer published. It says nothing about how large those bytes
*become*. A few hundred kilobytes of deflate can inflate to gigabytes, and the
ratio is attacker-chosen even when the compressed input is bit-for-bit the
published artifact. `ADR-0005` states this directly: "Compressed size alone is
not a decompression bound." A digest over the archive constrains the *input*;
only a per-member and aggregate decompressed budget constrains the resource the
extraction actually consumes.

There is no cancellation flag or cancel-check API in this crate. The only
mechanism that can stop a maliciously large input is the per-chunk budget check
in `copy_bounded`, which does terminate without reading the remainder of the
archive. ADR-0005's resource-bounds section additionally lists "cancellation/
check opportunities"; the code satisfies the *check* half and provides no caller
cancellation.

## 6. Format handling

`ArchiveFormat` (`lib.rs:29`) is declared by the caller and never sniffed.
`eggup-eggpack::archive_format_for_name`
(`crates/eggup-eggpack/src/lib.rs:458`) maps `.tar.gz`, `.tgz`, `.zip` to the
variants and fails closed on anything else.

### `TarGz`

`extract_tar_gz` (`lib.rs:968`) wraps the file in `GzDecoder` and iterates
`tar::Archive::entries`. Per entry it rejects any raw name containing `\`
(`988`), requires valid UTF-8 (`991-992`), normalizes with `validate_archive_path`
allowing a directory trailing `/`, and records the normalized path in `seen`.
After the iterator ends it drains the decoder and requires every trailing byte
to be `0` within `TAR_TRAILING_PADDING_LIMIT` (1 MiB), otherwise
`MalformedArchive` (`1029-1041`) — this validates the gzip footer and refuses a
concatenated second tar stream or an inflate tail. Everything after the
per-entry parsing is delegated to the shared member-write helper
(`write_declared_member`, `lib.rs:916`).

### `Zip`

`extract_zip` (`lib.rs:1036`) opens `ZipArchive`, checks `archive.len()` against
`max_entries` once, then iterates by index. Each name is taken from
`entry.name_raw()` and required to be UTF-8. Everything after the per-entry
parsing is delegated to the same shared member-write helper
(`write_declared_member`, `lib.rs:916`).

### Regular-file-only rule

A member is materializable only if it is a regular file **and** is declared.

- tar (`lib.rs:1002`): `matches!(entry_type.as_byte(), 0 | b'0')` — only the old
  regular (`NUL`) and GNU regular (`0`) typeflags.
- zip (`zip_entry_is_regular`, `lib.rs:1076`): requires `is_file()`, not
  `is_symlink()`, not `is_dir()`, `enclosed_name().is_some()`, and either no
  `unix_mode()` or a mode whose file-type bits are `0` or `0o100000`. The
  `enclosed_name()` check is independent defense in depth — it rejects
  traversal-shaped zip names even if the other predicates pass.

Symlinks, hardlinks, directories, devices, FIFOs, sockets, and unknown/sparse
types therefore behave as follows:

| Entry | Declared | Outcome |
|---|---|---|
| regular file | yes | materialized, verified, evidence recorded |
| any non-regular type | yes | `NonRegularMember` — fails the whole extraction (`930`, `1063`) |
| regular file | no | drained under budget, discarded |
| any non-regular type | no | drained under budget, discarded; no filesystem object created |

Nothing from the archive — permission bits, timestamps, ownership, xattrs — is
ever restored onto an extracted file. Archive metadata grants no permission to
create filesystem objects.

## 7. The extraction pipeline, step by step

`extract(plan, output_parent)` (`lib.rs:751`):

1. **Pre-flight the archive.** `symlink_metadata(plan.archive_path)` must be a
   real file and not a symlink (`757-759`), else `InvalidPlan`; its length must
   be within `max_archive_bytes` (`760-762`), else `ArchiveTooLarge`.
2. **Create the private root** — `create_private_root(parent)` (`1259`). The
   parent must already exist, be a real directory, and not be a symlink
   (`1260-1263`). A parent handle is opened (`open_dir_handle`, `1243`; on
   Windows with `FILE_FLAG_BACKUP_SEMANTICS`). The root is then created
   **through that parent handle** with `fs_at::OpenOptions::mkdir_at`, mode
   `0700` on Unix, named `.eggup-extract-{pid}-{id:016x}` from a process-wide
   `NEXT_ROOT_ID` counter (`1267-1284`). Up to `ROOT_ATTEMPTS` (32) name
   collisions are retried; `AlreadyExists` continues, any other error is `Io`.
   This is where the private root and its permissions come from: one
   `mkdir_at` on a parent handle, and the **returned `File` handle is the
   authority for everything that follows**. The crate never creates a parent of
   a live install root and never writes outside this one child.
3. **Wrap the root in a `DirectoryGuard`** and run `extract_with_hook_inner`
   (`867`), which builds the shared `ExtractionScope` (`797`) — root pathname,
   root handle, write sequence, test hook — used by both format handlers so tar
   and zip share one authority path.
4. **Iterate entries** per format (§6). For each entry: validate the path,
   check the duplicate set, look up the declaration, classify the type. Steps 4's
   declaration lookup and type classification are handed to the shared
   member-write helper (`916`).
5. **Materialize declared regular members** via `ExtractionScope::create_member`
   (`813`) → `create_private_file_at(root_handle, output_name)`
   (`1305`). This is **where no-clobber is enforced**: `fs_at::OpenOptions`
   with `read(true)`, `write(Write)`, `create_new(true)`, and `follow(false)`,
   mode `0600` on Unix, resolved by `open_at` relative to the retained root
   handle. `create_new` rejects any pre-existing file, symlink, or directory
   atomically where the filesystem supports it; `follow(false)` refuses to
   follow a final-component symlink or reparse point. Any failure is reported
   as `Io`, and an existing file is left byte-identical
   (`exclusive_output_creation_preserves_an_existing_file`, `2814`).
6. **Stream, hash, and bound** — `copy_bounded` (`1086`) copies in fixed
   `IO_CHUNK_SIZE` (16 KiB) chunks: charge per-member and aggregate budgets,
   `write_all` when an output is present, and update SHA-256 in the same loop.
   One read error maps to `MalformedArchive`; a write error to `Io`.
7. **Flush and verify** — `output.flush()` (`949`), then
   `validate_expected` (`1134`) compares observed size then observed digest
   against the caller's optional facts, producing `SizeMismatch` or
   `DigestMismatch`. The mismatch check therefore happens *after* the bytes
   land in the owned root; the failure path below removes them.
8. **Re-order to declaration order** (`888-897`). Results are accumulated in a
   `HashMap` keyed by normalized `source_path` during the archive walk, then
   drained in `plan.members` order. A declared path never seen in the archive is
   `MissingMember`. Evidence order therefore follows the plan, not the archive.
9. **Wrap or clean up.** On success, `ExtractedArchive { guard, members,
   handles }` is returned with all member objects still open. On failure, the
   root is emptied through the handle and the original error category is
   preserved, with the residue path attached (`770-779`).

Steps 4–7 live in **one** shared helper, `write_declared_member`
(`lib.rs:916`): the allowlist lookup, the `NonRegularMember` check, the bounded
drain of undeclared and non-regular entries, the handle-authorized create, the
bounded copy with streaming SHA-256, the flush, the expected-value validation,
and the evidence recording. Both format handlers delegate to it; only their
per-format parsing (entry iteration, name decode, path validation, duplicate
detection, entry-count bound, entry-kind predicate, and the tar gzip-tail
validation) is handler-local. A source-level guard
(`crates/eggup-archive/tests/m005_member_write_parity.rs`) asserts that the
member creation primitive, the bounded copy with a populated output, and the
expected-value validation each have exactly one call site in non-test code.

The recorded member pathname is computed by `ExtractionScope::member_path`
(`824`) as `root_path.join(output_name)` and is explicitly **evidence only** —
the write was already authorized by the handle (`lib.rs:941-946`).

## 8. The three-stage handoff

This is the review focus. The three types are not convenience wrappers: each is a
different **ownership regime**, and the type change is what enforces the
transition.

```text
extract(&plan, parent)
        |
        v
ExtractedArchive  -- guard ARMED: drop empties the root
   |   |
   |   +-- cleanup()  -> abort; Ok = emptied, Err(CleanupFailed) = not emptied
   |
   +-- persist()  [explicit; disarms the guard]
        |
        v
PersistedExtraction  -- guard DISARMED: nothing removes the root
   |   |
   |   +-- cleanup()  -> abort
   |
   +-- into_bound_sources()  [flush + rewind each member to byte 0; demote path to advisory]
        |
        v
BoundExtraction  -- members are open objects; root handle still retained
   |   |
   |   +-- cleanup()  -> abort without staging
   |
   +-- into_members()  -> (Vec<BoundMember>, DeferredCleanup)
        |
        v
staging (eggup-core)  ->  DeferredCleanup::cleanup()  ->  caller removes the empty dir
```

### What each stage owns and guarantees

| | `ExtractedArchive` | `PersistedExtraction` | `BoundExtraction` |
|---|---|---|---|
| Owns | `DirectoryGuard` (root path + open handle), `Vec<ExtractedMember>`, `Vec<File>` | root path, open root handle, evidence, open member handles | open root handle, `Vec<BoundMember>` |
| Guard state | armed | disarmed (`DirectoryGuard::disarm`, `1489`) | disarmed; handle held directly |
| Guarantees | all declared members written, verified, still open; drop empties the root | the extraction survives the previous value being dropped; nothing deletes the root | each member's bytes are proven by the already-open object, rewound to 0; no name lookup needed downstream |
| Evidence access | `members()` → `ExtractedMember` (advisory `path` still meaningful) | `members()` → `ExtractedMember` | `members()` → `BoundMember` (`advisory_path` explicitly non-authoritative) |
| Drop does | `DirectoryGuard::drop` (`1502`) best-effort empties the root via the handle; result discarded | nothing — no `Drop` impl | nothing — no `Drop` impl |
| Next step | `persist()` (to keep the bytes) or `cleanup()` (to discard) | `into_bound_sources()` (to stage) or `cleanup()` (to abort) | `into_members()` (to stage then clean) or `cleanup()` (to abort) |

After `into_members()`, the root handle lives only in `DeferredCleanup`
(`lib.rs:686`). That is the point of the split: staging sources and cleanup
responsibility must be owned by different values, because the supported order is
**stage → close/consume handles → cleanup**. Running cleanup first fails closed
on platforms where open handles pin directory entries (`lib.rs:669-670`).

### Why the stages are separate rather than one flag

Three distinct hazards require three distinct types:

1. **The drop guard must be disarmed explicitly.** If a persisted extraction
   were the same type as a live one, the guard would delete the bytes the caller
   just asked to keep. `persist()` consumes `self` and takes the guard's handle
   out (`360-370`), so "keep it" is a type transition, not a boolean a caller can
   forget. This is why persistence is opt-in: an `ExtractedArchive` that is
   dropped unused is safe, and one that is kept must say so.
2. **The pathname must be demoted before staging.** `into_bound_sources`
   (`486-518`) flushes and rewinds each member to byte 0 and moves the recorded
   path into `advisory_path`. Once a `BoundMember` exists, the only way to get
   bytes is `into_open_object()`. A staging step physically cannot resolve a
   name, so a member-entry replacement or root rename after the handoff cannot
   redirect staged bytes. Asserting this as a flag on a shared type would leave
   the authority ambiguous.
3. **Cleanup must be deferred past staging.** Once member objects are handed to
   `eggup-core` as `BoundSources`, the crate must not be able to empty the root
   out from under them. Separating `Vec<BoundMember>` from `DeferredCleanup` makes
   that unrepresentable rather than a documented ordering rule.

### `persist()` versus `cleanup()`

| Situation | Call | Result |
|---|---|---|
| Bytes are needed downstream (staging) | `persist()` | `PersistedExtraction`; root survives; caller now owns cleanup |
| Extraction result is unused / abandoned | drop `ExtractedArchive` | root best-effort emptied, residue directory left, result discarded |
| Abort after persistence, before binding | `PersistedExtraction::cleanup()` | root emptied via handle → `Ok(())`; the residual empty directory is the caller's to remove |
| Abort after binding, before staging | `BoundExtraction::cleanup()` | member objects closed first, then root emptied → `Ok(())`; same caller-owned residual directory |
| Normal completion | `into_members()` → stage → `DeferredCleanup::cleanup()` | root emptied → `Ok(())`, then the caller removes the empty directory |
| Staging failed | same as normal completion | do **not** reopen the recorded path; the error is recoverable and the root is still handle-owned |

The single most important thing to know when reading call sites: **all four
`cleanup()` methods report the same thing.** `remove_owned_root_with_hook`
(`lib.rs:1439-1460`) consumes the `empty_result` of the handle-based emptying
and maps it to one of two outcomes: `Ok(())` when the owned root object held no
entries at the final handle-relative listing, and
`Err(CleanupFailed).with_residue(path)` when it could not be fully emptied.
There is no portable object-bound root unlink, so the crate empties the owned
tree and leaves the residual empty directory in place rather than recursively
touching the replacement pathname. The crate never removes that directory
itself, on any path.

Consequences for callers:

- `Ok` means *emptied*, not *removed*. The root directory still exists; the
  caller removes it. Because `cleanup()` consumes the value, capture the
  residual path from `root()` **before** the call.
- `Err(CleanupFailed)` means *not emptied*. The residue may be partially
  emptied and its emptiness is **not** guaranteed; the caller must inspect it.
- A caller that wrote `cleanup().unwrap_err()` now panics on the success path.
  This is a behavior change, not a source-breaking API change.

The crate's own tests encode the contract: `persisted.cleanup()` expected to
succeed, then `assert!(root.exists())` and
`assert_eq!(fs::read_dir(&root).unwrap().count(), 0)` followed by
`fs::remove_dir(&root)` (`lib.rs:2320-2336`, `2741-2787`). The failure path is
proven inline by `cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence`
(`lib.rs:2741`), because the crate exposes no portable public failure trigger.

## 9. Cleanup and residue

Three mechanisms cooperate.

**`DirectoryGuard`** (internal, `lib.rs:1461`) holds the root path and the open
`File`. It is created only after `mkdir_at` returns the handle, so directory
authority exists before any archive byte is materialized. `drop`
(`1502-1510`) runs only when `active` and calls `empty_dir_contents` best-effort,
discarding the result. `disarm()` and `cleanup()` both clear `active` first, so
exactly one of drop or explicit cleanup ever runs.

**`empty_dir_contents`** (`lib.rs:1326`) recurses using only `fs_at`
handle-relative operations — `read_dir`, `open_dir_at`, `unlink_at`, `rmdir_at`
— never a pathname traversal. Depth is capped at `CLEANUP_MAX_DEPTH` (32,
checked at `1331`), at
most `CLEANUP_MAX_ENTRIES_PER_PASS` (10 000) entries per pass (`1346`), and
`CLEANUP_MAX_PASSES` (8) passes (`1334`), after which a pass that deletes nothing is an
`Io` error (`1415`). `fs_at` deletions treat `NotFound` as success, so a racing removal
is not a failure. Each child is opened with `open_dir_at` (no follow) and
`fstat`-ed: a child that is a real directory recurses, everything else — symlink
or regular file — is `unlink_at`-ed and never followed. The explicit `is_symlink`
check (`1369-1372`) is redundant on Unix (where the open already refused to
follow) and load-bearing on Windows, where there is no `O_NOFOLLOW` equivalent.

**`ExtractionError::residue_path()`** has two distinct meanings depending on
which constructor set it, and this is the one place a reviewer must be precise:

| Constructor | Used by | `kind()` | Meaning of `Some(path)` |
|---|---|---|---|
| `with_empty_residue` (`250`) | the `extract()` failure path when the owned root **was** emptied (`770-773`) | original category, or `CleanupFailed` if the root is non-empty | root verified empty by `private_root_is_empty`; the path is a leftover *directory* |
| `with_residue` (`238`) | the `extract()` failure path when the owned root could **not** be emptied (`775-778`), and all four `cleanup()` methods (`1448-1460`) | always `CleanupFailed` | root directory still exists; **emptiness is not verified** |

So: a caller can conclude that residue lives at `residue_path()` and that the
crate did not delete the root itself. A caller can conclude the directory is
empty **only** on the `extract()` failure path where the owned root was
emptied. On any explicit `cleanup()` failure the caller must inspect the
directory itself.

The success path carries no residue path at all: `cleanup()` returns `Ok(())`,
so there is no error value to carry one. A caller that wants the residual
directory's path captures it from `root()` **before** the consuming cleanup
call.

The crate never removes the residual directory on any path, including the
success path, and exposes no convenience that removes it by pathname — that
would reintroduce the pathname-authority hazard M001b closed. The recorded root
path is **advisory**: if the root was renamed or replaced by a third party, the
crate emptied the owned object and left whatever occupies the recorded
pathname untouched.

## 10. Invariants and failure modes

| Condition | Observable outcome | Where handled |
|---|---|---|
| Compressed file over `max_archive_bytes` | `ArchiveTooLarge` before the root exists | `lib.rs:761` |
| More than `max_entries` entries | `TooManyEntries` | tar `983`; zip `1047` |
| Declared member exceeds `max_member_bytes` | `MemberTooLarge` before the over-budget chunk is written | `lib.rs:1108` |
| Extraction exceeds `max_total_uncompressed_bytes` | `TotalTooLarge` | `lib.rs:1111` |
| Byte counter overflow | `MemberTooLarge` / `TotalTooLarge` via `checked_add` | `1104`, `1107` |
| Undeclared member present | drained under budget, never materialized, extraction succeeds | `933-939` (shared helper) |
| Undeclared or non-regular entry is a decompression bomb | still charged to the aggregate budget | `drain_bounded` → `copy_bounded` |
| Declared member is a symlink/hardlink/dir/device/FIFO | `NonRegularMember`, whole extraction fails | `930` (shared helper), predicates at `1002` / `1062` |
| Undeclared member is a symlink/hardlink/dir/device | drained, never created | `933-935` (shared helper) |
| Absolute, `..`, `\`/`:`-bearing, control-char, `.`, empty-component, or non-UTF-8 path (either side) | `InvalidPath` | `validate_archive_path` `1154`; `validate_output_name` `1192` |
| Two archive entries normalize to one path | `DuplicateArchiveMember` | `997`, `1057` |
| Declared source or output declared twice | `InvalidPlan` at plan construction | `154-158` |
| Windows device / reserved output name | `InvalidPath` | `1209-1223` |
| Declared path absent from the archive | `MissingMember` after the walk | `893` |
| Observed size differs from `expected_size` | `SizeMismatch` | `1139-1144` |
| Observed SHA-256 differs from `expected_sha256` | `DigestMismatch` | `1145-1150` |
| Path escape attempt via entry name | `InvalidPath`; nothing written anywhere | `validate_archive_path` before `create_member` |
| Pre-existing or raced-in output name in the owned root | `Io`; existing entry untouched | `create_new` in `1310`; test `3115`, `3148` |
| Root renamed or replaced before/between member writes | bytes stay in the handle-owned directory; foreign replacement untouched | `open_at` at `1318`; tests `2883`, `2934`, `2977`, `3035`, `3072` |
| Corrupt/truncated gzip or zip, or a concatenated tar | `MalformedArchive` | `976`, `986`, `993`, `1050`, `2234` test |
| Non-file or symlink `output_parent`; symlink archive path | `InvalidPlan` | `757`, `1262` |
| Any failure after root creation | original category preserved; root emptied via handle; residue path attached | `770-779` |
| Explicit cleanup that cannot empty the root | `CleanupFailed`; residue path names the recorded root; emptiness **not** guaranteed | `1448-1460`; test `2741` |

`ExtractionError`'s `Display` is
`"archive extraction failed ({kind:?})"` (`lib.rs:284`) — categories only. No
member name, path, archive byte, or OS error string is ever rendered, matching
the workspace-wide redaction rule.

## 11. Relationship to the verification ladder

Per [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)
and the [verification ladder](overview.md#two-orthogonal-axes) in
[overview.md](overview.md), this crate occupies the **integrity** rung only.

| Layer | Question | Owner here |
|---|---|---|
| Integrity | *Are these bytes the bytes that were hashed?* | `eggup-archive` reports it; `eggup-core` re-verifies at prepare and commit |
| Authenticity | *Who vouched for that digest?* | **Nobody.** No signature, certificate, key, or trust store exists in this crate |
| Candidate identity | *Does the artifact behave as claimed?* | Not this crate. `eggup-core` bounded candidate execution + caller validator |

The division of labour is precise. The caller supplies the expected digest as
`ArchiveMember::expected_sha256`; this crate computes the observed digest by
hashing as it streams and reports it three ways — `ExtractedMember::sha256()`,
`BoundMember::sha256()`, and the immediate `DigestMismatch` failure. It never
decides whether the expected value is *trustworthy*. A caller who passes
`None` for both expected facts gets an extraction with evidence and no check at
all, which is why `eggup-eggpack::archive_plan_for`
(`crates/eggup-eggpack/src/lib.rs:546-551`) always supplies both.

The crate is explicitly not the first verifier of the archive itself
(ADR-0005 §Archive precondition). It checks the archive's *internal* structure
and gzip footer, but that is malformed-input detection, not an integrity gate.
The external gate is the caller's, and `validate_acquired_archive` is the
concrete implementation.

## 12. Testing approach

52 `#[test]` functions in one `#[cfg(test)] mod tests` (`lib.rs:1512`), plus two
integration test files under `crates/eggup-archive/tests/` (7 tests) that prove
the **published** contract from outside the crate. `crates/eggup-archive/Cargo.toml:14`
already excludes `tests/` from the packaged crate, so adding the directory does
not change the published package contents.

Archive parsing is made testable without fixture binaries by constructing
archives in memory:

- `tar_file` (`1582`) — `tar::Builder` over `GzEncoder<Vec<u8>>`, with an
  explicit `kind: u8` entry type per fixture so symlink/dir/device entries are
  one struct field away.
- `zip_file` (`1604`) — `ZipWriter<Cursor<Vec<u8>>>` with deflate; names are
  stored verbatim.
- `tar_file_raw_path` (`1623`) — the important one. `Header::set_path`
  *sanitises* traversal names, so a genuinely hostile tar entry cannot be built
  through the normal API. This helper writes the raw ustar name field (bytes
  `0..100`, zeroing the `345..500` prefix) and recomputes the checksum, which is
  the actual untrusted input the extraction loop must defend against.
- `TestDir` (`1528`) — a unique temp directory per test under
  `std::env::temp_dir()`, `remove_dir_all` on drop.
- `assert_no_escape_or_residue` (`1665`) — the central escape invariant: after a
  failure, the parent must contain only the archive and at most an empty
  `.eggup-extract-*` real directory.
- `extract_with_hook` (`834`) — a `#[cfg(test)]` seam accepting a
  `MaterializationHook` that runs synchronously on the extraction thread before
  each declared write, with the write sequence number and recorded root path.
  This makes root rename/replacement races **deterministic**, with no sleeps.

### The M005 integration fixtures

| File | Tests | Demonstrates |
|---|---|---|
| `tests/m005_cleanup_contract.rs` | `cleanup_reports_ok_and_leaves_an_empty_root_for_all_four_entry_points`, `cleanup_reports_no_residue_path_on_the_success_path`, `foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched` (`#[cfg(unix)]`), `cleanup_failure_category_is_distinct_from_success` | `Ok` on a successful emptying for **each** of the four entry points; the cleaned root still exists, is a directory, has zero entries, and is removable by the caller with `fs::remove_dir`; a foreign replacement at the recorded pathname survives untouched; the success path carries no residue path |
| `tests/m005_member_write_parity.rs` | `every_case_produces_identical_outcomes_in_both_formats`, `member_write_path_exists_exactly_once_in_non_test_code`, `windows_device_aliases_are_rejected_and_near_misses_remain_usable` | one shared case table (declared-member evidence, declaration order, missing member, duplicate normalized path, size mismatch, digest mismatch, non-regular declared member, over-limit member, cleanup outcome) driven through **both** tar.gz and zip with identical outcomes; a whitespace-insensitive source guard that the member creation primitive, the bounded copy with a populated output, and the expected-value validation each have exactly one call site in non-test code; the Windows device-name regression guard |

The `Err(CleanupFailed)` path is proven **inline**, not from an integration test:
the crate exposes no portable public failure trigger, because the only
deterministic mechanism is exceeding the crate's own private cleanup bounds
(`CLEANUP_MAX_DEPTH`, `CLEANUP_MAX_ENTRIES_PER_PASS`), which are not public
constants. The inline fixture
`cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence`
(`lib.rs:2741`) exceeds the depth bound and asserts `Err(CleanupFailed)`, a
residue path, and that the crate does not report success. No external coverage
of that path is claimed.

| Group | Tests | Demonstrates |
|---|---|---|
| Hostile entry names on the untrusted path | `1711`, `1731`, `1755` | 8 traversal vectors (`../escape`, `a/../../escape`, `/etc/passwd`, `..\..\evil.exe`, `a\b`, `a//b`, `./app`, `C:/Windows/x`) written as real tar/zip entries, each rejected as `InvalidPath` with no escape and no partial extraction — including a hostile entry placed *after* a valid member |
| Positive extraction + evidence | `1805`, `1849` | allowlisted members extracted in declaration order with exact size/digest evidence for both formats |
| Owner-private permissions | `1877` (`#[cfg(unix)]`) | `0700` root, `0600` members |
| Path and output-name rejection | `1904`, `1932`, `1959` | traversal/absolute/alias/overlong paths; Windows reserved names; `COM0`/`CLOCK$`/`CONIN$` near-misses |
| Plan validation | `2024` | duplicate members, non-positive and inconsistent limits |
| Missing/duplicate archive members | `2044` | `MissingMember`, `DuplicateArchiveMember` |
| Non-regular members | `2089`, `2122` | declared tar links and special files; declared zip symlink |
| Budgets | `2143`, `2180` | per-member and aggregate decompression limits; entry count and compressed size |
| Evidence mismatch | `2210` | exact size and SHA-256 mismatches |
| Corruption | `2234` | corrupt and truncated tar.gz and zip |
| Cleanup and foreign replacement | `2276`, `2320`, `2354`, `2389`, `2441`, `2480`, `2524`, `2560`, `2789` | drop vs `persist()`; failure cleans only its own partial root; foreign non-empty directory, foreign empty directory, symlink substitution, and raced replacement all preserved at the original pathname |
| Cleanup determinism and bounds | `2610`, `2639`, `2662`, `2688`, `2710`, `2741` | deterministic races after content deletion; symlink after deletion not followed; cleanup stays bounded under concurrent file creation; cleanup that cannot empty the root fails closed with residue evidence |
| Platform reparse points | `2764` (`#[cfg(windows)]`), `3072` (`#[cfg(windows)]`) | Windows reparse-point substitution is not followed |
| Handle-relative materialization | `2883`, `2934`, `2977`, `3035`, `3115`, `3148` | root rename before first member; two-member replacement between writes; root symlink replacement; raced-in existing output; raced output symlink leaves target untouched |
| Bound handoff | `3251`, `3301`, `3339`, `3413`, `3454`, `3508`, `3555`, `3624`, `3666` | member-entry replacement and post-handoff root replacement still stage owned bytes; root rename before first write; non-zero cursor still stages the full file from byte 0; `0600` mode and exact evidence preserved |
| Core composition | `2830`, plus helpers `3192`–`3245` | an extracted member stages as an ordinary `eggup-core` artifact through `BoundSources` + `prepare_with_bound_sources` |

Focused commands:

```sh
cargo test -p eggup-archive
cargo test -p eggup-archive hostile_undeclared
cargo test -p eggup-archive bound_
cargo test -p eggup-archive normal_explicit_cleanup_empties_owned_root_and_leaves_empty_residue
cargo test -p eggup-archive --locked --test m005_cleanup_contract --test m005_member_write_parity
cargo clippy -p eggup-archive --all-targets --locked -- -D warnings
```

The full gate is `./scripts/check-local.sh`. Platform coverage matters here:
9 of the race tests are `#[cfg(unix)]` and 2 are `#[cfg(windows)]`
(`lib.rs:1875`, `2522`, `2660`, `2762`, `2975`, `3033`, `3070`, `3146`, `3411`,
`3553`, `3664`), so a Linux-only run leaves the Windows reparse-point cases
unexecuted. The M005 integration fixture
`foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched` is
`#[cfg(unix)]` for the same reason: Windows refuses to rename a directory while
its handle is held. The crate's inline `#[cfg(windows)]` reparse-point test
covers the Windows equivalent.

## 13. Known doc/code drift

One point where a reader following the prose will draw a conclusion the code
does not support. Documented here as code behavior, per the review convention
that the code is authoritative.

(M005 closed the two entries that stood here before it — the `residue_path()`
"the directory is empty" wording and the unreachable `Ok` arm — by making the
rustdoc state the two proven meanings and by making the `Ok` path reachable.
See `plans/closure/archive-extraction/005-status.md`.)

1. **ADR-0005 §Resource bounds lists "cancellation/check opportunities"; the
   crate has no cancellation API.** Budget checks happen per 16 KiB chunk, so a
   maliciously large input is stopped without reading the archive's remainder,
   but a caller cannot cancel an in-flight extraction.

## Review observations

Recorded for the parent; no code was changed.

- **Field drop order on the un-persisted path.** `ExtractedArchive` declares
  `guard` (340), `members` (341), `handles` (344), and Rust drops fields in
  declaration order — so `DirectoryGuard::drop` empties the root *before* the
  member `File`s are closed. Every explicit cleanup path does the opposite
  deliberately, with a comment: `drop(std::mem::take(&mut self.handles))` before
  `guard.cleanup()` (`399-401`, mirrored at `467-470`, `669-672`). On a platform
  where deleting an open file is denied, the drop path could therefore leave
  content behind silently, since the drop result is discarded. Whether this
  actually bites depends on `fs_at`'s Windows share mode, which I did not verify;
  the ordering asymmetry itself is certain from the code and is worth a decision.
- **`private_root_is_empty` is the one remaining pathname-based read.**
  `lib.rs:1235` uses `fs::read_dir(path)`, not the retained handle, so in the
  extraction-failure path the emptiness assertion is made against whatever
  currently occupies the recorded name. If a foreign replacement were installed
  at that path, the assertion describes the foreign directory. Deletion is fully
  handle-based; this is the only place a decision is made by name.
- **`max_archive_bytes` is not enforced on the bytes read.** It is a
  `symlink_metadata().len()` pre-flight check (`761`) and the handler reopens by
  path afterwards (`975`, `1043`). This is consistent with the caller's
  precondition, but the limit is weaker than the field name suggests.
- **Decompression is CPU-bound before the budget can bite.** Per-chunk budget
  checks bound bytes written, not CPU spent inflating them; a highly compressible
  member stops at the byte limit but the work to reach it has already happened.
  Within ADR-0005's stated bounds, and not a defect.
- **Duplicated pre-flight logic.** `extract_with_hook` (`834-860`) is a
  `#[cfg(test)]` near-copy of `extract` (`751-781`) rather than a shared
  helper with an optional hook. Two copies of the symlink/size/root-creation
  sequence can drift. M005 did not change this; the `lib.rs` module
  decomposition is deferred to Archive Extraction M006.

## 14. Cross-references

- [overview.md](overview.md) — workspace index, dependency graph, verification ladder
- [core-transaction.md](core-transaction.md) — `InstallPlan::prepare_with_bound_sources`, `BoundSources`, staged-digest revalidation
- [eggpack-adapter.md](eggpack-adapter.md) — the sole consumer adapter
- [acquisition.md](acquisition.md) — the upstream seam that produces the verified archive file
- [ADR-0005](../plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md) — the normative local archive extraction safety contract
- [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) — verification layers and transport neutrality
- Closure records: [M001](../plans/closure/archive-extraction/001-status.md),
  [M001a](../plans/closure/archive-extraction/001a-status.md),
  [M001b](../plans/closure/archive-extraction/001b-status.md),
  [M001c](../plans/closure/archive-extraction/001c-status.md),
  [M001d](../plans/closure/archive-extraction/001d-status.md),
  [M005](../plans/closure/archive-extraction/005-status.md) — requirement→evidence
  matrices, the handle-backed handoff qualification, and the cleanup-contract
  and member-write corrective
- Consumer side, read-only reference:
  [`crates/eggup-eggpack/src/lib.rs`](../crates/eggup-eggpack/src/lib.rs) lines
  430-709 — `archive_format_for_name`, `validate_acquired_archive`,
  `archive_plan_for`, `core_plan_for_archive*`, `bind_archive_members`; and
  [`tests/archive_handoff.rs`](../crates/eggup-eggpack/tests/archive_handoff.rs)
  `run_handoff` (line 231) for the full end-to-end order.
