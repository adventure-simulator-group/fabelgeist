"""Invoke offline equipment finishing without shell-dependent path quoting."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--stage", choices=["uv", "bake", "trim", "all"], default="all")
    parser.add_argument("--finish-recipe", type=Path, default=Path("assets_src/equipment/armor-finishes.json"))
    args = parser.parse_args()
    blender = os.environ.get("BLENDER_BIN") or shutil.which("blender")
    if not blender and args.stage != "trim":
        parser.error("Set BLENDER_BIN to the Blender executable for equipment UV unwrapping")
    scripts = {"uv": ["unwrap_armor.py"], "bake": ["bake_armor.py"],
               "trim": [], "all": ["unwrap_armor.py", "bake_armor.py"]}[args.stage]
    for name in scripts:
        script = Path(__file__).with_name(name)
        subprocess.run([blender, "--background", "--python-exit-code", "1", "--python",
                        str(script), "--", str(args.directory)], check=True)

    if args.stage in {"trim", "all"}:
        import sys
        subprocess.run([sys.executable, str(Path(__file__).with_name("trim_armor.py")),
                        str(args.directory), "--recipe", str(args.finish_recipe)], check=True)


if __name__ == "__main__":
    main()
