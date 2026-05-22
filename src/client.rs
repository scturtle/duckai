use axum::http::{HeaderMap, HeaderValue};
use reqwest::{Client, header};
use std::time::Duration;
use typed_builder::TypedBuilder;

pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36";

#[derive(TypedBuilder, Clone)]
pub struct HttpConfig {
    /// Timeout for each request.
    timeout: u64,
    /// Timeout for each connecting.
    connect_timeout: u64,
    /// TCP keepalive interval.
    tcp_keepalive: Option<u64>,
}

/// Build a client
pub async fn build_client(config: HttpConfig) -> Client {
    let mut builder = Client::builder();

    // disable keep alive
    builder = match config.tcp_keepalive {
        Some(tcp_keepalive) => builder.tcp_keepalive(Duration::from_secs(tcp_keepalive)),
        None => builder.tcp_keepalive(None).pool_max_idle_per_host(0),
    };

    // headers
    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));

    builder
        // .impersonate(random_impersonate())
        .default_headers(headers)
        .cookie_store(true)
        .timeout(Duration::from_secs(config.timeout))
        .connect_timeout(Duration::from_secs(config.connect_timeout))
        .build()
        .expect("Failed to build Api client")
}
