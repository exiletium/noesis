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


def wa_answer(data):
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


NUM = re.compile(r"-?\d+(?:\.\d+)?(?:\s*[×x]\s*10\^\s*\(?-?\d+\)?)?")


def to_float(s):
    s = s.replace(" ", "")
    m = re.match(r"(-?\d+(?:\.\d+)?)(?:[×x]10\^\(?(-?\d+)\)?)?$", s)
    if not m:
        return None
    v = float(m.group(1))
    if m.group(2):
        v *= 10 ** int(m.group(2))
    return v


def normalise(s):
    s = s.lower().replace("−", "-").replace("…", "").replace("≈", "").replace("ℯ", "e").replace("π", "pi")
    s = re.sub(r"\(\s*(?:irreducible|approx\.?|exact)\s*\)", "", s)
    s = re.sub(r"\bconstant\b|\+\s*c\b", "", s)
    return re.sub(r"\s+", "", s)


def rhs(s):
    # take the right-hand side of the last '=' (WolframAlpha writes "integral = result")
    s = s.split("|")[0]
    if "=" in s and "≠" not in s:
        s = s.split("=")[-1]
    s = re.sub(r"\+\s*(?:constant|C)\s*$", "", s.strip())
    return s.strip()


def numbers_agree(a, b):
    fa = [to_float(x) for x in NUM.findall(a.replace(",", ""))]
    fb = [to_float(x) for x in NUM.findall(b.replace(",", ""))]
    fa, fb = [x for x in fa if x is not None], [x for x in fb if x is not None]
    if not fa or not fb:
        return None
    x, y = fa[0], fb[0]
    return abs(x - y) <= 1e-4 * max(1.0, abs(x), abs(y))


def expressions_agree(a, b):
    """Evaluate both sides with Noesis at sample points of x (and n, t)."""
    a, b = rhs(a), rhs(b)
    for bad in ["|", "for", "where", "≈"]:
        if bad in a or bad in b:
            return None
    a = a.replace("log", "ln") if "ln" not in a else a
    vals = []
    for pt in ["0.731", "1.37"]:
        pa = re.sub(r"\b[xnt]\b", f"({pt})", a)
        pb = re.sub(r"\b[xnt]\b", f"({pt})", b.replace(" constant", ""))
        va, vb = short(pa), short(pb)
        if va is None or vb is None:
            return None
        fa, fb = to_float(va.replace("…", "")), to_float(vb.replace("…", ""))
        if fa is None or fb is None:
            return None
        vals.append((fa, fb))
    # antiderivatives may differ by a constant
    d0 = vals[0][0] - vals[0][1]
    d1 = vals[1][0] - vals[1][1]
    if abs(d0) <= 1e-6 * max(1, abs(vals[0][0])) and abs(d1) <= 1e-6 * max(1, abs(vals[1][0])):
        return True
    if abs(d0 - d1) <= 1e-6 * max(1, abs(d0)):
        return "constant"
    return False


def compare(query, wa, no):
    if wa is None and no is None:
        return "both-fail"
    if wa is None:
        return "wa-fail"
    if no is None:
        return "noesis-fail"
    a, b = normalise(rhs(wa)), normalise(rhs(no))
    if a == b:
        return "match"
    # short text answers: one contained in the other ("Paris" vs "Paris, Île-de-France, France")
    if not re.search(r"\d", a + b) and min(len(a), len(b)) >= 3 and (a in b or b in a):
        return "match"
    e = expressions_agree(wa, no)
    if e is True or (e == "constant" and "integra" in query.lower()):
        return "match"
    n = numbers_agree(rhs(wa), rhs(no))
    if n is True:
        return "match"
    if e is False or n is False:
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
        wa, interp = wa_answer(data)
        no = noesis_answer(q)
        verdict = compare(q, wa, no)
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
