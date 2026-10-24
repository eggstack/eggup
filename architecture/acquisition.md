# Acquisition Seam Deep-Dive (`crates/eggup-acquisition`)

> Source: `crates/eggup-acquisition/src/lib.rs` (2229 lines, single file, 38 `#[test]`
> functions), `README.md`, `CHANGELOG.md`, `Cargo.toml`. Crate attributes:
> `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]` (`lib.rs:1-2`).
> Manifest: `description = "Transport-neutral acquisition seam and deterministic fixture
> transport (no release policy, no execution)"`, empty `[dependencies]`.
>
> Line references below are `crates/eggup-acquisition/src/lib.rs` unless another file is
> named. Adapter internals are out of scope here: see
> [eggfetch-adapter.md](eggfetch-adapter.md) and [curl-adapter.md](curl-adapter.md).

## 1. Purpose and ownership boundary

`eggup-acquisition` is the seam between caller policy and byte movement. An adapter
receives one exact caller-selected URL, enforces caller-provided byte and time bounds,
writes artifacts into a transaction-owned destination, and returns a typed
`Success | NotFound | Failure`. The seam owns no release resolution, no version
selection, no mirror logic, and no execution: downloaded bytes are data
(`lib.rs:5-8`, `lib.rs:615-624`).

What the seam explicitly never decides, and where that decision lives instead:

| Never decided here | Owner | Evidence |
|---|---|---|
| Which release/version to fetch | caller policy (optional `eggup-eggpack` projection) | `lib.rs:18-26` (`AcquisitionRequest` holds one exact URL, no discovery) |
| Which transport to use | caller, via `CompositionPolicy` | `lib.rs:970-1002` |
| Whether a `NotFound` is acceptable | caller | `lib.rs:180-183` |
| Authenticity / who vouched for the digest | **nobody in this workspace**, by design | [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) |
| Install root, permissions, ownership | caller + `eggup-core` | `lib.rs:639-642` (`dest` parent must already exist) |
| Executing what was downloaded | a separate, explicit, bounded step | `lib.rs:8` |

The `plans/` boundary rule is explicit: `eggup-acquisition` "must not gain any
dependency at all, and must not learn release policy" (no "try the next version", no
mirror logic) — [overview.md](overview.md#boundary-rules-what-must-not-cross).

## 2. Position in the workspace

`eggup-acquisition` has **zero dependencies** — not even `sha2`. It uses only `std`
(`use` block, `lib.rs:10-16`). It is the second-deepest crate in the graph, below
`eggup-core`.

Crates that depend on it:

| Dependent | Declared requirement | How it uses the seam |
|---|---|---|
| `eggup-eggfetch` | `0.1.0` (caret) + path | native HTTP adapter; implements `AcquisitionTransport` |
| `eggup-curl` | `0.1.0` (caret) + path | external-process adapter; implements `AcquisitionTransport` |
| `eggup-eggpack` | `=0.1.2` exact pin + path | `use eggup_acquisition::{AcquisitionRequest, FetchLimits}` only (`eggup-eggpack/src/lib.rs:6`) — binds requests, never transports |
| `eggup-transport-footprint` | `0.1.0` (caret) + path | `curl_only.rs` / `eggfetch_only.rs` use `AcquisitionRequest` + `FetchLimits`; `dual.rs` uses `ComposedTransport` + `CompositionPolicy` |

The caret/exact-pin split is deliberate: adapters inherit a seam fix on the next
resolve, `eggup-eggpack` does not
([acquisition-transport-roadmap.md](../plans/subsystems/acquisition-transport-roadmap.md#m010--eggup-acquisition-013-publication),
M010).

See [overview.md](overview.md) for the full graph, the two orthogonal axes (data flow
vs the verification ladder), and the cross-cutting invariant table this crate
contributes to.

## 3. Public surface

### 3.1 Stable API (consumer contract)

| Item | Kind | Role | Line |
|---|---|---|---|
| `AcquisitionRequest` | struct | One exact caller-selected URL, verbatim | `24-26` |
| `AcquisitionRequest::new` | fn | Validate + construct; rejects empty, control chars, non-`http(s)`, >8192 bytes | `33-49` |
| `AcquisitionRequest::url` | fn | The exact URL for the transport | `52-54` |
| `AcquisitionRequest::redacted` | fn | Diagnostics-only redacted URL | `57-59` |
| `FetchLimits` | struct | Caller byte/time bounds; public fields for 0.1.x source compat | `64-73` |
| `FetchLimits::default` | impl | 256 KiB metadata, 128 MiB artifact, 10 s connect, 120 s total | `75-84` |
| `FetchLimits::validate` | fn | The single authoritative rule set | `92-108` |
| `FetchLimits::new` | fn | Construct + `validate()` | `116-130` |
| `FetchLimits::effective` | fn | `min(request, adapter ceiling)` per phase | `143-151` |
| `CancelFlag` | struct | Cooperative cancellation (`AtomicBool`, `SeqCst`) | `159-161` |
| `CancelFlag::new` / `cancel` / `is_cancelled` | fn | Cancellation control | `165-177` |
| `FetchOutcome<T>` | enum | `Success(T) \| NotFound` | `185-190` |
| `FetchOutcome::success` / `is_success` / `is_not_found` | fn | Outcome accessors | `194-209` |
| `MetadataBytes` | struct | Bounded in-memory body; private field, no public constructor | `214-216` |
| `MetadataBytes::bytes` / `len` / `is_empty` | fn | Body accessors | `220-232` |
| `ArtifactEvidence` | struct | `{ bytes_written: u64 }`; public field + getter | `237-247` |
| `AcquisitionError` | enum | `#[non_exhaustive]` hard-failure taxonomy | `258-297` |
| `AcquisitionError::is_unavailable` | fn | `Unavailable` predicate for callers | `334-336` |
| `AcquisitionTransport` | trait | The seam contract (2 methods) | `625-653` |
| `redact_url` | fn | Strip userinfo, fragment, query; truncate | `404-432` |
| `FixtureResponse` | struct | One deterministic fixture outcome | `659-661` |
| `FixtureResponse::body` / `not_found` / `failure` / `truncated` / `slow` / `unavailable` | fn | Fixture constructors | `675-720` |
| `FixtureTransport` | struct | Deterministic in-memory transport | `728-730` |
| `FixtureTransport::new` / `route` | fn | Build and register exact-URL routes | `734-744` |
| `SharedTransport` | type alias | `Arc<FixtureTransport>`, for tests needing shared ownership | `950` |
| `CompositionPolicy` | enum | Caller-selected fallback scope | `979-987` |
| `ComposedTransport<'a>` | struct | Two transports + policy, itself a transport | `998-1002` |
| `ComposedTransport::new` / `policy` | fn | Record order and policy only | `1017-1032` |

`AcquisitionError::invalid`, `transport_redacted`, and `io` are `pub(crate)`
(`lib.rs:300`, `304`, `329`) — adapters outside the crate cannot use them, which
is why the `__adapter_*` constructors exist.

`AcquisitionError` is `#[non_exhaustive]`, which constrains **consumer matching**
(a wildcard arm is required) but not **adapter construction**: both adapters build
`InvalidInput`, `Transport`, `Timeout`, `TooLarge`, and `Cancelled` directly
(`eggfetch/src/lib.rs:139,163,282,314,493`; `curl/src/lib.rs:138,955,941`). The
typed `Io` variant is the exception: because it carries a `std::io::Error`, it is
built through the seam's adapter-facing `__adapter_io` constructor
(`eggfetch/src/lib.rs:352`; `curl/src/lib.rs:382,416,468,764,856,864,870,881,885`).

`ArtifactEvidence` has a public field, so a consumer can construct one;
`MetadataBytes` has no public constructor at all (`lib.rs:214-216`). That asymmetry
is in the code, not an oversight to read into.

### 3.2 `__`-prefixed, adapter-facing (`#[doc(hidden)]`)

These are public because the seam and its adapters are **separate crates** — but they
are not consumer contract. The `__` prefix plus `#[doc(hidden)]` is the marker: if you
are writing a consumer, these do not exist for you; if you are writing an adapter,
these are the only sanctioned way to build a temp file, promote it, and return
`MetadataBytes` / `ArtifactEvidence` / `Unavailable`.

| Item | Kind | Role | Line |
|---|---|---|---|
| `__adapter_unavailable` | fn | Build `Unavailable(detail)`; bounds the detail | `314-316` |
| `__adapter_io` | fn | Build the typed `Io { operation, source }` | `325-327` |
| `__adapter_metadata` | fn | Build `MetadataBytes` (the only constructor) | `958-960` |
| `__adapter_artifact` | fn | Build `ArtifactEvidence` | `966-968` |
| `__adapter_bound_detail` | fn | Apply the seam's 512-byte UTF-8-safe bound to an adapter detail | `376-378` |
| `__TEMP_COLLISION_BOUND` | const | `= 32`; max exclusive-temp creation attempts | `489` |
| `__acquire_exclusive_temp` | fn | `(File, PathBuf)`; `create_new` + `0600` sibling in `parent` | `503-565` |
| `__promote_no_clobber` | fn | `hard_link` promotion, fails closed on existing `dest` | `582-584` |
| `__remove_owned_temp` | fn | Best-effort removal of exactly one owned temp | `611-613` |
| `__scrub_upstream_text` | fn | Defense-in-depth scrub for arbitrary upstream text | `441-490` |

Private, for reference: `NEXT_TEMP_NONCE` (`655`), `FixtureKind` (`664-671`),
`bound_detail` / `truncate_utf8_bytes` (`380-392`), and the test-only
`__promote_no_clobber_with` injection point (`586-608`).

`__scrub_upstream_text` has **no in-tree caller**: neither adapter uses it. Both rely
on category-only error text plus `redact_url` instead, which is what the doc comment
recommends (`lib.rs:434-439`).

## 4. The seam contract: `AcquisitionTransport`

```rust
pub trait AcquisitionTransport {
    fn fetch_metadata(&self, request: &AcquisitionRequest, limits: FetchLimits,
                      cancel: &CancelFlag)
        -> Result<FetchOutcome<MetadataBytes>, AcquisitionError>;

    fn fetch_artifact(&self, request: &AcquisitionRequest, dest: &Path, limits: FetchLimits,
                      cancel: &CancelFlag)
        -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError>;
}
```

Both methods take `&self` (no `&mut self`) and are synchronous. Async adapters bridge
internally — `eggup-eggfetch` uses a sync-over-async `block_on`
(`eggfetch/src/lib.rs:371-455`).

### `fetch_metadata`

| Must guarantee | Must never |
|---|---|
| Use `request.url()` verbatim; no discovery, version selection, or mirror choice | Select a release, version, or alternative source |
| Call `limits.validate()` **before** any route lookup, filesystem, or network I/O | Extend a caller deadline; derive time bounds any other way than `limits.effective(...)` |
| Bound the body by `limits.max_metadata_bytes`, reporting `TooLarge { limit }` with the real bound | Embed raw upstream/proxy error `Display` in diagnostics |
| Return `Ok(NotFound)` for a definitive absence, never `Err` for it | Map a timeout, cancellation, or bound violation to `NotFound` |
| Observe `cancel` before and during the body read | Promote anything to a caller-visible destination |
| Redact URL material in every diagnostic string | Execute or parse the body as a release policy |

`fetch_metadata` has no destination argument, so an adapter is free to use scratch
storage. `eggup-curl` writes to an exclusive `0600` temp in `std::env::temp_dir()` and
reads it back (`curl/src/lib.rs:345-390`) because curl delivers a body on stdout.

### `fetch_artifact`

| Must guarantee | Must never |
|---|---|
| Validate limits, then require `dest.parent()` to be an **existing real directory** (not a symlink) | Create or repair a missing destination parent (`lib.rs:639-642`) |
| Require `dest` absent at promotion time; fail explicitly and preserve the foreign file | Overwrite `dest`, on any platform, including a destination that raced in mid-fetch |
| Stream to an exclusively-created, owner-private temp **in the exact `dest` parent** | Buffer the whole body in memory |
| Bound total bytes by `limits.max_artifact_bytes` and report `TooLarge { limit }` | Promote a partial body |
| Re-check cancellation immediately before promotion | Promote after cancellation was observed |
| Remove only its own temp on any failure path | Scavenge by filename prefix, or touch foreign siblings |
| Return `Ok(NotFound)` without creating `dest` | Report `NotFound` for a failure that was actually attempted |

A consequence worth stating plainly: an adapter that promotes successfully cannot then
report a fallback-eligible error. `__promote_no_clobber` returns `Ok` at the commit
point (`lib.rs:591-597`) and the adapters return `Success` on that path
(`eggfetch/src/lib.rs:442-448`; `curl/src/lib.rs:477-483`). This is what makes
artifact fallback under `ComposedTransport` safe against a primary that already
promoted.

## 5. Request and limits

### `AcquisitionRequest`

`new()` (`lib.rs:33-49`) applies four checks, in this order: empty or any
`char::is_control`; scheme must be `http://` or `https://`; `url.len() > 8192`
(**bytes**). On success the URL is stored verbatim — redaction is diagnostics-only
(`lib.rs:29-32`). There is no URL parsing, so host shape is not validated here;
`https://` with an empty authority is accepted by the seam and left to the adapter.

### `FetchLimits`

| Field | Type | `Default` | `validate()` rule |
|---|---|---|---|
| `max_metadata_bytes` | `usize` | `256 * 1024` | `1..=16 * 1024 * 1024` |
| `max_artifact_bytes` | `u64` | `128 * 1024 * 1024` | `> 0` (no upper ceiling) |
| `connect_timeout` | `Duration` | `10 s` | non-zero, and `<= total_timeout` |
| `total_timeout` | `Duration` | `120 s` | non-zero |

`connect <= total` is a hard rule because the connect phase is part of the total
wall-clock budget, so a connect deadline beyond the total deadline cannot be satisfied
truthfully (`lib.rs:112-115`, `102-106`).

The fields stay public for 0.1.x source compatibility, so `validate()` is re-run at
every transport entry point rather than trusted from the constructor
(`lib.rs:87-91`). The fixture calls it first in both methods (`lib.rs:768`, `lib.rs:825`);
both adapters do the same (`eggfetch/src/lib.rs:344`; `curl/src/lib.rs:407`), as does
`ComposedTransport` (`lib.rs:1083`, `lib.rs:1109`).

`max_artifact_bytes` is a mandatory plain `u64`: `None` is not representable, so every
fetch is finitely bounded. Migration from the pre-0.1.2 `Option<u64>` field is
`Some(n) -> n`, and every former `None` must pick a positive bound
([CHANGELOG.md](../crates/eggup-acquisition/CHANGELOG.md)).

### `effective()` — `min(request, adapter ceiling)`

```text
effective connect timeout = min(request connect, adapter connect ceiling)
effective total timeout   = min(request total,   adapter total ceiling)
```

`lib.rs:143-151`. A stricter adapter ceiling tightens a deadline; an adapter can never
silently extend one. Both metadata and artifact operations use the same derivation,
and the effective total covers response headers plus body streaming (`lib.rs:140-142`).

Worked example, using the real in-tree ceilings (both adapters default to
`DEFAULT_CONNECT_TIMEOUT = 10 s` / `DEFAULT_TOTAL_TIMEOUT = 120 s`;
`eggfetch/src/lib.rs:20-22`, `curl/src/lib.rs:24-26`):

| Case | Caller `connect` / `total` | Adapter ceiling | Effective |
|---|---|---|---|
| Caller stricter on both | 2 s / 45 s | 10 s / 120 s | **2 s / 45 s** |
| Adapter stricter on both | 30 s / 300 s | 10 s / 120 s | **10 s / 120 s** |
| Mixed, per phase | 2 s / 300 s | 10 s / 120 s | **2 s / 120 s** |

The third case is the one reviewers get wrong: each phase takes its own minimum
(`effective_timeouts_take_minimums`, `lib.rs:1490-1515`).

A fourth case belongs to composition, not `effective()`. When a fallback adapter runs,
the seam subtracts elapsed time from the caller's single total budget and clamps
`connect_timeout` alongside it to preserve `connect <= total` (`lib.rs:1052-1070`).
Caller total 10 s, primary stalls 300 ms then reports `Unavailable` → the secondary
receives `total ≈ 9.7 s`, `connect = min(1 s, 9.7 s) = 1 s`
(`composition_fallback_shares_one_total_budget`, `lib.rs:1937-1975`). If the budget is
already spent, the fallback is **not attempted** and the result is
`Timeout { phase: "total" }` (`lib.rs:1060-1064`; test `lib.rs:1977-2013`).

## 6. Outcomes and error taxonomy

`FetchOutcome<T>` is two-valued: `Success(T)` or `NotFound` (`lib.rs:185-190`).
`NotFound` is **data** — "this exact URL is definitively absent" — and never triggers
fallback inside a transport (`lib.rs:180-183`). Hard failures are `Err(AcquisitionError)`.

`ArtifactEvidence` reports only `bytes_written` (`lib.rs:237-240`). The seam
deliberately computes **no digest**: SHA-256 is `eggup-core`/`eggup-archive` evidence
([overview.md](overview.md#cross-cutting-invariants)), and the acquisition layer makes
no integrity or authenticity claim.

`AcquisitionError` (`lib.rs:258-297`) is `#[non_exhaustive]`, `Display` is a
per-variant prefix (`lib.rs:339-354`), and every string detail goes through
`bound_detail` → 512 bytes, truncated only at a UTF-8 character boundary
(`lib.rs:380-393`).

| Category | Meaning | Caller action | Composition fallback? |
|---|---|---|---|
| `NotFound` (an `Ok`, not a variant) | Exact URL is definitively absent (HTTP 404 equivalent) | Caller decides; often a clean skip for an optional artifact | **No** — terminal under every policy |
| `InvalidInput(String)` | Caller input violates the seam contract: bad URL, invalid limits, `dest` parent missing/not a real dir, `dest` already exists | Fix the caller; do not retry unchanged | **No** — terminal |
| `Transport(String)` | Request was attempted and failed: TLS, proxy, 5xx, malformed response, early disconnect | Ordinary retry decision, caller-owned; already redacted | Only under `UnavailableOrTransport` |
| `Timeout { phase }` | A connect or total deadline elapsed (`phase` is `"connect"` / `"total"`) | Never treat as `NotFound`; never retry with a longer budget | Only under `UnavailableOrTransport` |
| `TooLarge { limit }` | Response exceeded a caller byte bound; `limit` echoes the **bound**, not the body | Raise the bound deliberately, or fail the update | **No** — terminal |
| `Cancelled` | Cooperative cancellation observed | Not a failure of the update; the caller asked for it | **No** — terminal |
| `Io { operation, source }` | Transaction-owned filesystem operation failed: parent read, temp create, write, flush, promotion | Match `source.kind()`; investigate the destination/staging area | **No** — terminal |
| `Unavailable(String)` | The adapter could not attempt the request at all: missing executable, failed discovery, spawn failure | Retry with a different transport, or fail closed | **Yes** — always, under every policy |

Two distinctions carry the taxonomy:

- `Transport` vs `Unavailable` is "attempted and failed" vs "could not attempt"
  (`lib.rs:261-263` vs `lib.rs:290-296`). This is what makes a curl-without-curl-binary
  situation fall back instead of looking like a broken network.
- `Io` is **always** terminal, including a promotion failure. Silently retrying a
  staging failure on a second transport would hide a real local problem, so the policy
  table treats it as non-fallback (`lib.rs:1081-1091`).

A caller deadline produces `Timeout`, never `NotFound` and never fallback
(`timeout_never_maps_to_not_found`, `lib.rs:1517-1544`).

### 6.1 The typed I/O variant

`Io` is **typed**: it carries the operation that failed and the underlying
`std::io::Error` (`lib.rs:276-302`), mirroring `eggup-core`'s published
`Error::Io { operation, source }`. `impl std::error::Error for AcquisitionError`
overrides `source()` and returns that `io::Error` (`lib.rs:356-367`), so a
consumer can match on `std::io::ErrorKind` (`NotFound` vs `PermissionDenied` vs
`AlreadyExists`) and walk the chain instead of parsing `Display` text.

Three properties are load-bearing and are pinned by fixtures:

- **The rendered detail stays bounded.** `Display` renders
  `"acquisition I/O failed: {operation}: {source}"` through the same
  `bound_detail` rule as every other variant, so the detail never exceeds 512
  bytes and never ends mid-character (`lib.rs:347-350`).
- **No URL or credential material is carried.** `operation` is a static label
  chosen at the construction site and `source` is the raw OS/`io` error; neither
  can contain request material. A fixture proves that a failure raised while
  fetching a credential-bearing URL surfaces neither the userinfo nor the
  query through `Display` or `source()`.
- **It stays terminal.** Retyping did not reclassify the variant: `Io` is
  neither `Unavailable` nor `Transport`, so it never triggers transport
  fallback under either `CompositionPolicy` (`lib.rs:1081-1091`).

**Derive decision (M013, explicit).** `AcquisitionError` derives **only
`Debug`**. `Clone`, `PartialEq`, and `Eq` are **removed**, because
`std::io::Error` implements none of them. This matches `eggup-core`'s
published `Error`, which derives only `Debug` for exactly the same reason. The
alternative — preserving the three derives through a manual implementation
over the operation plus the error's kind and rendered message — was rejected:
it would have to fabricate a *different* `io::Error` on `clone`, so the clone's
`source()` would no longer be the error the caller was given, which is the
property this variant exists to provide. The consequence is source-breaking
and is stated in the crate changelog; the next publication of this crate is a
source-breaking release (`0.2.0`, or the 0.2/1.0 boundary release), **not** a
`0.1.4`.

## 7. Destination safety

The seam's staging sequence, as coded in the fixture (`lib.rs:818-946`) and followed
closely by both adapters:

| Step | Operation | Line | Failure result |
|---|---|---|---|
| 1 | `limits.validate()` | `825` | `InvalidInput`, before any I/O |
| 2 | cancellation pre-check | `826-828` | `Cancelled` |
| 3 | `dest.parent()` must exist | `830-832` | `InvalidInput("artifact destination has no parent")` |
| 4 | parent must be a real directory, not a symlink | `833-839` | `InvalidInput` |
| 5 | fast-fail if `dest` exists | `843-847` | `InvalidInput`, foreign bytes preserved |
| 6 | route lookup | `848` | `Transport` (unregistered URL) |
| 7 | `max_artifact_bytes` check | `884-888` | `TooLarge`, no temp created |
| 8 | `__acquire_exclusive_temp(parent, prefix)` | `896` | `Io` on exhaustion; `0600` on Unix |
| 9 | chunked write (8 KiB) with per-chunk cancel check | `916-923` | `Cancelled` / `Io`; `Drop` guard removes the owned temp |
| 10 | `flush()`, then drop the handle | `924-927` | `Io` |
| 11 | **second** cancel check | `929-931` | `Cancelled`; never promote after cancel |
| 12 | `__promote_no_clobber(tmp, dest)` | `932-945` | `InvalidInput` (dest exists) or `Io`; owned temp removed, foreign `dest` untouched |

### Temp creation (`__acquire_exclusive_temp`, `lib.rs:503-564`)

- `OpenOptions::new().write(true).create_new(true)` — creation fails if the path
  exists, so it never truncates a foreign file and never follows a precreated symlink
  (`lib.rs:518-525`, `lib.rs:553`).
- Name: `.{prefix}-{pid}-{nanos}-{nonce}.part` in the **exact** destination parent, so
  promotion stays on one filesystem and no parent is ever created
  (`lib.rs:514-517`).
- Permissions: `mode(0o600)` at create, then a post-create repair that normalizes the
  mode to exactly `0600` whenever it differs (`lib.rs:520-550`). Both failure paths in
  the repair drop the handle and remove the candidate before returning `Io`, because
  this function is the only owner of the path at that point — callers build their
  `Drop` guard only after it returns `Ok` (`lib.rs:528-530`).
- Collision bound: `__TEMP_COLLISION_BOUND = 32` attempts, each with a fresh
  `AtomicU64` nonce plus wall-clock nanos and the pid (`lib.rs:489`, `508-509`).
  Exhaustion is `Io { operation: "creating part file", source: AlreadyExists
  "too many temporary name collisions" }` (`lib.rs:557-563`) — bounded failure,
  not unbounded retry.

### Promotion (`__promote_no_clobber`, `lib.rs:582-603`)

`fs::hard_link(tmp, dest)`, then a best-effort unlink of the redundant temp name.

- `AlreadyExists` → `InvalidInput("artifact destination already exists; refusing to
  overwrite")`; the foreign destination is preserved (`lib.rs:598-600`).
- Any other error → `Io("promoting artifact")` (`lib.rs:601`).
- Success → `Ok` regardless of whether the temp unlink succeeded (`lib.rs:592-596`).
  **The hard link is the commit point**; cleanup after it cannot retroactively turn a
  complete destination into a failure. This was the M004 defect
  ([004-status.md](../plans/closure/acquisition-transport/004-status.md)), and
  `post_link_cleanup_failure_keeps_committed_destination_successful`
  (`lib.rs:1585-1601`) injects the unlink failure to prove it.

`hard_link` was chosen over `rename` because `rename` silently replaces on Unix and
fails on Windows — divergent no-clobber semantics (`lib.rs:577-580`). It fails closed
on both, and `AlreadyExists` maps to the same category on both platforms.

### What makes a partial write unable to become the live artifact

1. `dest` is never opened for writing. Bytes only ever reach a temp created with
   `create_new`.
2. The temp is in `dest`'s own parent, so the only remaining step is a name operation
   that either fully succeeds or fully fails.
3. Promotion requires `dest` to be absent, and the primitive fails closed if it is
   not — including when a foreign file appears *during* the fetch
   (`promote_no_clobber_preserves_raced_destination`, `lib.rs:1603-1624`).
4. Cancellation is re-checked after the write and before promotion, so a cancel
   observed mid-stream cannot promote.
5. Every failure path removes only the exact owned temp via `__remove_owned_temp`
   (`remove_file`, never recursive, never prefix-based — `lib.rs:605-613`) or a
   `Drop` guard disarmed only on commit (`lib.rs:897-911`).

**What this is not:** the commit point is *name-level*. `flush()`
(`lib.rs:924-925`) is not `sync_all()`, and neither the file nor the parent directory
is fsynced. The seam therefore makes **no durability claim**. That is consistent with
the workspace rule that no crate may claim crash journaling
([overview.md](overview.md#boundary-rules-what-must-not-cross)), but a reviewer should
not read "no-clobber promotion" as "crash-durable".

## 8. Redaction

Three mechanisms, with different jobs:

| Mechanism | Scope | Credential block | Query | Fragment | Length cap |
|---|---|---|---|---|---|
| `AcquisitionRequest::redacted()` (`57-59`) | delegates to `redact_url` | — | — | — | — |
| `redact_url` (`lib.rs:404-432`) | one URL, diagnostics | first `@` after `://` → `://<redacted>@` | always → `?<redacted>` | always → `#<redacted>` | 256 bytes + `…` |
| `__scrub_upstream_text` (`lib.rs:441-490`) | arbitrary text, defense-in-depth | `://…@` blocks ≤512 bytes, advancing cursor | only if the `?` suffix contains `=` or `&` | not handled | 512 bytes, no ellipsis |

Preserved by `redact_url`: scheme, the `<redacted>@` marker, host, port, and path.
Stripped: `user:password@` (or a bare `user@`), the whole query string, the whole
fragment, and everything past 256 bytes.

Three details a reviewer should know:

- The fragment is redacted **before** the query (`lib.rs:414-423`) specifically so a
  fragment cannot smuggle a token past query handling.
- The credential strip takes the **first** `@` after the scheme, so
  `https://a@b@example.com/x` becomes `https://<redacted>@example.com/x` — the
  conservative reading.
- Truncation is at 256 **bytes**, then the 3-byte `…` is appended, so the returned
  string can reach 259 bytes (`lib.rs:426-430`).

The primary defense is not the scrubber but **category-only diagnostics**: transport
failures never embed upstream or proxy error `Display`, so credential-bearing upstream
strings have no path into `Display` (`lib.rs:395-403`). Production adapters follow
this: `eggup-eggfetch` maps proxy errors to a fixed
`"proxy configuration/routing failure"` string (`eggfetch/src/lib.rs:490-491`,
`534-535`). The fixture enforces the same discipline — `FixtureResponse::failure(detail)`
accepts a detail for call-site compatibility and drops it (`lib.rs:693-697`,
`729-733`), so a test that writes a secret into a fixture detail proves it cannot leak
(`redaction_sentinels_are_absent_from_diagnostics`, `lib.rs:1776-1812`).

## 9. Composition

`CompositionPolicy` (`lib.rs:979-987`) has exactly two values:

| Variant | Fallback triggers | Default |
|---|---|---|
| `UnavailableOnly` | `Unavailable` only | Yes (`#[default]`, `lib.rs:982-983`) |
| `UnavailableOrTransport` | `Unavailable`, plus `Transport` and `Timeout { .. }` | No — explicit caller opt-in |

`ComposedTransport<'a>` (`lib.rs:998-1002`) borrows two `&'a dyn AcquisitionTransport`
values and implements `AcquisitionTransport` itself, so callers keep one seam.
`new(primary, secondary, policy)` records order and policy only — no network,
filesystem, or process work (`lib.rs:1017-1027`). Preferred order is construction order.
`Debug` is `finish_non_exhaustive` and prints only the policy, so a composed transport
cannot leak adapter internals through a log line (`lib.rs:1004-1010`).

Behavior on partial failure:

| Primary outcome | `UnavailableOnly` | `UnavailableOrTransport` |
|---|---|---|
| `Ok(Success)` | return, secondary untouched | return, secondary untouched |
| `Ok(NotFound)` | return `NotFound` | return `NotFound` |
| `Err(Unavailable)` | secondary with reduced budget | secondary with reduced budget |
| `Err(Transport)` / `Err(Timeout)` | return the error | secondary with reduced budget |
| `Err(InvalidInput)` / `TooLarge` / `Cancelled` / `Io` | return the error | return the error |

Decisions are made from typed outcomes by `should_fallback` (`lib.rs:1034-1042`) — never
by parsing human-readable error text (`lib.rs:995-997`). Both methods validate limits
and check cancellation at the composition boundary before either adapter does any
work (`lib.rs:1083-1087`, `lib.rs:1109-1112`).

**Composition is transport fallback, never release fallback.** Both transports are
handed the *same* `request` object, so a fallback can only ever attempt the identical
exact URL. There is no mechanism in the seam to substitute a different URL, host, path,
or version — `ComposedTransport` holds no URL state at all
(`lib.rs:998-1002`). Silently swapping a release source would be policy the caller never
granted ([overview.md](overview.md#cross-cutting-invariants),
[ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)).
Verification and candidate failures occur above this layer and are never
transport-fallback inputs (`lib.rs:976-977`).

## 10. Fixture transport

`FixtureTransport` (`lib.rs:728-730`) is a `Mutex<HashMap<String, FixtureResponse>>`
behind exact-URL keys. `route()` takes `&self`, so a shared `SharedTransport` can be
routed after construction (`lib.rs:739-744`, `lib.rs:949-950`).

`FixtureResponse` constructors (`lib.rs:675-719`) map to `FixtureKind`
(`lib.rs:664-671`):

| Constructor | Metadata path | Artifact path |
|---|---|---|
| `body(bytes)` | `Ok(Success(MetadataBytes))` after bounds check | streamed, promoted, `bytes_written` |
| `not_found()` | `Ok(NotFound)` | `Ok(NotFound)`, no `dest`, no temp |
| `failure(detail)` | `Err(Transport("fixture transport failure"))` — detail dropped | same, after the dest pre-checks |
| `truncated(prefix_len)` | `Err(Transport("truncated fixture body"))` | writes a partial temp, removes it, `Err(Transport("early disconnect during artifact body"))` |
| `slow(body, delay)` | `Timeout { phase: "total" }` if `delay > total_timeout`, else immediate success | `Timeout` if `delay > total_timeout` or `elapsed + delay > total_timeout`, else immediate success |
| `unavailable()` | `Err(Unavailable)` | `Err(Unavailable)` |

Determinism the fixture guarantees:

- **No network, ever.** Every URL must be registered; an unregistered URL is a hard
  `Transport` failure with only the redacted URL in the message
  (`lib.rs:746-758`) — never a silent fallback, so a test cannot accidentally pass by
  reaching something.
- **No sleeping.** `slow` *models* a delay purely as a comparison against
  `limits.total_timeout` (`lib.rs:801-814`, `lib.rs:877-882`); wall-clock checks use
  `Instant::now()` on the same thread. A `slow(body, 30 s)` fixture returns in
  microseconds.
- **Same ordering as a real transport.** Validate → cancel → parent → `dest` → lookup →
  bound → write → cancel → promote, so fixture behavior is a fair model of what an
  adapter must do.

Two fidelity limits to know when writing assertions: the byte bound is checked
*before* writing because the fixture already holds the whole body in memory
(`lib.rs:884-888`) — a real adapter must enforce the bound mid-stream (see
`eggfetch/src/lib.rs:425-430`); and `truncated` writes `prefix_len.min(1024)` **zero**
bytes, not a prefix of a real body (`lib.rs:866-872`).

Why a fixture-based test is meaningful evidence for the seam: it pins the contract in
the only place where a reviewer can assert it without a network. The composition tests
(`lib.rs:1833-2229`) go further — `CountingTransport` and `StallingTransport` are
hand-written `AcquisitionTransport` implementations used to prove fallback counts and
budget arithmetic without depending on any real adapter.

## 11. Adapter implementor checklist

Review a new adapter against this list:

- [ ] `limits.validate()` is the first statement, before route lookup, filesystem, and network I/O.
- [ ] Time bounds come from `limits.effective(adapter_connect, adapter_total)` and nowhere else. No hard-coded timeout, no upward rounding of a sub-second bound.
- [ ] `dest.parent()` exists and is a real directory; the adapter never creates it. `dest` is checked absent, and promotion is `__promote_no_clobber`.
- [ ] Bytes go to `__acquire_exclusive_temp(dest_parent, "<adapter-prefix>")`; `prefix` is a literal with no path separators.
- [ ] The byte bound is enforced **during** streaming, not only before it, and `TooLarge { limit }` reports the real bound.
- [ ] Cancellation is checked before the fetch, per chunk, and once more immediately before promotion.
- [ ] Cleanup uses `__remove_owned_temp` or a `Drop` guard disarmed only on commit. No prefix scavenging, no recursion, no touching foreign siblings.
- [ ] Every error is a category plus `redact_url`d material. No upstream/proxy `Display`, no `e.kind()` that embeds a credential-bearing URL.
- [ ] `Unavailable` is used only for "could not attempt" (missing executable, failed discovery, spawn failure). Use `__adapter_unavailable` so the detail is bounded.
- [ ] `Ok(NotFound)` only for a definitive absence. Timeout, cancellation, and bound violations are `Err`, never `NotFound`.
- [ ] The adapter returns seam types only through `__adapter_metadata` / `__adapter_artifact`, and does not duplicate temp/promotion logic locally.
- [ ] The adapter does not know the other adapter exists, and never constructs a different URL.

### 11.1 Adapter capability parity: the `User-Agent` asymmetry (recorded, resolved by M014)

**The seam contract does not include a user-agent.** `AcquisitionRequest` carries
only a URL (`lib.rs:24-27`) and `AcquisitionTransport` carries only the request,
the destination, the limits, and the cancellation flag (`lib.rs:625-653`). There
is no seam-level user-agent invariant, so the difference between the two adapters
violates **no** seam contract.

**The difference as M013 recorded it** (before M014):

| Adapter | User-agent surface | Default |
|---|---|---|
| `eggup-eggfetch` | Caller-settable: `EggfetchConfig::user_agent` | `DEFAULT_USER_AGENT` |
| `eggup-curl` | **None** | curl's own default |

**The concrete consequence M013 recorded.** A caller that switches transports
loses control of the `User-Agent`: the wire value changes from the
`eggup-eggfetch` default (or whatever the caller set) to curl's own default. On
a transport switch the request is therefore *not* byte-identical on the wire,
even though the seam contract is satisfied by both.

#### M014's resolution: capability parity

M014 closes the capability gap in the curl adapter. A caller can now set the
outbound `User-Agent` on **either** adapter, for **every** request that adapter
makes:

| Adapter | User-agent surface | Default | Placement |
|---|---|---|---|
| `eggup-eggfetch` | Caller-settable: `EggfetchConfig::user_agent` (`eggfetch/src/lib.rs:52-87`) | `DEFAULT_USER_AGENT` (`eggup-eggfetch`) | `EggfetchConfig` (published `0.1.3`) |
| `eggup-curl` | Caller-settable: `CurlTransport::with_executable_and_user_agent` (`curl/src/lib.rs:238-257`) | **none sent**; curl's own default applies | `CurlTransport` (private field, `curl/src/lib.rs:174-179`) |

**Parity means capability parity, not placement parity.** Both adapters let a
caller set a `User-Agent`; that is the claim, and it is what M014 delivers.
It is explicitly **not** placement parity: the two controls live in different
types and are deliberately not aligned. `eggup-eggfetch`'s control lives in
`EggfetchConfig`, whose `user_agent` is a public field of a public,
non-`#[non_exhaustive]` struct in a crate published at `0.1.3`; moving it
would be a source break for published consumers. `eggup-curl`'s control lives
on `CurlTransport` because `CurlConfig`'s fields are public and **not**
`#[non_exhaustive]` (`curl/src/lib.rs:58-76`), so adding a field there would
be source-breaking for every external struct-literal construction. Each
adapter therefore keeps the placement its own publication history allows, and
neither may be changed to match the other.

**Parity is also not default-value parity, deliberately.** The eggfetch adapter
defaults to `eggup-eggfetch`; the curl adapter sends **no** `User-Agent` by
default, so curl applies its own default (`curl/<version>`). Making the curl
adapter default to `eggup-curl` would have changed the observable
`User-Agent` header of every consumer of the published `eggup-curl 0.1.2`, for
no concrete benefit. The default wire behavior of the published adapter is
therefore unchanged, and the default-value difference is recorded here as a
deliberate choice rather than a gap.

**Validation is the adapter's own responsibility.** The curl adapter validates
the value against a stated grammar (`1*( VCHAR / SP / obs-text )`, at most
`MAX_USER_AGENT_BYTES` = 512 bytes) before it can reach the wire, and rejects
every violation as `AcquisitionError::InvalidInput`. `InvalidInput` is
terminal under every composition policy (§ "Composition"), so a malformed
caller value can never turn into a silent fallback to another adapter. The
validation is not redundant with curl's own behavior: curl does **not**
sanitize `--user-agent`, and a value containing `CR LF` is written into the
request head verbatim, injecting an additional header.

**How parity is enforced.** A mechanical cross-adapter parity test is
**impossible** here and none is claimed: `eggup-curl` must not know
`eggup-eggfetch` exists, `eggup-eggfetch` must not know `eggup-curl` exists,
and the seam must not depend on either adapter. A test that asserted "both
adapters expose the same capability" would have to import both adapters into
one crate, violating the boundary in one direction or the other. Parity is
therefore enforced by **per-adapter tests** (each adapter proves its own
control) plus **this documented contract**, which a reviewer compares as two
documented contracts rather than as a shared assertion.

**What remains open.** Nothing at the capability level: M013's UF-1 finding is
closed. The two remaining differences — placement and default value — are
deliberate, documented consequences of the two adapters' separate publication
histories, and are recorded as **low** severity rather than as a defect.

### 11.2 Why the curl adapter owns its bounded subprocess runner (recorded)

`eggup-curl` implements its own bounded subprocess runner
([curl-adapter.md](curl-adapter.md) §7) **because it may depend only on
`eggup-acquisition`** (`curl/src/lib.rs` declares exactly one dependency). The
workspace's other bounded runners are therefore unreachable from it:

| Runner | Owner | Why it is unreachable from `eggup-curl` |
|---|---|---|
| `run_bounded` | `eggup-core` | `eggup-core` is *above* the seam in the graph; an `eggup-curl` → `eggup-core` edge would invert the dependency direction the seam sits under |
| `SystemExecutor::run` | `eggup-service` | `eggup-service` is above `eggup-core`; an `eggup-curl` → `eggup-service` edge would create a cycle and pull the service-manager surface into a transport adapter |

Adding either dependency edge to remove the duplication would be a **boundary
violation**, not a simplification. The duplication is therefore deliberate and
bounded.

The runner shares one shape with core's `run_bounded`: an **unbounded post-exit
reader join**. The deadline defect that shape carries is tracked by
`plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`
and is **out of scope for this seam milestone** — M013 changes no process
lifecycle, deadline, kill/reap behavior, or environment clearing in the curl
runner, and does not depend on that milestone closing.

## 12. Testing approach

38 `#[test]` functions, all in `lib.rs:1159-2229`, all offline, no sleeps. Groups:

| Group | Lines | Demonstrates |
|---|---|---|
| Outcome classification | `1159-1298` | exact body success, 2048-byte streamed artifact, metadata/artifact `TooLarge` (artifact asserts `!dest.exists()`), `NotFound`-is-data for both paths, failure ≠ `NotFound`, `Timeout` is hard |
| Cancellation and partial writes | `1300-1346` | pre-cancelled fetch leaves no `.part`; `truncated` promotes nothing |
| Redaction | `1348-1370`, `1776-1831` | userinfo/query/fragment sentinels absent from `Display`; fixture failure detail never echoed; `__scrub_upstream_text`; UTF-8-safe truncation at the 256/512-byte edges |
| Limits validation | `1372-1487` | zero artifact bound; a 5-literal invalid matrix rejected before I/O with an empty destination dir; `connect > total`; zero timeouts; `effective()` min-per-phase matrix |
| Destination and temp safety | `1449-1772` | missing parent creates nothing; existing `dest` preserves `FOREIGN-BYTES`; raced `dest` preserves `RACED-FOREIGN`; post-link cleanup failure still `Ok`; promote-ok removes the temp; Unix-only `0600`, no truncate, no symlink follow; cancel and early disconnect preserve a foreign sibling and remove only owned temps |
| Timeout truthfulness | `1517-1544` | `Timeout` never becomes `NotFound`; nothing promoted |
| Composition | `1833-2228` | one shared total budget; no attempt after exhaustion; primary preferred; default policy does not retry `Transport`; broad policy retries `Transport` and `Timeout`; `NotFound`/`Cancelled`/`TooLarge`/`InvalidInput` terminal; artifact fallback preserves no-clobber |

Two **integration** suites live in `tests/` (they are excluded from the
published package by `Cargo.toml`'s `exclude`):

| Suite | Tests | Demonstrates |
|---|---|---|
| `tests/typed_io_contract.rs` | 5 | the typed `Io` variant preserves the underlying `io::Error` and its `ErrorKind`; `source()` returns it; the rendered detail keeps the 512-byte UTF-8-safe bound; the typed path surfaces no URL material; `Io` is terminal under **both** composition policies; `redact_url` / `__scrub_upstream_text` / `__adapter_bound_detail` keep their exact byte caps after the truncation consolidation |
| `tests/assertion_guard.rs` | 3 | the **pattern guard** (no vacuous early exit in this crate's sources), the **reason guard** (every ignored test records a reason), and the **behavior witness** (each required behavior has a named, non-vacuous, asserting test) |

The guards read the crate's own sources at compile time with `include_str!`, so
they need no filesystem discovery, no network, and no wall clock.

Commands:

```sh
cargo test -p eggup-acquisition
cargo test -p eggup-acquisition composition      # one group
cargo test -p eggup-acquisition promote_no_clobber
cargo test -p eggup-acquisition --test assertion_guard
cargo clippy -p eggup-acquisition --all-targets --locked -- -D warnings
```

Full gate: `./scripts/check-local.sh` (fmt, clippy, workspace tests, doc, tree) — see
[tooling-governance.md](tooling-governance.md). CI adds MSRV 1.89 `cargo check`, macOS
tests, and Windows `cargo check`.

## 13. Cross-references

- [overview.md](overview.md) — index, dependency graph, cross-cutting invariants, composition paths.
- [eggfetch-adapter.md](eggfetch-adapter.md) — native HTTP adapter, status classification, streaming.
- [curl-adapter.md](curl-adapter.md) — external-process adapter, executable discovery, protocol/proxy control.
- [transport-footprint.md](transport-footprint.md) — what each transport choice links.
- [eggpack-adapter.md](eggpack-adapter.md) — the only producer-aware consumer of `AcquisitionRequest` / `FetchLimits`.
- [core-transaction.md](core-transaction.md) — what happens to acquired bytes next.
- [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) — transport neutrality and the verification layers.
- [acquisition-transport-roadmap.md](../plans/subsystems/acquisition-transport-roadmap.md) — M001–M010; M010 republishes this seam as 0.1.3 with the two post-0.1.2 fixes.
- Closure records: [001](../plans/closure/acquisition-transport/001-status.md) seam, [003](../plans/closure/acquisition-transport/003-status.md) timeout/temp/redaction, [004](../plans/closure/acquisition-transport/004-status.md) validated limits + promotion state, [005](../plans/closure/acquisition-transport/005-status.md) curl + composition, [007](../plans/closure/acquisition-transport/007-status.md) boundary hardening, [008](../plans/closure/acquisition-transport/008-status.md) sub-second deadlines.

## 14. Known doc/code drift

- **Test counts in closure records lag the tree.** M007 records "acquisition 34
  passed" and M008 records "acquisition 36"; the current file has 38 `#[test]`
  functions. Treat closure counts as historical evidence, not a live inventory.
- **`__acquire_exclusive_temp` umask comment is inverted.** The comment says the
  post-create permission repair handles a "permissive umask" (`lib.rs:527-528`). A
  permissive umask cannot add bits to `mode(0o600)`, so creation already yields exactly
  `0600`; the repair can only trigger when the umask *masks owner bits* (e.g. `0222` →
  `0400`). Code behavior: the mode is normalized to exactly `0600` whenever it differs
  (`lib.rs:542-549`).
- **`FixtureResponse::slow` does not wait.** Its doc says "A body delivered only after
  `delay`" (`lib.rs:706-707`); the code returns immediately and uses `delay` only as a
  comparison against `limits.total_timeout` (`lib.rs:801-814`, `lib.rs:877-882`). The tests
  depend on the simulated form.
- **`README.md` omits `redact_url`'s length cap.** The README Diagnostics section lists
  userinfo, query, and fragment only; the code also truncates at 256 bytes and appends
  `…` (`lib.rs:426-430`).
- **`FetchLimits::validate` bounds are asymmetric.** Metadata has a 16 MiB ceiling;
  `max_artifact_bytes` has only a positivity check (`lib.rs:96-98`), so `u64::MAX`
  validates. The README's "positive finite artifact-byte cap" is accurate, but the
  asymmetry is easy to misread as a missing bound.
- **`__scrub_upstream_text` is documented as defense-in-depth and has no in-tree
  caller.** Neither adapter uses it; both rely on category-only text plus
  `redact_url` (`lib.rs:434-439`).
- **Detail bounding survives as one implementation per unreachable dependency
  group.** M013 consolidated the group-1 copies into this crate:
  the two **inlined** UTF-8-safe truncation walks that used to sit inside
  `redact_url` (the 256-byte cap plus `…`) and `__scrub_upstream_text` (the
  512-byte cap) now both call this crate's own `truncate_utf8_bytes`
  (`lib.rs:437-445`), and `eggup-curl` / `eggup-eggfetch` delegate their private
  `bound` helpers to the adapter-facing `__adapter_bound_detail`
  (`lib.rs:429-431`) instead of keeping a local copy. The **group-2** copy —
  `eggup-service`'s `bounded_detail` (`service/src/commit_engine.rs:236`) —
  is **not** reached by this crate and is owned by
  `plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`,
  which reduced that crate to one implementation for the whole crate; M013
  does not touch it. The two groups cannot be merged because no location
  is reachable by both: the seam has **zero dependencies** and `eggup-core`
  depends only on `sha2`, so the seam cannot import a core or service helper
  without adding a dependency edge to a crate that must stay dependency-free.
  The duplication is therefore **bounded and deliberate**, not accidental.
  Each remaining copy is pinned by a cap test: `redact_url` (256 bytes + `…`),
  `__scrub_upstream_text` (512 bytes, no ellipsis), and `__adapter_bound_detail`
  (512 bytes) in `tests/typed_io_contract.rs`. The service-side pair is pinned
  by its own owner, service M012.
- **`eggup-curl` no longer constructs `AcquisitionError::Unavailable` directly.**
  M013 routed the two discovery call sites and the spawn-failure site through
  `__adapter_unavailable` (`curl/src/lib.rs:512-513`, `531-533`, `curl/src/lib.rs:775-777`), so
  the seam's bounding now applies to them. Classification and content are
  unchanged.
