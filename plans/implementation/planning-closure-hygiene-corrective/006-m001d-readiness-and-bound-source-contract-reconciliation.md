# Planning / Closure Hygiene Corrective C006 — M001d Readiness and Bound-Source Contract Reconciliation

Status: implemented (see `plans/closure/planning-closure-hygiene-corrective/006-status.md`)

Repository baseline: `0d2f1f06110e3949755120fa9e80ff5a5b0b4b3c`

Primary class: polish/corrective

Original plan corrected by this pass:

- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`

Related implementation / closure evidence:

- `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`
- `plans/closure/archive-extraction/001c-status.md`
- `plans/closure/planning-closure-hygiene-corrective/005-status.md`
- M001c implementation `09c953fe1fe512717584bb3e5509e79da894b35a`
- M001c/C005 closure and M001d registration `f7a18e4762bd0549349eaa6f014b0649f36594c4`
- hosted qualification run `36257884083`
- current documentation head `0d2f1f06110e3949755120fa9e80ff5a5b0b4b3c`
- current-head CI run `36257992801`

Applicable architecture:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`
- `plans/003-planning-process.md#16-public-api-discipline`

## 1. Objective

Correct the M001d implementation plan before runtime work begins.

The M001d direction is correct: M001c proved that archive member writes can remain bound to the operation-owned extraction root, but a later path-only handoff into `eggup-core` can still be redirected after extraction. However, the current M001d plan contains four planning defects that could cause a technically incorrect or unnecessarily breaking implementation:

1. its repository baseline predates the M001c implementation and closure evidence it depends on;
2. it treats a retained root handle plus member name reopened later with `open_at` as equivalent to an already-open member object, even though the member directory entry itself can be replaced after extraction and before staging;
3. its compatibility section implies removing/deprecating the existing path-source constructor, even though ordinary acquired-file consumers have a legitimate path-based source model and should not be migrated merely to support archive-bound sources;
4. it does not define readable-handle, cursor, lifetime, and cleanup semantics precisely enough for a cross-platform implementation.

C006 is documentation/planning only. It must tighten M001d into an executable, dependency-ready plan without changing runtime code.

## 2. Readiness and dependencies

C006 is dependency-ready.

Current facts are stable:

- M001c write authority is implemented at `09c953f`;
- its Section 14 stop is recorded at `plans/closure/archive-extraction/001c-status.md`;
- hosted run `36257884083` passed Stable Linux, Rust 1.89 MSRV, macOS, and Windows;
- current head `0d2f1f0` also has green CI run `36257992801`;
- C005 is closed;
- Egress M006 and Eggpack M002 are already blocked on M001d;
- no open issue or PR changes this dependency graph.

M001d MUST be treated as blocked on C006 planning correction until C006 closes.

## 3. Current evidence and detection gaps

### 3.1 Stale baseline

M001d currently records:

~~~text
Repository baseline: 1388a02356dfa01d72c63c97c1baca09ff6004a1
~~~

That predates:

- M001c implementation `09c953f`;
- M001c Section 14 closure/stop + M001d creation `f7a18e4`;
- hosted write-authority qualification `36257884083`;
- current documentation head `0d2f1f0`.

This violates the planning process requirement that handoff plans use an exact current repository baseline.

### 3.2 Root-handle + member-name token is not member-object authority

M001d Section 6.1 currently permits a handle-carrying shape described as:

~~~text
open member File (or a root-handle + name token opened only via open_at)
~~~

The latter is insufficient for the stated invariant. A retained root handle prevents a root-path replacement from redirecting the lookup, but a concurrent actor can still replace the member entry inside the owned root after extraction and before staging. Reopening `name` with `open_at(root, name)` then acquires the replacement object, not the object whose bytes were extracted and verified.

M001d must therefore distinguish:

- **object-bound authority:** an already-open member file object, or bytes copied from that already-open object into core-owned staging before authority is released;
- **directory-bound name authority:** root handle + member name reopened later.

Only the first class satisfies the M001d acceptance contract.

### 3.3 Existing path-source APIs remain legitimate

`ArtifactMember::new(id, source_path, destination)` is appropriate for ordinary already-acquired local artifacts where the caller intentionally supplies a path and accepts that source model. Existing simple consumers depend on this behavior.

The archive-specific authority problem does not justify converting all Eggup sources into handles or breaking the established path constructor.

M001d should add the smallest explicit bound-source path needed by archive extraction while preserving the existing path-source API and semantics for non-archive callers.

### 3.4 Readable-handle semantics are underspecified

M001c's current creation helper opens members for write authority:

~~~rust
options
    .write(fs_at::OpenOptionsWriteMode::Write)
    .create_new(true)
    .follow(false);
~~~

M001d cannot assume this handle is suitable for later staging reads.

The corrected plan must require one of:

- create the member with read + write authority from the outset using the same atomic handle-relative create-new/no-follow operation; or
- transfer bytes into core-owned staging from an already-open readable member object before closing/releasing that authority.

It MUST NOT regain read authority by reopening the member by pathname/name after the security boundary.

### 3.5 Cursor aliasing is underspecified

Rust `File::try_clone()` creates another `File` referring to the same underlying file handle/description; reads, writes, and seeks affect both instances' shared cursor state. M001d must not assume clones have independent offsets.

A bound-source API therefore needs explicit single-owner or cursor discipline.

### 3.6 Open-handle cleanup/lifetime is underspecified

On Windows and other platforms, open file handles can affect rename/delete behavior and cleanup ordering. M001d must define when per-member handles are consumed/closed relative to:

- archive persistence;
- core staging;
- explicit extraction cleanup;
- drop cleanup;
- error paths.

Cleanup must never fall back to pathname recursion merely because a live member handle prevents removal.

Research references used by this corrective:

- Rust `File::try_clone` shared-cursor semantics: https://doc.rust-lang.org/std/fs/struct.File.html
- `fs_at 0.2.1::OpenOptions::read` + handle-relative `open_at`: https://docs.rs/fs_at/0.2.1/fs_at/struct.OpenOptions.html

## 4. Invariants that must not regress

- M001c handle-relative archive writes remain unchanged in authority model;
- M001b cleanup remains handle-relative and foreign-preserving;
- no staged archive byte is obtained by re-resolving an extraction pathname;
- no staged archive byte is obtained by reopening a member name after the extracted member object could have been replaced;
- SHA-256 remains integrity evidence, not ownership evidence;
- the ordinary path-source API remains available for non-archive callers;
- archive support must not force an unrelated breaking migration on eggsact, stegoeggo, eggsearch, CodeGG, or other path-source consumers;
- `eggup-core` remains archive-format independent;
- no first-party unsafe/FFI;
- Rust 1.89 remains the MSRV;
- handle/cursor ownership must be deterministic;
- cleanup failure must remain fail-closed and must not mutate foreign state;
- Egress M006 and Eggpack M002 stay blocked until corrected M001d closes with hosted qualification.

## 5. Scope and non-scope

### In scope

C006 must revise M001d planning to cover:

- correct repository baseline and qualification evidence;
- exact definition of object-bound member authority;
- prohibition on root-handle + later member-name reopen as the final handoff;
- readable-handle creation/transfer semantics;
- cursor/seek/clone ownership semantics;
- member-handle lifetime and cleanup ordering;
- additive compatibility strategy preserving existing path-source APIs;
- deterministic replacement tests specifically for the member entry after extraction but before staging;
- Windows handle-lifetime tests;
- closure evidence requirements for API migration and auto-trait changes;
- registry/roadmap bookkeeping.

### Explicitly out of scope

- implementing M001d;
- editing Rust source;
- changing Cargo manifests/lockfile;
- changing the M001c implementation;
- authoring Egress M006;
- authoring Eggpack M002;
- Gregg migration;
- generic conversion of all Eggup sources to handles;
- authenticity/signatures.

## 6. Required planning changes

### 6.1 Update M001d baseline and readiness evidence

M001d must be rebased to the latest reviewed qualified planning/runtime baseline available when C006 is implemented, no earlier than:

~~~text
0d2f1f06110e3949755120fa9e80ff5a5b0b4b3c
~~~

Record:

- M001c implementation `09c953f`;
- M001c/C005 closure batch `f7a18e4`;
- hosted run `36257884083`;
- current-head green run `36257992801`.

### 6.2 Narrow acceptable bound-source shapes

Revise M001d Section 6.1 so acceptable shapes are object-bound.

Preferred options:

**A. Moved open member object.**
Archive extraction returns/owns an already-open readable `File` for each member. Core staging consumes that object directly.

**B. Immediate object-bound copy into core-owned stage.**
The already-open member object is copied into core-owned stage before the extraction authority is released. The copy operation reads the existing object, not a reopened name.

**C. Equivalent object-bound source.**
Any alternative must be demonstrably tied to the same opened member object and must not perform a name lookup after the handoff boundary.

Explicitly forbidden:

~~~text
root handle + member name -> later open_at(root, name) -> stage
~~~

as the final authority, because the directory entry may have been replaced.

### 6.3 Preserve ordinary path-source compatibility

Correct Section 9:

- keep `ArtifactMember::new(id, source_path, destination)` and its existing path-source semantics unless an independent defect is found;
- add a narrow orthogonal bound-source path for archive handoff;
- do not require existing non-archive consumers to migrate;
- do not deprecate the path constructor merely because archive extraction needs stronger authority;
- record any public type/auto-trait change caused by handle ownership.

The implementation should prefer an additive source/staging seam over changing `ArtifactMember` into a broad generic handle framework.

### 6.4 Establish readable authority at object creation

M001d must specify how the same object becomes readable for staging.

Preferred path:

- change M001c's `fs_at::OpenOptions` creation to request `read(true)` plus write/create-new/no-follow when M001d is implemented;
- continue using the returned handle for extraction;
- flush and seek the same owned object back to offset zero before staging consumption;
- never reopen by path/member name to obtain read access.

If a different implementation is chosen, it must prove the same object identity without name re-resolution.

### 6.5 Define cursor ownership

The corrected M001d plan must state:

- do not assume `File::try_clone()` provides an independent cursor;
- prefer moving a single owned member `File` into the staging operation;
- before staging reads, flush any writer state and seek the owned handle to `SeekFrom::Start(0)`;
- if cloning is unavoidable, document the shared-offset semantics and prove no aliased reader/writer can advance the cursor concurrently;
- do not expose a cloneable bound-source API unless clone semantics are explicitly safe.

Tests must include a non-zero cursor before staging and prove the full file is staged from byte zero.

### 6.6 Define handle lifetime and cleanup ordering

M001d must specify a state transition equivalent to:

~~~text
Extracted member handle owned by extraction
        |
        v
handle transferred/borrowed exclusively into staging
        |
        v
stage copy complete + handle consumed/closed
        |
        v
extraction cleanup may empty root
~~~

Required behavior:

- explicit cleanup cannot destructively race a member still being staged;
- dropping `PersistedExtraction` while bound sources are outstanding must have deterministic ownership semantics (compile-time ownership preferred over runtime booleans);
- Windows hosted tests must exercise cleanup after bound member handles are consumed/closed;
- no pathname-recursive fallback if deletion is temporarily prevented by an open handle;
- residue remains acceptable under M001b semantics.

### 6.7 Expand race tests

M001d must add a deterministic race that specifically:

1. completes extraction and retains the original member object;
2. replaces the member directory entry inside the still-owned root with foreign bytes;
3. stages from the bound source;
4. proves staged bytes are the originally extracted bytes;
5. proves the foreign replacement remains untouched.

This test is required in addition to root rename/replacement races.

## 7. Ordered work packages

1. Rebase M001d to current qualified evidence.
2. Remove root-handle + later-name reopen from the accepted authority shapes.
3. Add readable-object requirements to the chosen handoff model.
4. Correct compatibility language to preserve the existing path-source API.
5. Add cursor ownership/rewind requirements and shared-clone warning.
6. Add handle lifetime/cleanup ordering and Windows-specific acceptance evidence.
7. Add the post-extraction member-entry replacement regression test requirement.
8. Update M001d acceptance and closure evidence accordingly.
9. Reconcile archive roadmap and registry to show M001d blocked on C006 until the corrected plan is committed, then ready.
10. Write C006 closure record with zero-runtime-delta evidence.

## 8. Failure, restart, cancellation, and contention semantics

C006 changes no runtime behavior.

The corrected M001d plan must make these future semantics explicit:

- handle/read/seek failure fails closed;
- failed stage copy does not reopen the path as fallback;
- foreign replacements are never removed;
- cleanup waits through ownership sequencing rather than retry loops;
- private residue is preferred to unsafe fallback;
- no hidden downgrade from bound source to path source.

## 9. Compatibility and migration

C006 must correct M001d to an additive compatibility posture.

Path-based sources are not deprecated by default.

The future M001d implementation may add:

- a bound-source type;
- a bound-source staging constructor;
- a separate archive-to-core staging helper;

but it should preserve existing public path constructors unless an independent compatibility review justifies change.

Any type that begins owning `File` may lose `Clone`, `Eq`, or `Sync`; M001d closure must record these auto-trait/API effects and demonstrate they are limited to the new bound-source path where possible.

## 10. Required tests/checks for C006

Documentation-only checks:

- M001d baseline is corrected;
- M001d no longer treats root-handle + later-name reopen as object-bound;
- M001d explicitly preserves ordinary path-source APIs;
- M001d specifies readable authority without reopening a name;
- M001d documents `File::try_clone` shared-cursor behavior;
- M001d requires seek-to-zero/full-byte staging evidence;
- M001d defines handle lifetime/cleanup ordering;
- M001d requires member-entry replacement-after-extraction race coverage;
- Egress/Eggpack remain blocked;
- no runtime/Cargo/workflow file changes in C006.

## 11. Required verification commands

~~~text
git diff --check
git diff --name-only <c006-baseline>..HEAD
rg -n "001d|M001d|bound source|open_at|try_clone|cursor|Egress M006|Eggpack.*M002" plans/
~~~

A Cargo check is not required for C006 because it is documentation/planning only.

## 12. Documentation updates

C006 implementation updates:

- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`;
- `plans/subsystems/archive-extraction-roadmap.md`;
- `plans/registry.md`;
- `plans/closure/planning-closure-hygiene-corrective/006-status.md`.

Consumer and Eggpack roadmaps require edits only if their blocker wording becomes stale; both already point to M001d and should otherwise remain unchanged.

## 13. Acceptance criteria

C006 closes when:

- M001d uses a current exact baseline;
- M001d only accepts true member-object-bound handoff shapes;
- later `open_at(root, member_name)` is explicitly rejected as final ownership authority;
- existing path-source APIs are preserved for non-archive callers;
- readable-handle creation/transfer is specified without pathname re-resolution;
- cursor ownership and seek-to-zero behavior are explicit;
- cleanup/lifetime ordering is explicit across Unix and Windows;
- post-extraction member-entry replacement is a required deterministic regression;
- Egress M006 and Eggpack M002 remain blocked on M001d;
- the C006 diff is documentation-only.

After C006 closure, M001d returns to ready-for-handoff.

## 14. Stop conditions

Stop and write a broader ADR/API plan if correcting M001d reveals that:

- the only sound implementation requires changing all `ArtifactMember` source semantics;
- existing non-archive path consumers cannot coexist with a separate bound-source staging path;
- a sound object-bound source requires first-party unsafe/FFI;
- Windows cannot retain/read/transfer the same opened member object safely under Rust 1.89;
- cleanup ownership cannot be represented without ambiguous shared lifetimes.

Do not paper over these with a root-handle + name reopen.

## 15. Closure evidence required

Record:

- C006 implementation SHA;
- corrected M001d baseline;
- exact M001d sections changed;
- before/after authority-shape language;
- path-source compatibility disposition;
- cursor/lifetime rules added;
- registry/roadmap state;
- proof that Egress/Eggpack remain blocked;
- zero-runtime-delta file list.

## 16. Handoff notes

C006 is a planning-quality correction, not a runtime redesign.

The intended M001d implementation remains narrow: preserve the ordinary path-source model, add one archive-specific object-bound source/staging seam, and consume the exact already-open extracted member object without any later namespace lookup.

The strongest practical default is:

~~~text
create member read+write/create-new/no-follow via retained root handle
        -> write/verify
        -> flush
        -> transfer the same owned File
        -> seek to 0
        -> core stage from that File
        -> close/consume File
        -> allow extraction cleanup
~~~

The implementation agent may choose an equivalent object-bound design, but not a name-reopen design.
