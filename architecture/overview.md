# Eggup architecture overview

Bird's-eye view of the workspace: what each module owns, which invariants are
cross-cutting, and where to read deeper. Every crate in the workspace appears
exactly once in the [module map](#module-map) and links to its own deep dive.

This document is an **index and orientation layer**, not a contract. Authoritative
contracts are the `plans/` documents and the per-crate contract docs under
`crates/eggup-core/docs/`. Where this file and a contract disagree, the contract
wins.

## How to use this document

| If you want to… | Read |
|---|---|
| Orient in under five minutes | [The one-paragraph model](#the-one-paragraph-model) + [module map](#module-map) |
| Review a change's blast radius | [Dependency graph](#dependency-graph) + [boundary rules](#boundary-rules-what-must-not-cross) |
| Check whether a claim is actually implemented | [Cross-cutting invariants](#cross-cutting-invariants) → deep dive |
| Do a real review of one component | The linked deep dive under [deep-dive index](#deep-dive-index-review-entry-points) |
| Understand the data flow end to end | [Composition paths](#composition-paths) |
| Understand why the design is shaped this way | [`plans/000`–`003`](../plans/000-long-term-specification.md) + [`plans/adrs/`](../plans/adrs/README.md) |

## The one-paragraph model

Eggup separates **policy from local update mechanism**. A consumer resolves a
release and acquires local inputs; `eggup-core` validates, stages, and commits an
explicitly described artifact set. Network transport, authenticity trust choices,
release ordering, service lifecycle, and producer-side bootstrap installers are
separate layers. Eggpack owns release contracts, packaging, and bootstrap
generation; Eggup stays usable with non-Eggpack releases. Integrity is SHA-256
checksum evidence only — the workspace makes **no authenticity or signature claim
anywhere**, by design ([ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)).

The workspace is deliberately **dependency-inverted from the call order**: the
crate that runs *last* (`eggup-core`, the local mutation engine) is depended on
by the layers that run *before* it, and it depends on nothing but `sha2`. That is
what keeps the mutation core testable in isolation and free of transport, service
manager, and producer concerns.

## Dependency graph

Derived from the `crates/*/Cargo.toml` manifests. Arrows point from dependent to
dependency (compile-time direction). All seven published crates are `0.1.2`.

```text
                         eggup-transport-footprint   (publish = false, 3 bins)
                            |            |
        +-------------------+            +----------------+
        |                                         |
   eggup-curl  ------\                    eggup-eggfetch  (eggfetch-core 0.2 + tokio)
                     >-- eggup-acquisition <-----/         (zero dependencies)
                     |   (seam + fixture + composition)
   eggup-eggpack ----+        |         +----- eggup-transport-footprint
   (leaf adapter)           |         |
   |                        |         +---- eggup-service (windows-args; windows-service on win)
   +--> eggup-archive       |
   (tar/zip/flate2/fs_at/   +---- eggup-core  <---- eggup-service
    sha2; dev-dep: core)         (sha2 only)
```

In plain dependency order, deepest first:

1. **`eggup-core`** → `sha2`. Policy-neutral local mechanics.
2. **`eggup-acquisition`** → *nothing*. Transport-neutral seam, fixtures, composition.
3. **`eggup-eggfetch`** → `eggup-acquisition` + `eggfetch-core` + `tokio` + `futures-util`. Native HTTP adapter.
4. **`eggup-curl`** → `eggup-acquisition` only. External-process adapter, no HTTP/TLS stack.
5. **`eggup-service`** → `eggup-core` + `windows-args` (+ `windows-service` on Windows). Lifecycle adapters.
6. **`eggup-archive`** → `tar`, `zip`, `flate2`, `fs_at`, `sha2` (dev-dep `eggup-core`). Bounded local extraction.
7. **`eggup-eggpack`** → `eggup-core` + `eggup-archive` + `eggup-acquisition` (`=0.1.2`) + `eggpack-manifest =0.1.0` + `sha2`. Leaf producer adapter.
8. **`eggup-transport-footprint`** → `eggup-acquisition` + optional `eggup-curl` / `eggup-eggfetch`. **Non-published** footprint fixtures.

Two consequences worth knowing before reviewing anything:

- **`eggup-core` and `eggup-acquisition` are the only two crates with no
  Eggpack-domain knowledge.** Everything Eggpack-shaped terminates at
  `eggup-eggpack`.
- **`eggup-curl` and `eggup-eggfetch` are interchangeable.** Both implement the
  same `AcquisitionTransport` trait and neither may know about the other.
  Choosing between them is a *footprint and trust* decision, made by the caller —
  which is exactly what `eggup-transport-footprint` exists to measure.

## Module map

`LOC` is source lines; `tests` is the count of `#[test]` functions. Every crate
carries `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`.

| Module (crate) | Role | LOC / tests | Key capabilities | Deep dive |
|---|---|---|---|---|
| `eggup-core` | Policy-neutral local mechanics | 4748 / 50 | `InstallPlan` → `Prepared` → `Verified` → `Validated` → `Receipt`; SHA-256 integrity, bounded candidate execution, caller-proven `Absent \| Owned \| Foreign \| Unknown`, `MutationLock`, locked ownership + staged-digest revalidation, commit/rollback, `commit_with_post_commit` | [core-transaction.md](core-transaction.md) |
| `eggup-acquisition` | Transport-neutral seam, fixtures, composition | 2182 / 38 | `AcquisitionRequest`, `FetchLimits` + `effective()` = `min(request, adapter)`, `CancelFlag`, `Success \| NotFound` vs hard failure (`Transport`/`Timeout`/`TooLarge`/`Cancelled`/`Io`/`Unavailable`), `AcquisitionTransport`, `FixtureTransport`, `ComposedTransport` + `CompositionPolicy`, exclusive 0600 temp + no-clobber promotion, URL redaction | [acquisition.md](acquisition.md) |
| `eggup-eggfetch` | Native HTTP adapter | 1231 / 25 | `EggfetchConfig::strict()` single policy point (timeout ceilings, redirect bound, explicit `ProxyDecision`, HTTP/1 + Rustls), status classification, streaming artifacts, sync-over-async bridge, category-only redacted errors | [eggfetch-adapter.md](eggfetch-adapter.md) |
| `eggup-curl` | External curl adapter | 1672 / 24 | `CurlConfig::strict()` single policy point (explicit executable, opt-in PATH discovery, connect/total ceilings, redirect/protocol/proxy explicit, `--disable` against curlrc), direct-process execution with kill/reap, same-request HTTP status capture, private temp + no-clobber promotion | [curl-adapter.md](curl-adapter.md) |
| `eggup-service` | Manager-neutral lifecycle | 10101 / 104 | `ServiceSpec` / `Ownership` / `LifecycleSnapshot`, `ServiceManager` + `TestDoubleManager`, `SystemExecutor` (allowlisted paths, cleared env, bounded I/O), `atomic_write_definition`, systemd / launchd / cron / Windows SCM adapters, `commit_with_lifecycle`, `UpdateRuntimeDisposition` + `plan_unix`/`plan_windows` + `DirectRuntimeControl` + `commit_with_disposition` | [service-lifecycle.md](service-lifecycle.md) |
| `eggup-archive` | Bounded local archive extraction | 3512 / 51 | Explicit tar.gz/zip member allowlists, finite compressed/decompressed budgets, regular-file-only extraction, private root, no-clobber files, streamed SHA-256/size evidence, `ExtractedArchive` → `PersistedExtraction` → `BoundExtraction` handoff, `DirectoryGuard` cleanup | [archive-extraction.md](archive-extraction.md) |
| `eggup-eggpack` | Optional Eggpack ReleaseManifest v1 adapter (leaf) | 849 / 63 | `project` / `project_json`, `bind_requests` (tightened byte caps), `default_destinations`, `materialize_artifact_set[_with_destinations]`, direct/bundle installable vs archive extraction-required, `core_plan_for_archive*`, `bind_archive_members`, `install_ids` | [eggpack-adapter.md](eggpack-adapter.md) |
| `eggup-transport-footprint` | Non-published footprint fixtures | 45 / 0 | Three `required-features` binaries — `curl_only`, `eggfetch_only`, `dual` — that pin which transport stacks a consumer links; measures the cost of the transport choice | [transport-footprint.md](transport-footprint.md) |
| tooling + governance | Workspace, verification, planning | — | `scripts/check-local.sh` (fmt/clippy/test/doc/tree), CI (stable, MSRV 1.89, macOS, Windows), workspace lints, `plans/` ADRs + subsystem roadmaps + closure records + registry | [tooling-governance.md](tooling-governance.md) |

### Source layout inside the larger crates

- `eggup-core` — `lib.rs` (state machine, facade) + `domain.rs`, `candidate.rs`,
  `transaction.rs`, `stage.rs`, `integrity.rs`, `lock.rs`, `error.rs`,
  `test_support.rs`; five runnable `examples/`.
- `eggup-service` — `lib.rs` (neutral model + Unix mechanics),
  `disposition.rs` (M006 runtime-authority barrier), `lifecycle_update.rs`
  (definition writing + `commit_with_lifecycle`), `windows_scm.rs` (native SCM).
- `eggup-archive` — single `lib.rs`; the extraction, binding, and
  cleanup-guard machinery is large enough that the internal stages are the
  interesting review surface.
- `eggup-eggpack` — single `lib.rs` plus three integration test files
  (`interoperability.rs`, `archive_handoff.rs`, `caller_destinations.rs`);
  integration tests outnumber unit tests, which is the right shape for a
  conformance adapter.

## Two orthogonal axes

Most review confusion in this workspace comes from mixing these up. They are
independent.

**Axis 1 — data flow (what calls what, in order):**

```text
release policy (caller)   NOT in this workspace
  -> eggup-eggpack (optional)        project manifest -> ManifestProjection
  -> eggup-acquisition                AcquisitionRequest -> AcquisitionTransport
       exact URL verbatim, caller bounds, private temp, NotFound vs failure
  -> eggup-eggpack (optional)        materialize_artifact_set -> eggup-core::ArtifactSet
  -> eggup-archive (optional)        verified archive -> declared private extracted members
  -> eggup-core                      prepare -> verify_integrity -> validate -> commit
  -> eggup-service (optional)        quiesce Owned service, commit, restore, post-install check
```

**Axis 2 — the verification ladder ([ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)):**

| Layer | Question it answers | Owner in this workspace |
|---|---|---|
| Integrity | *Are these bytes the bytes that were hashed?* | `eggup-core` — SHA-256 only |
| Authenticity | *Who vouched for that digest?* | **Nobody.** Deliberately absent. Trust is the caller's |
| Candidate identity | *Does the artifact behave as claimed?* | `eggup-core` — bounded candidate execution, caller-defined validator |

`eggup-archive` sits on the integrity rung: it emits streamed SHA-256/size
evidence for members it extracts, so the caller can bind an extraction to the
digest the producer published without the extractor ever claiming authenticity.

## Cross-cutting invariants

These hold across most of the workspace. When reviewing a diff, check the
invariant, not just the function.

| Invariant | Why | Where enforced |
|---|---|---|
| SHA-256 is **integrity evidence only**; no authenticity or signature claim | Keeps the core usable without a trust root it cannot verify | workspace-wide; stated in every crate's crate docs |
| `Owned` is **never inferred** — it is caller-proven via `OwnershipVerifier` | Prevents clobbering a foreign install | `eggup-core` `commit`; `eggup-service` `require_owned` |
| Foreign / Unknown / flapping ownership **fails closed with a receipt**, not an `Err` | Callers get structured recovery evidence; only lock contention and setup failures are `Err` | `eggup-core` commit/rollback |
| Core never creates live destination parents, never deletes stale locks | Avoids unrequested mutation of live install locations | `eggup-core`; `MutationLock::inspect` is read-only |
| Staging is owner-private: `0700` dirs, `0600` files | Other local users cannot read or race the transaction | `eggup-core` stage; `eggup-acquisition` temp helpers |
| No-clobber promotion everywhere bytes land | A partial write never becomes the live artifact | `eggup-acquisition`; `eggup-archive`; `eggup-service` |
| Timeouts are **ceilings**, resolved `min(request, adapter)` | No layer can invent a longer budget than another set | `FetchLimits::effective`, `EggfetchConfig`, `CurlConfig` |
| Errors are **category-only and redacted**; URLs pass through `redact_url` | Credentials and query strings do not leak into logs or receipts | `eggup-acquisition`, both adapters |
| Destructive manager operations require `Owned`; `Foreign`/`Unknown` fail closed | No privilege escalation is implied by an updater | `eggup-service` |
| Manager state and application health are distinct concepts | A running-but-broken service is not "installed correctly" | `eggup-service` `HealthState` vs `LifecycleState` |
| Transport fallback is allowed; **release fallback is never** | Silently swapping a release source would be policy the caller never granted | `eggup-acquisition` `CompositionPolicy` |
| Downloaded bytes are data and are never executed | Candidate execution is a separate, explicit, bounded step | `eggup-acquisition`, adapters |
| No shell interpolation anywhere | Argument vectors are passed as argv, never strings | `eggup-service` `SystemExecutor`, `eggup-curl` |

## Composition paths

Four supported shapes, in increasing Eggpack-specificity:

1. **Plain consumer, no Eggpack.** Acquire bytes with `eggup-acquisition` +
   a transport, build an `eggup_core::ArtifactSet` directly, commit via
   `eggup-core`. The smallest useful Eggup.
2. **Eggpack consumer.** Project the manifest with `eggup-eggpack`, fetch each
   `PlannedAcquisition` through the seam, materialize one `ArtifactSet`, commit
   via `eggup-core`. When the manifest marks an archive installation-required,
   `eggup-archive` materializes its declared members first.
3. **Service-aware consumer.** Wrap a `ValidatedTransaction` with
   `eggup_service::commit_with_lifecycle` for stop → commit → restore
   semantics, and `commit_with_disposition` when the manager (rather than
   Eggup) must own runtime control. Service state and application health stay
   distinct outcomes.
4. **Transport-selection consumer.** Build a `ComposedTransport` over
   `eggup-curl` and/or `eggup-eggfetch` with an explicit `CompositionPolicy`.
   Same request, one fallback axis, no release fallback.

## Boundary rules (what must not cross)

The failure modes this workspace is most protective against, stated as review
checks:

- **`eggup-core` must not gain** transport, service-manager, Eggpack, or
  authenticity-verifier dependencies. Its only dependency is `sha2`.
- **`eggup-acquisition` must not gain** any dependency at all, and must not
  learn release policy (no "try the next version", no mirror logic).
- **Adapters must not coordinate with each other.** `eggup-curl` does not know
  Eggfetch exists; composition lives in the seam, not in an adapter.
- **`eggup-eggpack` is the only crate allowed to touch producer types.** It
  translates release evidence and stops at extraction-required.
- **`eggup-eggpack` stops at extraction-required evidence.** URL policy,
  install-root selection, ownership, permission, and authenticity policy all
  stay caller-owned.
- **No crate may claim crash journaling.** A failed transaction is recoverable
  through receipts and backups, not through an automatic journal replay.

## Deep-dive index (review entry points)

| Deep dive | Module | Use it to review |
|---|---|---|
| [core-transaction.md](core-transaction.md) | `eggup-core` | The state machine, its invariants, every rollback path |
| [acquisition.md](acquisition.md) | `eggup-acquisition` | The seam contract, bounds resolution, temp/promotion safety, fixtures, composition policy |
| [eggfetch-adapter.md](eggfetch-adapter.md) | `eggup-eggfetch` | Native HTTP policy, timeout ceilings, status classification, redaction, streaming |
| [curl-adapter.md](curl-adapter.md) | `eggup-curl` | External-process policy, executable discovery, process lifecycle, protocol/proxy control |
| [service-lifecycle.md](service-lifecycle.md) | `eggup-service` | The neutral model, Unix/Windows adapters, lifecycle composition, runtime disposition |
| [archive-extraction.md](archive-extraction.md) | `eggup-archive` | Bounded allowlisted tar.gz/zip extraction, budgets, extraction→binding handoff |
| [eggpack-adapter.md](eggpack-adapter.md) | `eggup-eggpack` | Manifest projection, request binding, materialization, archive handoff, conformance tests |
| [transport-footprint.md](transport-footprint.md) | `eggup-transport-footprint` | Which transport stack a consumer links, and what that costs |
| [tooling-governance.md](tooling-governance.md) | workspace | Lints, the local verification gate, CI matrix, planning and closure process |

Contract-level detail that belongs to no single crate:

- [`crates/eggup-core/docs/domain.md`](../crates/eggup-core/docs/domain.md) — domain preparation rules
- [`crates/eggup-core/docs/verification.md`](../crates/eggup-core/docs/verification.md) — verification ordering, bounded candidates
- [`crates/eggup-core/docs/transaction.md`](../crates/eggup-core/docs/transaction.md) — commit/rollback mechanics

## Where the durable requirements live

- [long-term specification](../plans/000-long-term-specification.md) — normative
- [terminology and domain model](../plans/001-terminology-and-domain-model.md) — normative
- [long-term roadmap](../plans/002-long-term-roadmap.md) — active
- [planning process](../plans/003-planning-process.md) — normative
- [planning registry](../plans/registry.md) — the active control surface
- [subsystem roadmaps](../plans/subsystems/README.md) — one per subsystem
- [ADRs](../plans/adrs/README.md) — accepted decisions, superseded not rewritten
- [closure records](../plans/closure/README.md) — requirement → evidence matrices

Publication state: `eggup-core`, `eggup-archive`, `eggup-acquisition`,
`eggup-eggfetch`, `eggup-eggpack`, and `eggup-curl` are published at `0.1.2`.
`eggup-service` is unpublished. `eggup-transport-footprint` is `publish = false`
by design. CI never publishes; releases are manual.
