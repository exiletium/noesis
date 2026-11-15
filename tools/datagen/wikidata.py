#!/usr/bin/env python3
"""Build the bundled general-knowledge facts from Wikidata (CC0).

    python3 tools/datagen/wikidata.py [--cache DIR]

Runs class-anchored SPARQL queries against query.wikidata.org and writes
crates/engine/data/facts.z (raw deflate, tab-separated lines):

  W  title  kind  role  names  year          creative works and their creators
  E  name   aliases  start  end  point       events (wars, battles, revolutions, treaties)
  P  office  holder  start  end  ordinal     holders of the top offices of each country
  H  name   description  born  died  birthplace   the people named above
  A  award  year  winners                    Nobel prizes

Only entities with many Wikipedia language editions are kept, as a proxy
for notability.  Dates are ISO (negative years for BCE), day precision
where Wikidata has it.
"""
import hashlib
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "facts.z")
ENDPOINT = "https://query.wikidata.org/sparql"
API = "https://www.wikidata.org/w/api.php"
UA = "noesis-datagen/0.1 (open-source data build)"
CACHE = None

# (class, kind, property, role, minimum sitelinks)
WORKS = [
    ("Q7725634", "literary work", "P50", "author", 20), ("Q8261", "novel", "P50", "author", 20), ("Q25379", "play", "P50", "author", 15),
    ("Q5185279", "poem", "P50", "author", 15), ("Q571", "book", "P50", "author", 20), ("Q49084", "short story", "P50", "author", 15),
    ("Q37484", "epic poem", "P50", "author", 15), ("Q149537", "novella", "P50", "author", 15), ("Q47461344", "written work", "P50", "author", 25),
    ("Q1667921", "novel series", "P50", "author", 20), ("Q277759", "book series", "P50", "author", 20), ("Q80930", "tragedy", "P50", "author", 15),
    ("Q11424", "film", "P57", "director", 40), ("Q24862", "short film", "P57", "director", 30), ("Q202866", "animated film", "P57", "director", 35),
    ("Q5398426", "television series", "P170", "creator", 40),
    ("Q3305213", "painting", "P170", "painter", 15), ("Q860861", "sculpture", "P170", "sculptor", 15), ("Q4502142", "visual artwork", "P170", "artist", 20),
    ("Q1344", "opera", "P86", "composer", 15), ("Q9734", "symphony", "P86", "composer", 10), ("Q105543609", "musical work", "P86", "composer", 15),
    ("Q207628", "musical composition", "P86", "composer", 15), ("Q2743", "musical", "P86", "composer", 20),
    ("Q482994", "album", "P175", "performer", 30), ("Q7366", "song", "P175", "performer", 30), ("Q134556", "single", "P175", "performer", 30),
    ("Q7889", "video game", "P178", "developer", 35),
    ("Q116476516", "play", "P50", "author", 15), ("Q1279564", "literary work", "P50", "author", 20),
    ("g:Q8261", "novel", "P50", "author", 20), ("g:Q25379", "play", "P50", "author", 15), ("g:Q80930", "tragedy", "P50", "author", 15),
    ("g:Q40831", "comedy", "P50", "author", 20), ("g:Q482", "poem", "P50", "author", 20),
    ("g:Q1344", "opera", "P86", "composer", 10), ("g:Q9734", "symphony", "P86", "composer", 10), ("g:Q2743", "musical", "P86", "composer", 15),
    ("g:Q1344", "opera", "P87", "librettist", 10),
]
EVENTS = [("Q198", 20), ("Q178561", 30), ("Q10931", 25), ("Q131569", 25), ("Q8465", 25), ("Q124757", 30), ("Q3839081", 40), ("Q13418847", 40), ("Q1190554", 60)]
NOBEL = ["Q38104", "Q44585", "Q80061", "Q37922", "Q35637", "Q47170"]


def fetch(url, data=None, accept="application/sparql-results+json", valid=None):
    """POST a query; a body is cached only once `valid(body)` accepts it, because
    the endpoint can cut a response off mid-stream when a query runs long."""
    key = hashlib.sha1((url + (data or "") + accept).encode()).hexdigest()
    path = os.path.join(CACHE, key) if CACHE else None
    if path and os.path.exists(path):
        body = open(path, encoding="utf-8").read()
        if valid is None or valid(body):
            return body
    for attempt in range(6):
        try:
            req = urllib.request.Request(url, data=data.encode() if data else None, headers={"User-Agent": UA, "Accept": accept})
            body = urllib.request.urlopen(req, timeout=75).read().decode("utf-8")
            time.sleep(1.0)
            if valid is not None and not valid(body):
                raise ValueError(f"incomplete response ({len(body)} bytes)")
            if path:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(body)
            return body
        except urllib.error.HTTPError as err:
            wait = int(err.headers.get("Retry-After", "0") or 0) if err.code == 429 else 0
            sys.stderr.write(f"  retry {attempt + 1}: {err}\n")
            time.sleep(max(wait, 10 * (attempt + 1)))
        except Exception as err:  # noqa: BLE001 - retry any transport error
            sys.stderr.write(f"  retry {attempt + 1}: {err}\n")
            time.sleep(10 * (attempt + 1))
    return None


def parses(body):
    try:
        json.loads(body)
        return True
    except ValueError:
        return False


LABELS = {}


def resolve_labels(qids):
    """English labels for items the label service left unlabelled (it returns
    the bare id under load)."""
    todo = sorted({q for q in qids if q not in LABELS})
    for k in range(0, len(todo), 200):
        values = " ".join("wd:" + q for q in todo[k:k + 200])
        body = fetch(ENDPOINT, "query=" + urllib.parse.quote(f'SELECT ?x ?l WHERE {{ VALUES ?x {{ {values} }} ?x rdfs:label ?l . FILTER(LANG(?l) = "en") }}'), valid=parses)
        for r in json.loads(body)["results"]["bindings"] if body else []:
            LABELS[qid(r["x"]["value"])] = r["l"]["value"]


def sparql(query):
    body = fetch(ENDPOINT, "query=" + urllib.parse.quote(query), valid=parses)
    if not body:
        return []
    rows = [{k: v["value"] for k, v in r.items()} for r in json.loads(body)["results"]["bindings"]]
    missing = [r[k] for r in rows for k in r if k.endswith("Label") and re.fullmatch(r"Q\d+", r[k])]
    if missing:
        resolve_labels(missing)
        for r in rows:
            for k in list(r):
                if k.endswith("Label") and r[k] in LABELS:
                    r[k] = LABELS[r[k]]
    return rows


def sparql_ids(query):
    """{qid: sitelinks} for a query selecting ?x and ?s, fetched as TSV (small)."""
    def whole(body):
        return body.startswith("?") and body.endswith("\n")

    body = fetch(ENDPOINT, "query=" + urllib.parse.quote(query), accept="text/tab-separated-values", valid=whole)
    out = {}
    for line in (body or "").splitlines()[1:]:
        m = re.match(r"<http://www.wikidata.org/entity/(Q\d+)>\t\"?(\d+)", line)
        if m:
            out[m.group(1)] = int(m.group(2))
    return out


def qid(uri):
    return uri.rsplit("/", 1)[-1]


def date(value):
    """ISO date from a Wikidata timestamp; January 1 is taken as year-only."""
    m = re.match(r"(-?\d+)-(\d\d)-(\d\d)", value or "")
    if not m:
        return ""
    y, mo, d = int(m[1]), m[2], m[3]
    return str(y) if (mo, d) == ("01", "01") else f"{y}-{mo}-{d}"


def clean(s):
    return re.sub(r"[\t\n|]+", " ", s or "").strip()


def is_label(s):
    return bool(s) and not re.fullmatch(r"Q\d+", s)


def main():
    global CACHE
    if "--cache" in sys.argv:
        CACHE = sys.argv[sys.argv.index("--cache") + 1]
        os.makedirs(CACHE, exist_ok=True)
    lines, people = [], {}

    works = {}
    for cls, kind, prop, role, least in WORKS:
        # step 1: the ids only (cheap); step 2: details in batches
        via, cls = ("wdt:P136", cls[2:]) if cls.startswith("g:") else ("wdt:P31", cls)
        links = sparql_ids(f"SELECT ?w ?s WHERE {{ ?w {via} wd:{cls} ; wikibase:sitelinks ?s . FILTER(?s >= {least}) }}")
        found = 0
        keys = sorted(links)
        for k in range(0, len(keys), 150):
            values = " ".join("wd:" + q for q in keys[k:k + 150])
            rows = sparql(f"""SELECT ?w ?wLabel ?a ?aLabel (MIN(?d) AS ?date) WHERE {{
  VALUES ?w {{ {values} }} ?w wdt:{prop} ?a . OPTIONAL {{ ?w wdt:P577 ?d }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }} GROUP BY ?w ?wLabel ?a ?aLabel""")
            for r in rows:
                if not (is_label(r.get("wLabel")) and is_label(r.get("aLabel"))):
                    continue
                key = (qid(r["w"]), role)
                w = works.setdefault(key, {"title": clean(r["wLabel"]), "kind": kind, "role": role, "names": [], "year": date(r.get("date", "")).split("-")[0] if r.get("date") else "", "links": links[qid(r["w"])]})
                if clean(r["aLabel"]) not in w["names"]:
                    w["names"].append(clean(r["aLabel"]))
                    found += 1
                people[qid(r["a"])] = None
        sys.stderr.write(f"{kind}: {len(links)} works, {found} creators\n")
    for w in sorted(works.values(), key=lambda x: -x["links"]):
        lines.append("\t".join(["W", w["title"], w["kind"], w["role"], "; ".join(w["names"][:6]), w["year"]]))

    events = {}
    for cls, least in EVENTS:
        rows = sparql(f"""SELECT ?e ?eLabel ?s (MIN(?st) AS ?start) (MAX(?en) AS ?end) (MIN(?pt) AS ?point) (GROUP_CONCAT(DISTINCT ?alt; separator="|") AS ?alts) WHERE {{
  ?e wdt:P31/wdt:P279* wd:{cls} ; wikibase:sitelinks ?s . FILTER(?s >= {least})
  OPTIONAL {{ ?e wdt:P580 ?st }} OPTIONAL {{ ?e wdt:P582 ?en }} OPTIONAL {{ ?e wdt:P585 ?pt }}
  OPTIONAL {{ ?e skos:altLabel ?alt . FILTER(LANG(?alt) = "en") }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }} GROUP BY ?e ?eLabel ?s""")
        sys.stderr.write(f"events {cls}: {len(rows)} rows\n")
        for r in rows:
            if is_label(r.get("eLabel")) and (r.get("start") or r.get("end") or r.get("point")):
                events[qid(r["e"])] = (int(r["s"]), "\t".join(["E", clean(r["eLabel"]), "|".join(clean(a) for a in r.get("alts", "").split("|")[:12] if a), date(r.get("start")), date(r.get("end")), date(r.get("point"))]))
    lines += [line for _, line in sorted(events.values(), key=lambda x: -x[0])]

    offices = sparql("""SELECT DISTINCT ?o ?oLabel (GROUP_CONCAT(DISTINCT ?alt; separator="|") AS ?alts) WHERE {
  ?c wdt:P31 wd:Q3624078 . { ?c wdt:P1906 ?o } UNION { ?c wdt:P1313 ?o }
  OPTIONAL { ?o skos:altLabel ?alt . FILTER(LANG(?alt) = "en") }
  SERVICE wikibase:label { bd:serviceParam wikibase:language "en". } } GROUP BY ?o ?oLabel""")
    # plain names: "president of France" for "President of the French Republic"
    plain = {}
    for r in sparql("""SELECT ?o ?cLabel ?role WHERE {
  ?c wdt:P31 wd:Q3624078 . { ?c wdt:P1906 ?o . BIND("head of state" AS ?role) } UNION { ?c wdt:P1313 ?o . BIND("head of government" AS ?role) }
  SERVICE wikibase:label { bd:serviceParam wikibase:language "en". } }"""):
        if is_label(r.get("cLabel")):
            plain.setdefault(qid(r["o"]), []).append((clean(r["cLabel"]), r["role"]))
    for o in offices:
        names = []
        for country, role in plain.get(qid(o["o"]), []):
            names.append(f"{role} of {country}")
            for title in ["president", "prime minister", "king", "queen", "monarch", "emperor", "chancellor", "premier", "sultan", "emir"]:
                if title in o.get("oLabel", "").lower():
                    names.append(f"{title} of {country}")
        o["alts"] = "|".join(names + [a for a in o.get("alts", "").split("|") if a])
    offices += [{"o": "http://www.wikidata.org/entity/" + q, "oLabel": name, "alts": ""} for q, name in [("Q19546", "pope"), ("Q11699", "Vice President of the United States"), ("Q14212", "Prime Minister of the United Kingdom")]]
    sys.stderr.write(f"offices: {len(offices)}\n")
    seen = set()
    for o in offices:
        if not is_label(o.get("oLabel")) or qid(o["o"]) in seen:
            continue
        seen.add(qid(o["o"]))
        rows = sparql(f"""SELECT ?h ?hLabel ?st ?en ?ord WHERE {{
  ?h wdt:P31 wd:Q5 ; p:P39 ?stmt . ?stmt ps:P39 wd:{qid(o["o"])} .
  OPTIONAL {{ ?stmt pq:P580 ?st }} OPTIONAL {{ ?stmt pq:P582 ?en }} OPTIONAL {{ ?stmt pq:P1545 ?ord }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }}""")
        label = clean(o["oLabel"])
        alts = "|".join(clean(a) for a in o.get("alts", "").split("|")[:16] if a)
        terms = set()
        for r in rows:
            start = date(r.get("st"))
            if not is_label(r.get("hLabel")) or not start:
                continue
            term = (clean(r["hLabel"]), start)
            if term in terms:
                continue
            terms.add(term)
            lines.append("\t".join(["P", label, alts, clean(r["hLabel"]), start, date(r.get("en")), r.get("ord", "") if re.fullmatch(r"\d+", r.get("ord", "")) else ""]))
            people[qid(r["h"])] = None

    for prize in NOBEL:
        rows = sparql(f"""SELECT ?p ?pLabel ?t ?awardLabel WHERE {{
  BIND(wd:{prize} AS ?award) ?p p:P166 ?stmt . ?stmt ps:P166 ?award . OPTIONAL {{ ?stmt pq:P585 ?t }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }}""")
        by_year = {}
        for r in rows:
            if is_label(r.get("pLabel")):
                people[qid(r["p"])] = None
                if r.get("t"):
                    by_year.setdefault(date(r["t"])[:4], []).append(clean(r["pLabel"]))
        for year, names in sorted(by_year.items()):
            lines.append("\t".join(["A", clean(rows[0].get("awardLabel", "")), year, "; ".join(sorted(set(names)))]))

    for q in sparql_ids("SELECT ?h ?s WHERE { ?h wdt:P31 wd:Q5 ; wikibase:sitelinks ?s . FILTER(?s >= 90) }"):
        people[q] = None
    ids = sorted(people)
    sys.stderr.write(f"people: {len(ids)}\n")
    for k in range(0, len(ids), 250):
        values = " ".join("wd:" + q for q in ids[k:k + 250])
        rows = sparql(f"""SELECT ?h ?hLabel ?hDescription (MIN(?b) AS ?born) (MIN(?d) AS ?died) (SAMPLE(?placeLabel) AS ?place) WHERE {{
  VALUES ?h {{ {values} }} ?h wdt:P31 wd:Q5 .
  OPTIONAL {{ ?h wdt:P569 ?b }} OPTIONAL {{ ?h wdt:P570 ?d }}
  OPTIONAL {{ ?h wdt:P19 ?pl . ?pl rdfs:label ?placeLabel . FILTER(LANG(?placeLabel) = "en") }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }} GROUP BY ?h ?hLabel ?hDescription""")
        for r in rows:
            if is_label(r.get("hLabel")) and r.get("born"):
                lines.append("\t".join(["H", clean(r["hLabel"]), clean(r.get("hDescription", "")), date(r.get("born")), date(r.get("died")), clean(r.get("place", ""))]))

    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    with open(OUT, "wb") as f:
        f.write(c.compress("\n".join(lines).encode("utf-8")) + c.flush())
    kinds = {}
    for line in lines:
        kinds[line[0]] = kinds.get(line[0], 0) + 1
    print(kinds, os.path.getsize(OUT), "bytes")


if __name__ == "__main__":
    main()
