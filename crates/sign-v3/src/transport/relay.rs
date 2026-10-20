//! Relay calls with their retry rules, the client JWT, the poll request,
//! and the poll config with local bounds.

use {
    super::{HttpClient, HttpRequest},
    crate::{
        protocol::{
            constants::JWT_LIFETIME_S,
            crypto::{random_bytes, sign},
            encoding::b64u,
            types::{Backoff, PollConfig},
        },
        rt,
    },
    ed25519_dalek::SigningKey,
    serde_json::json,
    std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum RelayError {
    #[error("{status} {error}")]
    Http { status: u16, error: String },
    #[error("network: {0}")]
    Network(String),
}

const CALL_TIMEOUT_MS: u64 = 10_000;

pub struct Relay {
    pub http: Arc<dyn HttpClient>,
    configured: Vec<String>,
    audience: String,
    hosts: Mutex<Vec<String>>,
    host_index: AtomicUsize,
    project_id: String,
    auth: SigningKey,
    jwt: Mutex<Option<(String, u64)>>,
    config: Mutex<PollConfig>,
    pub max_call_attempts: u32,
}

/// `did:key:z` + base58btc(0xed01 ‖ pub), as v2.
pub fn did_key(public: &[u8; 32]) -> String {
    let mut b = vec![0xed, 0x01];
    b.extend_from_slice(public);
    format!("did:key:z{}", bs58::encode(b).into_string())
}

impl Relay {
    pub fn new(
        http: Arc<dyn HttpClient>,
        hosts: Vec<String>,
        audience: Option<String>,
        project_id: String,
        auth: SigningKey,
    ) -> Self {
        Self {
            http,
            audience: audience
                .unwrap_or_else(|| hosts.first().cloned().unwrap_or_default()),
            hosts: Mutex::new(hosts.clone()),
            configured: hosts,
            host_index: AtomicUsize::new(0),
            project_id,
            auth,
            jwt: Mutex::new(None),
            config: Mutex::new(PollConfig::default()),
            max_call_attempts: 4,
        }
    }

    pub fn client_id(&self) -> String {
        did_key(&self.auth.verifying_key().to_bytes())
    }

    pub fn hosts(&self) -> Vec<String> {
        self.hosts.lock().expect("lock").clone()
    }

    pub fn host_index(&self) -> usize {
        self.host_index.load(Ordering::SeqCst)
    }

    pub fn set_host_index(&self, i: usize) {
        self.host_index.store(i, Ordering::SeqCst);
    }

    pub fn host(&self) -> String {
        let hosts = self.hosts.lock().expect("lock");
        hosts
            .get(self.host_index())
            .or(hosts.first())
            .cloned()
            .unwrap_or_default()
    }

    pub fn config(&self) -> PollConfig {
        self.config.lock().expect("lock").clone()
    }

    /// The client's connection JWT, as v2. Re-signed 60 s before expiry or when forced (401).
    pub fn jwt(&self, force: bool) -> String {
        let now = rt::now_ms() / 1000;
        let mut cache = self.jwt.lock().expect("lock");
        if let Some((token, exp)) = cache.as_ref()
            && !force
            && *exp > now + 60
        {
            return token.clone();
        }
        let exp = now + JWT_LIFETIME_S;
        let header = b64u(br#"{"alg":"EdDSA","typ":"JWT"}"#);
        let claims = json!({
            "iss": self.client_id(),
            "sub": hex::encode(random_bytes::<32>()),
            "aud": self.audience,
            "iat": now,
            "exp": exp,
        });
        let claims = b64u(claims.to_string().as_bytes());
        let input = format!("{header}.{claims}");
        let token =
            format!("{input}.{}", b64u(&sign(&self.auth, input.as_bytes())));
        *cache = Some((token.clone(), exp));
        token
    }

    pub fn url(&self, host: &str, path: &str) -> String {
        let sep = if path.contains('?') { '&' } else { '?' };
        format!("{host}{path}{sep}projectId={}", self.project_id)
    }

    pub fn backoff_ms(&self, attempt: u32) -> u64 {
        let Backoff { base_ms, max_ms } = self.config().backoff;
        let raw = base_ms.saturating_mul(1u64 << attempt.min(20)).min(max_ms);
        (raw as f64 * rt::jitter()) as u64
    }

    /// Calls 1-6: on the poller's host; network errors, 5xx and 429 retried with backoff or
    /// `Retry-After`; 401 re-signs the JWT once; any other 4xx is final. Retries send identical bytes.
    pub fn call<'a>(
        &'a self,
        method: &'static str,
        path: &'a str,
        body: Option<Vec<u8>>,
        headers: Vec<(String, String)>,
    ) -> rt::BoxFut<'a, Result<Vec<u8>, RelayError>> {
        rt::boxed(self.call_inner(method, path, body, headers))
    }

    async fn call_inner(
        &self,
        method: &'static str,
        path: &str,
        body: Option<Vec<u8>>,
        headers: Vec<(String, String)>,
    ) -> Result<Vec<u8>, RelayError> {
        let mut jwt_retried = false;
        let mut attempt = 0;
        loop {
            let mut h = vec![(
                "authorization".to_string(),
                format!("Bearer {}", self.jwt(false)),
            )];
            h.extend(headers.iter().cloned());
            let req = HttpRequest {
                method,
                url: self.url(&self.host(), path),
                headers: h,
                body: body.clone(),
                timeout_ms: Some(CALL_TIMEOUT_MS),
            };
            match self.http.request(req).await {
                Ok(res) if (200..300).contains(&res.status) => {
                    return Ok(res.body);
                }
                Ok(res) => {
                    let error = crate::protocol::ijson::value(&res.body)
                        .and_then(|v| {
                            v.get("error")
                                .and_then(|e| e.as_str())
                                .map(str::to_string)
                        })
                        .unwrap_or_else(|| "http_error".into());
                    if res.status == 401 && !jwt_retried {
                        jwt_retried = true;
                        self.jwt(true);
                        continue;
                    }
                    let retryable = res.status == 429 || res.status >= 500;
                    attempt += 1;
                    if !retryable || attempt >= self.max_call_attempts {
                        return Err(RelayError::Http {
                            status: res.status,
                            error,
                        });
                    }
                    let wait = res
                        .retry_after_s
                        .map(|s| s * 1000)
                        .unwrap_or_else(|| self.backoff_ms(attempt - 1));
                    rt::sleep_ms(wait).await;
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.max_call_attempts {
                        return Err(RelayError::Network(e.to_string()));
                    }
                    rt::sleep_ms(self.backoff_ms(attempt - 1)).await;
                }
            }
        }
    }

    pub fn poll_request(&self, host: &str, body: Vec<u8>) -> HttpRequest {
        HttpRequest {
            method: "POST",
            url: self.url(host, "/v3/events"),
            headers: vec![(
                "authorization".into(),
                format!("Bearer {}", self.jwt(false)),
            )],
            body: Some(body),
            timeout_ms: None,
        }
    }

    /// Applies the Relay's config within local bounds. A missing or invalid field keeps its previous
    /// value. Relay-provided hosts replace only the alternates and must be on an allowed host.
    pub fn apply_config(&self, raw: &serde_json::Value) {
        let mut c = self.config.lock().expect("lock");
        let num = |k: &str| raw.get(k).and_then(|v| v.as_u64());
        if let Some(v) = num("heartbeats_ms") {
            c.heartbeats_ms = v.clamp(1000, 20_000);
        }
        if let Some(v) = num("repoll_delay_ms") {
            c.repoll_delay_ms = v.min(60_000);
        }
        if let Some(v) = num("foreground_stale_ms") {
            c.foreground_stale_ms = v.clamp(2000, 60_000);
        }
        c.foreground_stale_ms =
            c.foreground_stale_ms.max(c.heartbeats_ms * 2).min(60_000);
        if let Some(b) = raw.get("backoff") {
            let base = b
                .get("base_ms")
                .and_then(|v| v.as_u64())
                .map(|v| v.clamp(10, 60_000))
                .unwrap_or(c.backoff.base_ms);
            let max = b
                .get("max_ms")
                .and_then(|v| v.as_u64())
                .map(|v| v.clamp(base, 300_000))
                .unwrap_or(c.backoff.max_ms.max(base));
            c.backoff = Backoff { base_ms: base, max_ms: max };
        }
        if let Some(list) = raw.get("hosts").and_then(|v| v.as_array()) {
            let offered: Vec<String> = list
                .iter()
                .filter_map(|h| h.as_str())
                .map(normalize_host)
                .filter(|h| self.allowed(h))
                .collect();
            c.hosts = offered.clone();
            let mut hosts = vec![self.configured[0].clone()];
            for h in self.configured.iter().skip(1).chain(offered.iter()) {
                if !hosts.contains(h) {
                    hosts.push(h.clone());
                }
            }
            *self.hosts.lock().expect("lock") = hosts;
        }
    }

    fn allowed(&self, host: &str) -> bool {
        if self.configured.iter().any(|h| h == host) {
            return true;
        }
        let Some(rest) = host.strip_prefix("https://") else { return false };
        let domain = rest.split(['/', ':']).next().unwrap_or("");
        domain.ends_with(".walletconnect.org")
            || domain.ends_with(".walletconnect.com")
            || domain == "walletconnect.org"
            || domain == "walletconnect.com"
    }
}

fn normalize_host(h: &str) -> String {
    let h = h.trim_end_matches('/');
    if h.contains("://") { h.to_string() } else { format!("https://{h}") }
}
