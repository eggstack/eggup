use eggup_eggfetch::{EggfetchConfig, EggfetchTransport};

/// Constructs the public transport type from the exact crates.io package.
pub fn registry_transport() -> EggfetchTransport {
    // Compiles the public constructor without making a network request.
    EggfetchTransport::strict(EggfetchConfig::strict()).expect("registry Eggfetch adapter")
}
