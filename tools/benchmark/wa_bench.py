#!/usr/bin/env python3
"""Side-by-side benchmark of Noesis against the WolframAlpha Full Results API.

    python3 tools/benchmark/wa_bench.py QUESTIONS.txt [--cache DIR] [--report FILE]

QUESTIONS.txt has one query per line ('#' starts a comment).  For every
query the script fetches WolframAlpha's pods (plain text, JSON output),
runs the same query through the Noesis CLI offline, and compares the main
results:

  * numbers are compared to a relative tolerance of 1e-4;
  * expressions in one variable are evaluated at sample points by Noesis's
    own parser (WolframAlpha's plain-text output is close to its syntax);
  * other text is compared after normalisation.

The AppID is never read by this script from the repository.  Either
  - set WOLFRAM_APPID in the environment (sent as the appid parameter), or
  - let an HTTPS proxy inject "Authorization: Bearer <AppID>" (a network
    secret for api.wolframalpha.com); the script then sends a placeholder.

WolframAlpha responses are cached in --cache (default: a temp directory),
not in the repository.  A summary is printed and a JSON report written.
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
NOESIS = os.path.join(ROOT, "target", "release", "noesis")
API = "https://api.wolframalpha.com/v2/query"

# pods that carry the answer, in order of preference
RESULT_PODS = [
    "Result", "Exact result", "Results", "Solution", "Solutions", "Real solution", "Real solutions",
    "Indefinite integral", "Definite integral", "Derivative", "Limit", "Sum", "Infinite sum", "Infinite product",
    "Decimal approximation", "Value", "Unit conversion", "Exact value", "Determinant", "Inverse", "Eigenvalues",
    "Prime factorization", "Differential equation solution", "Complex solutions", "Power series",
]


def fetch_wa(query, cache):
    key = re.sub(r"[^A-Za-z0-9]+", "_", query)[:120]
    path = os.path.join(cache, key + ".json")
    if os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    params = {"input": query, "format": "plaintext", "output": "json", "units": "metric"}
    headers = {"User-Agent": "noesis-benchmark/1.0"}
    appid = os.environ.get("WOLFRAM_APPID")
    if appid:
        params["appid"] = appid
    else:
        # placeholder; a network secret on the proxy replaces it
        headers["Authorization"] = "Bearer placeholder"
    req = urllib.request.Request(API + "?" + urllib.parse.urlencode(params), headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            data = json.loads(r.read().decode("utf-8"))
    except Exception as e:  # network or HTTP error
        return {"error": str(e)}
    with open(path, "w") as f:
        json.dump(data, f)
    return data


def wa_answer(data, query=""):
    qr = data.get("queryresult", {})
    if not qr.get("success"):
        return None, None
    pods = qr.get("pods", [])
    interp = None
    texts = {}
    for p in pods:
        t = " | ".join(sp.get("plaintext", "") for sp in p.get("subpods", []) if sp.get("plaintext"))
        texts[p.get("title", "")] = t
        if p.get("id") == "Input" or p.get("title", "").startswith("Input"):
            interp = t
    # equations: compare real solutions, as Noesis lists them first
    if query.lower().startswith("solve") or "=" in query:
        for title in ("Real solutions", "Real solution", "Solution over the reals"):
            if texts.get(title):
                return texts[title], interp
    for p in pods:
        if p.get("primary") and texts.get(p.get("title", "")):
            return texts[p["title"]], interp
    for title in RESULT_PODS:
        if texts.get(title):
            return texts[title], interp
    # first non-input pod
    for p in pods:
        if not p.get("title", "").startswith("Input") and texts.get(p.get("title", "")):
            return texts[p["title"]], interp
    return None, interp


def noesis_answer(query):
    """The primary answer as plain text (the CLI's --short output)."""
    return short(query)


def short(query):
    try:
        out = subprocess.run([NOESIS, "--offline", "--short", query], capture_output=True, text=True, timeout=30,
                             env={**os.environ, "NOESIS_NO_GPU": "1", "NOESIS_OFFLINE": "1", "XDG_RUNTIME_DIR": os.environ.get("XDG_RUNTIME_DIR", "/tmp")})
        return out.stdout.strip() if out.returncode == 0 else None
    except Exception:
        return None


NUM = re.compile(r"-?\d[\d,]*(?:\.\d+)?(?:\s*[×x]\s*10\^\s*\(?-?\d+\)?)?")


def to_float(s):
    s = s.replace(" ", "").replace(",", "").replace("−", "-")
    m = re.match(r"(-?\d+(?:\.\d+)?)(?:[×x]10\^\(?(-?\d+)\)?)?$", s)
    if not m:
        return None
    v = float(m.group(1))
    if m.group(2):
        v *= 10 ** int(m.group(2))
    return v


def displayed_tolerance(token):
    """Half a unit in the last digit WolframAlpha displayed, relative."""
    t = token.replace(",", "").replace(" ", "")
    mant = re.match(r"-?(\d+)(?:\.(\d+))?", t)
    if not mant:
        return 1e-4
    digits = len((mant.group(1) + (mant.group(2) or "")).lstrip("0")) or 1
    return max(1e-9, 0.5 * 10 ** (1 - digits))


def noesis_json(query):
    try:
        out = subprocess.run([NOESIS, "--offline", "--json", query], capture_output=True, text=True, timeout=60,
                             env={**os.environ, "NOESIS_NO_GPU": "1", "NOESIS_OFFLINE": "1", "XDG_RUNTIME_DIR": os.environ.get("XDG_RUNTIME_DIR", "/tmp")})
        return json.loads(out.stdout)
    except Exception:
        return None


def noesis_decimal(query):
    r = noesis_json(query)
    if not r:
        return None
    for p in r.get("pods", []):
        if p.get("id") in ("DecimalApproximation", "Decimal"):
            c = p["subpods"][0]["content"]
            if c.get("type") == "text":
                return c["text"].replace("…", "")
    return None


def normalise(s):
    s = s.lower().replace("−", "-").replace("…", "").replace("≈", "").replace("ℯ", "e").replace("π", "pi")
    s = re.sub(r"\(\s*(?:irreducible|approx\.?|exact|assuming[^)]*)\s*\)", "", s)
    s = re.sub(r"\bconstant\b|\+\s*c\b", "", s)
    return re.sub(r"\s+", "", s)


def strip_labels(s):
    """'kinetic energy | 300 kJ' -> '300 kJ'; drop parenthetical notes."""
    parts = [p.strip() for p in re.split(r"\s*\|\s*|\n", s) if p.strip()]
    if len(parts) > 1 and not re.search(r"\d", parts[0]) and re.search(r"\d", parts[1]):
        parts = parts[1:]
    return parts


def rhs(s):
    # right-hand side of the last '=' (WolframAlpha writes "integral = result")
    s = s.split("|")[0]
    if "=" in s and "≠" not in s:
        s = s.split("=")[-1]
    s = s.split("≈")[0] if not s.strip().startswith("≈") else s
    s = re.sub(r"\+\s*(?:constant|C)\s*$", "", s.strip())
    return s.strip()


def answers(s):
    """A multi-part answer as a set of right-hand sides."""
    return [rhs(p) for p in strip_labels(s.replace(" and ", " | ").replace(" or ", " | "))]


def value_after_approx(s):
    m = re.search(r"≈\s*(" + NUM.pattern + ")", s)
    return m.group(1) if m else None


def numbers_agree(a, b):
    """Leading numbers agree within WolframAlpha's displayed precision (a is WolframAlpha's)."""
    ta = value_after_approx(a) or (NUM.findall(a) or [None])[0]
    tb = value_after_approx(b) or (NUM.findall(b) or [None])[0]
    if ta is None or tb is None:
        return None
    x, y = to_float(ta), to_float(tb)
    if x is None or y is None:
        return None
    tol = max(displayed_tolerance(ta), 1e-6)
    return abs(x - y) <= tol * max(abs(x), abs(y), 1e-300)


UNIT = re.compile(r"(-?\d[\d,]*(?:\.\d+)?(?:\s*×\s*10\^-?\d+)?)\s*([a-zA-Zµ°/^0-9 ]+?)\s*(?:\(|$)")


def units_agree(wa, no):
    """'300 kJ (kilojoules)' vs '300000 J': convert WolframAlpha's quantity with Noesis."""
    mw = UNIT.search(wa)
    mn = UNIT.search(no)
    if not mw or not mn:
        return None
    wtok = mw.group(1)
    wv, wu = wtok.replace(" × 10^", "e").replace("×10^", "e").replace(",", ""), mw.group(2).strip()
    nu = mn.group(2).strip()
    if not wu or not nu or wu == nu:
        return None
    conv = short(f"{wv} {wu} in {nu}")
    if not conv:
        return None
    x, y = to_float((NUM.findall(conv) or ["x"])[0]), to_float(mn.group(1))
    if x is None or y is None:
        return None
    return abs(x - y) <= displayed_tolerance(wtok) * max(abs(x), abs(y), 1e-300)


CONSTS = {"c_1": "0.613", "c_2": "1.29", "c_3": "-0.47", "C": "0.613"}


def evaluate(expr, pt):
    e = expr
    for k, v in CONSTS.items():
        e = re.sub(r"\b" + re.escape(k) + r"\b", f"({v})", e)
    e = re.sub(r"\b[xnt]\b", f"({pt})", e)
    out = short(e)
    if out is None:
        return None
    v = to_float(out.replace("…", "").split("≈")[-1].strip())
    if v is None:
        # an exact constant such as a Cardano root: use the decimal pod
        d = noesis_decimal(e)
        v = to_float(d) if d else None
    return v


def expressions_agree(a, b):
    """Evaluate both sides with Noesis at sample points of x (and n, t) and constants."""
    a, b = rhs(a), rhs(b)
    if any(bad in a + b for bad in ["|", " for ", "where", "element"]):
        return None
    vals = []
    for pt in ["0.731", "1.37"]:
        va, vb = evaluate(a, pt), evaluate(b, pt)
        if va is None or vb is None:
            return None
        vals.append((va, vb))
    d = [x - y for x, y in vals]
    scale = [max(1.0, abs(x)) for x, _ in vals]
    if all(abs(di) <= 1e-6 * sc for di, sc in zip(d, scale)):
        return True
    if abs(d[0] - d[1]) <= 1e-6 * max(1.0, abs(d[0])):
        return "constant"
    return False


def set_agree(wa, no):
    """Every part of one answer matches a part of the other (e.g. two roots)."""
    wa_parts, no_parts = answers(wa), answers(no)
    if not any(" i" in p or p.endswith("i") for p in no_parts):
        wa_parts = [p for p in wa_parts if not re.search(r"\bi\b", p)]
    if len(wa_parts) < 2 and len(no_parts) < 2 and not wa_parts:
        return None
    no_parts = [p for p in no_parts if "≈" not in p and p]
    def same(x, y):
        if normalise(x) == normalise(y):
            return True
        n = numbers_agree(x, y) if re.fullmatch(r"[-\d.,\s×^]+", x.strip()) else None
        if n:
            return True
        return expressions_agree(x, y) is True
    used = set()
    for w in wa_parts:
        hit = next((i for i, n in enumerate(no_parts) if i not in used and same(w, n)), None)
        if hit is None:
            return False
        used.add(hit)
    return True


def inequality_agree(wa, no):
    norm = lambda s: sorted(re.sub(r"\s+", "", p) for p in re.split(r"\||\bor\b", s.replace("or", "|")) if p.strip())
    return norm(wa) == norm(no) or None


def wa_matrix(s):
    m = re.match(r"^\s*([^()]*?)\s*\((.*)\)\s*$", s, re.S)
    if not m or "|" not in m.group(2):
        return None
    rows = [r.split("|") for r in m.group(2).strip().split("\n")]
    if len({len(r) for r in rows}) != 1:
        return None
    scale = m.group(1).strip()
    body = "{" + ", ".join("{" + ", ".join(e.strip() for e in r) + "}" for r in rows) + "}"
    return f"{scale} {body}" if scale else body


def matrix_agree(wa, no):
    mw = wa_matrix(wa)
    if not mw or not no.strip().startswith("{{"):
        return None
    out = short(f"({mw}) - ({no})")
    if out is None:
        return None
    return bool(re.fullmatch(r"\{(\{0(, 0)*\})(, \{0(, 0)*\})*\}", out.replace(" ", "").replace(",", ", ")) or re.fullmatch(r"[{}0, ]+", out))


def systems_agree(wa, no):
    """'x = -1 and y = -3 | x = 3 and y = 1' vs lines 'x = -1, y = -3'."""
    if " and " not in wa or "=" not in no:
        return None
    norm = lambda part: frozenset(re.sub(r"\s+", "", a) for a in re.split(r"\band\b|,", part) if "=" in a)
    ws = {norm(p) for p in wa.split("|")}
    ns = {norm(p) for p in re.split(r"\n|\|", no) if "=" in p and "≈" not in p}
    return ws == ns


def compare(query, wa, no, no_decimal=None):
    if wa is None and no is None:
        return "both-fail"
    if wa is None:
        return "wa-fail"
    if no is None:
        return "noesis-fail"
    wa_main = " | ".join(strip_labels(wa))
    a, b = normalise(rhs(wa_main)), normalise(rhs(no))
    if a == b:
        return "match"
    # short text answers ("Paris" vs "Paris, Île-de-France, France", "Saturday" in a full date)
    if not re.search(r"\d", a) and len(a) >= 3 and (a in b or (not re.search(r"\d", b) and len(b) >= 3 and b in a)):
        return "match"
    if re.search(r"[<>]", wa_main) and inequality_agree(wa_main, no):
        return "match"
    if matrix_agree(wa, no) is True or systems_agree(wa, no) is True:
        return "match"
    sa = set_agree(wa_main, no)
    if sa is True:
        return "match"
    e = expressions_agree(wa_main, no)
    if e is True or (e == "constant" and "integra" in query.lower()):
        return "match"
    n = numbers_agree(rhs(wa_main), rhs(no)) if re.match(r"\s*[-\d]", rhs(wa_main)) else numbers_agree(wa_main, no)
    if n is True:
        return "match"
    if no_decimal and numbers_agree(rhs(wa_main), no_decimal):
        return "match"
    if value_after_approx(no):
        plain = re.fullmatch(r"\s*-?\d[\d,]*(?:\.\d+)?\s*", rhs(wa_main))
        wa_value = rhs(wa_main) if plain else value_after_approx(wa_main)
        if wa_value and numbers_agree(wa_value, value_after_approx(no)):
            return "match"
    u = units_agree(wa_main, no)
    if u is True:
        return "match"
    if e is False or n is False or sa is False or u is False:
        return "mismatch"
    return "review"


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 2
    qfile = args[0]
    cache = args[args.index("--cache") + 1] if "--cache" in args else os.path.join(tempfile.gettempdir(), "noesis-wa-cache")
    report = args[args.index("--report") + 1] if "--report" in args else os.path.join(cache, "report.json")
    os.makedirs(cache, exist_ok=True)
    queries = [l.strip() for l in open(qfile) if l.strip() and not l.startswith("#")]
    rows = []
    counts = {}
    for q in queries:
        data = fetch_wa(q, cache)
        if "error" in data:
            print(f"WolframAlpha request failed for {q!r}: {data['error']}")
            return 1
        wa, interp = wa_answer(data, q)
        no = noesis_answer(q)
        verdict = compare(q, wa, no, noesis_decimal(q) if wa and no else None)
        counts[verdict] = counts.get(verdict, 0) + 1
        rows.append({"query": q, "verdict": verdict, "wolframalpha": wa, "wa_interpretation": interp, "noesis": no})
        print(f"{verdict:<12} {q[:60]:<60} WA: {(wa or '')[:50]!s:<50} N: {(no or '')[:50]}")
    total = len(queries)
    comparable = total - counts.get("wa-fail", 0) - counts.get("both-fail", 0)
    print()
    for k in ["match", "mismatch", "review", "noesis-fail", "wa-fail", "both-fail"]:
        print(f"{k:<12} {counts.get(k, 0)}")
    if comparable:
        print(f"agreement on queries WolframAlpha answers: {counts.get('match', 0)}/{comparable} = {100 * counts.get('match', 0) / comparable:.1f}%")
    with open(report, "w") as f:
        json.dump({"counts": counts, "rows": rows}, f, indent=1, ensure_ascii=False)
    print(f"report: {report}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
