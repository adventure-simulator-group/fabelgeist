import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path

import scripts.showcase as showcase


class ShowcaseRoutingTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)

        self.site = root / "site"
        (self.site / "static" / "art-demo").mkdir(parents=True)
        (self.site / "static" / "art-demo" / "index.html").write_text("demo", encoding="utf-8")
        (self.site / "tactical" / "wasm").mkdir(parents=True)
        (self.site / "tactical" / "wasm" / "art-demo_bg.wasm").write_bytes(b"\0asm")
        (self.site / "texture-studio").mkdir()
        (self.site / "texture-studio" / "index.html").write_text("studio", encoding="utf-8")
        (self.site / "index.html").write_text("landing", encoding="utf-8")

        # Overlay order: the crate root wins over the top-level assets/.
        crate_assets = root / "crate" / "assets"
        top_assets = root / "assets"
        for base in (crate_assets, top_assets):
            (base / "art-demo").mkdir(parents=True)
        (crate_assets / "art-demo" / "shared.json").write_text("crate", encoding="utf-8")
        (top_assets / "art-demo" / "shared.json").write_text("top", encoding="utf-8")
        (top_assets / "art-demo" / "catalog.json").write_text("[]", encoding="utf-8")
        (root / "secret.txt").write_text("nope", encoding="utf-8")

        handler = partial(showcase.ShowcaseHandler, site=self.site, asset_roots=[crate_assets, top_assets])
        self.httpd = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        self.addCleanup(self.httpd.server_close)
        self.addCleanup(self.httpd.shutdown)
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
        self.origin = "http://127.0.0.1:{}".format(self.httpd.server_address[1])

    def get(self, path: str):
        try:
            with urllib.request.urlopen(self.origin + path, timeout=5) as response:
                return response.status, response.read().decode(), response.headers
        except urllib.error.HTTPError as error:
            return error.code, "", error.headers

    def test_art_demo_route_serves_the_strategic_web_page_shell(self):
        status, body, _ = self.get("/art-demo")
        self.assertEqual((status, body), (200, "demo"))

    def test_landing_and_studio_directories_serve_index(self):
        self.assertEqual(self.get("/")[:2], (200, "landing"))
        self.assertEqual(self.get("/texture-studio/")[:2], (200, "studio"))

    def test_asset_overlay_prefers_crate_roots_then_falls_back(self):
        self.assertEqual(self.get("/tactical/assets/art-demo/shared.json")[:2], (200, "crate"))
        self.assertEqual(self.get("/tactical/assets/art-demo/catalog.json")[:2], (200, "[]"))
        self.assertEqual(self.get("/tactical/assets/art-demo/missing.json")[0], 404)

    def test_wasm_content_type_and_no_cache(self):
        status, _, headers = self.get("/tactical/wasm/art-demo_bg.wasm")
        self.assertEqual(status, 200)
        self.assertEqual(headers["Content-Type"], "application/wasm")
        self.assertEqual(headers["Cache-Control"], "no-cache")

    def test_traversal_cannot_escape_any_root(self):
        self.assertEqual(self.get("/tactical/assets/../../secret.txt")[0], 404)
        self.assertEqual(self.get("/tactical/assets/%2e%2e/%2e%2e/secret.txt")[0], 404)
        self.assertEqual(self.get("/../secret.txt")[0], 404)


if __name__ == "__main__":
    unittest.main()
