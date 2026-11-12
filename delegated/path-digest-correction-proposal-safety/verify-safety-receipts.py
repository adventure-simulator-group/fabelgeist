#!/usr/bin/env python3
import pathlib,json,hashlib
p=pathlib.Path(__file__).resolve().parent;m=json.loads((p/"MANIFEST.json").read_bytes())
for r in m["files"]:
 b=(p/r["path"]).read_bytes();assert (hashlib.sha256(b).hexdigest(),len(b))==(r["sha256"],r["bytes"])
a=json.loads((p/"V2-INDEPENDENT-PRIVACY-SCAN.json").read_bytes());b=json.loads((p/"V2-VALUE-DIGEST-SCAN.json").read_bytes());c=json.loads((p/"PUBLIC-ARCHIVE-RECEIPT.json").read_bytes())
assert not a["hits"] and not a["errors"] and not b["matches"] and not b["exceptions"]
assert c["stage_manifest_sha256"]=="a3b2acb186938355f76d32f5ea7b83b9d91b9163bf013183b55dfb0c6bbe48f7"
print("PASS: bounded scan receipts and proposal archive binding; unknown values and legacy verifier adaptations remain open")
