#!/usr/bin/env python3
"""Check reconstructed frontier source observations against pinned Git bytes."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

if len(sys.argv) != 2:
    raise SystemExit("usage: verify_frontier.py REPOSITORY")
repo = Path(sys.argv[1]).absolute()
base = Path(__file__).parent
trace = json.loads((base / "TRACE.json").read_text(encoding="utf-8"))
history = base.parent / "late-command-history" / "RECORDS.jsonl"
ordinals = {x["read_ordinal"] for x in trace["sources"]}
records = {}
with history.open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] in ordinals:
            records[item["transcript_ordinal"]] = item
count = 0
for item in trace["sources"]:
    result = subprocess.run(["git", "-C", str(repo), "show", item["revision"] + ":" + item["path"]],
                            check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    data = result.stdout
    if hashlib.sha256(data).hexdigest() != item["source_sha256"]:
        raise ValueError("Source hash mismatch")
    lines = data.decode("utf-8").splitlines()
    record = records[item["read_ordinal"]]
    if record["recorded_exit_code"] != 0 or any(record["redactions"].values()):
        raise ValueError("Source read record failed or was redacted")
    output = record["output_text"].replace("\r\r\n", "\n").replace("\r\n", "\n")
    if sum(x["read_ordinal"] == item["read_ordinal"] for x in trace["sources"]) > 1:
        marker = "\n " + item["path"] + "\n"
        start = output.find(marker)
        if start < 0:
            raise ValueError("Source output heading missing")
        output = output[start + len(marker):]
        end = output.find("\n crates/")
        if end >= 0:
            output = output[:end]
    printed = {int(m.group(1)): m.group(2) for m in re.finditer(r"(?m)^(\d+): (.*)$", output)}
    needed = {i for lo, hi in item["ranges"] for i in range(lo, hi + 1)}
    if not needed.issubset(printed) or any(printed[i] != lines[i - 1] for i in needed):
        raise ValueError("Recorded source lines do not match pinned bytes")
    count += len(needed)
print("Verified", len(trace["sources"]), "source identities and", count, "recorded lines")
