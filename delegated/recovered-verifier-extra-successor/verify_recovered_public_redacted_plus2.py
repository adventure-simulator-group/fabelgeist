#!/usr/bin/env python3
"""Verify recovered evidence files without adopting their audit claims."""
import ast
import gzip
import hashlib
import json
import re
import subprocess
import sys
import tempfile
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).parent

# BEGIN QUALIFIED PUBLIC REDACTIONS (three historical objects only)
# The historic manifest hash and original filename remain historical identity.
# These are public reconstructions: their content hashes are different and the
# original private bytes are not verified by this successor.
_qualified_crosswalk_path = "recovered/privacy-redaction-evidence/CROSSWALK.json"
_qualified_crosswalk_sha = "6f555e7505f101b2b51df1ae793db8cf8a24cba8ad354a5fa39a4f481d731e7f"
_qualified_objects = {
    "recovered/l12-final-registration-evidence/prior-direct-dependencies/0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0":
        ("0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0", 109772, "67106f070825d632c93c91a29d495ecb4157d735179ee278f55dc6700e7e1ee9", 109688, 1),
    "recovered/l2-canonical-comparison-evidence/74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48":
        ("74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48", 2913027, "25cbe3dcc967db682e2db78a4371d8ed8605976df5f2d3f150db509b2a5b7ea3", 2911040, 28),
    "recovered/prior-current-transition-evidence/7ab4700aa59883fb73e3c7ef55ea2c05ed5e13661ee8a5b82dae1732d76188a5":
        ("7ab4700aa59883fb73e3c7ef55ea2c05ed5e13661ee8a5b82dae1732d76188a5", 120752, "c21dca58dc77ed95ec680a61b91d45f49c1c86be0274ebf48409919f514d8c30", 120669, 1),
}
_qualified_manifests = {
    "recovered/l12-final-registration-evidence/MANIFEST.json": "48d91f4d15f4f4035c94668d707b5956fe65f878664406cfc697621120a537b3",
    "recovered/l2-canonical-comparison-evidence/MANIFEST.json": "fa8daf479053b9185d5efda335d63ab303ea8b2e63649f986c865690d892d4f9",
    "recovered/prior-current-transition-evidence/MANIFEST.json": "cae5df98ad256d51427c669c15cd84128b22d554f7f11116fc1e0b51152de51f",
}
_qualified_crosswalk_bytes = (root / _qualified_crosswalk_path).read_bytes()
if hashlib.sha256(_qualified_crosswalk_bytes).hexdigest() != _qualified_crosswalk_sha:
    raise ValueError("Qualified public redaction crosswalk hash mismatch")
_qualified_crosswalk = json.loads(_qualified_crosswalk_bytes)
if (_qualified_crosswalk.get("schema") != "726-privacy-proposal-crosswalk-v1"
        or _qualified_crosswalk.get("redaction_count") != 30
        or len(_qualified_crosswalk.get("objects", [])) != 3):
    raise ValueError("Qualified public redaction crosswalk scope mismatch")
_qualified_rows = {}
for _row in _qualified_crosswalk["objects"]:
    _original = _row["original"]
    _relative = _original["publication_path"]
    if _relative in _qualified_rows or _relative not in _qualified_objects:
        raise ValueError("Unexpected qualified public redaction identity")
    _orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans = _qualified_objects[_relative]
    _public = _row["variant"]
    if ((_original["sha256"], _original["bytes"], _public["sha256"], _public["bytes"], len(_row["redactions"]))
            != (_orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans)):
        raise ValueError("Qualified public redaction identity mismatch")
    _qualified_rows[_relative] = _row
if set(_qualified_rows) != set(_qualified_objects):
    raise ValueError("Incomplete qualified public redaction crosswalk")
_qualified_seen = set()

def _qualified_historical_manifest(relative):
    if relative not in _qualified_manifests:
        raise ValueError("Unexpected qualified historical manifest")
    data = (root / relative).read_bytes()
    if hashlib.sha256(data).hexdigest() != _qualified_manifests[relative]:
        raise ValueError("Qualified historical manifest hash mismatch")
    return data

def _qualified_public_object(relative, historical_sha, data):
    if relative not in _qualified_objects:
        raise ValueError("Unexpected qualified public object path")
    original_sha, _, public_sha, public_bytes, _ = _qualified_objects[relative]
    if historical_sha != original_sha:
        raise ValueError("Historical manifest and crosswalk identity mismatch")
    if len(data) != public_bytes or hashlib.sha256(data).hexdigest() != public_sha:
        raise ValueError("Qualified reconstructed public bytes mismatch")
    document = json.loads(data)
    for span in _qualified_rows[relative]["redactions"]:
        value = document
        for part in span["json_pointer"].split("/")[1:]:
            part = part.replace("~1", "/").replace("~0", "~")
            value = value[int(part)] if isinstance(value, list) else value[part]
        marker = span["replacement"]
        start, size = span["public_byte_offset"], span["public_token_bytes"]
        if value != marker or data[start:start + size] != json.dumps(marker).encode("utf-8"):
            raise ValueError("Qualified reconstructed public redaction span mismatch")
    _qualified_seen.add(relative)
    return data
# END QUALIFIED PUBLIC REDACTIONS

# BEGIN ADDITIONAL QUALIFIED PUBLIC REDACTIONS (two historical records only)
# The writer output is a reconstructed historical transcript; the portability
# input was already a redacted record, not the unredacted candidate it cites.
# Public hashes identify reconstructed bytes, not the withheld original bytes.
_extra_crosswalk_path = "recovered/privacy-redaction-evidence/ADDITIONAL-CROSSWALK.json"
_extra_crosswalk_sha = "617070c5ebd6d57ea79cdf4a69f1adc9cbece96feb144dccba99f40e234e2dc1"
_extra_objects = {
    "recovered/final-writer-static-trace/OUTPUT-419499.txt":
        ("57acc4af7fb1ed29ab96640a6f584028be9898dc9ff790add5ae5f2cca6af0ad", 590, "8564fa7a5b482ca1b774590be271608f572e8020abc7fa9d3d609821b690e192", 458, 3),
    "recovered/transcript-other-candidates/portability-review.redacted.json":
        ("21c676a13fb8e24af1bbbcdf4f19ffd911fc33e19d65b4f939b6c5da6bcf3e84", 8746, "37cd6bdad0a3c4f5779f5161a7292bc88f7cc701678ee279f98112c11766acf5", 8723, 2),
}
_extra_manifests = {
    "recovered/final-writer-static-trace/MANIFEST.json": "7d3b0330a9bede96b03a3f237148ff72b078724b51bb3347b54e368345ffda2a",
    "recovered/late-current-cell-count-evidence/MANIFEST.json": "727392524ed24cfb9a1f43b4851b7fdf7ed7f9c91952bcb1c8c2791c63659786",
    "recovered/transcript-other-candidates/portability-review.manifest.json": "5350e430c3926481aa099e9a17d8e977006e990aa355627462fb6ba60882c559",
}
_extra_crosswalk_bytes = (root / _extra_crosswalk_path).read_bytes()
if hashlib.sha256(_extra_crosswalk_bytes).hexdigest() != _extra_crosswalk_sha:
    raise ValueError("Additional qualified public redaction crosswalk hash mismatch")
_extra_crosswalk = json.loads(_extra_crosswalk_bytes)
if _extra_crosswalk.get("schema") != "726-extra-qualified-redaction-crosswalk-v1" or len(_extra_crosswalk.get("records", [])) != 2:
    raise ValueError("Additional qualified redaction scope mismatch")
_extra_rows = {}
for _extra_row in _extra_crosswalk["records"]:
    _extra_relative = _extra_row["publication_path"]
    if _extra_relative in _extra_rows or _extra_relative not in _extra_objects:
        raise ValueError("Unexpected additional redaction identity")
    _orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans = _extra_objects[_extra_relative]
    if ((_extra_row["original_sha256"], _extra_row["original_bytes"], _extra_row["public_sha256"], _extra_row["public_bytes"], len(_extra_row["redactions"]))
            != (_orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans)):
        raise ValueError("Additional qualified public redaction identity mismatch")
    _extra_rows[_extra_relative] = _extra_row
if set(_extra_rows) != set(_extra_objects):
    raise ValueError("Incomplete additional qualified redaction crosswalk")
_extra_seen = set()

def _extra_historical_manifest(relative):
    if relative not in _extra_manifests:
        raise ValueError("Unexpected additional historical manifest")
    data = (root / relative).read_bytes()
    if hashlib.sha256(data).hexdigest() != _extra_manifests[relative]:
        raise ValueError("Additional historical manifest hash mismatch")
    return data

def _extra_public_object(relative, historical_sha, data):
    if relative not in _extra_objects:
        raise ValueError("Unexpected additional qualified public object path")
    original_sha, _, public_sha, public_bytes, _ = _extra_objects[relative]
    if historical_sha != original_sha:
        raise ValueError("Additional historical manifest identity mismatch")
    if len(data) != public_bytes or hashlib.sha256(data).hexdigest() != public_sha:
        raise ValueError("Additional reconstructed public bytes mismatch")
    row = _extra_rows[relative]
    for span in row["redactions"]:
        start, size, marker = span["public_byte_offset"], span["public_token_bytes"], span["replacement"]
        if marker != "<redacted-local-path>" or data[start:start + size] != marker.encode("utf-8"):
            raise ValueError("Additional reconstructed public redaction span mismatch")
    if row["kind"] == "historical-temp-locators":
        doc = json.loads(data)
        locator = doc["external_locator_constraints"][4]["locator"]
        if locator.count("<redacted-local-path>") != 2:
            raise ValueError("Reconstructed portability locator mismatch")
    _extra_seen.add(relative)
    return data
# END ADDITIONAL QUALIFIED PUBLIC REDACTIONS
checks = [
    ("recovered/transcript-hash-matched", "sha256"),
    ("recovered/transcript-notes", "published_sha256"),
    ("recovered/transcript-queue-fragments", "normalized_json_sha256"),
]
count = 0
for relative, key in checks:
    directory = root / relative
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        name = item["file"]
        if Path(name).name != name:
            raise ValueError("Unsafe manifest filename")
        data = (directory / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != item[key]:
            raise ValueError("Hash mismatch: " + relative + "/" + name)
        count += 1
directory = root / "recovered/transcript-queue-fragments"
index = json.loads((directory / "EXTENDED-INDEX.json").read_text(encoding="utf-8"))
expected = {item["normalized_sha256"] for item in index["entries"]}
found = set()
with (directory / "PRINTED-VALUES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        data = (json.dumps(item["value"], indent=2, ensure_ascii=False) + "\n").encode("utf-8")
        sha = hashlib.sha256(data).hexdigest()
        if sha != item["normalized_sha256"] or sha in found:
            raise ValueError("Invalid or duplicate printed value: " + sha)
        found.add(sha)
if found != expected:
    raise ValueError("Printed value corpus and index differ")
contract_count = len(found)
directory = root / "recovered/printed-audit-documents"
index = json.loads((directory / "INDEX.json").read_text(encoding="utf-8"))
expected = {item["normalized_sha256"] for item in index["entries"]}
found = set()
with (directory / "VALUES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        data = (json.dumps(item["value"], indent=2, ensure_ascii=False) + "\n").encode("utf-8")
        sha = hashlib.sha256(data).hexdigest()
        if sha != item["normalized_sha256"] or sha in found:
            raise ValueError("Invalid or duplicate printed audit document: " + sha)
        found.add(sha)
if found != expected:
    raise ValueError("Printed audit document corpus and index differ")
document_count = len(found)
markdown_count = 0
with (root / "recovered/transcript-markdown-writes/WRITES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        data = item["literal_text"].encode("utf-8")
        if hashlib.sha256(data).hexdigest() != item["normalized_sha256"]:
            raise ValueError("Markdown write hash mismatch")
        markdown_count += 1
helper_count = 0
with (root / "recovered/transcript-helper-candidates/CANDIDATES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["candidate_text"].encode("utf-8")).hexdigest() != item["candidate_sha256"]:
            raise ValueError("Helper candidate hash mismatch")
        helper_count += 1
redacted_count = 0
with (root / "recovered/transcript-helper-candidates/REDACTED-CANDIDATES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["redacted_text"].encode("utf-8")).hexdigest() != item["redacted_sha256"]:
            raise ValueError("Redacted helper hash mismatch")
        redacted_count += 1
python_writes_count = 0
with (root / "recovered/transcript-python-writes/WRITES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["literal_text"].encode("utf-8")).hexdigest() != item["published_sha256"]:
            raise ValueError("Python literal write hash mismatch")
        python_writes_count += 1
other_count = 0
with (root / "recovered/transcript-other-candidates/CANDIDATES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["candidate_text"].encode("utf-8")).hexdigest() != item["candidate_sha256"]:
            raise ValueError("Other candidate hash mismatch")
        other_count += 1
directory = root / "recovered/transcript-other-candidates"
item = json.loads(_extra_historical_manifest("recovered/transcript-other-candidates/portability-review.manifest.json"))
if item["file"] != "portability-review.redacted.json":
    raise ValueError("Historical portability review identity mismatch")
_extra_public_object("recovered/transcript-other-candidates/" + item["file"], item["redacted_sha256"], (directory / item["file"]).read_bytes())
redacted_documents_count = 0
with (root / "recovered/printed-audit-documents/REDACTED-VALUES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        data = (json.dumps(item["value"], indent=2, ensure_ascii=False) + "\n").encode("utf-8")
        if hashlib.sha256(data).hexdigest() != item["redacted_sha256"]:
            raise ValueError("Redacted printed document hash mismatch")
        redacted_documents_count += 1
other_writes_count = 0
with (root / "recovered/transcript-other-writes/WRITES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["literal_text"].encode("utf-8")).hexdigest() != item["normalized_sha256"]:
            raise ValueError("Other literal write hash mismatch")
        other_writes_count += 1
patch_count = 0
with (root / "recovered/transcript-unresolved-patches/PATCHES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["patch_text"].encode("utf-8")).hexdigest() != item["patch_sha256"]:
            raise ValueError("Unresolved patch hash mismatch")
        patch_count += 1
command_count = 0
with (root / "recovered/late-audit-write-commands/COMMANDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["command_sha256"]:
            raise ValueError("Late command hash mismatch")
        command_count += 1
audit_commands_count = 0
with (root / "recovered/audit-write-command-candidates/COMMANDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["command_sha256"]:
            raise ValueError("Audit command hash mismatch")
        audit_commands_count += 1
reviewed_command_count = 0
with (root / "recovered/audit-write-command-candidates/REVIEWED-COMMANDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["published_sha256"]:
            raise ValueError("Reviewed audit command hash mismatch")
        reviewed_command_count += 1
directory = root / "recovered/printed-ledger-fragments"
index = json.loads((directory / "INDEX.json").read_text(encoding="utf-8"))
expected = {item["normalized_sha256"] for item in index["entries"]}
found = set()
with (directory / "VALUES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        data = (json.dumps(item["value"], indent=2, ensure_ascii=False) + "\n").encode("utf-8")
        sha = hashlib.sha256(data).hexdigest()
        if sha != item["normalized_sha256"] or sha in found:
            raise ValueError("Printed ledger fragment hash mismatch")
        found.add(sha)
if found != expected:
    raise ValueError("Printed ledger fragment index differs")
ledger_output_count = 0
with (root / "recovered/ledger-command-outputs/OUTPUTS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["output_sha256"]:
            raise ValueError("Ledger-related output hash mismatch")
        ledger_output_count += 1
redacted_ledger_outputs = 0
with (root / "recovered/ledger-command-outputs/REDACTED-OUTPUTS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["published_sha256"]:
            raise ValueError("Redacted ledger output hash mismatch")
        redacted_ledger_outputs += 1
ledger_context_count = 0
with (root / "recovered/ledger-command-outputs/COMMANDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["published_sha256"]:
            raise ValueError("Ledger command context hash mismatch")
        ledger_context_count += 1
if ledger_context_count != ledger_output_count + redacted_ledger_outputs:
    raise ValueError("Ledger command and output counts differ")
queue_records_count = 0
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["published_command_sha256"]:
            raise ValueError("Queue command hash mismatch")
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["published_output_sha256"]:
            raise ValueError("Queue output hash mismatch")
        queue_records_count += 1
late_source_count = 0
with (root / "recovered/late-source-read-records/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["command_sha256"]:
            raise ValueError("Late source command hash mismatch")
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["output_sha256"]:
            raise ValueError("Late source output hash mismatch")
        late_source_count += 1
directory = root / "recovered/source-registration-outputs"
manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
source_records_count = 0
for part in manifest["parts"]:
    name = part["file"]
    if Path(name).name != name:
        raise ValueError("Unsafe source record part name")
    path = directory / name
    if hashlib.sha256(path.read_bytes()).hexdigest() != part["sha256"]:
        raise ValueError("Source record part hash mismatch")
    part_count = 0
    with path.open(encoding="utf-8") as stream:
        for line in stream:
            item = json.loads(line)
            if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["command_sha256"]:
                raise ValueError("Source command hash mismatch")
            if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["output_sha256"]:
                raise ValueError("Source output hash mismatch")
            part_count += 1
    if part_count != part["records"]:
        raise ValueError("Source record part count mismatch")
    source_records_count += part_count
if source_records_count != manifest["screened_records"]:
    raise ValueError("Source record total mismatch")
redacted_source_path = directory / "REDACTED-RECORDS.jsonl"
if hashlib.sha256(redacted_source_path.read_bytes()).hexdigest() != manifest["redacted_part_sha256"]:
    raise ValueError("Redacted source records part hash mismatch")
redacted_source_count = 0
with redacted_source_path.open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["published_command_sha256"]:
            raise ValueError("Redacted source command hash mismatch")
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["published_output_sha256"]:
            raise ValueError("Redacted source output hash mismatch")
        redacted_source_count += 1
if redacted_source_count != manifest["redacted_records"]:
    raise ValueError("Redacted source record total mismatch")
draft_change_count = 0
with (root / "recovered/draft-filechanges/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["change_text"].encode("utf-8")).hexdigest() != item["change_sha256"]:
            raise ValueError("Draft file-change hash mismatch")
        draft_change_count += 1
draft_validation_count = 0
with (root / "recovered/draft-validation-records/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["published_command_sha256"]:
            raise ValueError("Draft validation command hash mismatch")
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["published_output_sha256"]:
            raise ValueError("Draft validation output hash mismatch")
        draft_validation_count += 1
late_history_count = 0
with (root / "recovered/late-command-history/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if hashlib.sha256(item["command_text"].encode("utf-8")).hexdigest() != item["published_command_sha256"]:
            raise ValueError("Late history command hash mismatch")
        if hashlib.sha256(item["output_text"].encode("utf-8")).hexdigest() != item["published_output_sha256"]:
            raise ValueError("Late history output hash mismatch")
        late_history_count += 1
directory = root / "recovered/hash-matched-dependencies"
manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
dependency_count = 0
for item in manifest["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["file"] != "objects/" + sha:
        raise ValueError("Unsafe dependency object path")
    if hashlib.sha256((directory / "objects" / sha).read_bytes()).hexdigest() != sha:
        raise ValueError("Dependency object hash mismatch")
    parent_name = item["citing_recovered_file"]
    if Path(parent_name).name != parent_name:
        raise ValueError("Unsafe citing file name")
    parent = json.loads((root / "recovered/transcript-hash-matched" / parent_name).read_bytes())
    def has_reference(value):
        if isinstance(value, dict):
            return ((value.get("path") == item["cited_path"] and value.get("sha256") == sha)
                    or any(has_reference(child) for child in value.values()))
        if isinstance(value, list):
            return any(has_reference(child) for child in value)
        return False
    if not has_reference(parent):
        raise ValueError("Citing file does not contain dependency reference")
    dependency_count += 1
later = json.loads((directory / "LATER-MATCHES.json").read_bytes())
coverage = json.loads((directory / "DEPENDENCY-COVERAGE.json").read_bytes())
missing_refs = {}
for ref in coverage["references"]:
    if ref.get("availability") == "not_located_by_hash":
        missing_refs.setdefault(ref["sha256"], []).append({key: ref[key] for key in ("cited_path", "from_file", "json_pointer")})
if len(missing_refs) != later["earlier_missing_distinct"] or len(later["matches"]) != later["later_matched_distinct"]:
    raise ValueError("Later dependency inventory count mismatch")
seen = set()
for match in later["matches"]:
    sha, rel = match["sha256"], Path(match["object"])
    if sha in seen or rel.is_absolute() or ".." in rel.parts or not rel.parts or rel.parts[0] != "recovered":
        raise ValueError("Unsafe later dependency locator")
    seen.add(sha)
    if match["citations"] != missing_refs.get(sha) or hashlib.sha256((root / rel).read_bytes()).hexdigest() != sha:
        raise ValueError("Later dependency reference mismatch")
if len(set(missing_refs) - seen) != later["still_unmatched_distinct"]:
    raise ValueError("Later dependency remaining count mismatch")

private_extract_scan = json.loads((directory / "PRIVATE-EXTRACT-SCAN.json").read_bytes())
if (private_extract_scan["schema"] != "726-private-extract-dependency-scan-v1"
        or private_extract_scan["target_distinct_sha256"] != 484
        or private_extract_scan["extracted_files_hashed"] != 18084
        or private_extract_scan["new_hash_matches"] != 0
        or private_extract_scan["published_helper_candidates_checked"] + private_extract_scan["python_literal_writes_checked"] + private_extract_scan["other_literal_writes_checked"] != 2727):
    raise ValueError("Private extract scan report mismatch")
# The private inputs are not shipped; this checks report consistency, not the scan itself.
dependency_current = json.loads((root / "CURRENT.json").read_bytes())
dependency_checkpoint = json.loads((root / "RECOVERY-CHECKPOINT.json").read_bytes())
dependency_locator = json.loads((root / "SOURCE-LOCATORS.json").read_bytes())["later_dependency_matches"]
for inventory in (dependency_current, dependency_checkpoint):
    if (inventory["later_matched_dependency_hashes"] != later["later_matched_distinct"]
            or inventory["dependency_inventory_hashes_still_unmatched"] != later["still_unmatched_distinct"]):
        raise ValueError("Dependency count summary mismatch")
if (dependency_locator["matched_distinct"] != later["later_matched_distinct"]
        or dependency_locator["still_unmatched_distinct"] != later["still_unmatched_distinct"]):
    raise ValueError("Dependency locator count mismatch")

direct_dir = root / "recovered/direct-printed-dependencies"
direct = json.loads((direct_dir / "MANIFEST.json").read_bytes())
if direct["schema"] != "726-direct-printed-dependencies-v1" or len(direct["files"]) != 9:
    raise ValueError("Direct printed dependency manifest mismatch")
for item in direct["files"]:
    sha = item["sha256"]
    if item["object"] != "objects/" + sha or len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe direct printed dependency path")
    data = (direct_dir / item["object"]).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha or item["references"] != missing_refs.get(sha):
        raise ValueError("Direct printed dependency hash or reference mismatch")

owned_dir = root / "recovered/owned-repair-capture"
owned_manifest = json.loads((owned_dir / "MANIFEST.json").read_bytes())
owned_sha = owned_manifest["sha256"]
if owned_sha != "1388d72009fa9d4052878c3c2328afd779ff64be99018808f600deda774171c3":
    raise ValueError("Unexpected capture repair evidence")
owned_bytes = (owned_dir / owned_sha).read_bytes()
if hashlib.sha256(owned_bytes).hexdigest() != owned_sha or owned_manifest["references"] != missing_refs.get(owned_sha):
    raise ValueError("Capture repair hash or citation mismatch")
owned = json.loads(owned_bytes)
if owned["at"] != owned_manifest["timestamp"] or owned["implementation_gate_complete"] is not False or owned["product_execution"] is not False:
    raise ValueError("Capture repair qualification mismatch")
for filename, key in (("OUTPUT-62523.txt", "output_sha256"), ("RECORDED-REPAIR-EXPRESSION.txt", "expression_sha256")):
    if hashlib.sha256((owned_dir / filename).read_bytes()).hexdigest() != owned_manifest[key]:
        raise ValueError("Capture repair source evidence mismatch")
reported = json.loads((owned_dir / "OUTPUT-62523.txt").read_text(encoding="utf-8"))
if reported["owned_repair"]["sha256"] != owned_sha or reported["owned_repair"]["path"] != owned_manifest["historical_path"]:
    raise ValueError("Capture repair printed hash mismatch")

asset_dir = root / "recovered/bevy-asset-event-plan"
asset_manifest = json.loads((asset_dir / "MANIFEST.json").read_bytes())
asset_sha = asset_manifest["sha256"]
if asset_sha != "dfaee2bc1a08cadeb48bc0aa3c8e73bc87ffa93b42d4fefe41646b4e01b7842a":
    raise ValueError("Unexpected asset plan hash")
asset_bytes = (asset_dir / asset_sha).read_bytes()
asset = json.loads(asset_bytes)
asset_source = root / asset_manifest["source_object"]
if (hashlib.sha256(asset_bytes).hexdigest() != asset_sha
        or asset_manifest["references"] != missing_refs.get(asset_sha)
        or hashlib.sha256(asset_source.read_bytes()).hexdigest() != asset_manifest["source_sha256"]
        or asset["sha256"] != asset_manifest["source_sha256"]
        or asset["member"] != "bevy_asset-0.19.1/src/event.rs"
        or asset["credit"] != 0):
    raise ValueError("Asset plan source or citation mismatch")

lock_dir = root / "recovered/historical-lock-delta"
lock_manifest = json.loads((lock_dir / "MANIFEST.json").read_bytes())
lock_sha = lock_manifest["sha256"]
if lock_sha != "90ef2adb9c90c6f469458557c19ec5655b91059e0d37d4372cb3cdf957f14420":
    raise ValueError("Unexpected historical lock delta")
if hashlib.sha256((lock_dir / lock_sha).read_bytes()).hexdigest() != lock_sha or lock_manifest["references"] != missing_refs.get(lock_sha):
    raise ValueError("Historical lock delta hash or citations mismatch")

embedded_dir = root / "recovered/embedded-printed-audit-records"
embedded = json.loads((embedded_dir / "MANIFEST.json").read_bytes())
if embedded["schema"] != "726-embedded-printed-audit-records-v1" or len(embedded["files"]) != 4:
    raise ValueError("Embedded audit record manifest mismatch")
for item in embedded["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != "objects/" + sha or item["fragment"] != "fragments/" + sha:
        raise ValueError("Unsafe embedded audit record locator")
    data = (embedded_dir / item["object"]).read_bytes()
    fragment = (embedded_dir / item["fragment"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != sha
            or hashlib.sha256(fragment).hexdigest() != item["fragment_sha256"]
            or json.loads(data) != json.loads(fragment)
            or item["references"] != missing_refs.get(sha)):
        raise ValueError("Embedded audit record value or citation mismatch")

ai_plan_sha = "f463bb6ec405791461853e00c96e68cda5f4ba98723900e6d933aced97ed1682"
ai_qualification_sha = "831d63435823b3201d0f7b8f8d45caac0c96acc0bf5aeed11ba83b570af73d8d"
ai_plan = json.loads((embedded_dir / "objects" / ai_plan_sha).read_bytes())
if ai_plan["source_qualification"]["sha256"] != ai_qualification_sha or ai_plan["implementation_gate_complete"] is not False:
    raise ValueError("AI-precision plan qualification mismatch")

linked_dir = root / "recovered/qualification-linked-registrations"
linked = json.loads((linked_dir / "MANIFEST.json").read_bytes())
if linked["schema"] != "726-qualification-linked-registrations-v1" or len(linked["files"]) != 2:
    raise ValueError("Qualification-linked registration manifest mismatch")
linked_sources = set()
for item in linked["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != "objects/" + sha:
        raise ValueError("Unsafe linked registration path")
    data = (linked_dir / item["object"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Linked registration hash mismatch")
    registration = json.loads(data)
    parent_path = Path(item["parent_qualification"])
    if parent_path.is_absolute() or ".." in parent_path.parts or parent_path.parts[:2] != ("recovered", "embedded-printed-audit-records"):
        raise ValueError("Unsafe parent qualification path")
    parent_data = (root / parent_path).read_bytes()
    if hashlib.sha256(parent_data).hexdigest() != item["parent_sha256"]:
        raise ValueError("Linked parent qualification mismatch")
    def contains_link(value):
        if isinstance(value, dict):
            return (value.get("path") == item["historical_path"] and value.get("sha256") == sha) or any(contains_link(v) for v in value.values())
        return isinstance(value, list) and any(contains_link(v) for v in value)
    if not contains_link(json.loads(parent_data)) or registration["at"] != item["timestamp"] or len(registration["rows"]) != len(item["source_rows"]):
        raise ValueError("Linked registration citation mismatch")
    for row, source in zip(registration["rows"], item["source_rows"]):
        source_sha = source["sha256"]
        if len(source_sha) != 64 or any(c not in "0123456789abcdef" for c in source_sha):
            raise ValueError("Unsafe linked source path")
        body = (linked_dir / "sources" / source_sha).read_bytes()
        if (hashlib.sha256(body).hexdigest() != source_sha or len(body) != source["bytes"]
                or source_sha != row["source_sha256"] or source["revision"] != row["revision"]
                or source["git_path"] != row["git_path"] or len(body.splitlines()) != row["actual_EOF"]):
            raise ValueError("Linked source body mismatch")
        linked_sources.add(source_sha)
if len(linked_sources) != linked["source_object_count"]:
    raise ValueError("Linked source object count mismatch")

l13_dir = root / "recovered/l13-state388-lineage"
l13 = json.loads((l13_dir / "MANIFEST.json").read_bytes())
l13_state_sha = l13["state_sha256"]
l13_plan_sha = l13["plan_sha256"]
if l13_state_sha != "ce8ab6fb497efbca3b780d6e57fda02d0aea40c1f8b1ce74eedc5a9f34c29587" or l13_plan_sha != "735a64dd7b4838cd51fbbbec52f4986a13f3255573239747a106ebfc54cfa25f":
    raise ValueError("Unexpected historical Lens 13 lineage")
l13_state_bytes = (l13_dir / l13_state_sha).read_bytes()
l13_plan_bytes = (l13_dir / l13_plan_sha).read_bytes()
l13_fragment = (l13_dir / "PLAN-PRINTED-11522.txt").read_bytes()
l13_parent = (root / l13["state_parent_verification"]).read_bytes()
if (hashlib.sha256(l13_state_bytes).hexdigest() != l13_state_sha
        or hashlib.sha256(l13_plan_bytes).hexdigest() != l13_plan_sha
        or hashlib.sha256(l13_fragment).hexdigest() != l13["plan_fragment_sha256"]
        or hashlib.sha256(l13_parent).hexdigest() != l13["state_parent_verification_sha256"]):
    raise ValueError("Lens 13 lineage hash mismatch")
l13_printed_hash = (l13_dir / "PLAN-HASH-11422.txt").read_bytes()
if hashlib.sha256(l13_printed_hash).hexdigest() != l13["plan_hash_output_sha256"] or json.loads(l13_printed_hash)["plan"]["sha256"] != l13_plan_sha:
    raise ValueError("Lens 13 printed plan hash mismatch")
l13_state = json.loads(l13_state_bytes)
l13_plan = json.loads(l13_plan_bytes)
l13_verification = json.loads(l13_parent)
if (json.loads(l13_fragment) != l13_plan
        or l13_plan["previous_state"]["sha256"] != l13_state["predecessor_sha256"]
        or l13_plan["previous_state"]["path"] != l13_state["predecessor"]
        or l13_verification["state"]["sha256"] != l13_state_sha
        or l13_verification["receipt"]["sha256"] != l13_state["receipt_sha256"]
        or l13_state["predecessor_sha256"] != l13["predecessor_sha256"]
        or l13_state["receipt_sha256"] != l13["receipt_sha256"]):
    raise ValueError("Lens 13 lineage citation mismatch")

l13_batch_dir = root / "recovered/l13-historical-state-batch"
l13_batch = json.loads((l13_batch_dir / "MANIFEST.json").read_bytes())
if [item["sequence"] for item in l13_batch["states"]] != [343, 348, 353, 359, 364, 368, 381] or l13_batch["new_states"] != 6 or l13_batch["already_preserved_states"] != 1:
    raise ValueError("Historical Lens 13 batch manifest mismatch")
for item in l13_batch["states"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe Lens 13 batch hash")
    paths = [Path(item[k]) for k in ("object", "verification", "predecessor_plan")]
    if any(p.is_absolute() or ".." in p.parts or not p.parts or p.parts[0] != "recovered" for p in paths):
        raise ValueError("Unsafe Lens 13 batch path")
    state_data, verification_data, predecessor_data = [(root / p).read_bytes() for p in paths]
    if (hashlib.sha256(state_data).hexdigest() != sha
            or hashlib.sha256(verification_data).hexdigest() != item["verification_sha256"]
            or hashlib.sha256(predecessor_data).hexdigest() != item["predecessor_plan_sha256"]):
        raise ValueError("Historical Lens 13 batch hash mismatch")
    state, verification, predecessor = map(json.loads, (state_data, verification_data, predecessor_data))
    if (verification["state"]["sha256"] != sha
            or verification["receipt"]["sha256"] != state["receipt_sha256"]
            or predecessor["previous_state"]["sha256"] != state["predecessor_sha256"]
            or predecessor["previous_state"]["path"] != state["predecessor"]
            or state["receipt_sha256"] != item["receipt_sha256"]
            or state["predecessor_sha256"] != item["predecessor_sha256"]
            or verification["original_after"] != state["original_accounted"]
            or verification["original_after"] != item["original_after"]):
        raise ValueError("Historical Lens 13 batch citation mismatch")

l13_output_dir = root / "recovered/l13-output-state-batch"
l13_output = json.loads((l13_output_dir / "MANIFEST.json").read_bytes())
if l13_output["schema"] != "726-l13-output-state-batch-v1" or len(l13_output["states"]) != 9:
    raise ValueError("Lens 13 output state manifest mismatch")
for item in l13_output["states"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != "objects/" + sha:
        raise ValueError("Unsafe Lens 13 output state locator")
    data = (l13_output_dir / item["object"]).read_bytes()
    state = json.loads(data)
    if hashlib.sha256(data).hexdigest() != sha or state["original_accounted"] != item["original_after"]:
        raise ValueError("Lens 13 output state hash mismatch")
    expected = {"state": sha, "predecessor": state["predecessor_sha256"], "receipt": state["receipt_sha256"]}
    if {ref["kind"] for ref in item["references"]} != set(expected):
        raise ValueError("Incomplete Lens 13 printed references")
    for ref in item["references"]:
        fragpath = Path(ref["fragment"])
        if fragpath.is_absolute() or ".." in fragpath.parts or fragpath.parts[0] != "fragments":
            raise ValueError("Unsafe Lens 13 output fragment locator")
        fragment = (l13_output_dir / fragpath).read_bytes()
        if (hashlib.sha256(fragment).hexdigest() != ref["fragment_sha256"]
                or ref["sha256"] != expected[ref["kind"]]
                or ref["historical_path"].encode() not in fragment
                or ref["sha256"].encode() not in fragment):
            raise ValueError("Lens 13 output reference mismatch")

owned_successor = owned_manifest["successor"]
successor_sha = owned_successor["sha256"]
if successor_sha != "a643c4bc1273a98cb678f257e699268cebf4f64161cd607e0a8c3d222ebe7583":
    raise ValueError("Unexpected capture repair successor")
successor_bytes = (owned_dir / successor_sha).read_bytes()
if hashlib.sha256(successor_bytes).hexdigest() != successor_sha:
    raise ValueError("Capture repair successor hash mismatch")
successor = json.loads(successor_bytes)
if (successor["supersedes"]["repair"]["sha256"] != owned_sha
        or successor["at"] != owned_successor["timestamp"]
        or successor["implementation_gate_complete"] is not False
        or successor["product_execution"] is not False):
    raise ValueError("Capture repair successor lineage mismatch")
for filename, key in (("OUTPUT-62739.txt", "output_sha256"), ("RECORDED-SUCCESSOR-EXPRESSION.txt", "expression_sha256")):
    if hashlib.sha256((owned_dir / filename).read_bytes()).hexdigest() != owned_successor[key]:
        raise ValueError("Capture repair successor source evidence mismatch")
reported_successor = json.loads((owned_dir / "OUTPUT-62739.txt").read_text(encoding="utf-8"))["owned_repair"]
if reported_successor["sha256"] != successor_sha or reported_successor["path"] != owned_successor["historical_path"]:
    raise ValueError("Capture repair successor printed reference mismatch")

capture_family_dir = root / "recovered/capture-family-repair-contracts"
capture_family = json.loads((capture_family_dir / "MANIFEST.json").read_bytes())
if capture_family["schema"] != "726-capture-family-repair-contracts-v1" or len(capture_family["contracts"]) != 3:
    raise ValueError("Capture family contract manifest mismatch")
for item in capture_family["contracts"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != "objects/" + sha:
        raise ValueError("Unsafe capture family contract path")
    data = (capture_family_dir / item["object"]).read_bytes()
    contract = json.loads(data)
    if hashlib.sha256(data).hexdigest() != sha or contract["at"] != item["timestamp"] or contract["implementation_gate_complete"] is not False:
        raise ValueError("Capture family contract hash mismatch")
    for path_key, hash_key in (("expression_fragment", "expression_sha256"), ("output_fragment", "output_sha256")):
        rel = Path(item[path_key])
        if rel.is_absolute() or ".." in rel.parts or rel.parts[0] != "fragments" or hashlib.sha256((capture_family_dir / rel).read_bytes()).hexdigest() != item[hash_key]:
            raise ValueError("Capture family evidence fragment mismatch")
    printed = json.loads((capture_family_dir / item["output_fragment"]).read_bytes())
    if printed["owned_repair"]["sha256"] != sha or printed["owned_repair"]["path"] != item["historical_path"] or printed["read"] != item["read_packet"]:
        raise ValueError("Capture family printed reference mismatch")
    def references_successor(value):
        if isinstance(value, dict):
            return value.get("sha256") == successor_sha or any(references_successor(v) for v in value.values())
        return isinstance(value, list) and any(references_successor(v) for v in value)
    if not references_successor(contract):
        raise ValueError("Capture family persistence dependency missing")
    if "uncredited_source" in item:
        source = item["uncredited_source"]
        source_sha = source["sha256"]
        if len(source_sha) != 64 or any(c not in "0123456789abcdef" for c in source_sha) or source["object"] != "objects/" + source_sha:
            raise ValueError("Unsafe capture family source path")
        body = (capture_family_dir / source["object"]).read_bytes()
        if (hashlib.sha256(body).hexdigest() != source_sha or len(body.splitlines()) != source["actual_eof"]
                or not any(row.get("source_sha256") == source_sha and row.get("body_credit") == 0 for row in contract["uncredited_new_source_scopes"])):
            raise ValueError("Capture family source identity mismatch")

view_mapping_dir = root / "recovered/capture-view-mapping"
view_mapping = json.loads((view_mapping_dir / "MANIFEST.json").read_bytes())
view_sha = view_mapping["sha256"]
if view_sha != "e8298267bb80376589cbc3b3d98e89301655de163dacb32ca1cfa95f85e91ea1":
    raise ValueError("Unexpected view mapping contract")
view_data = (view_mapping_dir / view_sha).read_bytes()
view = json.loads(view_data)
if hashlib.sha256(view_data).hexdigest() != view_sha or view["at"] != view_mapping["timestamp"] or view["implementation_gate_complete"] is not False:
    raise ValueError("View mapping contract hash mismatch")
for name, key in (("EXPRESSION.txt", "expression_sha256"), ("OUTPUT.txt", "output_sha256")):
    if hashlib.sha256((view_mapping_dir / name).read_bytes()).hexdigest() != view_mapping[key]:
        raise ValueError("View mapping evidence fragment mismatch")
printed_view = json.loads((view_mapping_dir / "OUTPUT.txt").read_bytes())["owned_repair"]
if printed_view["sha256"] != view_sha or printed_view["path"] != view_mapping["historical_path"]:
    raise ValueError("View mapping printed reference mismatch")
animation_sha = view_mapping["citing_animation_contract_sha256"]
if animation_sha != "97754ef56d3249ac8f65aeba78fe28844e9bc79e5345406016115c302752ad06":
    raise ValueError("Unexpected animation contract reference")
animation = json.loads((capture_family_dir / "objects" / animation_sha).read_bytes())
def contains_view_ref(value):
    if isinstance(value, dict):
        return value.get("sha256") == view_sha or any(contains_view_ref(v) for v in value.values())
    return isinstance(value, list) and any(contains_view_ref(v) for v in value)
if not contains_view_ref(animation):
    raise ValueError("Animation view mapping citation mismatch")

owner_dir = root / "recovered/capture-owner-outcomes"
owner_manifest = json.loads((owner_dir / "MANIFEST.json").read_bytes())
if owner_manifest["schema"] != "726-capture-owner-outcomes-recovery-v1" or [x["kind"] for x in owner_manifest["objects"]] != ["relations", "repair"]:
    raise ValueError("Capture owner recovery manifest mismatch")
owner_output = (owner_dir / "OUTPUT.txt").read_bytes()
if hashlib.sha256(owner_output).hexdigest() != owner_manifest["output_sha256"]:
    raise ValueError("Capture owner printed output mismatch")
owner_reported = json.loads(owner_output)
owner_values = {}
for item in owner_manifest["objects"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["expression"] != item["kind"] + "-expression.txt":
        raise ValueError("Unsafe capture owner recovery path")
    body = (owner_dir / sha).read_bytes()
    expression = (owner_dir / item["expression"]).read_bytes()
    printed_key = "facet_comparison" if item["kind"] == "relations" else "owned_repair"
    if (hashlib.sha256(body).hexdigest() != sha or hashlib.sha256(expression).hexdigest() != item["expression_sha256"]
            or owner_reported[printed_key]["sha256"] != sha or owner_reported[printed_key]["path"] != item["historical_path"]):
        raise ValueError("Capture owner object or citation mismatch")
    value = json.loads(body)
    if value["at"] != item["timestamp"] or value["implementation_gate_complete"] is not False:
        raise ValueError("Capture owner qualification mismatch")
    owner_values[item["kind"]] = value
if (owner_values["repair"]["facet_comparison"]["sha256"] != owner_manifest["objects"][0]["sha256"]
        or owner_values["repair"]["scene_native_port_dependency"]["sha256"] != successor_sha
        or len(owner_values["relations"]["facets"]) != 5):
    raise ValueError("Capture owner dependency lineage mismatch")
owner_sha = owner_manifest["objects"][1]["sha256"]
for item in capture_family["contracts"]:
    contract = json.loads((capture_family_dir / item["object"]).read_bytes())
    def cites_owner(value):
        if isinstance(value, dict):
            return value.get("sha256") == owner_sha or any(cites_owner(v) for v in value.values())
        return isinstance(value, list) and any(cites_owner(v) for v in value)
    if not cites_owner(contract):
        raise ValueError("Capture family owner dependency mismatch")

callers_dir = root / "recovered/capture-callers-successor"
callers = json.loads((callers_dir / "MANIFEST.json").read_bytes())
callers_sha = callers["sha256"]
if callers_sha != "d6f5e95db795892559219697930237b879932a7de5c10a9e4b5ad8632279d7b6":
    raise ValueError("Unexpected capture callers successor")
callers_data = (callers_dir / callers_sha).read_bytes()
callers_value = json.loads(callers_data)
if (hashlib.sha256(callers_data).hexdigest() != callers_sha or callers_value["at"] != callers["timestamp"]
        or callers_value["refines_prior_repair"]["sha256"] != callers["predecessor_sha256"]
        or callers_value["implementation_gate_complete"] is not False):
    raise ValueError("Capture callers successor hash or lineage mismatch")
for filename, key in (("EXPRESSION.txt", "expression_sha256"), ("OUTPUT.txt", "output_sha256")):
    if hashlib.sha256((callers_dir / filename).read_bytes()).hexdigest() != callers[key]:
        raise ValueError("Capture callers successor evidence mismatch")
callers_printed = json.loads((callers_dir / "OUTPUT.txt").read_bytes())["owned_repair"]
if callers_printed["sha256"] != callers_sha or callers_printed["path"] != callers["historical_path"]:
    raise ValueError("Capture callers successor printed reference mismatch")
if not any(item["sha256"] == callers["predecessor_sha256"] for item in capture_family["contracts"]):
    raise ValueError("Capture callers predecessor not recovered")

building_dir = root / "recovered/building-capture-identity"
building = json.loads((building_dir / "MANIFEST.json").read_bytes())
building_sha = building["sha256"]
if building_sha != "2e57004055c98bf54ac2ccc800669cf1ea39360ca05e24bf18c84bc04ae15a09":
    raise ValueError("Unexpected building capture proposal")
building_data = (building_dir / building_sha).read_bytes()
building_value = json.loads(building_data)
if (hashlib.sha256(building_data).hexdigest() != building_sha or building_value["at"] != building["timestamp"]
        or building_value["refines_prior_repair"]["sha256"] != building["predecessor_sha256"]
        or building_value["implementation_gate_complete"] is not False):
    raise ValueError("Building capture proposal hash or lineage mismatch")
for filename, key in (("EXPRESSION.txt", "expression_sha256"), ("OUTPUT.txt", "output_sha256")):
    if hashlib.sha256((building_dir / filename).read_bytes()).hexdigest() != building[key]:
        raise ValueError("Building capture proposal evidence mismatch")
building_printed = json.loads((building_dir / "OUTPUT.txt").read_bytes())
if (building_printed["owned_repair"]["sha256"] != building_sha or building_printed["owned_repair"]["path"] != building["historical_path"]
        or building_printed["required_steps"] != len(building_value["required_repair"])
        or building_printed["later_acceptance"] != len(building_value["later_acceptance"])):
    raise ValueError("Building capture proposal printed reference mismatch")

building_sources_dir = root / "recovered/building-source-registrations"
building_sources = json.loads((building_sources_dir / "MANIFEST.json").read_bytes())
if building_sources["schema"] != "726-building-source-registrations-v1" or len(building_sources["registrations"]) != 3:
    raise ValueError("Building registration manifest mismatch")
building_hashes = set()
for item in building_sources["registrations"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != "objects/" + sha:
        raise ValueError("Unsafe building registration path")
    data = (building_sources_dir / item["object"]).read_bytes()
    registration = json.loads(data)
    fragment_path = Path(item["fragment"])
    if fragment_path.is_absolute() or ".." in fragment_path.parts or fragment_path.parts[0] != "fragments":
        raise ValueError("Unsafe building registration fragment")
    fragment = (building_sources_dir / fragment_path).read_bytes()
    printed = json.loads(fragment)
    if (hashlib.sha256(data).hexdigest() != sha or hashlib.sha256(fragment).hexdigest() != item["fragment_sha256"]
            or printed["registration"]["sha256"] != sha or printed["registration"]["path"] != item["historical_path"]
            or registration["registered_at"] != item["timestamp"] or len(registration["sources"]) != item["source_rows"]
            or registration["uncredited_total_source_lines_registered"] != item["uncredited_lines"]):
        raise ValueError("Building registration citation mismatch")
    for row in registration["sources"]:
        source_sha = row["source_sha256"]
        if len(source_sha) != 64 or any(c not in "0123456789abcdef" for c in source_sha):
            raise ValueError("Unsafe building source hash")
        body = (building_sources_dir / "sources" / source_sha).read_bytes()
        if hashlib.sha256(body).hexdigest() != source_sha or len(body) != row["bytes"] or len(body.splitlines()) != row["actual_EOF"]:
            raise ValueError("Building source object mismatch")
        building_hashes.add(source_sha)
if len(building_hashes) != building_sources["distinct_source_objects"]:
    raise ValueError("Building source object count mismatch")

building_read_dir = root / "recovered/building-source-read-packets"
building_reads = json.loads((building_read_dir / "MANIFEST.json").read_bytes())
if building_reads["schema"] != "726-building-source-read-packets-v1" or [x["suffix"] for x in building_reads["packets"]] != ["003", "004", "005"]:
    raise ValueError("Building read packet manifest mismatch")
building_registration_lookup = {item["sha256"]: item for item in building_sources["registrations"]}
proposal_packets = {ref["sha256"]: ref["path"] for ref in building_value["evidence_packets"]}
for item in building_reads["packets"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != "objects/" + sha or item["fragment"] != "fragments/" + item["suffix"] + ".txt":
        raise ValueError("Unsafe building read packet path")
    data = (building_read_dir / item["object"]).read_bytes()
    packet = json.loads(data)
    fragment = (building_read_dir / item["fragment"]).read_bytes()
    printed = json.loads(fragment)["source_packet"]
    registration_item = building_registration_lookup[item["registration_sha256"]]
    registration = json.loads((building_sources_dir / registration_item["object"]).read_bytes())
    if (hashlib.sha256(data).hexdigest() != sha or hashlib.sha256(fragment).hexdigest() != item["fragment_sha256"]
            or printed["sha256"] != sha or printed["path"] != item["historical_path"]
            or proposal_packets.get(sha) != item["historical_path"] or packet["at"] != item["timestamp"]
            or packet["registration"]["sha256"] != item["registration_sha256"]
            or packet["registration"]["path"] != registration_item["historical_path"]
            or len(packet["rows"]) != item["rows"] or packet["uncredited_total_source_lines"] != item["uncredited_lines"]
            or packet["fresh_root_personal_source_lines"] != 0 or packet["source_adoption_performed"] is not False):
        raise ValueError("Building read packet citation or qualification mismatch")
    if [row["source"] for row in packet["rows"]] != registration["sources"]:
        raise ValueError("Building read packet registration rows mismatch")

fingerprint_dir = root / "recovered/capture-fingerprint-proposal"
fingerprint = json.loads((fingerprint_dir / "MANIFEST.json").read_bytes())
fingerprint_sha = fingerprint["sha256"]
if fingerprint_sha != "eda722f887e98460e6fe8b72171efdc903cfbc152d6faeda83d6029816ec9d89":
    raise ValueError("Unexpected fingerprint proposal")
fingerprint_data = (fingerprint_dir / fingerprint_sha).read_bytes()
fingerprint_value = json.loads(fingerprint_data)
if hashlib.sha256(fingerprint_data).hexdigest() != fingerprint_sha or fingerprint_value["at"] != fingerprint["timestamp"] or fingerprint_value["implementation_gate_complete"] is not False:
    raise ValueError("Fingerprint proposal hash or qualification mismatch")
for filename, key in (("EXPRESSION.txt", "expression_sha256"), ("OUTPUT.txt", "output_sha256")):
    if hashlib.sha256((fingerprint_dir / filename).read_bytes()).hexdigest() != fingerprint[key]:
        raise ValueError("Fingerprint proposal evidence mismatch")
fingerprint_printed = json.loads((fingerprint_dir / "OUTPUT.txt").read_bytes())
if (fingerprint_printed["owned_repair"]["sha256"] != fingerprint_sha or fingerprint_printed["owned_repair"]["path"] != fingerprint["historical_path"]
        or fingerprint_printed["required_steps"] != len(fingerprint_value["required_repair"])
        or fingerprint_printed["later_acceptance"] != len(fingerprint_value["later_acceptance"])):
    raise ValueError("Fingerprint proposal printed reference mismatch")

fingerprint_evidence_dir = root / "recovered/fingerprint-source-evidence"
fingerprint_evidence = json.loads((fingerprint_evidence_dir / "MANIFEST.json").read_bytes())
if fingerprint_evidence["schema"] != "726-fingerprint-source-evidence-v1" or [x["suffix"] for x in fingerprint_evidence["packets"]] != ["001", "002"]:
    raise ValueError("Fingerprint source evidence manifest mismatch")
fingerprint_refs = {x["sha256"]: x["path"] for x in fingerprint_value["evidence_packets"]}
fingerprint_sources = set()
for item in fingerprint_evidence["packets"]:
    suffix = item["suffix"]
    values = {}
    for role, key in (("registration", "registration_sha256"), ("read", "read_sha256")):
        sha = item[key]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Unsafe fingerprint evidence hash")
        body = (fingerprint_evidence_dir / "objects" / sha).read_bytes()
        fragment = (fingerprint_evidence_dir / "fragments" / (suffix + "-" + role + ".txt")).read_bytes()
        printed = json.loads(fragment)["registration" if role == "registration" else "source_packet"]
        if hashlib.sha256(body).hexdigest() != sha or hashlib.sha256(fragment).hexdigest() != item[role + "_output_sha256"] or printed["sha256"] != sha:
            raise ValueError("Fingerprint evidence hash mismatch")
        values[role] = (json.loads(body), printed)
    reg, reg_ref = values["registration"]
    packet, packet_ref = values["read"]
    if (packet["registration"] != reg_ref or fingerprint_refs.get(item["read_sha256"]) != packet_ref["path"]
            or len(reg["sources"]) != item["source_rows"] or reg["uncredited_total_source_lines_registered"] != item["uncredited_lines"]
            or packet["uncredited_total_source_lines"] != item["uncredited_lines"] or packet["fresh_root_personal_source_lines"] != 0
            or packet["source_adoption_performed"] is not False or [row["source"] for row in packet["rows"]] != reg["sources"]):
        raise ValueError("Fingerprint evidence linkage mismatch")
    for row in reg["sources"]:
        sha = row["source_sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Unsafe fingerprint source hash")
        body = (fingerprint_evidence_dir / "sources" / sha).read_bytes()
        if hashlib.sha256(body).hexdigest() != sha or len(body) != row["bytes"] or len(body.splitlines()) != row["actual_EOF"]:
            raise ValueError("Fingerprint source object mismatch")
        fingerprint_sources.add(sha)
if len(fingerprint_sources) != fingerprint_evidence["distinct_source_objects"]:
    raise ValueError("Fingerprint source object count mismatch")

entries_dir = root / "recovered/capture-entries-evidence"
entries_manifest = json.loads((entries_dir / "MANIFEST.json").read_bytes())
if entries_manifest["schema"] != "726-capture-entries-evidence-v1" or [x["role"] for x in entries_manifest["objects"]] != ["registration", "read"]:
    raise ValueError("Capture entries evidence manifest mismatch")
entries_values = {}
for item in entries_manifest["objects"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe capture entries hash")
    data = (entries_dir / "objects" / sha).read_bytes()
    fragment = (entries_dir / "fragments" / (item["role"] + ".txt")).read_bytes()
    printed = json.loads(fragment)["registration" if item["role"] == "registration" else "source_packet"]
    if (hashlib.sha256(data).hexdigest() != sha or hashlib.sha256(fragment).hexdigest() != item["fragment_sha256"]
            or printed["sha256"] != sha or printed["path"] != item["historical_path"]):
        raise ValueError("Capture entries evidence hash mismatch")
    entries_values[item["role"]] = (json.loads(data), printed)
entries_reg, entries_reg_ref = entries_values["registration"]
entries_read, entries_read_ref = entries_values["read"]
if (entries_read["registration"] != entries_reg_ref or entries_read["fresh_root_personal_source_lines"] != 0
        or entries_read["source_adoption_performed"] is not False
        or [row["source"] for row in entries_read["rows"]] != entries_reg["sources"]
        or entries_read_ref not in callers_value["evidence_packets"]):
    raise ValueError("Capture entries packet linkage mismatch")
entries_sources = set()
for row in entries_reg["sources"]:
    sha = row["source_sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe capture entries source hash")
    body = (entries_dir / "sources" / sha).read_bytes()
    if hashlib.sha256(body).hexdigest() != sha or len(body) != row["bytes"] or len(body.splitlines()) != row["actual_EOF"]:
        raise ValueError("Capture entries source object mismatch")
    entries_sources.add(sha)
if len(entries_sources) != entries_manifest["distinct_source_objects"]:
    raise ValueError("Capture entries source count mismatch")

building_trace_dir = root / "recovered/building-source-display-trace"
building_trace = json.loads((building_trace_dir / "MANIFEST.json").read_bytes())
if building_trace["schema"] != "726-building-source-display-trace-v1" or len(building_trace["chunks"]) != 17 or building_trace["packet_count"] != 3:
    raise ValueError("Building display trace manifest mismatch")
trace_occurrences = 0
trace_scopes = set()
for chunk in building_trace["chunks"]:
    cid = chunk["chunk_id"]
    if not cid or any(c not in "0123456789abcdef" for c in cid) or chunk["fragment"] != "fragments/" + cid + ".txt":
        raise ValueError("Unsafe building display fragment")
    data = (building_trace_dir / chunk["fragment"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != chunk["sha256"]:
        raise ValueError("Building display fragment hash mismatch")
    sections = {}
    path = None
    for line in data.decode("utf-8").splitlines():
        header = re.match(r"^SOURCE(?:\s+\d+)?\s+(\S+\.rs)(?:\s|$)", line)
        if header:
            path = header.group(1)
            sections.setdefault(path, {})
            continue
        numbered = re.match(r"^(\d+): ?(.*)$", line)
        if numbered and path:
            sections[path][int(numbered.group(1))] = numbered.group(2)
    for scope in chunk["scopes"]:
        sha = scope["source_sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Unsafe building display source")
        source_lines = (building_sources_dir / "sources" / sha).read_bytes().decode("utf-8").splitlines()
        for number in range(scope["first"], scope["last"] + 1):
            if number < 1 or number > len(source_lines) or sections.get(scope["git_path"], {}).get(number) != source_lines[number - 1]:
                raise ValueError("Building display does not match pinned source")
            trace_occurrences += 1
        trace_scopes.add((scope["packet_sha256"], sha, scope["git_path"], cid, scope["first"], scope["last"]))
expected_scopes = set()
for item in building_reads["packets"]:
    packet = json.loads((building_read_dir / item["object"]).read_bytes())
    for row in packet["rows"]:
        for chunk in row["output_chunks"]:
            cid, span = chunk.split(":", 1)
            first, last = map(int, span.split(".."))
            expected_scopes.add((item["sha256"], row["source_sha256"], row["source"]["git_path"], cid, first, last))
if trace_scopes != expected_scopes or trace_occurrences != building_trace["matched_line_occurrences"] or trace_occurrences != 4674:
    raise ValueError("Building display trace scope mismatch")

capture_trace_dir = root / "recovered/capture-source-display-trace"
capture_trace = json.loads((capture_trace_dir / "MANIFEST.json").read_bytes())
if capture_trace["schema"] != "726-capture-source-display-trace-v1" or len(capture_trace["chunks"]) != 9 or capture_trace["packet_count"] != 3:
    raise ValueError("Capture display trace manifest mismatch")
capture_occurrences = 0
capture_scopes = set()
for chunk in capture_trace["chunks"]:
    cid = chunk["chunk_id"]
    if not cid or any(c not in "0123456789abcdef" for c in cid) or chunk["fragment"] != "fragments/" + cid + ".txt":
        raise ValueError("Unsafe capture display fragment")
    data = (capture_trace_dir / chunk["fragment"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != chunk["sha256"]:
        raise ValueError("Capture display fragment hash mismatch")
    sections = {}
    path = None
    for line in data.decode("utf-8").splitlines():
        header = re.match(r"^SOURCE(?:\s+\d+)?\s+(\S+\.rs)(?:\s|$)", line)
        if header:
            path = header.group(1)
            sections.setdefault(path, {})
            continue
        numbered = re.match(r"^(\d+): ?(.*)$", line)
        if numbered and path:
            sections[path][int(numbered.group(1))] = numbered.group(2)
    for scope in chunk["scopes"]:
        sha = scope["source_sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or scope["family"] not in ("fingerprint", "callers"):
            raise ValueError("Unsafe capture display source")
        source_dir = fingerprint_evidence_dir if scope["family"] == "fingerprint" else entries_dir
        lines = (source_dir / "sources" / sha).read_bytes().decode("utf-8").splitlines()
        for number in range(scope["first"], scope["last"] + 1):
            if number < 1 or number > len(lines) or sections.get(scope["git_path"], {}).get(number) != lines[number - 1]:
                raise ValueError("Capture display does not match pinned source")
            capture_occurrences += 1
        capture_scopes.add((scope["family"], scope["packet_sha256"], sha, scope["git_path"], cid, scope["first"], scope["last"]))
expected_capture_scopes = set()
packet_list = [("fingerprint", item["read_sha256"], fingerprint_evidence_dir / "objects" / item["read_sha256"]) for item in fingerprint_evidence["packets"]]
packet_list.append(("callers", entries_read_ref["sha256"], entries_dir / "objects" / entries_read_ref["sha256"]))
for family, packet_sha, path in packet_list:
    packet = json.loads(path.read_bytes())
    for row in packet["rows"]:
        for chunk in row["output_chunks"]:
            cid, span = chunk.split(":", 1)
            first, last = map(int, span.split(".."))
            expected_capture_scopes.add((family, packet_sha, row["source_sha256"], row["source"]["git_path"], cid, first, last))
if capture_scopes != expected_capture_scopes or capture_occurrences != capture_trace["matched_line_occurrences"] or capture_occurrences != 2531:
    raise ValueError("Capture display trace scope mismatch")

recursive_directory = root / "recovered/recursive-dependencies"
recursive_manifest = json.loads(
    (recursive_directory / "MANIFEST.json").read_text(encoding="utf-8")
)
recursive_count = 0
for item in recursive_manifest:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe recursive dependency object path")
    if hashlib.sha256((recursive_directory / "objects" / sha).read_bytes()).hexdigest() != sha:
        raise ValueError("Recursive dependency object hash mismatch")
    if not item["references"]:
        raise ValueError("Recursive dependency has no parent citation")
    for reference in item["references"]:
        parent_sha = reference["from_dependency"]
        if (len(parent_sha) != 64
                or any(c not in "0123456789abcdef" for c in parent_sha)):
            raise ValueError("Unsafe recursive dependency parent path")
        parent_data = (directory / "objects" / parent_sha).read_bytes()
        if hashlib.sha256(parent_data).hexdigest() != parent_sha:
            raise ValueError("Recursive dependency parent hash mismatch")
        def contains_reference(value):
            if isinstance(value, dict):
                return ((value.get("path") == reference["cited_path"]
                         and value.get("sha256") == sha)
                        or any(contains_reference(child) for child in value.values()))
            if isinstance(value, list):
                return any(contains_reference(child) for child in value)
            return False
        if not contains_reference(json.loads(parent_data)):
            raise ValueError("Recursive dependency parent citation mismatch")
    recursive_count += 1
printed_directory = root / "recovered/printed-hash-registrations"
printed_manifest = json.loads((printed_directory / "MANIFEST.json").read_text(encoding="utf-8"))
printed_count = 0
for item in printed_manifest["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe printed-hash registration path")
    data = (printed_directory / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Printed-hash registration object mismatch")
    output_name = item.get("printed_output_file", "PRINTED-OUTPUT.json")
    if output_name not in ("PRINTED-OUTPUT.json", "printed/" + sha + ".json"):
        raise ValueError("Unsafe printed output path")
    output = (printed_directory / output_name).read_bytes()
    if hashlib.sha256(output).hexdigest() != item["printed_output_sha256"]:
        raise ValueError("Printed registration output mismatch")
    reference = json.loads(output)["registration"]
    if reference != {"path": item["path"], "sha256": sha}:
        raise ValueError("Printed registration reference mismatch")
    printed_count += 1
audit_directory = root / "recovered/printed-hash-audit-records"
audit_manifest = json.loads((audit_directory / "MANIFEST.json").read_text(encoding="utf-8"))
audit_record_count = 0
for item in audit_manifest["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe printed-hash audit record path")
    data = (audit_directory / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Printed-hash audit record mismatch")
    output_name = item.get("printed_output_file", "PRINTED-OUTPUT.json")
    if output_name not in ("PRINTED-OUTPUT.json", "printed/" + sha + ".json"):
        raise ValueError("Unsafe printed audit output path")
    output = (audit_directory / output_name).read_bytes()
    if hashlib.sha256(output).hexdigest() != item["printed_output_sha256"]:
        raise ValueError("Printed audit output mismatch")
    reference = json.loads(output)[item["printed_reference_key"]]
    if reference != {"path": item["path"], "sha256": sha}:
        raise ValueError("Printed audit reference mismatch")
    audit_record_count += 1
reference_directory = root / "recovered/printed-reference-json"
reference_manifest = json.loads((reference_directory / "MANIFEST.json").read_text(encoding="utf-8"))
reference_count = 0
for item in reference_manifest["files"]:
    sha = item["sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
            or item["file"] != "objects/" + sha):
        raise ValueError("Unsafe printed-reference object path")
    if hashlib.sha256((reference_directory / item["file"]).read_bytes()).hexdigest() != sha:
        raise ValueError("Printed-reference JSON hash mismatch")
    if not item["cited_path"] or not isinstance(item["reference_ordinal"], int):
        raise ValueError("Missing printed-reference provenance")
    reference_count += 1
linked_directory = root / "recovered/recursive-printed-json"
linked_manifest = json.loads((linked_directory / "MANIFEST.json").read_text(encoding="utf-8"))
linked_count = 0
for item in linked_manifest["files"]:
    sha = item["sha256"]
    parent_sha = item["parent_sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
            or item["file"] != "objects/" + sha
            or len(parent_sha) != 64
            or any(c not in "0123456789abcdef" for c in parent_sha)
            or item["parent_collection"] not in ("printed-reference-json",
                "printed-hash-audit-records", "printed-hash-registrations")):
        raise ValueError("Unsafe linked JSON path")
    data = (linked_directory / item["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Linked JSON hash mismatch")
    parent = (root / "recovered" / item["parent_collection"] / "objects" / parent_sha).read_bytes()
    if hashlib.sha256(parent).hexdigest() != parent_sha:
        raise ValueError("Linked JSON parent hash mismatch")
    def cited(value):
        if isinstance(value, dict):
            return ((value.get("path") == item["cited_path"] and value.get("sha256") == sha)
                    or any(cited(child) for child in value.values()))
        if isinstance(value, list):
            return any(cited(child) for child in value)
        return False
    if not cited(json.loads(parent)):
        raise ValueError("Linked JSON parent citation mismatch")
    linked_count += 1
literal_directory = root / "recovered/linked-literal-evidence"
literal_manifest = json.loads((literal_directory / "MANIFEST.json").read_text(encoding="utf-8"))
literal_count = 0
for item in literal_manifest["files"]:
    sha = item["sha256"]
    parent_sha = item["parent_sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
            or item["file"] != "objects/" + sha
            or len(parent_sha) != 64
            or any(c not in "0123456789abcdef" for c in parent_sha)
            or item["parent_collection"] not in ("printed-reference-json",
                "recursive-printed-json", "printed-hash-audit-records",
                "printed-hash-registrations", "hash-matched-dependencies")):
        raise ValueError("Unsafe linked literal path")
    data = (literal_directory / item["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Linked literal hash mismatch")
    parent = (root / "recovered" / item["parent_collection"] / "objects" / parent_sha).read_bytes()
    if hashlib.sha256(parent).hexdigest() != parent_sha:
        raise ValueError("Linked literal parent hash mismatch")
    def cites_literal(value):
        if isinstance(value, dict):
            return ((value.get("path") == item["cited_path"] and value.get("sha256") == sha)
                    or any(cites_literal(child) for child in value.values()))
        if isinstance(value, list):
            return any(cites_literal(child) for child in value)
        return False
    if not cites_literal(json.loads(parent)):
        raise ValueError("Linked literal parent citation mismatch")
    literal_count += 1
filename_directory = root / "recovered/printed-filename-json"
filename_manifest = json.loads((filename_directory / "MANIFEST.json").read_text(encoding="utf-8"))
filename_count = 0
for item in filename_manifest["files"]:
    sha = item["sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
            or item["file"] != "objects/" + sha
            or Path(item["printed_filename"]).name != item["printed_filename"]):
        raise ValueError("Unsafe printed-filename object path")
    if hashlib.sha256((filename_directory / item["file"]).read_bytes()).hexdigest() != sha:
        raise ValueError("Printed-filename JSON hash mismatch")
    if (item["printed_filename"] not in item["printed_reference"]
            or sha not in item["printed_reference"].lower()
            or not isinstance(item["reference_ordinal"], int)):
        raise ValueError("Invalid printed-filename provenance")
    filename_count += 1
receipt_dir = root / "recovered/historical-remote-receipts"
receipt_manifest = json.loads((receipt_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if receipt_manifest["schema"] != "726-historical-remote-receipts-v1" or len(receipt_manifest["files"]) != 1:
    raise ValueError("Invalid historical receipt manifest")
for item in receipt_manifest["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe historical receipt path")
    data = (receipt_dir / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Historical receipt hash mismatch")
    if item["historical_copy_command_ordinal"] != 419017:
        raise ValueError("Invalid historical receipt provenance")

receipt_deps = root / "recovered/historical-receipt-dependencies"
deps_manifest = json.loads((receipt_deps / "MANIFEST.json").read_text(encoding="utf-8"))
if deps_manifest["schema"] != "726-historical-receipt-dependencies-v1" or len(deps_manifest["references"]) != 3:
    raise ValueError("Invalid historical receipt dependency manifest")
matched_deps = 0
for item in deps_manifest["references"]:
    parent = item["parent_sha256"]
    if len(parent) != 64 or any(c not in "0123456789abcdef" for c in parent):
        raise ValueError("Unsafe receipt dependency parent")
    cited = json.loads((receipt_dir / parent).read_bytes())[item["parent_pointer"].lstrip("/")]
    if cited["sha256"] != item["cited_sha256"] or cited["record"] != item["record"]:
        raise ValueError("Receipt dependency citation mismatch")
    if item["matches_cited_sha256"]:
        sha = item["cited_sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
            raise ValueError("Unsafe receipt dependency path")
        data = (receipt_deps / sha).read_bytes()
        if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha or item["candidate_sha256"] != sha:
            raise ValueError("Receipt dependency hash mismatch")
        matched_deps += 1
    elif item["candidate_sha256"] == item["cited_sha256"] or "object" in item:
        raise ValueError("Mismatched candidate presented as recovered")
if matched_deps != 1:
    raise ValueError("Unexpected receipt dependency count")

for item in deps_manifest.get("unhashed_citation_candidates", []):
    parent = item["parent_sha256"]
    sha = item["candidate_sha256"]
    if any(len(x) != 64 or any(c not in "0123456789abcdef" for c in x) for x in (parent, sha)):
        raise ValueError("Unsafe unhashed citation object path")
    cited = json.loads((receipt_deps / parent).read_bytes())
    if cited[item["parent_pointer"].lstrip("/")] != item["record"] or item["object"] != sha:
        raise ValueError("Unhashed candidate citation mismatch")
    data = (receipt_deps / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Unhashed candidate object mismatch")

linked_original_dir = root / "recovered/frontier-linked-original-dependencies"
linked_original = json.loads((linked_original_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if linked_original["schema"] != "726-frontier-linked-original-dependencies-v1" or len(linked_original["files"]) != 3:
    raise ValueError("Invalid linked original frontier dependencies")
for item in linked_original["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe linked original object path")
    if item["parent_file"] != "recovered/transcript-queue-fragments/queue-4151-entry-2708.json":
        raise ValueError("Unsafe linked original parent")
    parent = json.loads((root / item["parent_file"]).read_bytes())
    for key in item["parent_pointer"].strip("/").split("/"):
        parent = parent[int(key)] if isinstance(parent, list) else parent[key]
    if parent != {"path": item["path"], "sha256": sha}:
        raise ValueError("Linked original parent citation mismatch")
    data = (linked_original_dir / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Linked original object mismatch")

terminal_dir = root / "recovered/terminal-registration-dependency"
terminal = json.loads((terminal_dir / "MANIFEST.json").read_text(encoding="utf-8"))
sha = terminal["sha256"]
if (terminal["schema"] != "726-terminal-registration-dependency-v1"
        or len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
        or terminal["object"] != sha
        or terminal["parent_file"] != "recovered/transcript-hash-matched/QUALIFICATION-TERMINAL-BOUNDARY-L13-3802.json"
        or terminal["parent_pointer"] != "/project_registrations/1"):
    raise ValueError("Invalid terminal registration provenance")
data = (terminal_dir / sha).read_bytes()
parent = (root / terminal["parent_file"]).read_bytes()
if (len(data) != terminal["bytes"] or hashlib.sha256(data).hexdigest() != sha
        or hashlib.sha256(parent).hexdigest() != terminal["parent_sha256"]
        or json.loads(parent)["project_registrations"][1] != {"path": terminal["historical_path"], "sha256": sha}):
    raise ValueError("Terminal registration citation mismatch")

terminal_trace_dir = root / "recovered/terminal-source-trace"
terminal_trace = json.loads((terminal_trace_dir / "TRACE.json").read_text(encoding="utf-8"))
terminal_sources = json.loads((terminal_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (terminal_trace["schema"] != "726-terminal-source-trace-v1"
        or len(terminal_trace["observations"]) != 3 or len(terminal_sources) != 3
        or sum(x["printed_lines_verified"] for x in terminal_trace["observations"]) != 475
        or len({x["sha256"] for x in terminal_sources}) != 2):
    raise ValueError("Invalid terminal source trace")
if terminal_trace["qualification_file"] != terminal["parent_file"] or terminal_trace["qualification_sha256"] != terminal["parent_sha256"]:
    raise ValueError("Terminal source qualification mismatch")
terminal_qualification = json.loads(parent)
for obs, source in zip(terminal_trace["observations"], terminal_sources):
    if (obs["revision"] != source["revision"] or obs["git_path"] != source["git_path"]
            or obs["source_sha256"] != source["sha256"]
            or sum(b - a + 1 for a, b in obs["ranges"]) != obs["printed_lines_verified"]
            or not any(x["registration"] == obs["registration"] and x["row_index"] == obs["row_index"]
                       and x["ranges"] == obs["ranges"] for x in terminal_qualification["project_observations"])):
        raise ValueError("Terminal source observation mismatch")

cited_regs_dir = root / "recovered/cited-reconstructed-registrations"
cited_regs = json.loads((cited_regs_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if cited_regs["schema"] != "726-cited-reconstructed-registrations-v1" or len(cited_regs["files"]) != 15:
    raise ValueError("Invalid cited registration manifest")
for item in cited_regs["files"]:
    sha = item["sha256"]
    parent_path = Path(item["parent_file"])
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
            or item["object"] != sha or parent_path.is_absolute() or ".." in parent_path.parts
            or parent_path.parts[:2] not in (("recovered", "transcript-hash-matched"),
                                            ("recovered", "printed-reference-json"),
                                            ("recovered", "hash-matched-dependencies"))):
        raise ValueError("Unsafe cited registration path")
    data = (cited_regs_dir / sha).read_bytes()
    parent_data = (root / parent_path).read_bytes()
    if (len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha
            or hashlib.sha256(parent_data).hexdigest() != item["parent_sha256"]):
        raise ValueError("Cited registration hash mismatch")
    parent_value = json.loads(parent_data)
    for token in item["parent_pointer"].strip("/").split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        parent_value = parent_value[int(token)] if isinstance(parent_value, list) else parent_value[token]
    if parent_value != {"path": item["historical_path"], "sha256": sha}:
        raise ValueError("Cited registration parent reference mismatch")

lock_lineage = json.loads((cited_regs_dir / "LOCK-LINEAGE.json").read_text(encoding="utf-8"))
if (lock_lineage["schema"] != "726-lock-registration-lineage-v1"
        or lock_lineage["old_file"] != "recovered/cited-reconstructed-registrations/9d5c1a0cd144cfc7b6965f6168a040e0e52a92c3b5d45c47661bd7eef331b963"
        or lock_lineage["successor_file"] != "recovered/transcript-hash-matched/a67fc16fef00--REGISTRATION-LOCK-FULL-682.json"):
    raise ValueError("Invalid lock registration lineage")
old_data = (root / lock_lineage["old_file"]).read_bytes()
new_data = (root / lock_lineage["successor_file"]).read_bytes()
if (hashlib.sha256(old_data).hexdigest() != lock_lineage["old_sha256"]
        or hashlib.sha256(new_data).hexdigest() != lock_lineage["successor_sha256"]):
    raise ValueError("Lock registration lineage hash mismatch")
old_lock, new_lock = json.loads(old_data), json.loads(new_data)
expected_rows = []
for row in old_lock["rows"]:
    row = {key: value for key, value in row.items() if key != "scope"}
    row["prospective_ranges"] = [[1, row["actual_EOF"]]]
    expected_rows.append(row)
if (new_lock["rows"] != expected_rows or new_lock["lenses"] != list(range(1, 14))
        or new_lock["purpose"] != "Full before and after lockfile review for remaining revision-specific cells; focused package observations remain separate."):
    raise ValueError("Lock registration derivation mismatch")

scatter_dir = root / "recovered/scatter-registration-parent"
scatter = json.loads((scatter_dir / "MANIFEST.json").read_text(encoding="utf-8"))
scatter_sha = scatter["sha256"]
if (scatter["schema"] != "726-scatter-registration-parent-v1" or len(scatter_sha) != 64
        or any(c not in "0123456789abcdef" for c in scatter_sha) or scatter["object"] != scatter_sha):
    raise ValueError("Invalid scatter parent")
scatter_data = (scatter_dir / scatter_sha).read_bytes()
projection_sha = "ca254720e3a691693ec497c0f02c6f536cce74ec193d8135f808eac0b24ad7e9"
projection = json.loads((cited_regs_dir / projection_sha).read_bytes())
if (len(scatter_data) != scatter["bytes"] or hashlib.sha256(scatter_data).hexdigest() != scatter_sha
        or projection["parent_prospective_registration"] != {"path": scatter["historical_path"], "sha256": scatter_sha}
        or projection["rows"] != [x for x in json.loads(scatter_data)["rows"] if x["git_path"].endswith(".wgsl")]):
    raise ValueError("Scatter parent citation or projection mismatch")

printed_parent_dir = root / "recovered/printed-parent-registration"
printed_reg = json.loads((printed_parent_dir / "MANIFEST.json").read_text(encoding="utf-8"))
printed_sha = printed_reg["sha256"]
if (printed_reg["schema"] != "726-printed-parent-registration-v1" or len(printed_sha) != 64
        or any(c not in "0123456789abcdef" for c in printed_sha) or printed_reg["object"] != printed_sha
        or printed_reg["parent_pointer"] != "/registration"):
    raise ValueError("Invalid printed-parent registration")
printed_data = (printed_parent_dir / printed_sha).read_bytes()
if len(printed_data) != printed_reg["bytes"] or hashlib.sha256(printed_data).hexdigest() != printed_sha:
    raise ValueError("Printed-parent registration hash mismatch")
with (root / "recovered/printed-audit-documents/VALUES.jsonl").open(encoding="utf-8") as stream:
    parents = [json.loads(line)["value"] for line in stream
               if printed_reg["parent_normalized_sha256"] in line]
if len(parents) != 1 or parents[0]["registration"] != {"path": printed_reg["historical_path"], "sha256": printed_sha}:
    raise ValueError("Printed-parent citation mismatch")

queue_reg_dir = root / "recovered/queue-output-registration"
queue_reg = json.loads((queue_reg_dir / "MANIFEST.json").read_text(encoding="utf-8"))
queue_sha = queue_reg["sha256"]
if (queue_reg["schema"] != "726-queue-output-registration-v1" or len(queue_sha) != 64
        or any(c not in "0123456789abcdef" for c in queue_sha) or queue_reg["object"] != queue_sha):
    raise ValueError("Invalid queue-output registration")
queue_data = (queue_reg_dir / queue_sha).read_bytes()
if len(queue_data) != queue_reg["bytes"] or hashlib.sha256(queue_data).hexdigest() != queue_sha:
    raise ValueError("Queue-output registration hash mismatch")
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf-8") as stream:
    outputs = [json.loads(line) for line in stream if '"transcript_ordinal":103874' in line]
pattern = r'"path"\s*:\s*"' + re.escape(queue_reg["historical_path"]) + r'"\s*,\s*"sha256"\s*:\s*"' + queue_sha + r'"'
if (len(outputs) != 1 or outputs[0]["published_output_sha256"] != queue_reg["source_output_sha256"]
        or re.search(pattern, outputs[0]["output_text"]) is None):
    raise ValueError("Queue-output registration citation mismatch")

output_regs_dir = root / "recovered/output-cited-registrations"
output_regs = json.loads((output_regs_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if output_regs["schema"] != "726-output-cited-registrations-v1" or len(output_regs["files"]) != 8:
    raise ValueError("Invalid output-cited registration manifest")
output_cache = {}
for item in output_regs["files"]:
    sha = item["sha256"]
    source_file = item["source_output_file"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha
            or source_file not in ("recovered/queue-command-outputs/RECORDS.jsonl", "recovered/source-registration-outputs/RECORDS-06.jsonl")
            or Path(item["historical_basename"]).name != item["historical_basename"]):
        raise ValueError("Unsafe output-cited registration path")
    data = (output_regs_dir / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Output-cited registration hash mismatch")
    if source_file not in output_cache:
        with (root / source_file).open(encoding="utf-8") as stream:
            needed = {x["source_output_ordinal"] for x in output_regs["files"] if x["source_output_file"] == source_file}
            output_cache[source_file] = {x["transcript_ordinal"]: x for x in map(json.loads, stream)
                                         if x["transcript_ordinal"] in needed}
    output = output_cache[source_file][item["source_output_ordinal"]]["output_text"]
    if hashlib.sha256(output.encode()).hexdigest() != item["source_output_sha256"]:
        raise ValueError("Output-cited registration source mismatch")
    if item["citation_kind"] == "sliced-queue-output":
        pattern = re.escape(item["historical_basename"]) + r'"\s*,\s*"sha256"\s*:\s*"' + sha + r'"'
        if re.search(pattern, output) is None:
            raise ValueError("Sliced queue registration citation mismatch")
    elif item["citation_kind"] == "printed-python-repr":
        pattern = re.escape(item["historical_basename"]) + r"'\s*,\s*'sha256'\s*:\s*'" + sha + r"'"
        if re.search(pattern, output) is None:
            raise ValueError("Printed representation registration citation mismatch")
    elif item["citation_kind"] == "direct-registration-output":
        recorded = json.loads(output)
        if recorded["registration"].replace("\\", "/").split("/")[-1] != item["historical_basename"] or recorded["sha256"] != sha:
            raise ValueError("Direct registration citation mismatch")
    else:
        raise ValueError("Invalid registration citation kind")

python_qual_dir = root / "recovered/printed-python-qualifications"
python_quals = json.loads((python_qual_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if python_quals["schema"] != "726-printed-python-qualifications-v1" or len(python_quals["files"]) != 1:
    raise ValueError("Invalid printed qualification manifest")
for item in python_quals["files"]:
    sha = item["normalized_sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha
            or item["source_output_file"] != "recovered/source-registration-outputs/RECORDS-06.jsonl"
            or item["historical_basename"] != "QUALIFICATION-REDUCER-ROLLBACK-L11-1960.json"):
        raise ValueError("Unsafe printed qualification path")
    data = (python_qual_dir / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Printed qualification hash mismatch")
    output = output_cache[item["source_output_file"]][item["source_output_ordinal"]]["output_text"]
    if hashlib.sha256(output.encode()).hexdigest() != item["source_output_sha256"]:
        raise ValueError("Printed qualification output mismatch")
    marker_text = item["historical_basename"] + " "
    printed = output.split(marker_text, 1)[1].strip()
    if (len(printed) != item["printed_chars"] or len(printed) >= item["display_limit"]
            or ast.literal_eval(printed) != json.loads(data)
            or len(json.loads(data)["project_observations"]) != item["project_observations"]):
        raise ValueError("Printed qualification representation mismatch")

supplemental_dir = root / "recovered/supplemental-registration-sources"
supplemental = json.loads((supplemental_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if len(supplemental) != 43 or len({(x["revision"], x["git_path"], x["sha256"]) for x in supplemental}) != 43:
    raise ValueError("Invalid supplemental source identities")
for row in supplemental:
    for ref in row["registration_refs"]:
        path = Path(ref["registration_object"])
        if (path.is_absolute() or ".." in path.parts or path.parts[0] != "recovered"
                or len(path.parts) != 3 or path.name != ref["registration_sha256"]):
            raise ValueError("Unsafe supplemental registration reference")
        data = (root / path).read_bytes()
        if hashlib.sha256(data).hexdigest() != ref["registration_sha256"]:
            raise ValueError("Supplemental registration hash mismatch")
        source = json.loads(data)["rows"][ref["row_index"]]
        if (source["revision"] != row["revision"] or source["git_path"] != row["git_path"]
                or source["source_sha256"] != row["sha256"] or source["actual_EOF"] != row["actual_eof"]):
            raise ValueError("Supplemental registration source mismatch")

native_dir = root / "recovered/native-bevy-source-objects"
native_manifest = json.loads((native_dir / "MANIFEST.json").read_text(encoding="utf-8"))
registration_name = "recovered/cited-reconstructed-registrations/fc47d7de6bddfa8d18d734454d8d19fc9f41c5fe3a276b3202f4b24b0786854d"
if (native_manifest["schema"] != "726-native-bevy-source-objects-v1"
        or native_manifest["registration_file"] != registration_name or len(native_manifest["files"]) != 2):
    raise ValueError("Invalid native source manifest")
registration_bytes = (root / registration_name).read_bytes()
if hashlib.sha256(registration_bytes).hexdigest() != native_manifest["registration_sha256"]:
    raise ValueError("Native source registration mismatch")
native_registration = json.loads(registration_bytes)
for item in native_manifest["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe native source object path")
    row = native_registration["rows"][item["registration_row"]]
    data = (native_dir / sha).read_bytes()
    if (len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha
            or len(data.splitlines()) != item["actual_eof"] or row["actual_EOF"] != item["actual_eof"]
            or row["source_sha256"] != sha or row["archive_member"] != item["archive_member"]
            or row["archive_checksum"] != native_manifest["archive_sha256"]):
        raise ValueError("Native source object citation mismatch")

native_archives_dir = root / "recovered/native-archive-source-objects"
native_archives = json.loads((native_archives_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if native_archives["schema"] != "726-native-archive-source-objects-v1" or len(native_archives["files"]) != 10:
    raise ValueError("Invalid native archive source manifest")
for item in native_archives["files"]:
    sha = item["sha256"]
    reg_path = Path(item["registration_file"])
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha
            or reg_path.is_absolute() or ".." in reg_path.parts or reg_path.parts[0] != "recovered"):
        raise ValueError("Unsafe native archive object or registration path")
    parent_bytes = (root / reg_path).read_bytes()
    if hashlib.sha256(parent_bytes).hexdigest() != item["registration_sha256"]:
        raise ValueError("Native archive registration hash mismatch")
    row = json.loads(parent_bytes)["rows"][item["registration_row"]]
    data = (native_archives_dir / sha).read_bytes()
    if (len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha
            or len(data.splitlines()) != item["actual_eof"] or row["actual_EOF"] != item["actual_eof"]
            or row["source_sha256"] != sha or row["package"] != item["package"]
            or row["version"] != item["version"] or row["archive_member"] != item["archive_member"]
            or row["archive_checksum"] != item["archive_sha256"]):
        raise ValueError("Native archive source citation mismatch")

tangent_dir = root / "recovered/native-tangent-source-objects"
tangent = json.loads((tangent_dir / "MANIFEST.json").read_text(encoding="utf-8"))
tangent_parent = "recovered/transcript-hash-matched/REGISTRATION-ROOT-L12-ACTIVE-NATIVE-TANGENT-QUALITY-001.json"
if (tangent["schema"] != "726-native-tangent-source-objects-v1"
        or tangent["registration_file"] != tangent_parent or len(tangent["files"]) != 6):
    raise ValueError("Invalid native tangent source manifest")
parent_data = (root / tangent_parent).read_bytes()
if hashlib.sha256(parent_data).hexdigest() != tangent["registration_sha256"]:
    raise ValueError("Native tangent registration hash mismatch")
parent_value = json.loads(parent_data)
for item in tangent["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe native tangent source path")
    row = parent_value["sources"][item["registration_source_index"]]
    data = (tangent_dir / sha).read_bytes()
    if (len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha
            or len(data.splitlines()) != item["actual_eof"] or row["actual_EOF"] != item["actual_eof"]
            or row["source_sha256"] != sha or row["archive_member"] != item["archive_member"]
            or row["archive"]["sha256"] != tangent["archive_sha256"]):
        raise ValueError("Native tangent source citation mismatch")

printed_native_dir = root / "recovered/printed-native-source-objects"
printed_native = json.loads((printed_native_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if printed_native["schema"] != "726-printed-native-source-objects-v1" or len(printed_native["files"]) != 53:
    raise ValueError("Invalid printed native source manifest")
printed_native_parents = {}
for parent_file in {item["parent_values_file"] for item in printed_native["files"]}:
    if parent_file not in ("recovered/printed-audit-documents/VALUES.jsonl", "recovered/transcript-queue-fragments/PRINTED-VALUES.jsonl"):
        raise ValueError("Unsafe printed native parent collection")
    needed = {item["parent_normalized_sha256"] for item in printed_native["files"] if item["parent_values_file"] == parent_file}
    with (root / parent_file).open(encoding="utf-8") as stream:
        for line in stream:
            item = json.loads(line)
            if item["normalized_sha256"] in needed:
                printed_native_parents[(parent_file, item["normalized_sha256"])] = item["value"]
for item in printed_native["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe printed native source path")
    data = (printed_native_dir / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Printed native source hash mismatch")
    parent = printed_native_parents[(item["parent_values_file"], item["parent_normalized_sha256"])]
    for token in item["parent_pointer"].strip("/").split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        parent = parent[int(token)] if isinstance(parent, list) else parent[token]
    checksum = parent.get("archive_checksum") or parent.get("archive", {}).get("sha256")
    if (parent["source_sha256"] != sha or parent["archive_member"] != item["archive_member"]
            or checksum != item["archive_sha256"]):
        raise ValueError("Printed native source citation mismatch")

raw_native_dir = root / "recovered/raw-output-native-source-objects"
raw_native = json.loads((raw_native_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (raw_native["schema"] != "726-raw-output-native-source-objects-v1" or len(raw_native["files"]) != 3
        or raw_native["source_output_file"] != "recovered/source-registration-outputs/RECORDS-05.jsonl"
        or raw_native["source_output_ordinal"] != 205791):
    raise ValueError("Invalid raw native source manifest")
with (root / raw_native["source_output_file"]).open(encoding="utf-8") as stream:
    outputs = [json.loads(line) for line in stream if '"transcript_ordinal":205791' in line]
if len(outputs) != 1 or hashlib.sha256(outputs[0]["output_text"].encode()).hexdigest() != raw_native["source_output_sha256"]:
    raise ValueError("Raw native source output mismatch")
raw_rows = json.loads(outputs[0]["output_text"])
for item in raw_native["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe raw native source path")
    row = raw_rows[item["output_row"]]
    data = (raw_native_dir / sha).read_bytes()
    if (len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha
            or len(data.splitlines()) != item["actual_eof"] or row["actual_EOF"] != item["actual_eof"]
            or row["source_sha256"] != sha or row["archive_member"] != item["archive_member"]
            or row["archive_checksum"] != raw_native["archive_sha256"]):
        raise ValueError("Raw native source citation mismatch")

partial_native_dir = root / "recovered/partial-output-native-source-objects"
partial_native = json.loads((partial_native_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if partial_native["schema"] != "726-partial-output-native-source-objects-v1" or len(partial_native["files"]) != 7:
    raise ValueError("Invalid partial-output native source manifest")
needed = {item["source_output_ordinal"] for item in partial_native["files"]}
with (root / "recovered/source-registration-outputs/RECORDS-06.jsonl").open(encoding="utf-8") as stream:
    partial_outputs = {x["transcript_ordinal"]: x["output_text"] for x in map(json.loads, stream)
                       if x["transcript_ordinal"] in needed}
for item in partial_native["files"]:
    sha = item["sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha
            or item["source_output_file"] != "recovered/source-registration-outputs/RECORDS-06.jsonl"):
        raise ValueError("Unsafe partial native source path")
    data = (partial_native_dir / sha).read_bytes()
    output = partial_outputs[item["source_output_ordinal"]]
    if (len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha
            or len(data.splitlines()) != item["actual_eof"]
            or hashlib.sha256(output.encode()).hexdigest() != item["source_output_sha256"]):
        raise ValueError("Partial native source hash mismatch")
    row = json.loads(output[item["row_start"]:item["row_end"]])
    if (row["source_sha256"] != sha or row["archive_member"] != item["archive_member"]
            or row["archive_checksum"] != item["archive_sha256"] or row["actual_EOF"] != item["actual_eof"]):
        raise ValueError("Partial native source row mismatch")

raw_git_dir = root / "recovered/raw-output-git-sources"
raw_git = json.loads((raw_git_dir / "MANIFEST.json").read_text(encoding="utf-8"))
raw_external_dir = root / "recovered/raw-output-external-sources"
raw_external = json.loads((raw_external_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if len(raw_git) != 248 or raw_external["schema"] != "726-raw-output-external-sources-v1" or len(raw_external["files"]) != 2:
    raise ValueError("Invalid raw-output source collections")
raw_source_rows = raw_git + raw_external["files"]
raw_output_cache = {}
for source_file in {x["source_output_file"] for x in raw_source_rows}:
    source_path = Path(source_file)
    if (source_path.is_absolute() or ".." in source_path.parts or source_path.parts[0] != "recovered"
            or source_path.parts[1] not in ("source-registration-outputs", "queue-command-outputs", "ledger-command-outputs")):
        raise ValueError("Unsafe raw source output path")
    needed = {x["source_output_ordinal"] for x in raw_source_rows if x["source_output_file"] == source_file}
    with (root / source_path).open(encoding="utf-8") as stream:
        for line in stream:
            item = json.loads(line)
            if item["transcript_ordinal"] in needed:
                raw_output_cache[(source_file, item["transcript_ordinal"])] = item["output_text"]
for item in raw_source_rows:
    output = raw_output_cache[(item["source_output_file"], item["source_output_ordinal"])]
    if hashlib.sha256(output.encode()).hexdigest() != item["source_output_sha256"]:
        raise ValueError("Raw Git source output mismatch")
    row = json.loads(output[item["row_start"]:item["row_end"]])
    if (row["revision"] != item["revision"] or row["git_path"] != item["git_path"]
            or row["source_sha256"] != item["sha256"]):
        raise ValueError("Raw Git source identity mismatch")
for item in raw_external["files"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["object"] != sha:
        raise ValueError("Unsafe raw external source path")
    data = (raw_external_dir / sha).read_bytes()
    if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Raw external source hash mismatch")

alternate_sources = json.loads((root / "recovered/alternate-output-git-sources/MANIFEST.json").read_text(encoding="utf-8"))
if len(alternate_sources) != 15 or len({(x["revision"], x["git_path"], x["sha256"]) for x in alternate_sources}) != 15:
    raise ValueError("Invalid alternate output source identities")
alternate_outputs = {}
for source_file in {x["source_output_file"] for x in alternate_sources}:
    source_path = Path(source_file)
    if (source_path.is_absolute() or ".." in source_path.parts or source_path.parts[0] != "recovered"
            or source_path.parts[1] not in ("source-registration-outputs", "queue-command-outputs", "ledger-command-outputs")):
        raise ValueError("Unsafe alternate source output path")
    needed = {x["source_output_ordinal"] for x in alternate_sources if x["source_output_file"] == source_file}
    with (root / source_path).open(encoding="utf-8") as stream:
        for line in stream:
            item = json.loads(line)
            if item["transcript_ordinal"] in needed:
                alternate_outputs[(source_file, item["transcript_ordinal"])] = item["output_text"]
for item in alternate_sources:
    if item["path_field"] not in ("git_path", "path") or item["sha_field"] not in ("source_sha256", "sha256"):
        raise ValueError("Invalid alternate source field")
    output = alternate_outputs[(item["source_output_file"], item["source_output_ordinal"])]
    if hashlib.sha256(output.encode()).hexdigest() != item["source_output_sha256"]:
        raise ValueError("Alternate source output mismatch")
    row = json.loads(output[item["row_start"]:item["row_end"]])
    if (row["revision"] != item["revision"] or row[item["path_field"]] != item["git_path"]
            or row[item["sha_field"]] != item["sha256"]):
        raise ValueError("Alternate source row mismatch")

source_counts = json.loads((root / "SOURCE-COUNTS.json").read_text(encoding="utf-8"))
expected_git_collections = ["registered-source-objects", "printed-source-identities", "supplemental-registration-sources", "frontier-source-trace", "terminal-source-trace", "raw-output-git-sources", "alternate-output-git-sources"]
expected_dependency_collections = ["native-bevy-source-objects", "native-archive-source-objects", "native-tangent-source-objects", "printed-native-source-objects", "raw-output-native-source-objects", "partial-output-native-source-objects", "raw-output-external-sources"]
if (source_counts["schema"] != "726-source-counts-v1" or source_counts["git_manifest_collections"] != expected_git_collections
        or source_counts["preserved_dependency_collections"] != expected_dependency_collections):
    raise ValueError("Invalid source count collection list")
source_identities = set()
for collection in expected_git_collections:
    for row in json.loads((root / "recovered" / collection / "MANIFEST.json").read_text(encoding="utf-8")):
        source_identities.add((row["revision"], row["git_path"], row["sha256"]))
git_blobs = {row[2] for row in source_identities}
dependency_blobs = set()
for collection in expected_dependency_collections:
    manifest = json.loads((root / "recovered" / collection / "MANIFEST.json").read_text(encoding="utf-8"))
    dependency_blobs.update(row["sha256"] for row in manifest["files"])
identity_bytes = (json.dumps(sorted(source_identities), ensure_ascii=False, separators=(",", ":")) + "\n").encode()
if (len(source_identities) != source_counts["git_identity_count"] or len(git_blobs) != source_counts["git_blob_count"]
        or len(dependency_blobs) != source_counts["preserved_dependency_blob_count"]
        or len(git_blobs & dependency_blobs) != source_counts["git_and_dependency_blob_overlap"]
        or hashlib.sha256(identity_bytes).hexdigest() != source_counts["git_identity_set_sha256"]):
    raise ValueError("Source count reconciliation mismatch")

draft_bundle_dir = root / "recovered/draft-git-bundle"
draft_bundle = json.loads((draft_bundle_dir / "MANIFEST.json").read_text(encoding="utf-8"))
draft_history = json.loads((root / "recovered/draft-filechanges/MANIFEST.json").read_text(encoding="utf-8"))
draft_bundle_data = (draft_bundle_dir / "drafts.bundle").read_bytes()
if (draft_bundle["schema"] != "726-incremental-draft-git-bundle-v1"
        or draft_bundle["base_commit"] != "ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04"
        or len(draft_bundle["drafts"]) != 8 or len(draft_bundle_data) != draft_bundle["bundle_bytes"]
        or hashlib.sha256(draft_bundle_data).hexdigest() != draft_bundle["bundle_sha256"]
        or not {(x["branch"], x["pinned_commit"]) for x in draft_history["drafts"]}
           <= {(x["branch"], x["commit"]) for x in draft_bundle["drafts"]}
        or {(x["branch"], x["commit"]) for x in draft_bundle["drafts"]}
           != {(x["branch"], x["commit"]) for x in json.loads((root / "SOURCE-LOCATORS.json").read_text(encoding="utf-8"))["published_implementation_drafts"]}):
    raise ValueError("Draft bundle manifest mismatch")

selection_dir = root / "recovered/selected-lens-440-values"
selection = json.loads((selection_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if selection["schema"] != "726-selected-lens-440-values-v1" or [x["lens"] for x in selection["values"]] != [1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13]:
    raise ValueError("Invalid selected-lens value manifest")
selection_values = {}
for item in selection["values"]:
    sha = item["normalized_sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or item["file"] != sha + ".json":
        raise ValueError("Unsafe selected-lens value path")
    data = (selection_dir / item["file"]).read_bytes()
    fragment = item["source_fragment"]
    if fragment not in ("OUTPUT-347151.txt", "OUTPUT-369225.txt", "OUTPUT-84871.txt", "OUTPUT-106325.txt"):
        raise ValueError("Unsafe selected-lens fragment")
    source = (selection_dir / fragment).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or hashlib.sha256(source).hexdigest() != item["source_fragment_sha256"]:
        raise ValueError("Selected-lens value hash mismatch")
    text = source.decode("utf-8")
    if fragment == "OUTPUT-347151.txt":
        begin = text.index("{", text.index('"lenses"'))
        first, end = json.JSONDecoder().raw_decode(text, begin)
        second, _ = json.JSONDecoder().raw_decode(text, text.index("{", end))
        printed = {1: first, 2: second}[item["lens"]]
    elif fragment == "OUTPUT-84871.txt":
        position = text.index("{", text.index('"lenses"'))
        printed_values = {}
        while position >= 0:
            try:
                candidate, end = json.JSONDecoder().raw_decode(text, position)
            except ValueError:
                break
            printed_values[candidate["lens"]] = candidate
            position = text.find("{", end)
        printed = printed_values[item["lens"]]
    elif fragment == "OUTPUT-106325.txt":
        import ast
        printed = {v["lens"]: v for v in map(ast.literal_eval, text.splitlines())}[item["lens"]]
    else:
        printed = json.loads(text)[0]
    value = json.loads(data)
    if value != printed or value["selected_state"] != item["selected_state"]:
        raise ValueError("Selected-lens printed value mismatch")
    selection_values[item["lens"]] = value
lens4 = json.loads((root / selection["previously_preserved_lens4"]).read_bytes())
if lens4["lens"] != 4 or selection["lens4_state_object"] != lens4["selected_state"]["sha256"]:
    raise ValueError("Selected Lens 4 state mismatch")
if hashlib.sha256((selection_dir / selection["lens4_state_object"]).read_bytes()).hexdigest() != selection["lens4_state_object"]:
    raise ValueError("Selected Lens 4 source-state hash mismatch")
if [x["lens"] for x in selection["additional_verified_state_objects"]] != [3, 5, 6, 7, 8, 9, 10, 11]:
    raise ValueError("Invalid additional selected states")
for item in selection["additional_verified_state_objects"]:
    ref = selection_values[item["lens"]]["selected_state"]
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or ref["sha256"] != sha:
        raise ValueError("Unsafe additional selected state path")
    data = (selection_dir / sha).read_bytes()
    if len(data) != item["bytes"] or len(data) != ref["bytes"] or hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Additional selected state mismatch")
if selection["lens1_state_object"] != "recovered/printed-reference-json/objects/" + selection_values[1]["selected_state"]["sha256"]:
    raise ValueError("Selected Lens 1 state locator mismatch")
if hashlib.sha256((root / selection["lens1_state_object"]).read_bytes()).hexdigest() != selection_values[1]["selected_state"]["sha256"]:
    raise ValueError("Selected Lens 1 source-state hash mismatch")

snapshot = selection["exact_original_snapshot"]
snapshot_sha = snapshot["sha256"]
if snapshot_sha != "eccc459c300b5faddd10db40aec1bfa7b0f59d91f58e5243760680670f51d232" or snapshot["file"] != snapshot_sha:
    raise ValueError("Unsafe selected snapshot locator")
snapshot_bytes = (selection_dir / snapshot_sha).read_bytes()
if hashlib.sha256(snapshot_bytes).hexdigest() != snapshot_sha:
    raise ValueError("Selected snapshot original hash mismatch")
original_snapshot = json.loads(snapshot_bytes)
expected_lenses = dict(selection_values)
expected_lenses[4] = lens4
if (original_snapshot["lenses"] != [expected_lenses[n] for n in range(1, 14)]
        or original_snapshot["at"] != snapshot["timestamp"]
        or original_snapshot["all_lenses_clear"] is not False
        or original_snapshot["implementation_gate_complete"] is not False):
    raise ValueError("Selected snapshot content mismatch")
reference = json.loads((root / snapshot["recorded_hash_reference"]).read_bytes())["base_authority_snapshot"][4]
if reference["sha256"] != snapshot_sha or not reference["path"].endswith("/SELECTED-LENSES-CONTINUATION-440.json"):
    raise ValueError("Selected snapshot cited hash mismatch")

reuse_dir = root / "recovered/l2-errantry-reuse-qualification"
reuse_manifest = json.loads((reuse_dir / "MANIFEST.json").read_bytes())
reuse_sha = reuse_manifest["qualification_sha256"]
if reuse_sha != "c8ac419fd0fe6b38f12eeb8f2338f54331c2589ef46b7f13aa09e31c04904299":
    raise ValueError("Unexpected Lens 2 reuse qualification")
reuse_data = (reuse_dir / reuse_sha).read_bytes()
if hashlib.sha256(reuse_data).hexdigest() != reuse_sha:
    raise ValueError("Lens 2 reuse qualification hash mismatch")
reuse = json.loads(reuse_data)
if len(reuse["rows"]) != 21 or reuse["source_receipts"] != reuse_manifest["original_receipts"]:
    raise ValueError("Lens 2 reuse qualification source mismatch")
for ref in reuse_manifest["original_receipts"]:
    sha = ref["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha) or hashlib.sha256((reuse_dir / sha).read_bytes()).hexdigest() != sha:
        raise ValueError("Lens 2 reuse receipt hash mismatch")
printed_reuse = None
with (root / "recovered/printed-audit-documents/VALUES.jsonl").open(encoding="utf-8") as source:
    for line in source:
        item = json.loads(line)
        if item["normalized_sha256"] == reuse_manifest["printed_reference_normalized_sha256"]:
            printed_reuse = item["value"]
            break
if (printed_reuse is None or printed_reuse["qualification"]["sha256"] != reuse_sha
        or printed_reuse["qualification"]["path"] != reuse_manifest["historical_path"]):
    raise ValueError("Lens 2 reuse printed reference mismatch")

writer_dir = root / "recovered/final-writer-static-trace"
writer_trace = json.loads(_extra_historical_manifest("recovered/final-writer-static-trace/MANIFEST.json"))
base_bytes = (writer_dir / "BASE-3802.txt").read_bytes()
if hashlib.sha256(base_bytes).hexdigest() != writer_trace["base_sha256"]:
    raise ValueError("Historical writer printed base hash mismatch")
writer = base_bytes.decode("utf-8")
if len(writer_trace["steps"]) != 19:
    raise ValueError("Unexpected historical writer transformation count")
frontier_writers = []
for step in writer_trace["steps"]:
    source_tree = ast.parse(step["recorded_source_line"])
    calls = [node for node in ast.walk(source_tree) if isinstance(node, ast.Call)
             and isinstance(node.func, ast.Attribute) and node.func.attr == "replace"
             and ".read_text()" in ast.unparse(node)]
    if not calls:
        raise ValueError("Recorded writer replacement expression missing")
    call = max(calls, key=lambda node: len(ast.unparse(node)))
    recorded_replacements = []
    while isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and call.func.attr == "replace":
        recorded_replacements.append([ast.literal_eval(arg) for arg in call.args])
        call = call.func.value
    if list(reversed(recorded_replacements)) != step["replacements"]:
        raise ValueError("Recorded writer replacements differ from trace")
    if "insertion" in step:
        insertion_ast = ast.parse(step["insertion_source_line"])
        recorded_insertion = {}
        for statement in insertion_ast.body:
            if (isinstance(statement, ast.Assign) and len(statement.targets) == 1
                    and isinstance(statement.targets[0], ast.Name)
                    and statement.targets[0].id in ("needle", "extra")):
                recorded_insertion[statement.targets[0].id] = ast.literal_eval(statement.value)
        if recorded_insertion != step["insertion"]:
            raise ValueError("Recorded writer insertion differs from trace")
    for old, new in step["replacements"]:
        if old not in writer:
            raise ValueError("Historical writer replacement has no source")
        writer = writer.replace(old, new)
    if "insertion" in step:
        insertion = step["insertion"]
        if writer.count(insertion["needle"]) != 1:
            raise ValueError("Historical writer insertion has no unique source")
        writer = writer.replace(insertion["needle"], insertion["needle"] + insertion["extra"])
    if step["transcript_ordinal"] in (419026, 419184, 419288, 419395, 419492):
        required = ("item['entry_index']==2708", "=={2708}",
                    "entry['granular_contract_views']=entry.get('granular_contract_views',[])+[view]",
                    "entry['remaining_evidence']=entry.get('remaining_evidence',[])+decision['remaining_after_discovery']",
                    "view['whole_contract_closed'] is False")
        if not all(fragment in writer for fragment in required):
            raise ValueError("Frontier entry writer derivation mismatch")
        frontier_writers.append(step["transcript_ordinal"])
    if ("selected=check(old['current']['selected_lenses']);selected_bytes=selected.read_bytes()" not in writer
            or "assert selected.read_bytes()==selected_bytes and new['current']['progress']==expected_progress" not in writer
            or "put('current',['selected_lenses']" in writer):
        raise ValueError("Historical writer selection-preservation trace mismatch")
prefix_bytes = (writer_dir / "CURRENT-3802-PREFIX.txt").read_bytes()
reports_bytes = (writer_dir / "APPLY-REPORTS-3803-3821.json").read_bytes()
if (hashlib.sha256(prefix_bytes).hexdigest() != writer_trace["current_3802_prefix_sha256"]
        or hashlib.sha256(reports_bytes).hexdigest() != writer_trace["success_reports_sha256"]):
    raise ValueError("Historical selection lineage evidence hash mismatch")
prefix = prefix_bytes.decode("utf-8")
pos = prefix.index("{", prefix.index('"selected_lenses"'))
printed_ref, _ = json.JSONDecoder().raw_decode(prefix, pos)
if printed_ref != writer_trace["current_3802_selected_lenses"] or printed_ref["sha256"] != snapshot_sha:
    raise ValueError("Historical CURRENT selection reference mismatch")
reports = json.loads(reports_bytes)
if [item["checkpoint"] for item in reports] != list(range(3803, 3822)):
    raise ValueError("Historical checkpoint sequence mismatch")
if len(writer_trace["individual_report_fragments"]) != len(reports):
    raise ValueError("Missing individual historical report fragments")
for item, fragment in zip(reports, writer_trace["individual_report_fragments"]):
    if fragment["checkpoint"] != item["checkpoint"] or fragment["ordinal"] != item["ordinal"] or fragment["file"] != f"REPORT-{item['checkpoint']}.txt":
        raise ValueError("Historical report fragment locator mismatch")
    data = (writer_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"] or [json.loads(line) for line in data.decode("utf-8").splitlines()] != item["records"]:
        raise ValueError("Historical report fragment mismatch")
for item in reports:
    if not all(any(record.get("status") == "pass" and record.get("mode") == mode for record in item["records"]) for mode in ("--preflight", "--apply", "--verify")):
        raise ValueError("Missing recorded checkpoint success report")

candidate_bytes = (writer_dir / "CANDIDATE-3821.py.txt").read_bytes()
if writer.encode("utf-8") != candidate_bytes or hashlib.sha256(candidate_bytes).hexdigest() != writer_trace["candidate_sha256"]:
    raise ValueError("Historical writer derivation mismatch")
for fragment in ("selected=check(old['current']['selected_lenses']);selected_bytes=selected.read_bytes()",
                 "assert selected.read_bytes()==selected_bytes and new['current']['progress']==expected_progress",
                 "cp_path=i/'CHECKPOINT-3821.json'"):
    if fragment not in writer:
        raise ValueError("Historical writer selection evidence missing")
for file, key in (("OUTPUT-419499.txt", "apply_output_sha256"), ("FINAL-BUNDLE-LINE.txt", "bundle_line_sha256")):
    if file == "OUTPUT-419499.txt":
        _extra_public_object("recovered/final-writer-static-trace/" + file, writer_trace[key], (writer_dir / file).read_bytes())
    elif hashlib.sha256((writer_dir / file).read_bytes()).hexdigest() != writer_trace[key]:
        raise ValueError("Historical writer evidence hash mismatch")
if "SELECTED-LENSES-CONTINUATION-440.json" not in (writer_dir / "FINAL-BUNDLE-LINE.txt").read_text(encoding="utf-8"):
    raise ValueError("Historical final bundle selection input missing")

source_test = json.loads((root / "SOURCE-RESTORATION-TEST.json").read_bytes())
source_test_counts = json.loads((root / "SOURCE-COUNTS.json").read_bytes())
if (source_test["schema"] != "726-source-restoration-test-v1"
        or [item["collection"] for item in source_test["results"]] != source_test_counts["git_manifest_collections"]
        or source_test["summed_rows"] != 812 or source_test["distinct_blobs_across_collections"] != source_test_counts["git_blob_count"]):
    raise ValueError("Source restoration test inventory mismatch")
source_test_hashes = set()
for item in source_test["results"]:
    manifest_data = (root / "recovered" / item["collection"] / "MANIFEST.json").read_bytes()
    rows = json.loads(manifest_data)
    hashes = {row["sha256"] for row in rows}
    if (hashlib.sha256(manifest_data).hexdigest() != item["manifest_sha256"]
            or len(rows) != item["identities_listed"] or len(hashes) != item["distinct_blobs"]
            or item["restoration_passed"] is not True or item["existing_destination_refused"] is not True):
        raise ValueError("Source restoration test manifest mismatch")
    source_test_hashes.update(hashes)
if len(source_test_hashes) != source_test["distinct_blobs_across_collections"]:
    raise ValueError("Source restoration test distinct blob mismatch")
# This verifies the recorded test's manifest inputs, not the unavailable test environment.

trace_dir = root / "recovered/frontier-source-trace"
trace = json.loads((trace_dir / "TRACE.json").read_text(encoding="utf-8"))
source_manifest = json.loads((trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if len(trace["sources"]) != 12 or len(source_manifest) != 12:
    raise ValueError("Invalid late source observation count")
if sum(row["verified_recorded_output_lines"] for row in trace["sources"]) != 755:
    raise ValueError("Invalid late source line count")
if len({row["sha256"] for row in source_manifest}) != 7:
    raise ValueError("Invalid late source object count")
for row, source in zip(trace["sources"], source_manifest):
    if (row["revision"] != source["revision"] or row["path"] != source["git_path"]
            or row["source_sha256"] != source["sha256"]
            or row["registration_write_ordinal"] != source["registration_write_ordinal"]
            or row["read_ordinal"] != source["read_ordinal"]
            or row["registration_original_bytes_recovered"] is not False
            or sum(b - a + 1 for a, b in row["ranges"]) != row["verified_recorded_output_lines"]):
        raise ValueError("Inconsistent late source trace")

frontier_templates = json.loads((trace_dir / "REGISTRATION-TEMPLATES.json").read_text(encoding="utf-8"))
frontier_ordinals = {419073, 419106, 419133, 419334, 419346, 419438, 419450}
if (frontier_templates["schema"] != "726-frontier-registration-templates-v1"
        or {item["write_ordinal"] for item in frontier_templates["templates"]} != frontier_ordinals
        or sum(len(item["template"]["rows"]) for item in frontier_templates["templates"]) != 12):
    raise ValueError("Frontier registration template inventory mismatch")
frontier_commands = {}
for source_line in (root / "recovered/late-command-history/RECORDS.jsonl").read_text(encoding="utf-8").splitlines():
    source_record = json.loads(source_line)
    if source_record["transcript_ordinal"] in frontier_ordinals:
        frontier_commands[source_record["transcript_ordinal"]] = source_record
if set(frontier_commands) != frontier_ordinals:
    raise ValueError("Frontier template writer source missing")
for item in frontier_templates["templates"]:
    ordinal = item["write_ordinal"]
    record = frontier_commands[ordinal]
    if record["recorded_exit_code"] != 0:
        raise ValueError("Frontier template writer did not report success")
    source_ast = ast.parse(record["command_text"].split("@'\n", 1)[1].split("\n'@", 1)[0])
    purposes = []
    for node in ast.walk(source_ast):
        if isinstance(node, ast.Dict):
            for key, value in zip(node.keys, node.values):
                if isinstance(key, ast.Constant) and key.value == "purpose" and isinstance(value, ast.Constant):
                    purposes.append(value.value)
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if (isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name)
                        and target.value.id == "j" and isinstance(target.slice, ast.Constant)
                        and target.slice.value == "purpose"):
                    purposes.append(ast.literal_eval(node.value))
    if len(purposes) != 1:
        raise ValueError("Frontier writer purpose missing")
    observed_rows = [row for row in trace["sources"] if row["registration_write_ordinal"] == ordinal]
    if not observed_rows or {row["registration"] for row in observed_rows} != {item["registration"]}:
        raise ValueError("Frontier registration writer identity mismatch")
    expected_rows = [{"revision": row["revision"], "git_path": row["path"],
                      "source_sha256": row["source_sha256"], "actual_EOF": row["actual_eof"],
                      "source_object": "adler-continuation-20261030/objects/" + row["source_sha256"],
                      "prospective_ranges": row["ranges"]} for row in observed_rows]
    expected = {"schema": "726-prospective-current-source-registration-v1", "at": "UNRECOVERED_ORIGINAL_TIMESTAMP",
                "lenses": [4], "purpose": purposes[0], "rows": expected_rows}
    if item["template"] != expected or item["registration"] not in record["command_text"]:
        raise ValueError("Frontier registration template mismatch")
    if ordinal in (419073, 419106, 419334):
        specs = next(ast.literal_eval(node.value) for node in ast.walk(source_ast)
                     if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "specs" for target in node.targets))
        if specs != [(row["path"], row["ranges"]) for row in observed_rows]:
            raise ValueError("Frontier registration original source specifications mismatch")
    if ordinal == 419133:
        path_value = next(ast.literal_eval(node.value) for node in ast.walk(source_ast)
                          if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "path" for target in node.targets))
        range_values = [ast.literal_eval(value) for node in ast.walk(source_ast) if isinstance(node, ast.Dict)
                        for key, value in zip(node.keys, node.values)
                        if isinstance(key, ast.Constant) and key.value == "prospective_ranges"]
        if path_value != observed_rows[0]["path"] or range_values != [observed_rows[0]["ranges"]]:
            raise ValueError("Frontier single source specification mismatch")
    predecessor = {419346: 419334, 419438: 419346, 419450: 419133}.get(ordinal)
    if predecessor is not None:
        earlier = [row for row in trace["sources"] if row["registration_write_ordinal"] == predecessor]
        if not earlier or observed_rows[0]["path"] != earlier[0]["path"] or observed_rows[0]["source_sha256"] != earlier[0]["source_sha256"]:
            raise ValueError("Frontier copied registration source mismatch")
    if ordinal in (419346, 419438, 419450):
        range_node = next(node.value for node in ast.walk(source_ast) if isinstance(node, ast.Assign)
                          and any(isinstance(target, ast.Subscript) and isinstance(target.slice, ast.Constant)
                                  and target.slice.value == "prospective_ranges" for target in node.targets))
        if ast.literal_eval(range_node) != observed_rows[0]["ranges"]:
            raise ValueError("Frontier registration copied source range mismatch")

frontier_timing = json.loads((trace_dir / "REGISTRATION-TIMING.json").read_bytes())
if (frontier_timing["schema"] != "726-frontier-registration-timing-v1"
        or frontier_timing["private_snapshot_sha256"] != "d9fe8bef817fea966bf869df3dbde5dd006f350f5aeef58f09f541a296c1c68f"
        or len(frontier_timing["events"]) != 7):
    raise ValueError("Frontier registration timing inventory mismatch")
for timing, template in zip(frontier_timing["events"], frontier_templates["templates"]):
    if (timing["write_ordinal"] != template["write_ordinal"] or timing["registration"] != template["registration"]
            or not isinstance(timing["recorded_started_at_ms"], int)
            or not isinstance(timing["recorded_completed_at_ms"], int)
            or timing["recorded_completed_at_ms"] < timing["recorded_started_at_ms"]
            or not re.fullmatch(r"[0-9a-f]{64}", timing["private_snapshot_event_sha256"])
            or not re.fullmatch(r"2026-11-09T01:1[024]:[0-9]{2}\.[0-9]{3}Z|2026-11-09T01:0[78]:[0-9]{2}\.[0-9]{3}Z", timing["event_timestamp"])):
        raise ValueError("Frontier registration timing metadata mismatch")

lineage = json.loads((root / "recovered/frontier-source-trace/LATE-LINEAGE.json").read_text(encoding="utf-8"))
if lineage["schema"] != "726-late-errantry-lineage-v1" or len(lineage["entries"]) != 5:
    raise ValueError("Invalid late checkpoint lineage")
for index, item in enumerate(lineage["entries"]):
    number = 3817 + index
    if (item["checkpoint"] != number
            or item["queue_before"] != f"CLOSURE-QUEUE-{number + 334}.json"
            or item["queue_after"] != f"CLOSURE-QUEUE-{number + 335}.json"
            or item["original_authority_bytes_recovered"] is not False):
        raise ValueError("Invalid late checkpoint lineage identity")
    note = item["preserved_note"]
    if note != f"recovered/transcript-notes/ERRANTRY-L4-{number}-NOTES.md":
        raise ValueError("Unsafe late checkpoint note path")
    if hashlib.sha256((root / note).read_bytes()).hexdigest() != item["preserved_note_sha256"]:
        raise ValueError("Late checkpoint note hash mismatch")
    modes = item["reported_modes"]
    if ([m.get("mode") for m in modes] != ["--preflight", "--apply", "--verify"]
            or any(m.get("status") != "pass" for m in modes)
            or modes[1].get("checkpoint") != number):
        raise ValueError("Invalid reported late checkpoint modes")

filename_text_directory = root / "recovered/printed-filename-text"
filename_text_manifest = json.loads((filename_text_directory / "MANIFEST.json").read_text(encoding="utf-8"))
filename_text_count = 0
for item in filename_text_manifest["files"]:
    sha = item["sha256"]
    if (len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha)
            or item["file"] != "objects/" + sha
            or Path(item["printed_filename"]).name != item["printed_filename"]):
        raise ValueError("Unsafe printed-filename text path")
    if hashlib.sha256((filename_text_directory / item["file"]).read_bytes()).hexdigest() != sha:
        raise ValueError("Printed-filename text hash mismatch")
    if (item["printed_filename"] not in item["printed_reference"]
            or sha not in item["printed_reference"].lower()
            or not isinstance(item["reference_ordinal"], int)):
        raise ValueError("Invalid printed-filename text provenance")
    filename_text_count += 1
extra_sources = root / "recovered/additional-source-registrations"
extra_manifest = json.loads((extra_sources / "MANIFEST.json").read_text(encoding="utf-8"))
if extra_manifest["schema"] != "726-additional-source-registrations-v1" or len(extra_manifest["registrations"]) != 2:
    raise ValueError("Invalid additional registrations")
for entry in extra_manifest["registrations"] + [extra_manifest["read_packet"]]:
    ordinal, sha = entry["event_ordinal"], entry["sha256"]
    if ordinal not in (65627, 65753, 65778) or len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Invalid additional registration identity")
    data = (extra_sources / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Additional registration object hash mismatch")
    for kind in ("program", "output"):
        fragment = extra_sources / "fragments" / f"{ordinal}-{kind}.txt"
        if hashlib.sha256(fragment.read_bytes()).hexdigest() != entry[kind + "_sha256"]:
            raise ValueError("Additional registration provenance hash mismatch")
    if sha not in (extra_sources / "fragments" / f"{ordinal}-output.txt").read_text(encoding="utf-8"):
        raise ValueError("Original hash absent from recorded output")
    obj = json.loads(data)
    if obj.get("registered_at", obj.get("at")) != entry["timestamp"]:
        raise ValueError("Additional registration timestamp mismatch")
    if ordinal != 65778:
        if len(obj["sources"]) != entry["source_count"] or sum(s["uncredited_assessment_lines_registered"] for s in obj["sources"]) != entry["line_occurrences"]:
            raise ValueError("Additional registration scope mismatch")
        for source in obj["sources"]:
            source_sha = source["source_sha256"]
            raw = (extra_sources / "sources" / source_sha).read_bytes()
            if hashlib.sha256(raw).hexdigest() != source_sha or len(raw) != source["bytes"] or len(raw.splitlines()) != source["actual_EOF"]:
                raise ValueError("Additional registered source mismatch")
            if any(a < 1 or b < a or b > source["actual_EOF"] for a, b in source["ranges"]):
                raise ValueError("Invalid additional registered range")
    elif obj["registration"]["sha256"] != entry["registration_sha256"] or obj["uncredited_total_source_lines"] != 625 or obj["new_body_population_unique_credit"] != 0 or obj["source_adoption_performed"] is not False:
        raise ValueError("Invalid additional packet relationship")
for name, key in (("65778-expression.txt", "expression_sha256"), ("65778-notes.json", "notes_sha256")):
    if hashlib.sha256((extra_sources / name).read_bytes()).hexdigest() != extra_manifest["read_packet"][key]:
        raise ValueError("Additional read packet reconstruction fragment mismatch")

chain = extra_manifest["building_issue_registration_chain"]
if [x["event_ordinal"] for x in chain] != [65866, 65878]:
    raise ValueError("Invalid building registration chain")
chain_objects = []
for entry in chain:
    ordinal, sha = entry["event_ordinal"], entry["sha256"]
    data = (extra_sources / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha:
        raise ValueError("Building registration chain hash mismatch")
    for kind in ("program", "output"):
        data_fragment = (extra_sources / "fragments" / f"{ordinal}-{kind}.txt").read_bytes()
        if hashlib.sha256(data_fragment).hexdigest() != entry[kind + "_sha256"]:
            raise ValueError("Building registration provenance mismatch")
        if kind == "output" and sha.encode() not in data_fragment:
            raise ValueError("Building registration hash not printed")
    obj = json.loads(data)
    source = obj["sources"][0]
    raw = (extra_sources / "sources" / source["source_sha256"]).read_bytes()
    if hashlib.sha256(raw).hexdigest() != source["source_sha256"] or len(raw.splitlines()) != source["actual_EOF"] or obj["registered_at"] != entry["timestamp"]:
        raise ValueError("Building registration source mismatch")
    chain_objects.append(obj)
invalid, corrected = chain_objects
if (invalid["sources"][0]["ranges"] != [[301, 314]] or invalid["sources"][0]["actual_EOF"] != 311
        or corrected["sources"][0]["ranges"] != [[301, 311]]
        or corrected["superseded_invalid_registration"]["sha256"] != chain[0]["sha256"]):
    raise ValueError("Building registration correction mismatch")

corrected_packet = extra_manifest["corrected_read_packet"]
packet_raw = (extra_sources / "objects" / corrected_packet["sha256"]).read_bytes()
if hashlib.sha256(packet_raw).hexdigest() != corrected_packet["sha256"]:
    raise ValueError("Corrected read packet hash mismatch")
packet = json.loads(packet_raw)
if (packet["registration"]["sha256"] != chain[1]["sha256"]
        or packet["at"] != corrected_packet["timestamp"]
        or packet["uncredited_total_source_lines"] != 11
        or packet["new_body_population_unique_credit"] != 0
        or packet["source_adoption_performed"] is not False):
    raise ValueError("Corrected read packet relationship mismatch")
for ordinal in (65883, 65892):
    for kind in ("program", "output"):
        fragment = (extra_sources / "fragments" / f"{ordinal}-{kind}.txt").read_bytes()
        if hashlib.sha256(fragment).hexdigest() != corrected_packet[f"{ordinal}_{kind}_sha256"]:
            raise ValueError("Corrected read packet provenance mismatch")
if corrected_packet["sha256"] not in (extra_sources / "fragments/65892-output.txt").read_text(encoding="utf-8"):
    raise ValueError("Corrected packet hash not printed")
source = corrected["sources"][0]
source_lines = (extra_sources / "sources" / source["source_sha256"]).read_text(encoding="utf-8").splitlines()
printed = (extra_sources / "fragments/65883-output.txt").read_text(encoding="utf-8").splitlines()
if printed != [f"{n}: {source_lines[n - 1]}" for n in range(301, 312)]:
    raise ValueError("Corrected packet recorded source display mismatch")

profile_reg = extra_manifest["capture_profile_first_registration"]
profile_reg_bytes = (extra_sources / "objects" / profile_reg["sha256"]).read_bytes()
if hashlib.sha256(profile_reg_bytes).hexdigest() != profile_reg["sha256"]:
    raise ValueError("Capture profile registration hash mismatch")
for kind in ("program", "output"):
    fragment = (extra_sources / "fragments" / f"65494-{kind}.txt").read_bytes()
    if hashlib.sha256(fragment).hexdigest() != profile_reg[kind + "_sha256"]:
        raise ValueError("Capture profile registration provenance mismatch")
    if kind == "output" and profile_reg["sha256"].encode() not in fragment:
        raise ValueError("Capture profile registration hash not printed")
profile_reg_obj = json.loads(profile_reg_bytes)
if (profile_reg_obj["registered_at"] != profile_reg["timestamp"]
        or len(profile_reg_obj["sources"]) != profile_reg["source_count"]
        or profile_reg_obj["uncredited_total_source_lines_registered"] != profile_reg["line_occurrences"]):
    raise ValueError("Capture profile registration scope mismatch")

profile_packets = extra_manifest["capture_profile_read_packets"]
if len(profile_packets) != 3:
    raise ValueError("Invalid capture profile packet count")
for kind in ("program", "output"):
    fragment = (extra_sources / "fragments" / f"65653-{kind}.txt").read_bytes()
    if hashlib.sha256(fragment).hexdigest() != extra_manifest[f"capture_profile_{kind}_sha256"]:
        raise ValueError("Capture profile packet provenance mismatch")
printed_profile_output = (extra_sources / "fragments/65653-output.txt").read_text(encoding="utf-8")
for entry in profile_packets:
    sha = entry["sha256"]
    data = (extra_sources / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or sha not in printed_profile_output:
        raise ValueError("Capture profile packet hash mismatch")
    packet = json.loads(data)
    if (packet["registration"]["sha256"] != entry["registration_sha256"]
            or packet["at"] != entry["timestamp"] or len(packet["rows"]) != entry["rows"]
            or packet["uncredited_total_source_lines"] != entry["line_occurrences"]
            or packet["new_body_population_unique_credit"] != 0
            or packet["source_adoption_performed"] is not False):
        raise ValueError("Capture profile packet scope mismatch")
    registration_path = (root / "recovered/transcript-hash-matched/a4d6e385ce23--REGISTRATION-ROOT-L12-CAPTURE-PROFILE-OWNERS-002.json"
                         if entry["registration_sha256"].startswith("a4d6e385")
                         else extra_sources / "objects" / entry["registration_sha256"])
    registration_bytes = registration_path.read_bytes()
    if hashlib.sha256(registration_bytes).hexdigest() != entry["registration_sha256"]:
        raise ValueError("Capture profile registration hash mismatch")
    registration = json.loads(registration_bytes)
    if [row["source"] for row in packet["rows"]] != registration["sources"]:
        raise ValueError("Capture profile packet registration rows mismatch")
    for row in packet["rows"]:
        source = row["source"]
        raw = (extra_sources / "sources" / source["source_sha256"]).read_bytes()
        if (hashlib.sha256(raw).hexdigest() != source["source_sha256"]
                or len(raw) != source["bytes"] or len(raw.splitlines()) != source["actual_EOF"]
                or row["assessed_ranges"] != source["ranges"]):
            raise ValueError("Capture profile packet source mismatch")

profile_trace_dir = root / "recovered/profile-source-display-trace"
profile_trace = json.loads((profile_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (profile_trace["schema"] != "726-profile-source-display-trace-v1"
        or len(profile_trace["chunks"]) != 8 or profile_trace["packet_count"] != 3):
    raise ValueError("Invalid profile display trace")
profile_lines = profile_differences = 0
packet_map = {x["sha256"]: json.loads((extra_sources / "objects" / x["sha256"]).read_bytes()) for x in profile_packets}
for chunk in profile_trace["chunks"]:
    cid = chunk["chunk_id"]
    if len(cid) != 6 or any(c not in "0123456789abcdef" for c in cid) or chunk["fragment"] != f"fragments/{cid}.txt":
        raise ValueError("Invalid profile display fragment path")
    data = (profile_trace_dir / chunk["fragment"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != chunk["sha256"]:
        raise ValueError("Profile display fragment hash mismatch")
    scopes = {scope["git_path"]: scope for scope in chunk["scopes"]}
    observed = {path: [] for path in scopes}
    differences = []
    current = None
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            current = line[7:]
            if current not in scopes:
                raise ValueError("Unknown profile display source")
            continue
        number, separator, displayed = line.partition(": ")
        if not separator or not number.isdigit() or current is None:
            raise ValueError("Invalid profile displayed line")
        n = int(number)
        source = scopes[current]
        raw = (extra_sources / "sources" / source["source_sha256"]).read_bytes()
        actual = raw.decode("utf-8").splitlines()[n - 1]
        if displayed != actual:
            if "\ufffd" not in displayed:
                raise ValueError("Profile display source mismatch")
            differences.append({"git_path": current, "line": n, "reason": "recorded display contains Unicode replacement character"})
        observed[current].append(n)
        profile_lines += 1
    if differences != chunk["display_differences"]:
        raise ValueError("Profile display difference mismatch")
    profile_differences += len(differences)
    for path, scope in scopes.items():
        expected = [n for a, b in scope["ranges"] for n in range(a, b + 1)]
        if observed[path] != expected:
            raise ValueError("Profile display range mismatch")
        packet = packet_map[scope["packet_sha256"]]
        if not any(row["source_sha256"] == scope["source_sha256"] and row["source"]["git_path"] == path and any(c.split(":")[0] == cid for c in row["output_chunks"]) for row in packet["rows"]):
            raise ValueError("Profile display packet link mismatch")
if profile_lines != 2374 or profile_lines != profile_trace["line_occurrences"] or profile_differences != 1 or profile_differences != profile_trace["replacement_character_lines"]:
    raise ValueError("Invalid profile display totals")

issues_trace_dir = root / "recovered/building-issues-display-trace"
issues_trace = json.loads((issues_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (issues_trace["schema"] != "726-building-issues-display-trace-v1"
        or issues_trace["packet_sha256"] != extra_manifest["read_packet"]["sha256"]
        or len(issues_trace["chunks"]) != 2):
    raise ValueError("Invalid building issues display trace")
issues_packet = json.loads((extra_sources / "objects" / issues_trace["packet_sha256"]).read_bytes())
issues_lines = 0
for chunk in issues_trace["chunks"]:
    cid = chunk["chunk_id"]
    if len(cid) != 6 or any(c not in "0123456789abcdef" for c in cid) or chunk["fragment"] != f"fragments/{cid}.txt":
        raise ValueError("Invalid building issues display path")
    data = (issues_trace_dir / chunk["fragment"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != chunk["sha256"] or chunk["display_differences"]:
        raise ValueError("Building issues display fragment mismatch")
    scopes = {scope["git_path"]: scope for scope in chunk["scopes"]}
    observed = {path: [] for path in scopes}
    current = None
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            current = line[7:]
            if current not in scopes:
                raise ValueError("Unknown building issues display source")
            continue
        number, separator, displayed = line.partition(": ")
        if not separator or not number.isdigit() or current is None:
            raise ValueError("Invalid building issues displayed line")
        n = int(number)
        source = scopes[current]
        raw = (extra_sources / "sources" / source["source_sha256"]).read_bytes()
        if displayed != raw.decode("utf-8").splitlines()[n - 1]:
            raise ValueError("Building issues display source mismatch")
        observed[current].append(n)
        issues_lines += 1
    for path, scope in scopes.items():
        if observed[path] != [n for a, b in scope["ranges"] for n in range(a, b + 1)]:
            raise ValueError("Building issues display range mismatch")
        if not any(row["source_sha256"] == scope["source_sha256"] and row["source"]["git_path"] == path and cid in row["output_chunks"] for row in issues_packet["rows"]):
            raise ValueError("Building issues display packet link mismatch")
if issues_lines != 625 or issues_lines != issues_trace["line_occurrences"] or issues_trace["replacement_character_lines"] != 0:
    raise ValueError("Invalid building issues display totals")

l2_trace_dir = root / "recovered/l2-selected-state-transition-trace"
l2_trace = json.loads((l2_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (l2_trace["schema"] != "726-l2-selected-state-transition-trace-v1"
        or l2_trace["target_selected_state_sha256"] != "199a4a9479b5ed3491ccc6df6d5f667cf41db2f9745c26bff939348c9a2f4158"
        or len(l2_trace["transitions"]) != 6):
    raise ValueError("Invalid Lens 2 state transition trace")
for index, transition in enumerate(l2_trace["transitions"]):
    ordinal = transition["write_ordinal"]
    if ordinal != [72674, 72782, 72875, 73050, 73181, 73399][index] or transition["intended_state_destination"] != f"segovb/STATE-L2-EXT-CURRENT-{index + 23:03}.json" or transition["view_expression"] != f"{ordinal}-view-expression.txt":
        raise ValueError("Invalid Lens 2 state transition identity")
    data = (l2_trace_dir / transition["view_expression"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != transition["view_expression_sha256"]:
        raise ValueError("Lens 2 state view expression hash mismatch")
    expression = ast.parse(data.decode("utf-8"), mode="eval").body
    if not isinstance(expression, ast.Dict):
        raise ValueError("Invalid Lens 2 state view expression")
    fields = {}
    for key, value in zip(expression.keys, expression.values):
        try:
            fields[ast.literal_eval(key)] = ast.literal_eval(value)
        except (ValueError, TypeError):
            fields[ast.literal_eval(key)] = {"unresolved_expression": ast.unparse(value)}
    if fields != transition["view_fields"]:
        raise ValueError("Lens 2 state view literal mismatch")
    update = ast.parse(transition["state_update_expression"], mode="eval").body
    if not isinstance(update, ast.Call) or ast.unparse(update.func) != "state.update" or not any(k.arg == "predecessor" for k in update.keywords):
        raise ValueError("Invalid Lens 2 state update expression")
    recorded = transition["recorded_verification"]
    if recorded.get("status") != "pass" or recorded.get("mode") != "--verify" or recorded.get("checkpoint") != 489 + index:
        raise ValueError("Invalid Lens 2 recorded verification")

l12_trace_dir = root / "recovered/l12-selected-state-transition-trace"
l12_trace = json.loads((l12_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (l12_trace["schema"] != "726-l12-selected-state-transition-trace-v1"
        or l12_trace["target_sha256"] != "01ff18cca3dd9be3c117fd8e4ca55c872a4cbb7edd3805dabab296ff076f6b9c"
        or l12_trace["intended_state_destination"] != "segovb/STATE-L12-METADATA-CURRENT-008.json"):
    raise ValueError("Invalid Lens 12 state transition trace")
view_bytes = (l12_trace_dir / "VIEW-EXPRESSION.txt").read_bytes()
if hashlib.sha256(view_bytes).hexdigest() != l12_trace["view_expression_sha256"]:
    raise ValueError("Lens 12 state view expression hash mismatch")
view_expression = ast.parse(view_bytes.decode("utf-8"), mode="eval").body
fields = {}
for key, value in zip(view_expression.keys, view_expression.values):
    try:
        fields[ast.literal_eval(key)] = ast.literal_eval(value)
    except (ValueError, TypeError):
        fields[ast.literal_eval(key)] = {"unresolved_expression": ast.unparse(value)}
if fields != l12_trace["view_fields"]:
    raise ValueError("Lens 12 state view literal mismatch")
helper = None
with (root / "recovered/transcript-helper-candidates/CANDIDATES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        candidate = json.loads(line)
        if candidate["historical_relative_path"] == l12_trace["candidate_path"]:
            helper = candidate
            break
if helper is None or helper["candidate_sha256"] != l12_trace["candidate_sha256"] or helper["first_filechange_ordinal"] != l12_trace["candidate_filechange_ordinal"]:
    raise ValueError("Lens 12 helper provenance mismatch")
helper_ast = ast.parse(helper["candidate_text"])
helper_view = next(node.value for node in helper_ast.body if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "view" for t in node.targets))
if ast.dump(helper_view) != ast.dump(view_expression):
    raise ValueError("Lens 12 helper view mismatch")
helper_update = next(node for node in ast.walk(helper_ast) if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name) and node.func.value.id == "state" and node.func.attr == "update")
if ast.unparse(helper_update) != l12_trace["state_update_expression"]:
    raise ValueError("Lens 12 helper state update mismatch")
for entry, ordinal, mode in zip(l12_trace["reported_results"], [74897, 74910, 74924], ["--preflight", "--apply", "--verify"]):
    result = entry["result"]
    if entry["event_ordinal"] != ordinal or result.get("status") != "pass" or result.get("mode") != mode or result.get("checkpoint") != 502:
        raise ValueError("Invalid Lens 12 reported result")

l13_trace_dir = root / "recovered/l13-selected-state-transition-trace"
l13_trace = json.loads((l13_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (l13_trace["schema"] != "726-l13-selected-state-transition-trace-v1"
        or l13_trace["target_sha256"] != "e177a5f553ffb2e438702ac6e547c40475f42a0ffdbe014fe88283cfd38137e2"
        or l13_trace["target_state"] != "segovb/STATE-L13-PARALLEL-723.json"):
    raise ValueError("Invalid Lens 13 selected state trace")
transformation = (l13_trace_dir / "TRANSFORMATION.txt").read_bytes()
if hashlib.sha256(transformation).hexdigest() != l13_trace["transformation_sha256"]:
    raise ValueError("Lens 13 transformation hash mismatch")
recorded_transform = None
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        candidate = json.loads(line)
        if candidate.get("transcript_ordinal") == 56141:
            recorded_transform = candidate["command_text"]
            break
if recorded_transform is None or recorded_transform.split("@'\n", 1)[1].split("\n'@", 1)[0] + "\n" != transformation.decode("utf-8"):
    raise ValueError("Lens 13 transformation provenance mismatch")
transformation_ast = ast.parse(transformation.decode("utf-8"))
facts_node = next(node.value for node in transformation_ast.body if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "facts" for t in node.targets))
if {str(k): v for k, v in ast.literal_eval(facts_node).items()} != l13_trace["facts"] or sorted(l13_trace["facts"]) != [str(i) for i in range(2846, 2853)]:
    raise ValueError("Lens 13 source facts mismatch")
results = l13_trace["reported_results"]
if [entry["event_ordinal"] for entry in results] != [56142, 56153, 56160, 56161, 56169, 56174, 56175]:
    raise ValueError("Invalid Lens 13 result sequence")
packet, preflight, applied, verification = [entry["result"] for entry in results[:4]]
if (packet["fresh"] != 409 or packet["reused"] != 47 or preflight["preflight"] != "passed"
        or preflight["authority_writes"] != 0 or applied["authority_writes"] != 2
        or applied["plan"] != packet["plan"] or verification["result"] != "passed"
        or verification["original_before"] != 2846 or verification["original_after"] != 2853):
    raise ValueError("Invalid Lens 13 reported adoption")

native_trace_dir = root / "recovered/building-issues-native-source-trace"
native_trace = json.loads((native_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (native_trace["schema"] != "726-building-issues-native-source-trace-v1"
        or native_trace["registration_reference"]["original_bytes_recovered"] is not False
        or len(native_trace["sources"]) != 8 or len(native_trace["fragments"]) != 4):
    raise ValueError("Invalid building native source trace")
native_sources = {source["source_id"]: source for source in native_trace["sources"]}
native_lines = {}
for source in native_trace["sources"]:
    sha = source["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Invalid native source path")
    data = (native_trace_dir / "sources" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or len(data) != source["bytes"] or len(data.splitlines()) != source["eof"]:
        raise ValueError("Building native source hash mismatch")
    native_lines[source["source_id"]] = data.decode("utf-8").splitlines()
    if source["origin"]["kind"] == "crate" and source["origin"]["archive_checksum"] != native_trace["native_archive_checksum"]:
        raise ValueError("Building native archive checksum mismatch")
lock_source = native_sources["current-337:Cargo.lock"]
if lock_source["sha256"] != native_trace["lock_sha256"]:
    raise ValueError("Building native lock mismatch")
lock_text = (native_trace_dir / "sources" / lock_source["sha256"]).read_text(encoding="utf-8")
packages = [section.splitlines() for section in lock_text.split("[[package]]")]
packages = [lines for lines in packages if 'name = "bevy_ecs"' in lines and 'version = "0.19.1"' in lines]
if len(packages) != 1 or f'checksum = "{native_trace["native_archive_checksum"]}"' not in packages[0]:
    raise ValueError("Building native lock checksum mismatch")
native_observed = {key: [] for key in native_sources}
native_differences = []
for fragment in native_trace["fragments"]:
    ordinal = fragment["event_ordinal"]
    if ordinal not in (65828, 65835, 65842, 65847) or fragment["file"] != f"fragments/{ordinal}.txt":
        raise ValueError("Invalid native display fragment path")
    data = (native_trace_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Native display fragment hash mismatch")
    current = None
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            current = line[7:]
            if current not in native_sources:
                raise ValueError("Unknown native displayed source")
            continue
        number, separator, displayed = line.partition(": ")
        if current is None or not separator or not number.isdigit():
            raise ValueError("Invalid native displayed line")
        n = int(number)
        if displayed != native_lines[current][n - 1]:
            if "\ufffd" not in displayed:
                raise ValueError("Native displayed line mismatch")
            native_differences.append({"event_ordinal": ordinal, "source_id": current, "line": n})
        native_observed[current].append(n)
for source in native_trace["sources"]:
    expected = [n for a, b in source["ranges"] for n in range(a, b + 1)]
    if native_observed[source["source_id"]] != expected:
        raise ValueError("Native displayed range mismatch")
if (sum(len(v) for v in native_observed.values()) != 1017 or native_trace["line_occurrences"] != 1017
        or native_differences != native_trace["display_differences"] or len(native_differences) != 8):
    raise ValueError("Invalid native displayed totals")

helper_chain = l13_trace["static_helper_chain"]
commands = {}
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if item.get("transcript_ordinal") in (55896, 56018, 56141):
            commands[item["transcript_ordinal"]] = item["command_text"].split("@'\n", 1)[1].split("\n'@", 1)[0]
if set(commands) != {55896, 56018, 56141}:
    raise ValueError("Missing Lens 13 helper transformation command")
base_tree = ast.parse(commands[55896])
helper_text = ast.literal_eval(next(node.value for node in base_tree.body if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "s" for t in node.targets)))
def static_expression(node, env):
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, ast.Name):
        return env[node.id]
    if isinstance(node, (ast.List, ast.Dict, ast.Tuple)):
        return ast.literal_eval(node)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        return static_expression(node.left, env) + static_expression(node.right, env)
    if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Slice):
        lower = static_expression(node.slice.lower, env) if node.slice.lower else None
        upper = static_expression(node.slice.upper, env) if node.slice.upper else None
        return static_expression(node.value, env)[slice(lower, upper)]
    if isinstance(node, ast.Call):
        args = [static_expression(arg, env) for arg in node.args]
        if isinstance(node.func, ast.Name) and node.func.id == "repr":
            return repr(*args)
        if isinstance(node.func, ast.Attribute) and node.func.attr in ("index", "replace", "count"):
            return getattr(static_expression(node.func.value, env), node.func.attr)(*args)
    raise ValueError("Unsupported Lens 13 static expression")
for entry, ordinal in zip(helper_chain["records"], (56018, 56141)):
    if entry["transformation_ordinal"] != ordinal or entry["file"] != f"RECONSTRUCTED-{ordinal}.py.txt":
        raise ValueError("Invalid Lens 13 helper chain identity")
    env = {"s": helper_text}
    for node in ast.parse(commands[ordinal]).body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            name = node.targets[0].id
            if name == "p" or (name == "s" and isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Attribute) and node.value.func.attr == "read_text"):
                continue
            env[name] = static_expression(node.value, env)
        elif isinstance(node, ast.Assert):
            if not isinstance(node.test, ast.Compare) or static_expression(node.test.left, env) != static_expression(node.test.comparators[0], env):
                raise ValueError("Lens 13 static transformation assertion failed")
        elif not isinstance(node, (ast.ImportFrom, ast.With)):
            raise ValueError("Unsupported Lens 13 transformation statement")
    helper_text = env["s"]
    data = (l13_trace_dir / entry["file"]).read_bytes()
    if data != helper_text.encode("utf-8") or hashlib.sha256(data).hexdigest() != entry["normalized_sha256"] or len(data) != entry["bytes"]:
        raise ValueError("Lens 13 reconstructed helper mismatch")
    ast.parse(helper_text)

l13_inherited_dir = root / "recovered/l13-final-inherited-evidence"
l13_inherited = json.loads((l13_inherited_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if l13_inherited["schema"] != "726-l13-final-inherited-evidence-v1" or len(l13_inherited["objects"]) != 7:
    raise ValueError("Invalid Lens 13 inherited evidence")
discovery_bytes = (l13_inherited_dir / "DISCOVERY-OUTPUT.json").read_bytes()
if hashlib.sha256(discovery_bytes).hexdigest() != l13_inherited["discovery_output_sha256"]:
    raise ValueError("Lens 13 discovery output hash mismatch")
discovery = json.loads(discovery_bytes)
objects = {}
for item in l13_inherited["objects"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Unsafe Lens 13 inherited object path")
    data = (l13_inherited_dir / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or len(data) != item["bytes"]:
        raise ValueError("Lens 13 inherited object hash mismatch")
    objects[sha] = data
if len(discovery["hits"]) != 2 or len(discovery["futures"]) != 2 or discovery["apps"]:
    raise ValueError("Invalid Lens 13 inherited discovery")
parent_hash = discovery["hits"][0]["receipt"]["sha256"]
parent = json.loads(objects[parent_hash])
raw = json.loads(objects[parent["delegated_receipt"]["sha256"]])
registration = json.loads(objects[discovery["futures"][0]["registration"]["sha256"]])
for reference in (parent["delegated_verification"], parent["agent_validation_manifest"], parent["prospective_chronology_attestation"]):
    if reference["sha256"] not in objects:
        raise ValueError("Missing Lens 13 inherited supporting object")
if parent["evidence"] != raw or raw["lens"] != 13 or registration["lens"] != 13:
    raise ValueError("Lens 13 inherited reader relationship mismatch")
for i, future in enumerate(discovery["futures"], start=1):
    row = raw["connected_source_reads"][i]
    if (future["original"] != 2845 + i or future["row_pointer"] != f"/connected_source_reads/{i}"
            or future["receipt_sha256"] != parent["delegated_receipt"]["sha256"]
            or future["source_sha256"] != row["sha256"] or future["source_id"] != row["source_id"]
            or future["assessed_ranges"] != row["prospective_ranges"] or future["ordered_credit_now"] != 0
            or len(objects[row["sha256"]].splitlines()) != row["actual_eof"]):
        raise ValueError("Lens 13 inherited source row mismatch")

l13_helper = l13_trace["qualification_helper"]
l13_helper_sha = l13_helper["sha256"]
if l13_helper_sha != "660d703a9fbb1d0f3af36ee71a538f74189e1ebd521a88c51c1d2f259398635e":
    raise ValueError("Invalid Lens 13 qualification helper identity")
helper_bytes = (l13_trace_dir / l13_helper_sha).read_bytes()
if hashlib.sha256(helper_bytes).hexdigest() != l13_helper_sha:
    raise ValueError("Lens 13 qualification helper hash mismatch")
ast.parse(helper_bytes.decode("utf-8"))
helper_output_bytes = (l13_trace_dir / "QUALIFICATION-HELPER-OUTPUT.txt").read_bytes()
if hashlib.sha256(helper_output_bytes).hexdigest() != l13_helper["output_sha256"]:
    raise ValueError("Lens 13 qualification helper output hash mismatch")
helper_output = json.loads(helper_output_bytes)
if (helper_output["helper"]["sha256"] != l13_helper_sha
        or helper_output["registration"]["sha256"] != "ec0c5b93a0d57a0e3970291e4dcedc00182515bdcc57732e0c0198a639bd4918"
        or helper_output["qualification"]["sha256"] != "2c1ea660bff7fa7fdc8e500681a775087b4c0985a8d3f981f0bf847d21c3d506"
        or helper_output["fresh"] != 409 or helper_output["reused"] != 47):
    raise ValueError("Lens 13 qualification helper output relationship mismatch")

l13_registration = l13_trace["final_registration"]
l13_reg_sha = l13_registration["sha256"]
if l13_reg_sha != "ec0c5b93a0d57a0e3970291e4dcedc00182515bdcc57732e0c0198a639bd4918":
    raise ValueError("Invalid final Lens 13 registration identity")
l13_reg_bytes = (l13_trace_dir / l13_reg_sha).read_bytes()
if hashlib.sha256(l13_reg_bytes).hexdigest() != l13_reg_sha:
    raise ValueError("Final Lens 13 registration hash mismatch")
l13_reg = json.loads(l13_reg_bytes)
if (l13_reg["at"] != l13_registration["timestamp"] or l13_reg["lens"] != 13
        or l13_reg["fresh_lines_registered"] != 409 or l13_reg["prior_material_lines_reused"] != 47
        or l13_reg["body_credit"] != 0 or len(l13_reg["valid_nested_range_fields"]) != 14
        or l13_reg["exact_hash_prior_hits"] != discovery["hits"]
        or l13_reg["future_proofs_preserved"] != discovery["futures"]
        or l13_reg["old_ordered_applications_preserved"] != discovery["apps"]):
    raise ValueError("Final Lens 13 registration scope mismatch")
if [row["original"] for row in l13_reg["sources"]] != list(range(2846, 2853)):
    raise ValueError("Invalid final Lens 13 registration rows")
for row in l13_reg["sources"]:
    sha = row["sha256"]
    data = (l13_trace_dir / "sources" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or len(data.splitlines()) != row["actual_eof"]:
        raise ValueError("Final Lens 13 registered source mismatch")
    if any(lo < 1 or hi < lo or hi > row["actual_eof"] for lo, hi in row["prospective_ranges"]):
        raise ValueError("Invalid final Lens 13 source range")
register_text = (l13_trace_dir / "REGISTER-RECONSTRUCTED.txt").read_bytes()
if hashlib.sha256(register_text).hexdigest() != l13_trace["register_helper_text"]["sha256"]:
    raise ValueError("Final Lens 13 registration helper text mismatch")
ast.parse(register_text.decode("utf-8"))
if helper_output["registration"]["sha256"] != l13_reg_sha:
    raise ValueError("Final Lens 13 registration provenance mismatch")

l13_qualification = l13_trace["final_qualification"]
l13_q_sha = l13_qualification["sha256"]
if l13_q_sha != "2c1ea660bff7fa7fdc8e500681a775087b4c0985a8d3f981f0bf847d21c3d506":
    raise ValueError("Invalid final Lens 13 qualification identity")
l13_q_bytes = (l13_trace_dir / l13_q_sha).read_bytes()
if hashlib.sha256(l13_q_bytes).hexdigest() != l13_q_sha:
    raise ValueError("Final Lens 13 qualification hash mismatch")
l13_q = json.loads(l13_q_bytes)
if (l13_q["at"] != l13_qualification["timestamp"] or l13_q["registration"]["sha256"] != l13_reg_sha
        or l13_q["proof_builder"]["sha256"] != l13_helper_sha or len(l13_q["entries"]) != 7
        or l13_q["qualified_against_ledger"]["sha256"] != l13_qualification["ledger_sha256"]
        or l13_q["qualified_against_selector"]["sha256"] != l13_qualification["selector_sha256"]
        or l13_q["product_execution"] is not False or l13_q["implementation_gate_complete"] is not False
        or [entry["source"] for entry in l13_q["entries"]] != l13_reg["sources"]):
    raise ValueError("Final Lens 13 qualification relationship mismatch")
l13_final_packet = l13_trace["final_packet"]
l13_packet_sha = l13_final_packet["sha256"]
packet_bytes = (l13_trace_dir / l13_packet_sha).read_bytes()
if hashlib.sha256(packet_bytes).hexdigest() != l13_packet_sha or l13_packet_sha != l13_trace["reported_results"][0]["result"]["packet"]["sha256"]:
    raise ValueError("Final Lens 13 packet hash mismatch")
packet = json.loads(packet_bytes)
if (packet["at"] != l13_final_packet["timestamp"] or packet["original_before"] != 2846 or packet["original_after"] != 2853
        or packet["new_root_ordered_lines"] != 409 or packet["source_line_roles"]["reused_production"] != 47
        or packet["product_execution"] is not False or packet["implementation_gate_complete"] is not False
        or [row["source"] for row in packet["ordered"]] != l13_reg["sources"]
        or any(row["registration"]["sha256"] != l13_reg_sha for row in packet["ordered"])):
    raise ValueError("Final Lens 13 packet relationship mismatch")

l13_plan_meta = l13_trace["plan_candidate"]
l13_plan_candidate_data = (l13_trace_dir / l13_plan_meta["file"]).read_bytes()
if hashlib.sha256(l13_plan_candidate_data).hexdigest() != l13_plan_meta["sha256"]:
    raise ValueError("Lens 13 candidate plan inventory hash mismatch")
l13_plan_candidate = json.loads(l13_plan_candidate_data)
l13_plan = l13_plan_candidate["template"]
l13_previous_results = {item["ordinal"]: item["recorded_result"] for item in l13_plan_candidate["printed_predecessor_evidence"]}
if (set(l13_previous_results) != {56035, 56053} or l13_plan_candidate["searched_distinct_tokens"] != 16973
        or l13_plan_candidate["matched_state_hashes"] != [] or l13_plan_meta["match_found"] is not False
        or l13_plan_candidate["plan_sha256"] != l13_trace["reported_results"][0]["result"]["plan"]["sha256"]
        or l13_plan_meta["original_plan_sha256"] != l13_plan_candidate["plan_sha256"]
        or l13_plan["previous_state"] != {"path": "segovb/STATE-L13-PARALLEL-722.json", "sha256": "X" * 64}
        or l13_plan["packet"] != l13_trace["reported_results"][0]["result"]["packet"]
        or l13_plan["admission_2846_2852_material_qualification"]["sha256"] != l13_q_sha
        or l13_plan["ledger"]["sha256"] != l13_q["qualified_against_ledger"]["sha256"]
        or l13_plan["selector"]["sha256"] != l13_q["qualified_against_selector"]["sha256"]
        or any(l13_plan[key] != l13_previous_results[56053][key] for key in ("ledger", "selector", "queue"))
        or l13_plan["selected_lenses"] != l13_previous_results[56035]["selected_lenses"]
        or l13_plan["selected_lenses"] != l13_previous_results[56053]["all13_selections_unchanged"]
        or (l13_plan["root_sequence"], l13_plan["queue_sequence"]) != (723, 767)
        or l13_plan["next_selected_lenses_filename"] != "SELECTED-LENSES-CONTINUATION-415.json"):
    raise ValueError("Lens 13 candidate plan reference mismatch")
l13_expected_reuses = [{"kind": "exact_admission_2846_2852_material", "original": entry["original"],
                         "adoption": entry["prior"]["prior_adoption"],
                         "qualification": l13_plan["admission_2846_2852_material_qualification"]}
                        for entry in l13_q["entries"] if entry["mode"] != "fresh"]
if l13_plan["prior_reuses"] != l13_expected_reuses or len(l13_expected_reuses) != 1:
    raise ValueError("Lens 13 candidate plan reuse mismatch")
# The private transcript token search and historical state equality cannot be rerun from this package.

l13_prev_dir = root / "recovered/l13-predecessor-discovery"
l13_prev_manifest = json.loads((l13_prev_dir / "MANIFEST.json").read_bytes())
l13_prev_sha = l13_prev_manifest["original_sha256"]
if (l13_prev_manifest["schema"] != "726-l13-predecessor-discovery-v1"
        or l13_prev_sha != "833bf8943cafb748d73f2d2caf90b13c3e68a65f8fc7566941b3b5a855ff7a92"
        or l13_prev_manifest["source_registry_sha256"] != l13_registration["registry_sha256"]):
    raise ValueError("Lens 13 predecessor discovery identity mismatch")
for item in l13_prev_manifest["files"]:
    if hashlib.sha256((l13_prev_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Lens 13 predecessor discovery supporting file mismatch")
l13_prev_data = (l13_prev_dir / l13_prev_sha).read_bytes()
if hashlib.sha256(l13_prev_data).hexdigest() != l13_prev_sha:
    raise ValueError("Lens 13 predecessor discovery original hash mismatch")
l13_prev = json.loads(l13_prev_data)
l13_prev_result = json.loads((l13_prev_dir / "PRINTED-RESULT.json").read_bytes())
if (l13_prev_result["sha256"] != l13_prev_sha or l13_prev["at"] != l13_prev_manifest["matched_timestamp"]
        or l13_prev["frontier"] != 2836 or l13_prev["prior_same_lens_receipts_authenticated"] != 725
        or [entry["original"] for entry in l13_prev["entries"]] != list(range(2836, 2846))
        or l13_prev["exact_hash_prior_hits"] != l13_prev_result["hits"]
        or l13_prev["future_same_lens_proofs_navigation"] != l13_prev_result["futures"]
        or l13_prev["prior_applications_preserved_navigation"] != l13_prev_result["apps"]
        or l13_prev["body_credit"] != 0 or l13_prev["authority_changes"] != 0):
    raise ValueError("Lens 13 predecessor discovery recorded values mismatch")
l13_prev_print = json.loads((l13_prev_dir / "PRINTED-REGISTRATION-RESULT.json").read_bytes())
for kind in ("helper", "registration"):
    expected_sha = l13_prev_manifest[kind]["sha256"]
    if expected_sha != l13_prev_print[kind]["sha256"] or hashlib.sha256((l13_prev_dir / expected_sha).read_bytes()).hexdigest() != expected_sha:
        raise ValueError("Lens 13 predecessor original registration or helper mismatch")
if l13_prev_print["qualification"]["sha256"] != l13_prev_manifest["qualification_original_sha256"]:
    raise ValueError("Lens 13 predecessor missing qualification reference mismatch")
l13_prev_helper = (l13_prev_dir / l13_prev_manifest["helper"]["sha256"]).read_text(encoding="utf-8")
ast.parse(l13_prev_helper)
if l13_prev_sha not in l13_prev_helper or "range(2836,2846)" not in l13_prev_helper:
    raise ValueError("Lens 13 predecessor helper source mismatch")
l13_prev_transforms = json.loads((l13_prev_dir / "HELPER-TRANSFORM-COMMANDS.json").read_bytes())["records"]
if [item["ordinal"] for item in l13_prev_transforms] != [55974, 55982, 56086]:
    raise ValueError("Lens 13 helper transformation chronology mismatch")
l13_transform_trees = {item["ordinal"]: ast.parse(item["command_body"]) for item in l13_prev_transforms}
def l13_replacement_chain(node):
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "replace":
        return l13_replacement_chain(node.func.value) + [(ast.literal_eval(node.args[0]), ast.literal_eval(node.args[1]))]
    return []
l13_later_tree = l13_transform_trees[56086]
l13_prior_tree = l13_transform_trees[55974]
l13_inverse = (l13_trace_dir / l13_helper_sha).read_text(encoding="utf-8")
for old, new in reversed(l13_replacement_chain(l13_later_tree.body[8].value)):
    if new not in l13_inverse:
        raise ValueError("Lens 13 later helper replacement missing")
    l13_inverse = l13_inverse.replace(new, old)
l13_added = "\n  if partial_prior:e.update(partial_prior=partial_prior,fresh_personal_assessed_ranges=[[216,216]],qualified_prior_material_ranges=[[217,243]],prior_material_lines_reused=27,new_root_personal_lines=1)"
if l13_inverse.count(l13_added) != 1:
    raise ValueError("Lens 13 later helper partial branch mismatch")
l13_inverse = l13_inverse.replace(l13_added, "")
l13_original_block = ast.literal_eval(l13_prior_tree.body[17].value.left.right)
l13_lo = l13_inverse.index("  if n in(2846,2847):")
l13_hi = l13_inverse.index("  e={'original':n", l13_lo)
l13_inverse = l13_inverse[:l13_lo] + l13_original_block + l13_inverse[l13_hi:]
for old, new in reversed(l13_replacement_chain(l13_later_tree.body[2].value)):
    if new not in l13_inverse:
        raise ValueError("Lens 13 original helper replacement missing")
    l13_inverse = l13_inverse.replace(new, old)
if l13_inverse != l13_prev_helper or l13_inverse.replace("\n", "\r\n").encode("utf-8") != (l13_prev_dir / l13_prev_manifest["helper"]["sha256"]).read_bytes():
    raise ValueError("Lens 13 inverse helper reconstruction mismatch")

l13_prev_reg = json.loads((l13_prev_dir / l13_prev_manifest["registration"]["sha256"]).read_bytes())
if (l13_prev_reg["schema"] != "726-exact2836-2845-admission-registration-v1"
        or l13_prev_reg["at"] != l13_prev_manifest["registration"]["matched_timestamp"]
        or len(l13_prev_reg["sources"]) != 10 or len(l13_prev_reg["valid_nested_range_fields"]) != 20
        or l13_prev_reg["fresh_lines_registered"] != 573 or l13_prev_reg["prior_material_lines_reused"] != 285
        or l13_prev_reg["prior_same_lens_receipts_checked"] != 725
        or l13_prev_reg["exact_hash_prior_hits"] != l13_prev["exact_hash_prior_hits"]
        or l13_prev_reg["future_proofs_preserved"] != l13_prev["future_same_lens_proofs_navigation"]
        or l13_prev_reg["old_ordered_applications_preserved"] != l13_prev["prior_applications_preserved_navigation"]
        or l13_prev_reg["authority_changes"] != 0 or l13_prev_reg["body_credit"] != 0
        or l13_prev_reg["product_execution"] is not False):
    raise ValueError("Lens 13 predecessor registration scope mismatch")
for entry, row in zip(l13_prev["entries"], l13_prev_reg["sources"]):
    registry_row = entry["registry_entry"]
    if (row["original"] != entry["original"] or row["source_id"] != registry_row["source_id"]
            or row["sha256"] != registry_row["sha256"] or row["actual_eof"] != registry_row["actual_line_count"]
            or row["prospective_ranges"] != registry_row["material_ranges"]):
        raise ValueError("Lens 13 predecessor registration registry link mismatch")

l13_prev_q_meta = l13_prev_manifest["exact_qualification"]
l13_prev_q_sha = l13_prev_q_meta["sha256"]
l13_prev_q_data = (l13_prev_dir / l13_prev_q_sha).read_bytes()
if (l13_prev_q_sha != l13_prev_print["qualification"]["sha256"]
        or hashlib.sha256(l13_prev_q_data).hexdigest() != l13_prev_q_sha):
    raise ValueError("Lens 13 predecessor original qualification hash mismatch")
l13_prev_q = json.loads(l13_prev_q_data)
l13_prior_authority = json.loads((l13_prev_dir / "PRINTED-PRIOR-AUTHORITY.json").read_bytes())
if (l13_prev_q["at"] != l13_prev_q_meta["matched_timestamp"]
        or l13_prev_q["registration"]["sha256"] != l13_prev_manifest["registration"]["sha256"]
        or l13_prev_q["proof_builder"]["sha256"] != l13_prev_manifest["helper"]["sha256"]
        or l13_prev_q["discovery_metadata_sha256"] != l13_prev_sha
        or l13_prev_q["qualified_against_ledger"] != l13_prior_authority["ledger"]
        or l13_prev_q["qualified_against_selector"] != l13_prior_authority["selector"]
        or l13_prior_authority["ordinal"] != 55937 or len(l13_prev_q["entries"]) != 10
        or [entry["source"] for entry in l13_prev_q["entries"]] != l13_prev_reg["sources"]
        or l13_prev_q["all_nested_range_fields"] != l13_prev_reg["valid_nested_range_fields"]
        or (l13_prev_q["material_lines"], l13_prev_q["fresh_lines"], l13_prev_q["reused_material_lines"]) != (858, 573, 285)
        or l13_prev_q["body_credit"] != 0 or l13_prev_q["authority_changes"] != 0
        or l13_prev_q["product_execution"] is not False or l13_prev_q["implementation_gate_complete"] is not False):
    raise ValueError("Lens 13 predecessor qualification relationships mismatch")
l13_prev_objects = {}
for item in l13_prev_q_meta["inherited_objects"]:
    data = (l13_prev_dir / "inherited-objects" / item["sha256"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["sha256"]:
        raise ValueError("Lens 13 predecessor inherited evidence mismatch")
    l13_prev_objects[item["sha256"]] = json.loads(data)
if len(l13_prev_objects) != 6:
    raise ValueError("Lens 13 predecessor inherited object count mismatch")
for entry, item in zip(l13_prev_q["entries"], l13_prev_q_meta["source_records"]):
    source = entry["source"]
    data = (l13_prev_dir / "sources" / item["sha256"]).read_bytes()
    if (entry["original"] != item["original"] or source["sha256"] != item["sha256"]
            or hashlib.sha256(data).hexdigest() != item["sha256"] or len(data) != item["bytes"]
            or len(data.splitlines()) != source["actual_eof"] or entry["bytes"] != len(data)
            or entry["native_pr_comparison"]["sha256"] != item["sha256"]):
        raise ValueError("Lens 13 predecessor source object mismatch")
    if entry["mode"] == "delegated-reuse":
        prior = entry["prior"]
        for key in ("prior_adoption", "prior_raw_receipt", "prior_verification", "prior_registration", "prior_validation_manifest"):
            if prior[key]["sha256"] not in l13_prev_objects:
                raise ValueError("Lens 13 predecessor reuse object missing")
        raw_receipt = l13_prev_objects[prior["prior_raw_receipt"]["sha256"]]
        index = int(prior["prior_row_pointer"].split("/")[-1])
        if (raw_receipt["connected"][index] != prior["prior_source_read"]
                or prior["prior_source_read"]["sha256"] != item["sha256"]
                or entry["new_root_personal_lines"] != 0):
            raise ValueError("Lens 13 predecessor inherited source link mismatch")
if sum(entry["mode"] == "delegated-reuse" for entry in l13_prev_q["entries"]) != 2:
    raise ValueError("Lens 13 predecessor reuse count mismatch")

l13_prev_packet_meta = l13_prev_manifest["exact_packet"]
l13_prev_packet_data = (l13_prev_dir / l13_prev_packet_meta["sha256"]).read_bytes()
l13_prev_packet_result = json.loads((l13_prev_dir / "PRINTED-PACKET-RESULT.json").read_bytes())
if (hashlib.sha256(l13_prev_packet_data).hexdigest() != l13_prev_packet_meta["sha256"]
        or l13_prev_packet_result["packet"]["sha256"] != l13_prev_packet_meta["sha256"]):
    raise ValueError("Lens 13 predecessor packet original hash mismatch")
l13_prev_packet = json.loads(l13_prev_packet_data)
if (l13_prev_packet["at"] != l13_prev_packet_meta["matched_timestamp"]
        or (l13_prev_packet["original_before"], l13_prev_packet["original_after"]) != (2836, 2846)
        or l13_prev_packet["new_root_ordered_lines"] != 573
        or l13_prev_packet["source_line_roles"]["reused_production"] != 285
        or l13_prev_packet["product_execution"] is not False or l13_prev_packet["implementation_gate_complete"] is not False
        or [row["source"] for row in l13_prev_packet["ordered"]] != l13_prev_reg["sources"]
        or any(row["registration"]["sha256"] != l13_prev_manifest["registration"]["sha256"] for row in l13_prev_packet["ordered"])):
    raise ValueError("Lens 13 predecessor packet scope mismatch")
l13_prev_helper_ast = ast.parse((l13_trace_dir / "RECONSTRUCTED-56018.py.txt").read_text(encoding="utf-8"))
l13_prev_facts = ast.literal_eval(next(node.value for node in l13_prev_helper_ast.body if isinstance(node, ast.Assign)
                                      and any(isinstance(target, ast.Name) and target.id == "facts" for target in node.targets)))
for row in l13_prev_packet["ordered"]:
    if row["observations"] != [{"ranges": row["source"]["prospective_ranges"], "assessment": l13_prev_facts[row["source"]["original"]]}]:
        raise ValueError("Lens 13 predecessor packet source assessment mismatch")
l13_prev_source_rows = {entry["original"]: entry for entry in l13_prev_q["entries"]}
l13_prev_displayed = {}
for fragment in l13_prev_packet_meta["source_display_fragments"]:
    if fragment["ordinal"] not in (55988, 55989) or fragment["file"] != f"display-fragments/{fragment['ordinal']}.txt":
        raise ValueError("Lens 13 predecessor display path mismatch")
    data = (l13_prev_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 predecessor display hash mismatch")
    current = None
    lines_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, source_id = line.split(" ", 2)
            current = int(number)
            if current not in l13_prev_source_rows or l13_prev_source_rows[current]["source"]["source_id"] != source_id:
                raise ValueError("Lens 13 predecessor displayed source identity mismatch")
            continue
        number, separator, shown = line.partition(": ")
        if current is None or not separator or not number.isdigit():
            raise ValueError("Lens 13 predecessor displayed line format mismatch")
        n = int(number)
        source_sha = l13_prev_source_rows[current]["source"]["sha256"]
        source_lines = (l13_prev_dir / "sources" / source_sha).read_text(encoding="utf-8").splitlines()
        if shown != source_lines[n - 1]:
            raise ValueError("Lens 13 predecessor displayed line mismatch")
        l13_prev_displayed.setdefault(current, []).append(n)
        lines_count += 1
    if lines_count != fragment["line_occurrences"]:
        raise ValueError("Lens 13 predecessor display occurrence mismatch")
for entry in l13_prev_q["entries"]:
    expected = [n for low, high in entry["source"]["prospective_ranges"] for n in range(low, high + 1)]
    if entry["mode"] == "fresh":
        if l13_prev_displayed.get(entry["original"]) != expected:
            raise ValueError("Lens 13 predecessor fresh display coverage mismatch")
    elif entry["original"] in l13_prev_displayed:
        raise ValueError("Lens 13 inherited source misattributed as fresh")
if sum(len(lines) for lines in l13_prev_displayed.values()) != 573 or l13_prev_packet_meta["historical_reused_material_lines"] != 285:
    raise ValueError("Lens 13 predecessor historical source totals mismatch")

l13_prev_plan_meta = l13_prev_manifest["plan_candidate"]
l13_prev_plan_data = (l13_prev_dir / l13_prev_plan_meta["file"]).read_bytes()
if hashlib.sha256(l13_prev_plan_data).hexdigest() != l13_prev_plan_meta["sha256"]:
    raise ValueError("Lens 13 earlier plan candidate hash mismatch")
l13_prev_plan_candidate = json.loads(l13_prev_plan_data)
l13_prev_plan = l13_prev_plan_candidate["template"]
l13_prev_plan_evidence = {item["ordinal"]: item for item in l13_prev_plan_candidate["printed_predecessor_evidence"]}
if (set(l13_prev_plan_evidence) != {55913, 55937}
        or l13_prev_plan_candidate["searched_distinct_tokens"] != 16973
        or l13_prev_plan_candidate["matched_state_hashes"] != [] or l13_prev_plan_meta["match_found"] is not False
        or l13_prev_plan_candidate["plan_sha256"] != l13_prev_packet_result["plan"]["sha256"]
        or l13_prev_plan_meta["original_plan_sha256"] != l13_prev_plan_candidate["plan_sha256"]
        or l13_prev_plan["previous_state"] != {"path": "segovb/STATE-L13-PARALLEL-721.json", "sha256": "X" * 64}
        or l13_prev_plan["packet"] != l13_prev_packet_result["packet"]
        or l13_prev_plan["admission_2836_2845_material_qualification"]["sha256"] != l13_prev_q_sha
        or any(l13_prev_plan[key] != l13_prev_plan_evidence[55937][key] for key in ("ledger", "selector", "queue", "selected_lenses"))
        or l13_prev_plan["selected_lenses"] != l13_prev_plan_evidence[55913]["selected_lenses"]
        or (l13_prev_plan["root_sequence"], l13_prev_plan["queue_sequence"]) != (722, 765)
        or l13_prev_plan["next_selected_lenses_filename"] != "SELECTED-LENSES-CONTINUATION-414.json"):
    raise ValueError("Lens 13 earlier plan candidate reference mismatch")
l13_prev_plan_reuses = [{"kind": "exact_admission_2836_2845_material", "original": entry["original"],
                         "adoption": entry["prior"]["prior_adoption"],
                         "qualification": l13_prev_plan["admission_2836_2845_material_qualification"]}
                        for entry in l13_prev_q["entries"] if entry["mode"] != "fresh"]
if l13_prev_plan["prior_reuses"] != l13_prev_plan_reuses or len(l13_prev_plan_reuses) != 2:
    raise ValueError("Lens 13 earlier plan candidate reuse mismatch")
# The private transcript search is a recorded result and cannot be rerun from this package alone.

l13_prev_base = (l13_prev_dir / "BASE-HELPER.txt").read_text(encoding="utf-8")
l13_prev_derivation = (l13_prev_dir / "DERIVATION.txt").read_text(encoding="utf-8")
ast.parse(l13_prev_base)
ast.parse(l13_prev_derivation)
for fragment in ("'frontier':2805,'prior_same_lens_receipts_authenticated':723", "range(2805,2823)"):
    if fragment not in l13_prev_base or fragment not in l13_prev_derivation:
        raise ValueError("Lens 13 predecessor discovery derivation missing")

l13_earlier_dir = root / "recovered/l13-earlier-discovery"
l13_earlier = json.loads((l13_earlier_dir / "MANIFEST.json").read_bytes())
if l13_earlier["schema"] != "726-l13-earlier-discovery-v1":
    raise ValueError("Invalid earlier Lens 13 evidence inventory")
for item in l13_earlier["files"]:
    if hashlib.sha256((l13_earlier_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Earlier Lens 13 supporting evidence hash mismatch")
l13_earlier_result = json.loads((l13_earlier_dir / "PRINTED-HELPER-RESULT.json").read_bytes())
l13_earlier_helper = (l13_earlier_dir / l13_earlier["helper_sha256"]).read_bytes()
l13_earlier_command = ast.parse((l13_earlier_dir / "HELPER-WRITE-COMMAND.txt").read_text(encoding="utf-8"))
l13_earlier_literal = next(ast.literal_eval(node.value) for node in l13_earlier_command.body if isinstance(node, ast.Assign)
                           and any(isinstance(target, ast.Name) and target.id == "s" for target in node.targets))
if (hashlib.sha256(l13_earlier_helper).hexdigest() != l13_earlier["helper_sha256"]
        or l13_earlier_literal.encode("utf-8") != l13_earlier_helper
        or l13_earlier_result["helper"]["sha256"] != l13_earlier["helper_sha256"]
        or l13_earlier_result["registration"]["sha256"] != l13_earlier["registration_sha256"]
        or l13_earlier_result["qualification"]["sha256"] != l13_earlier["qualification_sha256"]):
    raise ValueError("Earlier Lens 13 original helper mismatch")
l13_earlier_discovery_meta = l13_earlier["exact_discovery"]
l13_earlier_discovery_data = (l13_earlier_dir / l13_earlier_discovery_meta["sha256"]).read_bytes()
l13_earlier_printed = json.loads((l13_earlier_dir / "PRINTED-DISCOVERY-RESULT.json").read_bytes())
if (hashlib.sha256(l13_earlier_discovery_data).hexdigest() != l13_earlier_discovery_meta["sha256"]
        or l13_earlier_printed["sha256"] != l13_earlier_discovery_meta["sha256"]
        or l13_earlier_discovery_meta["source_registry_sha256"] != l13_prev_manifest["source_registry_sha256"]):
    raise ValueError("Earlier Lens 13 original discovery hash mismatch")
l13_earlier_discovery = json.loads(l13_earlier_discovery_data)
if (l13_earlier_discovery["at"] != l13_earlier_discovery_meta["matched_timestamp"]
        or l13_earlier_discovery["frontier"] != 2823 or l13_earlier_discovery["prior_same_lens_receipts_authenticated"] != 724
        or [entry["original"] for entry in l13_earlier_discovery["entries"]] != list(range(2823, 2853))
        or l13_earlier_discovery["exact_hash_prior_hits"] != l13_earlier_printed["hits"]
        or l13_earlier_discovery["future_same_lens_proofs_navigation"] != l13_earlier_printed["futures"]
        or l13_earlier_discovery["prior_applications_preserved_navigation"] != l13_earlier_printed["apps"]
        or l13_earlier_discovery["body_credit"] != 0 or l13_earlier_discovery["authority_changes"] != 0):
    raise ValueError("Earlier Lens 13 discovery scope mismatch")
if [entry["registry_entry"] for entry in l13_earlier_discovery["entries"][13:23]] != [entry["registry_entry"] for entry in l13_prev["entries"]]:
    raise ValueError("Earlier Lens 13 original registry overlap mismatch")
ast.parse((l13_earlier_dir / "DISCOVERY-DERIVATION.txt").read_text(encoding="utf-8"))

l13_earlier_reg_meta = l13_earlier["exact_registration"]
l13_earlier_reg_data = (l13_earlier_dir / l13_earlier_reg_meta["sha256"]).read_bytes()
if (hashlib.sha256(l13_earlier_reg_data).hexdigest() != l13_earlier_reg_meta["sha256"]
        or l13_earlier_reg_meta["sha256"] != l13_earlier_result["registration"]["sha256"]):
    raise ValueError("Earlier Lens 13 original registration hash mismatch")
l13_earlier_reg = json.loads(l13_earlier_reg_data)
if (l13_earlier_reg["schema"] != "726-exact2823-2835-admission-registration-v1"
        or l13_earlier_reg["at"] != l13_earlier_reg_meta["matched_timestamp"]
        or len(l13_earlier_reg["sources"]) != 13 or len(l13_earlier_reg["valid_nested_range_fields"]) != 26
        or (l13_earlier_reg["fresh_lines_registered"], l13_earlier_reg["prior_material_lines_reused"]) != (425, 1965)
        or l13_earlier_reg["prior_same_lens_receipts_checked"] != 724
        or l13_earlier_reg["exact_hash_prior_hits"] != [hit for hit in l13_earlier_discovery["exact_hash_prior_hits"] if hit["original"] < 2836]
        or l13_earlier_reg["future_proofs_preserved"] != [] or l13_earlier_reg["old_ordered_applications_preserved"] != []
        or l13_earlier_reg["authority_changes"] != 0 or l13_earlier_reg["body_credit"] != 0
        or l13_earlier_reg["product_execution"] is not False):
    raise ValueError("Earlier Lens 13 registration scope mismatch")
for discovery, source, item in zip(l13_earlier_discovery["entries"][:13], l13_earlier_reg["sources"], l13_earlier_reg_meta["source_records"]):
    registry_row = discovery["registry_entry"]
    data = (l13_earlier_dir / "sources" / item["sha256"]).read_bytes()
    if (source["original"] != discovery["original"] or item["original"] != discovery["original"]
            or source["source_id"] != registry_row["source_id"] or source["sha256"] != registry_row["sha256"]
            or source["sha256"] != item["sha256"] or source["actual_eof"] != registry_row["actual_line_count"]
            or source["prospective_ranges"] != registry_row["material_ranges"]
            or hashlib.sha256(data).hexdigest() != item["sha256"] or len(data) != item["bytes"]
            or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Earlier Lens 13 registration source mismatch")
l13_earlier_display = l13_earlier["fresh_source_displays"]
if (l13_earlier_display["originals"] != [2823, 2825, 2827]
        or l13_earlier_display["historical_fresh_line_occurrences"] != 425
        or [item["ordinal"] for item in l13_earlier_display["fragments"]] != [55866, 55867]):
    raise ValueError("Earlier Lens 13 fresh display inventory mismatch")
l13_earlier_display_sources = {row["original"]: row for row in l13_earlier_reg["sources"]}
l13_earlier_shown = {}
for fragment in l13_earlier_display["fragments"]:
    if fragment["file"] != f"display-fragments/{fragment['ordinal']}.txt":
        raise ValueError("Earlier Lens 13 fresh display path mismatch")
    data = (l13_earlier_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Earlier Lens 13 fresh display hash mismatch")
    current = None
    l13_earlier_fragment_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if current not in l13_earlier_display["originals"] or l13_earlier_display_sources[current]["source_id"] != identity:
                raise ValueError("Earlier Lens 13 fresh display source mismatch")
            continue
        number, separator, shown = line.partition(": ")
        if current is None or not separator or not number.isdigit():
            raise ValueError("Earlier Lens 13 fresh display line format mismatch")
        n = int(number)
        source_sha = l13_earlier_display_sources[current]["sha256"]
        source_lines = (l13_earlier_dir / "sources" / source_sha).read_text(encoding="utf-8").splitlines()
        if shown != source_lines[n - 1]:
            raise ValueError("Earlier Lens 13 fresh displayed line mismatch")
        l13_earlier_shown.setdefault(current, []).append(n)
        l13_earlier_fragment_count += 1
    if l13_earlier_fragment_count != fragment["line_occurrences"]:
        raise ValueError("Earlier Lens 13 fresh display occurrence mismatch")
for n in l13_earlier_display["originals"]:
    expected = [line for low, high in l13_earlier_display_sources[n]["prospective_ranges"] for line in range(low, high + 1)]
    if l13_earlier_shown.get(n) != expected:
        raise ValueError("Earlier Lens 13 fresh display scope mismatch")
if sum(map(len, l13_earlier_shown.values())) != 425:
    raise ValueError("Earlier Lens 13 fresh display total mismatch")

l13_earlier_inherited = l13_earlier["inherited_evidence"]
if (len(l13_earlier_inherited["groups"]) != 3 or len(l13_earlier_inherited["objects"]) != 15
        or l13_earlier_inherited["delegated_roles"] != 7 or l13_earlier_inherited["delegated_material_lines"] != 1762):
    raise ValueError("Earlier Lens 13 inherited inventory mismatch")
l13_earlier_objects = {}
for item in l13_earlier_inherited["objects"]:
    data = (l13_earlier_dir / "inherited-objects" / item["sha256"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["sha256"]:
        raise ValueError("Earlier Lens 13 inherited evidence hash mismatch")
    l13_earlier_objects[item["sha256"]] = json.loads(data)
l13_earlier_sources = {item["original"]: item for item in l13_earlier_reg["sources"]}
l13_earlier_roles = set()
l13_earlier_lines = 0
for group in l13_earlier_inherited["groups"]:
    root_receipt = l13_earlier_objects[group["root"]["sha256"]]
    raw_receipt = l13_earlier_objects[group["delegated_receipt"]["sha256"]]
    if (root_receipt["delegated_receipt"] != group["delegated_receipt"]
            or root_receipt["delegated_verification"] != group["delegated_verification"]
            or root_receipt["agent_validation_manifest"] != group["agent_validation_manifest"]
            or root_receipt["evidence"] != raw_receipt or raw_receipt["registration"] != group["registration"]
            or raw_receipt.get("partial_checkpoints", []) != group["partials"]):
        raise ValueError("Earlier Lens 13 inherited receipt linkage mismatch")
    for kind in ("delegated_receipt", "delegated_verification", "agent_validation_manifest", "registration"):
        if group[kind]["sha256"] not in l13_earlier_objects:
            raise ValueError("Earlier Lens 13 inherited proof object missing")
    for ref in group["partials"]:
        if ref["sha256"] not in l13_earlier_objects:
            raise ValueError("Earlier Lens 13 inherited partial missing")
    for mapping in group["mappings"]:
        source = l13_earlier_sources[mapping["original"]]
        prior = raw_receipt["rows"][mapping["receipt_row"]]
        wanted = {n for low, high in source["prospective_ranges"] for n in range(low, high + 1)}
        observed = {n for low, high in prior["fresh_personal_assessed_ranges"] for n in range(low, high + 1)}
        if (prior["original"] != mapping["prior_original"] or prior["sha256"] != source["sha256"]
                or prior["actual_eof"] != source["actual_eof"]
                or prior["source_id"].split(":", 1)[1] != source["source_id"].split(":", 1)[1]
                or not wanted.issubset(observed) or len(wanted) != mapping["material_lines"]):
            raise ValueError("Earlier Lens 13 inherited source scope mismatch")
        l13_earlier_roles.add(mapping["original"])
        l13_earlier_lines += len(wanted)
if len(l13_earlier_roles) != 7 or l13_earlier_lines != 1762:
    raise ValueError("Earlier Lens 13 inherited scope count mismatch")
l13_earlier_root_rows = json.loads((l13_earlier_dir / "PRINTED-ROOT-REUSE-ROWS.json").read_bytes())
if (l13_earlier_root_rows["source_output_ordinal"] != 55843
        or [item["original"] for item in l13_earlier_root_rows["records"]] != [2824, 2831, 2832]
        or [item["complete_json"] for item in l13_earlier_root_rows["records"]] != [True, False, True]):
    raise ValueError("Earlier Lens 13 root row projection inventory mismatch")
for item in l13_earlier_root_rows["records"]:
    source = l13_earlier_sources[item["original"]]
    if source["sha256"] not in item["captured_json_or_prefix"]:
        raise ValueError("Earlier Lens 13 root row source hash missing")
    if item["complete_json"]:
        prior = json.loads(item["captured_json_or_prefix"])
        wanted = {n for low, high in source["prospective_ranges"] for n in range(low, high + 1)}
        observed = {n for low, high in prior["assessed_ranges"] for n in range(low, high + 1)}
        if (prior["source"]["sha256"] != source["sha256"] or prior["source"]["actual_eof"] != source["actual_eof"]
                or not wanted.issubset(observed)):
            raise ValueError("Earlier Lens 13 printed root source mismatch")

l13_earlier_assessments = json.loads((l13_earlier_dir / "PRINTED-ROOT-OBSERVATIONS.json").read_bytes())
if (l13_earlier_assessments["source_output_ordinal"] != 55875
        or [item["original"] for item in l13_earlier_assessments["records"]] != [2824, 2831, 2832]
        or l13_earlier_inherited["root_assessment_projections_complete"] != 3):
    raise ValueError("Earlier Lens 13 root assessment inventory mismatch")
for observed, projected in zip(l13_earlier_assessments["records"], l13_earlier_root_rows["records"]):
    if observed["original"] != projected["original"]:
        raise ValueError("Earlier Lens 13 assessment identity mismatch")
    if projected["complete_json"]:
        if json.loads(projected["captured_json_or_prefix"])["observations"] != observed["observations"]:
            raise ValueError("Earlier Lens 13 complete assessment mismatch")
    else:
        prefix = projected["captured_json_or_prefix"].split('"observations": ', 1)[1]
        if not json.dumps(observed["observations"], ensure_ascii=True).startswith(prefix):
            raise ValueError("Earlier Lens 13 truncated assessment prefix mismatch")
    source = l13_earlier_sources[observed["original"]]
    wanted = {n for low, high in source["prospective_ranges"] for n in range(low, high + 1)}
    assessed = {n for observation in observed["observations"] for low, high in observation["ranges"] for n in range(low, high + 1)}
    if not wanted.issubset(assessed):
        raise ValueError("Earlier Lens 13 assessment scope mismatch")

l13_563_meta = l13_earlier["root_563_registration"]
l13_563_data = (l13_earlier_dir / l13_563_meta["sha256"]).read_bytes()
l13_563_print = json.loads((l13_earlier_dir / "PRINTED-ROOT-563-REGISTRATION.json").read_bytes())
l13_563_adoption = json.loads((l13_earlier_dir / "PRINTED-ROOT-563-ADOPTION.json").read_bytes())
l13_563_writer = (l13_earlier_dir / "ROOT-563-REGISTRATION-WRITER.txt").read_text(encoding="utf-8")
ast.parse(l13_563_writer)
if (hashlib.sha256(l13_563_data).hexdigest() != l13_563_meta["sha256"]
        or l13_563_print["registration_sha256"] != l13_563_meta["sha256"]
        or "assert count==562 and not hits" not in l13_563_writer or "assert not future" not in l13_563_writer):
    raise ValueError("Lens 13 root 563 registration provenance mismatch")
l13_563_reg = json.loads(l13_563_data)
if (l13_563_reg["at"] != l13_563_meta["matched_timestamp"]
        or l13_563_reg["sources"] != l13_563_print["sources"]
        or [row["original"] for row in l13_563_reg["sources"]] != [2058, 2059, 2060]
        or l13_563_reg["prior_same_lens_receipts_checked"] != 562
        or l13_563_reg["exact_hash_prior_hits"] != [] or l13_563_reg["future_same_lens_proof_hits"] != []
        or l13_563_reg["authority_changes"] != 0 or l13_563_reg["source_credit_from_materialization"] != 0
        or l13_563_reg["product_execution"] is not False):
    raise ValueError("Lens 13 root 563 registration scope mismatch")
for row, item in zip(l13_563_reg["sources"], l13_563_meta["source_records"]):
    data = (l13_earlier_dir / "root-563-sources" / item["sha256"]).read_bytes()
    if (row["original"] != item["original"] or row["sha256"] != item["sha256"]
            or hashlib.sha256(data).hexdigest() != item["sha256"] or len(data) != item["bytes"]
            or len(data.splitlines()) != row["actual_eof"]):
        raise ValueError("Lens 13 root 563 source mismatch")
l13_563_row = json.loads(l13_earlier_root_rows["records"][0]["captured_json_or_prefix"])
if (l13_563_row["registration"]["sha256"] != l13_563_meta["sha256"]
        or l13_563_row["registration_pointer"] != "/sources/1" or l13_563_row["source"] != l13_563_reg["sources"][1]
        or l13_563_adoption["source_adoption"] != next(hit["receipt"] for hit in l13_earlier_discovery["exact_hash_prior_hits"] if hit["original"] == 2824)
        or l13_563_adoption["preflight"] != "passed" or l13_563_adoption["new_body_credit"] != 0):
    raise ValueError("Lens 13 root 563 historical receipt reference mismatch")

l13_563_displays = l13_563_meta["historical_source_displays"]
if (l13_563_displays["roles"] != 3 or l13_563_displays["fresh_historical_lines"] != 683
        or [row["ordinal"] for row in l13_563_displays["fragments"]] != [29960, 29961, 29962, 29971, 29972, 29977, 29978]):
    raise ValueError("Lens 13 root 563 historical display inventory mismatch")
l13_563_sources = {row["original"]: row for row in l13_563_reg["sources"]}
l13_563_shown = {}
for fragment in l13_563_displays["fragments"]:
    source = l13_563_sources[fragment["original"]]
    data = (l13_earlier_dir / "root-563-displays" / f"{fragment['ordinal']}.txt").read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 563 display hash mismatch")
    source_lines = (l13_earlier_dir / "root-563-sources" / source["sha256"]).read_text(encoding="utf-8").splitlines()
    numbers = []
    for line in data.decode("utf-8").splitlines():
        number, separator, shown = line.partition(": ")
        if not separator or not number.isdigit():
            raise ValueError("Lens 13 root 563 display format mismatch")
        number = int(number)
        if shown != source_lines[number - 1]:
            raise ValueError("Lens 13 root 563 displayed source mismatch")
        numbers.append(number)
    if numbers != list(range(fragment["first_line"], fragment["last_line"] + 1)):
        raise ValueError("Lens 13 root 563 displayed range mismatch")
    l13_563_shown.setdefault(fragment["original"], []).extend(numbers)
for original, source in l13_563_sources.items():
    if l13_563_shown.get(original) != list(range(1, source["actual_eof"] + 1)):
        raise ValueError("Lens 13 root 563 displayed source coverage mismatch")
if sum(map(len, l13_563_shown.values())) != 683:
    raise ValueError("Lens 13 root 563 historical display total mismatch")

l13_691_display = l13_earlier["root_691_fresh_display"]
l13_691_command = (l13_earlier_dir / "ROOT-691-DISPLAY-COMMAND.txt").read_text(encoding="utf-8")
ast.parse(l13_691_command)
if (l13_691_display["source_output_ordinal"] != 50176 or l13_691_display["historical_lines"] != 264
        or l13_691_display["qualification_sha256"] not in l13_691_command
        or [row["original"] for row in l13_691_display["source_records"]] != [2592, 2595, 2596]):
    raise ValueError("Lens 13 root 691 fresh display provenance mismatch")
l13_691_data = (l13_earlier_dir / "ROOT-691-FRESH-DISPLAY.txt").read_bytes()
if hashlib.sha256(l13_691_data).hexdigest() != l13_691_display["sha256"]:
    raise ValueError("Lens 13 root 691 fresh display hash mismatch")
l13_691_sources = {row["original"]: row for row in l13_691_display["source_records"]}
l13_691_shown = {}
l13_691_current = None
for line in l13_691_data.decode("utf-8").splitlines():
    if line.startswith("FRESH "):
        head, raw_ranges = line.split(" SCOPE ", 1)
        _, number, identity = head.split(" ", 2)
        l13_691_current = int(number)
        source = l13_691_sources[l13_691_current]
        if source["source_id"] != identity or source["ranges"] != json.loads(raw_ranges):
            raise ValueError("Lens 13 root 691 fresh source identity mismatch")
        continue
    number, separator, shown = line.partition(" ")
    if l13_691_current is None or not separator or not number.isdigit():
        raise ValueError("Lens 13 root 691 fresh display format mismatch")
    source = l13_691_sources[l13_691_current]
    source_data = (l13_earlier_dir / "root-691-sources" / source["sha256"]).read_bytes()
    if hashlib.sha256(source_data).hexdigest() != source["sha256"] or len(source_data) != source["bytes"]:
        raise ValueError("Lens 13 root 691 fresh source hash mismatch")
    number = int(number)
    if shown != source_data.decode("utf-8").splitlines()[number - 1]:
        raise ValueError("Lens 13 root 691 fresh displayed line mismatch")
    l13_691_shown.setdefault(l13_691_current, []).append(number)
for original, source in l13_691_sources.items():
    expected = [line for low, high in source["ranges"] for line in range(low, high + 1)]
    if l13_691_shown.get(original) != expected:
        raise ValueError("Lens 13 root 691 fresh display scope mismatch")
if (sum(map(len, l13_691_shown.values())) != 264
        or l13_691_sources[2595]["sha256"] != l13_earlier_sources[2831]["sha256"]
        or l13_691_sources[2596]["sha256"] != l13_earlier_sources[2832]["sha256"]):
    raise ValueError("Lens 13 root 691 fresh source reuse mismatch")

l13_691_helpers = l13_earlier["root_691_qualification_helpers"]
for name, expected in (("intermediate", "8ec5f1d203dc577ba44669354a313c6f921ef02cd7f463213c9afabbb084f776"),
                       ("final", "6385d471d045c7d00ebeaa037d4f677d2400c23fad1540e8ff2b5ec378f69833")):
    row = l13_691_helpers[name]
    data = (l13_earlier_dir / row["file"]).read_bytes()
    if row["file"] != expected or row["sha256"] != expected or len(data) != row["bytes"] or hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 691 qualification helper mismatch")
    ast.parse(data.decode("utf-8"))
l13_691_registration_result = (l13_earlier_dir / l13_691_helpers["printed_result_file"]).read_bytes()
if hashlib.sha256(l13_691_registration_result).hexdigest() != l13_691_helpers["printed_result_sha256"]:
    raise ValueError("Lens 13 root 691 printed registration result mismatch")
l13_691_registration_result = json.loads(l13_691_registration_result)
if (l13_691_registration_result["helper"]["sha256"] != l13_691_helpers["final"]["sha256"]
        or l13_691_registration_result["registration"]["sha256"] != l13_691_display["registration_sha256"]
        or l13_691_registration_result["qualification"]["sha256"] != l13_691_display["qualification_sha256"]
        or (l13_691_registration_result["fresh"], l13_691_registration_result["reused"], l13_691_registration_result["nested"]) != (264, 596, 26)):
    raise ValueError("Lens 13 root 691 printed registration result identities mismatch")

l13_691_objects = l13_earlier["root_691_registered_source_objects"]["sources"]
l13_691_helper_ast = ast.parse((l13_earlier_dir / l13_691_helpers["final"]["file"]).read_text(encoding="utf-8"))
l13_691_specs = next(ast.literal_eval(node.value) for node in ast.walk(l13_691_helper_ast)
                     if isinstance(node, ast.Assign) and len(node.targets) == 1
                     and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "specs")
if len(l13_691_objects) != 8 or len(l13_691_specs) != 8:
    raise ValueError("Lens 13 root 691 source object count mismatch")
for source, spec in zip(l13_691_objects, l13_691_specs):
    original, _, _, _, _, eof, size, ranges, _ = spec
    data = (l13_earlier_dir / source["file"]).read_bytes()
    if (source["original"] != original or source["actual_eof"] != eof or source["bytes"] != size
            or source["material_ranges"] != ranges or len(data) != size or len(data.splitlines()) != eof
            or hashlib.sha256(data).hexdigest() != source["sha256"]
            or source["file"] != "root-691-sources/" + source["sha256"]):
        raise ValueError("Lens 13 root 691 source object mismatch")
l13_691_writer = l13_earlier["root_691_registration_writer_candidate"]
l13_691_126 = l13_earlier["root_691_registered_source_objects"]["additional_prior_receipt"]
l13_691_126_data = (l13_earlier_dir / l13_691_126["file"]).read_bytes()
if (l13_691_126["sha256"] != "3f5f759ae773741ba26754e95f6283ebfad5243b1f2be16f3ef72afc4cf158a1"
        or hashlib.sha256(l13_691_126_data).hexdigest() != l13_691_126["sha256"]
        or len(l13_691_126_data) != l13_691_126["bytes"]):
    raise ValueError("Lens 13 root 691 prior receipt 126 mismatch")
l13_691_126_receipt = json.loads(l13_691_126_data)
for index, original in zip(l13_691_126["connected_rows"], l13_691_126["originals"]):
    prior = l13_691_126_receipt["evidence"]["connected"][index]
    source = next(row for row in l13_691_objects if row["original"] == original)
    if (prior["original"] != original or prior["source_id"] != source["source_id"]
            or prior["sha256"] != source["sha256"] or prior["actual_eof"] != source["actual_eof"]):
        raise ValueError("Lens 13 root 691 prior receipt source mismatch")
l13_691_writer_data = (l13_earlier_dir / l13_691_writer["file"]).read_bytes()
if hashlib.sha256(l13_691_writer_data).hexdigest() != l13_691_writer["sha256"]:
    raise ValueError("Lens 13 root 691 writer candidate hash mismatch")
ast.parse(l13_691_writer_data.decode("utf-8"))

l13_691_prior = l13_earlier["root_691_prior_source_metadata"]
for name, key in (("command_file", "command_sha256"), ("metadata_file", "metadata_sha256")):
    data = (l13_earlier_dir / l13_691_prior[name]).read_bytes()
    if hashlib.sha256(data).hexdigest() != l13_691_prior[key]:
        raise ValueError("Lens 13 root 691 prior source evidence hash mismatch")
ast.parse((l13_earlier_dir / l13_691_prior["command_file"]).read_text(encoding="utf-8"))
l13_691_prior_data = json.loads((l13_earlier_dir / l13_691_prior["metadata_file"]).read_text(encoding="utf-8"))
if (l13_691_prior_data["transcript_output_ordinal"] != 50130
        or [row["root_receipt_number"] for row in l13_691_prior_data["roots"]] != [109, 188, 371]):
    raise ValueError("Lens 13 root 691 prior source provenance mismatch")
l13_691_prior_rows = {row["root_receipt_number"]: row["source_rows"][0]
                      for row in l13_691_prior_data["roots"]}
if [(n, row[0], row[1], row[2]["original"]) for n, row in l13_691_prior_rows.items()] != [
        (109, "connected", 0, 2597), (188, "connected", 1, 2248), (371, "ordered", 5, 1482)]:
    raise ValueError("Lens 13 root 691 prior source pointer mismatch")
for receipt in l13_691_prior["receipt_objects"]:
    data = (l13_earlier_dir / receipt["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != receipt["sha256"] or len(data) != receipt["bytes"]:
        raise ValueError("Lens 13 root 691 prior receipt object mismatch")
    row = l13_691_prior_rows[receipt["receipt"]]
    actual = json.loads(data)["evidence"][row[0]][row[1]]
    if actual.get("source", actual) != row[2]:
        raise ValueError("Lens 13 root 691 printed metadata differs from prior receipt")
if {row["sha256"] for row in l13_691_prior["receipt_objects"]} != {
        "e0871ab6712bbfffc3442b90c3e4e1af80e5759b8a018758d8c566eabcc7f83a",
        "46c07471cbe565050b808c9037babfe4a3c82bc3b30b02f499ec086f16c4fbc1"}:
    raise ValueError("Lens 13 root 691 prior receipt identities mismatch")

l13_691_navigation = l13_earlier["root_691_discovery_navigation"]
l13_691_discovery_hash = "be50051d71b8da783bd7066602acfbf8678b2c8f7d380f6895e32fc844602c32"
l13_691_reg_hash = "d6a4d2ba55cf53e493451cf9c88c7a707589f624f61b40b2ea755667e787f365"
l13_691_discovery_data = (l13_earlier_dir / l13_691_discovery_hash).read_bytes()
l13_691_reg_data = (l13_earlier_dir / l13_691_reg_hash).read_bytes()
if (hashlib.sha256(l13_691_discovery_data).hexdigest() != l13_691_discovery_hash
        or hashlib.sha256(l13_691_reg_data).hexdigest() != l13_691_reg_hash):
    raise ValueError("Lens 13 root 691 discovery or registration hash mismatch")
l13_691_discovery, l13_691_reg = json.loads(l13_691_discovery_data), json.loads(l13_691_reg_data)
if (l13_691_discovery["original_before"] != 2592
        or [row["original"] for row in l13_691_discovery["entries"]] != list(range(2592, 2600))
        or len(l13_691_discovery["exact_hash_prior_hits"]) != 10
        or len(l13_691_discovery["all_nested_range_fields"]) != 26
        or len(l13_691_discovery["out_of_eof_raw_fields_requiring_exact_qualification"]) != 2
        or [row["original"] for row in l13_691_reg["sources"]] != list(range(2592, 2600))
        or l13_691_reg["exact_hash_prior_hits"] != l13_691_discovery["exact_hash_prior_hits"]
        or l13_691_reg["future_proofs_preserved"] != l13_691_discovery["future_same_lens_proofs_navigation"]
        or l13_691_reg["old_ordered_applications_preserved"] != l13_691_discovery["prior_applications_preserved_navigation"]
        or [row["original"] for row in l13_691_reg["old_ordered_applications_preserved"]] != [2248]):
    raise ValueError("Lens 13 root 691 discovery and registration contents mismatch")
for source, entry in zip(l13_691_reg["sources"], l13_691_discovery["entries"]):
    if source["original"] != entry["original"] or source["sha256"] != entry["registry_entry"]["sha256"]:
        raise ValueError("Lens 13 root 691 registered source identity mismatch")
for name, key in (("command_file", "command_sha256"), ("printed_result_file", "printed_result_sha256")):
    data = (l13_earlier_dir / l13_691_navigation[name]).read_bytes()
    if hashlib.sha256(data).hexdigest() != l13_691_navigation[key]:
        raise ValueError("Lens 13 root 691 navigation evidence hash mismatch")
ast.parse((l13_earlier_dir / l13_691_navigation["command_file"]).read_text(encoding="utf-8"))
l13_691_result = json.loads((l13_earlier_dir / l13_691_navigation["printed_result_file"]).read_text(encoding="utf-8"))
if (l13_691_navigation["output_ordinal"] != 50109
        or l13_691_result["discovery"]["sha256"] != l13_691_navigation["original_discovery_sha256"]
        or len(l13_691_result["hits"]) != 10
        or len({row[1] for row in l13_691_result["hits"]}) != 7
        or l13_691_result["future_originals"] != [2597, 2598, 2599, 2248]
        or l13_691_result["nested_fields"] != 26
        or len(l13_691_result["out_of_eof_fields"]) != 2):
    raise ValueError("Lens 13 root 691 printed discovery mismatch")

l13_691_reuse = l13_earlier["root_691_reuse_display"]
l13_691_reuse_data = (l13_earlier_dir / l13_691_reuse["file"]).read_bytes()
if (hashlib.sha256(l13_691_reuse_data).hexdigest() != l13_691_reuse["sha256"]
        or l13_691_reuse["source_output_ordinal"] != 50176
        or l13_691_reuse["qualification_sha256"] not in l13_691_command):
    raise ValueError("Lens 13 root 691 reuse display provenance mismatch")
l13_691_reuse_text = l13_691_reuse_data.decode("utf-8")
l13_691_headers = re.findall(r"^REUSE (\d+) (\{[^\n]+\}) (\S+)$", l13_691_reuse_text, re.M)
l13_691_expected = [
    (2593, "371", "ba32f276addc7064922a2f7c2c56766c453f1d109fb6e26d6c92d850554e7553", "/ordered/5"),
    (2594, "188", "46c07471cbe565050b808c9037babfe4a3c82bc3b30b02f499ec086f16c4fbc1", "/connected/1"),
    (2597, "109", "e0871ab6712bbfffc3442b90c3e4e1af80e5759b8a018758d8c566eabcc7f83a", "/connected/0"),
    (2598, "126", "3f5f759ae773741ba26754e95f6283ebfad5243b1f2be16f3ef72afc4cf158a1", "/connected/2"),
    (2599, "126", "3f5f759ae773741ba26754e95f6283ebfad5243b1f2be16f3ef72afc4cf158a1", "/connected/3"),
]
if len(l13_691_headers) != len(l13_691_expected):
    raise ValueError("Lens 13 root 691 reuse row count mismatch")
for (number, raw_ref, pointer), (original, receipt, digest, expected_pointer) in zip(l13_691_headers, l13_691_expected):
    if (int(number) != original or ast.literal_eval(raw_ref) != {
            "path": f"segovb/READ-L13-PARALLEL-{receipt}.json", "sha256": digest}
            or pointer != expected_pointer):
        raise ValueError("Lens 13 root 691 prior receipt mismatch")
for label, expected in (("OBS", 1), ("INVENTORY", 3), ("CONTRACTS", 3)):
    matches = list(re.finditer(r"^" + label + r" (?=\[)", l13_691_reuse_text, re.M))
    if len(matches) != expected:
        raise ValueError("Lens 13 root 691 reuse detail count mismatch")
    for match in matches:
        value, _ = json.JSONDecoder().raw_decode(l13_691_reuse_text[match.end():])
        if not isinstance(value, list):
            raise ValueError("Lens 13 root 691 reuse detail is not an array")
if (l13_691_reuse["originals"] != [row[0] for row in l13_691_expected]
        or l13_691_reuse["unique_receipts"] != 4):
    raise ValueError("Lens 13 root 691 reuse manifest mismatch")

l13_371_dir = root / "recovered/l13-root-371"
l13_371_manifest = json.loads((l13_371_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_371_hash = "b528062675b11c2e94b9a5dbe81a34234992791199bf6c759e759a05c192fe09"
l13_371_reg_data = (l13_371_dir / l13_371_hash).read_bytes()
if (l13_371_manifest["schema"] != "726-l13-root-371-registration-and-display-v1"
        or l13_371_manifest["registration_sha256"] != l13_371_hash
        or hashlib.sha256(l13_371_reg_data).hexdigest() != l13_371_hash):
    raise ValueError("Lens 13 root 371 registration mismatch")
l13_371_reg = json.loads(l13_371_reg_data)
if ([row["original"] for row in l13_371_reg["sources"]] != list(range(1477, 1486))
        or l13_371_reg["prior_same_lens_receipts_checked"] != 308
        or l13_371_reg["exact_hash_prior_hits"] or l13_371_reg["future_same_lens_proof_hits"]):
    raise ValueError("Lens 13 root 371 registration contents mismatch")
l13_371_sources = {row["sha256"]: row for row in l13_371_reg["sources"]}
l13_371_coverage = {digest: [] for digest in l13_371_sources}
for row in l13_371_manifest["sources"]:
    data = (l13_371_dir / "sources" / row["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != row["actual_eof"]
            or l13_371_sources[row["sha256"]]["original"] != row["original"]):
        raise ValueError("Lens 13 root 371 source object mismatch")
for fragment in l13_371_manifest["display_fragments"]:
    data = (l13_371_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 371 display hash mismatch")
    current = fragment["default_source_sha256"]
    l13_371_fragment_count = 0
    for line in data.decode("utf-8").splitlines():
        header = re.fullmatch(r"SOURCE ([0-9a-f]{64}) EOF (\d+)", line)
        if header:
            current = header[1]
            if current not in l13_371_sources or int(header[2]) != l13_371_sources[current]["actual_eof"]:
                raise ValueError("Lens 13 root 371 display header mismatch")
            continue
        number, separator, shown = line.partition(": ")
        if current not in l13_371_sources or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 371 display format mismatch")
        number = int(number)
        if shown != (l13_371_dir / "sources" / current).read_text(encoding="utf-8").splitlines()[number - 1]:
            raise ValueError("Lens 13 root 371 displayed source mismatch")
        l13_371_coverage[current].append(number)
        l13_371_fragment_count += 1
    if l13_371_fragment_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 371 display count mismatch")
if (len(l13_371_manifest["display_fragments"]) != 9
        or sum(map(len, l13_371_coverage.values())) != 2178
        or any(lines != list(range(1, l13_371_sources[h]["actual_eof"] + 1))
               for h, lines in l13_371_coverage.items())):
    raise ValueError("Lens 13 root 371 complete historical display mismatch")
l13_371_obs_data = (l13_371_dir / "OBSERVATIONS.json").read_bytes()
l13_371_literal = (l13_371_dir / "OBSERVATIONS-WRITER-LITERAL.txt").read_bytes()
if (hashlib.sha256(l13_371_obs_data).hexdigest() != l13_371_manifest["observations_sha256"]
        or hashlib.sha256(l13_371_literal).hexdigest() != l13_371_manifest["observation_literal_sha256"]):
    raise ValueError("Lens 13 root 371 observation evidence mismatch")
l13_371_obs = json.loads(l13_371_obs_data)["observations_by_registered_source"]
if (json.loads(json.dumps(ast.literal_eval(ast.parse(l13_371_literal.decode("utf-8")).body[0].value))) != l13_371_obs
        or len(l13_371_obs) != 9 or sum(map(len, l13_371_obs)) != 24):
    raise ValueError("Lens 13 root 371 observation literal mismatch")
l13_371_packet_data = (l13_371_dir / l13_371_manifest["packet_sha256"]).read_bytes()
l13_371_plan_data = (l13_371_dir / l13_371_manifest["plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_371_packet_data).hexdigest() != "cf67f3c1eb1990f6cb14be75c5b59006ba1ad6014cf5c8eb7ba53cc20187ca2c"
        or hashlib.sha256(l13_371_plan_data).hexdigest() != "b8e829f1a920026d1d9f46a3d927fcc63610be95cc51cde26e885e797bb9f36c"
        or len(l13_371_packet_data) != l13_371_manifest["packet_bytes"]
        or len(l13_371_plan_data) != l13_371_manifest["plan_bytes"]):
    raise ValueError("Lens 13 root 371 packet or plan hash mismatch")
l13_371_packet, l13_371_plan = json.loads(l13_371_packet_data), json.loads(l13_371_plan_data)
l13_371_receipt_data = (l13_371_dir / l13_371_manifest["root_adoption_sha256"]).read_bytes()
l13_371_state_data = (l13_371_dir / l13_371_manifest["state_sha256"]).read_bytes()
if (hashlib.sha256(l13_371_receipt_data).hexdigest() != "ba32f276addc7064922a2f7c2c56766c453f1d109fb6e26d6c92d850554e7553"
        or hashlib.sha256(l13_371_state_data).hexdigest() != "6820171553ccce322eea61a1b2a52c201c7b290650fdf78f6f3abed080825034"):
    raise ValueError("Lens 13 root 371 receipt or state hash mismatch")
l13_371_receipt, l13_371_state = json.loads(l13_371_receipt_data), json.loads(l13_371_state_data)
l13_691_qual_hash = "86498a32b09ed3a5c2cb415365b9d2714026e60caf79a5072227b29c12013d72"
l13_691_qual_data = (l13_earlier_dir / l13_691_qual_hash).read_bytes()
l13_371_verify_data = (l13_371_dir / l13_371_manifest["native_verification_sha256"]).read_bytes()
if (hashlib.sha256(l13_691_qual_data).hexdigest() != l13_691_qual_hash
        or hashlib.sha256(l13_371_verify_data).hexdigest() != l13_371_manifest["native_verification_sha256"]):
    raise ValueError("Lens 13 root 691 qualification or root 371 verification hash mismatch")
l13_691_qual, l13_371_verify = json.loads(l13_691_qual_data), json.loads(l13_371_verify_data)
l13_691_packet_manifest = l13_earlier["root_691_packet"]
l13_691_packet_data = (l13_earlier_dir / l13_691_packet_manifest["sha256"]).read_bytes()
l13_691_plan_data = (l13_earlier_dir / l13_691_packet_manifest["plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_691_packet_data).hexdigest() != "6a8f37bc31cb70242165619c102ddb19e4f65875123533679ffe6f9f51ef60e1"
        or hashlib.sha256(l13_691_plan_data).hexdigest() != "9a098d171f0de2f41b6e0839d5923393f440f0ed10489bb3f6f546d6249675e5"):
    raise ValueError("Lens 13 root 691 packet or plan hash mismatch")
l13_691_packet, l13_691_plan = json.loads(l13_691_packet_data), json.loads(l13_691_plan_data)
l13_691_receipt_data = (l13_earlier_dir / l13_691_packet_manifest["adoption_receipt_sha256"]).read_bytes()
l13_691_state_data = (l13_earlier_dir / l13_691_packet_manifest["state_sha256"]).read_bytes()
l13_692_plan_data = (l13_earlier_dir / l13_691_packet_manifest["successor_plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_691_receipt_data).hexdigest() != "80960e5eb8281018a4087dbe007a17c451974e7cc7021ef4963e54688b6e37e9"
        or hashlib.sha256(l13_691_state_data).hexdigest() != "e4cc900a56beda840c4e65c6057b6416a42ee178e65f8f23a84d23f0d2ae0b63"
        or hashlib.sha256(l13_692_plan_data).hexdigest() != "ef600e41824c786289f6b184e3d6d90dd11371d3c7bb51777e243f56b4bbdbaa"):
    raise ValueError("Lens 13 root 691 receipt, state or successor plan hash mismatch")
l13_691_receipt, l13_691_state = json.loads(l13_691_receipt_data), json.loads(l13_691_state_data)
l13_692_plan = json.loads(l13_692_plan_data)
if (l13_691_receipt["evidence"] != l13_691_packet
        or l13_691_receipt["packet"]["sha256"] != l13_691_packet_manifest["sha256"]
        or l13_691_receipt["plan"]["sha256"] != l13_691_packet_manifest["plan_sha256"]
        or l13_691_receipt["prior_same_lens_reuses"] != l13_691_plan["prior_reuses"]
        or len(l13_691_receipt["qualified_ordered_reuses"]) != 5
        or l13_691_state["receipt_sha256"] != l13_691_packet_manifest["adoption_receipt_sha256"]
        or l13_691_state["predecessor_sha256"] != l13_691_plan["previous_state"]["sha256"]
        or l13_692_plan["previous_state"]["sha256"] != l13_691_packet_manifest["state_sha256"]
        or l13_692_plan["root_sequence"] != 692):
    raise ValueError("Lens 13 root 691 adoption chain mismatch")
for projection in json.loads((l13_earlier_dir / "PRINTED-ROOT-REUSE-ROWS.json").read_text(encoding="utf-8"))["records"]:
    if projection["historical_root_sequence"] == 691 and projection["complete_json"]:
        if json.loads(projection["captured_json_or_prefix"]) != l13_691_packet["ordered"][projection["row_index"]]:
            raise ValueError("Lens 13 root 691 later reuse projection mismatch")
if (l13_691_plan["packet"]["sha256"] != l13_691_packet_manifest["sha256"]
        or l13_691_plan["admission_2592_2599_material_qualification"]["sha256"] != l13_691_qual_hash
        or l13_691_plan["root_sequence"] != 691 or l13_691_plan["queue_sequence"] != 703
        or len(l13_691_packet["ordered"]) != 8 or len(l13_691_plan["prior_reuses"]) != 5
        or len(l13_691_packet["contract_revisits_pending_queue_adoption"]) != 7
        or len(l13_691_packet["new_questions"]) != 1
        or [row["source"] for row in l13_691_packet["ordered"]] != l13_691_reg["sources"]):
    raise ValueError("Lens 13 root 691 packet chain mismatch")
if (l13_691_qual["registration"]["sha256"] != l13_691_reg_hash
        or l13_691_qual["proof_builder"]["sha256"] != l13_691_helpers["final"]["sha256"]
        or l13_691_qual["discovery_metadata_sha256"] != l13_691_discovery_hash
        or [row["source"] for row in l13_691_qual["entries"]] != l13_691_reg["sources"]
        or l13_691_qual["entries"][1]["prior"]["prior_native_verification"]["sha256"] != l13_371_manifest["native_verification_sha256"]
        or l13_371_verify["receipt_sha256"] != l13_371_manifest["root_adoption_sha256"]
        or l13_371_verify["state_sha256"] != l13_371_manifest["state_sha256"]
        or l13_691_qual["material_lines"] != 860 or l13_691_qual["fresh_lines"] != 264
        or l13_691_qual["reused_material_lines"] != 596):
    raise ValueError("Lens 13 root 691 qualification chain mismatch")
if (l13_371_receipt["evidence"] != l13_371_packet
        or l13_371_receipt["packet"]["sha256"] != l13_371_manifest["packet_sha256"]
        or l13_371_receipt["plan"]["sha256"] != l13_371_manifest["plan_sha256"]
        or l13_371_state["receipt_sha256"] != l13_371_manifest["root_adoption_sha256"]
        or l13_371_state["predecessor_sha256"] != l13_371_plan["previous_state"]["sha256"]
        or l13_371_state["next_original"] != 1486
        or l13_371_receipt["evidence"]["ordered"][5]["source"] != l13_691_prior_rows[371][2]):
    raise ValueError("Lens 13 root 371 adoption chain mismatch")
if (l13_371_plan["packet"]["sha256"] != l13_371_manifest["packet_sha256"]
        or l13_371_plan["root_sequence"] != 371 or l13_371_plan["queue_sequence"] != 221
        or len(l13_371_packet["ordered"]) != 9
        or len(l13_371_packet["contract_revisits_pending_queue_adoption"]) != 4
        or len(l13_371_packet["new_questions"]) != 1):
    raise ValueError("Lens 13 root 371 packet or plan structure mismatch")
for index, row in enumerate(l13_371_packet["ordered"]):
    if (row["source"] != l13_371_reg["sources"][index] or row["registration"]["sha256"] != l13_371_hash
            or row["registration_pointer"] != f"/sources/{index}"
            or row["observations"] != [{"ranges": [[lo, hi]], "assessment": text}
                                      for lo, hi, text in l13_371_obs[index]]):
        raise ValueError("Lens 13 root 371 packet source or observation mismatch")
l13_691_obs_match = re.search(r"^OBS (?=\[)", l13_691_reuse_text, re.M)
l13_691_obs = json.JSONDecoder().raw_decode(l13_691_reuse_text[l13_691_obs_match.end():])[0]
if [{"ranges": [[lo, hi]], "assessment": assessment} for lo, hi, assessment in l13_371_obs[5]] != l13_691_obs:
    raise ValueError("Lens 13 root 371 and 691 assessment mismatch")
if (l13_371_reg["sources"][5]["sha256"] != next(row["sha256"] for row in l13_691_objects if row["original"] == 2593)
        or l13_371_reg["sources"][5]["actual_eof"] != 143):
    raise ValueError("Lens 13 root 371 and 691 source identity mismatch")

l13_clock_dir = root / "recovered/l13-root-563-clock"
l13_clock_manifest = json.loads((l13_clock_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_clock_hash = "d3b5f0b0858c13fe4e39dc8449dee63af2b860ce16fc3a56542926e60a424fa1"
l13_clock_data = (l13_clock_dir / l13_clock_hash).read_bytes()
if (l13_clock_manifest["schema"] != "726-l13-root-563-clock-reuse-v1"
        or l13_clock_manifest["qualification_sha256"] != l13_clock_hash
        or hashlib.sha256(l13_clock_data).hexdigest() != l13_clock_hash):
    raise ValueError("Lens 13 root 563 clock qualification mismatch")
l13_clock = json.loads(l13_clock_data)
l13_clock_deps = {}
for dep in l13_clock_manifest["dependencies"]:
    data = (l13_clock_dir / dep["file"]).read_bytes()
    if (len(data) != dep["bytes"] or hashlib.sha256(data).hexdigest() != dep["sha256"]
            or l13_clock[dep["field"]]["sha256"] != dep["sha256"]):
        raise ValueError("Lens 13 root 563 clock dependency mismatch")
    l13_clock_deps[dep["field"]] = json.loads(data)
if (len(l13_clock_deps) != 4 or l13_clock_deps["prior_adoption"]["evidence"] != l13_clock_deps["prior_raw_receipt"]
        or l13_clock_deps["prior_raw_receipt"]["connected_source_reads"][0] != l13_clock["prior_source_read"]
        or l13_clock["prior_original"] != 2677 or l13_clock["original"] != 2056
        or l13_clock["new_body_credit"] != 0 or l13_clock["physical2677_ordered_credit_now"] != 0):
    raise ValueError("Lens 13 root 563 clock attribution mismatch")
l13_clock_source = (l13_clock_dir / "dependencies" / l13_clock_manifest["source_sha256"]).read_bytes()
if (hashlib.sha256(l13_clock_source).hexdigest() != l13_clock["source"]["sha256"]
        or len(l13_clock_source) != l13_clock_manifest["source_bytes"]
        or len(l13_clock_source.splitlines()) != 29):
    raise ValueError("Lens 13 root 563 clock source mismatch")

l13_fixture_dir = root / "recovered/l13-root-563-fixture"
l13_fixture = json.loads((l13_fixture_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_fixture_hash = "08572d9b77120a92cff6640e139e346c976e5685d008ffd4a82e43128f25f1d9"
l13_fixture_data = (l13_fixture_dir / l13_fixture_hash).read_bytes()
if (l13_fixture["schema"] != "726-l13-root-563-fixture-registration-v1"
        or l13_fixture["registration_sha256"] != l13_fixture_hash
        or hashlib.sha256(l13_fixture_data).hexdigest() != l13_fixture_hash):
    raise ValueError("Lens 13 root 563 fixture registration mismatch")
l13_fixture_reg = json.loads(l13_fixture_data)
if [row["original"] for row in l13_fixture_reg["sources"]] != [2054, 2055, 2057]:
    raise ValueError("Lens 13 root 563 fixture registration scope mismatch")
l13_fixture_sources = {row["sha256"]: row for row in l13_fixture_reg["sources"]}
l13_fixture_coverage = {digest: [] for digest in l13_fixture_sources}
for row in l13_fixture["sources"]:
    data = (l13_fixture_dir / "sources" / row["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != row["actual_eof"]
            or l13_fixture_sources[row["sha256"]]["original"] != row["original"]):
        raise ValueError("Lens 13 root 563 fixture source mismatch")
for fragment in l13_fixture["display_fragments"]:
    data = (l13_fixture_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 563 fixture display hash mismatch")
    digest = fragment["source_sha256"]
    lines = (l13_fixture_dir / "sources" / digest).read_text(encoding="utf-8").splitlines()
    displayed = data.decode("utf-8").splitlines()
    if len(displayed) != fragment["display_lines"]:
        raise ValueError("Lens 13 root 563 fixture display count mismatch")
    for line in displayed:
        number, separator, shown = line.partition(": ")
        if not separator or not number.isdigit() or shown != lines[int(number) - 1]:
            raise ValueError("Lens 13 root 563 fixture displayed line mismatch")
        l13_fixture_coverage[digest].append(int(number))
if (len(l13_fixture["display_fragments"]) != 9 or sum(map(len, l13_fixture_coverage.values())) != 871
        or any(lines != list(range(1, l13_fixture_sources[h]["actual_eof"] + 1))
               for h, lines in l13_fixture_coverage.items())):
    raise ValueError("Lens 13 root 563 fixture display coverage mismatch")
l13_fixture_obs_data = (l13_fixture_dir / "PACKET-OBSERVATIONS.json").read_bytes()
l13_fixture_literal = (l13_fixture_dir / "PACKET-OBSERVATIONS-LITERAL.txt").read_bytes()
if (hashlib.sha256(l13_fixture_obs_data).hexdigest() != l13_fixture["packet_observations_sha256"]
        or hashlib.sha256(l13_fixture_literal).hexdigest() != l13_fixture["packet_observation_literal_sha256"]):
    raise ValueError("Lens 13 root 563 packet observation evidence mismatch")
l13_fixture_obs = json.loads(l13_fixture_obs_data)
if (l13_fixture_obs["source_originals"] != [2057, 2058, 2059, 2060]
        or json.loads(json.dumps(ast.literal_eval(ast.parse(l13_fixture_literal.decode("utf-8")).body[0].value))) != l13_fixture_obs["observations"]
        or sum(map(len, l13_fixture_obs["observations"])) != 6):
    raise ValueError("Lens 13 root 563 packet observation literal mismatch")
l13_563_later = json.loads((l13_earlier_dir / "PRINTED-ROOT-OBSERVATIONS.json").read_text(encoding="utf-8"))["records"][0]
if (l13_563_later["original"] != 2824
        or [{"ranges": [ranges], "assessment": assessment} for ranges, assessment in l13_fixture_obs["observations"][2]] != l13_563_later["observations"]):
    raise ValueError("Lens 13 root 563 later assessment mismatch")

l13_692_dir = root / "recovered/l13-root-692-fresh"
l13_692 = json.loads((l13_692_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_692_helper_hash = "6c9625a8a5e51535eaaa48cfbf449cd8c5a47d5ba6301d05cc1b3c71eabd43c5"
l13_692_helper = (l13_692_dir / l13_692_helper_hash).read_bytes()
if (l13_692["schema"] != "726-l13-root-692-fresh-source-evidence-v1"
        or l13_692["helper_sha256"] != l13_692_helper_hash
        or hashlib.sha256(l13_692_helper).hexdigest() != l13_692_helper_hash
        or l13_692["plan_sha256"] != l13_691_packet_manifest["successor_plan_sha256"]
        or l13_692_plan["admission_2600_2607_material_qualification"]["sha256"] != l13_692["qualification_sha256"]):
    raise ValueError("Lens 13 root 692 helper or plan mismatch")
l13_692_ast = ast.parse(l13_692_helper.decode("utf-8"))
l13_692_specs = next(ast.literal_eval(node.value) for node in ast.walk(l13_692_ast)
                     if isinstance(node, ast.Assign) and len(node.targets) == 1
                     and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "specs")
l13_692_sources = {row["original"]: row for row in l13_692["source_records"]}
if len(l13_692_sources) != 8 or len(l13_692_specs) != 8:
    raise ValueError("Lens 13 root 692 source count mismatch")
for original, eof, size, ranges in l13_692_specs:
    row = l13_692_sources[original]
    data = (l13_692_dir / "sources" / row["sha256"]).read_bytes()
    if (row["actual_eof"] != eof or row["bytes"] != size or row["ranges"] != ranges
            or hashlib.sha256(data).hexdigest() != row["sha256"]
            or len(data) != size or len(data.splitlines()) != eof):
        raise ValueError("Lens 13 root 692 source object mismatch")
l13_692_display = (l13_692_dir / "FRESH-DISPLAY.txt").read_bytes()
l13_692_command = (l13_692_dir / "DISPLAY-COMMAND.txt").read_bytes()
if (hashlib.sha256(l13_692_display).hexdigest() != l13_692["display_sha256"]
        or hashlib.sha256(l13_692_command).hexdigest() != l13_692["display_command_sha256"]
        or l13_692["qualification_sha256"] not in l13_692_command.decode("utf-8")):
    raise ValueError("Lens 13 root 692 display provenance mismatch")
ast.parse(l13_692_command.decode("utf-8"))
l13_692_seen = {2600: [], 2601: []}
l13_692_current = None
for line in l13_692_display.decode("utf-8").splitlines():
    if line.startswith("FRESH "):
        head, raw_ranges = line.split(" SCOPE ", 1)
        _, original, identity = head.split(" ", 2)
        l13_692_current = int(original)
        row = l13_692_sources[l13_692_current]
        if row["source_id"] != identity or row["ranges"] != json.loads(raw_ranges):
            raise ValueError("Lens 13 root 692 displayed source identity mismatch")
        continue
    number, separator, shown = line.partition(" ")
    if l13_692_current not in l13_692_seen or not separator or not number.isdigit():
        raise ValueError("Lens 13 root 692 displayed line format mismatch")
    row = l13_692_sources[l13_692_current]
    lines = (l13_692_dir / "sources" / row["sha256"]).read_text(encoding="utf-8").splitlines()
    if shown != lines[int(number) - 1]:
        raise ValueError("Lens 13 root 692 displayed line mismatch")
    l13_692_seen[l13_692_current].append(int(number))
if (sum(map(len, l13_692_seen.values())) != 360
        or any(lines != [n for lo, hi in l13_692_sources[original]["ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_692_seen.items())):
    raise ValueError("Lens 13 root 692 displayed scope mismatch")
l13_692_literal = (l13_692_dir / "PACKET-WRITER-LITERALS.txt").read_bytes()
l13_692_literal_json = (l13_692_dir / "PACKET-WRITER-LITERALS.json").read_bytes()
if (hashlib.sha256(l13_692_literal).hexdigest() != l13_692["packet_writer_literals_sha256"]
        or hashlib.sha256(l13_692_literal_json).hexdigest() != l13_692["packet_writer_json_sha256"]):
    raise ValueError("Lens 13 root 692 packet writer literal hash mismatch")
l13_692_literal_doc = json.loads(l13_692_literal_json)
l13_692_assignments = {node.targets[0].id: ast.literal_eval(node.value)
                       for node in ast.parse(l13_692_literal.decode("utf-8")).body
                       if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name)}
if (l13_692_literal_doc["writer_filechange_ordinal"] != 50511
        or l13_692_literal_doc["writer_candidate_sha256"] != l13_692["packet_writer_candidate_sha256"]
        or {str(k): v for k, v in l13_692_assignments["facts"].items()} != l13_692_literal_doc["source_assessments"]
        or l13_692_assignments["remaining"] != l13_692_literal_doc["remaining_connected_obligations"]
        or sorted(l13_692_assignments["facts"]) != list(range(2600, 2608))
        or len(l13_692_assignments["remaining"]) != 6):
    raise ValueError("Lens 13 root 692 packet writer assessments mismatch")
l13_692_packet_data = (l13_692_dir / l13_692["packet_sha256"]).read_bytes()
if hashlib.sha256(l13_692_packet_data).hexdigest() != "edadeaf61d4466db932faaaffb223c970cfb1f174d67d54dcea0326f082b53f1":
    raise ValueError("Lens 13 root 692 packet hash mismatch")
l13_692_packet = json.loads(l13_692_packet_data)
l13_692_receipt_data = (l13_692_dir / l13_692["receipt_sha256"]).read_bytes()
if hashlib.sha256(l13_692_receipt_data).hexdigest() != l13_692["receipt_sha256"]:
    raise ValueError("Lens 13 root 692 adoption receipt hash mismatch")
l13_692_receipt = json.loads(l13_692_receipt_data)
l13_692_state_data = (l13_692_dir / l13_692["state_sha256"]).read_bytes()
l13_693_plan_data = (l13_692_dir / l13_692["successor_plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_692_state_data).hexdigest() != "0f12b5426e81661e668983e3d6b210deae5c26c2ac5e6d80dd2623de457899fe"
        or hashlib.sha256(l13_693_plan_data).hexdigest() != "2e31e5ec0ed0c25db8c10e290f19712411ea4ffab64939f1fb9ff857447fc7f5"):
    raise ValueError("Lens 13 root 692 state or successor plan hash mismatch")
l13_692_state, l13_693_plan = json.loads(l13_692_state_data), json.loads(l13_693_plan_data)
if (l13_692_receipt["evidence"] != l13_692_packet
        or l13_692_receipt["packet"]["sha256"] != l13_692["packet_sha256"]
        or l13_692_receipt["plan"]["sha256"] != l13_692["plan_sha256"]
        or l13_692_receipt["kind"] != "Root exact original material scope assessment adoption"
        or l13_692_receipt["qualified_ordered_reuses"]
        or l13_692_state["receipt_sha256"] != l13_692["receipt_sha256"]
        or l13_692_state["predecessor_sha256"] != l13_691_packet_manifest["state_sha256"]
        or l13_693_plan["previous_state"]["sha256"] != l13_692["state_sha256"]
        or l13_693_plan["root_sequence"] != 693):
    raise ValueError("Lens 13 root 692 adoption chain mismatch")
l13_692_reference_data = (l13_692_dir / l13_692["receipt_reference_file"]).read_bytes()
if hashlib.sha256(l13_692_reference_data).hexdigest() != l13_692["receipt_reference_sha256"]:
    raise ValueError("Lens 13 root 692 receipt reference hash mismatch")
l13_692_reference = json.loads(l13_692_reference_data)
if not any(row["receipt"] == {"path": "segovb/READ-L13-PARALLEL-692.json",
                               "sha256": l13_692["receipt_sha256"]}
           for row in l13_692_reference["hits"]):
    raise ValueError("Lens 13 root 692 printed receipt reference mismatch")
if (l13_692_plan["packet"]["sha256"] != l13_692["packet_sha256"]
        or len(l13_692_packet["ordered"]) != 8 or l13_692_packet["new_root_ordered_lines"] != 942
        or l13_692_packet["remaining_connected_obligations"] != l13_692_literal_doc["remaining_connected_obligations"]):
    raise ValueError("Lens 13 root 692 packet scope mismatch")
for row in l13_692_packet["ordered"]:
    original = row["source"]["original"]
    if (row["source"]["sha256"] != l13_692_sources[original]["sha256"]
            or row["registration"]["sha256"] != l13_692["registration_sha256"]
            or row["observations"] != [{"ranges": l13_692_sources[original]["ranges"],
                                        "assessment": l13_692_literal_doc["source_assessments"][str(original)]}]):
        raise ValueError("Lens 13 root 692 packet source or assessment mismatch")

l13_576_dir = root / "recovered/l13-root-576"
l13_576 = json.loads((l13_576_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_576_qhash = "637d8fe6e626abe7d0d137e88d0dc80e10b429ef9c233847c3fa5626420e3f23"
l13_576_rhash = "9c5e2ca8880e9d58bcebcd4b8d6c712420b7542636a91628cb28d4b0e8920f9f"
l13_576_qdata = (l13_576_dir / l13_576_qhash).read_bytes()
l13_576_rdata = (l13_576_dir / l13_576_rhash).read_bytes()
if (l13_576["schema"] != "726-l13-root-576-evidence-v1"
        or hashlib.sha256(l13_576_qdata).hexdigest() != l13_576_qhash
        or hashlib.sha256(l13_576_rdata).hexdigest() != l13_576_rhash):
    raise ValueError("Lens 13 root 576 qualification or registration mismatch")
l13_576_q, l13_576_reg = json.loads(l13_576_qdata), json.loads(l13_576_rdata)
if ([row["original"] for row in l13_576_q["records"]] != [2099, 2103, 2104]
        or [row["original"] for row in l13_576_reg["sources"]] != [2100, 2101, 2102]
        or l13_576_q["records"][-1]["historical_range_overrun_qualification"]["raw_ranges"] != [[1, 150]]
        or l13_576_q["records"][-1]["historical_range_overrun_qualification"]["effective_existing_ranges"] != [[1, 145]]):
    raise ValueError("Lens 13 root 576 scopes or overrun qualification mismatch")
l13_576_deps = {}
for dep in l13_576["prior_dependencies"]:
    data = (l13_576_dir / "dependencies" / dep["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != dep["sha256"] or len(data) != dep["bytes"]
            or l13_576_q[dep["field"]]["sha256"] != dep["sha256"]):
        raise ValueError("Lens 13 root 576 prior dependency mismatch")
    l13_576_deps[dep["field"]] = json.loads(data)
if l13_576_deps["prior_adoption"]["evidence"] != l13_576_deps["prior_raw_receipt"]:
    raise ValueError("Lens 13 root 576 prior receipt mismatch")
l13_576_objects = {row["sha256"]: row for row in l13_576["source_objects"]}
for row in l13_576["source_objects"]:
    data = (l13_576_dir / "sources" / row["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != row["actual_eof"]):
        raise ValueError("Lens 13 root 576 source mismatch")
for record in l13_576_q["records"]:
    index = int(record["prior_row_pointer"].rsplit("/", 1)[1])
    if (l13_576_deps["prior_raw_receipt"]["rows"][index] != record["prior_source_read"]
            or record["source"]["sha256"] not in l13_576_objects):
        raise ValueError("Lens 13 root 576 inherited source link mismatch")
l13_576_coverage = {row["sha256"]: [] for row in l13_576_reg["sources"]}
for fragment in l13_576["display_fragments"]:
    data = (l13_576_dir / fragment["file"]).read_bytes()
    digest = fragment["source_sha256"]
    if hashlib.sha256(data).hexdigest() != fragment["sha256"] or digest not in l13_576_coverage:
        raise ValueError("Lens 13 root 576 display hash mismatch")
    lines = (l13_576_dir / "sources" / digest).read_text(encoding="utf-8").splitlines()
    displayed = data.decode("utf-8").splitlines()
    if len(displayed) != fragment["display_lines"]:
        raise ValueError("Lens 13 root 576 display count mismatch")
    for line in displayed:
        number, separator, shown = line.partition(": ")
        if not separator or not number.isdigit() or shown != lines[int(number) - 1]:
            raise ValueError("Lens 13 root 576 displayed line mismatch")
        l13_576_coverage[digest].append(int(number))
if (len(l13_576["display_fragments"]) != 9 or sum(map(len, l13_576_coverage.values())) != 887
        or any(sorted(lines) != list(range(1, l13_576_objects[h]["actual_eof"] + 1))
               for h, lines in l13_576_coverage.items())):
    raise ValueError("Lens 13 root 576 display coverage mismatch")

l13_693_dir = root / "recovered/l13-root-693"
l13_693 = json.loads((l13_693_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_693_hashes = {
    "discovery_sha256": "080ffbd61692fd1ad3beab38f56c6a85f304a15eff8add62f7a739ce19015655",
    "registration_sha256": "e41778a288ed2407a42f3630c0ec07ad77ecdcaaac232f2dd4769421c528dba4",
    "qualification_sha256": "4c3dbaada71c67c85af1738578483b2a103a5f0afd4b8b2afde59a0a62b70d6c",
    "helper_sha256": "f378e46e000f0d285cecc5bf031a92dd596980d99796a88c10c3f5341db23075",
    "packet_sha256": "1aa2fabb5d167d6044da8c1e2e9b566d596d0dc78df1be7d43541bbec530eebf",
}
l13_693_files = {}
for key, digest in l13_693_hashes.items():
    data = (l13_693_dir / digest).read_bytes()
    if l13_693[key] != digest or hashlib.sha256(data).hexdigest() != digest:
        raise ValueError("Lens 13 root 693 original artifact hash mismatch")
    l13_693_files[key] = data
ast.parse(l13_693_files["helper_sha256"].decode("utf-8"))
l13_693_reg = json.loads(l13_693_files["registration_sha256"])
l13_693_qual = json.loads(l13_693_files["qualification_sha256"])
l13_693_packet = json.loads(l13_693_files["packet_sha256"])
l13_693_discovery = json.loads(l13_693_files["discovery_sha256"])
if (l13_693_qual["discovery_metadata_sha256"] != l13_693["discovery_sha256"]
        or [row["original"] for row in l13_693_discovery["entries"]] != list(range(2608, 2616))
        or l13_693_discovery["exact_hash_prior_hits"] or l13_693_discovery["future_same_lens_proofs_navigation"]
        or l13_693_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 693 discovery mismatch")
l13_693_receipt_data = (l13_693_dir / l13_693["receipt_sha256"]).read_bytes()
l13_693_state_data = (l13_693_dir / l13_693["state_sha256"]).read_bytes()
l13_694_plan_data = (l13_693_dir / l13_693["successor_plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_693_receipt_data).hexdigest() != "502870258c6bbf60f36c1b1151bada48e998a58e1eff32b96fe9ddf5cf05d821"
        or hashlib.sha256(l13_693_state_data).hexdigest() != "81db1d88b9a6056596f1d9a0523e53d8f69e97573e849af5cdd027d42cca41c7"
        or hashlib.sha256(l13_694_plan_data).hexdigest() != "2b28359e408396fe3fff5dc1040c3c84a63aef5dadbc383a20c017bbedf7e630"):
    raise ValueError("Lens 13 root 693 receipt, state or successor plan hash mismatch")
l13_693_receipt, l13_693_state, l13_694_plan = map(json.loads, (l13_693_receipt_data, l13_693_state_data, l13_694_plan_data))
if (l13_693_receipt["evidence"] != l13_693_packet
        or l13_693_receipt["plan"]["sha256"] != l13_693["plan_sha256"]
        or l13_693_state["receipt_sha256"] != l13_693["receipt_sha256"]
        or l13_693_state["predecessor_sha256"] != l13_692["state_sha256"]
        or l13_694_plan["previous_state"]["sha256"] != l13_693["state_sha256"]
        or not any(row["receipt"]["sha256"] == l13_693["receipt_sha256"] for row in l13_692_reference["hits"])):
    raise ValueError("Lens 13 root 693 adoption chain mismatch")
if (l13_693_plan["packet"]["sha256"] != l13_693["packet_sha256"]
        or l13_693_plan["admission_2608_2615_material_qualification"]["sha256"] != l13_693["qualification_sha256"]
        or l13_693_qual["registration"]["sha256"] != l13_693["registration_sha256"]
        or l13_693_qual["proof_builder"]["sha256"] != l13_693["helper_sha256"]
        or l13_693_qual["helmet391_exclusion_preserved"] is not True
        or l13_693_qual["material_lines"] != 1030
        or [row["source"] for row in l13_693_qual["entries"]] != l13_693_reg["sources"]
        or [row["source"] for row in l13_693_packet["ordered"]] != l13_693_reg["sources"]):
    raise ValueError("Lens 13 root 693 artifact chain mismatch")
l13_693_sources = {row["original"]: row for row in l13_693_reg["sources"]}
l13_693_seen = {original: [] for original in l13_693_sources}
for row in l13_693["source_objects"]:
    data = (l13_693_dir / "sources" / row["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != row["actual_eof"]
            or l13_693_sources[row["original"]]["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 693 source mismatch")
for fragment in l13_693["display_fragments"]:
    data = (l13_693_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 693 display hash mismatch")
    current = None
    l13_693_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, original, identity = line.split(" ", 2)
            current = int(original)
            if l13_693_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 693 display identity mismatch")
            continue
        number, separator, shown = line.partition(": ")
        if current not in l13_693_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 693 display format mismatch")
        lines = (l13_693_dir / "sources" / l13_693_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if shown != lines[int(number) - 1]:
            raise ValueError("Lens 13 root 693 displayed line mismatch")
        l13_693_seen[current].append(int(number))
        l13_693_count += 1
    if l13_693_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 693 display count mismatch")
if (len(l13_693["display_fragments"]) != 5 or sum(map(len, l13_693_seen.values())) != 1030
        or any(lines != [n for lo, hi in l13_693_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_693_seen.items())):
    raise ValueError("Lens 13 root 693 display coverage mismatch")

l13_694_dir = root / "recovered/l13-root-694"
l13_694 = json.loads((l13_694_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_694_hashes = {
    "discovery_sha256": "14d3ff96343fa5bdf68f96a8db8f7bd7f465528a88b2ec4b10762f8dbffd674c",
    "registration_sha256": "2929f5f1652ba4282cf8b0b467d61b54922c37153cdf9aa4b5437443a68a9d2e",
    "qualification_sha256": "5f2e4090f365f7dc7ce19d126d72e6a2ac9d404f026c43d91fc70631a011eb23",
    "helper_sha256": "833649b493baa3f57c5e48369f249273a1411480a9c788c85abf7d2867fb7dd1",
    "packet_sha256": "3abb945c823b69be0e7d694138827a9d18b55144786029469b4eaa49794f4595",
}
l13_694_files = {}
for key, digest in l13_694_hashes.items():
    data = (l13_694_dir / digest).read_bytes()
    if l13_694[key] != digest or hashlib.sha256(data).hexdigest() != digest:
        raise ValueError("Lens 13 root 694 original artifact hash mismatch")
    l13_694_files[key] = data
ast.parse(l13_694_files["helper_sha256"].decode("utf-8"))
l13_694_reg = json.loads(l13_694_files["registration_sha256"])
l13_694_qual = json.loads(l13_694_files["qualification_sha256"])
l13_694_packet = json.loads(l13_694_files["packet_sha256"])
l13_694_discovery = json.loads(l13_694_files["discovery_sha256"])
if (l13_694_qual["discovery_metadata_sha256"] != l13_694["discovery_sha256"]
        or [row["original"] for row in l13_694_discovery["entries"]] != list(range(2616, 2624))
        or l13_694_discovery["exact_hash_prior_hits"] or l13_694_discovery["future_same_lens_proofs_navigation"]
        or l13_694_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 694 discovery mismatch")
l13_694_receipt_data = (l13_694_dir / l13_694["receipt_sha256"]).read_bytes()
l13_694_state_data = (l13_694_dir / l13_694["state_sha256"]).read_bytes()
l13_695_plan_data = (l13_694_dir / l13_694["successor_plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_694_receipt_data).hexdigest() != "d450379633ff5d995fc8ef0e91d8828798233698faa8ca8fdedd02c1b81ac8ba"
        or hashlib.sha256(l13_694_state_data).hexdigest() != "f2f5a57384533b751d80e079be6ffe07212c2a4f1f57d7de96c86f09cade909a"
        or hashlib.sha256(l13_695_plan_data).hexdigest() != "9c6b26c379bc238dc95d83fdea30a8885a40b72089027f7c42f7a086697b03f4"):
    raise ValueError("Lens 13 root 694 receipt, state or successor plan hash mismatch")
l13_694_receipt, l13_694_state, l13_695_plan = map(json.loads, (l13_694_receipt_data, l13_694_state_data, l13_695_plan_data))
if (l13_694_receipt["evidence"] != l13_694_packet
        or l13_694_receipt["plan"]["sha256"] != l13_694["plan_sha256"]
        or l13_694_state["receipt_sha256"] != l13_694["receipt_sha256"]
        or l13_694_state["predecessor_sha256"] != l13_693["state_sha256"]
        or l13_695_plan["previous_state"]["sha256"] != l13_694["state_sha256"]
        or l13_695_plan["root_sequence"] != 695):
    raise ValueError("Lens 13 root 694 adoption chain mismatch")
if (l13_694_plan["packet"]["sha256"] != l13_694["packet_sha256"]
        or l13_694_plan["admission_2616_2623_material_qualification"]["sha256"] != l13_694["qualification_sha256"]
        or l13_694_qual["registration"]["sha256"] != l13_694["registration_sha256"]
        or l13_694_qual["proof_builder"]["sha256"] != l13_694["helper_sha256"]
        or l13_694_qual["sampling_tail_exclusion_preserved"] is not True
        or l13_694_qual["material_lines"] != 666
        or [row["source"] for row in l13_694_qual["entries"]] != l13_694_reg["sources"]
        or [row["source"] for row in l13_694_packet["ordered"]] != l13_694_reg["sources"]):
    raise ValueError("Lens 13 root 694 artifact chain mismatch")
l13_694_sources = {row["original"]: row for row in l13_694_reg["sources"]}
l13_694_seen = {original: [] for original in l13_694_sources}
for row in l13_694["source_objects"]:
    data = (l13_694_dir / "sources" / row["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != row["actual_eof"]
            or l13_694_sources[row["original"]]["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 694 source mismatch")
for fragment in l13_694["display_fragments"]:
    data = (l13_694_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 694 display hash mismatch")
    current = None
    l13_694_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, original, identity = line.split(" ", 2)
            current = int(original)
            if l13_694_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 694 display identity mismatch")
            continue
        number, separator, shown = line.partition(": ")
        if current not in l13_694_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 694 display format mismatch")
        lines = (l13_694_dir / "sources" / l13_694_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if shown != lines[int(number) - 1]:
            raise ValueError("Lens 13 root 694 displayed line mismatch")
        l13_694_seen[current].append(int(number))
        l13_694_count += 1
    if l13_694_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 694 display count mismatch")
if (len(l13_694["display_fragments"]) != 3 or sum(map(len, l13_694_seen.values())) != 666
        or any(lines != [n for lo, hi in l13_694_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_694_seen.items())):
    raise ValueError("Lens 13 root 694 display coverage mismatch")

l13_695_dir = root / "recovered/l13-root-695"
l13_695 = json.loads((l13_695_dir / "MANIFEST.json").read_text(encoding="utf-8"))
l13_695_helper = (l13_695_dir / l13_695["helper_sha256"]).read_bytes()
if (l13_695["schema"] != "726-l13-root-695-source-evidence-v1"
        or hashlib.sha256(l13_695_helper).hexdigest() != "b8c4904aff9712404334376d46b88098cca0111503d0840e55e1ab3d32738fe9"
        or l13_695["plan_sha256"] != l13_694["successor_plan_sha256"]
        or l13_695_plan["admission_2624_2631_material_qualification"]["sha256"] != l13_695["qualification_sha256"]):
    raise ValueError("Lens 13 root 695 helper or plan mismatch")
l13_695_ast = ast.parse(l13_695_helper.decode("utf-8"))
l13_695_specs = next(ast.literal_eval(node.value) for node in ast.walk(l13_695_ast)
                     if isinstance(node, ast.Assign) and len(node.targets) == 1
                     and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "specs")
l13_695_sources = {row["original"]: row for row in l13_695["source_records"]}
if len(l13_695_sources) != 8 or len(l13_695_specs) != 8:
    raise ValueError("Lens 13 root 695 source count mismatch")
for original, eof, size, ranges in l13_695_specs:
    row = l13_695_sources[original]
    data = (l13_695_dir / "sources" / row["sha256"]).read_bytes()
    if (row["actual_eof"] != eof or row["bytes"] != size or row["ranges"] != ranges
            or hashlib.sha256(data).hexdigest() != row["sha256"]
            or len(data) != size or len(data.splitlines()) != eof):
        raise ValueError("Lens 13 root 695 source object mismatch")


l13_695_reg_data = (l13_695_dir / l13_695["registration_sha256"]).read_bytes()
l13_695_qual_data = (l13_695_dir / l13_695["qualification_sha256"]).read_bytes()
if (hashlib.sha256(l13_695_reg_data).hexdigest() != "ec548065c5ea9f9316b680e67d4ee2410bb8808c37d9f98616b23041fb455b3b"
        or hashlib.sha256(l13_695_qual_data).hexdigest() != "a7d2c915476789eb39f8af33207955c5ad231b76a3575a6bdbbcd05a077a6590"):
    raise ValueError("Lens 13 root 695 registration or qualification hash mismatch")
l13_695_reg, l13_695_qual = map(json.loads, (l13_695_reg_data, l13_695_qual_data))
if (l13_695_qual["registration"]["sha256"] != l13_695["registration_sha256"]
        or l13_695_qual["proof_builder"]["sha256"] != l13_695["helper_sha256"]
        or l13_695_qual["material_lines"] != 1546
        or [row["source"] for row in l13_695_qual["entries"]] != l13_695_reg["sources"]):
    raise ValueError("Lens 13 root 695 qualification chain mismatch")
l13_695_discovery_data = (l13_695_dir / l13_695["discovery_sha256"]).read_bytes()
if hashlib.sha256(l13_695_discovery_data).hexdigest() != "e64cc33bba52b320059961139157025182bf5552db9b8d0577588f16eae28bb3":
    raise ValueError("Lens 13 root 695 discovery hash mismatch")
l13_695_discovery = json.loads(l13_695_discovery_data)
if (l13_695_qual["discovery_metadata_sha256"] != l13_695["discovery_sha256"]
        or [row["original"] for row in l13_695_discovery["entries"]] != list(range(2624, 2632))
        or l13_695_discovery["exact_hash_prior_hits"] or l13_695_discovery["exact_hash_leaves"]
        or l13_695_discovery["future_same_lens_proofs_navigation"]
        or l13_695_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 695 discovery chain mismatch")
l13_695_packet_data = (l13_695_dir / l13_695["packet_sha256"]).read_bytes()
if hashlib.sha256(l13_695_packet_data).hexdigest() != "38b85aff08d258394b09ff8a9ad886d08650dbb4ab88ce03623a1cfc62537952":
    raise ValueError("Lens 13 root 695 packet hash mismatch")
l13_695_packet = json.loads(l13_695_packet_data)
if (l13_695_plan["packet"]["sha256"] != l13_695["packet_sha256"]
        or [row["source"] for row in l13_695_packet["ordered"]] != l13_695_reg["sources"]
        or l13_695_packet["new_root_ordered_lines"] != 1546
        or l13_695_packet["scope_note_resolution"]["separate_2625_tail_unread"] != [[55, 62]]):
    raise ValueError("Lens 13 root 695 packet chain mismatch")
l13_695_receipt_data = (l13_695_dir / l13_695["receipt_sha256"]).read_bytes()
l13_695_state_data = (l13_695_dir / l13_695["state_sha256"]).read_bytes()
l13_696_plan_data = (l13_695_dir / l13_695["successor_plan_sha256"]).read_bytes()
if (hashlib.sha256(l13_695_receipt_data).hexdigest() != "d49e15a397288fd7f159a4bf48d28343a7f77aff91cee5efee35b9378ddb0585"
        or hashlib.sha256(l13_695_state_data).hexdigest() != "3583ea0fba88ddc4cf7ab63987f21bf98fe82b2ea9690a500a15db1c2c55035b"
        or hashlib.sha256(l13_696_plan_data).hexdigest() != "1d9ef75650bf82e6cebb2c357fe291dc93a16756fe4ea8bdb9fa7e8f5c99a91d"):
    raise ValueError("Lens 13 root 695 receipt, state or successor plan hash mismatch")
l13_695_receipt, l13_695_state, l13_696_plan = map(json.loads, (l13_695_receipt_data, l13_695_state_data, l13_696_plan_data))
if (l13_695_receipt["evidence"] != l13_695_packet
        or l13_695_receipt["plan"]["sha256"] != l13_695["plan_sha256"]
        or l13_695_state["receipt_sha256"] != l13_695["receipt_sha256"]
        or l13_695_state["predecessor_sha256"] != l13_694["state_sha256"]
        or l13_696_plan["previous_state"]["sha256"] != l13_695["state_sha256"]):
    raise ValueError("Lens 13 root 695 adoption chain mismatch")
l13_695_seen = {original: [] for original in l13_695_sources}
for fragment in l13_695["display_fragments"]:
    data = (l13_695_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 695 display hash mismatch")
    current = None
    count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if l13_695_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 695 display identity mismatch")
            continue
        number, separator, text = line.partition(": ")
        if current not in l13_695_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 695 display format mismatch")
        lines = (l13_695_dir / "sources" / l13_695_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if lines[int(number) - 1] != text:
            raise ValueError("Lens 13 root 695 displayed line mismatch")
        l13_695_seen[current].append(int(number))
        count += 1
    if count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 695 display count mismatch")
if (len(l13_695["display_fragments"]) != 7 or sum(map(len, l13_695_seen.values())) != 1546
        or any(lines != [n for lo, hi in l13_695_sources[original]["ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_695_seen.items())):
    raise ValueError("Lens 13 root 695 display coverage mismatch")


l13_696_dir = root / "recovered/l13-root-696"
l13_696 = json.loads((l13_696_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "9d1e09af97cb7ec98cea639883e72242c4f5781df4a2930b32daf355abb39627",
    "helper_sha256": "98104d51e6c2da2202c11d12ee9e18510f1311b94779afeae10953ceaa1f362a",
    "registration_sha256": "27cb259c9d714eb6ca5eaeb4a76103711f3c57cc24ffda9271bdd8ea23527be3",
    "qualification_sha256": "395a3a05f46afc9784125ec77dceae4ed292c7d09d7f0439af8b7c57e001153c",
}.items():
    if l13_696[key] != expected or hashlib.sha256((l13_696_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 696 artifact hash mismatch")
ast.parse((l13_696_dir / l13_696["helper_sha256"]).read_text(encoding="utf-8"))
l13_696_discovery = json.loads((l13_696_dir / l13_696["discovery_sha256"]).read_bytes())
l13_696_reg = json.loads((l13_696_dir / l13_696["registration_sha256"]).read_bytes())
l13_696_qual = json.loads((l13_696_dir / l13_696["qualification_sha256"]).read_bytes())
if (l13_696["plan_sha256"] != l13_695["successor_plan_sha256"]
        or l13_696_plan["admission_2632_2639_material_qualification"]["sha256"] != l13_696["qualification_sha256"]
        or l13_696_qual["registration"]["sha256"] != l13_696["registration_sha256"]
        or l13_696_qual["proof_builder"]["sha256"] != l13_696["helper_sha256"]
        or l13_696_qual["discovery_metadata_sha256"] != l13_696["discovery_sha256"]
        or l13_696_qual["material_lines"] != 1672
        or [row["source"] for row in l13_696_qual["entries"]] != l13_696_reg["sources"]
        or [row["original"] for row in l13_696_discovery["entries"]] != list(range(2632, 2640))
        or l13_696_discovery["exact_hash_prior_hits"] or l13_696_discovery["exact_hash_leaves"]
        or l13_696_discovery["future_same_lens_proofs_navigation"]
        or l13_696_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 696 qualification chain mismatch")
l13_696_packet_data = (l13_696_dir / l13_696["packet_sha256"]).read_bytes()
l13_696_receipt_data = (l13_696_dir / l13_696["receipt_sha256"]).read_bytes()
l13_696_state_data = (l13_696_dir / l13_696["state_sha256"]).read_bytes()
l13_697_plan_data = (l13_696_dir / l13_696["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_696_packet_data, "644c62e69fe8b9d9cb83d76220f09eb65ccacdc5b7d0350de9dad192b503f67e"),
                       (l13_696_receipt_data, "2c4834b4e785ba6cd61e34c2e73b203fb88daec8cb4c305c6ca8018c7db821c1"),
                       (l13_696_state_data, "fc2a7329c8bb29f8e9e7f054571f11c3751717cd05d77fe068012d817eab3cea"),
                       (l13_697_plan_data, "e2e15e8cc4395467d9be909710e26b311cd8e8d23a970000bc3dd8d00a82b68e")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 696 packet or adoption hash mismatch")
l13_696_packet, l13_696_receipt, l13_696_state, l13_697_plan = map(json.loads, (l13_696_packet_data, l13_696_receipt_data, l13_696_state_data, l13_697_plan_data))
if (l13_696_plan["packet"]["sha256"] != l13_696["packet_sha256"]
        or [row["source"] for row in l13_696_packet["ordered"]] != l13_696_reg["sources"]
        or l13_696_packet["new_root_ordered_lines"] != 1672
        or l13_696_receipt["evidence"] != l13_696_packet
        or l13_696_receipt["plan"]["sha256"] != l13_696["plan_sha256"]
        or l13_696_state["receipt_sha256"] != l13_696["receipt_sha256"]
        or l13_696_state["predecessor_sha256"] != l13_695["state_sha256"]
        or l13_697_plan["previous_state"]["sha256"] != l13_696["state_sha256"]):
    raise ValueError("Lens 13 root 696 adoption chain mismatch")
l13_696_sources = {row["original"]: row for row in l13_696_reg["sources"]}
l13_696_seen = {original: [] for original in l13_696_sources}
if len(l13_696_sources) != 8 or len(l13_696["source_objects"]) != 8:
    raise ValueError("Lens 13 root 696 source count mismatch")
for row in l13_696["source_objects"]:
    data = (l13_696_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_696_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != source["actual_eof"] or source["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 696 source mismatch")
for fragment in l13_696["display_fragments"]:
    data = (l13_696_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 696 display hash mismatch")
    current = None
    l13_696_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if l13_696_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 696 display identity mismatch")
            continue
        number, separator, text = line.partition(": ")
        if current not in l13_696_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 696 display format mismatch")
        lines = (l13_696_dir / "sources" / l13_696_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if lines[int(number) - 1] != text:
            raise ValueError("Lens 13 root 696 displayed line mismatch")
        l13_696_seen[current].append(int(number))
        l13_696_count += 1
    if l13_696_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 696 display count mismatch")
if (len(l13_696["display_fragments"]) != 8 or sum(map(len, l13_696_seen.values())) != 1672
        or any(lines != [n for lo, hi in l13_696_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_696_seen.items())):
    raise ValueError("Lens 13 root 696 display coverage mismatch")

l13_697_dir = root / "recovered/l13-root-697"
l13_697 = json.loads((l13_697_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "02885f1079b0a9cdc99a4b99fd9f6274d37086f6daffd62aeb935f024beb8c2b",
    "helper_sha256": "ceff7f07d1c9a11d5a1bf4e5c9e44958fa1433d5aa20f8e0abeb2aa83ff0ef1d",
    "registration_sha256": "3abefbfa67bd562399d51d2bcf760145cf0cfe79b3f05b5b202576167d79f1d7",
    "qualification_sha256": "fbcd2e9c66a124bce45f6afb744aaa65c21371cf1cc85364a16e4f9aa615a6a3",
}.items():
    if l13_697[key] != expected or hashlib.sha256((l13_697_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 697 artifact hash mismatch")
ast.parse((l13_697_dir / l13_697["helper_sha256"]).read_text(encoding="utf-8"))
l13_697_discovery = json.loads((l13_697_dir / l13_697["discovery_sha256"]).read_bytes())
l13_697_reg = json.loads((l13_697_dir / l13_697["registration_sha256"]).read_bytes())
l13_697_qual = json.loads((l13_697_dir / l13_697["qualification_sha256"]).read_bytes())
if (l13_697["plan_sha256"] != l13_696["successor_plan_sha256"]
        or l13_697_plan["admission_2640_2647_material_qualification"]["sha256"] != l13_697["qualification_sha256"]
        or l13_697_qual["registration"]["sha256"] != l13_697["registration_sha256"]
        or l13_697_qual["proof_builder"]["sha256"] != l13_697["helper_sha256"]
        or l13_697_qual["discovery_metadata_sha256"] != l13_697["discovery_sha256"]
        or l13_697_qual["material_lines"] != 834
        or [row["source"] for row in l13_697_qual["entries"]] != l13_697_reg["sources"]
        or [row["original"] for row in l13_697_discovery["entries"]] != list(range(2640, 2648))
        or l13_697_discovery["exact_hash_prior_hits"] or l13_697_discovery["exact_hash_leaves"]
        or l13_697_discovery["future_same_lens_proofs_navigation"]
        or l13_697_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 697 qualification chain mismatch")
l13_697_packet_data = (l13_697_dir / l13_697["packet_sha256"]).read_bytes()
l13_697_receipt_data = (l13_697_dir / l13_697["receipt_sha256"]).read_bytes()
l13_697_state_data = (l13_697_dir / l13_697["state_sha256"]).read_bytes()
l13_698_plan_data = (l13_697_dir / l13_697["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_697_packet_data, "e6d57a37eed98ed7d273aef52340e7fa2dc83b02af63b7ab89917ad587ff33e7"),
                       (l13_697_receipt_data, "8f5e390fab62313b4fc1704ad9d29abf773ed8175c89a706d2104f37f0b49b9a"),
                       (l13_697_state_data, "cd31b171a5e36111f917140ec0ffa643c70e23b78030ed7442148b55539ced71"),
                       (l13_698_plan_data, "8e0c5deae16da56f5d6cca09bc2342af4b2b11cf5b5a065d715b8525644da99f")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 697 packet or adoption hash mismatch")
l13_697_packet, l13_697_receipt, l13_697_state, l13_698_plan = map(json.loads, (l13_697_packet_data, l13_697_receipt_data, l13_697_state_data, l13_698_plan_data))
if (l13_697_plan["packet"]["sha256"] != l13_697["packet_sha256"]
        or [row["source"] for row in l13_697_packet["ordered"]] != l13_697_reg["sources"]
        or l13_697_packet["new_root_ordered_lines"] != 834
        or l13_697_receipt["evidence"] != l13_697_packet
        or l13_697_receipt["plan"]["sha256"] != l13_697["plan_sha256"]
        or l13_697_state["receipt_sha256"] != l13_697["receipt_sha256"]
        or l13_697_state["predecessor_sha256"] != l13_696["state_sha256"]
        or l13_698_plan["previous_state"]["sha256"] != l13_697["state_sha256"]):
    raise ValueError("Lens 13 root 697 adoption chain mismatch")
l13_697_sources = {row["original"]: row for row in l13_697_reg["sources"]}
l13_697_seen = {original: [] for original in l13_697_sources}
if len(l13_697_sources) != 8 or len(l13_697["source_objects"]) != 8:
    raise ValueError("Lens 13 root 697 source count mismatch")
for row in l13_697["source_objects"]:
    data = (l13_697_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_697_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != source["actual_eof"] or source["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 697 source mismatch")
for fragment in l13_697["display_fragments"]:
    data = (l13_697_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 697 display hash mismatch")
    current = None
    l13_697_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if l13_697_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 697 display identity mismatch")
            continue
        number, separator, text = line.partition(": ")
        if current not in l13_697_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 697 display format mismatch")
        lines = (l13_697_dir / "sources" / l13_697_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if lines[int(number) - 1] != text:
            raise ValueError("Lens 13 root 697 displayed line mismatch")
        l13_697_seen[current].append(int(number))
        l13_697_count += 1
    if l13_697_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 697 display count mismatch")
if (len(l13_697["display_fragments"]) != 3 or sum(map(len, l13_697_seen.values())) != 834
        or any(lines != [n for lo, hi in l13_697_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_697_seen.items())):
    raise ValueError("Lens 13 root 697 display coverage mismatch")


l13_698_dir = root / "recovered/l13-root-698"
l13_698 = json.loads((l13_698_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "b848db7758c7c24c630f6bdead96584a34adcafc1a9a535526836c57e9cd7fcb",
    "helper_sha256": "a624a748616e4ff8783567df91fcfe2f9e761d24370b3928d7684e1fb837895c",
    "registration_sha256": "10152bd844e2907981cffc0d0318650ef7a4cf4aad1618dfa1b3e4710215d094",
    "qualification_sha256": "fbd8605f9cf0f71fac34f1039f3070c8a3e4116250ec692dc2b5c4a843619299",
}.items():
    if l13_698[key] != expected or hashlib.sha256((l13_698_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 698 artifact hash mismatch")
ast.parse((l13_698_dir / l13_698["helper_sha256"]).read_text(encoding="utf-8"))
l13_698_discovery = json.loads((l13_698_dir / l13_698["discovery_sha256"]).read_bytes())
l13_698_reg = json.loads((l13_698_dir / l13_698["registration_sha256"]).read_bytes())
l13_698_qual = json.loads((l13_698_dir / l13_698["qualification_sha256"]).read_bytes())
if (l13_698["plan_sha256"] != l13_697["successor_plan_sha256"]
        or l13_698_plan["admission_2648_2655_material_qualification"]["sha256"] != l13_698["qualification_sha256"]
        or l13_698_qual["registration"]["sha256"] != l13_698["registration_sha256"]
        or l13_698_qual["proof_builder"]["sha256"] != l13_698["helper_sha256"]
        or l13_698_qual["discovery_metadata_sha256"] != l13_698["discovery_sha256"]
        or l13_698_qual["material_lines"] != 705
        or l13_698_qual["production_source_lines"] != 632
        or l13_698_qual["fixture_source_lines"] != 73
        or l13_698_qual["fixture_execution"] is not False
        or [row["source"] for row in l13_698_qual["entries"]] != l13_698_reg["sources"]
        or [row["original"] for row in l13_698_discovery["entries"]] != list(range(2648, 2656))
        or l13_698_discovery["exact_hash_prior_hits"] or l13_698_discovery["exact_hash_leaves"]
        or l13_698_discovery["future_same_lens_proofs_navigation"]
        or l13_698_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 698 qualification chain mismatch")
l13_698_packet_data = (l13_698_dir / l13_698["packet_sha256"]).read_bytes()
l13_698_receipt_data = (l13_698_dir / l13_698["receipt_sha256"]).read_bytes()
l13_698_state_data = (l13_698_dir / l13_698["state_sha256"]).read_bytes()
l13_699_plan_data = (l13_698_dir / l13_698["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_698_packet_data, "9da4090de379a78d713ca7874dd7394b4b30c76cbb2bf5bbb185e49b35ec09ca"),
                       (l13_698_receipt_data, "9d137b1083db3ce7febc54b133c771656594ae4326cdf0b9adf7a2fec459d536"),
                       (l13_698_state_data, "3ba00209482394c2ae66edc9d8930398cb5953a43967cf3899eb7a42ccf17a0c"),
                       (l13_699_plan_data, "e7dcad5d86f0a21e607d78ece0d49ad1cb4dd7c9e05d38bacdae122217a07dfd")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 698 packet or adoption hash mismatch")
l13_698_packet, l13_698_receipt, l13_698_state, l13_699_plan = map(json.loads, (l13_698_packet_data, l13_698_receipt_data, l13_698_state_data, l13_699_plan_data))
if (l13_698_plan["packet"]["sha256"] != l13_698["packet_sha256"]
        or [row["source"] for row in l13_698_packet["ordered"]] != l13_698_reg["sources"]
        or l13_698_packet["new_root_ordered_lines"] != 705
        or l13_698_packet["source_line_roles"] != {"production": 632, "declared_fixture_source": 73, "fixture_execution": False}
        or l13_698_packet["tests_executed"] is not False
        or l13_698_receipt["evidence"] != l13_698_packet
        or l13_698_receipt["plan"]["sha256"] != l13_698["plan_sha256"]
        or l13_698_state["receipt_sha256"] != l13_698["receipt_sha256"]
        or l13_698_state["predecessor_sha256"] != l13_697["state_sha256"]
        or l13_699_plan["previous_state"]["sha256"] != l13_698["state_sha256"]):
    raise ValueError("Lens 13 root 698 adoption chain mismatch")
l13_698_sources = {row["original"]: row for row in l13_698_reg["sources"]}
l13_698_seen = {original: [] for original in l13_698_sources}
if len(l13_698_sources) != 8 or len(l13_698["source_objects"]) != 8:
    raise ValueError("Lens 13 root 698 source count mismatch")
for row in l13_698["source_objects"]:
    data = (l13_698_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_698_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != source["actual_eof"] or source["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 698 source mismatch")
for fragment in l13_698["display_fragments"]:
    data = (l13_698_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 698 display hash mismatch")
    current = None
    l13_698_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if l13_698_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 698 display identity mismatch")
            continue
        number, separator, text = line.partition(": ")
        if current not in l13_698_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 698 display format mismatch")
        lines = (l13_698_dir / "sources" / l13_698_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if lines[int(number) - 1] != text:
            raise ValueError("Lens 13 root 698 displayed line mismatch")
        l13_698_seen[current].append(int(number))
        l13_698_count += 1
    if l13_698_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 698 display count mismatch")
if (len(l13_698["display_fragments"]) != 3 or sum(map(len, l13_698_seen.values())) != 705
        or any(lines != [n for lo, hi in l13_698_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_698_seen.items())):
    raise ValueError("Lens 13 root 698 display coverage mismatch")



l13_699_dir = root / "recovered/l13-root-699"
l13_699 = json.loads((l13_699_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "d497cac17e64ca4834567ae01dd39409fa935a5a83ba9cf259e3cb145ae9efb7",
    "helper_sha256": "94135cc7207f179f43b9da796944b398a7b63a51a59f11ce9ddb27343d7b9269",
    "registration_sha256": "da4b8d82e4dc605e1451d38950e5e543eae8a61b9db262c39d8b8c89dc3b24e7",
    "qualification_sha256": "8b1ed64d0a1ce2324397d22de72154550a0c3c507c1653f6041be0defdd1518b",
}.items():
    if l13_699[key] != expected or hashlib.sha256((l13_699_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 699 artifact hash mismatch")
ast.parse((l13_699_dir / l13_699["helper_sha256"]).read_text(encoding="utf-8"))
l13_699_discovery = json.loads((l13_699_dir / l13_699["discovery_sha256"]).read_bytes())
l13_699_reg = json.loads((l13_699_dir / l13_699["registration_sha256"]).read_bytes())
l13_699_qual = json.loads((l13_699_dir / l13_699["qualification_sha256"]).read_bytes())
if (l13_699["plan_sha256"] != l13_698["successor_plan_sha256"]
        or l13_699_plan["admission_2656_2663_material_qualification"]["sha256"] != l13_699["qualification_sha256"]
        or l13_699_qual["registration"]["sha256"] != l13_699["registration_sha256"]
        or l13_699_qual["proof_builder"]["sha256"] != l13_699["helper_sha256"]
        or l13_699_qual["discovery_metadata_sha256"] != l13_699["discovery_sha256"]
        or l13_699_qual["material_lines"] != 1007
        or l13_699_qual["production_source_lines"] != 840
        or l13_699_qual["fixture_source_lines"] != 167
        or l13_699_qual["fixture_execution"] is not False
        or [row["source"] for row in l13_699_qual["entries"]] != l13_699_reg["sources"]
        or [row["original"] for row in l13_699_discovery["entries"]] != list(range(2656, 2664))
        or l13_699_discovery["exact_hash_prior_hits"] or l13_699_discovery["exact_hash_leaves"]
        or l13_699_discovery["future_same_lens_proofs_navigation"]
        or l13_699_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 699 qualification chain mismatch")
l13_699_packet_data = (l13_699_dir / l13_699["packet_sha256"]).read_bytes()
l13_699_receipt_data = (l13_699_dir / l13_699["receipt_sha256"]).read_bytes()
l13_699_state_data = (l13_699_dir / l13_699["state_sha256"]).read_bytes()
l13_700_plan_data = (l13_699_dir / l13_699["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_699_packet_data, "5da6a2f15ed3591e0cd1d30dffca409eb84ca273382dadb32d67e3ced98b4a65"),
                       (l13_699_receipt_data, "941204c7472aee3129889c4cfb3fe4b8a4ed386e8e4fbe505cdff1896b8e1890"),
                       (l13_699_state_data, "190aa2f559bf8e04530cb3e85603cdc6417954da4a268b0d88f5fc6e4441f0bf"),
                       (l13_700_plan_data, "412234f45230bab1e65ee37861dba6bcc63b816142d65d73303c1772e9076bc0")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 699 packet or adoption hash mismatch")
l13_699_packet, l13_699_receipt, l13_699_state, l13_700_plan = map(json.loads, (l13_699_packet_data, l13_699_receipt_data, l13_699_state_data, l13_700_plan_data))
if (l13_699_plan["packet"]["sha256"] != l13_699["packet_sha256"]
        or [row["source"] for row in l13_699_packet["ordered"]] != l13_699_reg["sources"]
        or l13_699_packet["new_root_ordered_lines"] != 1007
        or l13_699_packet["source_line_roles"] != {"production": 840, "declared_fixture_source": 167, "fixture_execution": False}
        or l13_699_packet["tests_executed"] is not False
        or l13_699_receipt["evidence"] != l13_699_packet
        or l13_699_receipt["plan"]["sha256"] != l13_699["plan_sha256"]
        or l13_699_state["receipt_sha256"] != l13_699["receipt_sha256"]
        or l13_699_state["predecessor_sha256"] != l13_698["state_sha256"]
        or l13_700_plan["previous_state"]["sha256"] != l13_699["state_sha256"]):
    raise ValueError("Lens 13 root 699 adoption chain mismatch")
l13_699_sources = {row["original"]: row for row in l13_699_reg["sources"]}
l13_699_seen = {original: [] for original in l13_699_sources}
if len(l13_699_sources) != 8 or len(l13_699["source_objects"]) != 8:
    raise ValueError("Lens 13 root 699 source count mismatch")
for row in l13_699["source_objects"]:
    data = (l13_699_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_699_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != source["actual_eof"] or source["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 699 source mismatch")
for fragment in l13_699["display_fragments"]:
    data = (l13_699_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 699 display hash mismatch")
    current = None
    l13_699_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if l13_699_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 699 display identity mismatch")
            continue
        number, separator, text = line.partition(": ")
        if current not in l13_699_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 699 display format mismatch")
        lines = (l13_699_dir / "sources" / l13_699_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if lines[int(number) - 1] != text:
            raise ValueError("Lens 13 root 699 displayed line mismatch")
        l13_699_seen[current].append(int(number))
        l13_699_count += 1
    if l13_699_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 699 display count mismatch")
if (len(l13_699["display_fragments"]) != 4 or sum(map(len, l13_699_seen.values())) != 1007
        or any(sorted(lines) != [n for lo, hi in l13_699_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_699_seen.items())):
    raise ValueError("Lens 13 root 699 display coverage mismatch")




l13_700_dir = root / "recovered/l13-root-700"
l13_700 = json.loads((l13_700_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "44118f283f6e3527b20dc1d76a3425bbb5afb9e3c84feda35ed2e2226ddc747e",
    "helper_sha256": "9d7033ec220c647ff8bf46a170bdd2d71abe2a890dd2b0c6d094eb1e2165965c",
    "registration_sha256": "4d4a3bf7a1bfd5458034161edf98d1e7bd5410c76531eac0aa1a12424a4a542f",
    "qualification_sha256": "e7dfc48fecf10a55e3db8a35211667be48dd2cb3e35a09df7e6373d312924ed6",
}.items():
    if l13_700[key] != expected or hashlib.sha256((l13_700_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 700 artifact hash mismatch")
ast.parse((l13_700_dir / l13_700["helper_sha256"]).read_text(encoding="utf-8"))
l13_700_discovery = json.loads((l13_700_dir / l13_700["discovery_sha256"]).read_bytes())
l13_700_reg = json.loads((l13_700_dir / l13_700["registration_sha256"]).read_bytes())
l13_700_qual = json.loads((l13_700_dir / l13_700["qualification_sha256"]).read_bytes())
if (l13_700["plan_sha256"] != l13_699["successor_plan_sha256"]
        or l13_700_plan["admission_2664_2670_material_qualification"]["sha256"] != l13_700["qualification_sha256"]
        or l13_700_qual["registration"]["sha256"] != l13_700["registration_sha256"]
        or l13_700_qual["proof_builder"]["sha256"] != l13_700["helper_sha256"]
        or l13_700_qual["discovery_metadata_sha256"] != l13_700["narrow_discovery_sha256"]
        or l13_700_qual["material_lines"] != 5111
        or l13_700_qual["production_source_lines"] != 668
        or l13_700_qual["fixture_source_lines"] != 4443
        or l13_700_qual["fixture_execution"] is not False
        or [row["source"] for row in l13_700_qual["entries"]] != l13_700_reg["sources"]
        or [row["original"] for row in l13_700_discovery["entries"]] != list(range(2664, 2672))
        or l13_700_discovery["exact_hash_prior_hits"] != [{"original": 2671, "receipt": {"path": "segovb/READ-L13-PARALLEL-023.json", "sha256": "46ff7eccfcf1fda9ca02cdbc76f115db63246e2882f228b83101896ca9bee4cb"}}]
        or l13_700_discovery["future_same_lens_proofs_navigation"]
        or l13_700_discovery["prior_applications_preserved_navigation"]):
    raise ValueError("Lens 13 root 700 qualification chain mismatch")
l13_700_narrow_data = (l13_700_dir / l13_700["narrow_discovery_sha256"]).read_bytes()
if hashlib.sha256(l13_700_narrow_data).hexdigest() != "1b10c242e9c7eb1f9bcef6239451d7233dcf8f0fbfc87585aa4a9349af51b159":
    raise ValueError("Lens 13 root 700 narrowed discovery mismatch")
l13_700_expected_narrow = dict(l13_700_discovery)
l13_700_expected_narrow["entries"] = l13_700_expected_narrow["entries"][:7]
l13_700_expected_narrow["exact_hash_prior_hits"] = []
if json.loads(l13_700_narrow_data) != l13_700_expected_narrow:
    raise ValueError("Lens 13 root 700 discovery narrowing mismatch")
l13_700_packet_data = (l13_700_dir / l13_700["packet_sha256"]).read_bytes()
if hashlib.sha256(l13_700_packet_data).hexdigest() != "44c5236c523321c69b835d0641f0d8be3fb59662929903abb31e7c6053140212":
    raise ValueError("Lens 13 root 700 packet hash mismatch")
l13_700_packet = json.loads(l13_700_packet_data)
if (l13_700_plan["packet"]["sha256"] != l13_700["packet_sha256"]
        or [row["source"] for row in l13_700_packet["ordered"]] != l13_700_reg["sources"]
        or l13_700_packet["new_root_ordered_lines"] != 5111
        or l13_700_packet["source_line_roles"] != {"production": 668, "declared_fixture_source": 4443, "fixture_execution": False}
        or l13_700_packet["tests_executed"] is not False):
    raise ValueError("Lens 13 root 700 packet chain mismatch")
l13_700_receipt_data = (l13_700_dir / l13_700["receipt_sha256"]).read_bytes()
l13_700_state_data = (l13_700_dir / l13_700["state_sha256"]).read_bytes()
l13_701_plan_data = (l13_700_dir / l13_700["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_700_receipt_data, "67582059789e3c97815da4f0f33c443d62f48334f3426aefbe0717b5497641a3"),
                       (l13_700_state_data, "848baad7ce2c75d2b1a493d49a9470662768be8f32c8c740e34227639cd3790a"),
                       (l13_701_plan_data, "ab2da21208f9ae042198ea9b156d0b0eaec197ddab9da8e462bb75aee84e0221")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 700 adoption hash mismatch")
l13_700_receipt, l13_700_state, l13_701_plan = map(json.loads, (l13_700_receipt_data, l13_700_state_data, l13_701_plan_data))
if (l13_700_receipt["evidence"] != l13_700_packet
        or l13_700_receipt["plan"]["sha256"] != l13_700["plan_sha256"]
        or l13_700_state["receipt_sha256"] != l13_700["receipt_sha256"]
        or l13_700_state["predecessor_sha256"] != l13_699["state_sha256"]
        or l13_701_plan["previous_state"]["sha256"] != l13_700["state_sha256"]
        or l13_701_plan["prior_reuses"][0]["original"] != 2671
        or l13_701_plan["prior_reuses"][0]["adoption"]["sha256"] != "46ff7eccfcf1fda9ca02cdbc76f115db63246e2882f228b83101896ca9bee4cb"):
    raise ValueError("Lens 13 root 700 adoption chain mismatch")
l13_700_sources = {row["original"]: row for row in l13_700_reg["sources"]}
l13_700_seen = {original: [] for original in l13_700_sources}
if len(l13_700_sources) != 7 or len(l13_700["source_objects"]) != 7:
    raise ValueError("Lens 13 root 700 source count mismatch")
for row in l13_700["source_objects"]:
    data = (l13_700_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_700_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.splitlines()) != source["actual_eof"] or source["sha256"] != row["sha256"]):
        raise ValueError("Lens 13 root 700 source mismatch")
l13_700_source_lines = {n: (l13_700_dir / "sources" / row["sha256"]).read_text(encoding="utf-8").splitlines() for n, row in l13_700_sources.items()}
for fragment in l13_700["display_fragments"]:
    data = (l13_700_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 13 root 700 display hash mismatch")
    current = None
    l13_700_count = 0
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            _, number, identity = line.split(" ", 2)
            current = int(number)
            if l13_700_sources[current]["source_id"] != identity:
                raise ValueError("Lens 13 root 700 display identity mismatch")
            continue
        number, separator, text = line.partition(": ")
        if current not in l13_700_seen or not separator or not number.isdigit():
            raise ValueError("Lens 13 root 700 display format mismatch")
        lines = l13_700_source_lines[current]
        if lines[int(number) - 1] != text:
            raise ValueError("Lens 13 root 700 displayed line mismatch")
        l13_700_seen[current].append(int(number))
        l13_700_count += 1
    if l13_700_count != fragment["display_lines"]:
        raise ValueError("Lens 13 root 700 display count mismatch")
if (len(l13_700["display_fragments"]) != 6 or sum(map(len, l13_700_seen.values())) != 5111
        or any(sorted(lines) != [n for lo, hi in l13_700_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_700_seen.items())):
    raise ValueError("Lens 13 root 700 display coverage mismatch")





l13_701_dir = root / "recovered/l13-root-701"
l13_701 = json.loads((l13_701_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "2ac5858b5a0927ce334e13bc3cda69dba3c034d3a4cfedc4bdc7b9ce2f24dc9d",
    "narrow_discovery_sha256": "050a652e9dae1e041059fb8b2fff02f2f740060af59c50343c8a29cd8e5a6988",
    "helper_sha256": "9ce16ee52cd9da59290243a2932a51b2d0de2565d4e262ccb8b4278ca6ea0fec",
}.items():
    if l13_701[key] != expected or hashlib.sha256((l13_701_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 701 artifact hash mismatch")
ast.parse((l13_701_dir / l13_701["helper_sha256"]).read_text(encoding="utf-8"))
l13_701_discovery = json.loads((l13_701_dir / l13_701["discovery_sha256"]).read_bytes())
l13_701_narrow = json.loads((l13_701_dir / l13_701["narrow_discovery_sha256"]).read_bytes())
l13_701_expected = dict(l13_701_discovery)
l13_701_expected["entries"] = l13_701_expected["entries"][:4]
l13_701_expected["exact_hash_prior_hits"] = [row for row in l13_701_expected["exact_hash_prior_hits"] if row["original"] == 2671]
l13_701_expected["future_same_lens_proofs_navigation"] = []
l13_701_expected["prior_applications_preserved_navigation"] = []
if (l13_701["plan_sha256"] != l13_700["successor_plan_sha256"]
        or l13_701_narrow != l13_701_expected
        or len(l13_701_discovery["exact_hash_prior_hits"]) != 7
        or len(l13_701_discovery["future_same_lens_proofs_navigation"]) != 2
        or len(l13_701_discovery["prior_applications_preserved_navigation"]) != 1
        or l13_701_narrow["exact_hash_prior_hits"] != [{"original": 2671, "receipt": {"path": "segovb/READ-L13-PARALLEL-023.json", "sha256": "46ff7eccfcf1fda9ca02cdbc76f115db63246e2882f228b83101896ca9bee4cb"}}]):
    raise ValueError("Lens 13 root 701 discovery narrowing mismatch")

l13_701_reg_data = (l13_701_dir / l13_701["registration_sha256"]).read_bytes()
if hashlib.sha256(l13_701_reg_data).hexdigest() != "cb2b38bf24a8a832973e4462cb4b82e78d698de20318fcb4bf2e9d4766d4dfa9":
    raise ValueError("Lens 13 root 701 registration hash mismatch")
l13_701_reg = json.loads(l13_701_reg_data)
l13_701_sources = {row["original"]: row for row in l13_701_reg["sources"]}
if (list(l13_701_sources) != [2671, 2672, 2673, 2674]
        or l13_701_reg["fresh_lines_registered"] != 35 or l13_701_reg["prior_material_lines_reused"] != 5
        or l13_701_reg["exact_hash_prior_hits"] != l13_701_narrow["exact_hash_prior_hits"]):
    raise ValueError("Lens 13 root 701 mixed registration mismatch")
for row in l13_701["source_objects"]:
    data = (l13_701_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_701_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or source["sha256"] != row["sha256"] or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 701 source mismatch")
if len(l13_701["source_objects"]) != 4 or len(l13_701["prior_files"]) != 4:
    raise ValueError("Lens 13 root 701 evidence count mismatch")
for row in l13_701["prior_files"]:
    if hashlib.sha256((l13_701_dir / "prior" / row["sha256"]).read_bytes()).hexdigest() != row["sha256"]:
        raise ValueError("Lens 13 root 701 prior file mismatch")
l13_701_display = l13_701["fresh_display"]
l13_701_display_data = (l13_701_dir / l13_701_display["file"]).read_bytes()
if hashlib.sha256(l13_701_display_data).hexdigest() != l13_701_display["sha256"]:
    raise ValueError("Lens 13 root 701 display hash mismatch")
l13_701_seen = {n: [] for n in (2672, 2673, 2674)}
l13_701_current = None
for line in l13_701_display_data.decode("utf-8").splitlines():
    if line.startswith("SOURCE "):
        _, number, identity = line.split(" ", 2)
        l13_701_current = int(number)
        if l13_701_sources[l13_701_current]["source_id"] != identity:
            raise ValueError("Lens 13 root 701 display identity mismatch")
        continue
    number, separator, text = line.partition(": ")
    if l13_701_current not in l13_701_seen or not separator or not number.isdigit():
        raise ValueError("Lens 13 root 701 display format mismatch")
    lines = (l13_701_dir / "sources" / l13_701_sources[l13_701_current]["sha256"]).read_text(encoding="utf-8").splitlines()
    if lines[int(number) - 1] != text:
        raise ValueError("Lens 13 root 701 displayed line mismatch")
    l13_701_seen[l13_701_current].append(int(number))
if (sum(map(len, l13_701_seen.values())) != 35 or l13_701_display["display_lines"] != 35
        or any(lines != [n for lo, hi in l13_701_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_701_seen.items())):
    raise ValueError("Lens 13 root 701 fresh display coverage mismatch")

l13_701_qual_data = (l13_701_dir / l13_701["qualification_sha256"]).read_bytes()
l13_701_packet_data = (l13_701_dir / l13_701["packet_sha256"]).read_bytes()
if (hashlib.sha256(l13_701_qual_data).hexdigest() != "45689d000cfd935d3121188534243f935525480459f1a24699aa87c1c86fc4f2"
        or hashlib.sha256(l13_701_packet_data).hexdigest() != "8206c9aa0e7cc7de79dd4ac9d0e43512e96543d8422f5d9be2fce3a1c1a73d23"):
    raise ValueError("Lens 13 root 701 qualification or packet hash mismatch")
l13_701_qual, l13_701_packet = map(json.loads, (l13_701_qual_data, l13_701_packet_data))
if (l13_701_plan["packet"]["sha256"] != l13_701["packet_sha256"]
        or l13_701_plan["admission_2671_2674_material_qualification"]["sha256"] != l13_701["qualification_sha256"]
        or l13_701_qual["registration"]["sha256"] != l13_701["registration_sha256"]
        or l13_701_qual["proof_builder"]["sha256"] != l13_701["helper_sha256"]
        or l13_701_qual["discovery_metadata_sha256"] != l13_701["narrow_discovery_sha256"]
        or [row["source"] for row in l13_701_qual["entries"]] != l13_701_reg["sources"]
        or [row["source"] for row in l13_701_packet["ordered"]] != l13_701_reg["sources"]
        or l13_701_qual["fresh_lines"] != 35 or l13_701_qual["reused_material_lines"] != 5
        or l13_701_packet["source_line_roles"] != {"fresh_production": 35, "reused_production": 5, "declared_fixture_source": 0, "fixture_execution": False}
        or l13_701_packet["tests_executed"] is not False
        or l13_701_qual["entries"][0]["prior"]["prior_adoption"]["sha256"] != "46ff7eccfcf1fda9ca02cdbc76f115db63246e2882f228b83101896ca9bee4cb"
        or l13_701_qual["entries"][0]["prior"]["historical_raw_fields_absent"] != ["at", "registered_before_read_attestation"]):
    raise ValueError("Lens 13 root 701 mixed evidence mismatch")

l13_701_receipt_data = (l13_701_dir / l13_701["receipt_sha256"]).read_bytes()
l13_701_state_data = (l13_701_dir / l13_701["state_sha256"]).read_bytes()
l13_702_plan_data = (l13_701_dir / l13_701["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_701_receipt_data, "06dfa5b4524406999a513b57d2b935001caa671075cd5895e1851067c156482a"),
                       (l13_701_state_data, "1e014a2bd380ad39ab9e2b6e824d4ff4b9d441a383f1734ab2cdccbf4066e5fe"),
                       (l13_702_plan_data, "dd082ef2e661b81a8f63c8d6df1b4c8e786dcde246eda05b2164bd2cf187366f")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 701 adoption hash mismatch")
l13_701_receipt, l13_701_state, l13_702_plan = map(json.loads, (l13_701_receipt_data, l13_701_state_data, l13_702_plan_data))
if (l13_701_receipt["evidence"] != l13_701_packet
        or l13_701_receipt["plan"]["sha256"] != l13_701["plan_sha256"]
        or l13_701_receipt["prior_same_lens_reuses"] != l13_701_plan["prior_reuses"]
        or l13_701_receipt["new_original_roles_from_prior_connected_reads"] != 1
        or l13_701_state["receipt_sha256"] != l13_701["receipt_sha256"]
        or l13_701_state["predecessor_sha256"] != l13_700["state_sha256"]
        or l13_702_plan["previous_state"]["sha256"] != l13_701["state_sha256"]
        or [row["original"] for row in l13_702_plan["prior_reuses"]] != [2675, 2676]):
    raise ValueError("Lens 13 root 701 adoption chain mismatch")

l13_702_dir = root / "recovered/l13-root-702"
l13_702 = json.loads((l13_702_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "3669f5847901e36e9bfe094df0e186f6a47f7dc24dff9bd26fd6cafba596d62a",
    "helper_sha256": "d98db67266d4198cda27c7a51bd1feba396b4133e3ee09d421c7eff5ccdd3e29",
    "registration_sha256": "a6b4aad7431c25bac3b8cc110d07514f05bc7fd9e66bca8027155f3daf868c6f",
}.items():
    if l13_702[key] != expected or hashlib.sha256((l13_702_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 702 artifact hash mismatch")
ast.parse((l13_702_dir / l13_702["helper_sha256"]).read_text(encoding="utf-8"))
l13_702_discovery = json.loads((l13_702_dir / l13_702["discovery_sha256"]).read_bytes())
l13_702_reg = json.loads((l13_702_dir / l13_702["registration_sha256"]).read_bytes())
if (l13_702["plan_sha256"] != l13_701["successor_plan_sha256"]
        or [row["original"] for row in l13_702_discovery["entries"]] != [2675, 2676]
        or len(l13_702_discovery["exact_hash_prior_hits"]) != 2
        or l13_702_reg["exact_hash_prior_hits"] != l13_702_discovery["exact_hash_prior_hits"]
        or l13_702_reg["fresh_lines_registered"] != 0
        or l13_702_reg["prior_material_lines_reused"] != 365):
    raise ValueError("Lens 13 root 702 registration mismatch")
l13_702_sources = {row["original"]: row for row in l13_702_reg["sources"]}
if list(l13_702_sources) != [2675, 2676] or len(l13_702["source_objects"]) != 2:
    raise ValueError("Lens 13 root 702 source count mismatch")
for row in l13_702["source_objects"]:
    data = (l13_702_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_702_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or source["sha256"] != row["sha256"] or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 702 source mismatch")

l13_702_qual_data = (l13_702_dir / l13_702["qualification_sha256"]).read_bytes()
l13_702_packet_data = (l13_702_dir / l13_702["packet_sha256"]).read_bytes()
if (hashlib.sha256(l13_702_qual_data).hexdigest() != "23dcaf86fcbb5892727ec4dd44ba56280b57ee99f3201a342d67e4294d19dedf"
        or hashlib.sha256(l13_702_packet_data).hexdigest() != "623f806cb9f64ce9d5ae2a3dc05268978c325ead3543ccad7be6e2b06e8c42cc"):
    raise ValueError("Lens 13 root 702 qualification or packet hash mismatch")
l13_702_qual, l13_702_packet = map(json.loads, (l13_702_qual_data, l13_702_packet_data))
if (l13_702_plan["packet"]["sha256"] != l13_702["packet_sha256"]
        or l13_702_plan["admission_2675_2676_material_qualification"]["sha256"] != l13_702["qualification_sha256"]
        or l13_702_qual["registration"]["sha256"] != l13_702["registration_sha256"]
        or l13_702_qual["proof_builder"]["sha256"] != l13_702["helper_sha256"]
        or l13_702_qual["discovery_metadata_sha256"] != l13_702["discovery_sha256"]
        or [row["source"] for row in l13_702_qual["entries"]] != l13_702_reg["sources"]
        or [row["source"] for row in l13_702_packet["ordered"]] != l13_702_reg["sources"]
        or l13_702_qual["fresh_lines"] != 0 or l13_702_qual["reused_material_lines"] != 365
        or l13_702_packet["source_line_roles"] != {"fresh_production": 0, "reused_production": 365, "declared_fixture_source": 0, "fixture_execution": False}
        or l13_702_packet["tests_executed"] is not False or len(l13_702["prior_files"]) != 8):
    raise ValueError("Lens 13 root 702 mixed evidence mismatch")
for row in l13_702["prior_files"]:
    data = (l13_702_dir / "prior" / row["sha256"]).read_bytes()
    entry = next(e for e in l13_702_qual["entries"] if e["original"] == row["original"])
    if hashlib.sha256(data).hexdigest() != row["sha256"] or entry["prior"][row["role"]]["sha256"] != row["sha256"]:
        raise ValueError("Lens 13 root 702 prior supporting file mismatch")

l13_702_receipt_data = (l13_702_dir / l13_702["receipt_sha256"]).read_bytes()
l13_702_state_data = (l13_702_dir / l13_702["state_sha256"]).read_bytes()
l13_703_plan_data = (l13_702_dir / l13_702["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_702_receipt_data, "6c35ba2d5a47975c336cf394ef28c1de4c5ed8ed3f6642a27a29ded0ef4fbf3f"),
                       (l13_702_state_data, "e86b2e2ee611eebaa15968207b1dd321f8c659163d91c21852461938831d9cd2"),
                       (l13_703_plan_data, "b79f5d1acdd60e8a689161756743926442bc0ebefabe81e9ef2f56eec6c64835")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 702 adoption hash mismatch")
l13_702_receipt, l13_702_state, l13_703_plan = map(json.loads, (l13_702_receipt_data, l13_702_state_data, l13_703_plan_data))
if (l13_702_receipt["evidence"] != l13_702_packet
        or l13_702_receipt["plan"]["sha256"] != l13_702["plan_sha256"]
        or l13_702_receipt["prior_same_lens_reuses"] != l13_702_plan["prior_reuses"]
        or l13_702_receipt["new_original_roles_from_prior_connected_reads"] != 2
        or l13_702_state["receipt_sha256"] != l13_702["receipt_sha256"]
        or l13_702_state["predecessor_sha256"] != l13_701["state_sha256"]
        or l13_703_plan["previous_state"]["sha256"] != l13_702["state_sha256"]
        or [row["original"] for row in l13_703_plan["prior_reuses"]] != [2677, 2678]):
    raise ValueError("Lens 13 root 702 adoption chain mismatch")

l13_703_dir = root / "recovered/l13-root-703"
l13_703 = json.loads((l13_703_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "43f3144840d38fae90478dcf16e64ea30dade2381b6db14305b1f960ab406eb1",
    "narrow_discovery_sha256": "2fe6e64f1cbf1830824678a324bb88a39da38694a72aed2a8acefa868812eefe",
    "helper_sha256": "d1e367ba0c44dd53ef6a89010e3e9ee1d374584ad3573c385231937b0fff2ea4",
    "registration_sha256": "ddbae026663a703f4a9df2b908480d0ca5fc311ba2bd9f24237a91bebf9e4638",
}.items():
    if l13_703[key] != expected or hashlib.sha256((l13_703_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 703 artifact hash mismatch")
ast.parse((l13_703_dir / l13_703["helper_sha256"]).read_text(encoding="utf-8"))
l13_703_discovery = json.loads((l13_703_dir / l13_703["discovery_sha256"]).read_bytes())
l13_703_narrow = json.loads((l13_703_dir / l13_703["narrow_discovery_sha256"]).read_bytes())
l13_703_expected = dict(l13_703_discovery)
l13_703_expected["entries"] = l13_703_expected["entries"][:2]
l13_703_expected["exact_hash_prior_hits"] = [row for row in l13_703_expected["exact_hash_prior_hits"] if row["original"] in (2677, 2678)]
l13_703_expected["future_same_lens_proofs_navigation"] = [row for row in l13_703_expected["future_same_lens_proofs_navigation"] if row["original"] in (2677, 2678)]
l13_703_expected["prior_applications_preserved_navigation"] = [row for row in l13_703_expected["prior_applications_preserved_navigation"] if row["original"] == 2056]
l13_703_reg = json.loads((l13_703_dir / l13_703["registration_sha256"]).read_bytes())
if (l13_703["plan_sha256"] != l13_702["successor_plan_sha256"] or l13_703_narrow != l13_703_expected
        or len(l13_703_narrow["exact_hash_prior_hits"]) != 4
        or len(l13_703_narrow["future_same_lens_proofs_navigation"]) != 2
        or len(l13_703_narrow["prior_applications_preserved_navigation"]) != 1
        or l13_703_reg["exact_hash_prior_hits"] != l13_703_narrow["exact_hash_prior_hits"]
        or l13_703_reg["future_proofs_preserved"] != l13_703_narrow["future_same_lens_proofs_navigation"]
        or l13_703_reg["old_ordered_applications_preserved"] != l13_703_narrow["prior_applications_preserved_navigation"]
        or l13_703_reg["fresh_lines_registered"] != 0 or l13_703_reg["prior_material_lines_reused"] != 36):
    raise ValueError("Lens 13 root 703 discovery or registration mismatch")
l13_703_sources = {row["original"]: row for row in l13_703_reg["sources"]}
if list(l13_703_sources) != [2677, 2678] or len(l13_703["source_objects"]) != 2:
    raise ValueError("Lens 13 root 703 source count mismatch")
for row in l13_703["source_objects"]:
    data = (l13_703_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_703_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or source["sha256"] != row["sha256"] or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 703 source mismatch")

l13_703_qual_data = (l13_703_dir / l13_703["qualification_sha256"]).read_bytes()
l13_703_packet_data = (l13_703_dir / l13_703["packet_sha256"]).read_bytes()
if (hashlib.sha256(l13_703_qual_data).hexdigest() != "3a6b3f3df7fa4eb23e6da65f5560168d3f3ae945524fad43e2761d3b0572955b"
        or hashlib.sha256(l13_703_packet_data).hexdigest() != "73a5ca680f9fc12d4a5a4ddfad0fd39e082c54341a062cb70256b9578b706153"):
    raise ValueError("Lens 13 root 703 qualification or packet hash mismatch")
l13_703_qual, l13_703_packet = map(json.loads, (l13_703_qual_data, l13_703_packet_data))
if (l13_703_plan["packet"]["sha256"] != l13_703["packet_sha256"]
        or l13_703_plan["admission_2677_2678_material_qualification"]["sha256"] != l13_703["qualification_sha256"]
        or l13_703_qual["registration"]["sha256"] != l13_703["registration_sha256"]
        or l13_703_qual["proof_builder"]["sha256"] != l13_703["helper_sha256"]
        or l13_703_qual["discovery_metadata_sha256"] != l13_703["narrow_discovery_sha256"]
        or [row["source"] for row in l13_703_qual["entries"]] != l13_703_reg["sources"]
        or [row["source"] for row in l13_703_packet["ordered"]] != l13_703_reg["sources"]
        or l13_703_qual["fresh_lines"] != 0 or l13_703_qual["reused_material_lines"] != 36
        or l13_703_qual["old_ordered_applications_preserved"] != l13_703_narrow["prior_applications_preserved_navigation"]
        or l13_703_packet["source_line_roles"] != {"fresh_production": 0, "reused_production": 36, "declared_fixture_source": 0, "fixture_execution": False}
        or l13_703_packet["tests_executed"] is not False or len(l13_703["prior_files"]) != 7):
    raise ValueError("Lens 13 root 703 mixed evidence mismatch")
for row in l13_703["prior_files"]:
    data = (l13_703_dir / "prior" / row["sha256"]).read_bytes()
    entry = next(e for e in l13_703_qual["entries"] if e["original"] == row["original"])
    if hashlib.sha256(data).hexdigest() != row["sha256"] or entry["prior"][row["role"]]["sha256"] != row["sha256"]:
        raise ValueError("Lens 13 root 703 prior supporting file mismatch")

l13_703_receipt_data = (l13_703_dir / l13_703["receipt_sha256"]).read_bytes()
l13_703_state_data = (l13_703_dir / l13_703["state_sha256"]).read_bytes()
l13_704_plan_data = (l13_703_dir / l13_703["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_703_receipt_data, "b3bb5138a472a1bf4207c0b1eb81c242803355c1fd14a069bd996e439f96784a"),
                       (l13_703_state_data, "c17710e2ccfad9020c0c8e479f714e7341f11b5d9ad67f7293876766d55cedf4"),
                       (l13_704_plan_data, "3bf532bc33ca53ce000424a208ed24e1dababee573651529f5ce66ef9249f8c4")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 703 adoption hash mismatch")
l13_703_receipt, l13_703_state, l13_704_plan = map(json.loads, (l13_703_receipt_data, l13_703_state_data, l13_704_plan_data))
if (l13_703_receipt["evidence"] != l13_703_packet
        or l13_703_receipt["plan"]["sha256"] != l13_703["plan_sha256"]
        or l13_703_receipt["prior_same_lens_reuses"] != l13_703_plan["prior_reuses"]
        or l13_703_receipt["new_original_roles_from_prior_connected_reads"] != 2
        or l13_703_state["receipt_sha256"] != l13_703["receipt_sha256"]
        or l13_703_state["predecessor_sha256"] != l13_702["state_sha256"]
        or l13_704_plan["previous_state"]["sha256"] != l13_703["state_sha256"]
        or [row["original"] for row in l13_704_plan["prior_reuses"]] != [2680, 2682]):
    raise ValueError("Lens 13 root 703 adoption chain mismatch")

l13_704_dir = root / "recovered/l13-root-704"
l13_704 = json.loads((l13_704_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "07d73384f40c63c5844783911073f82585e88fcecd37cc7c46d5b980b1028825",
    "helper_sha256": "e08a0b0b5b7d2e4acd6bbd70db55bff8ca46871a5f0ab65353d414c1169a5046",
    "registration_sha256": "59d0c00a0442dcbea9fbaf097c8293979a2d156f1611bec1cf7f9ef687d96369",
}.items():
    if l13_704[key] != expected or hashlib.sha256((l13_704_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 704 artifact hash mismatch")
ast.parse((l13_704_dir / l13_704["helper_sha256"]).read_text(encoding="utf-8"))
l13_704_discovery = json.loads((l13_704_dir / l13_704["discovery_sha256"]).read_bytes())
l13_704_reg = json.loads((l13_704_dir / l13_704["registration_sha256"]).read_bytes())
if (l13_704["plan_sha256"] != l13_703["successor_plan_sha256"]
        or [row["original"] for row in l13_704_discovery["entries"]] != list(range(2679, 2685))
        or l13_704_reg["exact_hash_prior_hits"] != l13_704_discovery["exact_hash_prior_hits"]
        or l13_704_reg["future_proofs_preserved"] != l13_704_discovery["future_same_lens_proofs_navigation"]
        or l13_704_reg["old_ordered_applications_preserved"] != l13_704_discovery["prior_applications_preserved_navigation"]
        or l13_704_reg["fresh_lines_registered"] != 245 or l13_704_reg["prior_material_lines_reused"] != 150):
    raise ValueError("Lens 13 root 704 registration mismatch")
l13_704_sources = {row["original"]: row for row in l13_704_reg["sources"]}
if len(l13_704_sources) != 6 or len(l13_704["source_objects"]) != 6:
    raise ValueError("Lens 13 root 704 source count mismatch")
for row in l13_704["source_objects"]:
    data = (l13_704_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_704_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or source["sha256"] != row["sha256"] or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 704 source mismatch")
l13_704_display = l13_704["fresh_display"]
l13_704_display_data = (l13_704_dir / l13_704_display["file"]).read_bytes()
if hashlib.sha256(l13_704_display_data).hexdigest() != l13_704_display["sha256"]:
    raise ValueError("Lens 13 root 704 display hash mismatch")
l13_704_seen = {n: [] for n in (2679, 2681, 2683, 2684)}
l13_704_current = None
for line in l13_704_display_data.decode("utf-8").splitlines():
    if line.startswith("SOURCE "):
        _, number, identity = line.split(" ", 2)
        l13_704_current = int(number)
        if l13_704_sources[l13_704_current]["source_id"] != identity:
            raise ValueError("Lens 13 root 704 display identity mismatch")
        continue
    number, separator, text = line.partition(": ")
    if l13_704_current not in l13_704_seen or not separator or not number.isdigit():
        raise ValueError("Lens 13 root 704 display format mismatch")
    lines = (l13_704_dir / "sources" / l13_704_sources[l13_704_current]["sha256"]).read_text(encoding="utf-8").splitlines()
    if lines[int(number) - 1] != text:
        raise ValueError("Lens 13 root 704 displayed line mismatch")
    l13_704_seen[l13_704_current].append(int(number))
if (sum(map(len, l13_704_seen.values())) != 245 or l13_704_display["display_lines"] != 245
        or any(lines != [n for lo, hi in l13_704_sources[original]["prospective_ranges"] for n in range(lo, hi + 1)]
               for original, lines in l13_704_seen.items())):
    raise ValueError("Lens 13 root 704 fresh display coverage mismatch")

l13_704_qual_data = (l13_704_dir / l13_704["qualification_sha256"]).read_bytes()
l13_704_packet_data = (l13_704_dir / l13_704["packet_sha256"]).read_bytes()
if (hashlib.sha256(l13_704_qual_data).hexdigest() != "dbaa907ae864f94710b1f068d2050b7bfac87c8f88355c774ec92623773b68a2"
        or hashlib.sha256(l13_704_packet_data).hexdigest() != "431eea26b32aae6750396f515c2e15b67a4451a51f9c0658e3d84f6c89725512"):
    raise ValueError("Lens 13 root 704 qualification or packet hash mismatch")
l13_704_qual, l13_704_packet = map(json.loads, (l13_704_qual_data, l13_704_packet_data))
if (l13_704_plan["packet"]["sha256"] != l13_704["packet_sha256"]
        or l13_704_plan["admission_2679_2684_material_qualification"]["sha256"] != l13_704["qualification_sha256"]
        or l13_704_qual["registration"]["sha256"] != l13_704["registration_sha256"]
        or l13_704_qual["proof_builder"]["sha256"] != l13_704["helper_sha256"]
        or l13_704_qual["discovery_metadata_sha256"] != l13_704["discovery_sha256"]
        or [row["source"] for row in l13_704_qual["entries"]] != l13_704_reg["sources"]
        or [row["source"] for row in l13_704_packet["ordered"]] != l13_704_reg["sources"]
        or l13_704_qual["fresh_lines"] != 245 or l13_704_qual["reused_material_lines"] != 150
        or [row["mode"] for row in l13_704_qual["entries"]] != ["fresh", "delegated-reuse", "fresh", "root-reuse", "fresh", "fresh"]
        or l13_704_packet["source_line_roles"] != {"fresh_production": 245, "reused_production": 150, "declared_fixture_source": 0, "fixture_execution": False}
        or l13_704_packet["tests_executed"] is not False or len(l13_704["prior_files"]) != 4):
    raise ValueError("Lens 13 root 704 mixed evidence mismatch")
for row in l13_704["prior_files"]:
    data = (l13_704_dir / "prior" / row["sha256"]).read_bytes()
    entry = next(e for e in l13_704_qual["entries"] if e["original"] == row["original"])
    if hashlib.sha256(data).hexdigest() != row["sha256"] or entry["prior"][row["role"]]["sha256"] != row["sha256"]:
        raise ValueError("Lens 13 root 704 prior supporting file mismatch")

l13_704_receipt_data = (l13_704_dir / l13_704["receipt_sha256"]).read_bytes()
l13_704_state_data = (l13_704_dir / l13_704["state_sha256"]).read_bytes()
l13_705_plan_data = (l13_704_dir / l13_704["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_704_receipt_data, "527b48ecfd0a2a655dd00f2364898020ff9589fc323f561e02035b2956497173"),
                       (l13_704_state_data, "93e92686f8d543f77312535cb62d5b1e944d012f40d1c7192805ddacbbc0d7a3"),
                       (l13_705_plan_data, "dd172372bf954061fc3376c91dbbc3b8d97d7661236b287bff4d8ede3f1416a3")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 704 adoption hash mismatch")
l13_704_receipt, l13_704_state, l13_705_plan = map(json.loads, (l13_704_receipt_data, l13_704_state_data, l13_705_plan_data))
if (l13_704_receipt["evidence"] != l13_704_packet
        or l13_704_receipt["plan"]["sha256"] != l13_704["plan_sha256"]
        or l13_704_receipt["prior_same_lens_reuses"] != l13_704_plan["prior_reuses"]
        or l13_704_receipt["new_original_roles_from_prior_connected_reads"] != 2
        or l13_704_state["receipt_sha256"] != l13_704["receipt_sha256"]
        or l13_704_state["predecessor_sha256"] != l13_703["state_sha256"]
        or l13_705_plan["previous_state"]["sha256"] != l13_704["state_sha256"]
        or [row["original"] for row in l13_705_plan["prior_reuses"]] != [2685]):
    raise ValueError("Lens 13 root 704 adoption chain mismatch")

l13_705_dir = root / "recovered/l13-root-705"
l13_705 = json.loads((l13_705_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "a4782489f5b77220b7e5a5160826ca3c4e439548d282eff2013299df08e60054",
    "narrow_discovery_sha256": "8f76dce9ba21db1e6708681631d6817a4c8ed619ccefda3c49d71d109c5338d6",
    "helper_sha256": "baca457f19e798e07cf4c7d1b7aaaf36bba017349934c98d9d6c1554807b3bec",
    "registration_sha256": "d470a22a4fe2201a3e8c9c08d548a4f80343b1a977279175d9977b295ca9d5ba",
}.items():
    if l13_705[key] != expected or hashlib.sha256((l13_705_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 705 artifact hash mismatch")
ast.parse((l13_705_dir / l13_705["helper_sha256"]).read_text(encoding="utf-8"))
l13_705_discovery = json.loads((l13_705_dir / l13_705["discovery_sha256"]).read_bytes())
l13_705_narrow = json.loads((l13_705_dir / l13_705["narrow_discovery_sha256"]).read_bytes())
l13_705_expected = dict(l13_705_discovery)
l13_705_expected["entries"] = l13_705_expected["entries"][:2]
l13_705_expected["exact_hash_prior_hits"] = [row for row in l13_705_expected["exact_hash_prior_hits"] if row["original"] == 2685]
l13_705_expected["future_same_lens_proofs_navigation"] = []
l13_705_expected["prior_applications_preserved_navigation"] = [row for row in l13_705_expected["prior_applications_preserved_navigation"] if row["original"] == 2671]
l13_705_reg = json.loads((l13_705_dir / l13_705["registration_sha256"]).read_bytes())
if (l13_705["plan_sha256"] != l13_704["successor_plan_sha256"] or l13_705_narrow != l13_705_expected
        or len(l13_705_narrow["exact_hash_prior_hits"]) != 2
        or len(l13_705_narrow["prior_applications_preserved_navigation"]) != 1
        or l13_705_reg["exact_hash_prior_hits"] != l13_705_narrow["exact_hash_prior_hits"]
        or l13_705_reg["old_ordered_applications_preserved"] != l13_705_narrow["prior_applications_preserved_navigation"]
        or l13_705_reg["fresh_lines_registered"] != 113 or l13_705_reg["prior_material_lines_reused"] != 2):
    raise ValueError("Lens 13 root 705 discovery or registration mismatch")
l13_705_sources = {row["original"]: row for row in l13_705_reg["sources"]}
if list(l13_705_sources) != [2685, 2686] or len(l13_705["source_objects"]) != 2:
    raise ValueError("Lens 13 root 705 source count mismatch")
for row in l13_705["source_objects"]:
    data = (l13_705_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_705_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or source["sha256"] != row["sha256"] or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 705 source mismatch")
l13_705_display = l13_705["fresh_display"]
l13_705_data = (l13_705_dir / l13_705_display["file"]).read_bytes()
if hashlib.sha256(l13_705_data).hexdigest() != l13_705_display["sha256"]:
    raise ValueError("Lens 13 root 705 display hash mismatch")
l13_705_lines = (l13_705_dir / "sources" / l13_705_sources[2686]["sha256"]).read_text(encoding="utf-8").splitlines()
l13_705_seen = []
for line in l13_705_data.decode("utf-8").splitlines():
    if line.startswith("SOURCE "):
        if line != "SOURCE 2686 " + l13_705_sources[2686]["source_id"]:
            raise ValueError("Lens 13 root 705 display identity mismatch")
        continue
    number, separator, text = line.partition(": ")
    if not separator or not number.isdigit() or l13_705_lines[int(number) - 1] != text:
        raise ValueError("Lens 13 root 705 displayed line mismatch")
    l13_705_seen.append(int(number))
if (l13_705_display["display_lines"] != 113 or l13_705_seen !=
        [n for lo, hi in l13_705_sources[2686]["prospective_ranges"] for n in range(lo, hi + 1)]):
    raise ValueError("Lens 13 root 705 display coverage mismatch")

l13_705_qual_data = (l13_705_dir / l13_705["qualification_sha256"]).read_bytes()
l13_705_packet_data = (l13_705_dir / l13_705["packet_sha256"]).read_bytes()
if (hashlib.sha256(l13_705_qual_data).hexdigest() != "15a00586ff833d298c122faee4b27a3e52e4ac4ff4094432e8d0f8a2f220b125"
        or hashlib.sha256(l13_705_packet_data).hexdigest() != "bad2e0aef3f5f6eb46cfa4c4dd9bf8333f1a0b873c49dbff97d233ec7c09ff4c"):
    raise ValueError("Lens 13 root 705 qualification or packet hash mismatch")
l13_705_qual, l13_705_packet = map(json.loads, (l13_705_qual_data, l13_705_packet_data))
if (l13_705_plan["packet"]["sha256"] != l13_705["packet_sha256"]
        or l13_705_plan["admission_2685_2686_material_qualification"]["sha256"] != l13_705["qualification_sha256"]
        or l13_705_qual["registration"]["sha256"] != l13_705["registration_sha256"]
        or l13_705_qual["proof_builder"]["sha256"] != l13_705["helper_sha256"]
        or l13_705_qual["discovery_metadata_sha256"] != l13_705["narrow_discovery_sha256"]
        or [row["source"] for row in l13_705_qual["entries"]] != l13_705_reg["sources"]
        or [row["source"] for row in l13_705_packet["ordered"]] != l13_705_reg["sources"]
        or l13_705_qual["fresh_lines"] != 113 or l13_705_qual["reused_material_lines"] != 2
        or l13_705_qual["entries"][0]["prior"]["prior_alias_application_preserved"] != l13_705_narrow["prior_applications_preserved_navigation"][0]
        or l13_705_packet["source_line_roles"] != {"fresh_production": 113, "reused_production": 2, "declared_fixture_source": 0, "fixture_execution": False}
        or l13_705_packet["tests_executed"] is not False):
    raise ValueError("Lens 13 root 705 mixed evidence mismatch")

l13_705_receipt_data = (l13_705_dir / l13_705["receipt_sha256"]).read_bytes()
l13_705_state_data = (l13_705_dir / l13_705["state_sha256"]).read_bytes()
l13_706_plan_data = (l13_705_dir / l13_705["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_705_receipt_data, "05d519d6b089e3d596241d4be3243b91baf30d453921afbc544a35e14a651aa8"),
                       (l13_705_state_data, "31d5ef8dc20f32f1e1cb1230aadc085a800e45a93756b71b4bc27977f72c01b2"),
                       (l13_706_plan_data, "451d9950acc4761aa3812041cc6d7cdfc900cddfc36090be7bf59f32fe1efa8c")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 705 adoption hash mismatch")
l13_705_receipt, l13_705_state, l13_706_plan = map(json.loads, (l13_705_receipt_data, l13_705_state_data, l13_706_plan_data))
if (l13_705_receipt["evidence"] != l13_705_packet
        or l13_705_receipt["plan"]["sha256"] != l13_705["plan_sha256"]
        or l13_705_receipt["prior_same_lens_reuses"] != l13_705_plan["prior_reuses"]
        or l13_705_receipt["new_original_roles_from_prior_connected_reads"] != 1
        or l13_705_state["receipt_sha256"] != l13_705["receipt_sha256"]
        or l13_705_state["predecessor_sha256"] != l13_704["state_sha256"]
        or l13_706_plan["previous_state"]["sha256"] != l13_705["state_sha256"]
        or [row["original"] for row in l13_706_plan["prior_reuses"]] != [2687, 2688]):
    raise ValueError("Lens 13 root 705 adoption chain mismatch")

l13_706_dir = root / "recovered/l13-root-706"
l13_706 = json.loads((l13_706_dir / "MANIFEST.json").read_text(encoding="utf-8"))
for key, expected in {
    "discovery_sha256": "be78d4512e6ac4abc80f8518b9e331205600e4af337ccf38cbd4e29c3c739386",
    "helper_sha256": "9b104b5b784cbf34edea7cfcbafa25b588eabca758ee3338977c6f88e9ff38f4",
    "registration_sha256": "9578d2273075065a0a7ef800b0bdb8a73a3e95d8a21183e4eaa094c56e73ef3b",
}.items():
    if l13_706[key] != expected or hashlib.sha256((l13_706_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 706 artifact hash mismatch")
ast.parse((l13_706_dir / l13_706["helper_sha256"]).read_text(encoding="utf-8"))
l13_706_discovery = json.loads((l13_706_dir / l13_706["discovery_sha256"]).read_bytes())
l13_706_reg = json.loads((l13_706_dir / l13_706["registration_sha256"]).read_bytes())
if (l13_706["plan_sha256"] != l13_705["successor_plan_sha256"]
        or [row["original"] for row in l13_706_discovery["entries"]] != [2687, 2688]
        or len(l13_706_discovery["exact_hash_prior_hits"]) != 5
        or l13_706_reg["exact_hash_prior_hits"] != l13_706_discovery["exact_hash_prior_hits"]
        or l13_706_reg["future_proofs_preserved"] != l13_706_discovery["future_same_lens_proofs_navigation"]
        or l13_706_reg["old_ordered_applications_preserved"] != l13_706_discovery["prior_applications_preserved_navigation"]
        or len(l13_706_reg["old_ordered_applications_preserved"]) != 3
        or l13_706_reg["fresh_lines_registered"] != 0
        or l13_706_reg["prior_material_lines_reused"] != 168):
    raise ValueError("Lens 13 root 706 registration mismatch")
l13_706_sources = {row["original"]: row for row in l13_706_reg["sources"]}
if list(l13_706_sources) != [2687, 2688] or len(l13_706["source_objects"]) != 2:
    raise ValueError("Lens 13 root 706 source count mismatch")
for row in l13_706["source_objects"]:
    data = (l13_706_dir / "sources" / row["sha256"]).read_bytes()
    source = l13_706_sources[row["original"]]
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or source["sha256"] != row["sha256"] or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 706 source mismatch")


l13_706_qual_data = (l13_706_dir / l13_706["qualification_sha256"]).read_bytes()
l13_706_packet_data = (l13_706_dir / l13_706["packet_sha256"]).read_bytes()
if (hashlib.sha256(l13_706_qual_data).hexdigest() != "4d095758ea59ddda20c9007e4618197341c5ac64f9f27c2e0136567f31e8f242"
        or hashlib.sha256(l13_706_packet_data).hexdigest() != "d05142ad169327a4c3210f13b529e15822117a689493b35c7db26ae51be597d8"):
    raise ValueError("Lens 13 root 706 qualification or packet hash mismatch")
l13_706_qual, l13_706_packet = map(json.loads, (l13_706_qual_data, l13_706_packet_data))
if (l13_706_plan["packet"]["sha256"] != l13_706["packet_sha256"]
        or l13_706_plan["admission_2687_2688_material_qualification"]["sha256"] != l13_706["qualification_sha256"]
        or l13_706_qual["registration"]["sha256"] != l13_706["registration_sha256"]
        or l13_706_qual["proof_builder"]["sha256"] != l13_706["helper_sha256"]
        or l13_706_qual["discovery_metadata_sha256"] != l13_706["discovery_sha256"]
        or [row["source"] for row in l13_706_qual["entries"]] != l13_706_reg["sources"]
        or [row["source"] for row in l13_706_packet["ordered"]] != l13_706_reg["sources"]
        or l13_706_qual["fresh_lines"] != 0 or l13_706_qual["reused_material_lines"] != 168
        or l13_706_qual["old_ordered_applications_preserved"] != l13_706_discovery["prior_applications_preserved_navigation"]
        or l13_706_packet["source_line_roles"] != {"fresh_production": 0, "reused_production": 168, "declared_fixture_source": 0, "fixture_execution": False}
        or l13_706_packet["tests_executed"] is not False or len(l13_706["prior_files"]) != 4):
    raise ValueError("Lens 13 root 706 mixed evidence mismatch")
for row in l13_706["prior_files"]:
    data = (l13_706_dir / "prior" / row["sha256"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != row["sha256"] or any(e["prior"][row["role"]]["sha256"] != row["sha256"] for e in l13_706_qual["entries"]):
        raise ValueError("Lens 13 root 706 prior supporting file mismatch")

l13_706_receipt_data = (l13_706_dir / l13_706["receipt_sha256"]).read_bytes()
l13_706_state_data = (l13_706_dir / l13_706["state_sha256"]).read_bytes()
l13_707_plan_data = (l13_706_dir / l13_706["successor_plan_sha256"]).read_bytes()
for data, expected in ((l13_706_receipt_data, "53c3516e21f9bc23421e4e460238f3d8c357a4bc1e9fa692b7fc3e07f8a37a4a"),
                       (l13_706_state_data, "ae8eb1c73237018f4972ea8232d664019f930e740ef12e59f6f8eff7f2aa03cc"),
                       (l13_707_plan_data, "85249b95f96ea772fd8161ea9b7fbb726f2939e5df2ada1c4ac956d63425f9aa")):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("Lens 13 root 706 adoption hash mismatch")
l13_706_receipt, l13_706_state, l13_707_plan = map(json.loads, (l13_706_receipt_data, l13_706_state_data, l13_707_plan_data))
if (l13_706_receipt["evidence"] != l13_706_packet
        or l13_706_receipt["plan"]["sha256"] != l13_706["plan_sha256"]
        or l13_706_receipt["prior_same_lens_reuses"] != l13_706_plan["prior_reuses"]
        or l13_706_receipt["new_original_roles_from_prior_connected_reads"] != 2
        or l13_706_state["receipt_sha256"] != l13_706["receipt_sha256"]
        or l13_706_state["predecessor_sha256"] != l13_705["state_sha256"]
        or l13_707_plan["previous_state"]["sha256"] != l13_706["state_sha256"]
        or [row["original"] for row in l13_707_plan["prior_reuses"]] != [2689, 2690, 2692]):
    raise ValueError("Lens 13 root 706 adoption chain mismatch")

l13_display_dir = root / "recovered/l13-final-source-display-trace"
l13_display = json.loads((l13_display_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (l13_display["schema"] != "726-l13-final-source-display-trace-v1"
        or l13_display["registration_sha256"] != l13_reg_sha or l13_display["packet_sha256"] != l13_packet_sha
        or len(l13_display["fragments"]) != 2):
    raise ValueError("Invalid final Lens 13 source display trace")
l13_sources = {row["original"]: row for row in l13_reg["sources"]}
l13_displayed = {int(k): [] for k in l13_display["displayed_ranges"]}
for fragment in l13_display["fragments"]:
    ordinal = fragment["event_ordinal"]
    if ordinal not in (56096, 56097) or fragment["file"] != f"fragments/{ordinal}.txt":
        raise ValueError("Invalid final Lens 13 display fragment path")
    data = (l13_display_dir / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Final Lens 13 display fragment hash mismatch")
    current = None
    for line in data.decode("utf-8").splitlines():
        if line.startswith("SOURCE "):
            current = int(line[7:])
            if current not in l13_displayed:
                raise ValueError("Unknown final Lens 13 displayed source")
            continue
        number, separator, shown = line.partition(": ")
        if current is None or not separator or not number.isdigit():
            raise ValueError("Invalid final Lens 13 displayed line")
        n = int(number)
        source = (l13_trace_dir / "sources" / l13_sources[current]["sha256"]).read_text(encoding="utf-8").splitlines()
        if shown != source[n - 1]:
            raise ValueError("Final Lens 13 displayed source mismatch")
        l13_displayed[current].append(n)
for key, spans in l13_display["displayed_ranges"].items():
    expected_spans = [[216, 216]] if int(key) == 2846 else l13_sources[int(key)]["prospective_ranges"]
    if spans != expected_spans:
        raise ValueError("Final Lens 13 display registration scope mismatch")
    if l13_displayed[int(key)] != [n for lo, hi in spans for n in range(lo, hi + 1)]:
        raise ValueError("Final Lens 13 displayed range mismatch")
if sum(len(lines) for lines in l13_displayed.values()) != 409 or l13_display["line_occurrences"] != 409:
    raise ValueError("Invalid final Lens 13 display total")

transition_dir = root / "recovered/current-transition-path-evidence"
transition = json.loads((transition_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (transition["schema"] != "726-current-transition-path-evidence-v1" or len(transition["paths"]) != 59
        or transition["transition_original_bytes_recovered"] is not True
        or transition["before_revision"] != "e2755a732c84e67296b5e3441a4259a84f461b2f"
        or transition["after_revision"] != "33714256b6313ce86fdfbb534c54d92d024d7726"):
    raise ValueError("Invalid current transition path evidence")
printed_paths = None
with (root / "recovered/ledger-command-outputs/OUTPUTS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        entry = json.loads(line)
        if entry.get("transcript_output_ordinal") == 74817:
            printed_line = next(s for s in entry["output_text"].splitlines() if s.startswith("delta paths "))
            printed_paths = json.loads(printed_line[len("delta paths "):])
            break
if printed_paths != [row["path"] for row in transition["paths"]] or len(set(printed_paths)) != 59:
    raise ValueError("Current transition printed path mismatch")
transition_hashes = set()
for row in transition["paths"]:
    status = row["status"]
    if status not in ("A", "M", "D") or (row["before"] is None) != (status == "A") or (row["after"] is None) != (status == "D"):
        raise ValueError("Invalid current transition status")
    for source in (row["before"], row["after"]):
        if source is None:
            continue
        sha = source["sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Invalid current transition object path")
        data = (transition_dir / "objects" / sha).read_bytes()
        if hashlib.sha256(data).hexdigest() != sha or len(data) != source["bytes"]:
            raise ValueError("Current transition object hash mismatch")
        transition_hashes.add(sha)
if len(transition_hashes) != 96 or transition["distinct_object_count"] != 96:
    raise ValueError("Invalid current transition object count")
transition_sha = transition["transition_sha256"]
transition_bytes = (transition_dir / transition_sha).read_bytes()
if hashlib.sha256(transition_bytes).hexdigest() != transition_sha:
    raise ValueError("Recovered current transition hash mismatch")
original_transition = json.loads(transition_bytes)
if (original_transition["from_revision"] != transition["before_revision"]
        or original_transition["to_revision"] != transition["after_revision"]
        or original_transition["path_count"] != 59 or original_transition["per_lens_cell_count"] != 767
        or [row["path"] for row in original_transition["paths"]] != printed_paths):
    raise ValueError("Recovered current transition scope mismatch")
for actual, preserved in zip(original_transition["paths"], transition["paths"]):
    if actual["change"] != preserved["status"] or len(actual["per_lens"]) != 13:
        raise ValueError("Recovered current transition cell mismatch")
    for actual_key, preserved_key in (("before", "before"), ("delta_as_recorded", "after")):
        expected = preserved[preserved_key]
        if expected is None:
            if actual_key == "before" and actual[actual_key] is not None:
                raise ValueError("Recovered addition prior state mismatch")
            if actual_key == "delta_as_recorded" and actual[actual_key].get("absent") is not True:
                raise ValueError("Recovered deletion state mismatch")
        elif actual[actual_key]["sha256"] != expected["sha256"]:
            raise ValueError("Recovered current transition source mismatch")

cell_trace_dir = root / "recovered/current-cell-checkpoint-trace"
cell_trace = json.loads((cell_trace_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (cell_trace["schema"] != "726-current-cell-checkpoint-trace-v1" or cell_trace["transition_sha256"] != transition_sha
        or len(cell_trace["reports"]) != 8 or len(cell_trace["checkpoint_509_reported_cells"]) != 25):
    raise ValueError("Invalid current cell checkpoint trace")
for entry, ordinal in zip(cell_trace["reports"], [74938, 75116, 75297, 75408, 75609, 75771, 76123, 76346]):
    if entry["event_ordinal"] != ordinal or entry["expression"] != f"{ordinal}-expression.txt":
        raise ValueError("Invalid current cell report identity")
    data = (cell_trace_dir / entry["expression"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != entry["expression_sha256"]:
        raise ValueError("Current cell report expression hash mismatch")
    expression = ast.parse(data.decode("utf-8"), mode="eval").body
    fields = {}
    for key, value in zip(expression.keys, expression.values):
        name = ast.literal_eval(key)
        if name not in ("schema", "checkpoint", "completed", "counts", "closed_this_checkpoint", "next", "limits"):
            continue
        try:
            fields[name] = ast.literal_eval(value)
        except (ValueError, TypeError):
            fields[name] = {"unresolved_expression": ast.unparse(value)}
    if fields != entry["fields"]:
        raise ValueError("Current cell report literal mismatch")
reported = cell_trace["reports"][-1]["fields"]["closed_this_checkpoint"]
expected_cells = []
for key, indices in reported.items():
    lens = int("".join(c for c in key if c.isdigit()))
    for index in indices:
        expected_cells.append({"lens": lens, "path_index": index, "path": transition["paths"][index]["path"]})
if expected_cells != cell_trace["checkpoint_509_reported_cells"]:
    raise ValueError("Current cell path mapping mismatch")
for entry, ordinal, checkpoint, mode, expected_count in zip(cell_trace["reported_apply_verify_results"], [75097, 75109, 76296, 76335], [503, 503, 509, 509], ["--apply", "--verify", "--apply", "--verify"], [12, 12, 25, 25]):
    result = entry["result"]
    if entry["event_ordinal"] != ordinal or result.get("status") != "pass" or result.get("mode") != mode or result.get("checkpoint") != checkpoint or result.get("cells_closed") != expected_count:
        raise ValueError("Invalid current cell reported verification")

late_cell = json.loads(_extra_historical_manifest("recovered/late-current-cell-count-evidence/MANIFEST.json"))
if late_cell["schema"] != "726-late-current-cell-count-evidence-v1" or late_cell["writer"] != "recovered/final-writer-static-trace/CANDIDATE-3821.py.txt" or late_cell["output"] != "recovered/final-writer-static-trace/OUTPUT-419499.txt":
    raise ValueError("Invalid late cell count evidence")
writer_bytes = (root / late_cell["writer"]).read_bytes()
output_bytes = (root / late_cell["output"]).read_bytes()
if hashlib.sha256(writer_bytes).hexdigest() != late_cell["writer_sha256"]:
    raise ValueError("Late cell count evidence hash mismatch")
_extra_public_object(late_cell["output"], late_cell["output_sha256"], output_bytes)
writer_source = writer_bytes.decode("utf-8")
writer_tree = ast.parse(writer_source)
cell_assertions = sorted(ast.get_source_segment(writer_source, node) for node in ast.walk(writer_tree) if isinstance(node, ast.Assert) and "current_main_deltas" in ast.unparse(node.test) and "delta_closed" in ast.unparse(node.test))
if cell_assertions != late_cell["assertions"] or len(cell_assertions) != 2 or not any("==767" in x for x in cell_assertions) or not any("==403" in x for x in cell_assertions):
    raise ValueError("Late cell count assertions mismatch")
reported_modes = [json.loads(line) for line in output_bytes.decode("utf-8").splitlines() if line.startswith("{")]
if (reported_modes != late_cell["reported_modes"] or [x.get("mode") for x in reported_modes] != ["--preflight", "--apply", "--verify"]
        or any(x.get("status") != "pass" for x in reported_modes)):
    raise ValueError("Late cell count reported modes mismatch")

ff9_dir = root / "recovered/current-transition-ff9-evidence"
ff9 = json.loads((ff9_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (ff9["schema"] != "726-current-transition-ff9-evidence-v1" or len(ff9["paths"]) != 31
        or ff9["original_bytes_recovered"] is not True or ff9["reported_cells"] != 403
        or ff9["before_revision"] != transition["after_revision"]
        or ff9["after_revision"] != "ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04"):
    raise ValueError("Invalid later current transition evidence")
diff_bytes = (ff9_dir / "RECORDED-DIFF.txt").read_bytes()
if hashlib.sha256(diff_bytes).hexdigest() != ff9["recorded_diff_sha256"]:
    raise ValueError("Later current transition diff hash mismatch")
if diff_bytes.decode("utf-8").splitlines() != [row["status"] + "\t" + row["path"] for row in ff9["paths"]]:
    raise ValueError("Later current transition path mismatch")
if sorted(set(row["path"] for row in ff9["paths"]) & set(printed_paths)) != ff9["overlap"] or len(ff9["overlap"]) != 4:
    raise ValueError("Later current transition overlap mismatch")
ff9_hashes = set()
for row in ff9["paths"]:
    if row["status"] not in ("A", "M", "D") or (row["before"] is None) != (row["status"] == "A") or (row["after"] is None) != (row["status"] == "D"):
        raise ValueError("Invalid later current transition status")
    for source in (row["before"], row["after"]):
        if source is None:
            continue
        sha = source["sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Invalid later transition object path")
        data = (ff9_dir / "objects" / sha).read_bytes()
        if hashlib.sha256(data).hexdigest() != sha or len(data) != source["bytes"]:
            raise ValueError("Later transition source hash mismatch")
        ff9_hashes.add(sha)
if len(ff9_hashes) != ff9["source_object_count"] or len(ff9_hashes) != 53:
    raise ValueError("Invalid later transition object count")
candidate = ff9["candidate_template"]
candidate_bytes = (ff9_dir / "CANDIDATE-TEMPLATE.txt").read_bytes()
if candidate["file"] != "CANDIDATE-TEMPLATE.txt" or hashlib.sha256(candidate_bytes).hexdigest() != candidate["sha256"] or candidate["matched_timestamp"] is None:
    raise ValueError("Invalid later transition candidate")
matched_timestamp = candidate["matched_timestamp"]
matched_bytes = candidate_bytes.replace(b"TIMESTAMP", matched_timestamp.encode("utf-8"), 1)
original_ff9 = (ff9_dir / ff9["transition_reference"]["sha256"]).read_bytes()
if matched_bytes != original_ff9 or hashlib.sha256(original_ff9).hexdigest() != ff9["transition_reference"]["sha256"]:
    raise ValueError("Later transition original hash mismatch")
candidate_json = json.loads(candidate_bytes)
if (candidate_json["at"] != "TIMESTAMP" or candidate_json["path_count"] != 31
        or candidate_json["per_lens_cell_count"] != 403
        or candidate_json["from_revision"] != ff9["before_revision"] or candidate_json["to_revision"] != ff9["after_revision"]
        or [x["path"] for x in candidate_json["paths"]] != [x["path"] for x in ff9["paths"]]):
    raise ValueError("Later transition candidate scope mismatch")
candidate_row_checks = {entry["transcript_ordinal"]: entry["path_index"] for entry in candidate["independent_printed_row_checks"]}
if candidate_row_checks != {83351: 0, 86940: 22, 87014: 18}:
    raise ValueError("Invalid candidate printed row references")
seen_candidate_rows = set()
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        recorded = json.loads(line)
        ordinal = recorded.get("transcript_ordinal")
        if ordinal not in candidate_row_checks:
            continue
        row = json.loads(recorded["output_text"])
        expected_row = {k: v for k, v in candidate_json["paths"][candidate_row_checks[ordinal]].items() if k != "per_lens"}
        if row != expected_row:
            raise ValueError("Candidate printed row mismatch")
        seen_candidate_rows.add(ordinal)
if seen_candidate_rows != set(candidate_row_checks):
    raise ValueError("Missing candidate printed row")
for expected, actual in zip(ff9["paths"], candidate_json["paths"]):
    if expected["status"] != actual["change"] or len(actual["per_lens"]) != 13:
        raise ValueError("Later transition candidate cell mismatch")
    for key, field in (("before", "before"), ("after", "delta_as_recorded")):
        if expected[key] is not None and expected[key]["sha256"] != actual[field]["sha256"]:
            raise ValueError("Later transition candidate source mismatch")


prior_transition_dir = root / "recovered/prior-current-transition-evidence"
prior_transition = json.loads(_qualified_historical_manifest("recovered/prior-current-transition-evidence/MANIFEST.json"))
prior_sha = prior_transition["transition_sha256"]
if (prior_transition["schema"] != "726-prior-current-transition-evidence-v1"
        or prior_sha != "7ab4700aa59883fb73e3c7ef55ea2c05ed5e13661ee8a5b82dae1732d76188a5"
        or prior_transition["to_revision"] != transition["before_revision"]):
    raise ValueError("Invalid prior current transition evidence")
prior_data = (prior_transition_dir / prior_sha).read_bytes()
_qualified_public_object("recovered/prior-current-transition-evidence/" + prior_sha, prior_sha, prior_data)
prior_doc = json.loads(prior_data)
if (prior_doc["prior_target"] != prior_transition["from_revision"] or prior_doc["new_target"] != prior_transition["to_revision"]
        or len(prior_doc["rows"]) != 154 or prior_transition["path_count"] != 154):
    raise ValueError("Prior current transition scope mismatch")
prior_objects = set()
for row in prior_doc["rows"]:
    sha = row["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Invalid prior transition object path")
    data = (prior_transition_dir / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or len(data) != row["bytes"] or (row["text_EOF"] is not None and len(data.splitlines()) != row["text_EOF"]):
        raise ValueError("Prior transition source mismatch")
    prior_objects.add(sha)
if len(prior_objects) != 148 or prior_transition["distinct_object_count"] != 148 or sum(row["text_EOF"] is None for row in prior_doc["rows"]) != prior_transition["binary_scope_rows"]:
    raise ValueError("Prior transition source counts mismatch")
prior_diff = (prior_transition_dir / "RECORDED-GIT-DIFF.txt").read_bytes()
if hashlib.sha256(prior_diff).hexdigest() != prior_transition["git_diff_sha256"]:
    raise ValueError("Prior transition Git diff hash mismatch")
prior_rows = prior_transition["before_after_rows"]
if prior_diff.decode("utf-8").splitlines() != [row["status"] + "\t" + row["path"] for row in prior_rows] or len(prior_rows) != 154:
    raise ValueError("Prior transition Git diff mismatch")
prior_before = set()
for item, source in zip(prior_rows, prior_doc["rows"]):
    if item["path"] != source["path"] or item["after_sha256"] != source["sha256"] or item["status"] not in ("A", "M") or (item["before"] is None) != (item["status"] == "A"):
        raise ValueError("Prior transition comparison mismatch")
    if item["before"] is not None:
        sha = item["before"]["sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Invalid prior transition before object path")
        data = (prior_transition_dir / "objects" / sha).read_bytes()
        if hashlib.sha256(data).hexdigest() != sha or len(data) != item["before"]["bytes"]:
            raise ValueError("Prior transition before object mismatch")
        prior_before.add(sha)
if len(prior_before) != 45 or prior_transition["before_distinct_object_count"] != 45 or prior_transition["modified_path_count"] != 45:
    raise ValueError("Invalid prior transition before counts")

l12_refs = l12_trace["printed_state_references"]
if l12_refs["transcript_ordinal"] != 99495 or l12_refs["possible_output_truncation"] is not True or len(l12_refs["references"]) != 8:
    raise ValueError("Invalid Lens 12 state reference inventory")
for sequence, reference in enumerate(l12_refs["references"], start=1):
    if reference["sequence"] != sequence or reference["fragment"] != f"STATE-{sequence:03}-REFERENCE.txt" or reference["path"] != f"segovb/STATE-L12-METADATA-CURRENT-{sequence:03}.json":
        raise ValueError("Invalid Lens 12 state reference identity")
    data = (l12_trace_dir / reference["fragment"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != reference["fragment_sha256"]:
        raise ValueError("Lens 12 state reference fragment mismatch")
    state = json.loads("{" + data.decode("utf-8") + "}")["state"]
    if state != {"path": reference["path"], "sha256": reference["sha256"]}:
        raise ValueError("Lens 12 printed state reference mismatch")
if l12_refs["references"][-1]["sha256"] != l12_trace["target_sha256"]:
    raise ValueError("Lens 12 target state reference mismatch")

l12_baseline = l12_trace["pre_metadata_baseline"]
if (l12_baseline["path"] != "segovb/STATE-L12-960.json"
        or l12_baseline["sha256"] != "3d78ed9e6405a8741f30d190cf6fe6e93d490d2b0ab6938d09504407188b5ceb"
        or l12_baseline["prefix_file"] != "STATE-960-PREFIX.txt"):
    raise ValueError("Invalid Lens 12 baseline reference")
baseline_ref = (l12_trace_dir / "STATE-960-REFERENCE.txt").read_bytes()
if hashlib.sha256(baseline_ref).hexdigest() != l12_baseline["reference_sha256"] or l12_baseline["sha256"].encode() not in baseline_ref or l12_baseline["path"].encode() not in baseline_ref:
    raise ValueError("Lens 12 baseline printed reference mismatch")
baseline_prefix = (l12_trace_dir / l12_baseline["prefix_file"]).read_bytes()
if hashlib.sha256(baseline_prefix).hexdigest() != l12_baseline["prefix_sha256"] or len(baseline_prefix.splitlines()) != 60:
    raise ValueError("Lens 12 baseline prefix mismatch")
for key, expected in l12_baseline["references"].items():
    match = re.search(r'"' + re.escape(key) + r'": "([^"]+)"', baseline_prefix.decode("utf-8"))
    if not match or match.group(1) != expected:
        raise ValueError("Lens 12 baseline dependency reference mismatch")

sats_dir = root / "recovered/l12-sats-inherited-evidence"
sats = json.loads((sats_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if sats["schema"] != "726-l12-sats-inherited-evidence-v1" or len(sats["objects"]) != 13 or sats["qualified_union"] != [[1, 2523]]:
    raise ValueError("Invalid Lens 12 inherited SATS evidence")
sats_objects = {}
for item in sats["objects"]:
    sha = item["sha256"]
    if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError("Invalid Lens 12 SATS object path")
    data = (sats_dir / "objects" / sha).read_bytes()
    if hashlib.sha256(data).hexdigest() != sha or len(data) != item["bytes"]:
        raise ValueError("Lens 12 SATS object mismatch")
    sats_objects[item["path"]] = (item, data)
team = json.loads(sats_objects[sats["team_receipt"]["path"]][1])
ordered = json.loads(sats_objects[sats["ordered_receipt"]["path"]][1])
for reference in [team["manifest"], *team["evidence_refs"]]:
    if sats_objects[reference["path"]][0]["sha256"] != reference["sha256"]:
        raise ValueError("Lens 12 SATS receipt reference mismatch")
for row in team["checked_rows"] + team["primary_source_proofs"]:
    if (sats_objects[row["registration"]][0]["sha256"] != row["registration_sha256"]
            or sats_objects[row["receipt"]["path"]][0]["sha256"] != row["receipt"]["sha256"]
            or sats_objects["git:" + row["git_path"]][0]["sha256"] != row["source_sha256"]):
        raise ValueError("Lens 12 SATS source relationship mismatch")
raw_receipt = json.loads(sats_objects[team["evidence_refs"][0]["path"]][1])
partial = {n for lo, hi in raw_receipt["rows"][2]["assessed_ranges"] for n in range(lo, hi + 1)}
fresh = {n for lo, hi in ordered["rows"][0]["fresh_personal_body_reads"] for n in range(lo, hi + 1)}
if (ordered["rows"][0]["original_index"] != 2389 or partial & fresh or partial | fresh != set(range(1, 2524))
        or raw_receipt["rows"][2]["assessed_ranges"] != sats["partial_ranges"]
        or ordered["rows"][0]["fresh_personal_body_reads"] != sats["fresh_ranges"]):
    raise ValueError("Lens 12 SATS attribution or range mismatch")

sats_follow = sats["follow_on_construction"]
def symbolic_value(node):
    try:
        return ast.literal_eval(node)
    except (ValueError, TypeError):
        pass
    if isinstance(node, ast.Dict):
        return {ast.literal_eval(k): symbolic_value(v) for k, v in zip(node.keys, node.values)}
    if isinstance(node, ast.List):
        return [symbolic_value(x) for x in node.elts]
    return {"unresolved_expression": ast.unparse(node)}
if [(x["event_ordinal"], x["kind"]) for x in sats_follow["expressions"]] != [(73557, "proof"), (73584, "policy"), (73584, "view")]:
    raise ValueError("Invalid Lens 12 SATS construction trace")
for item in sats_follow["expressions"]:
    if item["file"] != f'{item["event_ordinal"]}-{item["kind"]}-expression.txt':
        raise ValueError("Invalid Lens 12 SATS expression path")
    data = (sats_dir / item["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["sha256"] or symbolic_value(ast.parse(data.decode("utf-8"), mode="eval").body) != item["fields"]:
        raise ValueError("Lens 12 SATS expression mismatch")
update = ast.parse(sats_follow["state_update_expression"], mode="eval").body
if not isinstance(update, ast.Call) or ast.unparse(update.func) != "state.update" or not any(x.arg == "metadata_predecessor" for x in update.keywords):
    raise ValueError("Invalid Lens 12 SATS state update")
results = sats_follow["reported_outputs"]
if [x["event_ordinal"] for x in results] != [73557, 73584, 73609] or results[0]["results"][0]["qualified_sources"] != 4:
    raise ValueError("Invalid Lens 12 SATS reported result")
for entry, mode in zip(results[1:], ["--preflight", "--verify"]):
    result = entry["results"][-1]
    if result.get("status") != "pass" or result.get("mode") != mode or result.get("checkpoint") != 495:
        raise ValueError("Invalid Lens 12 SATS reported verification")

l2_refs = l2_trace["printed_state_references"]
if [x["sequence"] for x in l2_refs["references"]] != list(range(23, 29)):
    raise ValueError("Invalid Lens 2 state references")
for reference in l2_refs["references"]:
    sequence = reference["sequence"]
    if reference["fragment"] != f"STATE-{sequence:03}-REFERENCE.txt" or reference["path"] != f"segovb/STATE-L2-EXT-CURRENT-{sequence:03}.json":
        raise ValueError("Invalid Lens 2 state reference path")
    data = (l2_trace_dir / reference["fragment"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != reference["fragment_sha256"]:
        raise ValueError("Lens 2 state reference fragment mismatch")
    value = json.loads("{" + data.decode("utf-8") + "}")["state"]
    if value != {"path": reference["path"], "sha256": reference["sha256"]}:
        raise ValueError("Lens 2 state reference mismatch")
l2_candidate = l2_trace["state_023_candidate"]
state_template = (l2_trace_dir / "STATE-023-CANDIDATE.txt").read_bytes()
policy_template = (l2_trace_dir / "POLICY-023-CANDIDATE.txt").read_bytes()
if hashlib.sha256(state_template).hexdigest() != l2_candidate["state_template_sha256"] or hashlib.sha256(policy_template).hexdigest() != l2_candidate["policy_template_sha256"]:
    raise ValueError("Lens 2 state reconstruction template mismatch")
stamp = l2_candidate["matched_timestamp"].encode("utf-8")
policy_sha = l2_candidate["policy_sha256"]
policy_data = (l2_trace_dir / policy_sha).read_bytes()
state_sha = l2_candidate["target_sha256"]
state_data = (l2_trace_dir / state_sha).read_bytes()
if (hashlib.sha256(policy_data).hexdigest() != policy_sha or hashlib.sha256(state_data).hexdigest() != state_sha
        or policy_template.replace(b"STAMP_MARKER", stamp) != policy_data
        or state_template.replace(b"STAMP_MARKER", stamp).replace(b"POLICY_HASH_MARKER", policy_sha.encode()) != state_data
        or state_sha != l2_refs["references"][0]["sha256"]):
    raise ValueError("Lens 2 state or policy original hash mismatch")
state = json.loads(state_data)
if (state["predecessor"]["sha256"] != "c13841f9c16ac9189b8aad7ef79c8e8822a57e107de07df0b27e1300a7c98320"
        or state["current_receipt"]["sha256"] != "c8ac419fd0fe6b38f12eeb8f2338f54331c2589ef46b7f13aa09e31c04904299"
        or state["camp_carrier_source_disposition"]["policy"]["sha256"] != policy_sha
        or state["camp_carrier_source_disposition"]["implementation_gate_complete"] is not False):
    raise ValueError("Lens 2 state dependency mismatch")

l2_024 = l2_trace["state_024_candidate"]
state_024 = (l2_trace_dir / l2_024["target_sha256"]).read_bytes()
read_024 = (l2_trace_dir / l2_024["read_sha256"]).read_bytes()
template_024 = (l2_trace_dir / "STATE-024-STATE-CANDIDATE.txt").read_bytes()
read_template_024 = (l2_trace_dir / "STATE-024-READ-CANDIDATE.txt").read_bytes()
stamp_024 = l2_024["matched_timestamp"].encode("utf-8")
read_hash_024 = l2_024["read_sha256"].encode("ascii")
if (hashlib.sha256(state_024).hexdigest() != l2_024["target_sha256"]
        or l2_024["target_sha256"] != l2_refs["references"][1]["sha256"]
        or hashlib.sha256(read_024).hexdigest() != l2_024["read_sha256"]
        or hashlib.sha256(template_024).hexdigest() != l2_024["state_template_sha256"]
        or hashlib.sha256(read_template_024).hexdigest() != l2_024["read_template_sha256"]
        or template_024.replace(b"STAMP", stamp_024).replace(b"READHASH", read_hash_024) != state_024
        or read_template_024.replace(b"STAMP", stamp_024) != read_024):
    raise ValueError("Lens 2 state 024 or receipt original hash mismatch")
state_value_024 = json.loads(state_024)
read_value_024 = json.loads(read_024)
policy_candidate_024 = (l2_trace_dir / "POLICY-024-RECONSTRUCTED-CANDIDATE.json").read_bytes()
if (state_value_024["predecessor"]["sha256"] != state_sha
        or state_value_024["current_receipt"]["sha256"] != l2_024["read_sha256"]
        or len(read_value_024["rows"]) != 5
        or read_value_024["uncredited_excerpt_lines"] != 401
        or read_value_024["new_body_or_population_credit"] != 0
        or hashlib.sha256(policy_candidate_024).hexdigest() != l2_024["policy_candidate_sha256"]):
    raise ValueError("Lens 2 state 024 dependency mismatch")

l2_025 = l2_trace["state_025_candidate"]
state_025 = (l2_trace_dir / l2_025["target_sha256"]).read_bytes()
read_025 = (l2_trace_dir / l2_025["read_sha256"]).read_bytes()
policy_025 = (l2_trace_dir / l2_025["policy_sha256"]).read_bytes()
templates_025 = {name: (l2_trace_dir / ("STATE-025-" + name.upper() + "-CANDIDATE.txt")).read_bytes() for name in ("state", "read", "policy")}
stamp_025 = l2_025["matched_timestamp"].encode("utf-8")
read_hash_025 = l2_025["read_sha256"].encode("ascii")
policy_hash_025 = l2_025["policy_sha256"].encode("ascii")
for name, data in templates_025.items():
    if hashlib.sha256(data).hexdigest() != l2_025[name + "_template_sha256"]:
        raise ValueError("Lens 2 state 025 template mismatch")
if (hashlib.sha256(state_025).hexdigest() != l2_refs["references"][2]["sha256"]
        or hashlib.sha256(read_025).hexdigest() != l2_025["read_sha256"]
        or hashlib.sha256(policy_025).hexdigest() != l2_025["policy_sha256"]
        or templates_025["state"].replace(b"STAMP", stamp_025).replace(b"READHASH", read_hash_025).replace(b"POLICYHASH", policy_hash_025) != state_025
        or templates_025["read"].replace(b"STAMP", stamp_025) != read_025
        or templates_025["policy"].replace(b"STAMP", stamp_025).replace(b"READHASH", read_hash_025) != policy_025):
    raise ValueError("Lens 2 state 025, receipt or policy original hash mismatch")
state_value_025 = json.loads(state_025)
read_value_025 = json.loads(read_025)
if (state_value_025["predecessor"]["sha256"] != l2_024["target_sha256"]
        or state_value_025["current_receipt"]["sha256"] != l2_025["read_sha256"]
        or state_value_025["errantry_id_source_disposition"]["policy"]["sha256"] != l2_025["policy_sha256"]
        or read_value_025["qualified_prior_rows"] != 26
        or read_value_025["new_body_credit"] != 0):
    raise ValueError("Lens 2 state 025 dependency mismatch")

journey_reg_dir = root / "recovered/l2-journey-source-registrations"
journey_regs = json.loads((journey_reg_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if journey_regs["schema"] != "726-l2-journey-source-registrations-v1" or [e["ordinal"] for e in journey_regs["entries"]] != [72738, 72758, 72768]:
    raise ValueError("Invalid journey source registration inventory")
for entry in journey_regs["entries"]:
    data = (journey_reg_dir / entry["printed"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != entry["printed_sha256"]:
        raise ValueError("Journey printed registration mismatch")
    printed = json.loads(data)
    if printed["registration"] != entry["registration"] or printed["uncredited_lines"] != entry["uncredited_lines"]:
        raise ValueError("Journey registration reference mismatch")
    total = 0
    for source in printed["sources"]:
        raw = (journey_reg_dir / "objects" / source["source_sha256"]).read_bytes()
        if hashlib.sha256(raw).hexdigest() != source["source_sha256"] or len(raw.decode("utf-8").splitlines()) != source["actual_EOF"]:
            raise ValueError("Journey source object mismatch")
        for low, high in source["ranges"]:
            if not 1 <= low <= high <= source["actual_EOF"]:
                raise ValueError("Invalid journey source range")
            total += high - low + 1
    if total != entry["uncredited_lines"]:
        raise ValueError("Journey source line count mismatch")

journey_display_count = 0
journey_lines = set()
journey_fragments = journey_regs["source_displays"]["fragments"]
if len(journey_fragments) != len(read_value_024["rows"]):
    raise ValueError("Journey source display row count mismatch")
for fragment, row in zip(journey_fragments, read_value_024["rows"]):
    data = (journey_reg_dir / "display-fragments" / fragment["file"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != fragment["sha256"]
            or fragment["source_sha256"] != row["source"]["source_sha256"]
            or fragment["ranges"] != row["assessed_ranges"]):
        raise ValueError("Journey source display reference mismatch")
    source_lines = (journey_reg_dir / "objects" / fragment["source_sha256"]).read_text(encoding="utf-8").splitlines()
    expected = [f"{line}: {source_lines[line - 1]}" for low, high in fragment["ranges"] for line in range(low, high + 1)]
    if data.decode("utf-8").splitlines() != expected or len(expected) != fragment["lines"]:
        raise ValueError("Journey source display mismatch")
    journey_display_count += len(expected)
    journey_lines.update((fragment["source_sha256"], line) for low, high in fragment["ranges"] for line in range(low, high + 1))
if journey_display_count != 401 or len(journey_lines) != 401:
    raise ValueError("Journey source display count mismatch")

native_dir = root / "recovered/l2-native-registration-evidence"
native = json.loads((native_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if native["schema"] != "726-l2-native-registration-evidence-v1" or len(native["sdk_registration_candidates"]) != 3 or len(native["printed_git_registrations"]) != 2:
    raise ValueError("Invalid native registration evidence inventory")
lock_data = (native_dir / "Cargo.lock").read_bytes()
if hashlib.sha256(lock_data).hexdigest() != native["lock_sha256"]:
    raise ValueError("Native source lock mismatch")
packages = {p["name"]: p for p in tomllib.loads(lock_data.decode("utf-8"))["package"]}
for archive in native["archives"]:
    data = (native_dir / "archives" / archive["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != archive["sha256"]:
        raise ValueError("Native archive hash mismatch")
for candidate in native["sdk_registration_candidates"]:
    data = (native_dir / candidate["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != candidate["sha256"]:
        raise ValueError("Native registration candidate mismatch")
    value = json.loads(data)
    if "package" not in value:
        continue
    package = value["package"]
    if package != packages[package["name"]] or value["lock_sha256"] != native["lock_sha256"]:
        raise ValueError("Native registration lock dependency mismatch")
    archive_path = native_dir / "archives" / (package["name"] + "-" + package["version"] + ".crate")
    if hashlib.sha256(archive_path.read_bytes()).hexdigest() != package["checksum"]:
        raise ValueError("Native archive differs from Cargo.lock")
    with tarfile.open(archive_path) as archive:
        for row in value["rows"]:
            data = archive.extractfile(row["archive_member"]).read()
            if (hashlib.sha256(data).hexdigest() != row["source_sha256"]
                    or data != (native_dir / "objects" / row["source_sha256"]).read_bytes()
                    or len(data.decode("utf-8").splitlines()) != row["actual_EOF"]):
                raise ValueError("Native source object differs from archive")
sdk_display_count = 0
for fragment in native["sdk_source_displays"]["fragments"]:
    data = (native_dir / "display-fragments" / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Native SDK source display hash mismatch")
    source_lines = (native_dir / "objects" / fragment["source_sha256"]).read_text(encoding="utf-8").splitlines()
    expected = [f"{line}: {source_lines[line - 1]}" for low, high in fragment["ranges"] for line in range(low, high + 1)]
    if data.decode("utf-8").splitlines() != expected or len(expected) != fragment["lines"]:
        raise ValueError("Native SDK source display mismatch")
    sdk_display_count += len(expected)
if sdk_display_count != native["sdk_source_displays"]["matched_historical_line_occurrences"] or sdk_display_count != 511:
    raise ValueError("Native SDK source display count mismatch")
git_display_count = 0
for fragment in native["git_source_displays"]["fragments"]:
    data = (native_dir / "display-fragments" / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Native Git source display hash mismatch")
    source_lines = (native_dir / "objects" / fragment["source_sha256"]).read_text(encoding="utf-8").splitlines()
    expected = [f"{line}: {source_lines[line - 1]}" for low, high in fragment["ranges"] for line in range(low, high + 1)]
    if data.decode("utf-8").splitlines() != expected or len(expected) != fragment["lines"]:
        raise ValueError("Native Git source display mismatch")
    git_display_count += len(expected)
if git_display_count != native["git_source_displays"]["matched_historical_line_occurrences"] or git_display_count != 339:
    raise ValueError("Native Git source display count mismatch")
for entry in native["printed_git_registrations"]:
    data = (native_dir / entry["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != entry["sha256"]:
        raise ValueError("Native printed registration mismatch")
    printed = json.loads(data)
    if printed["registration"] != entry["registration"]:
        raise ValueError("Native printed registration reference mismatch")
    for row in printed["sources"]:
        source = (native_dir / "objects" / row["source_sha256"]).read_bytes()
        if hashlib.sha256(source).hexdigest() != row["source_sha256"] or len(source.decode("utf-8").splitlines()) != row["actual_EOF"]:
            raise ValueError("Native printed Git source mismatch")

contract_dir = root / "recovered/l2-contract-registration-evidence"
contract_regs = json.loads((contract_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if contract_regs["schema"] != "726-l2-contract-registration-evidence-v1" or [e["ordinal"] for e in contract_regs["entries"]] != [73113, 73161]:
    raise ValueError("Invalid contract registration inventory")
for entry in contract_regs["entries"]:
    data = (contract_dir / entry["printed"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != entry["printed_sha256"]:
        raise ValueError("Contract printed registration mismatch")
    printed = json.loads(data)
    if printed["registration"] != entry["registration"] or printed["uncredited_lines"] != entry["uncredited_lines"]:
        raise ValueError("Contract registration reference mismatch")
    total = 0
    for source in printed["sources"]:
        raw = (contract_dir / "objects" / source["source_sha256"]).read_bytes()
        if hashlib.sha256(raw).hexdigest() != source["source_sha256"] or len(raw.decode("utf-8").splitlines()) != source["actual_EOF"]:
            raise ValueError("Contract source object mismatch")
        for low, high in source["ranges"]:
            if not 1 <= low <= high <= source["actual_EOF"]:
                raise ValueError("Invalid contract source range")
            total += high - low + 1
    if total != entry["uncredited_lines"]:
        raise ValueError("Contract source line count mismatch")

contract_display_count = 0
contract_distinct_lines = set()
for fragment in contract_regs["source_displays"]["fragments"]:
    data = (contract_dir / "display-fragments" / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Contract source display hash mismatch")
    source_lines = (contract_dir / "objects" / fragment["source_sha256"]).read_text(encoding="utf-8").splitlines()
    expected = [f"{line}: {source_lines[line - 1]}" for low, high in fragment["ranges"] for line in range(low, high + 1)]
    if data.decode("utf-8").splitlines() != expected or len(expected) != fragment["lines"]:
        raise ValueError("Contract source display mismatch")
    contract_display_count += len(expected)
    contract_distinct_lines.update((fragment["source_sha256"], line) for low, high in fragment["ranges"] for line in range(low, high + 1))
if (contract_display_count != 931 or len(contract_distinct_lines) != 928
        or contract_display_count != contract_regs["source_displays"]["matched_historical_line_occurrences"]
        or len(contract_distinct_lines) != contract_regs["source_displays"]["distinct_source_lines"]):
    raise ValueError("Contract source display count mismatch")

comparison_dir = root / "recovered/l2-canonical-comparison-evidence"
comparison = json.loads(_qualified_historical_manifest("recovered/l2-canonical-comparison-evidence/MANIFEST.json"))
if comparison["schema"] != "726-l2-canonical-comparison-evidence-v1":
    raise ValueError("Invalid canonical comparison evidence inventory")
comparison_master_bytes = None
for item in comparison["files"]:
    if item["file"] == "74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48":
        comparison_master_bytes = _qualified_public_object(
            "recovered/l2-canonical-comparison-evidence/" + item["file"], item["sha256"],
            (comparison_dir / item["file"]).read_bytes())
    elif hashlib.sha256((comparison_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Canonical comparison evidence hash mismatch")
if (comparison["inherited_master_sha256"] != "74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48"
        or comparison_master_bytes is None):
    raise ValueError("Qualified canonical comparison original identity missing")
lines = (comparison_dir / "PRINTED-CANDIDATES.txt").read_text(encoding="utf-8").splitlines()
result = json.loads((comparison_dir / "PRINTED-RESULT.json").read_bytes())
literals = json.loads((comparison_dir / "RECORDED-LITERALS.json").read_bytes())
master = json.loads(comparison_master_bytes)
special_dispositions = json.loads((comparison_dir / "RECORDED-SPECIAL-DISPOSITIONS.json").read_bytes())
comparison_commands = [json.loads(line) for line in (root / "recovered/queue-command-outputs/RECORDS.jsonl").read_text(encoding="utf-8").splitlines() if '"transcript_ordinal":73385' in line]
if len(comparison_commands) != 1:
    raise ValueError("Canonical comparison write command missing")
comparison_command = comparison_commands[0]["command_text"]
comparison_ast = ast.parse(comparison_command.split("@'\n", 1)[1].split("\n'@", 1)[0])
special_node = next(item.value for item in comparison_ast.body if isinstance(item, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "special" for target in item.targets))
special_original = ast.literal_eval(special_node)
if (special_dispositions["source_write_ordinal"] != 73385 or len(special_dispositions["records"]) != 15
        or special_dispositions["records"] != [{"queue_index": key, "disposition": value[0], "reason": value[1]} for key, value in special_original.items()]
        or not set(special_original).issubset({int(line.split()[1]) for line in lines if line.startswith("queue ")})):
    raise ValueError("Canonical comparison recorded dispositions mismatch")
first_comparison_refs = json.loads((comparison_dir / "FIRST-COMPARISON-RECOVERY.json").read_bytes())
first_comparison_data = (comparison_dir / first_comparison_refs["comparison"]["sha256"]).read_bytes()
first_comparison_template = (comparison_dir / "FIRST-COMPARISON-TEMPLATE.txt").read_bytes()
if (hashlib.sha256(first_comparison_data).hexdigest() != first_comparison_refs["comparison"]["sha256"]
        or hashlib.sha256(first_comparison_template).hexdigest() != first_comparison_refs["template_sha256"]
        or first_comparison_template.replace(b"STAMP_MARKER", first_comparison_refs["matched_timestamp"].encode("utf-8")) != first_comparison_data):
    raise ValueError("First comparison original hash mismatch")
first_comparison = json.loads(first_comparison_data)
if (first_comparison["index"]["sha256"] != first_comparison_refs["index"]["sha256"]
        or first_comparison["queue"]["sha256"] != first_comparison_refs["queue"]["sha256"]
        or first_comparison["candidate_rows_total"] != 73 or first_comparison["candidate_comparisons_saved"] != 5
        or first_comparison["global_alias_complete"] is not False or first_comparison["new_unique_or_body_credit"] != 0):
    raise ValueError("First comparison dependency mismatch")
first_compare_outputs = {}
for line in (root / "recovered/ledger-command-outputs/OUTPUTS.jsonl").read_text(encoding="utf-8").splitlines():
    if '"transcript_output_ordinal":73214' in line or '"transcript_output_ordinal":73271' in line:
        item = json.loads(line)
        first_compare_outputs[item["transcript_output_ordinal"]] = item["output_text"]
if set(first_compare_outputs) != {73214, 73271}:
    raise ValueError("First comparison printed references missing")
for key in ("comparison", "index"):
    ref = first_comparison_refs[key]
    if ref["sha256"] not in first_compare_outputs[73271] or ref["path"].split("/")[-1] not in first_compare_outputs[73271]:
        raise ValueError("First comparison printed dependency mismatch")
if first_comparison_refs["queue"]["sha256"] not in first_compare_outputs[73214] or first_comparison_refs["queue"]["path"] not in first_compare_outputs[73214]:
    raise ValueError("First comparison printed queue mismatch")

nested = json.loads((comparison_dir / "PRINTED-NESTED-JOURNAL-VIEWS.json").read_bytes())
if (nested["parent_index"] != 1880 or nested["parent_declared_lens"] != 12
        or nested["related_queue_index"] != 991 or nested["related_declared_lens"] != 12
        or len(nested["nested_views"]) != 2):
    raise ValueError("Nested journal projection inventory mismatch")
nested_sources = json.loads((comparison_dir / "PRINTED-NESTED-SOURCE-REFERENCES.json").read_bytes())
if [item["ordinal"] for item in nested_sources["records"]] != [43403, 43439, 43540, 43567]:
    raise ValueError("Nested source-reference inventory mismatch")
for item in nested_sources["records"]:
    printed = item["printed"]
    keys = ("registration", "qualification") if "registration" in printed else ("packet", "plan")
    for key in keys:
        ref = printed[key]
        if not re.fullmatch(r"[0-9a-f]{64}", ref["sha256"]) or not ref["path"].startswith("adler-continuation-20261030/"):
            raise ValueError("Invalid nested source reference")
if [item["printed"]["fresh_lines"] for item in nested_sources["records"]] != [540, 540, 173, 173]:
    raise ValueError("Nested historical source counts mismatch")
nested_dependencies = json.loads((comparison_dir / "PRINTED-NESTED-PROPOSAL-DEPENDENCIES.json").read_bytes())
if len(nested_dependencies["records"]) != 2:
    raise ValueError("Nested proposal dependency inventory mismatch")
for index, record in enumerate(nested_dependencies["records"]):
    if record["printed_references"]["proposal"] != nested["nested_views"][index]["reference"]["proposal"]:
        raise ValueError("Nested proposal reference mismatch")
    for kind in ("proposal", "qualification", "writer"):
        reference = record["printed_references"][kind]
        if not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"]) or not reference["path"].startswith("adler-continuation-20261030/"):
            raise ValueError("Invalid nested proposal dependency reference")
    for name in ("qual", "proposal"):
        expression = ast.parse((comparison_dir / f"{record['writer_event']}-{name}-expression.txt").read_text(encoding="utf-8"), mode="eval").body
        if not isinstance(expression, ast.Dict):
            raise ValueError("Invalid nested proposal write expression")

queue_outputs = {}
for line in (root / "recovered/queue-command-outputs/RECORDS.jsonl").read_text(encoding="utf-8").splitlines():
    if '"transcript_ordinal":73363' in line or '"transcript_ordinal":73370' in line:
        item = json.loads(line)
        queue_outputs[item["transcript_ordinal"]] = item["output_text"]
if set(queue_outputs) != {73363, 73370}:
    raise ValueError("Nested journal printed output missing")
printed_nested = [json.JSONDecoder().raw_decode(queue_outputs[73363][m.start():])[0] for m in re.finditer(r"(?m)^\{", queue_outputs[73363])]
for index, item in enumerate(nested["nested_views"]):
    ref = item["reference"]
    if (item["projection"] != printed_nested[index] or ref["index"] != index or ref["lens"] != 13
            or item["projection"]["lens"] != 13
            or len(str(item["projection"]["first_failed_contract_or_unresolved_question"])) != ref["first_length"]
            or ref["proposal"]["sha256"] not in queue_outputs[73370] or ref["status"] != "open-reconciliation"):
        raise ValueError("Nested journal view projection mismatch")
suppressed = json.loads((comparison_dir / "SUPPRESSED-FIRST-CONTRACT-CANDIDATE.json").read_bytes())
suppressed_candidate = suppressed["candidate"]
if (suppressed_candidate["value"]["contract_id"] != "Q-L12-b20c483ed43f4bb0dd61"
        or len(json.dumps(suppressed_candidate["value"]["first_failed_contract_or_unresolved_question"])) != suppressed["display_length"]
        or suppressed["display_length"] != 2397):
    raise ValueError("Suppressed first-contract candidate mismatch")
if not any(json.loads(line) == suppressed_candidate for line in (root / "recovered/transcript-queue-fragments/PRINTED-VALUES.jsonl").read_text(encoding="utf-8").splitlines()):
    raise ValueError("Suppressed first-contract candidate source missing")
suppressed_outputs = [json.loads(line) for line in (root / "recovered/queue-command-outputs/RECORDS.jsonl").read_text(encoding="utf-8").splitlines() if '"transcript_ordinal":69232' in line]
if len(suppressed_outputs) != 1 or "CLOSURE-QUEUE-815.json" not in suppressed_outputs[0]["command_text"] or suppressed["source_output_ordinal"] != 69232:
    raise ValueError("Suppressed first-contract historical queue source missing")
suppressed_match = re.search(r"(?m)^1879 (\{)", suppressed_outputs[0]["output_text"])
if suppressed_match is None:
    raise ValueError("Suppressed first-contract historical projection missing")
suppressed_projection = json.JSONDecoder().raw_decode(suppressed_outputs[0]["output_text"][suppressed_match.start(1):])[0]
if (suppressed_projection["contract_id"] != suppressed_candidate["value"]["contract_id"]
        or suppressed_projection["first_failed_contract_or_unresolved_question"] != suppressed_candidate["value"]["first_failed_contract_or_unresolved_question"]):
    raise ValueError("Suppressed first-contract historical projection mismatch")
long_1879 = json.loads((comparison_dir / "PRINTED-LONG1879-FIRST-CONTRACT.json").read_bytes())
long_outputs = [json.loads(line) for line in (root / "recovered/queue-command-outputs/RECORDS.jsonl").read_text(encoding="utf-8").splitlines() if '"transcript_ordinal":73307' in line]
if len(long_outputs) != 1 or long_1879["output_ordinal"] != 73307:
    raise ValueError("Queue 828 long first-contract output missing")
long_source = long_outputs[0]
if ("CLOSURE-QUEUE-828.json" not in long_source["command_text"]
        or "q['entries'][1879]['first_failed_contract_or_unresolved_question']" not in long_source["command_text"]
        or long_1879["queue_index"] != 1879
        or long_1879["contract_id"] != suppressed_candidate["value"]["contract_id"]):
    raise ValueError("Queue 828 long first-contract source mismatch")
long_printed = json.JSONDecoder().raw_decode(long_source["output_text"].split("LONG1879 ", 1)[1])[0]
if (long_printed != long_1879["first_contract"]
        or long_printed != suppressed_candidate["value"]["first_failed_contract_or_unresolved_question"]
        or long_1879["matches_queue_815_candidate"] is not True):
    raise ValueError("Queue 828 long first-contract projection mismatch")

queue_projections = json.loads((comparison_dir / "PRINTED-QUEUE-828-PROJECTIONS.json").read_bytes())
if (queue_projections["queue_path"] != "workflow-20261030/CLOSURE-QUEUE-828.json"
        or [x["queue_index"] for x in queue_projections["records"]] != [205, 245, 3292, 3293, 3768, 1880, 991, 2707]):
    raise ValueError("Canonical comparison queue projection inventory mismatch")
candidate_ids = {int(parts[1]): parts[2] for line in lines if (parts := line.split())[0] == "queue"}
candidate_observations = json.loads((comparison_dir / "SEPARATE-CANDIDATE-FIELD-OBSERVATIONS.json").read_bytes())
if (len(candidate_observations["records"]) != 36 or candidate_observations["first_contract_values"] != 34
        or candidate_observations["canonical_owner_values"] != 36):
    raise ValueError("Candidate field observation inventory mismatch")
normalized_values = {}
for line in (root / "recovered/transcript-queue-fragments/PRINTED-VALUES.jsonl").read_text(encoding="utf-8").splitlines():
    item = json.loads(line)
    normalized_values[item["normalized_sha256"]] = item["value"]
for row in candidate_observations["records"]:
    if candidate_ids.get(row["queue_index"]) != row["contract_id"]:
        raise ValueError("Candidate field observation identity mismatch")
    for field, observation in row["observations"].items():
        value = normalized_values.get(observation["source_normalized_sha256"])
        if not isinstance(value, dict) or value.get("contract_id") != row["contract_id"] or value.get(field) != observation["value"]:
            raise ValueError("Candidate field observation source mismatch")

for projection in queue_projections["records"]:
    if candidate_ids.get(projection["queue_index"]) != projection["projection"]["contract_id"]:
        raise ValueError("Canonical comparison projected candidate mismatch")
first_contracts = json.loads((comparison_dir / "PRINTED-FIRST-CONTRACTS.json").read_bytes())
if (len(first_contracts["primary"]) != 6 or len(first_contracts["queue"]) != 45
        or sum(row["first_contract"] is None for row in first_contracts["queue"]) != 1):
    raise ValueError("Canonical comparison first-contract inventory mismatch")
for recorded, line in zip(first_contracts["primary"], lines[:6]):
    if line.split()[2] != recorded["contract_id"]:
        raise ValueError("Canonical comparison primary candidate mismatch")
primary_observations = json.loads((comparison_dir / "PRINTED-PRIMARY-OBSERVATIONS.json").read_bytes())
if (primary_observations["schema"] != "726-l2-primary-observations-v1"
        or primary_observations["source_output_ordinal"] != 73278
        or primary_observations["recorded_exit_code"] != 1
        or len(primary_observations["records"]) != 6
        or len(primary_observations["output_excerpt"]) != 18):
    raise ValueError("Primary comparison observation inventory mismatch")
intervening = json.loads((comparison_dir / "INTERVENING-COMMAND-TRACE.json").read_bytes())
expected_intervening = [73285, 73286, 73293, 73300, 73307, 73314, 73325, 73332, 73345, 73352, 73363, 73370]
if (intervening["schema"] != "726-l2-comparison-intervening-command-trace-v1"
        or intervening["after_observation_ordinal"] != 73278 or intervening["before_comparison_ordinal"] != 73385
        or [item["ordinal"] for item in intervening["recorded_command_events"]] != expected_intervening):
    raise ValueError("Intervening command inventory mismatch")
for item in intervening["recorded_command_events"]:
    if item["recorded_exit_code"] != 0 or not re.fullmatch(r"[0-9a-f]{64}", item["original_command_sha256"]):
        raise ValueError("Invalid intervening command metadata")
    if item["kind"] == "rg-file-list":
        if item["ordinal"] != 73285 or not item["command_body"].startswith("rg --files --hidden --no-ignore "):
            raise ValueError("Unexpected intervening file listing")
    elif item["kind"] == "python-read-command":
        command_ast = ast.parse(item["command_body"])
        for node in ast.walk(command_ast):
            if isinstance(node, (ast.Import, ast.ImportFrom)):
                modules = [alias.name for alias in node.names] if isinstance(node, ast.Import) else [node.module]
                if not set(modules).issubset({"pathlib", "json", "hashlib"}):
                    raise ValueError("Unexpected intervening command import")
            if isinstance(node, ast.Call):
                allowed_names = {"Path", "str", "print", "len", "enumerate", "isinstance", "type", "any", "list"}
                allowed_attrs = {"read_bytes", "dumps", "loads", "sha256", "hexdigest", "lower", "get", "items"}
                if not ((isinstance(node.func, ast.Name) and node.func.id in allowed_names)
                        or (isinstance(node.func, ast.Attribute) and node.func.attr in allowed_attrs)):
                    raise ValueError("Unexpected intervening command call")
    else:
        raise ValueError("Unexpected intervening command type")

primary_command = (comparison_dir / "PRIMARY-OBSERVATION-COMMAND.txt").read_text(encoding="utf-8")
ast.parse(primary_command)
if ("evidence/continuation-segovb/ledger.json" not in primary_command
        or "print('FIRST',v.get('first_failed_contract'))" not in primary_command
        or "print('OWNER',v.get('canonical_owner'))" not in primary_command):
    raise ValueError("Primary observation command source mismatch")
primary_loop = next(item for item in comparison_ast.body if isinstance(item, ast.For)
                    and isinstance(item.target, ast.Name) and item.target.id == "c")
primary_branch = primary_loop.body[0].body[-1]
if ast.unparse(primary_branch.test) != "c['key'] == 'SEGOVB-L11-CONTRACT-ABANDON-PARTY-ACTIVE-RELATION'":
    raise ValueError("Primary comparison writer condition mismatch")
primary_special_pair = ast.literal_eval(primary_branch.body[0].value)
primary_default_pair = ast.literal_eval(primary_branch.orelse[0].value)
for index, (record, original, line) in enumerate(zip(primary_observations["records"], first_contracts["primary"], lines[:6])):
    kind, key, contract_id, terms = line.split()
    pair = primary_special_pair if key == "SEGOVB-L11-CONTRACT-ABANDON-PARTY-ACTIVE-RELATION" else primary_default_pair
    if (kind != "primary" or key != contract_id or contract_id != original["contract_id"]
            or record != {"key": key, "contract_id": contract_id, "matched_terms": terms.split(","),
                          "observed_first_contract": original["first_contract"],
                          "observed_canonical_owner": original["canonical_owner"],
                          "writer_disposition": pair[0], "writer_reason": pair[1]}
            or primary_observations["output_excerpt"][index * 3:index * 3 + 3]
                != [key, "FIRST " + original["first_contract"], "OWNER " + original["canonical_owner"]]):
        raise ValueError("Primary comparison observation mismatch")

remaining_first = json.loads((comparison_dir / "PRINTED-REMAINING-FIRST-CONTRACTS.json").read_bytes())
remaining_output = [json.loads(line) for line in (root / "recovered/queue-command-outputs/RECORDS.jsonl").read_text(encoding="utf-8").splitlines() if '"transcript_ordinal":73300' in line]
if remaining_first["output_ordinal"] != 73300 or len(remaining_first["records"]) != 22 or len(remaining_output) != 1:
    raise ValueError("Remaining comparison first-contract inventory mismatch")
remaining_lines = remaining_output[0]["output_text"].splitlines()
for row, line in zip(remaining_first["records"], lines[51:]):
    fields = line.split()
    if fields[0] != "queue" or int(fields[1]) != row["entry_index"] or fields[2] != row["contract_id"]:
        raise ValueError("Remaining comparison candidate mismatch")
    prefix = f"{row['entry_index']} {row['contract_id']} "
    matches = [item[len(prefix):] for item in remaining_lines if item.startswith(prefix)]
    if len(matches) != 1 or json.loads(matches[0]) != row["first_contract"]:
        raise ValueError("Remaining comparison printed first-contract mismatch")

for recorded, line in zip(first_contracts["queue"], lines[6:51]):
    fields = line.split()
    if int(fields[1]) != recorded["entry_index"] or fields[2] != recorded["contract_id"]:
        raise ValueError("Canonical comparison queue candidate mismatch")
comparison_source_outputs = {}
for source_line in (root / "recovered/queue-command-outputs/RECORDS.jsonl").read_text(encoding="utf-8").splitlines():
    if any(f'"transcript_ordinal":{ordinal}' in source_line for ordinal in (73286, 73293)):
        source_item = json.loads(source_line)
        if source_item["transcript_ordinal"] in (73286, 73293):
            comparison_source_outputs[source_item["transcript_ordinal"]] = source_item["output_text"].splitlines()
if set(comparison_source_outputs) != {73286, 73293}:
    raise ValueError("Comparison first-contract source outputs missing")
for index, row in enumerate(first_contracts["queue"]):
    source_lines = comparison_source_outputs[73286 if index < 22 else 73293]
    prefix = f"{row['entry_index']} {row['contract_id']} "
    matches = [line[len(prefix):] for line in source_lines if line.startswith(prefix)]
    if len(matches) != 1:
        raise ValueError("Comparison first-contract source row missing")
    if row["first_contract"] is None:
        if row["entry_index"] != 1879 or matches[0] != "[LONG 2397]":
            raise ValueError("Comparison suppressed first-contract marker mismatch")
    elif json.loads(matches[0]) != row["first_contract"]:
        raise ValueError("Comparison first-contract printed value mismatch")

comparison_projection = json.loads((comparison_dir / "RECONSTRUCTED-QUEUE-COMPARISON-PROJECTION.json").read_bytes())
if (comparison_projection["schema"] != "726-l2-comparison-queue-projection-v1"
        or comparison_projection["source_write_ordinal"] != 73385
        or comparison_projection["queue"] != "workflow-20261030/CLOSURE-QUEUE-828.json"
        or len(comparison_projection["rows"]) != 67):
    raise ValueError("Partial comparison projection inventory mismatch")
comparison_loop = next(item for item in comparison_ast.body if isinstance(item, ast.For)
                       and isinstance(item.target, ast.Name) and item.target.id == "c")
queue_branch = comparison_loop.body[0].orelse[-1]
range_branch, mixed_branch = queue_branch.orelse[0], queue_branch.orelse[0].orelse[0]
if (ast.unparse(queue_branch.test) != "n in special"
        or ast.unparse(range_branch.test) != "2706 <= n <= 2717"
        or ast.unparse(mixed_branch.test) != "n in (1864, 1869, 1878, 1879, 1895)"):
    raise ValueError("Comparison projection writer conditions mismatch")
range_pair = ast.literal_eval(range_branch.body[0].value)
mixed_pair = ast.literal_eval(mixed_branch.body[0].value)
default_pair = ast.literal_eval(mixed_branch.orelse[0].value)
projection_by_index = {item["queue_index"]: item["projection"] for item in queue_projections["records"]}
for saved, candidate_line, observed in zip(comparison_projection["rows"], lines[6:], first_contracts["queue"] + remaining_first["records"]):
    kind, raw_index, contract_id, terms = candidate_line.split()
    index = int(raw_index)
    pair = special_original.get(index, range_pair if 2706 <= index <= 2717 else mixed_pair if index in (1864, 1869, 1878, 1879, 1895) else default_pair)
    expected_first = long_1879["first_contract"] if index == 1879 else observed["first_contract"]
    expected_lens = 12 if index in (991, 1880) else projection_by_index.get(index, {}).get("lens")
    if (kind != "queue" or index != observed["entry_index"] or contract_id != observed["contract_id"]
            or saved != {"queue_index": index, "contract_id": contract_id, "matched_terms": terms.split(","),
                         "recorded_first_contract": expected_first, "recorded_lens_if_directly_observed": expected_lens,
                         "disposition": pair[0], "reason": pair[1], "canonical_alias_applied": False,
                         "other_lens_body_or_verdict_transferred": False}):
        raise ValueError("Partial comparison projected row mismatch")

master_prints = [json.loads(line) for line in (root / "recovered/ledger-command-outputs/OUTPUTS.jsonl").read_text(encoding="utf-8").splitlines() if '"transcript_output_ordinal":73293' in line]
if len(master_prints) != 1 or "INHERITED_HASH " + comparison["inherited_master_sha256"] not in master_prints[0]["output_text"]:
    raise ValueError("Canonical comparison preceding master reference mismatch")
if (len(lines) != 73 or sum(line.startswith("primary ") for line in lines) != 6
        or sum(line.startswith("queue ") for line in lines) != 67
        or result["comparisons"] != 73 or result["inherited_records_compared"] != 6
        or result["M40_locations_covered"] != 5 or result["global_alias_complete"] is not False
        or len(master["findings"]) != 173 or master["findings"][55]["id"] != "M40"
        or literals["canonical_repair_mapping"]["canonical_alias_applied_to_whole_parent"] is not False
        or literals["implementation_gate_complete"] is not False):
    raise ValueError("Canonical comparison evidence scope mismatch")

nested_source_dir = root / "recovered/l13-nested-source-evidence"
nested_source = json.loads((nested_source_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (nested_source["schema"] != "726-l13-nested-source-evidence-v1"
        or nested_source["original_registry_sha256"] != "d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f"
        or [x["original"] for x in nested_source["rows"]] != list(range(2385, 2391))):
    raise ValueError("Nested original source inventory mismatch")
nested_source_rows = {x["original"]: x for x in nested_source["rows"]}
for row in nested_source["rows"]:
    data = (nested_source_dir / "objects" / row["sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]
            or len(data.decode("utf-8").splitlines()) != row["actual_eof"]
            or row["source_id"] != "pr-working:" + row["git_path"]):
        raise ValueError("Nested original source object mismatch")
inherited_groups = nested_source["inherited_proof"]["groups"]
if [group["group"] for group in inherited_groups] != ["017", "018"]:
    raise ValueError("Nested inherited proof inventory mismatch")
for group in inherited_groups:
    objects = {}
    for ref in group["files"]:
        data = (nested_source_dir / "inherited-objects" / ref["sha256"]).read_bytes()
        if hashlib.sha256(data).hexdigest() != ref["sha256"]:
            raise ValueError("Nested inherited proof object mismatch")
        objects[ref["role"]] = (ref, json.loads(data))
    if set(objects) != {"root", "receipt", "verification", "registration"}:
        raise ValueError("Incomplete nested inherited proof")
    root_record, receipt, verification, registration = (objects[x][1] for x in ("root", "receipt", "verification", "registration"))
    if (root_record["delegated_receipt_sha256"] != objects["receipt"][0]["sha256"]
            or root_record["delegated_verification_sha256"] != objects["verification"][0]["sha256"]
            or receipt["registration"]["sha256"] != objects["registration"][0]["sha256"]
            or root_record["evidence"] != receipt or root_record["lens"] != 13
            or verification["product_execution"] is not False or verification["current_reconciled"] is not False):
        raise ValueError("Nested inherited proof dependency mismatch")
    for mapping in group["reuse_mappings"]:
        index = mapping["receipt_row"]
        prior = receipt["rows"][index]
        later = nested_source_rows[mapping["later_original"]]
        if (prior["original"] != mapping["previous_original"] or prior["sha256"] != later["sha256"]
                or prior["actual_eof"] != later["actual_eof"]
                or prior["prospective_ranges"] != [[1, later["actual_eof"]]]
                or prior["fresh_personal_assessed_ranges"] != prior["prospective_ranges"]
                or prior["untruncated_body_outputs"] is not True
                or any(prior[key] != value for key, value in registration["sources"][index].items())
                or verification["checks"][index] != {"original": mapping["previous_original"], "exact_hash_eof_prospective_scope": True, "ranges": [[1, later["actual_eof"]]], "ordered_credit": 1}):
            raise ValueError("Nested same-lens source reuse mismatch")

nested_display_count = 0
for fragment in nested_source["display_fragments"]:
    data = (nested_source_dir / "display-fragments" / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Nested source display hash mismatch")
    row = nested_source_rows[fragment["original"]]
    source_lines = (nested_source_dir / "objects" / row["sha256"]).read_text(encoding="utf-8").splitlines()
    expected = [f"{line}: {source_lines[line - 1]}" for low, high in fragment["ranges"] for line in range(low, high + 1)]
    if data.decode("utf-8").splitlines() != expected or len(expected) != fragment["lines"]:
        raise ValueError("Nested source display mismatch")
    nested_display_count += len(expected)
if nested_display_count != 713 or nested_source["historical_reused_source_lines"] != 2932:
    raise ValueError("Nested historical attribution count mismatch")

native_candidate_dir = root / "recovered/l2-native-state-candidate"
native_candidate = json.loads((native_candidate_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (native_candidate["schema"] != "726-l2-native-state-candidate-v1"
        or native_candidate["target_state_sha256"] != l2_refs["references"][3]["sha256"]
        or native_candidate["predecessor_sha256"] != l2_025["target_sha256"]):
    raise ValueError("Invalid native state candidate inventory")
for item in native_candidate["files"]:
    if hashlib.sha256((native_candidate_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Native state candidate template mismatch")
native_read_template = json.loads((native_candidate_dir / "READ-TEMPLATE.json").read_bytes())
native_policy_template = json.loads((native_candidate_dir / "POLICY-TEMPLATE.json").read_bytes())
native_state_template = json.loads((native_candidate_dir / "STATE-TEMPLATE.json").read_bytes())
native_view_template = native_state_template["native_identity_admission_source_disposition"]
if (native_state_template["predecessor"]["sha256"] != l2_025["target_sha256"]
        or native_state_template["current_receipt"]["sha256"] != "READHASH"
        or native_view_template["policy"]["sha256"] != "POLICYHASH"
        or native_view_template["exact_source_identities_and_assessed_ranges"] != native_read_template["rows"]
        or len(native_read_template["rows"]) != 8 or native_read_template["uncredited_excerpt_lines"] != 850
        or native_read_template["new_body_credit"] != 0 or native_policy_template["implementation_gate_complete"] is not False):
    raise ValueError("Native state candidate scope mismatch")
native_known_registrations = {item["sha256"] for item in native["sdk_registration_candidates"]}
native_known_registrations.update(item["registration"]["sha256"] for item in native["printed_git_registrations"])
for row in native_read_template["rows"]:
    if (row["registration"]["sha256"] not in native_known_registrations
            or hashlib.sha256((native_dir / "objects" / row["source"]["source_sha256"]).read_bytes()).hexdigest() != row["source"]["source_sha256"]
            or row["new_body_credit"] != 0):
        raise ValueError("Native state candidate source mismatch")

native_original_state = (native_candidate_dir / native_candidate["target_state_sha256"]).read_bytes()
native_original_read = (native_candidate_dir / native_candidate["read_sha256"]).read_bytes()
native_original_policy = (native_candidate_dir / native_candidate["policy_sha256"]).read_bytes()
native_stamp = native_candidate["matched_timestamp"].encode("utf-8")
native_read_hash = native_candidate["read_sha256"].encode("ascii")
native_policy_hash = native_candidate["policy_sha256"].encode("ascii")
native_templates = {name: (native_candidate_dir / (name.upper() + "-TEMPLATE.json")).read_bytes() for name in ("state", "read", "policy")}
if (native_candidate["hash_matched"] is not True
        or hashlib.sha256(native_original_state).hexdigest() != l2_refs["references"][3]["sha256"]
        or hashlib.sha256(native_original_read).hexdigest() != native_candidate["read_sha256"]
        or hashlib.sha256(native_original_policy).hexdigest() != native_candidate["policy_sha256"]
        or native_templates["state"].replace(b"STAMP", native_stamp).replace(b"READHASH", native_read_hash).replace(b"POLICYHASH", native_policy_hash) != native_original_state
        or native_templates["read"].replace(b"STAMP", native_stamp) != native_original_read
        or native_templates["policy"].replace(b"STAMP", native_stamp).replace(b"READHASH", native_read_hash) != native_original_policy):
    raise ValueError("Native state 026 original hash mismatch")
native_state_value = json.loads(native_original_state)
native_read_value = json.loads(native_original_read)
if (native_state_value["predecessor"]["sha256"] != l2_025["target_sha256"]
        or native_state_value["current_receipt"]["sha256"] != native_candidate["read_sha256"]
        or native_state_value["native_identity_admission_source_disposition"]["policy"]["sha256"] != native_candidate["policy_sha256"]):
    raise ValueError("Native state 026 original dependency mismatch")
sdk_002 = (native_dir / "REGISTRATION-ROOT-L2-SATS-002.json").read_bytes()
sdk_003 = (native_dir / "REGISTRATION-ROOT-L2-SATS-003.json").read_bytes()
sdk_rejected = (native_dir / "REGISTRATION-ROOT-L2-SATS-001-REJECTED.json").read_bytes()
if (native["sdk_registration_original_bytes_verified"] is not True
        or any(row["registration"]["sha256"] != hashlib.sha256(sdk_002).hexdigest() for row in native_read_value["rows"][:2])
        or native_read_value["rows"][2]["registration"]["sha256"] != hashlib.sha256(sdk_003).hexdigest()
        or json.loads(sdk_002)["predecessor_rejected"]["sha256"] != hashlib.sha256(sdk_rejected).hexdigest()):
    raise ValueError("Native original SDK registration dependency mismatch")

contract_candidate_dir = root / "recovered/l2-contract-state-candidate"
contract_candidate = json.loads((contract_candidate_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (contract_candidate["schema"] != "726-l2-contract-state-candidate-v1"
        or contract_candidate["target_state_sha256"] != l2_refs["references"][4]["sha256"]
        or contract_candidate["required_predecessor_sha256"] != l2_refs["references"][3]["sha256"]):
    raise ValueError("Invalid contract state candidate inventory")
for item in contract_candidate["files"]:
    if hashlib.sha256((contract_candidate_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Contract state candidate template mismatch")
contract_read_template = json.loads((contract_candidate_dir / "READ-TEMPLATE.json").read_bytes())
contract_policy_template = json.loads((contract_candidate_dir / "POLICY-TEMPLATE.json").read_bytes())
contract_view_template = json.loads((contract_candidate_dir / "VIEW-TEMPLATE.json").read_bytes())
contract_state_template = json.loads((contract_candidate_dir / "STATE-TEMPLATE.json").read_bytes())
if (contract_state_template["predecessor"]["sha256"] != l2_refs["references"][3]["sha256"]
        or contract_state_template["current_receipt"]["sha256"] != "READHASH"
        or contract_state_template["contract_identity_source_disposition"] != contract_view_template):
    raise ValueError("Contract state candidate predecessor mismatch")
if (len(contract_read_template["rows"]) != 7 or contract_read_template["uncredited_excerpt_lines"] != 928
        or contract_read_template["new_body_credit"] != 0
        or contract_view_template["exact_source_identities_and_assessed_ranges"] != contract_read_template["rows"]
        or contract_view_template["source_evidence"]["sha256"] != "READHASH"
        or contract_view_template["policy"]["sha256"] != "POLICYHASH"
        or contract_policy_template["implementation_gate_complete"] is not False
        or [row["entry_index"] for row in contract_policy_template["existing_canonical_relationships"]] != [204, 3720]):
    raise ValueError("Contract state candidate scope mismatch")
queue_3720 = next(item for item in remaining_first["records"] if item["entry_index"] == 3720)
contract_3720 = next(item for item in contract_policy_template["existing_canonical_relationships"] if item["entry_index"] == 3720)
if contract_3720["contract_id"] != queue_3720["contract_id"] or contract_3720["first_contract"] != queue_3720["first_contract"]:
    raise ValueError("Contract candidate later queue predicate mismatch")
contract_known_registrations = {item["registration"]["sha256"] for item in contract_regs["entries"]}
for row in contract_read_template["rows"]:
    if (row["registration"]["sha256"] not in contract_known_registrations
            or hashlib.sha256((contract_dir / "objects" / row["source"]["source_sha256"]).read_bytes()).hexdigest() != row["source"]["source_sha256"]
            or row["new_body_credit"] != 0):
        raise ValueError("Contract state candidate source mismatch")

contract_original_state = (contract_candidate_dir / contract_candidate["target_state_sha256"]).read_bytes()
contract_original_read = (contract_candidate_dir / contract_candidate["read_sha256"]).read_bytes()
contract_original_policy = (contract_candidate_dir / contract_candidate["policy_sha256"]).read_bytes()
contract_stamp = contract_candidate["matched_timestamp"].encode("utf-8")
contract_read_hash = contract_candidate["read_sha256"].encode("ascii")
contract_policy_hash = contract_candidate["policy_sha256"].encode("ascii")
contract_templates = {name: (contract_candidate_dir / (name.upper() + "-TEMPLATE.json")).read_bytes() for name in ("state", "read", "policy")}
if (contract_candidate["hash_matched"] is not True
        or hashlib.sha256(contract_original_state).hexdigest() != l2_refs["references"][4]["sha256"]
        or hashlib.sha256(contract_original_read).hexdigest() != contract_candidate["read_sha256"]
        or hashlib.sha256(contract_original_policy).hexdigest() != contract_candidate["policy_sha256"]
        or contract_templates["state"].replace(b"STAMP", contract_stamp).replace(b"READHASH", contract_read_hash).replace(b"POLICYHASH", contract_policy_hash) != contract_original_state
        or contract_templates["read"].replace(b"STAMP", contract_stamp) != contract_original_read
        or contract_templates["policy"].replace(b"STAMP", contract_stamp).replace(b"READHASH", contract_read_hash) != contract_original_policy):
    raise ValueError("Contract state 027 original hash mismatch")
contract_state_value = json.loads(contract_original_state)
contract_policy_value = json.loads(contract_original_policy)
if (contract_state_value["predecessor"]["sha256"] != native_candidate["target_state_sha256"]
        or contract_state_value["current_receipt"]["sha256"] != contract_candidate["read_sha256"]
        or contract_state_value["contract_identity_source_disposition"]["policy"]["sha256"] != contract_candidate["policy_sha256"]
        or next(item for item in contract_policy_value["existing_canonical_relationships"] if item["entry_index"] == 3720)["first_contract"] != queue_3720["first_contract"]):
    raise ValueError("Contract state 027 dependency mismatch")

reconciliation_dir = root / "recovered/l2-reconciliation-state-candidate"
reconciliation = json.loads((reconciliation_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if (reconciliation["schema"] != "726-l2-reconciliation-state-candidate-v1"
        or reconciliation["target_state_sha256"] != l2_refs["references"][5]["sha256"]
        or reconciliation["required_predecessor_sha256"] != l2_refs["references"][4]["sha256"]):
    raise ValueError("Invalid reconciliation candidate inventory")
for item in reconciliation["files"]:
    if hashlib.sha256((reconciliation_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Reconciliation candidate template mismatch")
reconciliation_read = json.loads((reconciliation_dir / "READ-TEMPLATE.json").read_bytes())
reconciliation_policy = json.loads((reconciliation_dir / "POLICY-TEMPLATE.json").read_bytes())
reconciliation_view = json.loads((reconciliation_dir / "VIEW-TEMPLATE.json").read_bytes())
reconciliation_state = json.loads((reconciliation_dir / "STATE-TEMPLATE.json").read_bytes())
if (reconciliation_state["predecessor"]["sha256"] != contract_candidate["target_state_sha256"]
        or reconciliation_state["current_receipt"]["sha256"] != "READHASH"
        or reconciliation_state["errantry_canonical_reconciliation"] != reconciliation_view):
    raise ValueError("Reconciliation candidate predecessor mismatch")
proof = json.loads((root / "recovered/l2-errantry-reuse-qualification/c8ac419fd0fe6b38f12eeb8f2338f54331c2589ef46b7f13aa09e31c04904299").read_bytes())
if (reconciliation_read["comparison"]["sha256"] != "COMPARISONHASH"
        or reconciliation_view["comparison"]["sha256"] != "COMPARISONHASH"
        or reconciliation_view["source_evidence"]["sha256"] != "READHASH"
        or reconciliation_view["exact_source_identities_and_assessed_ranges"] != proof["rows"]
        or reconciliation_policy["canonical_repair_mapping"] != literals["canonical_repair_mapping"]
        or reconciliation_policy["remaining_evidence"] != literals["remaining"]
        or reconciliation_view["remaining_evidence"] != literals["remaining"]
        or reconciliation_read["new_body_credit"] != 0
        or reconciliation_view["canonical_alias_applied"] is not False
        or reconciliation_view["implementation_gate_complete"] is not False):
    raise ValueError("Reconciliation candidate scope mismatch")

continuity_dir = root / "recovered/l2-publication-continuity-evidence"
continuity = json.loads((continuity_dir / "MANIFEST.json").read_text(encoding="utf-8"))
if continuity["schema"] != "726-l2-publication-continuity-evidence-v1":
    raise ValueError("Invalid publication continuity inventory")
continuity_files = {}
for item in continuity["files"]:
    data = (continuity_dir / item["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["sha256"]:
        raise ValueError("Publication continuity object mismatch")
    continuity_files[item["role"]] = data
if (hashlib.sha256(continuity_files["prior-kernel"]).hexdigest() != continuity["prior_kernel_printed_sha256"]
        or continuity["prior_kernel_printed_sha256"] != "ee09ffccaa023892744d453ec1983d2b80499b7dedc9fcaa62da3acf3943d9ae"):
    raise ValueError("Publication kernel base reference mismatch")
continuity_commands = [json.loads(line) for line in (root / "recovered/audit-write-command-candidates/COMMANDS.jsonl").read_text(encoding="utf-8").splitlines() if '"transcript_command_ordinal":72859' in line]
if len(continuity_commands) != 1 or continuity_commands[0]["recorded_exit_code"] != 0:
    raise ValueError("Publication kernel derivation command missing")
continuity_command = continuity_commands[0]["command_text"]
continuity_ast = ast.parse(continuity_command.split("@'\n", 1)[1].split("\n'@", 1)[0])
replacement_literals = {target.id: ast.literal_eval(item.value) for item in continuity_ast.body if isinstance(item, ast.Assign) for target in item.targets if isinstance(target, ast.Name) and target.id in ("needle", "replacement")}
prior_kernel = continuity_files["prior-kernel"].decode("utf-8")
if (set(replacement_literals) != {"needle", "replacement"} or prior_kernel.count(replacement_literals["needle"]) != 1
        or prior_kernel.replace(replacement_literals["needle"], replacement_literals["replacement"]).encode("utf-8") != continuity_files["derived-kernel"]
        or b'assert all(a==b for i,(a,b) in enumerate(zip(queue["entries"],queue_old["entries"])) if i!=entry_index)' not in continuity_files["derived-kernel"]
        or b"entry_index=2342" not in continuity_files["contract-writer-candidate"]
        or b"queue_number=828" not in continuity_files["contract-writer-candidate"]):
    raise ValueError("Publication queue preservation derivation mismatch")
wrapper_evidence = continuity["wrapper_evidence"]
workflow = json.loads((root / "recovered/transcript-hash-matched/343ba811d165--REPORTING-WORKFLOW.json").read_bytes())
if (workflow["new_l2_publication_backend"]["sha256"] != wrapper_evidence["original_sha256"]
        or hashlib.sha256(continuity_files["wrapper-original"]).hexdigest() != wrapper_evidence["original_sha256"]
        or hashlib.sha256(continuity_files["wrapper-patch"]).hexdigest() != wrapper_evidence["patch_sha256"]):
    raise ValueError("Publication wrapper base reference mismatch")
old_lines = continuity_files["wrapper-original"].decode("utf-8").splitlines()
patch_lines = continuity_files["wrapper-patch"].decode("utf-8").splitlines()
patched_lines = []
cursor = patch_index = 0
while patch_index < len(patch_lines):
    header = re.match(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", patch_lines[patch_index])
    if header is None:
        raise ValueError("Invalid publication wrapper patch header")
    position = int(header.group(1)) - 1
    if not cursor <= position <= len(old_lines):
        raise ValueError("Invalid publication wrapper patch position")
    patched_lines.extend(old_lines[cursor:position])
    cursor = position
    patch_index += 1
    consumed = produced = 0
    while patch_index < len(patch_lines) and not patch_lines[patch_index].startswith("@@ "):
        line = patch_lines[patch_index]
        if not line or line[0] not in " +-":
            raise ValueError("Invalid publication wrapper patch line")
        op, value = line[0], line[1:]
        if op in " -":
            if cursor >= len(old_lines) or old_lines[cursor] != value:
                raise ValueError("Publication wrapper patch context mismatch")
            cursor += 1
            consumed += 1
        if op in " +":
            patched_lines.append(value)
            produced += 1
        patch_index += 1
    if consumed != int(header.group(2) or "1") or produced != int(header.group(4) or "1"):
        raise ValueError("Publication wrapper patch count mismatch")
patched_lines.extend(old_lines[cursor:])
patched_wrapper = ("\n".join(patched_lines) + "\n").encode("utf-8")
expected_wrapper = patched_wrapper.decode("utf-8").replace("parents[4]", "parents[3]").replace("parents[3]", "parents[4]").replace("from audit_l2_publication_v1 import Publication as EvidencePublication", "from audit_l2_publication_kernel_v2 import Publication as EvidencePublication").encode("utf-8")
if (patched_wrapper != continuity_files["wrapper-patched-candidate"]
        or hashlib.sha256(patched_wrapper).hexdigest() != wrapper_evidence["patched_candidate_sha256"]
        or expected_wrapper != continuity_files["wrapper-v3-candidate"]
        or hashlib.sha256(expected_wrapper).hexdigest() != wrapper_evidence["v3_candidate_sha256"]
        or b"super().publish(**kwargs)" not in expected_wrapper):
    raise ValueError("Publication wrapper candidate derivation mismatch")

continuity_status = json.loads(continuity_files["printed-status"])
if ([item["ordinal"] for item in continuity_status["records"]] != [73190, 73207]
        or [item["printed"]["mode"] for item in continuity_status["records"]] != ["--apply", "--verify"]
        or any(item["printed"]["status"] != "pass" or item["printed"]["whole_canonical_cases_closed"] != 0 for item in continuity_status["records"])):
    raise ValueError("Publication continuity status mismatch")

l12_final_dir = root / "recovered/l12-final-registration-evidence"
l12_final = json.loads(_qualified_historical_manifest("recovered/l12-final-registration-evidence/MANIFEST.json"))
if (l12_final["schema"] != "726-l12-final-registration-evidence-v1"
        or l12_final["original_registry_sha256"] != "d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f"
        or [item["original_index"] for item in l12_final["files"]] != list(range(2787, 2794))):
    raise ValueError("Invalid Lens 12 final registration inventory")
l12_prefix = (root / "recovered/l12-selected-state-transition-trace/STATE-960-PREFIX.txt").read_text(encoding="utf-8")
for item in l12_final["files"]:
    registration = (l12_final_dir / "objects" / item["sha256"]).read_bytes()
    template = (l12_final_dir / "templates" / item["sha256"]).read_bytes()
    if (hashlib.sha256(registration).hexdigest() != item["sha256"]
            or hashlib.sha256(template).hexdigest() != item["template_sha256"]
            or template.replace(b"STAMP_MARKER", item["timestamp"].encode("utf-8")) != registration):
        raise ValueError("Lens 12 final registration original hash mismatch")
    expected_path = f"adler-continuation-20261030/REGISTRATION-ROOT-L12-{item['registration_number']}-ORIGINAL{item['original_index']}.json"
    if expected_path not in l12_prefix or item["sha256"] not in l12_prefix:
        raise ValueError("Lens 12 final registration printed reference missing")
    value = json.loads(registration)
    source = (l12_final_dir / "sources" / item["source_sha256"]).read_bytes()
    if (value["original_index"] != item["original_index"] or value["registered_at"] != item["timestamp"]
            or value["source_sha256"] != item["source_sha256"] or value["original_registry_sha256"] != l12_final["original_registry_sha256"]
            or value["body_credit"] != 0 or value["product_execution"] is not False
            or hashlib.sha256(source).hexdigest() != item["source_sha256"]
            or len(source.decode("utf-8").splitlines()) != value["actual_EOF"]
            or any(not 1 <= low <= high <= value["actual_EOF"] for low, high in value["ranges"])):
        raise ValueError("Lens 12 final registration source mismatch")

l12_registry_rows_data = (l12_final_dir / l12_final["batch_registry_sources"]["file"]).read_bytes()
l12_registry_rows = json.loads(l12_registry_rows_data)
if (hashlib.sha256(l12_registry_rows_data).hexdigest() != l12_final["batch_registry_sources"]["sha256"]
        or l12_registry_rows["registry_sha256"] != l12_final["original_registry_sha256"]
        or [row["original_index"] for row in l12_registry_rows["rows"]] != list(range(2787, 2853))
        or len({row["source_sha256"] for row in l12_registry_rows["rows"]}) != 62):
    raise ValueError("Lens 12 final batch registry inventory mismatch")
l12_material_lines = 0
for row in l12_registry_rows["rows"]:
    source = (l12_final_dir / "sources" / row["source_sha256"]).read_bytes()
    if (hashlib.sha256(source).hexdigest() != row["source_sha256"] or len(source) != row["bytes"]
            or len(source.decode("utf-8").splitlines()) != row["actual_EOF"]
            or row["registry_pointer"] != f"/files/{row['original_index']}"
            or any(not 1 <= low <= high <= row["actual_EOF"] for low, high in row["material_ranges"])):
        raise ValueError("Lens 12 final batch source mismatch")
    l12_material_lines += len({line for low, high in row["material_ranges"] for line in range(low, high + 1)})
if l12_material_lines != 6784 or l12_registry_rows["material_role_lines"] != 6784:
    raise ValueError("Lens 12 final batch material-scope count mismatch")
for item in l12_final["files"]:
    row = l12_registry_rows["rows"][item["original_index"] - 2787]
    value = json.loads((l12_final_dir / "objects" / item["sha256"]).read_bytes())
    if row["source_sha256"] != item["source_sha256"] or row["source_id"] != value["source_id"] or row["material_ranges"] != value["ranges"]:
        raise ValueError("Lens 12 final registration registry link mismatch")

l12_batch_refs_data = (l12_final_dir / l12_final["batch_references"]["file"]).read_bytes()
l12_batch_refs = json.loads(l12_batch_refs_data)
l12_helper = (l12_final_dir / l12_final["qualification_helper"]["file"]).read_bytes()
if (hashlib.sha256(l12_batch_refs_data).hexdigest() != l12_final["batch_references"]["sha256"]
        or hashlib.sha256(l12_helper).hexdigest() != l12_final["qualification_helper"]["sha256"]
        or l12_batch_refs["helper"]["sha256"] != l12_final["qualification_helper"]["sha256"]
        or l12_batch_refs["fresh"] != 5795 or l12_batch_refs["historical"] != 766 or l12_batch_refs["same_batch"] != 223):
    raise ValueError("Lens 12 batch printed references mismatch")
ast.parse(l12_helper.decode("utf-8"))
l12_recorded_outputs = {}
for line in (root / "recovered/source-registration-outputs/RECORDS-02.jsonl").read_text(encoding="utf-8").splitlines():
    if any(f'"transcript_ordinal":{number}' in line for number in (56926, 56935, 56936, 56985)):
        item = json.loads(line)
        l12_recorded_outputs[item["transcript_ordinal"]] = item["output_text"]
if set(l12_recorded_outputs) != {56926, 56935, 56936, 56985} or l12_final["qualification_helper"]["sha256"] not in l12_recorded_outputs[56926]:
    raise ValueError("Lens 12 historical batch output missing")
l12_printed_rows_data = (l12_final_dir / l12_final["printed_batch_source_rows"]["file"]).read_bytes()
l12_printed_rows = json.loads(l12_printed_rows_data)
if (hashlib.sha256(l12_printed_rows_data).hexdigest() != l12_final["printed_batch_source_rows"]["sha256"]
        or l12_printed_rows["output_ordinal"] != 56985 or len(l12_printed_rows["rows"]) != 4):
    raise ValueError("Lens 12 batch source projection inventory mismatch")
l12_original_printed = []
for line in l12_recorded_outputs[56985].splitlines():
    if line.startswith("{"):
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if value.get("original_index") in (2790, 2791, 2792, 2793):
            l12_original_printed.append(value)
if l12_printed_rows["rows"] != l12_original_printed:
    raise ValueError("Lens 12 batch source projection output mismatch")
for row in l12_printed_rows["rows"]:
    item = next(x for x in l12_final["files"] if x["original_index"] == row["original_index"])
    registration = json.loads((l12_final_dir / "objects" / item["sha256"]).read_bytes())
    if (row["legacy_registration"]["sha256"] != item["sha256"] or row["source_sha256"] != item["source_sha256"]
            or row["assessed_ranges"] != registration["ranges"] or row["fresh_personal_body_reads"] != registration["ranges"]):
        raise ValueError("Lens 12 batch source projection dependency mismatch")

l12_display_count = 0
for fragment in l12_final["source_displays"]["fragments"]:
    data = (l12_final_dir / "display-fragments" / fragment["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != fragment["sha256"]:
        raise ValueError("Lens 12 final source display hash mismatch")
    item = next(row for row in l12_final["files"] if row["original_index"] == fragment["original_index"])
    registration = json.loads((l12_final_dir / "objects" / item["sha256"]).read_bytes())
    source_lines = (l12_final_dir / "sources" / item["source_sha256"]).read_text(encoding="utf-8").splitlines()
    expected = [f"{line}: {source_lines[line - 1]}" for low, high in registration["ranges"] for line in range(low, high + 1)]
    recorded = l12_recorded_outputs[fragment["ordinal"]].splitlines()
    header = next((i for i, line in enumerate(recorded) if line.startswith(f"LENS12FRESH {item['original_index']} ")), None)
    if (header is None or data.decode("utf-8").splitlines() != expected
            or recorded[header + 1:header + 1 + len(expected)] != expected or len(expected) != fragment["lines"]):
        raise ValueError("Lens 12 historical source display mismatch")
    l12_display_count += len(expected)
if l12_display_count != 611 or l12_display_count != l12_final["source_displays"]["historical_display_lines"]:
    raise ValueError("Lens 12 historical source display count mismatch")

l12_batch_display = l12_final["batch_source_displays"]
if (len(l12_batch_display["fragments"]) != 66 or l12_batch_display["roles"] != 52
        or l12_batch_display["line_occurrences"] != 6770 or l12_batch_display["unique_role_lines"] != 5795):
    raise ValueError("Lens 12 batch display inventory mismatch")
l12_role_lines = set()
l12_display_roles = set()
l12_occurrences = 0
for fragment in l12_batch_display["fragments"]:
    data = (l12_final_dir / "batch-display-fragments" / fragment["file"]).read_bytes()
    row = l12_registry_rows["rows"][fragment["original_index"] - 2787]
    if (row["original_index"] != fragment["original_index"] or row["source_sha256"] != fragment["source_sha256"]
            or hashlib.sha256(data).hexdigest() != fragment["sha256"]
            or not re.fullmatch(r"[0-9a-f]{64}", fragment["recorded_output_sha256"])
            or fragment["original_output_truncation_flag"] is not False):
        raise ValueError("Lens 12 batch display reference mismatch")
    source_lines = (l12_final_dir / "sources" / row["source_sha256"]).read_text(encoding="utf-8").splitlines()
    if (len(fragment["numbers"]) != fragment["lines"]
            or any(not any(low <= number <= high for low, high in row["material_ranges"]) for number in fragment["numbers"])):
        raise ValueError("Lens 12 batch display range mismatch")
    expected = [f"{number}: {source_lines[number - 1]}" for number in fragment["numbers"]]
    if data.decode("utf-8").splitlines() != expected:
        raise ValueError("Lens 12 batch display source mismatch")
    l12_occurrences += len(expected)
    l12_display_roles.add(fragment["original_index"])
    l12_role_lines.update((fragment["original_index"], number) for number in fragment["numbers"])
if (l12_occurrences != 6770 or len(l12_display_roles) != 52 or len(l12_role_lines) != 5795
        or len(l12_role_lines) != l12_batch_refs["fresh"]):
    raise ValueError("Lens 12 batch display count mismatch")

l12_reuse_data = (l12_final_dir / l12_final["batch_reuse_projections"]["file"]).read_bytes()
l12_reuse = json.loads(l12_reuse_data)
if (hashlib.sha256(l12_reuse_data).hexdigest() != l12_final["batch_reuse_projections"]["sha256"]
        or len(l12_reuse["prior_registration_references"]) != 11 or len(l12_reuse["shared_body_projections"]) != 3):
    raise ValueError("Lens 12 final reuse inventory mismatch")
l12_prior_hashes = set()
for item in l12_reuse["prior_registration_references"]:
    sha = item["registration"]["sha256"]
    data = (l12_final_dir / "inherited-registrations" / sha).read_bytes()
    row = l12_registry_rows["rows"][item["original_index"] - 2787]
    if hashlib.sha256(data).hexdigest() != sha or row["source_sha256"].encode("ascii") not in data:
        raise ValueError("Lens 12 prior registration source mismatch")
    l12_prior_hashes.add(sha)
if len(l12_prior_hashes) != 10:
    raise ValueError("Lens 12 prior registration count mismatch")
l12_shared_lines = 0
for item in l12_reuse["shared_body_projections"]:
    row = l12_registry_rows["rows"][item["original_index"] - 2787]
    proof = item["proof"]
    parent = l12_registry_rows["rows"][proof["original_index"] - 2787]
    if (row["source_sha256"] != proof["source_sha256"] or parent["source_sha256"] != proof["source_sha256"]
            or row["material_ranges"] != proof["assessed_ranges"] or proof["new_body_credit"] != 0):
        raise ValueError("Lens 12 same-batch source mismatch")
    for low, high in proof["assessed_ranges"]:
        for number in range(low, high + 1):
            if (proof["original_index"], number) not in l12_role_lines:
                raise ValueError("Lens 12 shared range lacks recorded parent display")
            l12_shared_lines += 1
if l12_shared_lines != 223 or l12_shared_lines != l12_batch_refs["same_batch"]:
    raise ValueError("Lens 12 shared source count mismatch")

l12_prior_proof = l12_final["prior_read_proof"]
if len(l12_prior_proof["groups"]) != 8 or l12_prior_proof["roles"] != 11:
    raise ValueError("Lens 12 prior read proof inventory mismatch")
l12_prior_proven_roles = set()
l12_prior_proven_lines = 0
for group in l12_prior_proof["groups"]:
    receipt_data = (l12_final_dir / "prior-read-objects" / group["receipt"]["sha256"]).read_bytes()
    state_data = (l12_final_dir / "prior-read-objects" / group["state"]["sha256"]).read_bytes()
    if (hashlib.sha256(receipt_data).hexdigest() != group["receipt"]["sha256"]
            or hashlib.sha256(state_data).hexdigest() != group["state"]["sha256"]):
        raise ValueError("Lens 12 prior state/receipt hash mismatch")
    receipt_value, state_value = json.loads(receipt_data), json.loads(state_data)
    if receipt_value["lens"] != 12 or state_value["receipt_sha256"] != group["receipt"]["sha256"]:
        raise ValueError("Lens 12 prior state/receipt link mismatch")
    for mapping in group["reuse_mappings"]:
        row = l12_registry_rows["rows"][mapping["original_index"] - 2787]
        prior = receipt_value["rows"][mapping["receipt_row"]]
        wanted = {line for low, high in mapping["requested_ranges"] for line in range(low, high + 1)}
        observed = {line for low, high in prior["fresh_personal_body_reads"] for line in range(low, high + 1)}
        if (mapping["requested_ranges"] != row["material_ranges"]
                or prior.get("source_sha256", prior.get("sha256")) != row["source_sha256"]
                or mapping["registration_sha256"] not in json.dumps(prior) or not wanted.issubset(observed)):
            raise ValueError("Lens 12 prior receipt source scope mismatch")
        l12_prior_proven_roles.add(mapping["original_index"])
        l12_prior_proven_lines += len(wanted)
armor = l12_prior_proof["armor"]
armor_receipt_data = (l12_final_dir / "prior-read-objects" / armor["receipt"]["sha256"]).read_bytes()
armor_verify_data = (l12_final_dir / "prior-read-objects" / armor["verification"]["sha256"]).read_bytes()
if (hashlib.sha256(armor_receipt_data).hexdigest() != armor["receipt"]["sha256"]
        or hashlib.sha256(armor_verify_data).hexdigest() != armor["verification"]["sha256"]):
    raise ValueError("Lens 12 armor proof hash mismatch")
armor_receipt, armor_verify = json.loads(armor_receipt_data), json.loads(armor_verify_data)
if (armor_verify["receipt_sha256"] != armor["receipt"]["sha256"]
        or armor_verify["product_execution"] is not False or armor_verify["implementation_gate_complete"] is not False):
    raise ValueError("Lens 12 armor verification link mismatch")
for mapping in armor["reuse_mappings"]:
    row = l12_registry_rows["rows"][mapping["original_index"] - 2787]
    prior = armor_receipt["rows"][mapping["receipt_row"]]
    wanted = {line for low, high in mapping["requested_ranges"] for line in range(low, high + 1)}
    observed = {line for low, high in prior["fresh_personal_body_reads"] for line in range(low, high + 1)}
    if (mapping["requested_ranges"] != row["material_ranges"] or prior["source_sha256"] != row["source_sha256"]
            or prior["registration_sha256"] != mapping["registration_sha256"] or not wanted.issubset(observed)):
        raise ValueError("Lens 12 armor source scope mismatch")
    l12_prior_proven_roles.add(mapping["original_index"])
    l12_prior_proven_lines += len(wanted)
if len(l12_prior_proven_roles) != 11 or l12_prior_proven_lines != 766 or l12_prior_proven_lines != l12_batch_refs["historical"]:
    raise ValueError("Lens 12 prior source coverage count mismatch")

l12_direct = l12_prior_proof["direct_dependencies"]
if len(l12_direct["references"]) != 72 or l12_direct["unique_objects"] != 65:
    raise ValueError("Lens 12 prior direct dependency inventory mismatch")
l12_direct_hashes = set()
for ref in l12_direct["references"]:
    parent_bytes = (l12_final_dir / "prior-read-objects" / ref["parent_sha256"]).read_bytes()
    if hashlib.sha256(parent_bytes).hexdigest() != ref["parent_sha256"]:
        raise ValueError("Lens 12 prior dependency parent hash mismatch")
    node = json.loads(parent_bytes)
    for segment in ref["pointer"].split("/")[1:]:
        node = node[int(segment)] if isinstance(node, list) else node[segment]
    if node[ref["path_key"]] != ref["path"] or node[ref["hash_key"]] != ref["sha256"]:
        raise ValueError("Lens 12 prior dependency reference mismatch")
    data = (l12_final_dir / "prior-direct-dependencies" / ref["sha256"]).read_bytes()
    if ref["sha256"] == "0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0":
        _qualified_public_object("recovered/l12-final-registration-evidence/prior-direct-dependencies/" + ref["sha256"], ref["sha256"], data)
    elif hashlib.sha256(data).hexdigest() != ref["sha256"]:
        raise ValueError("Lens 12 prior dependency content hash mismatch")
    l12_direct_hashes.add(ref["sha256"])
if len(l12_direct_hashes) != l12_direct["unique_objects"]:
    raise ValueError("Lens 12 prior dependency unique count mismatch")

additional = root / "recovered/additional-owned-repair-records"
additional_manifest = json.loads((additional / "MANIFEST.json").read_text(encoding="utf-8"))
if additional_manifest["schema"] != "726-additional-owned-repair-records-v1" or len(additional_manifest["records"]) != 2:
    raise ValueError("Invalid additional proposal manifest")
for entry in additional_manifest["records"]:
    ordinal = entry["event_ordinal"]
    if ordinal not in (65653, 65892) or entry["record"] != f"{ordinal}.json" or entry["expression"] != f"{ordinal}-expression.txt":
        raise ValueError("Invalid additional proposal path")
    for field in ("record", "expression"):
        if hashlib.sha256((additional / entry[field]).read_bytes()).hexdigest() != entry[field + "_sha256"]:
            raise ValueError("Additional proposal evidence hash mismatch")
    record = json.loads((additional / entry["record"]).read_text(encoding="utf-8"))
    if record["event_ordinal"] != ordinal or record["printed_output"]["repair"] != entry["historical_repair"]:
        raise ValueError("Additional proposal provenance mismatch")
    expression = ast.parse((additional / entry["expression"]).read_text(encoding="utf-8"), mode="eval").body
    if not isinstance(expression, ast.Call) or not isinstance(expression.func, ast.Name) or expression.func.id != "dict":
        raise ValueError("Invalid proposal expression")
    for keyword in expression.keywords:
        try:
            value = ast.literal_eval(keyword.value)
        except (ValueError, TypeError):
            value = {"unresolved_expression": ast.unparse(keyword.value)}
        if record["literal_fields"].get(keyword.arg) != value:
            raise ValueError("Additional proposal literal mismatch")


l13_707_dir = root / "recovered/l13-root-707"
l13_707 = json.loads((l13_707_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "2a85551e707a1c6efc7c66f1eb62e6e00b8c50e8b157f9da7ab108ac3066fe75"), ("helper_sha256", "15e234a97af58609829081d996798339a4e79fe7a4d05148fc0fd098437b6cc4"), ("registration_sha256", "0773593fe4344bccedd08c4264ed1692206055c888fef6b788e3fc315e28c786")]:
    if l13_707[field] != expected or hashlib.sha256((l13_707_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 707 preparation hash mismatch")
l13_707_reg = json.loads((l13_707_dir / l13_707["registration_sha256"]).read_bytes())
if (len(l13_707_reg["sources"]) != 4 or len(l13_707_reg["valid_nested_range_fields"]) != 32
        or l13_707_reg["fresh_lines_registered"] != 30 or l13_707_reg["prior_material_lines_reused"] != 270):
    raise ValueError("Lens 13 root 707 registration scope mismatch")
for item, source in zip(l13_707["source_objects"], l13_707_reg["sources"]):
    data = (l13_707_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 707 source mismatch")
l13_707_display = (l13_707_dir / "fresh-display.txt").read_bytes()
l13_707_lines = (l13_707_dir / l13_707["source_objects"][2]["sha256"]).read_bytes().decode().splitlines()
l13_707_expected = "".join(f"{i}: {l13_707_lines[i-1]}\n" for lo, hi in [(49, 57), (88, 108)] for i in range(lo, hi + 1)).encode()
if hashlib.sha256(l13_707_display).hexdigest() != l13_707["fresh_display_sha256"] or l13_707_display != l13_707_expected:
    raise ValueError("Lens 13 root 707 historical display mismatch")


for field, expected in [("qualification_sha256", "fbf4f4d55b396166523f30bbb94a6980017dfb97e5efe0e205512b1b721eee2c"), ("packet_sha256", "93ffd69e4f72c5d146b5d4ac3d53c2f1d9328791fbeb7dae22909cbcdc9b035b"), ("successor_plan_sha256", "730c169a2db4cc41b9309028e27ec7b6c94d827f1564aa8d94d436e62aa551d0")]:
    if l13_707[field] != expected or hashlib.sha256((l13_707_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 707 historical link mismatch")
for field in ("receipt_sha256", "state_sha256"):
    if hashlib.sha256((l13_707_dir / l13_707[field]).read_bytes()).hexdigest() != l13_707[field]:
        raise ValueError("Lens 13 root 707 adoption artifact mismatch")
l13_707_q = json.loads((l13_707_dir / l13_707["qualification_sha256"]).read_bytes())
l13_707_packet = json.loads((l13_707_dir / l13_707["packet_sha256"]).read_bytes())
l13_707_receipt = json.loads((l13_707_dir / l13_707["receipt_sha256"]).read_bytes())
l13_707_state = json.loads((l13_707_dir / l13_707["state_sha256"]).read_bytes())
l13_708_plan = json.loads((l13_707_dir / l13_707["successor_plan_sha256"]).read_bytes())
if (l13_707_q["registration"]["sha256"] != l13_707["registration_sha256"]
        or l13_707_q["material_lines"] != 300 or l13_707_q["fresh_lines"] != 30
        or l13_707_q["reused_material_lines"] != 270
        or l13_707_packet["new_root_ordered_lines"] != 30
        or l13_707_receipt["evidence"] != l13_707_packet
        or l13_707_state["receipt_sha256"] != l13_707["receipt_sha256"]
        or l13_708_plan["previous_state"]["sha256"] != l13_707["state_sha256"]):
    raise ValueError("Lens 13 root 707 qualification or adoption chain mismatch")
if len(l13_707["prior_support_files"]) != 11:
    raise ValueError("Lens 13 root 707 prior support count mismatch")
for item in l13_707["prior_support_files"]:
    if hashlib.sha256((l13_707_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"] or item["sha256"] not in json.dumps(l13_707_q):
        raise ValueError("Lens 13 root 707 prior support mismatch")


l13_708_dir = root / "recovered/l13-root-708"
l13_708 = json.loads((l13_708_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "855cf7de434498208f43b4306a27057f4411d8774f2842ed0205618d04cdfb45"), ("helper_sha256", "ebe353e65db333aec9bf21e7cd41ca7f1a748a5a661a09e8ffc8c01e398cff4f"), ("registration_sha256", "7285d9ebc2cccb0ac748fae28a17bea518b965bc449e5585bd12027f0f696826"), ("qualification_sha256", "5bd36e599a8c2b5744f6e4aa52ae368494f9b74c46853530656959ecf03f34e3"), ("packet_sha256", "a5773242d8b7727727f8a7f3d6db93a4658894a3fd2de3e78a93fd2e22e95c4a"), ("plan_sha256", "730c169a2db4cc41b9309028e27ec7b6c94d827f1564aa8d94d436e62aa551d0")]:
    if l13_708[field] != expected or hashlib.sha256((l13_708_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 708 historical artifact mismatch")
l13_708_q = json.loads((l13_708_dir / l13_708["qualification_sha256"]).read_bytes())
l13_708_plan = json.loads((l13_708_dir / l13_708["plan_sha256"]).read_bytes())
if (len(l13_708_q["entries"]) != 7 or l13_708_q["fresh_lines"] != 913
        or l13_708_q["reused_material_lines"] != 0 or len(l13_708_q["all_nested_range_fields"]) != 35
        or l13_708_plan["packet"]["sha256"] != l13_708["packet_sha256"]
        or l13_708_plan["previous_state"]["sha256"] != l13_707["state_sha256"]):
    raise ValueError("Lens 13 root 708 scope or chain mismatch")
l13_708_display = []
for item, entry in zip(l13_708["source_objects"], l13_708_q["entries"]):
    data = (l13_708_dir / item["sha256"]).read_bytes()
    if (item["original"] != entry["original"] or item["sha256"] != entry["source"]["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 708 source mismatch")
    lines = data.decode().splitlines()
    l13_708_display.append("SOURCE " + str(item["original"]))
    l13_708_display.extend(f"{i}: {lines[i-1]}" for lo, hi in entry["assessed_union"] for i in range(lo, hi + 1))
l13_708_display_bytes = (l13_708_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(l13_708_display_bytes).hexdigest() != l13_708["fresh_display_sha256"]
        or l13_708_display_bytes != ("\n".join(l13_708_display) + "\n").encode()):
    raise ValueError("Lens 13 root 708 display mismatch")


for field in ("receipt_sha256", "state_sha256", "successor_plan_sha256"):
    if hashlib.sha256((l13_708_dir / l13_708[field]).read_bytes()).hexdigest() != l13_708[field]:
        raise ValueError("Lens 13 root 708 adoption artifact mismatch")
l13_708_receipt = json.loads((l13_708_dir / l13_708["receipt_sha256"]).read_bytes())
l13_708_state = json.loads((l13_708_dir / l13_708["state_sha256"]).read_bytes())
l13_709_plan = json.loads((l13_708_dir / l13_708["successor_plan_sha256"]).read_bytes())
if (l13_708["successor_plan_sha256"] != "537fc7d925ff1da7eca19f662490a9043172331322b13a1adf4c1764c448f6df"
        or l13_708_receipt["packet"]["sha256"] != l13_708["packet_sha256"]
        or l13_708_receipt["evidence"] != json.loads((l13_708_dir / l13_708["packet_sha256"]).read_bytes())
        or l13_708_state["receipt_sha256"] != l13_708["receipt_sha256"]
        or l13_709_plan["previous_state"]["sha256"] != l13_708["state_sha256"]):
    raise ValueError("Lens 13 root 708 adoption chain mismatch")
l13_709_dir = root / "recovered/l13-root-709"
l13_709 = json.loads((l13_709_dir / "MANIFEST.json").read_bytes())
l13_709_data = (l13_709_dir / l13_709["discovery_sha256"]).read_bytes()
if (l13_709["discovery_sha256"] != "67ece7d660b09449cf9ef941f145de150b5cdd7eab262b38646bc1cf14a79601"
        or hashlib.sha256(l13_709_data).hexdigest() != l13_709["discovery_sha256"]):
    raise ValueError("Lens 13 root 709 discovery hash mismatch")
l13_709_discovery = json.loads(l13_709_data)
if (l13_709_discovery["frontier"] != 2700 or len(l13_709_discovery["entries"]) != 4
        or len(l13_709_discovery["exact_hash_prior_hits"]) != 1 or l13_709_discovery["body_credit"] != 0):
    raise ValueError("Lens 13 root 709 discovery scope mismatch")


for field, expected in [("helper_sha256", "c95b313355cad3d0b32fd252cbfd4c676398c57928a691e06a5d4ebf6ee09b1d"), ("registration_sha256", "64bf72810b2756d217610f9afb276430e6cdd63f91a4ffe649374309476fb067"), ("qualification_sha256", "ea6f6fe8c14b3fc504ab72d40eca1773ccb853631a6f80b6c918a11f4fd250e1"), ("packet_sha256", "ce4576fa8d86810669d1c51808300bac83d86c2d52a888532b295e86dd536942"), ("plan_sha256", "537fc7d925ff1da7eca19f662490a9043172331322b13a1adf4c1764c448f6df")]:
    if l13_709[field] != expected or hashlib.sha256((l13_709_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 709 historical artifact mismatch")
l13_709_q = json.loads((l13_709_dir / l13_709["qualification_sha256"]).read_bytes())
if (len(l13_709_q["entries"]) != 4 or l13_709_q["fresh_lines"] != 340
        or l13_709_q["reused_material_lines"] != 132 or len(l13_709_q["all_nested_range_fields"]) != 16
        or l13_709_plan["packet"]["sha256"] != l13_709["packet_sha256"]):
    raise ValueError("Lens 13 root 709 scope mismatch")
l13_709_display = []
for item, entry in zip(l13_709["source_objects"], l13_709_q["entries"]):
    data = (l13_709_dir / item["sha256"]).read_bytes()
    if (item["original"] != entry["original"] or item["sha256"] != entry["source"]["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 709 source mismatch")
    if entry["mode"] == "fresh":
        lines = data.decode().splitlines()
        l13_709_display.append("SOURCE " + str(item["original"]))
        l13_709_display.extend(f"{i}: {lines[i-1]}" for lo, hi in entry["assessed_union"] for i in range(lo, hi + 1))
l13_709_display_bytes = (l13_709_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(l13_709_display_bytes).hexdigest() != l13_709["fresh_display_sha256"]
        or l13_709_display_bytes != ("\n".join(l13_709_display) + "\n").encode()):
    raise ValueError("Lens 13 root 709 display mismatch")
l13_709_prior = l13_709_q["entries"][2]["prior"]["prior_native_verification"]["sha256"]
if (l13_709_prior != l13_709["prior_verification_sha256"]
        or hashlib.sha256((l13_709_dir / l13_709_prior).read_bytes()).hexdigest() != l13_709_prior):
    raise ValueError("Lens 13 root 709 prior verification mismatch")


for field in ("receipt_sha256", "state_sha256", "successor_plan_sha256"):
    if hashlib.sha256((l13_709_dir / l13_709[field]).read_bytes()).hexdigest() != l13_709[field]:
        raise ValueError("Lens 13 root 709 adoption artifact mismatch")
l13_709_receipt = json.loads((l13_709_dir / l13_709["receipt_sha256"]).read_bytes())
l13_709_state = json.loads((l13_709_dir / l13_709["state_sha256"]).read_bytes())
l13_710_plan = json.loads((l13_709_dir / l13_709["successor_plan_sha256"]).read_bytes())
if (l13_709["successor_plan_sha256"] != "93d80493a87ba9255e44ba8a46a5611a675fbe101d506068a661956d99e6dcc8"
        or l13_709_receipt["packet"]["sha256"] != l13_709["packet_sha256"]
        or l13_709_receipt["evidence"] != json.loads((l13_709_dir / l13_709["packet_sha256"]).read_bytes())
        or l13_709_state["receipt_sha256"] != l13_709["receipt_sha256"]
        or l13_710_plan["previous_state"]["sha256"] != l13_709["state_sha256"]):
    raise ValueError("Lens 13 root 709 adoption chain mismatch")
l13_710_dir = root / "recovered/l13-root-710"
l13_710 = json.loads((l13_710_dir / "MANIFEST.json").read_bytes())
l13_710_data = (l13_710_dir / l13_710["discovery_sha256"]).read_bytes()
if (l13_710["discovery_sha256"] != "1248d65bd69ab571420946071d3d720635284c3857c82b178c4bee12642c1d10"
        or hashlib.sha256(l13_710_data).hexdigest() != l13_710["discovery_sha256"]):
    raise ValueError("Lens 13 root 710 discovery hash mismatch")
l13_710_discovery = json.loads(l13_710_data)
if (l13_710_discovery["frontier"] != 2704 or len(l13_710_discovery["entries"]) != 5
        or len(l13_710_discovery["exact_hash_prior_hits"]) != 3 or l13_710_discovery["body_credit"] != 0):
    raise ValueError("Lens 13 root 710 discovery scope mismatch")


for field, expected in [("helper_sha256", "5c6961cc03b94bcbf2e944dc0a1d668c97a7b98be44fc9ad5104b654c5d54894"), ("registration_sha256", "bc9442e2dcfea25792c199828c8bdc863ce87fa54762c3ebc8063f6b31f10647"), ("plan_sha256", "93d80493a87ba9255e44ba8a46a5611a675fbe101d506068a661956d99e6dcc8")]:
    if l13_710[field] != expected or hashlib.sha256((l13_710_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 710 preparation hash mismatch")
l13_710_reg = json.loads((l13_710_dir / l13_710["registration_sha256"]).read_bytes())
if (len(l13_710_reg["sources"]) != 5 or l13_710_reg["fresh_lines_registered"] != 769
        or l13_710_reg["prior_material_lines_reused"] != 271
        or len(l13_710_reg["valid_nested_range_fields"]) != 33
        or l13_710_reg["sources"][3]["fresh_personal_assessed_ranges"] != [[44, 44]]):
    raise ValueError("Lens 13 root 710 registration scope mismatch")
l13_710_display = []
for item, source in zip(l13_710["source_objects"], l13_710_reg["sources"]):
    data = (l13_710_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 710 source mismatch")
    if item["original"] in (2704, 2707, 2708):
        lines = data.decode().splitlines()
        l13_710_display.append("SOURCE " + str(item["original"]))
        ranges = [[44, 44]] if item["original"] == 2707 else source["prospective_ranges"]
        l13_710_display.extend(f"{i}: {lines[i-1]}" for lo, hi in ranges for i in range(lo, hi + 1))
l13_710_display_bytes = (l13_710_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(l13_710_display_bytes).hexdigest() != l13_710["fresh_display_sha256"]
        or l13_710_display_bytes != ("\n".join(l13_710_display) + "\n").encode()):
    raise ValueError("Lens 13 root 710 display mismatch")
l13_710_rows = (l13_710_dir / "prior-row-extracts.json").read_bytes()
if hashlib.sha256(l13_710_rows).hexdigest() != l13_710["prior_row_extracts_sha256"] or len(json.loads(l13_710_rows)) != 3:
    raise ValueError("Lens 13 root 710 prior row extracts mismatch")


l13_711_dir = root / "recovered/l13-root-711"
l13_711 = json.loads((l13_711_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "283d3f63c0ccf6789e3dde0ab1ddf8300fe194d3e705cda40eab6bcee22b1fb8"), ("helper_sha256", "56fbe7176dc065e87e9aa148ece9bd68a7a699ff86c0d13bb0327d73118382c1"), ("registration_sha256", "c070fe3cf72758c8b3223ee08933ce9e5b1978d6872f199c6e2ca27d6f158dd3")]:
    if l13_711[field] != expected or hashlib.sha256((l13_711_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 711 preparation hash mismatch")
l13_711_reg = json.loads((l13_711_dir / l13_711["registration_sha256"]).read_bytes())
if (len(l13_711_reg["sources"]) != 6 or l13_711_reg["fresh_lines_registered"] != 335
        or l13_711_reg["prior_material_lines_reused"] != 56 or len(l13_711_reg["valid_nested_range_fields"]) != 47):
    raise ValueError("Lens 13 root 711 registration scope mismatch")
l13_711_display = []
for item, source in zip(l13_711["source_objects"], l13_711_reg["sources"]):
    data = (l13_711_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 711 source mismatch")
    if item["original"] in (2709, 2710, 2713, 2714):
        lines = data.decode().splitlines()
        l13_711_display.append("SOURCE " + str(item["original"]))
        l13_711_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
l13_711_display_bytes = (l13_711_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(l13_711_display_bytes).hexdigest() != l13_711["fresh_display_sha256"]
        or l13_711_display_bytes != ("\n".join(l13_711_display) + "\n").encode()):
    raise ValueError("Lens 13 root 711 display mismatch")
l13_711_rows = (l13_711_dir / "prior-row-extracts.json").read_bytes()
if hashlib.sha256(l13_711_rows).hexdigest() != l13_711["prior_row_extracts_sha256"] or len(json.loads(l13_711_rows)) != 2:
    raise ValueError("Lens 13 root 711 prior row extracts mismatch")


l13_712_dir = root / "recovered/l13-root-712"
l13_712 = json.loads((l13_712_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "8e065223bda181a554f19226f80a720bf7ea98addf2a111114b766825626a168"), ("helper_sha256", "1a7552de90e9aea20d2b0f3f7873e9e499b8b27cae8f9593ad7a98ef0bb35dc6"), ("registration_sha256", "41b31dd5c77ea6be97d35b092cb357c81e3f8ba78702cdcee3acd8dfd1a22607")]:
    if l13_712[field] != expected or hashlib.sha256((l13_712_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 712 preparation hash mismatch")
l13_712_reg = json.loads((l13_712_dir / l13_712["registration_sha256"]).read_bytes())
if (len(l13_712_reg["sources"]) != 6 or l13_712_reg["fresh_lines_registered"] != 200
        or l13_712_reg["prior_material_lines_reused"] != 1607 or len(l13_712_reg["valid_nested_range_fields"]) != 48):
    raise ValueError("Lens 13 root 712 registration scope mismatch")
for item, source in zip(l13_712["source_objects"], l13_712_reg["sources"]):
    data = (l13_712_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 712 source mismatch")
source = l13_712_reg["sources"][0]
lines = (l13_712_dir / source["sha256"]).read_bytes().decode().splitlines()
expected_display = "".join(f"{i}: {lines[i-1]}\n" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1)).encode()
display = (l13_712_dir / "fresh-display.txt").read_bytes()
if hashlib.sha256(display).hexdigest() != l13_712["fresh_display_sha256"] or display != expected_display:
    raise ValueError("Lens 13 root 712 display mismatch")
rows = (l13_712_dir / "prior-row-extracts.json").read_bytes()
if hashlib.sha256(rows).hexdigest() != l13_712["prior_row_extracts_sha256"] or len(json.loads(rows)) != 3:
    raise ValueError("Lens 13 root 712 prior row extracts mismatch")


l13_713_dir = root / "recovered/l13-root-713"
l13_713 = json.loads((l13_713_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "77384c8beb75db8cd6ce4fd47429ecd9d6252635bd8f148e7c726930171c38da"), ("helper_sha256", "ab39981f8993ba6139a234f67269d0735fb1fe54de44b3bdd6578ee67b72bff4"), ("registration_sha256", "2de410af6df66ce6634292cce2c16611b442f913c2b401ac2ebece3f8f290238")]:
    if l13_713[field] != expected or hashlib.sha256((l13_713_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 713 preparation hash mismatch")
l13_713_reg = json.loads((l13_713_dir / l13_713["registration_sha256"]).read_bytes())
if (len(l13_713_reg["sources"]) != 8 or l13_713_reg["fresh_lines_registered"] != 200
        or l13_713_reg["prior_material_lines_reused"] != 369 or len(l13_713_reg["valid_nested_range_fields"]) != 66):
    raise ValueError("Lens 13 root 713 registration scope mismatch")
l13_713_display = []
for item, source in zip(l13_713["source_objects"], l13_713_reg["sources"]):
    data = (l13_713_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 713 source mismatch")
    if item["original"] in (2721, 2722, 2723, 2728):
        lines = data.decode().splitlines()
        l13_713_display.append("SOURCE " + str(item["original"]))
        l13_713_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_713_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_713["fresh_display_sha256"]
        or display != ("\n".join(l13_713_display) + "\n").encode()):
    raise ValueError("Lens 13 root 713 display mismatch")


l13_713_qual_sha = "342652024177e4c485ee880c611fed750ccc272461a704daab3142df992b9b97"
if (l13_713["qualification_sha256"] != l13_713_qual_sha
        or hashlib.sha256((l13_713_dir / l13_713_qual_sha).read_bytes()).hexdigest() != l13_713_qual_sha):
    raise ValueError("Lens 13 root 713 qualification mismatch")
l13_713_q = json.loads((l13_713_dir / l13_713_qual_sha).read_bytes())
if (l13_713_q["registration"]["sha256"] != l13_713["registration_sha256"]
        or l13_713_q["fresh_lines"] != 200 or l13_713_q["reused_material_lines"] != 369
        or len(l13_713["prior_native_verifications"]) != 2):
    raise ValueError("Lens 13 root 713 qualification scope mismatch")
for item in l13_713["prior_native_verifications"]:
    if (hashlib.sha256((l13_713_dir / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]
            or item["sha256"] not in json.dumps(l13_713_q)):
        raise ValueError("Lens 13 root 713 prior verification mismatch")


l13_713_packet_sha = "3c0b171cb6fb9e3ed2fd0d47a0cba901ca8d404d7409e047f977b27dfe2b2e3a"
if (l13_713["packet_sha256"] != l13_713_packet_sha
        or hashlib.sha256((l13_713_dir / l13_713_packet_sha).read_bytes()).hexdigest() != l13_713_packet_sha):
    raise ValueError("Lens 13 root 713 packet mismatch")
l13_713_packet = json.loads((l13_713_dir / l13_713_packet_sha).read_bytes())
if (l13_713_packet["original_before"] != 2721 or l13_713_packet["original_after"] != 2729
        or l13_713_packet["new_root_ordered_lines"] != 200
        or l13_713_packet["source_line_roles"]["reused_production"] != 369
        or len(l13_713_packet["ordered"]) != 8):
    raise ValueError("Lens 13 root 713 packet scope mismatch")


l13_714_dir = root / "recovered/l13-root-714"
l13_714 = json.loads((l13_714_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "0b1f1ab4af04e6bb3b637a53195b7ed667b1091bd1bda2e31e1de0576aa51fca"), ("helper_sha256", "3a38e4cc4c2240b5a4f7c038ce6347f85d0c12bc3b116fac4dc3327480ddb6c8"), ("registration_sha256", "baaa48dae9da12538fb69b67332e4fc5078892f95b49bcd96b46414e8bd3fa45")]:
    if l13_714[field] != expected or hashlib.sha256((l13_714_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 714 preparation hash mismatch")
l13_714_reg = json.loads((l13_714_dir / l13_714["registration_sha256"]).read_bytes())
if (len(l13_714_reg["sources"]) != 7 or l13_714_reg["fresh_lines_registered"] != 498
        or l13_714_reg["prior_material_lines_reused"] != 544 or len(l13_714_reg["valid_nested_range_fields"]) != 56):
    raise ValueError("Lens 13 root 714 registration scope mismatch")
l13_714_display = []
for item, source in zip(l13_714["source_objects"], l13_714_reg["sources"]):
    data = (l13_714_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 714 source mismatch")
    if item["original"] in (2730, 2731, 2733, 2734, 2735):
        lines = data.decode().splitlines()
        l13_714_display.append("SOURCE " + str(item["original"]))
        l13_714_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_714_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_714["fresh_display_sha256"]
        or display != ("\n".join(l13_714_display) + "\n").encode()):
    raise ValueError("Lens 13 root 714 display mismatch")


l13_715_dir = root / "recovered/l13-root-715"
l13_715 = json.loads((l13_715_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "1a1697d0c1e1fe4cd126160823bdfdbd2c66ef15b21f212b8a038305cb0b0674"), ("helper_sha256", "e630cac6d5febe6c1241948ccd6bcfefeb9247bb8c3f4eb28afca2db014f1554"), ("registration_sha256", "384d6c11141f2bc91db35f343b9aa80891cd36372754a284e248caa1539095f5")]:
    if l13_715[field] != expected or hashlib.sha256((l13_715_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 715 preparation hash mismatch")
l13_715_reg = json.loads((l13_715_dir / l13_715["registration_sha256"]).read_bytes())
if (len(l13_715_reg["sources"]) != 7 or l13_715_reg["fresh_lines_registered"] != 111
        or l13_715_reg["prior_material_lines_reused"] != 960 or len(l13_715_reg["valid_nested_range_fields"]) != 39):
    raise ValueError("Lens 13 root 715 registration scope mismatch")
l13_715_display = []
for item, source in zip(l13_715["source_objects"], l13_715_reg["sources"]):
    data = (l13_715_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 715 source mismatch")
    if item["original"] in (2741, 2742):
        lines = data.decode().splitlines()
        l13_715_display.append("SOURCE " + str(item["original"]))
        l13_715_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_715_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_715["fresh_display_sha256"]
        or display != ("\n".join(l13_715_display) + "\n").encode()):
    raise ValueError("Lens 13 root 715 display mismatch")


l13_716_dir = root / "recovered/l13-root-716"
l13_716 = json.loads((l13_716_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "057ca7cd2fca07f25c3bc3cdc51a4727c1f703fe1e3c88a48bb2f8cb12483c62"), ("helper_sha256", "51b0f5d9c80288e511335e4cd91fabdc2044f90fde621dccf9bd69f67ed57674"), ("registration_sha256", "85e76c34619370d9677fe17b72c67c7a1134c5ad58aa192770789e9e22b1ef39")]:
    if l13_716[field] != expected or hashlib.sha256((l13_716_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 716 preparation hash mismatch")
l13_716_reg = json.loads((l13_716_dir / l13_716["registration_sha256"]).read_bytes())
if (len(l13_716_reg["sources"]) != 14 or l13_716_reg["fresh_lines_registered"] != 821
        or l13_716_reg["prior_material_lines_reused"] != 1294 or len(l13_716_reg["valid_nested_range_fields"]) != 53):
    raise ValueError("Lens 13 root 716 registration scope mismatch")
l13_716_display = []
for item, source in zip(l13_716["source_objects"], l13_716_reg["sources"]):
    data = (l13_716_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 716 source mismatch")
    if item["original"] not in (2743, 2744, 2746, 2747):
        lines = data.decode().splitlines()
        l13_716_display.append("SOURCE " + str(item["original"]))
        l13_716_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_716_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_716["fresh_display_sha256"]
        or display != ("\n".join(l13_716_display) + "\n").encode()):
    raise ValueError("Lens 13 root 716 display mismatch")


l13_563_logging_dir = root / "recovered/l13-root-563-logging"
l13_563_logging = json.loads((l13_563_logging_dir / "MANIFEST.json").read_bytes())
l13_563_logging_sha = "a4b577c907d340efb88f72340d5e1d4ec907bb54e13927871ba04dee00cfe7db"
if (l13_563_logging["registration_sha256"] != l13_563_logging_sha
        or hashlib.sha256((l13_563_logging_dir / l13_563_logging_sha).read_bytes()).hexdigest() != l13_563_logging_sha):
    raise ValueError("Lens 13 root 563 logging registration mismatch")
l13_563_logging_reg = json.loads((l13_563_logging_dir / l13_563_logging_sha).read_bytes())
if len(l13_563_logging_reg["sources"]) != 3 or len(l13_563_logging["sources"]) != 3:
    raise ValueError("Lens 13 root 563 logging source count mismatch")
for item, source in zip(l13_563_logging["sources"], l13_563_logging_reg["sources"]):
    data = (l13_563_logging_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]
            or len(data.splitlines()) != source["actual_eof"]):
        raise ValueError("Lens 13 root 563 logging source mismatch")


l13_716_qual_sha = "6e47154583cda63f6e7e566d8fabd216571f16b51c5d3715524adfa35f98bd63"
if (l13_716["qualification_sha256"] != l13_716_qual_sha
        or hashlib.sha256((l13_716_dir / l13_716_qual_sha).read_bytes()).hexdigest() != l13_716_qual_sha):
    raise ValueError("Lens 13 root 716 qualification mismatch")
l13_716_q = json.loads((l13_716_dir / l13_716_qual_sha).read_bytes())
if (l13_716_q["registration"]["sha256"] != l13_716["registration_sha256"]
        or l13_716_q["fresh_lines"] != 821 or l13_716_q["reused_material_lines"] != 1294
        or len(l13_716_q["entries"]) != 14 or len(l13_716["prior_support_files"]) != 22):
    raise ValueError("Lens 13 root 716 qualification scope mismatch")
for item in l13_716["prior_support_files"]:
    if (hashlib.sha256((l13_716_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]
            or item["sha256"] not in json.dumps(l13_716_q)):
        raise ValueError("Lens 13 root 716 prior support mismatch")


l13_716_packet_sha = "25cc1253513d2a23be457dd16260c54300aa6e99879c9a22026ca5a2fb79c841"
if (l13_716["packet_sha256"] != l13_716_packet_sha
        or hashlib.sha256((l13_716_dir / l13_716_packet_sha).read_bytes()).hexdigest() != l13_716_packet_sha):
    raise ValueError("Lens 13 root 716 packet mismatch")
l13_716_packet = json.loads((l13_716_dir / l13_716_packet_sha).read_bytes())
if (l13_716_packet["original_before"] != 2743 or l13_716_packet["original_after"] != 2757
        or l13_716_packet["new_root_ordered_lines"] != 821
        or l13_716_packet["source_line_roles"]["reused_production"] != 1294
        or len(l13_716_packet["ordered"]) != 14):
    raise ValueError("Lens 13 root 716 packet scope mismatch")


l13_717_dir = root / "recovered/l13-root-717"
l13_717 = json.loads((l13_717_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "426f381a4c1e3273a81a3bf5543c5f3adc21ac7adccd1ceab6f6911b5fb68725"), ("helper_sha256", "e083ea8ef39a997b71575baf7c6fe8c7a3aee182993c0660a0259c01b5e8b1f0"), ("registration_sha256", "c112cbb94b87a95667b63372e569e80abc781371d82ba6c06ebd0f8b28d23be5"), ("prior_provenance_sha256", "17b914328ffc351335fc43f7b6bc1883434b9c336205c6cdf6916b7b71b0ec73")]:
    if l13_717[field] != expected or hashlib.sha256((l13_717_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 717 preparation hash mismatch")
l13_717_reg = json.loads((l13_717_dir / l13_717["registration_sha256"]).read_bytes())
if (len(l13_717_reg["sources"]) != 14 or l13_717_reg["fresh_lines_registered"] != 538
        or l13_717_reg["prior_material_lines_reused"] != 581 or len(l13_717_reg["valid_nested_range_fields"]) != 30):
    raise ValueError("Lens 13 root 717 registration scope mismatch")
l13_717_display = []
for item, source in zip(l13_717["source_objects"], l13_717_reg["sources"]):
    data = (l13_717_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 717 source mismatch")
    if item["original"] >= 2764:
        lines = data.decode().splitlines()
        l13_717_display.append("SOURCE " + str(item["original"]))
        l13_717_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_717_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_717["fresh_display_sha256"]
        or display != ("\n".join(l13_717_display) + "\n").encode()):
    raise ValueError("Lens 13 root 717 display mismatch")


l13_717_provenance = json.loads((l13_717_dir / l13_717["prior_provenance_sha256"]).read_bytes())
l13_717_archive_sha = "b9ef1d0d795eb7d84685bca4f72f3649f064e6641543d3a8c415898726a57b41"
l13_717_archive = l13_717_dir / l13_717_archive_sha
if (l13_717["native_archive_sha256"] != l13_717_archive_sha
        or l13_717_provenance["registry_archive"]["sha256"] != l13_717_archive_sha
        or hashlib.sha256(l13_717_archive.read_bytes()).hexdigest() != l13_717_archive_sha
        or l13_717_archive.stat().st_size != l13_717_provenance["archive_bytes"]
        or len(l13_717_provenance["member_sources"]) != 7):
    raise ValueError("Lens 13 root 717 native archive mismatch")
with tarfile.open(l13_717_archive, "r:gz") as archive:
    for member in l13_717_provenance["member_sources"]:
        entry = archive.extractfile(member["archive_member"])
        if entry is None:
            raise ValueError("Lens 13 root 717 archive member missing")
        data = entry.read()
        if (len(data) != member["bytes"] or len(data.splitlines()) != member["actual_eof"]
                or hashlib.sha256(data).hexdigest() != member["source_sha256"]
                or data != (l13_717_dir / member["source_sha256"]).read_bytes()):
            raise ValueError("Lens 13 root 717 archive member mismatch")


for field, expected in [("prior_native_registration_sha256", "fe8eb7ace372139f5a61cff0b1f9d3995270b69d92921e23bd9359357e6d2640"), ("prior_manifest_registration_sha256", "8ab107bad8cf69c87f6ee506105c388164530db980b12f6370b654fe282ba1f6"), ("prior_native_verification_sha256", "df2988f65b69eca35678b6e4fd9920a234975aab79f5f961830a2b8d67c9bf02"), ("prior_manifest_source_sha256", "c39a8a5055ca88572a587d0eadbde46eeb889d9060ec6b8930f59bfb76e44303")]:
    if l13_717[field] != expected or hashlib.sha256((l13_717_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 717 prior connected evidence mismatch")
l13_717_prior_reg = json.loads((l13_717_dir / l13_717["prior_native_registration_sha256"]).read_bytes())
l13_717_prior_supp = json.loads((l13_717_dir / l13_717["prior_manifest_registration_sha256"]).read_bytes())
l13_717_prior_verify = json.loads((l13_717_dir / l13_717["prior_native_verification_sha256"]).read_bytes())
if (l13_717_prior_supp["parent_registration"]["sha256"] != l13_717["prior_native_registration_sha256"]
        or l13_717_prior_supp["sources"][0]["sha256"] != l13_717["prior_manifest_source_sha256"]
        or [x["sha256"] for x in l13_717_prior_reg["sources"]] != [x["source_sha256"] for x in l13_717_provenance["member_sources"]]
        or l13_717_prior_verify["receipt"]["sha256"] != "ad3fb5978d211b757973d947dc3638011f2e10d49feb7ecf74b6b77be8250e3a"):
    raise ValueError("Lens 13 root 717 prior connected references mismatch")


l13_717_qual_sha = "1947a70940b290a7a3637dd60900652916f47ae829403b105983470dd2c4250c"
if (l13_717["qualification_sha256"] != l13_717_qual_sha
        or hashlib.sha256((l13_717_dir / l13_717_qual_sha).read_bytes()).hexdigest() != l13_717_qual_sha):
    raise ValueError("Lens 13 root 717 qualification hash mismatch")
l13_717_q = json.loads((l13_717_dir / l13_717_qual_sha).read_bytes())
if (l13_717_q["registration"]["sha256"] != l13_717["registration_sha256"]
        or l13_717_q["fresh_lines"] != 538 or l13_717_q["reused_material_lines"] != 581
        or len(l13_717_q["entries"]) != 14
        or any(e["prior"]["prior_native_verification"]["sha256"] != l13_717["prior_native_verification_sha256"] for e in l13_717_q["entries"][:7])):
    raise ValueError("Lens 13 root 717 qualification scope mismatch")


l13_717_packet_sha = "113856a874029a947ba3060ff2a594f5d30983577de186227077d65ea7e6dbd0"
if (l13_717["packet_sha256"] != l13_717_packet_sha
        or hashlib.sha256((l13_717_dir / l13_717_packet_sha).read_bytes()).hexdigest() != l13_717_packet_sha):
    raise ValueError("Lens 13 root 717 packet hash mismatch")
l13_717_packet = json.loads((l13_717_dir / l13_717_packet_sha).read_bytes())
if (l13_717_packet["original_before"] != 2757 or l13_717_packet["original_after"] != 2771
        or l13_717_packet["new_root_ordered_lines"] != 538
        or l13_717_packet["source_line_roles"]["reused_production"] != 581
        or len(l13_717_packet["ordered"]) != 14
        or len(l13_717_packet["new_questions"]) != 1):
    raise ValueError("Lens 13 root 717 packet scope mismatch")


for field, expected in [("prior_connected_packet_sha256", "15b0da2eddd6207a2ee30416277e4fa38fec9964bca27bce79258bc40ce40d37"), ("prior_connected_adoption_sha256", "ad3fb5978d211b757973d947dc3638011f2e10d49feb7ecf74b6b77be8250e3a")]:
    if l13_717[field] != expected or hashlib.sha256((l13_717_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 717 connected evidence mismatch")
l13_717_connected_packet = json.loads((l13_717_dir / l13_717["prior_connected_packet_sha256"]).read_bytes())
l13_717_connected_receipt = json.loads((l13_717_dir / l13_717["prior_connected_adoption_sha256"]).read_bytes())
if (l13_717_connected_receipt["packet"]["sha256"] != l13_717["prior_connected_packet_sha256"]
        or l13_717_connected_receipt["evidence"] != l13_717_connected_packet
        or l13_717_prior_verify["receipt"]["sha256"] != l13_717["prior_connected_adoption_sha256"]
        or l13_717_connected_packet["fresh_root_connected_lines"] != 607
        or len(l13_717_connected_packet["sources"]) != 8):
    raise ValueError("Lens 13 root 717 connected adoption scope mismatch")


l13_718_dir = root / "recovered/l13-root-718"
l13_718 = json.loads((l13_718_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "4da748116913bd89478304031438c868b57629de94b9edbd7da1766ac3e03deb"), ("helper_sha256", "da35e12f8a01bf649db16095513ea6a4e0e1bd6fe8eac901aa8d5d91666450e9"), ("registration_sha256", "0416fd07732e71ddee87bb731e0905d4fdf5d9bd67c4bf5b25d70913daf549c4")]:
    if l13_718[field] != expected or hashlib.sha256((l13_718_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 718 preparation hash mismatch")
l13_718_reg = json.loads((l13_718_dir / l13_718["registration_sha256"]).read_bytes())
if (len(l13_718_reg["sources"]) != 16 or l13_718_reg["fresh_lines_registered"] != 437
        or l13_718_reg["prior_material_lines_reused"] != 812 or len(l13_718_reg["valid_nested_range_fields"]) != 32):
    raise ValueError("Lens 13 root 718 registration scope mismatch")
l13_718_display = []
for item, source in zip(l13_718["source_objects"], l13_718_reg["sources"]):
    data = (l13_718_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 718 source mismatch")
    if item["original"] not in (2773, 2775, 2781, 2782, 2783, 2784):
        lines = data.decode().splitlines()
        l13_718_display.append("SOURCE " + str(item["original"]))
        l13_718_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_718_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_718["fresh_display_sha256"]
        or display != ("\n".join(l13_718_display) + "\n").encode()):
    raise ValueError("Lens 13 root 718 display mismatch")


l13_718_qual_sha = "83b6d6c6d92ea2014351b809403c842e547cfddc73c05285e1eb4bb7a55b540d"
if (l13_718["qualification_sha256"] != l13_718_qual_sha
        or hashlib.sha256((l13_718_dir / l13_718_qual_sha).read_bytes()).hexdigest() != l13_718_qual_sha):
    raise ValueError("Lens 13 root 718 qualification mismatch")
l13_718_q = json.loads((l13_718_dir / l13_718_qual_sha).read_bytes())
if (l13_718_q["registration"]["sha256"] != l13_718["registration_sha256"]
        or l13_718_q["fresh_lines"] != 437 or l13_718_q["reused_material_lines"] != 812
        or len(l13_718_q["entries"]) != 16 or len(l13_718["prior_support_files"]) != 27):
    raise ValueError("Lens 13 root 718 qualification scope mismatch")
for item in l13_718["prior_support_files"]:
    if (hashlib.sha256((l13_718_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]
            or item["sha256"] not in json.dumps(l13_718_q)):
        raise ValueError("Lens 13 root 718 prior support mismatch")


l13_718_packet_sha = "79961ca9d3ccb615599a113397a6fe58b8113a70b139053b9027c62f928b9e25"
if (l13_718["packet_sha256"] != l13_718_packet_sha
        or hashlib.sha256((l13_718_dir / l13_718_packet_sha).read_bytes()).hexdigest() != l13_718_packet_sha):
    raise ValueError("Lens 13 root 718 packet mismatch")
l13_718_packet = json.loads((l13_718_dir / l13_718_packet_sha).read_bytes())
if (l13_718_packet["original_before"] != 2771 or l13_718_packet["original_after"] != 2787
        or l13_718_packet["new_root_ordered_lines"] != 437
        or l13_718_packet["source_line_roles"]["reused_production"] != 812
        or len(l13_718_packet["ordered"]) != 16):
    raise ValueError("Lens 13 root 718 packet scope mismatch")


l13_718_archive_sha = "c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14"
l13_718_lock_sha = "82b8a6ebb3685ec699069eed2cd8cdf76be94b6fe70f23b7e8eb2a62fe10c1ce"
archive_path = l13_718_dir / l13_718_archive_sha
lock_path = l13_718_dir / l13_718_lock_sha
if (l13_718["native_archive_sha256"] != l13_718_archive_sha
        or l13_718["frozen_lock_sha256"] != l13_718_lock_sha
        or hashlib.sha256(archive_path.read_bytes()).hexdigest() != l13_718_archive_sha
        or hashlib.sha256(lock_path.read_bytes()).hexdigest() != l13_718_lock_sha):
    raise ValueError("Lens 13 root 718 archive or lock mismatch")
lock_packages = tomllib.loads(lock_path.read_text(encoding="utf-8"))["package"]
package = next(x for x in lock_packages if x["name"] == "serde_json" and x["version"] == "1.0.151")
if package["checksum"] != l13_718_archive_sha:
    raise ValueError("Lens 13 root 718 archive does not match frozen lock")
with tarfile.open(archive_path, "r:gz") as archive:
    member = archive.extractfile("serde_json-1.0.151/src/map.rs")
    source = next(x for x in l13_718_reg["sources"] if x["original"] == 2785)
    if member is None or member.read() != (l13_718_dir / source["sha256"]).read_bytes():
        raise ValueError("Lens 13 root 718 archive member mismatch")


native_serde_dir = root / "recovered/native-serde-pinned-originals"
native_serde = json.loads((native_serde_dir / "MANIFEST.json").read_bytes())
expected_serde_hashes = ["ceda1bf415aafde89caa83bf275811b70d393717d4e1038e0c61822d2da02993", "70b83c36286f6c58123f544f27db2b54a1135f928fe33b08b521b2997773f862"]
if (native_serde["archive_sha256"] != l13_718_archive_sha
        or [x["original"] for x in native_serde["sources"]] != [2589, 2590]
        or [x["sha256"] for x in native_serde["sources"]] != expected_serde_hashes):
    raise ValueError("Pinned serde original inventory mismatch")
with tarfile.open(archive_path, "r:gz") as archive:
    for item in native_serde["sources"]:
        member = archive.extractfile(item["archive_member"])
        if member is None:
            raise ValueError("Pinned serde archive member missing")
        data = member.read()
        if (len(data) != item["bytes"] or len(data.splitlines()) != item["actual_eof_derived"]
                or hashlib.sha256(data).hexdigest() != item["sha256"]
                or data != (native_serde_dir / "objects" / item["sha256"]).read_bytes()):
            raise ValueError("Pinned serde source object mismatch")


l13_692_dir = root / "recovered/l13-root-692-fresh"
l13_692_manifest = json.loads((l13_692_dir / "MANIFEST.json").read_bytes())
l13_692_archive = l13_692_manifest["pinned_archive_evidence"]
if (l13_692_archive["archive_sha256"] != l13_718_archive_sha
        or l13_692_archive["frozen_lock_sha256"] != l13_718_lock_sha
        or l13_692_archive["original"] != 2601):
    raise ValueError("Lens 13 root 692 pinned decoder reference mismatch")
with tarfile.open(root / l13_692_archive["archive_file"], "r:gz") as archive:
    member = archive.extractfile(l13_692_archive["archive_member"])
    if member is None or member.read() != (l13_692_dir / "sources" / l13_692_archive["source_sha256"]).read_bytes():
        raise ValueError("Lens 13 root 692 pinned decoder member mismatch")


native_serde_dir = root / "recovered/native-serde-pinned-originals"
native_serde = json.loads((native_serde_dir / "MANIFEST.json").read_bytes())
expected_serde_hashes = ["ceda1bf415aafde89caa83bf275811b70d393717d4e1038e0c61822d2da02993", "70b83c36286f6c58123f544f27db2b54a1135f928fe33b08b521b2997773f862"]
if (native_serde["archive_sha256"] != l13_718_archive_sha
        or [x["original"] for x in native_serde["sources"]] != [2589, 2590]
        or [x["sha256"] for x in native_serde["sources"]] != expected_serde_hashes):
    raise ValueError("Pinned serde original inventory mismatch")
with tarfile.open(archive_path, "r:gz") as archive:
    for item in native_serde["sources"]:
        member = archive.extractfile(item["archive_member"])
        if member is None:
            raise ValueError("Pinned serde archive member missing")
        data = member.read()
        if (len(data) != item["bytes"] or len(data.splitlines()) != item["actual_eof_derived"]
                or hashlib.sha256(data).hexdigest() != item["sha256"]
                or data != (native_serde_dir / "objects" / item["sha256"]).read_bytes()):
            raise ValueError("Pinned serde source object mismatch")


l13_719_dir = root / "recovered/l13-root-719"
l13_719 = json.loads((l13_719_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "8ee83ab90f91775c84f40c02410ae21fb9ce71a01d638781bbc5d22d880a26c7"), ("helper_sha256", "a725bb782aa3e44997e26cc302d6714735d6b010bae3519d258677ff3e7bed03"), ("registration_sha256", "75bb948cd24dddfe1c2f07e9e872e412497a5e464319cf8609af07efd96e9f83"), ("frozen_lock_sha256", "82b8a6ebb3685ec699069eed2cd8cdf76be94b6fe70f23b7e8eb2a62fe10c1ce")]:
    if l13_719[field] != expected or hashlib.sha256((l13_719_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 719 preparation hash mismatch")
l13_719_reg = json.loads((l13_719_dir / l13_719["registration_sha256"]).read_bytes())
if (len(l13_719_reg["sources"]) != 18 or l13_719_reg["fresh_lines_registered"] != 1282
        or l13_719_reg["prior_material_lines_reused"] != 540 or len(l13_719_reg["valid_nested_range_fields"]) != 36):
    raise ValueError("Lens 13 root 719 registration scope mismatch")
l13_719_display = []
for item, source in zip(l13_719["source_objects"], l13_719_reg["sources"]):
    data = (l13_719_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 719 source mismatch")
    if item["original"] not in (2787, 2788, 2789, 2801, 2803):
        lines = data.decode().splitlines()
        l13_719_display.append("SOURCE " + str(item["original"]))
        l13_719_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_719_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_719["fresh_display_sha256"]
        or display != ("\n".join(l13_719_display) + "\n").encode()):
    raise ValueError("Lens 13 root 719 display mismatch")


if len(l13_719["prior_support_files"]) != 18:
    raise ValueError("Lens 13 root 719 support count mismatch")
for item in l13_719["prior_support_files"]:
    if hashlib.sha256((l13_719_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Lens 13 root 719 prior support mismatch")
prior_314 = (l13_719_dir / "printed-prior-verification-314.json").read_bytes()
if hashlib.sha256(prior_314).hexdigest() != l13_719["printed_prior_verification_314_sha256"]:
    raise ValueError("Lens 13 root 719 printed verification mismatch")
prior_314_record = json.loads(prior_314)
if (prior_314_record["transcript_event_ordinal"] != 55385
        or prior_314_record["record"]["receipt_sha256"] != "0fdb8c8faa044fe3513459ee36952db71d9e42ce48bb2f9c83fd0d485a06857b"):
    raise ValueError("Lens 13 root 719 printed verification reference mismatch")


native_bundle = l13_719_dir / "bevy-eidolon-native.bundle"
if hashlib.sha256(native_bundle.read_bytes()).hexdigest() != l13_719["native_git_bundle_sha256"]:
    raise ValueError("Lens 13 root 719 native Git bundle mismatch")
native_commit = "1a5aff8d4809287e5f5713fb4cf72181b2285fff"
if l13_719["native_git_commit"] != native_commit or l13_719["native_git_source_originals"] != list(range(2790, 2798)):
    raise ValueError("Lens 13 root 719 native Git revision mismatch")
lock = tomllib.loads((l13_719_dir / l13_719["frozen_lock_sha256"]).read_text(encoding="utf-8"))
native_package = next(x for x in lock["package"] if x["name"] == "bevy_eidolon" and x["version"] == "0.5.0")
if not native_package["source"].endswith("#" + native_commit):
    raise ValueError("Lens 13 root 719 native Git lock pin mismatch")
with tempfile.TemporaryDirectory(prefix=".726-native-git-", dir=root) as directory:
    git_dir = Path(directory) / "source.git"
    subprocess.run(["git", "clone", "--quiet", "--bare", str(native_bundle), str(git_dir)], check=True)
    if subprocess.check_output(["git", "--git-dir", str(git_dir), "rev-parse", "refs/heads/master"], text=True).strip() != native_commit:
        raise ValueError("Lens 13 root 719 native Git bundle ref mismatch")
    for source in l13_719_reg["sources"]:
        if source["original"] in range(2790, 2798):
            path = source["source_id"].split("/", 1)[1]
            data = subprocess.check_output(["git", "--git-dir", str(git_dir), "show", native_commit + ":" + path])
            if data != (l13_719_dir / source["sha256"]).read_bytes():
                raise ValueError("Lens 13 root 719 native Git object mismatch")

    l13_721_native_dir = root / "recovered/l13-root-721"
    l13_721_native = json.loads((l13_721_native_dir / "MANIFEST.json").read_bytes())["native_git_source_evidence"]
    if (l13_721_native["original"] != 2827 or l13_721_native["revision"] != native_commit
            or l13_721_native["git_path"] != "src/cull/node.rs"):
        raise ValueError("Lens 13 root 721 native Git reference mismatch")
    native_bytes = subprocess.check_output(["git", "--git-dir", str(git_dir), "show", native_commit + ":" + l13_721_native["git_path"]])
    source_721 = next(x for x in json.loads((l13_721_native_dir / "MANIFEST.json").read_bytes())["source_objects"] if x["original"] == 2827)
    if native_bytes != (l13_721_native_dir / source_721["sha256"]).read_bytes():
        raise ValueError("Lens 13 root 721 native Git source mismatch")

    native_original_dir = root / "recovered/native-bevy-pinned-originals"
    native_originals = json.loads((native_original_dir / "MANIFEST.json").read_bytes())
    if (native_originals["revision"] != native_commit
            or [x["original"] for x in native_originals["sources"]] != list(range(31, 39))):
        raise ValueError("Native original source inventory mismatch")
    expected_native_hashes = ['e2053ee2dbf1a837b1a4e80d2d2afa22c0ecc2231bf586041222082971b9cbea', '31551b31caac91e5decc47ac147154bba899d469dad8c0113e0a476ae53dc7d4', 'bedf4e9e3cb681d6cd4ef75b146537e8e2c4876cad464addd82ba0bc5cdc83ab', '63c3f55c90686a73788212c94f4d979fd2bc3240dd143c6b5e778d949d68fd65', '8fdf638edbf13ddc04a1a8392f3b873ec730f86f085522de471108c623ac129b', '3529f71ec77f88963ee969691ef2dd2f871abf490a8a4beed073b98ff10974b9', '9bd27e4a4bb61b319e4ee13ae7e1407be2fdd432471ef6e9aede72e206321d90', 'e681dfe05b5df31bfba37904eae24631e48be8a91e90910253cba046bdecfcc9']
    if [x["sha256"] for x in native_originals["sources"]] != expected_native_hashes:
        raise ValueError("Native original registry hashes mismatch")
    for item in native_originals["sources"]:
        data = subprocess.check_output(["git", "--git-dir", str(git_dir), "show", native_commit + ":" + item["git_path"]])
        if (len(data) != item["bytes"] or len(data.splitlines()) != item["actual_eof_derived"]
                or hashlib.sha256(data).hexdigest() != item["sha256"]
                or data != (native_original_dir / "objects" / item["sha256"]).read_bytes()):
            raise ValueError("Native original source object mismatch")


l13_720_dir = root / "recovered/l13-root-720"
l13_720 = json.loads((l13_720_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "ac195e80a27789cb5b40e7eadb7200bafbf9fe44a5a502636aae0a49201254b5"), ("helper_sha256", "9f04f35fbbc5cc7080fb4fd1bcd1573b7572204d8e9de29d3502a7aa04d13338"), ("registration_sha256", "05df017e6e0ab147fc5fa3a31f222d3aaf7611fbe2a98aaf8a5cb66d25f58a5b")]:
    if l13_720[field] != expected or hashlib.sha256((l13_720_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 720 preparation hash mismatch")
l13_720_reg = json.loads((l13_720_dir / l13_720["registration_sha256"]).read_bytes())
if (len(l13_720_reg["sources"]) != 18 or l13_720_reg["fresh_lines_registered"] != 713
        or l13_720_reg["prior_material_lines_reused"] != 545 or len(l13_720_reg["valid_nested_range_fields"]) != 36
        or len({item["sha256"] for item in l13_720["source_objects"]}) != 15):
    raise ValueError("Lens 13 root 720 registration scope mismatch")
l13_720_display = []
for item, source in zip(l13_720["source_objects"], l13_720_reg["sources"]):
    data = (l13_720_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 720 source mismatch")
    if item["original"] in (2806, 2808, 2810, 2811, 2812, 2813, 2814, 2816, 2817, 2821):
        lines = data.decode().splitlines()
        l13_720_display.append("SOURCE " + str(item["original"]))
        l13_720_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_720_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_720["fresh_display_sha256"]
        or display != ("\n".join(l13_720_display) + "\n").encode()):
    raise ValueError("Lens 13 root 720 display mismatch")


l13_721_dir = root / "recovered/l13-root-721"
l13_721 = json.loads((l13_721_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "e1fd1a9064206d80865201cadedde7917f7c3737a71236a07e1fa815a43e44a3"), ("helper_sha256", "9b2cd6cb78b6635b567e3d0e7aeaa2f089eaa1e17571d0e45e500bf5379e5d9d"), ("registration_sha256", "332b04804f99922023363e7a1d651a4902329912f3c8ddf06ecb03143abf6b68")]:
    if l13_721[field] != expected or hashlib.sha256((l13_721_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 721 preparation hash mismatch")
l13_721_discovery = json.loads((l13_721_dir / l13_721["discovery_sha256"]).read_bytes())
l13_721_reg = json.loads((l13_721_dir / l13_721["registration_sha256"]).read_bytes())
if (len(l13_721_discovery["entries"]) != 30 or len(l13_721_reg["sources"]) != 13
        or l13_721_reg["fresh_lines_registered"] != 425 or l13_721_reg["prior_material_lines_reused"] != 1965
        or len(l13_721_reg["valid_nested_range_fields"]) != 26):
    raise ValueError("Lens 13 root 721 registration scope mismatch")
l13_721_display = []
for item, source in zip(l13_721["source_objects"], l13_721_reg["sources"]):
    data = (l13_721_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 721 source mismatch")
    if item["original"] in (2823, 2825, 2827):
        lines = data.decode().splitlines()
        l13_721_display.append("SOURCE " + str(item["original"]))
        l13_721_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_721_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_721["fresh_display_sha256"]
        or display != ("\n".join(l13_721_display) + "\n").encode()):
    raise ValueError("Lens 13 root 721 display mismatch")


summary_path = root / "recovered/l13-root-710/prior-verification-summaries.json"
summary_manifest = json.loads((summary_path.parent / "MANIFEST.json").read_bytes())
if hashlib.sha256(summary_path.read_bytes()).hexdigest() != summary_manifest["prior_verification_summaries_sha256"]:
    raise ValueError("Lens 13 root 710 prior summary mismatch")
prior_summaries = json.loads(summary_path.read_bytes())
if [(x["root_sequence"], x["transcript_event_ordinal"], x["printed_verification_summary"]["verification"]["sha256"]) for x in prior_summaries] != [
    (641, 42609, "32363e2b9a3b2ded159ab803dceaacdbd2a96b492298bdc1973dfb6ddae3424e"),
    (644, 43023, "39d6323a1c6116c4e0c7615c3c3a6fe64d671efcc58218bcb1be39a365e61d41")]:
    raise ValueError("Lens 13 root 710 historical report references mismatch")


if len(l13_721["prior_support_files"]) != 12:
    raise ValueError("Lens 13 root 721 support count mismatch")
for item in l13_721["prior_support_files"]:
    if hashlib.sha256((l13_721_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]:
        raise ValueError("Lens 13 root 721 prior support hash mismatch")
l13_721_row = (l13_721_dir / "prior-root563-row.json").read_bytes()
if hashlib.sha256(l13_721_row).hexdigest() != l13_721["prior_root563_row_extract_sha256"]:
    raise ValueError("Lens 13 root 721 prior row extract mismatch")
prior_row = json.loads(l13_721_row)
if (prior_row["transcript_event_ordinal"] != 55843 or prior_row["root_sequence"] != 563
        or prior_row["row"]["source"]["sha256"] != l13_721_reg["sources"][1]["sha256"]):
    raise ValueError("Lens 13 root 721 prior row identity mismatch")


l13_722_dir = root / "recovered/l13-root-722"
l13_722 = json.loads((l13_722_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "833bf8943cafb748d73f2d2caf90b13c3e68a65f8fc7566941b3b5a855ff7a92"), ("helper_sha256", "50ab67fb073b7317c27948d932b2b2b7496e3525e375f9698ac180cb3740b830"), ("registration_sha256", "62fb22103d5c5b8930b1c2d2b4f620ef867f131721f29fd3dbc7c600623d5623")]:
    if l13_722[field] != expected or hashlib.sha256((l13_722_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 722 preparation hash mismatch")
l13_722_reg = json.loads((l13_722_dir / l13_722["registration_sha256"]).read_bytes())
if (len(l13_722_reg["sources"]) != 10 or l13_722_reg["fresh_lines_registered"] != 573
        or l13_722_reg["prior_material_lines_reused"] != 285 or len(l13_722_reg["valid_nested_range_fields"]) != 20):
    raise ValueError("Lens 13 root 722 registration scope mismatch")
l13_722_display = []
for item, source in zip(l13_722["source_objects"], l13_722_reg["sources"]):
    data = (l13_722_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 722 source mismatch")
    if item["original"] in range(2836, 2844):
        lines = data.decode().splitlines()
        l13_722_display.append("SOURCE " + str(item["original"]))
        l13_722_display.extend(f"{i}: {lines[i-1]}" for lo, hi in source["prospective_ranges"] for i in range(lo, hi + 1))
display = (l13_722_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_722["fresh_display_sha256"]
        or display != ("\n".join(l13_722_display) + "\n").encode()):
    raise ValueError("Lens 13 root 722 display mismatch")


for field, expected in [("qualification_sha256", "12bd59861e3c57be9dce4b4efe6b824e00eb58e74b655effe2fc3d494d9bd0fc"), ("packet_sha256", "6e30bc3515a4ee8d2504e780da0c96017c2e719fffa8123f16dfb0d26dbd29eb")]:
    if l13_722[field] != expected or hashlib.sha256((l13_722_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 722 qualification or packet hash mismatch")
l13_722_q = json.loads((l13_722_dir / l13_722["qualification_sha256"]).read_bytes())
l13_722_packet = json.loads((l13_722_dir / l13_722["packet_sha256"]).read_bytes())
if (l13_722_q["registration"]["sha256"] != l13_722["registration_sha256"]
        or l13_722_q["fresh_lines"] != 573 or l13_722_q["reused_material_lines"] != 285
        or l13_722_packet["original_after"] != 2846 or l13_722_packet["new_root_ordered_lines"] != 573
        or len(l13_722["prior_support_files"]) != 5):
    raise ValueError("Lens 13 root 722 qualification scope mismatch")
for item in l13_722["prior_support_files"]:
    if (hashlib.sha256((l13_722_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]
            or item["sha256"] not in json.dumps(l13_722_q)):
        raise ValueError("Lens 13 root 722 prior support mismatch")


l13_723_dir = root / "recovered/l13-root-723"
l13_723 = json.loads((l13_723_dir / "MANIFEST.json").read_bytes())
for field, expected in [("discovery_sha256", "3dfc0d3d341f1dab4cef098697b1dc35a977347e7fd12e9bd552ab2a9907aeb0"), ("helper_sha256", "660d703a9fbb1d0f3af36ee71a538f74189e1ebd521a88c51c1d2f259398635e"), ("registration_sha256", "ec0c5b93a0d57a0e3970291e4dcedc00182515bdcc57732e0c0198a639bd4918")]:
    if l13_723[field] != expected or hashlib.sha256((l13_723_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 723 preparation hash mismatch")
l13_723_reg = json.loads((l13_723_dir / l13_723["registration_sha256"]).read_bytes())
if (len(l13_723_reg["sources"]) != 7 or l13_723_reg["fresh_lines_registered"] != 409
        or l13_723_reg["prior_material_lines_reused"] != 47 or len(l13_723_reg["valid_nested_range_fields"]) != 14):
    raise ValueError("Lens 13 root 723 registration scope mismatch")
l13_723_display = []
for item, source in zip(l13_723["source_objects"], l13_723_reg["sources"]):
    data = (l13_723_dir / item["sha256"]).read_bytes()
    if (item["original"] != source["original"] or item["sha256"] != source["sha256"]
            or len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]):
        raise ValueError("Lens 13 root 723 source mismatch")
    if item["original"] != 2847:
        lines = data.decode().splitlines()
        l13_723_display.append("SOURCE " + str(item["original"]))
        ranges = [[216, 216]] if item["original"] == 2846 else source["prospective_ranges"]
        l13_723_display.extend(f"{i}: {lines[i-1]}" for lo, hi in ranges for i in range(lo, hi + 1))
display = (l13_723_dir / "fresh-display.txt").read_bytes()
if (hashlib.sha256(display).hexdigest() != l13_723["fresh_display_sha256"]
        or display != ("\n".join(l13_723_display) + "\n").encode()):
    raise ValueError("Lens 13 root 723 display mismatch")


for field, expected in [("qualification_sha256", "2c1ea660bff7fa7fdc8e500681a775087b4c0985a8d3f981f0bf847d21c3d506"), ("packet_sha256", "d60d2086dc5c770ebc811fb3715e15cd804d95e7b06e3d9a5261c5c1396ea588")]:
    if l13_723[field] != expected or hashlib.sha256((l13_723_dir / expected).read_bytes()).hexdigest() != expected:
        raise ValueError("Lens 13 root 723 qualification or packet hash mismatch")
l13_723_q = json.loads((l13_723_dir / l13_723["qualification_sha256"]).read_bytes())
l13_723_packet = json.loads((l13_723_dir / l13_723["packet_sha256"]).read_bytes())
if (l13_723_q["registration"]["sha256"] != l13_723["registration_sha256"]
        or l13_723_q["fresh_lines"] != 409 or l13_723_q["reused_material_lines"] != 47
        or l13_723_q["entries"][0]["fresh_personal_assessed_ranges"] != [[216, 216]]
        or l13_723_q["entries"][0]["qualified_prior_material_ranges"] != [[217, 243]]
        or l13_723_packet["original_after"] != 2853 or l13_723_packet["new_root_ordered_lines"] != 409):
    raise ValueError("Lens 13 root 723 qualification scope mismatch")
if len(l13_723["prior_support_files"]) != 4:
    raise ValueError("Lens 13 root 723 prior support count mismatch")
for item in l13_723["prior_support_files"]:
    if (hashlib.sha256((l13_723_dir / "prior-support" / item["sha256"]).read_bytes()).hexdigest() != item["sha256"]
            or item["sha256"] not in json.dumps(l13_723_q)):
        raise ValueError("Lens 13 root 723 prior support mismatch")


original_archive_dir = root / "recovered/original-registry-source-archive"
original_archive = json.loads((original_archive_dir / "MANIFEST.json").read_bytes())
registry_gz = (original_archive_dir / "registry.json.gz").read_bytes()
objects_gz = (original_archive_dir / "objects.tar.gz").read_bytes()
if (hashlib.sha256(registry_gz).hexdigest() != original_archive["registry_gzip_sha256"]
        or hashlib.sha256(objects_gz).hexdigest() != original_archive["objects_archive_sha256"]):
    raise ValueError("Original registry archive checksum mismatch")
registry_raw = gzip.decompress(registry_gz)
if hashlib.sha256(registry_raw).hexdigest() != "d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f":
    raise ValueError("Original registry checksum mismatch")
registry_files = json.loads(registry_raw)["files"]
archive_rows = original_archive["sources"]
missing_registry_indices = [272, 501, 669, 670, 1537, 1958, 2416]
if (len(registry_files) != 2853 or len(archive_rows) != 2846
        or [i for i, item in enumerate(registry_files) if not item.get("sha256")] != missing_registry_indices
        or [x["original"] for x in archive_rows] != [i for i in range(2853) if i not in missing_registry_indices]
        or any(registry_files[x["original"]].get("sha256") != x["sha256"] for x in archive_rows)):
    raise ValueError("Original registry identity inventory mismatch")
expected_originals = {x["sha256"] for x in archive_rows}
if len(expected_originals) != 2462:
    raise ValueError("Original source object count mismatch")
seen_originals = set()
with tarfile.open(original_archive_dir / "objects.tar.gz", "r:gz") as archive:
    for member in archive:
        if not member.isfile() or member.name not in expected_originals or member.name in seen_originals:
            raise ValueError("Original source archive member mismatch")
        inp = archive.extractfile(member)
        if inp is None or hashlib.sha256(inp.read()).hexdigest() != member.name:
            raise ValueError("Original source object hash mismatch")
        seen_originals.add(member.name)
if seen_originals != expected_originals:
    raise ValueError("Original source archive incomplete")


deleted_dir = root / "recovered/deleted-source-base-objects"
deleted_rows = json.loads((deleted_dir / "MANIFEST.json").read_bytes())["rows"]
if [row["registry_index"] for row in deleted_rows] != missing_registry_indices:
    raise ValueError("Deleted base identity inventory mismatch")
for row in deleted_rows:
    original = registry_files[row["registry_index"]]
    if (original["state"] != "deleted" or original["source_id"] != row["source_id"]
            or original["base_sha256"] != row["base_sha256"]
            or hashlib.sha256((deleted_dir / row["base_sha256"]).read_bytes()).hexdigest() != row["base_sha256"]):
        raise ValueError("Deleted base source hash mismatch")
if len({row["base_sha256"] for row in deleted_rows}) != 6:
    raise ValueError("Deleted base object count mismatch")


selected_dir = root / "recovered/selected-revision-registry-sources"
selected = json.loads((selected_dir / "MANIFEST.json").read_bytes())
selected_rows = selected["rows"]
selected_absent = selected["absent_or_nonblob_indices"]
if (selected["revision"] != "ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04"
        or selected["registry_sha256"] != "d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f"
        or len(selected_rows) != 2193 or len(selected_absent) != 660
        or sorted([r["registry_index"] for r in selected_rows] + selected_absent) != list(range(2853))
        or any(registry_files[r["registry_index"]]["source_id"] != r["source_id"]
               or registry_files[r["registry_index"]]["path"] != r["path"] for r in selected_rows)):
    raise ValueError("Selected revision source identity mismatch")
selected_by_sha = {}
for row in selected_rows:
    previous = selected_by_sha.setdefault(row["sha256"], row["git_blob"])
    if previous != row["git_blob"]:
        raise ValueError("Selected revision blob identity mismatch")
if (len(selected_by_sha) != 1824 or len(set(selected_by_sha.values())) != 1824
        or hashlib.sha256((selected_dir / "objects.tar.gz").read_bytes()).hexdigest() != selected["objects_archive_sha256"]):
    raise ValueError("Selected revision archive mismatch")
selected_seen = set()
with tarfile.open(selected_dir / "objects.tar.gz", "r:gz") as archive:
    for member in archive:
        if not member.isfile() or member.name not in selected_by_sha or member.name in selected_seen:
            raise ValueError("Selected revision archive member mismatch")
        inp = archive.extractfile(member)
        if inp is None:
            raise ValueError("Selected revision member unreadable")
        data = inp.read()
        git_oid = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        if hashlib.sha256(data).hexdigest() != member.name or git_oid != selected_by_sha[member.name]:
            raise ValueError("Selected revision source hash mismatch")
        selected_seen.add(member.name)
if selected_seen != set(selected_by_sha):
    raise ValueError("Selected revision source archive incomplete")


commit_bytes = (selected_dir / "selected.commit").read_bytes()
if (hashlib.sha1(b"commit " + str(len(commit_bytes)).encode() + b"\0" + commit_bytes).hexdigest() != selected["revision"]
        or commit_bytes.split(b"\n", 1)[0] != b"tree " + selected["git_root_tree"].encode()
        or hashlib.sha256((selected_dir / "trees.tar.gz").read_bytes()).hexdigest() != selected["trees_archive_sha256"]):
    raise ValueError("Selected revision commit or tree archive mismatch")
selected_trees = {}
with tarfile.open(selected_dir / "trees.tar.gz", "r:gz") as archive:
    for member in archive:
        if not member.isfile() or member.name in selected_trees:
            raise ValueError("Selected revision tree member mismatch")
        inp = archive.extractfile(member)
        if inp is None:
            raise ValueError("Selected revision tree unreadable")
        data = inp.read()
        if hashlib.sha1(b"tree " + str(len(data)).encode() + b"\0" + data).hexdigest() != member.name:
            raise ValueError("Selected revision tree hash mismatch")
        entries = {}
        pos = 0
        while pos < len(data):
            space = data.index(b" ", pos)
            end = data.index(b"\0", space + 1)
            name = data[space + 1:end]
            if name in entries or end + 21 > len(data):
                raise ValueError("Malformed selected revision tree")
            entries[name] = (data[pos:space], data[end + 1:end + 21].hex())
            pos = end + 21
        selected_trees[member.name] = entries
if len(selected_trees) != selected["git_tree_objects"]:
    raise ValueError("Selected revision tree count mismatch")
for row in selected_rows:
    current = selected["git_root_tree"]
    parts = row["path"].encode("utf-8", "surrogateescape").split(b"/")
    for i, part in enumerate(parts):
        entry = selected_trees[current][part]
        if i + 1 == len(parts):
            if entry[1] != row["git_blob"] or int(entry[0], 8) != int(row["mode"], 8):
                raise ValueError("Selected revision path membership mismatch")
        else:
            if int(entry[0], 8) != 0o40000:
                raise ValueError("Selected revision parent is not a tree")
            current = entry[1]


for index in selected_absent:
    current = selected["git_root_tree"]
    parts = registry_files[index]["path"].encode("utf-8", "surrogateescape").split(b"/")
    proven = False
    for i, part in enumerate(parts):
        entry = selected_trees[current].get(part)
        if entry is None:
            proven = True
            break
        mode = int(entry[0], 8)
        if i + 1 == len(parts):
            proven = mode in (0o40000, 0o160000)
            break
        if mode != 0o40000:
            proven = True
            break
        current = entry[1]
    if not proven:
        raise ValueError("Selected revision absent or non-blob claim is false")


coverage_dir = root / "recovered/hash-matched-dependencies"
coverage = json.loads((coverage_dir / "DEPENDENCY-COVERAGE.json").read_bytes())
later = json.loads((coverage_dir / "LATER-MATCHES.json").read_bytes())
crosscheck = json.loads((coverage_dir / "SOURCE-ARCHIVE-CROSSCHECK.json").read_bytes())
prior_missing = {r["sha256"] for r in coverage["references"] if r["availability"] == "not_located_by_hash"}
remaining_missing = prior_missing - {r["sha256"] for r in later["matches"]}
if (len(remaining_missing) != crosscheck["previously_unmatched_distinct"]
        or len(expected_originals) != crosscheck["original_registry_source_objects_checked"]
        or len(selected_by_sha) != crosscheck["selected_revision_source_objects_checked"]
        or sorted(remaining_missing & expected_originals) != crosscheck["original_archive_matching_hashes"]
        or sorted(remaining_missing & set(selected_by_sha)) != crosscheck["selected_archive_matching_hashes"]):
    raise ValueError("Source archive dependency crosscheck mismatch")


subprocess.run([sys.executable, str(root / "recovered/draft-change-source-archive/verify_archive.py")], check=True)


frontier_dir = root / "recovered/final-frontier-entry-2708"
frontier = json.loads((frontier_dir / "MANIFEST.json").read_bytes())
frontier_data = (frontier_dir / "OBSERVED-QUEUE-4151-ENTRY.json").read_bytes()
frontier_entry = json.loads(frontier_data)
if (hashlib.sha256(frontier_data).hexdigest() != frontier["observed_normalized_sha256"]
        or frontier["observed_normalized_sha256"] != "e69931d1c91aea60f75c640e4c0891910a90bac29554542fab9a095cdc518b10"
        or frontier_entry["contract_id"] != frontier["contract_id"]
        or frontier_entry["lens"] != 4 or len(frontier_entry.get("granular_contract_views", [])) != 0
        or len(frontier_entry.get("remaining_evidence", [])) != 1
        or frontier_writers != frontier["writer_change_ordinals"]
        or frontier["reported_checkpoints"] != list(range(3817, 3822))):
    raise ValueError("Frontier entry trace mismatch")
frontier_output = None
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        record = json.loads(line)
        if record["transcript_ordinal"] == frontier["observed_output_ordinal"]:
            frontier_output = record
            break
if (frontier_output is None or "CLOSURE-QUEUE-4151.json" not in frontier_output["command_text"]
        or "['entries'][2708]" not in frontier_output["command_text"]
        or hashlib.sha256(frontier_output["output_text"].encode()).hexdigest() != frontier["observed_output_sha256"]
        or json.loads(frontier_output["output_text"]) != frontier_entry):
    raise ValueError("Frontier entry printed output mismatch")
if any(not all(any(r.get("mode") == mode and r.get("status") == "pass"
                       for r in item["records"]) for mode in ("--preflight", "--apply", "--verify"))
       for item in reports if item["checkpoint"] in frontier["reported_checkpoints"]):
    raise ValueError("Frontier entry report mismatch")


fields_bytes = (frontier_dir / frontier["literal_command_fields"]["file"]).read_bytes()
if hashlib.sha256(fields_bytes).hexdigest() != frontier["literal_command_fields"]["sha256"]:
    raise ValueError("Frontier literal field file hash mismatch")
fields = json.loads(fields_bytes)["entries"]
if [e["checkpoint"] for e in fields] != list(range(3817, 3822)):
    raise ValueError("Frontier literal field checkpoints mismatch")
command_records = {}
with (root / "recovered/late-audit-write-commands/COMMANDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        record = json.loads(line)
        command_records[record["transcript_command_ordinal"]] = record
for entry in fields:
    record = command_records[entry["command_ordinal"]]
    command = record["command_text"]
    if hashlib.sha256(command.encode()).hexdigest() != entry["command_sha256"]:
        raise ValueError("Frontier literal command hash mismatch")
    source = command.split("@'\n", 1)[1].rsplit("\n'@", 1)[0]
    parsed = ast.parse(source)
    groups = {}
    remaining = None
    for node in ast.walk(parsed):
        if (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
                and node.func.attr == "update" and isinstance(node.func.value, ast.Name)
                and node.func.value.id in ("v", "d", "p", "o")):
            literals = {}
            for keyword in node.keywords:
                try:
                    literals[keyword.arg] = ast.literal_eval(keyword.value)
                except (ValueError, TypeError):
                    pass
            if literals:
                groups[node.func.value.id] = literals
        if (isinstance(node, ast.Assign) and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "remaining"):
            remaining = ast.literal_eval(node.value)
    if groups != entry["literal_update_fields"] or remaining != entry["remaining_literal"]:
        raise ValueError("Frontier literal fields differ from recorded commands")


tail = frontier["first_update_source_evidence"]
tail_source = (frontier_dir / tail["source_sha256"]).read_bytes()
tail_display = (frontier_dir / "ERRANTRY-TAIL-DISPLAY.txt").read_bytes()
if (hashlib.sha256(tail_source).hexdigest() != tail["source_sha256"]
        or hashlib.sha1(b"blob " + str(len(tail_source)).encode() + b"\0" + tail_source).hexdigest() != tail["source_git_blob_sha1"]
        or hashlib.sha256(tail_display).hexdigest() != tail["tail_display_sha256"]):
    raise ValueError("Frontier tail evidence checksum mismatch")
source_lines = tail_source.decode("utf-8").splitlines()
displayed_lines = tail_display.decode("utf-8").splitlines()
if (len(source_lines) != 827 or len(displayed_lines) != 382
        or any(line != f"{n}: {source_lines[n - 1]}" for n, line in enumerate(displayed_lines, 446))):
    raise ValueError("Frontier tail displayed ranges mismatch")
receipt_bytes = (root / tail["historical_receipt"]).read_bytes()
if hashlib.sha256(receipt_bytes).hexdigest() != "b9cff73df660fb081c0937df71b83bfd17f32c77bbc479279bb304b9cd5fa1ac":
    raise ValueError("Frontier historical receipt checksum mismatch")
receipt = json.loads(receipt_bytes)
roles = [r for r in receipt["source_roles"] if r.get("sha256") == tail["source_sha256"]]
if len(roles) != 1 or roles[0]["actual_compared_ranges"] != [[1, 445]]:
    raise ValueError("Frontier historical receipt range mismatch")
late_records = {}
with (root / "recovered/late-command-history/RECORDS.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] in (418957, 418962):
            late_records[item["transcript_ordinal"]] = item
if (set(late_records) != {418957, 418962}
        or any(item["recorded_exit_code"] != 0 for item in late_records.values())
        or late_records[418962]["output_text"].encode() != tail_display
        or "range(446,828)" not in late_records[418962]["command_text"]
        or "REGISTRATION-ERRANTRY-TAIL-L4-3817.json" not in late_records[418957]["command_text"]
        or tail["source_sha256"] not in late_records[418957]["output_text"]):
    raise ValueError("Frontier tail recorded command mismatch")


template_meta = frontier["tail_registration_template"]
template_bytes = (frontier_dir / template_meta["file"]).read_bytes()
if hashlib.sha256(template_bytes).hexdigest() != template_meta["sha256"]:
    raise ValueError("Frontier registration template checksum mismatch")
template = json.loads(template_bytes)["template"]
extracted = json.loads(subprocess.check_output([sys.executable, str(frontier_dir / template_meta["extractor"])]))
if (template != extracted or template["at"] != "UNRECOVERED_ORIGINAL_TIMESTAMP"
        or template_meta["original_timestamp_recovered"] is not False
        or template_meta["original_hash_recovered"] is not False):
    raise ValueError("Frontier registration template derivation mismatch")


subprocess.run([sys.executable, str(root / "recovered/frontier-source-trace/verify_portable_frontier.py")], check=True)


transport = gzip.decompress((root / "recovered/original-transport-manifest.json.gz").read_bytes())
if hashlib.sha256(transport).hexdigest() != "c3e04095a9e4cb3e2da6ee2a4f561c811ebeca445a50461ddb63b2d426ed6858":
    raise ValueError("Original transport manifest checksum mismatch")
transport_entries = {item["path"]: item for item in json.loads(transport)["entries"]}
root712_dir = root / "recovered/l13-root-712"
root712 = json.loads((root712_dir / "MANIFEST.json").read_bytes())
if len(root712["prior_support_files"]) != 11:
    raise ValueError("Lens 13 root 712 support inventory mismatch")
root712_support = {}
for item in root712["prior_support_files"]:
    data = (root712_dir / "prior-support" / item["name"]).read_bytes()
    expected = transport_entries[item["original_manifest_path"]]
    if (hashlib.sha256(data).hexdigest() != item["sha256"]
            or expected["sha256"] != item["sha256"] or expected["bytes"] != len(data)):
        raise ValueError("Lens 13 root 712 original support checksum mismatch")
    root712_support[item["name"]] = json.loads(data)
source_hashes = {item["original"]: item["sha256"] for item in root712["source_objects"]}
if len(root712["delegated_partial_union_evidence"]) != 2:
    raise ValueError("Lens 13 root 712 union inventory mismatch")
for item in root712["delegated_partial_union_evidence"]:
    raw = root712_support[item["read"]]
    number = item["read"].split("-")[2].split(".")[0]
    verification = root712_support[f"VERIFY-PL13-{number}.json"]
    registration = root712_support[f"REGISTRATION-PL13-{number}.json"]
    names = {x["name"]: x["sha256"] for x in root712["prior_support_files"]}
    if (raw["registration"]["sha256"] != names[f"REGISTRATION-PL13-{number}.json"]
            or verification["read"]["sha256"] != names[item["read"]]
            or verification["registration"]["sha256"] != raw["registration"]["sha256"]
            or raw["lens"] != 13 or registration["lens"] != 13 or verification["lens"] != 13):
        raise ValueError("Lens 13 root 712 delegated record links mismatch")
    matching = [r for r in raw["rows"] if r["original"] == item["delegated_original"]]
    unions = [u for u in raw["partial_read_union"] if u["original"] == item["delegated_original"]]
    if (len(matching) != 1 or matching[0]["sha256"] != source_hashes[item["root_original"]]
            or len(unions) != item["prior_receipts"]
            or unions[-1]["union"] != item["final_recorded_union"]):
        raise ValueError("Lens 13 root 712 delegated source or union mismatch")
    for union in unions:
        name = union["read"]["path"].replace("\\", "/").rsplit("/", 1)[-1]
        if union["read"]["sha256"] != names[name] or name not in root712_support:
            raise ValueError("Lens 13 root 712 partial receipt mismatch")
        partial = root712_support[name]
        partial_rows = [r for r in partial["rows"] if r["original"] == item["delegated_original"]]
        if (len(partial_rows) != 1 or partial["registration"] != raw["registration"]
                or partial_rows[0]["sha256"] != matching[0]["sha256"]
                or partial_rows[0]["fresh_personal_assessed_ranges"] != union["previous_personal_ranges"]
                or union["reread_claimed"] is not False):
            raise ValueError("Lens 13 root 712 partial range evidence mismatch")


root720_dir = root / "recovered/l13-root-720"
root720 = json.loads((root720_dir / "MANIFEST.json").read_bytes())
if len(root720["prior_support_files"]) != 14 or len(root720["delegated_source_evidence"]) != 3:
    raise ValueError("Lens 13 root 720 support inventory mismatch")
root720_support = {}
root720_hashes = {}
for item in root720["prior_support_files"]:
    data = (root720_dir / "prior-support" / item["name"]).read_bytes()
    expected = transport_entries[item["original_manifest_path"]]
    if (hashlib.sha256(data).hexdigest() != item["sha256"]
            or expected["sha256"] != item["sha256"] or expected["bytes"] != len(data)):
        raise ValueError("Lens 13 root 720 original support checksum mismatch")
    root720_support[item["name"]] = json.loads(data)
    root720_hashes[item["name"]] = item["sha256"]
root720_sources = {r["original"]: r["sha256"] for r in root720["source_objects"]}
for item in root720["delegated_source_evidence"]:
    raw = root720_support[item["read"]]
    number = item["read"].split("-")[2].split(".")[0]
    verification = root720_support[f"VERIFY-PL13-{number}.json"]
    registration = root720_support[f"REGISTRATION-PL13-{number}.json"]
    row = raw[item["section"]][item["row_index"]]
    if (raw["lens"] != 13 or registration["lens"] != 13 or verification["lens"] != 13
            or raw["registration"]["sha256"] != root720_hashes[f"REGISTRATION-PL13-{number}.json"]
            or verification["read"]["sha256"] != root720_hashes[item["read"]]
            or verification["registration"]["sha256"] != raw["registration"]["sha256"]
            or row["original"] != item["delegated_original"]
            or row["sha256"] != root720_sources[item["root_original"]]):
        raise ValueError("Lens 13 root 720 delegated source links mismatch")


root719_connected = l13_719["prior_connected_162_support"]
root719_new = [x for x in l13_719["prior_support_files"] if "original_manifest_path" in x]
if len(root719_new) != 4:
    raise ValueError("Lens 13 root 719 connected support inventory mismatch")
root719_named = {}
root719_hashes = {}
for item in root719_new:
    data = (l13_719_dir / "prior-support" / item["sha256"]).read_bytes()
    orig = transport_entries[item["original_manifest_path"]]
    if orig["sha256"] != item["sha256"] or orig["bytes"] != len(data):
        raise ValueError("Lens 13 root 719 original connected support mismatch")
    root719_named[item["name"]] = json.loads(data)
    root719_hashes[item["name"]] = item["sha256"]
root719_raw = root719_named["READ-PL13-162.json"]
root719_ver = root719_named["VERIFY-PL13-162.json"]
root719_reg = root719_named["REGISTRATION-PL13-162.json"]
root719_row = root719_raw[root719_connected["section"]][root719_connected["row_index"]]
root719_source = next(x for x in l13_719["source_objects"] if x["original"] == root719_connected["root_original"])
if (root719_raw["lens"] != 13 or root719_ver["lens"] != 13 or root719_reg["lens"] != 13
        or root719_raw["registration"]["sha256"] != root719_hashes["REGISTRATION-PL13-162.json"]
        or root719_ver["read"]["sha256"] != root719_hashes["READ-PL13-162.json"]
        or root719_ver["registration"]["sha256"] != root719_raw["registration"]["sha256"]
        or root719_row["original"] != root719_connected["delegated_original"]
        or root719_row["sha256"] != root719_source["sha256"]):
    raise ValueError("Lens 13 root 719 connected source links mismatch")


root715_dir = root / "recovered/l13-root-715"
root715 = json.loads((root715_dir / "MANIFEST.json").read_bytes())
meta715 = root715["prior_row_extracts"]
extract715 = (root715_dir / meta715["file"]).read_bytes()
excerpt715 = (root715_dir / meta715["display_excerpt"]).read_bytes()
if (hashlib.sha256(extract715).hexdigest() != meta715["sha256"]
        or hashlib.sha256(excerpt715).hexdigest() != meta715["display_excerpt_sha256"]):
    raise ValueError("Lens 13 root 715 prior row extract checksum mismatch")
parsed715 = []
pending715 = None
for line in excerpt715.decode("utf8").splitlines():
    if line.startswith("PRIOR "):
        match = re.match(r"^PRIOR (\d+) (\S+) packet (\{.*?\}) plan (\{.*?\}) writer (\{.*\})$", line)
        if match is None or pending715 is not None:
            raise ValueError("Lens 13 root 715 prior header mismatch")
        pending715 = {"root_original": int(match.group(1)), "prior_receipt_name": match.group(2),
                      "prior_packet": ast.literal_eval(match.group(3)),
                      "prior_plan": ast.literal_eval(match.group(4)),
                      "prior_writer": ast.literal_eval(match.group(5))}
    elif line.startswith("ROW ") and pending715 is not None:
        match = re.match(r"^ROW (\w+) (\d+) (\{.*\})$", line)
        if match is None:
            raise ValueError("Lens 13 root 715 prior row mismatch")
        pending715.update(section=match.group(1), row_index=int(match.group(2)),
                          row=json.loads(match.group(3)))
        parsed715.append(pending715)
        pending715 = None
    else:
        raise ValueError("Unexpected Lens 13 root 715 excerpt line")
if pending715 is not None or parsed715 != json.loads(extract715) or len(parsed715) != 5:
    raise ValueError("Lens 13 root 715 prior row reconstruction mismatch")
discovery715 = json.loads((root715_dir / root715["discovery_sha256"]).read_bytes())
hits715 = {(h["original"], h["receipt"]["path"].rsplit("/", 1)[-1])
           for h in discovery715["exact_hash_prior_hits"]}
sources715 = {r["original"]: r["sha256"] for r in root715["source_objects"]}
expected715 = {2736: (2155, 0), 2737: (2165, 0), 2738: (2166, 1),
               2739: (2167, 0), 2740: (2591, 7)}
for item in parsed715:
    n = item["root_original"]
    old, index = expected715[n]
    if ((n, item["prior_receipt_name"]) not in hits715 or item["section"] != "ordered"
            or item["row_index"] != index or item["row"]["source"]["original"] != old
            or item["row"]["source"]["sha256"] != sources715[n]):
        raise ValueError("Lens 13 root 715 prior source identity mismatch")


root714_dir = root / "recovered/l13-root-714"
root714 = json.loads((root714_dir / "MANIFEST.json").read_bytes())
meta714 = root714["prior_row_extracts"]
extract714 = (root714_dir / meta714["file"]).read_bytes()
excerpt714 = (root714_dir / meta714["display_excerpt"]).read_bytes()
if (hashlib.sha256(extract714).hexdigest() != meta714["sha256"]
        or hashlib.sha256(excerpt714).hexdigest() != meta714["display_excerpt_sha256"]):
    raise ValueError("Lens 13 root 714 prior row checksum mismatch")
sources714 = {r["sha256"]: r["original"] for r in root714["source_objects"]}
parsed714 = []
pending714 = None
for line in excerpt714.decode("utf8").splitlines():
    if line.startswith("OLD "):
        match = re.match(r"^OLD (\d+) ROOTMETA (\{.*\})$", line)
        if match is None or pending714 is not None:
            raise ValueError("Lens 13 root 714 prior header mismatch")
        pending714 = {"prior_sequence": int(match.group(1)),
                      "prior_root_metadata": json.loads(match.group(2))}
    elif line.startswith("ROW ") and pending714 is not None:
        match = re.match(r"^ROW (\d+) (\{.*\})$", line)
        if match is None:
            raise ValueError("Lens 13 root 714 prior row mismatch")
        row = json.loads(match.group(2))
        pending714.update(root_original=sources714[row["source"]["sha256"]],
                          row_index=int(match.group(1)), row=row)
        parsed714.append(pending714)
        pending714 = None
    else:
        raise ValueError("Unexpected Lens 13 root 714 excerpt line")
if pending714 is not None or parsed714 != json.loads(extract714) or len(parsed714) != 2:
    raise ValueError("Lens 13 root 714 prior row reconstruction mismatch")
discovery714 = json.loads((root714_dir / root714["discovery_sha256"]).read_bytes())
hits714 = {(h["original"], h["receipt"]["path"].rsplit("/", 1)[-1])
           for h in discovery714["exact_hash_prior_hits"]}
expected714 = {2729: (646, 2386), 2732: (633, 2355)}
for item in parsed714:
    seq, old = expected714[item["root_original"]]
    if ((item["root_original"], f"READ-L13-PARALLEL-{seq}.json") not in hits714
            or item["prior_sequence"] != seq or item["row_index"] != 1
            or item["row"]["source"]["original"] != old):
        raise ValueError("Lens 13 root 714 prior source identity mismatch")


meta720 = root720["prior_row_extracts"]
extract720 = (root720_dir / meta720["file"]).read_bytes()
excerpt720 = (root720_dir / meta720["display_excerpt"]).read_bytes()
if (hashlib.sha256(extract720).hexdigest() != meta720["sha256"]
        or hashlib.sha256(excerpt720).hexdigest() != meta720["display_excerpt_sha256"]):
    raise ValueError("Lens 13 root 720 prior row checksum mismatch")
parsed720 = []
pending720 = None
for line in excerpt720.decode("utf8").splitlines():
    if line.startswith("PRIOR "):
        match = re.match(r"^PRIOR (\d+) (\S+) keys (\[.*\])$", line)
        if match is None or pending720 is not None:
            raise ValueError("Lens 13 root 720 prior header mismatch")
        pending720 = {"root_original": int(match.group(1)),
                      "prior_receipt_path": match.group(2),
                      "printed_root_keys": ast.literal_eval(match.group(3))}
    elif line.startswith("ROW ") and pending720 is not None:
        match = re.match(r"^ROW (\w+) (\d+) (\{.*\})$", line)
        if match is None:
            raise ValueError("Lens 13 root 720 prior row mismatch")
        pending720.update(section=match.group(1), row_index=int(match.group(2)),
                          row=json.loads(match.group(3)))
        parsed720.append(pending720)
        pending720 = None
    else:
        raise ValueError("Unexpected Lens 13 root 720 excerpt line")
if pending720 is not None or parsed720 != json.loads(extract720) or len(parsed720) != 4:
    raise ValueError("Lens 13 root 720 prior row reconstruction mismatch")
discovery720 = json.loads((root720_dir / root720["discovery_sha256"]).read_bytes())
hits720 = {(h["original"], h["receipt"]["path"]) for h in discovery720["exact_hash_prior_hits"]}
expected720 = {2807: ("rows", 0, 1152), 2815: ("connected", 2, 2815),
               2819: ("rows", 1, 366), 2820: ("ordered", 0, 1887)}
for item in parsed720:
    n = item["root_original"]
    section, index, old = expected720[n]
    source = item["row"].get("source", item["row"])
    if ((n, item["prior_receipt_path"]) not in hits720 or item["section"] != section
            or item["row_index"] != index or source["original"] != old
            or source["sha256"] != root720_sources[n]):
        raise ValueError("Lens 13 root 720 prior source identity mismatch")
    prior = next((e for e in root720["delegated_source_evidence"] if e["root_original"] == n), None)
    if prior is not None and root720_support[prior["read"]][section][index] != item["row"]:
        raise ValueError("Lens 13 root 720 printed and original delegated rows differ")


row2805_meta = root720["last_prior_row_recovery"]
row2805_files = {}
for key in ("original_printed_prefix", "later_output_excerpt", "later_normalized_context"):
    data = (root720_dir / row2805_meta[key]).read_bytes()
    if hashlib.sha256(data).hexdigest() != row2805_meta[key + "_sha256"]:
        raise ValueError("Lens 13 root 720 recovered final row checksum mismatch")
    row2805_files[key] = data
row2805 = json.loads(row2805_files["later_normalized_context"])
if json.loads(row2805_files["later_output_excerpt"]) != row2805:
    raise ValueError("Lens 13 root 720 later row output mismatch")
row2805_prior = row2805["source_contract_read"]
if (not json.dumps(row2805_prior, ensure_ascii=True).startswith(row2805_files["original_printed_prefix"].decode("utf8"))
        or row2805_prior["source"] != row2805["source"]
        or row2805_prior["source"] != l13_719_reg["sources"][17]
        or row2805_prior["registration"]["sha256"] != l13_719["registration_sha256"]
        or row2805["read_pointer"] != "/ordered/17"
        or row2805["packet"]["sha256"] != l13_719["packet_sha256_recorded_from_later_queue"]
        or row2805["root_adoption"]["sha256"] != l13_719["adoption_sha256_recorded_from_later_queue"]
        or (2805, row2805["root_adoption"]["path"]) not in hits720):
    raise ValueError("Lens 13 root 720 recovered final row provenance mismatch")


proposal_meta719 = l13_719["proposal_literal_evidence"]
proposal_bytes719 = (l13_719_dir / proposal_meta719["file"]).read_bytes()
if hashlib.sha256(proposal_bytes719).hexdigest() != proposal_meta719["sha256"]:
    raise ValueError("Lens 13 root 719 proposal literal checksum mismatch")
proposal719 = json.loads(proposal_bytes719)
record719 = None
with (root / "recovered/audit-write-command-candidates/COMMANDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_command_ordinal"] == proposal_meta719["command_ordinal"]:
            record719 = item
            break
if (record719 is None or hashlib.sha256(record719["command_text"].encode()).hexdigest()
        != proposal719["command_sha256"]):
    raise ValueError("Lens 13 root 719 recorded proposal command mismatch")
source719 = record719["command_text"].split("@'\n", 1)[1].split("\n'@", 1)[0]
parsed_command719 = ast.parse(source719)
assignments719 = {}
for node in parsed_command719.body:
    if (isinstance(node, ast.Assign) and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name) and node.targets[0].id in ("facts", "question")):
        assignments719[node.targets[0].id] = node.value
facts719 = ast.literal_eval(assignments719["facts"])
question719 = {}
for key_node, value_node in zip(assignments719["question"].keys, assignments719["question"].values):
    key = ast.literal_eval(key_node)
    if key == "all20_reconciliation":
        if ast.unparse(value_node) != "{'families': [f'P{n:02d}' for n in range(1, 21)], 'complete': False, 'navigation_only': True}":
            raise ValueError("Lens 13 root 719 family expression mismatch")
        question719[key] = {"families": [f"P{n:02d}" for n in range(1, 21)],
                            "complete": False, "navigation_only": True}
    else:
        question719[key] = ast.literal_eval(value_node)
proposal_output719 = None
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] == 55491:
            proposal_output719 = json.loads(item["output_text"])
            break
if (proposal_output719 is None or proposal_output719["packet"]["sha256"] != l13_719["packet_sha256_recorded_from_later_queue"]
        or proposal_output719["plan"]["sha256"] != l13_719["plan_sha256_recorded"]
        or l13_719["plan_original_recovered"] is not False):
    raise ValueError("Lens 13 root 719 printed plan reference mismatch")
if (proposal719["source_assessments"] != [{"original": k, "assessment": facts719[k]} for k in sorted(facts719)]
        or sorted(facts719) != list(range(2787, 2805)) or proposal719["proposed_question"] != question719
        or question719["contract_id"] != "ROOT-L13-2792-NATIVE-BATCH-RESET-INVARIANT"
        or question719["runtime_failure_observed"] is not False):
    raise ValueError("Lens 13 root 719 proposal literal reconstruction mismatch")


proposal_specs = {714: (54389, 2729, 2735), 720: (55663, 2805, 2822)}
proposal_commands = {}
with (root / "recovered/audit-write-command-candidates/COMMANDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_command_ordinal"] in {v[0] for v in proposal_specs.values()}:
            proposal_commands[item["transcript_command_ordinal"]] = item
proposal_outputs = {}
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] in {v[0] for v in proposal_specs.values()}:
            text = item["output_text"]
            proposal_outputs[item["transcript_ordinal"]] = json.JSONDecoder().raw_decode(text, text.index("{"))[0]
for root_number, (ordinal, first, last) in proposal_specs.items():
    directory = root / f"recovered/l13-root-{root_number}"
    manifest = json.loads((directory / "MANIFEST.json").read_bytes())
    meta = manifest["proposal_literal_evidence"]
    data = (directory / meta["file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != meta["sha256"]:
        raise ValueError("Lens 13 proposal literal checksum mismatch")
    proposal = json.loads(data)
    command = proposal_commands[ordinal]["command_text"]
    if hashlib.sha256(command.encode()).hexdigest() != proposal["command_sha256"]:
        raise ValueError("Lens 13 proposal command checksum mismatch")
    parsed = ast.parse(command.split("@'\n", 1)[1].split("\n'@", 1)[0])
    assignments = {node.targets[0].id: node.value for node in parsed.body
                   if isinstance(node, ast.Assign) and len(node.targets) == 1
                   and isinstance(node.targets[0], ast.Name) and node.targets[0].id in ("facts", "question")}
    facts = ast.literal_eval(assignments["facts"])
    question = ast.literal_eval(assignments["question"]) if "question" in assignments else None
    output = proposal_outputs[ordinal]
    if (sorted(facts) != list(range(first, last + 1))
            or proposal["source_assessments"] != [{"original": k, "assessment": facts[k]} for k in sorted(facts)]
            or proposal["proposed_question"] != question
            or output["packet"]["sha256"] != manifest["packet_sha256_recorded_from_output"]
            or output["plan"]["sha256"] != manifest["plan_sha256_recorded_from_output"]
            or manifest["plan_original_recovered"] is not False
            or (question is not None and question.get("runtime_failure_observed") is not False)):
        raise ValueError("Lens 13 proposal literal reconstruction mismatch")


proposal_specs_private = {710: (2704, 2708), 711: (2709, 2714),
                          712: (2715, 2720), 721: (2823, 2835)}
for root_number, (first, last) in proposal_specs_private.items():
    directory = root / f"recovered/l13-root-{root_number}"
    manifest = json.loads((directory / "MANIFEST.json").read_bytes())
    meta = manifest["proposal_literal_evidence"]
    data = (directory / meta["file"]).read_bytes()
    excerpt = (directory / meta["assignment_excerpt"]).read_bytes()
    printed = (directory / meta["printed_output_file"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != meta["sha256"]
            or hashlib.sha256(excerpt).hexdigest() != meta["assignment_excerpt_sha256"]
            or hashlib.sha256(printed).hexdigest() != meta["printed_output_sha256"]):
        raise ValueError("Private proposal excerpt checksum mismatch")
    proposal = json.loads(data)
    assignments = {}
    for node in ast.parse(excerpt.decode("utf8")).body:
        if (not isinstance(node, ast.Assign) or len(node.targets) != 1
                or not isinstance(node.targets[0], ast.Name)
                or node.targets[0].id not in ("facts", "question")
                or node.targets[0].id in assignments):
            raise ValueError("Unexpected private proposal excerpt statement")
        assignments[node.targets[0].id] = node.value
    facts = ast.literal_eval(assignments["facts"])
    question = None
    if "question" in assignments:
        question = {}
        for key_node, value_node in zip(assignments["question"].keys, assignments["question"].values):
            key = ast.literal_eval(key_node)
            if key == "all20_reconciliation":
                if ast.unparse(value_node) != "{'families': [f'P{n:02d}' for n in range(1, 21)], 'complete': False, 'navigation_only': True}":
                    raise ValueError("Private proposal family expression mismatch")
                question[key] = {"families": [f"P{n:02d}" for n in range(1, 21)],
                                 "complete": False, "navigation_only": True}
            else:
                question[key] = ast.literal_eval(value_node)
    printed_refs = json.loads(printed)
    if (sorted(facts) != list(range(first, last + 1))
            or proposal["source_assessments"] != [{"original": k, "assessment": facts[k]} for k in sorted(facts)]
            or proposal["proposed_question"] != question
            or printed_refs["packet"] != proposal["printed_packet"]
            or printed_refs["plan"] != proposal["printed_plan"]
            or printed_refs["packet"]["sha256"] != manifest["packet_sha256_recorded_from_output"]
            or printed_refs["plan"]["sha256"] != manifest["plan_sha256_recorded_from_output"]
            or manifest["plan_original_recovered"] is not (root_number == 710)):
        raise ValueError("Private proposal excerpt reconstruction mismatch")
    if root_number == 710 and hashlib.sha256((directory / manifest["plan_sha256"]).read_bytes()).hexdigest() != printed_refs["plan"]["sha256"]:
        raise ValueError("Root 710 recovered original plan mismatch")


conditional_meta715 = root715["conditional_proposal_assessments"]
conditional_bytes715 = (root715_dir / conditional_meta715["file"]).read_bytes()
if hashlib.sha256(conditional_bytes715).hexdigest() != conditional_meta715["sha256"]:
    raise ValueError("Root 715 conditional assessment checksum mismatch")
conditional715 = json.loads(conditional_bytes715)
command715 = None
with (root / "recovered/audit-write-command-candidates/COMMANDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_command_ordinal"] == 54682:
            command715 = item
            break
if (command715 is None or command715["recorded_exit_code"] != 0
        or hashlib.sha256(command715["command_text"].encode()).hexdigest() != conditional715["command_sha256"]):
    raise ValueError("Root 715 preparation command mismatch")
source715 = command715["command_text"].split("@'\n", 1)[1].split("\n'@", 1)[0]
parsed_command715 = ast.parse(source715)
assignments715 = {node.targets[0].id: node.value for node in parsed_command715.body
                  if isinstance(node, ast.Assign) and len(node.targets) == 1
                  and isinstance(node.targets[0], ast.Name) and node.targets[0].id in ("fresh", "expr")}
fresh715 = ast.literal_eval(assignments715["fresh"])
expr715 = assignments715["expr"]
if (not isinstance(expr715, ast.BinOp) or not isinstance(expr715.op, ast.Add)
        or ast.unparse(expr715.left) != "repr(fresh)"):
    raise ValueError("Root 715 fact construction shape mismatch")
suffix715 = ast.literal_eval(expr715.right)
if hashlib.sha256(suffix715.encode()).hexdigest() != "055413d879d23ae84e46120d07f4f45813f5775242cea683e3099b2361deebc1":
    raise ValueError("Root 715 fact construction rule mismatch")
additions715 = {}
for node in ast.parse(suffix715).body:
    if isinstance(node, ast.AugAssign):
        if (not isinstance(node.target, ast.Subscript) or not isinstance(node.target.value, ast.Name)
                or node.target.value.id != "facts" or not isinstance(node.op, ast.Add)):
            raise ValueError("Root 715 additional assessment shape mismatch")
        additions715[ast.literal_eval(node.target.slice)] = ast.literal_eval(node.value)
if set(fresh715) != {2741, 2742} or set(additions715) != {2738, 2740}:
    raise ValueError("Root 715 assessment scope mismatch")
root715_reg = json.loads((root715_dir / root715["registration_sha256"]).read_bytes())
prior715 = {item["root_original"]: item["row"] for item in parsed715}
rebuilt715 = dict(fresh715)
for source in root715_reg["sources"][:5]:
    n = source["original"]
    ranges = source["prospective_ranges"]
    eligible = [obs["assessment"] for obs in prior715[n]["observations"]
                if any(max(lo, b) <= min(hi, e) for lo, hi in ranges for b, e in obs["ranges"])]
    rebuilt715[n] = ("Exact qualified prior root read, zero reread. " + " ".join(eligible)
                    + " New role only " + str(ranges)
                    + "; historical extra lines/tests/context are not new role credit. "
                    + "All source/reg/read/row/plan/state/native verification/writer/hash/EOF/authorship retained; "
                    + "actual producer/native/current and connected ownership remain required.")
for n, addition in additions715.items():
    rebuilt715[n] += addition
if (sorted(rebuilt715) != list(range(2736, 2743))
        or conditional715["source_assessments"] != [{"original": k, "assessment": rebuilt715[k]} for k in sorted(rebuilt715)]):
    raise ValueError("Root 715 conditional assessments differ from recorded construction")
printed715 = None
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] == 54682:
            printed715 = json.loads(item["output_text"])
            break
if (printed715 is None or printed715["packet"]["sha256"] != root715["packet_sha256_recorded_from_output"]
        or printed715["plan"]["sha256"] != root715["plan_sha256_recorded_from_output"]
        or root715["plan_original_recovered"] is not False):
    raise ValueError("Root 715 printed packet or plan reference mismatch")


meta719 = l13_719["prior_row_extracts"]
extract719 = (l13_719_dir / meta719["file"]).read_bytes()
excerpt719 = (l13_719_dir / meta719["display_excerpt"]).read_bytes()
if (hashlib.sha256(extract719).hexdigest() != meta719["sha256"]
        or hashlib.sha256(excerpt719).hexdigest() != meta719["display_excerpt_sha256"]):
    raise ValueError("Lens 13 root 719 prior row checksum mismatch")
parsed719 = []
pending719 = None
for line in excerpt719.decode("utf8").splitlines():
    if line.startswith("PRIOR "):
        match = re.match(r"^PRIOR (\d+) (\S+) ROOTKEYS (\[.*\])$", line)
        if match is None or pending719 is not None:
            raise ValueError("Lens 13 root 719 prior header mismatch")
        pending719 = {"root_original": int(match.group(1)), "prior_receipt_path": match.group(2),
                      "printed_root_keys": ast.literal_eval(match.group(3))}
    elif line.startswith("ROW ") and pending719 is not None:
        match = re.match(r"^ROW (\w+) (\d+) (\{.*\})$", line)
        if match is None:
            raise ValueError("Lens 13 root 719 prior row mismatch")
        pending719.update(section=match.group(1), row_index=int(match.group(2)),
                          row=json.loads(match.group(3)))
        parsed719.append(pending719)
        pending719 = None
    else:
        raise ValueError("Unexpected Lens 13 root 719 excerpt line")
if pending719 is not None or parsed719 != json.loads(extract719) or len(parsed719) != 3:
    raise ValueError("Lens 13 root 719 prior row reconstruction mismatch")
discovery719 = json.loads((l13_719_dir / l13_719["discovery_sha256"]).read_bytes())
hits719 = {(h["original"], h["receipt"]["path"]) for h in discovery719["exact_hash_prior_hits"]}
sources719 = {r["original"]: r["sha256"] for r in l13_719["source_objects"]}
support719 = {r["name"]: r["sha256"] for r in l13_719["prior_support_files"]}
expected719 = {2788: ("connected", 0, 1242, "READ-PL13-162.json"),
               2801: ("connected", 0, None, "READ-PL13-309.json"),
               2803: ("rows", 1, 1216, "READ-PL13-301.json")}
for item in parsed719:
    n = item["root_original"]
    section, index, old, name = expected719[n]
    raw = json.loads((l13_719_dir / "prior-support" / support719[name]).read_bytes())
    if ((n, item["prior_receipt_path"]) not in hits719 or item["section"] != section
            or item["row_index"] != index or item["row"]["original"] != old
            or item["row"]["sha256"] != sources719[n]
            or raw[section][index] != item["row"]):
        raise ValueError("Lens 13 root 719 printed and original delegated rows differ")


plan_dir = root / "recovered/l13-plan-reference-inventory"
plan_inventory = json.loads((plan_dir / "MANIFEST.json").read_bytes())
plan_rows = plan_inventory["rows"]
if ([row["root"] for row in plan_rows] != list(range(710, 724))
        or plan_inventory["roots"] != 14 or plan_inventory["packet_originals_recovered"] != 6
        or plan_inventory["plan_originals_recovered"] != 1):
    raise ValueError("Lens 13 late plan inventory mismatch")
queue_records = {}
ordinals = {row["source_ordinal"] for row in plan_rows if row["source"] == "queue-command-outputs"}
with (root / "recovered/queue-command-outputs/RECORDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        record = json.loads(line)
        if record["transcript_ordinal"] in ordinals:
            queue_records[record["transcript_ordinal"]] = record
for row in plan_rows:
    data = (plan_dir / row["printed_file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != row["printed_sha256"]:
        raise ValueError("Lens 13 printed plan reference checksum mismatch")
    printed = json.loads(data)
    if printed["packet"] != row["packet"] or printed["plan"] != row["plan"]:
        raise ValueError("Lens 13 printed plan reference mismatch")
    directory = root / f"recovered/l13-root-{row['root']}"
    manifest = json.loads((directory / "MANIFEST.json").read_bytes())
    source_numbers = [x["original"] for x in manifest["source_objects"]]
    if f"{min(source_numbers)}-{max(source_numbers)}" not in row["packet"]["path"]:
        raise ValueError("Lens 13 printed packet range mismatch")
    expected_packet = (manifest.get("packet_sha256") or manifest.get("packet_sha256_recorded")
                       or manifest.get("packet_sha256_recorded_from_output")
                       or manifest.get("packet_sha256_recorded_from_later_queue"))
    if expected_packet != row["packet"]["sha256"]:
        raise ValueError("Lens 13 printed packet identity mismatch")
    if row["packet_original_recovered"] != (manifest.get("packet_sha256") == expected_packet):
        raise ValueError("Lens 13 packet recovery count mismatch")
    plan_path = directory / row["plan"]["sha256"]
    plan_recovered = plan_path.is_file() and hashlib.sha256(plan_path.read_bytes()).hexdigest() == row["plan"]["sha256"]
    if row["plan_original_recovered"] != plan_recovered:
        raise ValueError("Lens 13 plan recovery count mismatch")
    if row["source"] == "queue-command-outputs":
        record = queue_records[row["source_ordinal"]]
        if not any(line == data.decode("utf8").rstrip("\n") for line in record["output_text"].splitlines()):
            raise ValueError("Lens 13 plan reference not in preserved output")
    elif row["source"] != "private-transcript-excerpt":
        raise ValueError("Unknown Lens 13 plan reference provenance")
if (sum(row["packet_original_recovered"] for row in plan_rows) != 6
        or sum(row["plan_original_recovered"] for row in plan_rows) != 1):
    raise ValueError("Lens 13 plan inventory recovery counts mismatch")


post_dir = root / "recovered/l13-late-post-apply-references"
post = json.loads((post_dir / "MANIFEST.json").read_bytes())
if post["root_count"] != 14 or [r["root"] for r in post["roots"]] != list(range(710, 724)):
    raise ValueError("Lens 13 post-apply reference inventory mismatch")
plan_lookup = {r["root"]: r for r in plan_rows}
post_hashes = set()
for item in post["roots"]:
    data = (post_dir / item["printed_file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["printed_sha256"]:
        raise ValueError("Lens 13 post-apply output checksum mismatch")
    output = json.loads(data)
    manifest = json.loads((root / f"recovered/l13-root-{item['root']}/MANIFEST.json").read_bytes())
    source_numbers = [x["original"] for x in manifest["source_objects"]]
    if (output["plan"] != item["plan"] or output["plan"] != plan_lookup[item["root"]]["plan"]
            or output["preflight"] != "passed" or output["original_before"] != min(source_numbers)
            or output["original_after"] != max(source_numbers) + 1
            or output["implementation_gate_complete"] is not False
            or output["product_execution"] is not False):
        raise ValueError("Lens 13 post-apply historical report mismatch")
    for key in ("ledger", "selector", "queue", "selected_lenses"):
        if output[key] != item[key] or re.fullmatch(r"[0-9a-f]{64}", item[key]["sha256"]) is None:
            raise ValueError("Lens 13 post-apply reference mismatch")
        post_hashes.add(item[key]["sha256"])
if len(post_hashes) != 56:
    raise ValueError("Lens 13 post-apply distinct reference count mismatch")


verification_dir = root / "recovered/l13-late-verification-references"
verification_inventory = json.loads((verification_dir / "MANIFEST.json").read_bytes())
verification_rows = verification_inventory["rows"]
if (verification_inventory["root_count"] != 14
        or verification_inventory["original_verification_bodies_recovered"] != 0
        or [row["root"] for row in verification_rows] != list(range(710, 724))):
    raise ValueError("Lens 13 verification reference inventory mismatch")
for item in verification_rows:
    data = (verification_dir / item["printed_file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["printed_sha256"]:
        raise ValueError("Lens 13 verification summary checksum mismatch")
    printed = json.loads(data)
    directory = root / f"recovered/l13-root-{item['root']}"
    manifest = json.loads((directory / "MANIFEST.json").read_bytes())
    indices = [r["original"] for r in manifest["source_objects"]]
    if (printed["result"] != "passed" or printed["verification"] != item["verification"]
            or printed["original_before"] != min(indices) or printed["original_after"] != max(indices) + 1
            or any(printed[key] != item[key] for key in ("original_before", "original_after",
                   "fresh_ordered_lines", "queue_records", "nested_range_fields"))
            or item["original_verification_body_recovered"] is not False
            or re.fullmatch(r"[0-9a-f]{64}", item["verification"]["sha256"]) is None):
        raise ValueError("Lens 13 verification summary mismatch")


revisit_dir = root / "recovered/l13-late-revisit-references"
revisits = json.loads((revisit_dir / "MANIFEST.json").read_bytes())
revisit_rows = revisits["rows"]
if (revisits["report_count"] != 14 or revisits["selected_pointers_matching_prior_batch_reports"] != 14
        or [r["queue_number"] for r in revisit_rows] != list(range(742, 769, 2))):
    raise ValueError("Lens 13 contract-revisit inventory mismatch")
post_lookup = {r["root"]: r for r in post["roots"]}
for row in revisit_rows:
    data = (revisit_dir / row["printed_file"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != row["printed_sha256"]:
        raise ValueError("Lens 13 contract-revisit summary checksum mismatch")
    printed = json.loads(data)
    prior = post_lookup[row["following_root"]]
    if (row["following_root"] != 710 + (row["queue_number"] - 742) // 2
            or printed["result"] != "passed" or printed["preflight"] != "passed"
            or printed["independent_agent_review"] is not False
            or printed["product_execution"] is not False
            or printed["implementation_gate_complete"] is not False
            or len(printed["contract_updates"]) != row["contract_updates_reported"]
            or printed["all13_selections_unchanged"] != row["selected_lenses"]
            or row["selected_lenses"] != prior["selected_lenses"]
            or printed.get("verification") != row["verification"]):
        raise ValueError("Lens 13 contract-revisit summary mismatch")
    for key in ("ledger", "selector", "queue"):
        if printed[key] != row[key] or re.fullmatch(r"[0-9a-f]{64}", row[key]["sha256"]) is None:
            raise ValueError("Lens 13 contract-revisit state reference mismatch")
    if not row["queue"]["path"].endswith(f"CLOSURE-QUEUE-{row['queue_number']}.json"):
        raise ValueError("Lens 13 contract-revisit queue number mismatch")


plan_cross_dir = root / "recovered/l13-selected-state-transition-trace"
plan_cross = json.loads((plan_cross_dir / "PLAN-CANDIDATE-722-CROSSCHECK.json").read_bytes())
plan_candidate_bytes = (plan_cross_dir / plan_cross["candidate"]).read_bytes()
if hashlib.sha256(plan_candidate_bytes).hexdigest() != plan_cross["candidate_sha256"]:
    raise ValueError("Final Lens 13 plan candidate checksum mismatch")
plan_candidate = json.loads(plan_candidate_bytes)["template"]
revisit766 = next(row for row in revisit_rows if row["queue_number"] == 766)
if (plan_cross["matched_fields"] != ["ledger", "selector", "queue", "selected_lenses"]
        or any(plan_candidate[key] != revisit766[key] for key in plan_cross["matched_fields"])
        or plan_candidate["previous_state"]["sha256"] != "X" * 64
        or plan_cross["unknown_field"] != "previous_state.sha256"
        or plan_cross["original_plan_sha256"] != plan_lookup[723]["plan"]["sha256"]):
    raise ValueError("Final Lens 13 plan predecessor crosscheck mismatch")


checkpoint_counts = json.loads((root / "RECOVERY-CHECKPOINT.json").read_bytes())
summary = checkpoint_counts["l13_roots_710_723_quantitative_summary"]
for kind in ("qualification", "packet", "adoption"):
    recovered = sum(checkpoint_counts.get(f"l13_root_{n}_{kind}_original_recovered") is True
                    for n in range(710, 724))
    if summary[f"{kind}_original_recovered"] != recovered or summary[f"{kind}_missing"] != 14 - recovered:
        raise ValueError("Lens 13 batch summary disagrees with checkpoint")

if _qualified_seen != set(_qualified_objects):
    raise ValueError("Not all three qualified public redactions were checked")
print("Qualified redaction: three reconstructed public objects and their original manifest identities checked; original private bytes were not verified.")

if _extra_seen != set(_extra_objects):
    raise ValueError("Not all additional qualified redactions were checked")
print("Qualified redaction: reconstructed historical writer output and previously redacted portability record checked; their earlier bytes were not verified.")

print("Verified", count, "evidence files,", dependency_count, "hash-matched dependencies,", recursive_count, "recursive dependencies,", printed_count, "printed-hash registrations,", audit_record_count, "printed-hash audit records,", reference_count, "printed-reference JSON bodies,", linked_count, "linked JSON bodies,", literal_count, "linked literal objects,", filename_count, "printed-filename JSON bodies,", filename_text_count, "printed-filename text objects,", contract_count, "contract values,", document_count, "audit documents,", len(found), "ledger fragments,", ledger_context_count, "ledger outputs,", queue_records_count, "queue outputs,", source_records_count + redacted_source_count, "source records,", late_source_count, "late source records,", markdown_count, "Markdown writes,", helper_count, "helper candidates,", redacted_count, "redacted helpers,", python_writes_count, "Python writes,", other_count, "other candidates,", other_writes_count, "other writes,", patch_count, "unresolved patches,", draft_change_count, "draft changes,", draft_validation_count, "draft validation records,", redacted_documents_count, "redacted documents,", command_count, "late commands,", late_history_count, "late history records and", audit_commands_count + reviewed_command_count, "audit commands; claims remain qualified")
