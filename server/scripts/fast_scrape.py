#!/usr/bin/env python3
"""Fast cell tower scraper for OpenCelliD - works through credits efficiently."""
import urllib.request, json, time, sqlite3, os, sys, math

API_KEY = sys.argv[1] if len(sys.argv) > 1 else os.getenv("OPENCELLID_KEY", "")
BASE = "https://opencellid.org"
DB_PATH = os.path.join(os.path.dirname(__file__), "..", "data", "towers.db")

# Request counter
req_count = 0
MAX_REQ = 950  # Use up to 950 of 1000 daily credits

def api_get(path, params):
    global req_count
    if req_count >= MAX_REQ:
        print(f"Reached {MAX_REQ} requests, stopping.")
        sys.exit(0)
    params["key"] = API_KEY
    qs = "&".join(f"{k}={v}" for k, v in params.items())
    url = f"{BASE}{path}?{qs}"
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "NETRA/1.0"})
        resp = urllib.request.urlopen(req, timeout=15)
        req_count += 1
        data = resp.read().decode()
        return json.loads(data)
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        try:
            err = json.loads(body)
            code = err.get("error", {}).get("code", 0)
        except:
            code = e.code
        if code == 6 or code == 7 or e.code == 429:
            print(f"Rate limited at request {req_count}. Done.")
            sys.exit(0)
        return None
    except Exception as e:
        print(f"Request error: {e}")
        return None

def query_area(lat_min, lon_min, lat_max, lon_max, page=0):
    """Query a bounding box, paginating through results."""
    all_cells = []
    offset = page * 50
    while True:
        result = api_get("/cell/getInArea", {
            "BBOX": f"{lat_min},{lon_min},{lat_max},{lon_max}",
            "limit": "50",
            "offset": str(offset),
            "format": "json",
            "mcc": "404",  # India
        })
        if not result or "cells" not in result:
            break
        cells = result["cells"]
        if not cells:
            break
        all_cells.extend(cells)
        if len(cells) < 50:
            break
        offset += 50
        time.sleep(0.15)
        if req_count >= MAX_REQ:
            break
    return all_cells

def init_db():
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS cell_towers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            lat REAL NOT NULL,
            lng REAL NOT NULL,
            range_m INTEGER DEFAULT 0,
            operator TEXT DEFAULT '',
            mcc INTEGER DEFAULT 404,
            mnc INTEGER DEFAULT 0,
            lac INTEGER DEFAULT 0,
            cid INTEGER DEFAULT 0,
            samples INTEGER DEFAULT 1,
            radio TEXT DEFAULT 'GSM'
        )
    """)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_towers_lac_cid ON cell_towers(lac, cid)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_towers_lat_lng ON cell_towers(lat, lng)")
    conn.commit()
    return conn

def save_cells(conn, cells, seen_keys):
    new = 0
    for c in cells:
        key = (c.get("mcc", 0), c.get("mnc", 0), c.get("lac", 0), c.get("cellid", 0))
        if key in seen_keys:
            continue
        seen_keys.add(key)
        conn.execute(
            "INSERT INTO cell_towers (lat, lng, range_m, operator, mcc, mnc, lac, cid, samples, radio) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                c.get("lat", 0), c.get("lon", 0), c.get("range", 0),
                c.get("operator", ""), c.get("mcc", 404), c.get("mnc", 0),
                c.get("lac", 0), c.get("cellid", 0), c.get("samples", 1),
                c.get("radio", "GSM")
            )
        )
        new += 1
    conn.commit()
    return new

# Indian cities with coordinates (lat, lon) - expanding from Chandigarh outward
CITIES = [
    # Chandigarh metro
    (30.7333, 76.7794, "Chandigarh"),
    (30.7420, 76.7860, "Chandigarh Sec 17"),
    (30.7500, 76.7700, "Chandigarh Sec 35"),
    (30.7200, 76.7900, "Chandigarh South"),
    
    # Mohali / Panchkula
    (30.7040, 76.7170, "Mohali"),
    (30.6940, 76.8450, "Panchkula"),
    
    # Ludhiana (major city)
    (30.9010, 75.8573, "Ludhiana"),
    (30.9100, 75.8700, "Ludhiana Central"),
    (30.8950, 75.8400, "Ludhiana West"),
    
    # Amritsar
    (31.6340, 74.8723, "Amritsar"),
    (31.6200, 74.8900, "Amritsar City"),
    (31.6500, 74.8600, "Amritsar North"),
    
    # Jalandhar
    (31.3260, 75.5762, "Jalandhar"),
    (31.3400, 75.5900, "Jalandhar Central"),
    
    # Patiala
    (30.3398, 76.3869, "Patiala"),
    
    # Ambala
    (30.3752, 76.7821, "Ambala"),
    
    # Karnal
    (29.6857, 76.9905, "Karnal"),
    
    # Kurukshetra
    (29.9695, 76.8783, "Kurukshetra"),
    
    # Hisar
    (29.1458, 75.7229, "Hisar"),
    
    # Rohtak
    (28.8955, 76.5762, "Rohtak"),
    
    # Panipat
    (29.3909, 76.9635, "Panipat"),
    
    # Sonipat
    (28.9922, 77.0187, "Sonipat"),
    
    # Gurugram
    (28.4595, 77.0266, "Gurugram"),
    (28.4700, 77.0400, "Gurugram Cyber Hub"),
    
    # Faridabad
    (28.4089, 77.3178, "Faridabad"),
    
    # Delhi border areas
    (28.6139, 77.2090, "New Delhi"),
    (28.7041, 77.1025, "North Delhi"),
    (28.5245, 77.2066, "South Delhi"),
    (28.6692, 77.2800, "East Delhi"),
    (28.6700, 77.0800, "West Delhi"),
    
    # Haryana rural
    (29.1400, 76.7700, "Jind"),
    (28.9000, 76.5700, "Bhiwani"),
    (28.0500, 76.9900, "Palwal"),
    (28.2500, 76.9500, "Rewari"),
    (28.0800, 76.6200, "Mahendragarh"),
    
    # Punjab rural
    (31.1500, 75.3500, "Hoshiarpur"),
    (31.2800, 75.5800, "Nawanshahr"),
    (30.9500, 75.5000, "Moga"),
    (30.8200, 75.4500, "Barnala"),
    (30.5300, 75.6500, "Sangrur"),
    (30.3500, 75.6300, "Rajpura"),
    (31.3200, 75.5800, "Phillaur"),
    (30.8100, 75.9000, "Khanna"),
    (30.5500, 75.9800, "Samrala"),
    (31.7100, 74.9700, "Gurdaspur"),
    (31.8700, 74.5200, "Pathankot"),
    (31.5600, 74.3500, "Tarn Taran"),
    (31.1000, 75.7000, "Kapurthala"),
    (31.2200, 75.2600, "Dasuya"),
    (31.4000, 75.6000, "Phagwara"),
    (30.8000, 75.2000, "Ludhiana Rural"),
]

def main():
    if not API_KEY:
        print("Usage: python3 fast_scrape.py <api_key>")
        sys.exit(1)
    
    conn = init_db()
    # Load existing towers as seen
    seen = set()
    for row in conn.execute("SELECT mcc, mnc, lac, cid FROM cell_towers"):
        seen.add((row[0], row[1], row[2], row[3]))
    print(f"Loaded {len(seen)} existing towers from DB")
    
    total_new = 0
    for lat, lon, name in CITIES:
        if req_count >= MAX_REQ:
            break
        
        # Query 1km x 1km box around each city center
        step = 0.009  # ~1km
        print(f"[{req_count}/{MAX_REQ}] {name} ({lat:.3f}, {lon:.3f})...", end=" ", flush=True)
        
        cells = query_area(lat - step, lon - step, lat + step, lon + step)
        new = save_cells(conn, cells, seen)
        total_new += new
        print(f"{len(cells)} cells, {new} new")
        time.sleep(0.12)
        
        # Also do a 2km surrounding ring for coverage
        if req_count < MAX_REQ:
            for dlat, dlon in [(0.018, 0), (-0.018, 0), (0, 0.018), (0, -0.018)]:
                if req_count >= MAX_REQ:
                    break
                cells = query_area(lat + dlat - step, lon + dlon - step, lat + dlat + step, lon + dlon + step)
                new = save_cells(conn, cells, seen)
                total_new += new
                time.sleep(0.12)
    
    print(f"\n=== DONE ===")
    print(f"API requests used: {req_count}")
    print(f"New towers added: {total_new}")
    
    final = conn.execute("SELECT COUNT(*) FROM cell_towers").fetchone()[0]
    print(f"Total towers in DB: {final}")
    conn.close()

if __name__ == "__main__":
    main()
