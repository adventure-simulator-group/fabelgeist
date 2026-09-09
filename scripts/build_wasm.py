#!/usr/bin/env python3
"""Build the tactical WebAssembly client and synchronize browser assets."""

from __future__ import annotations

from pathlib import Path
import argparse
import json
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
STATIC_DIR = ROOT / "crates" / "adventuresim-stdb-module" / "static"
WASM_DIR = STATIC_DIR / "wasm"
ASSET_DIR = STATIC_DIR / "assets"


def run(command: list[str], *, check: bool = True) -> int:
    result = subprocess.run(command, cwd=ROOT)
    if check and result.returncode:
        raise subprocess.CalledProcessError(result.returncode, command)
    return result.returncode


def target_dir() -> Path:
    """Resolve cargo's target directory.

    Do not assume ``ROOT/target``: a global cargo config or ``CARGO_TARGET_DIR``
    can redirect it elsewhere (this environment points it at a shared cache), in
    which case the built .wasm is not under the repo at all.
    """
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return Path(json.loads(out.stdout)["target_directory"])


def sync_assets() -> None:
    if ASSET_DIR.exists():
        shutil.rmtree(ASSET_DIR)
    shutil.copytree(ROOT / "assets", ASSET_DIR)
    for source in (ROOT / "crates").glob("*/assets"):
        if source.is_dir():
            shutil.copytree(source, ASSET_DIR, dirs_exist_ok=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--keep-name-section",
        action="store_true",
        help=(
            "keep wasm function names, for readable panic traces at the cost "
            "of roughly doubling the module"
        ),
    )
    args = parser.parse_args(argv)
    wasm_bindgen = shutil.which("wasm-bindgen")
    if wasm_bindgen is None:
        print("Missing wasm-bindgen. Install with: cargo install wasm-bindgen-cli", file=sys.stderr)
        return 1
    try:
        print("Building WASM client...")
        # The Nix toolchain already provides the wasm32 target (see
        # rust-toolchain.toml). Only nudge rustup when it's actually installed;
        # otherwise `subprocess.run` raises FileNotFoundError and the build dies
        # on a target that's already present.
        if shutil.which("rustup"):
            run(["rustup", "target", "add", "wasm32-unknown-unknown"], check=False)
        run(["cargo", "build", "--package", "adventuresim-tactical-client", "--target", "wasm32-unknown-unknown", "--release"])
        wasm = target_dir() / "wasm32-unknown-unknown" / "release" / "adventuresim-tactical-client.wasm"
        WASM_DIR.mkdir(parents=True, exist_ok=True)
        print("Generating JS bindings...")
        bindgen = [
            wasm_bindgen, "--out-dir", str(WASM_DIR), "--target", "web", "--no-typescript",
        ]
        if not args.keep_name_section:
            # The name section is about half the module and the browser pays
            # for it twice: in the download and again when devtools indexes
            # the module as a source, which is what makes Firefox report the
            # page as slow. Pass --keep-name-section when a panic trace needs
            # readable frames.
            bindgen.extend(["--remove-name-section", "--remove-producers-section"])
        run([*bindgen, str(wasm)])
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
