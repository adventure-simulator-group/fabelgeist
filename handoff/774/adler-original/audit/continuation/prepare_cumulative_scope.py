"""Reconcile explicit receipt metadata; no project scan or audit credit."""

from pathlib import Path
import collections
import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent.parent
OLD = ROOT / "records/final-acceptance-20261021/wave-01/source-scope-after-owner.json"
INDEX = ROOT / "continuation/producer-reached-role-index.json"
MAP = ROOT / "continuation/rig/source-scope-reconciliation-044.json"
EXPANDED = ROOT / "continuation/rig/expanded-producer-delta-map-049.json"
scope = json.loads(OLD.read_text())
index = json.loads(INDEX.read_text())
reconciliation = json.loads(MAP.read_text())
expanded = json.loads(EXPANDED.read_text())
by_id = {entry["source_id"]: entry for entry in scope["files"]}
aliases = {}


def assigned_owner(identifier, observations):
    """Administrative source assignment only, never repair ownership."""
    path = identifier.split(":", 1)[1]
    if "/adventuresim-building-generator/" in path:
        return "root"
    if "/adventuresim-weapon-model/" in path:
        return "camera"
    if identifier.startswith(("native-dependency:", "pinned-")):
        return "native"
    suppliers = []
    for observation in observations:
        record = observation["record"]
        supplier = record.split("/")[-2]
        if supplier in ("root", "camera", "rig", "native"):
            suppliers.append(supplier)
    return collections.Counter(suppliers).most_common(1)[0][0]


for variant in reconciliation["new_variant_identity_and_range_map"]:
    identifier = variant["source_id"]
    if variant["classification"] == "metadata-normalization-package-alias":
        target = variant["frozen_alias_candidates"][0]["source_id"]
        aliases[identifier] = target
        continue
    locator = variant["resolved_candidate"]
    by_id[identifier] = {
        "source_id": identifier,
        "snapshot": identifier.split(":", 1)[0],
        "path": str(Path(locator["path"])),
        "sha256": variant["recorded_sha256"],
        "scope_kind": "materially_reached_producer_contract",
        "audit_owner": assigned_owner(identifier, variant["observations"]),
        "audit_assignment_limits": "Read-only audit-source assignment; authoritative whole-function repair claims remain separate. No transfer of code ownership.",
        "scope_limits": "Only explicit materially reached contract ranges and their necessary signature/recipient dependencies. No implied full-file or other-lens coverage.",
        "locator_proof": "continuation/rig/source-scope-reconciliation-044.json",
        "actual_line_count": locator["actual_line_count"],
    }

for observation in index["explicit_source_range_observations"]:
    identifier = aliases.get(observation["source_id"], observation["source_id"])
    entry = by_id[identifier]
    if observation["sha256"] != entry["sha256"]:
        raise ValueError(f"Unreconciled fingerprint: {identifier}")
    proof = {
        "record": observation["record"],
        "receipt": observation["receipt"],
        "json_pointer": observation["json_pointer"],
        "source_id_as_recorded": observation["source_id"],
        "read_ranges_as_recorded": observation["read_ranges"],
        "source_context": observation["source_context"],
        "status": "producer_material_role_reconciliation_pending; no owner-fit or later-lens credit",
    }
    entry.setdefault("producer_reached_contract_observations", []).append(proof)

for identity in expanded["delta_source_identity_map"]:
    identifier = identity["canonical_source_id_candidate"]
    if not identity["new_identity"]:
        continue
    if identifier in by_id:
        raise ValueError(f"Unreconciled expanded identity collision: {identifier}")
    locator = identity["locator_metadata"]
    observations = [expanded["delta_observation_map"][i] for i in identity["delta_observation_indices"]]
    by_id[identifier] = {
        "source_id": identifier,
        "snapshot": identifier.split(":", 1)[0],
        "path": locator["resolved_path"],
        "sha256": identity["sha256"],
        "scope_kind": "materially_reached_producer_contract",
        "audit_owner": "native" if identifier.startswith("native-dependency:") else "camera",
        "audit_assignment_limits": "Read-only source assignment; whole-function repair ownership remains in authoritative claims.",
        "scope_limits": "Explicit material contracts/ranges and necessary signature dependencies only; no implied whole-file or other-lens coverage.",
        "locator_proof": "continuation/rig/expanded-producer-delta-map-049.json",
        "actual_line_count": locator["actual_EOF"],
    }

for ordinal, observation in enumerate(expanded["delta_observation_map"]):
    identifiers = observation["current_scope_source_ids"] or [observation["canonical_source_id_candidate"]]
    for identifier in identifiers:
        entry = by_id[identifier]
        if entry["sha256"] != observation["sha256"]:
            raise ValueError(f"Unreconciled expanded fingerprint: {identifier}")
        entry.setdefault("expanded_producer_contract_observations", []).append({
            "delta_observation_index": ordinal,
            "record": observation["record"],
            "receipt": observation["parent_trace"],
            "json_pointer": observation["json_pointer"],
            "expanded_ref": observation["expanded_ref"],
            "read_ranges_as_recorded": observation["recorded_read_ranges"],
            "material_contract_ranges_as_recorded": observation["material_contract_ranges"],
            "scope_qualification": "Explicit material qualifier limits this supplementary role; never reduces original requested changed-file fullscope.",
            "role_as_recorded": observation["role"],
            "read_status_as_recorded": observation["read_status"],
            "status": "producer_material_role_delta_pending; no owner-fit or later-lens credit",
        })

scope["generated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
scope["status"] = "pending_producer_role_qualification_then_owner_fit_delta; no clearance"
scope["authority"] = "Cumulative explicit scope proposal; audit-ledger remains sole discovery/repair/validation authority. Immutable historical export untouched."
scope["files"] = sorted(by_id.values(), key=lambda entry: entry["source_id"])
scope["previous_scope"] = {"record": str(OLD.relative_to(ROOT)), "sha256": hashlib.sha256(OLD.read_bytes()).hexdigest(), "variants": 2279}
scope["producer_receipt_index"] = {"record": str(INDEX.relative_to(ROOT)), "sha256": hashlib.sha256(INDEX.read_bytes()).hexdigest()}
scope["identity_aliases"] = aliases
scope["expanded_path_schema_delta"] = {"record": str(EXPANDED.relative_to(ROOT)), "sha256": hashlib.sha256(EXPANDED.read_bytes()).hexdigest(), "observations": 187, "new_context_variants": 36, "status": "actual producer role qualification pending"}
scope["unqualified_metadata"] = {"record": str(INDEX.relative_to(ROOT)), "field": "incomplete_metadata_observations", "count": len(index["incomplete_metadata_observations"]), "status": "explicit qualifications pending; not coverage"}
destination = ROOT / "continuation/cumulative-source-scope-pending.json"
destination.write_text(json.dumps(scope, indent=2) + "\n")
print(json.dumps({"variants": len(by_id), "new_variants": len(by_id)-2279, "new_source_assignments": dict(collections.Counter(x["audit_owner"] for x in scope["files"] if x["scope_kind"] == "materially_reached_producer_contract")), "variants_with_role_observations": sum(bool(x.get("producer_reached_contract_observations")) for x in scope["files"]), "status": scope["status"]}))
