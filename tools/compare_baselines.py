#!/usr/bin/env python3
"""The CI perf gate's comparison (the §4 contract's committed baseline).

Parses the divan output of `cargo bench -p aeolus-gpui --bench
perf_contract`, extracts the pinned arms' median nanoseconds, and
fails (exit 1) when the static walk regresses more than 5% or the
cell-warm path exceeds 2x its baseline — the additive gate, never
touching the other CI jobs.

Usage: python3 tools/compare_baselines.py <divan-output-file>
"""
import json
import re
import sys
from pathlib import Path

BASELINES = Path(__file__).parent.parent / "crates/aeolus-gpui/benches/baselines.json"


def parse_divan(text):
    """Extract (bench, ns) pairs from divan's default output lines."""
    out = {}
    for line in text.splitlines():
        # divan formats: `name  <value><unit>  ...` — the time reg in ns.
        m = re.match(r"\s*([\w/]+)\s+([\d.]+)\s*(ns|µs|ms)\b", line)
        if not m:
            continue
        name, value, unit = m.group(1), float(m.group(2)), m.group(3)
        factor = {"ns": 1.0, "µs": 1e3, "ms": 1e6}[unit]
        out[name] = value * factor
    return out


def main() -> int:
    raw = Path(sys.argv[1]).read_text() if len(sys.argv) > 1 else sys.stdin.read()
    measured = parse_divan(raw)
    baselines = json.loads(BASELINES.read_text())
    failures = []
    for arm in ("static_walk_button", "hand_button_chain", "static_clone_button", "cell_warm", "cell_cold"):
        base = baselines[arm]
        got = measured.get(arm)
        if got is None:
            failures.append(f"{arm}: no measurement found in the bench output")
            continue
        ratio = got / base
        line = f"{arm}: {got:.0f}ns vs baseline {base}ns (x{ratio:.2f})"
        print(line)
        if arm == "static_walk_button" and ratio > 1.05:
            failures.append(line + " — the walk regression exceeds the 5% gate")
        if arm == "cell_warm" and ratio > 2.0:
            failures.append(line + " — the cell-warm path exceeds 2x the baseline")
    if failures:
        print("PERF GATE FAILED:")
        print("\n".join(failures))
        return 1
    print("perf gate passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
