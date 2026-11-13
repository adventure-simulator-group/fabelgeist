#!/usr/bin/env python3
"""Pure evidence-only 8b successor transform; this file does not install it."""
import copy
import hashlib
import json
import sys
import textwrap

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
NAMES = ["BASELINE-95E-20261110.json", "CONTINUATION-PROGRESS-95E.json", "RECONCILIATION-059.json", "CONTINUATION.md"]
REPORT = "ASSESSMENT-CENTRAL-FOLLOWUPS-24-95E.json"
MAIN = "MAIN-RECHECK-POST8B.json"
PARENT = "8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b"
REV = "95e63628f9d70b8dd6e6fede64334696c6995009"
TREE = "1eec97de340d9b7debbb30ca0c0fb2240a73b93f"
IDS = [2561, 3044, 2545, 2743, 2744]
COUNTS = {"queue_records": 3713, "partial_before": 1404, "unassessed_before": 2309,
          "partial_after": 1409, "unassessed_after": 2304, "ledger_before": 1135,
          "proposed_relationships": 4, "ledger_after": 1139, "queue_closures": 0,
          "lens_completions": 0, "accepted_unique_defect_increment": 0}

def render(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()

BEFORE_IDENTITIES = {'BASELINE-95E-20261110.json': (1571392, 'f5be20ed6244f0828686340da2fc1ecff853b66ecf0f3dcd9fad09ef60212e4e', 'ab510bc2e0e6a10ecec9c535a04e827ceaa2d4fe'), 'CONTINUATION-PROGRESS-95E.json': (130658, 'd38601e2458b34f1f699716843fae197560b7ceb3d37516c19d3e0608be51950', '06f9590e4611aab7764c30c289816a86daa0bc7a'), 'RECONCILIATION-059.json': (1277686, '41f566e7b5de5d670bae16eaa317a58a5872df6330cdd70d1d0c41ffcbc4548e', '816bf7a4ece744895e40be3f64e3ffe7a82c2bdd'), 'CONTINUATION.md': (126760, '932420805532b49177e5bfebe1b18925c520114d2c56c413342b6aa4a1d120bc', '5f1d4928b2e2a8515e0147eb36f464e0bbb3265c')}
PROPOSAL_SHA256 = "145555a6f5ad092174f4214b04e255660e5903dcb18bbef293baaca8b8500d44"
ASSEMBLY_REVIEW = {
    "reviewer": "wrapped_scope",
    "scope": "Independent central assembly and successor comparisons; not independent source credit across overlapping work.",
    "public_packets": ["delegated/post8b-followups/later-transports/five-v3.tar.gz", "delegated/post8b-followups/later-transports/five-v4.tar.gz", "delegated/post8b-followups/later-transports/five-v5.tar.gz"],
    "original_manifest_sha256": ["91416b0d02a591d54d513192082b264ba6bd2759dbbd77a58a3a5108049ddcf0", "602935fff52bd5e01c400b00dae9ee79f4ea77e78e2a95f86369e917888a089b", "2d7d1e744ac7381a4ad039ace994581d17bcacf70816068fe9698ce96d8cf4cb"],
    "qualification": "Reviewer authored q2545 and q3044 source and independently source-reviewed q2561 and q2743; q2744 central review is also separately attributed. No reading credit transfers."
}
REPORT_QUALIFICATION = "Bounded central assessment of separately attributed source reports and selected historical metadata, not a new coordinator product-source read or proof of product behavior. Selected original and reconstructed 059 objects agree as parsed while complete streams differ. Bounded navigation is not an exhaustive alias inventory; missing checkpoint 3821, SOURCE004 and 73 nontext/no-positive-EOF contracts remain open."
ADOPTION_QUALIFICATION = "This report describes a prospective application until its exact reviewed after-images are installed. The retained proposal's unadopted wording and review chronology remain historical; installation yields only partial current-source assessments and planning relationships, with no unique-defect increment or semantic closure."

def expected_report(proposal):
    result = copy.deepcopy(proposal)
    result["schema"] = "726-post8b-central-assessment-v1"
    result["status"] = "adopted as five bounded partial assessments and four planning relationships; no semantic closure"
    result["pending_reviews"] = []
    result["qualification"] = REPORT_QUALIFICATION
    result["independent_assembly_review"] = copy.deepcopy(ASSEMBLY_REVIEW)
    result["adoption_qualification"] = ADOPTION_QUALIFICATION
    for item in result["assessments"]:
        item["disposition"] = "partial current-source assessment; no semantic closure, unique-defect increment or checkpoint 3821 attribution"
    return result

def transform(before, proposal_raw, report_raw, observation_raw, bindings):
    proposal, report, observation = (json.loads(raw) for raw in (proposal_raw, report_raw, observation_raw))
    assert render(proposal) == proposal_raw and render(report) == report_raw and render(observation) == observation_raw
    assert set(before) == set(NAMES)
    assert set(BEFORE_IDENTITIES) == set(NAMES)
    for name, raw in before.items():
        size, digest, blob = BEFORE_IDENTITIES[name]
        assert (len(raw), hashlib.sha256(raw).hexdigest(), hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()) == (size, digest, blob)
    assert hashlib.sha256(proposal_raw).hexdigest() == PROPOSAL_SHA256
    assert report == expected_report(proposal)
    assert set(bindings) == {"parent_checkpoint", "source_revision", "proposal_sha256", "report_sha256", "observation_sha256", "assembly_review_manifest_sha256"}
    assert bindings["parent_checkpoint"] == PARENT and bindings["source_revision"] == REV
    assert bindings["proposal_sha256"] == PROPOSAL_SHA256
    assert bindings["report_sha256"] == hashlib.sha256(report_raw).hexdigest()
    assert bindings["observation_sha256"] == hashlib.sha256(observation_raw).hexdigest()
    assert bindings["assembly_review_manifest_sha256"] == ASSEMBLY_REVIEW["original_manifest_sha256"]
    assert observation["schema"] == "726-post8b-main-observation-v1"
    assert proposal["schema"] == "726-post8b-five-synthesis-proposal-v5"
    assert proposal["status"] == "unadopted; independent review and application pending"
    assert report["status"] == "adopted as five bounded partial assessments and four planning relationships; no semantic closure"
    assert report["published_checkpoint"] == proposal["published_checkpoint"] == PARENT
    assert report["source_revision"] == proposal["source_revision"] == observation["observed_revision"] == REV
    assert report["source_tree"] == proposal["source_tree"] == observation["observed_tree"] == TREE
    assert observation["ref"] == "refs/heads/main" and observation["before_utc"] <= observation["after_utc"]
    assert [x["index"] for x in report["assessments"]] == IDS
    assert report["counts_if_adopted"] == proposal["counts_if_adopted"] == COUNTS
    assert len(report["assessments"]) == len(proposal["assessments"]) == 5
    for proposed, adopted in zip(proposal["assessments"], report["assessments"]):
        assert {k: v for k, v in adopted.items() if k != "disposition"} == {k: v for k, v in proposed.items() if k != "disposition"}
        assert proposed["disposition"].startswith("unadopted proposal")
        assert adopted["disposition"] == "partial current-source assessment; no semantic closure, unique-defect increment or checkpoint 3821 attribution"
    assert report["proposed_relationships"] == proposal["proposed_relationships"]
    assert report["pending_reviews"] == [] and report["independent_assembly_review"] and report["adoption_qualification"]
    assert len(report["proposed_relationships"]) == 4

    baseline, progress, queue = (json.loads(before[n]) for n in NAMES[:3])
    doc = before[NAMES[3]].decode()
    oldb, oldp, oldq = copy.deepcopy(baseline), copy.deepcopy(progress), copy.deepcopy(queue)
    assert baseline["source_revision"] == progress["source_revision"] == REV
    assert baseline["historical_queue_records_semantically_reconciled"] == progress["queue_records_semantically_reconciled"] == progress["lenses_complete"] == 0
    assert progress["repair"] == "not started" and progress["product_validation"] == "not performed"
    entries = queue["entries"]
    ledger = baseline["newly_assessed_ledger"]["entries"]
    assert len(entries) == 3713 and len(ledger) == progress["new_current_ledger_candidates"] == 1135
    assert sum(x["current_reconciliation"].startswith("partial") for x in entries) == 1404
    assert sum(x["current_reconciliation"] == "unassessed" for x in entries) == 2309
    rels = report["proposed_relationships"]
    assert len({x["id"] for x in rels}) == 4 and not ({x["id"] for x in ledger} & {x["id"] for x in rels})
    lenses = {x["index"]: x["historical_lens"] for x in report["assessments"]}
    for rel in rels:
        assert all(0 <= i < len(entries) for i in rel["queue"])
        assert all(0 <= i < 1135 for i in rel["ledger"])
        item = {"id": rel["id"], "assessed_lenses": sorted({lenses[i] for i in rel["queue"] if i in lenses}),
                "classification": rel["classification"],
                "historical_related_records": ["baseline-059/queue.json#/entries/" + str(i) for i in rel["queue"]],
                "current_related_records": ["BASELINE-95E-20261110.json#/newly_assessed_ledger/entries/" + str(i) for i in rel["ledger"]],
                "relationship_qualification": rel["qualification"], "connected_repair_direction": rel["direction"],
                "discovery": "partial; supported reach, policy, source complements and exact aliases remain open",
                "repair": "not started", "validation": "not performed", "record": REPORT,
                "historical_original_and_reconstruction": "Selected original and reconstructed 059 objects match as parsed; complete streams differ and neither stream restores or represents the missing checkpoint 3821 authority."}
        if "historical_status_qualification" in rel:
            item["historical_status_qualification"] = rel["historical_status_qualification"]
        ledger.append(item)
    for a in report["assessments"]:
        row = entries[a["index"]]
        assert (row["contract_id"], row["historical_lens"], row["historical_status"], row["current_reconciliation"]) == (a["contract_id"], a["historical_lens"], a["historical_status_projection"], "unassessed")
        assert "further_current_evidence" not in row and len(a["source_packets"]) == 2
        row["current_reconciliation"] = "partial current 95e source assessment and historical relationship; see " + REPORT + "; no semantic queue or lens closure and no checkpoint3821 attribution"
        row["further_current_evidence"] = [REPORT, "delegated/post8b-followups/INDEX.md", *a["source_packets"]]
    assert len(ledger) == 1139
    assert sum(x["current_reconciliation"].startswith("partial") for x in entries) == 1409
    assert sum(x["current_reconciliation"] == "unassessed" for x in entries) == 2304

    baseline["latest_main_recheck"] = MAIN
    progress["new_current_ledger_candidates"] = 1139
    progress["central_post8b_bounded_reconciliations"] = REPORT + "; five partial assessments and four planning relationships, with no semantic closure or accepted unique-defect increment."
    progress["post8b_queue_snapshot"] = "3,713 indexed: 1,409 partial and 2,304 unassessed; all remain semantically unreconciled."
    progress["post8b_history_qualification"] = "The prior post8ca later-evidence field describes its 8b predecessor snapshot; the five listed assessments are now partial. Original and reconstructed selected records, their revisions and readers remain separate; checkpoint3821 is missing."
    progress["checkpoint_8b_same_machine_recovery"] = "delegated/post8b-followups/INDEX.md records the separate shared-object recovery and its key, object and same-machine limitations."
    progress["main_recheck_post8b"] = MAIN
    progress["current_main_qualification"] = "GitHub connector observed main at 95e between " + observation["before_utc"] + " and " + observation["after_utc"] + "; this is point-in-time and main may advance. See " + MAIN + "."
    marker = "product validation is authorized here.\n\n"
    assert doc.count(marker) == 1
    paragraphs = [
        REPORT + " records five bounded partial assessments and four planning relationships. The working queue has 1,409 partial and 2,304 unassessed records, all semantically unreconciled; the ledger has 1,139 entries and relationships, not unique defects. No queue record or lens is closed and no unique-defect increment is accepted.",
        "delegated/post8b-followups/INDEX.md locates separately attributed source packets, independent reviews, custody and recovery records. Selected original and reconstructed 059 objects agree as parsed while complete streams differ. Missing checkpoint 3821, SOURCE004 and the 73 nontext or no-positive-EOF contracts remain open.",
        MAIN + " records a point-in-time main observation; main may advance. Later Lens 5, 8, 9, 10, 12 and 13 source work remains separately attributed and pending central reconciliation. Older paragraphs retain their historical snapshot counts and states."
    ]
    insert = "\n\n".join(textwrap.fill("`" + s.split(" ", 1)[0] + "` " + s.split(" ", 1)[1], width=80,
                                        break_long_words=False, break_on_hyphens=False) for s in paragraphs) + "\n\n"
    doc = doc.replace(marker, marker + insert, 1)
    assert queue.keys() == oldq.keys() and all(queue[k] == oldq[k] for k in queue if k != "entries")
    for i, (old, new) in enumerate(zip(oldq["entries"], entries)):
        if i not in IDS:
            assert old == new
        else:
            assert {k:v for k,v in new.items() if k not in ("current_reconciliation", "further_current_evidence")} == {k:v for k,v in old.items() if k != "current_reconciliation"}
    assert ledger[:1135] == oldb["newly_assessed_ledger"]["entries"]
    assert baseline["newly_assessed_ledger"]["qualification"] == oldb["newly_assessed_ledger"]["qualification"]
    assert all(baseline[k] == oldb[k] for k in oldb if k not in ("newly_assessed_ledger", "latest_main_recheck"))
    assert all(progress[k] == oldp[k] for k in oldp if k not in ("new_current_ledger_candidates", "current_main_qualification"))
    return {NAMES[0]: render(baseline), NAMES[1]: render(progress), NAMES[2]: render(queue), NAMES[3]: doc.encode()}
