"""
Sniffer für BCUK-LMU-Kommunikation.
Zeichnet alle HTTP-Requests auf, die BCUK an LMU sendet.
Starte: python scripts/sniff_bcuk.py
Dann starte BCUK und drücke Kamera-Buttons.
"""

import socket
import threading
import time
import sys
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse

# LMU REST API läuft auf 6397
LMU_HOST = "localhost"
LMU_PORT = 6397

# Unser Proxy läuft auf 6398
PROXY_PORT = 6398

class ProxyHandler(BaseHTTPRequestHandler):
    """Fängt Requests ab, leitet sie weiter und loggt sie"""
    
    def do_GET(self):
        self.log_and_forward("GET")
    
    def do_PUT(self):
        self.log_and_forward("PUT")
    
    def do_POST(self):
        self.log_and_forward("POST")
    
    def log_and_forward(self, method):
        # Body lesen
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length) if content_length > 0 else b''
        
        # Request loggen
        print(f"\n{'='*60}")
        print(f"  >>> BCUK -> LMU: {method} {self.path}")
        if body:
            try:
                parsed = json.loads(body)
                print(f"  Body: {json.dumps(parsed, indent=2)}")
            except:
                print(f"  Body: {body.decode('utf-8', errors='ignore')[:200]}")
        
        # Headers loggen
        for key, val in self.headers.items():
            if key.lower() in ('content-type', 'content-length'):
                print(f"  Header: {key}: {val}")
        
        print(f"{'='*60}")
        
        # An LMU weiterleiten
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.connect((LMU_HOST, LMU_PORT))
            
            # Original-Request zusammenbauen und senden
            request = f"{method} {self.path} HTTP/1.1\r\n"
            for key, val in self.headers.items():
                if key.lower() not in ('host', 'content-length'):
                    request += f"{key}: {val}\r\n"
            request += f"Host: localhost:{LMU_PORT}\r\n"
            request += f"Content-Length: {len(body)}\r\n" if body else ""
            request += "\r\n"
            
            sock.sendall(request.encode() + body)
            
            # Antwort empfangen
            response = b''
            while True:
                chunk = sock.recv(4096)
                if not chunk:
                    break
                response += chunk
            
            sock.close()
            
            # Antwort parsen
            header_end = response.find(b'\r\n\r\n')
            status_line = response[:response.find(b'\r\n')].decode()
            response_body = response[header_end+4:] if header_end > 0 else b''
            
            print(f"  <<< LMU -> BCUK: {status_line}")
            if response_body:
                print(f"  Response: {response_body.decode('utf-8', errors='ignore')[:200]}")
            
            # Original-Antwort an BCUK zurückgeben
            self.send_response_only(int(status_line.split()[1]))
            for header_line in response[:header_end].decode().split('\r\n')[1:]:
                if header_line:
                    key, _, val = header_line.partition(': ')
                    self.send_header(key, val)
            self.end_headers()
            self.wfile.write(response_body)
            
        except Exception as e:
            print(f"  FEHLER: {e}")
            self.send_response(502)
            self.end_headers()
            self.wfile.write(b'Proxy Error')
    
    def log_message(self, format, *args):
        pass  # Kein Standard-Logging

def main():
    # Prüfe ob LMU läuft
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(2)
        sock.connect((LMU_HOST, LMU_PORT))
        sock.close()
        print(f"✅ LMU REST-API läuft auf Port {LMU_PORT}")
    except:
        print(f"❌ LMU nicht erreichbar auf Port {LMU_PORT}")
        print("  Starte LMU zuerst!")
        return
    
    print(f"\n🚀 Proxy läuft auf Port {PROXY_PORT}")
    print(f"  Starte BCUK und richte es auf http://localhost:{PROXY_PORT} ein")
    print(f"  Dann klicke auf Kamera-Buttons in BCUK")
    print(f"\n  Drücke STRG+C zum Beenden")
    print(f"\n  {'='*60}")
    print(f"  Warte auf Requests...")
    print(f"  {'='*60}")
    
    server = HTTPServer(('', PROXY_PORT), ProxyHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n  Beende...")
        server.shutdown()

if __name__ == '__main__':
    main()