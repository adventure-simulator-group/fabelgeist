#!/usr/bin/env python3
"""Check printed JSON values against a fixed historical hash inventory."""
import hashlib
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parent.parent.parent
coverage_dir = root / "recovered/hash-matched-dependencies"
coverage = json.loads((coverage_dir / "DEPENDENCY-COVERAGE.json").read_bytes())
later = json.loads((coverage_dir / "LATER-MATCHES.json").read_bytes())
missing = {r["sha256"] for r in coverage["references"]
           if r["availability"] == "not_located_by_hash"}
missing -= {r["sha256"] for r in later["matches"]}
paths = sorted((root / "recovered/source-registration-outputs").glob("*.jsonl"))
paths += [root / "recovered/ledger-command-outputs/OUTPUTS.jsonl",
          root / "recovered/queue-command-outputs/RECORDS.jsonl",
          root / "recovered/late-command-history/RECORDS.jsonl"]
decoder = json.JSONDecoder()
seen = set()
found = set()
values = 0
for path in paths:
    with path.open(encoding="utf-8") as stream:
        for line in stream:
            rec = json.loads(line)
            text = rec.get("output_text", "")
            if not text or text in seen:
                continue
            seen.add(text)
            for match in re.finditer(r"(?m)^[ \t]*([\{\[])", text):
                try:
                    value, _ = decoder.raw_decode(text, match.start(1))
                except (ValueError, RecursionError):
                    continue
                if not isinstance(value, (dict, list)):
                    continue
                values += 1
                for ascii_only in (True, False):
                    for indent in (2, 4, None):
                        payload = json.dumps(value, ensure_ascii=ascii_only, indent=indent).encode()
                        for suffix in (b"", b"\n", b"\r\n"):
                            digest = hashlib.sha256(payload + suffix).hexdigest()
                            if digest in missing:
                                found.add(digest)
print(json.dumps({"unmatched_hashes_checked": len(missing),
                  "distinct_outputs_checked": len(seen),
                  "decoded_json_values": values,
                  "matching_hashes": sorted(found)}, indent=2))
