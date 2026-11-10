"""Run the unmodified original restore; normalize Git's Windows root spelling."""
import os
from pathlib import Path
import runpy
import subprocess
import sys

root = Path.cwd().resolve()
original = subprocess.check_output


def checked_output(args, *positional, **keywords):
    result = original(args, *positional, **keywords)
    if args == ["git", "rev-parse", "--show-toplevel"] and keywords.get("text"):
        actual = Path(result.strip()).resolve()
        if os.path.normcase(str(actual)) != os.path.normcase(str(root)):
            raise RuntimeError("Git root is not the working directory")
        return str(root) + "\n"
    return result


subprocess.check_output = checked_output
script = root / "handoff/726-audit-20261030-adler/restore.py"
sys.argv = [str(script), *sys.argv[1:]]
runpy.run_path(str(script), run_name="__main__")
