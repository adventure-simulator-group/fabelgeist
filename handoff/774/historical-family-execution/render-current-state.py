"""Render status views from the authoritative ledger; no project execution."""

from collections import Counter
from pathlib import Path
import hashlib
import json
import textwrap

reports = Path(__file__).resolve().parent
ledger_bytes = (reports / "execution-ledger.json").read_bytes()
ledger = json.loads(ledger_bytes)
ledger_hash = hashlib.sha256(ledger_bytes).hexdigest()


def write(path, content):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(content)
    temporary.replace(path)


def prose(value):
    return textwrap.fill(value, 80, break_long_words=False, break_on_hyphens=False)


def json_text(value):
    return json.dumps(value, indent=2) + "\n"


attempt = ledger["latest_hosted_attempt"]
expectations = ledger["next_hosted_contract"]
active_request = ledger["active_hosted_request"]
request_status = (
    f"Active run: {active_request['run']}, {active_request['status']} at "
    f"head {active_request['head']}. No current-head acceptance is inferred."
    if active_request
    else "No run is pending for the new increments."
)
lines = [
    "# Current migration state",
    "",
    prose("Generated from execution-ledger.json by render-current-state.py. "
          "Edit family status in the ledger and exact ownership in the claim "
          "files; regenerate this view after each integration, transfer or "
          "validation outcome and before a handoff/context reset."),
    "",
    f"Ledger snapshot: `{ledger['updated_at']}`.",
    "",
    f"Integration head: `{ledger['head']}`.",
    "",
    f"Published head: `{ledger['published_head']}`.",
    "",
    f"Verified main: `{ledger['verified_main']}`.",
    "",
    "Goal #726 remains active; PR #774 remains open. Do not merge.",
    "",
    prose("Final acceptance gate: " + ledger.get("acceptance_gate", {}).get("status", "see family records") + ". " + "Audit coverage authority: " + ledger.get("acceptance_gate", {}).get("audit_ledger", "execution-ledger.json") + ". Mandatory P16/P17 implementation and integration remain in scope."),
    "",
    "## Family state",
    "",
    prose("Source completion, integration and validation are independent. "
          "The six earlier acceptances remain historical evidence at their "
          "exact head; none validates the current integration head."),
    "",
    "| Family | Owner | Source | Integration | Current validation |",
    "| --- | --- | --- | --- | --- |",
]
for item in ledger["items"]:
    owner = ledger["owners"][item["responsible_owner"]]["label"]
    source = item["source_status"]["state"].replace("_", " ")
    integration = item["integration_status"]["state"].replace("_", " ")
    validation = item["validation_status"]["state"].replace("_", " ")
    if item["validation_status"]["historical_acceptance"]:
        validation += "; historical acceptance retained"
    lines.append(f"| {item['id']} | {owner} | {source} | {integration} | {validation} |")

lines += ["", "## Active ownership and next actions", ""]
for owner_id, owner in ledger["owners"].items():
    families = [i for i in ledger["items"] if i["responsible_owner"] == owner_id]
    active = [i["id"] for i in families if i["source_status"]["remaining_gaps"]]
    label = ", ".join(active) or "integration, reconciliation and validation"
    lines += [prose(f"{owner['label']}: {label}. Exact scopes: "
                    f"{owner['claims_file']}. See the ledger for each family's "
                    "remaining gaps, shared dependencies, commit mapping and next action."), ""]
lines += [prose(ledger["next_action"]), "", "## Hosted evidence", ""]
lines += [
    prose(f"Latest completed attempt: run {attempt['run']}, job {attempt['job']}, "
          f"{attempt['status']}/{attempt['conclusion']} at head {attempt['head']}. "
          f"{attempt['first_failed_contract']} "
          f"Covers the current integration head: {attempt['covers_current_integration_head']}. "
          f"{request_status}"),
    "",
    prose(f"Next hosted checkpoint: {expectations['status']}. "
          + (f"Selected head {expectations['head']}: "
             f"{expectations['required_groups']} required groups, "
             f"{expectations['log_count']} logs and "
             f"{expectations['command_vectors']} command vectors. "
             f"Evidence: {expectations['evidence']}."
             if expectations.get('head') else
             "No next head or acceptance selection has been declared. "
             "The previous completed selection is historical evidence in "
             "last_completed_hosted_contract.")),
    "",
    prose("Historical acceptance: "
          f"{ledger['historical_accepted_selection']['artifact_report']}, "
          f"head {ledger['historical_accepted_selection']['head']}. "
          "Historical closure and CI reports stay pinned to their recorded revisions."),
    "",
    "## Sequencing and constraints",
    "",
    prose(ledger["constraints"]["hosted_ci_gate"]),
    "",
    prose(ledger["constraints"]["quality_sequencing"]),
    "",
    prose(ledger["constraints"]["parallel_work"]),
    "",
    prose(ledger["constraints"]["local_project_checks"]),
    "",
    prose(ledger["constraints"]["tests"]),
    "",
    prose(ledger["constraints"]["reviews"]),
    "",
    prose("Preserve the original stash, unrelated work and historical evidence. "
          "No schema/state/generated binding changes or live database operations "
          "are authorized by this bookkeeping. Coverage limits remain explicit "
          "in each family and the pinned acceptance reports."),
    "",
]
write(reports / "CURRENT.md", "\n".join(lines))

owners = []
for owner_id, owner in ledger["owners"].items():
    families = [i for i in ledger["items"] if i["responsible_owner"] == owner_id]
    owners.append({
        "owner": owner_id,
        "families": [i["id"] for i in families],
        "worktree": owner["worktree"],
        "branch": owner["branch"],
        "claims_file": Path(owner["claims_file"]).name,
        "scope_authority": owner["claims_file"],
        "family_status": {i["id"]: {
            "source": i["source_status"]["state"],
            "integration": i["integration_status"]["state"],
            "validation": i["validation_status"]["state"],
        } for i in families},
    })
ownership = {
    "version": 2,
    "derived_from": "../execution-ledger.json",
    "ledger_sha256": ledger_hash,
    "updated_at": ledger["updated_at"],
    "authority": "Generated family/owner summary only. Exact file/function "
                 "ownership and transfer exclusions belong to existing per-agent "
                 "claim files. Family status belongs to execution-ledger.json.",
    "goal": "existing #726 / PR #774",
    "integration_head": ledger["head"],
    "published_head": ledger["published_head"],
    "verified_main": ledger["verified_main"],
    "coordinator": "/root",
    "policy": [
        "Read the ledger and all applicable existing claim files before editing.",
        "Claim exact files/functions before edits. First trace owner repairs "
        "the whole shared function for every affected family; coordinate "
        "transfers/exclusions in claims, then update the ledger dependencies.",
        "No competing edits, duplicate owners/types or temporary primitive adapters.",
        "Credit shared repairs to affected requirements without declaring "
        "those families complete automatically.",
        ledger["constraints"]["parallel_work"],
        ledger["constraints"]["hosted_ci_gate"],
        ledger["constraints"]["quality_sequencing"],
        ledger["constraints"]["local_project_checks"],
        ledger["constraints"]["tests"],
    ],
    "owners": owners,
    "shared_repairs_record": "../execution-ledger.json#shared_repairs",
    "historical_ownership_snapshot": "../" + ledger["historical_records"]["superseded_ownership_summary"],
}
write(reports / "parallel-ownership/ownership-map.json", json_text(ownership))
write(reports / "P06-hosted-status.json", json_text({
    "derived_from": "execution-ledger.json#latest_hosted_attempt",
    "ledger_sha256": ledger_hash,
    "current_family_status_record": "execution-ledger.json#items",
    **attempt,
}))
print(json_text({
    "derived_from": "execution-ledger.json",
    "ledger_sha256": ledger_hash,
    "family_count": len(ledger["items"]),
    "source_states": dict(Counter(i["source_status"]["state"] for i in ledger["items"])),
    "rendered": ["CURRENT.md", "parallel-ownership/ownership-map.json", "P06-hosted-status.json"],
    "project_commands_executed": False,
}))
