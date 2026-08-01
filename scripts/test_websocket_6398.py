"""
Testet WebSocket auf Port 6398 mit dem BCUK-Pfad /websocket/replaymetrics.
Starte mit LMU + BCUK in einer Session.
"""
import socket
import base64
import os
import json
import struct
import time

HOST = "localhost"
PORT = 6398

def websocket_connect(host, port, path="/websocket/replaymetrics"):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(3)
    sock.connect((host, port))
    key = base64.b64encode(os.urandom(16)).decode()
    request = (
        f"GET {path} HTTP/1.1\r\n"
        f"Host: {host}:{port}\r\n"
        "Upgrade: websocket\r\n"
        "Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\n"
        "Sec-WebSocket-Version: 13\r\n"
        "\r\n"
    )
    sock.send(request.encode())
    response = sock.recv(4096).decode(errors="replace")
    print(f"  Antwort: {response.splitlines()[0] if response else 'Keine Antwort'}")
    if "101" in response:
        return sock
    return None

def ws_recv(sock, timeout=3):
    try:
        sock.settimeout(timeout)
        header = sock.recv(2)
        if len(header) < 2:
            return None, None
        opcode = header[0] & 0x0F
        length = header[1] & 0x7F
        if length == 126:
            length = struct.unpack(">H", sock.recv(2))[0]
        elif length == 127:
            length = struct.unpack(">Q", sock.recv(8))[0]
        data = b""
        while len(data) < length:
            chunk = sock.recv(length - len(data))
            if not chunk:
                break
            data += chunk
        if opcode == 0x1:
            return opcode, data.decode(errors="replace")
        return opcode, data
    except socket.timeout:
        return None, None
    except:
        return None, None

print("=" * 60)
print("WebSocket auf /websocket/replaymetrics")
print("=" * 60)

sock = websocket_connect(HOST, PORT, "/websocket/replaymetrics")
if not sock:
    print("❌ Verbindung fehlgeschlagen auf /websocket/replaymetrics")
    # Fallback: andere Pfade
    for path in ["/websocket", "/replaymetrics", "/ws"]:
        print(f"\n--- Versuche {path} ---")
        sock = websocket_connect(HOST, PORT, path)
        if sock:
            print("  ✅ Verbunden!")
            break
    if not sock:
        print("❌ Kein Pfad funktioniert")
        exit(1)

print("\n📨 Empfange Nachrichten (10 Sekunden)...")
print("   Wenn Du in BCUK auf Replay klickst, sehen wir die Nachrichten!")
print()

start_time = time.time()
count = 0
while time.time() - start_time < 10:
    opcode, msg = ws_recv(sock, timeout=2)
    if opcode is not None:
        count += 1
        if isinstance(msg, bytes):
            print(f"  📨 Binär-Nachricht: {msg.hex()[:100]}")
        else:
            print(f"  📨 {msg[:300]}")
    else:
        print(f"  ⏳ {int(10 - (time.time() - start_time))}s... (warte auf Nachrichten)")

print(f"\n✅ {count} Nachrichten empfangen")
sock.close()
print("Fertig.")