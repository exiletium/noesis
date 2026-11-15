#!/usr/bin/env python3
"""Build the bundled word data.

    python3 tools/datagen/words.py WORDS.txt WORDNET_DICT_DIR

WORDS.txt       one lower-case word per line (the `word-list` npm package, MIT,
                which redistributes the atebits "Words" list)
WORDNET_DICT    the dict/ directory of WordNet 3.1 (Princeton WordNet licence;
                the `wordnet-db` npm package carries it)

Writes crates/engine/data/words.z (the word list) and dictionary.z
(definitions and synonyms), both raw-deflate compressed.

dictionary line:  word TAB pos:gloss|pos:gloss… TAB synonym,synonym…
"""
import os
import re
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data")
POS = {"noun": "n", "verb": "v", "adj": "adj", "adv": "adv"}
SENSES_PER_POS = 2


def deflate(data):
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    return c.compress(data) + c.flush()


def main():
    words_path, wn = sys.argv[1], sys.argv[2]
    words = sorted({w.strip() for w in open(words_path) if re.fullmatch(r"[a-z]{2,}", w.strip())})
    with open(os.path.join(OUT, "words.z"), "wb") as f:
        f.write(deflate("\n".join(words).encode()))

    entries = {}
    for name, tag in POS.items():
        synsets = {}
        for line in open(os.path.join(wn, "data." + name), encoding="utf-8"):
            if line.startswith("  "):
                continue
            head, _, gloss = line.partition(" | ")
            parts = head.split()
            n = int(parts[3], 16)
            lemmas = [parts[4 + 2 * k].split("(")[0] for k in range(n)]
            gloss = re.split(r';\s*"', gloss.strip())[0].strip().rstrip(";")
            synsets[parts[0]] = (lemmas, gloss)
        for line in open(os.path.join(wn, "index." + name), encoding="utf-8"):
            if line.startswith("  "):
                continue
            parts = line.split()
            lemma = parts[0]
            if not re.fullmatch(r"[a-z]{2,}", lemma):
                continue
            count, pointers = int(parts[2]), int(parts[3])
            offsets = parts[4 + pointers + 2:][:count]
            glosses, syns = entries.setdefault(lemma, ([], []))
            for k, off in enumerate(offsets):
                lemmas, gloss = synsets[off]
                if k < SENSES_PER_POS:
                    glosses.append(tag + ":" + gloss.replace("|", "/").replace("\t", " "))
                for other in lemmas:
                    other = other.replace("_", " ")
                    if other.lower() != lemma and other not in syns and len(syns) < 12:
                        syns.append(other)
    lines = [f"{w}\t{'|'.join(g)}\t{','.join(s)}" for w, (g, s) in sorted(entries.items())]
    with open(os.path.join(OUT, "dictionary.z"), "wb") as f:
        f.write(deflate("\n".join(lines).encode()))
    print(len(words), "words;", len(lines), "dictionary entries")


if __name__ == "__main__":
    main()
