//! Linkage-discipline and manifest-shape guard for `eggup-transport-footprint`.
//!
//! The crate exists so that exactly one fact is visible: which HTTP transport
//! stack a consumer of `eggup-acquisition` links into its binary. The
//! `required-features` split makes an isolated single-adapter configuration
//! *representable*, but it does **not** make an accidental cross-link a build
//! error: under `--all-features` both optional adapters are visible to every
//! target in the package, and a `#[cfg(feature = "...")]`-gated reference is
//! stripped before name resolution, so even the isolated builds cannot see it.
//!
//! These two guards therefore check the claim at the source level, where it is
//! visible in every feature configuration:
//!
//! - **T1** rejects a sibling-adapter reference in a single-adapter fixture and
//!   requires the composition fixture to keep referencing both adapters.
//! - **T2** pins the manifest shape that produces the recorded dependency
//!   counts 3 / 92 / 93, and the crate's non-publishability.
//!
//! The checks are **source-level**, not behavioral: the three binaries are never
//! executed, they perform no measurement, and this file asserts nothing about
//! throughput, latency, reliability, or binary size.
//!
//! `std` only — no dependency is added, so `Cargo.lock` does not change. The
//! crate root is resolved through `CARGO_MANIFEST_DIR`, so the guard does not
//! depend on the working directory. The test target carries no
//! `required-features`, so it is built and run in **every** feature
//! configuration, including `default = []`.

use std::fs;
use std::path::{Path, PathBuf};

/// The crate directory name, used to name the crate in guard messages.
const CRATE_DIR: &str = "eggup-transport-footprint";

/// The crate root, resolved through the manifest so the guard does not depend
/// on the working directory.
fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Reads a file, failing with the path when it cannot be read.
fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", path.display()))
}

/// The sibling-adapter tokens a single-adapter fixture must never contain.
const FORBIDDEN_ADAPTER_TOKENS: &[(&str, &str)] = &[
    ("src/bin/curl_only.rs", "eggup_eggfetch"),
    ("src/bin/eggfetch_only.rs", "eggup_curl"),
];

/// The adapter tokens the composition fixture must keep containing.
const REQUIRED_ADAPTER_TOKENS: &[(&str, &str)] = &[
    ("src/bin/dual.rs", "eggup_curl"),
    ("src/bin/dual.rs", "eggup_eggfetch"),
];

/// T1 — the cross-link guard.
///
/// The token check is over the **whole file text**, not over the lines that
/// survive a `cfg` evaluation. That is the entire point: a `#[cfg]`-gated
/// sibling-adapter reference compiles cleanly in the isolated configurations,
/// so only a whole-file text check can reject it.
#[test]
fn single_adapter_fixtures_never_reference_the_sibling_adapter() {
    let root = crate_root();
    let mut violations: Vec<String> = Vec::new();

    for (relative, forbidden) in FORBIDDEN_ADAPTER_TOKENS {
        let text = read_text(&root.join(relative));
        if text.contains(forbidden) {
            violations.push(format!(
                "{relative} references {forbidden}; the single-adapter fixture must link one adapter only"
            ));
        }
    }

    for (relative, required) in REQUIRED_ADAPTER_TOKENS {
        let text = read_text(&root.join(relative));
        if !text.contains(required) {
            violations.push(format!(
                "{relative} does not reference {required}; the composition fixture must keep exercising both adapters"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "cross-link guard failed for {CRATE_DIR}:\n  - {}",
        violations.join("\n  - ")
    );
}

/// One table of the manifest: its header text and its `key = value` pairs.
struct Table {
    header: String,
    entries: Vec<(String, String)>,
}

/// Drops a trailing `#` comment, ignoring a `#` inside a quoted string.
fn strip_comment(line: &str) -> &str {
    let mut quote: Option<u8> = None;
    for (index, byte) in line.as_bytes().iter().enumerate() {
        match quote {
            Some(open) => {
                if *byte == open {
                    quote = None;
                }
            }
            None => match byte {
                b'"' | b'\'' => quote = Some(*byte),
                b'#' => return &line[..index],
                _ => {}
            },
        }
    }
    line
}

/// Reads the subset of TOML the manifest uses — table headers, `[[array of
/// tables]]` headers, and `key = value` pairs — without a TOML parser, because
/// adding one would add a dependency and change `Cargo.lock`.
fn parse_tables(text: &str) -> Vec<Table> {
    let mut tables: Vec<Table> = Vec::new();
    let mut current: Option<usize> = None;

    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("[[") {
            let header = rest.strip_suffix("]]").unwrap_or(rest).trim();
            tables.push(Table {
                header: format!("[[{header}]]"),
                entries: Vec::new(),
            });
            current = Some(tables.len() - 1);
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let header = rest.strip_suffix(']').unwrap_or(rest).trim();
            tables.push(Table {
                header: header.to_string(),
                entries: Vec::new(),
            });
            current = Some(tables.len() - 1);
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if let Some(index) = current {
                tables[index]
                    .entries
                    .push((key.trim().to_string(), value.trim().to_string()));
            }
        }
    }

    tables
}

/// Finds the first table with the given header.
fn table<'a>(tables: &'a [Table], header: &str) -> Option<&'a Table> {
    tables.iter().find(|table| table.header == header)
}

/// Removes a surrounding pair of quotes from a scalar value.
///
/// A bracketed list such as `["dep:eggup-curl"]` does not begin and end with
/// a quote, so it is returned unchanged.
fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        if (first == b'"' || first == b'\'') && bytes[bytes.len() - 1] == first {
            return &value[1..value.len() - 1];
        }
    }
    value
}

/// Reads one value out of a table, with a scalar's surrounding quotes removed.
fn value<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    table
        .entries
        .iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, entry_value)| unquote(entry_value.as_str()))
}

/// Normalizes a bracketed list so that whitespace inside it is not a
/// difference the guard cares about.
fn normalize_list(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Collects every dependency name the manifest declares, unconditional or
/// optional, across the dependency, dev-dependency, and build-dependency
/// tables.
fn declared_dependencies(tables: &[Table]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for table in tables {
        if matches!(
            table.header.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        ) {
            for (key, _) in &table.entries {
                names.push(key.clone());
            }
        } else if let Some(rest) = table
            .header
            .strip_prefix("dependencies.")
            .or_else(|| table.header.strip_prefix("dev-dependencies."))
            .or_else(|| table.header.strip_prefix("build-dependencies."))
        {
            names.push(rest.to_string());
        }
    }
    names
}

/// T2 — the manifest dependency-shape guard.
///
/// It pins the **shape** that produces the recorded dependency counts, not the
/// counts themselves: the counts are a property of a 92-crate resolution
/// against the current registry state, so a hard-coded count assertion would
/// break on an upstream release and would guard nothing this guard does not
/// already guard more stably.
#[test]
fn manifest_keeps_the_shape_that_produces_the_recorded_counts() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_text(&manifest_path);
    let tables = parse_tables(&manifest);
    let mut violations: Vec<String> = Vec::new();

    // Invariant 1: the crate is non-published and must never acquire a
    // registry version.
    match table(&tables, "package") {
        Some(package) => match value(package, "publish") {
            Some("false") => {}
            Some(other) => violations.push(format!(
                "Cargo.toml [package] publish must be false; found {other}"
            )),
            None => violations.push(
                "Cargo.toml [package] must declare publish = false; it is absent".to_string(),
            ),
        },
        None => violations.push("Cargo.toml must declare a [package] table".to_string()),
    }

    // Invariant 4: `eggup-acquisition` is the only unconditional dependency.
    match table(&tables, "dependencies") {
        Some(dependencies) => {
            let mut names: Vec<&str> = dependencies
                .entries
                .iter()
                .map(|(key, _)| key.as_str())
                .collect();
            names.sort_unstable();
            if names != ["eggup-acquisition"] {
                violations.push(format!(
                    "Cargo.toml [dependencies] must declare exactly eggup-acquisition as its unconditional dependency; found {names:?}"
                ));
            }
        }
        None => violations.push("Cargo.toml must declare a [dependencies] table".to_string()),
    }
    for kind in ["dev-dependencies", "build-dependencies"] {
        if let Some(extra) = table(&tables, kind) {
            if !extra.entries.is_empty() {
                violations.push(format!(
                    "Cargo.toml [{kind}] must stay empty; found {:?}",
                    extra.entries
                ));
            }
        }
    }

    // The optionality of the two adapters is what produces the 3 / 92 / 93
    // split: an unconditional adapter edge would put both stacks into every
    // build.
    for adapter in ["eggup-curl", "eggup-eggfetch"] {
        let header = format!("dependencies.{adapter}");
        match table(&tables, &header) {
            Some(dependency) => match value(dependency, "optional") {
                Some("true") => {}
                Some(other) => violations.push(format!(
                    "Cargo.toml [{header}] optional must be true; found {other}"
                )),
                None => violations.push(format!(
                    "Cargo.toml [{header}] must declare optional = true; it is absent"
                )),
            },
            None => violations.push(format!("Cargo.toml must declare a [{header}] table")),
        }
    }

    // Invariant 3: the feature surface that makes the isolated configurations
    // representable. A non-empty `default` would put the dual configuration
    // into every default build.
    match table(&tables, "features") {
        Some(features) => {
            for (key, expected) in [
                ("curl", "[\"dep:eggup-curl\"]"),
                ("eggfetch", "[\"dep:eggup-eggfetch\"]"),
                ("default", "[]"),
            ] {
                match value(features, key) {
                    Some(found) if normalize_list(found) == expected => {}
                    Some(found) => violations.push(format!(
                        "Cargo.toml [features] {key} must be {expected}; found {found}"
                    )),
                    None => violations.push(format!(
                        "Cargo.toml [features] must declare {key}; it is absent"
                    )),
                }
            }
        }
        None => violations.push("Cargo.toml must declare a [features] table".to_string()),
    }

    // Invariant 3: each binary keeps the `required-features` that makes it
    // belong to exactly one configuration.
    let bins: Vec<&Table> = tables
        .iter()
        .filter(|table| table.header == "[[bin]]")
        .collect();
    if bins.len() != 3 {
        violations.push(format!(
            "Cargo.toml must declare exactly three [[bin]] targets; found {}",
            bins.len()
        ));
    }
    for (name, expected) in [
        ("curl_only", "[\"curl\"]"),
        ("eggfetch_only", "[\"eggfetch\"]"),
        ("dual", "[\"curl\",\"eggfetch\"]"),
    ] {
        let bin = bins
            .iter()
            .copied()
            .find(|bin| value(bin, "name") == Some(name));
        match bin {
            Some(bin) => match value(bin, "required-features") {
                Some(found) if normalize_list(found) == expected => {}
                Some(found) => violations.push(format!(
                    "Cargo.toml [[bin]] {name} required-features must be {expected}; found {found}"
                )),
                None => violations.push(format!(
                    "Cargo.toml [[bin]] {name} must declare required-features = {expected}"
                )),
            },
            None => violations.push(format!("Cargo.toml must declare a [[bin]] named {name}")),
        }
    }

    // Invariant 4: the crate must never gain a dependency on a policy-bearing
    // crate.
    for forbidden in [
        "eggup-core",
        "eggup-service",
        "eggup-archive",
        "eggup-eggpack",
    ] {
        if declared_dependencies(&tables)
            .iter()
            .any(|name| name == forbidden)
        {
            violations.push(format!(
                "Cargo.toml must not depend on {forbidden}; the footprint crate's only unconditional dependency is eggup-acquisition and its only optional dependencies are eggup-curl and eggup-eggfetch"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "manifest-shape guard failed for {CRATE_DIR}:\n  - {}",
        violations.join("\n  - ")
    );
}
