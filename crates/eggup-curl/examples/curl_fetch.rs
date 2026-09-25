//! Curl-only footprint fixture: links `eggup-acquisition` + `eggup-curl` only.
//!
//! This example exists to produce a comparable release binary proving that
//! curl-only consumption does not require Eggfetch or an embedded HTTP/TLS
//! stack. It performs no network I/O by default; it constructs the adapter
//! from an explicit executable path and prints the redacted request.

use eggup_acquisition::{AcquisitionRequest, FetchLimits};
use eggup_curl::{discover_curl_executable, CurlConfig, CurlTransport};

fn main() {
    let exe = discover_curl_executable().unwrap_or_else(|_| "/usr/bin/curl".into());
    let transport =
        CurlTransport::with_executable(exe, CurlConfig::strict()).expect("curl transport");
    let request = AcquisitionRequest::new("https://example.com/artifact").expect("request");
    println!("curl executable: {}", transport.executable().display());
    println!("request: {}", request.redacted());
    println!("limits: {:?}", FetchLimits::default());
}
