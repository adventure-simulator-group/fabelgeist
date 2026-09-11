"""Invoke offline equipment finishing without shell-dependent path quoting."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--stage", choices=["uv", "bake", "all"], default="all")
    args = parser.parse_args()
    blender = os.environ.get("BLENDER_BIN") or shutil.which("blender")
    if not blender:
        parser.error("Set BLENDER_BIN to the Blender executable for equipment UV unwrapping")
    scripts = {"uv": ["unwrap_armor.py"], "bake": ["bake_armor.py"],
               "all": ["unwrap_armor.py", "bake_armor.py"]}[args.stage]
    for name in scripts:
        script = Path(__file__).with_name(name)
        subprocess.run([blender, "--background", "--python-exit-code", "1", "--python",
                        str(script), "--", str(args.directory)], check=True)


if __name__ == "__main__":
    main()
