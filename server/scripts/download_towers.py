#!/usr/bin/env python3
"""
Download OpenCelliD India cell tower data and build SQLite DB.
Usage: python3 download_towers.py [api_key]
If no API key, uses a pre-built India subset.
"""
import sqlite3
import csv
import io
import sys
import os

DB_PATH = os.path.join(os.path.dirname(__file__), "..", "data", "towers.db")

# India MCC codes
INDIA_MCCS = {"404", "405"}

def build_from_opencellid_api(api_key: str):
    """Download from OpenCelliD API (requires free registration at opencellid.org)"""
    import urllib.request
    
    url = f"https://opencellid.org/api/get?BBOX=60.0,6.0,100.0,40.0&cc=IN&format=csv&key={api_key}"
    print(f"Downloading from OpenCelliD...")
    req = urllib.request.Request(url)
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = resp.read().decode("utf-8")
    return data

def build_sample_data():
    """Build a sample India tower dataset for demo"""
    print("Building sample India tower dataset...")
    
    # Major cities with realistic tower coordinates
    towers = []
    tower_id = 1
    
    cities = [
        # (city, lat, lng, operators, tower_count)
        ("Chandigarh", 30.7333, 76.7794, ["JIO", "Airtel", "BSNL", "VI"], 40),
        ("Ludhiana", 30.9010, 75.8573, ["JIO", "Airtel", "BSNL"], 35),
        ("Amritsar", 31.6340, 74.8723, ["JIO", "Airtel", "BSNL", "VI"], 30),
        ("Jalandhar", 31.3260, 75.5762, ["JIO", "Airtel", "BSNL"], 25),
        ("Patiala", 30.3398, 76.3869, ["JIO", "Airtel", "BSNL"], 20),
        ("Ambala", 30.3752, 76.7821, ["JIO", "Airtel", "BSNL"], 15),
        ("Hisar", 29.1492, 75.7217, ["JIO", "Airtel", "BSNL"], 15),
        ("Karnal", 29.6857, 76.9905, ["JIO", "Airtel", "BSNL"], 15),
        ("Panipat", 29.3909, 76.9635, ["JIO", "Airtel"], 12),
        ("Kurukshetra", 29.9695, 76.8783, ["JIO", "BSNL"], 10),
        ("Sonepat", 28.9958, 77.0145, ["JIO", "Airtel"], 10),
        ("Rohtak", 28.8955, 76.6066, ["JIO", "Airtel", "BSNL"], 12),
        ("Bhiwani", 28.7930, 76.1380, ["JIO", "BSNL"], 8),
        ("Karnal", 29.6857, 76.9905, ["JIO", "Airtel"], 10),
        ("Moga", 30.8160, 75.1738, ["JIO", "Airtel"], 8),
        ("Bathinda", 30.2070, 74.9521, ["JIO", "Airtel", "BSNL"], 15),
        ("Malerkotla", 30.5277, 75.8809, ["JIO", "Airtel"], 6),
        ("Hoshiarpur", 31.5143, 75.9115, ["JIO", "BSNL"], 10),
        ("Pathankot", 32.2643, 75.6472, ["JIO", "BSNL"], 8),
        ("Gurdaspur", 32.0431, 75.4059, ["JIO", "BSNL"], 6),
        ("Ferozepur", 30.9257, 74.6042, ["JIO", "Airtel"], 8),
        ("Kapurthala", 31.3804, 75.3970, ["JIO", "Airtel"], 6),
        ("Sangrur", 30.2500, 75.8500, ["JIO", "Airtel"], 8),
        ("Barnala", 30.3750, 75.5500, ["JIO"], 5),
        ("Mandi", 31.7100, 76.9300, ["JIO", "BSNL"], 6),
        ("Hamirpur", 31.6800, 76.5200, ["JIO", "BSNL"], 5),
        ("Una", 31.4800, 76.2700, ["JIO"], 4),
        ("Shimla", 31.1048, 77.1734, ["JIO", "Airtel", "BSNL"], 10),
        ("Solan", 30.9040, 77.0960, ["JIO", "BSNL"], 4),
        ("Manali", 32.2432, 77.1892, ["JIO", "Airtel"], 4),
    ]
    
    for city, base_lat, base_lng, operators, count in cities:
        for i in range(count):
            # Spread towers around city center (within ~5km)
            import random
            random.seed(hash(f"{city}_{i}") % 2**32)
            
            lat = base_lat + random.uniform(-0.045, 0.045)  # ~5km spread
            lng = base_lng + random.uniform(-0.045, 0.045)
            operator = random.choice(operators)
            lac = random.randint(1000, 9999)
            cid = random.randint(10000, 99999)
            range_m = random.randint(100, 5000)
            samples = random.randint(10, 1000)
            
            # Determine MCC/MNC
            if operator == "JIO":
                mcc, mnc = "405", "862"
            elif operator == "Airtel":
                mcc, mnc = "404", "10"
            elif operator == "BSNL":
                mcc, mnc = "404", "72"
            elif operator == "VI":
                mcc, mnc = "404", "20"
            else:
                mcc, mnc = "404", "00"
            
            towers.append((tower_id, lat, lng, range_m, operator, mcc, mnc, lac, cid, samples))
            tower_id += 1
    
    return towers

def save_to_sqlite(towers, db_path):
    """Save tower data to SQLite"""
    os.makedirs(os.path.dirname(db_path), exist_ok=True)
    
    conn = sqlite3.connect(db_path)
    c = conn.cursor()
    
    c.execute("DROP TABLE IF EXISTS cell_towers")
    c.execute("""
        CREATE TABLE cell_towers (
            id INTEGER PRIMARY KEY,
            lat REAL NOT NULL,
            lng REAL NOT NULL,
            range_m INTEGER,
            operator TEXT,
            mcc TEXT,
            mnc TEXT,
            lac INTEGER,
            cid INTEGER,
            samples INTEGER
        )
    """)
    c.execute("CREATE INDEX IF NOT EXISTS idx_towers_lac_cid ON cell_towers(lac, cid)")
    c.execute("CREATE INDEX IF NOT EXISTS idx_towers_operator ON cell_towers(operator)")
    
    c.executemany(
        "INSERT INTO cell_towers (id, lat, lng, range_m, operator, mcc, mnc, lac, cid, samples) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        towers
    )
    
    conn.commit()
    conn.close()
    
    print(f"Saved {len(towers)} towers to {db_path}")

def main():
    api_key = sys.argv[1] if len(sys.argv) > 1 else None
    
    if api_key:
        try:
            csv_data = build_from_opencellid_api(api_key)
            # Parse CSV and filter to India
            reader = csv.DictReader(io.StringIO(csv_data))
            towers = []
            tid = 1
            for row in reader:
                mcc = row.get("mcc", "")
                if mcc in INDIA_MCCS:
                    towers.append((
                        tid,
                        float(row.get("lat", 0)),
                        float(row.get("lon", 0)),
                        int(row.get("range", 0)) if row.get("range") else None,
                        row.get("mob", ""),
                        mcc,
                        row.get("mnc", ""),
                        int(row.get("lac", 0)) if row.get("lac") else None,
                        int(row.get("cid", 0)) if row.get("cid") else None,
                        int(row.get("samples", 0)) if row.get("samples") else None,
                    ))
                    tid += 1
            save_to_sqlite(towers, DB_PATH)
        except Exception as e:
            print(f"API download failed: {e}")
            print("Falling back to sample data...")
            towers = build_sample_data()
            save_to_sqlite(towers, DB_PATH)
    else:
        towers = build_sample_data()
        save_to_sqlite(towers, DB_PATH)

if __name__ == "__main__":
    main()
