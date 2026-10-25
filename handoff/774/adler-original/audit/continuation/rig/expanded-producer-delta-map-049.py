"""Map only expanded receipt-schema deltas; never award source audit coverage."""
from collections import Counter, defaultdict
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[1]
WAVE = ROOT / "records/final-acceptance-20261021/wave-01"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


expanded_path = WAVE / "producer-role-receipts-expanded-pending.jsonl"
expanded_observation_path = WAVE / "producer-role-receipts-expanded-observation.json"
verification_path = WAVE / "producer-role-receipt-verification-pending.json"
current_path = ROOT / "continuation/producer-reached-role-index.json"
scope_path = ROOT / "continuation/cumulative-source-scope-pending.json"
expanded = [json.loads(line) for line in expanded_path.read_text(encoding="utf-8").splitlines() if line.strip()]
current = read(current_path)
scope = read(scope_path)
context = read(ROOT / "continuation-context.json")
locators = read(ROOT / "continuation/source-locators.json")


def normalize_path(path, source=None):
    path = path.replace("\\", "/")
    for variant, base in context["snapshots"].items():
        base = base.replace("\\", "/").rstrip("/")
        if path.casefold().startswith(base.casefold() + "/"):
            return "SNAPSHOTS/" + variant + "/" + path[len(base) + 1:]
    if "/registry/src/" in path:
        return "CARGO_REGISTRY/" + path.split("/registry/src/", 1)[1].split("/", 1)[1]
    if path.startswith(("SNAPSHOTS/", "CARGO_REGISTRY/")):
        return path
    if source and source.partition(":")[0] in context["snapshots"]:
        return "SNAPSHOTS/" + source.partition(":")[0] + "/" + source.partition(":")[2]
    return path


scope_by_id = {r["source_id"]: r for r in scope["files"]}
scope_by_path_hash = defaultdict(list)
for row in scope["files"]:
    scope_by_path_hash[(normalize_path(row["path"], row["source_id"]), row.get("sha256"))].append(row["source_id"])

registry_roots = set()
for row in locators["matched"]:
    path = row["resolved_path"].replace("\\", "/")
    if "/registry/src/" in path:
        head, tail = path.split("/registry/src/", 1)
        registry_roots.add(Path(head + "/registry/src/" + tail.split("/")[0]))


def resolve(path, digest):
    if path.startswith("SNAPSHOTS/"):
        _, variant, rel = path.split("/", 2)
        candidates = [Path(context["snapshots"][variant]) / rel]
    elif path.startswith("CARGO_REGISTRY/"):
        candidates = [base / path[len("CARGO_REGISTRY/"):] for base in registry_roots]
    else:
        candidates = []
    for candidate in candidates:
        if candidate.is_file():
            actual = sha(candidate)
            if actual == digest:
                return {"resolved_path": str(candidate), "sha256_match": True, "actual_EOF": len(candidate.read_bytes().splitlines())}
    return {"candidate_paths": [str(p) for p in candidates], "sha256_match": False}


def record_path(path):
    return "records/final-acceptance-20261021/" + path if path.startswith("wave-01/") else path


def proof_key(record, trace, pointer):
    return record_path(record), trace, pointer


def ranges(value):
    if value and len(value) == 2 and all(isinstance(x, int) for x in value):
        value = [value]
    return sorted(value or [])


qualified_by_ref = defaultdict(list)
unqualified_by_ref = defaultdict(list)
for row in current["explicit_source_range_observations"]:
    qualified_by_ref[proof_key(row["record"], row["receipt"], row["json_pointer"])].append(row)
for row in current["incomplete_metadata_observations"]:
    unqualified_by_ref[proof_key(row["record"], row["receipt"], row["json_pointer"])].append(row)


def observation_path(row):
    source = row["source_id"]
    if source in scope_by_id:
        return normalize_path(scope_by_id[source]["path"], source)
    for key in ("resolved_path", "path"):
        value = row["source_context"].get(key)
        if isinstance(value, str):
            return normalize_path(value, source)
    prefix, _, suffix = source.partition(":")
    if prefix in context["snapshots"]:
        return "SNAPSHOTS/" + prefix + "/" + suffix
    if prefix in ("native-dependency", "pinned-registry", "foreign"):
        return "CARGO_REGISTRY/" + suffix
    if prefix.startswith("pinned-"):
        return "CARGO_REGISTRY/" + prefix[7:] + "/" + suffix
    return source


def canonical_id(path, current_ids, aliases):
    if current_ids:
        return current_ids[0]
    if path.startswith("SNAPSHOTS/"):
        _, variant, rel = path.split("/", 2)
        return variant + ":" + rel
    if path.startswith("CARGO_REGISTRY/"):
        return "native-dependency:" + path[len("CARGO_REGISTRY/"):]
    return aliases[0] if aliases else path


delta = []
accounted = 0
already_unqualified = 0
all_expanded_refs = 0
identity_rows = []
for ordinal, entry in enumerate(expanded):
    path, digest = entry["path"], entry["sha256"]
    current_ids = scope_by_path_hash[(path, digest)]
    source_id = canonical_id(path, current_ids, entry["source_id_aliases"])
    source_delta_rows = []
    for proof_ordinal, proof in enumerate(entry["proofs"]):
        all_expanded_refs += 1
        key = proof_key(proof["record"], proof["trace"], proof["json_pointer"])
        named = qualified_by_ref[key]
        previously_unqualified = unqualified_by_ref[key]
        equivalent = [row for row in named if observation_path(row) == path and row["sha256"] == digest]
        recorded = ranges(proof["ranges"])
        material = ranges(proof["material_contract_ranges"])
        classification = None
        if equivalent:
            same_range = any(ranges(row["read_ranges"]) == recorded for row in equivalent)
            # The reconstructed index preserved the object, but cumulative scope used
            # read ranges rather than a distinct material-contract range qualifier.
            if material != recorded:
                classification = "existing-observation-distinct-material-contract-range-qualifier"
            elif not same_range:
                classification = "existing-observation-expanded-range-shape-difference"
            else:
                accounted += 1
                continue
        elif previously_unqualified:
            already_unqualified += 1
            # 045/046 explicitly qualify these metadata observations; preserve only
            # genuinely additional path/range information, not duplicate work.
            classification = "existing-unqualified-ref-expanded-path-range-metadata"
        elif named:
            classification = "existing-ref-context-or-path-alias-needs-identity-reconciliation"
        else:
            classification = "additional-path-based-producer-role-observation"
        row = {
            "expanded_ref": f"producer-role-receipts-expanded-pending.jsonl#row{ordinal + 1}/proofs/{proof_ordinal}",
            "canonical_source_id_candidate": source_id, "path": path, "sha256": digest,
            "parent_trace": proof["trace"], "record": record_path(proof["record"]),
            "receipt_row": proof["row"], "json_pointer": proof["json_pointer"],
            "recorded_read_ranges": proof["ranges"], "material_contract_ranges": proof["material_contract_ranges"],
            "role": proof["role"], "read_status": proof["read_status"],
            "classification": classification,
            "current_scope_source_ids": current_ids,
            "identity_status": "existing-cumulative-path-and-fingerprint" if current_ids else "new-path-and-fingerprint-outside2437",
            "named_observation_refs": [{"source_id": r["source_id"], "sha256": r["sha256"], "ranges": r["read_ranges"]} for r in named],
            "previously_unqualified_refs": [{"record": r["record"], "receipt": r["receipt"], "json_pointer": r["json_pointer"]} for r in previously_unqualified],
            "qualification_status": "metadata-delta-only; actual role review not started",
        }
        delta.append(row)
        source_delta_rows.append(len(delta) - 1)
    if source_delta_rows or not current_ids:
        identity_rows.append({
            "canonical_source_id_candidate": source_id, "path": path, "sha256": digest,
            "source_id_aliases": entry["source_id_aliases"], "frozen_source_ids": entry["frozen_source_ids"],
            "cumulative_source_ids": current_ids, "new_identity": not bool(current_ids),
            "delta_observation_indices": source_delta_rows,
            "locator_metadata": resolve(path, digest),
            "limit": "Preserve frozen original/context membership independently; actual path/hash equivalence is locator metadata only.",
        })

verification = read(verification_path)
qualifications = []
for i, item in enumerate(verification["range_qualifications_required"]):
    proof = item["proof"]
    key = proof_key(proof["record"], proof["trace"], proof["json_pointer"])
    if proof["trace"] in ("ROOT-PPS-RESOLVED-GEOMETRY-036", "ROOT-PPS-SIMULATION-CORE-GENERATED-PARTIAL-047", "CAM-PPS-SIMULATION-OBSERVATION-047", "ROOT-PPS-SIMULATION-POLICY-WIRE-RECEIVERS-049"):
        existing_qualification = "continuation/rig/source-role-qualification-045.json#/overrun_observation_qualifications"
    elif proof["trace"].startswith("CAM-PPS-"):
        existing_qualification = "records/final-acceptance-20261021/wave-01/producer-supplement-after-owner/camera/traces.jsonl#CAM-PPS-STATE-NATIVE-PORTS-054/metadata_qualifications"
    else:
        existing_qualification = "records/final-acceptance-20261021/wave-01/source-range-errata.json"
    qualifications.append({
        "verification_ref": f"producer-role-receipt-verification-pending.json#/range_qualifications_required/{i}",
        "path": item["path"], "actual_EOF": item["actual_eof"],
        "record": key[0], "parent_trace": key[1], "json_pointer": key[2],
        "original_ranges": proof["ranges"], "material_ranges": proof["material_contract_ranges"],
        "existing_qualification_candidate": existing_qualification,
        "status": "exact qualification evidence must be consumed; no automatic clipping or nonexistent-line credit",
    })

result = {
    "id": "RIG-CONT-EXPANDED-PRODUCER-SCHEMA-DELTA-MAP-049",
    "status": "metadata delta prepared only; no additional role clearance",
    "inputs": [{"path": str(p.relative_to(ROOT)), "sha256": sha(p)} for p in
               (expanded_path, expanded_observation_path, verification_path, current_path, scope_path)],
    "comparison_key": "normalized record + trace + JSON pointer; canonical snapshot/package path + SHA retained separately from aliases/context membership",
    "summary": {
        "exported_variants": len(expanded), "exported_proof_observations": all_expanded_refs,
        "current_named_observations": len(current["explicit_source_range_observations"]), "current_scope_variants": len(scope["files"]),
        "unchanged_exact_observations_not_repeated": accounted,
        "delta_observations": len(delta), "delta_classifications": dict(Counter(r["classification"] for r in delta)),
        "delta_sources": len(identity_rows), "new_actual_path_hash_variants_outside2437": sum(r["new_identity"] for r in identity_rows),
        "unresolved_delta_locators": sum(not r["locator_metadata"]["sha256_match"] for r in identity_rows),
        "range_qualification_observations": len(qualifications),
    },
    "delta_source_identity_map": identity_rows,
    "delta_observation_map": delta,
    "range_qualification_map": qualifications,
    "limits": [
        "Metadata supplement only. Original exported index and current authoritative scope/ledgers remain unchanged.",
        "Already accounted source-role observations are omitted, but same-source new ranges/roles and context-alias distinctions remain explicit.",
        "The reconstructed source_context may already preserve contract_ranges even when its range selector did not use them; those rows are narrower material qualifiers, not automatic new source reads.",
        "045/046 navigation qualifications remain valid only for their exact original roles and references; this supplement grants no broader role coverage.",
        "Root assigns actual source-role qualification after inspecting this delta. No owner lens or admission started."
    ],
    "source_edits": 0, "authoritative_scope_or_ledger_edits": 0, "project_commands_or_tests": 0,
    "overall_lens_clearance": "not awarded",
}
target = OUT / "expanded-producer-delta-map-049.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["summary"], indent=2))
print("sha256=" + sha(target))
