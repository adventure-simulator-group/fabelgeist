"""One-shot append-only source/range bookkeeping. Awards no audit coverage."""
import copy
import datetime
import hashlib
import json
import pathlib
import re

base = pathlib.Path(__file__).resolve().parents[2]
cont = base / "continuation"
out = cont / "cumulative-source-scope-admission03-013.json"
assert not out.exists(), "One-shot reconciliation already applied"
now = datetime.datetime.now(datetime.timezone.utc).isoformat()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def pin(path):
    return {"record": path.relative_to(base).as_posix(), "sha256": sha(path.read_bytes())}

def merge(ranges):
    result = []
    for start, end in sorted(set(map(tuple, ranges))):
        if result and start <= result[-1][1] + 1:
            result[-1][1] = max(end, result[-1][1])
        else:
            result.append([start, end])
    return result

previous_path = cont / "cumulative-source-scope-admission03-012.json"
scope = json.loads(previous_path.read_text(encoding="utf-8"))
by_id = {row["source_id"]: row for row in scope["files"]}
assert len(by_id) == 2595
locators = {row["source_id"]: row["resolved_path"] for row in json.loads((cont / "source-locators-2559-verified.json").read_text(encoding="utf-8"))["matched"]}
overlays_path = cont / "owner-range-extensions.jsonl"
overlays = [json.loads(line) for line in overlays_path.read_text(encoding="utf-8").splitlines()]
assert len(overlays) == 81
cuts = {"root": 18, "rig": 35, "native": 9}
caps = {"root": 22, "rig": 138, "native": 57}
inputs = []
observations = []
qualifier_observations = []
for supplier, cutoff in cuts.items():
    folder = cont / supplier / "admission03"
    registry_path = folder / "source-range-registration.jsonl"
    registry_bytes = registry_path.read_bytes()
    registry_all = [json.loads(line) for line in registry_bytes.decode("utf-8").splitlines()]
    registry = registry_all[:caps[supplier]]
    inputs.append({"supplier": supplier, **pin(registry_path), "captured_rows": len(registry_all), "selected_prefix_rows": len(registry), "previously_reconciled_rows": cutoff})
    qualifier_path = folder / ("source-qualifications-final-082.jsonl" if supplier == "native" else "source-qualifications.jsonl")
    qualifiers = []
    if qualifier_path.exists():
        qualifier_bytes = qualifier_path.read_bytes()
        qualifiers = [json.loads(line) for line in qualifier_bytes.decode("utf-8").splitlines()]
        inputs.append({"supplier": supplier, **pin(qualifier_path), "captured_rows": len(qualifiers), "kind": "role_qualifications"})
    qualified = {}
    for note in qualifiers:
        registration_id = note.get("registration_id", note.get("registration_ref", note.get("registration")))
        if not registration_id and supplier == "rig":
            match = re.search(r"REG(\d{3})", note.get("id", ""))
            if match:
                registration_id = "RIG-ADMISSION03-REGISTER-" + match[1]
        if registration_id:
            suffix = re.search(r"(\d{3})$", registration_id)
            if suffix:
                qualified.setdefault(int(suffix[1]), []).append(note)
        qualifier_observations.append({"supplier": supplier, "registry": qualifier_path.relative_to(base).as_posix(), "original_record": note, "registration_id_resolved": registration_id})
    for row_number, registration in enumerate(registry, 1):
        notes = qualified.get(int(re.search(r"(\d{3})$", registration["id"])[1]), [])
        ranges = registration.get("material_ranges", registration.get("ranges"))
        for note in notes:
            for key in ("effective_material_ranges", "qualified_material_ranges", "actual_material_body_ranges", "actual_ranges", "material_ranges"):
                if key in note:
                    ranges = note[key]
                    break
        path = pathlib.Path(registration.get("resolved_path", registration.get("path", locators.get(registration["source_id"], by_id.get(registration["source_id"], {}).get("path", "")))))
        assert path.is_absolute() and path.is_file(), str(path)
        data = path.read_bytes()
        assert sha(data) == registration["sha256"], registration["id"]
        eof = len(data.decode("utf-8").splitlines())
        assert all(1 <= start <= end <= eof for start, end in ranges), (registration["id"], ranges, eof)
        observation = {"supplier": supplier, "registry": registry_path.relative_to(base).as_posix(), "row_1based": row_number, "registration_id": registration["id"], "original_record": registration, "local_qualifications": notes, "effective_material_ranges": merge(ranges), "resolved_path": str(path.resolve()), "actual_EOF": eof, "bytes": len(data)}
        if row_number > cutoff:
            observations.append(observation)
        elif notes:
            qualifier_observations.append({"supplier": supplier, "registration_id": registration["id"], "existing_previously_reconciled_role_additional_qualification": observation, "limits": "Preserve all prior original scoped roles; qualifies only this prospective new admission role, no global source narrowing."})

grouped = {}
for observation in observations:
    grouped.setdefault(observation["original_record"]["source_id"], []).append(observation)
new_ids = []
new_overlay_ids = []
for source_id, source_observations in grouped.items():
    registration = source_observations[0]["original_record"]
    source_sha = registration["sha256"]
    assert all(item["original_record"]["sha256"] == source_sha for item in source_observations)
    material_ranges = merge([part for item in source_observations for part in item["effective_material_ranges"]])
    supplier = source_observations[0]["supplier"]
    if source_id in by_id:
        source = by_id[source_id]
        assert source["sha256"] == source_sha
        overlay_id = f"CONT-OWNER-RANGE-EXTENSION-{len(overlays)+1:03}"
        overlays.append({"id": overlay_id, "at": now, "source_id": source_id, "sha256": source_sha, "audit_owner": source.get("audit_owner", supplier), "material_ranges": material_ranges, "admission03_scope_observations": source_observations, "scope_limits": "Exact qualified current03 direct-definition/native-format roles only; preserve original scope. No metadata semantic credit or ownership transfer.", "producer_inventory_accounting": "bounded_new_material_role_qualification_pending_after_initial_admission03", "admission_discovery": "source_backed_packet_acceptance_pending"})
        new_overlay_ids.append(overlay_id)
        source.setdefault("admission03_increment050_role_observations", []).extend(source_observations)
    else:
        first = source_observations[0]
        source = {"source_id": source_id, "snapshot": registration.get("snapshot", "pinned-native-dependency"), "path": first["resolved_path"], "sha256": source_sha, "bytes": first["bytes"], "actual_line_count": first["actual_EOF"], "scope_kind": "materially_reached_admission_contract", "audit_owner": supplier, "material_ranges": material_ranges, "admission03_scope_observations": source_observations, "scope_limits": "Only exact qualified directly reached definitions/native decoder routes; no implied whole-file/SDK or other-lens coverage.", "producer_inventory_accounting": "bounded_new_material_role_qualification_pending_after_initial_admission03", "admission_discovery": "source_backed_packet_acceptance_pending"}
        scope["files"].append(source)
        by_id[source_id] = source
        new_ids.append(source_id)

scope["previous_pinned_scope"] = pin(previous_path)
scope["generated_at"] = now
scope["admission03_scope_increment050"] = {"at": now, "input_pins": inputs, "new_registrations": len(observations), "new_source_ids": new_ids, "new_overlay_ids": new_overlay_ids, "all_local_role_qualifications": qualifier_observations, "limits": "Pure source/hash/range reconciliation; no semantic coverage. 042 exact2560/59 initial01/02 acceptance remains historical; all newly material roles need bounded qualification after current03 discovery. Initial03-13 and all repairs remain incomplete."}
out.write_text(json.dumps(scope, indent=2) + "\n", encoding="utf-8")
overlays_path.write_text("".join(json.dumps(row) + "\n" for row in overlays), encoding="utf-8")
record_path = cont / "root/admission03-scope-reconciliation-050.json"
record = {"id": "CONT-ROOT-ADMISSION03-SCOPE-RECONCILIATION-050", "at": now, "previous_scope": pin(previous_path), "scope": pin(out), "mandatory_overlays": pin(overlays_path), "total_variants": len(scope["files"]), "total_overlays": len(overlays), "new_registrations": len(observations), "new_source_ids": new_ids, "new_overlay_ids": new_overlay_ids, "input_pins": inputs, "role_qualifications": qualifier_observations, "limits": scope["admission03_scope_increment050"]["limits"], "source_edits": 0, "project_execution": 0}
record_path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
scope_ref = {**pin(out), "variants": len(scope["files"]), "range_extensions": {**pin(overlays_path), "mandatory_overlay": True, "through": len(overlays)}, "registry_reconciliation": pin(record_path), "limits": record["limits"]}
audit_path = base / "records/final-acceptance-20261021/audit-ledger.json"
audit = json.loads(audit_path.read_text(encoding="utf-8"))
audit["updated_at"] = now
audit["current_cumulative_scope"] = scope_ref
audit["discovery_gate"]["new_admission03_material_scope050"] = pin(record_path)
for lens in audit["lenses"]:
    existing_cells = {cell["source_id"] for cell in lens["cells"]}
    for source_id in new_ids:
        assert source_id not in existing_cells
        lens["cells"].append({"source_id": source_id, "owner": by_id[source_id]["audit_owner"], "discovery": "incomplete", "source_read": "not_credited_from_scope_metadata", "evidence": []})
    lens["scoped_variants"] = len(scope["files"])
    lens["scope_ref"] = copy.deepcopy(scope_ref)
audit_path.write_text(json.dumps(audit, indent=2) + "\n", encoding="utf-8")
execution_path = base / "records/execution-ledger.json"
execution = json.loads(execution_path.read_text(encoding="utf-8"))
execution["updated_at"] = now
execution.setdefault("continuation", {})["admission03_scope050"] = pin(record_path)
execution["acceptance_gate"]["audit_inventory_progress"]["scope_limit"] = record["limits"]
execution_path.write_text(json.dumps(execution, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"scope": pin(out), "variants": len(scope["files"]), "overlays": len(overlays), "new_registrations": len(observations), "new_variants": len(new_ids), "new_existing_source_overlays": len(new_overlay_ids), "semantic_coverage_awarded": False}, indent=2))
