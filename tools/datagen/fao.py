#!/usr/bin/env python3
"""Build crates/engine/data/fao.z from FAOSTAT (FAO, CC BY 4.0): crop and
livestock production (2014 on) and food balance sheets (2019 on).

    python3 tools/datagen/fao.py WORKDIR

WORKDIR receives the two bulk downloads (about 1.2 GB unpacked).

Sections: "#A" area code<TAB>ISO3 (countries only)<TAB>name; "#I" item code<TAB>name; "#P" production
rows: area, item, element (P production, H area harvested, S stocks), unit,
then comma-separated values for 2014, 2015, ...; "#F" food balance rows:
area, item, element (K kcal/capita/day, Q kg/capita/year, T food in
1000 t), then values for 2021, 2020, ...
"""
import csv
import os
import subprocess
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "fao.z")
BASE = "https://bulks-faostat.fao.org/production/"
FILES = ["Production_Crops_Livestock_E_All_Data_(Normalized)", "FoodBalanceSheets_E_All_Data_(Normalized)"]
P0, F0 = 2014, 2021


def fetch(work, name):
    csvp = os.path.join(work, name + ".csv")
    if not os.path.exists(csvp):
        z = os.path.join(work, name + ".zip")
        subprocess.run(["curl", "-sS", "-o", z, BASE + name + ".zip"], check=True)
        subprocess.run(["unzip", "-o", "-q", z, "-d", work], check=True)
        os.remove(z)
    return csvp


def num(v):
    x = float(v)
    return f"{x:.4g}" if x != int(x) or abs(x) >= 1e5 else str(int(x))


def main():
    work = sys.argv[1]
    info = subprocess.run(["curl", "-sS", "https://download.geonames.org/export/dump/countryInfo.txt"], capture_output=True, check=True).stdout.decode()
    iso = {f[2]: f[1] for f in (l.split("\t") for l in info.splitlines() if l and not l.startswith("#")) if len(f) > 2}
    iso["159"] = "CHN"  # FAO "China" (mainland, Hong Kong, Macao, Taiwan)
    iso.pop("156", None)  # "China, mainland"
    areas, items = {}, {}
    prod, food = {}, {}
    pel = {"Production": "P", "Area harvested": "H", "Stocks": "S"}
    for r in csv.DictReader(open(fetch(work, FILES[0]), encoding="utf-8")):
        e = pel.get(r["Element"])
        y = int(r["Year"])
        if not e or y < P0 or r["Value"] == "":
            continue
        areas[r["Area Code"]] = (iso.get(r["Area Code (M49)"].strip("'"), ""), r["Area"])
        items["p" + r["Item Code"]] = r["Item"]
        key = (r["Area Code"], "p" + r["Item Code"], e, r["Unit"])
        prod.setdefault(key, {})[y] = num(r["Value"])
    fel = {"Food supply (kcal/capita/day)": "K", "Food supply quantity (kg/capita/yr)": "Q", "Food": "T"}
    for r in csv.DictReader(open(fetch(work, FILES[1]), encoding="utf-8")):
        e = fel.get(r["Element"])
        y = int(r["Year"])
        if not e or y < F0 or r["Value"] == "":
            continue
        areas[r["Area Code"]] = (iso.get(r["Area Code (M49)"].strip("'"), ""), r["Area"])
        items["f" + r["Item Code"]] = r["Item"]
        food.setdefault((r["Area Code"], "f" + r["Item Code"], e), {})[y] = num(r["Value"])
    out = ["#A"] + [f"{k}\t{v[0]}\t{v[1]}" for k, v in sorted(areas.items(), key=lambda x: int(x[0]))]
    out += ["#I"] + [f"{k}\t{v}" for k, v in sorted(items.items())]
    out.append("#P")
    for (a, i, e, u), vals in sorted(prod.items()):
        last = max(vals)
        out.append(f"{a}\t{i}\t{e}\t{u}\t" + ",".join(vals.get(y, "") for y in range(P0, last + 1)))
    out.append("#F")
    for (a, i, e), vals in sorted(food.items()):
        last = max(vals)
        out.append(f"{a}\t{i}\t{e}\t" + ",".join(vals.get(y, "") for y in range(F0, last + 1)))
    data = "\n".join(out).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(len(areas), "areas", len(items), "items", len(prod), "production rows", len(food), "food rows", os.path.getsize(OUT) // 1024, "KB")


if __name__ == "__main__":
    main()
