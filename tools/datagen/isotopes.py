#!/usr/bin/env python3
"""Nuclides from the ``mendeleev`` package (MIT): atomic masses, natural
abundances, half-lives, spins and parities, written to
crates/engine/data/isotopes.z (raw deflate).

    pip install mendeleev
    python3 tools/datagen/isotopes.py

Each line: Z<TAB>A<TAB>atomic mass (u)<TAB>abundance (%)<TAB>half-life (s,
"stable" or blank)<TAB>spin<TAB>parity.
"""
import math
import os
import zlib

from mendeleev.fetch import fetch_table

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "isotopes.z")

SECONDS = {
    "ysec": 1e-24, "zsec": 1e-21, "asec": 1e-18, "fsec": 1e-15, "psec": 1e-12, "nsec": 1e-9,
    "usec": 1e-6, "msec": 1e-3, "sec": 1.0, "min": 60.0, "minute": 60.0, "hour": 3600.0, "h": 3600.0,
    "day": 86400.0, "d": 86400.0, "year": 3.15576e7, "y": 3.15576e7, "kyear": 3.15576e10,
    "Myear": 3.15576e13, "Gyear": 3.15576e16, "Tyear": 3.15576e19, "Pyear": 3.15576e22,
    "Eyear": 3.15576e25, "Zyear": 3.15576e28, "Yyear": 3.15576e31,
}


def num(v):
    return "" if v is None or (isinstance(v, float) and math.isnan(v)) else v


def main():
    t = fetch_table("isotopes")
    lines = []
    unknown = set()
    for r in t.sort_values(["atomic_number", "mass_number"]).itertuples():
        hl = ""
        if not r.is_radioactive:
            hl = "stable"
        elif num(r.half_life) != "":
            f = SECONDS.get(str(r.half_life_unit))
            if f is None:
                unknown.add(r.half_life_unit)
            else:
                hl = f"{r.half_life * f:.6g}"
        mass = f"{r.mass:.9f}".rstrip("0") if num(r.mass) != "" else ""
        ab = f"{r.abundance:.6g}" if num(r.abundance) != "" else ""
        lines.append("\t".join(str(x) for x in (r.atomic_number, r.mass_number, mass, ab, hl, num(r.spin), num(r.parity))))
    if unknown:
        print("unknown half-life units:", unknown)
    text = "\n".join(lines) + "\n"
    c = zlib.compressobj(9, zlib.DEFLATED, -15)
    data = c.compress(text.encode()) + c.flush()
    open(OUT, "wb").write(data)
    print(f"{len(lines)} nuclides, {len(data)} bytes")


if __name__ == "__main__":
    main()
