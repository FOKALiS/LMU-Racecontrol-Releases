"""
Testet REST-Pfade auf Port 6398 (BCUK nutzt womöglich REST auch auf 6398).
Verwendet exakte BCUK-Pfade aus den DLLs.
"""
import urllib.request
import urllib.error
import json

BASE = "http://localhost:6398"

def test(method, path, body=None):
    url = BASE + path
    data = None
    if body is not None:
        data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, method=method)
    if body is not None:
        req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            content = resp.read().decode("utf-8", errors="replace")
            print(f"✅ {method} {path} → {resp.status} | {content[:100]}")
            return resp.status
    except urllib.error.HTTPError as e:
        content = e.read().decode("utf-8", errors="replace")
        print(f"❌ {method} {path} → HTTP {e.code} | {content[:100]}")
        return e.code
    except Exception as e:
        print(f"❌ {method} {path} → FEHLER: {e}")
        return 0

def get_json(path):
    try:
        with urllib.request.urlopen(BASE + path, timeout=5) as resp:
            return json.loads(resp.read().decode())
    except Exception as e:
        return None

print("=" * 60)
print("REST auf Port 6398 (BCUK-Pfade)")
print("=" * 60)

# sessionInfo auf 6398 – liefert der Server REST?
print("\n--- sessionInfo auf 6398 ---")
info = get_json("/rest/watch/sessionInfo")
if info:
    print(f"  ✅ inRealtime: {info.get('inRealtime')}, session: {info.get('session')}")
else:
    print("  ❌ Kein REST auf 6398 für sessionInfo")

# BCUK-Pfade auf 6398 testen
print("\n--- replayCommand-Befehle auf 6398 ---")
for cmd in ["enter", "replay", "toggleactive", "toggle", "play", "pause", "live"]:
    test("PUT", f"/rest/watch/replayCommand/{cmd}")

print("\n--- replaytime auf 6398 ---")
test("PUT", "/rest/watch/replaytime/100")

print("\n--- replay pfade auf 6398 ---")
test("GET", "/rest/replay/isActive")
test("PUT", "/rest/replay/toggleactive")

print("\n" + "=" * 60)
print("Fertig. Hat LMU reagiert?")
print("=" * 60)