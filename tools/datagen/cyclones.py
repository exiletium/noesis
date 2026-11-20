#!/usr/bin/env python3
"""Build crates/engine/data/cyclones.z: one line per tropical cyclone of
tropical-storm strength or more in IBTrACS v04r01 (NOAA NCEI, public domain).

    python3 tools/datagen/cyclones.py [ibtracs.ALL.list.v04r01.csv]

Without a file the CSV is streamed from NCEI.  Fields (tab-separated):
season, basin, name, first and last time (UTC, ISO), peak 1-minute wind in
knots (US agencies, else the WMO agency's), minimum pressure (hPa), peak
Saffir-Simpson category (-1 tropical storm, 1..5), latitude and longitude of
the peak.
"""
import csv
import io
import os
import sys
import urllib.request
import zlib

URL = "https://www.ncei.noaa.gov/data/international-best-track-archive-for-climate-stewardship-ibtracs/v04r01/access/csv/ibtracs.ALL.list.v04r01.csv"
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "cyclones.z")


def num(s):
    try:
        return float(s)
    except ValueError:
        return None


def main():
    src = open(sys.argv[1], encoding="utf-8") if len(sys.argv) > 1 else io.TextIOWrapper(urllib.request.urlopen(URL), encoding="utf-8")
    rows = csv.reader(src)
    head = next(rows)
    next(rows)  # units
    ix = {h.strip(): i for i, h in enumerate(head)}
    storms = {}
    for r in rows:
        if r[ix["TRACK_TYPE"]] != "main":
            continue
        sid = r[ix["SID"]]
        s = storms.get(sid)
        t = r[ix["ISO_TIME"]]
        if s is None:
            s = storms[sid] = {"season": r[ix["SEASON"]], "basin": r[ix["BASIN"]], "name": r[ix["NAME"]], "first": t, "last": t, "wind": None, "pres": None, "cat": -9, "lat": "", "lon": ""}
        s["last"] = t
        w = num(r[ix["USA_WIND"]])
        if w is None:
            w = num(r[ix["WMO_WIND"]])
        if w is not None and (s["wind"] is None or w > s["wind"]):
            s["wind"], s["lat"], s["lon"] = w, r[ix["LAT"]], r[ix["LON"]]
        p = num(r[ix["USA_PRES"]])
        if p is None:
            p = num(r[ix["WMO_PRES"]])
        if p is not None and p > 800 and (s["pres"] is None or p < s["pres"]):
            s["pres"] = p
        c = num(r[ix["USA_SSHS"]])
        if c is not None and c > s["cat"]:
            s["cat"] = int(c)
    lines = []
    for s in storms.values():
        if s["wind"] is None or s["wind"] < 34:
            continue
        cat = s["cat"]
        if cat < -1:
            w = s["wind"]
            cat = 5 if w >= 137 else 4 if w >= 113 else 3 if w >= 96 else 2 if w >= 83 else 1 if w >= 64 else -1
        lines.append("\t".join([s["season"], s["basin"], s["name"].title(), s["first"][:16], s["last"][:16], str(int(s["wind"])), str(int(s["pres"])) if s["pres"] else "", str(max(cat, -1)), s["lat"], s["lon"]]))
    lines.sort()
    data = "\n".join(lines).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{len(lines)} cyclones; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
