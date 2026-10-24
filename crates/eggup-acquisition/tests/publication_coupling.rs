//! M016 — the publication-coupling guard.
//!
//! The seam's **published** version is the version every in-workspace
//! dependent must resolve. `cargo package` resolves a dependency from the
//! **registry**, so an adapter whose requirement was never widened does not
//! compile against the published seam, and a manifest that was never widened
//! does not resolve at all once the seam's version moves. The coupling is
//! expensive to rediscover, so it is enforced mechanically here.
//!
//! These tests read the seam's own `CHANGELOG.md`, the in-workspace
//! dependents' `Cargo.toml` manifests, and the dependents' `CHANGELOG.md`
//! files at compile time, so they need no filesystem discovery, no network,
//! and no wall clock. They live in `tests/` so that the sources they scan can
//! be copied into a pristine baseline worktree.
//!
//! The seam's published version is **derived** from the highest `## X.Y.Z`
//! heading its changelog declares, never from a literal. A literal would stop
//! guarding the moment the next release shipped, which is exactly the drift
//! this guard exists to catch; the derivation keeps following the changelog as
//! the seam's releases move.
//!
//! The guard implements Cargo's requirement comparison itself, for the
//! requirement forms this workspace uses, because the seam must keep **zero**
//! dependencies. A requirement form the guard does not model is a **failure**,
//! never an assumption of acceptance.
//!
//! The three test names below are the ones `tests/assertion_guard.rs` names
//! as its required behavior witnesses, so they are retained verbatim even
//! where their wording ("next release") predates the re-pointing at the
//! published version. Renaming them is a two-file change and belongs to the
//! milestone that owns both files.

use std::fmt;

/// The seam's own changelog. Its release sections are the seam's record of
/// what it has published.
const SEAM_CHANGELOG: &str = include_str!("../CHANGELOG.md");

/// The dependency whose requirement this guard tracks.
const DEPENDENCY: &str = "eggup-acquisition";

/// The prefix every changelog section heading carries.
const HEADING: &str = "## ";

/// The heading of the section that stages the next release.
const UNRELEASED: &str = "## Unreleased";

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
    /// never an assumption that the requirement accepts the published release.
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

/// A changelog section: its heading line and the body that follows it.
struct Section<'a> {
    /// The section's heading line, including its `## ` prefix.
    heading: &'a str,
    /// The section's body: everything after the heading line.
    body: &'a str,
}

/// Splits a changelog into its `##` sections, in the order they appear.
fn sections(changelog: &str) -> Vec<Section<'_>> {
    let mut found: Vec<Section<'_>> = Vec::new();
    let mut heading: Option<&str> = None;
    let mut start = 0usize;
    let mut offset = 0usize;
    for line in changelog.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with(HEADING) {
            if let Some(previous) = heading {
                found.push(Section {
                    heading: previous,
                    body: &changelog[start..offset],
                });
            }
            heading = Some(trimmed);
            start = offset;
        }
        offset += line.len();
    }
    if let Some(previous) = heading {
        found.push(Section {
            heading: previous,
            body: &changelog[start..],
        });
    }
    found
}

/// The version a section heading declares, when it declares one. A heading
/// that is not a `## X.Y.Z` release heading — `## Unreleased`, for one —
/// declares no version.
fn section_version(heading: &str) -> Option<Version> {
    let title = heading.trim().strip_prefix(HEADING)?;
    let token = title.split_whitespace().next()?;
    Version::parse(token)
}

/// The version headings a changelog declares, in the order they appear.
fn version_headings(changelog: &str) -> Vec<Version> {
    sections(changelog)
        .iter()
        .filter_map(|section| section_version(section.heading))
        .collect()
}

/// The highest version heading a changelog declares: the version it has
/// published.
fn published_version(changelog: &str) -> Option<Version> {
    version_headings(changelog).into_iter().max()
}

/// The seam's published version, or a loud failure when the seam's changelog
/// declares no release version at all. A changelog whose shape the guard does
/// not understand is a failure, never an assumption.
fn seam_published_version() -> Version {
    published_version(SEAM_CHANGELOG).unwrap_or_else(|| {
        panic!(
            "the seam's changelog must declare at least one `## X.Y.Z` version heading; the \
             guard reads the published version from the changelog, never from a literal"
        )
    })
}

/// The sections that describe the current coupling: the section that stages
/// the next release, and the most recent published release. Older sections
/// are frozen records of what was true when they were written, so they are
/// not re-judged against the current release.
fn current_sections(changelog: &str) -> Vec<Section<'_>> {
    let published = published_version(changelog);
    sections(changelog)
        .into_iter()
        .filter(|section| {
            section.heading.trim() == UNRELEASED
                || published
                    .is_some_and(|version| section_version(section.heading) == Some(version))
        })
        .collect()
}

/// The number of `## Unreleased` sections a changelog carries.
fn unreleased_count(changelog: &str) -> usize {
    sections(changelog)
        .iter()
        .filter(|section| section.heading.trim() == UNRELEASED)
        .count()
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

/// The requirements a changelog records for the dependency, read from every
/// section that carries one. The scan is section-agnostic on purpose: the
/// requirement a changelog records lives in the release section that published
/// it, not only in a staging section.
fn changelog_requirements(changelog: &str, dependency: &str) -> Vec<String> {
    let mut found = Vec::new();
    for section in sections(changelog) {
        for line in section.body.lines() {
            found.extend(recorded_requirements(line, dependency));
        }
    }
    found
}

/// The requirements the seam's changelog records for the named dependent.
fn seam_requirements_for(changelog: &str, dependent: &str) -> Vec<String> {
    let mut found = Vec::new();
    for section in sections(changelog) {
        for line in section.body.lines() {
            if line.contains(dependent) {
                found.extend(recorded_requirements(line, DEPENDENCY));
            }
        }
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
/// accepts the seam's published version. The record may live in the seam's
/// own changelog or in the dependent's changelog; either one discharges the
/// obligation, because the guard protects against a **silent** coupling, not
/// against a recorded one.
fn widening_is_recorded(dependent: &Dependent, published: Version) -> Result<bool, String> {
    let mut candidates = seam_requirements_for(SEAM_CHANGELOG, dependent.name);
    if let Some(changelog) = dependent.changelog {
        candidates.extend(changelog_requirements(changelog, DEPENDENCY));
    }
    for candidate in &candidates {
        let requirement = Requirement::parse(candidate)
            .map_err(|error| format!("{}: {error}", dependent.name))?;
        if requirement.accepts(published) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The republication versions a changelog's current sections claim are
/// non-breaking while their dependency range excludes the seam's published
/// release.
fn contradicted_claims(changelog: &str, published: Version) -> Vec<String> {
    let mut contradicted = Vec::new();
    for section in current_sections(changelog) {
        for line in section.body.lines() {
            for claim in non_breaking_claims(line) {
                let Ok(requirement) = Requirement::parse(&claim) else {
                    continue;
                };
                if !requirement.accepts(published) {
                    contradicted.push(claim);
                }
            }
        }
    }
    contradicted
}

/// The seam's changelog declares the published version this guard checks, and
/// carries exactly one `## Unreleased` section.
#[test]
fn seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release() {
    let published = seam_published_version();
    let unreleased = unreleased_count(SEAM_CHANGELOG);
    assert_eq!(
        unreleased, 1,
        "the seam's changelog must carry exactly one `## Unreleased` section, found {unreleased}"
    );
    let headings = version_headings(SEAM_CHANGELOG);
    assert!(
        !headings.is_empty(),
        "the seam's changelog must declare at least one `## X.Y.Z` version heading"
    );
    assert_eq!(
        headings[0], published,
        "the seam's changelog must list its releases newest-first, so that its first version \
         heading is the published version the guard checks"
    );
}

/// Every in-workspace dependent either already accepts the seam's published
/// version, or a changelog records the widening it requires.
#[test]
fn dependent_requirements_accept_the_next_release_or_record_the_widening() {
    let published = seam_published_version();
    for dependent in DEPENDENTS {
        let raw = declared_requirement(dependent.manifest, DEPENDENCY).unwrap_or_else(|| {
            panic!(
                "{} must declare a `{DEPENDENCY}` requirement in its manifest",
                dependent.name
            )
        });
        let requirement =
            Requirement::parse(&raw).unwrap_or_else(|error| panic!("{}: {error}", dependent.name));
        if requirement.accepts(published) {
            continue;
        }
        let recorded = widening_is_recorded(dependent, published)
            .unwrap_or_else(|error| panic!("{}: {error}", dependent.name));
        assert!(
            recorded,
            "{} declares `{DEPENDENCY}` = `{}`, which does not accept the seam's published \
             version `{published}`; no changelog records the required widening for it",
            dependent.name, requirement.raw
        );
    }
}

/// The two published adapters' changelogs record the requirement they publish,
/// and neither claims a republication version whose dependency range excludes
/// the seam's published version.
#[test]
fn adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version() {
    let published = seam_published_version();
    for name in PUBLISHED_ADAPTERS {
        let dependent = DEPENDENTS
            .iter()
            .find(|dependent| dependent.name == *name)
            .unwrap_or_else(|| panic!("{name} must be listed as a dependent of the seam"));
        let changelog = dependent
            .changelog
            .unwrap_or_else(|| panic!("{name} must carry a changelog"));
        let recorded = changelog_requirements(changelog, DEPENDENCY);
        let mut accepts = false;
        for candidate in &recorded {
            let requirement =
                Requirement::parse(candidate).unwrap_or_else(|error| panic!("{name}: {error}"));
            if requirement.accepts(published) {
                accepts = true;
            }
        }
        assert!(
            accepts,
            "{name}'s changelog must record the `{DEPENDENCY}` requirement it publishes, and \
             that requirement must accept the seam's published version `{published}`; the \
             changelog records {recorded:?}"
        );
        let contradicted = contradicted_claims(changelog, published);
        assert!(
            contradicted.is_empty(),
            "{name}'s changelog claims a republication at {contradicted:?}, whose dependency \
             range excludes the seam's published version `{published}`"
        );
    }
}
