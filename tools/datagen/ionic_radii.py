#!/usr/bin/env python3
"""Build crates/engine/data/ionic_radii.txt: Shannon (1976) effective ionic
and crystal radii from the `mendeleev` package (MIT licence).

    pip install mendeleev && python3 tools/datagen/ionic_radii.py

One line per entry: symbol, charge, coordination, spin, ionic radius (pm),
crystal radius (pm), 1 if Shannon marked it most reliable.
"""
import os

from mendeleev import element

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "ionic_radii.txt")


def main():
    lines = []
    for z in range(1, 104):
        e = element(z)
        for r in e.ionic_radii:
            if r.ionic_radius is None:
                continue
            lines.append(f"{e.symbol}\t{r.charge}\t{r.coordination}\t{r.spin or ''}\t{round(r.ionic_radius, 1):g}\t{round(r.crystal_radius, 1):g}\t{1 if r.most_reliable else 0}")
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write("# Shannon effective ionic radii via mendeleev: symbol charge coordination spin ionic_pm crystal_pm most_reliable\n")
        fh.write("\n".join(lines) + "\n")
    print(f"{len(lines)} radii")


if __name__ == "__main__":
    main()
