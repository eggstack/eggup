#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "External curl acquisition adapter with explicit bounded policy."]
#![doc = ""]
#![doc = "Implements `eggup-acquisition::AcquisitionTransport` without an embedded"]
#![doc = "HTTP/TLS stack. See `README.md` for the single policy configuration point."]
#![doc = ""]
#![doc = "Transport fallback (curl <-> Eggfetch for the same exact URL) lives in"]
#![doc = "`eggup-acquisition::ComposedTransport` and is never release/source fallback."]

use eggup_acquisition::{
    AcquisitionError, AcquisitionRequest, AcquisitionTransport, CancelFlag, FetchLimits,
    FetchOutcome,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Default connect deadline ceiling.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default total wall-clock deadline ceiling.
pub const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(120);
/// Default redirect bound when following is enabled.
pub const DEFAULT_MAX_REDIRECTS: usize = 10;
/// Bound for captured curl stdout (`-w "%{http_code}"` plus slack).
const MAX_WRITE_OUT_BYTES: u64 = 64;
/// Parent polling granularity while waiting for the owned child.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Explicit proxy decision. Routing is never inherited accidentally.
#[derive(Debug, Clone, Default)]
pub enum CurlProxy {
    /// No proxy is used. The child environment is cleared and
    /// `--noproxy "*"` is passed as defense-in-depth.
    #[default]
    Disabled,
    /// Use an explicit snapshot of the current process proxy environment.
    FromEnvironment,
    /// Use explicit `KEY=value` pairs (e.g. `HTTPS_PROXY`, `NO_PROXY`).
    Custom(Vec<(String, String)>),
}

/// Strict transport configuration. All policy lives here.
#[derive(Debug, Clone)]
pub struct CurlConfig {
    /// Connect deadline ceiling.
    pub connect_timeout: Duration,
    /// Total deadline ceiling.
    pub total_timeout: Duration,
    /// Redirect bound when following is enabled.
    pub max_redirects: usize,
    /// Whether HTTP redirects are followed.
    pub follow_redirects: bool,
    /// Allowed URL protocols (default `http`, `https`).
    pub allowed_protocols: Vec<String>,
    /// Proxy routing decision.
    pub proxy: CurlProxy,
}

impl Default for CurlConfig {
    fn default() -> Self {
        Self {
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            total_timeout: DEFAULT_TOTAL_TIMEOUT,
            max_redirects: DEFAULT_MAX_REDIRECTS,
            follow_redirects: true,
            allowed_protocols: vec!["http".to_string(), "https".to_string()],
            proxy: CurlProxy::Disabled,
        }
    }
}

impl CurlConfig {
    /// Strict default: bounded timeouts, redirect bound, explicit proxy.
    pub fn strict() -> Self {
        Self::default()
    }

    /// Sets connect/total adapter ceilings.
    ///
    /// These are ceilings, not defaults: the effective deadline for one fetch
    /// is `min(request, adapter ceiling)` (see [`FetchLimits::effective`]).
    pub fn timeouts(mut self, connect: Duration, total: Duration) -> Self {
        self.connect_timeout = connect;
        self.total_timeout = total;
        self
    }

    /// Sets the redirect bound.
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = max;
        self
    }

    /// Sets whether redirects are followed.
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Sets the allowed protocols.
    pub fn allowed_protocols(mut self, protocols: Vec<String>) -> Self {
        self.allowed_protocols = protocols;
        self
    }

    /// Sets the proxy decision.
    pub fn proxy(mut self, proxy: CurlProxy) -> Self {
        self.proxy = proxy;
        self
    }

    /// Derives the authoritative effective deadlines for one fetch.
    pub fn effective_timeouts(&self, limits: FetchLimits) -> (Duration, Duration) {
        limits.effective(self.connect_timeout, self.total_timeout)
    }

    fn validate(&self) -> Result<(), AcquisitionError> {
        if self.connect_timeout.is_zero() || self.total_timeout.is_zero() {
            return Err(AcquisitionError::InvalidInput(bound(
                "adapter timeouts must be positive".to_string(),
            )));
        }
        if self.connect_timeout > self.total_timeout {
            return Err(AcquisitionError::InvalidInput(bound(
                "adapter connect timeout must not exceed total timeout".to_string(),
            )));
        }
        if self.max_redirects > 50 {
            return Err(AcquisitionError::InvalidInput(bound(
                "adapter redirect bound out of range".to_string(),
            )));
        }
        if self.allowed_protocols.is_empty() || self.allowed_protocols.len() > 8 {
            return Err(AcquisitionError::InvalidInput(bound(
                "adapter protocol list out of range".to_string(),
            )));
        }
        for p in &self.allowed_protocols {
            if p.is_empty() || p.len() > 16 || !p.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(AcquisitionError::InvalidInput(bound(
                    "adapter protocol names must be alphanumeric".to_string(),
                )));
            }
        }
        Ok(())
    }
}

/// External curl adapter implementing the acquisition seam.
///
/// No release, version, fallback, destination, or service policy lives here.
/// Only exact 404 becomes `NotFound`; TLS/5xx/timeout are hard failures and
/// missing/spawn failures become `Unavailable` for safe composition.
#[derive(Debug)]
pub struct CurlTransport {
    executable: PathBuf,
    config: CurlConfig,
}

impl CurlTransport {
    /// Builds an adapter from an explicit executable path plus config.
    ///
    /// The path is the strongest selection; discovery via
    /// [`discover_curl_executable`] is opt-in only. Config validation happens
    /// here; executable existence/spawn is classified per-request as
    /// `Unavailable` so composition can fall back.
    pub fn with_executable(
        executable: impl Into<PathBuf>,
        config: CurlConfig,
    ) -> Result<Self, AcquisitionError> {
        config.validate()?;
        let executable = executable.into();
        if executable.as_os_str().is_empty() {
            return Err(AcquisitionError::InvalidInput(bound(
                "curl executable path must not be empty".to_string(),
            )));
        }
        Ok(Self { executable, config })
    }

    /// Returns the configured executable path.
    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Returns the transport configuration.
    pub fn config(&self) -> &CurlConfig {
        &self.config
    }
}

impl AcquisitionTransport for CurlTransport {
    fn fetch_metadata(
        &self,
        request: &AcquisitionRequest,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<eggup_acquisition::MetadataBytes>, AcquisitionError> {
        limits.validate()?;
        self.config.validate()?;
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        let (eff_connect, eff_total) = self.config.effective_timeouts(limits);
        let max = limits.max_metadata_bytes;
        // Owned private temp in the system temp dir; same exclusive/0600
        // machinery as artifact staging, cleaned unconditionally.
        let parent = std::env::temp_dir();
        let _ = fs::create_dir_all(&parent);
        let (_file, tmp) = eggup_acquisition::__acquire_exclusive_temp(&parent, "eggup-curl-meta")?;
        struct Guard {
            path: PathBuf,
            disarm: bool,
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                if !self.disarm {
                    eggup_acquisition::__remove_owned_temp(&self.path);
                }
            }
        }
        let mut guard = Guard {
            path: tmp.clone(),
            disarm: false,
        };
        // Close the pre-created handle; curl truncates and writes the path.
        drop(_file);
        let outcome = run_curl_to_file(
            &self.executable,
            &self.config,
            request,
            &tmp,
            Some(max as u64),
            eff_connect,
            eff_total,
            cancel,
        )?;
        match outcome {
            CurlOutcome::Success => {
                if cancel.is_cancelled() {
                    return Err(AcquisitionError::Cancelled);
                }
                let bytes = fs::read(&tmp)
                    .map_err(|e| AcquisitionError::Io(bound(format!("reading part file: {e}"))))?;
                eggup_acquisition::__remove_owned_temp(&tmp);
                guard.disarm = true;
                if bytes.len() > max {
                    return Err(AcquisitionError::TooLarge { limit: max as u64 });
                }
                Ok(FetchOutcome::Success(
                    eggup_acquisition::__adapter_metadata(bytes),
                ))
            }
            CurlOutcome::NotFound => {
                eggup_acquisition::__remove_owned_temp(&tmp);
                guard.disarm = true;
                Ok(FetchOutcome::NotFound)
            }
        }
    }

    fn fetch_artifact(
        &self,
        request: &AcquisitionRequest,
        dest: &Path,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<eggup_acquisition::ArtifactEvidence>, AcquisitionError> {
        limits.validate()?;
        self.config.validate()?;
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        let parent = dest.parent().ok_or_else(|| {
            AcquisitionError::InvalidInput(bound("destination has no parent".into()))
        })?;
        let parent_meta = fs::symlink_metadata(parent)
            .map_err(|e| AcquisitionError::Io(bound(format!("reading artifact parent: {e}"))))?;
        if !parent_meta.is_dir() || parent_meta.file_type().is_symlink() {
            return Err(AcquisitionError::InvalidInput(bound(
                "artifact parent must be an existing real directory".into(),
            )));
        }
        if fs::symlink_metadata(dest).is_ok() {
            return Err(AcquisitionError::InvalidInput(bound(
                "artifact destination already exists; refusing to overwrite".into(),
            )));
        }
        let (eff_connect, eff_total) = self.config.effective_timeouts(limits);
        let max_artifact = limits.max_artifact_bytes;
        let (file, tmp) = eggup_acquisition::__acquire_exclusive_temp(parent, "eggup-curl")?;
        struct Guard {
            path: PathBuf,
            disarm: bool,
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                if !self.disarm {
                    eggup_acquisition::__remove_owned_temp(&self.path);
                }
            }
        }
        let mut guard = Guard {
            path: tmp.clone(),
            disarm: false,
        };
        drop(file);
        let outcome = run_curl_to_file(
            &self.executable,
            &self.config,
            request,
            &tmp,
            max_artifact,
            eff_connect,
            eff_total,
            cancel,
        )?;
        match outcome {
            CurlOutcome::NotFound => {
                eggup_acquisition::__remove_owned_temp(&tmp);
                guard.disarm = true;
                Ok(FetchOutcome::NotFound)
            }
            CurlOutcome::Success => {
                if cancel.is_cancelled() {
                    return Err(AcquisitionError::Cancelled);
                }
                let size = fs::metadata(&tmp)
                    .map_err(|e| AcquisitionError::Io(bound(format!("reading part file: {e}"))))?
                    .len();
                if let Some(max) = max_artifact {
                    if size > max {
                        eggup_acquisition::__remove_owned_temp(&tmp);
                        guard.disarm = true;
                        return Err(AcquisitionError::TooLarge { limit: max });
                    }
                }
                match eggup_acquisition::__promote_no_clobber(&tmp, dest) {
                    Ok(()) => {
                        guard.disarm = true;
                        Ok(FetchOutcome::Success(
                            eggup_acquisition::__adapter_artifact(size),
                        ))
                    }
                    Err(e) => {
                        eggup_acquisition::__remove_owned_temp(&tmp);
                        guard.disarm = true;
                        Err(e)
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurlOutcome {
    Success,
    NotFound,
}

/// Opt-in discovery of a `curl` executable via `PATH`.
///
/// Searches `PATH` for `curl` (plus `curl.exe` on Windows) and returns the
/// first existing regular file. Returns `Unavailable` when nothing is found.
/// Callers that require an explicit path should use
/// [`CurlTransport::with_executable`] directly instead.
pub fn discover_curl_executable() -> Result<PathBuf, AcquisitionError> {
    discover_in(std::env::var_os("PATH"))
}

fn discover_in(path_var: Option<std::ffi::OsString>) -> Result<PathBuf, AcquisitionError> {
    let path_var = path_var.ok_or_else(|| {
        AcquisitionError::Unavailable(bound("curl not found: PATH is unset".to_string()))
    })?;
    #[cfg(windows)]
    let names = vec!["curl", "curl.exe"];
    #[cfg(not(windows))]
    let names = vec!["curl"];
    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        for name in &names {
            let candidate = dir.join(name);
            if let Ok(meta) = fs::symlink_metadata(&candidate) {
                if meta.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    Err(AcquisitionError::Unavailable(bound(
        "curl not found on PATH".to_string(),
    )))
}

fn bound(mut s: String) -> String {
    if s.len() > 512 {
        s.truncate(512);
    }
    s
}

fn ceil_secs(d: Duration) -> u64 {
    let secs = d.as_secs();
    let extra = if d.subsec_nanos() == 0 { 0 } else { 1 };
    (secs + extra).max(1)
}

fn proxy_env_snapshot(proxy: &CurlProxy) -> Vec<(String, String)> {
    const KEYS: &[&str] = &[
        "HTTP_PROXY",
        "http_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "ALL_PROXY",
        "all_proxy",
        "NO_PROXY",
        "no_proxy",
    ];
    match proxy {
        CurlProxy::Disabled => Vec::new(),
        CurlProxy::FromEnvironment => KEYS
            .iter()
            .filter_map(|k| {
                std::env::var_os(k).map(|v| ((*k).to_string(), v.to_string_lossy().into_owned()))
            })
            .collect(),
        CurlProxy::Custom(pairs) => pairs.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_curl_args(
    config: &CurlConfig,
    url: &str,
    output: &Path,
    max_bytes: Option<u64>,
    eff_connect: Duration,
    eff_total: Duration,
) -> Vec<String> {
    // Ignore ~/.curlrc so ambient config cannot smuggle proxy/auth policy.
    let mut args: Vec<String> = vec![
        "--disable".to_string(),
        "--silent".to_string(),
        "--show-error".to_string(),
        "--no-progress-meter".to_string(),
    ];
    if config.follow_redirects {
        args.push("--location".to_string());
        args.push("--max-redirs".to_string());
        args.push(config.max_redirects.to_string());
    }
    args.push("--connect-timeout".to_string());
    args.push(ceil_secs(eff_connect).to_string());
    args.push("--max-time".to_string());
    args.push(ceil_secs(eff_total).to_string());
    if let Some(max) = max_bytes {
        args.push("--max-filesize".to_string());
        args.push(max.to_string());
    }
    if !config.allowed_protocols.is_empty() {
        let list = format!("={}", config.allowed_protocols.join(","));
        args.push("--proto".to_string());
        args.push(list.clone());
        args.push("--proto-redir".to_string());
        args.push(list);
    }
    if matches!(proxy_kind(config), ProxyKind::Disabled) {
        args.push("--noproxy".to_string());
        args.push("*".to_string());
    }
    args.push("--output".to_string());
    args.push(output.to_string_lossy().into_owned());
    args.push("--write-out".to_string());
    args.push("%{http_code}".to_string());
    args.push("--".to_string());
    args.push(url.to_string());
    args
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProxyKind {
    Disabled,
    Enabled,
}

fn proxy_kind(config: &CurlConfig) -> ProxyKind {
    match &config.proxy {
        CurlProxy::Disabled => ProxyKind::Disabled,
        CurlProxy::FromEnvironment | CurlProxy::Custom(_) => ProxyKind::Enabled,
    }
}

#[allow(clippy::too_many_arguments)]
fn run_curl_to_file(
    executable: &Path,
    config: &CurlConfig,
    request: &AcquisitionRequest,
    output: &Path,
    max_bytes: Option<u64>,
    eff_connect: Duration,
    eff_total: Duration,
    cancel: &CancelFlag,
) -> Result<CurlOutcome, AcquisitionError> {
    let url = request.url().to_string();
    let redacted = request.redacted();
    let args = build_curl_args(config, &url, output, max_bytes, eff_connect, eff_total);
    let proxy_env = proxy_env_snapshot(&config.proxy);
    let start = Instant::now();
    let deadline = start + eff_total;

    let mut cmd = Command::new(executable);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env_clear();
    for (k, v) in &proxy_env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().map_err(|_| {
        AcquisitionError::Unavailable(bound(format!("curl spawn failed for {redacted}")))
    })?;
    // Take stdout before the wait loop so the pipe stays open; http_code is
    // at most a few bytes so post-exit bounded read cannot deadlock.
    let mut stdout = child.stdout.take();

    // Bounded wait: poll child, cancellation, and the parent wall deadline.
    // Kill and reap before returning on timeout/cancellation.
    let exit_code: Option<i32> = loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            eggup_acquisition::__remove_owned_temp(output);
            return Err(AcquisitionError::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status.code(),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    eggup_acquisition::__remove_owned_temp(output);
                    return Err(AcquisitionError::Timeout { phase: "total" });
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                eggup_acquisition::__remove_owned_temp(output);
                return Err(AcquisitionError::Io(bound(format!(
                    "waiting for curl: {e}"
                ))));
            }
        }
    };

    // Bounded stdout capture for `-w "%{http_code}"` (at most a few bytes).
    let mut code_bytes: Vec<u8> = Vec::new();
    if let Some(mut out) = stdout.take() {
        use std::io::Read;
        let mut chunk = [0u8; 128];
        loop {
            match out.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    code_bytes.extend_from_slice(&chunk[..n]);
                    if code_bytes.len() as u64 > MAX_WRITE_OUT_BYTES {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
    // Ensure the child is fully reaped (try_wait already reaped on Some, but
    // wait again is harmless if still running in a race).
    let _ = child.wait();
    classify_curl_result(
        exit_code,
        &code_bytes,
        output,
        &redacted,
        start,
        eff_connect,
        max_bytes,
    )
}

fn classify_curl_result(
    exit_code: Option<i32>,
    code_bytes: &[u8],
    output: &Path,
    redacted: &str,
    start: Instant,
    eff_connect: Duration,
    max_bytes: Option<u64>,
) -> Result<CurlOutcome, AcquisitionError> {
    let elapsed = start.elapsed();
    let code_text = String::from_utf8_lossy(code_bytes).trim().to_string();
    let http_code: Option<u16> = code_text.parse().ok();
    match exit_code {
        // curl operation timeout (connect or total). Distinguish by elapsed
        // against the effective connect ceiling so callers keep truthful
        // phase evidence without parsing stderr.
        Some(28) => {
            // Curl ceils sub-second timeouts to whole seconds, so allow 1s
            // slack when attributing a timeout to the connect phase.
            if elapsed <= eff_connect + Duration::from_secs(1) {
                return Err(AcquisitionError::Timeout { phase: "connect" });
            }
            return Err(AcquisitionError::Timeout { phase: "total" });
        }
        // curl --max-filesize exceeded.
        Some(63) => {
            eggup_acquisition::__remove_owned_temp(output);
            return Err(AcquisitionError::TooLarge {
                limit: max_bytes.unwrap_or(0),
            });
        }
        Some(0) => {}
        Some(_) | None => {
            // Any non-zero curl exit means the transfer did not complete
            // (partial body, TLS, connect, resolve). Even when curl still
            // prints an HTTP code, the body is incomplete and must not be
            // promoted. Report an ordinary transport failure.
            eggup_acquisition::__remove_owned_temp(output);
            return Err(AcquisitionError::Transport(bound(format!(
                "curl transfer failed for {redacted}"
            ))));
        }
    }
    match http_code {
        Some(200..=299) => Ok(CurlOutcome::Success),
        Some(404) => Ok(CurlOutcome::NotFound),
        Some(status) => {
            eggup_acquisition::__remove_owned_temp(output);
            Err(AcquisitionError::Transport(bound(format!(
                "HTTP {status} from {redacted}"
            ))))
        }
        None => {
            eggup_acquisition::__remove_owned_temp(output);
            Err(AcquisitionError::Transport(bound(format!(
                "curl status capture failed for {redacted}"
            ))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eggup_acquisition::{FixtureResponse, FixtureTransport};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::thread;

    fn req(url: &str) -> AcquisitionRequest {
        AcquisitionRequest::new(url).unwrap()
    }

    fn limits() -> FetchLimits {
        FetchLimits {
            max_metadata_bytes: 64 * 1024,
            max_artifact_bytes: Some(256 * 1024),
            connect_timeout: Duration::from_secs(2),
            total_timeout: Duration::from_secs(10),
        }
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let p = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        let _ = fs::create_dir_all(&p);
        p
    }

    fn real_curl() -> Option<PathBuf> {
        discover_curl_executable().ok()
    }

    fn transport() -> Option<CurlTransport> {
        real_curl().map(|exe| {
            CurlTransport::with_executable(exe, CurlConfig::strict()).expect("curl transport")
        })
    }

    fn serve_once(
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        stall_before_body_ms: u64,
        abort_after_headers: bool,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                let mut buf = [0u8; 8192];
                let _ = stream.read(&mut buf);
                let reason = match status {
                    200 => "OK",
                    302 => "Found",
                    404 => "Not Found",
                    500 => "Internal Server Error",
                    _ => "OK",
                };
                let mut head = format!("HTTP/1.1 {status} {reason}\r\n");
                for (k, v) in headers {
                    head.push_str(&format!("{k}: {v}\r\n"));
                }
                head.push_str("Connection: close\r\n\r\n");
                let _ = stream.write_all(head.as_bytes());
                if abort_after_headers {
                    return;
                }
                if stall_before_body_ms > 0 {
                    thread::sleep(Duration::from_millis(stall_before_body_ms));
                }
                let _ = stream.write_all(&body);
            }
        });
        format!("http://{addr}")
    }

    fn ok_server(body: Vec<u8>) -> String {
        serve_once(
            200,
            vec![
                ("Content-Length".into(), body.len().to_string()),
                ("Content-Type".into(), "application/octet-stream".into()),
            ],
            body,
            0,
            false,
        )
    }

    /// Minimal fake curl executable (Unix-only): parses `--output <path>`,
    /// writes a fixed body, and prints a fixed HTTP code to stdout. Records
    /// argv/env for policy assertions.
    ///
    /// Unix-only because it relies on `sh` + `xxd` and Unix executable-mode
    /// setup. Windows exercises the same transport paths through the real
    /// `curl.exe` integration tests and the platform-neutral unit tests below;
    /// shell-dependent fake-child tests are `#[cfg(unix)]`-gated, never the
    /// whole module.
    #[cfg(unix)]
    fn fake_curl_script(body: &[u8], code: &str, exit: i32, record_dir: &Path) -> PathBuf {
        let dir = record_dir.to_path_buf();
        let script = dir.join(format!(
            "fake-curl-{}-{}.sh",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let body_hex: String = body.iter().map(|b| format!("{b:02x}")).collect();
        let content = format!(
            "#!/bin/sh\n\
             echo \"$@\" > \"{rec}/argv\"\n\
             env > \"{rec}/env\"\n\
             OUT=\"\"\n\
             PREV=\"\"\n\
             for A in \"$@\"; do\n\
               if [ \"$PREV\" = \"--output\" ]; then OUT=\"$A\"; fi\n\
               PREV=\"$A\"\n\
             done\n\
             if [ -n \"$OUT\" ]; then\n\
               printf '{hex}' | xxd -r -p > \"$OUT\"\n\
             fi\n\
             printf '{code}'\n\
             exit {exit}\n",
            rec = dir.display(),
            hex = body_hex,
            code = code,
            exit = exit,
        );
        fs::write(&script, content).unwrap();
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&script, perms).unwrap();
        script
    }

    #[test]
    fn executable_selection_and_discovery_are_explicit() {
        // Empty path is invalid input, not unavailable.
        let err = CurlTransport::with_executable("", CurlConfig::strict()).unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
        // Invalid config fails closed at construction.
        let bad = CurlConfig::strict().timeouts(Duration::ZERO, Duration::from_secs(1));
        let err = CurlTransport::with_executable("/usr/bin/curl", bad).unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
        // Discovery is opt-in and returns a real file when curl exists.
        if let Ok(path) = discover_curl_executable() {
            assert!(fs::symlink_metadata(&path).unwrap().is_file());
        }
        // Absent PATH entry yields typed Unavailable, never Transport.
        // Uses the injectable helper so parallel tests never mutate process PATH.
        let err = discover_in(Some("/nonexistent-eggup-curl-path-xyz".into())).unwrap_err();
        assert!(matches!(err, AcquisitionError::Unavailable(_)));
        assert!(err.is_unavailable());
        let err = discover_in(None).unwrap_err();
        assert!(matches!(err, AcquisitionError::Unavailable(_)));
    }

    #[test]
    fn missing_executable_and_spawn_failure_are_unavailable() {
        let t = CurlTransport::with_executable(
            "/nonexistent-eggup-curl-binary-xyz",
            CurlConfig::strict(),
        )
        .unwrap();
        let err = t
            .fetch_metadata(&req("http://127.0.0.1:1/x"), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Unavailable(_)));
        let dir = temp_dir("curl-unavail");
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(
                &req("http://127.0.0.1:1/x"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Unavailable(_)));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_success_not_found_and_server_error() {
        let Some(t) = transport() else { return };
        let base = ok_server(b"hello-curl-meta".to_vec());
        let out = t
            .fetch_metadata(&req(&format!("{base}/meta")), limits(), &CancelFlag::new())
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"hello-curl-meta");

        let base = serve_once(404, vec![], b"nope".to_vec(), 0, false);
        let out = t
            .fetch_metadata(&req(&format!("{base}/m")), limits(), &CancelFlag::new())
            .unwrap();
        assert!(out.is_not_found());

        let base = serve_once(500, vec![], b"boom".to_vec(), 0, false);
        let err = t
            .fetch_metadata(&req(&format!("{base}/e")), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert!(!matches!(err, AcquisitionError::Unavailable(_)));
    }

    #[test]
    fn artifact_success_not_found_and_server_error() {
        let Some(t) = transport() else { return };
        let body: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
        let base = ok_server(body.clone());
        let dir = temp_dir("curl-artifact");
        let dest = dir.join("app");
        let out = t
            .fetch_artifact(
                &req(&format!("{base}/app")),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes_written, 4096);
        assert_eq!(fs::read(&dest).unwrap(), body);

        let base = serve_once(404, vec![], b"no".to_vec(), 0, false);
        let dest2 = dir.join("missing");
        let out = t
            .fetch_artifact(
                &req(&format!("{base}/m")),
                &dest2,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
        assert!(!dest2.exists());

        let base = serve_once(500, vec![], b"boom".to_vec(), 0, false);
        let dest3 = dir.join("err");
        let err = t
            .fetch_artifact(
                &req(&format!("{base}/e")),
                &dest3,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert!(!dest3.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn tls_like_process_failure_is_transport_not_unavailable() {
        let dir = temp_dir("curl-tls");
        let script = fake_curl_script(b"", "", 35, &dir);
        let t =
            CurlTransport::with_executable(&script, CurlConfig::strict()).expect("fake transport");
        let err = t
            .fetch_metadata(&req("https://example.com/x"), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert!(!err.is_unavailable());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn truncated_transfer_is_hard_failure_without_promotion() {
        let Some(t) = transport() else { return };
        let body = vec![5u8; 4096];
        let base = serve_once(
            200,
            vec![("Content-Length".into(), "4096".into())],
            body,
            0,
            true,
        );
        let dir = temp_dir("curl-trunc");
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(
                &req(&format!("{base}/app")),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_and_artifact_size_overflow() {
        let Some(t) = transport() else { return };
        let big = vec![9u8; 128 * 1024];
        let base = ok_server(big);
        let err = t
            .fetch_metadata(&req(&format!("{base}/big")), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::TooLarge { .. }));

        let body = vec![1u8; 8192];
        let base = ok_server(body);
        let dir = temp_dir("curl-overflow");
        let dest = dir.join("app");
        let small = FetchLimits {
            max_metadata_bytes: 64 * 1024,
            max_artifact_bytes: Some(1024),
            connect_timeout: Duration::from_secs(2),
            total_timeout: Duration::from_secs(10),
        };
        let err = t
            .fetch_artifact(
                &req(&format!("{base}/app")),
                &dest,
                small,
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::TooLarge { .. }));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn connect_and_total_timeouts_are_typed() {
        let Some(t) = transport() else { return };
        // Unroutable connect with a short connect ceiling.
        let short_connect = FetchLimits {
            max_metadata_bytes: 64 * 1024,
            max_artifact_bytes: Some(256 * 1024),
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(10),
        };
        let err = t
            .fetch_metadata(
                &req("http://10.255.255.1:81/unreachable"),
                short_connect,
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(
            matches!(err, AcquisitionError::Timeout { .. }),
            "expected timeout, got {err}"
        );
        // Slow body with a short total ceiling.
        let body = vec![2u8; 1024];
        let base = serve_once(
            200,
            vec![("Content-Length".into(), body.len().to_string())],
            body,
            5000,
            false,
        );
        let short_total = FetchLimits {
            max_metadata_bytes: 64 * 1024,
            max_artifact_bytes: Some(256 * 1024),
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
        };
        let err = t
            .fetch_metadata(
                &req(&format!("{base}/slow")),
                short_total,
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Timeout { phase: "total" }));
    }

    #[test]
    fn cancellation_kills_and_reaps_child() {
        let Some(t) = transport() else { return };
        let body = vec![3u8; 64 * 1024];
        let base = ok_server(body);
        let dir = temp_dir("curl-cancel");
        let dest = dir.join("app");
        let cancel = CancelFlag::new();
        cancel.cancel();
        let err = t
            .fetch_artifact(&req(&format!("{base}/app")), &dest, limits(), &cancel)
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Cancelled));
        assert!(!dest.exists());
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn redirect_follow_and_reject() {
        let Some(_) = real_curl() else { return };
        let target_body = b"redirect-target".to_vec();
        let target = ok_server(target_body.clone());
        let redirect = serve_once(
            302,
            vec![("Location".into(), format!("{target}/final"))],
            b"".to_vec(),
            0,
            false,
        );
        let follow = CurlTransport::with_executable(
            real_curl().unwrap(),
            CurlConfig::strict()
                .follow_redirects(true)
                .max_redirects(10),
        )
        .unwrap();
        let out = follow
            .fetch_metadata(&req(&format!("{redirect}/r")), limits(), &CancelFlag::new())
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"redirect-target");

        let nofollow = CurlTransport::with_executable(
            real_curl().unwrap(),
            CurlConfig::strict().follow_redirects(false),
        )
        .unwrap();
        let err = nofollow
            .fetch_metadata(&req(&format!("{redirect}/r")), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
    }

    #[cfg(unix)]
    #[test]
    fn proxy_disabled_uses_noproxy_and_cleared_env() {
        let dir = temp_dir("curl-proxy");
        let script = fake_curl_script(b"ok", "200", 0, &dir);
        let t = CurlTransport::with_executable(&script, CurlConfig::strict()).unwrap();
        let out = t
            .fetch_metadata(&req("http://example.com/x"), limits(), &CancelFlag::new())
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"ok");
        let argv = fs::read_to_string(dir.join("argv")).unwrap();
        assert!(argv.contains("--noproxy"));
        assert!(argv.contains("--disable"));
        let env = fs::read_to_string(dir.join("env")).unwrap();
        assert!(!env.contains("HTTP_PROXY=hunter"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn proxy_custom_env_is_explicit() {
        let dir = temp_dir("curl-proxy-custom");
        let script = fake_curl_script(b"ok", "200", 0, &dir);
        let t = CurlTransport::with_executable(
            &script,
            CurlConfig::strict().proxy(CurlProxy::Custom(vec![(
                "HTTPS_PROXY".to_string(),
                "http://proxy.example:8080".to_string(),
            )])),
        )
        .unwrap();
        t.fetch_metadata(&req("http://example.com/x"), limits(), &CancelFlag::new())
            .unwrap();
        let argv = fs::read_to_string(dir.join("argv")).unwrap();
        assert!(!argv.contains("--noproxy"));
        let env = fs::read_to_string(dir.join("env")).unwrap();
        assert!(env.contains("HTTPS_PROXY=http://proxy.example:8080"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn artifact_temp_is_owner_private_and_no_clobber() {
        let dir = temp_dir("curl-perms");
        let script = fake_curl_script(b"owned-bytes", "200", 0, &dir);
        // Use a separate work dir for the destination so the fake script dir
        // (which holds argv/env records) does not interfere.
        let work = temp_dir("curl-perms-work");
        // Fake curl writes directly to the Eggup-owned temp; verify promotion
        // preserves foreign destinations and cleans only owned temps.
        let t = CurlTransport::with_executable(&script, CurlConfig::strict()).unwrap();
        let dest = work.join("app");
        let out = t
            .fetch_artifact(
                &req("http://example.com/app"),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(
            out.success().unwrap().bytes_written,
            b"owned-bytes".len() as u64
        );
        #[cfg(unix)]
        {
            let mode = fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
            // Promoted artifact inherits the owner-private temp mode via
            // hard-link promotion (0600), never a world-readable default.
            assert_eq!(mode, 0o600);
        }
        // Existing destination is never overwritten.
        fs::write(&dest, b"FOREIGN").unwrap();
        let dest2 = work.join("app");
        let err = t
            .fetch_artifact(
                &req("http://example.com/app"),
                &dest2,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
        assert_eq!(fs::read(&dest2).unwrap(), b"FOREIGN");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&work);
    }

    #[cfg(unix)]
    #[test]
    fn credential_material_is_redacted() {
        let dir = temp_dir("curl-redact");
        let script = fake_curl_script(b"", "", 7, &dir);
        let t = CurlTransport::with_executable(&script, CurlConfig::strict()).unwrap();
        let url = "https://user:S3CR3T-USERINFO-9917@example.com/app?token=TOKEN-QUERY-5523#FRAG-SENTINEL-7788";
        let err = t
            .fetch_metadata(&req(url), limits(), &CancelFlag::new())
            .unwrap_err();
        let msg = format!("{err}");
        assert!(!msg.contains("S3CR3T-USERINFO-9917"));
        assert!(!msg.contains("TOKEN-QUERY-5523"));
        assert!(!msg.contains("FRAG-SENTINEL-7788"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn invalid_limits_fail_before_spawn() {
        let dir = temp_dir("curl-invalid");
        let script = fake_curl_script(b"ok", "200", 0, &dir);
        let t = CurlTransport::with_executable(&script, CurlConfig::strict()).unwrap();
        let bad = FetchLimits {
            max_metadata_bytes: 0,
            ..limits()
        };
        let err = t
            .fetch_metadata(&req("http://example.com/x"), bad, &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
        // Fake was never invoked (no argv record beyond setup is acceptable;
        // assert no artifact side effects).
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(&req("http://example.com/x"), &dest, bad, &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::InvalidInput(_)));
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn composition_with_fixture_falls_back_from_unavailable_curl() {
        use eggup_acquisition::{ComposedTransport, CompositionPolicy};
        let dir = temp_dir("curl-composed");
        let script = dir.join("missing-curl");
        let curl = CurlTransport::with_executable(&script, CurlConfig::strict()).unwrap();
        let fixture = FixtureTransport::new();
        fixture.route(
            "https://example.com/meta",
            FixtureResponse::body(b"via-fixture".to_vec()),
        );
        let composed = ComposedTransport::new(&curl, &fixture, CompositionPolicy::UnavailableOnly);
        let out = composed
            .fetch_metadata(
                &req("https://example.com/meta"),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"via-fixture");
        let _ = fs::remove_dir_all(&dir);
    }
}
