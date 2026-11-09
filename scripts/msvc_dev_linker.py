#!/usr/bin/env python3
"""Keep MSVC development links free of PDB files."""

from __future__ import annotations

import ctypes
from ctypes import wintypes
import os
from pathlib import Path, PureWindowsPath
import subprocess
import sys


def windows_arguments(command_line: str) -> list[str]:
    shell = ctypes.WinDLL("shell32", use_last_error=True)
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    shell.CommandLineToArgvW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(ctypes.c_int)]
    shell.CommandLineToArgvW.restype = ctypes.POINTER(wintypes.LPWSTR)
    kernel.LocalFree.argtypes = [wintypes.HLOCAL]
    kernel.LocalFree.restype = wintypes.HLOCAL
    count = ctypes.c_int()
    # Rust separates response-file arguments with newlines; this Windows API
    # recognizes spaces and tabs, unlike LLD's response-file tokenizer.
    command_line = command_line.replace("\r", " ").replace("\n", " ")
    arguments = shell.CommandLineToArgvW("link.exe " + command_line, ctypes.byref(count))
    if not arguments:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        return [arguments[index] for index in range(1, count.value)]
    finally:
        kernel.LocalFree(arguments)


def expanded_arguments(arguments: list[str], active: set[Path] | None = None) -> list[str]:
    active = set() if active is None else active
    expanded = []
    for argument in arguments:
        if not argument.startswith("@"):
            expanded.append(argument)
            continue
        response = Path(argument[1:]).resolve()
        if response in active:
            raise ValueError(f"Recursive linker response file: {response}")
        data = response.read_bytes()
        if data.startswith((b"\xff\xfe", b"\xfe\xff")):
            text = data.decode("utf-16")
        elif b"\0" in data[:64]:
            text = data.decode("utf-16-le")
        else:
            text = data.decode("utf-8-sig")
        expanded.extend(expanded_arguments(windows_arguments(text), active | {response}))
    return expanded


def development_link(arguments: list[str]) -> bool:
    outputs = [argument[5:] for argument in arguments if argument.lower().startswith("/out:")]
    if not outputs:
        return False
    parent = PureWindowsPath(outputs[-1]).parent
    if parent.name.lower() in ("deps", "examples"):
        return parent.parent.name.lower() == "debug"
    return (
        parent.parent.name.lower() == "build"
        and parent.parent.parent.name.lower() == "debug"
    )


def rust_linker() -> Path:
    compiler = os.environ.get("RUSTC", "rustc")
    sysroot = Path(subprocess.check_output([compiler, "--print", "sysroot"], text=True).strip())
    version = subprocess.check_output([compiler, "--version", "--verbose"], text=True)
    host = next((line[6:] for line in version.splitlines() if line.startswith("host: ")), None)
    if host is None:
        raise ValueError("rustc did not report its host architecture")
    linker = sysroot / "lib" / "rustlib" / host / "bin" / "rust-lld.exe"
    if not linker.is_file():
        raise FileNotFoundError(f"Rust's bundled linker is missing: {linker}")
    return linker


def main() -> int:
    if os.name != "nt":
        raise ValueError("The MSVC linker policy runs on Windows")
    arguments = sys.argv[1:]
    expanded = expanded_arguments(arguments)
    command = [str(rust_linker())]
    if arguments[:2] != ["-flavor", "link"]:
        command.extend(["-flavor", "link"])
    command.extend(arguments)
    if development_link(expanded):
        command.append("/DEBUG:NONE")
    return subprocess.run(command).returncode


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"MSVC linker policy: {error}", file=sys.stderr)
        sys.exit(1)
