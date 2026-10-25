"""Qualify mutable JSONL inputs against exact indexed byte prefixes."""
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[1]
index_path = ROOT / "continuation/producer-reached-role-index.json"
index = json.loads(index_path.read_text(encoding="utf-8-sig"))
rows = []
for entry in index["input_receipts"]:
    path = ROOT / entry["path"]
    data = path.read_bytes()
    actual = hashlib.sha256(data).hexdigest()
    target = entry["sha256"]
    length = None
    if actual == target:
        length = len(data)
        status = "exact-current-input-bytes"
    else:
        hasher = hashlib.sha256()
        consumed = 0
        for line in data.splitlines(keepends=True):
            hasher.update(line)
            consumed += len(line)
            if hasher.hexdigest() == target:
                length = consumed
                break
        status = "exact-indexed-prefix-plus-later-appends" if length is not None else "unresolved-input-byte-change"
    rows.append({
        "path": entry["path"], "indexed_sha256": target,
        "current_sha256_at_check": actual, "indexed_prefix_length_bytes": length,
        "current_length_bytes_at_check": len(data), "qualification": status,
        "limit": "Only exact indexed prefix participates in044 map; subsequent appended observations remain outside this bounded input."
    })
result = {
    "id": "RIG-CONT-PPS-SOURCE-SCOPE-RECONCILIATION-044-INPUT-PREFIXES",
    "index_sha256": hashlib.sha256(index_path.read_bytes()).hexdigest(),
    "input_prefix_qualifications": rows,
    "reproduction": "Before reproducing044, use each input file's first indexed_prefix_length_bytes and verify indexed_sha256. Four camera JSONL files received later appends; exact indexed prefixes prove older IDs/pointers used here remain unchanged. Index source_context observations themselves are preserved verbatim in044.",
    "unresolved_input_changes": [r for r in rows if r["qualification"] == "unresolved-input-byte-change"],
    "coverage_change": "none",
}
target = OUT / "source-scope-reconciliation-044-input-prefixes.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"inputs": len(rows), "later_appends": sum(r["qualification"] == "exact-indexed-prefix-plus-later-appends" for r in rows), "unresolved": len(result["unresolved_input_changes"])}))
