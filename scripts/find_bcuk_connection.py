"""
Findet heraus, wie BCUK mit LMU kommuniziert.
Testet: Named Pipes, zusätzliche Ports, Prozess-Liste.
"""
import subprocess
import sys

print("=" * 60)
print("BCUK-Verbindungsanalyse")
print("=" * 60)

# 1. Alle laufenden Prozesse mit "BCUK" oder "Broadcast" im Namen
print("\n--- Prozesse mit BCUK/Broadcast ---")
try:
    result = subprocess.run(
        'tasklist /FO CSV /NH',
        capture_output=True, text=True, shell=True
    )
    for line in result.stdout.split('\n'):
        if 'BCUK' in line.upper() or 'BROADCAST' in line.upper() or 'CONTROL' in line.upper():
            print(f"  {line.strip()}")
except:
    print("  (Fehler beim Lesen)")

# 2. Offene Ports (netstat)
print("\n--- Offene TCP-Verbindungen (localhost) ---")
try:
    result = subprocess.run(
        'netstat -n -p TCP',
        capture_output=True, text=True, shell=True
    )
    for line in result.stdout.split('\n'):
        if '127.0.0.1' in line or '::1' in line:
            print(f"  {line.strip()}")
except:
    print("  (Fehler beim Lesen)")

# 3. Named Pipes
print("\n--- Named Pipes ---")
try:
    result = subprocess.run(
        'powershell -Command "Get-ChildItem \\\\.\\pipe\\"',
        capture_output=True, text=True, shell=True
    )
    for line in result.stdout.split('\n'):
        if 'LMU' in line.upper() or 'BCUK' in line.upper() or 'RACE' in line.upper() or 'REPLAY' in line.upper():
            print(f"  {line.strip()}")
except:
    print("  (Fehler beim Lesen)")

# 4. LMU-Prozessdetails
print("\n--- LMU-Prozess ---")
try:
    result = subprocess.run(
        'tasklist /FI "IMAGENAME eq LMU.exe" /FO CSV /NH',
        capture_output=True, text=True, shell=True
    )
    for line in result.stdout.split('\n'):
        if line.strip():
            print(f"  {line.strip()}")
except:
    print("  (Fehler)")

# 5. Teste alternative Ports
print("\n--- Alternative Ports testen ---")
import urllib.request
for port in [6395, 6396, 6397, 6398, 6399, 6400, 8080, 5000, 5001]:
    try:
        url = f"http://localhost:{port}/"
        req = urllib.request.Request(url, method="GET")
        with urllib.request.urlopen(req, timeout=1) as resp:
            print(f"  Port {port}: Antwort (Status {resp.status})")
    except Exception as e:
        pass  # Keine Antwort = kein Server

print("\n" + "=" * 60)
print("Fertig.")
print("=" * 60)