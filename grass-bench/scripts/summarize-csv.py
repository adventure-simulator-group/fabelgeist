#!/usr/bin/env python3
"""Median of a bench CSV's samples after warm-up: `summarize-csv.py run.csv [warmup_secs]`."""
import csv
import statistics
import sys
from pathlib import Path

path = Path(sys.argv[1])
warmup = float(sys.argv[2]) if len(sys.argv) > 2 else 7.0
if not path.exists():
    print('  no csv')
    sys.exit(0)
rows = [r for r in csv.DictReader(path.open()) if float(r['t']) >= warmup]
if not rows:
    print('  no samples after warm-up')
    sys.exit(0)


def med(key):
    return statistics.median(float(r[key]) for r in rows)


print(
    f"  median mean {med('mean_ms'):.2f} ms  p95 {med('p95_ms'):.2f}  "
    f"fps {1000 / med('mean_ms'):.1f}  ({len(rows)} samples)  gpu: {rows[-1].get('gpu_ms', '')}"
)
