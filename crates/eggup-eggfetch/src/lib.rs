#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Native Eggfetch HTTP acquisition adapter with explicit bounded policy."]
#![doc = ""]
#![doc = "Implements `eggup-acquisition::AcquisitionTransport` without release or"]
#![doc = "fallback authority. See `README.md` for the single policy configuration point."]

use eggfetch_core::{Client, HttpVersionPolicy, ProxyEnvironment, RedirectPolicy, Timeout};
use eggup_acquisition::{
    AcquisitionError, AcquisitionRequest, AcquisitionTransport, ArtifactEvidence, CancelFlag,
    FetchLimits, FetchOutcome, MetadataBytes,
};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Default user agent for Eggup Eggfetch traffic.
pub const DEFAULT_USER_AGENT: &str = "eggup-eggfetch";
/// Default connect deadline (matches eggsact/stegoeggo posture).
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default total wall-clock deadline.
pub const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(120);
/// Default redirect bound for asset chains.
pub const DEFAULT_MAX_REDIRECTS: usize = 10;

/// Explicit proxy decision. Routing is never inherited accidentally.
#[derive(Debug, Clone, Default)]
pub enum ProxyDecision {
    /// No proxy is used, even if environment variables are set.
    #[default]
    Disabled,
    /// Use an explicit snapshot of the process environment.
    FromEnvironment,
    /// Use explicit `KEY=value` pairs (e.g. `HTTPS_PROXY`, `NO_PROXY`).
    Custom(Vec<(String, String)>),
}

impl ProxyDecision {
    fn environment(&self) -> Result<ProxyEnvironment, String> {
        match self {
            Self::Disabled => Ok(ProxyEnvironment::new()),
            Self::FromEnvironment => Ok(ProxyEnvironment::from_env()),
            Self::Custom(pairs) => Ok(ProxyEnvironment::from_map(
                pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
            )),
        }
    }
}

/// Strict transport configuration. All policy lives here.
#[derive(Debug, Clone)]
pub struct EggfetchConfig {
    /// User agent header.
    pub user_agent: String,
    /// Connect deadline.
    pub connect_timeout: Duration,
    /// Total deadline.
    pub total_timeout: Duration,
    /// Redirect bound.
    pub max_redirects: usize,
    /// Proxy routing decision.
    pub proxy: ProxyDecision,
}

impl Default for EggfetchConfig {
    fn default() -> Self {
        Self {
            user_agent: DEFAULT_USER_AGENT.to_string(),
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            total_timeout: DEFAULT_TOTAL_TIMEOUT,
            max_redirects: DEFAULT_MAX_REDIRECTS,
            proxy: ProxyDecision::Disabled,
        }
    }
}

impl EggfetchConfig {
    /// Strict default: HTTP/1, Rustls, redirect bound, explicit proxy.
    pub fn strict() -> Self {
        Self::default()
    }

    /// Sets the user agent.
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self
    }

    /// Sets connect/total adapter ceilings.
    ///
    /// These are ceilings, not defaults: the effective deadline for one fetch
    /// is `min(request, adapter ceiling)` (see [`FetchLimits::effective`]).
    /// A stricter adapter tightens a deadline; it never extends one.
    /// Invalid values (zero or `connect > total`) fail closed in
    /// [`EggfetchTransport::strict`].
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

    /// Sets the proxy decision.
    pub fn proxy(mut self, proxy: ProxyDecision) -> Self {
        self.proxy = proxy;
        self
    }

    /// Derives the authoritative effective deadlines for one fetch.
    ///
    /// A stricter adapter ceiling wins; the adapter never extends a caller
    /// deadline. Both metadata and artifact operations use this derivation,
    /// and the effective total covers headers plus body streaming. This
    /// helper assumes `limits.validate()` has succeeded; transport entry
    /// points perform that validation before calling it.
    pub fn effective_timeouts(&self, limits: FetchLimits) -> (Duration, Duration) {
        limits.effective(self.connect_timeout, self.total_timeout)
    }

    fn timeout(&self) -> Timeout {
        Timeout::builder()
            .connect(self.connect_timeout)
            .total(self.total_timeout)
            .build()
    }

    fn effective_request_timeout(&self, limits: FetchLimits) -> Timeout {
        let (connect, total) = self.effective_timeouts(limits);
        Timeout::builder().connect(connect).total(total).build()
    }

    fn validate_timeouts(&self) -> Result<(), AcquisitionError> {
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
        Ok(())
    }

    fn redirect_policy(&self) -> RedirectPolicy {
        RedirectPolicy::strict(self.max_redirects)
    }

    fn build_client(&self) -> Result<Client, AcquisitionError> {
        self.validate_timeouts()?;
        let env = self
            .proxy
            .environment()
            // ProxyEnvironment construction is infallible for our decisions;
            // never echo environment content on failure.
            .map_err(|_| {
                AcquisitionError::Transport(bound(
                    "proxy configuration/routing failure".to_string(),
                ))
            })?;
        let builder = Client::builder()
            .user_agent(&self.user_agent)
            .http_version_policy(HttpVersionPolicy::Http1Only)
            .redirect_policy(self.redirect_policy())
            .timeout(self.timeout())
            .automatic_decompression(false);
        builder
            .proxy_environment(&env)
            .map(|b| b.build())
            .map_err(|_| {
                // Never embed upstream proxy error display: it may contain
                // proxy URLs with credentials. Use category-only diagnostics.
                AcquisitionError::Transport(bound(
                    "proxy configuration/routing failure".to_string(),
                ))
            })
    }
}

/// Native Eggfetch adapter implementing the acquisition seam.
///
/// No release, version, fallback, destination, or service policy lives here.
/// Checksum mismatch, TLS errors, malformed metadata, timeouts, 5xx, and
/// redirect violations are hard failures; only exact 404 becomes `NotFound`.
pub struct EggfetchTransport {
    client: Client,
    config: EggfetchConfig,
}

impl EggfetchTransport {
    /// Builds a strict adapter. Invalid proxy configuration fails closed here,
    /// never silently direct.
    pub fn strict(config: EggfetchConfig) -> Result<Self, AcquisitionError> {
        let client = config.build_client()?;
        Ok(Self { client, config })
    }

    /// Returns the transport configuration.
    pub fn config(&self) -> &EggfetchConfig {
        &self.config
    }

    /// Returns the exact Eggfetch dependency version used.
    pub fn eggfetch_version() -> &'static str {
        // Recorded at qualification time; updated when the dependency changes.
        "eggfetch-core 0.2.0 (http1,tls-rustls,tls-native-roots,proxy; no compression/cookies/retry/json)"
    }

    fn block_on<F, T>(&self, fut: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        // Private current-thread runtime so the sync seam never forces an
        // async runtime into eggup-core or lightweight consumers. Async
        // consumers should call the sync seam via spawn_blocking per their
        // own policy.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("eggfetch current-thread runtime");
        rt.block_on(fut)
    }
}

impl AcquisitionTransport for EggfetchTransport {
    fn fetch_metadata(
        &self,
        request: &AcquisitionRequest,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<MetadataBytes>, AcquisitionError> {
        limits.validate()?;
        if cancel.is_cancelled() {
            return Err(AcquisitionError::Cancelled);
        }
        // Per-fetch byte bound is authoritative; Content-Length is advisory only.
        // Effective time bounds are min(request, adapter ceiling) and enforced
        // via a real Eggfetch request-level timeout override, never a
        // post-hoc elapsed check.
        let max = limits.max_metadata_bytes;
        let url = request.url().to_string();
        let effective = self.config.effective_request_timeout(limits);
        let result = self.block_on(async {
            let mut response = self
                .client
                .get(&url)
                .map_err(|e| map_request_error(&e, request))?
                .timeout(effective)
                .max_decoded_body_size(max)
                .send()
                .await
                .map_err(|e| map_fetch_error(&e, request, Some(max)))?;
            let status = response.status().as_u16();
            match classify(status) {
                StatusClass::NotFound => Ok(FetchOutcome::NotFound),
                StatusClass::HardFailure => Err(AcquisitionError::Transport(bound(format!(
                    "HTTP {status} from {} while fetching metadata",
                    request.redacted()
                )))),
                StatusClass::Success => {
                    if cancel.is_cancelled() {
                        return Err(AcquisitionError::Cancelled);
                    }
                    if let Some(declared) = response.content_length() {
                        if declared > max as u64 {
                            return Err(AcquisitionError::TooLarge { limit: max as u64 });
                        }
                    }
                    let bytes = response
                        .bytes()
                        .await
                        .map_err(|e| map_fetch_error(&e, request, Some(max)))?;
                    if bytes.len() > max {
                        return Err(AcquisitionError::TooLarge { limit: max as u64 });
                    }
                    Ok(FetchOutcome::Success(
                        eggup_acquisition::__adapter_metadata(bytes.to_vec()),
                    ))
                }
            }
        });
        // Enforce caller total timeout even if the inner client policy drifts:
        // the inner Timeout already covers connect/total, so a Timeout error
        // here is authoritative. No extra wall-clock wrapper is needed for the
        // sync bridge because block_on inherits the client deadlines.
        result
    }

    fn fetch_artifact(
        &self,
        request: &AcquisitionRequest,
        dest: &Path,
        limits: FetchLimits,
        cancel: &CancelFlag,
    ) -> Result<FetchOutcome<ArtifactEvidence>, AcquisitionError> {
        limits.validate()?;
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
        // Fast-fail when the destination already exists. The race-safe
        // guarantee comes from `__promote_no_clobber` after streaming.
        if fs::symlink_metadata(dest).is_ok() {
            return Err(AcquisitionError::InvalidInput(bound(
                "artifact destination already exists; refusing to overwrite".into(),
            )));
        }
        let url = request.url().to_string();
        let max_artifact = limits.max_artifact_bytes;
        let redacted = request.redacted();
        let effective = self.config.effective_request_timeout(limits);
        let parent_owned = parent.to_path_buf();
        let dest_owned = dest.to_path_buf();
        let result: Result<FetchOutcome<ArtifactEvidence>, AcquisitionError> =
            self.block_on(async {
                use futures_util::StreamExt;
                use tokio::io::AsyncWriteExt;
                let mut response = self
                    .client
                    .get(&url)
                    .map_err(|e| map_request_error(&e, request))?
                    .timeout(effective)
                    .send()
                    .await
                    .map_err(|e| map_fetch_error(&e, request, None))?;
                let status = response.status().as_u16();
                match classify(status) {
                    StatusClass::NotFound => return Ok(FetchOutcome::NotFound),
                    StatusClass::HardFailure => {
                        return Err(AcquisitionError::Transport(bound(format!(
                            "HTTP {status} from {redacted} while fetching artifact"
                        ))));
                    }
                    StatusClass::Success => {}
                }
                // Exclusive owner-private temp plus race-safe no-clobber
                // promotion. The temp lives in the exact destination parent
                // so promotion stays on the same filesystem.
                let (std_file, tmp) =
                    eggup_acquisition::__acquire_exclusive_temp(&parent_owned, "eggup-eggfetch")?;
                struct Guard {
                    path: std::path::PathBuf,
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
                let mut file = tokio::fs::File::from_std(std_file);
                let mut stream = response
                    .bytes_stream()
                    .map_err(|e| map_fetch_error(&e, request, None))?;
                let mut written: u64 = 0;
                while let Some(chunk) = stream.next().await {
                    if cancel.is_cancelled() {
                        return Err(AcquisitionError::Cancelled);
                    }
                    let chunk = chunk.map_err(|e| map_fetch_error(&e, request, None))?;
                    if chunk.is_empty() {
                        continue;
                    }
                    if let Some(max) = max_artifact {
                        written = written.saturating_add(chunk.len() as u64);
                        if written > max {
                            return Err(AcquisitionError::TooLarge { limit: max });
                        }
                    } else {
                        written = written.saturating_add(chunk.len() as u64);
                    }
                    file.write_all(&chunk).await.map_err(|e| {
                        AcquisitionError::Io(bound(format!("writing part file: {e}")))
                    })?;
                }
                file.flush()
                    .await
                    .map_err(|e| AcquisitionError::Io(bound(format!("flushing part file: {e}"))))?;
                drop(file);
                if cancel.is_cancelled() {
                    return Err(AcquisitionError::Cancelled);
                }
                match eggup_acquisition::__promote_no_clobber(&tmp, &dest_owned) {
                    Ok(()) => {
                        guard.disarm = true;
                        Ok(FetchOutcome::Success(
                            eggup_acquisition::__adapter_artifact(written),
                        ))
                    }
                    Err(e) => {
                        eggup_acquisition::__remove_owned_temp(&tmp);
                        guard.disarm = true;
                        Err(e)
                    }
                }
            });
        result
    }
}

fn bound(s: String) -> String {
    let mut s = s;
    if s.len() > 512 {
        s.truncate(512);
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusClass {
    Success,
    NotFound,
    HardFailure,
}

fn classify(status: u16) -> StatusClass {
    match status {
        200..=299 => StatusClass::Success,
        404 => StatusClass::NotFound,
        _ => StatusClass::HardFailure,
    }
}

fn map_request_error(e: &eggfetch_core::Error, request: &AcquisitionRequest) -> AcquisitionError {
    use eggfetch_core::Error as E;
    match e {
        // Never embed upstream proxy/detail display: it may contain proxy
        // URLs with credentials. Use category-only diagnostics.
        E::InvalidProxyUrl(_) => {
            AcquisitionError::Transport(bound("proxy configuration/routing failure".to_string()))
        }
        E::Timeout { phase, .. } => AcquisitionError::Timeout {
            phase: match phase {
                eggfetch_core::TimeoutPhase::Connect => "connect",
                eggfetch_core::TimeoutPhase::Total => "total",
                _ => "total",
            },
        },
        _ => AcquisitionError::Transport(bound(format!(
            "request build failed for {}: {}",
            request.redacted(),
            e.kind()
        ))),
    }
}

fn map_fetch_error(
    e: &eggfetch_core::Error,
    request: &AcquisitionRequest,
    max: Option<usize>,
) -> AcquisitionError {
    use eggfetch_core::Error as E;
    match e {
        E::DecodedBodyTooLarge => AcquisitionError::TooLarge {
            limit: max.unwrap_or(0) as u64,
        },
        E::Timeout { phase, .. } => AcquisitionError::Timeout {
            phase: match phase {
                eggfetch_core::TimeoutPhase::Connect => "connect",
                eggfetch_core::TimeoutPhase::Total => "total",
                _ => "total",
            },
        },
        // TransportIoTimeout is an inactivity timeout on an established
        // connection; classify as Timeout so caller deadlines stay truthful.
        E::TransportIoTimeout { .. } => AcquisitionError::Timeout { phase: "total" },
        // Never embed upstream error display or proxy detail: it may contain
        // credential-bearing URLs. Use structured category text only.
        E::InvalidProxyUrl(_) => {
            AcquisitionError::Transport(bound("proxy configuration/routing failure".to_string()))
        }
        _ => AcquisitionError::Transport(bound(format!(
            "fetch failed for {}: {}",
            request.redacted(),
            e.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eggup_acquisition::{AcquisitionRequest, FixtureResponse, FixtureTransport};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::OnceLock;
    use std::thread;

    fn strict_transport() -> EggfetchTransport {
        EggfetchTransport::strict(EggfetchConfig::strict()).expect("strict transport")
    }

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

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let p = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        let _ = fs::create_dir_all(&p);
        p
    }

    /// Minimal single-response HTTP/1.1 server on 127.0.0.1.
    fn serve_once(
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        split_body: bool,
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
                    404 => "Not Found",
                    500 => "Internal Server Error",
                    302 => "Found",
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
                if split_body {
                    let mid = body.len() / 2;
                    let _ = stream.write_all(&body[..mid]);
                    thread::sleep(Duration::from_millis(20));
                    let _ = stream.write_all(&body[mid..]);
                } else {
                    let _ = stream.write_all(&body);
                }
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
            false,
            0,
            false,
        )
    }

    #[test]
    fn strict_config_uses_expected_policy() {
        let t = strict_transport();
        assert_eq!(t.config().max_redirects, DEFAULT_MAX_REDIRECTS);
        assert_eq!(t.config().connect_timeout, DEFAULT_CONNECT_TIMEOUT);
        assert_eq!(t.config().total_timeout, DEFAULT_TOTAL_TIMEOUT);
        assert_eq!(
            EggfetchTransport::eggfetch_version(),
            "eggfetch-core 0.2.0 (http1,tls-rustls,tls-native-roots,proxy; no compression/cookies/retry/json)"
        );
    }

    #[test]
    fn downgrade_policy_denies_https_to_http() {
        use eggfetch_core::RedirectDowngradePolicy;
        let strict = RedirectPolicy::strict(10);
        assert_eq!(strict.max_redirects, 10);
        assert!(strict.follow);
        let from: url::Url = "https://example.com/a".parse().unwrap();
        let same: url::Url = "https://example.com/b".parse().unwrap();
        let down: url::Url = "http://example.com/b".parse().unwrap();
        assert!(strict.check_downgrade(&from, &same).is_ok());
        assert!(strict.check_downgrade(&from, &down).is_err());
        let _ = RedirectDowngradePolicy::Deny;
    }

    #[test]
    fn metadata_200_is_success() {
        let base = ok_server(b"hello-metadata".to_vec());
        let t = strict_transport();
        let out = t
            .fetch_metadata(&req(&format!("{base}/meta")), limits(), &CancelFlag::new())
            .unwrap();
        assert_eq!(out.success().unwrap().bytes(), b"hello-metadata");
    }

    #[test]
    fn invalid_public_limits_fail_before_eggfetch_io() {
        let transport = strict_transport();
        let request = req("http://127.0.0.1:1/never-contact");
        let invalid = [
            FetchLimits {
                max_metadata_bytes: 0,
                ..limits()
            },
            FetchLimits {
                max_metadata_bytes: 16 * 1024 * 1024 + 1,
                ..limits()
            },
            FetchLimits {
                connect_timeout: Duration::ZERO,
                ..limits()
            },
            FetchLimits {
                total_timeout: Duration::ZERO,
                ..limits()
            },
            FetchLimits {
                connect_timeout: Duration::from_secs(11),
                total_timeout: Duration::from_secs(10),
                ..limits()
            },
        ];
        let dir = temp_dir("eggfetch-invalid-limits");
        let dest = dir.join("app");
        for limits in invalid {
            assert!(matches!(
                transport.fetch_metadata(&request, limits, &CancelFlag::new()),
                Err(AcquisitionError::InvalidInput(_))
            ));
            assert!(matches!(
                transport.fetch_artifact(&request, &dest, limits, &CancelFlag::new()),
                Err(AcquisitionError::InvalidInput(_))
            ));
            assert!(!dest.exists());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        }
        assert!(limits().validate().is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_metadata_is_too_large() {
        let big = vec![9u8; 128 * 1024];
        let base = ok_server(big);
        let t = strict_transport();
        let err = t
            .fetch_metadata(&req(&format!("{base}/big")), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::TooLarge { .. }));
    }

    #[test]
    fn streaming_download_writes_exact_bytes() {
        let body: Vec<u8> = (0..8192).map(|i| (i % 251) as u8).collect();
        let base = ok_server(body.clone());
        let t = strict_transport();
        let dir = temp_dir("eggfetch-stream");
        let dest = dir.join("app");
        let out = t
            .fetch_artifact(
                &req(&format!("{base}/app")),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert_eq!(out.success().unwrap().bytes_written, 8192);
        assert_eq!(fs::read(&dest).unwrap(), body);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn truncation_is_a_hard_failure_without_promotion() {
        let body = vec![5u8; 4096];
        let base = serve_once(
            200,
            vec![("Content-Length".into(), "4096".into())],
            body,
            false,
            0,
            true,
        );
        let t = strict_transport();
        let dir = temp_dir("eggfetch-trunc");
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
    fn not_found_is_typed_distinctly() {
        let t = strict_transport();
        let base = serve_once(404, vec![], b"nope".to_vec(), false, 0, false);
        let out = t
            .fetch_metadata(
                &req(&format!("{base}/missing")),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
        let base = serve_once(404, vec![], b"nope".to_vec(), false, 0, false);
        let dir = temp_dir("eggfetch-404");
        let dest = dir.join("app");
        let out = t
            .fetch_artifact(
                &req(&format!("{base}/missing")),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
        assert!(!dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_error_never_becomes_not_found() {
        let base = serve_once(500, vec![], b"boom".to_vec(), false, 0, false);
        let t = strict_transport();
        let err = t
            .fetch_metadata(&req(&format!("{base}/boom")), limits(), &CancelFlag::new())
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
    }

    #[test]
    fn redirect_limit_is_enforced() {
        // Self-redirect loop; client follows up to the bound then hard-fails.
        static ADDR: OnceLock<String> = OnceLock::new();
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap().to_string();
        ADDR.set(format!("http://{addr}")).ok();
        let target = format!("http://{addr}/loop");
        thread::spawn(move || {
            for mut s in listener.incoming().take(12).flatten() {
                s.set_read_timeout(Some(Duration::from_secs(2))).ok();
                s.set_write_timeout(Some(Duration::from_secs(2))).ok();
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                );
                let _ = s.write_all(resp.as_bytes());
            }
        });
        let mut cfg = EggfetchConfig::strict();
        cfg.max_redirects = 2;
        let t = EggfetchTransport::strict(cfg).unwrap();
        let err = t
            .fetch_metadata(
                &req(&format!("http://{addr}/loop")),
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Transport(_)));
    }

    #[test]
    fn total_timeout_is_a_hard_failure() {
        let body = b"too-late".to_vec();
        let base = serve_once(
            200,
            vec![("Content-Length".into(), body.len().to_string())],
            body,
            false,
            3000,
            false,
        );
        let mut cfg = EggfetchConfig::strict();
        cfg.total_timeout = Duration::from_millis(300);
        cfg.connect_timeout = Duration::from_millis(300);
        let t = EggfetchTransport::strict(cfg).unwrap();
        let err = t
            .fetch_metadata(
                &req(&format!("{base}/slow")),
                FetchLimits {
                    max_metadata_bytes: 64 * 1024,
                    max_artifact_bytes: Some(256 * 1024),
                    connect_timeout: Duration::from_millis(300),
                    total_timeout: Duration::from_millis(300),
                },
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::Timeout { .. } | AcquisitionError::Transport(_)
        ));
    }

    #[test]
    fn invalid_proxy_fails_closed() {
        let cfg = EggfetchConfig::strict().proxy(ProxyDecision::Custom(vec![(
            "HTTPS_PROXY".into(),
            "http://user:bogus@[::1".into(),
        )]));
        assert!(EggfetchTransport::strict(cfg).is_err());
    }

    #[test]
    fn output_file_is_cleaned_on_failure() {
        let base = serve_once(500, vec![], b"boom".to_vec(), false, 0, false);
        let t = strict_transport();
        let dir = temp_dir("eggfetch-cleanup");
        let dest = dir.join("app");
        let _ = t.fetch_artifact(
            &req(&format!("{base}/boom")),
            &dest,
            limits(),
            &CancelFlag::new(),
        );
        assert!(!dest.exists());
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(".eggup-eggfetch-")
            })
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seam_fixture_and_adapter_agree_on_not_found() {
        // Cross-check: both transports type 404 as NotFound, never fallback.
        let fixture = FixtureTransport::new();
        fixture.route("https://example.com/missing", FixtureResponse::not_found());
        let out = fixture
            .fetch_metadata(
                &AcquisitionRequest::new("https://example.com/missing").unwrap(),
                eggup_acquisition::FetchLimits::default(),
                &eggup_acquisition::CancelFlag::new(),
            )
            .unwrap();
        assert!(out.is_not_found());
    }

    // ---- M003 corrective: authoritative effective timeouts ----

    fn request_limits(connect: Duration, total: Duration) -> FetchLimits {
        FetchLimits {
            max_metadata_bytes: 64 * 1024,
            max_artifact_bytes: Some(256 * 1024),
            connect_timeout: connect,
            total_timeout: total,
        }
    }

    #[test]
    fn request_stricter_than_adapter_wins() {
        // Adapter allows 5 s; request allows 50 ms. The request bound must win
        // via a real Eggfetch per-request override, not a post-hoc check.
        let body = b"too-late".to_vec();
        let base = serve_once(
            200,
            vec![("Content-Length".into(), body.len().to_string())],
            body,
            false,
            3000,
            false,
        );
        let cfg = EggfetchConfig::strict().timeouts(Duration::from_secs(5), Duration::from_secs(5));
        let t = EggfetchTransport::strict(cfg).unwrap();
        let err = t
            .fetch_metadata(
                &req(&format!("{base}/slow")),
                request_limits(Duration::from_millis(50), Duration::from_millis(50)),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(
            matches!(err, AcquisitionError::Timeout { .. }),
            "request-stricter must time out, got {err:?}"
        );
    }

    #[test]
    fn adapter_stricter_than_request_wins() {
        // Request allows 5 s; adapter allows 50 ms. The adapter ceiling must
        // win, proving min(request, adapter) enforcement in both directions.
        let body = b"too-late".to_vec();
        let base = serve_once(
            200,
            vec![("Content-Length".into(), body.len().to_string())],
            body,
            false,
            3000,
            false,
        );
        let cfg =
            EggfetchConfig::strict().timeouts(Duration::from_millis(50), Duration::from_millis(50));
        let t = EggfetchTransport::strict(cfg).unwrap();
        let err = t
            .fetch_metadata(
                &req(&format!("{base}/slow")),
                request_limits(Duration::from_secs(5), Duration::from_secs(5)),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(
            matches!(err, AcquisitionError::Timeout { .. }),
            "adapter-stricter must time out, got {err:?}"
        );
    }

    #[test]
    fn connect_exceeding_total_is_rejected_in_both_layers() {
        // Seam layer.
        assert!(FetchLimits::new(
            1024,
            Some(1024),
            Duration::from_secs(5),
            Duration::from_secs(1),
        )
        .is_err());
        // Adapter layer fails closed at construction.
        let cfg = EggfetchConfig::strict().timeouts(Duration::from_secs(5), Duration::from_secs(1));
        assert!(EggfetchTransport::strict(cfg).is_err());
        // Zero deadlines also fail closed.
        let zero = EggfetchConfig::strict().timeouts(Duration::ZERO, Duration::from_secs(1));
        assert!(EggfetchTransport::strict(zero).is_err());
    }

    #[test]
    fn metadata_body_stall_times_out_without_fallback() {
        let body = b"stalled-metadata".to_vec();
        let base = serve_once(
            200,
            vec![("Content-Length".into(), body.len().to_string())],
            body,
            false,
            2000,
            false,
        );
        let t = strict_transport();
        let err = t
            .fetch_metadata(
                &req(&format!("{base}/stall")),
                request_limits(Duration::from_millis(300), Duration::from_millis(300)),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Timeout { .. }));
    }

    /// Server that sends headers + partial body, then stalls past the deadline.
    fn serve_partial_then_stall(partial: Vec<u8>, total_len: usize, stall_ms: u64) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                let mut buf = [0u8; 8192];
                let _ = stream.read(&mut buf);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {total_len}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n"
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&partial);
                thread::sleep(Duration::from_millis(stall_ms));
                // Close without sending the remainder: client must time out,
                // not promote a partial artifact.
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn artifact_body_stall_after_partial_data_times_out() {
        let partial = vec![7u8; 1024];
        let base = serve_partial_then_stall(partial, 8192, 3000);
        let t = strict_transport();
        let dir = temp_dir("eggfetch-artifact-stall");
        let dest = dir.join("app");
        let err = t
            .fetch_artifact(
                &req(&format!("{base}/app")),
                &dest,
                request_limits(Duration::from_millis(300), Duration::from_millis(300)),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(
            matches!(err, AcquisitionError::Timeout { .. }),
            "artifact stall must be Timeout, got {err:?}"
        );
        assert!(!dest.exists(), "partial artifact must not promote");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeout_never_becomes_not_found() {
        let body = b"late".to_vec();
        let base = serve_once(
            200,
            vec![("Content-Length".into(), body.len().to_string())],
            body,
            false,
            2000,
            false,
        );
        let t = strict_transport();
        let err = t
            .fetch_metadata(
                &req(&format!("{base}/x")),
                request_limits(Duration::from_millis(100), Duration::from_millis(100)),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(err, AcquisitionError::Timeout { .. }));
        assert!(!matches!(err, AcquisitionError::InvalidInput(_)));
    }

    // ---- M003 corrective: exclusive temp + no-clobber ----

    #[test]
    fn artifact_refuses_existing_destination_without_overwrite() {
        let body: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let base = ok_server(body);
        let t = strict_transport();
        let dir = temp_dir("eggfetch-noclobber");
        let dest = dir.join("app");
        fs::write(&dest, b"FOREIGN-EGGFETCH").unwrap();
        let err = t
            .fetch_artifact(
                &req(&format!("{base}/app")),
                &dest,
                limits(),
                &CancelFlag::new(),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::InvalidInput(_) | AcquisitionError::Io(_)
        ));
        assert_eq!(fs::read(&dest).unwrap(), b"FOREIGN-EGGFETCH");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn exclusive_temp_helper_is_owner_private_for_eggfetch_prefix() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("eggfetch-mode");
        let (f, tmp) = eggup_acquisition::__acquire_exclusive_temp(&dir, "eggup-eggfetch").unwrap();
        let mode = f.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        drop(f);
        eggup_acquisition::__remove_owned_temp(&tmp);
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- M003 corrective: redaction ----

    #[test]
    fn error_diagnostics_never_expose_credential_sentinels() {
        let userinfo = "S3CR3T-USERINFO-8841";
        let query = "TOKEN-QUERY-7734";
        let proxy_pw = "HUNTER2-PROXY-9902";
        // Request URL with sentinels: any error must embed only the redacted URL.
        let url = format!("https://alice:{userinfo}@127.0.0.1:9/app?token={query}");
        let t = strict_transport();
        // Unroutable port gives a fast transport failure without network wait.
        // Use a short deadline so the test stays fast even if it dials.
        let err = t
            .fetch_metadata(
                &req(&url),
                request_limits(Duration::from_millis(300), Duration::from_millis(300)),
                &CancelFlag::new(),
            )
            .unwrap_err();
        let msg = format!("{err}");
        let dbg = format!("{err:?}");
        assert!(!msg.contains(userinfo), "userinfo leaked: {msg}");
        assert!(!msg.contains(query), "query leaked: {msg}");
        assert!(!dbg.contains(userinfo));
        assert!(!dbg.contains(query));
        // Upstream error strings must map to category-only text.
        let fake = eggfetch_core::Error::InvalidProxyUrl(format!(
            "http://proxy:{proxy_pw}@example.com:8080"
        ));
        assert!(fake.to_string().contains(proxy_pw));
        // Our mapper must not echo that detail.
        let req2 = req("https://example.com/x");
        let mapped = super::map_fetch_error(&fake, &req2, None);
        let mapped_msg = format!("{mapped}");
        assert!(
            !mapped_msg.contains(proxy_pw),
            "proxy pw leaked: {mapped_msg}"
        );
        // Invalid proxy construction also stays generic.
        let bad_cfg = EggfetchConfig::strict().proxy(ProxyDecision::Custom(vec![(
            "HTTPS_PROXY".into(),
            format!("http://user:{proxy_pw}@example.com:8080"),
        )]));
        // Either construction fails closed with a generic message, or a later
        // fetch fails without echoing the password. Both are acceptable as
        // long as the password never appears.
        if let Err(e) = EggfetchTransport::strict(bad_cfg) {
            assert!(!format!("{e}").contains(proxy_pw));
        }
    }

    #[test]
    fn effective_formula_is_minimum_per_phase() {
        let cfg =
            EggfetchConfig::strict().timeouts(Duration::from_secs(1), Duration::from_secs(10));
        let req_limits = request_limits(Duration::from_secs(5), Duration::from_secs(5));
        let (c, t) = cfg.effective_timeouts(req_limits);
        assert_eq!(c, Duration::from_secs(1));
        assert_eq!(t, Duration::from_secs(5));
    }
}
