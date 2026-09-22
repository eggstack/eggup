#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Policy-neutral local mechanics for verified multi-artifact updates."]
#![doc = ""]
#![doc = "The core crate deliberately does not fetch bytes, manage services, or choose release policy."]
#![doc = "At the foundation stage it exposes no live-update operation; later milestones add"]
#![doc = "validated domain, preparation, verification, and transaction layers independently."]

/// The crate version recorded by the package metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests {
    use super::test_support::{FailureInjector, FailurePoint, InstallationRoot};

    #[test]
    fn fixture_writes_and_reads_exact_bytes() {
        let fixture = InstallationRoot::new().expect("fixture");
        fixture
            .write_file("bin/eggup", b"exact bytes")
            .expect("write");

        assert_eq!(
            fixture.read_file("bin/eggup").expect("read"),
            b"exact bytes"
        );
        assert!(fixture.path().join("bin/eggup").is_file());
    }

    #[test]
    fn fixture_rejects_escape_paths() {
        let fixture = InstallationRoot::new().expect("fixture");

        let error = fixture.write_file("../outside", b"nope").unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(!fixture.path().parent().unwrap().join("outside").exists());
    }

    #[test]
    fn failure_injector_defaults_to_no_failure() {
        let mut injector = FailureInjector::default();

        assert!(injector.check(FailurePoint::Prepare).is_ok());
        assert!(injector.check(FailurePoint::Commit).is_ok());
    }
}
