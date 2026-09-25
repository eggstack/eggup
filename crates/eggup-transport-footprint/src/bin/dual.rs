//! Dual-transport footprint binary: links seam + curl + Eggfetch + composition.

use eggup_acquisition::{ComposedTransport, CompositionPolicy};

fn main() {
    let curl = eggup_curl::CurlTransport::with_executable(
        "/usr/bin/curl",
        eggup_curl::CurlConfig::strict(),
    )
    .expect("curl transport");
    let eggfetch =
        eggup_eggfetch::EggfetchTransport::strict(eggup_eggfetch::EggfetchConfig::strict())
            .expect("eggfetch transport");
    let composed = ComposedTransport::new(&curl, &eggfetch, CompositionPolicy::UnavailableOnly);
    println!("dual policy: {:?}", composed.policy());
}
