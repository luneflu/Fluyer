//! Loopback HTTP bridge for provider streams.
//!
//! macOS BASS has no TLS: `BASS_StreamCreateURL` on a `https://googlevideo…` URL blocks
//! forever, while `http://127.0.0.1` plays. So the core fetches over rustls and BASS
//! reads the same bytes from a socket on this machine. `Range` is forwarded, so seek
//! works the way it does for a local file.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A track URL is only valid for its own lifetime; entries are dropped well before the
/// CDN starts answering 403.
struct Route {
    url: String,
    expires_at: Instant,
}

pub struct StreamProxy {
    /// `http://127.0.0.1:<port>`, set once the listener is bound.
    base: Mutex<Option<String>>,
    routes: Mutex<HashMap<String, Route>>,
    client: reqwest::Client,
}

impl StreamProxy {
    pub fn new() -> Self {
        Self {
            base: Mutex::new(None),
            routes: Mutex::new(HashMap::new()),
            client: reqwest::Client::new(),
        }
    }

    /// Binds an ephemeral loopback port and serves it from `handle` for the process
    /// lifetime. Binding happens here (sync) so the port is known before any play call.
    pub fn spawn(self: &Arc<Self>, handle: &tokio::runtime::Handle) -> Result<(), String> {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))
            .map_err(|e| format!("stream proxy bind: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        *self.base.lock().unwrap() = Some(format!("http://127.0.0.1:{port}"));

        let proxy = Arc::clone(self);
        handle.spawn(async move {
            let Ok(listener) = TcpListener::from_std(listener) else { return };
            loop {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        let proxy = Arc::clone(&proxy);
                        tokio::spawn(async move {
                            let _ = proxy.serve(socket).await;
                        });
                    }
                    // Transient accept errors (fd pressure) must not kill the loop.
                    Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
                }
            }
        });
        Ok(())
    }

    /// Registers `url` under `id` and returns the loopback URL BASS should open.
    pub fn link(&self, id: &str, url: &str, expires_in_seconds: Option<u64>) -> Result<String, String> {
        let base = self.base.lock().unwrap().clone().ok_or("stream proxy not started")?;
        self.purge();
        // Leave a minute of headroom so a URL that is about to expire is not offered.
        let ttl = expires_in_seconds.unwrap_or(21_600).min(21_600).saturating_sub(60).max(60);
        self.routes.lock().unwrap().insert(
            id.to_string(),
            Route { url: url.to_string(), expires_at: Instant::now() + Duration::from_secs(ttl as u64) },
        );
        Ok(format!("{base}/{id}"))
    }

    fn purge(&self) {
        let now = Instant::now();
        self.routes.lock().unwrap().retain(|_, route| route.expires_at > now);
    }

    fn resolve(&self, id: &str) -> Option<String> {
        self.purge();
        self.routes.lock().unwrap().get(id).map(|route| route.url.clone())
    }

    async fn serve(&self, mut socket: TcpStream) -> std::io::Result<()> {
        let Some(head) = read_head(&mut socket).await? else { return Ok(()) };
        let mut lines = head.split("\r\n");
        let request_line = lines.next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("GET").to_string();
        let target = parts.next().unwrap_or("/").to_string();
        let id = target.trim_start_matches('/').split(['?', '#']).next().unwrap_or_default().to_string();
        let range = lines
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.eq_ignore_ascii_case("range"))
            .map(|(_, value)| value.trim().to_string());

        let Some(url) = self.resolve(&id) else {
            return reply(&mut socket, "404 Not Found", &[], b"").await;
        };

        let response = self
            .client
            .get(&url)
            .header("Range", range.unwrap_or_else(|| "bytes=0-".into()))
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(_) => return reply(&mut socket, "502 Bad Gateway", &[], b"").await,
        };

        let status = if response.status().is_success() {
            format!("{} {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or("OK"))
        } else {
            "502 Bad Gateway".to_string()
        };
        let mut headers: Vec<(String, String)> = Vec::new();
        for name in ["content-type", "content-length", "content-range"] {
            if let Some(value) = response.headers().get(name).and_then(|v| v.to_str().ok()) {
                headers.push((name.to_string(), value.to_string()));
            }
        }
        headers.push(("accept-ranges".into(), "bytes".into()));

        if method == "HEAD" {
            return reply(&mut socket, &status, &headers, b"").await;
        }

        // Header first, then the body straight from the CDN: BASS starts decoding while
        // the rest is still in flight.
        let mut stream = response;
        socket.write_all(render(&status, &headers).as_bytes()).await?;
        while let Some(chunk) = stream.chunk().await.map_err(std::io::Error::other)? {
            socket.write_all(&chunk).await?;
        }
        socket.flush().await
    }
}

impl Default for StreamProxy {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads up to the blank line that ends the request head.
async fn read_head(socket: &mut TcpStream) -> std::io::Result<Option<String>> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let read = socket.read(&mut chunk).await?;
        if read == 0 {
            return Ok(None);
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            return Ok(Some(String::from_utf8_lossy(&buffer).into_owned()));
        }
        if buffer.len() > 16 * 1024 {
            return Ok(None);
        }
    }
}

fn render(status: &str, headers: &[(String, String)]) -> String {
    let mut head = format!("HTTP/1.1 {status}\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    // One request per connection: BASS reopens on seek, and this keeps the loop simple.
    head.push_str("connection: close\r\n\r\n");
    head
}

async fn reply(
    socket: &mut TcpStream,
    status: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> std::io::Result<()> {
    socket.write_all(render(status, headers).as_bytes()).await?;
    if !body.is_empty() {
        socket.write_all(body).await?;
    }
    socket.flush().await
}
