use reqwest::header::CONTENT_TYPE;
use reqwest::{redirect::Policy, Client};
use std::time::Duration;

const USER_AGENT: &str = concat!("webfetch/", env!("CARGO_PKG_VERSION"));
const MAX_ATTEMPTS: u32 = 3;

/// Outcome of an HTTP fetch: the body, the URL we actually landed on after
/// following redirects, and the response's `Content-Type` (if any).
pub struct FetchedPage {
    pub body: String,
    pub final_url: String,
    pub content_type: Option<String>,
}

fn build_client(timeout_secs: u64) -> anyhow::Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(Policy::limited(5))
        .user_agent(USER_AGENT)
        .gzip(true)
        .brotli(true)
        .build()?)
}

/// One request attempt. The bool in the error reports whether the failure is
/// transient (worth retrying): connection/timeout errors, 5xx, and 429.
async fn attempt(client: &Client, url: &str) -> Result<FetchedPage, (anyhow::Error, bool)> {
    let resp = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            let transient = e.is_timeout() || e.is_connect() || e.is_request();
            return Err((e.into(), transient));
        }
    };

    let status = resp.status();
    let resp = match resp.error_for_status() {
        Ok(r) => r,
        Err(e) => {
            let transient = status.is_server_error() || status.as_u16() == 429;
            return Err((e.into(), transient));
        }
    };

    let final_url = resp.url().to_string();
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match resp.text().await {
        Ok(body) => Ok(FetchedPage {
            body,
            final_url,
            content_type,
        }),
        Err(e) => {
            let transient = e.is_timeout();
            Err((e.into(), transient))
        }
    }
}

/// Fetch a URL, following redirects, retrying transient failures with
/// exponential backoff (200ms, 400ms).
pub async fn fetch_page(url: &str, timeout_secs: u64) -> anyhow::Result<FetchedPage> {
    let client = build_client(timeout_secs)?;

    let mut delay = Duration::from_millis(200);
    for attempt_no in 1..=MAX_ATTEMPTS {
        match attempt(&client, url).await {
            Ok(page) => return Ok(page),
            Err((err, transient)) => {
                if attempt_no == MAX_ATTEMPTS || !transient {
                    return Err(err);
                }
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
    unreachable!("loop returns on the final attempt")
}
