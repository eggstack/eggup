//! Eggfetch-only footprint binary: links seam + Eggfetch, never curl.

use eggup_acquisition::{AcquisitionRequest, FetchLimits};
use eggup_eggfetch::{EggfetchConfig, EggfetchTransport};

fn main() {
    let transport =
        EggfetchTransport::strict(EggfetchConfig::strict()).expect("eggfetch transport");
    let request = AcquisitionRequest::new("https://example.com/artifact").expect("request");
    println!(
        "eggfetch-only redirects: {}",
        transport.config().max_redirects
    );
    println!("request: {}", request.redacted());
    println!("limits: {:?}", FetchLimits::default());
}
