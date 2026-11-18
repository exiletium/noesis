#!/usr/bin/env python3
"""Build crates/engine/data/relations.z, broader and narrower terms from WordNet.

    python3 tools/datagen/wordnet_relations.py WORDNET_DICT_DIR

WORDNET_DICT_DIR is the dict/ directory of WordNet 3.1 (Princeton WordNet
licence; the `wordnet-db` npm package carries it).

Output (raw deflate, UTF-8 lines):
    word TAB pos TAB broader;broader… TAB narrower;narrower…
for the two most common senses of each noun and verb (the noun when a word
is both).  Broader terms are hypernyms (including instance-of), narrower
terms are hyponyms (including instances).
"""
import os
import re
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "relations.z")
SENSES = 2
MAX_NARROWER = 30


def main():
    wn = sys.argv[1]
    out = {}
    for name, tag in (("noun", "n"), ("verb", "v")):
        synsets = {}
        for line in open(os.path.join(wn, "data." + name), encoding="utf-8"):
            if line.startswith("  "):
                continue
            head = line.partition(" | ")[0].split()
            n = int(head[3], 16)
            lemmas = [head[4 + 2 * k].split("(")[0].replace("_", " ") for k in range(n)]
            i = 4 + 2 * n
            pcount = int(head[i])
            up, down = [], []
            for k in range(pcount):
                sym, off = head[i + 1 + 4 * k], head[i + 2 + 4 * k]
                if sym in ("@", "@i"):
                    up.append(off)
                elif sym in ("~", "~i"):
                    down.append(off)
            synsets[head[0]] = (lemmas, up, down)
        for line in open(os.path.join(wn, "index." + name), encoding="utf-8"):
            if line.startswith("  "):
                continue
            parts = line.split()
            lemma = parts[0]
            if not re.fullmatch(r"[a-z]{2,}", lemma) or lemma in out:
                continue
            count, pointers = int(parts[2]), int(parts[3])
            offsets = parts[4 + pointers + 2:][:count][:SENSES]
            broader, narrower = [], []
            for off in offsets:
                _, up, down = synsets[off]
                for o in up:
                    w = synsets[o][0][0]
                    if w not in broader:
                        broader.append(w)
                for o in down:
                    for w in synsets[o][0][:1]:
                        if w not in narrower and w != lemma:
                            narrower.append(w)
            if broader or narrower:
                out[lemma] = (tag, broader, narrower[:MAX_NARROWER])
    lines = [f"{w}\t{t}\t{';'.join(b)}\t{';'.join(n)}" for w, (t, b, n) in sorted(out.items())]
    data = "\n".join(lines).encode()
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    with open(OUT, "wb") as f:
        f.write(c.compress(data) + c.flush())
    print(f"{len(lines)} words; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
