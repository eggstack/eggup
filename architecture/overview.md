# Eggup architecture overview

Eggup separates policy from local update mechanism. A consumer resolves a
release and acquires local inputs; `eggup-core` validates, stages, and commits
an explicitly described artifact set. Network transport, authenticity trust
choices, release ordering, service lifecycle, and producer-side bootstrap
installers are separate layers. Eggpack owns release contracts, packaging,
and bootstrap generation; Eggup remains usable with non-Eggpack releases.

The durable requirements and decisions are maintained in:

- [long-term specification](../plans/000-long-term-specification.md)
- [terminology and domain model](../plans/001-terminology-and-domain-model.md)
- [verified update core roadmap](../plans/subsystems/verified-update-core-roadmap.md)
- [layered ownership ADR](../plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md)
- [transaction and rollback ADR](../plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md)
- [verification and transport ADR](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)

Current workspace layers:

```text
consumer/release policy  ->  acquisition/transport  ->  eggup-core
                                                       local mechanics
```

`eggup-core` implements validated plans, owner-private staging (0700/0600),
SHA-256 integrity verification, bounded candidate execution, explicit
`Absent | Owned | Foreign | Unknown` ownership proofs, locked ownership and
staged-digest revalidation, mutation locking with inspection-only stale
handling, and commit/rollback with structured failure reports. The optional
`commit_with_post_commit` path retains the mutation lock and backup through one
caller check and resolves failure with explicit keep-or-rollback policy. No
live destination parent is created automatically, and no authenticity claim
is made. Commit/rollback mechanics are documented in
the [transaction contract](../crates/eggup-core/docs/transaction.md); service
and transport layers remain outside the crate. Verification ordering and
bounded candidate execution are defined in the [verification contract](../crates/eggup-core/docs/verification.md).
Domain preparation rules are defined in the [domain contract](../crates/eggup-core/docs/domain.md).

## Modules at a glance

| Module (crate) | Role | Key capabilities / tools | Deep dive |
|---|---|---|---|
| `eggup-core` | Policy-neutral local mechanics | `InstallPlan` → `Prepared` → `Verified` → `Validated` → `Receipt`; SHA-256 integrity, bounded candidates, ownership proofs, `MutationLock`, commit/rollback + post-commit policy | [core-transaction.md](core-transaction.md) |
| `eggup-acquisition` | Transport-neutral acquisition seam + fixture + composition | `AcquisitionRequest`, `FetchLimits` (+ `min(request,adapter)`), `CancelFlag`, `Success \| NotFound` vs hard failure (`Transport/Timeout/TooLarge/Cancelled/Io/Unavailable`), `AcquisitionTransport`, deterministic `FixtureTransport`, `ComposedTransport` + `CompositionPolicy` (transport fallback, never release fallback), exclusive 0600 temp + no-clobber promotion, URL redaction | [acquisition.md](acquisition.md) |
| `eggup-eggfetch` | Native HTTP adapter | `EggfetchConfig` single policy point (timeouts as ceilings, redirect bound, explicit proxy, HTTP/1 + Rustls), status classification, streaming artifacts, sync-over-async bridge, category-only redacted errors | [eggfetch-adapter.md](eggfetch-adapter.md) |
| `eggup-curl` | External curl adapter | `CurlConfig` single policy point (explicit executable, opt-in PATH discovery, connect/total ceilings, redirect/protocol/proxy explicit, `--disable` for curlrc), direct-process execution with kill/reap, same-request HTTP status capture, private temp + no-clobber promotion, category-only redacted errors | [acquisition.md](acquisition.md) |
| `eggup-service` | Manager-neutral service lifecycle | `ServiceSpec`/`Ownership`/`LifecycleSnapshot`, `ServiceManager` + `TestDoubleManager`, `SystemExecutor` (allowlisted paths, cleared env, bounded I/O), `atomic_write_definition`, systemd / launchd / cron / Windows SCM adapters, `commit_with_lifecycle`, M006 `UpdateRuntimeDisposition` + `plan_unix`/`plan_windows` + `DirectRuntimeControl` + `commit_with_disposition` (artifact vs manager authority, revalidation barrier) | [service-lifecycle.md](service-lifecycle.md) |
| `eggup-eggpack` | Optional Eggpack ReleaseManifest v1 adapter (leaf) | `project` / `project_json`, `bind_requests` (tightened byte caps), `materialize_artifact_set`, direct/bundle installable vs archive extraction-required | [eggpack-adapter.md](eggpack-adapter.md) |
| tooling + governance | Workspace, verification, planning | `scripts/check-local.sh` (fmt/clippy/test/doc/tree), CI (stable/msrv/macos/windows-check), `forbid(unsafe)` + `deny(missing_docs)`, plans / ADRs / roadmaps / closure / registry | [tooling-governance.md](tooling-governance.md) |

## How everything works together

```text
Eggpack ReleaseManifest (optional)
  -> eggup-eggpack: project target -> bind_requests (tightened limits)
  -> acquisition transport (fixture or eggfetch): fetch_metadata / fetch_artifact
       exact URL verbatim, caller bounds, 0600 temp + no-clobber, NotFound vs failure
  -> eggup-eggpack: materialize_artifact_set (size/regular-file checks + digests)
  -> eggup-core: prepare (private stage) -> verify_integrity (SHA-256)
       -> validate (bounded candidates) -> commit (locked ownership +
          staged-digest revalidation -> backup -> rename -> receipt)
  -> eggup-service (optional): quiesce Owned service, commit, restore + post-install check
```

Typical flows:

1. Non-Eggpack consumer acquires bytes with `eggup-acquisition` + `eggup-eggfetch`,
   builds `ArtifactSet` directly, and commits via `eggup-core`.
2. Eggpack consumer projects a manifest with `eggup-eggpack`, fetches each
   `PlannedAcquisition` via the acquisition seam, materializes one `ArtifactSet`,
   and commits via `eggup-core`. Archives stop at evidence; extraction is a
   separate qualified boundary.
3. Service-aware consumer wraps a `ValidatedTransaction` with
   `eggup-service::commit_with_lifecycle` for stop-commit-restore semantics.
   Service state and application health stay distinct.

## Deep-dive index (review entry points)

- [core-transaction.md](core-transaction.md) — `eggup-core` state machine, invariants, rollback paths.
- [acquisition.md](acquisition.md) — seam contract, bounds, temp/promotion safety, fixtures.
- [eggfetch-adapter.md](eggfetch-adapter.md) — native HTTP policy, timeouts, redaction, streaming.
- [service-lifecycle.md](service-lifecycle.md) — neutral model, Unix/Windows adapters, lifecycle composition.
- [eggpack-adapter.md](eggpack-adapter.md) — manifest projection, binding, materialization, fixtures.
- [tooling-governance.md](tooling-governance.md) — workspace lints, verification workflow, plans/ADRs/closure.
