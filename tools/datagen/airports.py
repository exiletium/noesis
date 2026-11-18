#!/usr/bin/env python3
"""Build crates/engine/data/airports.z from OurAirports (public domain).

    python3 tools/datagen/airports.py AIRPORTS_CSV RUNWAYS_CSV

The CSV files are https://davidmegginson.github.io/ourairports-data/airports.csv
and runways.csv.  Keeps airports with an IATA code that are large or medium,
or have scheduled service.

Output (raw deflate, UTF-8, tab-separated lines):
    iata icao size name municipality country region lat lon elevation_ft runways longest_ft surface
size is L (large), M (medium) or S (small).
"""
import csv
import os
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "airports.z")


SURFACES = [("asp", "asphalt"), ("bit", "asphalt"), ("mac", "asphalt"), ("con", "concrete"), ("pem", "paved"), ("per", "paved"),
            ("cop", "paved"), ("gvl", "gravel"), ("grv", "gravel"), ("gra", "gravel"), ("cor", "coral"), ("lat", "laterite"),
            ("grs", "grass"), ("gre", "grass"), ("turf", "grass"), ("dirt", "dirt"), ("water", "water"), ("paved", "paved"),
            ("crushed", "crushed rock"), ("hard", "paved")]


def surface(s):
    s = (s or "").lower()
    if "asphalt/concrete" in s:
        return "asphalt and concrete"
    return next((name for k, name in SURFACES if s.startswith(k)), "")


def main():
    runways = {}
    for r in csv.DictReader(open(sys.argv[2], encoding="utf-8")):
        if r["closed"] == "1":
            continue
        try:
            length = int(r["length_ft"] or 0)
        except ValueError:
            length = 0
        n, best, surf = runways.get(r["airport_ident"], (0, 0, ""))
        runways[r["airport_ident"]] = (n + 1, max(best, length), r["surface"] if length >= best else surf)
    lines = []
    for a in csv.DictReader(open(sys.argv[1], encoding="utf-8")):
        iata = a["iata_code"].strip()
        if len(iata) != 3 or not iata.isalpha():
            continue
        size = {"large_airport": "L", "medium_airport": "M", "small_airport": "S"}.get(a["type"])
        if size is None or (size == "S" and a["scheduled_service"] != "yes"):
            continue
        n, best, surf = runways.get(a["ident"], (0, 0, ""))
        icao = a["icao_code"] or a["gps_code"] or ""
        f = [iata.upper(), icao, size, a["name"], a["municipality"], a["iso_country"], a["iso_region"],
             f"{float(a['latitude_deg']):.4f}", f"{float(a['longitude_deg']):.4f}", a["elevation_ft"] or "",
             str(n), str(best or ""), surface(surf)]
        lines.append("\t".join(x.replace("\t", " ") for x in f))
    lines.sort()
    data = "\n".join(lines).encode("utf-8")
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    with open(OUT, "wb") as fh:
        fh.write(c.compress(data) + c.flush())
    print(f"{len(lines)} airports; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
