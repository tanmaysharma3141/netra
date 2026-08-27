#!/usr/bin/env python3
"""
Download OpenStreetMap tiles for the Punjab/Haryana/Chandigarh region.

Usage:
    python3 download_tiles.py [--output-dir ../client/public/tiles]
                              [--min-zoom 5] [--max-zoom 14]

Bounding box covers all of Punjab + Chandigarh + parts of Haryana/HP/J&K.
Tile coords are computed using the slippy-map formula:
    x = floor((lng + 180) / 360 * 2^zoom)
    y = floor((1 - ln(tan(lat_rad) + sec(lat_rad)) / π) / 2 * 2^zoom)

Downloads ~12k tiles at zoom 5-12 (~80 MB), skips already-downloaded files.
"""

import argparse
import math
import os
import sys
import time
import urllib.request
import urllib.error
from concurrent.futures import ThreadPoolExecutor, as_completed

# Force UTF-8 output on Windows
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")

# ── Bounding box: Punjab + Chandigarh + bordering areas ──────────────
# (south_lat, west_lng, north_lat, east_lng)
BBOX = (28.5, 72.5, 33.5, 78.0)  # covers Punjab + Haryana + HP + J&K south

TILE_SERVER = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
USER_AGENT = "NETRA/1.0 (forensic-intelligence; offline-tile-cache)"
MAX_WORKERS = 8


def lat_lng_to_tile(lat: float, lng: float, zoom: int):
    n = 2 ** zoom
    x = int((lng + 180.0) / 360.0 * n)
    lat_rad = math.radians(lat)
    y = int((1.0 - math.log(math.tan(lat_rad) + 1.0 / math.cos(lat_rad)) / math.pi) / 2.0 * n)
    return x, y


def tile_bbox(zoom: int):
    south, west, north, east = BBOX
    # y increases southward (toward equator), x increases eastward
    x_west, y_north = lat_lng_to_tile(north, west, zoom)
    x_east, y_south = lat_lng_to_tile(south, east, zoom)
    return x_west, y_north, x_east, y_south


def download_tile(z: int, x: int, y: int, out_dir: str):
    tile_dir = os.path.join(out_dir, str(z), str(x))
    tile_path = os.path.join(tile_dir, f"{y}.png")
    if os.path.exists(tile_path) and os.path.getsize(tile_path) > 0:
        return "skip"
    os.makedirs(tile_dir, exist_ok=True)
    url = TILE_SERVER.format(z=z, x=x, y=y)
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                data = resp.read()
                with open(tile_path, "wb") as f:
                    f.write(data)
                return "ok"
        except (urllib.error.URLError, OSError):
            if attempt < 2:
                time.sleep(1 * (attempt + 1))
            else:
                return "fail"
    return "fail"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", default=os.path.join(os.path.dirname(__file__), "..", "client", "public", "tiles"))
    parser.add_argument("--min-zoom", type=int, default=5)
    parser.add_argument("--max-zoom", type=int, default=12)
    args = parser.parse_args()

    out_dir = os.path.abspath(args.output_dir)
    os.makedirs(out_dir, exist_ok=True)

    # Collect all tile coords
    jobs = []
    for z in range(args.min_zoom, args.max_zoom + 1):
        x_min, y_min, x_max, y_max = tile_bbox(z)
        for x in range(x_min, x_max + 1):
            for y in range(y_min, y_max + 1):
                jobs.append((z, x, y))

    total = len(jobs)
    print(f"Downloading {total} tiles for zoom {args.min_zoom}-{args.max_zoom} -> {out_dir}")

    ok = skip = fail = 0
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = {pool.submit(download_tile, z, x, y, out_dir): (z, x, y) for z, x, y in jobs}
        for i, future in enumerate(as_completed(futures), 1):
            result = future.result()
            if result == "ok":
                ok += 1
            elif result == "skip":
                skip += 1
            else:
                fail += 1
            if i % 500 == 0 or i == total:
                print(f"  [{i}/{total}] ok={ok} skip={skip} fail={fail}")

    print(f"\nDone. Downloaded {ok}, skipped {skip} (cached), failed {fail}")
    print("Set VITE_TILE_URL=\"/tiles/{z}/{x}/{y}.png\" for offline use")
    return 0 if fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
