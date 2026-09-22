use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::domain::{IntegrityRequirement, MemberId};
use crate::error::{Error, Result};
use crate::stage::PreparedTransaction;

/// A parsed single-entry SHA-256 sidecar or manifest line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256Manifest {
    digest: [u8; 32],
    filename: Option<String>,
}

impl Sha256Manifest {
    /// Returns the declared digest bytes.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Returns the exact optional filename binding.
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }
}

/// The result of applying a member's integrity declaration to staged bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityStatus {
    /// The declared SHA-256 digest matched the staged bytes.
    Verified,
    /// The member explicitly declared that no integrity evidence is required.
    NotRequired,
}

/// One member's bounded integrity result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityResult {
    member: MemberId,
    digest: [u8; 32],
    status: IntegrityStatus,
}

impl IntegrityResult {
    /// Returns the member identity.
    pub fn member(&self) -> &MemberId {
        &self.member
    }

    /// Returns the computed staged-file digest.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Returns whether the declared integrity requirement passed or was absent.
    pub fn status(&self) -> IntegrityStatus {
        self.status
    }
}

/// Streams a local file through SHA-256 without buffering the complete artifact.
pub fn hash_file(path: &Path) -> Result<[u8; 32]> {
    let mut file =
        File::open(path).map_err(|source| Error::io("opening file for hashing", source))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|source| Error::io("reading file for hashing", source))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().into())
}

/// Parses exactly one strict SHA-256 sidecar line.
pub fn parse_sha256_sidecar(input: &str) -> Result<Sha256Manifest> {
    let lines = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.len() != 1 {
        return Err(Error::VerificationFailed(
            "SHA-256 sidecar must contain exactly one non-empty entry".into(),
        ));
    }
    let mut fields = lines[0].splitn(2, char::is_whitespace);
    let digest_text = fields.next().unwrap_or_default();
    let digest = decode_digest(digest_text)?;
    let filename = fields
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let filename = filename.map(|value| value.strip_prefix('*').unwrap_or(value).to_owned());
    if filename.as_deref().is_some_and(|value| {
        value.is_empty()
            || value.chars().any(char::is_control)
            || Path::new(value)
                .file_name()
                .is_none_or(|name| name != value)
    }) {
        return Err(Error::VerificationFailed(
            "sidecar filename must be one exact non-control filename".into(),
        ));
    }
    Ok(Sha256Manifest { digest, filename })
}

/// Verifies a local file against a parsed SHA-256 declaration.
pub fn verify_file(path: &Path, manifest: &Sha256Manifest) -> Result<[u8; 32]> {
    if let Some(expected_name) = manifest.filename() {
        let actual_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::VerificationFailed("candidate has no valid filename".into()))?;
        if actual_name != expected_name {
            return Err(Error::VerificationFailed(format!(
                "sidecar filename {expected_name:?} does not bind to {actual_name:?}"
            )));
        }
    }
    let digest = hash_file(path)?;
    if &digest != manifest.digest() {
        return Err(Error::VerificationFailed(format!(
            "SHA-256 mismatch for {}",
            path.display()
        )));
    }
    Ok(digest)
}

fn decode_digest(text: &str) -> Result<[u8; 32]> {
    if text.len() != 64 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::VerificationFailed(
            "SHA-256 digest must contain exactly 64 hexadecimal characters".into(),
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, slot) in digest.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = (hex_value(text.as_bytes()[offset]) << 4) | hex_value(text.as_bytes()[offset + 1]);
    }
    Ok(digest)
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => unreachable!("validated hexadecimal input"),
    }
}

/// A prepared transaction whose declared integrity requirements have passed.
#[derive(Debug)]
pub struct VerifiedTransaction {
    pub(crate) prepared: PreparedTransaction,
    results: Vec<IntegrityResult>,
}

impl PreparedTransaction {
    /// Applies every member's declared integrity requirement before candidate validation.
    pub fn verify_integrity(self) -> Result<VerifiedTransaction> {
        let members = self.artifacts().iter().cloned().collect::<Vec<_>>();
        let mut results = Vec::with_capacity(members.len());
        for member in members {
            let path = self.staged_path(member.id())?;
            let metadata = fs::symlink_metadata(&path)
                .map_err(|source| Error::io("reading staged candidate", source))?;
            if !metadata.is_file() {
                return Err(Error::VerificationFailed(format!(
                    "staged member {} is not a regular file",
                    member.id()
                )));
            }
            let digest = hash_file(&path)?;
            let status = match member.integrity() {
                IntegrityRequirement::None => IntegrityStatus::NotRequired,
                IntegrityRequirement::Sha256(expected) if expected == digest => {
                    IntegrityStatus::Verified
                }
                IntegrityRequirement::Sha256(_) => {
                    return Err(Error::VerificationFailed(format!(
                        "SHA-256 mismatch for member {}",
                        member.id()
                    )));
                }
            };
            results.push(IntegrityResult {
                member: member.id().clone(),
                digest,
                status,
            });
        }
        Ok(VerifiedTransaction {
            prepared: self,
            results,
        })
    }
}

impl VerifiedTransaction {
    /// Returns the validated artifact set.
    pub fn artifacts(&self) -> &crate::domain::ArtifactSet {
        self.prepared.artifacts()
    }

    /// Returns the product identity.
    pub fn product(&self) -> &crate::domain::ProductId {
        self.prepared.product()
    }

    /// Returns the release identity.
    pub fn release(&self) -> &crate::domain::ReleaseId {
        self.prepared.release()
    }

    /// Returns the private stage root for bounded candidate execution.
    pub fn stage_root(&self) -> &Path {
        self.prepared.stage_root()
    }

    /// Returns a verified member's private staged path.
    pub fn staged_path(&self, member: &MemberId) -> Result<PathBuf> {
        self.prepared.staged_path(member)
    }

    /// Returns the integrity result for one member.
    pub fn integrity(&self, member: &MemberId) -> Option<&IntegrityResult> {
        self.results.iter().find(|result| result.member() == member)
    }

    pub(crate) fn verified_digests(&self) -> std::collections::HashMap<MemberId, [u8; 32]> {
        self.results
            .iter()
            .filter(|r| r.status() == IntegrityStatus::Verified)
            .map(|r| (r.member().clone(), *r.digest()))
            .collect()
    }
}
