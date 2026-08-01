"""
Der ENTSCHEIDENDE TEST: WebSocket-Metrics + replaytime gleichzeitig.
Wenn replaytime funktioniert, springt currentReplayPos von ~1900 auf 100!
"""
import socket
import base64
import os
import json
import struct
import time
import urllib.request
import threading

HOST = "localhost"
PORT_WS = 6398
PORT_REST = 6397
PATH = "/websocket/replaymetrics"

ws_sock = None
ws_connected = threading.Event()
metrics_data = {}
metrics_lock = threading.Lock()

def ws_connect():
    global ws_sock
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(3)
    sock.connect((HOST, PORT_WS))
    key = base64.b64encode(os.urandom(16)).decode()
    request = (
        f"GET {PATH} HTTP/1.1\r\n"
        f"Host: {HOST}:{PORT_WS}\r\n"
        "Upgrade: websocket\r\n"
        "Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\n"
        "Sec-WebSocket-Version: 13\r\n"
        "\r\n"
    )
    sock.send(request.encode())
    response = sock.recv(4096).decode(errors="replace")
    if "101" in response:
        ws_sock = sock
        return True
    return False

def ws_recv_loop():
    """Empfängt kontinuierlich WebSocket-Nachrichten im Hintergrund."""
    global ws_sock
    while ws_sock:
        try:
            ws_sock.settimeout(0.5)
            header = ws_sock.recv(2)
            if len(header) < 2:
                continue
            opcode = header[0] & 0x0F
            masked = header[1] & 0x80
            length = header[1] & 0x7F
            if length == 126:
                length = struct.unpack(">H", ws_sock.recv(2))[0]
            elif length == 127:
                length = struct.unpack(">Q", ws_sock.recv(8))[0]
            mask = ws_sock.recv(4) if masked else None
            data = b""
            while len(data) < length:
                chunk = ws_sock.recv(length - len(data))
                if not chunk:
                    break
                data += chunk
            if mask:
                data = bytes(b ^ mask[i % 4] for i, b in enumerate(data))
            if opcode == 0x1:
                text = data.decode(errors="replace")
                try:
                    msg = json.loads(text)
                    if msg.get("type") == "replayMetrics":
                        with metrics_lock:
                            metrics_data.update(msg.get("body", {}))
                except:
                    pass
        except socket.timeout:
            continue
        except:
            break

def get_replay_pos():
    with metrics_lock:
        return metrics_data.get("currentReplayPos")

def rest_put(path):
    req = urllib.request.Request(f"http://localhost:{PORT_REST}{path}", method="PUT")
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            return resp.status
    except Exception as e:
        return 0

print("=" * 60)
print("replaytime-Test mit WebSocket-Metrics")
print("=" * 60)

# WebSocket verbinden
print("\n1️⃣ WebSocket verbinden...")
if not ws_connect():
    print("❌ WebSocket-Fehler")
    exit(1)
print("✅ WebSocket verbunden!")

# Empfangs-Thread starten
t = threading.Thread(target=ws_recv_loop, daemon=True)
t.start()

# Position anzeigen
time.sleep(1)
pos = get_replay_pos()
print(f"\n📊 Aktuelle Replay-Position: {pos}")
print(f"   (Wenn {pos} läuft = live bzw. Replay läuft)")

# Zeitsprung auf 100
print("\n2️⃣ Sende PUT /rest/watch/replaytime/100 ...")
status = rest_put("/rest/watch/replaytime/100")
print(f"   Status: {status}")

# Position danach prüfen
time.sleep(2)
pos_after = get_replay_pos()
print(f"\n📊 Replay-Position NACH Zeitsprung: {pos_after}")

if pos_after is not None and pos is not None:
    diff = abs(pos_after - pos)
    print(f"\n   Differenz: {abs(pos_after - pos):.2f}s")
    if diff > 500:
        print("   ✅✅✅ ZEITSPRUNG FUNKTIONIERT! Replay-Position hat sich massiv geändert!")
    elif diff > 10:
        print("   ✅ ZEITSPRUNG FUNKTIONIERT! Replay-Position hat sich geändert!")
    else:
        print("   ❌ Zeitsprung hat NICHT funktioniert. Position fast unverändert.")

print("\n" + "=" * 60)
print("Fertig.")
print("=" * 60)

# Aufräumen
if ws_sock:
    ws_sock.close()