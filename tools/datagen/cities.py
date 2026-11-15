#!/usr/bin/env python3
"""Update the populations of large cities in cities.json from Wikidata (CC0).

    python3 tools/datagen/cities.py [--cache DIR]

GeoNames often counts a city proper (Tokyo's 23 special wards) where
reference works count the city as an administrative whole (the Tokyo
Metropolis).  For every city with a Wikidata population of a million or
more, the most recent dated figure replaces the bundled one when
the name and country match, and its year is recorded in "population_year".
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import wikidata  # noqa: E402

PATH = os.path.join(wikidata.ROOT, "crates", "engine", "data", "cities.json")


def main():
    if "--cache" in sys.argv:
        wikidata.CACHE = sys.argv[sys.argv.index("--cache") + 1]
        os.makedirs(wikidata.CACHE, exist_ok=True)
    # candidate cities: national capitals and the large-city classes (cheap id lists)
    ids = {}
    for r in wikidata.sparql("SELECT DISTINCT ?c WHERE { ?country wdt:P31 wd:Q3624078 ; wdt:P36 ?c }"):
        ids[wikidata.qid(r["c"])] = 0
    for cls, least in [("Q1637706", 0), ("Q174844", 0), ("Q1549591", 20)]:
        ids.update(wikidata.sparql_ids(f"SELECT ?c ?s WHERE {{ ?c wdt:P31 wd:{cls} ; wikibase:sitelinks ?s . FILTER(?s >= {least}) }}"))
    keys = sorted(ids)
    sys.stderr.write(f"{len(keys)} candidate cities\n")
    rows = []
    for k in range(0, len(keys), 150):
        values = " ".join("wd:" + q for q in keys[k:k + 150])
        rows += wikidata.sparql(f"""SELECT ?c ?cLabel ?iso ?pop ?t WHERE {{
  VALUES ?c {{ {values} }}
  ?c p:P1082 ?st . ?st ps:P1082 ?pop ; wikibase:rank ?rank . FILTER(?rank != wikibase:DeprecatedRank)
  OPTIONAL {{ ?st pq:P585 ?t }}
  ?c wdt:P17 ?country . ?country wdt:P297 ?iso .
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en,mul". }} }}""")
    best = {}
    for r in rows:
        if float(r["pop"]) < 1_000_000:
            continue
        key = (r.get("cLabel", "").lower(), r["iso"])
        year = wikidata.date(r.get("t", ""))[:4]
        if key not in best or year > best[key][1]:
            best[key] = (int(float(r["pop"])), year)
    data = json.load(open(PATH, encoding="utf-8"))
    fields = data["fields"]
    if "population_year" not in fields:
        fields.append("population_year")
    ni, ci, pi = fields.index("name"), fields.index("country"), fields.index("population")
    changed = []
    for row in data["rows"]:
        while len(row) < len(fields):
            row.append("")
        hit = best.get((row[ni].lower(), row[ci]))
        if hit and hit[0] != row[pi]:
            changed.append((row[ni], row[pi], hit[0], hit[1]))
            row[pi], row[-1] = hit[0], hit[1]
    json.dump(data, open(PATH, "w", encoding="utf-8"), ensure_ascii=False, separators=(",", ":"))
    print(f"{len(best)} Wikidata cities of 1 million or more; {len(changed)} populations updated")
    sys.stdout.flush()
    for name, old, new, year in sorted(changed, key=lambda c: -c[2])[:15]:
        print(f"  {name}: {old:,} -> {new:,} ({year})")


if __name__ == "__main__":
    main()
