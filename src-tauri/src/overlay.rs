//! The optional OBS overlay: a browser source at http://127.0.0.1:38517/overlay shows a small
//! indicator while the AI talks. Plain std TCP: two fixed responses do not need an HTTP library.

use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::Phase;

pub const OVERLAY_PORT: u16 = 38517;
pub const OVERLAY_URL: &str = "http://127.0.0.1:38517/overlay";

/// Binds loopback only (nothing on the network sees it) and answers on its own thread.
pub fn serve(port: u16, phase: Arc<Mutex<Phase>>) -> io::Result<SocketAddr> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let addr = listener.local_addr()?;
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let phase = phase.clone();
            std::thread::spawn(move || {
                let _ = answer(stream, &phase);
            });
        }
    });
    Ok(addr)
}

fn answer(mut stream: TcpStream, phase: &Mutex<Phase>) -> io::Result<()> {
    // A client that connects and sends nothing must not keep a thread forever.
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
    let mut loopback = false;
    let mut header = String::new();
    while reader.read_line(&mut header)? > 0 && header.trim() != "" {
        if let Some((name, value)) = header.split_once(':') {
            if name.trim().eq_ignore_ascii_case("host") {
                let host = value.trim().rsplit_once(':').map_or(value.trim(), |(h, _)| h);
                loopback = host == "127.0.0.1" || host.eq_ignore_ascii_case("localhost");
            }
        }
        header.clear();
    }
    let (status, kind, body) = match path.as_str() {
        _ if !loopback => ("403 Forbidden", "text/plain", "forbidden".to_string()),
        "/overlay" => ("200 OK", "text/html; charset=utf-8", include_str!("../../ui/overlay.html").to_string()),
        "/state" => {
            let name = match *phase.lock().unwrap() {
                Phase::Idle => "idle",
                Phase::Starting => "starting",
                Phase::Listening => "listening",
                Phase::Error(_) => "error",
            };
            ("200 OK", "application/json", format!(r#"{{"phase":"{name}"}}"#))
        }
        _ => ("404 Not Found", "text/plain", "not found".to_string()),
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    fn get(addr: SocketAddr, path: &str) -> String {
        let mut s = TcpStream::connect(addr).unwrap();
        write!(s, "GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    fn start(phase: Phase) -> SocketAddr {
        serve(0, Arc::new(Mutex::new(phase))).unwrap()
    }

    #[test]
    fn serves_state_json() {
        let resp = get(start(Phase::Listening), "/state");
        assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
        assert!(resp.contains("Cache-Control: no-store"), "{resp}");
        assert!(resp.ends_with(r#"{"phase":"listening"}"#), "{resp}");
    }

    #[test]
    fn serves_overlay_html() {
        let resp = get(start(Phase::Idle), "/overlay");
        assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
        assert!(resp.contains("text/html"), "{resp}");
        assert!(resp.contains("/state"), "{resp}");
    }

    fn get_with_host(addr: SocketAddr, host: &str) -> String {
        let mut s = TcpStream::connect(addr).unwrap();
        write!(
            s,
            "GET /state HTTP/1.1
Host: {host}

"
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn foreign_host_refused() {
        // A web page could point its own name at 127.0.0.1 (DNS rebinding); only loopback names are served.
        let addr = start(Phase::Idle);
        assert!(get_with_host(addr, "evil.test").starts_with("HTTP/1.1 403"));
        assert!(get_with_host(addr, &format!("127.0.0.1:{}", addr.port())).starts_with("HTTP/1.1 200"));
        assert!(get_with_host(addr, "localhost:38517").starts_with("HTTP/1.1 200"));
    }

    #[test]
    fn unknown_path_404() {
        assert!(get(start(Phase::Idle), "/secret").starts_with("HTTP/1.1 404"));
    }

    #[test]
    fn binds_loopback_only() {
        assert!(start(Phase::Idle).ip().is_loopback());
    }
}
