#!/usr/bin/env python3
"""Verify recovered evidence files without adopting their audit claims."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).parent
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
item = json.loads((directory / "portability-review.manifest.json").read_text(encoding="utf-8"))
if hashlib.sha256((directory / item["file"]).read_bytes()).hexdigest() != item["redacted_sha256"]:
    raise ValueError("Redacted portability review hash mismatch")
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
print("Verified", count, "evidence files,", contract_count, "contract values,", document_count, "audit documents,", markdown_count, "Markdown writes,", helper_count, "helper candidates,", redacted_count, "redacted helpers,", python_writes_count, "Python writes,", other_count, "other candidates,", other_writes_count, "other writes,", patch_count, "unresolved patches,", redacted_documents_count, "redacted documents and", command_count, "late commands; claims remain qualified")
