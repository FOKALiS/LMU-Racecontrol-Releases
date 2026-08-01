"""
Sendet REST-Pfade als MASKIERTE WebSocket-Nachrichten an Port 6398.
WebSocket-Client-Frames MÜSSEN maskiert sein (Bit 0x80 + 4-Byte-Maske)!
"""
import socket
import base64
import os
import json
import struct
import time

HOST = "localhost"
PORT = 6398
PATH = "/websocket/replaymetrics"

def ws_connect(path=PATH):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(3)
    sock.connect((HOST, PORT))
    key = base64.b64encode(os.urandom(16)).decode()
    request = (
        f"GET {path} HTTP/1.1\r\n"
        f"Host: {HOST}:{PORT}\r\n"
        "Upgrade: websocket\r\n"
        "Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\n"
        "Sec-WebSocket-Version: 13\r\n"
        "\r\n"
    )
    sock.send(request.encode())
    response = sock.recv(4096).decode(errors="replace")
    if "101" in response:
        return sock
    return None

def ws_send_masked(sock, data, opcode=0x1):
    """Sendet einen MASKIERTEN WebSocket-Frame (Client→Server)."""
    if isinstance(data, str):
        payload = data.encode()
    else:
        payload = data
    
    # Maske generieren (4 Bytes)
    mask = os.urandom(4)
    masked_payload = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
    
    header = bytearray()
    header.append(0x80 | opcode)  # FIN + Opcode
    length = len(payload)
    if length < 126:
        header.append(0x80 | length)  # MASK-Bit (0x80) + Länge
    elif length < 65536:
        header.append(0x80 | 126)
        header.extend(struct.pack(">H", length))
    else:
        header.append(0x80 | 127)
        header.extend(struct.pack(">Q", length))
    
    header.extend(mask)  # 4-Byte-Maske
    sock.send(bytes(header) + masked_payload)

def ws_recv(sock, timeout=2):
    try:
        sock.settimeout(timeout)
        header = sock.recv(2)
        if len(header) < 2:
            return None, None
        opcode = header[0] & 0x0F
        masked = header[1] & 0x80
        length = header[1] & 0x7F
        if length == 126:
            length = struct.unpack(">H", sock.recv(2))[0]
        elif length == 127:
            length = struct.unpack(">Q", sock.recv(8))[0]
        mask = sock.recv(4) if masked else None
        data = b""
        while len(data) < length:
            chunk = sock.recv(length - len(data))
            if not chunk:
                break
            data += chunk
        if mask:
            data = bytes(b ^ mask[i % 4] for i, b in enumerate(data))
        if opcode == 0x1:
            return opcode, data.decode(errors="replace")
        return opcode, data
    except socket.timeout:
        return None, None
    except:
        return None, None

print("=" * 60)
print("WebSocket-Befehle (maskiert) an /websocket/replaymetrics")
print("=" * 60)

sock = ws_connect()
if not sock:
    print("❌ Verbindung fehlgeschlagen")
    exit(1)
print("✅ Verbunden!")

commands = [
    ("REST enter", "/rest/watch/replayCommand/enter"),
    ("REST toggleactive", "/rest/watch/replayCommand/toggleactive"),
    ("REST replay", "/rest/watch/replayCommand/replay"),
    ("REST play", "/rest/watch/replayCommand/play"),
]

for desc, msg in commands:
    print(f"\n--- {desc}: {msg} ---")
    try:
        ws_send_masked(sock, msg)
        opcode, response = ws_recv(sock, timeout=2)
        if opcode is not None:
            if isinstance(response, bytes):
                print(f"  📨 Binär: {response.hex()}")
            else:
                print(f"  📨 Text: {str(response)[:300]}")
        else:
            print("  ⏳ Keine Antwort")
        time.sleep(0.7)
    except Exception as e:
        print(f"  ❌ Fehler: {e}")
        print("  ⚠️ Verbindung neu aufbauen...")
        sock.close()
        sock = ws_connect()
        if not sock:
            print("  ❌ Reconnect fehlgeschlagen")
            break

print("\n" + "=" * 60)
print("Fertig. Hat LMU reagiert?")
print("=" * 60)
sock.close()