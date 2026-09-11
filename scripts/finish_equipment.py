"""Invoke offline equipment finishing without shell-dependent path quoting."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    blender = os.environ.get("BLENDER_BIN") or shutil.which("blender")
    if not blender:
        parser.error("Set BLENDER_BIN to the Blender executable for equipment UV unwrapping")
    script = Path(__file__).with_name("unwrap_armor.py")
    subprocess.run([blender, "--background", "--python-exit-code", "1", "--python",
                    str(script), "--", str(args.directory)], check=True)


if __name__ == "__main__":
    main()
