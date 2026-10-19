#!/usr/bin/env python3
"""Run a prebuilt native forest component on a coordinated idle machine.

This script never builds and never reports browser-area acceptance. Inspect
the saved readbacks before interpreting a fast sample as useful foliage work.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
from pathlib import Path
import statistics
import subprocess
import time


FIXTURES = (
    {"name": "mixed-context", "scene": "mixed", "years": 120, "camera": []},
    {"name": "boreal-context", "scene": "boreal", "years": 120, "camera": []},
    {"name": "mixed-closeup", "scene": "mixed", "years": 120,
     "camera": ["--closeup", "--yaw", "0.7", "--aim", "0.45"]},
)


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--provenance", type=Path, required=True,
                        help="frozen hardware, driver, backend, power mode and tool metadata")
    parser.add_argument("--repetitions", type=int, default=5)
    parser.add_argument("--frames", type=int, default=600)
    parser.add_argument("--timeout", type=float, default=300)
    parser.add_argument("--fixture", choices=[f["name"] for f in FIXTURES],
                        action="append")
    parser.add_argument("--run", action="store_true",
                        help="run only after the user confirms the quiet window")
    parser.add_argument("--representation", default="replay",
                        choices=("replay", "unique", "shared"))
    parser.add_argument("--prepared-root", type=Path,
                        help="root containing frozen mixed/ and boreal/ asset folders")
    result = parser.parse_args()
    if result.repetitions < 1 or result.frames < 1 or result.timeout <= 0:
        parser.error("repetitions, frames and timeout must be positive")
    if (result.representation == "replay") != (result.prepared_root is None):
        parser.error("prepared representations require --prepared-root; replay must not use it")
    return result


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def digest(path: Path) -> str:
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def quantiles(samples: list[float]) -> dict:
    if not samples:
        return {}
    ordered = sorted(samples)
    return {
        "count": len(samples), "median": statistics.median(samples),
        "p95": ordered[max(0, (95 * len(ordered) + 99) // 100 - 1)],
        "p99": ordered[max(0, (99 * len(ordered) + 99) // 100 - 1)],
        "worst": max(samples),
    }


def main() -> int:
    args = arguments()
    executable = args.executable.resolve()
    if not executable.is_file():
        raise SystemExit(f"prebuilt executable does not exist: {executable}")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    repo = Path(__file__).resolve().parents[1]
    revision = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    dirty = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=repo, text=True)
    fixtures = [f for f in FIXTURES if not args.fixture or f["name"] in args.fixture]
    assets = {}
    if args.prepared_root:
        names = ["unique.forest"] if args.representation == "unique" else [
            "library.forest", "population.forest"]
        for scene in sorted({fixture["scene"] for fixture in fixtures}):
            for name in names:
                path = args.prepared_root.resolve() / scene / name
                assets[str(path)] = {"bytes": path.stat().st_size, "sha256": digest(path)}
    provenance = json.loads(args.provenance.read_text(encoding="utf-8"))
    for field in ("hardware", "software", "tool_versions", "render_settings"):
        if not provenance.get(field):
            raise SystemExit(f"provenance is missing {field}")
    plan = {
        "schema_version": 1,
        "scope": "native_offscreen_component_diagnostic",
        "revision": revision, "dirty_tree": dirty.splitlines(),
        "executable_sha256": digest(executable),
        "repetitions": args.repetitions, "frames": args.frames,
        "resolution": [1600, 1000], "seed": 1, "profile": "fast",
        "fixtures": fixtures,
        "representation": args.representation, "required_prepared_assets": assets,
        "provenance": provenance,
        "build_profile": "release (must match the prebuilt executable)",
        "conditions": "fresh process; existing driver and filesystem caches retained",
        "area_acceptance": "unassessed",
        "limitations": [
            "built-in forest patches are not full landscape vistas",
            "automatic framing is fixed for each deterministic fixture, not between fixtures",
            "no browser, network transfer, production travel or cross-area library residency",
            "no claim of cold driver cache, complete GPU allocation counts or peak memory",
            "readback coverage needs independent visual review",
        ],
    }
    write_json(output / "contract.json", plan)
    if not args.run:
        print(f"Prepared contract only: {output}")
        return 0

    results = []
    for fixture in fixtures:
        for repetition in range(args.repetitions):
            run_dir = output / f"{fixture['name']}-{repetition + 1:02d}"
            run_dir.mkdir()
            image = run_dir / "forest.png"
            report = run_dir / "measurement.json"
            command = [str(executable), "--snapshot", str(image),
                       "--frames", str(args.frames),
                       "--benchmark-report", str(report), *fixture["camera"]]
            if args.representation == "replay":
                command += ["--scene", fixture["scene"], "--years", str(fixture["years"]),
                            "--profile", "fast"]
            else:
                command += ["--prepared-forest", str(args.prepared_root.resolve() / fixture["scene"]),
                            "--representation", args.representation]
                if any(digest(Path(path)) != data["sha256"] for path, data in assets.items()):
                    raise SystemExit("prepared input changed after the contract was frozen")
            write_json(run_dir / "command.json", command)
            started = time.perf_counter()
            result = {"fixture": fixture["name"], "repetition": repetition + 1,
                      "status": "invalid", "visual_review": "unassessed"}
            try:
                with (run_dir / "process.log").open("w", encoding="utf-8") as log:
                    completed = subprocess.run(
                        command, cwd=repo, stdout=log, stderr=subprocess.STDOUT,
                        timeout=args.timeout, check=False)
                result["exit_code"] = completed.returncode
                renderer_errors = [line for line in (run_dir / "process.log").read_text(
                    encoding="utf-8").splitlines() if re.search(r"\bERROR\b", line)]
                if renderer_errors:
                    result["reason"] = "renderer emitted errors; inspect process.log"
                    result["renderer_error_lines"] = len(renderer_errors)
                elif (completed.returncode == 0 and report.is_file() and image.is_file()
                        and image.with_suffix(".readback-01.png").is_file()):
                    data = json.loads(report.read_text(encoding="utf-8"))
                    counts = data["counts"]
                    previous = image.with_suffix(
                        f".readback-{data['readback_attempts'] - 1:02d}.png")
                    validation = data["probe_validation"]
                    arrival_image = report.with_suffix(".arrival.png")
                    observed_times = [
                        data["elapsed_from_request_ms"]["first_completed_forest_draw_commands"],
                        data["arrival_frame"]["elapsed_from_request_ms"],
                        data["elapsed_from_request_ms"]["first_readback"],
                        data["elapsed_from_request_ms"]["stable_readback"],
                    ]
                    valid = (data["schema_version"] == 7
                             and data["arrival_frame"]["image"] == str(arrival_image)
                             and arrival_image.is_file()
                             and all(isinstance(value, (int, float))
                                     and math.isfinite(value) and value > 0
                                     for value in observed_times)
                             and observed_times == sorted(observed_times)
                             and data["arrival_frame"]["resolution"] == data["resolution"]
                             and digest(arrival_image) == digest(image)
                             and validation["policy"] == "every_frame"
                             and validation["phase"] == "complete"
                             and validation["successful_full_checks"] >= 2
                             and validation["skipped_structural_checks"] == 0
                             and validation["sampled_render_checks"] >= 1
                             and previous.is_file()
                             and digest(previous) == digest(image)
                             and counts["branches"] > 0 and counts["leaf_clusters"] > 0
                             and counts["queued_writes"] == counts["submitted_writes"]
                             and counts["queued_writes"] > 0
                             and counts["discarded_writes"] == 0
                             and counts["superseded_writes"] == 0
                             and counts["gpu_failures"] == 0
                             and data["elapsed_from_request_ms"][
                                 "first_completed_forest_draw_commands"] is not None)
                    if valid:
                        result["status"] = "component_measured_visual_review_pending"
                        result["measurement"] = data
                        result["image_sha256"] = digest(image)
                        result["stable_pair"] = [str(previous), str(image)]
                        result["stable_pair_sha256"] = [digest(previous), digest(image)]
                        result["arrival_image_sha256"] = digest(arrival_image)
                else:
                    result["reason"] = "process failed or required evidence is missing"
            except subprocess.TimeoutExpired:
                result["reason"] = "bounded process timeout"
            result["process_elapsed_ms"] = (time.perf_counter() - started) * 1000
            write_json(run_dir / "result.json", result)
            results.append(result)
            print(f"{fixture['name']} {repetition + 1}: {result['status']}", flush=True)
            if result["status"] == "invalid":
                write_json(output / "results.json", results)
                print(f"Stopped at the first invalid run; inspect {run_dir}")
                return 1

    summaries = {}
    for fixture in fixtures:
        measured = [r["measurement"] for r in results if r["fixture"] == fixture["name"]]
        summaries[fixture["name"]] = {
            "completed_draw_commands_ms": quantiles([
                m["elapsed_from_request_ms"]["first_completed_forest_draw_commands"]
                for m in measured]),
            "first_gpu_ready_readback_ms": quantiles([
                m["arrival_frame"]["elapsed_from_request_ms"] for m in measured]),
            "frame_ms": quantiles([
                sample for m in measured for sample in m["cpu_gpu_waited_frame_ms"]]),
            "area_acceptance": "unassessed",
        }
    write_json(output / "results.json", results)
    write_json(output / "summary.json", summaries)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
