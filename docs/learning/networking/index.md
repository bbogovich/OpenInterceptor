# Networking

Networking taught from OpenInterceptor's own code. The first worked example is the
**tablet control console** (`src/remote/`, app 0.4.0): a game that serves a web page and a
WebSocket to a tablet on the local network, with every protocol layer written in the crate.
How the pieces fit is on [Remote console](../../implementation/architecture/remote.md); this
page explains the protocol mechanics.

More sections (the protocol's design, latency and rates, a serial transport) will follow as the
console grows.

## The WebSocket handshake

A browser cannot open a raw TCP socket. What it can do is send an HTTP request and ask for the
connection to be **upgraded** to a WebSocket (RFC 6455). The tablet's page does this with
`new WebSocket("ws://<host>/ws")`, which sends:

```http
GET /ws HTTP/1.1
Host: 192.168.1.20:8765
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
```

`Sec-WebSocket-Key` is 16 random bytes in base64, new for every connection. The server must
prove that it understood the request as a WebSocket upgrade — not, say, a plain web server that
echoes headers back — by transforming the key in a fixed way:

1. append the fixed GUID `258EAFA5-E914-47DA-95CA-C5AB0DC85B11` (the same for every server);
2. take the **SHA-1** of that text;
3. encode the 20-byte digest in **base64**.

```rust
// src/remote/ws.rs
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn accept_key(client_key: &str) -> String {
    base64(&sha1(format!("{}{GUID}", client_key.trim()).as_bytes()))
}
```

and answer with status 101:

```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

For the key above, RFC 6455 §1.3 gives exactly that accept value, so the unit test
`remote::ws::sha1_and_base64_match_the_published_test_vectors` checks `accept_key` against it,
along with SHA-1's FIPS 180 examples and base64's RFC 4648 examples. Test vectors from the
standard are the cheapest way to know a hand-written algorithm is right.

SHA-1 is no longer safe for signatures, but here it is only a checksum that proves the server
read the key; that use is unaffected. From the 101 response on, the same TCP connection carries
WebSocket **frames** instead of HTTP.

**In the code.** `remote::http::parse_request` reads the request head up to the blank line
(`\r\n\r\n`) — returning `Ok(None)` while it has not all arrived, because TCP delivers a stream
of bytes, not messages — and `Request::is_websocket` checks the method, the `Upgrade` header and
the key. `remote::server::serve` then writes `http::upgrade_response(&ws::accept_key(key))`.

## Frames

Each WebSocket message travels as one or more frames:

```text
 byte 0            byte 1                   then                       then           then
+-----+--------+------+-------------+  +-------------------------+  +-----------+  +---------+
| FIN | opcode | MASK | length (7b) |  | 2 or 8 more length bytes|  | 4-byte    |  | payload |
| 1b  | 4b     | 1b   |             |  | if length is 126 or 127 |  | mask key  |  |         |
+-----+--------+------+-------------+  +-------------------------+  +-----------+  +---------+
```

- **FIN** marks the last frame of a message; **opcode** says what it is: 1 text, 2 binary,
  8 close, 9 ping, 10 pong (0 continues a split message).
- The 7-bit **length** holds the payload length directly up to 125. The value **126** means "the
  real length is in the next 2 bytes", **127** "in the next 8 bytes" (big-endian). A 300-byte
  masked console message therefore starts `81 FE 01 2C`: final text frame, mask bit plus 126,
  then the 16-bit length 300.
- The **MASK** bit and the 4-byte mask key are present on frames from the client.

`ws::decode` returns `Ok(None)` when the buffer does not yet hold a whole frame, and the number
of bytes it used when it does, so the connection loop can drain exactly one frame and keep any
bytes of the next:

```rust
// src/remote/server.rs, the connection loop (simplified)
loop {
    let (frame, used) = match ws::decode(&buf) {
        Ok(Some(f)) => f,
        Ok(None) => break,        // wait for more bytes from the socket
        Err(_) => break 'conn,    // protocol violation: drop the connection
    };
    buf.drain(..used);
    // … handle frame.opcode and frame.payload
}
```

## Masking: why every browser frame is XORed

Every frame a browser sends is **masked**: the browser picks 4 random bytes per frame and XORs
payload byte `i` with mask byte `i % 4`. Masking is not encryption — the key is sent right in
front of the payload. Its purpose is to protect **proxies and caches** in the network path. A
malicious page could otherwise choose payload bytes that look like an HTTP request to a
confused intermediary and poison its cache; with a random mask chosen by the browser, the page
cannot control the bytes that appear on the wire.

So the rules are asymmetric:

| Direction | Masked? | What OpenInterceptor does |
|---|---|---|
| Client → server | **Always** (RFC 6455 §5.3) | `ws::decode` refuses an unmasked frame as an error ("client frames must be masked") and unmasks the payload |
| Server → client | **Never** | `ws::encode` writes the plain payload |

Unmasking is one line, because XOR undoes itself (`(b ^ m) ^ m == b`):

```rust
// src/remote/ws.rs, decode
let mask = [buf[at], buf[at + 1], buf[at + 2], buf[at + 3]];
at += 4;
let payload = buf[at..at + len].iter().enumerate().map(|(i, b)| b ^ mask[i % 4]).collect();
```

RFC 6455 §5.7 has a worked example: the masked text frame `"Hello"` is

```text
81 85 37 FA 21 3D 7F 9F 4D 51 58
│  │  └─ mask ──┘ └─ "Hello" XOR mask ┘
│  └ MASK bit set, length 5
└ FIN + opcode 1 (text)
```

and `remote::ws::frames_decode_masked_wait_for_more_bytes_and_encode` decodes exactly those
bytes. The server's unmasked reply of the same message is just `81 05 48 65 6C 6C 6F`.

`tools/console_client.py` (Python standard library) does the client's side by hand — random key,
handshake, masked frames — which makes it a second, independent implementation to test the
server against.

## Defensive limits

A server on a network must assume its peer may be broken or hostile. The console server:

- refuses a request head over 16 KB and a frame over 64 KB (`http::MAX_HEAD`, `ws::MAX_FRAME`)
  before allocating for it, so a bogus 8-byte length cannot make it reserve gigabytes;
- rejects nesting deeper than 32 levels in JSON, so `[[[[…` cannot overflow the stack of the
  recursive-descent parser;
- serves only peers on the local network (`server::is_local`) and acts only after the pairing
  code.

## Threads and channels in Rust

The game loop must never wait for the network, so the server runs on its own threads and the
game talks to them through **channels** (`std::sync::mpsc`). A channel has a `Sender` and a
`Receiver`; each end can be **moved** to another thread, and values sent through it change
owner, so no data is shared and nothing needs a lock:

```rust
let (tx, events) = std::sync::mpsc::channel();      // tx: Sender<Event>, events: Receiver<Event>
std::thread::spawn(move || accept_loop(listener, code, tx, clients, stop));  // tx moves in
// … later, once per frame on the game thread:
let new_events: Vec<Event> = events.try_iter().collect();   // never blocks
```

The one piece of state both sides need — the list of connected clients — is an
`Arc<Mutex<Vec<Client>>>`: `Arc` (atomically reference-counted) lets several threads own it,
and `Mutex` lets one at a time change it. Simple flags such as "this client has paired" and
"stop accepting" are `Arc<AtomicBool>`, which needs no lock at all. The compiler enforces this:
a type that is not safe to send between threads (an `Rc`, a `RefCell`) cannot be moved into
`thread::spawn`, so the mistakes that cause data races elsewhere do not compile.
