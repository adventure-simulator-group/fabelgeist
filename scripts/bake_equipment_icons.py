"""Run the equipment portrait bake with the configured Blender executable."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--web-output", type=Path)
    args = parser.parse_args()
    blender = os.environ.get("BLENDER_BIN") or shutil.which("blender")
    if not blender:
        parser.error("Set BLENDER_BIN to Blender 5.2 or put blender on PATH")
    root = Path(__file__).resolve().parent.parent
    web_output = args.web_output
    if web_output is None and args.directory.resolve() == root / "assets/equipment/procedural":
        web_output = root / "crates/strategic-web/static/equipment-icons"
    web_args = ["--web-output", str(web_output)] if web_output else []
    subprocess.run([
        blender, "--background", "--python-exit-code", "1", "--python",
        str(root / "scripts/render_equipment_icons.py"), "--", str(args.directory),
        *web_args,
    ], check=True)


if __name__ == "__main__":
    main()
