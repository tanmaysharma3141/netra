#!/usr/bin/env python3
"""Seed NETRA with demo data so the app feels alive on first load."""
import urllib.request, json, time, sys, os

BASE = "http://127.0.0.1:8420/api/v1"

def api(method, path, data=None, token=None, files=None):
    url = f"{BASE}{path}"
    headers = {}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    if files:
        boundary = "----SeedBoundary"
        body = b""
        for fname, fdata in files:
            body += f"--{boundary}\r\n".encode()
            body += f'Content-Disposition: form-data; name="files"; filename="{fname}"\r\n'.encode()
            body += b"Content-Type: text/csv\r\n\r\n"
            body += fdata
            body += b"\r\n"
        body += f"--{boundary}--\r\n".encode()
        headers["Content-Type"] = f"multipart/form-data; boundary={boundary}"
        req = urllib.request.Request(url, data=body, headers=headers, method=method)
    elif data:
        body = json.dumps(data).encode()
        headers["Content-Type"] = "application/json"
        req = urllib.request.Request(url, data=body, headers=headers, method=method)
    else:
        req = urllib.request.Request(url, headers=headers, method=method)
    resp = urllib.request.urlopen(req, timeout=30)
    return json.loads(resp.read().decode())

print("=" * 60)
print("NETRA Demo Seed")
print("=" * 60)

# Check if server is running
try:
    urllib.request.urlopen("http://127.0.0.1:8420/health", timeout=3)
    print("[OK] Server is running")
except:
    print("[!] Server not running. Start it first: cd server && cargo run --bin netra-server")
    sys.exit(1)

# Login
login = api("POST", "/auth/login", {"username": "admin", "password": "netra-admin"})
token = login.get("token")
print(f"[OK] Logged in as admin")

# Create demo case
print("\nCreating demo case...")
case = api("POST", "/cases", {
    "title": "OP-2026-041: Cross-border hawala ring",
    "description": "Investigation into suspected hawala network operating across Punjab-Haryana border. Multiple phone numbers sharing IMEIs, rapid fund transfers, coordinated communication blackout.",
    "classification": "confidential",
    "tags": ["hawala", "telecom", "priority-1"]
}, token=token)
case_id = case["id"]
print(f"[OK] Created case: {case_id[:8]}")

# Ingest demo data
csv_path = os.path.join(os.path.dirname(__file__), "..", "demo_cdr.csv")
print(f"\nIngesting {csv_path}...")
with open(csv_path, "rb") as f:
    csv_data = f.read()
print(f"  File size: {len(csv_data) / 1024:.0f} KB")

upload = api("POST", f"/cases/{case_id}/ingest", token=token, files=[("demo_cdr.csv", csv_data)])
job_id = upload.get("job_id")
print(f"  Uploaded, job={job_id[:8]}")

# Poll until done
print("  Waiting for ingest to complete...")
for i in range(60):
    time.sleep(2)
    job = api("GET", f"/ingest/jobs/{job_id}", token=token)
    status = job.get("status")
    parsed = job.get("records_parsed", 0)
    if status == "done":
        print(f"  [OK] Ingest complete: {parsed} records parsed")
        break
    elif status == "failed":
        print(f"  [FAIL] Ingest failed: {job.get('errors', [])}")
        sys.exit(1)
else:
    print(f"  [TIMEOUT] Still running after 120s")

# Wait for auto-pipeline (resolve + analyze)
print("\nWaiting for auto-pipeline (resolve + analyze)...")
for i in range(30):
    time.sleep(2)
    entities = api("GET", f"/cases/{case_id}/entities", token=token)
    if len(entities) > 0:
        print(f"  [OK] Pipeline complete after {(i+1)*2}s")
        break
else:
    print("  [WARN] Pipeline may still be running — entities: 0")

# Final stats
print("\n--- Final Stats ---")
entities = api("GET", f"/cases/{case_id}/entities", token=token)
alerts = api("GET", f"/alerts?case_id={case_id}", token=token)
graph = api("GET", f"/cases/{case_id}/graph?hops=2", token=token)

print(f"  Entities: {len(entities)}")
print(f"  Alerts: {len(alerts)}")
print(f"  Graph: {len(graph.get('nodes', []))} nodes, {len(graph.get('edges', []))} edges")

# Show alert breakdown
severities = {}
for a in alerts:
    s = a.get("severity", "unknown")
    severities[s] = severities.get(s, 0) + 1
print(f"  Alert breakdown: {severities}")

print(f"\n{'=' * 60}")
print(f"Demo seeded successfully!")
print(f"Case ID: {case_id}")
print(f"Open the app and explore — the dashboard should be fully populated now.")
print(f"{'=' * 60}")
