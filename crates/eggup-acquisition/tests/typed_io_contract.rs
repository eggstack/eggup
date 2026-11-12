//! M013, Defect B — the seam's typed I/O surface.
//!
//! `AcquisitionError::Io` used to flatten the underlying [`std::io::Error`] into
//! a `String`, so a consumer could not match on [`std::io::ErrorKind`] and
//! could not walk the [`std::error::Error::source`] chain. These tests pin the
//! typed contract, its bounding rule, its redaction rule, and its terminality
//! under composition.
//!
//! These live in `tests/` rather than in `src/lib.rs` so that the crate source
//! they exercise can be copied into a pristine baseline worktree; see the
//! M013 closure record for the baseline-failure proof.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use eggup_acquisition::{
    redact_url, AcquisitionError, AcquisitionRequest, AcquisitionTransport, ArtifactEvidence,
    CancelFlag, ComposedTransport, CompositionPolicy, FetchLimits, FetchOutcome, FixtureResponse,
    FixtureTransport, MetadataBytes,
};

const USERINFO_SENTINEL: &str = "eggup-m013-userinfo-sentinel";
const QUERY_SENTINEL: &str = "eggup-m013-query-sentinel";

fn credentialed_url() -> String {
    format!("https://{USERINFO_SENTINEL}:p@ss@example.com/a?token={QUERY_SENTINEL}")
}

fn limits() -> FetchLimits {
    FetchLimits {
        max_metadata_bytes: 64 * 1024,
        max_artifact_bytes: 256 * 1024,
        connect_timeout: Duration::from_secs(2),
        total_timeout: Duration::from_secs(10),
    }
}

fn temp_dir(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    let _ = std::fs::create_dir_all(&path);
    path
}

/// Extracts the `Io` payload of a typed I/O error.
fn io_parts(err: &AcquisitionError) -> (&'static str, &io::Error) {
    match err {
        AcquisitionError::Io { operation, source } => (operation, source),
        other => panic!("expected a typed Io error, got {other:?}"),
    }
}

/// The underlying `io::Error` survives the seam unchanged: it is matchable on
/// `ErrorKind` and reachable through `Error::source()`.
#[test]
fn typed_io_preserves_the_underlying_error() {
    let err =
        AcquisitionError::__adapter_io("reading artifact parent", io::Error::from_raw_os_error(2));
    let (operation, source) = io_parts(&err);
    assert_eq!(operation, "reading artifact parent");
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
    assert_eq!(source.raw_os_error(), Some(2));

    let chained = std::error::Error::source(&err).expect("a typed Io error has a source");
    assert_eq!(
        chained.downcast_ref::<io::Error>().map(io::Error::kind),
        Some(io::ErrorKind::NotFound)
    );

    // A non-I/O variant has no underlying cause.
    let plain = AcquisitionError::InvalidInput("bad".to_string());
    assert!(std::error::Error::source(&plain).is_none());
}

/// The rendered I/O detail keeps the seam's 512-byte bound and never splits a
/// UTF-8 character.
#[test]
fn typed_io_detail_is_bounded_and_utf8_safe() {
    let long_message = "é".repeat(600);
    let err = AcquisitionError::__adapter_io("writing part file", io::Error::other(long_message));
    let rendered = err.to_string();
    let detail = rendered
        .strip_prefix("acquisition I/O failed: ")
        .expect("the typed I/O detail keeps the seam's display prefix");
    assert!(
        detail.len() <= 512,
        "the typed I/O detail must not exceed 512 bytes, got {}",
        detail.len()
    );
    assert!(
        detail.is_char_boundary(detail.len()),
        "the typed I/O detail must end on a UTF-8 character boundary"
    );
    assert!(detail.starts_with("writing part file: "));
}

/// A failure raised while fetching a credential-bearing URL must not surface
/// that URL — not through `Display`, and not through `source()`.
#[test]
fn typed_io_failure_does_not_surface_url_material() {
    let dir = temp_dir("acq-m013-redaction");
    let dest = dir.join("missing-parent").join("app");
    let url = credentialed_url();
    let transport = FixtureTransport::new();
    transport.route(&url, FixtureResponse::body(b"payload".to_vec()));

    let request = AcquisitionRequest::new(&url).expect("request");
    let err = transport
        .fetch_artifact(&request, &dest, limits(), &CancelFlag::new())
        .expect_err("a missing artifact parent is an I/O failure");
    let (operation, _) = io_parts(&err);

    assert!(
        !operation.contains(USERINFO_SENTINEL),
        "the typed I/O operation must not carry URL material: {operation}"
    );
    let rendered = err.to_string();
    for sentinel in [USERINFO_SENTINEL, QUERY_SENTINEL] {
        assert!(
            !rendered.contains(sentinel),
            "the typed I/O display must not carry URL material: {rendered}"
        );
        let source_text = std::error::Error::source(&err)
            .map(|s| s.to_string())
            .unwrap_or_default();
        assert!(
            !source_text.contains(sentinel),
            "the typed I/O source must not carry URL material: {source_text}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A typed I/O failure is terminal under **both** composition policies: the
/// seam never retries a local filesystem problem on a second transport.
#[test]
fn typed_io_is_terminal_under_composition() {
    struct IoThenSuccess {
        calls: AtomicUsize,
    }

    impl AcquisitionTransport for IoThenSuccess {
        fn fetch_metadata(
            &self,
            _request: &AcquisitionRequest,
            _limits: FetchLimits,
            _cancel: &CancelFlag,
        ) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(AcquisitionError::__adapter_io(
                "reading artifact parent",
                io::Error::from_raw_os_error(2),
            ))
        }

        fn fetch_artifact(
            &self,
            _request: &AcquisitionRequest,
            _dest: &Path,
            _limits: FetchLimits,
            _cancel: &CancelFlag,
        ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(AcquisitionError::__adapter_io(
                "reading artifact parent",
                io::Error::from_raw_os_error(2),
            ))
        }
    }

    let primary = IoThenSuccess {
        calls: AtomicUsize::new(0),
    };
    let secondary = FixtureTransport::new();
    let request = AcquisitionRequest::new("https://example.com/a").expect("request");

    for policy in [
        CompositionPolicy::UnavailableOnly,
        CompositionPolicy::UnavailableOrTransport,
    ] {
        let composed = ComposedTransport::new(&primary, &secondary, policy);
        let err = composed
            .fetch_metadata(&request, limits(), &CancelFlag::new())
            .expect_err("the primary always fails with a typed I/O error");
        let (operation, source) = io_parts(&err);
        assert_eq!(operation, "reading artifact parent");
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
    }

    assert_eq!(
        primary.calls.load(Ordering::SeqCst),
        2,
        "the primary is attempted exactly once per policy and never retried"
    );
}

/// M013, Defect D — the redaction caps are byte-identical after the
/// truncation consolidation, and the seam's adapter-facing bounding entry
/// point carries the same 512-byte rule.
#[test]
fn redaction_caps_are_byte_identical_after_consolidation() {
    // `redact_url` keeps at most 256 bytes and appends the ellipsis.
    let ascii_url = format!("https://example.com/{}", "a".repeat(240));
    let redacted = redact_url(&ascii_url);
    assert_eq!(
        redacted,
        format!("https://example.com/{}…", "a".repeat(236))
    );
    assert_eq!(redacted.len(), 256 + '…'.len_utf8());

    // A multi-byte character straddling the 256-byte edge is dropped whole.
    let boundary_url = format!("https://example.com/{}étail", "a".repeat(235));
    let redacted = redact_url(&boundary_url);
    assert_eq!(
        redacted,
        format!("https://example.com/{}…", "a".repeat(235))
    );
    assert_eq!(redacted.len(), 255 + '…'.len_utf8());

    // The scrub helper keeps at most 512 bytes and appends no ellipsis.
    let scrubbed = eggup_acquisition::__scrub_upstream_text(&"b".repeat(600));
    assert_eq!(scrubbed, "b".repeat(512));
    assert_eq!(scrubbed.len(), 512);

    let scrubbed = eggup_acquisition::__scrub_upstream_text(&format!("{}étail", "b".repeat(511)));
    assert_eq!(scrubbed, "b".repeat(511));
    assert_eq!(scrubbed.len(), 511);

    // The adapter-facing bounding entry point applies the same 512-byte rule.
    let bounded = eggup_acquisition::__adapter_bound_detail("c".repeat(600));
    assert_eq!(bounded, "c".repeat(512));
    let bounded = eggup_acquisition::__adapter_bound_detail(format!("{}étail", "c".repeat(511)));
    assert_eq!(bounded, "c".repeat(511));
}
