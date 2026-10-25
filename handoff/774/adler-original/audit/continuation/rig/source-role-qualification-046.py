"""Qualify the fifteen stable-ID delta rows without copying whole receipts."""
from pathlib import Path
from collections import Counter
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[1]


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stable(row):
    return tuple(row[k] for k in ("record", "receipt", "json_pointer"))


old_path = OUT / "source-scope-reconciliation-044.json"
new_path = ROOT / "continuation/producer-reached-role-index.json"
old = load(old_path)
current = load(new_path)
old_keys = {stable(row) for row in old["unqualified_observation_schema_map"]}
delta = [row for row in current["incomplete_metadata_observations"] if stable(row) not in old_keys]
assert len(delta) == 15
camera_path = ROOT / "continuation/camera/traces.jsonl"
camera = next(json.loads(line) for line in camera_path.read_text(encoding="utf-8-sig").splitlines()
              if json.loads(line).get("id") == "CONT-CAMERA-PP-009")
native_path = ROOT / "continuation/native/world5-final-packet.json"
native = load(native_path)
native_sources_path = ROOT / "continuation/native/source-reads.json"
native_sources = load(native_sources_path)
rows = []
for item in delta:
    source_id = item["source_id"]
    camera_index, proof = next((i, p) for i, p in enumerate(camera["exact_source_proofs"]) if p["source_id"] == source_id)
    native_index, native_proof = next((i, p) for i, p in enumerate(native_sources) if p["source_id"] == source_id)
    path = Path(proof["resolved_path"])
    observed = sha(path)
    assert observed == proof["sha256"] == native_proof["sha256"]
    assert item["sha256"] is None or item["sha256"] == observed
    count = len(path.read_bytes().splitlines())
    assert all(1 <= a <= b <= count for a, b in proof["read_ranges"])
    original = item["source_context"]
    cell = original["index"]
    per_cell_index = next(i for i, p in enumerate(camera["per_cell"]) if p["index"] == cell)
    roles = camera["per_cell"][per_cell_index]
    rows.append({
        "input_ref": {k: item[k] for k in ("record", "receipt", "json_pointer")},
        "source_id": source_id, "original_schema": "coverage-navigation" if "trace_ref" in original else "per-cell-role-summary-without-inline-hash-or-range",
        "source_sha256_verified": observed, "exact_material_ranges": proof["read_ranges"],
        "actual_EOF": count, "source_proof_refs": [
            {"record": "continuation/camera/traces.jsonl", "receipt": camera["id"], "json_pointer": f"/exact_source_proofs/{camera_index}"},
            {"record": "continuation/native/source-reads.json", "json_pointer": "/" + str(native_index)},
        ],
        "role_graph_ref": {"record": "continuation/camera/traces.jsonl", "receipt": camera["id"], "json_pointer": f"/per_cell/{per_cell_index}"},
        "actual_contract_refs": ["CONT-CAMERA-PP-009/contracts_consumed/" + str(i) for i in roles["contracts"]],
        "disposition": "navigation-summary-qualified-by-exact-world5-source-and-actual-recipient-proof; not-independent-read",
        "remaining_actual_unread": roles["remaining_actual_unread"],
        "limit": "Preserve all world5 findings and counterpart proof limits. Matching hashes confirm locator only; individually traced source/retained-state/operations/actual recipient graph supplies producer evidence. No owner-fit or lens clearance.",
    })
result = {
    "id": "RIG-CONT-PPS-SOURCE-ROLE-QUALIFICATION-046-REFRESH-DELTA",
    "lens": "audit-producer-propagation-only",
    "inputs": [{"path": str(path.relative_to(ROOT)), "sha256": sha(path)} for path in
               (old_path, new_path, camera_path, native_path, native_sources_path, OUT / "source-role-qualification-045.json")],
    "stable_observation_key": ["record", "receipt", "json_pointer"],
    "original_unqualified_count": len(old_keys), "refreshed_unqualified_count": len(current["incomplete_metadata_observations"]),
    "original_keys_missing_from_refresh": [list(key) for key in sorted(old_keys - {stable(row) for row in current["incomplete_metadata_observations"]})],
    "delta_dispositions": rows,
    "summary": {"delta_rows_qualified": len(rows), "original_rows_qualified_by045": 231, "cumulative_rows": 246,
                "schema_counts": dict(Counter(row["original_schema"] for row in rows)),
                "genuinely_unresolved_material_obligations": sum(bool(row["remaining_actual_unread"]) for row in rows)},
    "previous_four_EOF_observations": "Exact original ranges and fresh real-body proof are preserved in045/overrun_observation_qualifications; nonexistent lines receive zero credit.",
    "remaining_material_obligations": [],
    "first_unfinished_cell": None,
    "next": "Coordinator evaluates producer reached-role scope then owner-fit delta. Findings and later lenses remain mandatory; no edits or additional lens started here.",
    "source_edits": 0, "project_commands_or_tests": 0, "overall_lens_clearance": "not awarded",
}
target = OUT / "source-role-qualification-046-refresh-delta.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["summary"]))
print("sha256=" + sha(target))
