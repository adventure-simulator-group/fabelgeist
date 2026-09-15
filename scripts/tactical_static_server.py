#!/usr/bin/env python3
"""Serve the tactical wasm bundle at the mount point the client expects.

The wasm client hardcodes `/tactical/assets` as its Bevy asset root (see
`crates/adventuresim-tactical-client/src/main.rs`) because the browser stack
nests this directory at `/tactical`:

    app.nest_service("/tactical", ServeDir::new(tactical_static_path))

Serving the bundle at the server root instead answers `tactical.html` and its
own relative config fetches while 404ing every asset Bevy asks for, which
renders as an untextured scene rather than an error. Mirror the mount point.
"""

from __future__ import annotations

import argparse
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import sys


MOUNT = "/tactical"


class TacticalBundleHandler(SimpleHTTPRequestHandler):
    """Strip the mount point, then resolve inside the served directory."""

    def mounted_path(self) -> str | None:
        route = self.path.split("?", 1)[0].split("#", 1)[0]
        if route == MOUNT:
            return "/"
        if route.startswith(MOUNT + "/"):
            return self.path[len(MOUNT):]
        return None

    def translate_path(self, path: str) -> str:
        # The base class re-reads nothing from self.path, so rewriting the
        # argument is enough to keep its traversal containment intact.
        if path.split("?", 1)[0].split("#", 1)[0].startswith(MOUNT):
            path = path[len(MOUNT):] or "/"
        return super().translate_path(path)

    def send_head(self):
        if self.mounted_path() is None:
            self.send_error(404, f"Not Found (the bundle is served at {MOUNT}/)")
            return None
        return super().send_head()


def serve(directory: Path, port: int, bind: str = "127.0.0.1") -> int:
    if not (directory / "tactical.html").is_file():
        print(f"not a tactical bundle: {directory}", file=sys.stderr)
        return 1

    def build(*args: object, **kwargs: object) -> TacticalBundleHandler:
        return TacticalBundleHandler(*args, directory=str(directory), **kwargs)

    with ThreadingHTTPServer((bind, port), build) as httpd:
        print(f"serving {directory} at http://{bind}:{port}{MOUNT}/", file=sys.stderr)
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            return 0
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", required=True, type=Path)
    parser.add_argument("--port", required=True, type=int)
    parser.add_argument("--bind", default="127.0.0.1")
    args = parser.parse_args(argv)
    return serve(args.directory, args.port, args.bind)


if __name__ == "__main__":
    raise SystemExit(main())
