#!/usr/bin/env python3
"""Execute the same deterministic sampling contract on native and wasm32."""

import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    runner = shutil.which("wasm-bindgen-test-runner")
    if not runner or not shutil.which("node"):
        raise SystemExit("Install Node and wasm-bindgen-cli 0.2.108 before testing.")
    command = ["cargo", "test", "--locked", "-p", "fabelgeist-determinism"]
    subprocess.run(command, cwd=ROOT, check=True)
    environment = dict(os.environ)
    environment["CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER"] = runner
    subprocess.run(
        [*command, "--target", "wasm32-unknown-unknown"],
        cwd=ROOT, env=environment, check=True,
    )


if __name__ == "__main__":
    main()
