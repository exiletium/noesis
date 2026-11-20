#!/usr/bin/env python3
"""Build crates/engine/data/towns.z: GeoNames places of 15,000 people or more
(CC BY 4.0, via the geonamescache package) that are not already in cities.json.

    pip install geonamescache && python3 tools/datagen/towns.py

One tab-separated line per town: name, country, latitude, longitude,
population, time zone, admin1 code.
"""
import json
import os
import zlib

import geonamescache

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(ROOT, "crates", "engine", "data")


def main():
    cities = json.load(open(os.path.join(DATA, "cities.json"), encoding="utf-8"))["rows"]
    have = {(r[0].lower(), r[1], round(r[2]), round(r[3])) for r in cities}
    lines = []
    for c in sorted(geonamescache.GeonamesCache().get_cities().values(), key=lambda c: -c["population"]):
        key = (c["name"].lower(), c["countrycode"], round(c["latitude"]), round(c["longitude"]))
        if key in have or not c["timezone"]:
            continue
        have.add(key)
        name = c["name"].replace("\t", " ")
        lines.append(f"{name}\t{c['countrycode']}\t{c['latitude']:.4f}\t{c['longitude']:.4f}\t{c['population']}\t{c['timezone']}\t{c.get('admin1code', '')}")
    data = "\n".join(lines).encode()
    out = os.path.join(DATA, "towns.z")
    with open(out, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{len(lines)} towns; {os.path.getsize(out) // 1024} KB compressed")


if __name__ == "__main__":
    main()
