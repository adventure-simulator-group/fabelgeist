#!/usr/bin/env python3
"""Dev server for bench/web/public: no-store caching, /v<hash>/ prefix strip.

    bench/scripts/serve-web.py [port] [dir]
"""
import re
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
root = Path(sys.argv[2] if len(sys.argv) > 2 else Path(__file__).resolve().parent.parent / 'web/public')


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(root), **kwargs)

    def translate_path(self, path):
        return super().translate_path(re.sub(r'^/v[0-9a-f]{8}/', '/', path))

    def end_headers(self):
        self.send_header('Cache-Control', 'no-store')
        super().end_headers()

    def log_message(self, fmt, *args):
        pass


print(f'serving {root} on http://localhost:{port}', flush=True)
ThreadingHTTPServer(('', port), Handler).serve_forever()
