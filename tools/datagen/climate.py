#!/usr/bin/env python3
"""Build crates/engine/data/climate.z: 1991-2020 monthly mean temperature
normals for every GHCN-Monthly v4 station with enough data (NOAA NCEI,
public domain).

    curl -O https://www.ncei.noaa.gov/pub/data/ghcn/v4/ghcnm.tavg.latest.qcf.tar.gz
    tar xzf ghcnm.tavg.latest.qcf.tar.gz
    python3 tools/datagen/climate.py ghcnm.v4*/ghcnm.tavg.*.qcf.dat

A month's normal needs 20 of the 30 years; a station needs all twelve.
One line per station: name, latitude, longitude, elevation (m), then the
twelve monthly means in °C.
"""
import os
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates", "engine", "data", "climate.z")


def main():
    dat = sys.argv[1]
    inv = dat[:-4] + ".inv"
    stations = {}
    for line in open(inv, encoding="latin-1"):
        stations[line[:11]] = (float(line[12:20]), float(line[21:30]), float(line[31:37]), line[38:68].strip())
    sums = {}
    for line in open(dat, encoding="latin-1"):
        year = int(line[11:15])
        if year < 1991 or year > 2020:
            continue
        sid = line[:11]
        acc = sums.setdefault(sid, [[0.0, 0] for _ in range(12)])
        for m in range(12):
            f = line[19 + 8 * m: 27 + 8 * m]
            v = int(f[:5])
            if v == -9999 or f[6].strip():
                continue
            acc[m][0] += v / 100
            acc[m][1] += 1
    lines = []
    for sid, acc in sums.items():
        if sid not in stations or any(n < 20 for _, n in acc):
            continue
        lat, lon, elev, name = stations[sid]
        vals = [f"{s / n:.1f}" for s, n in acc]
        lines.append("\t".join([name.replace("_", " ").title(), f"{lat:.3f}", f"{lon:.3f}", f"{elev:.0f}"] + vals))
    lines.sort()
    data = "\n".join(lines).encode()
    with open(OUT, "wb") as fh:
        z = zlib.compressobj(9, zlib.DEFLATED, -15)
        fh.write(z.compress(data) + z.flush())
    print(f"{len(lines)} stations; {os.path.getsize(OUT) // 1024} KB compressed")


if __name__ == "__main__":
    main()
