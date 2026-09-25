//! Eggfetch-only footprint fixture: links `eggup-acquisition` + `eggup-eggfetch` only.
//!
//! This example exists to produce a comparable release binary proving that
//! Eggfetch-only consumption does not require curl. It performs no network
//! I/O by default; it constructs the strict adapter and prints the redacted
//! request.

use eggup_acquisition::{AcquisitionRequest, FetchLimits};
use eggup_eggfetch::{EggfetchConfig, EggfetchTransport};

fn main() {
    let transport =
        EggfetchTransport::strict(EggfetchConfig::strict()).expect("eggfetch transport");
    let request = AcquisitionRequest::new("https://example.com/artifact").expect("request");
    println!("redirects: {}", transport.config().max_redirects);
    println!("request: {}", request.redacted());
    println!("limits: {:?}", FetchLimits::default());
}
