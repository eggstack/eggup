use std::fmt::Write as _;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::{CommitOwnership, MemberId};
use crate::error::{Error, RecoveryError, RecoveryResult, Result};
use crate::integrity::{IntegrityStatus, VerifiedTransaction};
use crate::transaction::{PostCommitFailurePolicy, TransactionReceipt};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_OUTPUT_LIMIT: usize = 64 * 1024;

/// The maximum number of bytes one bounded stdin payload may carry.
///
/// The bound is **inclusive**: a payload of exactly `MAX_STDIN_BYTES` is
/// accepted. A larger payload is rejected by [`run_bounded`] with
/// [`Error::InvalidInput`] **before any child is spawned**, so the bound fails
/// closed and no process is created for an oversized payload.
///
/// The bound is a single per-call bound on the whole payload. It is separate
/// from [`CommandSpec::max_output_bytes`], which bounds each captured output
/// stream independently.
pub const MAX_STDIN_BYTES: usize = 1024 * 1024;

/// An argv-based candidate command with bounded execution policy.
#[derive(Debug, Clone)]
pub struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
    timeout: Duration,
    max_output_bytes: usize,
    environment: Vec<(String, String)>,
    stdin_data: Option<Vec<u8>>,
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
            stdin_data: None,
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
    ///
    /// The value is an unconditional maximum bound on the whole bounded
    /// execution: it is evaluated whether or not the direct child has already
    /// exited, so [`run_bounded`] always returns. The deadline starts once the
    /// spawn has returned, so a blocking spawn is not covered by it.
    ///
    /// [`run_bounded`] states the exact bound, the teardown allowance, and
    /// what a caller observes.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the per-stream output bound.
    ///
    /// The bound applies to each stream separately: stdout and stderr are
    /// captured independently, so one run may capture up to twice the
    /// configured value in total. A stream that exceeds the bound is reported
    /// through [`CommandOutput::output_limited`] and its captured bytes are
    /// discarded, so that stream is reported as empty.
    pub fn max_output_bytes(mut self, limit: usize) -> Self {
        self.max_output_bytes = limit;
        self
    }

    /// Adds one explicit environment variable; inherited environment is cleared.
    pub fn environment(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.push((name.into(), value.into()));
        self
    }

    /// Attaches a bounded stdin payload for the command to read.
    ///
    /// The builder is **infallible**: the bound is enforced at execution time,
    /// not here, so an oversized payload is rejected by [`run_bounded`] before
    /// any child is spawned rather than by this builder.
    ///
    /// Without a payload the child's stdin is null, exactly as before. With a
    /// payload the child's stdin is a pipe, and a dedicated writer thread
    /// writes the bytes and closes the pipe, so the command observes EOF when
    /// the payload has been delivered. The writer is bounded by the same
    /// deadline that bounds the output readers.
    ///
    /// The payload is bounded by [`MAX_STDIN_BYTES`], inclusively.
    ///
    /// [`run_bounded`]: crate::run_bounded
    /// [`MAX_STDIN_BYTES`]: crate::MAX_STDIN_BYTES
    pub fn stdin_data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.stdin_data = Some(data.into());
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

    /// Returns whether the deadline was reached before the bounded execution completed.
    ///
    /// The direct child is killed only when it was still running. When it had
    /// already exited — for example because a descendant still held the output
    /// pipe open — the flag still reports that the deadline was reached and
    /// the bounded execution did not complete, and the direct child's exit
    /// code stays observable through [`Self::exit_code`].
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

/// Executes a command with cleared environment, bounded stdin, bounded output, and kill/reap.
///
/// The wall-clock deadline set by [`CommandSpec::timeout`] is an unconditional
/// maximum bound on the whole bounded execution. It is evaluated on every poll
/// whether or not the direct child has already exited, so this call always
/// returns. The deadline starts once the spawn has returned, so a blocking
/// spawn is not covered by it.
///
/// Stdin is **null** unless the caller attached a payload with
/// [`CommandSpec::stdin_data`]. An attached payload is bounded by
/// [`MAX_STDIN_BYTES`], inclusively: a larger payload is rejected with
/// [`Error::InvalidInput`] **before any child is spawned**, so no process is
/// created for it. A payload within the bound is written by a dedicated writer
/// thread, which closes the pipe when it ends, so the child observes EOF once
/// the payload has been delivered. The writer thread is started only after the
/// output readers are attached, so a child that fills stdout or stderr before
/// draining its stdin cannot deadlock both sides.
///
/// The deadline bounds the stdin writer exactly as it bounds the output
/// readers: when the deadline is reached, the child is killed, which breaks the
/// child's stdin pipe and unblocks the writer, and a writer that is still
/// blocked in `write` is abandoned explicitly, exactly like a reader blocked
/// on a pipe a surviving descendant holds. A bounded execution is not reported
/// as complete until the writer has delivered its verdict, so a write failure
/// is never lost behind a "the child exited" observation. A write failure
/// detected **after** the deadline was reached does not retroactively turn the
/// deadline outcome into an `Err`: the deadline already decided that outcome.
///
/// A descendant of the direct child can inherit the stdout or stderr pipe and
/// hold it open after the direct child has exited. Core terminates only the
/// direct child it spawned, so such a descendant is never killed: the call
/// returns at the deadline, the result reports `timed_out` together with the
/// direct child's exit code, and the stream the descendant still holds
/// contributes no bytes, because they were never captured.
///
/// The only work that may extend past the deadline is the kill and reap of a
/// direct child that is still running. That allowance is bounded by the
/// child's own termination; it is not a second, larger deadline.
///
/// A reader thread that delivered its result is joined, which is bounded
/// because a reader only has to return after its send. A reader that was still
/// blocked in `read()` when the bound was reached cannot be joined without
/// waiting past the deadline, and core cannot terminate a descendant it does
/// not own: it has no process-group or job-object dependency and `unsafe` is
/// denied. Such a reader is abandoned explicitly, and it ends when the pipe
/// reaches EOF, that is, when the holding descendant exits or closes the pipe.
/// The same rule applies to the stdin writer.
///
/// The caller observes four distinguishable outcomes:
///
/// - the child ran to completion: `Ok`, with the exit code, and `timed_out`
///   and `output_limited` both `false`;
/// - the deadline was reached: `Ok`, with `timed_out` set; the direct child
///   is killed only if it was still running;
/// - a stream exceeded `max_output_bytes`: `Ok`, with `output_limited` set and
///   that stream's captured bytes discarded;
/// - the command could not be spawned or polled, or the stdin payload could
///   not be written: `Err`.
///
/// # Error classification
///
/// The `operation` string of [`Error::Io`] is a **stable classification
/// surface** for this function: a downstream adapter may branch on it, so
/// these three strings are part of the published contract.
///
/// - `"spawning bounded candidate"` — the child could not be spawned. Nothing
///   was created, so the caller should treat it as an environment problem and
///   may retry under its own policy.
/// - `"polling bounded candidate"` — the child's status could not be polled.
///   The child may still be running, so the caller must not treat the outcome
///   as a completed execution.
/// - `"writing bounded stdin"` — the stdin payload could not be written, which
///   includes the broken pipe a child produces when it exits without reading
///   its stdin. The child is killed and reaped before the error is returned,
///   so the payload was **not delivered** and the caller must not treat the
///   execution as complete.
///
/// When a failed stdin write and an output overflow are live at the same time,
/// the **write failure wins**: it is an `Err`, and the caller must not be told
/// that the execution merely overflowed.
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] when the attached stdin payload is larger
/// than [`MAX_STDIN_BYTES`]. Nothing is spawned in that case.
///
/// Returns [`Error::Io`] when the command cannot be spawned, when polling the
/// child fails, or when the stdin payload cannot be written. The `operation`
/// string distinguishes the three cases; see
/// [Error classification](#error-classification).
///
/// Returns [`Error::CandidateExecution`] when the spawned child's stdout,
/// stderr, or stdin pipe is missing.
///
/// A command that ran to completion, that reached the deadline, or that
/// exceeded its output bound is **not** an error: those outcomes are reported
/// through [`CommandOutput::timed_out`], [`CommandOutput::output_limited`],
/// and [`CommandOutput::exit_code`] on an `Ok` result. A reader error is
/// surfaced as `output_limited`, never as an `Err`.
pub fn run_bounded(spec: &CommandSpec) -> Result<CommandOutput> {
    // The stdin bound is enforced before anything is spawned, so an oversized
    // payload fails closed with no child created.
    if let Some(data) = &spec.stdin_data {
        if data.len() > MAX_STDIN_BYTES {
            return Err(Error::InvalidInput(
                "stdin data exceeds the bounded stdin bound".into(),
            ));
        }
    }
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .env_clear()
        .stdin(if spec.stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
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
    // The stdin handle is taken before the readers start, but the write itself
    // starts only after both readers are attached.
    let stdin_pipe = if spec.stdin_data.is_some() {
        match child.stdin.take() {
            Some(pipe) => Some(pipe),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Error::CandidateExecution("missing stdin pipe".into()));
            }
        }
    } else {
        None
    };
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();
    let limit = spec.max_output_bytes;
    let stdout_thread = thread::spawn(move || {
        let _ = stdout_tx.send(read_limited(stdout, limit));
    });
    let stderr_thread = thread::spawn(move || {
        let _ = stderr_tx.send(read_limited(stderr, limit));
    });
    let (stdin_tx, stdin_rx) = mpsc::channel();
    let stdin_thread = stdin_pipe.map(|mut pipe| {
        let payload = spec.stdin_data.clone().unwrap_or_default();
        thread::spawn(move || {
            use std::io::Write as _;
            let result = pipe.write_all(&payload).and_then(|()| pipe.flush());
            let _ = stdin_tx.send(result);
        })
    });

    let deadline = Instant::now() + spec.timeout;
    let mut timed_out = false;
    let mut output_limited = false;
    let mut status = None;
    let mut stdout_result = None;
    let mut stderr_result = None;
    let mut stdout_finished = false;
    let mut stderr_finished = false;
    let mut stdin_result: Option<io::Result<()>> = None;
    let mut stdin_finished = stdin_thread.is_none();
    let mut stdin_error: Option<io::Error> = None;
    loop {
        if !stdout_finished {
            stdout_finished = poll_reader(&stdout_rx, &mut stdout_result, &mut output_limited);
        }
        if !stderr_finished {
            stderr_finished = poll_reader(&stderr_rx, &mut stderr_result, &mut output_limited);
        }
        if !stdin_finished {
            stdin_finished = poll_writer(&stdin_rx, &mut stdin_result);
        }
        if status.is_none() {
            status = child
                .try_wait()
                .map_err(|source| Error::io("polling bounded candidate", source))?;
        }
        if let Some(Err(source)) = stdin_result.take() {
            // A failed stdin write decides the outcome: the payload was not
            // delivered, so the child is killed and reaped and the call fails
            // closed rather than reporting a partial success.
            stdin_error = Some(source);
            if status.is_none() {
                let _ = child.kill();
                status = child.wait().ok();
            }
            break;
        }
        if output_limited {
            // The output bound decides the outcome: the child is killed and
            // reaped, and the streams are no longer waited for.
            if status.is_none() {
                let _ = child.kill();
                status = child.wait().ok();
            }
            break;
        }
        if status.is_some() && stdout_result.is_some() && stderr_result.is_some() && stdin_finished
        {
            // The bounded execution completed: the child exited, both pipes
            // reached EOF, and the stdin writer delivered its verdict.
            break;
        }
        // The deadline is unconditional: it is evaluated whether or not the
        // direct child has already exited, and the loop leaves without waiting
        // for pipe EOF.
        if Instant::now() >= deadline {
            timed_out = true;
            if status.is_none() {
                let _ = child.kill();
                status = child.wait().ok();
            }
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    // One last non-blocking drain, so a result that arrived at the bound is
    // reported rather than lost.
    stdout_finished = poll_reader(&stdout_rx, &mut stdout_result, &mut output_limited);
    stderr_finished = poll_reader(&stderr_rx, &mut stderr_result, &mut output_limited);
    if stdout_finished || stdout_thread.is_finished() {
        // The reader delivered its result, or it has already ended, so it only
        // has to return: the join is bounded.
        let _ = stdout_thread.join();
    }
    if stderr_finished || stderr_thread.is_finished() {
        let _ = stderr_thread.join();
    }
    if let Some(writer) = stdin_thread {
        if stdin_finished || writer.is_finished() {
            // The writer delivered its result, or it has already ended, so it
            // only has to return: the join is bounded.
            let _ = writer.join();
        }
        // A writer that was still blocked in `write` when the bound was reached
        // is abandoned explicitly, exactly like a reader blocked on a pipe a
        // surviving descendant holds. It ends when the child's stdin pipe
        // breaks, that is, when the child is killed or a holding descendant
        // exits or closes the pipe.
    }
    if let Some(source) = stdin_error {
        return Err(Error::io("writing bounded stdin", source));
    }
    Ok(CommandOutput {
        exit_code: status.and_then(|status| status.code()),
        stdout: stdout_result
            .and_then(|result| result.ok())
            .unwrap_or_default(),
        stderr: stderr_result
            .and_then(|result| result.ok())
            .unwrap_or_default(),
        timed_out,
        output_limited,
    })
}

/// Drains one stdin writer result if it has already arrived.
///
/// Returns `true` when the writer thread is known to have ended, that is, when
/// it delivered its result, and `false` while the writer is still blocked in
/// `write`.
fn poll_writer(
    receiver: &mpsc::Receiver<io::Result<()>>,
    slot: &mut Option<io::Result<()>>,
) -> bool {
    if slot.is_some() {
        return true;
    }
    match receiver.try_recv() {
        Ok(result) => {
            *slot = Some(result);
            true
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            // The writer ended without delivering a verdict. The writer this
            // module spawns cannot do that, because its send is unconditional
            // and the thread cannot panic; it is still treated as a write
            // failure so that the outcome fails closed.
            *slot = Some(Err(io::Error::other(
                "bounded stdin writer ended without a result",
            )));
            true
        }
        Err(mpsc::TryRecvError::Empty) => false,
    }
}

/// Drains one reader result if it has already arrived.
///
/// Returns `true` when the reader thread is known to have ended, that is, when
/// it delivered its result or ended without one, and `false` while the reader
/// is still blocked in `read()`.
fn poll_reader(
    receiver: &mpsc::Receiver<io::Result<Vec<u8>>>,
    slot: &mut Option<io::Result<Vec<u8>>>,
    output_limited: &mut bool,
) -> bool {
    if slot.is_some() {
        return true;
    }
    match receiver.try_recv() {
        Ok(result) => {
            *output_limited |= result.is_err();
            *slot = Some(result);
            true
        }
        Err(mpsc::TryRecvError::Disconnected) => true,
        Err(mpsc::TryRecvError::Empty) => false,
    }
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
    ///
    /// # Errors
    ///
    /// Returns whatever the validator returns. The validators this crate
    /// provides return [`Error::UnknownMember`] when a configured member is
    /// not part of the transaction, [`Error::Io`] when a staged member cannot
    /// be spawned or polled, and [`Error::CandidateExecution`] when a bounded
    /// execution does not produce the expected identity.
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::VerificationFailed`] when any member of the
    /// transaction does not carry verified integrity evidence. A member
    /// declared [`IntegrityRequirement::None`](crate::IntegrityRequirement::None)
    /// is never verified, so it is always rejected here: there is no escape
    /// hatch. Otherwise the error is the validator's own.
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownMember`] when `member` is not part of this
    /// transaction's artifact set.
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
    ///
    /// # Errors
    ///
    /// Returns `Err` only for lock contention ([`Error::UpdateInProgress`])
    /// and for setup failures such as [`Error::Io`] that occur while the
    /// mutation lock is being taken. Every failure reached once the lock is
    /// held — an ownership conflict, a staged-digest mismatch, a failed
    /// rename, a failed rollback — is reported through the returned
    /// [`TransactionReceipt`], whose disposition is `RolledBack` or
    /// `RecoveryRequired`, and never through `Err`.
    ///
    /// The direction is fail-closed: `Owned` is never inferred, and a
    /// verifier's ambiguity is `Unknown`, which denies mutation. Every member
    /// must carry verified SHA-256 evidence, so a member declared `None` can
    /// never reach this call. SHA-256 is integrity evidence only, never a
    /// publisher-trust or signature claim.
    ///
    /// See `architecture/core-transaction.md` §8 "Failure semantics".
    pub fn commit(self, ownership: CommitOwnership<'_>) -> Result<TransactionReceipt> {
        let digests = self.verified.verified_digests();
        self.verified
            .prepared
            .commit_inner(ownership, &digests, None, None, None)
            .map_err(RecoveryError::into_core)
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
    ///
    /// # Errors
    ///
    /// The same as [`commit`](Self::commit): only lock contention and setup
    /// failures return `Err`. A failing post-commit check is **not** an
    /// error — it is reported in the receipt's
    /// [`post_commit_failure`](TransactionReceipt::post_commit_failure). With
    /// [`PostCommitFailurePolicy::KeepInstalled`] the disposition stays
    /// `Committed`; with [`PostCommitFailurePolicy::RollBack`] the previous
    /// generation is restored and the disposition becomes `RolledBack`. A
    /// panic in the callback is caught and reported the same way.
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
            .commit_inner(ownership, &digests, None, Some((policy, check)), None)
            .map_err(RecoveryError::into_core)
    }

    /// Commits with caller-authorized stale-lock recovery.
    ///
    /// Identical to [`commit`](Self::commit) except that a pre-existing lock
    /// record may be displaced — but only when `recovery` returns
    /// [`StaleLockDecision::ProvenStale`](crate::lock::StaleLockDecision::ProvenStale)
    /// for the exact record Core observed. Every other verdict, and every record
    /// Core cannot prove safe, keeps the fail-closed contention result.
    ///
    /// Core never decides staleness from PID liveness, record age, executable
    /// name, or service state; that evidence belongs to the caller.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryError::Core`](crate::RecoveryError::Core) carrying
    /// [`Error::UpdateInProgress`] when the lock is contended, when the
    /// caller's verifier does not return
    /// [`StaleLockDecision::ProvenStale`](crate::lock::StaleLockDecision::ProvenStale)
    /// for the exact record core observed, or when another writer wins the
    /// race after a claim; and carrying a setup failure such as
    /// [`Error::Io`] otherwise.
    ///
    /// Returns [`RecoveryError::RecoveryRequired`](crate::RecoveryError::RecoveryRequired)
    /// when a stale record was displaced but could not be resolved
    /// automatically; the real retained evidence path is in
    /// [`RecoveryError::evidence`](crate::RecoveryError::evidence).
    ///
    /// Every failure reached once the lock is held is reported through the
    /// returned [`TransactionReceipt`], never through `Err`.
    pub fn commit_with_stale_lock_recovery(
        self,
        ownership: CommitOwnership<'_>,
        recovery: &dyn crate::lock::StaleLockVerifier,
    ) -> RecoveryResult<TransactionReceipt> {
        let digests = self.verified.verified_digests();
        self.verified
            .prepared
            .commit_inner(ownership, &digests, None, None, Some(recovery))
    }

    /// Commits with a caller post-commit check and caller-authorized stale-lock
    /// recovery, combining both opt-in policies.
    ///
    /// # Errors
    ///
    /// The union of [`commit_with_post_commit`](Self::commit_with_post_commit)
    /// and
    /// [`commit_with_stale_lock_recovery`](Self::commit_with_stale_lock_recovery):
    /// only lock contention and setup failures return `Err`, a failing
    /// post-commit check is reported in the receipt's
    /// [`post_commit_failure`](TransactionReceipt::post_commit_failure), and
    /// displaced stale evidence that could not be resolved is reported as
    /// [`RecoveryError::RecoveryRequired`](crate::RecoveryError::RecoveryRequired)
    /// with its real retained path.
    pub fn commit_with_post_commit_and_stale_lock_recovery<F, E>(
        self,
        ownership: CommitOwnership<'_>,
        policy: PostCommitFailurePolicy,
        recovery: &dyn crate::lock::StaleLockVerifier,
        check: F,
    ) -> RecoveryResult<TransactionReceipt>
    where
        F: FnOnce() -> std::result::Result<(), E>,
        E: std::fmt::Display,
    {
        let digests = self.verified.verified_digests();
        let check = Box::new(move || check().map_err(|error| bounded_display(&error)));
        self.verified.prepared.commit_inner(
            ownership,
            &digests,
            None,
            Some((policy, check)),
            Some(recovery),
        )
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
            .commit_inner(ownership, &digests, Some(fault), None, None)
            .map_err(RecoveryError::into_core)
    }

    #[cfg(test)]
    pub(crate) fn commit_with_post_commit_fault(
        self,
        ownership: CommitOwnership<'_>,
        policy: PostCommitFailurePolicy,
        fault: crate::transaction::CommitFault,
    ) -> Result<TransactionReceipt> {
        let digests = self.verified.verified_digests();
        self.verified
            .prepared
            .commit_inner(
                ownership,
                &digests,
                Some(fault),
                Some((
                    policy,
                    Box::new(|| Err("injected post-commit failure".to_string())),
                )),
                None,
            )
            .map_err(RecoveryError::into_core)
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
