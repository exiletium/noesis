#!/usr/bin/env python3
"""Build crates/engine/data/quakes.z: every earthquake of magnitude 5.5 or
more since 1900 in the USGS ComCat catalog (US government work, public
domain).

    python3 tools/datagen/quakes.py

One line per event: UTC time (YYYY-MM-DD HH:MM), latitude, longitude,
depth (km), magnitude, and the USGS place description.
"""
import csv
import io
import os
import urllib.request
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "quakes.z")
API = "https://earthquake.usgs.gov/fdsnws/event/1/query?format=csv&minmagnitude=5.5&orderby=time-asc&starttime={}&endtime={}"


def main():
    lines = []
    edges = [1900, 1960, 1980, 1995, 2005, 2015, 2027]
    for a, b in zip(edges, edges[1:]):
        with urllib.request.urlopen(API.format(f"{a}-01-01", f"{b}-01-01")) as r:
            for row in csv.DictReader(io.TextIOWrapper(r, encoding="utf-8")):
                if row["type"] != "earthquake" or not row["mag"]:
                    continue
                depth = row["depth"] or "0"
                lines.append("\t".join([row["time"][:16].replace("T", " "), f"{float(row['latitude']):.2f}", f"{float(row['longitude']):.2f}", f"{float(depth):.0f}", row["mag"], row["place"].replace("\t", " ")]))
    data = "\n".join(lines).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{len(lines)} earthquakes; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
