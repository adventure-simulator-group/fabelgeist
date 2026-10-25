"""Observe ignored audit evidence; never inspect or execute project source."""

from pathlib import Path
import datetime
import hashlib
import json

A = Path(__file__).resolve().parent
master_path = A / "audit-ledger.json"
master = json.loads(master_path.read_text())
lens_name = master["current_lens"]
D = A / f"wave-{master['current_wave']:02d}" / lens_name
lens = next(row for row in master["lenses"] if row["lens"] == lens_name)
files = {}
observed = {}
groups = []


def read(path):
    data = path.read_bytes()
    files[str(path.relative_to(A))] = {
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
    }
    return json.loads(data)


def read_lines(path):
    data = path.read_bytes()
    files[str(path.relative_to(A))] = {
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
    }
    return [json.loads(line) for line in data.splitlines() if line.strip()]


def finding(row, owner, path):
    identifier = row.get("id", row.get("finding_id"))
    groups.append({
        "id": identifier,
        "lens_support": [lens_name],
        "status": "provisional_initial_discovery",
        "responsible_evidence_owner": owner,
        "source_evidence": str(path.relative_to(A)),
        "finding_ref": identifier,
        "disposition": row.get("disposition", row.get("status")),
        "remaining": row.get("dependencies", row.get("shared_dependencies", [])),
        "repair": "not_started_gate_closed",
        "validation": "not_established",
    })


root_path = D / "root/coverage.jsonl"
for row in read_lines(root_path):
    observed[row["source_id"]] = {
        "discovery": ("complete_non_applicable"
                      if row.get("discovery_status") == "complete" and row.get("disposition") == "non-applicable"
                      else "complete" if row.get("discovery_status") == "complete"
                      else "incomplete_receivers_pending"),
        "source_read": row["read_status"],
        "observed_sha256": row["observed_sha256"],
        "evidence": str(root_path.relative_to(A)),
        "trace_refs": row.get("trace_refs", []),
    }

camera_path = D / "camera/coverage.jsonl"
for row in read_lines(camera_path):
    if row.get("discovery") == "accounted_deleted_owner":
        observed[row["source_id"]] = {
            "discovery": "complete",
            "source_read": "deleted_observed_with_full_baseline_and_replacement",
            "observed_sha256": None,
            "observed_baseline_sha256": row["observed_baseline_sha256"],
            "evidence": str(camera_path.relative_to(A)),
            "trace_refs": row.get("trace_ids", []),
            "replacement_evidence": row.get("replacement_evidence"),
        }
    elif row.get("observed_sha256") and row.get("read_ranges"):
        observed[row["source_id"]] = {
            "discovery": ("complete_non_applicable"
                          if row.get("discovery") == "non_applicable"
                          else "complete" if row.get("discovery") == "inventory_accounted_with_findings" and row.get("receiver_reconciliation")
                          else "incomplete_receivers_pending"),
            "source_read": "source_ranges_recorded",
            "observed_sha256": row["observed_sha256"],
            "evidence": str(camera_path.relative_to(A)),
            "trace_refs": row.get("trace_ids", []),
        }

rig_path = D / "rig/sources.json"
for row in read(rig_path):
    if row.get("read_ranges"):
        observed[row["source_id"]] = {
            "discovery": ("complete_non_applicable"
                          if row.get("coverage") == "complete-non-applicable"
                          else "complete" if row.get("inventory_status") == "accounted-current-lens" and row.get("source_read_complete")
                          else "incomplete_receivers_pending"),
            "source_read": "source_ranges_recorded",
            "observed_sha256": row.get("observed_sha256"),
            "evidence": str(rig_path.relative_to(A)),
            "trace_refs": row.get("trace_refs", []),
        }

native_path = D / "native/progress.json"
native = read(native_path)
for row in native.get("coverage", []):
    if row.get("read_ranges"):
        observed[row["source_id"]] = {
            "discovery": (
                "complete"
                if row.get("discovery_status") in ["source inventory reconciled; actionable findings remain", "source inventory reconciled; no focused propagation defect on listed paths"]
                else "incomplete_receivers_pending"
            ),
            "source_read": row.get("read_status", "source_ranges_recorded"),
            "observed_sha256": row["observed_sha256"],
            "evidence": str(native_path.relative_to(A)),
            "trace_refs": [row.get("trace_id")],
        }

# Supplemental reads retain their actual evidence owner; the frozen source
# manifest remains unchanged. A full read is not a completed receiver trace.
manifest = read(A / "source-manifest.json")
pinned = {row["source_id"]: row for row in manifest["files"]}
for owner, filename in [
    ("rig", "supplemental-root-sources.json"),
    ("native", "supplemental-root-sources.json"),
    ("native", "supplemental-camera-sources.json"),
]:
    path = D / owner / filename
    if not path.exists():
        continue
    for row in read(path):
        sid = row["source_id"]
        assert sid in pinned, sid
        sha = row.get("observed_sha256")
        expected = pinned[sid]["sha256"] or pinned[sid]["base_sha256"]
        assert sha == expected, sid
        disposition = row.get("discovery_disposition", "")
        accounted = (
            disposition == "inventory-exhausted-with-connected-dispositions-and-findings"
            or disposition.startswith("Full exact assigned source variant and connected receiver inventory completed")
            or row.get("discovery_status") == "complete_with_findings"
            or disposition == "inventory-accounted-with-findings" and bool(row.get("receiver_reconciliation"))
            or disposition.startswith("Assigned full P16 source and exact reversePR delta read; authorized actual receiver inventory completed") and bool(row.get("current_receiver_evidence"))
        )
        observed[sid] = {
            "discovery": "complete" if accounted else "incomplete_receivers_pending",
            "source_read": row.get("read_status", "source_ranges_recorded"),
            "observed_sha256": sha,
            "evidence": str(path.relative_to(A)),
            "trace_refs": row.get("trace_refs", []),
            "actual_evidence_owner": owner,
        }

for cell in lens["cells"]:
    if cell["source_id"] in observed:
        cell.update(observed[cell["source_id"]])

for owner in ["root", "camera"]:
    path = D / owner / "findings.jsonl"
    for row in read_lines(path):
        finding(row, owner, path)
for path in (D / "rig").glob("contracts-*.json"):
    for row in read(path):
        if row.get("id", "").startswith(("RIG-PP-", "RIG-CODE-")):
            finding(row, "rig", path)
for name in ["supplemental-root-findings.json", "supplemental-db-findings.json", "supplemental-geometry-findings.json"]:
    supplemental_findings = D / "rig" / name
    if supplemental_findings.exists():
        for row in read(supplemental_findings):
            finding(row, "rig", supplemental_findings)
            if row.get("dedup"):
                groups[-1]["canonical_group"] = row["dedup"]
for row in native.get("findings", []):
    finding(row, "native", native_path)

# Merge explicit coordinator/owner defect-remedy links, preserving every
# supporting finding and source reference rather than dropping supplements.
merged = {}
for group in groups:
    key = group.pop("canonical_group", group["id"])
    if key not in merged:
        merged[key] = dict(group)
        merged[key]["id"] = key
        merged[key]["supporting_evidence"] = []
    target = merged[key]
    target["supporting_evidence"].append({
        "finding": group["finding_ref"],
        "evidence_owner": group["responsible_evidence_owner"],
        "source_evidence": group["source_evidence"],
        "disposition": group["disposition"],
    })
    for dependency in group["remaining"]:
        if dependency not in target["remaining"]:
            target["remaining"].append(dependency)
groups = list(merged.values())

lens["evidence_shard_observation"] = files
lens["progress_counts"] = {
    "source_variants_with_recorded_reads": len(observed),
    "complete_non_applicable_this_lens": sum(
        cell["discovery"] == "complete_non_applicable" for cell in lens["cells"]
    ),
    "initial_discovery_accounted_variants": sum(cell["discovery"] in ["complete", "complete_non_applicable"] for cell in lens["cells"]),
    "total_variants": len(lens["cells"]),
    "coverage_status": "incomplete; recorded reads do not imply completed traces",
}
lens["discovery"] = "incomplete"
first = next((cell for cell in lens["cells"]
              if cell["discovery"] not in ["complete", "complete_non_applicable"]), None)
if first:
    master["first_unfinished_cell"] = (
        f"wave-{master['current_wave']:02d}/{lens_name}/"
        f"{first['owner']}/{first['source_id']}"
    )
existing = [row for row in master["deduplicated_findings"]
            if row.get("lens_support") != [lens_name]]
master["deduplicated_findings"] = existing + groups
master["deduplication_status"] = (
    "Provisional defect/remedy groups; reconcile union of lens support and all "
    "shared repairs after exhaustive initial discovery before source edits."
)
master["updated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
master_path.write_text(json.dumps(master, indent=2) + "\n")
print(json.dumps({
    "lens": lens_name,
    "progress": lens["progress_counts"],
    "provisional_candidates": len(groups),
    "repairs_allowed": master["discovery_gate"]["repairs_allowed"],
    "project_commands_executed": False,
}))
