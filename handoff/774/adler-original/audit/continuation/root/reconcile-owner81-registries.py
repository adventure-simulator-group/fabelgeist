import argparse
import datetime
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--label", required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[2]
c = root / "continuation"
destination = c / f"root/owner81-registry-candidate-{args.label}.json"
assert not destination.exists()
paths = {
    "base": "continuation/cumulative-source-scope-producer-followup-008.json",
    "overlay": "continuation/owner-range-extensions.jsonl",
    "camera": "continuation/camera/owner81/source-range-registration.jsonl",
    "rig": "continuation/rig/producer-reached-owner81/additional-sources.jsonl",
    "native": "continuation/native/owner81/local-owner-source-registrations.jsonl",
}
blobs = {name: (root / path).read_bytes() for name, path in paths.items()}
pins = {name: {"record": paths[name], "sha256": hashlib.sha256(blob).hexdigest(), "bytes": len(blob)} for name, blob in blobs.items()}
base = json.loads(blobs["base"])
known = {row["source_id"]: row for row in base["files"]}
observations = []
groups = {}
issues = []
qualifications = []
raw_registry_rows = []
for supplier in ["camera", "rig", "native"]:
    parsed = [(line, json.loads(raw)) for line, raw in enumerate(blobs[supplier].decode("utf-8-sig").splitlines(), 1) if raw.strip()]
    qualified_ranges = {}
    for line, row in parsed:
        reference = {"registry": paths[supplier], "row_1based": line, "supplier": supplier, "original_record": row}
        raw_registry_rows.append(reference)
        if "source_id" not in row:
            assert row.get("kind") == "source_role_qualification", row
            qualifications.append(reference)
            for key, value in row.get("qualified_material_ranges_by_registration", {}).items():
                assert key not in qualified_ranges, key
                qualified_ranges[key] = {"ranges": value, "qualification": reference}
    for line, row in parsed:
        if "source_id" not in row:
            continue
        sid = row["source_id"]
        snapshot, relative = sid.split(":", 1)
        assert snapshot in ["pr-working", "physical-p16"], sid
        folder = "p16" if snapshot == "physical-p16" else snapshot
        frozen_root = (root.parent / "774-snapshots" / folder).resolve()
        path = (frozen_root / relative).resolve()
        assert frozen_root in path.parents
        content = path.read_bytes()
        sha = hashlib.sha256(content).hexdigest()
        eof = len(content.decode("utf-8-sig").splitlines())
        ranges = row.get("ranges", row.get("read_ranges", row.get("material_ranges")))
        assert ranges, row
        effective = qualified_ranges.get(row["id"], {}).get("ranges", ranges)
        declared_eof = row.get("actual_EOF", row.get("actual_eof", row.get("actual_line_count")))
        assert row["sha256"] == sha, (sid, "registration SHA mismatch")
        assert declared_eof == eof, (sid, "registration EOF mismatch", declared_eof, eof)
        assert all(1 <= lo <= hi <= eof for lo, hi in ranges), sid
        assert all(any(a <= lo <= hi <= b for a, b in ranges) for lo, hi in effective), (sid, "qualification expanded original scope")
        if sid in known:
            assert known[sid]["sha256"] == sha, (sid, "baseline identity mismatch")
        observation = {
            "registry": paths[supplier],
            "row_1based": line,
            "supplier": supplier,
            "source_id": sid,
            "sha256": sha,
            "actual_EOF": eof,
            "ranges": ranges,
            "effective_material_ranges": effective,
            "qualification": qualified_ranges.get(row["id"], {}).get("qualification"),
            "original_registration": row,
            "frozen_path": path.as_posix(),
            "bytes": len(content),
        }
        observations.append(observation)
        groups.setdefault(sid, []).append(observation)
end_changes = [name for name, path in paths.items() if hashlib.sha256((root / path).read_bytes()).hexdigest() != pins[name]["sha256"]]
new_ids = sorted(set(groups) - set(known))
result = {
    "id": "CONT-ROOT-OWNER81-REGISTRY-CANDIDATE-" + args.label,
    "at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "mode": "metadata-only exact prospective owner registrations; no body semantics or coverage",
    "input_pins": pins,
    "all_original_registrations": observations,
    "all_original_registry_rows_including_qualifications": raw_registry_rows,
    "local_qualifications": qualifications,
    "source_groups": [{"source_id": sid, "already_global": sid in known, "prior_source_scope": known.get(sid), "registration_observations": records} for sid, records in sorted(groups.items())],
    "new_source_ids": new_ids,
    "counts": {"registration_rows": len(observations), "qualification_rows": len(qualifications), "source_variants": len(groups), "new_variants": len(new_ids), "existing_variants_with_exact_new_roles_or_support": len(groups) - len(new_ids), "proposed_total_variants": len(known) + len(new_ids)},
    "changed_inputs_during_snapshot": end_changes,
    "issues": issues,
    "global_scope_mutations": 0,
    "source_edits": 0,
    "semantic_credit": False,
    "next": "Root must review exact actual new roles, exclusions and existing material scope before global reconciliation. These current-owner source proofs must join a bounded producer supplement and owner fixedpoint before admission03. Re-run fresh label after all registries final; preserve this candidate.",
}
destination.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"candidate": str(destination), "counts": result["counts"], "changed_inputs": end_changes, "semantic_credit": False}))
