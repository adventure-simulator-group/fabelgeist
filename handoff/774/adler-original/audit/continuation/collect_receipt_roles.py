"""Index named-source JSON receipts; never scan source or award coverage.

This is deliberately only a named-source navigation index. It does not recognize
all path-only or contract_ranges schemas. The exported
producer-role-receipts-expanded-pending.jsonl and its continuation delta must be
reconciled as well; this output cannot establish exhaustive source scope.
"""

from pathlib import Path
import collections
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent.parent
RECORDS = ROOT / "records/final-acceptance-20261021"
FROZEN = json.loads((RECORDS / "wave-01/source-scope-after-owner.json").read_text())
SOURCES = {entry["source_id"]: entry for entry in FROZEN["files"]}
HASH_KEYS = ("observed_sha256", "sha256", "source_sha256", "expected_sha256")
RANGE_KEYS = ("read_ranges", "ranges", "source_ranges")
observations = []
unqualified = []


def visit(value, record, receipt, pointer=""):
    if isinstance(value, list):
        for index, child in enumerate(value):
            visit(child, record, receipt, f"{pointer}/{index}")
        return
    if not isinstance(value, dict):
        return
    identifier = value.get("source_id")
    if isinstance(identifier, str):
        fingerprints = [value.get(key) for key in HASH_KEYS if value.get(key)]
        fingerprint = next(
            (item for item in fingerprints if isinstance(item, str) and re.fullmatch(r"[0-9a-fA-F]{64}", item)),
            None,
        )
        ranges = next((value[key] for key in RANGE_KEYS if value.get(key)), None)
        if isinstance(ranges, list) and len(ranges) == 2 and all(isinstance(x, int) for x in ranges):
            ranges = [ranges]
        entry = {
            "source_id": identifier,
            "sha256": fingerprint,
            "read_ranges": ranges,
            "receipt": receipt,
            "record": record,
            "json_pointer": pointer,
            "source_context": value,
            "frozen_source_identity": identifier in SOURCES,
            "frozen_fingerprint_match": identifier in SOURCES
            and fingerprint == SOURCES[identifier].get("sha256"),
            "status": "pending_material_role_and_owner_fit_reconciliation_no_coverage_credit",
        }
        (observations if fingerprint and ranges else unqualified).append(entry)
    for key, child in value.items():
        if isinstance(child, (dict, list)):
            visit(child, record, receipt, f"{pointer}/{key}")


files = [
    RECORDS / f"wave-01/producer-supplement-after-owner/{owner}/traces.jsonl"
    for owner in ("root", "camera", "rig", "native")
]
# Explicit source receipts only. Derived indexes must never recursively become
# source-read evidence. Progress/cross-reference rows remain navigation metadata.
files += [ROOT / "continuation" / name for name in (
    "camera/coverage.jsonl", "camera/findings.jsonl", "camera/progress.jsonl",
    "camera/receivers.jsonl", "camera/sources.jsonl", "camera/traces.jsonl",
    "camera/camera716-final-receipt.json",
    "rig/progress-042.json", "rig/pure-food-recipient-carry-forward-041.json",
    "rig/range-schema-errata-043.json", "rig/web-request-reconciliation-039.json",
    "root/pure8-packet.json", "native/source-reads.json",
    "native/receiver-reads.json", "native/findings.json",
    "native/world5-final-packet.json",
)]
inputs = []
for path in files:
    raw = path.read_bytes()
    relative = str(path.relative_to(ROOT)).replace("\\", "/")
    inputs.append({"path": relative, "sha256": hashlib.sha256(raw).hexdigest()})
    rows = [json.loads(line) for line in raw.decode().splitlines() if line.strip()] if path.suffix == ".jsonl" else [json.loads(raw)]
    for index, value in enumerate(rows):
        receipt = value.get("id", value.get("trace_id", f"row-{index + 1}")) if isinstance(value, dict) else f"row-{index + 1}"
        visit(value, relative, receipt)

output = {
    "authority": "Derived named-source JSON receipt navigation only; path-only and contract_ranges schemas require the separate exported expanded index/delta. audit-ledger remains sole discovery/repair/validation authority. No source scanning, inferred source reads, owner-fit disposition or clearance.",
    "input_receipts": inputs,
    "explicit_source_range_observations": observations,
    "incomplete_metadata_observations": unqualified,
}
destination = ROOT / "continuation/producer-reached-role-index.json"
destination.write_text(json.dumps(output, indent=2) + "\n")
unique = {(entry["source_id"], entry["sha256"]) for entry in observations}
new = {(identifier, digest) for identifier, digest in unique if identifier not in SOURCES or SOURCES[identifier].get("sha256") != digest}
print(json.dumps({"explicit_observations": len(observations), "unique_named_variants": len(unique), "outside_frozen_identity_or_fingerprint": len(new), "unqualified_observations": len(unqualified), "new_variant_prefixes": dict(collections.Counter(identifier.split(":", 1)[0] for identifier, _ in new))}))
