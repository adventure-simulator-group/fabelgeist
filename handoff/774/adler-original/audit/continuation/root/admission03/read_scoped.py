"""Frozen source display only. No semantic analysis, validation, or coverage."""
import hashlib
import json
import pathlib
import sys

sys.stdout.reconfigure(encoding="utf-8")
root = pathlib.Path(__file__).resolve().parents[2]
queue = json.loads((root / "admission03-queue-v002/root.json").read_text(encoding="utf-8"))["queue"]
locators = {x["source_id"]: x for x in json.loads((root / "source-locators-2559-verified.json").read_text(encoding="utf-8"))["matched"]}
for arg in sys.argv[1:]:
    ix, _, spec = arg.partition(":")
    row = queue[int(ix)]
    source = row["source_scope_record"]
    path = pathlib.Path(locators.get(row["source_id"], {}).get("resolved_path", source.get("resolved_path", source["path"])))
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == row["sha256"], row["source_id"]
    lines = data.decode("utf-8").splitlines()
    ranges = []
    def collect(node):
        if isinstance(node, dict):
            for key, value in node.items():
                if key in ("read_ranges", "read_ranges_as_recorded", "material_ranges"):
                    ranges.extend(value)
                elif isinstance(value, (list, dict)):
                    collect(value)
        elif isinstance(node, list):
            for value in node:
                collect(value)
    collect(row)
    print(f"SOURCE {ix} {row['source_id']} EOF={len(lines)} scoped={sorted(set(map(tuple,ranges)))}")
    if spec == "meta":
        continue
    if spec:
        ranges = [list(map(int, part.split("-"))) for part in spec.split(",")]
    elif source.get("scope_kind") in ("changed", "changed_file", "original_changed_source") or not ranges:
        ranges = [[1, len(lines)]]
    chosen = set()
    for start, end in ranges:
        chosen.update(range(start, min(end, len(lines)) + 1))
    for i in sorted(chosen):
        print(f"{i}: {lines[i-1]}")
