#!/usr/bin/env python3
"""Build crates/engine/data/factbook.z, country indicators from the CIA World Factbook.

    python3 tools/datagen/factbook_stats.py FACTBOOK_DIR

FACTBOOK_DIR is a checkout of https://github.com/factbook/factbook.json
(CC0; the Factbook itself is a US government work in the public domain).

Output (raw deflate, UTF-8, tab-separated lines):
    country<TAB>field<TAB>subfield<TAB>text
where country is the name used in countries.json when the entity matches
one of its countries, and the Factbook's short name otherwise (European
Union, World, ...).  For a yearly series only the latest year is kept.
Long narrative fields are left out.
"""
import glob
import html
import json
import os
import re
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
COUNTRIES = os.path.join(ROOT, "crates", "engine", "data", "countries.json")
OUT = os.path.join(ROOT, "crates", "engine", "data", "factbook.z")

SKIP_SECTIONS = {"Introduction", "Terrorism", "Transnational Issues", "Space"}
SKIP_FIELDS = {
    "Economic overview", "Broadcast media", "Military deployments", "Military equipment inventories and acquisitions",
    "Military - note", "Geography - note", "International organization participation", "Map references",
    "Household income or consumption by percentage share", "International environmental agreements",
    "Executive branch", "Legislative branch", "Legislative branch - lower chamber", "Legislative branch - upper chamber",
    "Judicial branch", "Political parties", "Population distribution", "Dependent areas", "International law organization participation",
    "Military and security forces", "Citizenship", "Average household expenditures", "Remittances", "Government - note",
    "People - note", "Diplomatic representation in the US", "Diplomatic representation from the US", "Economy - note", "Communications - note", "Transportation - note", "Energy - note", "Area - comparative",
}
MAX = 420


def key(s):
    s = re.sub(r"\(.*?\)", "", s.lower()).replace("&", "and")
    s = re.sub(r"^the\s+|,?\s+the$", "", s.strip())
    return re.sub(r"[^a-z]+", "", s)


def clean(t):
    t = re.sub(r"<br\s*/?>", "; ", t or "")
    t = re.sub(r"<[^>]+>", "", t)
    t = html.unescape(t).replace("\t", " ").replace("\n", " ")
    t = re.sub(r"\s*;\s*;\s*", "; ", t)
    t = re.sub(r"\s+", " ", t).strip(" ;")
    return t if len(t) <= MAX else t[: MAX - 1].rsplit(" ", 1)[0] + "…"


def labelled(t):
    """Split '<strong>description:</strong> ... <strong>meaning:</strong> ...' into parts."""
    parts = re.split(r"<(?:strong|b)>\s*([^<:]+):\s*</(?:strong|b)>", t or "")
    if len(parts) < 3:
        return None
    return [(parts[i].strip(), parts[i + 1]) for i in range(1, len(parts) - 1, 2)]


def rows_of(field, v):
    if isinstance(v, str):
        return []
    if "text" in v and isinstance(v["text"], str):
        lab = labelled(v["text"])
        if lab:
            return [(sub, clean(t)) for sub, t in lab if clean(t)]
        return [("", clean(v["text"]))]
    subs = [(k, x) for k, x in v.items() if k != "note" and isinstance(x, dict)]
    # a yearly series: "Unemployment rate 2024", "Unemployment rate 2023", ...
    years = [k for k, _ in subs if re.search(r"\b(19|20)\d\d$", k)]
    if years and len(years) == len(subs):
        return [("", clean(subs[0][1].get("text", "")))]
    out = []
    for k, x in subs:
        if "text" in x:
            out.append((k.strip(), clean(x["text"])))
        else:
            for k2, y in x.items():
                if isinstance(y, dict) and "text" in y:
                    out.append((f"{k.strip()}: {k2.strip()}", clean(y["text"])))
    return out


def main():
    ours = {key(c["name"]): c["name"] for c in json.load(open(COUNTRIES, encoding="utf-8")).values()}
    alias = {"burma": "Myanmar", "cotedivoire": "Ivory Coast", "koreasouth": "South Korea", "koreanorth": "North Korea",
             "gambia": "Gambia", "bahamas": "Bahamas", "congodemocraticrepublicofthe": "Democratic Republic of the Congo",
             "congorepublicofthe": "Republic of the Congo", "micronesiafederatedstatesof": "Micronesia",
             "turkeyturkiye": "Turkey", "czechia": "Czechia", "holysee": "Vatican"}
    lines = []
    names = 0
    for path in sorted(glob.glob(os.path.join(sys.argv[1], "*", "*.json"))):
        d = json.load(open(path, encoding="utf-8"))
        cn = d.get("Government", {}).get("Country name", {})
        short = cn.get("conventional short form", {}).get("text", "")
        long = cn.get("conventional long form", {}).get("text", "")
        if not short or short.lower() == "none":
            short = long if long and long.lower() != "none" else os.path.basename(path)[:-5]
        short = clean(re.sub(r"\s*\(.*?\)", "", short))
        name = None
        for cand in [short, long]:
            if cand and cand.lower() != "none":
                k = key(cand)
                name = ours.get(k) or (alias.get(k) if alias.get(k) and key(alias[k]) in ours else None) or name
        if name is None and key(short) in alias and key(alias[key(short)]) in ours:
            name = ours[key(alias[key(short)])]
        name = name or short
        code = os.path.basename(path)[:-5]
        name = {"ee": "European Union", "xx": "World", "oo": "Southern Ocean", "xo": "Indian Ocean", "xq": "Arctic Ocean",
                "zh": "Atlantic Ocean", "zn": "Pacific Ocean"}.get(code, name)
        name = {"Côte d'Ivoire": "Ivory Coast", "Federated States of Micronesia": "Micronesia", "Macau": "Macao",
                "Pitcairn Islands": "Pitcairn", "Virgin Islands": "U.S. Virgin Islands", "Gaza, Gaza Strip": "Gaza Strip",
                "Saint Helena, Ascension, and Tristan da Cunha": "Saint Helena"}.get(name, name)
        names += 1
        for section, fields in d.items():
            if section in SKIP_SECTIONS or not isinstance(fields, dict):
                continue
            for field, v in fields.items():
                if field in SKIP_FIELDS or not isinstance(v, dict):
                    continue
                for sub, t in rows_of(field, v):
                    if t and t.lower() not in ("na", "none", "n/a"):
                        lines.append(f"{name}\t{field}\t{sub}\t{t}")
    data = "\n".join(lines).encode("utf-8")
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    with open(OUT, "wb") as f:
        f.write(c.compress(data) + c.flush())
    print(f"{len(lines)} rows for {names} entities; {len(data) // 1024} KB, {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
