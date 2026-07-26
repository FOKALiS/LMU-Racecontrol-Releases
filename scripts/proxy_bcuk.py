"""
TCP Proxy für LMU REST API (v2)
Fängt alle HTTP-Requests zwischen BCUK und LMU ab.

Problem v1: BCUK hat "Rest failed" gemeldet, weil der Proxy
die HTTP-Responses nicht korrekt weitergeleitet hat.

Jetzt: Korrektes HTTP-Forwarding mit vollständigen Headern.

Verwendung:
1. LMU starten (Port 6397)
2. python scripts/proxy_bcuk.py
3. BCUK starten (Port automatisch auf 6398 umleiten)
4. In BCUK Kamera-Buttons klicken
5. Requests erscheinen hier
"""

import socket
import threading
import sys
import os
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
            chunk = client_socket.recv(8192)
            if not chunk:
                break
            data += chunk
            
            # Prüfe ob Header komplett sind
            if b"\r\n\r\n" in data:
                headers = data.split(b"\r\n\r\n")[0]
                
                # Prüfe Content-Length für Body
                content_length = 0
                for line in headers.split(b"\r\n"):
                    if line.lower().startswith(b"content-length:"):
                        try:
                            content_length = int(line.split(b":")[1].strip())
                        except:
                            pass
                        break
                
                body_start = data.find(b"\r\n\r\n") + 4
                if len(data) - body_start >= content_length:
                    break
                if content_length == 0:
                    break

        if not data:
            client_socket.close()
            return

        # Parse und zeige den Request an
        request_text = data.decode("utf-8", errors="replace")
        lines = request_text.split("\r\n")
        first_line = lines[0] if lines else ""
        
        match = re.match(r"(GET|POST|PUT|DELETE) (.+?) HTTP", first_line)
        if match:
            method = match.group(1)
            path = match.group(2)
            
            # Extrahiere Body
            body = ""
            if "\r\n\r\n" in request_text:
                body = request_text.split("\r\n\r\n", 1)[1]
            
            print(f"\n{'='*60}")
            print(f">>> {method} {path}")
            if body and body.strip() and body.strip() != "{}":
                print(f"    Body: {body.strip()}")
        else:
            print(f"\n>>> {first_line}")

        # Leite an LMU weiter
        lmu_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        lmu_socket.settimeout(5.0)
        lmu_socket.connect((LMU_HOST, LMU_PORT))
        lmu_socket.sendall(data)
        
        # Lese komplette Response von LMU
        response = b""
        while True:
            try:
                chunk = lmu_socket.recv(8192)
                if not chunk:
                    break
                response += chunk
                
                # Prüfe ob Response komplett ist
                if b"\r\n\r\n" in response:
                    resp_headers = response.split(b"\r\n\r\n")[0]
                    
                    # Prüfe Content-Length
                    content_length = 0
                    for line in resp_headers.split(b"\r\n"):
                        if line.lower().startswith(b"content-length:"):
                            try:
                                content_length = int(line.split(b":")[1].strip())
                            except:
                                pass
                            break
                    
                    resp_body_start = response.find(b"\r\n\r\n") + 4
                    if len(response) - resp_body_start >= content_length:
                        break
                    if content_length == 0:
                        break
            except socket.timeout:
                break
        
        # Zeige Response
        resp_text = response.decode("utf-8", errors="replace")
        resp_lines = resp_text.split("\r\n")
        status_line = resp_lines[0] if resp_lines else ""
        
        match = re.match(r"HTTP/[\d.]+ (\d+)", status_line)
        status_code = match.group(1) if match else "???"
        
        # Extrahiere Response-Body
        resp_body = ""
        if "\r\n\r\n" in resp_text:
            resp_body = resp_text.split("\r\n\r\n", 1)[1]
        
        if status_code == "200":
            print(f"<<< {status_code} OK")
        else:
            print(f"<<< {status_code} {status_line}")
        
        if resp_body and resp_body.strip():
            if len(resp_body) < 500:
                print(f"    Response: {resp_body.strip()}")
            else:
                print(f"    Response: {resp_body[:200]}...")
        
        # Sende Response zurück an BCUK
        client_socket.sendall(response)
        
        lmu_socket.close()
        
    except Exception as e:
        print(f"\n!!! Fehler: {e}")
    finally:
        try:
            client_socket.close()
        except:
            pass

def main():
    print(f"{'='*60}")
    print(f"  LMU REST API Proxy v2")
    print(f"  Proxy: 127.0.0.1:{PROXY_PORT} -> LMU: {LMU_HOST}:{LMU_PORT}")
    print(f"{'='*60}")
    print(f"")
    print(f"  ANLEITUNG:")
    print(f"  1. LMU starten (muss laufen!)")
    print(f"  2. BCUK starten")
    print(f"  3. In BCUK Einstellungen -> Port auf {PROXY_PORT} aendern")
    print(f"  4. In BCUK auf Kamera-Buttons klicken")
    print(f"  5. Hier erscheinen die Requests LIVE")
    print(f"")
    print(f"  Druecke STRG+C zum Beenden")
    print(f"{'='*60}")
    print(f"")
    
    # Alten Proxy-Prozess beenden falls vorhanden
    try:
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("0.0.0.0", PROXY_PORT))
        server.listen(5)
    except OSError as e:
        print(f"Port {PROXY_PORT} bereits belegt. Bitte alten Proxy beenden.")
        sys.exit(1)
    
    try:
        while True:
            client, addr = server.accept()
            thread = threading.Thread(target=handle_client, args=(client,))
            thread.daemon = True
            thread.start()
    except KeyboardInterrupt:
        print("\nProxy gestoppt.")
    finally:
        server.close()

if __name__ == "__main__":
    main()