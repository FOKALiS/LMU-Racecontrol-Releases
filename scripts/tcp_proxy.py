"""
TCP Proxy für LMU REST API
Fängt alle HTTP-Requests zwischen BCUK und LMU ab und zeigt sie an.

Verwendung:
1. LMU starten (Port 6397)
2. Dieses Skript starten: python scripts/tcp_proxy.py
3. BCUK starten und Port auf 6398 ändern
4. In BCUK Kamera-Buttons klicken
5. Hier erscheinen die Requests
"""

import socket
import threading
import sys
import re

LMU_HOST = "127.0.0.1"
LMU_PORT = 6397
PROXY_PORT = 6398

def handle_client(client_socket):
    """Verarbeitet einen eingehenden Request von BCUK"""
    try:
        # Lese den kompletten HTTP-Request
        data = b""
        while True:
            chunk = client_socket.recv(4096)
            if not chunk:
                break
            data += chunk
            if b"\r\n\r\n" in data:
                # Prüfe ob Body folgt (Content-Length)
                headers = data.split(b"\r\n\r\n")[0]
                content_length = 0
                for line in headers.split(b"\r\n"):
                    if line.lower().startswith(b"content-length:"):
                        content_length = int(line.split(b":")[1].strip())
                        break
                
                # Warte auf vollständigen Body
                body_start = data.find(b"\r\n\r\n") + 4
                if len(data) - body_start >= content_length:
                    break
                if content_length == 0:
                    break

        # Parse und zeige den Request an
        request_text = data.decode("utf-8", errors="replace")
        first_line = request_text.split("\r\n")[0]
        
        # Extrahiere Methode und Pfad
        match = re.match(r"(GET|POST|PUT|DELETE) (.+?) HTTP", first_line)
        if match:
            method = match.group(1)
            path = match.group(2)
            
            # Extrahiere Body
            body = ""
            if "\r\n\r\n" in request_text:
                body = request_text.split("\r\n\r\n", 1)[1]
            
            print(f"\n{'='*60}")
            print(f"📤 {method} {path}")
            if body and body.strip() and body.strip() != "{}":
                print(f"   Body: {body.strip()}")
            print(f"{'='*60}")

        # Leite an LMU weiter
        lmu_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        lmu_socket.connect((LMU_HOST, LMU_PORT))
        lmu_socket.sendall(data)
        
        # Lese Response von LMU
        response = b""
        while True:
            chunk = lmu_socket.recv(4096)
            if not chunk:
                break
            response += chunk
            if b"\r\n\r\n" in response:
                # Prüfe Content-Length
                headers = response.split(b"\r\n\r\n")[0]
                content_length = 0
                for line in headers.split(b"\r\n"):
                    if line.lower().startswith(b"content-length:"):
                        content_length = int(line.split(b":")[1].strip())
                        break
                
                body_start = response.find(b"\r\n\r\n") + 4
                if len(response) - body_start >= content_length:
                    break
                if content_length == 0:
                    break
        
        # Zeige Response-Status
        response_text = response.decode("utf-8", errors="replace")
        status_line = response_text.split("\r\n")[0]
        print(f"📥 {status_line}")
        
        # Extrahiere Response-Body
        if "\r\n\r\n" in response_text:
            resp_body = response_text.split("\r\n\r\n", 1)[1]
            if resp_body and resp_body.strip():
                print(f"   Response: {resp_body.strip()}")
        
        # Sende Response zurück an BCUK
        client_socket.sendall(response)
        
        lmu_socket.close()
        
    except Exception as e:
        print(f"❌ Fehler: {e}")
    finally:
        client_socket.close()

def main():
    print(f"🚀 LMU REST API Proxy gestartet")
    print(f"   Proxy: 127.0.0.1:{PROXY_PORT} -> LMU: {LMU_HOST}:{LMU_PORT}")
    print(f"   ")
    print(f"   📋 ANLEITUNG:")
    print(f"   1. LMU laufen lassen")
    print(f"   2. BCUK starten")
    print(f"   3. In BCUK Einstellungen -> Port auf {PROXY_PORT} ändern")
    print(f"   4. In BCUK Kamera-Buttons klicken")
    print(f"   5. Hier erscheinen die Requests LIVE")
    print(f"   ")
    print(f"   Drücke STRG+C zum Beenden")
    print(f"{'='*60}")
    
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("0.0.0.0", PROXY_PORT))
    server.listen(5)
    
    try:
        while True:
            client, addr = server.accept()
            thread = threading.Thread(target=handle_client, args=(client,))
            thread.daemon = True
            thread.start()
    except KeyboardInterrupt:
        print("\n\n👋 Proxy gestoppt")
    finally:
        server.close()

if __name__ == "__main__":
    main()