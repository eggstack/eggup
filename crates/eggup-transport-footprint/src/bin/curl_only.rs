//! Curl-only footprint binary: links seam + curl, never Eggfetch.

use eggup_acquisition::{AcquisitionRequest, FetchLimits};
use eggup_curl::{CurlConfig, CurlTransport};

fn main() {
    let transport = CurlTransport::with_executable("/usr/bin/curl", CurlConfig::strict())
        .expect("curl transport");
    let request = AcquisitionRequest::new("https://example.com/artifact").expect("request");
    println!("curl-only: {}", transport.executable().display());
    println!("request: {}", request.redacted());
    println!("limits: {:?}", FetchLimits::default());
}
