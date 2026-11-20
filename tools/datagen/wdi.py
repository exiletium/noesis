#!/usr/bin/env python3
"""Build crates/engine/data/wdi.z: selected World Development Indicators
(World Bank, CC BY 4.0) for every country and regional aggregate.

    python3 tools/datagen/wdi.py

A line "#CODE<TAB>name" starts each indicator; then one line per economy:
ISO3 code, economy name, and year=value pairs.
"""
import json
import os
import subprocess
import time
import re
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "wdi.z")

CODES = [
    "FR.INR.LEND", "NE.GDI.STKB.CN", "NE.GDI.STKB.CD", "NY.GNS.ICTR.GN.ZS", "NY.GDS.TOTL.ZS", "IC.FRM.CORR.ZS",
    "IC.BUS.NREG", "IC.BUS.NDNS.ZS", "NE.DAB.TOTL.CD", "NY.ADJ.NNTY.CD", "NE.CON.PRVT.CD", "NE.GDI.TOTL.CD", "SP.POP.TOTL",
    "SH.IMM.MEAS", "SE.ADT.LITR.ZS", "SE.PRM.CMPT.ZS", "SE.SEC.TCHR.FE.ZS", "SL.TLF.PART.ZS", "SL.EMP.SELF.ZS", "SL.AGR.EMPL.ZS",
    "SL.IND.EMPL.ZS", "SL.SRV.EMPL.ZS", "SL.TLF.TOTL.IN", "SL.FAM.WORK.ZS", "GB.XPD.RSDV.GD.ZS", "ST.INT.ARVL",
    "AG.PRD.LVSK.XD", "IQ.CPA.GNDR.XQ", "SE.PRM.REPT.ZS", "SP.DYN.LE00.IN", "SH.DYN.AIDS.ZS", "SM.POP.TOTL",
    "SL.EMP.TOTL.SP.ZS", "SH.XPD.CHEX.GD.ZS", "AG.LND.ARBL.HA", "IT.NET.USER.ZS",
    "SP.POP.0014.TO", "NE.CON.GOVT.CD", "NY.GNP.PCAP.CD", "AG.PRD.FOOD.XD", "SP.DYN.LE00.FE.IN", "SP.DYN.LE00.MA.IN", "SE.SEC.TCHR",
    "SE.PRM.TCHR", "SE.PRM.ENRL", "SL.UEM.TOTL.ZS", "NE.DAB.DEFL.ZS", "NY.GDP.MKTP.CD",
]

SKIP = re.compile(r"IDA|IBRD|dividend|Fragil|small states|HIPC|Least developed|excluding|Not classified|middle income|Low income|Middle income|Africa Eastern|Africa Western|Pacific island|Other small|Euro area|OECD members|Central Europe")


def get(url):
    for _ in range(5):
        r = subprocess.run(["curl", "-sS", "--retry", "3", url], capture_output=True)
        if r.returncode == 0 and r.stdout.startswith(b"["):
            return json.loads(r.stdout)
        time.sleep(3)
    raise RuntimeError(url)


def main():
    out = []
    for code in CODES:
        page = get(f"https://api.worldbank.org/v2/country/all/indicator/{code}?format=json&per_page=20000")
        if len(page) < 2 or not page[1]:
            print("no data", code)
            continue
        name = page[1][0]["indicator"]["value"]
        by = {}
        for r in page[1]:
            if r["value"] is None or not r["countryiso3code"] or int(r["date"]) < 1980 or SKIP.search(r["country"]["value"]):
                continue
            by.setdefault((r["countryiso3code"], r["country"]["value"]), []).append((r["date"], r["value"]))
        out.append(f"#{code}\t{name}")
        for (iso, cname), vals in sorted(by.items()):
            vals.sort()
            out.append(f"{iso}\t{cname}\t" + " ".join(f"{d}={float(v):.4g}" for d, v in vals))
        print(code, name, len(by))
    data = "\n".join(out).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
