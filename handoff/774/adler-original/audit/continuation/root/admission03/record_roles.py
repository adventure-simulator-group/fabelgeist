"""Append an explicitly supplied semantic review; never infer source coverage."""
import datetime
import json
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parents[2]
out = pathlib.Path(__file__).resolve().parent
queue = json.loads((root / "admission03-queue-v002/root.json").read_text(encoding="utf-8"))["queue"]
packet = json.load(sys.stdin)
packet["at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
packet["lens"] = "audit-admission-contracts"
packet["repair"] = "frozen_initial_all13"
packet["validation"] = "not_established"
roles = packet.pop("coverage")
rows = [json.loads(line) for line in (out / "coverage.jsonl").read_text(encoding="utf-8").splitlines()]
seen = {row["source_id"] for row in rows}
for role in roles:
    source = queue[role["index"]]
    assert source["source_id"] not in seen, source["source_id"]
    rows.append({"id": f"ROOT-ADM-COVERAGE-{len(rows)+1:03}", "source_id": source["source_id"], "sha256": source["sha256"], "read_ranges": role["ranges"], "source_read": role.get("read", "actual_scoped_body_thislens"), "contract_refs": [packet["id"]], "admission_discovery": role.get("status", "complete_exact_scoped_routes_with_prior_findings"), "pending": role.get("pending", []), "repair": "frozen", "validation": "not_established"})
    seen.add(source["source_id"])
with (out / "contracts.jsonl").open("a", encoding="utf-8") as stream:
    stream.write(json.dumps(packet) + "\n")
(out / "coverage.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")
progress = json.loads((out / "progress.json").read_text(encoding="utf-8"))
progress.update(at=packet["at"], complete_source_roles=sum(not row.get("pending") for row in rows), body_or_metadata_source_roles_consumed=len(rows), pending_shared_material_contracts=sum(bool(row.get("pending")) for row in rows), next="Continue remaining exact original source roles and registered direct dependencies; initial03 incomplete")
(out / "progress.json").write_text(json.dumps(progress, indent=2) + "\n", encoding="utf-8")
print(packet["id"], len(roles), "roles; total inventoried", len(rows))
