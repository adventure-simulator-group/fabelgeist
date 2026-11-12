#!/usr/bin/env python3
from pathlib import Path
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
p=Path(__file__).resolve().parent;r=json.loads((p/"RESULT.json").read_bytes());h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
assert r["schema"]=="726-postdac7-control-restore-v1" and r["control_manifest_sha256"]==h(p.parent/"post-encryption-control/MANIFEST.json") and r["q2968_manifest_sha256"]==h(p.parent/"later-q2968/MANIFEST.json")
assert [x["label"] for x in r["steps"]]==["control-keyless","control-keyed-restore","q2968-restored"] and all(x["exit"]==0 and x["stderr_bytes"]==0 for x in r["steps"])
print("Verified recorded control restore and later q2968 replay; same-machine and product limits remain.")
