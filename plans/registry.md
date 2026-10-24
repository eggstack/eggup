# Eggup Planning Registry

Status: active

Last implementation/closure baseline reviewed: `2fbb3c6f75ae536b53a7d30087c0b0edef7c74ec` (Service M011 post-publication fixture and test-harness correction; final hosted run `37883703334` green; package source `feb6ae5`, checksum `9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`). Downstream handoff refreshed 2026-10-09 against `dbowm91/wg-basic@ff40f85`.

Superseded baseline: `44d7fdf6d31d67a2e2c8b0f62aa4f00ec365daeb` (Core M010/M011 final hosted-qualified line; run `37376971555` green on Stable/MSRV/macOS/Windows; subsequent `44d7fdf6` is docs-only governance correction)

Latest planning registration head before this header refresh: `d6a4722c30494e2aee29c2d8ee58a0eb69f51c26` (Core M012/M013, Acquisition M011, Archive M004, Eggpack M005 registered; Acquisition M010 unblocked; Gregg/EggPool versioned-handoff gates reconciled).

Latest reviewed pre-C003 planning/status baseline: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` (M003 producer-gate execution/status record; historical)

Latest registration reconciliation before this header refresh: `8f1d10db370c0fda2b84f2665751c00b15c2adb2` (service 0.1.2 authorization wording reconciled after the new M009 registration)

Newest qualified runtime delta: the Core M010/M011 corrective line, final run `37376971555` green on all four lanes. M003a (`39ff6260`) is historical adapter evidence; `git log 39ff626..HEAD -- crates/` returns `e548b64`, `0b8cb98`, `b485228`, `4a95e4d`. The two `0b8cb98` acquisition fixes published 2026-10-06 as `eggup-acquisition 0.1.3`. Baseline provenance, the code-qualification head `9a5500e`, and the consumer closures are recorded under `plans/closure/`.

The newest service runtime correction is M010 at `0bde3fe`, with run
`37875012280` green including Linux systemd. M011 published `eggup-service
0.1.3` from source `feb6ae5`; the 72.7 KiB package checksum is
`9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`. The
pre-publication head `f65496f` passed run `37876276212`; post-publication exact
version/systemd proof passed run `37883703334`, closing M011.

This file is the compact control surface for active Eggup planning. Detailed requirements live in the linked plans and roadmaps.

## Canonical documents

| Document | Status |
|---|---|
| `plans/000-long-term-specification.md` | normative |
| `plans/001-terminology-and-domain-model.md` | normative |
| `plans/002-long-term-roadmap.md` | active |
| `plans/003-planning-process.md` | normative |

## Accepted architecture decisions

| ADR | Decision |
|---|---|
| `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md` | core/acquisition/service remain layered; original producer-distribution ownership is superseded by ADR-0004 |
| `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet is the mutation unit; rollback and post-commit policy are explicit |
| `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md` | integrity/authenticity/candidate identity are distinct; core has no mandatory transport |
| `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md` | Eggpack owns producer release contracts/construction/evidence; Eggup owns local deployment; applications own release/install policy |
| `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md` | Local archive extraction is optional Eggup deployment machinery: already-verified archives, explicit member allowlists, finite decompression budgets, regular-file-only/no-clobber private extraction, no live mutation |

## Producer/consumer ownership guard

Eggup owns consumer-side acquisition, verification, candidate validation, local extraction required for installation, ownership/staging/locking/commit/rollback/recovery, install receipts, and service lifecycle. Producer release contracts, release conformance, package/archive construction, final release manifests, bootstrap-installer generation, generated release CI, staging/publication, and producer provenance belong to `eggstack/eggpack`.

Eggpack's producer evidence is a separate upstream publication: `eggpack-manifest 0.1.0` published from Eggpack `8d661e4` (checksum `bb8b9341…0e179`); closure at `plans/closure/eggpack-manifest-interoperability/003-status.md`. The 0.1.2 adapter set consumes that manifest, and the `eggsact` consumer closes the Eggpack→Eggup chain on the registry (`65c916b`, pin `e336b32`; runs `36902758482` + `36902758396` green). `eggup-dist` was unpublished predecessor evidence only.

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M019 closed; `eggup-core 0.2.0` published 2026-10-24 from `b612608`, hosted run `40079833945` green on all five lanes; M018 hosted qualification closed | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M017 closed, M012 conditionally; M017 published `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch` 0.2.0 (2026-10-24) from `b612608` | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M017 closed; `eggup-service 0.2.0` published 2026-10-24 from `b612608`, carrying the stdin-verdict fix the published `0.1.3` lacks | `plans/closure/service-lifecycle/` |
| Archive extraction | M001 lineage + M001d + M005 + M006 closed; M006 published `eggup-archive 0.2.0` 2026-10-24 from `b612608` | `plans/closure/archive-extraction/` |
| Eggpack manifest interoperability | M001-M007 all closed; M007 published `eggup-eggpack 0.2.0` 2026-10-24 from `b612608`, re-pinning all three in-workspace deps at `=0.2.0` | `plans/closure/eggpack-manifest-interoperability/` |
| Transport footprint | M001 closed; the three isolated transport configurations are built or checked in the `stable`, `msrv`, and `macos` lanes | `plans/closure/transport-footprint/` |
| Distribution/bootstrap (archived/transferred) | M001-M004 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch + M005 CodeGG + M006 Egress (closed 2026-09-29) | `plans/closure/consumer-adoption/` |
| Planning/closure hygiene corrective | C001-C015 all closed (docs-only reconciliation track); C014 documentation-truth and C015 architecture-anchor correctives closed concurrently | `plans/closure/planning-closure-hygiene-corrective/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.1 crates (lockstep patch superseding 0.1.0):

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

Published `0.1.2` set on crates.io — **all seven published crates**, in four steps: `eggup-core`, `eggup-archive` (2026-09-28); `eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack` (2026-10-02); `eggup-curl` (2026-10-04); `eggup-service` (2026-10-05, from `7fb84bc`, registry id `3411496`, checksum `c6288eb1…50e3f`). `eggup-service` was published later from a different source commit than the shared `v0.1.2` tag; the tag still denotes the core/archive publication source
and was not moved. `eggup-transport-footprint` is `publish = false` by design and has no registry version.

Initial `0.1.3` train — **five crates, two source commits**, published 2026-10-06:

| Package | Published from | Registry checksum |
|---|---|---|
| `eggup-acquisition 0.1.3` | `bb8fe41` | `71ce1a0d…a285a7` |
| `eggup-eggfetch 0.1.3` | `bd43683` | `e2bdffe9…56ada` |
| `eggup-archive 0.1.3` | `bd43683` | `a28af652…9824c` |
| `eggup-core 0.1.3` | `bd43683` | `161b244b…789ad` |
| `eggup-eggpack 0.1.3` | `bd43683` | `952e88b8…19c5` |

`eggup-acquisition` had to publish first because the other four depend on it —
`eggpack` exact-pins it and the others resolve it as a caret requirement — so it
was released from the earlier ancestor `bb8fe41` rather than the tag commit.
`eggup-curl` remains at `0.1.2` and receives the acquisition/core fixes through
its caret requirements. Service M011 separately published `eggup-service
0.1.3` on 2026-10-09 from `feb6ae5` (checksum
`9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`); it extends
GitHub Release `0.1.3` without moving the shared tag `v0.1.3`. `v0.1.1`/`v0.1.2`
and release `0.1.2` were not moved. `eggup-transport-footprint` remains
`publish = false`.

`0.2.0` train — **all seven published crates, one source commit**, published
2026-10-24:

| Package | Published from | Registry checksum |
|---|---|---|
| `eggup-core 0.2.0` | `b612608` | `2c86a0eb…eb23b8` |
| `eggup-archive 0.2.0` | `b612608` | `ce35edd5…ff57ad` |
| `eggup-acquisition 0.2.0` | `b612608` | `2ef74b4d…299fc3` |
| `eggup-curl 0.2.0` | `b612608` | `15cf536f…90d3f7` |
| `eggup-eggfetch 0.2.0` | `b612608` | `9fc8f648…658721` |
| `eggup-service 0.2.0` | `b612608` | `31dd2e22…fe1b4a` |
| `eggup-eggpack 0.2.0` | `b612608` | `33dc1da4…14f1f6` |

The version is `0.2.0`, not `0.1.4`: the workspace is version-lockstep and
`eggup-acquisition`'s next release is source-breaking — `AcquisitionError::Io(String)`
became `Io { operation, source }` and the enum derives `Debug` only — so the
seam had to advance and carried every crate with it. Published in dependency
order — core, archive, seam, curl, eggfetch, service, eggpack — because a crate
is packaged only after every in-workspace dependency it names is on the
registry. The `v0.2.0` tag and GitHub Release `0.2.0` point at `b612608`, the
publication source commit; the `v0.1.1`/`v0.1.2`/`v0.1.3` tags and the published
`0.1.x` versions were not moved or withdrawn. `eggup-transport-footprint`
remains `publish = false`.

One line differs between the published packages and this repository. The seven
`.crate` files were uploaded on 2026-10-24 and their `CHANGELOG.md` carries the
heading `## 0.2.0 — 2026-10-23`, the date the release was cut; the repository's
copy carries `## 0.2.0 — 2026-10-24`, the date it was published, which is the
convention every earlier per-crate release section follows. Registry state is
immutable, so the published heading is not corrected there.

The close-out that follows the publication is qualified separately, because it
is documentation-only and cannot change what was published. Commit `fd60b4b`
("close out the 0.2.0 publication train") is green on all five lanes in run
`40102904892`, and commit `390c224` (the registry's publication-row correction)
was **red in the `stable` lane** in run `40103357142` on a transient `ETXTBSY`
in the pre-existing core unit test `bounded_candidates_are_exact_and_environment_is_cleared`;
`git diff fd60b4b 390c224 -- crates/ scripts/ .github/` is empty, so the code was
byte-identical to the green run, and the rerun on the same source SHA (attempt
2) was green on all five lanes. That is the third recorded re-observation of
registry finding 1, and it is recorded there rather than here.

`eggup-dist` was unpublished and existed only as migration predecessor evidence. It was removed after Eggpack Contract M002 qualified the complete M003 behavior; see `plans/closure/distribution-bootstrap/004-status.md`.

Existing simple consumers:

- eggsact at `eggstack/eggsact@65c916b` (current; first adopted at `576f4b0`);
- stegoeggo at `eggstack/stegoeggo@10d8448`.

## Post-closure review findings

The core remains qualified. The 2026-10-23 workspace audit registered new
medium-or-higher findings, each with a corrective; they are recorded below
rather than in this summary.

### Post-0.1.3 open findings

Findings 1 and 2 are **low**-severity and block nothing. Per the repository's own
gate, "medium-or-higher" issues are what force a corrective before a milestone
closes. Findings 3-14 were registered by a read-only audit of the whole
workspace at baseline `c4805411f74b85158e0ae5d0a219a32c4209d98e`; each carries a
registered corrective, except 12, 13, and 14, which are recorded as follow-on
work for the reasons stated. **No production failure was reported for any of them.**
All were found by reading the code, so the severities below are the assessed
severities, not observed ones, and each corrective states the same limitation.
Findings 3, 4, 5, 7, 10, and 11 have since been implemented and closed by their
registered correctives; finding 11 is retained here, marked closed, so that its
number is never reused. Finding 15 was registered later, by Service M015, and
finding 16 by the same pass; both are entries here whose defect was
**observed** rather than assessed — the published `eggup-service 0.1.3`
misreports a successful stdin write, and the packaged-service step fails
against the registry. Finding 17 was registered by Acquisition M016.

1. **`eggup-curl` sub-second-deadline tests are load-sensitive on a busy host.**
   `build_curl_args_passes_sub_second_deadlines_to_fake_curl` and
   `sub_second_total_timeout_kills_child_promptly` carry sub-second wall-clock
   budgets and fail intermittently on a loaded developer machine (reproduced at
   pristine `bb8fe41` under load, so it predates and is unrelated to the 0.1.3
   train). Hosted CI passes both on every lane. The tests assert real behavior;
   the defect is test robustness, not transport semantics. A corrective should
   separate the deadline-under-test from host scheduling pressure. **Acquisition
   M012 is registered for this** —
   `plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`,
   `ready for handoff`. Reproduced at baseline `9326730` under load on macOS
   (14 cores, loadavg 62–68): **3/40 failures** on
   `build_curl_args_passes_sub_second_deadlines_to_fake_curl`, panicking with
   `Timeout { phase: "total" }` at a reported `0.77s` against a 750 ms budget.
   **Re-observed 2026-10-23** during the five-milestone implementation pass, on
   an unmodified `crates/eggup-curl/`: the same test failed once under a load
   average of 95.33 and then passed 8/8 in isolation and across a
   `--no-fail-fast` whole-workspace run. This is new evidence for M012's
   plan and confirms the finding is unrelated to the 0.1.3 corrective work.
   **Re-observed again on the hosted Ubuntu lane** in run `40005168760`, where
   the same test failed with a *spawn* failure rather than a timeout —
   `Unavailable("curl spawn failed for https://example.com/x")` — while the
   identical code had passed the same lane in run `40004604594`. The cause was
   not diagnosable, because `Command::spawn()`'s `io::Error` was discarded and
   a missing executable, a non-executable file, and a transient resource
   exhaustion were indistinguishable. Acquisition M015 was authored and
   implemented: the `Unavailable` detail now names the `ErrorKind` and the raw
   OS code, so the cause is attributable.
   The finding is therefore **narrowed, not closed**: M012 closed the
   *timeout-coupling* mode and M015 made the *spawn* mode attributable, but an
   intermittent spawn failure under host resource pressure can still occur.
   Acquisition M012 is **conditionally closed** on exactly that re-observation,
   and this finding stays open.
   **Re-observed 2026-10-24 on the hosted Ubuntu lane in run `40067739294`**, in
   a different crate and with an attributable cause: the core crate's
   pre-existing unit test `bounded_candidates_are_exact_and_environment_is_cleared`
   failed with `Io { operation: "spawning bounded candidate", source: Os {
   code: 26, kind: ExecutableFileBusy, message: "Text file busy" } }` — an
   `ETXTBSY` on the script the fixture had just written. The test existed at
   baseline `c480541` and the branch did not modify it; the branch's added core
   test load made the race easier to reach. It did not reproduce in 25
   consecutive local runs of `cargo test -p eggup-core --lib`, and the next
   hosted run, `40067964510`, was **green on all five lanes**. This is the same
   class as the two modes already recorded here — a load-sensitive test whose
   host-level cause is not the crate's semantics — and it is recorded here
   rather than registered separately, so that the class stays in one place.
   `ETXTBSY` is transient by construction: the kernel refuses to execute a file
   that is momentarily open for writing, which is what a host package
   replacement does to the interpreter a `#!/bin/sh` script names.
   **Re-observed again 2026-10-24 on the hosted Ubuntu lane in run `40103357142`**,
   with the identical `ETXTBSY` in the identical pre-existing core unit test
   `bounded_candidates_are_exact_and_environment_is_cleared`. The failing
   commit `390c224` changed `plans/registry.md` only — `git diff fd60b4b
   390c224 -- crates/ scripts/ .github/` is empty, so the code was byte-identical
   to the green run `40102904892` — and the rerun on the same source SHA
   (attempt 2) was **green on all five lanes**. This is the third recorded
   re-observation of the same class and the same test; the transient host-level
   cause is unchanged, so the class stays in this one place rather than being
   registered separately.
2. **`CleanupDisposition::DeferredToProcessExit` is a source-visible addition to
   a closed enum.** Required by M010 for truthful Windows self-update reporting;
   no truthful value exists in the `0.1.2` two-variant set, and marking the enum
   `#[non_exhaustive]` would break the same matches without helping. Zero
   `eggstack` consumers reference the type. Revisit at the intentional 0.2/1.0
   API boundary together with `Error` extensibility. See
   `plans/closure/verified-update-core/012-status.md`.
3. **`eggup-core` bounded candidate execution can exceed its documented
   wall-clock deadline.** `run_bounded` (`crates/eggup-core/src/candidate.rs:125`)
   evaluates its deadline only while the direct child is still alive (`:187`),
   and joins its reader threads only on the success path (`:199`). A descendant
   holding the inherited stdout/stderr pipe pins the call past the stated
   maximum; the timeout path additionally leaks both reader threads.
   **Severity: high** — a published library can hang indefinitely and violates a
   documented safety bound. **Core M014 is registered for this** —
   `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`,
   `ready for handoff`. **Implemented and closed** at
   `plans/closure/verified-update-core/014-status.md`: the deadline is now an
   unconditional maximum wall-clock bound on the whole bounded execution, the
   descendant case is bounded and fail-closed, and the reader-thread lifecycle
   is explicit. The public surface is unchanged. Linux, Windows, and the hosted
   CI matrix were **not run** from the implementation machine.
4. **`eggup-archive`'s public cleanup contract is mis-documented and can never
   succeed.** Three of the four `cleanup()` entry points
   (`crates/eggup-archive/src/lib.rs:353`, `:392`, `:580`) document that they
   "remove" the extraction directory, but the shared implementation at `:1287`
   only empties it and unconditionally returns
   `Err(ExtractionErrorKind::CleanupFailed)`. **Severity: high** — a published
   API whose `Result` success path is unreachable, documented as doing something
   it does not do. **Archive M005 is registered for this** —
   `plans/implementation/archive-extraction/005-cleanup-contract-and-member-write-corrective.md`,
   `ready for handoff`. **Implemented and conditionally closed** at
   `plans/closure/archive-extraction/005-status.md`: all four entry points now
   share one truthful contract with a reachable `Ok` path, the crate still never
   removes the root directory or traverses the root pathname, and the duplicated
   tar/zip member-write body is now one shared helper. One downstream
   consequence — the `eggup-eggpack` test helper encoding the removed
   expectation — was migrated by the parent session. The remaining condition is
   the outstanding hosted qualification: Linux, Windows, and the hosted CI
   matrix were **not run** from the implementation machine.
5. **`eggup-service` carries two divergent copies of the commit/rollback
   engine.** `disposition.rs:321` and `lifecycle_update.rs:254` implement the same
   orchestrator, and the copies already differ in safety-relevant ways:
   post-transition verification at `lifecycle_update.rs:552` against
   `disposition.rs:918`, and a typed error at `disposition.rs:946` against an
   `unreachable!()` at `lifecycle_update.rs:541`. Three structurally identical
   ownership enums also exist across two crates
   (`crates/eggup-core/src/domain.rs:116`, `crates/eggup-service/src/lib.rs:132`,
   `crates/eggup-service/src/lib.rs:2773`) with no conversion between them.
   **Severity: high.** **Service M012 is registered for this** —
   `plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`,
   `ready for handoff`. **Implemented and closed** at
   `plans/closure/service-lifecycle/012-status.md`: there is now one engine
   (`commit_engine`) reached by both public orchestrators, and the stricter of
   the two former behaviours wins everywhere — the manager's `completed` flag is
   re-verified by observation, an ownership that is no longer `Owned` is always
   a typed failure, an unobservable or non-owned service is never stopped before
   a rollback, and an invalid desired state produces a typed bounded failure
   instead of a panic. The two former behavioural differences survive as named,
   separately testable policy inputs that both default to the stricter
   behaviour. `eggup_service::Ownership` now re-exports `eggup_core::Ownership`
   and `CronOwnership` is a public alias of it; the manager command output type
   is renamed `ManagerCommandOutput` with the old spelling kept as a
   non-deprecated alias; and manager-output diagnostic text is bounded by one
   declared ceiling. Linux, Windows, and the hosted CI matrix were **not run**
   from the implementation machine.
6. **`eggup-service`'s real Windows SCM backend has never been executed against
   the real SCM API, and the published API surface has no local-gate guard.**
   `WindowsBackend` (`crates/eggup-service/src/windows_scm.rs:907`) has no test
   call site; all 15 SCM tests drive `FakeBackend` (`:1111`), so the
   `windows-check` lane proves compilation rather than behavior. The untested
   surface includes the `windows-service` mapping layer and five `unreachable!()`
   sites. Separately, the `published-api-0.1.{2,3}` fixtures are separate
   workspaces that `scripts/check-local.sh` cannot reach, and
   `tests/api_compatibility.rs` asserts against a helper defined in its own file.
   **Severity: high.** **Service M014 is registered for this** —
   `plans/implementation/service-lifecycle/014-windows-scm-execution-and-api-surface-corrective.md`,
   `ready for handoff`.
7. **`eggup-service` collapses manager failures into untyped strings and applies
   three different timeout models across its adapters.** `ServiceError` has four
   variants (`crates/eggup-service/src/lib.rs:289-303`) and the crate
   substring-matches its own error text at `:1788`; systemd, launchd, and SCM
   each bound a chained operation differently. **Severity: medium.** **Service
   M013 is registered for this** —
   `plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`,
   `ready for handoff`. **Implemented and conditionally closed** at
   `plans/closure/service-lifecycle/013-status.md`: `ServiceError::Manager`
   carries a machine-readable `ManagerFailureKind` and the crate no longer
   classifies its own failures by substring-matching its own error text; every
   adapter creates exactly one `OperationDeadline` per operation; and
   `CronManager` lists the crontab once, states its ownership rule once, and
   refuses inconsistent request material before any listing or mutation.
   Three **low**-severity findings remain open (the plan's literal
   "sum of recorded budgets ≤ one transition timeout" arithmetic form is
   unachievable with a wall-clock deadline and an instant executor — the
   substantive one-deadline contract is proven instead; the unrun platform and
   hosted lanes; and stderr-only access denial classifying as `Protocol`
   rather than `Permission`). Linux, Windows, and the hosted CI matrix were
   **not run** from the implementation machine.
8. **Nine `eggup-curl` tests pass vacuously when their fixture is unavailable,
   and `AcquisitionError::Io` flattens the underlying `io::Error`.** The
   `else { return }` guard appears at `crates/eggup-curl/src/lib.rs:1036`, `:1060`,
   `:1122`, `:1149`, `:1183`, `:1394`, `:1432`, `:1454`, `:1485`; a test that
   returns early is indistinguishable from a passing one.
   `AcquisitionError::Io(String)` (`crates/eggup-acquisition/src/lib.rs:271`) has
   no `source()` implementation, so `io::ErrorKind` is unrecoverable by a
   consumer. **Severity: medium.** **Acquisition M013 is registered for this** —
   `plans/implementation/acquisition-transport/013-adapter-assertion-and-failure-typing-corrective.md`,
   `ready for handoff`.
9. **`eggup-eggpack` re-implements core's file-digest routine and carries a
   direct `sha2` dependency for that one call site.** The inline loop at
   `crates/eggup-eggpack/src/lib.rs:504-518` duplicates `eggup_core::hash_file`
   (`crates/eggup-core/src/integrity.rs:65`), in a crate that already depends on
   `eggup-core = "=0.1.3"`. **Severity: medium** — the behavior is correct; the
   defect is a redundant published-crate dependency edge and a drift site.
   **Eggpack M006 is registered for this** —
   `plans/implementation/eggpack-manifest-interoperability/006-canonical-digest-reuse-corrective.md`.
   **Implemented and closed** at
   `plans/closure/eggpack-manifest-interoperability/006-status.md` (2026-10-23):
   the adapter delegates to `eggup_core::hash_file`, so the workspace holds one
   file-digest implementation. That closure left the defect **unpublished** —
   the registry's `eggup-eggpack 0.1.3` still carries `sha2 ^0.10.9` as a
   runtime dependency. **Disposition — closed for consumers on 2026-10-24.**
   Eggpack M007 published `eggup-eggpack 0.2.0` from `b612608`, and the
   published manifest carries no `sha2` normal dependency — `sha2` survives only
   as a dev-dependency for independent fixture digests. The published `0.1.x`
   line still carries the redundant edge; registry versions are immutable, so
   consumers pinned to `0.1.3` must move to `0.2.0` to receive the fix.
10. **`eggup-core`'s published failure taxonomy contains a category that can
    never be produced, and the crate's public `Result` surface documents no
    failure modes.** `FailureCategory::RetainedEvidence`
    (`crates/eggup-core/src/transaction.rs:116`) is never constructed — the only
    producer, the error-to-category mapping at `:195`, cannot yield it — yet
    `as_str()` publishes the name `retained-evidence`. The crate has zero
    `# Errors` rustdoc sections across all of `src/`. **Severity: medium.**
    **Core M015 is registered for this** —
    `plans/implementation/verified-update-core/015-public-api-contract-truthfulness-corrective.md`,
    `ready for handoff`. **Implemented and closed** at
    `plans/closure/verified-update-core/015-status.md`: the unreachable
    `RetainedEvidence` variant and its `retained-evidence` string are removed,
    the remaining seven `as_str()` strings are byte-identical to the published
    ones, and all 32 public fallible items now carry an `# Errors` section,
    enforced by a crate-local `clippy::missing_errors_doc`. `Error` keeps
    exactly the seven published variants and `FailureCategory` stays
    `#[non_exhaustive]`. Linux, Windows, and the hosted CI matrix were **not
    run** from the implementation machine.
11. **Root `CHANGELOG.md`, the consumer-facing `docs/`, `AGENTS.md`, the
    architecture deep dives, and this registry itself carried stale or
    contradictory status text after the 0.1.3 train.** The root `Unreleased`
    section asserted "Published baselines are `0.1.2` for all seven published
    crates" and re-listed work the `0.1.3` section records as published;
    `docs/crates.md` and `docs/quickstart.md` contradicted `docs/releases.md`;
    `AGENTS.md` and `architecture/overview.md` /
    `architecture/tooling-governance.md` carried the same stale pin claims.
    Inside this registry, Acquisition M010 was "ready" in one table and "closed"
    in another, and Gregg M004 was "blocked" in one table and "authorable" in
    another. **Severity: medium.** **Planning C013 is registered for this** —
    `plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md`,
    **closed 2026-10-23** at
    `plans/closure/planning-closure-hygiene-corrective/013-status.md`: the root
    `Unreleased` section now names the current published baseline and holds only
    genuinely unpublished work, the `docs/`/`README.md`/`AGENTS.md`/architecture
    version claims match the manifests and the registry, and this registry states
    one status per milestone with the `Dependency-ready implementation work`
    table named as authoritative. The line references quoted above were true at
    baseline `c480541` and are superseded by that closure record.
12. **The `eggup-transport-footprint` crate's central claim is not verified by
    any CI job, and the crate has no subsystem roadmap.** CI and the local gate
    only ever build `--all-features` or no features, so the isolated
    single-adapter configurations — the crate's entire purpose — are never
    built. `architecture/transport-footprint.md` further claims that the
    `required-features` split "makes an accidental cross-link a build error";
    that is inaccurate under `--all-features`, where optional dependencies are
    visible to every binary in the package. The crate has three binaries, 45
    source lines, and zero tests, and its binaries perform no measurement in
    code. Its published dependency counts (3 / 92 / 93) still reproduce
    exactly and must not change. **Severity: medium.** **Closed by Transport
    Footprint M001** —
    `plans/implementation/transport-footprint/001-isolated-configuration-coverage-corrective.md`;
    closure `plans/closure/transport-footprint/001-status.md`. The isolated
    single-adapter configurations are now built by CI — `stable` and `macos`
    build them with `cargo build`, `msrv` checks them at the 1.89 floor — and a
    linkage-discipline guard rejects a sibling-adapter reference in every
    configuration, with a discriminating negative control. The crate's
    dependency counts 3 / 92 / 93 still reproduce exactly. The `windows-check`
    lane still builds none of the three binaries, before and after; that is
    recorded as `not run`, not inferred. The prerequisite that had no plan —
    the missing subsystem roadmap — was discharged first: the roadmap is at
    `plans/subsystems/transport-footprint-roadmap.md`.
13. **Roadmap Phases 12 (authenticity / supply-chain hardening) and 13 (public
    API stabilization) are unstarted, and Phase 13's fuzz/property coverage for
    the security-critical parsers does not exist.** The workspace contains no
    `proptest`, `quickcheck`, `arbitrary`, or `benches/` target, while the
    parsers carrying the most security weight — tar/zip entry handling, the
    Eggpack manifest, SHA-256 sidecar parsing — have example-based coverage
    only. **Severity: medium.** These are roadmap capabilities rather than
    correctives, so no implementation plan is registered here;
    `plans/002-long-term-roadmap.md` owns their sequencing, and each requires
    roadmap decomposition before a bounded plan can be authored.
14. **`eggup-core`'s authoritative contract docs are not shipped to crates.io
    consumers.** `crates/eggup-core/Cargo.toml` excludes `docs/`, so
    `cargo package -p eggup-core --list` shows `src/`, `examples/`, `README.md`,
    and `CHANGELOG.md` — and not `crates/eggup-core/docs/{domain,verification,transaction}.md`.
    `AGENTS.md` calls those three files the authoritative contracts and
    `architecture/overview.md` routes readers to them, so a consumer on crates.io
    or docs.rs cannot read the contract they are being bound to. **Severity:
    medium.** **No plan is registered yet**: the manifest change alone has no
    consumer effect without the `eggup-core` republication that would deliver
    it, so the gap is recorded and owned by planning C013, which names
    **Verified Update Core M016 — `eggup-core` contract-doc packaging** as the
    follow-on. That milestone needs the republication to be authorized
    separately and must not be treated as a packaging one-liner. C013 recorded
    the residual as an open drift row in `architecture/tooling-governance.md`
    §"Known doc/code drift", naming M016 as the owner.
15. **The published `eggup-service 0.1.3` reports a successful stdin write as
    a failure.** The replaced `SystemExecutor` runner polled the stdin writer's
    verdict with `try_recv()`, which *removes* the message, so the writer's
    `Ok(())` was discarded and the later `recv_timeout` observed an empty,
    disconnected channel and concluded the write had failed. The published
    source carries it at `~/.cargo/registry/src/*/eggup-service-0.1.3/src/lib.rs:801`
    (`if let Ok(Err(_)) = rx.try_recv()`) and again at `:819`. The affected
    path is `crontab -`, the crate's only production stdin caller, so a
    crontab install reports `manager(Spawn, "manager stdin failed for …")`
    after a write that in fact succeeded. **Severity: medium** — the defect is
    in the source line, the behavior is user-visible on the one path that
    uses it, and the failure is not transient. **Service M015 closed the
    source-line defect** (`plans/closure/service-lifecycle/015-status.md`):
    the consolidated runner delegates to `eggup_core::run_bounded`, whose
    writer protocol does not lose the verdict, and the closure's baseline
    evidence shows the new fixture failing against the pre-change
    implementation and the published `feb6ae5` source. The **published**
    `eggup-service 0.1.3` still carries the defect, so the exposure is open
    until a republication milestone delivers the fix; `v0.1.3` must not move.
    **Disposition — closed for consumers on 2026-10-24.** Service Lifecycle
    M017 published `eggup-service 0.2.0` from `b612608`, which carries the
    fix, so the defect is no longer reachable by a consumer who resolves
    `0.2.0`. The published `0.1.3` still carries it — that version is not
    withdrawn and `v0.1.3` was not moved — so a consumer pinned to `0.1.3`
    still carries the exposure, and the fix reaches it only by resolving
    `0.2.0`. Closure:
    `plans/closure/service-lifecycle/017-status.md`.
16. **The packaged-service verification is deferred because the source line now
    depends on an unpublished `eggup-core`.** `cargo package -p eggup-service
    --locked` resolves `eggup-core` from crates.io, so it compiles the
    packaged crate against the **published** `eggup-core 0.1.3`. Service M015's
    consolidated runner calls `CommandSpec::stdin_data` (Core M017) and relies
    on core's unconditional execution deadline (Core M014); neither is in the
    published `0.1.3`, so the step fails with `E0599: no method named
    'stdin_data' found for struct 'CommandSpec'`. Hosted run `40061134915`
    observed that failure in the `stable`, `msrv`, `macos`, and
    `windows-check` lanes. **Severity: medium.** It is a publication state,
    not a defect in the source line — the same condition this branch already
    created for `eggup-curl`, `eggup-eggfetch`, and `eggup-eggpack`, whose
    sources also use capabilities the registry does not carry; the service
    package step was the last one still resolving against the registry.
    **Disposition at the time of the deferral:** the four CI steps were moved to
    `cargo package -p eggup-service --locked --no-verify`, with an inline
    comment in `.github/workflows/ci.yml` recording why, so the deferral was
    visible in the workflow and not silently dropped. The step still packaged
    the crate and validated its manifest and file list, and the compile of the
    packaged sources stayed covered by the workspace build, test, and clippy
    steps. **Follow-on: the `eggup-core` republication**, after
    which the strict step is restored. That republication is a named follow-on,
    not yet a numbered milestone, and it needs explicit maintainer authorization.
    CI never performs it.
    **Disposition — closed on 2026-10-24.** The `eggup-core` republication
    landed: `eggup-core 0.2.0` is on crates.io, published from `b612608`, and
    the strict step was restored in commit `2d6e089` — all four lanes that
    carry it now run `cargo package -p eggup-service --locked`, and the
    packaged sources are compiled against the registry copy of
    `eggup-core 0.2.0` rather than only packaged. The registry-only fixture
    proves the same thing from outside the repository: a consumer of
    `eggup-service 0.2.0` resolves `eggup-core 0.2.0` from crates.io and
    cannot resolve `0.1.3`.

17. **The two transport adapters cannot be packaged against the registry,
    because the seam they depend on is not published at the version they
    require.** `crates/eggup-curl/Cargo.toml:24` and
    `crates/eggup-eggfetch/Cargo.toml:24` declare
    `eggup-acquisition = { version = "0.1.0", path = "../eggup-acquisition" }`,
    a caret requirement resolving to `^0.1.0`, while both adapters call
    `eggup_acquisition::__adapter_io` and `__adapter_bound_detail`, which exist
    **only** in the unpublished seam — published `eggup-acquisition 0.1.3`
    carries neither. The seam's next release is source-breaking and is therefore
    `0.2.0`, which `^0.1.0` does not accept. Measured: `cargo package
    -p eggup-curl` exits 101 with 11 errors and `-p eggup-eggfetch` exits 101
    with 4. The same cascade reaches `crates/eggup-eggpack/Cargo.toml:22`
    (`=0.1.3`) and `crates/eggup-transport-footprint/Cargo.toml:16` (`0.1.0`).
    **Severity: medium** — a publication state, not a source defect. **Closed
    as a *recorded* coupling by Acquisition M016**: the seam's changelog now
    declares its next release as `0.2.0` and records the widening each
    dependent needs, a guard fails when a dependent neither accepts that
    version nor records the widening, and the false "a future republication
    would be a non-breaking `0.1.3`" claim is removed from the adapters'
    changelogs. **Still open for the adapters themselves**, because only the
    publication milestone may widen a manifest or bump a version.
    **Disposition — closed on 2026-10-24.** Acquisition Transport M017 is that
    publication milestone. The requirements were widened in the same change set
    — `eggup-curl` and `eggup-eggfetch` to `eggup-acquisition = "0.2"`,
    `eggup-eggpack`'s three exact pins to `=0.2.0`, and
    `eggup-transport-footprint`'s three to `"0.2"` so the workspace still
    resolves — and all three published crates are on crates.io at `0.2.0`
    from `b612608`. The measured before state was real: `cargo package
    -p eggup-curl` exited 101 with 11 errors and `-p eggup-eggfetch` exited
    101 with 4; both now package and verify against the registry. The
    registry-only fixtures prove it from outside the repository — the
    eggfetch fixture depends only on `eggup-eggfetch` and its lockfile
    resolves the seam at `0.2.0` from crates.io, and the curl fixture resolves
    both. Closure: `plans/closure/acquisition-transport/017-status.md`.

Corrective work identified after reviewing implementation SHA `889a234c` was closed sequentially. Acquisition M003, service M002, and distribution M001 were added and closed in that batch, with hosted CI at that exact SHA green on stable fmt/clippy/test/doc, the Rust 1.89 check, macOS tests, and the Windows workspace check. The full per-milestone evidence lives in the closure records under `plans/closure/`.

### Acquisition

M004 closed the remaining M003 review findings. Public `FetchLimits` values
are validated at every transport boundary, and post-link temp cleanup failure
can no longer report ordinary failure after a complete destination exists.
Details and environment limits are in
`plans/closure/acquisition-transport/004-status.md`.

### Service

Service M003 closed the post-M002 restart, deadline, executable/environment,
and config-identity findings. See
`plans/closure/service-lifecycle/003-status.md` for evidence and platform
limits.

### Distribution

M001-M003 are closed predecessor work. M003 is the terminal Eggup producer-side implementation and provides the conformance behavior Eggpack Contract M002 ported and independently qualified. The implementation is `9941c58d7039410c728860f9e4e382881d4ccf54`; closure is `4169c8021b447fe73c8ee3ea71a80a535c940f54`.

Distribution M004 removed `eggup-dist` after Eggpack Contract M002 closed at `82f799f3d971b2999ac14c2d8fc1b965370e0f58`; see `plans/closure/distribution-bootstrap/004-status.md`. The subsystem is archived/transferred, with no active Eggup producer-distribution milestone.

The 2026-10-05 Gregg/EggPool parity gaps are now **fully closed**. Core M010/M011 are hosted-green, Service M009 published `eggup-service 0.1.2`, and on 2026-10-06 the whole versioned handoff shipped: Acquisition M010 published the corrected seam, Core M012 preserved the published 0.1.x `Error` surface, and Core M013 published the versioned Core. Gregg M004 and EggPool M007 were intentionally unwritten only until these handoffs existed, so both are now authorable and the
registry's consumer row is updated accordingly.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | M001-M019 closed; M014/M015/M017/M018 hosted-qualified; M019 published `eggup-core 0.2.0` 2026-10-24 from `b612608`, hosted run `40079833945` green on all five lanes | M016 `eggup-core` contract-doc packaging — now **authorable**, because the republication it was blocked on has landed; it needs its own publication, and `0.2.0` does not ship the contract docs |
| Acquisition transport | M001-M017 closed; M012 conditionally closed; M017 published `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch` 0.2.0 (2026-10-24) from `b612608`, run `40079833945` green | no further acquisition milestone is registered. **Open finding 1 stays open**: M012 narrowed its timeout-coupling mode, M015 made the spawn cause attributable, and the hosted Stable lane re-observed the flake |
| Transport footprint | M001 closed 2026-10-23 and hosted-qualified in run `40062299630`; the subsystem roadmap was created 2026-10-23, which was the prerequisite its first milestone required | no further transport-footprint milestone is registered |
| Service lifecycle | M001–M017 closed; M012/M013/M014/M015 hosted-qualified; M017 published `eggup-service 0.2.0` 2026-10-24 from `b612608`, run `40079833945` green | no further service-lifecycle milestone is registered. The `SystemExecutor` consolidation that M012 deferred is closed by M015, and finding 16's strict packaged-service step was restored in `2d6e089` |
| Archive extraction | M001 lineage + M004 closed; M005 closed 2026-10-23 and hosted-qualified in run `40008039843`; M006 published `eggup-archive 0.2.0` 2026-10-24 from `b612608` | no further archive milestone is registered |
| Distribution/bootstrap | archived/transferred; M001-M004 closed | no further Eggup producer work |
| Eggpack manifest interoperability | M001-M007 closed; `eggup-eggpack 0.1.3` carries the member-identity cross-check; M006 closed 2026-10-23 and hosted-qualified in run `40008039843`; M007 published `eggup-eggpack 0.2.0` 2026-10-24 from `b612608` | no further Eggpack manifest milestone is registered |
| Consumer adoption | simple, eggsearch, CodeGG M005, Egress M006 closed | **Gregg M004 and EggPool M007 are now authorable** — every versioned handoff they were gated on (acquisition 0.1.3, Core 0.1.3) is published, and the 0.2.0 line is published on top of it |

## Dependency-ready implementation work

**`Dependency-ready implementation work` is the authoritative milestone-status
table.** Every milestone listed here carries the path of the record that owns it
— its closure record for a closed milestone, its plan for an open one — and each
milestone appears in exactly one table in this registry.

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Verified update core | M019 eggup-core 0.2.0 publication | closed 2026-10-24 | `plans/implementation/verified-update-core/019-eggup-core-0.2.0-publication.md`; `plans/closure/verified-update-core/019-status.md` | published `eggup-core 0.2.0` (`2c86a0eb…eb23b8`) from `b612608`; hosted run `40079833945` green on all five lanes; the train-order exception in plan §9.1 is the only deviation; registry-only fixture green |
| Archive extraction | M006 eggup-archive 0.2.0 publication | closed 2026-10-24 | `plans/implementation/archive-extraction/006-eggup-archive-0.2.0-publication.md`; `plans/closure/archive-extraction/006-status.md` | published `eggup-archive 0.2.0` (`ce35edd5…ff57ad`) from `b612608`; run `40079833945` green; registry-only fixture green; the packaged manifest carries no `eggup-core` edge |
| Acquisition transport | M017 eggup-acquisition 0.2.0 publication | closed 2026-10-24 | `plans/implementation/acquisition-transport/017-eggup-acquisition-0.2.0-publication.md`; `plans/closure/acquisition-transport/017-status.md` | published `eggup-acquisition` (`2ef74b4d…299fc3`), `eggup-curl` (`15cf536f…90d3f7`), `eggup-eggfetch` (`9fc8f648…658721`) 0.2.0 from `b612608`; three registry-only fixtures green; finding 17 closed |
| Service lifecycle | M017 eggup-service 0.2.0 publication | closed 2026-10-24 | `plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md`; `plans/closure/service-lifecycle/017-status.md` | published `eggup-service 0.2.0` (`31dd2e22…fe1b4a`) from `b612608`; run `40079833945` green; the registry-only fixture resolves `eggup-core 0.2.0`; findings 15 and 16 closed |
| Eggpack manifest interoperability | M007 eggup-eggpack 0.2.0 publication | closed 2026-10-24 | `plans/implementation/eggpack-manifest-interoperability/007-eggup-eggpack-0.2.0-publication.md`; `plans/closure/eggpack-manifest-interoperability/007-status.md` | published `eggup-eggpack 0.2.0` (`33dc1da4…14f1f6`) from `b612608`, last in the train; the registry-only fixture resolves all three exact pins at `0.2.0` |
| Verified update core | M010 current-executable transaction parity corrective | closed; hosted run `37376971555` green on all four lanes | `plans/implementation/verified-update-core/010-current-executable-transaction-parity-corrective.md`; `plans/closure/verified-update-core/010-status.md` | 11 integration + 2 unit fixtures; the Windows lane caught a missing identity check, fixed with a cross-platform SHA-256 binding; `self-replace` Windows-only dep; unpublished |
| Verified update core | M011 proof-authorized stale-lock recovery | closed; hosted run `37376971555` green on all four lanes | `plans/implementation/verified-update-core/011-proof-authorized-stale-lock-recovery.md`; `plans/closure/verified-update-core/011-status.md` | 14 fixtures + example verifier; default `acquire` still fail-closed; unpublished |
| Verified update core | M012 0.1.x public-error compatibility corrective | closed | `plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`; `plans/closure/verified-update-core/012-status.md` | `Error` keeps the seven published 0.1.2 variants; `RecoveryError` added for retained evidence; byte-identical exhaustive-match fixture fails against pre-M012 main |
| Verified update core | M013 eggup-core 0.1.3 publication | closed 2026-10-06 | `plans/implementation/verified-update-core/013-eggup-core 0.1.3-publication.md`; `plans/closure/verified-update-core/013-status.md` | published from `bd43683` (run `37527706900` green); `v0.1.3` + release created; registry-only direct fixture and the `eggup-service 0.1.2 -> eggup-core 0.1.3` graph both green |
| Service lifecycle | M009 eggup-service 0.1.2 publication | closed 2026-10-05 | `plans/implementation/service-lifecycle/009-eggup-service-0.1.2-publication.md`; `plans/closure/service-lifecycle/009-status.md` | published from `7fb84bc`; registry checksum matches locally built `.crate`; registry-only fixture 10/10; no `src/` change; `v0.1.2` unmoved |
| Service lifecycle | M010 owned failed-systemd quiescence corrective | closed | `plans/implementation/service-lifecycle/010-owned-failed-systemd-service-quiescence-corrective.md`; `plans/closure/service-lifecycle/010-status.md` | Runtime correction required; hosted run `37875012280` green on all lanes including real systemd auto-restart race; residual cgroup task correctly remains incomplete |
| Service lifecycle | M011 qualified service patch publication | closed 2026-10-09 | `plans/implementation/service-lifecycle/011-verified-service-patch-publication.md`; `plans/closure/service-lifecycle/011-status.md` | Published from `feb6ae5`, checksum `9f7f7ea8…32124dc`; final hosted run `37883703334` green; release extended and package asset attached |
| Acquisition transport | M009 eggup-curl 0.1.2 publication | closed 2026-10-04 | `plans/implementation/acquisition-transport/009-eggup-curl-0.1.2-publication.md`; `plans/closure/acquisition-transport/009-status.md` | published from `b485228`, hosted run `37223895075` green on all lanes; registry-only fixture 6/6 with registry-source lockfile; no `src/` change; `v0.1.1`/`v0.1.2` unmoved, release `0.1.2` notes extended; M007 Windows live-loopback limitation retained |
| Acquisition transport | M010 eggup-acquisition 0.1.3 publication | closed 2026-10-06 | `plans/implementation/acquisition-transport/010-eggup-acquisition-0.1.3-publication.md`; `plans/closure/acquisition-transport/010-status.md` | published from `bb8fe41` (run `37525705132` green); registry-only fixture 4/4 and fails 2/4 against the defective 0.1.2 |
| Acquisition transport | M011 eggup-eggfetch 0.1.3 correctness publication | closed 2026-10-06 | `plans/implementation/acquisition-transport/011-eggup-eggfetch-0.1.3-correctness-publication.md`; `plans/closure/acquisition-transport/011-status.md` | published from `bd43683` (run `37527706900` green); registry-only fixture 4/4 and **aborts the process** against 0.1.2 |
| Acquisition transport | M012 sub-second deadline test determinism corrective | conditionally closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/acquisition-transport/012-status.md` | Test-only; no production change, no dependency, no republication of `eggup-curl 0.1.2`. The deadline under test is decoupled from the enforced budget. **Open condition**: the hosted Stable lane re-observed registry finding 1 in the spawn mode, so this milestone is not closed |
| Verified update core | M014 bounded-execution deadline corrective | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/verified-update-core/014-status.md` | Closes finding 3 (high). The deadline is an unconditional maximum bound and the descendant-holds-pipe case is fail-closed. `eggup-core 0.1.3` keeps the defect until republication; `eggup-eggpack`'s `=0.1.3` pin blocks propagation |
| Verified update core | M015 public API contract truthfulness corrective | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/verified-update-core/015-status.md` | Closes finding 10 (medium). The never-constructed `FailureCategory` variant is removed (`#[non_exhaustive]`, downstream-compatible); all 32 public fallible items carry an `# Errors` section enforced by `clippy::missing_errors_doc`. `run_bounded` is untouched — M014 owns that |
| Verified update core | M017 bounded stdin execution corrective | closed 2026-10-23; hosted-qualified (`40062299630`) | `plans/closure/verified-update-core/017-status.md` | Supplies the stdin capability Service M015 needs. `CommandSpec::stdin_data` + `MAX_STDIN_BYTES` (1 MiB, inclusive); an oversize payload is rejected before any spawn; the three `Error::Io` operation strings are a stable documented classification surface. Additive; unpublished |
| Verified update core | M018 bounded-stdin outcome-priority corrective | closed 2026-10-24; hosted-qualification pending | `plans/closure/verified-update-core/018-status.md` | The deadline has priority over the bounded stdin writer's verdict, classified by the instant the writer observed the write. A failure observed before the deadline is an `Err`; one observed at or after it is the deadline outcome, so an undelivered payload is never a success. Closes M015 **U1** |
| Archive extraction | M005 cleanup contract truthfulness and member-write deduplication | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/archive-extraction/005-status.md` | Closes finding 4 (high). The `eggup-archive 0.1.3` cleanup API has a reachable, truthful `Ok` path on all four entry points; `eggup-eggpack`'s `=0.1.3` pin blocks it. The hosted run also exposed and fixed a CRLF-fragile test anchor in the member-write guard |
| Service lifecycle | M012 commit-engine dedup and ownership-type unification | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/service-lifecycle/012-status.md` | Closes finding 5 (high). One commit/rollback engine serves both orchestrators; the stricter former behaviour wins; the former differences survive as policy inputs. `eggup_service::Ownership` re-exports `eggup_core::Ownership`. The `SystemExecutor` consolidation M012 deferred is closed by M015 |
| Service lifecycle | M013 manager failure taxonomy and deadline contract | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/service-lifecycle/013-status.md` | Closes finding 7 (medium). Typed `ManagerFailureKind` replaces `permission_hint` matching; one timeout model for systemd/launchd/SCM; one cron listing/rule. Source-breaking: `ServiceError::Manager` matchers — republication only. Three low-severity findings stay open |
| Service lifecycle | M014 Windows SCM execution coverage and API-surface guard | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/service-lifecycle/014-status.md` | Closes open finding 6 (high). The hosted Windows lane ran the real SCM fixture (`2 passed; 0 failed; 0 ignored`, no `EGGUP-SCM-LIVE-SKIP`), discharging the plan's condition that a closure without a real hosted Windows run is not a closure |
| Service lifecycle | M015 bounded executor consolidation corrective | closed 2026-10-23; hosted-qualified (`40062299630`) | `plans/closure/service-lifecycle/015-status.md` | Closes M012 **Finding 1** (**low**): `SystemExecutor` now delegates to `eggup_core::run_bounded`. The public API and every manager's argv are unchanged. **Also fixes finding 15** (**medium**): the old runner misreported a *successful* stdin write as a failure. No new dependency |
| Service lifecycle | M016 published-change release-note corrective | closed 2026-10-24; hosted-qualification pending | `plans/closure/service-lifecycle/016-status.md` | Adds the missing **D8** release note — the fix for the published crate's stdin-verdict defect — and corrects the count to eight. Flags the `engine_policy` source break, corrects the M015 test inventory, re-anchors the public-API snapshot. Documentation only |
| Acquisition transport | M013 adapter assertion strength and failure typing | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/acquisition-transport/013-status.md` | Closes open finding 8 (medium). Nine vacuously-passing `eggup-curl` tests; typed IO failure surface on the zero-dependency seam. Source-breaking: `AcquisitionError::Io(String)` became `Io { operation, source }` and the derives are `Debug` only |
| Acquisition transport | M015 spawn failure diagnosability corrective | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/acquisition-transport/015-status.md` | The `Unavailable` detail names the `ErrorKind` and the raw OS code, so a spawn failure is attributable instead of cause-free. The variant stays `Unavailable`; the fallback contract is unchanged. **Narrows** open finding 1 rather than closing it |
| Acquisition transport | M016 adapter publication-coupling corrective | conditionally closed 2026-10-24 | `plans/closure/acquisition-transport/016-status.md` | Records the seam's `0.2.0` coupling: both adapters must widen `eggup-acquisition` to `0.2`, `eggup-eggpack` re-pins to `=0.2.0`, footprint widens to `0.2`; publish seam then adapters then leaf. Guarded; no manifest, version, or publication |
| Acquisition transport | M014 adapter capability parity corrective | closed 2026-10-23; hosted-qualified (`40062299630`) | `plans/closure/acquisition-transport/014-status.md` | Closes the parity finding deferred by M013: the curl adapter now exposes a caller-settable `User-Agent`. The control sits on `CurlTransport`, not `CurlConfig`, because `CurlConfig`'s public fields make any added field source-breaking for the published `eggup-curl 0.1.2`. Additive; unpublished |
| Transport footprint | M001 isolated-configuration coverage corrective | closed 2026-10-23; hosted-qualified (`40062299630`) | `plans/closure/transport-footprint/001-status.md` | Closes open finding 12 (**medium**). CI builds the curl-only, eggfetch-only, and both-adapters configurations; a linkage-discipline guard rejects a sibling-adapter reference in every configuration, with a discriminating negative control. Counts 3 / 92 / 93 unchanged. `publish = false`; no tag moved |
| Eggpack manifest interoperability | M006 canonical digest reuse | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/eggpack-manifest-interoperability/006-status.md` | Closes open finding 9 (medium). Removes a redundant direct `sha2` runtime dependency from a published crate. `Cargo.lock` unchanged; the resolved `sha2` is still `0.10.9`; no pin cascade |
| Planning/closure hygiene corrective | C013 post-audit documentation truth reconciliation | closed 2026-10-23; hosted-qualified (`40008039843`) | `plans/closure/planning-closure-hygiene-corrective/013-status.md` | Closes open finding 11 (medium). Docs-only: root `CHANGELOG.md`, `docs/`, `AGENTS.md`, the architecture deep dives, and this registry's own contradictory rows. Adds the `scripts/check-docs-truth.sh` gate command |
| Planning/closure hygiene corrective | C014 documentation-truth corrective | closed 2026-10-24; hosted-qualification pending | `plans/closure/planning-closure-hygiene-corrective/014-status.md` | The documentation-truth gate now reads the crate manifests, covers `architecture/*.md`, and reads three-cell registry rows; the transport-footprint deep dive and the CI matrix are corrected. Red-first proven against five documented false negatives |
| Planning/closure hygiene corrective | C015 architecture anchor and claim corrective | closed 2026-10-24; hosted-qualification pending | `plans/closure/planning-closure-hygiene-corrective/015-status.md` | Docs-only: re-derived 179 stale `path:line` anchors in the five stable architecture deep dives; removed-API, `=0.1.2` pin, and `eggup-core` `self-replace` claims corrected |
| Archive extraction | M004 eggup-archive 0.1.3 correctness publication | closed 2026-10-06 | `plans/implementation/archive-extraction/004-eggup-archive-0.1.3-correctness-publication.md`; `plans/closure/archive-extraction/004-status.md` | published from `bd43683` (run `37527706900` green); registry-only tar.gz + zip fixtures 6/6; Windows device-alias case fails against 0.1.2 |
| Eggpack manifest interoperability | M005 eggup-eggpack 0.1.3 security/correctness publication | closed 2026-10-06 | `plans/implementation/eggpack-manifest-interoperability/005-eggup-eggpack-0.1.3-security-publication.md`; `plans/closure/eggpack-manifest-interoperability/005-status.md` | published from `bd43683` (run `37527706900` green); member-identity cross-check ships; the `eggup-eggpack 0.1.2` acquisition-pin caveat is closed |
| Planning/closure hygiene corrective | C012 post-M004 roadmap + registry reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/012-post-m004-roadmap-and-registry-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/012-status.md` | docs-only; Phase 9/10 + post-M004 registry/archive active surfaces reconciled |
| Planning/closure hygiene corrective | C011 post-M003 closure + registration reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/011-post-m003-closure-and-registration-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/011-status.md` | docs-only; M003a/M003 closures authoritative; M004a closed concurrently |
| Eggpack manifest interoperability | M004a package/API promotion readiness preflight | closed | `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md`; `plans/closure/eggpack-manifest-interoperability/004a-status.md` | M003a/M003 closed; qualification only, no publication occurred; M004 has since closed (see M004 row below) |
| Archive extraction | M001c handle-relative member materialization corrective | historical predecessor; stop resolved | `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`; `plans/closure/archive-extraction/001c-status.md` | resolved by M001d closure |
| Service lifecycle | M008 OperationDeadline test determinism corrective | closed | `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`; `plans/closure/service-lifecycle/008-status.md` | hosted run `36332823865` green on all lanes (supersedes `36261671380`); M001-M007 closed |
| Planning/closure hygiene corrective | C007 C006 head + current-CI reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/007-c006-head-and-current-ci-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/007-status.md` | docs-only; reconciled C006 head, current-CI evidence, M008 registration, M001d gate |
| Planning/closure hygiene corrective | C006 M001d readiness + bound-source contract reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/006-m001d-readiness-and-bound-source-contract-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/006-status.md` | docs-only; corrected M001d contract |
| Archive extraction | M001d handle-backed source handoff | closed | `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`; `plans/closure/archive-extraction/001d-status.md` | hosted run `36335233644` green on all lanes (supersedes `36334772510`); M001c stop closed |
| Planning/closure hygiene corrective | C008 post-M001d status/handoff reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/008-post-m001d-status-and-handoff-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/008-status.md` | docs-only; active M001c stop wording reconciled |
| Eggpack manifest interoperability | M002a hosted qualification + clippy corrective | closed | `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md`; `plans/closure/eggpack-manifest-interoperability/002a-status.md` | hosted run `36477024102` green on all lanes (supersedes failed `36463041223`); direct Windows adapter runtime evidence collected |
| Planning/closure hygiene corrective | C009 Consumer M006 Egress evidence reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/009-m006-egress-evidence-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/009-status.md` | docs-only; Egress updater + Delivery M003 evidence confirmed at `03134f8`; M006 record corrected |
| Planning/closure hygiene corrective | C010 post-Egress/M003 active-status reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/010-post-egress-and-m003-readiness-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/010-status.md` | docs-only; reconciled archive-roadmap Egress closure, registry M003 producer-gate resolution, and M003 plan stale dependency prose |
| Verified update core | M008a clean package/current-head evidence corrective | closed | `plans/implementation/verified-update-core/008a-clean-package-evidence-corrective.md`; `plans/closure/verified-update-core/008a-status.md` | exact clean-tree package/dry-run + fixture green; no material delta from provisional evidence |
| Verified update core | M009 eggup-core / eggup-archive 0.1.2 publication | closed 2026-09-28 | `plans/implementation/verified-update-core/009-core-archive-0.1.2-publication.md`; `plans/closure/verified-update-core/009-status.md` | core then archive published from `e8e07eb`; registry-only smoke 3/3; `v0.1.2` + GitHub Release `0.1.2`; manual only, no release CI added |
| Verified update core | M008 core/archive consumer package qualification | closed with clean evidence (M008a); publication gate satisfied by M009 | `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md`; `plans/closure/verified-update-core/008-status.md` (M008a addendum) | — |
| Eggpack manifest interoperability | M002 archive extraction handoff | closed with hosted qualification (M002a) | `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`; `plans/closure/eggpack-manifest-interoperability/002-status.md` (M002a addendum) | — |
| Consumer adoption | M006 Egress archive/pair adoption | closed 2026-09-29 | `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`; `plans/closure/consumer-adoption/006-status.md` (+ M003-landed addendum) | Egress Delivery M003 landed on registry 0.1.2 pair with hosted Linux/macOS/Windows updater evidence (run `36639694985`) |
| Planning/closure hygiene corrective | C005 post-M001b materialization gate reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/005-post-m001b-materialization-gate-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/005-status.md` | docs-only; reconciled M001c stop + M001d gate |
| Archive extraction | M001b handle-bound cleanup + Windows portability corrective | closed | `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md`; `plans/closure/archive-extraction/001b-status.md` | hosted run `36222536670` green on all lanes |
| Acquisition transport | M008 sub-second deadline truthfulness corrective | closed | `plans/implementation/acquisition-transport/008-subsecond-deadline-truthfulness-corrective.md`; `plans/closure/acquisition-transport/008-status.md` | hosted run `36222536670` reached Windows portable tests |
| Archive extraction | M001a owned-root cleanup authority corrective | historical; superseded | `plans/implementation/archive-extraction/001a-owned-root-cleanup-authority-corrective.md`; `plans/closure/archive-extraction/001a-status.md` | superseded by M001b |
| Planning/closure hygiene corrective | C004 post-M007/M001 status + baseline reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/004-post-m007-m001-status-baseline-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/004-status.md` | docs-only; reconcile the new corrective gates and stale status text |
| Acquisition transport | M007 boundary safety hardening corrective | conditionally closed | `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`; `plans/closure/acquisition-transport/007-status.md` | M001-M006 closed; review baseline `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` |
| Service lifecycle | M007 UTF-8-safe bounded diagnostics corrective | closed | `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md`; `plans/closure/service-lifecycle/007-status.md` | Hosted Stable/MSRV/macOS/Windows qualification `36215858056` |
| Archive extraction | M001 bounded allowlisted extraction contract | closed | `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`; `plans/closure/archive-extraction/001-status.md` | hosted Linux/macOS/Windows qualification `36214688691` |
| Planning/closure hygiene corrective | C001 post-batch status and evidence reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/001-status.md` | CodeGG M005 + core M007 closed |
| Planning/closure hygiene corrective | C002 C001 commit + registry baseline reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/002-c001-commit-and-registry-baseline-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/002-status.md` | — |
| Planning/closure hygiene corrective | C003 M003 registry + closure reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/003-m003-registry-and-closure-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/003-status.md` | bounded M003 execution/status record at `da1b4a8048bf863e6a653c25f1ba56bc42f4531b`; docs/evidence only |
| Verified update core | M007 post-commit policy / deferred finalization | closed | `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`; `plans/closure/verified-update-core/007-status.md` | — |
| Consumer adoption | M005 CodeGG managed-runfile bundle | closed | `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md` | `plans/closure/consumer-adoption/005-status.md` |
| Service lifecycle | M005 prepared-transaction lifecycle integration | closed | `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`; `plans/closure/service-lifecycle/005-status.md` | — |
| Acquisition transport | M005 curl adapter + explicit transport composition | closed/qualified with M006 | `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`; `plans/closure/acquisition-transport/005-status.md` (+ M006 addendum) | — |
| Acquisition transport | M006 M005 Windows portability + cross-closure qualification corrective | closed | `plans/implementation/acquisition-transport/006-m005-windows-portability-and-cross-closure-qualification-corrective.md`; `plans/closure/acquisition-transport/006-status.md` | green hosted matrix `36176009068` |
| Service lifecycle | M006 daemon update disposition + reference qualification | closed; evidence reconciled | `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`; `plans/closure/service-lifecycle/006-status.md` (+ M006 addendum) | — |
| Distribution/bootstrap | M004 retirement | closed | `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`; `plans/closure/distribution-bootstrap/004-status.md` | — |
| Eggpack manifest interoperability | M001 ReleaseManifest v1 direct/bundle adapter | closed (historical) | `plans/implementation/eggpack-manifest-interoperability/001-release-manifest-v1-adapter.md` | `plans/closure/eggpack-manifest-interoperability/001-status.md`; post-closure findings tracked by M001a |
| Eggpack manifest interoperability | M001a adapter qualification + closure hardening | closed | `plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md` | `plans/closure/eggpack-manifest-interoperability/001a-status.md`; full regression matrix qualified |
| Eggpack manifest interoperability | M003a caller-bound destination policy corrective | closed | `plans/implementation/eggpack-manifest-interoperability/003a-caller-bound-destination-policy-corrective.md`; `plans/closure/eggpack-manifest-interoperability/003a-status.md` | implementation `39ff626`, hosted run `36890986000` green on all lanes |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | closed | `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`; `plans/closure/eggpack-manifest-interoperability/003-status.md` | consumer `eggstack/eggsact@65c916b` on Eggup pin `e336b32`; hosted CI run `36902758482` + drift run `36902758396` green |
| Eggpack manifest interoperability | M004 package/API promotion | closed | `plans/closure/eggpack-manifest-interoperability/004-status.md` | published `eggup-acquisition` 0.1.2 (`0b01deb8…f170`), `eggup-eggfetch` 0.1.2 (`2e483152…3528`), `eggup-eggpack` 0.1.2 (`9dbfdfb7…b13fef`) from `02a1d32`; hosted run `37090397398` green; adapter-only and Eggsact-shaped registry-only proofs green; publication order proven load-bearing; `v0.1.1`/`v0.1.2` unmoved |

Closed-history detail lives in the closure records under `plans/closure/`: Core M007/M008, service M005-M008, CodeGG M005, acquisition M005/M006, and planning-hygiene C001-C003 and C008-C011 carry reviewed evidence. Both Gregg-reference upstream mechanisms are closed; Gregg consumer migration is writable but not scheduled. Eggpack adapter/API work is implemented and qualified, and historical M001 remains closed evidence. No Eggup installer-generator replacement is authorized.

## Open, authorable, and future work

This table carries only work with no milestone row elsewhere — open, authorable,
and future work. It is not the milestone-status table; see the note above.

| Subsystem | Milestone | State | Blocker / evidence |
|---|---|---|---|
| Consumer adoption | M004 Gregg | authorable; plan intentionally unwritten | Every versioned handoff it was gated on is published: `eggup-acquisition 0.1.3` (Acquisition M010) and `eggup-core 0.1.3` (Core M013); Service M009 and Core M010/M011 are also closed |
| Consumer adoption | M007 EggPool selective | authorable; plan intentionally unwritten | Core M012/M013 are closed and `eggup-core 0.1.3` is published; only the StandaloneRust transaction is in scope, and package-manager/provenance paths stay downstream |
| Verified update core | M016 eggup-core contract-doc packaging | authorable; plan intentionally unwritten | The republication M016 was blocked on has landed; `eggup-core 0.2.0` does not ship the contract docs, so this needs its own plan, its own authorization, and its own publication |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M004 [closed] --+
                            +--> eggsearch M003 [closed]
service M003 [closed] ------+          |
                                       +--> service-aware adoption evidence recorded

service M003 [closed] ----------> service M004 Windows SCM [closed]
                                      |
                                      +--------------------------+
                                                                 |
core M006 [closed] --> core M007 post-commit policy [CLOSED] ---+
                                                                 |
CodeGG M005 [closed] ---------------------------------------------+--> planning/closure hygiene C001 [CLOSED] --> C003 [CLOSED]
                                                                           |
                                                                           `--> service M005 implementation [CLOSED]

distribution M003 [closed/frozen] ---> Eggpack Contract M002 [closed]
                                              |
                                              `--> distribution M004 retirement [closed; subsystem archived]

Eggpack Interop M001a [CLOSED; corrected fixtures]
            |
            `--> eggup-eggpack adapter M001 [CLOSED HISTORICALLY]
                    |
                    v
              adapter M001a corrective [CLOSED]
                    |
                    v
              M003 bounded parse/API qualification [DONE]
                     |
                     v
               M003a caller-bound destination seam [CLOSED via 39ff626, run 36890986000]
                      |
                      `--> M003 real-consumer adoption [CLOSED via eggsact 65c916b, CI run 36902758482 + drift 36902758396]
                               |
                               `--> M004a readiness preflight [CLOSED; see 004a-status.md]
                                        |
                                        `--> M004 package/API promotion [CLOSED: acquisition/eggfetch/eggpack 0.1.2 published]

core M006 + acquisition M004 [closed] --> CodeGG M005 managed bundle [CLOSED]
                                            |
                                            `--> producer release mapping stays in application/Eggpack; strict archive extraction stays CodeGG-owned

Archive Extraction M001 [CLOSED; run 36214688691]
               |
               v
Archive M001a cleanup authority [HISTORICAL; SUPERSEDED by M001b]
               |
               v
Archive M001b handle-bound cleanup + Windows portability [CLOSED; run 36222536670]
               |
               v
Archive M001c handle-relative member materialization [HISTORICAL PREDECESSOR; HANDOFF STOP RESOLVED BY M001d]
                |
                v
Planning C006 M001d readiness correction [CLOSED]
                |
                v
Service M008 deadline-test determinism [CLOSED via run 36332823865] -- green repo baseline restored
                |
                +--> Planning C007 current-CI/head reconciliation [CLOSED]
                |
                v
Archive M001d handle-backed source handoff [CLOSED via run 36335233644]
                |
                +--> Eggpack interoperability M002 [CLOSED with hosted qualification via M002a, run 36477024102]
                |         |
                |         `--> M002a hosted qualification/clippy corrective [CLOSED via run 36477024102]
                |                   |
                |                   `--> Core M008a clean package/current-head evidence [CLOSED; exact clean-tree evidence, no delta]
                |                             |
                |                             `--> Core M009 manual 0.1.2 publication [CLOSED 2026-09-28]
                |                                       |
                |                                       `--> Egress consumer M006 [CLOSED 2026-09-29 via Delivery M003]
                |
                `--> Planning C009 M006 Egress evidence reconciliation [CLOSED; docs-only]

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

review baseline ee1476 -> acquisition M007 boundary safety [CONDITIONALLY CLOSED; hosted run 36213509807]
                       `-> service M007 UTF-8 diagnostics [CLOSED; hosted run 36215858056]

review baseline ea51fe -> acquisition M008 deadline truthfulness [CLOSED via run 36222536670]
                       `-> archive M001a cleanup authority [HISTORICAL; SUPERSEDED by M001b]
                       `-> M009 eggup-curl 0.1.2 publication [CLOSED 2026-10-04; published from b485228]
                            |
                            +-> cargo-cleanme Phase 10 M010C lightweight updater [UNBLOCKED; consumer-owned]
                            |
                            `-> M010 eggup-acquisition 0.1.3 publication [CLOSED 2026-10-06; shipped the 0b8cb98 seam fixes]
                                     |
                                     `-> eggup-eggpack adapter republication [RETIRED; eggup-eggpack 0.1.3 published 2026-10-06 and inherits the seam fixes]

implementation 0573996 -> archive M001b cleanup/Windows corrective [CLOSED]
                      `-> hosted run 36222536670 [STABLE/MSRV/MACOS/WINDOWS ALL PASSED]

Gregg read-only reference evidence -> acquisition M005 curl/composition [CLOSED]
                                  \-> service M006 disposition/revalidation [CLOSED]
                                                |
                                                v
                                acquisition M006 Windows/closure corrective [CLOSED via run 36176009068]
                                                |
                                                `-> Gregg M004 [WRITABLE; plan intentionally unwritten]
```

The 2026-09-26 review opened acquisition M007 and service M007 (both closed). The M001 archive lineage, M008, C004, and C008 closed with hosted evidence; Eggpack M002 and Core M008 closed in full (M002a run `36477024102` green, superseding failed `36463041223`). C009 corrected the stale Egress evidence; Core M009 closed the publication gate on 2026-09-28 and Egress M006 landed 2026-09-29. C001-C003 and C008-C011 are closed; per-milestone evidence is in `plans/closure/`.

## Current project state

- Rust baseline: 1.89.
- Workspace audit (2026-10-23, baseline `c4805411f74b85158e0ae5d0a219a32c4209d98e`): a
  read-only review of all eight crates registered nine new findings with bounded
  corrective plans — four **high** (core M014, archive M005, service M012, service
  M014) and five **medium** (core M015, service M013, acquisition M013, eggpack
  M006, planning C013) — plus three findings with no plan yet (12:
  transport-footprint CI coverage; 13: roadmap Phases 12 and 13; 14: `eggup-core`
  contract-doc packaging). No production
  failure was reported for any of them; every finding was made by reading the
  code, and the workspace suite was green (387 tests) at that baseline. None of
  the nine plans publishes and none moves a tag.
- Implementation pass (2026-10-23, same baseline): **five** of the nine
  correctives were implemented and closed on the native macOS lane — core
  M014, core M015, archive M005, service M012, and service M013 — with closure
  records under `plans/closure/`. The parent session then ran the whole-tree
  gate: `./scripts/check-local.sh` exits 0 and the workspace suite passes 449
  tests with 0 failures, up from 387 at the audit baseline. At that stage
  Linux, Windows, and every hosted CI lane were **not run**, because no hosted
  runner was reachable from the implementation machine.
- Second implementation pass (2026-10-23, branch `plans/corrective-batch-2026-10-23`):
  the **remaining four** correctives — service M014, acquisition M013, eggpack
  M006, and planning C013 — are implemented and closed, together with
  acquisition M012 and a new acquisition M015. A hosted runner **was**
  reachable, so the qualification was executed: CI run `40008039843` is green on **all
  five** lanes (`stable`, `msrv`, `service-systemd`, `macos`,
  `windows-check`). The Windows lane executed the real Windows SCM fixture
  against the genuine Service Control Manager, which discharges service M014's
  own condition that "a closure without a real hosted Windows run is not a
  closure". The local gate now passes **483** tests with 0 failures.
  The hosted run also found and fixed three defects the native macOS lane
  cannot see: a CRLF-fragile test anchor in `eggup-archive`, thirteen
  Unix-shaped service fixtures that needed per-test `#[cfg(unix)]` gating, and
  a discarded `spawn()` cause in `eggup-curl` (now acquisition M015). Registry
  finding 1 stays **open and narrowed**: acquisition M012 is conditionally
  closed because the hosted lane re-observed the flake in the spawn mode.
- Third implementation pass (2026-10-24, same branch): **five** further
  milestones are implemented and closed — acquisition M014 (adapter capability
  parity), core M017 (bounded stdin), service M015 (bounded-executor
  consolidation), transport-footprint M001 (isolated-configuration coverage,
  plus the subsystem roadmap its first milestone required), and a
  documentation-anchor repair that re-derived **376** stale line citations
  across four architecture deep dives. CI run `40062299630` is green on **all
  five** lanes. The local gate passes **518** tests with 0 failures. The first
  hosted run of this pass, `40061134915`, was **red in four lanes** and is the
  evidence behind two recorded consequences: the Windows argv guard needed
  per-test `#[cfg(unix)]` gating because the systemd `ExecStart` grammar is
  Unix-only by construction, and the packaged-service verification is
  **deferred** until the `eggup-core` republication. The pass also registered
  **finding 15** (**medium**), a defect the consolidation *found*: the
  published `eggup-service 0.1.3` reports a successful `crontab -` stdin write
  as a failure.
- Core: M001-M017 are closed. `eggup-core 0.1.3` was published 2026-10-06 and still carries every defect recorded since; M014 (unconditional bounded deadline), M015 (public-API contract truthfulness) and M017 (bounded stdin) closed in the corrective passes. M016 (`eggup-core` contract-doc packaging) is the named follow-on and is **not yet authored** — it needs the republication authorized separately.
  - Acquisition: M001-M015 are closed, M012 conditionally. M009 published `eggup-curl 0.1.2` (2026-10-04), M010 published `eggup-acquisition 0.1.3` (2026-10-06), M011 published `eggup-eggfetch 0.1.3` (2026-10-06). M012 (timeout coupling), M013 (typed I/O surface), M014 (adapter capability parity) and M015 (spawn-cause attribution) closed in the corrective passes; finding 1 stays **open and narrowed**.
  - Service: M001-M015 are closed. `eggup-service 0.1.3` was published 2026-10-09 and **still carries the stdin-verdict defect recorded as finding 15**; M012 (commit-engine deduplication), M013 (failure taxonomy), M014 (Windows SCM surface) and M015 (bounded-executor consolidation) closed in the corrective passes. See
  `plans/closure/service-lifecycle/`.
- Distribution: M001-M003 remain historical predecessor evidence; M004 removed the producer crate after Eggpack Contract M002 closure. The subsystem is archived/transferred to Eggpack.
  - Consumer adoption: Gregg M004 and EggPool M007 are both **authorable** now — every versioned handoff they were gated on is published (`eggup-acquisition 0.1.3` on 2026-10-06, `eggup-core 0.1.3` on 2026-10-06, Service M009 and Core M010/M011 closed), so both plans are intentionally unwritten pending an authoring decision, not blocked. EggPool's scope is still only its StandaloneRust transaction; package-manager/provenance paths stay downstream. Egress M006 is closed
  2026-09-29 (Egress Delivery M003 landed on the registry 0.1.2 pair; M003-landed addendum in `plans/closure/consumer-adoption/006-status.md`).
- Archive extraction: closed through M005. M004 published the already-implemented `0.1.3` correctness fixes as `eggup-archive 0.1.3` on 2026-10-06; M005 (member-write guard, made line-ending portable in `87cab50`) closed in the corrective pass.
- Eggpack manifest interoperability: M001-M006 all closed. M003 closed — `eggpack-manifest 0.1.0` published from Eggpack `8d661e4`; M003 consumer adoption closed (`eggsact@65c916b`).
  M004a readiness preflight closed (nothing published); M004 promotion closed — the `0.1.2` acquisition/eggfetch/eggpack chain from `02a1d32`.
  M005 published `eggup-eggpack 0.1.3` on 2026-10-06 with the member-identity cross-check; M006 (canonical digest reuse) closed in the corrective pass.
  Evidence: `plans/closure/eggpack-manifest-interoperability/`.
- Transport footprint: M001 closed in the third pass. The crate's three isolated transport configurations are now built or checked in the `stable`, `msrv`, and `macos` lanes, and its subsystem roadmap exists; its published versions are unchanged.
- Release process: manual crates.io publication only.
- The lockstep 0.1.1 patch is published (seam-then-adapter order) now that the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until downstream adoption moves; no publication is implicit in these plans.

## Next handoff

The bounded 0.1.3 release train, including Service M011 publication and closure, is complete. wg-basic Distribution M004 C001a adopted `eggup-service = "=0.1.3"` (source `feb6ae5`, checksum `9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`) and now has formal technical closures for M004, M005, and Phase 10; production signing/publication remains externally blocked. Its Phase 11 work is also closed, and wg-basic's registry reports no active implementation plan
(Phase 12 is deferred). No new Eggup implementation plan became ready from this downstream closure. Acquisition M012 remains independently ready and unchanged.

A read-only workspace audit at baseline `c480541` registered nine bounded
corrective plans; see "Dependency-ready implementation work" and the
"Post-0.1.3 open findings" section. **All nine are now implemented; eight are
closed and acquisition M012 is conditionally closed**, and a hosted runner was
reachable for the qualification: CI run
`40008039843` is green on **all five** lanes, and the local gate passes 483
tests with 0 failures. A third implementation pass added five more milestones
— acquisition M014, core M017, service M015, transport-footprint M001, and the
architecture-anchor repair — qualified by CI run `40062299630`, also green on
all five lanes, with the local gate now passing 518 tests with 0 failures. None
of them publishes, and none moves a tag; the published
crates still carry every defect they carry today.
- **Fourth implementation pass (2026-10-24, same branch): a read-only audit of
  the whole batch registered three release-blocking items and a set of
  documentation claims that contradicted the branch's own work, and all of them
  are now closed by five more milestones** — acquisition M016 (the seam's
  `0.2.0` publication coupling), core M018 (the bounded-stdin outcome
  priority), service M016 (the published-change release note), planning C014
  (the documentation-truth gate), and planning C015 (the architecture anchor
  and claim corrective). CI run `40067964510` is green on **all five** lanes
  (stable 528 passed, msrv check-only, service-systemd 9 passed, macOS 528
  passed, windows-check 650 passed), and the local gate exits 0 with **528**
  tests passing and 0 failures. The immediately preceding run `40067739294`
  was **red in the `stable` lane** on a transient `ETXTBSY` in a pre-existing
  core unit test; it is recorded as a re-observation of finding **1** and did
  not reproduce. None of this publishes, and no tag moved.

What is **not** done. **Acquisition M012 is conditionally closed, not
closed**: the hosted Stable lane re-observed registry finding 1 in the same
test, in the *spawn* mode rather than the timeout mode M012 addressed. M012's
substantive change is implemented and verified, but the finding stays **open
and narrowed** — M012 decoupled the deadline under test from the enforced
budget, and M015 made the spawn cause attributable rather than cause-free. An
intermittent spawn failure under host resource pressure can still occur, and
preventing it is not in scope of either milestone. **Acquisition M014**
(adapter capability parity) is now closed, in the third implementation pass.

Two consequences are now unblocked. First, service M012 deliberately deferred
the `SystemExecutor` consolidation because core's bounded-execution primitive
was defective; core M014 has since closed, and that consolidation is now
authored and closed as service M015. Second, service M013's failure taxonomy retyped
`ServiceError::Manager`, so a republication of `eggup-service` — and, through
the `=0.1.3` pins, of `eggup-eggpack` — is the only way the fixed crates reach
consumers. Acquisition M013's typed I/O surface is source-breaking in the same
way, and its adapter-capability-parity decision is now closed by acquisition
M014.

The same republication cascade now has a **third** member. Service M015's
consolidated runner calls core's bounded-stdin capability, which is not in the
published `eggup-core 0.1.3`, so `eggup-service` can no longer be packaged
against the registry. The four CI package steps were deferred to
`cargo package -p eggup-service --locked --no-verify` with the reason recorded
in the workflow itself, and registry finding **16** (**medium**) named
the `eggup-core` republication as the follow-on that restores the
strict step. **That republication was a named follow-on, not yet a numbered
milestone**: it had no plan, no closure path, and no roadmap row, and it needed
explicit maintainer authorization. CI never performs it.
**It has since landed** — the `0.2.0` publication train published
`eggup-core 0.2.0`, commit `2d6e089` restored the strict step in all four lanes
that carry it, hosted run `40100087952` on that commit is green on all five
lanes, and finding **16** is closed.

The registry's own pre-existing status contradictions (Acquisition M010 "ready" versus "closed"; Gregg M004 "blocked" versus "authorable") and the stale `0.1.2` claims in `CHANGELOG.md`, `docs/`, `AGENTS.md`, and the architecture deep dives were owned by planning C013 and **are now repaired** — C013 closed 2026-10-23 at `plans/closure/planning-closure-hygiene-corrective/013-status.md`. Each milestone now has one status in this registry, and the `Dependency-ready
implementation work` table above is the authoritative one.

Do not author or implement an Eggup installer generator. Keep producer behavior in Eggpack and archive extraction outside the adapter, and do not duplicate producer ownership in Eggup. M004a and M004 are closed — do not re-open them or re-publish their `0.1.2` versions; the `eggpack-manifest` producer item was discharged by Eggpack Release Manifest M003. M009 and M010 are likewise closed: do not re-publish `eggup-curl 0.1.2`, `eggup-acquisition 0.1.3`, or any other published
version, and do not move `v0.1.1`/`v0.1.2`.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
