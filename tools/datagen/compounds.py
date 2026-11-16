#!/usr/bin/env python3
"""Chemical compounds from Wikidata (CC0): names, formula and a few physical
properties, written to crates/engine/data/compounds.z (raw deflate).

    python3 tools/datagen/compounds.py [--cache DIR] [--min-sitelinks N] [--ids FILE]

Each line: name<TAB>formula<TAB>melting K<TAB>boiling K<TAB>density kg/m^3
<TAB>aliases separated by "|".  Empty fields are left blank.  Only items
with at least N Wikipedia/Wikimedia sitelinks (default 2) are kept, which
selects the compounds people ask about.
"""
import os
import re
import sys
import zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import wikidata  # noqa: E402

OUT = os.path.join(wikidata.ROOT, "crates", "engine", "data", "compounds.z")
SUB = str.maketrans("₀₁₂₃₄₅₆₇₈₉", "0123456789")


def formula(f):
    f = f.translate(SUB).replace(" ", "")
    # keep plain formulas and hydrates; drop charges, isotopes and ranges
    return f if re.fullmatch(r"[A-Z][A-Za-z0-9()\[\]·.]*", f) else ""


def main():
    if "--cache" in sys.argv:
        wikidata.CACHE = sys.argv[sys.argv.index("--cache") + 1]
        os.makedirs(wikidata.CACHE, exist_ok=True)
    least = int(sys.argv[sys.argv.index("--min-sitelinks") + 1]) if "--min-sitelinks" in sys.argv else 2
    if "--ids" in sys.argv:
        # an id list saved earlier: the id query streams for close to the
        # endpoint's time limit and can come back cut short
        import json
        ids = {k: v for k, v in json.load(open(sys.argv[sys.argv.index("--ids") + 1])).items() if v >= least}
    else:
        ids = wikidata.sparql_ids(f"SELECT ?x ?s WHERE {{ ?x wdt:P274 ?f ; wikibase:sitelinks ?s . FILTER(?s >= {least}) }}")
    keys = sorted(ids)
    sys.stderr.write(f"{len(keys)} compounds with {least}+ sitelinks\n")
    out = {}
    for k in range(0, len(keys), 200):
        values = " ".join("wd:" + q for q in keys[k:k + 200])
        rows = wikidata.sparql(f"""SELECT ?x ?xLabel ?f ?mp ?bp ?d WHERE {{
  VALUES ?x {{ {values} }}
  ?x wdt:P274 ?f .
  OPTIONAL {{ ?x p:P2101/psn:P2101/wikibase:quantityAmount ?mp }}
  OPTIONAL {{ ?x p:P2102/psn:P2102/wikibase:quantityAmount ?bp }}
  OPTIONAL {{ ?x p:P2054/psn:P2054/wikibase:quantityAmount ?d }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en,mul". }}
}}""")
        for r in rows:
            q = wikidata.qid(r["x"])
            name = wikidata.clean(r.get("xLabel", ""))
            f = formula(r.get("f", ""))
            if not f or not name or re.fullmatch(r"Q\d+", name):
                continue
            e = out.setdefault(q, {"name": name, "f": f, "mp": "", "bp": "", "d": "", "alias": set(), "s": ids.get(q, 0)})
            for key in ("mp", "bp", "d"):
                if r.get(key) and not e[key]:
                    try:
                        e[key] = f"{float(r[key]):.6g}"
                    except ValueError:
                        pass
        sys.stderr.write(f"\r{min(k + 200, len(keys))}/{len(keys)}")
    # English aliases ("NaCl" is an alias of sodium chloride, "aspirin" of acetylsalicylic acid)
    for k in range(0, len(keys), 300):
        values = " ".join("wd:" + q for q in keys[k:k + 300])
        for r in wikidata.sparql(f'SELECT ?x ?a WHERE {{ VALUES ?x {{ {values} }} ?x skos:altLabel ?a . FILTER(LANG(?a) = "en") }}'):
            q = wikidata.qid(r["x"])
            if q in out:
                a = wikidata.clean(r["a"])
                if 2 < len(a) <= 60:
                    out[q]["alias"].add(a)
    sys.stderr.write("\n")
    # one line per name; the item with more sitelinks wins a shared name
    lines = {}
    for e in sorted(out.values(), key=lambda e: e["s"]):
        aliases = "|".join(sorted(a for a in e["alias"] if a.lower() != e["name"].lower()))
        lines[e["name"].lower()] = "\t".join([e["name"], e["f"], e["mp"], e["bp"], e["d"], aliases])
    text = "\n".join(sorted(lines.values(), key=str.lower)) + "\n"
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    data = c.compress(text.encode()) + c.flush()
    open(OUT, "wb").write(data)
    print(f"{len(lines)} compounds, {len(data)} bytes")


if __name__ == "__main__":
    main()
