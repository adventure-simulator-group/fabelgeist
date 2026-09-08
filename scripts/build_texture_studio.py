#!/usr/bin/env python3
"""Build a self-contained static Texture Studio, independently of the strategic website."""
from pathlib import Path
import argparse
import os
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/adventuresim-texture-studio"

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "target/texture-studio/site")
    parser.add_argument("--bindgen", default="wasm-bindgen")
    parser.add_argument("--dev", action="store_true", help="Use the development profile for local iteration")
    args = parser.parse_args()
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    expected = next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")
    actual = subprocess.check_output([args.bindgen, "--version"], text=True).strip().split()[-1]
    if actual != expected:
        raise SystemExit(f"wasm-bindgen CLI {actual} does not match Rust {expected}. Install wasm-bindgen-cli --version {expected}, then pass --bindgen PATH.")
    command = ["cargo", "build", "--locked", "-p", "adventuresim-texture-studio", "--lib", "--target", "wasm32-unknown-unknown"]
    if not args.dev:
        command.append("--release")
    subprocess.run(command, cwd=ROOT, check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    wasm = target / "wasm32-unknown-unknown" / ("debug" if args.dev else "release") / "adventuresim_texture_studio.wasm"
    args.output.mkdir(parents=True, exist_ok=True)
    subprocess.run([args.bindgen, str(wasm), "--target", "web", "--no-typescript", "--out-dir", str(args.output / "pkg")], check=True)
    for name in ["index.html", "worker.js"]:
        shutil.copy2(CRATE / "web" / name, args.output / name)
    print(f"Static studio: {args.output.resolve()}")
    print("Serve this directory over HTTPS or localhost. No API server or isolation headers are required.")

if __name__ == "__main__":
    main()
