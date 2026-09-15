import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path

import scripts.tactical_static_server as static_server


class TacticalBundleMountTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        (self.root / "tactical.html").write_text("page", encoding="utf-8")
        (self.root / "assets" / "shaders").mkdir(parents=True)
        (self.root / "assets" / "shaders" / "tactical_sun.wgsl").write_text(
            "shader", encoding="utf-8"
        )

        handler = partial(static_server.TacticalBundleHandler, directory=str(self.root))
        self.httpd = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        self.addCleanup(self.httpd.server_close)
        self.addCleanup(self.httpd.shutdown)
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
        self.origin = "http://127.0.0.1:{}".format(self.httpd.server_address[1])

    def get(self, path: str) -> int:
        try:
            with urllib.request.urlopen(self.origin + path, timeout=5) as response:
                return response.status
        except urllib.error.HTTPError as error:
            return error.code

    def test_bundle_answers_the_hardcoded_wasm_asset_root(self):
        # The wasm client sets `file_path: "/tactical/assets"`; serving the
        # bundle anywhere else 404s every asset without failing the page.
        self.assertEqual(self.get("/tactical/assets/shaders/tactical_sun.wgsl"), 200)
        self.assertEqual(self.get("/tactical/tactical.html"), 200)

    def test_unmounted_paths_are_not_served(self):
        self.assertEqual(self.get("/assets/shaders/tactical_sun.wgsl"), 404)
        self.assertEqual(self.get("/tactical.html"), 404)

    def test_mount_prefix_does_not_defeat_traversal_containment(self):
        self.assertEqual(self.get("/tactical/../../etc/passwd"), 404)
        self.assertEqual(self.get("/tacticalother/tactical.html"), 404)

    def test_serve_rejects_a_directory_that_is_not_a_bundle(self):
        with tempfile.TemporaryDirectory() as empty:
            self.assertEqual(static_server.serve(Path(empty), 0), 1)


if __name__ == "__main__":
    unittest.main()
