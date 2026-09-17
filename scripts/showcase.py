#!/usr/bin/env python3
"""Build and serve the browser showcases on one local site, with no database.

    just showcase

Three WebGPU applications are already independent of SpacetimeDB and the
strategic server; only their routing lived inside strategic-web. This puts them
under one origin as plain static files:

    /                   landing page linking the three
    /art-demo           the procedural art demo (armor, weapons, city, oak)
    /texture-studio/    Texture Studio
    /heraldry-studio/   Heraldry Studio

Nothing here starts SpacetimeDB, strategic-web, or a tactical server. The art
demo's hardcoded roots (`/tactical/wasm`, `/tactical/assets`, `/static/art-demo`)
are honoured so its bundle runs unmodified; assets are served straight from the
repository rather than copied, mirroring `scripts/build_wasm.py`'s overlay
order (crate asset dirs override the top-level `assets/`).

Only Python, cargo, the wasm32 target and a matching wasm-bindgen CLI are
needed. The same directory tree is what a static host would serve.
"""

from __future__ import annotations

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import os
from pathlib import Path
import posixpath
import shutil
import subprocess
import sys
import tomllib
import urllib.parse
import webbrowser


ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "target" / "showcase" / "site"
ART_DEMO_STATIC = ROOT / "crates" / "strategic-web" / "static" / "art-demo"
# Same overlay as scripts/build_wasm.py: crate asset dirs win over assets/.
ASSET_ROOTS = [*sorted((ROOT / "crates").glob("*/assets")), ROOT / "assets"]

LANDING_PAGE = """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="theme-color" content="#171b1d">
  <title>Fabelgeist · Showcase</title>
  <style>
    html { background: #171b1d; color: #e9e9dd; font: 16px/1.5 system-ui, sans-serif; }
    body { margin: 0; min-height: 100vh; display: grid; place-content: center; padding: 3rem 1.5rem; }
    h1 { font-size: 1.1rem; font-weight: 500; letter-spacing: .12em; text-transform: uppercase; margin: 0 0 .25rem; }
    h1 span { color: #8fa08a; font-weight: 400; letter-spacing: .04em; text-transform: none; margin-left: .75rem; }
    p.lede { color: #b7c2ab; margin: 0 0 2rem; max-width: 40rem; }
    ul { list-style: none; padding: 0; margin: 0; display: grid; gap: 1rem; }
    a { display: block; padding: 1.25rem 1.5rem; border: 1px solid #2c3438; border-radius: 4px;
        color: inherit; text-decoration: none; transition: border-color .15s; }
    a:hover { border-color: #8fa08a; }
    a strong { display: block; font-weight: 500; font-size: 1.05rem; }
    a small { color: #b7c2ab; }
    footer { color: #6f7a70; font-size: .85rem; margin-top: 2.5rem; }
  </style>
</head>
<body>
  <main>
    <h1>Fabelgeist<span>Showcase</span></h1>
    <p class="lede">Procedural work from the Fabelgeist project, running in the browser. Each page needs a browser with WebGPU, such as current Chrome or Edge.</p>
    <ul>
      <li><a href="/art-demo"><strong>Procedural art</strong><small>Museum armor, weapons, a 30,000-resident city, and an oak with terrain.</small></a></li>
      <li><a href="/texture-studio/"><strong>Texture Studio</strong><small>Material editor for the procedural texture catalogue.</small></a></li>
      <li><a href="/heraldry-studio/"><strong>Heraldry Studio</strong><small>Parametric coats of arms on painted shields and panels.</small></a></li>
    </ul>
    <footer>No account, database, or game server is involved on these pages.</footer>
  </main>
</body>
</html>
"""


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def run(command: list[str]) -> None:
    result = subprocess.run(command, cwd=ROOT)
    if result.returncode:
        raise SystemExit(f"command failed ({result.returncode}): {' '.join(command)}")


def wasm_bindgen(cli: str) -> str:
    """Return the wasm-bindgen CLI path after checking it matches Cargo.lock."""
    resolved = shutil.which(cli)
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    expected = next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")
    if resolved is None:
        raise SystemExit(f"Missing wasm-bindgen. Install wasm-bindgen-cli --version {expected}")
    actual = subprocess.check_output([resolved, "--version"], text=True).strip().split()[-1]
    if actual != expected:
        raise SystemExit(
            f"wasm-bindgen CLI {actual} does not match Rust {expected}. "
            f"Install wasm-bindgen-cli --version {expected}, or pass --bindgen PATH."
        )
    return resolved


def build_art_demo(site: Path, bindgen: str, dev: bool) -> None:
    log("Building the art demo...")
    out = site / "tactical" / "wasm"
    out.mkdir(parents=True, exist_ok=True)
    if not dev:
        run([sys.executable, str(ROOT / "scripts" / "build_wasm.py"),
             "--bindgen", bindgen])
        generated = ROOT / "crates" / "adventuresim-stdb-module" / "static" / "wasm"
        for name in ("art-demo.js", "art-demo_bg.wasm", "bundle-sizes.json"):
            shutil.copy2(generated / name, out / name)
        shutil.copytree(ART_DEMO_STATIC, site / "static" / "art-demo", dirs_exist_ok=True)
        return
    if shutil.which("rustup"):
        subprocess.run(["rustup", "target", "add", "wasm32-unknown-unknown"], cwd=ROOT, check=False)
    command = [
        "cargo", "build", "--package", "adventuresim-tactical-client", "--bin", "art-demo",
        "--target", "wasm32-unknown-unknown",
    ]
    run(command)
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    wasm = target / "wasm32-unknown-unknown" / ("debug" if dev else "release") / "art-demo.wasm"
    run([
        bindgen, str(wasm), "--out-dir", str(out), "--target", "web", "--no-typescript",
        "--remove-name-section", "--remove-producers-section",
    ])
    # The page shell and its scripts, at the /static/art-demo root they expect.
    shutil.copytree(ART_DEMO_STATIC, site / "static" / "art-demo", dirs_exist_ok=True)


def build_studio(site: Path, script: str, folder: str, bindgen: str, dev: bool) -> None:
    log(f"Building {folder}...")
    command = [sys.executable, str(ROOT / "scripts" / script), "--output", str(site / folder), "--bindgen", bindgen]
    if dev:
        command.append("--dev")
    run(command)


def build(site: Path, bindgen_cli: str, dev: bool) -> None:
    bindgen = wasm_bindgen(bindgen_cli)
    site.mkdir(parents=True, exist_ok=True)
    build_art_demo(site, bindgen, dev)
    build_studio(site, "build_texture_studio.py", "texture-studio", bindgen, dev)
    build_studio(site, "build_heraldry_studio.py", "heraldry-studio", bindgen, dev)
    # Fixed newlines so the file hashes the same from Windows and Linux builds.
    (site / "index.html").write_text(LANDING_PAGE, encoding="utf-8", newline="\n")
    log(f"Showcase built to {site}")


class ShowcaseHandler(SimpleHTTPRequestHandler):
    """Static site plus the art demo's hardcoded roots, served without copies."""

    extensions_map = {
        **SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".mjs": "text/javascript",
        ".json": "application/json",
        ".glb": "model/gltf-binary",
        ".building": "application/octet-stream",
    }

    def __init__(self, *args, site: Path, asset_roots: list[Path], **kwargs):
        self.site = site
        self.asset_roots = asset_roots
        super().__init__(*args, directory=str(site), **kwargs)

    def end_headers(self) -> None:
        # Rebuilt bundles must not be served from the browser's heuristic cache.
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def translate_path(self, path: str) -> str:
        route = urllib.parse.unquote(path.split("?", 1)[0].split("#", 1)[0])
        route = posixpath.normpath(route)
        if route == "/art-demo":
            return str(self.site / "static" / "art-demo" / "index.html")
        if route.startswith("/tactical/assets/"):
            return self.overlay(route[len("/tactical/assets/"):])
        return self.contained(self.site, route.lstrip("/"))

    def overlay(self, relative: str) -> str:
        """First asset root that has the file; the last one otherwise, so it 404s."""
        candidates = [self.contained(root, relative) for root in self.asset_roots]
        return next((c for c in candidates if Path(c).is_file()), candidates[-1])

    @staticmethod
    def contained(root: Path, relative: str) -> str:
        """Join and refuse anything that escapes the root."""
        resolved = (root / relative).resolve()
        if resolved != root.resolve() and root.resolve() not in resolved.parents:
            return str(root / "__forbidden__")
        return str(resolved)

    def log_message(self, format: str, *args) -> None:  # noqa: A002 - base signature
        if os.environ.get("SHOWCASE_LOG"):
            super().log_message(format, *args)


def serve(site: Path, port: int, open_browser: bool) -> None:
    if not (site / "index.html").is_file():
        raise SystemExit(f"no showcase build at {site}; run without --skip-build first")
    handler = partial(ShowcaseHandler, site=site, asset_roots=ASSET_ROOTS)
    with ThreadingHTTPServer(("127.0.0.1", port), handler) as httpd:
        url = f"http://localhost:{httpd.server_address[1]}/"
        log(f"Showcase at {url}  (WebGPU browser required; Ctrl-C to stop)")
        if open_browser:
            webbrowser.open(url)
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            log("")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--port", type=int, default=8090)
    parser.add_argument("--site", type=Path, default=SITE, help="build output and web root")
    parser.add_argument("--bindgen", default="wasm-bindgen", help="wasm-bindgen CLI matching Cargo.lock")
    parser.add_argument("--dev", action="store_true", help="debug profiles for faster iteration")
    parser.add_argument("--skip-build", action="store_true", help="serve the previous build")
    parser.add_argument("--build-only", action="store_true", help="build, then exit without serving")
    parser.add_argument("--no-open", action="store_true", help="do not open a browser tab")
    args = parser.parse_args(argv)

    if not args.skip_build:
        build(args.site, args.bindgen, args.dev)
    if not args.build_only:
        serve(args.site, args.port, open_browser=not args.no_open)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
