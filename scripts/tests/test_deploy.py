import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import scripts.deploy as deploy


class PlanTests(unittest.TestCase):
    def test_upload_changed_and_new_delete_gone(self):
        local = {"a": "1", "b": "2new", "c": "3"}
        remote = {"a": "1", "b": "2old", "d": "4"}
        upload, delete = deploy.plan_sync(local, remote)
        self.assertEqual(upload, ["b", "c"])
        self.assertEqual(delete, ["d"])

    def test_empty_remote_uploads_everything(self):
        upload, delete = deploy.plan_sync({"x": "1", "y": "2"}, {})
        self.assertEqual((upload, delete), (["x", "y"], []))


class CollectTests(unittest.TestCase):
    def test_site_plus_asset_overlay_in_build_wasm_order(self):
        with tempfile.TemporaryDirectory() as work:
            root = Path(work)
            site = root / "site"
            (site / "tactical" / "wasm").mkdir(parents=True)
            (site / "index.html").write_text("landing")
            (site / "tactical" / "wasm" / "art-demo.js").write_text("js")
            crate_assets = root / "crate" / "assets"
            top_assets = root / "assets"
            (crate_assets / "fonts").mkdir(parents=True)
            (top_assets / "fonts").mkdir(parents=True)
            (crate_assets / "fonts" / "shared.ttf").write_text("crate")
            (top_assets / "fonts" / "shared.ttf").write_text("top")
            (top_assets / "only-top.json").write_text("{}")
            # deploy.py imports the module as plain `showcase`; patch that instance.
            with mock.patch.object(deploy.showcase, "SITE", site), mock.patch.object(
                deploy.showcase, "ASSET_ROOTS", [crate_assets, top_assets]
            ):
                files = deploy.collect_showcase_files()
            self.assertEqual(
                sorted(files),
                ["index.html", "tactical/assets/fonts/shared.ttf", "tactical/assets/only-top.json", "tactical/wasm/art-demo.js"],
            )
            self.assertEqual(files["tactical/assets/fonts/shared.ttf"].read_text(), "crate")


class RenderTests(unittest.TestCase):
    def test_showcase_caddyfile_renders_without_placeholders(self):
        text = deploy.render(deploy.DEPLOY_DIR / "Caddyfile", {"DOMAIN": "example.com", "DEPLOY_ROOT": "/opt/x"})
        self.assertIn("example.com {", text)
        self.assertIn("root * /opt/x/site", text)
        self.assertNotIn("{{", text)

    def test_game_templates_render_without_placeholders(self):
        values = {"DOMAIN": "example.com", "DEPLOY_ROOT": "/opt/x", "SERVICE_USER": "svc", "DATABASE": "db"}
        for name in ["Caddyfile.game", *(f"{unit}.service" for unit in deploy.GAME_UNITS)]:
            text = deploy.render(deploy.DEPLOY_DIR / name, values)
            self.assertNotIn("{{", text, name)

    def test_showcase_is_the_default_target(self):
        args = deploy.parse_args(["example.com"])
        self.assertEqual(args.target, "showcase")
        self.assertEqual(deploy.parse_args(["example.com", "--target", "game"]).target, "game")


class RemoteSyncTests(unittest.TestCase):
    """Run the remote-side bash steps locally against a temp root: same scripts, no ssh."""

    def run_script(self, script: str, *args: str) -> None:
        # Same byte-mode handoff as ssh_script: no newline translation on Windows.
        subprocess.run(["bash", "-s", "--", *args], input=script.encode("utf-8"), check=True)

    def test_remote_scripts_carry_no_carriage_returns(self):
        for name in ("BASE_SETUP_SCRIPT", "INSTALL_CADDY_SCRIPT", "SYNC_BEGIN_SCRIPT", "SYNC_FINISH_SCRIPT", "ROLLBACK_SCRIPT"):
            self.assertNotIn("\r", getattr(deploy, name), name)

    def sync(self, remote_root: Path, files: dict, remote_manifest: dict) -> None:
        local = {rel: deploy.digest(path) for rel, path in files.items()}
        upload, delete = deploy.plan_sync(local, remote_manifest)
        self.run_script(deploy.SYNC_BEGIN_SCRIPT, str(remote_root))
        buffer = io.BytesIO()
        deploy.write_tar(buffer, files, upload, {"deploy.json": b'{"stamp": true}'})
        subprocess.run(["tar", "xzf", "-", "--unlink-first", "-C", str(remote_root / "site.new")], input=buffer.getvalue(), check=True)
        if delete:
            subprocess.run(["bash", "-c", f"cd '{remote_root}/site.new' && xargs -0 -r rm -f"], input="\0".join(delete).encode(), check=True)
        (remote_root / "manifest.json.new").write_text(json.dumps(local))
        self.run_script(deploy.SYNC_FINISH_SCRIPT, str(remote_root))

    def test_first_upload_delta_and_rollback(self):
        with tempfile.TemporaryDirectory() as work:
            root = Path(work)
            local_dir = root / "local"
            (local_dir / "tactical" / "assets").mkdir(parents=True)
            (local_dir / "index.html").write_text("v1")
            (local_dir / "tactical" / "assets" / "a.glb").write_text("aaa")
            (local_dir / "tactical" / "assets" / "gone.glb").write_text("gone")
            remote = root / "remote"
            remote.mkdir()

            def files():
                return {p.relative_to(local_dir).as_posix(): p for p in local_dir.rglob("*") if p.is_file()}

            # First deploy: nothing on the box yet.
            self.sync(remote, files(), {})
            self.assertEqual((remote / "site" / "index.html").read_text(), "v1")
            self.assertEqual((remote / "site" / "tactical" / "assets" / "gone.glb").read_text(), "gone")
            self.assertTrue((remote / "site" / "deploy.json").is_file())
            manifest = json.loads((remote / "manifest.json").read_text())
            self.assertIn("tactical/assets/a.glb", manifest)

            # Second deploy: one change, one deletion; the delta is tiny and the swap is whole.
            (local_dir / "index.html").write_text("v2")
            (local_dir / "tactical" / "assets" / "gone.glb").unlink()
            local = {rel: deploy.digest(path) for rel, path in files().items()}
            upload, delete = deploy.plan_sync(local, manifest)
            self.assertEqual(upload, ["index.html"])
            self.assertEqual(delete, ["tactical/assets/gone.glb"])
            self.sync(remote, files(), manifest)
            self.assertEqual((remote / "site" / "index.html").read_text(), "v2")
            self.assertFalse((remote / "site" / "tactical" / "assets" / "gone.glb").exists())
            self.assertEqual((remote / "site" / "tactical" / "assets" / "a.glb").read_text(), "aaa")
            # The previous tree is intact: the hard-linked copy was never written through.
            self.assertEqual((remote / "site.prev" / "index.html").read_text(), "v1")
            self.assertTrue((remote / "site.prev" / "tactical" / "assets" / "gone.glb").exists())

            # Rollback swaps trees and manifests together.
            self.run_script(deploy.ROLLBACK_SCRIPT, str(remote))
            self.assertEqual((remote / "site" / "index.html").read_text(), "v1")
            self.assertEqual((remote / "site.prev" / "index.html").read_text(), "v2")
            self.assertIn("tactical/assets/gone.glb", json.loads((remote / "manifest.json").read_text()))


if __name__ == "__main__":
    unittest.main()
