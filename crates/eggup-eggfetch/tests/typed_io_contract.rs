//! M013, Defect B — the eggfetch adapter's typed I/O surface.
//!
//! The adapter used to flatten the underlying [`std::io::Error`] into a
//! `String`. These tests pin the typed contract and its redaction rule through
//! the adapter's own code path.
//!
//! These live in `tests/` so the crate source they exercise can be copied into
//! a pristine baseline worktree; see the M013 closure record.

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use eggup_acquisition::{
    AcquisitionError, AcquisitionRequest, AcquisitionTransport, CancelFlag, FetchLimits,
};
use eggup_eggfetch::{EggfetchConfig, EggfetchTransport};

const USERINFO_SENTINEL: &str = "eggup-m013-eggfetch-userinfo-sentinel";
const QUERY_SENTINEL: &str = "eggup-m013-eggfetch-query-sentinel";

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

/// The adapter reports a filesystem failure as a typed I/O error: the
/// underlying error survives, is matchable on `ErrorKind`, and is reachable
/// through `Error::source()`.
#[test]
fn typed_io_is_preserved_through_the_adapter() {
    let transport = EggfetchTransport::strict(EggfetchConfig::strict()).expect("transport");
    let dir = temp_dir("eggfetch-m013-typed-io");
    // The destination parent does not exist. The adapter checks the parent
    // before it opens a connection, so this proof needs no network.
    let dest = dir.join("missing-parent").join("app");
    let request = AcquisitionRequest::new(credentialed_url()).expect("request");

    let err = transport
        .fetch_artifact(&request, &dest, limits(), &CancelFlag::new())
        .expect_err("a missing artifact parent is an I/O failure");

    let (operation, source) = match &err {
        AcquisitionError::Io { operation, source } => (*operation, source),
        other => panic!("expected a typed Io error, got {other:?}"),
    };
    assert_eq!(operation, "reading artifact parent");
    assert_eq!(source.kind(), io::ErrorKind::NotFound);

    let chained = std::error::Error::source(&err).expect("a typed Io error has a source");
    assert_eq!(
        chained.downcast_ref::<io::Error>().map(io::Error::kind),
        Some(io::ErrorKind::NotFound)
    );

    // The rendered detail keeps the seam's bounded form.
    let rendered = err.to_string();
    assert!(
        rendered.starts_with("acquisition I/O failed: reading artifact parent: "),
        "unexpected typed I/O rendering: {rendered}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A typed I/O failure raised while fetching a credential-bearing URL must not
/// surface that URL — not through `Display`, and not through `source()`.
#[test]
fn typed_io_failure_does_not_surface_url_material() {
    let transport = EggfetchTransport::strict(EggfetchConfig::strict()).expect("transport");
    let dir = temp_dir("eggfetch-m013-redaction");
    let dest = dir.join("missing-parent").join("app");
    let request = AcquisitionRequest::new(credentialed_url()).expect("request");

    let err = transport
        .fetch_artifact(&request, &dest, limits(), &CancelFlag::new())
        .expect_err("a missing artifact parent is an I/O failure");

    let rendered = err.to_string();
    let source_text = std::error::Error::source(&err)
        .map(|s| s.to_string())
        .unwrap_or_default();
    for sentinel in [USERINFO_SENTINEL, QUERY_SENTINEL] {
        assert!(
            !rendered.contains(sentinel),
            "the typed I/O display must not carry URL material: {rendered}"
        );
        assert!(
            !source_text.contains(sentinel),
            "the typed I/O source must not carry URL material: {source_text}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
