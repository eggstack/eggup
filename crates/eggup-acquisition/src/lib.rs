#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Transport-neutral acquisition seam and deterministic fixture transport."]
#![doc = ""]
#![doc = "An adapter receives an exact caller-selected URL, enforces caller-provided"]
#![doc = "byte/time bounds, writes artifacts to a transaction-owned destination, and"]
#![doc = "returns typed `Success | NotFound | Failure` without release or fallback"]
#![doc = "policy. Downloaded bytes are data and are never executed."]

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// An exact acquisition request selected by caller policy.
///
/// The URL is used verbatim: the transport performs no discovery, version
/// selection, mirror choice, or fallback. Query and credential material is
/// redacted in all diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionRequest {
    url: String,
}

impl AcquisitionRequest {
    /// Creates a request for an exact URL after basic validation.
    ///
    /// Rejects empty URLs, control characters, and non-`http(s)` schemes.
    /// The URL is stored verbatim; redaction applies only to diagnostics.
    pub fn new(url: impl Into<String>) -> Result<Self, AcquisitionError> {
        let url = url.into();
        if url.is_empty() || url.chars().any(char::is_control) {
            return Err(AcquisitionError::invalid(
                "URL is empty or has control characters",
            ));
        }
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(AcquisitionError::invalid(
                "URL must use http:// or https://",
            ));
        }
        if url.len() > 8192 {
            return Err(AcquisitionError::invalid("URL exceeds bound"));
        }
        Ok(Self { url })
    }

    /// Returns the exact URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the redacted URL for diagnostics.
    pub fn redacted(&self) -> String {
        redact_url(&self.url)
    }
}

/// Caller-provided byte and time bounds for one fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FetchLimits {
    /// Maximum decoded metadata bytes held in memory.
    pub max_metadata_bytes: usize,
    /// Maximum artifact bytes written to disk. Every fetch is finitely bounded.
    pub max_artifact_bytes: u64,
    /// Deadline for establishing the connection (advisory for fixtures).
    pub connect_timeout: Duration,
    /// Total wall-clock deadline for the fetch.
    pub total_timeout: Duration,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            max_metadata_bytes: 256 * 1024,
            max_artifact_bytes: 128 * 1024 * 1024,
            connect_timeout: Duration::from_secs(10),
            total_timeout: Duration::from_secs(120),
        }
    }
}

impl FetchLimits {
    /// Validates caller-provided byte and time bounds.
    ///
    /// The fields remain public for 0.1.x source compatibility, so every
    /// transport validates limits again at its I/O boundary. Invalid direct
    /// struct literals are rejected before route, filesystem, or network I/O.
    pub fn validate(&self) -> Result<(), AcquisitionError> {
        if self.max_metadata_bytes == 0 || self.max_metadata_bytes > 16 * 1024 * 1024 {
            return Err(AcquisitionError::invalid("metadata bound out of range"));
        }
        if self.max_artifact_bytes == 0 {
            return Err(AcquisitionError::invalid("artifact bound must be positive"));
        }
        if self.total_timeout.is_zero() || self.connect_timeout.is_zero() {
            return Err(AcquisitionError::invalid("timeouts must be positive"));
        }
        if self.connect_timeout > self.total_timeout {
            return Err(AcquisitionError::invalid(
                "connect timeout must not exceed total timeout",
            ));
        }
        Ok(())
    }

    /// Creates limits with explicit bounds.
    ///
    /// Requires non-zero `connect_timeout` and `total_timeout` with
    /// `connect_timeout <= total_timeout`. The connect phase is part of the
    /// total wall-clock budget, so a connect deadline beyond the total
    /// deadline cannot be satisfied truthfully.
    pub fn new(
        max_metadata_bytes: usize,
        max_artifact_bytes: u64,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Result<Self, AcquisitionError> {
        let limits = Self {
            max_metadata_bytes,
            max_artifact_bytes,
            connect_timeout,
            total_timeout,
        };
        limits.validate()?;
        Ok(limits)
    }

    /// Derives the authoritative effective deadlines for one fetch.
    ///
    /// ```text
    /// effective connect timeout = min(request connect, adapter connect ceiling)
    /// effective total timeout   = min(request total, adapter total ceiling)
    /// ```
    ///
    /// A stricter adapter ceiling is allowed; an adapter may never silently
    /// extend a caller deadline. Both metadata and artifact operations use
    /// the same derivation, and the effective total deadline covers response
    /// headers plus body streaming.
    pub fn effective(
        &self,
        adapter_connect_ceiling: Duration,
        adapter_total_ceiling: Duration,
    ) -> (Duration, Duration) {
        let connect = std::cmp::min(self.connect_timeout, adapter_connect_ceiling);
        let total = std::cmp::min(self.total_timeout, adapter_total_ceiling);
        (connect, total)
    }
}

/// Cooperative cancellation flag checked during streaming.
///
/// Dropping a fetch without completing it also cleans any owned incomplete
/// output where safe (see `fetch_artifact`).
#[derive(Debug, Default)]
pub struct CancelFlag {
    cancelled: AtomicBool,
}

impl CancelFlag {
    /// Creates an uncancelled flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Typed success/absence outcome. Hard failures are `Err(AcquisitionError)`.
///
/// `NotFound` is data (the exact URL is absent); the consumer decides what it
/// means. It never triggers fallback inside a transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchOutcome<T> {
    /// The exact URL produced a usable response.
    Success(T),
    /// The exact URL is definitively absent (HTTP 404 equivalent).
    NotFound,
}

impl<T> FetchOutcome<T> {
    /// Returns the success value, if any.
    pub fn success(self) -> Option<T> {
        match self {
            Self::Success(v) => Some(v),
            Self::NotFound => None,
        }
    }

    /// Returns true for `Success`.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Returns true for `NotFound`.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

/// Bounded small-body metadata bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataBytes {
    bytes: Vec<u8>,
}

impl MetadataBytes {
    /// Returns the exact body bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the body length.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether the body is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// Streamed artifact evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactEvidence {
    /// Bytes written to the destination file.
    pub bytes_written: u64,
}

impl ArtifactEvidence {
    /// Returns bytes written.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

/// Hard acquisition failure. Never used for `NotFound`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AcquisitionError {
    /// Caller input violates the seam contract.
    InvalidInput(String),
    /// A transport-level failure (TLS, proxy, 5xx, malformed response, early
    /// disconnect). URL material is already redacted.
    Transport(String),
    /// The response exceeded a caller byte bound.
    TooLarge {
        /// The bound that was exceeded.
        limit: u64,
    },
    /// A connect or total deadline elapsed.
    Timeout {
        /// Which deadline elapsed.
        phase: &'static str,
    },
    /// Cooperative cancellation was observed.
    Cancelled,
    /// A transaction-owned filesystem operation failed.
    Io(String),
    /// The adapter could not attempt the request (missing executable, failed
    /// discovery, spawn failure). URL material is already redacted.
    ///
    /// Composition falls back to the next transport on `Unavailable`. Ordinary
    /// `Transport`/`Timeout` failures fall back only under an explicit
    /// opt-in policy. All other errors and `NotFound` are terminal.
    Unavailable(String),
}

impl AcquisitionError {
    pub(crate) fn invalid(detail: impl Into<String>) -> Self {
        Self::InvalidInput(bound_detail(detail.into()))
    }

    pub(crate) fn transport_redacted(detail: impl Into<String>) -> Self {
        Self::Transport(bound_detail(detail.into()))
    }

    /// Adapter-only constructor for unavailable evidence.
    ///
    /// `#[doc(hidden)]`: not part of the consumer contract; allows verified
    /// adapters (`eggup-curl`) to report missing/spawn failures without
    /// widening the public seam with a general constructor.
    #[doc(hidden)]
    pub fn __adapter_unavailable(detail: impl Into<String>) -> Self {
        Self::Unavailable(bound_detail(detail.into()))
    }

    pub(crate) fn io(operation: &'static str, source: std::io::Error) -> Self {
        Self::Io(bound_detail(format!("{operation}: {source}")))
    }

    /// Returns true when the adapter was unavailable to attempt the request.
    pub fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}

impl fmt::Display for AcquisitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(d) => write!(f, "invalid acquisition input: {d}"),
            Self::Transport(d) => write!(f, "acquisition transport failed: {d}"),
            Self::TooLarge { limit } => write!(f, "response exceeded bound ({limit} bytes)"),
            Self::Timeout { phase } => write!(f, "acquisition timed out ({phase})"),
            Self::Cancelled => write!(f, "acquisition cancelled"),
            Self::Io(d) => write!(f, "acquisition I/O failed: {d}"),
            Self::Unavailable(d) => write!(f, "acquisition transport unavailable: {d}"),
        }
    }
}

impl std::error::Error for AcquisitionError {}

fn bound_detail(s: String) -> String {
    truncate_utf8_bytes(s, 512)
}

fn truncate_utf8_bytes(mut s: String, max: usize) -> String {
    if s.len() > max {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
    s
}

/// Redacts credential-bearing URL material for diagnostics.
///
/// Strips `user:password@` credentials, query strings, and fragments, then
/// truncates overlong URLs. The exact URL is still used for the fetch; only
/// the diagnostic string is redacted.
///
/// Upstream transport/proxy error text is never embedded in diagnostics: all
/// transport failures use structured category text plus this redacted URL, so
/// credential-bearing upstream strings cannot leak through `Display`.
pub fn redact_url(url: &str) -> String {
    let mut out = url.to_string();
    if let Some(scheme_end) = out.find("://") {
        let after = scheme_end + 3;
        if let Some(at) = out[after..].find('@') {
            let at_abs = after + at;
            // Keep host/path, drop credentials.
            out = format!("{}://<redacted>@{}", &url[..scheme_end], &url[at_abs + 1..]);
        }
    }
    // Redact fragments first so they cannot smuggle tokens past query handling.
    if let Some(h) = out.find('#') {
        out.truncate(h);
        out.push_str("#<redacted>");
    }
    // Redact query strings, which commonly carry tokens.
    if let Some(q) = out.find('?') {
        out.truncate(q);
        out.push_str("?<redacted>");
    }
    if out.len() > 256 {
        let mut end = 256;
        while !out.is_char_boundary(end) {
            end -= 1;
        }
        out.truncate(end);
        out.push('…');
    }
    out
}

/// Scrubs credential-like URL patterns from arbitrary upstream text.
///
/// This is defense-in-depth only. Production adapters must not embed raw
/// upstream error display in diagnostics; they use structured category text
/// plus [`redact_url`]. This helper strips `://credentials@` and `?query`
/// patterns from any string that must be incorporated for debugging.
#[doc(hidden)]
pub fn __scrub_upstream_text(text: &str) -> String {
    // Cheap deterministic scrub: redact userinfo and query-looking suffixes.
    let mut out = text.to_string();
    // Redact `://...@` credential blocks with an advancing cursor so each
    // iteration makes progress and the loop always terminates.
    let mut search_from: usize = 0;
    while search_from < out.len() {
        let Some(rel_scheme) = out[search_from..].find("://") else {
            break;
        };
        let scheme = search_from + rel_scheme;
        let after = scheme + 3;
        let Some(rel_at) = out[after..].find('@') else {
            break;
        };
        let at_abs = after + rel_at;
        // If another `://` appears before the `@`, the current scheme has no
        // userinfo; skip past it instead of consuming a later URL's `@`.
        if out[after..at_abs].contains("://") {
            search_from = after;
            continue;
        }
        // Bound the credential block to avoid runaway on non-URL text.
        if at_abs - after > 512 {
            break;
        }
        out.replace_range(after..at_abs, "<redacted>");
        // Advance past the replacement and its trailing `@` so the same URL
        // is never reprocessed.
        search_from = after + "<redacted>".len() + 1;
    }
    // Redact query strings: truncate at first `?` and mark redacted, but
    // only when the `?` looks like URL query material (has `=` or `&` or
    // is followed by non-space token material). This avoids mangling
    // ordinary prose containing `?`.
    if let Some(q) = out.find('?') {
        let suffix = &out[q..];
        if suffix.contains('=') || suffix.contains('&') {
            out.truncate(q);
            out.push_str("?<redacted>");
        }
    }
    if out.len() > 512 {
        let mut end = 512;
        while !out.is_char_boundary(end) {
            end -= 1;
        }
        out.truncate(end);
    }
    out
}

/// Maximum exclusive-temp creation attempts before bounded failure.
#[doc(hidden)]
pub const __TEMP_COLLISION_BOUND: u32 = 32;

/// Creates an exclusively-owned temporary sibling in `parent`.
///
/// The file is created with `create_new` (fails if the candidate already
/// exists), retried with a fresh nonce on collision within
/// [`__TEMP_COLLISION_BOUND`], never follows a precreated symlink, lives in
/// the exact destination parent so promotion stays on the same filesystem,
/// and uses owner-private `0600` permissions on Unix. Returns the open file
/// plus its path; the caller owns cleanup of exactly this path.
///
/// `prefix` should identify the adapter (for example `eggup-acquire` or
/// `eggup-eggfetch`) and must not contain path separators.
#[doc(hidden)]
pub fn __acquire_exclusive_temp(
    parent: &Path,
    prefix: &str,
) -> Result<(std::fs::File, std::path::PathBuf), AcquisitionError> {
    use std::fs::OpenOptions;
    for _ in 0..__TEMP_COLLISION_BOUND {
        let nonce = NEXT_TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let candidate = parent.join(format!(
            ".{prefix}-{}-{nanos}-{nonce}.part",
            std::process::id()
        ));
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&candidate) {
            Ok(file) => {
                // Enforce owner-private permissions even under permissive umask.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = file
                        .metadata()
                        .map_err(|e| AcquisitionError::io("reading part permissions", e))?
                        .permissions();
                    if perms.mode() & 0o777 != 0o600 {
                        perms.set_mode(0o600);
                        std::fs::set_permissions(&candidate, perms)
                            .map_err(|e| AcquisitionError::io("securing part file", e))?;
                    }
                }
                return Ok((file, candidate));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(AcquisitionError::io("creating part file", e)),
        }
    }
    Err(AcquisitionError::io(
        "creating part file",
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "too many temporary name collisions",
        ),
    ))
}

/// Promotes an owned temporary file to `dest` without clobbering.
///
/// Uses a race-safe no-replace strategy (`hard_link` fails with
/// `AlreadyExists` when `dest` exists, without overwriting). The temp and
/// dest must share a parent directory (same filesystem). On success the temp
/// link cleanup is best-effort after commit; failure may leave the owned temp
/// link as residue but never turns a committed destination into ordinary
/// failure. If `dest` already
/// exists — before the call or raced in during the fetch — returns an
/// explicit `InvalidInput` error and preserves the foreign destination;
/// the caller must remove its owned temp separately.
///
/// This avoids platform-specific `rename`-overwrite semantics: Unix
/// `rename` would silently replace `dest`, while Windows `rename` fails.
/// `hard_link` fails closed on both when `dest` exists.
#[doc(hidden)]
pub fn __promote_no_clobber(tmp: &Path, dest: &Path) -> Result<(), AcquisitionError> {
    __promote_no_clobber_with(tmp, dest, |path| fs::remove_file(path))
}

fn __promote_no_clobber_with(
    tmp: &Path,
    dest: &Path,
    remove_temp: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<(), AcquisitionError> {
    match std::fs::hard_link(tmp, dest) {
        Ok(()) => {
            // The destination is committed. Cleanup of the redundant owned
            // name cannot retroactively make acquisition fail.
            let _ = remove_temp(tmp);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(AcquisitionError::invalid(
            "artifact destination already exists; refusing to overwrite",
        )),
        Err(e) => Err(AcquisitionError::io("promoting artifact", e)),
    }
}

/// Best-effort removal of exactly one owned temporary file.
///
/// Uses `remove_file` (never follows symlinks to targets, never recurses)
/// and ignores failures; used in `Drop` guards where no error can be
/// returned. Never performs prefix-based cleanup of unrelated siblings.
#[doc(hidden)]
pub fn __remove_owned_temp(path: &Path) {
    let _ = std::fs::remove_file(path);
}

/// The minimal transport-neutral acquisition contract.
///
/// Implementations receive an exact URL, enforce caller bounds, and stream
/// artifacts to files. They never select releases, versions, mirrors, or
/// fallback sources, and never execute downloaded content.
///
/// Effective time bounds obey `min(request, adapter ceiling)` per
/// [`FetchLimits::effective`]: a stricter adapter may tighten a deadline but
/// never extend it. Artifact promotion requires `dest` to be absent; an
/// existing or raced-in destination fails without overwrite.
pub trait AcquisitionTransport {
    /// Fetches a bounded small body (release metadata, checksums) into memory.
    fn fetch_metadata(
        &self,
        request: &AcquisitionRequest,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError>;

    /// Streams an artifact body to `dest` without requiring full buffering.
    ///
    /// The file at `dest` is created atomically: bytes stream to an
    /// exclusively-created, owner-private temporary sibling and are promoted
    /// with no-clobber semantics only on full success. `dest` must be absent
    /// at promotion time; an existing or raced-in destination fails with an
    /// explicit error and is never overwritten. Partial outputs clean only
    /// the owned temp on failure or cancellation where safe. `dest`'s parent
    /// must already exist as a real directory.
    ///
    /// Effective time bounds obey `min(request, adapter ceiling)`; see
    /// [`FetchLimits::effective`].
    fn fetch_artifact(
        &self,
        request: &AcquisitionRequest,
        dest: &Path,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError>;
}

static NEXT_TEMP_NONCE: AtomicU64 = AtomicU64::new(0);

/// One deterministic fixture response.
#[derive(Debug, Clone)]
pub struct FixtureResponse {
    kind: FixtureKind,
}

#[derive(Debug, Clone)]
enum FixtureKind {
    Body(Vec<u8>),
    NotFound,
    Failure,
    Unavailable,
    Truncated { prefix_len: usize },
    Slow { body: Vec<u8>, delay: Duration },
}

impl FixtureResponse {
    /// A successful body.
    pub fn body(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            kind: FixtureKind::Body(bytes.into()),
        }
    }

    /// A definitive absence (HTTP 404 equivalent).
    pub fn not_found() -> Self {
        Self {
            kind: FixtureKind::NotFound,
        }
    }

    /// A hard transport failure (TLS/5xx/malformed equivalent).
    ///
    /// The detail is accepted for call-site compatibility but never exposed
    /// in diagnostics, mirroring production category-only errors so
    /// credential-bearing strings cannot leak.
    pub fn failure(_detail: impl Into<String>) -> Self {
        Self {
            kind: FixtureKind::Failure,
        }
    }

    /// A body truncated after `prefix_len` bytes (early disconnect).
    pub fn truncated(prefix_len: usize) -> Self {
        Self {
            kind: FixtureKind::Truncated { prefix_len },
        }
    }

    /// A body delivered only after `delay` (timeout testing).
    pub fn slow(body: Vec<u8>, delay: Duration) -> Self {
        Self {
            kind: FixtureKind::Slow { body, delay },
        }
    }

    /// An adapter-unavailable condition (missing executable/spawn failure
    /// equivalent) for composition fallback testing.
    pub fn unavailable() -> Self {
        Self {
            kind: FixtureKind::Unavailable,
        }
    }
}

/// Deterministic in-memory transport for correctness tests.
///
/// Real network policy lives in `eggup-eggfetch`. This transport never touches
/// the network; every URL must be registered explicitly, and unregistered URLs
/// are hard failures (never silent fallback).
#[derive(Debug, Default)]
pub struct FixtureTransport {
    routes: Mutex<HashMap<String, FixtureResponse>>,
}

impl FixtureTransport {
    /// Creates an empty fixture transport.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one exact URL response.
    pub fn route(&self, url: &str, response: FixtureResponse) {
        self.routes
            .lock()
            .expect("fixture lock")
            .insert(url.to_string(), response);
    }

    fn lookup(&self, url: &str) -> Result<FixtureResponse, AcquisitionError> {
        self.routes
            .lock()
            .expect("fixture lock")
            .get(url)
            .cloned()
            .ok_or_else(|| {
                AcquisitionError::transport_redacted(format!(
                    "no fixture route for {}",
                    redact_url(url)
                ))
            })
    }
}

impl AcquisitionTransport for FixtureTransport {
    fn fetch_metadata(
        &self,
        request: &AcquisitionRequest,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError> {
        limits.validate()?;
        let start = Instant::now();
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        let fixture = self.lookup(request.url())?;
        match fixture.kind {
            FixtureKind::NotFound => Ok(FetchOutcome::NotFound),
            // Never echo fixture failure detail: it may contain test
            // secrets and production adapters use category-only errors.
            FixtureKind::Failure => Err(AcquisitionError::transport_redacted(
                "fixture transport failure",
            )),
            FixtureKind::Unavailable => Err(AcquisitionError::__adapter_unavailable(
                "fixture transport unavailable",
            )),
            FixtureKind::Body(bytes) => {
                if start.elapsed() > limits.total_timeout {
                    return Err(AcquisitionError::Timeout { phase: "total" });
                }
                if cancel.is_cancelled() {
                    return Err(AcquisitionError::Cancelled);
                }
                if bytes.len() > limits.max_metadata_bytes {
                    return Err(AcquisitionError::TooLarge {
                        limit: limits.max_metadata_bytes as u64,
                    });
                }
                Ok(FetchOutcome::Success(MetadataBytes { bytes }))
            }
            FixtureKind::Truncated { .. } => Err(AcquisitionError::transport_redacted(
                "truncated fixture body",
            )),
            FixtureKind::Slow { body, delay } => {
                if delay > limits.total_timeout {
                    return Err(AcquisitionError::Timeout { phase: "total" });
                }
                if cancel.is_cancelled() {
                    return Err(AcquisitionError::Cancelled);
                }
                if body.len() > limits.max_metadata_bytes {
                    return Err(AcquisitionError::TooLarge {
                        limit: limits.max_metadata_bytes as u64,
                    });
                }
                Ok(FetchOutcome::Success(MetadataBytes { bytes: body }))
            }
        }
    }

    fn fetch_artifact(
        &self,
        request: &AcquisitionRequest,
        dest: &Path,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError> {
        limits.validate()?;
        let start = Instant::now();
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        let parent = dest
            .parent()
            .ok_or_else(|| AcquisitionError::invalid("artifact destination has no parent"))?;
        let parent_meta = fs::symlink_metadata(parent)
            .map_err(|e| AcquisitionError::io("reading artifact parent", e))?;
        if !parent_meta.is_dir() || parent_meta.file_type().is_symlink() {
            return Err(AcquisitionError::invalid(
                "artifact parent must be an existing real directory",
            ));
        }
        // Fast-fail when the destination already exists. The race-safe
        // guarantee comes from `__promote_no_clobber` below; this pre-check
        // avoids wasted work for the common case.
        if fs::symlink_metadata(dest).is_ok() {
            return Err(AcquisitionError::invalid(
                "artifact destination already exists; refusing to overwrite",
            ));
        }
        let fixture = self.lookup(request.url())?;
        let body = match fixture.kind {
            FixtureKind::NotFound => return Ok(FetchOutcome::NotFound),
            FixtureKind::Failure => {
                return Err(AcquisitionError::transport_redacted(
                    "fixture transport failure",
                ));
            }
            FixtureKind::Unavailable => {
                return Err(AcquisitionError::__adapter_unavailable(
                    "fixture transport unavailable",
                ));
            }
            FixtureKind::Body(b) => b,
            FixtureKind::Truncated { prefix_len } => {
                // Simulate a partial write then a hard failure; no file is promoted.
                // Use exclusive creation so collision/symlink behavior matches
                // the success path, then clean only the owned temp.
                if let Ok((mut f, tmp)) = __acquire_exclusive_temp(parent, "eggup-acquire") {
                    use std::io::Write;
                    let prefix = vec![0u8; prefix_len.min(1024)];
                    let _ = f.write_all(&prefix);
                    drop(f);
                    __remove_owned_temp(&tmp);
                }
                return Err(AcquisitionError::transport_redacted(
                    "early disconnect during artifact body",
                ));
            }
            FixtureKind::Slow { body, delay } => {
                if delay > limits.total_timeout || start.elapsed() + delay > limits.total_timeout {
                    return Err(AcquisitionError::Timeout { phase: "total" });
                }
                body
            }
        };
        if body.len() as u64 > limits.max_artifact_bytes {
            return Err(AcquisitionError::TooLarge {
                limit: limits.max_artifact_bytes,
            });
        }
        if start.elapsed() > limits.total_timeout {
            return Err(AcquisitionError::Timeout { phase: "total" });
        }
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        // Exclusive owner-private temp plus race-safe no-clobber promotion.
        let (mut file, tmp) = __acquire_exclusive_temp(parent, "eggup-acquire")?;
        struct Guard {
            path: std::path::PathBuf,
            disarm: bool,
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                if !self.disarm {
                    __remove_owned_temp(&self.path);
                }
            }
        }
        let mut guard = Guard {
            path: tmp.clone(),
            disarm: false,
        };
        // Chunked write so cancellation and partial-write failures are observable.
        let mut written: u64 = 0;
        {
            use std::io::Write;
            for chunk in body.chunks(8192) {
                if cancel.is_cancelled() {
                    return Err(AcquisitionError::Cancelled);
                }
                file.write_all(chunk)
                    .map_err(|e| AcquisitionError::io("writing part file", e))?;
                written += chunk.len() as u64;
            }
            file.flush()
                .map_err(|e| AcquisitionError::io("flushing part file", e))?;
            drop(file);
        }
        // Re-check cancellation before promotion: never promote after cancel.
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        match __promote_no_clobber(&tmp, dest) {
            Ok(()) => {
                guard.disarm = true;
                Ok(FetchOutcome::Success(ArtifactEvidence {
                    bytes_written: written,
                }))
            }
            Err(e) => {
                // Preserve the foreign destination; remove only the owned temp.
                __remove_owned_temp(&tmp);
                guard.disarm = true;
                Err(e)
            }
        }
    }
}

/// Shared ownership helper for tests that need a transport behind `Arc`.
pub type SharedTransport = Arc<FixtureTransport>;

/// Adapter-only constructor for metadata bytes.
///
/// `#[doc(hidden)]`: not part of the consumer contract; allows verified
/// adapters (`eggup-eggfetch`) to return seam types without widening the
/// public seam with a general constructor.
#[doc(hidden)]
pub fn __adapter_metadata(bytes: Vec<u8>) -> MetadataBytes {
    MetadataBytes { bytes }
}

/// Adapter-only constructor for artifact evidence.
///
/// `#[doc(hidden)]`: see `__adapter_metadata`.
#[doc(hidden)]
pub fn __adapter_artifact(bytes_written: u64) -> ArtifactEvidence {
    ArtifactEvidence { bytes_written }
}

/// Caller-selected fallback scope for a composed transport.
///
/// Transport fallback (`curl <-> Eggfetch` for the same exact URL) is never
/// release/source fallback. `NotFound` is terminal for the exact URL under
/// every policy. `InvalidInput`, `Cancelled`, `TooLarge`, and `Io`
/// (staging/promotion) failures are terminal under every policy.
/// Verification/candidate failures occur above this layer and are never
/// transport-fallback inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositionPolicy {
    /// Fall back only when the preferred adapter is unavailable
    /// (`AcquisitionError::Unavailable`). This is the default.
    #[default]
    UnavailableOnly,
    /// Fall back on unavailability plus ordinary transport failures
    /// (`Transport` and `Timeout`). Terminal conditions above remain terminal.
    UnavailableOrTransport,
}

/// Explicit caller-selected preferred/fallback composition over two transports.
///
/// The composition itself implements [`AcquisitionTransport`] so callers keep
/// a single seam. Preferred order is the construction order: `primary` is
/// attempted first, `secondary` only when the policy allows fallback for the
/// primary outcome.
///
/// Fallback is decided from typed outcomes, never by parsing human-readable
/// errors.
pub struct ComposedTransport<'a> {
    primary: &'a dyn AcquisitionTransport,
    secondary: &'a dyn AcquisitionTransport,
    policy: CompositionPolicy,
}

impl std::fmt::Debug for ComposedTransport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposedTransport")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl<'a> ComposedTransport<'a> {
    /// Creates a composed transport with an explicit fallback policy.
    ///
    /// No network, filesystem, or process work occurs here; policy and
    /// ordering are the only decisions recorded.
    pub fn new(
        primary: &'a dyn AcquisitionTransport,
        secondary: &'a dyn AcquisitionTransport,
        policy: CompositionPolicy,
    ) -> Self {
        Self {
            primary,
            secondary,
            policy,
        }
    }

    /// Returns the fallback policy.
    pub fn policy(&self) -> CompositionPolicy {
        self.policy
    }

    fn should_fallback(&self, err: &AcquisitionError) -> bool {
        match err {
            AcquisitionError::Unavailable(_) => true,
            AcquisitionError::Transport(_) | AcquisitionError::Timeout { .. } => {
                self.policy == CompositionPolicy::UnavailableOrTransport
            }
            _ => false,
        }
    }
}

impl AcquisitionTransport for ComposedTransport<'_> {
    fn fetch_metadata(
        &self,
        request: &AcquisitionRequest,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError> {
        // Validate once at the composition boundary so invalid caller limits
        // fail before either adapter performs route/filesystem/network I/O.
        // Adapters revalidate at their own boundaries per the seam contract.
        limits.validate()?;
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        match self.primary.fetch_metadata(request, limits, cancel) {
            Ok(outcome) => Ok(outcome),
            Err(err) => {
                if self.should_fallback(&err) {
                    self.secondary.fetch_metadata(request, limits, cancel)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn fetch_artifact(
        &self,
        request: &AcquisitionRequest,
        dest: &Path,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError> {
        limits.validate()?;
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        match self.primary.fetch_artifact(request, dest, limits, cancel) {
            Ok(outcome) => Ok(outcome),
            Err(err) => {
                if self.should_fallback(&err) {
                    self.secondary.fetch_artifact(request, dest, limits, cancel)
                } else {
                    Err(err)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    fn req(url: &str) -> AcquisitionRequest {
        AcquisitionRequest::new(url).unwrap()
    }

    fn limits() -> FetchLimits {
        FetchLimits {
            max_metadata_bytes: 1024,
            max_artifact_bytes: 4096,
            connect_timeout: Duration::from_secs(1),
            total_timeout: Duration::from_secs(5),
        }
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let p = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        let _ = fs::create_dir_all(&p);
        p
    }

    #[test]
    fn exact_body_success() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/meta",
            FixtureResponse::body(b"hello".to_vec()),
        );
        let out = t
            .fetch_metadata(
                &req("https://example.com/meta"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"hello");
    }

    #[test]
    fn streamed_artifact_success() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/app",
            FixtureResponse::body(vec![7u8; 2048]),
        );
        let dir = temp_dir("acq-stream");
        let dest = dir.join("app");
        let out = t
            .fetch_artifact(
                &req("https://example.com/app"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        let ev = out.success().unwrap();
        assert_eq!(ev.bytes_written, 2048);
        assert_eq!(fs::read(&dest).unwrap().len(), 2048);
        assert!(!dir.join(".eggup-acquire-0.part").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_limit_is_enforced() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/big",
            FixtureResponse::body(vec![0u8; 2048]),
        );
        let err = t
            .fetch_metadata(
                &req("https://example.com/big"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::TooLarge { .. }));
    }

    #[test]
    fn artifact_limit_is_enforced_without_promotion() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/big",
            FixtureResponse::body(vec![0u8; 8192]),
        );
        let dir = temp_dir("acq-limit");
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(
                &req("https://example.com/big"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::TooLarge { .. }));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn not_found_is_data_not_failure() {
        let t = FixtureTransport::new();
        t.route("https://example.com/missing", FixtureResponse::not_found());
        let out = t
            .fetch_metadata(
                &req("https://example.com/missing"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
        let dir = temp_dir("acq-404");
        let dest = dir.join("app");
        let out = t
            .fetch_artifact(
                &req("https://example.com/missing"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hard_failures_never_become_not_found() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/boom",
            FixtureResponse::failure("500 internal error"),
        );
        let err = t
            .fetch_metadata(
                &req("https://example.com/boom"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
    }

    #[test]
    fn timeout_is_a_hard_failure() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/slow",
            FixtureResponse::slow(b"late".to_vec(), Duration::from_secs(30)),
        );
        let err = t
            .fetch_metadata(
                &req("https://example.com/slow"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Timeout { .. }));
    }

    #[test]
    fn cancellation_cleans_partial_output() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/app",
            FixtureResponse::body(vec![1u8; 1024]),
        );
        let dir = temp_dir("acq-cancel");
        let dest = dir.join("app");
        let cancel = CancelFlag::new();
        cancel.cancel();
        let err = t
            .fetch_artifact(&req("https://example.com/app"), &dest, limits(), &cancel)
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Cancelled));
        assert!(!dest.exists());
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(".eggup-acquire-")
            })
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn partial_write_failure_promotes_nothing() {
        let t = FixtureTransport::new();
        t.route("https://example.com/cut", FixtureResponse::truncated(512));
        let dir = temp_dir("acq-partial");
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(
                &req("https://example.com/cut"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diagnostics_are_redacted() {
        let redacted = redact_url("https://user:s3cret@example.com/app?token=abc");
        assert!(!redacted.contains("s3cret"));
        assert!(!redacted.contains("token=abc"));
        assert!(redacted.contains("example.com"));
        let err = AcquisitionRequest::new("https://user:pw@example.com/x?k=v").unwrap();
        assert!(!err.redacted().contains("pw"));
    }

    #[test]
    fn bounded_diagnostics_preserve_utf8_at_256_and_512_byte_edges() {
        for limit in [256, 512] {
            for (width, ch) in [(2, "é"), (3, "€"), (4, "🦀")] {
                let prefix = "a".repeat(limit - 1);
                let bounded = truncate_utf8_bytes(format!("{prefix}{ch}tail"), limit);
                assert!(bounded.len() <= limit);
                assert!(std::str::from_utf8(bounded.as_bytes()).is_ok());
                assert!(bounded.ends_with(&prefix));
                assert!(width > 1);
            }
        }
    }

    #[test]
    fn zero_artifact_limit_is_rejected() {
        let invalid = FetchLimits {
            max_artifact_bytes: 0,
            ..FetchLimits::default()
        };
        assert!(matches!(
            invalid.validate(),
            Err(AcquisitionError::InvalidInput(_))
        ));
    }

    #[test]
    fn no_fallback_on_unregistered_url() {
        let t = FixtureTransport::new();
        let err = t
            .fetch_metadata(
                &req("https://example.com/unknown"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
    }

    #[test]
    fn invalid_public_limits_are_rejected_before_fixture_io() {
        let transport = FixtureTransport::new();
        let request = req("https://example.com/no-route");
        let invalid = [
            FetchLimits {
                max_metadata_bytes: 0,
                ..limits()
            },
            FetchLimits {
                max_metadata_bytes: 16 * 1024 * 1024 + 1,
                ..limits()
            },
            FetchLimits {
                connect_timeout: Duration::ZERO,
                ..limits()
            },
            FetchLimits {
                total_timeout: Duration::ZERO,
                ..limits()
            },
            FetchLimits {
                connect_timeout: Duration::from_secs(6),
                total_timeout: Duration::from_secs(5),
                ..limits()
            },
        ];
        let dir = temp_dir("acq-invalid-limits");
        let dest = dir.join("app");
        for limits in invalid {
            assert!(matches!(
                limits.validate(),
                Err(AcquisitionError::InvalidInput(_))
            ));
            assert!(matches!(
                transport.fetch_metadata(&request, limits, &CancelFlag::new()),
                Err(AcquisitionError::InvalidInput(_))
            ));
            assert!(matches!(
                transport.fetch_artifact(&request, &dest, limits, &CancelFlag::new()),
                Err(AcquisitionError::InvalidInput(_))
            ));
            assert!(!dest.exists());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        }
        assert!(limits().validate().is_ok());
        assert!(
            FetchLimits::new(1024, 4096, Duration::from_secs(1), Duration::from_secs(5)).is_ok()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_artifact_parent_fails_without_creation() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/app",
            FixtureResponse::body(b"x".to_vec()),
        );
        let dir = temp_dir("acq-noparent");
        let dest = dir.join("no-such-dir/app");
        let err = t
            .fetch_artifact(
                &req("https://example.com/app"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::InvalidInput(_) | AcquisitionError::Io(_)
        ));
        assert!(!dir.join("no-such-dir").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- M003 corrective: timeout contract ----

    #[test]
    fn connect_exceeding_total_is_rejected() {
        let err = FetchLimits::new(1024, 4096, Duration::from_secs(5), Duration::from_secs(1))
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
    }

    #[test]
    fn zero_timeouts_are_rejected() {
        assert!(FetchLimits::new(1024, 1, Duration::ZERO, Duration::from_secs(1)).is_err());
        assert!(FetchLimits::new(1024, 1, Duration::from_secs(1), Duration::ZERO).is_err());
    }

    #[test]
    fn effective_timeouts_take_minimums() {
        let request = FetchLimits {
            max_metadata_bytes: 1024,
            max_artifact_bytes: 4096,
            connect_timeout: Duration::from_millis(50),
            total_timeout: Duration::from_millis(50),
        };
        // Request stricter than adapter: request wins.
        let (c, t) = request.effective(Duration::from_secs(5), Duration::from_secs(5));
        assert_eq!(c, Duration::from_millis(50));
        assert_eq!(t, Duration::from_millis(50));
        // Adapter stricter than request: adapter wins.
        let request2 = FetchLimits {
            max_metadata_bytes: 1024,
            max_artifact_bytes: 4096,
            connect_timeout: Duration::from_secs(5),
            total_timeout: Duration::from_secs(5),
        };
        let (c2, t2) = request2.effective(Duration::from_millis(50), Duration::from_millis(50));
        assert_eq!(c2, Duration::from_millis(50));
        assert_eq!(t2, Duration::from_millis(50));
        // Mixed: each phase takes its own minimum.
        let (c3, t3) = request2.effective(Duration::from_secs(1), Duration::from_secs(10));
        assert_eq!(c3, Duration::from_secs(1));
        assert_eq!(t3, Duration::from_secs(5));
    }

    #[test]
    fn timeout_never_maps_to_not_found() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/slow-art",
            FixtureResponse::slow(vec![1u8; 16], Duration::from_secs(30)),
        );
        let dir = temp_dir("acq-timeout-nf");
        let dest = dir.join("app");
        let tiny = FetchLimits {
            max_metadata_bytes: 1024,
            max_artifact_bytes: 4096,
            connect_timeout: Duration::from_millis(10),
            total_timeout: Duration::from_millis(10),
        };
        let err = t
            .fetch_artifact(
                &req("https://example.com/slow-art"),
                &dest,
                tiny,
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Timeout { .. }));
        assert!(!matches!(err, AcquisitionError::InvalidInput(_)));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- M003 corrective: exclusive temp + no-clobber ----

    #[test]
    fn existing_destination_is_a_hard_no_clobber_failure() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/app",
            FixtureResponse::body(vec![9u8; 64]),
        );
        let dir = temp_dir("acq-noclobber");
        let dest = dir.join("app");
        fs::write(&dest, b"FOREIGN-BYTES").unwrap();
        let err = t
            .fetch_artifact(
                &req("https://example.com/app"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::InvalidInput(_) | AcquisitionError::Io(_)
        ));
        // Foreign destination bytes are preserved verbatim.
        assert_eq!(fs::read(&dest).unwrap(), b"FOREIGN-BYTES");
        // No owned temp residue.
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.ends_with(".part") && n.starts_with(".eggup-acquire-")
            })
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn post_link_cleanup_failure_keeps_committed_destination_successful() {
        let dir = temp_dir("acq-promote-cleanup");
        let tmp = dir.join("owned.part");
        let dest = dir.join("app");
        fs::write(&tmp, b"complete artifact").unwrap();
        let result = __promote_no_clobber_with(&tmp, &dest, |_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected",
            ))
        });
        assert!(result.is_ok());
        assert_eq!(fs::read(&dest).unwrap(), b"complete artifact");
        assert_eq!(fs::read(&tmp).unwrap(), b"complete artifact");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn promote_no_clobber_preserves_raced_destination() {
        // Direct helper test for the race window: dest appears after the
        // pre-check but before promotion.
        let dir = temp_dir("acq-race");
        let (mut f, tmp) = __acquire_exclusive_temp(&dir, "eggup-acquire").unwrap();
        use std::io::Write;
        f.write_all(b"race-payload").unwrap();
        let dest = dir.join("app");
        fs::write(&dest, b"RACED-FOREIGN").unwrap();
        let err = __promote_no_clobber(&tmp, &dest).unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::InvalidInput(_) | AcquisitionError::Io(_)
        ));
        assert_eq!(fs::read(&dest).unwrap(), b"RACED-FOREIGN");
        // Owned temp still exists for caller cleanup; remove it explicitly.
        assert!(tmp.exists());
        __remove_owned_temp(&tmp);
        assert!(!tmp.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn promote_succeeds_when_destination_absent() {
        let dir = temp_dir("acq-promote-ok");
        let (mut f, tmp) = __acquire_exclusive_temp(&dir, "eggup-acquire").unwrap();
        use std::io::Write;
        f.write_all(b"payload").unwrap();
        drop(f);
        let dest = dir.join("app");
        __promote_no_clobber(&tmp, &dest).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"payload");
        assert!(!tmp.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn exclusive_temp_is_owner_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("acq-mode");
        let (f, tmp) = __acquire_exclusive_temp(&dir, "eggup-acquire").unwrap();
        let mode = f.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "temp must be owner-private 0600");
        drop(f);
        __remove_owned_temp(&tmp);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn exclusive_temp_never_truncates_existing_file() {
        use std::fs::OpenOptions;
        let dir = temp_dir("acq-notrunc");
        let victim = dir.join("victim");
        fs::write(&victim, b"SENTINEL").unwrap();
        // The creation primitive itself must fail on an existing path.
        let err = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&victim)
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&victim).unwrap(), b"SENTINEL");
        // Helper creates a distinct sibling and leaves the victim alone.
        let (f, tmp) = __acquire_exclusive_temp(&dir, "eggup-acquire").unwrap();
        assert_ne!(tmp, victim);
        assert_eq!(fs::read(&victim).unwrap(), b"SENTINEL");
        drop(f);
        __remove_owned_temp(&tmp);
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn exclusive_temp_does_not_follow_symlink() {
        use std::fs::OpenOptions;
        use std::os::unix::fs::symlink;
        let dir = temp_dir("acq-symlink");
        let target = dir.join("target");
        fs::write(&target, b"TARGET-SENTINEL").unwrap();
        let link = dir.join("link");
        symlink(&target, &link).unwrap();
        // create_new on the symlink path itself must fail without touching target.
        let err = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&link)
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&target).unwrap(), b"TARGET-SENTINEL");
        // A fixture fetch must not modify the symlink target either.
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/app",
            FixtureResponse::body(vec![1u8; 32]),
        );
        let dest = dir.join("app");
        t.fetch_artifact(
            &req("https://example.com/app"),
            &dest,
            limits(),
            &CancelFlag::new(),
        )
        .unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"TARGET-SENTINEL");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cancellation_preserves_foreign_sibling_and_removes_only_owned() {
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/app",
            FixtureResponse::body(vec![2u8; 128]),
        );
        let dir = temp_dir("acq-cancel-own");
        let foreign = dir.join("foreign-keep");
        fs::write(&foreign, b"KEEP").unwrap();
        let dest = dir.join("app");
        let cancel = CancelFlag::new();
        cancel.cancel();
        let err = t
            .fetch_artifact(&req("https://example.com/app"), &dest, limits(), &cancel)
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Cancelled));
        assert!(!dest.exists());
        assert_eq!(fs::read(&foreign).unwrap(), b"KEEP");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.ends_with(".part")
            })
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn early_disconnect_removes_only_owned_temp() {
        let t = FixtureTransport::new();
        t.route("https://example.com/cut", FixtureResponse::truncated(256));
        let dir = temp_dir("acq-disconnect-own");
        let foreign = dir.join("keep");
        fs::write(&foreign, b"KEEP").unwrap();
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(
                &req("https://example.com/cut"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert!(!dest.exists());
        assert_eq!(fs::read(&foreign).unwrap(), b"KEEP");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.ends_with(".part")
            })
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- M003 corrective: redaction ----

    #[test]
    fn redaction_sentinels_are_absent_from_diagnostics() {
        let userinfo_sentinel = "S3CR3T-USERINFO-9917";
        let query_sentinel = "TOKEN-QUERY-5523";
        let url =
            format!("https://user:{userinfo_sentinel}@example.com/app?token={query_sentinel}");
        // Unregistered URL error embeds only the redacted URL.
        let t = FixtureTransport::new();
        let err = t
            .fetch_metadata(&req(&url), limits(), &CancelFlag::new())
            .unwrap_err();
        let msg = format!("{err}");
        assert!(!msg.contains(userinfo_sentinel));
        assert!(!msg.contains(query_sentinel));
        assert!(!msg.contains("token="));
        // Fixture failure detail is never echoed.
        let t2 = FixtureTransport::new();
        t2.route(
            "https://example.com/secret",
            FixtureResponse::failure(format!(
                "upstream says {userinfo_sentinel} {query_sentinel}"
            )),
        );
        let err2 = t2
            .fetch_metadata(
                &req("https://example.com/secret"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        let msg2 = format!("{err2}");
        assert!(!msg2.contains(userinfo_sentinel));
        assert!(!msg2.contains(query_sentinel));
        // Fragment is also redacted.
        let red = redact_url("https://example.com/app#FRAG-SENTINEL-7788");
        assert!(!red.contains("FRAG-SENTINEL-7788"));
    }

    #[test]
    fn upstream_scrub_removes_credential_patterns() {
        let scrubbed = __scrub_upstream_text(
            "fetch failed for https://alice:HUNTER2-PROXY-3311@example.com/x?token=abc&v=1",
        );
        assert!(!scrubbed.contains("HUNTER2-PROXY-3311"));
        assert!(!scrubbed.contains("alice"));
        // Category-only errors never contain the sentinel at all.
        let t = FixtureTransport::new();
        t.route(
            "https://example.com/p",
            FixtureResponse::failure("proxy password HUNTER2-PROXY-3311 in detail"),
        );
        let err = t
            .fetch_metadata(&req("https://example.com/p"), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(!format!("{err}").contains("HUNTER2-PROXY-3311"));
    }

    // ---- M005: explicit transport composition ----

    struct CountingTransport {
        inner: FixtureTransport,
        metadata_calls: std::sync::atomic::AtomicUsize,
        artifact_calls: std::sync::atomic::AtomicUsize,
    }

    impl CountingTransport {
        fn new() -> Self {
            Self {
                inner: FixtureTransport::new(),
                metadata_calls: std::sync::atomic::AtomicUsize::new(0),
                artifact_calls: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn route(&self, url: &str, response: FixtureResponse) {
            self.inner.route(url, response);
        }

        fn metadata_calls(&self) -> usize {
            self.metadata_calls
                .load(std::sync::atomic::Ordering::SeqCst)
        }

        fn artifact_calls(&self) -> usize {
            self.artifact_calls
                .load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl AcquisitionTransport for CountingTransport {
        fn fetch_metadata(
            &self,
            request: &AcquisitionRequest,
            limits: FetchLimits,
            cancel: &CancelFlag,
        ) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError> {
            self.metadata_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.fetch_metadata(request, limits, cancel)
        }

        fn fetch_artifact(
            &self,
            request: &AcquisitionRequest,
            dest: &Path,
            limits: FetchLimits,
            cancel: &CancelFlag,
        ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError> {
            self.artifact_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.fetch_artifact(request, dest, limits, cancel)
        }
    }

    #[test]
    fn composition_prefers_primary_without_fallback_call() {
        let primary = CountingTransport::new();
        let secondary = CountingTransport::new();
        primary.route(
            "https://example.com/meta",
            FixtureResponse::body(b"primary".to_vec()),
        );
        secondary.route(
            "https://example.com/meta",
            FixtureResponse::body(b"secondary".to_vec()),
        );
        let composed =
            ComposedTransport::new(&primary, &secondary, CompositionPolicy::UnavailableOnly);
        let out = composed
            .fetch_metadata(
                &req("https://example.com/meta"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"primary");
        assert_eq!(primary.metadata_calls(), 1);
        assert_eq!(secondary.metadata_calls(), 0);
    }

    #[test]
    fn composition_falls_back_on_unavailable_by_default() {
        let primary = CountingTransport::new();
        let secondary = CountingTransport::new();
        primary.route("https://example.com/meta", FixtureResponse::unavailable());
        secondary.route(
            "https://example.com/meta",
            FixtureResponse::body(b"fallback".to_vec()),
        );
        let composed =
            ComposedTransport::new(&primary, &secondary, CompositionPolicy::UnavailableOnly);
        let out = composed
            .fetch_metadata(
                &req("https://example.com/meta"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"fallback");
        assert_eq!(primary.metadata_calls(), 1);
        assert_eq!(secondary.metadata_calls(), 1);
    }

    #[test]
    fn composition_default_does_not_retry_transport_failures() {
        let primary = CountingTransport::new();
        let secondary = CountingTransport::new();
        primary.route("https://example.com/meta", FixtureResponse::failure("boom"));
        secondary.route(
            "https://example.com/meta",
            FixtureResponse::body(b"secondary".to_vec()),
        );
        let composed =
            ComposedTransport::new(&primary, &secondary, CompositionPolicy::UnavailableOnly);
        let err = composed
            .fetch_metadata(
                &req("https://example.com/meta"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert_eq!(primary.metadata_calls(), 1);
        assert_eq!(secondary.metadata_calls(), 0);
    }

    #[test]
    fn composition_broad_policy_retries_transport_and_timeout() {
        let primary = CountingTransport::new();
        let secondary = CountingTransport::new();
        primary.route("https://example.com/meta", FixtureResponse::failure("boom"));
        secondary.route(
            "https://example.com/meta",
            FixtureResponse::body(b"recovered".to_vec()),
        );
        let composed = ComposedTransport::new(
            &primary,
            &secondary,
            CompositionPolicy::UnavailableOrTransport,
        );
        let out = composed
            .fetch_metadata(
                &req("https://example.com/meta"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"recovered");
        assert_eq!(secondary.metadata_calls(), 1);

        let p2 = CountingTransport::new();
        let s2 = CountingTransport::new();
        p2.route(
            "https://example.com/slow",
            FixtureResponse::slow(b"x".to_vec(), Duration::from_secs(30)),
        );
        s2.route(
            "https://example.com/slow",
            FixtureResponse::body(b"fast".to_vec()),
        );
        let composed2 = ComposedTransport::new(&p2, &s2, CompositionPolicy::UnavailableOrTransport);
        let out2 = composed2
            .fetch_metadata(
                &req("https://example.com/slow"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out2.success().unwrap().bytes(), b"fast");
    }

    #[test]
    fn composition_never_retries_terminal_conditions() {
        // NotFound is terminal even under the broad policy.
        let primary = CountingTransport::new();
        let secondary = CountingTransport::new();
        primary.route("https://example.com/missing", FixtureResponse::not_found());
        secondary.route(
            "https://example.com/missing",
            FixtureResponse::body(b"other".to_vec()),
        );
        let composed = ComposedTransport::new(
            &primary,
            &secondary,
            CompositionPolicy::UnavailableOrTransport,
        );
        let out = composed
            .fetch_metadata(
                &req("https://example.com/missing"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
        assert_eq!(secondary.metadata_calls(), 0);

        // Cancellation is terminal.
        let cancel = CancelFlag::new();
        cancel.cancel();
        let out = composed.fetch_metadata(&req("https://example.com/missing"), limits(), &cancel);
        assert!(matches!(out, Err(AcquisitionError::Cancelled)));
        assert_eq!(secondary.metadata_calls(), 0);

        // TooLarge is terminal.
        let p3 = CountingTransport::new();
        let s3 = CountingTransport::new();
        p3.route(
            "https://example.com/big",
            FixtureResponse::body(vec![0u8; 2048]),
        );
        s3.route(
            "https://example.com/big",
            FixtureResponse::body(b"small".to_vec()),
        );
        let c3 = ComposedTransport::new(&p3, &s3, CompositionPolicy::UnavailableOrTransport);
        let err = c3
            .fetch_metadata(
                &req("https://example.com/big"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::TooLarge { .. }));
        assert_eq!(s3.metadata_calls(), 0);

        // InvalidInput is terminal (fails before either adapter).
        let err = c3
            .fetch_metadata(
                &req("https://example.com/big"),
                FetchLimits {
                    max_metadata_bytes: 0,
                    ..limits()
                },
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
        assert_eq!(p3.metadata_calls(), 1);
        assert_eq!(s3.metadata_calls(), 0);
    }

    #[test]
    fn composition_artifact_fallback_preserves_no_clobber() {
        let primary = CountingTransport::new();
        let secondary = CountingTransport::new();
        primary.route("https://example.com/app", FixtureResponse::unavailable());
        secondary.route(
            "https://example.com/app",
            FixtureResponse::body(vec![9u8; 128]),
        );
        let composed =
            ComposedTransport::new(&primary, &secondary, CompositionPolicy::UnavailableOnly);
        let dir = temp_dir("acq-composed-artifact");
        let dest = dir.join("app");
        let out = composed
            .fetch_artifact(
                &req("https://example.com/app"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes_written, 128);
        assert_eq!(primary.artifact_calls(), 1);
        assert_eq!(secondary.artifact_calls(), 1);
        assert_eq!(fs::read(&dest).unwrap().len(), 128);
        let _ = fs::remove_dir_all(&dir);
    }
}
