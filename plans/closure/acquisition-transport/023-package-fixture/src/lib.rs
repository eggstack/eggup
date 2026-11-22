use eggup_acquisition::FetchLimits;
use eggup_curl::{CurlConfig, CurlTransport};
use eggup_eggfetch::{EggfetchConfig, EggfetchTransport};
use eggup_service::{ServiceId, ServiceSpec};
use std::path::PathBuf;
use std::time::Duration;

/// Compiles the exact registry releases' public seam and adapter constructors.
///
/// This fixture performs no network request and starts no service. The direct
/// legacy dependencies assert that Curl and Service's immutable caret releases
/// still resolve beside the 0.2.2 seam and Eggfetch adapter.
pub fn selected_public_types() -> (FetchLimits, EggfetchTransport, CurlTransport, ServiceSpec) {
    let limits = FetchLimits::new(1024, 4096, Duration::from_secs(1), Duration::from_secs(2))
        .expect("valid fixture limits");
    let transport =
        EggfetchTransport::strict(EggfetchConfig::strict()).expect("registry Eggfetch adapter");
    let curl_executable = if cfg!(windows) {
        r"C:\Windows\System32\curl.exe"
    } else {
        "/usr/bin/curl"
    };
    let curl = CurlTransport::with_executable(curl_executable, CurlConfig::strict())
        .expect("registry Curl adapter");
    let service_executable = if cfg!(windows) {
        r"C:\Windows\System32\cmd.exe"
    } else {
        "/usr/bin/true"
    };
    let service = ServiceSpec::new(
        ServiceId::new("registry-fixture").expect("valid service id"),
        PathBuf::from(service_executable),
        Vec::new(),
        None,
    )
    .expect("valid service spec");
    (limits, transport, curl, service)
}
