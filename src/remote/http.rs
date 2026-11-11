//! # Just enough HTTP/1.1
//!
//! The console page is a handful of static files, plus one WebSocket upgrade. So the
//! server needs only to read a request line and headers, and to write a response with a
//! `Content-Length`. No keep-alive, no chunking, no request bodies: each file request
//! gets its answer and the connection closes.

/// A parsed request head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    /// The path without any `?query`.
    pub path: String,
    /// Header names lower-cased (HTTP header names are case-insensitive).
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    /// True for a WebSocket upgrade request (RFC 6455 §4.1).
    pub fn is_websocket(&self) -> bool {
        self.method == "GET"
            && self.header("upgrade").is_some_and(|u| u.eq_ignore_ascii_case("websocket"))
            && self.header("sec-websocket-key").is_some()
    }
}

/// Largest request head accepted; real browsers send well under 8 KB.
pub const MAX_HEAD: usize = 16 * 1024;

/// Parses a request head from `buf`. `Ok(None)` means the blank line that ends the head
/// has not arrived yet; the second value is the number of bytes the head used.
pub fn parse_request(buf: &[u8]) -> Result<Option<(Request, usize)>, String> {
    let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
        return if buf.len() > MAX_HEAD { Err("request head too large".into()) } else { Ok(None) };
    };
    let head = std::str::from_utf8(&buf[..end]).map_err(|_| "request head is not UTF-8")?;
    let mut lines = head.split("\r\n");
    let first = lines.next().unwrap_or("");
    let mut parts = first.split(' ');
    let (method, target, version) = (parts.next(), parts.next(), parts.next());
    let (Some(method), Some(target), Some(version)) = (method, target, version) else {
        return Err(format!("bad request line '{first}'"));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(format!("unsupported version '{version}'"));
    }
    let path = target.split('?').next().unwrap_or("/").to_string();
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    Ok(Some((Request { method: method.to_string(), path, headers }, end + 4)))
}

/// A complete response with a body.
pub fn response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body);
    out
}

/// A file, honouring a `Range: bytes=a-b` request with `206 Partial Content`. iOS Safari
/// fetches video in ranges and will not play a clip from a server that ignores them.
pub fn file_response(content_type: &str, body: &[u8], range: Option<&str>) -> Vec<u8> {
    let Some((start, end)) = range.and_then(|r| parse_range(r, body.len())) else {
        let mut out = response("200 OK", content_type, body);
        // Say ranges are welcome (inserted after the status line).
        let at = out.windows(2).position(|w| w == b"\r\n").map(|p| p + 2).unwrap_or(0);
        out.splice(at..at, b"Accept-Ranges: bytes\r\n".iter().copied());
        return out;
    };
    let part = &body[start..=end];
    let mut out = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Content-Range: bytes {start}-{end}/{}\r\nAccept-Ranges: bytes\r\nCache-Control: no-store\r\n\
         Connection: close\r\n\r\n",
        part.len(),
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(part);
    out
}

/// `bytes=a-b`, `bytes=a-` or `bytes=-n` (the last n bytes) → an inclusive byte range
/// within `len`, or `None` for anything else (then the whole file is sent).
fn parse_range(header: &str, len: usize) -> Option<(usize, usize)> {
    let spec = header.trim().strip_prefix("bytes=")?;
    let (a, b) = spec.split_once('-')?;
    if len == 0 || spec.contains(',') {
        return None; // several ranges at once: not needed here
    }
    let (start, end) = match (a.trim(), b.trim()) {
        ("", n) => (len.saturating_sub(n.parse().ok()?), len - 1),
        (s, "") => (s.parse().ok()?, len - 1),
        (s, e) => (s.parse().ok()?, e.parse::<usize>().ok()?.min(len - 1)),
    };
    (start <= end && start < len).then_some((start, end))
}

/// The `101 Switching Protocols` answer that turns the connection into a WebSocket.
pub fn upgrade_response(accept: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept}\r\n\r\n"
    )
    .into_bytes()
}

/// The media type for a file name's extension.
pub fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_websocket_upgrade_and_waits_for_the_blank_line() {
        let req = b"GET /ws?x=1 HTTP/1.1\r\nHost: 192.168.1.5:8765\r\nUpgrade: websocket\r\n\
                    Connection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\nextra";
        assert_eq!(parse_request(&req[..30]).unwrap(), None);
        let (r, used) = parse_request(req).unwrap().unwrap();
        assert_eq!((r.method.as_str(), r.path.as_str(), used), ("GET", "/ws", req.len() - 5));
        assert_eq!(r.header("host"), Some("192.168.1.5:8765"));
        assert!(r.is_websocket());
        let (plain, _) = parse_request(b"GET / HTTP/1.1\r\n\r\n").unwrap().unwrap();
        assert!(!plain.is_websocket());
    }

    #[test]
    fn bad_requests_are_errors_and_responses_are_well_formed() {
        assert!(parse_request(b"NONSENSE\r\n\r\n").is_err());
        assert!(parse_request(b"GET / SPDY/3\r\n\r\n").is_err());
        assert!(parse_request(&vec![b'a'; MAX_HEAD + 1]).is_err());
        let r = String::from_utf8(response("200 OK", content_type("app.js"), b"hi")).unwrap();
        assert!(
            r.starts_with("HTTP/1.1 200 OK\r\n") && r.contains("Content-Length: 2\r\n") && r.ends_with("\r\n\r\nhi")
        );
        assert!(String::from_utf8(upgrade_response("abc")).unwrap().contains("Sec-WebSocket-Accept: abc\r\n"));
        assert_eq!(content_type("x.css"), "text/css; charset=utf-8");
    }

    #[test]
    fn ranges_are_served_as_partial_content() {
        let body = b"0123456789";
        let text = |v: Vec<u8>| String::from_utf8(v).unwrap();
        let first2 = text(file_response("video/mp4", body, Some("bytes=0-1")));
        assert!(
            first2.starts_with("HTTP/1.1 206")
                && first2.contains("Content-Range: bytes 0-1/10")
                && first2.ends_with("01")
        );
        assert!(text(file_response("video/mp4", body, Some("bytes=7-"))).ends_with("\r\n\r\n789"));
        assert!(text(file_response("video/mp4", body, Some("bytes=-3"))).ends_with("\r\n\r\n789"));
        assert!(text(file_response("video/mp4", body, Some("bytes=5-99"))).contains("bytes 5-9/10"));
        let whole = text(file_response("video/mp4", body, None));
        assert!(whole.starts_with("HTTP/1.1 200 OK\r\nAccept-Ranges: bytes\r\n") && whole.ends_with("0123456789"));
        assert!(
            text(file_response("video/mp4", body, Some("bytes=20-30"))).starts_with("HTTP/1.1 200"),
            "out of range: whole file"
        );
    }
}
