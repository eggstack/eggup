# Acquisition Transport Milestone 016 — adapter publication-coupling corrective

Status: implemented

Repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`

Branch: `plans/corrective-batch-2026-10-23`

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Closed work affected:
- plans/implementation/acquisition-transport/013-adapter-assertion-and-failure-typing-corrective.md
- plans/closure/acquisition-transport/013-status.md
- plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md
- plans/closure/acquisition-transport/014-status.md

Finding of record:
- The parent session's verification of the publication-coupling defect,
  recorded in this plan and in `plans/closure/acquisition-transport/016-status.md`.
  The registry row for this milestone is drafted (not applied) in the closure
  record's appendix; `plans/registry.md` and `plans/subsystems/**` are owned by
  the parent session.

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md

Primary class: invariant

## 1. Objective

Record the **publication coupling** that the seam's next release creates for
its in-workspace dependents, correct the two documents that currently assert
the opposite, and add a regression guard so the coupling cannot be forgotten
again.

The seam's next publication is `eggup-acquisition 0.2.0` and it is
**source-breaking** (M013: `AcquisitionError::Io(String)` became
`Io { operation, source }`, and the enum derives `Debug` only). `0.2.0` does
**not** satisfy the requirement the two adapters declare today
(`eggup-acquisition = { version = "0.1.0", … }` → `^0.1.0`), and it does not
satisfy the leaf adapter's exact pin (`=0.1.3`) either. The coupling is
therefore **load-bearing at publication time**, and it is currently recorded
nowhere except as a consequence buried in M013's closure.

Deliverables:

1. A **machine-checkable** declaration of the seam's next release in the seam's
   own `CHANGELOG.md` `Unreleased` section, plus the exact publication-time
   widening each in-workspace dependent requires.
2. A correction of the **false compatibility consequence** in
   `crates/eggup-curl/CHANGELOG.md` and `crates/eggup-eggfetch/CHANGELOG.md`,
   which currently state that a future republication "would be a non-breaking
   `0.1.3`" and that "no consumer migration is required".
3. A correction of the **duplicate `## Unreleased` heading** in
   `crates/eggup-acquisition/CHANGELOG.md` (two headings today: line 3 and line
   96).
4. A correction of the **`User-Agent` grammar mismatch** in
   `crates/eggup-curl/src/lib.rs`: the rustdoc says `SP` is "permitted only
   between two non-`SP` characters", while `validate_user_agent` accepts runs of
   interior `SP`.
5. A **regression guard** in `crates/eggup-acquisition/tests/` that fails when
   a dependent's declared requirement does not accept the seam's declared next
   release and no changelog records the required widening.
6. A **dated addendum** on the M014 plan and closure recording that the
   "non-breaking `0.1.3`" consequence stated there is superseded.

## 2. Why this milestone is ready

- The seam's next release is already **mandated** and **recorded**:
  `crates/eggup-acquisition/CHANGELOG.md` (`Unreleased`) states that the next
  publication "is therefore a source-breaking release (`0.2.0`, or the 0.2/1.0
  boundary release), **not** a `0.1.4`", and
  `plans/closure/acquisition-transport/013-status.md` repeats it under "The
  `0.2.0` consequence". Nothing about *that* decision is open; what is missing
  is its **coupling** to the dependents.
- The dependents' declared requirements are **present in the tree** and
  unchanged by this milestone, so the coupling can be recorded without
  touching a single manifest.
- The defect is **reproducible today**: `cargo package` resolves a crate's
  dependency from the **registry**, not from the in-tree path dependency, so
  both adapters already fail their own package verification (§3.3). The
  milestone therefore does not depend on a future publication to be
  demonstrable.
- The release workflow already owns the exact-pin cascade and the
  publish-in-dependency-order rule
  (`.opencode/skills/release-workflow/SKILL.md`), so this milestone only has to
  **apply** that rule to the acquisition subtree and record it.

## 3. Current implementation evidence

### 3.1 The seam's next release is `0.2.0` and is source-breaking

- `crates/eggup-acquisition/CHANGELOG.md` (`## Unreleased`): "The **next**
  publication of this crate is therefore a source-breaking release (`0.2.0`, or
  the 0.2/1.0 boundary release), **not** a `0.1.4`."
- `plans/closure/acquisition-transport/013-status.md` § "The `0.2.0`
  consequence": the same statement, plus the reason — the `Io` variant was
  retyped and `Clone`/`PartialEq`/`Eq` were dropped.
- The published `eggup-acquisition 0.1.3` artifact contains **neither**
  `__adapter_io` nor `__adapter_bound_detail`: `grep -c` over
  `~/.cargo/registry/src/index.crates.io-*/eggup-acquisition-0.1.3/src/lib.rs`
  returns **0** for both names.

### 3.2 The dependents' declared requirements

| Dependent | Manifest | Declared `eggup-acquisition` requirement | Accepts `0.2.0`? |
|---|---|---|---|
| `eggup-curl` | `crates/eggup-curl/Cargo.toml:24` | `version = "0.1.0"` → `^0.1.0` | **No** |
| `eggup-eggfetch` | `crates/eggup-eggfetch/Cargo.toml:24` | `version = "0.1.0"` → `^0.1.0` | **No** |
| `eggup-eggpack` | `crates/eggup-eggpack/Cargo.toml:22` | `version = "=0.1.3"` | **No** |
| `eggup-transport-footprint` | `crates/eggup-transport-footprint/Cargo.toml:16` | `version = "0.1.0"` → `^0.1.0` | **No** |

The acceptance facts are not assumptions; they were proven with Cargo's own
semver implementation in an isolated `/tmp` project (§3.5, command 4):
`^0.1.0` → **false**, `=0.1.3` → **false**, `^0.2.0` → **true**, `=0.2.0` →
**true**.

### 3.3 The coupling is already load-bearing: both adapters fail `cargo package`

`cargo package` resolves a dependency from the **registry**, so the packaged
adapter is built against the **published** seam, not the in-tree path
dependency. Both adapters call the two `#[doc(hidden)]` adapter entry points
that exist **only** in the unpublished seam:

- `crates/eggup-curl/src/lib.rs` → `eggup_acquisition::__adapter_bound_detail`
  and `AcquisitionError::__adapter_io`.
- `crates/eggup-eggfetch/src/lib.rs` → the same two names.

Empirically (§3.5, commands 1 and 2):

- `cargo package -p eggup-curl --locked --offline` → **exit 101**,
  `error: failed to verify package tarball`, 11 compile errors
  (`E0425: cannot find function '__adapter_bound_detail' in crate
  'eggup_acquisition'`, `E0599: no variant or associated item named
  '__adapter_io'`).
- `cargo package -p eggup-eggfetch --locked --offline` → **exit 101**,
  `error: failed to verify package tarball`, 4 compile errors of the same two
  kinds.

Two consequences follow, and they are the point of this milestone:

1. The adapters **cannot be published** in their current state — not because of
   a policy, but because they do not compile against the published seam.
2. The seam's `0.2.0` release makes the requirement widening **mandatory**:
   after the seam's version moves, `^0.1.0` no longer resolves at all, so the
   adapters' manifests must widen in the same publication milestone.

`eggup-eggpack` still packages cleanly today (`exit 0`) because its `=0.1.3`
pin still resolves to the published `0.1.3` and the leaf adapter does not use
the new adapter entry points; its pin is nevertheless **latently** broken,
because `=0.1.3` does not accept `0.2.0`.
`eggup-transport-footprint` is `publish = false` and is never published; its
`^0.1.0` requirement is nevertheless **latently** broken for the same reason,
and it must be widened so the workspace still resolves after the version bump.

### 3.4 The two documents that assert the opposite

- `crates/eggup-curl/CHANGELOG.md:80-81` (M014 entry): "No existing public
  signature was added, removed, or retyped, so **no consumer migration is
  required**, and a future republication would be a non-breaking `0.1.3`."
- `plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md:211-216`
  (§6.1) and `:439` (§9.3) repeat the same consequence.

Both are false once the seam's next release is `0.2.0`:

- the adapters' `eggup-acquisition` requirement **must** widen to `0.2`;
- the workspace is **version-lockstep** (`Cargo.toml` `[workspace.package]
  version = "0.1.3"`, inherited by all eight crates), so the seam's `0.2.0`
  release moves the workspace version to `0.2.0` and the adapters' own next
  published versions are therefore **`0.2.0`, not `0.1.3`**;
- a consumer that depends on the seam **directly** inherits the seam's source
  break, and so does a consumer of an adapter once that adapter is republished
  with the widened requirement — because the seam's `AcquisitionError` is part
  of the adapter's own public surface
  (`AcquisitionTransport::fetch` returns `AcquisitionError`).

What is **not** false, and stays true: the adapters' **own** API additions are
additive, and nothing published has changed.

### 3.5 The `User-Agent` grammar mismatch

- `crates/eggup-curl/src/lib.rs:222` documents
  `SP = %x20 ; permitted only between two non-SP characters`.
- `crates/eggup-curl/src/lib.rs:286-327` (`validate_user_agent`) accepts every
  `SP` (`'\u{20}' => true` at `:313`) and rejects only **leading** and
  **trailing** `SP` (`:308-311`). The value `"a  b"` (0x61 0x20 0x20 0x62) is
  therefore **accepted**, and `build_curl_args` transmits it verbatim as one
  literal argv element.
- RFC 7230 §3.2 defines `field-content = field-vchar [ 1*( SP / HTAB )
  field-vchar ]`, whose interior run is `1*` — repeated interior `SP` **is**
  permitted by the HTTP field-value grammar. The implemented rule (no C0/C1
  controls, no `DEL`, no leading/trailing `SP`, 512-byte bound) is therefore a
  **subset** of a well-formed HTTP field value, and the documented rule is
  **stricter** than both the implementation and the RFC.

### 3.6 The duplicate `## Unreleased` heading

`crates/eggup-acquisition/CHANGELOG.md` carries **two** `## Unreleased`
headings: line 3 (the real one, carrying the M013 entry) and line 96 (a stray
one between the `0.1.3` and `0.1.2` sections). The stray section's body is the
two-line boilerplate "Nothing in this section has been published. `0.1.3` is
the published baseline on crates.io, and no publication milestone authorizes
these entries." — which the top section already carries verbatim at lines 5-6.
The stray heading therefore holds **no unique content** and is removed rather
than merged.

### 3.7 Commands used to gather the evidence above

1. `cargo package -p eggup-curl --locked --offline` → **exit 101**, 11 errors,
   `failed to verify package tarball`.
2. `cargo package -p eggup-eggfetch --locked --offline` → **exit 101**, 4
   errors, `failed to verify package tarball`.
3. `cargo package -p eggup-eggpack --locked --offline` → **exit 0**.
4. An isolated `/tmp` project resolving `semver = "1"` offline:
   `^0.1.0` accepts `0.2.0` → **false**; `=0.1.3` → **false**; `^0.2.0` →
   **true**; `=0.2.0` → **true**; `^0.1.3` → **false**.

## 4. Invariants that must not regress

1. **Transport neutrality.** `eggup-curl` must not learn that `eggup-eggfetch`
   exists, and `eggup-acquisition` must not depend on any adapter. The guard
   this milestone adds reads the dependents' **manifests and changelogs as
   text**; it does not depend on them, so it cannot break the boundary.
2. **The seam stays zero-dependency.** `eggup-acquisition` keeps **zero**
   `[dependencies]`. The guard therefore implements the semver comparison
   itself instead of adding a `semver` dev-dependency.
3. **No manifest is changed.** No `Cargo.toml` version requirement is widened
   and no version is bumped by this milestone. The widening is *recorded*, and
   performed by the publication milestone.
4. **No publication act.** No crate is published, no tag is moved, no
   crates.io write of any kind occurs.
5. **The seam's public API is unchanged.** `crates/eggup-acquisition/src/**`
   is not modified; the guard lives in `tests/` and adds no public item.
6. **The adapter's public API and behavior are unchanged.** The `User-Agent`
   correction is a **documentation** correction: the accepted value set, the
   rejection set, the argv emission, and the default are untouched.
7. **No authenticity claim.** Nothing in the new prose implies publisher trust,
   signature validity, or provenance.
8. **Existing tests are not weakened.** No existing assertion is removed or
   relaxed; the existing `assertion_guard` and `typed_io_contract` suites still
   pass.
9. **The published artifacts are unchanged.** `eggup-acquisition 0.1.3`,
   `eggup-eggfetch 0.1.3`, `eggup-eggpack 0.1.3`, and `eggup-curl 0.1.2` keep
   their published shape; only their **unpublished** changelog sections change.

## 5. Scope

### In scope

- `crates/eggup-acquisition/CHANGELOG.md`: one `## Unreleased` section, at the
  top, carrying the M013 entry, the explicit **next-release declaration**, and
  the **publication-coupling record** with the per-dependent widening.
- `crates/eggup-acquisition/tests/publication_coupling.rs`: the regression
  guard.
- `crates/eggup-acquisition/tests/assertion_guard.rs`: the new test file is
  added to the scanned sources, and M016's behaviors are added to the named
  witness list.
- `crates/eggup-curl/CHANGELOG.md`: the corrected compatibility consequence.
- `crates/eggup-eggfetch/CHANGELOG.md`: the corrected compatibility consequence.
- `crates/eggup-curl/src/lib.rs`: the `User-Agent` grammar correction (rustdoc
  only).
- This plan, the M016 closure record, and the dated addenda on the M014 plan
  and closure.

### Explicitly out of scope (non-goals)

- **Any `Cargo.toml` change.** No requirement is widened, no version is bumped,
  no workspace version is moved, no pin is re-pinned. The publication milestone
  owns all of it; §9.3 lists the exact action list for it.
- **Any publication act**, including `cargo publish`, `cargo package` as a
  publication step, and any tag movement.
- Any change to `crates/eggup-acquisition/src/**` — the seam's public API and
  behavior are frozen for this milestone.
- Any change to `crates/eggup-eggfetch/src/**`, `crates/eggup-eggpack/**`,
  `crates/eggup-core/**`, `crates/eggup-service/**`, `crates/eggup-archive/**`,
  or `crates/eggup-transport-footprint/**`.
- Any change to `crates/eggup-curl/README.md` (the `User-Agent` grammar block
  is repeated there and is **wrong in the same way**; the file is owned by
  another writer and is reported as a finding instead).
- `architecture/**`, `docs/**`, `AGENTS.md`, `scripts/**`, `.github/**`,
  `.opencode/skills/**`, the root `CHANGELOG.md`, `plans/registry.md`,
  `plans/subsystems/**` — all owned by the parent session or another writer.
  The intended prose is drafted in the closure record, not applied.

## 6. Required production changes

### 6.1 The seam's changelog: one section, an explicit next release, and the coupling record

`crates/eggup-acquisition/CHANGELOG.md` must end up with **exactly one**
`## Unreleased` heading, **at the top**, carrying:

1. the existing M013 entry (unchanged in substance);
2. an explicit, machine-readable **next-release declaration**:
   `Declared next release: `0.2.0`` — source-breaking;
3. a **publication-coupling record** naming every in-workspace dependent and
   the requirement it must carry at publication time, in a form the guard can
   read:

   ```text
   - `eggup-curl`: `eggup-acquisition` = "0.2" (widened from "0.1.0").
   - `eggup-eggfetch`: `eggup-acquisition` = "0.2" (widened from "0.1.0").
   - `eggup-eggpack`: `eggup-acquisition` = "=0.2.0" (re-pinned from "=0.1.3").
   - `eggup-transport-footprint`: `eggup-acquisition` = "0.2" (widened from "0.1.0").
   ```

The stray second `## Unreleased` heading (line 96) and its duplicated
boilerplate are removed; no unique content is lost (§3.6).

### 6.2 The adapters' changelogs: the corrected compatibility consequence

Both `crates/eggup-curl/CHANGELOG.md` and `crates/eggup-eggfetch/CHANGELOG.md`
must state, in their `## Unreleased` section:

- the seam's next release is `eggup-acquisition 0.2.0` and it is
  **source-breaking**;
- the adapter's `eggup-acquisition` requirement **must widen to `0.2`** at
  publication time, because `0.2.0` does not satisfy `^0.1.0`;
- the adapter's own next published version is therefore **not `0.1.3`** — the
  workspace is version-lockstep, so it is `0.2.0`;
- a consumer that depends on the seam **directly**, and a consumer of the
  adapter once it is republished with the widened requirement, **inherit the
  seam's source break** (`AcquisitionError::Io` retyped; `Clone`/`PartialEq`/
  `Eq` dropped) and must adapt their matchers;
- the adapter's **own** API additions remain additive, and the **published**
  artifact is unchanged;
- the widening is **recorded, not performed**: no publication, no version
  bump, no manifest change is authorized by this milestone.

Each of the two changelogs must also carry the machine-readable marker the
guard reads:

```text
`eggup-acquisition` = "0.2" (widened from "0.1.0").
```

### 6.3 The `User-Agent` grammar: reword the documentation, keep the behavior

**Decision: reword the rustdoc to match the implementation. The accepted value
set does not change.**

Justification:

- The implemented rule is a **subset** of what RFC 7230 §3.2's `field-content`
  permits (its interior run is `1*( SP / HTAB )`, so repeated interior `SP` is
  permitted), and it refuses everything that could break the request head
  (C0 controls including `CR`/`LF`/`NUL`, `DEL`, C1 controls, leading and
  trailing `SP`). The behavior is therefore already **correct**; the
  **document** was wrong.
- Enforcing the documented rule instead would **narrow** the accepted set for
  no security gain: repeated interior `SP` cannot inject a header, cannot
  become an option (it is one literal argv element), and cannot escape the
  header. The `CR`/`LF` refusal is the actual injection defense, and it stays.
- A behavior change here would be a **consumer-visible narrowing** of a
  published adapter's accepted inputs, introduced for the sake of matching a
  sentence. The correction must be the cheaper and safer direction: fix the
  sentence.

The rustdoc grammar block becomes:

```text
user-agent = 1*( VCHAR / SP / obs-text )
VCHAR      = %x21-7E          ; printable ASCII, excluding SP
SP         = %x20            ; permitted inside the value; a run of SP is
                              ; permitted, but a leading or trailing SP is not
obs-text   = %xA0-10FFFF     ; any non-ASCII scalar value (C1 excluded)
```

and the prose that follows it already states the leading/trailing refusal, so
the two now agree. `MAX_USER_AGENT_BYTES`'s rustdoc does **not** repeat the
grammar (it points at the constructor), so it needs no change.

### 6.4 The regression guard

`crates/eggup-acquisition/tests/publication_coupling.rs` reads, at compile
time (no filesystem discovery, no network, no wall clock):

- the seam's own `CHANGELOG.md` — the `## Unreleased` section and the declared
  next release;
- every in-workspace dependent's `Cargo.toml` — its declared `eggup-acquisition`
  requirement;
- every dependent that has a changelog — its own `## Unreleased` section.

and asserts, for each dependent:

> **EITHER** the declared requirement already accepts the declared next
> release, **OR** a changelog records a requirement for that dependent that
> does accept it.

Two further assertions ride along:

- the seam's changelog carries **exactly one** `## Unreleased` section, so the
  guard cannot read a declaration from the wrong section;
- the two **published** adapters' `Unreleased` sections record the required
  widening, and neither of them claims a republication version whose
  dependency range excludes the seam's declared next release (this is the
  direct guard on the false "non-breaking `0.1.3`" claim).

The guard implements the Cargo semver comparison itself, for the requirement
forms this workspace uses (`^x.y.z`, `=x.y.z`, `~x.y.z`, and a bare version),
and **fails loudly on an unrecognized form** rather than assuming acceptance.
No new dependency is added: `eggup-acquisition` keeps zero `[dependencies]`.

## 7. Ordered work packages

- **WP1 — Plan.** This document, with the grammar decision (§6.3), the coupling
  record format (§6.1), and the guard's assertions (§6.4) fixed before any
  code is written.
- **WP2 — The guard.** `crates/eggup-acquisition/tests/publication_coupling.rs`
  and the `assertion_guard.rs` wiring.
- **WP3 — Red-first proof.** Run the guard against the uncorrected tree and
  record the failure (§15).
- **WP4 — The seam's changelog.** One `## Unreleased` section, the next-release
  declaration, the coupling record.
- **WP5 — The adapters' changelogs.** The corrected compatibility consequence
  and the widening marker.
- **WP6 — The `User-Agent` grammar.** `crates/eggup-curl/src/lib.rs` rustdoc.
- **WP7 — The M014 addenda.** Dated addenda on the M014 plan and closure.
- **WP8 — Verification.** The §11 commands; exact output recorded.
- **WP9 — Closure.** `plans/closure/acquisition-transport/016-status.md`, with
  the registry and roadmap rows drafted for the parent.

## 8. Failure, cancellation, restart, and contention semantics

- **The guard fails loudly; it never fails silently.** A missing next-release
  declaration, an unrecognized requirement form, a dependent whose requirement
  cannot be parsed, or a dependent with neither an accepting requirement nor a
  recorded widening are each a **test failure with a message naming the
  crate, the requirement, and the declared next release**. The guard does not
  treat an unknown form as acceptance, and it does not skip a dependent.
- **The guard reads text; it mutates nothing.** It has no file, network, clock,
  or process dependency, so it cannot leave state behind and cannot race with
  a concurrent writer. It is deterministic and order-independent.
- **No runtime behavior changes.** The seam, both adapters, and the leaf adapter
  keep their exact behavior; nothing in this milestone adds a wait, a lock, a
  file, or a child process.
- **A wrong correction is loud, not silent.** If the adapters' changelogs are
  edited to claim a republication version whose dependency range excludes the
  seam's next release, the guard fails. If the seam's next-release declaration
  is removed, the guard fails. If a dependent is added to the workspace without
  a recorded widening, the guard fails once that dependent is listed in the
  guard's table — the guard's table is the list of in-workspace dependents, and
  a new dependent is a deliberate edit to it.
- **Restart / retry / contention.** Not applicable: no runtime state exists.

## 9. Compatibility and migration

### 9.1 What is source-breaking for a consumer of the published artifacts

**Nothing published changes.** The published `eggup-acquisition 0.1.3`,
`eggup-eggfetch 0.1.3`, `eggup-eggpack 0.1.3`, and `eggup-curl 0.1.2` keep
their current shape and behavior. Every change in this milestone is to
**unpublished** content (changelog `Unreleased` sections, one rustdoc
sentence) plus a new test.

### 9.2 What a consumer inherits once the coupling is published

- A consumer that depends on the seam **directly** (`eggup-acquisition =
  "0.1.3"`) keeps resolving `0.1.3` — a caret does not cross a major boundary
  — and is therefore **unaffected** until it opts into `0.2`.
- A consumer that opts into `eggup-acquisition 0.2` **inherits the seam's
  source break**: `AcquisitionError::Io(String)` became
  `Io { operation, source }`, and `Clone`/`PartialEq`/`Eq` were dropped.
- A consumer of `eggup-curl 0.1.2` or `eggup-eggfetch 0.1.3` is unaffected
  today, for the same caret reason. Once the adapters are republished with the
  widened `0.2` requirement, **their** consumers inherit the seam's source
  break too, because the seam's `AcquisitionError` is part of the adapters'
  public surface.

### 9.3 The exact publication-time action list (recorded, not performed)

The publication milestone must, in this order:

1. **Bump the workspace version** `0.1.3` → `0.2.0` (`Cargo.toml`
   `[workspace.package] version`). The workspace is version-lockstep, so all
   eight crates move with it.
2. **Widen/re-pin every dependent's `eggup-acquisition` requirement**:
   `eggup-curl` `0.1.0` → `0.2`; `eggup-eggfetch` `0.1.0` → `0.2`;
   `eggup-eggpack` `=0.1.3` → `=0.2.0`; `eggup-transport-footprint` `0.1.0`
   → `0.2`. `eggup-transport-footprint` also carries optional
   `eggup-curl`/`eggup-eggfetch` requirements (`0.1.0`) that must widen with
   their crates.
3. **Re-pin `eggup-eggpack`'s other exact pins** (`eggup-core = "=0.1.3"`,
   `eggup-archive = "=0.1.3"`) to the new workspace version, because the
   lockstep bump moves those crates too.
4. **Publish in dependency order**: `eggup-acquisition 0.2.0` (the seam) →
   `eggup-curl 0.2.0` and `eggup-eggfetch 0.2.0` (the adapters) →
   `eggup-eggpack 0.2.0` (the leaf). `eggup-transport-footprint` is
   `publish = false` and is **never** published.
5. **Qualify with a registry-resolved `cargo package`** for each published
   crate — the check that already fails today (§3.3) and that the widening
   makes pass.

### 9.4 Migration

None is possible or required now: nothing published changed. The migration
that the **future** publication forces is the seam's own (`AcquisitionError::Io`
matchers, and the dropped derives), inherited by the adapters' consumers as
described in §9.2.

## 10. Required tests

| # | Test | Asserts |
|---|---|---|
| T1 | `seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release` | The seam's changelog has exactly **one** `## Unreleased` heading, it is the **first** section, and it declares a parseable next release |
| T2 | `dependent_requirements_accept_the_next_release_or_record_the_widening` | For each of the four in-workspace dependents: the declared requirement accepts the declared next release, **or** a changelog records a requirement for that dependent that does |
| T3 | `adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version` | Both published adapters' `Unreleased` sections record the widening, and neither claims a "non-breaking" republication at a version whose dependency range excludes the seam's next release |

The three tests are pure: no subprocess, no network, no wall clock, no
filesystem discovery — every input is embedded at compile time with
`include_str!`.

T1, T2, and T3 are added to the `assertion_guard`'s named witness list, and the
new file is added to the guard's scanned sources, so the existing vacuity and
ignore-reason guards cover it too.

The pre-existing `eggup-acquisition` suites (`assertion_guard`,
`typed_io_contract`) must still pass unchanged.

## 11. Required verification commands

```sh
cd /Users/davidbowman/projects/eggup
cargo fmt -p eggup-acquisition -p eggup-curl -p eggup-eggfetch -- --check
cargo clippy -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --all-targets --locked --offline -- -D warnings
cargo test -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --locked --offline
cargo doc -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --no-deps --locked --offline
```

The workspace-wide gate (`./scripts/check-local.sh`) is run by the **parent
session**, not by this milestone.

The red-first proof (§15) additionally uses:

```sh
cargo test -p eggup-acquisition --locked --offline publication_coupling
```

## 12. Documentation updates

| Document | Change |
|---|---|
| `crates/eggup-acquisition/CHANGELOG.md` | One `## Unreleased` section, at the top, carrying the M013 entry, the explicit next-release declaration (`0.2.0`, source-breaking), and the publication-coupling record with each dependent's required widening |
| `crates/eggup-curl/CHANGELOG.md` | The M014 entry's compatibility consequence is **corrected**: the false "a future republication would be a non-breaking `0.1.3`" claim is replaced by the true consequence (requirement must widen to `0.2`; the adapter's next version is not `0.1.3`; the seam's source break is inherited by consumers); the widening marker is added; the "nothing published / no migration yet" disclaimer is kept accurate |
| `crates/eggup-eggfetch/CHANGELOG.md` | The same correction and marker |
| `crates/eggup-curl/src/lib.rs` | The `User-Agent` grammar block: the `SP` line is reworded to match the implementation and the RFC (§6.3). No behavior change |
| `crates/eggup-acquisition/tests/assertion_guard.rs` | The new test file joins the scanned sources; M016's three behaviors join the named witness list |
| `plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md` | Dated addendum: the "non-breaking `0.1.3`" consequence in §6.1 and §9.3 is **superseded**; the `SP` grammar line in §6.3 is superseded |
| `plans/closure/acquisition-transport/014-status.md` | Dated addendum in the existing addendum style |
| `plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md` | This plan |
| `plans/closure/acquisition-transport/016-status.md` | The closure record |

**Not edited here** (owned by the parent session or another writer):
`plans/registry.md`, `plans/subsystems/**`, the root `CHANGELOG.md`,
`architecture/**`, `docs/**`, `AGENTS.md`, `scripts/**`, `.github/**`,
`.opencode/skills/**`, `crates/eggup-curl/README.md`, and every other crate.
The intended prose for them is drafted in the closure record.

## 13. Acceptance criteria

1. `crates/eggup-acquisition/CHANGELOG.md` has **exactly one** `## Unreleased`
   heading, at the **top**, carrying the M013 entry, an explicit next-release
   declaration, and the publication-coupling record.
2. The seam's changelog names **all four** in-workspace dependents and the
   requirement each must carry at publication time.
3. `crates/eggup-curl/CHANGELOG.md` and `crates/eggup-eggfetch/CHANGELOG.md`
   state that the seam's next release is `0.2.0` and source-breaking, that the
   adapter's requirement must widen to `0.2`, that the adapter's own next
   version is therefore not `0.1.3`, and that a consumer of the seam inherits
   the source break — and each keeps an accurate "nothing published / no
   migration yet" disclaimer.
4. Neither adapter's changelog still claims that a future republication would
   be a non-breaking `0.1.3`.
5. The `User-Agent` rustdoc and the implementation agree, and the decision is
   justified in this plan and in the closure record.
6. `crates/eggup-acquisition/tests/publication_coupling.rs` exists and asserts
   the coupling, with the **red-first proof** recorded in the closure: the
   guard **fails** before the changelog correction and **passes** after it.
7. The guard is added to the `assertion_guard`'s scanned sources and witness
   list, and all three guard suites pass.
8. `eggup-acquisition` still has **zero** `[dependencies]`, `eggup-curl` still
   depends only on `eggup-acquisition`, and `eggup-eggfetch` still does not
   know `eggup-curl` exists.
9. No `Cargo.toml` was changed and no version was bumped.
10. All §11 commands pass.
11. The closure record exists at `plans/closure/acquisition-transport/016-status.md`
    and this plan's `Status:` header reads `implemented`.
12. The M014 plan and closure carry a dated addendum recording the
    supersession.

## 14. Stop conditions

Stop and report rather than improvise if any of these occur:

- The guard cannot be implemented without adding a dependency to
  `eggup-acquisition` (it can: the semver comparison is implemented in-crate).
- The guard would have to **import** an adapter to read its manifest — it must
  not; it reads text.
- Correcting the `User-Agent` grammar appears to require a **behavior** change
  to the accepted value set (it does not: the implementation is already
  conformant and the documentation is what is wrong).
- Any acceptance criterion in §13 cannot be met within this scope.
- The seam's changelog turns out to contain **unique content** in the stray
  `## Unreleased` section that cannot be merged into the top section without
  loss (it does not: §3.6).
- A dependent's declared requirement turns out to be one the guard cannot
  parse — the guard then fails loudly and the form must be added explicitly
  rather than assumed.

## 15. Closure evidence required

The closure record `plans/closure/acquisition-transport/016-status.md` must
carry:

1. A requirement → evidence matrix, each requirement tied to a `file:line`
   anchor and the **exact** command and its result.
2. The **red-first proof**, with the exact failing command and its output
   **before** the changelog correction, and the exact passing command and its
   output **after** it.
3. Per-platform results reported **separately** (Linux / macOS / Windows), with
   a platform that was not run reported as *not run* — never inferred.
4. The exact §11 commands and their outputs.
5. An explicit statement that the **parent session** owns `plans/registry.md`,
   `plans/subsystems/**`, the root `CHANGELOG.md`, `architecture/**`, and
   `crates/eggup-curl/README.md`, with the **intended** prose for each drafted
   (not applied).
6. Unresolved findings with explicit severities, and — because
   **medium-or-higher is the blocking-severity threshold** — any finding still
   open at that severity named as a follow-on corrective in §16.
7. The compatibility review: exactly what is (and is not) source-breaking for
   a consumer of the published artifacts, and the statement that **no
   republication was performed**.
8. A documentation/operations evidence table: every document changed and what
   changed in it.
9. An **appendix** with the ready-to-paste `plans/registry.md` row and the
   ready-to-paste `plans/subsystems/acquisition-transport-roadmap.md` row,
   each within the workspace's row-size limit.

## 16. Handoff notes

- The finding this milestone closes is **the coupling being unrecorded and one
  document asserting the opposite**. It is not "the adapters fail to
  package" — that is the *symptom*, and it is loud. The defect that mattered is
  that the compatibility consequence was documented as **non-breaking**, so
  nobody widened the requirement.
- The publication milestone is **not** this milestone. §9.3 is its action
  list. Until it runs, `eggup-curl` and `eggup-eggfetch` **cannot** be
  published, and that is recorded rather than fixed.
- `crates/eggup-curl/README.md` repeats the same (now corrected) `User-Agent`
  grammar, and `architecture/curl-adapter.md` repeats it a third time. Both
  files are owned by another writer; both are reported in the closure record
  and must be corrected before the adapter's next publication.
- **No follow-on corrective is required by this milestone** unless the parent
  session judges the residual doc drift (README + architecture deep dive) to be
  medium-or-higher.
- The parent session owns all git mutations (commit, tag, publication). This
  milestone produces working-tree changes only.
