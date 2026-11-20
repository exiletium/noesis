#!/usr/bin/env python3
"""Build crates/engine/data/tornadoes.z from the NOAA Storm Prediction
Center's tornado database (US government work, public domain).

    python3 tools/datagen/tornadoes.py [1950-2024_actual_tornadoes.csv]

One line per tornado, or per state segment of a tornado that crossed
states: date (YYYYMMDD), state, (E)F magnitude (-9 unknown), injuries,
fatalities, start latitude/longitude, path length (miles), and 1 for a
whole track or 2 for a state segment.
"""
import csv
import io
import os
import sys
import urllib.request
import zlib

URL = "https://www.spc.noaa.gov/wcm/data/1950-2024_actual_tornadoes.csv"
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "tornadoes.z")


def main():
    src = open(sys.argv[1], encoding="utf-8") if len(sys.argv) > 1 else io.TextIOWrapper(urllib.request.urlopen(URL), encoding="utf-8")
    lines = []
    for r in csv.DictReader(src):
        sg, ns = r["sg"], r["ns"]
        # whole tracks, and the state pieces of tracks that crossed a state line
        if sg == "1" or sg == "-9":
            kind = "1"
        elif sg == "2" and ns != "1":
            kind = "2"
        else:
            continue
        lines.append("\t".join([r["date"].replace("-", ""), r["st"], r["mag"], r["inj"], r["fat"], r["slat"], r["slon"], r["len"], kind]))
    data = "\n".join(lines).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{len(lines)} tornado records; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
