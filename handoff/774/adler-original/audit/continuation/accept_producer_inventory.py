"""Commit reviewed receipt accounting to ignored ledgers; no project execution."""

from pathlib import Path
import collections
import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent.parent
AUDIT = ROOT / "records/final-acceptance-20261021/audit-ledger.json"
EXECUTION = ROOT / "records/execution-ledger.json"
SCOPE = ROOT / "continuation/cumulative-source-scope-pending.json"
FINAL = ROOT / "continuation/cumulative-source-scope-after-producer.json"


def read(relative):
    return json.loads((ROOT / relative).read_text())


def receipt(relative):
    path = ROOT / relative
    return {"record": relative, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


scope = json.loads(SCOPE.read_text())
audit = json.loads(AUDIT.read_text())
by_id = {entry["source_id"]: entry for entry in scope["files"]}
original = {cell["source_id"]: cell for cell in audit["lenses"][0]["cells"]}
supplement = {item["source_id"]: index for index, item in enumerate(scope["producer_supplement_obligations"])}
frozen = {entry["source_id"] for entry in read("records/final-acceptance-20261021/wave-01/source-scope-after-owner.json")["files"]}
assert set(original) | set(supplement) == frozen
assert len(by_id) == 2473

native = read("continuation/native/expanded-producer-role-delta-002.json")
camera = read("continuation/camera/expanded-producer-role-delta-013.json")
rig = read("continuation/rig/expanded-producer-role-delta-051.json")
root = read("continuation/root/expanded-material-qualifiers-004.json")
groups = [
    ("native", native["per_observation"], "delta_observation_index"),
    ("camera", camera["per_row"], "delta_index"),
    ("rig", rig["observation_dispositions"], "delta_observation_index"),
    ("root", root["dispositions"], "delta_observation_index"),
]
delta_owners = {}
for owner, rows, key in groups:
    for row in rows:
        ordinal = row[key]
        assert ordinal not in delta_owners
        delta_owners[ordinal] = owner
assert set(delta_owners) == set(range(187))
assert not native["remaining_actual_material_receivers"]
assert not camera["remaining_exact_material_obligations"]
assert not rig["remaining_actual_unread"]
assert not root["genuine_unresolved_material_obligations"]
new_roles = read("continuation/native/new-variant-producer-role-reconciliation.json")
assert not new_roles["genuine_unresolved_actual_receivers"]
new158 = {item["source_id"] for item in new_roles["source_dispositions"]}
expanded = read("continuation/rig/expanded-producer-delta-map-049.json")
new36 = {item["canonical_source_id_candidate"] for item in expanded["delta_source_identity_map"] if item["new_identity"]}
assert len(new158) == 158 and len(new36) == 36 and not new158 & new36
assert new158 | new36 == set(by_id) - frozen

evidence_names = [
    "continuation/camera/camera716-final-receipt.json",
    "continuation/root/pure8-packet.json",
    "continuation/rig/web-request-reconciliation-039.json",
    "continuation/rig/range-schema-errata-043.json",
    "continuation/native/world5-final-packet.json",
    "continuation/rig/source-scope-reconciliation-044.json",
    "continuation/rig/source-role-qualification-045.json",
    "continuation/rig/source-role-qualification-046-refresh-delta.json",
    "continuation/camera/reached-role-reconciliation/packet.json",
    "continuation/camera/reached-role-reconciliation/packet-addendum-011.json",
    "continuation/native/new-variant-producer-role-reconciliation.json",
    "continuation/root/door-grab-producer-002.json",
    "continuation/root/foreign-alias-producer-003.json",
    "continuation/rig/owner-delta-preparation-correction-048.json",
    "continuation/rig/expanded-producer-delta-map-049.json",
    "continuation/rig/expanded-producer-delta-candidates-050.json",
    "continuation/camera/reached-role-reconciliation/eof-role-qualification-012.json",
    "continuation/camera/expanded-producer-role-delta-013.json",
    "continuation/camera/expanded-producer-role-reference-errata-014.json",
    "continuation/native/expanded-producer-role-delta-002.json",
    "continuation/rig/expanded-producer-role-delta-051.json",
    "continuation/rig/expanded-producer-role-delta-reference-errata-052.json",
    "continuation/root/expanded-material-qualifiers-004.json",
]
evidence = [receipt(name) for name in evidence_names]


def attach_source_proof(relative, pointer, proof, identifier=None):
    identifier = identifier or proof["source_id"]
    identifier = scope["identity_aliases"].get(identifier, identifier)
    if identifier not in by_id:
        raise ValueError(f"Actual supporting source needs scope extension: {identifier}")
    entry = by_id[identifier]
    digest = proof.get("sha256", proof.get("observed_sha256"))
    assert entry["sha256"] == digest, identifier
    ranges = proof.get("read_ranges", proof.get("fresh_read_ranges"))
    assert ranges, (relative, pointer)
    entry.setdefault("continuation_actual_producer_source_proofs", []).append({
        "record": relative, "json_pointer": pointer,
        "source_id": identifier, "sha256": digest, "read_ranges": ranges,
        "proof": proof,
        "limits": "Actual source-role proof for producer lens only; no owner-fit or other-lens clearance.",
    })


for name, key in [
    ("continuation/rig/source-role-qualification-045.json", "fresh_source_proofs"),
    ("continuation/native/new-variant-producer-role-reconciliation.json", "fresh_bounded_source_proofs"),
    ("continuation/root/door-grab-producer-002.json", "source_proofs"),
    ("continuation/root/foreign-alias-producer-003.json", "source_proofs"),
    ("continuation/root/expanded-material-qualifiers-004.json", "fresh_source_proofs"),
    ("continuation/camera/reached-role-reconciliation/eof-role-qualification-012.json", "fresh_source_and_immediate_receiver_proofs"),
    ("continuation/rig/expanded-producer-role-delta-051.json", "fresh_source_proofs"),
]:
    for index, proof in enumerate(read(name)[key]):
        attach_source_proof(name, f"/{key}/{index}", proof)
for index, row in enumerate(native["per_observation"]):
    if row["fresh_source_actual_read_count"]:
        attach_source_proof("continuation/native/expanded-producer-role-delta-002.json", f"/per_observation/{index}/fresh_actual_source_read", row["fresh_actual_source_read"], row["source_id"])

now = datetime.datetime.now(datetime.timezone.utc).isoformat()
for entry in scope["files"]:
    identifier = entry["source_id"]
    entry["producer_inventory_accounting"] = {
        "initial946_cell": identifier in original,
        "frozen1650_supplement_index": supplement.get(identifier),
        "new158_material_role_packet": identifier in new158,
        "expanded36_context_variant": identifier in new36,
        "status": "accounted_for_exact_current_scoped_contracts_with_findings_preserved",
        "authority": "Combined original producer inventory, completed frozen supplier queues and personally reconciled exact role packets in producer_inventory_evidence. No ownership or later-lens credit.",
    }
    for key in ("producer_reached_contract_observations", "expanded_producer_contract_observations"):
        for observation in entry.get(key, []):
            observation["status"] = "producer_role_accounted_with_findings; owner-fit_delta_pending"
    if any(entry.get(key) for key in ("producer_reached_contract_observations", "expanded_producer_contract_observations", "continuation_actual_producer_source_proofs")):
        entry["owner_fit_delta_status"] = "incomplete_explicit_contract_comparison_required; prior_frozen_role_proofs_preserved"
scope["status"] = "producer_initial_inventory_exhausted_with_actionable_findings; owner_fit_delta_pending"
scope["generated_at"] = now
scope["producer_inventory_evidence"] = evidence
scope["expanded_path_schema_delta"]["status"] = "all187_exact_delta_observations_accounted_with_findings"
scope["unqualified_metadata"]["status"] = "all246_metadata_observations_explicitly_qualified; no inferred reads"
scope["producer_scope_limits"] = "Original946 changed/preserved variants retain full requested scope; reached unchanged/foreign contracts retain only explicit material roles, qualified ranges and necessary dependencies. Supplementary narrower material qualifiers never reduce original fullscope. All existing findings remain open."
FINAL.write_text(json.dumps(scope, indent=2) + "\n")
final_hash = hashlib.sha256(FINAL.read_bytes()).hexdigest()

new_entries = [entry for entry in scope["files"] if entry["source_id"] not in frozen]
for lens in audit["lenses"][1:]:
    present = {cell["source_id"] for cell in lens["cells"]}
    for entry in new_entries:
        assert entry["source_id"] not in present
        lens["cells"].append({"source_id": entry["source_id"], "owner": entry["audit_owner"], "discovery": "incomplete", "source_read": "unread_for_this_lens", "evidence": []})
    lens["scoped_variants"] = len(by_id)
    if lens["lens"] == "audit-existing-type-owners":
        lens["discovery"] = "incomplete_explicit_producer_reached_owner_fit_delta; frozen2279_exact_role_proofs_preserved"
        for cell in lens["cells"]:
            if by_id[cell["source_id"]].get("owner_fit_delta_status"):
                cell["new_producer_role_delta"] = "incomplete; no same-file/hash/range-only semantic carry"
producer = audit["lenses"][0]
producer["discovery"] = "initial_inventory_exhausted_with_actionable_findings"
producer["scoped_variants"] = len(by_id)
producer["current_scope_accounting"] = {"record": "../../continuation/cumulative-source-scope-after-producer.json", "sha256": final_hash, "variants": len(by_id), "original_cells": 946, "frozen_supplement": 1650, "base_union": 2279, "new_variants": 194, "expanded_delta_rows": 187, "evidence": evidence, "clearance": "not_clear; findings_open; no repair or validation"}
audit["updated_at"] = now
audit["current_cumulative_scope"] = {"record": "../../continuation/cumulative-source-scope-after-producer.json", "sha256": final_hash, "variants": len(by_id), "limits": scope["producer_scope_limits"]}
audit["current_lens"] = "audit-existing-type-owners"
audit["first_unfinished_cell"] = "Existing-owner fit delta on explicit newly reached material contracts in cumulative2473 scope; no source repair until initial lenses03-13 also exhausted."
audit["current_inventory_step"] = "existing_owner_fit_delta_after_exhaustive_producer_scope_reconciliation"
audit["continuation"]["producer_inventory_completion"] = producer["current_scope_accounting"]
audit["discovery_gate"]["status"] = "incomplete_owner_delta_and_initial_lenses03_13"
audit["discovery_gate"]["repairs_allowed"] = False
AUDIT.write_text(json.dumps(audit, indent=2) + "\n")

execution = json.loads(EXECUTION.read_text())
execution["updated_at"] = now
execution["next_action"] = "Perform explicit existing-owner fit delta on cumulative2473 source variants, preserving exact earlier role proofs. Then complete initial lenses03-13 sequentially before any source repair. Producer inventory is exhausted with actionable findings, not clear. Afterwards integrate mandatory P16/P17 and all connected/reopened repairs, complete full audit waves, independent Code/Rule/Comment reviews and hosted validation on one final pushed revision. Preserve primary GPU work separately; do not merge PR774 or close726."
execution.setdefault("continuation", {})["producer_inventory_completion"] = {"scope": "../continuation/cumulative-source-scope-after-producer.json", "sha256": final_hash, "variants": len(by_id), "discovery": producer["discovery"], "repairs_allowed": False, "validation": "not established"}
EXECUTION.write_text(json.dumps(execution, indent=2) + "\n")

queues = collections.defaultdict(list)
for entry in scope["files"]:
    if entry.get("owner_fit_delta_status"):
        queues[entry["audit_owner"]].append({"source_id": entry["source_id"], "sha256": entry["sha256"], "scope_kind": entry["scope_kind"], "new_variant": entry["source_id"] not in frozen, "exact_scope_record": "continuation/cumulative-source-scope-after-producer.json", "prior_owner_proofs": "Preserved owner_contract_proofs and original owner coverage; exact semantic fit must be compared, not assumed."})
for owner, sources in queues.items():
    (ROOT / f"continuation/{owner}/owner-fit-delta-queue.json").write_text(json.dumps({"lens": "audit-existing-type-owners", "status": "authorized read-only initial owner-fit delta; source freeze remains", "source_scope_sha256": final_hash, "sources": sources}, indent=2) + "\n")
print(json.dumps({"producer_discovery": producer["discovery"], "scope_variants": len(by_id), "new_variants": len(new_entries), "owner_delta_sources": {owner: len(rows) for owner, rows in queues.items()}, "current_lens": audit["current_lens"], "source_edits_or_project_commands": 0}))
