use std::fmt::Write as _;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::{CommitOwnership, MemberId};
use crate::error::{Error, Result};
use crate::integrity::{IntegrityStatus, VerifiedTransaction};
use crate::transaction::{PostCommitFailurePolicy, TransactionReceipt};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_OUTPUT_LIMIT: usize = 64 * 1024;

/// An argv-based candidate command with bounded execution policy.
#[derive(Debug, Clone)]
pub struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
    timeout: Duration,
    max_output_bytes: usize,
    environment: Vec<(String, String)>,
}

impl CommandSpec {
    /// Creates a command that never invokes a shell.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            timeout: DEFAULT_TIMEOUT,
            max_output_bytes: DEFAULT_OUTPUT_LIMIT,
            environment: Vec::new(),
        }
    }

    /// Appends one literal argv argument.
    pub fn arg(mut self, argument: impl Into<String>) -> Self {
        self.args.push(argument.into());
        self
    }

    /// Appends literal argv arguments.
    pub fn args<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Sets the controlled child working directory.
    pub fn current_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(path.into());
        self
    }

    /// Sets the maximum wall-clock execution time.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the combined per-stream output bound.
    pub fn max_output_bytes(mut self, limit: usize) -> Self {
        self.max_output_bytes = limit;
        self
    }

    /// Adds one explicit environment variable; inherited environment is cleared.
    pub fn environment(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.push((name.into(), value.into()));
        self
    }
}

/// A bounded child-process result with timeout and overflow state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
    output_limited: bool,
}

impl CommandOutput {
    /// Returns the platform exit code when one was available.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Returns bounded stdout bytes.
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    /// Returns bounded stderr bytes.
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    /// Returns whether the child was killed because its deadline elapsed.
    pub fn timed_out(&self) -> bool {
        self.timed_out
    }

    /// Returns whether the child was killed because an output stream exceeded its bound.
    pub fn output_limited(&self) -> bool {
        self.output_limited
    }

    /// Returns whether the child exited successfully without a timeout or overflow.
    pub fn success(&self) -> bool {
        !self.timed_out && !self.output_limited && self.exit_code == Some(0)
    }
}

/// Executes a command with cleared environment, null stdin, bounded output, and kill/reap.
pub fn run_bounded(spec: &CommandSpec) -> Result<CommandOutput> {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(current_dir) = &spec.current_dir {
        command.current_dir(current_dir);
    }
    for (name, value) in &spec.environment {
        command.env(name, value);
    }
    let mut child = command
        .spawn()
        .map_err(|source| Error::io("spawning bounded candidate", source))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::CandidateExecution("missing stdout pipe".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::CandidateExecution("missing stderr pipe".into()))?;
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();
    let limit = spec.max_output_bytes;
    let stdout_thread = thread::spawn(move || {
        let _ = stdout_tx.send(read_limited(stdout, limit));
    });
    let stderr_thread = thread::spawn(move || {
        let _ = stderr_tx.send(read_limited(stderr, limit));
    });

    let deadline = Instant::now() + spec.timeout;
    let mut timed_out = false;
    let mut output_limited = false;
    let mut status = None;
    let mut stdout_result = None;
    let mut stderr_result = None;
    loop {
        if stdout_result.is_none() {
            if let Ok(result) = stdout_rx.try_recv() {
                output_limited |= result.is_err();
                stdout_result = Some(result.unwrap_or_default());
            }
        }
        if stderr_result.is_none() {
            if let Ok(result) = stderr_rx.try_recv() {
                output_limited |= result.is_err();
                stderr_result = Some(result.unwrap_or_default());
            }
        }
        if status.is_none() {
            status = child
                .try_wait()
                .map_err(|source| Error::io("polling bounded candidate", source))?;
        }
        if output_limited {
            let _ = child.kill();
            status = child.wait().ok();
        } else if status.is_none() && Instant::now() >= deadline {
            timed_out = true;
            let _ = child.kill();
            status = child.wait().ok();
        }
        if ((timed_out || output_limited) && status.is_some())
            || (status.is_some() && stdout_result.is_some() && stderr_result.is_some())
        {
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    if !timed_out && !output_limited {
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();
    }
    Ok(CommandOutput {
        exit_code: status.and_then(|status| status.code()),
        stdout: stdout_result.unwrap_or_default(),
        stderr: stderr_result.unwrap_or_default(),
        timed_out,
        output_limited,
    })
}

fn read_limited(mut reader: impl Read, limit: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len().saturating_add(count) > limit {
            return Err(io::Error::other("candidate output limit exceeded"));
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

/// A candidate validator supplied by a consumer or by eggup-core helpers.
pub trait CandidateValidator {
    /// Validates all candidates in an integrity-verified transaction.
    fn validate(&self, transaction: &VerifiedTransaction) -> Result<()>;
}

/// A strict exact-output check for one or more staged member programs.
#[derive(Debug, Clone)]
pub struct ExactIdentityValidator {
    checks: Vec<IdentityCheck>,
    timeout: Duration,
    max_output_bytes: usize,
}

#[derive(Debug, Clone)]
struct IdentityCheck {
    member: MemberId,
    args: Vec<String>,
    expected_output: String,
}

impl ExactIdentityValidator {
    /// Creates a check for one member's exact stdout bytes.
    pub fn new(member: MemberId, expected_output: impl Into<String>) -> Self {
        Self {
            checks: vec![IdentityCheck {
                member,
                args: Vec::new(),
                expected_output: expected_output.into(),
            }],
            timeout: DEFAULT_TIMEOUT,
            max_output_bytes: DEFAULT_OUTPUT_LIMIT,
        }
    }

    /// Creates matching exact-output checks for a bundle of members.
    pub fn for_members<I>(members: I, expected_output: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = MemberId>,
    {
        let expected_output = expected_output.into();
        let checks = members
            .into_iter()
            .map(|member| IdentityCheck {
                member,
                args: Vec::new(),
                expected_output: expected_output.clone(),
            })
            .collect();
        Self {
            checks,
            timeout: DEFAULT_TIMEOUT,
            max_output_bytes: DEFAULT_OUTPUT_LIMIT,
        }
    }

    /// Sets literal argv arguments for every configured identity check.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        for check in &mut self.checks {
            check.args = args.clone();
        }
        self
    }

    /// Sets the per-candidate deadline.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the per-stream output cap.
    pub fn max_output_bytes(mut self, limit: usize) -> Self {
        self.max_output_bytes = limit;
        self
    }
}

impl CandidateValidator for ExactIdentityValidator {
    fn validate(&self, transaction: &VerifiedTransaction) -> Result<()> {
        for check in &self.checks {
            let program = transaction.staged_path(&check.member)?;
            let output = run_bounded(
                &CommandSpec::new(program)
                    .args(check.args.clone())
                    .current_dir(transaction.stage_root())
                    .timeout(self.timeout)
                    .max_output_bytes(self.max_output_bytes),
            )?;
            if !output.success()
                || !output.stderr().is_empty()
                || output.stdout() != check.expected_output.as_bytes()
            {
                return Err(Error::CandidateExecution(format!(
                    "member {} did not produce the exact expected identity",
                    check.member
                )));
            }
        }
        Ok(())
    }
}

/// A named helper for requiring one exact identity across all bundle members.
#[derive(Debug, Clone)]
pub struct CrossMemberAgreementValidator(ExactIdentityValidator);

impl CrossMemberAgreementValidator {
    /// Creates an exact-output agreement check for every supplied member.
    pub fn new<I>(members: I, expected_output: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = MemberId>,
    {
        Self(ExactIdentityValidator::for_members(
            members,
            expected_output,
        ))
    }

    /// Sets the per-member deadline.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.0 = self.0.timeout(timeout);
        self
    }

    /// Sets the per-stream output cap.
    pub fn max_output_bytes(mut self, limit: usize) -> Self {
        self.0 = self.0.max_output_bytes(limit);
        self
    }
}

impl CandidateValidator for CrossMemberAgreementValidator {
    fn validate(&self, transaction: &VerifiedTransaction) -> Result<()> {
        self.0.validate(transaction)
    }
}

/// A validator composition that requires every contained validator to pass.
pub struct AllValidators<'a> {
    validators: Vec<&'a dyn CandidateValidator>,
}

impl<'a> AllValidators<'a> {
    /// Creates an empty composition.
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Adds a validator while retaining its borrowed lifetime.
    pub fn push(mut self, validator: &'a dyn CandidateValidator) -> Self {
        self.validators.push(validator);
        self
    }
}

impl Default for AllValidators<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl CandidateValidator for AllValidators<'_> {
    fn validate(&self, transaction: &VerifiedTransaction) -> Result<()> {
        for validator in &self.validators {
            validator.validate(transaction)?;
        }
        Ok(())
    }
}

/// A transaction whose integrity-verified candidates passed a consumer validator.
#[derive(Debug)]
pub struct ValidatedTransaction {
    pub(crate) verified: VerifiedTransaction,
}

impl VerifiedTransaction {
    /// Runs a consumer-supplied validator before exposing the commit operation.
    pub fn validate(self, validator: &dyn CandidateValidator) -> Result<ValidatedTransaction> {
        if self.artifacts().iter().any(|member| {
            self.integrity(member.id())
                .is_none_or(|result| result.status() != IntegrityStatus::Verified)
        }) {
            return Err(Error::VerificationFailed(
                "candidate execution requires verified integrity evidence for every member".into(),
            ));
        }
        validator.validate(&self)?;
        Ok(ValidatedTransaction { verified: self })
    }
}

impl ValidatedTransaction {
    /// Returns the validated artifact set.
    pub fn artifacts(&self) -> &crate::domain::ArtifactSet {
        self.verified.artifacts()
    }

    /// Returns a validated member's private staged path.
    ///
    /// Exposed so consumers and tests can inspect staged bytes; mutating the
    /// staged copy after validation causes the final digest revalidation in
    /// [`Self::commit`] to fail before any live mutation.
    pub fn staged_path(&self, member: &MemberId) -> Result<std::path::PathBuf> {
        self.verified.staged_path(member)
    }

    /// Commits only after integrity and candidate validation have passed.
    ///
    /// Destination ownership is proven again under the mutation lock via
    /// `ownership`, and every staged member is re-hashed against the digest
    /// recorded at [`VerifiedTransaction`](crate::integrity::VerifiedTransaction)
    /// time before any live destination is mutated. `NotRequired` integrity
    /// state can never reach this path: validation rejects it, and the final
    /// revalidation requires a verified digest for every member.
    pub fn commit(self, ownership: CommitOwnership<'_>) -> Result<TransactionReceipt> {
        let digests = self.verified.verified_digests();
        self.verified
            .prepared
            .commit_inner(ownership, &digests, None, None)
    }

    /// Commits the complete artifact set, runs one caller check while the
    /// mutation lock and rollback evidence remain held, then resolves its
    /// result according to `policy`.
    ///
    /// The callback runs only after all members are live. Its error is copied
    /// into bounded core-owned evidence. Panics are caught and recorded as a
    /// failed check, so `RollBack` still restores the prior generation during
    /// unwinding. The callback must itself bound its work; core invents no
    /// timeout. Process termination or power loss is outside this guarantee.
    pub fn commit_with_post_commit<F, E>(
        self,
        ownership: CommitOwnership<'_>,
        policy: PostCommitFailurePolicy,
        check: F,
    ) -> Result<TransactionReceipt>
    where
        F: FnOnce() -> std::result::Result<(), E>,
        E: std::fmt::Display,
    {
        let digests = self.verified.verified_digests();
        let check = Box::new(move || check().map_err(|error| bounded_display(&error)));
        self.verified
            .prepared
            .commit_inner(ownership, &digests, None, Some((policy, check)))
    }

    /// Test-only commit with injected faults. The ownership and staged
    /// revalidation contracts are identical to [`Self::commit`].
    #[cfg(test)]
    pub(crate) fn commit_with_fault(
        self,
        ownership: CommitOwnership<'_>,
        fault: crate::transaction::CommitFault,
    ) -> Result<TransactionReceipt> {
        let digests = self.verified.verified_digests();
        self.verified
            .prepared
            .commit_inner(ownership, &digests, Some(fault), None)
    }

    #[cfg(test)]
    pub(crate) fn commit_with_post_commit_fault(
        self,
        ownership: CommitOwnership<'_>,
        policy: PostCommitFailurePolicy,
        fault: crate::transaction::CommitFault,
    ) -> Result<TransactionReceipt> {
        let digests = self.verified.verified_digests();
        self.verified.prepared.commit_inner(
            ownership,
            &digests,
            Some(fault),
            Some((
                policy,
                Box::new(|| Err("injected post-commit failure".to_string())),
            )),
        )
    }
}

fn bounded_display(value: &impl std::fmt::Display) -> String {
    struct Bounded(String);
    impl std::fmt::Write for Bounded {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            for character in text.chars() {
                if self.0.len() + character.len_utf8() > 512 {
                    break;
                }
                self.0.push(character);
            }
            Ok(())
        }
    }
    let mut output = Bounded(String::new());
    let _ = std::write!(&mut output, "{value}");
    output.0
}
