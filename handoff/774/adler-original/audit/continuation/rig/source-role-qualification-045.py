"""Qualify receipt roles and metadata omissions, preserving historical evidence."""
from collections import Counter, defaultdict
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[1]
base_path = OUT / "source-scope-reconciliation-044.json"
index_path = ROOT / "continuation/producer-reached-role-index.json"
base = json.loads(base_path.read_text(encoding="utf-8"))
index = json.loads(index_path.read_text(encoding="utf-8-sig"))
context = json.loads((ROOT / "continuation-context.json").read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def path_for(source):
    variant, rel = source.split(":", 1)
    return Path(context["snapshots"][variant]) / rel


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


record_cache = {}
used_records = {}


def receipt(record, receipt_id):
    if record not in record_cache:
        path = ROOT / record
        used_records[record] = sha(path)
        record_cache[record] = ([json.loads(line) for line in path.read_text(encoding="utf-8-sig").splitlines() if line.strip()]
                                if path.suffix == ".jsonl" else [read_json(path)])
    for ordinal, item in enumerate(record_cache[record], 1):
        if receipt_id in (item.get("id"), item.get("trace_id"), "row-" + str(ordinal)):
            return item
    raise ValueError((record, receipt_id))


def pointer(document, location):
    for key in location.strip("/").split("/") if location else []:
        key = key.replace("~1", "/").replace("~0", "~")
        document = document[int(key)] if isinstance(document, list) else document[key]
    return document


def normalized_ranges(raw):
    return [raw] if raw and isinstance(raw[0], int) else raw


def validate(source, digest, ranges):
    path = path_for(source)
    observed = sha(path)
    count = len(path.read_bytes().splitlines())
    normalized = normalized_ranges(ranges)
    return {
        "source_id": source, "sha256": observed, "resolved_path": str(path),
        "read_ranges": normalized, "actual_EOF": count,
        "fingerprint_matches_receipt": digest is None or digest == observed,
        "endpoints_within_actual_source": bool(normalized) and all(1 <= a <= b <= count for a, b in normalized),
    }


fresh = [
    ("pr-working:crates/adventuresim-building-generator/src/audit.rs", [[1, 73]],
     "Module imports/include and mod declarations only; no independent computed quantity or producer/recipient function in this file. Included modules are separate source roles and are not implicitly read by this proof."),
    ("pr-working:crates/adventuresim-strategic-sim/src/live_core/encounter_policy.rs", [[270, 330]],
     "PublicCombatFingerprint tail preserves sorted members and actual centipoint max/multiply/round encoding. generated_method_skill_fit maps method tags to insight/stealth or intentionally unknown-terrain zero then clamps/rounds. generated_defeat_decision compares whole fingerprints and returns named Proceed/SuppressUnchanged. Existing raw policy tags/quantities findings retained."),
    ("pr-working:crates/adventuresim-strategic-sim/src/live_core/fixture_policy.rs", [[1, 133]],
     "Exact party selection/tie order, FixtureQuestLane and borrowed QuestDecisionObservation tag/reason records through final formatted event detail. Raw quest path also drives internal dispatch upstream in bootstrap; terminal formatting does not excuse the earlier policy-tag representation finding."),
    ("pr-working:crates/adventuresim-strategic-sim/src/live_core/bootstrap.rs", [[881, 1021]],
     "Actual producer chooses activity/direct-continuation/direct-contract/generated-open/generated-discovery tag, computes selected, stores observation, then matches same raw tag for application actions. Existing ROOT049 classification requirement retained, not treated as terminal text."),
    ("pr-working:crates/adventuresim-core/src/mission/roster.rs", [[1, 91]],
     "TacticalPartyRoster admits native durable u64 IDs without reordering, rejects empty/over-limit/duplicate, derives NonZeroU32 enrollment count and projects vector at actual native request. Existing fixtures retain original values/order including valid zero identity; no execution."),
    ("physical-p16:crates/adventuresim-core/src/mission/roster.rs", [[1, 91]],
     "Exact same-byte counterpart role reconciliation against fresh PR91 constructor/count/native projection and existing fixtures, after SHA match. Not an independent fresh full P16 read or automatic equality clearance."),
    ("pr-working:crates/adventuresim-stdb-module/src/strategic/mission_roster/authority_tests.rs", [[1, 231]],
     "Fresh original authority fixtures source read: captured roster remains authoritative through live group drift, invalid snapshots cannot launch/autoresolve/emit reports, native request matches identities/count, standalone capture and finale retry ownership retained. Typed clock and typed result/String cause composition remains existing repair; no fixture/helper/assertion addition or execution."),
    ("physical-p16:crates/adventuresim-stdb-module/src/strategic/mission_roster/authority_tests.rs", [[1, 231]],
     "Complete exact P16 textual delta against fresh PR231: only first reducer result alias differs, StrategicCombatResult versus StrategicOperationResult. Unchanged whole fixtures and every original case/value/order/oracle explicitly reconciled; no independent fresh full P16 read or validation claim."),
]
fresh_proofs = [dict(validate(source, None, ranges), role=role, proof_origin="Fresh bounded source read or explicitly described complete counterpart-delta reconciliation in this turn") for source, ranges, role in fresh]
fresh_by_id = {r["source_id"]: r for r in fresh_proofs}
assert fresh_by_id["pr-working:crates/adventuresim-core/src/mission/roster.rs"]["sha256"] == fresh_by_id["physical-p16:crates/adventuresim-core/src/mission/roster.rs"]["sha256"]

camera_record = "continuation/camera/traces.jsonl"
world = receipt(camera_record, "CONT-CAMERA-PP-009")
pure_record = "continuation/root/pure8-packet.json"
pure = receipt(pure_record, "CONT-ROOT-PP-PURE8-001")
cam_web = receipt(camera_record, "CONT-CAMERA-PP-008")

by_source = defaultdict(list)
for observation in index["explicit_source_range_observations"]:
    by_source[observation["source_id"]].append(observation)


def prove_observation(observation):
    context_node = observation["source_context"]
    return dict(validate(observation["source_id"], observation["sha256"], observation["read_ranges"]),
                record=observation["record"], receipt=observation["receipt"], json_pointer=observation["json_pointer"],
                source_role_context=context_node)


def choose_candidates(row):
    metadata = row["original_metadata"]
    refs = set(metadata.get("trace_refs", []))
    refs.update(str(metadata.get(k, "")) for k in ("trace_ref", "evidence"))
    candidates = [o for o in by_source[row["source_id"]]
                  if not metadata.get("sha256", metadata.get("observed_sha256"))
                  or o["sha256"] == metadata.get("sha256", metadata.get("observed_sha256"))]
    def rank(o):
        ranges = normalized_ranges(o["read_ranges"])
        return (o["receipt"] in refs, sum(b - a + 1 for a, b in ranges))
    candidates.sort(key=rank, reverse=True)
    return [prove_observation(candidates[0])] if candidates else []


rows = []
for original in base["unqualified_observation_schema_map"]:
    row = {k: original[k] for k in ("input_ordinal", "record", "receipt", "json_pointer", "source_id", "classification")}
    metadata = original["original_metadata"]
    kind = original["classification"]
    evidence = []
    if kind == "nested-variant-group-owns-range":
        parent = original["ancestor_range_candidates"][0]
        document = receipt(parent["record"], parent["receipt"])
        group = pointer(document, parent["json_pointer"])
        observed = metadata.get("observed_sha256")
        proof = validate(row["source_id"], observed, group["read_ranges"])
        assert proof["sha256"] == group["sha256"] == observed
        evidence = [dict(proof, record=parent["record"], receipt=parent["receipt"],
                         json_pointer=parent["json_pointer"], group_source_id=group["source_id"],
                         proof_mode=group.get("proof_mode"),
                         producer_role_graph="RIG-PPS-031::producer_to_receiver; Camera030 explicitly consumes same supplier graph")]
        row["disposition"] = "qualified-contained-variant-metadata; original80 are duplicate counterpart metadata, not80 newly read bodies"
        row["reason"] = "Parent owns explicit exact ranges and same-byte PR/P16 variant group. RIG031 actual twenty relationship/child/fixture groups and producer-to-actual-clock/state/cause/seed/native SDK paths were inspected; explicit counterpart roles retained, no identity or owner-fit inference."
    elif kind == "explicit-unread-progress-navigation":
        proofs = [(i, p) for i, p in enumerate(world["exact_source_proofs"]) if p["source_id"] == row["source_id"]]
        assert proofs, row
        i, proof = proofs[0]
        evidence = [dict(validate(proof["source_id"], proof["sha256"], proof["read_ranges"]),
                         record=camera_record, receipt=world["id"], json_pointer=f"/exact_source_proofs/{i}",
                         source_role_context=proof)]
        row["disposition"] = "historical-unread-navigation-superseded-by-actual-world5-source-and-receiver-proof"
        row["reason"] = "Original empty progress row proves no read. Current009 actual individually traced source/clock/place/context/roster/error/native receivers supply explicit later proof with findings; preserve original status as historical only."
    elif row["source_id"] in fresh_by_id and kind in ("bounded-range-without-explicit-fingerprint", "range-free-disposition-cross-reference"):
        proof = fresh_by_id[row["source_id"]]
        evidence = [dict(proof, receipt="RIG-CONT-PPS-SOURCE-ROLE-QUALIFICATION-045", record="continuation/rig/source-role-qualification-045.json", json_pointer=f"/fresh_source_proofs/{fresh_proofs.index(proof)}")]
        row["disposition"] = "navigation-or-missing-hash-reconciled-with-fresh-bounded-source-role"
        row["reason"] = "Actual source body and exact frozen hash/range now explicit; roster/fixture receivers additionally consumed via current008, prior CAM054 and Native027. No change to original fixtures or source."
    elif kind == "prior-range-plus-explicit-unread-recipient":
        proof = pure["fresh_or_reconciled_receiver_reads"][0]
        assert proof["source_id"] == row["source_id"]
        evidence = [dict(validate(proof["source_id"], proof["sha256"], proof["read_ranges"]),
                         record=pure_record, receipt=pure["id"], json_pointer="/fresh_or_reconciled_receiver_reads/0", source_role_context=proof)]
        row["disposition"] = "historical-unread-clock-recipient-superseded-by-exact-pure8-mutation-proof"
        row["reason"] = "Parent pure8 separately read actual isolated clock advance through epoch/native row mutation256-320; old helper/prior-range metadata is not counted as that read."
    else:
        evidence = choose_candidates(original)
        row["disposition"] = ("bounded-missing-hash-reconciled-with-exact-current-source-receipt" if kind == "bounded-range-without-explicit-fingerprint"
                              else "navigation-comparison-or-status-metadata; source-role-proof-follows-explicit-receipt")
        row["reason"] = "This observation is not an independent source-read obligation. Exact admitted source identity/hash/ranges and original producer-role context are supplied by referenced source receipts below; compare whole role/finding graph, never read-count or blanket samebytes."
    row["original_metadata"] = metadata
    row["source_role_evidence"] = evidence
    row["material_obligation_unresolved"] = not evidence or any(not p["fingerprint_matches_receipt"] or not p["endpoints_within_actual_source"] for p in evidence)
    row["remaining_limits"] = "Source-propagation findings persist. Owner-fit/admission/other lenses and source repair remain outside this packet; no overall clearance."
    rows.append(row)

overruns = []
for variant in base["new_variant_identity_and_range_map"]:
    for observation in variant["observations"]:
        if observation["range_metadata"]["status"] != "endpoint-conflict":
            continue
        proof = fresh_by_id[variant["source_id"]]
        overruns.append({
            "source_id": variant["source_id"], "sha256": variant["recorded_sha256"],
            "record": observation["record"], "receipt": observation["receipt"], "json_pointer": observation["json_pointer"],
            "original_recorded_ranges": observation["range_metadata"]["original"],
            "fresh_actual_read_ranges": proof["read_ranges"], "actual_EOF": proof["actual_EOF"],
            "fresh_source_role": proof["role"],
            "disposition": "explicit-new-bounded-source-role-proof; nonexistent historical endpoints receive zero credit",
        })

result = {
    "id": "RIG-CONT-PPS-SOURCE-ROLE-QUALIFICATION-045",
    "lens": "audit-producer-propagation-only",
    "status": "bounded-qualification-exhausted-with-findings",
    "inputs": [{"path": str(p.relative_to(ROOT)), "sha256": sha(p)} for p in (base_path, index_path)],
    "used_current_receipts": [{"record": path, "sha256": digest} for path, digest in sorted(used_records.items())],
    "fresh_source_proofs": fresh_proofs,
    "overrun_observation_qualifications": overruns,
    "unqualified_observation_dispositions": rows,
    "summary": {
        "overrun_observations_qualified": len(overruns), "original_unqualified_observations": len(rows),
        "per_observation_dispositions": dict(Counter(r["disposition"] for r in rows)),
        "unresolved_material_obligations": sum(r["material_obligation_unresolved"] for r in rows),
    },
    "genuinely_unresolved_material_obligations": [r for r in rows if r["material_obligation_unresolved"]],
    "findings_preserved": [
        "RIG031 relationship state/cause/deterministic selection identity propagation and original clock fixture adaptation findings, including CAM-PROP-F006 and F020.",
        "ROOT049 generated policy tags remain internal branch/retained-record facts before formatting; source role is not terminal text exemption.",
        "Current pure8 scheduled versus realized time/cause/clock/food unit findings and world5 canonical place/context/episode/Seed/sampling/required-time findings remain actionable.",
        "Current mission/roster/native SDK/error/identity/body/physical/cache findings in current008/Native027 remain actionable; existing source fixtures were read only."
    ],
    "limits": [
        "80 nested counterpart entries reuse exact explicit group proof and actual role graph; they are not independent fresh source reads.",
        "Range-free navigation/comparison entries remain navigation. This packet records linked material proof, not fabricated read ranges on original entries.",
        "Later world5 and pure8 receipts close historical unread navigation only through exact actual source/receiver proofs, not status or matching hashes alone.",
        "No authoritative source scope/ledger, implementation source, test population, CI or other audit lens changed."
    ],
    "source_edits": 0, "project_commands_or_tests": 0, "overall_lens_clearance": "not awarded",
}
target = OUT / "source-role-qualification-045.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["summary"], indent=2))
print("packet_sha256=" + sha(target))
