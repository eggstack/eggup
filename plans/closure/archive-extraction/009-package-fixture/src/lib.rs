use eggup_archive::{ArchiveFormat, ArchiveLimits};

/// Builds public Archive values resolved from the crates.io 0.2.2 release.
pub fn registry_archive_values() -> (ArchiveFormat, ArchiveLimits) {
    (ArchiveFormat::Zip, ArchiveLimits::strict_default())
}
