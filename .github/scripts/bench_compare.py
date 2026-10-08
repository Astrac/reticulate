#!/usr/bin/env python3
"""Compare two criterion baselines and fail if any benchmark regressed.

Usage: bench_compare.py <base-baseline> <head-baseline> <threshold-percent>

Reads target/criterion/**/<baseline>/estimates.json and compares mean point
estimates. Prints a markdown table.
"""
import json
import sys
from pathlib import Path

base, head, threshold = sys.argv[1], sys.argv[2], float(sys.argv[3])
root = Path("target/criterion")


def load(name):
    out = {}
    for p in root.rglob(f"{name}/estimates.json"):
        bench = p.parent.parent.relative_to(root).as_posix()
        out[bench] = json.loads(p.read_text())["mean"]["point_estimate"]
    return out


b, h = load(base), load(head)
if not b or not h:
    sys.exit("No criterion results found for one of the baselines")

fmt = lambda ns: f"{ns / 1e3:.2f} µs" if ns < 1e6 else f"{ns / 1e6:.2f} ms"
print(f"### Benchmark comparison (threshold {threshold:g}%)\n")
print("| Benchmark | Base | Head | Change | |")
print("|---|---|---|---|---|")
regressions = []
for name in sorted(b.keys() | h.keys()):
    if name not in b or name not in h:
        print(f"| `{name}` | {fmt(b[name]) if name in b else '-'} | "
              f"{fmt(h[name]) if name in h else '-'} | n/a | "
              f"{'new' if name in h else 'removed'} |")
        continue
    change = (h[name] - b[name]) / b[name] * 100
    bad = change > threshold
    if bad:
        regressions.append((name, change))
    mark = "❌ regression" if bad else ("✅ faster" if change < -threshold else "")
    print(f"| `{name}` | {fmt(b[name])} | {fmt(h[name])} | {change:+.1f}% | {mark} |")

if regressions:
    print(f"\n**{len(regressions)} benchmark(s) regressed by more than {threshold:g}%.**")
    sys.exit(1)
