#!/usr/bin/env python3
import copy,gzip,hashlib,json,subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
p=Path(__file__).resolve().parent;r=p.parent.parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/"RECORD.json").read_bytes());assert m["schema"]=="726-postdac7-main-schema-correction-v1"
a=(r/m["original_observation"]).read_bytes();b=(r/m["successor_observation"]).read_bytes();assert h(a)==m["original_sha256"] and h(b)==m["successor_sha256"];x=json.loads(a);y=json.loads(b);expected=copy.deepcopy(x);expected["method"]="GitHub connector fetch branches/main API";expected["normalization"]=y["normalization"];assert expected==y and y["normalization"]["original_sha256"]==h(a)
before=gzip.decompress((p/"BASELINE-BEFORE.json.gz").read_bytes());after=gzip.decompress((p/"BASELINE-AFTER.json.gz").read_bytes());assert h(before)==m["baseline_before_sha256"] and h(after)==m["baseline_after_sha256"];old=json.loads(before);new=json.loads(after);expected=copy.deepcopy(old);expected["latest_main_recheck"]=m["successor_observation"];assert expected==new
failed=(p/"FAILED-DRIVER-RESULT.json").read_bytes();assert h(failed)==m["failed_driver_result_sha256"] and json.loads(failed)["exit"]==1 and h((p/"encrypted-originals/MANIFEST.json").read_bytes())==m["encrypted_manifest_sha256"]
current=json.loads((r/"BASELINE-95E-20261110.json").read_bytes());obs=json.loads((r/current["latest_main_recheck"]).read_bytes());assert obs["observed_revision"]==x["observed_revision"] and obs["ref"]==x["ref"]
subprocess.run([sys.executable,"-B",str(p/"encrypted-originals/VERIFY.py")],cwd=r,check=True)
print("Verified original observation, method-normalized successor, exact one-field baseline correction and preserved failed evidence run.")
