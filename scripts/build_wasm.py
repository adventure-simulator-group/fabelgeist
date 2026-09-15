#!/usr/bin/env python3
"""Build the tactical WebAssembly client and synchronize browser assets."""

from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
STATIC_DIR = ROOT / "crates" / "adventuresim-stdb-module" / "static"
WASM_DIR = STATIC_DIR / "wasm"
ASSET_DIR = STATIC_DIR / "assets"


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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bindgen", default="wasm-bindgen",
                        help="Path to the wasm-bindgen CLI matching Cargo.lock")
    args = parser.parse_args()
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    expected = next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")
    wasm_bindgen = shutil.which(args.bindgen)
    if wasm_bindgen is None:
        print(f"Missing wasm-bindgen. Install wasm-bindgen-cli --version {expected}",
              file=sys.stderr)
        return 1
    try:
        actual = subprocess.check_output([wasm_bindgen, "--version"], text=True).strip().split()[-1]
        if actual != expected:
            print(f"wasm-bindgen CLI {actual} does not match Rust {expected}. "
                  f"Install wasm-bindgen-cli --version {expected}, or pass --bindgen PATH.",
                  file=sys.stderr)
            return 1
        print("Building WASM client...")
        run(["rustup", "target", "add", "wasm32-unknown-unknown"], check=False)
        run(["cargo", "build", "--package", "adventuresim-tactical-client",
             "--bin", "adventuresim-tactical-client", "--bin", "art-demo",
             "--target", "wasm32-unknown-unknown", "--release"])
        WASM_DIR.mkdir(parents=True, exist_ok=True)
        print("Generating JS bindings...")
        for binary in ("adventuresim-tactical-client", "art-demo"):
            run([
                wasm_bindgen, "--out-dir", str(WASM_DIR), "--target", "web", "--no-typescript",
                str(ROOT / "target" / "wasm32-unknown-unknown" / "release" / f"{binary}.wasm"),
            ])
        print("Syncing browser assets...")
        sync_assets()
    except (OSError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f"WASM built to {WASM_DIR}")
    for path in sorted(WASM_DIR.iterdir()):
        print(f"  {path.name}: {path.stat().st_size} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
