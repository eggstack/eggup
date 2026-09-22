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
    /// Maximum artifact bytes written to disk, when bounded.
    pub max_artifact_bytes: Option<u64>,
    /// Deadline for establishing the connection (advisory for fixtures).
    pub connect_timeout: Duration,
    /// Total wall-clock deadline for the fetch.
    pub total_timeout: Duration,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            max_metadata_bytes: 256 * 1024,
            max_artifact_bytes: Some(128 * 1024 * 1024),
            connect_timeout: Duration::from_secs(10),
            total_timeout: Duration::from_secs(120),
        }
    }
}

impl FetchLimits {
    /// Creates limits with explicit bounds.
    pub fn new(
        max_metadata_bytes: usize,
        max_artifact_bytes: Option<u64>,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Result<Self, AcquisitionError> {
        if max_metadata_bytes == 0 || max_metadata_bytes > 16 * 1024 * 1024 {
            return Err(AcquisitionError::invalid("metadata bound out of range"));
        }
        if total_timeout.is_zero() || connect_timeout.is_zero() {
            return Err(AcquisitionError::invalid("timeouts must be positive"));
        }
        Ok(Self {
            max_metadata_bytes,
            max_artifact_bytes,
            connect_timeout,
            total_timeout,
        })
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
}

impl AcquisitionError {
    pub(crate) fn invalid(detail: impl Into<String>) -> Self {
        Self::InvalidInput(bound_detail(detail.into()))
    }

    pub(crate) fn transport_redacted(detail: impl Into<String>) -> Self {
        Self::Transport(bound_detail(detail.into()))
    }

    pub(crate) fn io(operation: &'static str, source: std::io::Error) -> Self {
        Self::Io(bound_detail(format!("{operation}: {source}")))
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
        }
    }
}

impl std::error::Error for AcquisitionError {}

fn bound_detail(mut s: String) -> String {
    if s.len() > 512 {
        s.truncate(512);
    }
    s
}

/// Redacts credential-bearing URL material for diagnostics.
///
/// Strips `user:password@` credentials and truncates overlong URLs. The exact
/// URL is still used for the fetch; only the diagnostic string is redacted.
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
    // Redact query strings, which commonly carry tokens.
    if let Some(q) = out.find('?') {
        out.truncate(q);
        out.push_str("?<redacted>");
    }
    if out.len() > 256 {
        out.truncate(256);
        out.push('…');
    }
    out
}

/// The minimal transport-neutral acquisition contract.
///
/// Implementations receive an exact URL, enforce caller bounds, and stream
/// artifacts to files. They never select releases, versions, mirrors, or
/// fallback sources, and never execute downloaded content.
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
    /// The file at `dest` is created atomically: bytes stream to a
    /// transaction-owned temporary sibling and are renamed into place only on
    /// full success. Partial outputs are cleaned on failure or cancellation
    /// where safe. `dest`'s parent must already exist.
    fn fetch_artifact(
        &self,
        request: &AcquisitionRequest,
        dest: &Path,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError>;
}

static NEXT_FIXTURE_NONCE: AtomicU64 = AtomicU64::new(0);

/// One deterministic fixture response.
#[derive(Debug, Clone)]
pub struct FixtureResponse {
    kind: FixtureKind,
}

#[derive(Debug, Clone)]
enum FixtureKind {
    Body(Vec<u8>),
    NotFound,
    Failure(String),
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
    pub fn failure(detail: impl Into<String>) -> Self {
        Self {
            kind: FixtureKind::Failure(detail.into()),
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
        let start = Instant::now();
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        let fixture = self.lookup(request.url())?;
        match fixture.kind {
            FixtureKind::NotFound => Ok(FetchOutcome::NotFound),
            FixtureKind::Failure(detail) => Err(AcquisitionError::transport_redacted(detail)),
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
        let fixture = self.lookup(request.url())?;
        let body = match fixture.kind {
            FixtureKind::NotFound => return Ok(FetchOutcome::NotFound),
            FixtureKind::Failure(detail) => {
                return Err(AcquisitionError::transport_redacted(detail));
            }
            FixtureKind::Body(b) => b,
            FixtureKind::Truncated { prefix_len } => {
                // Simulate a partial write then a hard failure; no file is promoted.
                let nonce = NEXT_FIXTURE_NONCE.fetch_add(1, Ordering::Relaxed);
                let tmp = parent.join(format!(".eggup-acquire-{nonce}.part"));
                let prefix = vec![0u8; prefix_len.min(1024)];
                if fs::write(&tmp, &prefix).is_ok() {
                    let _ = fs::remove_file(&tmp);
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
        if let Some(max) = limits.max_artifact_bytes {
            if body.len() as u64 > max {
                return Err(AcquisitionError::TooLarge { limit: max });
            }
        }
        if start.elapsed() > limits.total_timeout {
            return Err(AcquisitionError::Timeout { phase: "total" });
        }
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        // Atomic promotion: stream to a transaction-owned sibling, then rename.
        let nonce = NEXT_FIXTURE_NONCE.fetch_add(1, Ordering::Relaxed);
        let tmp = parent.join(format!(".eggup-acquire-{nonce}.part"));
        struct Guard<'a> {
            path: &'a Path,
            disarm: bool,
        }
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                if !self.disarm {
                    let _ = fs::remove_file(self.path);
                }
            }
        }
        let mut guard = Guard {
            path: &tmp,
            disarm: false,
        };
        // Chunked write so cancellation and partial-write failures are observable.
        let mut written: u64 = 0;
        {
            use std::io::Write;
            let mut file = fs::File::create(&tmp)
                .map_err(|e| AcquisitionError::io("creating part file", e))?;
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
        }
        fs::rename(&tmp, dest).map_err(|e| AcquisitionError::io("promoting artifact", e))?;
        guard.disarm = true;
        Ok(FetchOutcome::Success(ArtifactEvidence {
            bytes_written: written,
        }))
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
            max_artifact_bytes: Some(4096),
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
}
