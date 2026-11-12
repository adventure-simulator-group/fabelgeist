#!/usr/bin/env python3
import hashlib,json,pathlib
p=pathlib.Path(__file__).resolve().parent
m=json.loads((p/"MANIFEST.json").read_bytes())
for r in m["files"]:
 b=(p/r["path"]).read_bytes();assert len(b)==r["bytes"] and hashlib.sha256(b).hexdigest()==r["sha256"]
a=json.loads((p/"INDEPENDENT-PRIVACY-REVIEW-FINAL.json").read_bytes());b=json.loads((p/"VALUE-DIGEST-SCAN.json").read_bytes());c=json.loads((p/"PUBLIC-ARCHIVE-RECEIPT.json").read_bytes())
assert not a["hits"] and not a["errors"] and not b["matches"] and not b["exceptions"]
assert c["stage_manifest_sha256"]=="6221807cee4af7240841b2ed0a883d3e9f262071c4b65a93cb5b8b6cf77b7fcf"
print("PASS: recorded bounded scans and reviewed archive binding; no assertion of complete privacy or remote preservation")
