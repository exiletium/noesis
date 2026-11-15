#!/usr/bin/env python3
"""Collect the sample queries from WolframAlpha's public "Examples by Topic"
pages, as a benchmark set that Noesis's authors did not write.

    python3 tools/benchmark/collect_examples.py OUT.txt [--max-pages N]

Crawls https://www.wolframalpha.com/examples/ (one request a second) and
writes one query per line, grouped under "# topic: <path>" comments.
"""
import re
import sys
import time
import urllib.parse
import urllib.request

BASE = "https://www.wolframalpha.com"
UA = "Mozilla/5.0 (X11; Linux x86_64) noesis-benchmark/1.0"


def get(path):
    req = urllib.request.Request(BASE + path, headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=40) as r:
            return r.read().decode("utf-8", "replace")
    except Exception as err:  # noqa: BLE001
        sys.stderr.write(f"  {path}: {err}\n")
        return ""
    finally:
        time.sleep(1.0)


def main():
    out = sys.argv[1]
    limit = int(sys.argv[sys.argv.index("--max-pages") + 1]) if "--max-pages" in sys.argv else 400
    queue, seen, topics = ["/examples"], set(), {}
    while queue and len(seen) < limit:
        path = queue.pop(0)
        if path in seen:
            continue
        seen.add(path)
        html = get(path)
        for link in re.findall(r'href="(/examples/[a-z0-9\-/]+)"', html):
            link = link.rstrip("/")
            if link not in seen and "pro-features" not in link and link.count("/") <= 4:
                queue.append(link)
        found = [urllib.parse.unquote_plus(q).strip() for q in re.findall(r'/input\?i=([^"&]+)', html)]
        found = [q for q in found if 2 <= len(q) <= 160 and "\n" not in q]
        if found:
            topics[path] = list(dict.fromkeys(found))
        sys.stderr.write(f"{len(seen):4} {path}: {len(found)}\n")
    seen_q = set()
    with open(out, "w", encoding="utf-8") as f:
        for path, qs in sorted(topics.items()):
            qs = [q for q in qs if q.lower() not in seen_q]
            if not qs:
                continue
            f.write(f"# topic: {path}\n")
            for q in qs:
                seen_q.add(q.lower())
                f.write(q + "\n")
    print(f"{len(seen_q)} queries from {len(topics)} pages")


if __name__ == "__main__":
    main()
