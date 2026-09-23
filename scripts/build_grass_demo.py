#!/usr/bin/env python3
"""Build the grass bench (grass-bench/) as a static bundle for the showcase.

    python scripts/build_grass_demo.py --output target/showcase/site/grass

A Python port of `grass-bench/scripts/build-wasm-client.sh`'s release mode,
with one difference that matters here: the dev server strips the content-hash
URL prefix the wasm asks assets for, and a static host does not, so the assets
are written *under* that prefix. The output is therefore plain files any file
server can serve -- no rewrite rule, which is what lets this ride along in the
showcase tree with nothing added to the Caddyfile but `precompressed`.

Two engines are built: WebGPU, and a WebGL2 fallback the page picks when
`navigator.gpu` is missing. The bench pins its own wasm-bindgen (its lockfile
is its own, outside this workspace), so this resolves the CLI itself instead
of taking the showcase's.
"""
from __future__ import annotations

import argparse
import gzip
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "grass-bench"
TARGET = "wasm32-unknown-unknown"
# Only what the bench loads; `find`-equivalent roots relative to its assets/.
SHIPPED = ["textures/ground", "textures/oak", "models/oak.glb", "models/fabelgeist.glb"]
SHIPPED_GLOBS = [("models", "armor-*.png")]
# simd128: glam gates its SIMD backend on it, and it is both faster and
# smaller. Not +relaxed-simd (non-deterministic FMA contraction).
RUSTFLAGS = "-C target-feature=+simd128"


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def run(command: list[str], env: dict[str, str] | None = None) -> None:
    result = subprocess.run(command, cwd=CRATE, env=env)
    if result.returncode:
        raise SystemExit(f"command failed ({result.returncode}): {' '.join(command)}")


def locked_bindgen_version() -> str:
    lock = tomllib.loads((CRATE / "Cargo.lock").read_text(encoding="utf-8"))
    return next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")


def resolve_bindgen(preferred: str | None, target_dir: Path) -> str:
    """A wasm-bindgen CLI matching grass-bench's lockfile, installed if absent."""
    expected = locked_bindgen_version()

    def version_of(candidate: str) -> str | None:
        resolved = shutil.which(candidate) or (candidate if Path(candidate).is_file() else None)
        if resolved is None:
            return None
        try:
            return subprocess.check_output([resolved, "--version"], text=True).strip().split()[-1]
        except (OSError, subprocess.CalledProcessError):
            return None

    local = target_dir / "wasm-bindgen-cli" / expected / "bin" / "wasm-bindgen"
    for candidate in (preferred, "wasm-bindgen", str(local)):
        if candidate and version_of(candidate) == expected:
            return shutil.which(candidate) or candidate
    log(f"Installing wasm-bindgen-cli {expected} for the grass bench...")
    run([
        "cargo", "install", "--root", str(target_dir / "wasm-bindgen-cli" / expected),
        "--version", expected, "--locked", "wasm-bindgen-cli",
    ])
    return str(local)


def target_dir() -> Path:
    override = os.environ.get("CARGO_TARGET_DIR")
    return Path(override) if override else CRATE / "target"


def shipped_assets() -> list[Path]:
    assets = CRATE / "assets"
    files: list[Path] = []
    for entry in SHIPPED:
        path = assets / entry
        if path.is_dir():
            files.extend(p for p in path.rglob("*") if p.is_file())
        elif path.is_file():
            files.append(path)
    for folder, pattern in SHIPPED_GLOBS:
        files.extend(sorted((assets / folder).glob(pattern)))
    return sorted(files)


def asset_prefix(files: list[Path]) -> str:
    """Content hash of the shipped assets, the URL prefix baked into the wasm.

    A changed asset is a changed URL, so a browser cache can never pair a new
    build with a stale glb.
    """
    digest = hashlib.md5()
    for path in files:
        file_hash = hashlib.md5(path.read_bytes()).hexdigest()
        digest.update(f"{file_hash}  {path.relative_to(CRATE)}\n".encode())
    return "v" + digest.hexdigest()[:8]


def build_engine(profile: str, features: list[str], prefix: str) -> Path:
    env = {**os.environ, "WEB_ASSET_PREFIX": prefix}
    env["RUSTFLAGS"] = f"{env.get('RUSTFLAGS', '')} {RUSTFLAGS}".strip()
    command = ["cargo", "build", "--locked", "--profile", profile, "--target", TARGET]
    if features:
        command += ["--features", ",".join(features)]
    run(command, env)
    return target_dir() / TARGET / profile / "gpu-bench.wasm"


def compress(path: Path) -> None:
    """Write .gz (and .br where the brotli CLI exists) next to a file."""
    path.with_suffix(path.suffix + ".gz").write_bytes(
        gzip.compress(path.read_bytes(), 9, mtime=0)
    )
    if shutil.which("brotli"):
        subprocess.run(
            ["brotli", "-f", "-q", "11", "-o", str(path) + ".br", str(path)], check=True
        )


def write_pages(out: Path, page: str, *, dev: bool = False) -> None:
    """Publish shared navigation, dedicated tests, and the unrestricted bench."""
    if dev:
        (out / "index.html").write_text(page, encoding="utf-8", newline="\n")
        return
    for route in ("cpu-culled", "no-instancing", "no-instancing-displacement", "textured-sprites", "advanced", "eidolon"):
        destination = out / route / "index.html"
        destination.parent.mkdir(parents=True, exist_ok=True)
        # All routes share engines and assets at the bundle root.
        route_page = page.replace('<meta charset="utf-8" />',
            '<meta charset="utf-8" /><base href="../" />')
        destination.write_text(route_page, encoding="utf-8", newline="\n")
    (out / "index.html").write_text(
        '<!doctype html>\n<html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        '<link rel="icon" href="data:,">'
        '<meta http-equiv="refresh" content="0;url=./cpu-culled/">'
        '<title>Grass demos</title></head><body>'
        '<a href="./cpu-culled/">CPU-culled grass test</a> · '
        '<a href="./advanced/">Advanced (WebGPU)</a></body></html>\n',
        encoding="utf-8", newline="\n",
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "target/grass-demo/site")
    parser.add_argument("--bindgen", default=None, help="CLI to use if it matches the bench's lockfile")
    parser.add_argument("--dev", action="store_true", help="debug engine, no WebGL2 fallback")
    parser.add_argument("--no-compress", action="store_true", help="skip the .gz/.br siblings")
    args = parser.parse_args(argv)
    # Absolute: the cargo and wasm-bindgen calls run from the crate directory,
    # so a relative output would resolve against two different roots.
    args.output = args.output.resolve()

    bindgen = resolve_bindgen(args.bindgen, target_dir())
    assets = shipped_assets()
    if not assets:
        raise SystemExit(f"no shipped assets under {CRATE / 'assets'}")
    prefix = asset_prefix(assets)

    out = args.output
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    log("Building the grass bench (WebGPU)...")
    engines = [("bench", build_engine("wasm-dev" if args.dev else "wasm-release", ["webgpu"], prefix))]
    if not args.dev:
        log("Building the grass bench (WebGL2 fallback)...")
        engines.append(("bench_webgl2", build_engine("wasm-release-webgl2", ["downlevel"], prefix)))
    for name, wasm in engines:
        command = [
            bindgen, str(wasm), "--out-dir", str(out), "--target", "web",
            "--no-typescript", "--out-name", name,
        ]
        if args.dev:
            command.append("--keep-debug")
        else:
            # The name section is more than half the shipped engine (53 MB of
            # 99 on the first build). Browser stack traces lose their function
            # names with it; the local dev build keeps them, and the trace
            # profile exists for profiling.
            command += ["--remove-name-section", "--remove-producers-section"]
        run(command)

    for path in assets:
        destination = out / prefix / path.relative_to(CRATE)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, destination)

    # The page shell, with every URL it hands out versioned: the assets ride
    # the prefix baked into the wasm, each engine its own build hash.
    page = (CRATE / "web" / "index.html").read_text(encoding="utf-8")
    wasm_main = out / "bench_bg.wasm"
    page = page.replace("const WASM_BYTES = 0;", f"const WASM_BYTES = {wasm_main.stat().st_size};")
    page = page.replace("./assets/", f"./{prefix}/assets/")
    biggest = sorted(
        (p for p in (out / prefix).rglob("*") if p.suffix in {".glb", ".png", ".ktx2"}),
        key=lambda p: p.stat().st_size,
        reverse=True,
    )[:8]
    prefetch = ", ".join(f'"./{p.relative_to(out).as_posix()}"' for p in biggest)
    page = page.replace("const PREFETCH = [];", f"const PREFETCH = [{prefetch}];")
    for name in [name for name, _ in engines]:
        build_hash = hashlib.md5((out / f"{name}_bg.wasm").read_bytes()).hexdigest()[:8]
        page = page.replace(f"./{name}.js", f"./{name}.js?v={build_hash}")
        page = page.replace(f"./{name}_bg.wasm", f"./{name}_bg.wasm?v={build_hash}")
    if len(engines) > 1:
        size = (out / "bench_webgl2_bg.wasm").stat().st_size
        page = page.replace("const WASM_BYTES_WEBGL2 = 0;", f"const WASM_BYTES_WEBGL2 = {size};")
    write_pages(out, page, dev=args.dev)

    if not args.no_compress:
        log("Compressing the engines and the models...")
        for path in [*out.glob("*.wasm"), *out.glob("*.js"), *(out / prefix).rglob("*.glb")]:
            compress(path)

    total = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
    log(f"Grass demo built to {out} ({total / 1e6:.0f} MB)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
