#!/usr/bin/env python3
"""How much of a question file does Noesis understand?  (No WolframAlpha calls.)

    python3 tools/benchmark/coverage.py QUESTIONS.txt [--report FILE] [--online] [--jobs N]

Runs every question through the Noesis CLI and reports, per "# topic:"
section and overall, how many produced an answer.  This measures
understanding and coverage only, not correctness; wa_bench.py measures
agreement with WolframAlpha.
"""
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
NOESIS = os.environ.get("NOESIS_BIN") or os.path.join(ROOT, "target", "release", "noesis")
ONLINE = "--online" in sys.argv


def run(q):
    env = {**os.environ, "NOESIS_NO_GPU": "1", "XDG_RUNTIME_DIR": os.environ.get("XDG_RUNTIME_DIR", "/tmp")}
    if not ONLINE:
        env["NOESIS_OFFLINE"] = "1"
    try:
        out = subprocess.run([NOESIS] + ([] if ONLINE else ["--offline"]) + ["--short", q], capture_output=True, text=True, timeout=40, env=env)
        text = out.stdout.strip()
    except subprocess.TimeoutExpired:
        text = ""
    ok = bool(text) and not text.startswith("Noesis did not understand")
    return q, ok, text[:200]


def main():
    path = sys.argv[1]
    jobs = int(sys.argv[sys.argv.index("--jobs") + 1]) if "--jobs" in sys.argv else 4
    report = sys.argv[sys.argv.index("--report") + 1] if "--report" in sys.argv else None
    items, topic = [], "(none)"
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if line.startswith("# topic:"):
            topic = line.split(":", 1)[1].strip()
        elif line and not line.startswith("#"):
            items.append((topic, line.split("#!")[0].strip()))
    with ThreadPoolExecutor(jobs) as pool:
        results = list(pool.map(lambda it: (it[0],) + run(it[1]), items))
    by = {}
    for topic, q, ok, text in results:
        section = "/".join(topic.strip("/").split("/")[1:3]) or topic
        t = by.setdefault(section, [0, 0])
        t[0] += ok
        t[1] += 1
    for section, (ok, n) in sorted(by.items(), key=lambda kv: kv[1][0] / kv[1][1]):
        print(f"{100 * ok / n:5.1f}%  {ok:4}/{n:<4} {section}")
    total = sum(ok for _, _, ok, _ in results)
    print(f"\nunderstood: {total}/{len(results)} = {100 * total / len(results):.1f}%")
    if report:
        with open(report, "w", encoding="utf-8") as f:
            json.dump([{"topic": t, "query": q, "understood": ok, "answer": a} for t, q, ok, a in results], f, indent=1, ensure_ascii=False)


if __name__ == "__main__":
    main()
