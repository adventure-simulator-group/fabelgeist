"""Append dependency pins, exact qualification exceptions and progress."""
from collections import Counter
from pathlib import Path
import hashlib
import json
import tomllib

OUT = Path(__file__).parent
ROOT = OUT.parents[1]
packet_path = OUT / "source-scope-reconciliation-044.json"
packet = json.loads(packet_path.read_text(encoding="utf-8"))
context = json.loads((ROOT / "continuation-context.json").read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


locks = {}
for source, root in context["snapshots"].items():
    path = Path(root) / "Cargo.lock"
    if path.is_file():
        locks[source] = {
            "path": str(path), "sha256": sha(path),
            "packages": tomllib.loads(path.read_text(encoding="utf-8"))["package"],
        }

pins = []
for variant in packet["new_variant_identity_and_range_map"]:
    canonical = variant["canonical_locator_identity"]
    if not canonical.startswith("registry:"):
        continue
    package = canonical[len("registry:"):].split("/")[0]
    matches = []
    for source, lock in locks.items():
        for entry in lock["packages"]:
            if entry["name"] + "-" + entry["version"] == package:
                matches.append({
                    "snapshot": source, "lock_path": lock["path"],
                    "lock_sha256": lock["sha256"], "package": entry,
                })
    pins.append({
        "source_id": variant["source_id"], "package_version": package,
        "source_sha256": variant["recorded_sha256"],
        "resolved_path": variant["resolved_candidate"]["path"],
        "lock_entries": matches,
        "pin_status": "present-in-exact-snapshot-lock" if matches else "unresolved-lock-pin",
        "limit": "Pin and locator metadata only; no dependency implementation or material-role clearance.",
    })

exceptions = []
for variant in packet["new_variant_identity_and_range_map"]:
    for observation in variant["observations"]:
        if observation["range_metadata"]["status"] == "endpoint-conflict":
            exceptions.append({
                "source_id": variant["source_id"], "sha256": variant["recorded_sha256"],
                "resolved_path": variant["resolved_candidate"]["path"],
                "actual_EOF": variant["resolved_candidate"]["actual_line_count"],
                "receipt": observation["receipt"], "record": observation["record"],
                "json_pointer": observation["json_pointer"],
                "recorded_ranges": observation["range_metadata"]["original"],
                "unresolved_action": "Reconcile exact material body/read claim and append explicit source-range qualification. Do not clip or invent lines. Existing source-range-errata.json has no entry for this source/hash.",
            })

unqualified = packet["unqualified_observation_schema_map"]
unqualified_refs = [{
    k: row[k] for k in (
        "input_ordinal", "record", "receipt", "json_pointer", "source_id",
        "classification", "missing_explicit_hash", "missing_explicit_ranges"
    )
} for row in unqualified]
result = {
    "id": "RIG-CONT-PPS-SOURCE-SCOPE-RECONCILIATION-044-DETAILS",
    "packet": packet_path.name, "packet_sha256": sha(packet_path),
    "status": "metadata-map-complete-with-explicit-qualifications; owner-fit pending",
    "dependency_pin_map": pins,
    "range_endpoint_exceptions": exceptions,
    "unqualified_exact_refs": unqualified_refs,
    "summary": dict(packet["summary"],
        dependency_variants=len(pins),
        dependency_pin_status=dict(Counter(r["pin_status"] for r in pins)),
        new_variants_with_frozen_same_byte_candidates=sum(bool(r["same_byte_frozen_counterpart_candidates"]) for r in packet["new_variant_identity_and_range_map"]),
        unqualified_with_parent_range_candidates=sum(bool(r["ancestor_range_candidates"]) for r in unqualified),
        unqualified_with_same_source_qualified_candidates=sum(bool(r["same_source_qualified_metadata_candidates"]) for r in unqualified)),
    "first_unfinished_cell": exceptions[0] if exceptions else None,
    "remaining_work": [
        "Owner-fit coordinator must explicitly admit or reject new158 source-context/path variants and package aliases, then reconcile material roles at exact source ranges.",
        "Resolve four EOF-overrun observations on three source paths; all exact SHA values match concrete source.",
        "231 unqualified metadata entries remain represented by exact refs and schema reasons; containing/linked proofs are candidates, not manufactured read coverage.",
        "No admission lens, source repair, tests, CI, source-scope or authoritative ledger mutation is authorized by this packet.",
    ],
    "source_edits": 0, "project_commands_or_tests": 0,
    "overall_lens_clearance": "not awarded",
}
target = OUT / "source-scope-reconciliation-044-details.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["summary"], indent=2))
print("details_sha256=" + sha(target))
