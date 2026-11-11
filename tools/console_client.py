#!/usr/bin/env python3
"""A command-line client for the OpenInterceptor control console (protocol v1).

Standard library only. Connects to a running game started with --console, pairs with
the code the game shows, prints the catalogue summary and a few telemetry messages, and
can send actions. Useful for testing the console without a tablet:

    tools/console_client.py 127.0.0.1:8765 123456 --tap next_target --hold fire_beams:1.5
"""

import argparse
import base64
import json
import os
import socket
import struct
import sys
import time


def handshake(sock, host):
    key = base64.b64encode(os.urandom(16)).decode()
    sock.sendall(
        f"GET /ws HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n".encode()
    )
    head = b""
    while b"\r\n\r\n" not in head:
        chunk = sock.recv(1)
        if not chunk:
            raise SystemExit("connection closed during the handshake")
        head += chunk
    if b" 101 " not in head.split(b"\r\n")[0]:
        raise SystemExit(f"no upgrade: {head.decode(errors='replace')}")


def send(sock, obj):
    """Sends one masked text frame (clients must mask, RFC 6455 §5.3)."""
    data = json.dumps(obj).encode()
    mask = os.urandom(4)
    head = bytes([0x81])
    n = len(data)
    if n < 126:
        head += bytes([0x80 | n])
    else:
        head += bytes([0x80 | 126]) + struct.pack(">H", n)
    sock.sendall(head + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(data)))


def recv_exact(sock, n):
    out = b""
    while len(out) < n:
        chunk = sock.recv(n - len(out))
        if not chunk:
            raise EOFError
        out += chunk
    return out


def receive(sock):
    """Reads one (unmasked) server frame and returns its text, or None for non-text."""
    b0, b1 = recv_exact(sock, 2)
    n = b1 & 0x7F
    if n == 126:
        n = struct.unpack(">H", recv_exact(sock, 2))[0]
    elif n == 127:
        n = struct.unpack(">Q", recv_exact(sock, 8))[0]
    payload = recv_exact(sock, n)
    return payload.decode() if b0 & 0x0F == 1 else None


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("address", help="host:port, e.g. 127.0.0.1:8765")
    p.add_argument("code", help="the pairing code shown in the game")
    p.add_argument("--tap", action="append", default=[], help="tap an action (repeatable)")
    p.add_argument("--hold", action="append", default=[], help="action:seconds to hold (repeatable)")
    p.add_argument("--throttle", type=float, help="set the throttle, 0..1")
    p.add_argument("--telemetry", type=int, default=3, help="telemetry messages to print")
    a = p.parse_args()
    host, port = a.address.rsplit(":", 1)
    sock = socket.create_connection((host, int(port)), timeout=10)
    handshake(sock, a.address)
    send(sock, {"type": "hello", "code": a.code})
    shown = 0
    acted = False
    while shown < a.telemetry:
        text = receive(sock)
        if text is None:
            continue
        msg = json.loads(text)
        kind = msg.get("type")
        if kind == "denied":
            sys.exit(f"denied: {msg.get('reason')}")
        if kind == "welcome":
            print(f"welcome: protocol {msg['version']}, campaign {msg.get('campaign')}")
        elif kind == "catalogue":
            off = [x["name"] for x in msg["actions"] if not x["available"]]
            print(f"catalogue: {len(msg['groups'])} groups, {len(msg['actions'])} actions; not fitted: {off}")
            if not acted:
                acted = True
                for name in a.tap:
                    send(sock, {"type": "action", "name": name, "value": 1})
                if a.throttle is not None:
                    send(sock, {"type": "throttle", "value": a.throttle})
                for spec in a.hold:
                    name, secs = spec.split(":")
                    send(sock, {"type": "action", "name": name, "value": 1})
                    time.sleep(float(secs))
                    send(sock, {"type": "action", "name": name, "value": 0})
        elif kind == "telemetry":
            shown += 1
            t = msg.get("target") or {}
            print(f"telemetry t={msg.get('time')}: craft {msg.get('craft')} velocity {msg.get('velocity')} "
                  f"throttle {msg.get('throttle')} target {t.get('name')} {t.get('distance')}")
    send(sock, {"type": "close"})
    sock.close()


if __name__ == "__main__":
    main()
