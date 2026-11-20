#!/usr/bin/env python3
"""Build crates/engine/data/acs.z: selected American Community Survey
2019-2023 5-year estimates (US Census Bureau, public domain) for the nation,
states, counties, places of 2,500+ people, metro/micro areas and
congressional districts.

    python3 tools/datagen/acs.py WORKDIR

Streams the table-based summary file one table at a time. Output: "#V" then
the variable names, then one line per geography: summary level, state
abbreviation, name, comma-separated values (in the order of "#V").
"""
import os
import subprocess
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "acs.z")
BASE = "https://www2.census.gov/programs-surveys/acs/summary_file/2023/table-based-SF/"
LEVELS = {"010", "040", "050", "160", "310", "500"}


def r(t, a, b=None):
    return [f"{t}_E{i:03d}" for i in range(a, (b or a) + 1)]


VARS = [
    ("pop", r("B01003", 1)),
    ("male", r("B01001", 2)),
    ("female", r("B01001", 26)),
    ("under5", r("B01001", 3) + r("B01001", 27)),
    ("under18", r("B01001", 3, 6) + r("B01001", 27, 30)),
    ("over65", r("B01001", 20, 25) + r("B01001", 44, 49)),
    ("male65", r("B01001", 20, 25)),
    ("female65", r("B01001", 44, 49)),
    ("median_age", r("B01002", 1)),
    ("white", r("B02001", 2)),
    ("black", r("B02001", 3)),
    ("native", r("B02001", 4)),
    ("asian", r("B02001", 5)),
    ("pacific", r("B02001", 6)),
    ("other_race", r("B02001", 7)),
    ("multiracial", r("B02001", 8)),
    ("hispanic", r("B03001", 3)),
    ("mexican", r("B03001", 4)),
    ("puerto_rican", r("B03001", 5)),
    ("cuban", r("B03001", 6)),
    ("dominican", r("B03001", 7)),
    ("central_american", r("B03001", 8)),
    ("guatemalan", r("B03001", 10)),
    ("honduran", r("B03001", 11)),
    ("salvadoran", r("B03001", 14)),
    ("south_american", r("B03001", 16)),
    ("colombian", r("B03001", 20)),
    ("ecuadorian", r("B03001", 21)),
    ("peruvian", r("B03001", 23)),
    ("venezuelan", r("B03001", 25)),
    ("spaniard", r("B03001", 28)),
    ("white_nonhispanic", r("B03002", 3)),
    ("white_hispanic", r("B03002", 13)),
    ("black_hispanic", r("B03002", 14)),
    ("asian_hispanic", r("B03002", 16)),
    ("born_us", r("B05001", 2)),
    ("born_territories", r("B05001", 3)),
    ("born_abroad_us_parents", r("B05001", 4)),
    ("naturalized", r("B05001", 5)),
    ("noncitizen", r("B05001", 6)),
    ("mobility_pop", r("B07003", 1)),
    ("same_house", r("B07003", 4)),
    ("same_house_male", r("B07003", 5)),
    ("same_house_female", r("B07003", 6)),
    ("moved_same_county", r("B07003", 7)),
    ("moved_other_county", r("B07003", 10)),
    ("moved_other_state", r("B07003", 13)),
    ("moved_abroad", r("B07003", 16)),
    ("marital_pop", r("B12001", 1)),
    ("men15", r("B12001", 2)),
    ("women15", r("B12001", 11)),
    ("never_married_men", r("B12001", 3)),
    ("never_married_women", r("B12001", 12)),
    ("married_men", r("B12001", 4)),
    ("married_women", r("B12001", 13)),
    ("separated_men", r("B12001", 7)),
    ("separated_women", r("B12001", 16)),
    ("widowed_men", r("B12001", 9)),
    ("widowed_women", r("B12001", 18)),
    ("divorced_men", r("B12001", 10)),
    ("divorced_women", r("B12001", 19)),
    ("adults25", r("B15002", 1)),
    ("no_schooling", r("B15002", 3) + r("B15002", 20)),
    ("high_school", r("B15002", 11, 18) + r("B15002", 28, 35)),
    ("some_college", r("B15002", 12, 18) + r("B15002", 29, 35)),
    ("bachelors_plus", r("B15002", 15, 18) + r("B15002", 32, 35)),
    ("bachelors", r("B15002", 15) + r("B15002", 32)),
    ("masters", r("B15002", 16) + r("B15002", 33)),
    ("professional", r("B15002", 17) + r("B15002", 34)),
    ("doctorate", r("B15002", 18) + r("B15002", 35)),
    ("doctorate_men", r("B15002", 18)),
    ("doctorate_women", r("B15002", 35)),
    ("enrolled", r("B14001", 2)),
    ("undergrad", r("B14001", 8)),
    ("grad_school", r("B14001", 9)),
    ("language_pop", r("C16001", 1)),
    ("english_only", r("C16001", 2)),
    ("spanish", r("C16001", 3)),
    ("french", r("C16001", 6)),
    ("german", r("C16001", 9)),
    ("slavic", r("C16001", 12)),
    ("korean", r("C16001", 18)),
    ("chinese", r("C16001", 21)),
    ("vietnamese", r("C16001", 24)),
    ("tagalog", r("C16001", 27)),
    ("arabic", r("C16001", 33)),
    ("poverty_pop", r("B17001", 1)),
    ("poverty", r("B17001", 2)),
    ("poverty_male", r("B17001", 3)),
    ("poverty_female", r("B17001", 17)),
    ("households", r("B11001", 1)),
    ("median_household_income", r("B19013", 1)),
    ("aggregate_household_income", r("B19025", 1)),
    ("per_capita_income", r("B19301", 1)),
    ("gini", r("B19083", 1)),
    ("median_home_value", r("B25077", 1)),
    ("commute_minutes", r("B08013", 1)),
    ("commuters", r("B08012", 1)),
]


def stream(url):
    p = subprocess.Popen(["curl", "-sS", "--retry", "3", url], stdout=subprocess.PIPE)
    for line in p.stdout:
        yield line.decode("utf-8", "replace").rstrip("\r\n")
    p.wait()


def main():
    work = sys.argv[1]
    geos = os.path.join(work, "Geos20235YR.txt")
    if not os.path.exists(geos):
        subprocess.run(["curl", "-sS", "-o", geos, BASE + "documentation/Geos20235YR.txt"], check=True)
    keep = {}
    with open(geos, encoding="utf-8-sig") as fh:
        head = fh.readline().rstrip("\r\n").split("|")
        ix = {k: i for i, k in enumerate(head)}
        for line in fh:
            f = line.rstrip("\r\n").split("|")
            if f[ix["SUMLEVEL"]] in LEVELS and f[ix["COMPONENT"]] == "00":
                keep[f[ix["GEO_ID"]]] = (f[ix["SUMLEVEL"]], f[ix["STUSAB"]], f[ix["NAME"]])
    cells = {}
    for t in sorted({c.split("_")[0] for _, cs in VARS for c in cs}):
        want = {c for _, cs in VARS for c in cs if c.startswith(t + "_")}
        lines = stream(f"{BASE}data/5YRData/acsdt5y2023-{t.lower()}.dat")
        hdr = next(lines).split("|")
        cols = [(i, h) for i, h in enumerate(hdr) if h in want]
        n = 0
        for line in lines:
            f = line.split("|")
            if f[0] not in keep:
                continue
            for i, h in cols:
                try:
                    cells[(f[0], h)] = float(f[i])
                except ValueError:
                    pass
            n += 1
        print(t, len(cols), "cells", n, "geographies", flush=True)
    out = ["#V\t" + ",".join(k for k, _ in VARS)]
    for g, (lvl, st, name) in keep.items():
        pop = cells.get((g, "B01003_E001"), 0)
        if lvl == "160" and pop < 2500:
            continue
        vals = []
        for _, cs in VARS:
            xs = [cells.get((g, c)) for c in cs]
            vals.append("" if any(x is None or x < 0 for x in xs) else f"{sum(xs):.6g}")
        out.append(f"{lvl}\t{st}\t{name}\t" + ",".join(vals))
    data = "\n".join(out).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(len(out) - 1, "geographies", os.path.getsize(OUT) // 1024, "KB")


if __name__ == "__main__":
    main()
