//! A minimal HTTP/1.1 responder for offline wire tests (test-only).
//!
//! House style is fixture-based; these tests additionally need to see
//! real request lines/headers/bodies travel through reqwest, so this is
//! a `TcpListener` that answers one request per connection with a canned
//! `(status, json-body)` reply chosen by the test's closure — no new
//! dependencies. Requests are recorded before the response is written,
//! so by the time the client call returns the recorder is settled.

#![cfg(test)]

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// One recorded request, as it arrived.
#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    /// Path without the query string.
    pub path: String,
    pub query: String,
    /// Header names lowercased.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers.iter().find(|(n, _)| n == &name).map(|(_, v)| v.as_str())
    }

    pub fn body_json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).expect("request body is JSON")
    }
}

/// Test-supplied reply function: request → (status, json body).
type Responder = Arc<dyn Fn(&Recorded) -> (u16, String) + Send + Sync>;

/// Handle to a running test server. Dropping it stops the accept loop.
pub struct TestServer {
    pub base_url: String,
    requests: Arc<Mutex<Vec<Recorded>>>,
    task: tokio::task::JoinHandle<()>,
}

impl TestServer {
    /// Spawn on 127.0.0.1:0. `respond` maps each request to a
    /// `(status, json-body)` reply.
    pub async fn spawn(respond: Responder) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind loopback");
        let port = listener.local_addr().expect("local addr").port();
        let requests: Arc<Mutex<Vec<Recorded>>> = Arc::new(Mutex::new(Vec::new()));
        let task = tokio::spawn(accept_loop(listener, requests.clone(), respond));
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            requests,
            task,
        }
    }

    /// Snapshot of the requests seen so far.
    pub fn requests(&self) -> Vec<Recorded> {
        self.requests.lock().expect("requests lock").clone()
    }

    pub fn request_count(&self, path: &str) -> usize {
        self.requests().iter().filter(|r| r.path == path).count()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn accept_loop(listener: TcpListener, requests: Arc<Mutex<Vec<Recorded>>>, respond: Responder) {
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => return,
        };
        let requests = requests.clone();
        let respond = respond.clone();
        tokio::spawn(async move {
            if let Ok((stream, recorded)) = read_request(stream).await {
                let (status, body) = respond(&recorded);
                requests.lock().expect("requests lock").push(recorded);
                write_response(stream, status, &body).await.ok();
            }
        });
    }
}

/// Read one full HTTP/1.1 request head + body. Returns the (now
/// body-drained) stream for the reply.
async fn read_request(stream: TcpStream) -> std::io::Result<(TcpStream, Recorded)> {
    let mut reader = BufReader::new(stream);
    let mut method = String::new();
    let mut target = String::new();
    let mut headers = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no request"));
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if method.is_empty() {
            let mut parts = line.split_whitespace();
            method = parts.next().unwrap_or_default().to_string();
            target = parts.next().unwrap_or_default().to_string();
        } else if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
        }
    }
    for (name, value) in &headers {
        if name == "content-length" {
            content_length = value.parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).await?;
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };
    let recorded = Recorded {
        method,
        path,
        query,
        headers,
        body,
    };
    Ok((reader.into_inner(), recorded))
}

async fn write_response(mut stream: TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        _ => "Status",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await?;
    stream.shutdown().await
}
