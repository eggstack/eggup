#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Versioned distribution contract schema for release-time tooling."]
#![doc = ""]
#![doc = "A `DistributionContract` describes one product's release layout once:"]
#![doc = "target triples, platform aliases, asset file names, checksum sidecar"]
#![doc = "names, archive members, and install names. It does not fetch releases,"]
#![doc = "order versions, verify checksums, extract archives, generate installers,"]
#![doc = "or become a runtime dependency of ordinary consumers."]

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Schema version 1. The only version understood by this crate.
pub const SCHEMA_V1: u32 = 1;

/// Maximum lengths to keep diagnostics bounded.
const MAX_ID_LEN: usize = 64;
const MAX_NAME_LEN: usize = 128;
const MAX_TEMPLATE_LEN: usize = 256;
const MAX_DETAIL_LEN: usize = 512;

/// Typed distribution errors. Parsing never guesses or falls back.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DistError {
    /// Input violates the schema contract.
    InvalidInput(String),
    /// The document declares an unsupported schema version.
    UnsupportedVersion {
        /// The declared version.
        found: u32,
    },
    /// Two targets share a canonical triple.
    DuplicateTarget(String),
    /// An alias collides with another alias or triple.
    DuplicateAlias(String),
    /// No target matches the requested triple or alias.
    UnknownTarget(String),
    /// A template contains an unknown placeholder.
    UnknownPlaceholder(String),
    /// A template requires an input that was not supplied.
    MissingInput(String),
    /// An archive member path is unsafe.
    InvalidMemberPath(String),
}

impl DistError {
    fn invalid(detail: impl Into<String>) -> Self {
        Self::InvalidInput(bound(detail.into()))
    }
}

impl fmt::Display for DistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(d) => write!(f, "invalid distribution contract: {d}"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported distribution schema version: {found}")
            }
            Self::DuplicateTarget(t) => write!(f, "duplicate distribution target: {t}"),
            Self::DuplicateAlias(a) => write!(f, "duplicate distribution alias: {a}"),
            Self::UnknownTarget(t) => write!(f, "unsupported distribution target: {t}"),
            Self::UnknownPlaceholder(p) => {
                write!(f, "unknown name-template placeholder: {p}")
            }
            Self::MissingInput(m) => write!(f, "missing template input: {m}"),
            Self::InvalidMemberPath(p) => write!(f, "unsafe archive member path: {p}"),
        }
    }
}

impl std::error::Error for DistError {}

fn bound(mut s: String) -> String {
    if s.len() > MAX_DETAIL_LEN {
        s.truncate(MAX_DETAIL_LEN);
    }
    s
}

/// Opaque product identity. Version semantics remain caller-owned.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductIdentity {
    /// Opaque product id (for example `eggsact`). Used for `{product}` expansion.
    pub id: String,
    /// Optional human-readable display name. Never participates in expansion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Checksum sidecar naming. Integrity metadata only; never authenticity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChecksumSpec {
    /// Sidecar file-name template. May use `{product}`, `{version}`,
    /// `{target}`, `{alias}`, and `{asset}` (the expanded asset file name).
    pub sidecar: String,
}

/// One bundle entry: a single asset file plus its install name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleEntry {
    /// Release asset file-name template (flat, no directories).
    pub asset: String,
    /// Install/display file-name template (flat, no directories).
    pub install: String,
}

/// One archive member: a literal relative source path plus its install name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveMember {
    /// Literal relative path inside the archive. No templates; must be
    /// traversal-free (`a/b/c`, never `/abs`, `..`, or empty components).
    pub source: String,
    /// Install/display file-name template (flat, no directories).
    pub install: String,
}

/// Asset layout for one target. A small tagged enum, not a generic package language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetForm {
    /// One release asset corresponds to one member.
    Direct {
        /// Release asset file-name template.
        asset: String,
        /// Install file-name template.
        install: String,
    },
    /// Multiple release assets together form one logical release.
    Bundle {
        /// Each direct asset in the bundle.
        entries: Vec<BundleEntry>,
    },
    /// One release asset contains multiple required members.
    Archive {
        /// Archive file-name template.
        asset: String,
        /// Required members inside the archive.
        members: Vec<ArchiveMember>,
    },
}

/// One supported target entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetEntry {
    /// Canonical Rust target triple.
    pub triple: String,
    /// Zero or more consumer/platform aliases (globally unambiguous).
    pub aliases: Vec<String>,
    /// Asset mapping for this target.
    pub asset: AssetForm,
    /// Checksum sidecar naming for this target.
    pub checksum: ChecksumSpec,
}

/// A versioned distribution contract (schema v1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionContract {
    /// Always [`SCHEMA_V1`].
    pub schema_version: u32,
    /// Opaque product identity.
    pub product: ProductIdentity,
    /// Supported targets (non-empty, deterministic order as written).
    pub targets: Vec<TargetEntry>,
}

// ---- Raw TOML shapes (deny unknown fields for typo safety) ----

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContract {
    schema_version: u32,
    product: ProductIdentity,
    targets: Vec<RawTarget>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTarget {
    triple: String,
    #[serde(default)]
    aliases: Vec<String>,
    asset: RawAsset,
    checksum: ChecksumSpec,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAsset {
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    install: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<BundleEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    members: Option<Vec<ArchiveMember>>,
}

// ---- Parsing and validation ----

impl DistributionContract {
    /// Parses and structurally validates a TOML distribution contract.
    ///
    /// Returns typed validation errors; never guesses, falls back, touches
    /// the network, or rewrites the source. Unknown future schema versions
    /// fail with [`DistError::UnsupportedVersion`].
    pub fn parse_toml_str(s: &str) -> Result<Self, DistError> {
        let raw: RawContract =
            toml::from_str(s).map_err(|e| DistError::invalid(format!("TOML parse: {e}")))?;
        Self::from_raw(raw)
    }

    /// Serializes the contract deterministically to TOML.
    ///
    /// Field and target order are preserved as stored, so golden outputs are
    /// stable for a given contract value.
    pub fn to_toml_string(&self) -> Result<String, DistError> {
        let raw = self.to_raw();
        toml::to_string(&raw).map_err(|e| DistError::invalid(format!("TOML render: {e}")))
    }

    /// Resolves a canonical triple or alias to its target entry.
    ///
    /// Aliases are globally unambiguous; unknown inputs fail with
    /// [`DistError::UnknownTarget`] instead of guessing a nearby architecture.
    pub fn resolve(&self, target_or_alias: &str) -> Result<&TargetEntry, DistError> {
        for t in &self.targets {
            if t.triple == target_or_alias {
                return Ok(t);
            }
            if t.aliases.iter().any(|a| a == target_or_alias) {
                return Ok(t);
            }
        }
        Err(DistError::UnknownTarget(bound(target_or_alias.to_string())))
    }

    /// Expands all file names for one target and opaque version string.
    ///
    /// `version` is opaque (no SemVer ordering) but must be filesystem-safe
    /// (non-empty, no `/`, `\\`, or control characters). `{alias}` templates
    /// require looking up via that alias; looking up via the triple when the
    /// template needs `{alias}` fails with [`DistError::MissingInput`].
    pub fn expand(
        &self,
        target_or_alias: &str,
        version: &str,
    ) -> Result<ExpandedTarget, DistError> {
        validate_version(version)?;
        let target = self.resolve(target_or_alias)?;
        // `{alias}` expands to the alias used for lookup, when the lookup was
        // via an alias. A triple lookup leaves alias empty.
        let alias_used: Option<&str> = if target.triple == target_or_alias {
            None
        } else {
            Some(target_or_alias)
        };
        let ctx = ExpansionCtx {
            product: &self.product.id,
            version,
            target: &target.triple,
            alias: alias_used,
        };
        let assets = match &target.asset {
            AssetForm::Direct { asset, install } => {
                let asset_file = expand_template(asset, &ctx, None)?;
                let install_name = expand_template(install, &ctx, None)?;
                let sidecar_file =
                    expand_template(&target.checksum.sidecar, &ctx, Some(&asset_file))?;
                ExpandedAssets::Direct(ExpandedDirect {
                    asset_file,
                    install_name,
                    sidecar_file,
                })
            }
            AssetForm::Bundle { entries } => {
                let mut out = Vec::with_capacity(entries.len());
                for e in entries {
                    let asset_file = expand_template(&e.asset, &ctx, None)?;
                    let install_name = expand_template(&e.install, &ctx, None)?;
                    let sidecar_file =
                        expand_template(&target.checksum.sidecar, &ctx, Some(&asset_file))?;
                    out.push(ExpandedDirect {
                        asset_file,
                        install_name,
                        sidecar_file,
                    });
                }
                ExpandedAssets::Bundle(ExpandedBundle { entries: out })
            }
            AssetForm::Archive { asset, members } => {
                let archive_file = expand_template(asset, &ctx, None)?;
                let sidecar_file =
                    expand_template(&target.checksum.sidecar, &ctx, Some(&archive_file))?;
                let mut expanded_members = Vec::with_capacity(members.len());
                for m in members {
                    let install_name = expand_template(&m.install, &ctx, None)?;
                    expanded_members.push(ExpandedArchiveMember {
                        source: m.source.clone(),
                        install_name,
                    });
                }
                ExpandedAssets::Archive(ExpandedArchive {
                    archive_file,
                    sidecar_file,
                    members: expanded_members,
                })
            }
        };
        Ok(ExpandedTarget {
            triple: target.triple.clone(),
            assets,
        })
    }

    fn from_raw(raw: RawContract) -> Result<Self, DistError> {
        if raw.schema_version != SCHEMA_V1 {
            return Err(DistError::UnsupportedVersion {
                found: raw.schema_version,
            });
        }
        validate_product_id(&raw.product.id)?;
        if let Some(display) = &raw.product.display_name {
            validate_display_name(display)?;
        }
        if raw.targets.is_empty() {
            return Err(DistError::invalid("contract declares no targets"));
        }
        let all_triples: HashSet<&str> = raw.targets.iter().map(|t| t.triple.as_str()).collect();
        // Duplicate triples fail without guessing.
        {
            let mut seen: HashSet<&str> = HashSet::new();
            for t in &raw.targets {
                if !seen.insert(t.triple.as_str()) {
                    return Err(DistError::DuplicateTarget(bound(t.triple.clone())));
                }
            }
        }
        // Aliases must be valid, must not collide with any triple (even their
        // own) to keep triple-vs-alias resolution unambiguous, and must not
        // map one alias to different targets.
        let mut alias_owner: HashMap<&str, &str> = HashMap::new();
        for t in &raw.targets {
            validate_triple(&t.triple)?;
            for a in &t.aliases {
                validate_alias(a)?;
                if all_triples.contains(a.as_str()) {
                    return Err(DistError::DuplicateAlias(bound(format!(
                        "{a} collides with a target triple"
                    ))));
                }
                if let Some(owner) = alias_owner.get(a.as_str()) {
                    if *owner != t.triple.as_str() {
                        return Err(DistError::DuplicateAlias(bound(a.clone())));
                    }
                } else {
                    alias_owner.insert(a.as_str(), t.triple.as_str());
                }
            }
        }
        let mut out_targets = Vec::with_capacity(raw.targets.len());
        for t in raw.targets {
            let asset = convert_asset(t.asset)?;
            validate_checksum_template(&asset, &t.checksum.sidecar)?;
            out_targets.push(TargetEntry {
                triple: t.triple,
                aliases: t.aliases,
                asset,
                checksum: t.checksum,
            });
        }
        Ok(Self {
            schema_version: SCHEMA_V1,
            product: raw.product,
            targets: out_targets,
        })
    }

    fn to_raw(&self) -> RawContract {
        let targets = self
            .targets
            .iter()
            .map(|t| {
                let (kind, asset, install, entries, members) = match &t.asset {
                    AssetForm::Direct { asset, install } => (
                        "direct".to_string(),
                        Some(asset.clone()),
                        Some(install.clone()),
                        None,
                        None,
                    ),
                    AssetForm::Bundle { entries } => (
                        "bundle".to_string(),
                        None,
                        None,
                        Some(entries.clone()),
                        None,
                    ),
                    AssetForm::Archive { asset, members } => (
                        "archive".to_string(),
                        Some(asset.clone()),
                        None,
                        None,
                        Some(members.clone()),
                    ),
                };
                RawTarget {
                    triple: t.triple.clone(),
                    aliases: t.aliases.clone(),
                    asset: RawAsset {
                        kind,
                        asset,
                        install,
                        entries,
                        members,
                    },
                    checksum: t.checksum.clone(),
                }
            })
            .collect();
        RawContract {
            schema_version: self.schema_version,
            product: self.product.clone(),
            targets,
        }
    }
}

// ---- Asset conversion ----

fn convert_asset(raw: RawAsset) -> Result<AssetForm, DistError> {
    match raw.kind.as_str() {
        "direct" => {
            let asset = raw
                .asset
                .ok_or_else(|| DistError::invalid("direct asset requires `asset`"))?;
            let install = raw
                .install
                .ok_or_else(|| DistError::invalid("direct asset requires `install`"))?;
            if raw.entries.is_some() || raw.members.is_some() {
                return Err(DistError::invalid(
                    "direct asset must not declare `entries` or `members`",
                ));
            }
            validate_file_template(&asset, false)?;
            validate_file_template(&install, false)?;
            Ok(AssetForm::Direct { asset, install })
        }
        "bundle" => {
            let entries = raw
                .entries
                .ok_or_else(|| DistError::invalid("bundle asset requires `entries`"))?;
            if raw.asset.is_some() || raw.install.is_some() || raw.members.is_some() {
                return Err(DistError::invalid(
                    "bundle asset must declare only `entries`",
                ));
            }
            if entries.is_empty() {
                return Err(DistError::invalid("bundle asset declares no entries"));
            }
            if entries.len() > 64 {
                return Err(DistError::invalid("bundle asset declares too many entries"));
            }
            for e in &entries {
                validate_file_template(&e.asset, false)?;
                validate_file_template(&e.install, false)?;
            }
            Ok(AssetForm::Bundle { entries })
        }
        "archive" => {
            let asset = raw
                .asset
                .ok_or_else(|| DistError::invalid("archive asset requires `asset`"))?;
            let members = raw
                .members
                .ok_or_else(|| DistError::invalid("archive asset requires `members`"))?;
            if raw.install.is_some() || raw.entries.is_some() {
                return Err(DistError::invalid(
                    "archive asset must declare only `asset` and `members`",
                ));
            }
            validate_file_template(&asset, false)?;
            if members.is_empty() {
                return Err(DistError::invalid("archive asset declares no members"));
            }
            if members.len() > 64 {
                return Err(DistError::invalid(
                    "archive asset declares too many members",
                ));
            }
            let mut seen: HashSet<&str> = HashSet::new();
            for m in &members {
                validate_member_source(&m.source)?;
                validate_file_template(&m.install, false)?;
                if !seen.insert(m.source.as_str()) {
                    return Err(DistError::invalid(format!(
                        "duplicate archive member: {}",
                        m.source
                    )));
                }
            }
            Ok(AssetForm::Archive { asset, members })
        }
        other => Err(DistError::invalid(format!(
            "unknown asset kind: {other} (expected direct|bundle|archive)"
        ))),
    }
}

// ---- Validation helpers ----

fn is_safe_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
}

fn validate_product_id(id: &str) -> Result<(), DistError> {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err(DistError::invalid("product id is empty or overlong"));
    }
    if id.chars().any(|c| c.is_control()) {
        return Err(DistError::invalid("product id has control characters"));
    }
    if !id.chars().all(is_safe_name_char) {
        return Err(DistError::invalid(
            "product id must use [A-Za-z0-9-_.] only",
        ));
    }
    if id == "." || id == ".." {
        return Err(DistError::invalid("product id must name a product"));
    }
    Ok(())
}

fn validate_display_name(name: &str) -> Result<(), DistError> {
    if name.is_empty() || name.len() > 256 {
        return Err(DistError::invalid("display name is empty or overlong"));
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(DistError::invalid("display name has control characters"));
    }
    Ok(())
}

fn validate_triple(triple: &str) -> Result<(), DistError> {
    if triple.is_empty() || triple.len() > MAX_NAME_LEN {
        return Err(DistError::invalid("target triple is empty or overlong"));
    }
    if triple.chars().any(|c| c.is_control()) {
        return Err(DistError::invalid("target triple has control characters"));
    }
    if triple.contains(['/', '\\', ' ', '@', '?', '#', ':']) {
        return Err(DistError::invalid(
            "target triple must be a bare triple without separators",
        ));
    }
    if !triple.chars().all(is_safe_name_char) {
        return Err(DistError::invalid(
            "target triple must use [A-Za-z0-9-_.] only",
        ));
    }
    if !triple.contains('-') {
        return Err(DistError::invalid(
            "target triple must contain '-' (canonical Rust triple)",
        ));
    }
    Ok(())
}

fn validate_alias(alias: &str) -> Result<(), DistError> {
    if alias.is_empty() || alias.len() > MAX_NAME_LEN {
        return Err(DistError::invalid("alias is empty or overlong"));
    }
    if alias.chars().any(|c| c.is_control()) {
        return Err(DistError::invalid("alias has control characters"));
    }
    if alias.contains(['/', '\\', ' ', '@', '?', '#', ':']) {
        return Err(DistError::invalid("alias must not contain separators"));
    }
    if !alias.chars().all(is_safe_name_char) {
        return Err(DistError::invalid("alias must use [A-Za-z0-9-_.] only"));
    }
    Ok(())
}

fn validate_version(version: &str) -> Result<(), DistError> {
    if version.is_empty() || version.len() > MAX_NAME_LEN {
        return Err(DistError::invalid(
            "version is empty or overlong (opaque but filesystem-safe)",
        ));
    }
    if version.chars().any(|c| c.is_control()) {
        return Err(DistError::invalid("version has control characters"));
    }
    if version.contains(['/', '\\']) {
        return Err(DistError::invalid(
            "version must not contain path separators",
        ));
    }
    Ok(())
}

/// Validates a flat file-name template (asset, install, or sidecar).
///
/// `allow_asset` permits the `{asset}` placeholder (checksum sidecars only).
/// Rejects path separators, control characters, and unknown placeholders.
/// Archive member sources are literals and validated separately.
fn validate_file_template(template: &str, allow_asset: bool) -> Result<(), DistError> {
    if template.is_empty() || template.len() > MAX_TEMPLATE_LEN {
        return Err(DistError::invalid("name template is empty or overlong"));
    }
    if template.chars().any(|c| c.is_control()) {
        return Err(DistError::invalid("name template has control characters"));
    }
    if template.contains(['/', '\\']) {
        return Err(DistError::invalid(
            "name template must be a flat file name without directories",
        ));
    }
    for ph in placeholders_in(template) {
        match ph.as_str() {
            "product" | "version" | "target" | "alias" => {}
            "asset" if allow_asset => {}
            _ => return Err(DistError::UnknownPlaceholder(bound(ph))),
        }
    }
    Ok(())
}

fn validate_checksum_template(asset: &AssetForm, sidecar: &str) -> Result<(), DistError> {
    let _ = asset;
    validate_file_template(sidecar, true)
}

/// Validates a literal archive member source path (no templates).
fn validate_member_source(source: &str) -> Result<(), DistError> {
    if source.is_empty() || source.len() > MAX_TEMPLATE_LEN {
        return Err(DistError::InvalidMemberPath(bound(
            "member path is empty or overlong".to_string(),
        )));
    }
    if source.chars().any(|c| c.is_control()) {
        return Err(DistError::InvalidMemberPath(bound(
            "member path has control characters".to_string(),
        )));
    }
    if source.contains('{') || source.contains('}') {
        return Err(DistError::InvalidMemberPath(bound(
            "member source must be literal (no templates)".to_string(),
        )));
    }
    if source.contains('\\') {
        return Err(DistError::InvalidMemberPath(bound(
            "member path must use '/' separators".to_string(),
        )));
    }
    if source.starts_with('/') {
        return Err(DistError::InvalidMemberPath(bound(format!(
            "absolute member path: {source}"
        ))));
    }
    if source.contains(':') {
        return Err(DistError::InvalidMemberPath(bound(format!(
            "member path must not contain drive prefixes: {source}"
        ))));
    }
    for comp in source.split('/') {
        if comp.is_empty() {
            return Err(DistError::InvalidMemberPath(bound(format!(
                "empty member path component: {source}"
            ))));
        }
        if comp == "." || comp == ".." {
            return Err(DistError::InvalidMemberPath(bound(format!(
                "member path escapes root: {source}"
            ))));
        }
    }
    Ok(())
}

// ---- Template expansion ----

struct ExpansionCtx<'a> {
    product: &'a str,
    version: &'a str,
    target: &'a str,
    alias: Option<&'a str>,
}

fn placeholders_in(template: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(end) = template[i..].find('}') {
                let name = template[i + 1..i + end].to_string();
                if !name.is_empty() {
                    out.push(name);
                }
                i += end + 1;
                continue;
            }
            break;
        }
        i += 1;
    }
    out
}

fn expand_template(
    template: &str,
    ctx: &ExpansionCtx<'_>,
    asset: Option<&str>,
) -> Result<String, DistError> {
    let mut out = template.to_string();
    // Validate placeholders first so typos fail even when an input is missing.
    for ph in placeholders_in(template) {
        match ph.as_str() {
            "product" | "version" | "target" | "alias" => {}
            "asset" if asset.is_some() => {}
            "asset" => {
                return Err(DistError::MissingInput(bound(
                    "`{asset}` requires a checksum context".to_string(),
                )));
            }
            _ => return Err(DistError::UnknownPlaceholder(bound(ph))),
        }
    }
    out = out.replace("{product}", ctx.product);
    out = out.replace("{version}", ctx.version);
    out = out.replace("{target}", ctx.target);
    if let Some(alias) = ctx.alias {
        out = out.replace("{alias}", alias);
    } else if template.contains("{alias}") {
        return Err(DistError::MissingInput(bound(
            "`{alias}` requires lookup via that alias".to_string(),
        )));
    }
    if let Some(asset) = asset {
        out = out.replace("{asset}", asset);
    }
    if out.contains('{') || out.contains('}') {
        return Err(DistError::invalid("unexpanded placeholder remains"));
    }
    if out.is_empty() || out.len() > MAX_TEMPLATE_LEN {
        return Err(DistError::invalid("expanded name is empty or overlong"));
    }
    if out.contains(['/', '\\']) {
        return Err(DistError::invalid(
            "expanded name must remain a flat file name",
        ));
    }
    Ok(out)
}

// ---- Expanded model ----

/// One expanded direct asset (file, install name, and sidecar).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedDirect {
    /// Expanded release asset file name.
    pub asset_file: String,
    /// Expanded install/display file name.
    pub install_name: String,
    /// Expanded checksum sidecar file name.
    pub sidecar_file: String,
}

/// Expanded multi-asset bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedBundle {
    /// Each entry in the bundle.
    pub entries: Vec<ExpandedDirect>,
}

/// One expanded archive member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedArchiveMember {
    /// Literal source path inside the archive.
    pub source: String,
    /// Expanded install/display file name.
    pub install_name: String,
}

/// Expanded archive asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedArchive {
    /// Expanded archive file name.
    pub archive_file: String,
    /// Expanded checksum sidecar file name.
    pub sidecar_file: String,
    /// Required members.
    pub members: Vec<ExpandedArchiveMember>,
}

/// Expanded asset forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandedAssets {
    /// Single direct asset.
    Direct(ExpandedDirect),
    /// Multi-asset bundle.
    Bundle(ExpandedBundle),
    /// Archive with members.
    Archive(ExpandedArchive),
}

/// A resolved and expanded target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedTarget {
    /// Canonical triple.
    pub triple: String,
    /// Expanded file names.
    pub assets: ExpandedAssets,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE: &str = r#"
schema_version = 1

[product]
id = "eggsact"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
aliases = ["linux-x64"]

[targets.asset]
kind = "direct"
asset = "{product}-{version}-{target}"
install = "{product}"

[targets.checksum]
sidecar = "{asset}.sha256"
"#;

    const BUNDLE: &str = r#"
schema_version = 1

[product]
id = "codegg"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
aliases = []

[targets.asset]
kind = "bundle"

[[targets.asset.entries]]
asset = "{product}-{version}-{target}"
install = "{product}"

[[targets.asset.entries]]
asset = "{product}-helper-{version}-{target}"
install = "{product}-helper"

[[targets.asset.entries]]
asset = "{product}-manifest-{version}.json"
install = "{product}-manifest.json"

[targets.checksum]
sidecar = "{asset}.sha256"
"#;

    const ARCHIVE: &str = r#"
schema_version = 1

[product]
id = "egress"

[[targets]]
triple = "aarch64-apple-darwin"
aliases = ["macos-arm64"]

[targets.asset]
kind = "archive"
asset = "{product}-{version}-{target}.tar.gz"

[[targets.asset.members]]
source = "egress"
install = "egress"

[[targets.asset.members]]
source = "bin/egress-helper"
install = "egress-helper"

[targets.checksum]
sidecar = "{asset}.sha256"
"#;

    #[test]
    fn parse_minimal_valid_v1() {
        let c = DistributionContract::parse_toml_str(SIMPLE).unwrap();
        assert_eq!(c.schema_version, 1);
        assert_eq!(c.product.id, "eggsact");
        assert_eq!(c.targets.len(), 1);
    }

    #[test]
    fn unknown_schema_version_fails_typed() {
        let s = SIMPLE.replace("schema_version = 1", "schema_version = 2");
        let err = DistributionContract::parse_toml_str(&s).unwrap_err();
        assert!(matches!(err, DistError::UnsupportedVersion { found: 2 }));
    }

    #[test]
    fn duplicate_target_is_rejected() {
        let s = format!("{SIMPLE}\n[[targets]]\ntriple = \"x86_64-unknown-linux-gnu\"\n[targets.asset]\nkind = \"direct\"\nasset = \"a\"\ninstall = \"b\"\n[targets.checksum]\nsidecar = \"{{asset}}.sha256\"\n");
        let err = DistributionContract::parse_toml_str(&s).unwrap_err();
        assert!(matches!(err, DistError::DuplicateTarget(_)));
    }

    #[test]
    fn duplicate_alias_is_rejected() {
        let s = r#"
schema_version = 1
[product]
id = "p"
[[targets]]
triple = "x86_64-unknown-linux-gnu"
aliases = ["same"]
[targets.asset]
kind = "direct"
asset = "a"
install = "b"
[targets.checksum]
sidecar = "{asset}.sha256"
[[targets]]
triple = "aarch64-apple-darwin"
aliases = ["same"]
[targets.asset]
kind = "direct"
asset = "a"
install = "b"
[targets.checksum]
sidecar = "{asset}.sha256"
"#;
        let err = DistributionContract::parse_toml_str(s).unwrap_err();
        assert!(matches!(err, DistError::DuplicateAlias(_)));
    }

    #[test]
    fn alias_colliding_with_triple_is_rejected() {
        let s = r#"
schema_version = 1
[product]
id = "p"
[[targets]]
triple = "x86_64-unknown-linux-gnu"
aliases = ["aarch64-apple-darwin"]
[targets.asset]
kind = "direct"
asset = "a"
install = "b"
[targets.checksum]
sidecar = "{asset}.sha256"
[[targets]]
triple = "aarch64-apple-darwin"
aliases = []
[targets.asset]
kind = "direct"
asset = "a"
install = "b"
[targets.checksum]
sidecar = "{asset}.sha256"
"#;
        let err = DistributionContract::parse_toml_str(s).unwrap_err();
        assert!(matches!(err, DistError::DuplicateAlias(_)));
    }

    #[test]
    fn unsupported_target_lookup_fails_without_guessing() {
        let c = DistributionContract::parse_toml_str(SIMPLE).unwrap();
        let err = c.resolve("riscv64-unknown-linux-gnu").unwrap_err();
        assert!(matches!(err, DistError::UnknownTarget(_)));
        // Alias lookup works; wrong alias does not guess.
        assert!(c.resolve("linux-x64").is_ok());
        assert!(c.resolve("linux-arm64").is_err());
    }

    #[test]
    fn unknown_placeholder_is_rejected() {
        let s = SIMPLE.replace("{product}-{version}-{target}", "{product}-{env}-x");
        let err = DistributionContract::parse_toml_str(&s).unwrap_err();
        assert!(matches!(err, DistError::UnknownPlaceholder(_)));
    }

    #[test]
    fn missing_alias_input_fails() {
        let c = DistributionContract::parse_toml_str(
            &SIMPLE.replace("install = \"{product}\"", "install = \"{alias}\""),
        )
        .unwrap();
        // Lookup via triple leaves `{alias}` without input.
        let err = c.expand("x86_64-unknown-linux-gnu", "1.0.0").unwrap_err();
        assert!(matches!(err, DistError::MissingInput(_)));
        // Lookup via the alias supplies it.
        let ok = c.expand("linux-x64", "1.0.0").unwrap();
        match ok.assets {
            ExpandedAssets::Direct(d) => assert_eq!(d.install_name, "linux-x64"),
            _ => panic!("expected direct"),
        }
    }

    #[test]
    fn direct_single_member_expansion() {
        let c = DistributionContract::parse_toml_str(SIMPLE).unwrap();
        let e = c.expand("x86_64-unknown-linux-gnu", "1.2.6").unwrap();
        match e.assets {
            ExpandedAssets::Direct(d) => {
                assert_eq!(d.asset_file, "eggsact-1.2.6-x86_64-unknown-linux-gnu");
                assert_eq!(d.install_name, "eggsact");
                assert_eq!(
                    d.sidecar_file,
                    "eggsact-1.2.6-x86_64-unknown-linux-gnu.sha256"
                );
            }
            _ => panic!("expected direct"),
        }
    }

    #[test]
    fn multi_direct_bundle_expansion() {
        let c = DistributionContract::parse_toml_str(BUNDLE).unwrap();
        let e = c.expand("x86_64-unknown-linux-gnu", "0.9.0").unwrap();
        match e.assets {
            ExpandedAssets::Bundle(b) => {
                assert_eq!(b.entries.len(), 3);
                assert_eq!(
                    b.entries[0].asset_file,
                    "codegg-0.9.0-x86_64-unknown-linux-gnu"
                );
                assert_eq!(b.entries[1].install_name, "codegg-helper");
                assert_eq!(b.entries[2].asset_file, "codegg-manifest-0.9.0.json");
                assert!(b.entries[0].sidecar_file.ends_with(".sha256"));
            }
            _ => panic!("expected bundle"),
        }
    }

    #[test]
    fn archive_asset_expansion() {
        let c = DistributionContract::parse_toml_str(ARCHIVE).unwrap();
        let e = c.expand("macos-arm64", "2.1.0").unwrap();
        assert_eq!(e.triple, "aarch64-apple-darwin");
        match e.assets {
            ExpandedAssets::Archive(a) => {
                assert_eq!(a.archive_file, "egress-2.1.0-aarch64-apple-darwin.tar.gz");
                assert_eq!(
                    a.sidecar_file,
                    "egress-2.1.0-aarch64-apple-darwin.tar.gz.sha256"
                );
                assert_eq!(a.members.len(), 2);
                assert_eq!(a.members[0].source, "egress");
                assert_eq!(a.members[1].source, "bin/egress-helper");
            }
            _ => panic!("expected archive"),
        }
    }

    #[test]
    fn archive_member_absolute_path_is_rejected() {
        let s = ARCHIVE.replace("source = \"egress\"", "source = \"/egress\"");
        let err = DistributionContract::parse_toml_str(&s).unwrap_err();
        assert!(matches!(err, DistError::InvalidMemberPath(_)));
    }

    #[test]
    fn archive_member_traversal_is_rejected() {
        for bad in ["../evil", "a/../../b", "a/../b", "bin/..", ".", ".."] {
            let s = ARCHIVE.replace("source = \"egress\"", &format!("source = \"{bad}\""));
            let err = DistributionContract::parse_toml_str(&s).unwrap_err();
            assert!(matches!(err, DistError::InvalidMemberPath(_)), "{bad}");
        }
    }

    #[test]
    fn duplicate_normalized_archive_members_rejected() {
        let s = ARCHIVE.replace("source = \"bin/egress-helper\"", "source = \"egress\"");
        let err = DistributionContract::parse_toml_str(&s).unwrap_err();
        assert!(matches!(err, DistError::InvalidInput(_)));
    }

    #[test]
    fn checksum_sidecar_naming_is_explicit() {
        let c = DistributionContract::parse_toml_str(SIMPLE).unwrap();
        let e = c.expand("linux-x64", "1.0.0").unwrap();
        match e.assets {
            ExpandedAssets::Direct(d) => {
                assert!(d.sidecar_file.contains(&d.asset_file));
                assert!(!d.sidecar_file.contains("signature"));
            }
            _ => panic!("expected direct"),
        }
    }

    #[test]
    fn product_version_is_opaque_but_filesystem_safe() {
        let c = DistributionContract::parse_toml_str(SIMPLE).unwrap();
        // Opaque: dotted versions pass through unchanged.
        let e = c.expand("x86_64-unknown-linux-gnu", "1.2.6-rc.1").unwrap();
        match e.assets {
            ExpandedAssets::Direct(d) => assert!(d.asset_file.contains("1.2.6-rc.1")),
            _ => panic!("expected direct"),
        }
        // Unsafe versions fail instead of escaping.
        assert!(c.expand("x86_64-unknown-linux-gnu", "../evil").is_err());
        assert!(c.expand("x86_64-unknown-linux-gnu", "a/b").is_err());
        assert!(c.expand("x86_64-unknown-linux-gnu", "").is_err());
    }

    #[test]
    fn deterministic_round_trip_golden() {
        for fixture in [SIMPLE, BUNDLE, ARCHIVE] {
            let c = DistributionContract::parse_toml_str(fixture).unwrap();
            let rendered = c.to_toml_string().unwrap();
            let reparsed = DistributionContract::parse_toml_str(&rendered).unwrap();
            assert_eq!(c, reparsed);
            // Golden stability: rendering twice yields identical bytes.
            assert_eq!(rendered, reparsed.to_toml_string().unwrap());
        }
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let s = SIMPLE.replace(
            "[product]\nid = \"eggsact\"",
            "[product]\nid = \"eggsact\"\ntypo_field = 1",
        );
        assert!(DistributionContract::parse_toml_str(&s).is_err());
    }
}
