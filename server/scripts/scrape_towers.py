#!/usr/bin/env python3
"""
Persistent tower scraper for OpenCelliD.
Queries 0.01° x 0.01° cells across Punjab/Haryana with pagination.
Rate-limited to stay under 1000 requests/day.

Usage: python3 scrape_towers.py <api_key> [--resume] [--cities-only]
"""

import json
import os
import sqlite3
import sys
import time
import urllib.request
import urllib.error

API_BASE = "https://opencellid.org/cell"
DB_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "towers.db")

# Step size: 0.01 degrees ~ 1km (within 4M sq.m API limit)
STEP = 0.01

# Major cities in Punjab/Haryana/HP/Chandigarh
CITIES = {
    "Chandigarh": (30.73, 76.78),
    "Ludhiana": (30.90, 75.86),
    "Amritsar": (31.63, 74.87),
    "Jalandhar": (31.33, 75.58),
    "Patiala": (30.34, 76.39),
    "Ambala": (30.38, 76.78),
    "Hisar": (29.15, 75.72),
    "Karnal": (29.69, 76.99),
    "Panipat": (29.39, 76.96),
    "Rohtak": (28.90, 76.61),
    "Bathinda": (30.21, 74.95),
    "Shimla": (31.10, 77.17),
    "Mandi": (31.71, 76.93),
}

# Cover 10km radius around each city
CITY_RADIUS_STEP = 10  # number of 0.01° cells in each direction

# Full Punjab grid (for --full mode)
PUNJAB_LAT_RANGE = (28.5, 33.5)
PUNJAB_LON_RANGE = (72.5, 78.0)

MNC_MAP = {
    "862": "Jio", "863": "Jio", "867": "Jio", "840": "Jio",
    "10": "Airtel", "11": "Airtel", "492": "Airtel", "14": "Airtel", "42": "Airtel",
    "72": "BSNL", "73": "BSNL",
    "20": "Vi", "21": "Vi", "88": "Vi",
}

MAX_DAILY_REQUESTS = 900  # stay under 1000 limit


def api_get(url, retries=2):
    req = urllib.request.Request(url, headers={"User-Agent": "NETRA/1.0 forensic-intelligence"})
    for attempt in range(retries + 1):
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            if e.code == 429:
                print("  Rate limited! Stopping.")
                return None
            if attempt < retries:
                time.sleep(2 * (attempt + 1))
            else:
                return None
        except Exception:
            if attempt < retries:
                time.sleep(1)
            else:
                return None
    return None


def query_cell(lat, lon):
    """Query all towers in a 0.01° x 0.01° cell with pagination."""
    bbox = f"{lat:.2f},{lon:.2f},{lat+STEP:.2f},{lon+STEP:.2f}"
    cells = []
    offset = 0
    while True:
        url = f"{API_BASE}/getInArea?key={API_KEY}&BBOX={bbox}&format=json&limit=50&offset={offset}"
        data = api_get(url)
        if data is None or "cells" not in data:
            break
        batch = data["cells"]
        cells.extend(batch)
        if len(batch) < 50:
            break
        offset += 50
        time.sleep(0.2)
    return cells


def main():
    global API_KEY
    if len(sys.argv) < 2:
        print("Usage: python3 scrape_towers.py <api_key> [--resume] [--cities-only]")
        sys.exit(1)

    API_KEY = sys.argv[1]
    resume = "--resume" in sys.argv
    cities_only = "--cities-only" in sys.argv

    # Load progress
    progress_file = os.path.join(os.path.dirname(DB_PATH), "tower_scrape_progress.json")
    scraped_cells = set()
    if resume and os.path.exists(progress_file):
        with open(progress_file) as f:
            scraped_cells = set(json.load(f))
        print(f"Resuming: {len(scraped_cells)} cells already scraped")

    all_towers = []
    requests_made = 0

    if cities_only:
        # Scrape around major cities
        cells_to_query = []
        for city, (clat, clon) in CITIES.items():
            for dlat in range(-CITY_RADIUS_STEP, CITY_RADIUS_STEP + 1):
                for dlon in range(-CITY_RADIUS_STEP, CITY_RADIUS_STEP + 1):
                    lat = round(clat + dlat * STEP, 2)
                    lon = round(clon + dlon * STEP, 2)
                    cell_key = f"{lat:.2f},{lon:.2f}"
                    if cell_key not in scraped_cells:
                        cells_to_query.append((lat, lon, city))
        print(f"Cells to query: {len(cells_to_query)}")
    else:
        # Full Punjab grid
        cells_to_query = []
        lat = PUNJAB_LAT_RANGE[0]
        while lat < PUNJAB_LAT_RANGE[1]:
            lon = PUNJAB_LON_RANGE[0]
            while lon < PUNJAB_LON_RANGE[1]:
                cell_key = f"{lat:.2f},{lon:.2f}"
                if cell_key not in scraped_cells:
                    cells_to_query.append((lat, lon, "Punjab"))
                lon = round(lon + STEP, 2)
            lat = round(lat + STEP, 2)
        print(f"Cells to query: {len(cells_to_query)}")

    for lat, lon, area in cells_to_query:
        if requests_made >= MAX_DAILY_REQUESTS:
            print(f"\nHit daily limit ({MAX_DAILY_REQUESTS}). Saving progress...")
            break

        cells = query_cell(lat, lon)
        requests_made += 1
        all_towers.extend(cells)

        cell_key = f"{lat:.2f},{lon:.2f}"
        scraped_cells.add(cell_key)

        if requests_made % 10 == 0:
            print(f"  [{requests_made}] ({lat:.2f},{lon:.2f}) [{area}]: {len(cells)} towers (total: {len(all_towers)})")
            # Save progress
            with open(progress_file, "w") as f:
                json.dump(list(scraped_cells), f)

        time.sleep(0.3)

    # Save progress
    with open(progress_file, "w") as f:
        json.dump(list(scraped_cells), f)

    # Build SQLite
    save_to_sqlite(all_towers, scraped_cells)
    print(f"\nDone! {len(all_towers)} towers from {requests_made} requests")


def save_to_sqlite(new_towers, scraped_cells):
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    c = conn.cursor()

    # Create table if not exists
    c.execute("CREATE TABLE IF NOT EXISTS cell_towers ("
              "id INTEGER PRIMARY KEY AUTOINCREMENT, lat REAL NOT NULL, lng REAL NOT NULL, "
              "range_m INTEGER, operator TEXT, mcc TEXT, mnc TEXT, lac INTEGER, cid INTEGER, "
              "samples INTEGER, radio TEXT)")
    c.execute("CREATE INDEX IF NOT EXISTS idx_towers_lac_cid ON cell_towers(lac, cid)")
    c.execute("CREATE INDEX IF NOT EXISTS idx_towers_operator ON cell_towers(operator)")
    c.execute("CREATE INDEX IF NOT EXISTS idx_towers_location ON cell_towers(lat, lng)")

    # Get existing towers
    existing = set()
    for row in c.execute("SELECT mcc, mnc, lac, cid FROM cell_towers"):
        existing.add(row)

    inserted = 0
    for t in new_towers:
        key = (str(t.get("mcc", "")), str(t.get("mnc", "")), t.get("lac"), t.get("cellid"))
        if key in existing:
            continue
        existing.add(key)
        mnc_str = str(t.get("mnc", ""))
        operator = MNC_MAP.get(mnc_str, "Unknown")
        c.execute(
            "INSERT INTO cell_towers (lat,lng,range_m,operator,mcc,mnc,lac,cid,samples,radio) VALUES (?,?,?,?,?,?,?,?,?,?)",
            (t.get("lat", 0), t.get("lon", 0), t.get("range"), operator,
             str(t.get("mcc", "")), mnc_str, t.get("lac"), t.get("cellid"), t.get("samples"), t.get("radio", ""))
        )
        inserted += 1

    conn.commit()
    total = c.execute("SELECT COUNT(*) FROM cell_towers").fetchone()[0]
    ops = c.execute("SELECT operator, COUNT(*) FROM cell_towers GROUP BY operator ORDER BY COUNT(*) DESC").fetchall()
    conn.close()

    print(f"Inserted {inserted} new towers. Total in DB: {total}")
    print(f"Operators: {ops}")
    print(f"Cells scraped: {len(scraped_cells)}")


if __name__ == "__main__":
    main()
