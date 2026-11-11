//! # The console server: threads, pairing, and the game-facing queue
//!
//! The game loop must never wait on the network, so the server runs on its own threads
//! and talks to the game only through *channels* (`std::sync::mpsc`). A channel is a
//! queue with a sending end and a receiving end that can live on different threads; the
//! ends are what cross the thread boundary, so no data is shared and nothing needs a lock
//! except the small list of connected clients.
//!
//! ```text
//!  accept thread ──spawns──▶ one thread per connection
//!                              │  reads frames → Event ──▶ game: ConsoleServer::poll()
//!                              ◀── frames to send ─────── game: ConsoleServer::broadcast()
//! ```
//!
//! Safety rules (R-CON-009): off unless enabled; only peers on the local network (private,
//! link-local or loopback addresses) are served; a WebSocket client must send the 6-digit
//! pairing code shown in the game before any action is accepted.

use super::{http, protocol, ws};
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The default port (R-CON-009); `console: {port: …}` in the settings changes it.
pub const DEFAULT_PORT: u16 = 8765;

/// What the server tells the game.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A console entered the right code; the game should send it the catalogue.
    Paired(u32),
    Input(u32, protocol::Inbound),
    /// A console disconnected (its held actions should be released).
    Left(u32),
}

/// One connected console, as the game side sees it.
struct Client {
    id: u32,
    paired: Arc<AtomicBool>,
    /// Frames to write; the connection thread drains it.
    out: Sender<Vec<u8>>,
}

/// The running server. Dropping it stops accepting new consoles.
pub struct ConsoleServer {
    pub port: u16,
    /// The pairing code shown in the game.
    pub code: String,
    /// The address to open on the tablet, e.g. `http://192.168.1.20:8765`.
    pub url: String,
    events: Receiver<Event>,
    clients: Arc<Mutex<Vec<Client>>>,
    stop: Arc<AtomicBool>,
}

/// The files of the console page, built into the program so it needs nothing on disk.
/// `include_bytes!` embeds a file's bytes in the program at compile time.
const FILES: &[(&str, &[u8])] = &[
    ("/index.html", include_bytes!("../../web/console/index.html")),
    ("/console.js", include_bytes!("../../web/console/console.js")),
    ("/console.css", include_bytes!("../../web/console/console.css")),
    // A tiny black clip the page plays muted to keep a tablet awake (see the tool that
    // makes it: tools/console_assets/make_keepawake.swift).
    ("/keepawake.mp4", include_bytes!("../../web/console/keepawake.mp4")),
];

impl ConsoleServer {
    /// Starts listening on `port` on every interface (peers are filtered per
    /// connection). `seed` picks the pairing code.
    pub fn start(port: u16, seed: u64) -> Result<ConsoleServer, String> {
        let listener = TcpListener::bind(("0.0.0.0", port))
            .map_err(|e| format!("console: port {port} is not available ({e}); set console.port in settings.yaml"))?;
        // Non-blocking accept, polled with a short sleep, so the thread can notice `stop`.
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let code = pairing_code(seed);
        let (tx, events) = channel();
        let clients: Arc<Mutex<Vec<Client>>> = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (c, s, k) = (clients.clone(), stop.clone(), code.clone());
        std::thread::Builder::new()
            .name("console-accept".into())
            .spawn(move || accept_loop(listener, k, tx, c, s))
            .map_err(|e| e.to_string())?;
        let host = local_address().map(|i| i.to_string()).unwrap_or_else(|| "this-computer".into());
        Ok(ConsoleServer { port, code, url: format!("http://{host}:{port}"), events, clients, stop })
    }

    /// Everything the consoles said since the last call. Never blocks.
    pub fn poll(&self) -> Vec<Event> {
        self.events.try_iter().collect()
    }

    /// Sends a text message to every paired console.
    pub fn broadcast(&self, text: &str) {
        let frame = ws::encode(ws::Opcode::Text, text.as_bytes());
        // A poisoned lock means a connection thread panicked while holding it; the list
        // itself is still usable, so carry on with it.
        let mut clients = self.clients.lock().unwrap_or_else(|e| e.into_inner());
        // A failed send means that connection's thread has ended: forget the client.
        clients.retain(|c| !c.paired.load(Ordering::Relaxed) || c.out.send(frame.clone()).is_ok());
    }

    /// Sends a text message to one console.
    pub fn send(&self, id: u32, text: &str) {
        let clients = self.clients.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(c) = clients.iter().find(|c| c.id == id) {
            let _ = c.out.send(ws::encode(ws::Opcode::Text, text.as_bytes()));
        }
    }

    /// How many consoles are paired.
    pub fn paired(&self) -> usize {
        self.clients
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|c| c.paired.load(Ordering::Relaxed))
            .count()
    }
}

impl Drop for ConsoleServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// A 6-digit code from `seed` (xorshift; the code only has to be unguessable to someone
/// on the same network who cannot see the screen).
pub fn pairing_code(seed: u64) -> String {
    let mut x = seed | 1;
    for _ in 0..4 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
    }
    format!("{:06}", x % 1_000_000)
}

/// True for addresses on the local network: private, link-local, unique-local or
/// loopback. Anything else is refused.
pub fn is_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_link_local() || v4.is_loopback(),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_local(IpAddr::V4(v4));
            }
            let first = v6.segments()[0];
            v6.is_loopback() || (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
        }
    }
}

/// This machine's address on the local network, for the in-game hint ("open
/// http://192.168.1.20:8765"). Connecting a UDP socket sends nothing; it only asks the
/// system which interface would be used.
pub fn local_address() -> Option<IpAddr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?; // TEST-NET-1: a documentation address, never routed
    s.local_addr().ok().map(|a| a.ip())
}

fn accept_loop(
    listener: TcpListener,
    code: String,
    events: Sender<Event>,
    clients: Arc<Mutex<Vec<Client>>>,
    stop: Arc<AtomicBool>,
) {
    let next_id = Arc::new(AtomicU32::new(1));
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, peer)) if is_local(peer.ip()) => {
                let (code, events, clients, next_id) = (code.clone(), events.clone(), clients.clone(), next_id.clone());
                let _ = std::thread::Builder::new()
                    .name("console-conn".into())
                    .spawn(move || serve(stream, &code, events, clients, &next_id));
            }
            Ok(_) => {} // not on the local network: dropped without a word
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => std::thread::sleep(Duration::from_millis(200)),
        }
    }
}

/// Handles one connection: a file request, or a WebSocket for the rest of its life.
fn serve(
    mut stream: TcpStream,
    code: &str,
    events: Sender<Event>,
    clients: Arc<Mutex<Vec<Client>>>,
    next_id: &AtomicU32,
) {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let (req, used) = loop {
        match http::parse_request(&buf) {
            Ok(Some(r)) => break r,
            Ok(None) => {}
            Err(_) => return,
        }
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    };
    if !req.is_websocket() {
        let path = if req.path == "/" { "/index.html" } else { req.path.as_str() };
        let reply = match FILES.iter().find(|(p, _)| *p == path) {
            Some((p, body)) => http::file_response(http::content_type(p), body, req.header("range")),
            None => http::response("404 Not Found", "text/plain", b"not found"),
        };
        let _ = stream.write_all(&reply);
        return;
    }
    let key = req.header("sec-websocket-key").unwrap_or_default();
    if stream.write_all(&http::upgrade_response(&ws::accept_key(key))).is_err() {
        return;
    }
    let id = next_id.fetch_add(1, Ordering::Relaxed);
    let paired = Arc::new(AtomicBool::new(false));
    let (out_tx, out_rx) = channel::<Vec<u8>>();
    clients.lock().unwrap_or_else(|e| e.into_inner()).push(Client { id, paired: paired.clone(), out: out_tx });
    buf.drain(..used);
    // Short read timeouts turn the blocking socket into a polling loop: read whatever
    // arrived, handle it, write whatever the game queued, repeat.
    let _ = stream.set_read_timeout(Some(Duration::from_millis(15)));
    // A write that cannot complete in 2 s means the tablet is gone: end the connection
    // rather than block this thread (and keep the console's buttons held) indefinitely.
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let mut failures = 0u32;
    'conn: loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
        loop {
            let (frame, n) = match ws::decode(&buf) {
                Ok(Some(f)) => f,
                Ok(None) => break,
                Err(_) => break 'conn,
            };
            buf.drain(..n);
            match frame.opcode {
                ws::Opcode::Close => {
                    let _ = stream.write_all(&ws::encode(ws::Opcode::Close, &[]));
                    break 'conn;
                }
                ws::Opcode::Ping => {
                    let _ = stream.write_all(&ws::encode(ws::Opcode::Pong, &frame.payload));
                }
                ws::Opcode::Text => {
                    let text = String::from_utf8_lossy(&frame.payload);
                    match protocol::decode(&text) {
                        Ok(protocol::Inbound::Hello { code: c }) if c == code => {
                            paired.store(true, Ordering::Relaxed);
                            // Answer at once, from this thread: the page knows it is paired
                            // even if the game is busy (or between screens).
                            let welcome = format!(r#"{{"type":"welcome","version":{}}}"#, protocol::VERSION);
                            let _ = stream.write_all(&ws::encode(ws::Opcode::Text, welcome.as_bytes()));
                            let _ = events.send(Event::Paired(id));
                        }
                        Ok(protocol::Inbound::Hello { .. }) => {
                            failures += 1;
                            let msg = r#"{"type":"denied","reason":"wrong pairing code"}"#;
                            let _ = stream.write_all(&ws::encode(ws::Opcode::Text, msg.as_bytes()));
                            // Guessing is slow: three wrong codes end the connection.
                            if failures >= 3 {
                                break 'conn;
                            }
                        }
                        Ok(input) if paired.load(Ordering::Relaxed) => {
                            let _ = events.send(Event::Input(id, input));
                        }
                        _ => {} // unpaired or malformed: ignored
                    }
                }
                _ => {}
            }
        }
        for frame in out_rx.try_iter() {
            if stream.write_all(&frame).is_err() {
                break 'conn;
            }
        }
    }
    clients.lock().unwrap_or_else(|e| e.into_inner()).retain(|c| c.id != id);
    let _ = events.send(Event::Left(id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    /// Not a unit test: serves the console page on 127.0.0.1:8790 (code from seed 7) for
    /// 20 s, answering like the launcher does, so the real page can be tried in a browser:
    /// `cargo test serve_for_a_browser -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn serve_for_a_browser() {
        let server = ConsoleServer::start(8790, 7).unwrap();
        println!("http://127.0.0.1:8790/?code={}", server.code);
        let end = std::time::Instant::now() + Duration::from_secs(60);
        while std::time::Instant::now() < end {
            for e in server.poll() {
                println!("event {e:?}");
                if let Event::Paired(_) = e {
                    server.broadcast(&protocol::catalogue_idle());
                    // A sample target model for the viewport: a dart-like wedge.
                    let mesh = crate::formats::obj::parse(
                        "v 0 0 -2\nv -1 0 1\nv 1 0 1\nv 0 0.5 1\nf 1 3 2\nf 1 2 4\nf 1 4 3\nf 2 3 4\n",
                    )
                    .unwrap();
                    let mut m = crate::model::Model::from_obj("sample", &mesh, 1.0, &[]);
                    // Textured with a generated checkerboard (exercises the uvs and image path).
                    m.lods[0].groups[0].texture = Some("checker".into());
                    for (k, v) in m.lods[0].vertices.iter_mut().enumerate() {
                        v.uv = glam::Vec2::new((k % 2) as f32 * 2.0, (k / 2 % 2) as f32 * 2.0);
                    }
                    let checker: Vec<u8> = (0..64 * 64)
                        .flat_map(|i| {
                            if (i % 64 / 8 + i / 64 / 8) % 2 == 0 {
                                [230, 60, 40, 255]
                            } else {
                                [240, 220, 60, 255]
                            }
                        })
                        .collect();
                    let url = protocol::texture_url(64, 64, &checker);
                    server.broadcast(&protocol::model("Wraith", &m, &[[0.6, 0.75, 0.6]], &[Some(url)]));
                    // The default campaign's theme, from its rules file (the art-deco look).
                    if let Ok(c) = crate::content::Content::load(&crate::content::ContentConfig {
                        pack_dirs: crate::content::checkout_dirs(),
                        no_original: true,
                        campaign: Some("default".into()),
                        ..Default::default()
                    }) {
                        server.broadcast(&protocol::theme(&c.rules.console.base));
                    }
                }
            }
            // A sample of in-flight telemetry, so every panel has something to show.
            server.broadcast(concat!(
                r#"{"type":"telemetry","time":12,"alive":true,"hits":0,"hull_hits":0,"craft":"Lightning","#,
                r#""velocity":[24,-6,312],"throttle":0.7,"hull":0.82,"energy":0.64,"shields":[0.9,0.3],"#,
                r#""power":[2,1,5,3,4],"balance":"equal","link":"linked","afterburner":false,"braking":false,"#,
                r#""missile":{"name":"Hunter","count":4},"systems":[{"kind":"ecm","health":1,"destroyed":false,"on":true}],"#,
                r#""target":{"name":"Wraith","relation":"hostile","protect":false,"distance":2140,"closing":85,"#,
                r#""hull":0.55,"shields":[0.2,0],"rotation":[0,0.38,0,0.92]}}"#
            ));
            server.broadcast(concat!(
                r#"{"type":"contacts","range":6000,"list":["#,
                r#"{"id":"3:0","name":"Wraith","relation":"hostile","station":false,"target":true,"pos":[-400,300,2100],"vel":[40,0,-90]},"#,
                r#"{"id":"4:0","name":"Spectre","relation":"hostile","station":false,"target":false,"pos":[1500,-600,3400],"vel":[-60,10,0]},"#,
                r#"{"id":"5:0","name":"Lightning","relation":"friendly","station":false,"target":false,"pos":[-120,20,-150],"vel":[0,0,0]},"#,
                r#"{"id":"6:0","name":"X-COM base","relation":"objective","station":true,"target":false,"pos":[2500,-900,-3000],"vel":[0,0,0]}]}"#
            ));
            server.broadcast(r#"{"type":"event","text":"TARGET: WRAITH"}"#);
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    #[test]
    fn only_local_network_peers_are_served() {
        for ip in ["192.168.1.20", "10.0.0.5", "172.20.1.1", "127.0.0.1", "169.254.3.4"] {
            assert!(is_local(ip.parse().unwrap()), "{ip}");
        }
        for ip in ["8.8.8.8", "172.32.0.1", "100.64.0.1"] {
            assert!(!is_local(ip.parse().unwrap()), "{ip}");
        }
        assert!(is_local(IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert!(is_local("fe80::1".parse().unwrap()) && is_local("fd12::1".parse().unwrap()));
        assert!(!is_local("2001:db8::1".parse().unwrap()));
        assert!(is_local(IpAddr::V6(Ipv4Addr::new(192, 168, 0, 2).to_ipv6_mapped())));
    }

    #[test]
    fn pairing_codes_are_six_digits_and_vary_with_the_seed() {
        let a = pairing_code(1);
        assert_eq!(a.len(), 6);
        assert!(a.chars().all(|c| c.is_ascii_digit()));
        assert_ne!(a, pairing_code(2));
        assert_eq!(a, pairing_code(1), "deterministic for a seed");
    }

    /// A masked text frame, as a browser sends it.
    fn text_frame(s: &str) -> Vec<u8> {
        let mask = [3, 1, 4, 1];
        let mut f = vec![0x81, 0x80 | s.len() as u8];
        f.extend_from_slice(&mask);
        f.extend(s.bytes().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        f
    }

    /// Reads one server frame's payload from the socket.
    fn read_text(s: &mut TcpStream) -> String {
        let mut head = [0u8; 2];
        s.read_exact(&mut head).unwrap();
        let mut len = (head[1] & 0x7f) as usize;
        if len == 126 {
            let mut l = [0u8; 2];
            s.read_exact(&mut l).unwrap();
            len = u16::from_be_bytes(l) as usize;
        }
        let mut body = vec![0u8; len];
        s.read_exact(&mut body).unwrap();
        String::from_utf8(body).unwrap()
    }

    /// End to end over a real socket on localhost: the page is served, a wrong code is
    /// denied, the right code pairs, actions reach the game, broadcasts reach the console.
    #[test]
    fn serves_the_page_pairs_and_relays_actions_over_a_real_socket() {
        // Port 0 would be ideal, but the server reports its configured port; pick a high
        // port from the process id to avoid clashes between parallel test runs.
        let port = 20_000 + (std::process::id() % 20_000) as u16;
        let server = ConsoleServer::start(port, 7).unwrap();
        let get = |path: &str| {
            let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
            s.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes()).unwrap();
            let mut r = String::new();
            s.read_to_string(&mut r).unwrap();
            r
        };
        assert!(get("/").starts_with("HTTP/1.1 200 OK") && get("/").contains("<html"));
        assert!(get("/console.js").contains("text/javascript"));
        assert!(get("/secret").starts_with("HTTP/1.1 404"));

        let mut ws = TcpStream::connect(("127.0.0.1", port)).unwrap();
        ws.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        ws.write_all(b"GET /ws HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n").unwrap();
        let mut head = Vec::new();
        let mut b = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            ws.read_exact(&mut b).unwrap();
            head.push(b[0]);
        }
        assert!(String::from_utf8(head).unwrap().contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
        // Before pairing, actions are ignored; a wrong code is denied.
        ws.write_all(&text_frame(r#"{"type":"action","name":"fire_beams","value":1}"#)).unwrap();
        ws.write_all(&text_frame(r#"{"type":"hello","code":"000000x"}"#)).unwrap();
        assert!(read_text(&mut ws).contains("denied"));
        ws.write_all(&text_frame(&format!(r#"{{"type":"hello","code":"{}"}}"#, server.code))).unwrap();
        assert!(read_text(&mut ws).contains(r#""type":"welcome""#), "answered at once by the server");
        ws.write_all(&text_frame(r#"{"type":"action","name":"next_target","value":1}"#)).unwrap();
        let mut got = Vec::new();
        for _ in 0..100 {
            got.extend(server.poll());
            if got.len() >= 2 {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(matches!(got[0], Event::Paired(_)), "{got:?}");
        assert!(
            matches!(
                got[1],
                Event::Input(_, protocol::Inbound::Action { action: crate::input::Action::NextTarget, down: true })
            ),
            "{got:?}"
        );
        assert_eq!(got.len(), 2, "the unpaired fire_beams was dropped");
        assert_eq!(server.paired(), 1);
        server.broadcast(r#"{"type":"telemetry"}"#);
        assert_eq!(read_text(&mut ws), r#"{"type":"telemetry"}"#);
    }
}
