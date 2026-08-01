"""
Testet Port 6398 (BCUK-Port) – findet die echten Replay-Endpunkte.
Starte mit LMU + BCUK in einer Session.
"""
import urllib.request
import urllib.error
import json

def test(method, path, port=6398, body=None):
    url = f"http://localhost:{port}{path}"
    data = None
    if body is not None:
        data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, method=method)
    if body is not None:
        req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            content = resp.read().decode("utf-8", errors="replace")
            print(f"✅ {method} {path} → {resp.status}")
            if content:
                print(f"   Body: {content[:200]}")
            return resp.status
    except urllib.error.HTTPError as e:
        content = e.read().decode("utf-8", errors="replace")
        print(f"❌ {method} {path} → HTTP {e.code}")
        return e.code
    except Exception as e:
        print(f"❌ {method} {path} → FEHLER: {e}")
        return 0

print("=" * 60)
print("Port 6398 (BCUK-Port) – Endpunkt-Finding")
print("=" * 60)

# Basis-Endpunkte
print("\n--- Standard-Pfade ---")
test("GET", "/")
test("GET", "/rest")
test("GET", "/rest/watch")
test("GET", "/api")
test("GET", "/session")

# Bekannte LAN-Pfade (rFactor2 / LMU)
print("\n--- LAN API (rFactor2-Stil) ---")
test("GET", "/rest/sessionInfo")
test("GET", "/rest/standings")
test("GET", "/rest/replay/isActive")
test("PUT", "/rest/replay/time/100")
test("PUT", "/rest/replay/activate")
test("PUT", "/rest/replay/time")

# Einfache Pfade
print("\n--- Einfache Pfade ---")
test("GET", "/replay")
test("GET", "/replay/time")
test("PUT", "/replay/time/100")
test("PUT", "/replay/activate")
test("PUT", "/replay/toggle")
test("PUT", "/replay/play")
test("PUT", "/replay/live")

print("\n" + "=" * 60)
print("Fertig.")
print("=" * 60)