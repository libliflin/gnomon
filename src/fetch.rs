use std::time::{Duration, Instant};

use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use serde::Serialize;
use tokio::sync::Semaphore;
use url::Url;

use std::sync::Arc;

/// What came back from a single HTTP request. Both the raw (wire) body and our
/// recomputed brotli size are captured — we don't trust whatever a CDN
/// negotiated at the moment we looked.
#[derive(Debug, Clone, Serialize)]
pub struct Fetched {
    pub url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub wire_bytes: u64,
    pub raw_bytes: u64,
    pub brotli_bytes: u64,
    pub error: Option<String>,
    #[serde(skip)]
    pub body: Option<Vec<u8>>,
}

pub struct Fetcher {
    client: Client,
    max_concurrent: usize,
    timeout: Duration,
}

impl Fetcher {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .user_agent(concat!("gnomon/", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::limited(6))
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            client,
            max_concurrent: 16,
            timeout: Duration::from_secs(20),
        })
    }

    /// Fetch the root HTML, keeping the body. Brotli size is recomputed here
    /// too so the budget check never trusts a CDN's `content-length`.
    pub async fn fetch_root(&self, url: &Url) -> anyhow::Result<Fetched> {
        let started = Instant::now();
        let resp = self.client.get(url.as_str()).send().await?;
        let status = resp.status().as_u16();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body = resp.bytes().await?;
        let raw = body.len() as u64;
        let brotli = brotli_size(&body);
        let _ = started; // reserved for per-request timing later
        Ok(Fetched {
            url: url.to_string(),
            status,
            content_type,
            wire_bytes: raw, // best-effort: reqwest decompressed for us; treat as raw
            raw_bytes: raw,
            brotli_bytes: brotli,
            error: None,
            body: Some(body.to_vec()),
        })
    }

    /// Fan out to fetch many resources concurrently. Errors become `Fetched`
    /// rows with `error` set, not exceptions — the audit must continue so
    /// the report is complete.
    pub async fn fetch_many(&self, urls: Vec<Url>, keep_body: bool) -> Vec<Fetched> {
        if urls.is_empty() {
            return Vec::new();
        }
        let sem = Arc::new(Semaphore::new(self.max_concurrent));
        let mut futs = FuturesUnordered::new();
        for url in urls {
            let client = self.client.clone();
            let sem = sem.clone();
            let timeout = self.timeout;
            futs.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.ok();
                fetch_one(&client, &url, timeout, keep_body).await
            }));
        }

        let mut out = Vec::with_capacity(futs.len());
        while let Some(join) = futs.next().await {
            match join {
                Ok(f) => out.push(f),
                Err(e) => out.push(Fetched {
                    url: String::new(),
                    status: 0,
                    content_type: None,
                    wire_bytes: 0,
                    raw_bytes: 0,
                    brotli_bytes: 0,
                    error: Some(format!("join error: {e}")),
                    body: None,
                }),
            }
        }
        out
    }
}

async fn fetch_one(client: &Client, url: &Url, timeout: Duration, keep_body: bool) -> Fetched {
    let url_s = url.to_string();
    let res = tokio::time::timeout(timeout, client.get(url.as_str()).send()).await;
    let resp = match res {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return err_row(url_s, format!("{e}")),
        Err(_) => return err_row(url_s, "timeout".into()),
    };
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = match tokio::time::timeout(timeout, resp.bytes()).await {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => return err_row(url_s, format!("body: {e}")),
        Err(_) => return err_row(url_s, "body timeout".into()),
    };
    let raw = body.len() as u64;
    let brotli = brotli_size(&body);

    Fetched {
        url: url_s,
        status,
        content_type,
        wire_bytes: raw,
        raw_bytes: raw,
        brotli_bytes: brotli,
        error: None,
        body: if keep_body { Some(body.to_vec()) } else { None },
    }
}

fn err_row(url: String, msg: String) -> Fetched {
    Fetched {
        url,
        status: 0,
        content_type: None,
        wire_bytes: 0,
        raw_bytes: 0,
        brotli_bytes: 0,
        error: Some(msg),
        body: None,
    }
}

/// Recompute brotli-compressed size. We don't keep the output, only count bytes.
fn brotli_size(input: &[u8]) -> u64 {
    use std::io::Write;
    let mut sink = CountingWriter::default();
    {
        let params = brotli::enc::BrotliEncoderParams {
            quality: 5, // fast; we want an answer, not the global optimum
            ..Default::default()
        };
        let mut w = brotli::CompressorWriter::with_params(&mut sink, 4096, &params);
        if w.write_all(input).is_err() {
            return 0;
        }
        if w.flush().is_err() {
            return 0;
        }
    }
    sink.count
}

#[derive(Default)]
struct CountingWriter {
    count: u64,
}

impl std::io::Write for CountingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.count += buf.len() as u64;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
