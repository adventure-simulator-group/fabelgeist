#!/usr/bin/env python3
"""Reject duplicate deterministic RNGs and target-width sampling helpers."""

from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
CRATES = ROOT / "crates"
EXCLUDED_PARTS = {"fabelgeist-determinism", "adventuresim-stdb-client"}
FORBIDDEN = (
    (re.compile(r"\bfn\s+(?:splitmix64|mix64|unit_hash|seeded_hash)\b"), "custom RNG/mixer"),
    (re.compile(r"\bstruct\s+SplitMix64\b"), "duplicate SplitMix64"),
    (re.compile(r"\brand_xoshiro::SplitMix64\b"), "SplitMix64 outside shared crate"),
    (
        re.compile(r"\bUniform\s*(?:::\s*<[^;(){}]*>)?\s*::\s*new\b"),
        "Uniform construction outside shared crate",
    ),
    (re.compile(r"\brand(?:::\w+)+::Uniform\b"), "Rand Uniform outside shared crate"),
    (re.compile(r"\bUniform\s*<\s*usize\b"), "target-width Uniform"),
    (re.compile(r"\brandom_range\s*\("), "inferred-width random range"),
    (re.compile(r"\brand::seq::"), "Rand slice selection helper"),
    (re.compile(r"\b(?:SliceRandom|IndexedRandom|IteratorRandom)\b"), "Rand selection trait"),
)


def rust_code(source: str) -> str:
    """Replace Rust comments and strings with spaces while preserving lines."""
    code = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            end = len(source) if end == -1 else end
            code[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            start = index
            index += 2
            depth = 1
            while index < len(source) and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            for offset in range(start, index):
                if code[offset] != "\n":
                    code[offset] = " "
            continue
        raw = re.match(r"(?:br|rb|r)(#{0,255})\"", source[index:])
        if raw:
            start = index
            delimiter = '"' + raw.group(1)
            index += raw.end()
            end = source.find(delimiter, index)
            index = len(source) if end == -1 else end + len(delimiter)
            for offset in range(start, index):
                if code[offset] != "\n":
                    code[offset] = " "
            continue
        if source[index] == '"':
            start = index
            index += 1
            while index < len(source):
                if source[index] == "\\":
                    index += 2
                elif source[index] == '"':
                    index += 1
                    break
                else:
                    index += 1
            for offset in range(start, min(index, len(source))):
                if code[offset] != "\n":
                    code[offset] = " "
            continue
        index += 1
    return "".join(code)


def scan_text(source: str) -> list[tuple[int, str]]:
    code = rust_code(source)
    return [
        (match.start(), label)
        for pattern, label in FORBIDDEN
        for match in pattern.finditer(code)
    ]


def main() -> None:
    findings = []
    for path in sorted(CRATES.rglob("*.rs")):
        if EXCLUDED_PARTS.intersection(path.parts):
            continue
        source = path.read_text(encoding="utf-8")
        for offset, label in scan_text(source):
            line = source.count("\n", 0, offset) + 1
            findings.append(f"{path.relative_to(ROOT)}:{line}: {label}")
    if findings:
        raise SystemExit("\n".join(findings))
    print("deterministic RNG duplication check passed")


if __name__ == "__main__":
    main()
