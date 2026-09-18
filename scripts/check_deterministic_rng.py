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
    (re.compile(r"\bUniform\s*(?:::\s*)?<\s*usize\s*>"), "target-width Uniform"),
    (re.compile(r"\brandom_range\s*\("), "inferred-width random range"),
    (re.compile(r"\brand::seq::"), "Rand slice selection helper"),
    (re.compile(r"\b(?:SliceRandom|IndexedRandom|IteratorRandom)\b"), "Rand selection trait"),
)


def main() -> None:
    findings = []
    for path in sorted(CRATES.rglob("*.rs")):
        if EXCLUDED_PARTS.intersection(path.parts):
            continue
        text = path.read_text(encoding="utf-8")
        for pattern, label in FORBIDDEN:
            for match in pattern.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                findings.append(f"{path.relative_to(ROOT)}:{line}: {label}")
    if findings:
        raise SystemExit("\n".join(findings))
    print("deterministic RNG duplication check passed")


if __name__ == "__main__":
    main()
