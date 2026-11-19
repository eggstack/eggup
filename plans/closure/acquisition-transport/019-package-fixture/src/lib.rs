use eggup_acquisition::{AcquisitionError, AcquisitionRequest};

/// Constructs a request using the exact registry-pinned acquisition seam.
pub fn registry_request() -> Result<AcquisitionRequest, AcquisitionError> {
    AcquisitionRequest::new("https://example.com/releases.json")
}
