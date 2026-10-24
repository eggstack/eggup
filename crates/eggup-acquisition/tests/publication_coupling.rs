//! M016 — the publication-coupling guard.
//!
//! The seam's next release is source-breaking (`0.2.0`), and `0.2.0` does not
//! satisfy the `^0.1.0` requirement the two adapters declare today, nor the
//! `=0.1.3` pin the leaf adapter declares. The coupling is expensive to
//! rediscover: `cargo package` resolves a dependency from the **registry**, so
//! an adapter that was never widened does not compile against the published
//! seam, and a manifest that is never widened does not resolve at all once the
//! seam's version moves.
//!
//! These tests read the seam's own `CHANGELOG.md`, the in-workspace
//! dependents' `Cargo.toml` manifests, and the dependents' `CHANGELOG.md`
//! files at compile time, so they need no filesystem discovery, no network,
//! and no wall clock. They live in `tests/` so that the sources they scan can
//! be copied into a pristine baseline worktree.
//!
//! The guard implements Cargo's requirement comparison itself, for the
//! requirement forms this workspace uses, because the seam must keep **zero**
//! dependencies. A requirement form the guard does not model is a **failure**,
//! never an assumption of acceptance.

use std::fmt;

/// The seam's own changelog. Its `Unreleased` section is the seam's record of
/// what the next publication will be.
const SEAM_CHANGELOG: &str = include_str!("../CHANGELOG.md");

/// The marker that declares the seam's next release.
const NEXT_RELEASE_MARKER: &str = "Declared next release:";

/// The dependency whose requirement this guard tracks.
const DEPENDENCY: &str = "eggup-acquisition";

/// An in-workspace dependent of the seam.
struct Dependent {
    /// The dependent crate's name.
    name: &'static str,
    /// The dependent's manifest.
    manifest: &'static str,
    /// The dependent's changelog, when it has one.
    changelog: Option<&'static str>,
}

/// Every in-workspace dependent of the seam.
const DEPENDENTS: &[Dependent] = &[
    Dependent {
        name: "eggup-curl",
        manifest: include_str!("../../eggup-curl/Cargo.toml"),
        changelog: Some(include_str!("../../eggup-curl/CHANGELOG.md")),
    },
    Dependent {
        name: "eggup-eggfetch",
        manifest: include_str!("../../eggup-eggfetch/Cargo.toml"),
        changelog: Some(include_str!("../../eggup-eggfetch/CHANGELOG.md")),
    },
    Dependent {
        name: "eggup-eggpack",
        manifest: include_str!("../../eggup-eggpack/Cargo.toml"),
        changelog: Some(include_str!("../../eggup-eggpack/CHANGELOG.md")),
    },
    Dependent {
        name: "eggup-transport-footprint",
        manifest: include_str!("../../eggup-transport-footprint/Cargo.toml"),
        changelog: None,
    },
];

/// The two adapters whose changelogs are the consumer-facing statement of the
/// compatibility consequence.
const PUBLISHED_ADAPTERS: &[&str] = &["eggup-curl", "eggup-eggfetch"];

/// A semantic version, reduced to its three numeric components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    /// The major component.
    major: u64,
    /// The minor component.
    minor: u64,
    /// The patch component.
    patch: u64,
}

impl Version {
    /// Parses `major[.minor[.patch]]`, ignoring a pre-release or build suffix.
    fn parse(text: &str) -> Option<Self> {
        let core = text.split(['-', '+']).next()?;
        let mut parts = core.split('.');
        let major = parts.next()?.trim().parse::<u64>().ok()?;
        let minor = match parts.next() {
            Some(part) => part.trim().parse::<u64>().ok()?,
            None => 0,
        };
        let patch = match parts.next() {
            Some(part) => part.trim().parse::<u64>().ok()?,
            None => 0,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The requirement forms this guard models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// `^x.y.z` — Cargo's caret requirement.
    Caret,
    /// `=x.y.z` — an exact requirement.
    Exact,
    /// `~x.y.z` — Cargo's tilde requirement.
    Tilde,
}

/// A declared version requirement.
#[derive(Debug, Clone)]
struct Requirement {
    /// The requirement text exactly as it was declared.
    raw: String,
    /// The requirement's form.
    kind: Kind,
    /// The requirement's lower bound.
    base: Version,
}

impl Requirement {
    /// Parses a requirement. A form this guard does not model is an **error**,
    /// never an assumption that the requirement accepts the next release.
    fn parse(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim();
        let (kind, rest) = if let Some(rest) = trimmed.strip_prefix('^') {
            (Kind::Caret, rest)
        } else if let Some(rest) = trimmed.strip_prefix('=') {
            (Kind::Exact, rest)
        } else if let Some(rest) = trimmed.strip_prefix('~') {
            (Kind::Tilde, rest)
        } else if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
            // A bare version is a caret requirement in Cargo.
            (Kind::Caret, trimmed)
        } else {
            return Err(format!("unrecognized requirement form `{trimmed}`"));
        };
        let base = Version::parse(rest.trim())
            .ok_or_else(|| format!("unrecognized version in requirement `{trimmed}`"))?;
        Ok(Self {
            raw: trimmed.to_string(),
            kind,
            base,
        })
    }

    /// True when the requirement accepts the given version.
    fn accepts(&self, version: Version) -> bool {
        match self.kind {
            Kind::Exact => version == self.base,
            Kind::Caret => {
                if self.base.major > 0 {
                    version.major == self.base.major && version >= self.base
                } else if self.base.minor > 0 {
                    version.major == 0 && version.minor == self.base.minor && version >= self.base
                } else {
                    version.major == 0 && version.minor == 0 && version.patch >= self.base.patch
                }
            }
            Kind::Tilde => {
                version.major == self.base.major
                    && version.minor == self.base.minor
                    && version >= self.base
            }
        }
    }
}

/// Extracts every `## Unreleased` section of a changelog.
fn unreleased_sections(changelog: &str) -> Vec<&str> {
    let mut sections = Vec::new();
    let mut rest = changelog;
    loop {
        let index = if rest.starts_with("## Unreleased") {
            Some(0)
        } else {
            rest.find("\n## Unreleased").map(|offset| offset + 1)
        };
        let Some(index) = index else {
            break;
        };
        let body = &rest[index..];
        let end = match body[1..].find("\n## ") {
            Some(offset) => offset + 1,
            None => body.len(),
        };
        sections.push(&body[..end]);
        rest = &body[end..];
    }
    sections
}

/// Extracts the next release a changelog's `Unreleased` section declares.
fn declared_next_release(section: &str) -> Option<Version> {
    let index = section.find(NEXT_RELEASE_MARKER)?;
    let after = &section[index + NEXT_RELEASE_MARKER.len()..];
    let open = after.find('`')?;
    let rest = &after[open + 1..];
    let close = rest.find('`')?;
    Version::parse(rest[..close].trim())
}

/// Extracts the value of a `version = "…"` entry.
fn version_value(text: &str) -> Option<String> {
    let after = text.split_once("version")?.1;
    let after = after.split_once('=')?.1;
    let after = after.trim_start();
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let after = &after[1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

/// Extracts the version requirement a manifest declares for a dependency.
fn declared_requirement(manifest: &str, dependency: &str) -> Option<String> {
    let mut in_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == format!("[dependencies.{dependency}]");
            continue;
        }
        if in_section {
            if let Some(value) = version_value(trimmed) {
                return Some(value);
            }
            continue;
        }
        let Some(rest) = trimmed.strip_prefix(dependency) else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim_start();
        if !rest.starts_with('{') {
            continue;
        }
        if let Some(value) = version_value(rest) {
            return Some(value);
        }
    }
    None
}

/// Extracts every `eggup-acquisition = "…"` requirement recorded on a line.
fn recorded_requirements(line: &str, dependency: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(index) = rest.find(dependency) {
        let after = &rest[index + dependency.len()..];
        if let Some(equals) = after.find('=') {
            let value = after[equals + 1..].trim_start();
            if let Some(value) = value.strip_prefix('"') {
                if let Some(end) = value.find('"') {
                    found.push(value[..end].to_string());
                }
            }
        }
        rest = &rest[index + dependency.len()..];
    }
    found
}

/// Extracts the versions a line claims a republication would be
/// "non-breaking" at. The scan is line-scoped on purpose: a claim is only
/// read when the version sits on the same line as the claim itself.
fn non_breaking_claims(line: &str) -> Vec<String> {
    const MARKER: &str = "non-breaking";
    let mut claims = Vec::new();
    let lowered = line.to_ascii_lowercase();
    let mut rest = lowered.as_str();
    while let Some(index) = rest.find(MARKER) {
        let after = &line[index + MARKER.len()..];
        if let Some(open) = after.find('`') {
            let tail = &after[open + 1..];
            if let Some(close) = tail.find('`') {
                let token = tail[..close].trim();
                if Version::parse(token).is_some() {
                    claims.push(token.to_string());
                }
            }
        }
        rest = &rest[index + MARKER.len()..];
    }
    claims
}

/// True when a changelog records, for the dependent, a requirement that
/// accepts the seam's declared next release.
fn widening_is_recorded(
    dependent: &Dependent,
    next: Version,
    seam_section: &str,
) -> Result<bool, String> {
    let mut candidates: Vec<String> = Vec::new();
    for line in seam_section.lines() {
        if line.contains(dependent.name) {
            candidates.extend(recorded_requirements(line, DEPENDENCY));
        }
    }
    if let Some(changelog) = dependent.changelog {
        for section in unreleased_sections(changelog) {
            for line in section.lines() {
                candidates.extend(recorded_requirements(line, DEPENDENCY));
            }
        }
    }
    for candidate in &candidates {
        let requirement = Requirement::parse(candidate)
            .map_err(|error| format!("{}: {error}", dependent.name))?;
        if requirement.accepts(next) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The republication versions a changelog claims are non-breaking while their
/// dependency range excludes the seam's declared next release.
fn contradicted_claims(changelog: &str, next: Version) -> Vec<String> {
    let mut contradicted = Vec::new();
    for section in unreleased_sections(changelog) {
        for line in section.lines() {
            for claim in non_breaking_claims(line) {
                let Ok(requirement) = Requirement::parse(&claim) else {
                    continue;
                };
                if !requirement.accepts(next) {
                    contradicted.push(claim);
                }
            }
        }
    }
    contradicted
}

/// The seam's changelog carries exactly one `Unreleased` section, at the top,
/// and that section declares the next release.
#[test]
fn seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release() {
    let sections = unreleased_sections(SEAM_CHANGELOG);
    assert_eq!(
        sections.len(),
        1,
        "the seam's changelog must carry exactly one `## Unreleased` section, found {}",
        sections.len()
    );
    let first_heading = SEAM_CHANGELOG
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("## "))
        .unwrap_or("");
    assert_eq!(
        first_heading, "## Unreleased",
        "the `## Unreleased` section must be the first section of the seam's changelog"
    );
    let declared = declared_next_release(sections[0]);
    assert!(
        declared.is_some(),
        "the seam's `## Unreleased` section must declare its next release as \
         `{NEXT_RELEASE_MARKER} `0.2.0``"
    );
}

/// Every in-workspace dependent either already accepts the seam's declared
/// next release, or a changelog records the widening it requires.
#[test]
fn dependent_requirements_accept_the_next_release_or_record_the_widening() {
    let sections = unreleased_sections(SEAM_CHANGELOG);
    assert_eq!(
        sections.len(),
        1,
        "the seam's changelog must carry exactly one `## Unreleased` section, found {}",
        sections.len()
    );
    let section = sections[0];
    let Some(next) = declared_next_release(section) else {
        panic!("the seam's `## Unreleased` section must declare its next release");
    };
    for dependent in DEPENDENTS {
        let raw = declared_requirement(dependent.manifest, DEPENDENCY).unwrap_or_else(|| {
            panic!(
                "{} must declare a `{DEPENDENCY}` requirement in its manifest",
                dependent.name
            )
        });
        let requirement =
            Requirement::parse(&raw).unwrap_or_else(|error| panic!("{}: {error}", dependent.name));
        if requirement.accepts(next) {
            continue;
        }
        let recorded = widening_is_recorded(dependent, next, section)
            .unwrap_or_else(|error| panic!("{}: {error}", dependent.name));
        assert!(
            recorded,
            "{} declares `{DEPENDENCY}` = `{}`, which does not accept the seam's declared \
             next release `{next}`; no changelog records the required widening for it",
            dependent.name, requirement.raw
        );
    }
}

/// The two published adapters' changelogs record the required widening, and
/// neither claims a republication version whose dependency range excludes the
/// seam's declared next release.
#[test]
fn adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version() {
    let sections = unreleased_sections(SEAM_CHANGELOG);
    assert_eq!(
        sections.len(),
        1,
        "the seam's changelog must carry exactly one `## Unreleased` section, found {}",
        sections.len()
    );
    let Some(next) = declared_next_release(sections[0]) else {
        panic!("the seam's `## Unreleased` section must declare its next release");
    };
    for name in PUBLISHED_ADAPTERS {
        let dependent = DEPENDENTS
            .iter()
            .find(|dependent| dependent.name == *name)
            .unwrap_or_else(|| panic!("{name} must be listed as a dependent of the seam"));
        let changelog = dependent
            .changelog
            .unwrap_or_else(|| panic!("{name} must carry a changelog"));
        let mut recorded = false;
        for section in unreleased_sections(changelog) {
            for line in section.lines() {
                for candidate in recorded_requirements(line, DEPENDENCY) {
                    let requirement = Requirement::parse(&candidate)
                        .unwrap_or_else(|error| panic!("{name}: {error}"));
                    if requirement.accepts(next) {
                        recorded = true;
                    }
                }
            }
        }
        assert!(
            recorded,
            "{name}'s changelog must record the required widening of its `{DEPENDENCY}` \
             requirement, because `{next}` does not satisfy the requirement it declares today"
        );
        let contradicted = contradicted_claims(changelog, next);
        assert!(
            contradicted.is_empty(),
            "{name}'s changelog claims a republication at {contradicted:?}, whose dependency \
             range excludes the seam's declared next release `{next}`"
        );
    }
}
