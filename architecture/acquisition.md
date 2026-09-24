# Acquisition — `crates/eggup-acquisition` Deep Dive

Source: `crates/eggup-acquisition/src/lib.rs` (`#![forbid(unsafe_code)]`,
`#![deny(missing_docs)]`), `README.md`, `Cargo.toml` (zero `[dependencies]`,
workspace lints/version/edition; description: "Transport-neutral acquisition
seam and deterministic fixture transport").

## 1. Purpose and transport-neutrality boundary

`eggup-acquisition` is the narrow seam between caller policy and byte movement.
An adapter receives an exact caller-selected URL, enforces caller-provided
byte/time bounds, writes to a transaction-owned destination, and returns typed
`Success | NotFound | Failure`. It owns **no** release/version selection, mirror
choice, discovery, fallback, destination policy, service wiring, or execution —
downloaded bytes are data and are never executed (`lib.rs:1-8`, `README.md:5-17`,
`AcquisitionTransport` docs `lib.rs:516-526`).

The real network policy lives in `eggup-eggfetch`; this crate ships only the
trait plus a deterministic in-memory `FixtureTransport` for correctness tests
(`lib.rs:614-618`).

Key invariant: `NotFound` is data (exact URL absent, HTTP-404 equivalent); the
consumer decides its meaning. Hard failures (`Transport | TooLarge | Timeout |
Cancelled | Io | InvalidInput`) never trigger fallback inside a transport
(`lib.rs:177-187,246-269`).

## 2. Public API (factual)

- `AcquisitionRequest` (`lib.rs:24-60`): wraps exact `url: String` (private
  field, `url()` / `redacted()` accessors). `new()` rejects empty, control-char,
  non-`http(s)` scheme, `len > 8192`; stores URL **verbatim** — redaction is
  diagnostics-only.
- `FetchLimits` (`lib.rs:63-149`): public fields for 0.1.x source compat —
  `max_metadata_bytes: usize`, `max_artifact_bytes: Option<u64>`,
  `connect_timeout / total_timeout: Duration`. `Default`: 256 KiB metadata,
  128 MiB artifact, 10 s connect, 120 s total. `validate()` requires
  `1..=16 MiB` metadata, non-zero timeouts, `connect <= total` (connect is part
  of total budget). `new()` = construct + `validate()`. `effective(adapter_
  connect_ceiling, adapter_total_ceiling) -> (Duration, Duration)` returns
  per-phase `min(request, adapter ceiling)` — an adapter may tighten, never
  extend; both metadata and artifact paths use the same derivation; total covers
  headers + body streaming.
- `CancelFlag` (`lib.rs:156-175`): `AtomicBool` + `SeqCst`, `new/cancel/
  is_cancelled`. Checked pre-fetch, mid-stream per 8 KiB chunk, and pre-promote.
- `FetchOutcome<T>` (`lib.rs:182-207`): `Success(T) | NotFound`, with
  `success()/is_success()/is_not_found()`.
- `MetadataBytes` (`lib.rs:211-230`): private `bytes: Vec<u8>`, `bytes()/len()/
  is_empty()` only — no general constructor for consumers.
- `ArtifactEvidence` (`lib.rs:234-244`): `{ bytes_written: u64 }` (public field
  + getter).
- `AcquisitionError` (`lib.rs:248-298`, `#[non_exhaustive]`): `InvalidInput(
  String)`, `Transport(String)` (already redacted), `TooLarge{limit: u64}`,
  `Timeout{phase: &'static str}`, `Cancelled`, `Io(String)`. `Display` prefixes
  per variant; `bound_detail()` truncates details to 512 chars. Constructors
  `invalid / transport_redacted / io` are `pub(crate)`.
- `AcquisitionTransport` trait (`lib.rs:526-554`): `fetch_metadata(request,
  limits, cancel) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError>` and
  `fetch_artifact(request, dest, limits, cancel) -> Result<FetchOutcome<
  ArtifactEvidence>, AcquisitionError>`. Artifact contract: stream to
  exclusively-created owner-private temp sibling, promote no-clobber only on
  full success; `dest` parent must already exist as a real directory.
- `FixtureResponse` / `FixtureKind` (`lib.rs:559-612`): `body(Vec<u8>)`,
  `not_found()`, `failure(detail)` (detail accepted for call-site compat but
  never exposed — mirrors production category-only errors), `truncated(
  prefix_len)`, `slow(body, delay)`.
- `FixtureTransport` (`lib.rs:620-831`): `Mutex<HashMap<String,
  FixtureResponse>>`, `new()/route(url, response)/lookup()`. Unregistered URL =
  hard `Transport` failure with redacted URL — never silent fallback. Both
  methods call `limits.validate()` first, before route/filesystem/network I/O.
- Hidden/seam helpers (`#[doc(hidden)]`): `redact_url()` (public, `lib.rs:316`),
  `__scrub_upstream_text` (defense-in-depth, `lib.rs:350`), `__TEMP_COLLISION_
  BOUND = 32` (`lib.rs:400`), `__acquire_exclusive_temp(parent, prefix)` (
  `lib.rs:414`), `__promote_no_clobber(tmp, dest)` (`lib.rs:483`) + private
  `__promote_no_clobber_with` injection point for tests, `__remove_owned_temp(
  path)` (`lib.rs:512`), `__adapter_metadata(Vec<u8>)` / `__adapter_artifact(
  u64)` (adapter-only constructors, `lib.rs:842/850`), `SharedTransport = Arc<
  FixtureTransport>` (`lib.rs:834`).

## 3. Safety rules (as coded)

1. **Verbatim URL**: `AcquisitionRequest::new` stores the exact string; transport
   does no discovery/mirror/fallback. Lookup key is the exact URL
   (`lib.rs:638-650`).
2. **Redaction**: `redact_url` strips `user:pass@`, truncates `?...` → `?<
   redacted>`, `#...` → `#<redacted>` (fragment first), truncates >256 chars +
   `…`. Upstream/proxy error `Display` is never embedded — category text +
   redacted URL only; `FixtureResponse::failure` detail is dropped. `__scrub_
   upstream_text` additionally scrubs `://cred@` blocks (≤512 chars, advancing
   cursor, skips `://`-before-`@` false positives) and `?`-with-`=`/`&` suffixes,
   truncates to 512 (`lib.rs:316-396`).
3. **Bounds**: metadata `len > max_metadata_bytes` → `TooLarge`; artifact `len >
   max_artifact_bytes` → `TooLarge` before any promotion; `TooLarge.limit`
   echoes the bound, not the body. Invalid public struct literals rejected at
   I/O boundary (`lib.rs:660,714`).
4. **0600 temp**: `__acquire_exclusive_temp` uses `create_new` (fails on
   existing/symlink path), `.part` sibling in exact `dest` parent (same
   filesystem), `0o600` via `OpenOptionsExt` + post-create `set_permissions`
   repair under permissive umask (Unix), nonce = `pid + nanos + AtomicU64`,
   32-attempt collision bound (`lib.rs:414-465`).
5. **No-clobber**: `__promote_no_clobber` = `hard_link(tmp, dest)`; `AlreadyExists`
   → `InvalidInput("...refusing to overwrite")`, foreign bytes preserved;
   success → best-effort temp unlink that cannot fail the commit; other `io`
   errors → `Io("promoting artifact")`. Avoids Unix-`rename`-overwrite vs
   Windows-`rename`-fails divergence. Fast-fail `symlink_metadata(dest).is_ok()`
   pre-check + race-safe link covers TOCTOU (`lib.rs:467-504,732-736,816-829`).
6. **Parent validation**: `dest.parent()` missing → `InvalidInput`; `symlink_
   metadata(parent)` must be non-symlink dir, else `InvalidInput`; never creates
   parents (`lib.rs:719-728`).
7. **Cancellation**: pre-check, per-chunk (8 KiB), and pre-promote re-check —
   never promote after cancel. Failure/cancel cleans only the owned temp via
   `Drop` guard (`disarm` on commit); no prefix scavenging; foreign siblings
   preserved (`lib.rs:780-795,800-815`).

## 4. Fixture determinism and tests (~30 tests, `lib.rs:854-1539`)

Deterministic: no network, no clock sleeps (logical `delay` vs `total_timeout`
comparison + `Instant::now()` elapsed checks), `Mutex<HashMap>` routes, temp
dirs under `std::env::temp_dir()` with `pid-nanos` names. Coverage: exact-body
and 2048-byte streamed-artifact success; metadata/artifact `TooLarge` (artifact
asserts `!dest.exists()`); `NotFound`-is-data for both paths; failure≠`NotFound`;
`Slow(30 s)` → `Timeout{phase:"total"}` for metadata and artifact (never `NotFound`
); pre-cancelled `Cancelled` + no `.eggup-acquire-*.part` residue; `Truncated`
partial-write-then-`Transport` with owned-temp-only cleanup; redaction sentinels
(`S3CR3T-USERINFO-9917`, `TOKEN-QUERY-5523`, `FRAG-SENTINEL-7788`,
`HUNTER2-PROXY-3311`) absent from `Display`; unregistered URL → `Transport`;
5 invalid-limit literals rejected before I/O with empty dir; missing parent
creates nothing; `connect > total` / zero timeouts rejected; `effective()`
min-per-phase matrix; no-clobber preserves `FOREIGN-BYTES`/`RACED-FOREIGN`;
post-link-cleanup failure still `Ok` with committed bytes; promote-ok removes
temp; Unix-only 0600 / no-truncate / no-symlink-follow; cancel/disconnect
preserve foreign sibling and remove only owned `.part`.

## 5. Review checklist

- [ ] New transports call `limits.validate()` before any route/fs/net I/O.
- [ ] New timeouts derive via `limits.effective(ceiling_c, ceiling_t)`; never
  extend caller deadlines; `Timeout`, never `NotFound`/fallback.
- [ ] Artifact path uses `__acquire_exclusive_temp` → chunked write → cancel
  re-check → `__promote_no_clobber`; cleanup via `__remove_owned_temp`/guard only.
- [ ] Errors are category + `redact_url`; never embed upstream `Display`;
  use `__scrub_upstream_text` only as defense-in-depth.
- [ ] `dest` parent pre-validated (real dir, no auto-create); `dest` absent at
  commit; foreign bytes never overwritten or scavenged.
- [ ] Consumers use `FetchOutcome` (`Success` vs `NotFound` data) vs `Err` hard
  failure correctly; adapters construct seam types only via `__adapter_*`.
