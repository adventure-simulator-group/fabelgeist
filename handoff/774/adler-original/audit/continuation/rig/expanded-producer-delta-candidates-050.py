"""Add compact reuse candidates to049; candidates are not material clearance."""
from collections import Counter, defaultdict
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[1]


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


map_path = OUT / "expanded-producer-delta-map-049.json"
current_path = ROOT / "continuation/producer-reached-role-index.json"
scope_path = ROOT / "continuation/cumulative-source-scope-pending.json"
packet = read(map_path)
current = read(current_path)
scope = read(scope_path)
scope_by_id = {r["source_id"]: r for r in scope["files"]}


def normalized_path(row):
    path = row["path"].replace("\\", "/")
    source = row["source_id"]
    prefix, _, rel = source.partition(":")
    if path.startswith(("CARGO_REGISTRY/", "SNAPSHOTS/")):
        return path
    if prefix in ("pr-working", "physical-p16", "report-p17", "rig-quality"):
        return "SNAPSHOTS/" + prefix + "/" + rel
    return path


def merged(raw):
    rows = sorted(raw)
    answer = []
    for a, b in rows:
        if answer and a <= answer[-1][1] + 1:
            answer[-1][1] = max(answer[-1][1], b)
        else:
            answer.append([a, b])
    return answer


def contains(outer, inner):
    union = merged(outer)
    return all(any(a <= left <= right <= b for a, b in union) for left, right in inner)


by_path = defaultdict(list)
for observation in current["explicit_source_range_observations"]:
    source = observation["source_id"]
    if source in scope_by_id:
        path = normalized_path(scope_by_id[source])
    else:
        continue
    by_path[(path, observation["sha256"])].append(observation)

rows = []
for ordinal, delta in enumerate(packet["delta_observation_map"]):
    path, digest = delta["path"], delta["sha256"]
    requested = delta["material_contract_ranges"]
    observations = by_path[(path, digest)]
    covering = [r for r in observations if contains(r["read_ranges"], requested)]
    same_trace = [r for r in covering if r["receipt"] == delta["parent_trace"]]
    if delta["classification"] == "existing-observation-distinct-material-contract-range-qualifier":
        classification = "existing-ref-explicit-material-range-qualification"
    elif same_trace:
        classification = "same-path-and-trace-covering-range-candidate"
    elif covering:
        classification = "same-path-other-trace-covering-range-candidate"
    elif observations:
        classification = "same-path-newly-exposed-range-or-composite-range"
    else:
        classification = "new-source-context-needs-material-role-qualification"
    preferred = same_trace or covering
    preferred.sort(key=lambda r: sum(b - a + 1 for a, b in r["read_ranges"]))
    candidates = [{"record": r["record"], "receipt": r["receipt"], "json_pointer": r["json_pointer"], "ranges": r["read_ranges"]}
                  for r in preferred[:2]]
    rows.append({
        "delta_observation_index": ordinal, "classification": classification,
        "existing_range_receipt_candidates": candidates,
        "role_limit": "Containment and shared trace are navigation candidates only. Compare exact primitive/producer/state/recipient role, clock/frame/unit and original contract qualifiers; do not transfer semantic clearance from range inclusion.",
    })

new_sources = []
for source in packet["delta_source_identity_map"]:
    if not source["new_identity"]:
        continue
    path = source["path"]
    suffix = path.split("/", 2)[2] if path.startswith("SNAPSHOTS/") else path
    counterparts = [r["source_id"] for r in scope["files"] if r.get("sha256") == source["sha256"]
                    and (normalized_path(r).split("/", 2)[-1] if normalized_path(r).startswith("SNAPSHOTS/") else normalized_path(r)) == suffix]
    new_sources.append({
        "canonical_source_id_candidate": source["canonical_source_id_candidate"],
        "same-relative-path-and-byte-counterpart_candidates": counterparts,
        "identity_class": "new-snapshot-context-with-checked-counterpart-candidate" if counterparts else "new-path-role-without-matching-relative-path-counterpart",
        "limit": "Preserve original/context variants independently; same bytes do not establish same role or owner fit.",
    })

result = {
    "id": "RIG-CONT-EXPANDED-PRODUCER-DELTA-CANDIDATES-050",
    "status": "metadata navigation only; Root assigns actual delta qualification",
    "input_files": [{"path": str(p.relative_to(ROOT)), "sha256": sha(p)} for p in (map_path, current_path, scope_path)],
    "delta_observation_candidate_map": rows,
    "new_identity_counterpart_candidates": new_sources,
    "summary": {"delta_observations": len(rows), "metadata_candidate_groups": dict(Counter(r["classification"] for r in rows)),
                "new_identity_variants": len(new_sources), "new_identity_groups": dict(Counter(r["identity_class"] for r in new_sources))},
    "no_role_or_lens_clearance": True,
}
target = OUT / "expanded-producer-delta-candidates-050.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["summary"], indent=2))
print("sha256=" + sha(target))
