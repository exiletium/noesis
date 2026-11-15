#!/usr/bin/env python3
"""Update the populations of large cities in cities.json from Wikidata (CC0).

    python3 tools/datagen/cities.py [--cache DIR]

GeoNames often counts a city proper (Tokyo's 23 special wards) where
reference works count the city as an administrative whole (the Tokyo
Metropolis).  For every city with a Wikidata population of a million or
more, the most recent figure (preferred rank) replaces the bundled one when
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
    rows = wikidata.sparql("""SELECT ?c ?cLabel ?iso ?pop ?t WHERE {
  ?c p:P1082 ?st . ?st ps:P1082 ?pop ; wikibase:rank wikibase:PreferredRank . FILTER(?pop >= 1000000)
  OPTIONAL { ?st pq:P585 ?t }
  ?c wdt:P17 ?country . ?country wdt:P297 ?iso .
  SERVICE wikibase:label { bd:serviceParam wikibase:language "en". } }""")
    best = {}
    for r in rows:
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
    for name, old, new, year in sorted(changed, key=lambda c: -c[2])[:15]:
        print(f"  {name}: {old:,} -> {new:,} ({year})")


if __name__ == "__main__":
    main()
