#!/usr/bin/env python3
"""End-to-end smoke test for NETRA."""
import urllib.request, json, time, sys

BASE = "http://127.0.0.1:8420/api/v1"

def api(method, path, data=None, token=None, files=None):
    url = f"{BASE}{path}"
    headers = {}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    
    if files:
        import io
        boundary = "----SmokeTestBoundary"
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

results = []

def check(name, passed, detail=""):
    status = "PASS" if passed else "FAIL"
    results.append((name, passed))
    print(f"  [{status}] {name}" + (f" — {detail}" if detail else ""))

print("=" * 60)
print("NETRA End-to-End Smoke Test")
print("=" * 60)

# 1. Health check
print("\n1. Health Check")
import urllib.request as ur
resp = ur.urlopen("http://127.0.0.1:8420/health", timeout=5)
h = json.loads(resp.read().decode())
check("Server is running", h.get("status") == "ok", f"version={h.get('version')}")

# 2. Login
print("\n2. Authentication")
login = api("POST", "/auth/login", {"username": "admin", "password": "netra-admin"})
token = login.get("token")
check("Login succeeds", token is not None, f"user={login.get('user', {}).get('username')}")

# 3. Create case
print("\n3. Case Management")
case = api("POST", "/cases", {"title": "Smoke Test", "description": "E2E test", "classification": "internal", "tags": ["test"]}, token)
case_id = case.get("id")
check("Create case", case_id is not None, f"id={case_id[:8]}")

# List cases
cases = api("GET", "/cases", token=token)
check("List cases", len(cases) > 0, f"count={len(cases)}")

# Get case detail
detail = api("GET", f"/cases/{case_id}", token=token)
check("Get case detail", detail.get("title") == "Smoke Test")

# 4. Ingest
print("\n4. Data Ingestion")
with open("smoke_test.csv", "rb") as f:
    csv_data = f.read()

upload = api("POST", f"/cases/{case_id}/ingest", token=token, files=[("smoke_test.csv", csv_data)])
job_id = upload.get("job_id")
check("Upload file", job_id is not None, f"job={job_id[:8] if job_id else 'none'}")

# Poll until done
print("  Waiting for ingest to complete...")
for i in range(30):
    time.sleep(2)
    job = api("GET", f"/ingest/jobs/{job_id}", token=token)
    status = job.get("status")
    parsed = job.get("records_parsed", 0)
    if status in ("done", "failed"):
        break

check("Ingest completes", status == "done", f"status={status}, parsed={parsed}")

# Wait for async pipeline (resolve + analyze)
print("  Waiting for resolve + analyze pipeline...")
for i in range(20):
    time.sleep(2)
    ents = api("GET", f"/cases/{case_id}/entities", token=token)
    if len(ents) > 0:
        break
print(f"  Pipeline done after {(i+1)*2}s")

# 5. Events
print("\n5. Event Timeline")
events = api("GET", f"/cases/{case_id}/events?limit=10", token=token)
check("Events exist", len(events) > 0, f"count={len(events)}")

# 6. Entity Resolution
print("\n6. Entity Resolution")
entities = api("GET", f"/cases/{case_id}/entities", token=token)
check("Entities resolved", len(entities) > 0, f"count={len(entities)}")

# 7. Graph
print("\n7. Relationship Graph")
graph = api("GET", f"/cases/{case_id}/graph?hops=2", token=token)
nodes = graph.get("nodes", [])
edges = graph.get("edges", [])
check("Graph has nodes", len(nodes) > 0, f"nodes={len(nodes)}")
check("Graph has edges", len(edges) > 0, f"edges={len(edges)}")

# 8. Alerts
print("\n8. Anomaly Detection")
alerts = api("GET", f"/alerts?case_id={case_id}", token=token)
check("Alerts generated", len(alerts) > 0, f"count={len(alerts)}")
if alerts:
    severities = {}
    for a in alerts:
        s = a.get("severity", "unknown")
        severities[s] = severities.get(s, 0) + 1
    check("Alerts have severity", len(severities) > 0, f"by_severity={severities}")

# 9. Dashboard
print("\n9. Dashboard")
dash = api("GET", "/dashboard", token=token)
check("Dashboard returns stats", dash.get("total_cases", 0) > 0, f"cases={dash.get('total_cases')}, alerts={sum(dash.get('alerts_by_severity', {}).values())}")

# 10. Search
print("\n10. Cross-Case Search")
search = api("GET", "/search?q=test&search_type=case", token=token)
check("Search returns results", search.get("total", 0) > 0, f"total={search.get('total')}")

# 11. Settings
print("\n11. Settings & Config")
thresholds = api("GET", "/settings/alerts", token=token)
check("Alert thresholds", thresholds.get("imei_min_subscribers", 0) > 0)

retention = api("GET", "/settings/retention", token=token)
check("Retention config", "archive_after_days" in retention or "enabled" in retention)

webhooks = api("GET", "/settings/webhooks", token=token)
check("Webhooks config", "discord_url" in webhooks)

# 12. Reports
print("\n12. Reporting")
report = api("POST", f"/cases/{case_id}/reports", token=token)
report_id = report.get("report_id")
check("Generate report", report_id is not None, f"report={report_id[:8] if report_id else 'none'}")

reports = api("GET", f"/cases/{case_id}/reports", token=token)
check("List reports", len(reports) > 0, f"count={len(reports)}")

# 13. Export
print("\n13. Evidence Export")
try:
    import urllib.request as ur2
    req = ur2.Request(f"{BASE}/cases/{case_id}/export", headers={"Authorization": f"Bearer {token}"})
    resp = ur2.urlopen(req, timeout=15)
    data = resp.read()
    check("Evidence export", len(data) > 0, f"size={len(data)} bytes")
except Exception as e:
    check("Evidence export", False, str(e))

# 14. Movements
print("\n14. Geospatial")
movements = api("GET", f"/cases/{case_id}/movements", token=token)
check("Movements endpoint", isinstance(movements, (list, dict)), f"count={len(movements) if isinstance(movements, list) else 'dict'}")

# 15. Audit log
print("\n15. Audit Trail")
audit = api("GET", f"/audit?case_id={case_id}&limit=10", token=token)
check("Audit log", len(audit) > 0, f"entries={len(audit)}")

# Summary
print("\n" + "=" * 60)
passed = sum(1 for _, p in results if p)
total = len(results)
print(f"Results: {passed}/{total} passed")
if passed == total:
    print("ALL TESTS PASSED")
else:
    failed = [n for n, p in results if not p]
    print(f"FAILED: {', '.join(failed)}")
print("=" * 60)
