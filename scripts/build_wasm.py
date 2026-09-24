#!/usr/bin/env python3
"""Build the tactical WebAssembly client and synchronize browser assets."""

from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib


ROOT = Path(__file__).resolve().parents[1]
STATIC_DIR = ROOT / "crates" / "adventuresim-stdb-module" / "static"
WASM_DIR = STATIC_DIR / "wasm"
ASSET_DIR = STATIC_DIR / "assets"
WASM_OPT_VERSION = "123"


def run(command: list[str], *, check: bool = True) -> int:
    result = subprocess.run(command, cwd=ROOT)
    if check and result.returncode:
        raise subprocess.CalledProcessError(result.returncode, command)
    return result.returncode


def sync_assets() -> None:
    if ASSET_DIR.exists():
        shutil.rmtree(ASSET_DIR)
    shutil.copytree(ROOT / "assets", ASSET_DIR)
    for source in (ROOT / "crates").glob("*/assets"):
        if source.is_dir():
            shutil.copytree(source, ASSET_DIR, dirs_exist_ok=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bindgen", default="wasm-bindgen",
                        help="Path to the wasm-bindgen CLI matching Cargo.lock")
    parser.add_argument("--wasm-opt", default=str(ROOT / "node_modules" / ".bin" / "wasm-opt"),
                        help=f"Path to wasm-opt version {WASM_OPT_VERSION}")
    parser.add_argument("--keep-name-section", action="store_true",
                        help="Keep wasm function names for readable panic traces")
    args = parser.parse_args(argv)
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    expected = next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")
    wasm_bindgen = shutil.which(args.bindgen)
    if wasm_bindgen is None:
        print(f"Missing wasm-bindgen. Install wasm-bindgen-cli --version {expected}",
              file=sys.stderr)
        return 1
    wasm_opt = shutil.which(args.wasm_opt)
    if wasm_opt is None:
        print("Missing pinned wasm-opt. Run npm ci.", file=sys.stderr)
        return 1
    try:
        actual = subprocess.check_output([wasm_bindgen, "--version"], text=True).strip().split()[-1]
        if actual != expected:
            print(f"wasm-bindgen CLI {actual} does not match Rust {expected}. "
                  f"Install wasm-bindgen-cli --version {expected}, or pass --bindgen PATH.",
                  file=sys.stderr)
            return 1
        wasm_opt_version = subprocess.check_output(
            [wasm_opt, "--version"], text=True
        ).strip().split()[2]
        if wasm_opt_version != WASM_OPT_VERSION:
            print(f"wasm-opt {wasm_opt_version} does not match pinned "
                  f"version {WASM_OPT_VERSION}. Run npm ci.", file=sys.stderr)
            return 1
        print("Building WASM client...")
        if shutil.which("rustup"):
            run(["rustup", "target", "add", "wasm32-unknown-unknown"], check=False)
        started = time.perf_counter()
        common = ["cargo", "build", "--package", "adventuresim-tactical-client",
                  "--target", "wasm32-unknown-unknown"]
        # Preserve the ordinary tactical client's release build exactly. Build
        # the demo separately so Cargo cannot unify gameplay-only features into
        # its size-oriented binary.
        run([*common, "--bin", "adventuresim-tactical-client", "--release"])
        run([*common, "--bin", "art-demo", "--profile", "wasm-release",
             "--no-default-features"])
        build_seconds = time.perf_counter() - started
        WASM_DIR.mkdir(parents=True, exist_ok=True)
        print("Generating JS bindings...")
        for binary in ("adventuresim-tactical-client", "art-demo"):
            profile = "wasm-release" if binary == "art-demo" else "release"
            source = (ROOT / "target" / "wasm32-unknown-unknown" /
                      profile / f"{binary}.wasm")
            bindgen = [
                wasm_bindgen, "--out-dir", str(WASM_DIR), "--target", "web", "--no-typescript",
            ]
            if not args.keep_name_section:
                bindgen.extend(["--remove-name-section", "--remove-producers-section"])
            run([*bindgen, str(source)])
            if binary == "art-demo":
                generated = WASM_DIR / "art-demo_bg.wasm"
                optimized = generated.with_stem("art-demo_bg-optimized")
                run([wasm_opt, "--all-features", "-Oz", "--strip-debug",
                     "--strip-producers",
                     str(generated), "-o", str(optimized)])
                optimized.replace(generated)
        print("Syncing browser assets...")
        sync_assets()
    except (OSError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f"WASM built to {WASM_DIR}")
    for path in sorted(WASM_DIR.iterdir()):
        print(f"  {path.name}: {path.stat().st_size} bytes")
    sizes = {"build_seconds": round(build_seconds, 3), "bundles": {}}
    for path in sorted(WASM_DIR.glob("*_bg.wasm")):
        data = path.read_bytes()
        brotli_data = subprocess.check_output([
            "node", "-e",
            "const fs=require('fs'),z=require('zlib');process.stdout.write("
            "z.brotliCompressSync(fs.readFileSync(process.argv[1]),{params:{"
            "[z.constants.BROTLI_PARAM_QUALITY]:11}}))",
            str(path),
        ])
        sizes["bundles"][path.name] = {
            "raw_bytes": len(data),
            "gzip_bytes": len(gzip.compress(data, compresslevel=6, mtime=0)),
            "brotli_bytes": len(brotli_data),
        }
    report = WASM_DIR / "bundle-sizes.json"
    report.write_text(json.dumps(sizes, indent=2) + "\n", encoding="utf-8")
    print(f"  {report.name}: raw/gzip-6/brotli-11 measurements")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
