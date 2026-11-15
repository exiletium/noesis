#!/usr/bin/env python3
"""Add land areas from the CIA World Factbook to countries.json.

    python3 tools/datagen/factbook.py FACTBOOK_DIR

FACTBOOK_DIR is a checkout of https://github.com/factbook/factbook.json
(CC0; the Factbook itself is a US government work in the public domain).
Countries are matched by their conventional short or long name.  Adds
"land_km2" to each matched country in crates/engine/data/countries.json.
"""
import glob
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PATH = os.path.join(ROOT, "crates", "engine", "data", "countries.json")


def key(s):
    s = re.sub(r"\(.*?\)", "", s.lower()).replace("&", "and")
    s = re.sub(r"^the\s+|,?\s+the$", "", s.strip())
    return re.sub(r"[^a-z]+", "", s)


def km2(text):
    m = re.match(r"\s*([\d,.]+)\s*(million\s+)?sq km", text or "")
    if not m:
        return None
    v = float(m.group(1).replace(",", ""))
    return v * 1e6 if m.group(2) else v


def main():
    names = {}
    for path in glob.glob(os.path.join(sys.argv[1], "*", "*.json")):
        d = json.load(open(path, encoding="utf-8"))
        area = {k.strip(): v for k, v in d.get("Geography", {}).get("Area", {}).items()}
        land = km2(area.get("land", {}).get("text"))
        if land is None:
            continue
        cn = d.get("Government", {}).get("Country name", {})
        for field in ["conventional short form", "conventional long form"]:
            t = cn.get(field, {}).get("text", "")
            if t and t.lower() != "none":
                names.setdefault(key(t), land)
    # bundled names that differ from the Factbook's
    for ours, theirs in [("Myanmar", "Burma"), ("Ivory Coast", "Cote d'Ivoire")]:
        if key(theirs) in names:
            names[key(ours)] = names[key(theirs)]
    data = json.load(open(PATH, encoding="utf-8"))
    matched = 0
    for c in data.values():
        land = names.get(key(c["name"]))
        if land:
            c["land_km2"] = land
            matched += 1
    json.dump(data, open(PATH, "w", encoding="utf-8"), ensure_ascii=False, separators=(",", ":"))
    print(f"land area for {matched} of {len(data)} countries")
    missing = sorted(c["name"] for c in data.values() if "land_km2" not in c and c.get("population", 0) > 1_000_000)
    print("populous countries without a match:", missing)


if __name__ == "__main__":
    main()
