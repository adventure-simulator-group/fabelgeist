#!/usr/bin/env python3
"""Render all three laptop demos under WebGL2 limits; keep screenshots/logs.

Run from any directory. Needs a working desktop display and graphics adapter.
Inspect the resulting PNGs as well: clean logs alone do not prove grass draws.
"""
import json
import os
from pathlib import Path
import re
import subprocess

CRATE = Path(__file__).resolve().parents[1]
ROUTES = ("instanced-cpu-culled", "no-instancing-mesh-chunks", "no-instancing-displacement")


def main():
    subprocess.run(["cargo", "build", "--locked", "--features", "downlevel"],
                   cwd=CRATE, check=True)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=CRATE))
    target = Path(metadata["target_directory"])
    output = target / "downlevel-smoke"
    output.mkdir(parents=True, exist_ok=True)
    binary = target / "debug" / ("gpu-bench.exe" if os.name == "nt" else "gpu-bench")
    for route in ROUTES:
        screenshot = output / f"{route}.png"
        screenshot.unlink(missing_ok=True)
        log = output / f"{route}.log"
        env = {**os.environ, "BENCH_DOWNLEVEL": "1", "BENCH_DEMO": route,
               "BENCH_SETTINGS": "{}", "BEVY_ASSET_ROOT": str(CRATE), "BENCH_EXIT_AFTER": "20",
               "BENCH_SCREENSHOT_AT": "12", "BENCH_SCREENSHOT": str(screenshot)}
        env.pop("BENCH_SWITCH", None)
        with log.open("w") as stream:
            result = subprocess.run([str(binary)], cwd=CRATE, env=env,
                                    stdout=stream, stderr=subprocess.STDOUT, timeout=180)
        errors = re.search(r"Validation Error|render error ignored|panicked|ERROR",
                           "\n".join(line for line in log.read_text().splitlines()
                                     if "XDG Settings Portal" not in line))
        if result.returncode or errors or not screenshot.is_file():
            raise SystemExit(f"FAILED: {route}; inspect {log}")
        print(f"PASS: {route}; inspect {screenshot}", flush=True)


if __name__ == "__main__":
    main()
