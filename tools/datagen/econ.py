#!/usr/bin/env python3
"""Build crates/engine/data/econ.z: a snapshot of US economic time series
published by federal agencies (BEA, Census Bureau, Federal Reserve Board,
Treasury, BLS; US government works, public domain), fetched through FRED.

    python3 tools/datagen/econ.py

One line per series: FRED id, title, units, source agency, then
date=value pairs separated by spaces.  Daily series keep their last 30
observations.
"""
import csv
import io
import os
import re
import subprocess
import time
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "econ.z")

SERIES = [
    ("GDPC1", "BEA"), ("GDP", "BEA"), ("A191RL1Q225SBEA", "BEA"), ("A191RL1A225NBEA", "BEA"),
    ("RSAFS", "Census Bureau"), ("TOTALSL", "Federal Reserve Board"), ("IPMINE", "Federal Reserve Board"), ("INDPRO", "Federal Reserve Board"),
    ("GFDEBTN", "Treasury"), ("GFDEGDQ188S", "OMB and Federal Reserve Bank of St. Louis"), ("M1SL", "Federal Reserve Board"), ("M2SL", "Federal Reserve Board"),
    ("WRESCRT", "Federal Reserve Board"), ("IIPUSASSQ", "BEA"), ("IIPUSLIAQ", "BEA"), ("IEAXI", "BEA"), ("UNRATE", "BLS"), ("CPIAUCSL", "BLS"),
    ("HOUST", "Census Bureau"), ("PERMIT", "Census Bureau"), ("PAYEMS", "BLS"), ("FEDFUNDS", "Federal Reserve Board"),
    ("DGS1MO", "Federal Reserve Board"), ("DGS3MO", "Federal Reserve Board"), ("DGS6MO", "Federal Reserve Board"), ("DGS1", "Federal Reserve Board"),
    ("DGS2", "Federal Reserve Board"), ("DGS3", "Federal Reserve Board"), ("DGS5", "Federal Reserve Board"), ("DGS7", "Federal Reserve Board"),
    ("DGS10", "Federal Reserve Board"), ("DGS20", "Federal Reserve Board"), ("DGS30", "Federal Reserve Board"), ("GS30", "Federal Reserve Board"), ("GS10", "Federal Reserve Board"),
    ("FEDMINNFRWG", "Department of Labor"), ("EXPGS", "BEA"), ("IMPGS", "BEA"), ("PCE", "BEA"), ("PSAVERT", "BEA"),
]
STATES = "AL AK AZ AR CA CO CT DE DC FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY".split()
SERIES += [(f"{s}RGSP", "BEA") for s in STATES]


def get(url):
    for _ in range(4):
        r = subprocess.run(["curl", "-sS", "--retry", "3", url], capture_output=True)
        if r.returncode == 0 and r.stdout:
            return r.stdout.decode("utf-8", "replace")
        time.sleep(3)
    raise RuntimeError(url)


def main():
    lines = []
    for sid, src in SERIES:
        page = get(f"https://fred.stlouisfed.org/series/{sid}")
        title = re.search(r"<title>(.*?) \(" + re.escape(sid) + r"\)", page).group(1).replace("&amp;", "&").replace("&#039;", "'")
        u = re.search(r'series-meta-value-units">\s*([^<]+?)\s*</span>,(?:</div><div class="default-text">|&nbsp;)\s*([^<]+?)\s*</span>', page)
        units = f"{u.group(1)}, {u.group(2)}" if u else ""
        rows = [r for r in csv.reader(io.StringIO(get(f"https://fred.stlouisfed.org/graph/fredgraph.csv?id={sid}")))][1:]
        rows = [(d, v) for d, v in rows if v not in ("", ".")]
        if sid.startswith("DGS"):
            rows = rows[-30:]
        pairs = " ".join(f"{d}={v}" for d, v in rows)
        lines.append("\t".join([sid, title, units, src, pairs]))
        print(sid, title, "|", units, "|", len(rows))
    data = "\n".join(lines).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{len(lines)} series; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
