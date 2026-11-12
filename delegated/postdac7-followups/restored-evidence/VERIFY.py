#!/usr/bin/env python3
from pathlib import Path
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
p=Path(__file__).resolve().parent;r=json.loads((p/"RESULT.json").read_bytes());h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
assert r["schema"]=="726-postdac7-authenticated-restore-v1" and r["encrypted_manifest_sha256"]==h(p.parent/"encrypted-originals/MANIFEST.json") and r["guide_manifest_sha256"]==h(p.parent/"portable-guide/MANIFEST.json") and r["q2945_manifest_sha256"]==h(p.parent/"q2945-packet/MANIFEST.json")
assert [x["label"] for x in r["steps"]]==["keyless","keyed-restore","anchors","guide-restored","q2945-restored"] and all(x["exit"]==0 and x["stderr_bytes"]==0 for x in r["steps"])
assert len(r["selected_guide_checks"])==18 and all(x["exit"]==0 and not x["timeout"] and x["result"]=="PASS" for x in r["selected_guide_checks"])
print("Verified recorded same-machine encrypted restore and 18 selected replay results; independent custody and product behavior remain unproved.")
