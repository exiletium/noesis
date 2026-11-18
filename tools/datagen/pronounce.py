#!/usr/bin/env python3
"""Build crates/engine/data/cmudict.z, English pronunciations.

    python3 tools/datagen/pronounce.py CMUDICT

CMUDICT is cmudict.dict from https://github.com/cmusphinx/cmudict
(BSD-style licence, Carnegie Mellon University; see data/LICENSE-cmudict.txt).

Output (raw deflate, ASCII lines):  word TAB ARPAbet|ARPAbet…
Variant pronunciations ("word(2)") are joined after the first.
"""
import os
import re
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "cmudict.z")


def main():
    prons = {}
    for line in open(sys.argv[1], encoding="utf-8"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        head, *phones = line.split()
        word = re.sub(r"\(\d+\)$", "", head)
        if not re.fullmatch(r"[a-z][a-z'.-]*", word):
            continue
        prons.setdefault(word, []).append(" ".join(phones))
    lines = [f"{w}\t{'|'.join(p)}" for w, p in sorted(prons.items())]
    data = "\n".join(lines).encode()
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    with open(OUT, "wb") as f:
        f.write(c.compress(data) + c.flush())
    print(f"{len(lines)} words; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
