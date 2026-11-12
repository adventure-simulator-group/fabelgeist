#!/usr/bin/env python3
import hashlib,io,json,stat,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
from TRANSFORM import transform
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();r=json.loads((p/"RECORD.json").read_bytes());assert r["schema"]=="726-postdac7-application-v1" and h((p/"TRANSFORM.py").read_bytes())==r["transform_sha256"]
b=(p/r["archive"]).read_bytes();assert len(b)==r["archive_bytes"] and h(b)==r["archive_sha256"];expected={x["path"]:x for x in r["members"]};assert len(expected)==len(r["members"]);data={}
with tarfile.open(fileobj=io.BytesIO(b),mode="r:gz") as t:
 for item in t:
  n=PurePosixPath(item.name);assert not n.is_absolute() and ".." not in n.parts and str(n) in expected and str(n) not in data and item.isfile() and stat.S_IMODE(item.mode)==int(expected[str(n)]["mode"],8)
  raw=t.extractfile(item).read();row=expected[str(n)];assert len(raw)==row["bytes"] and h(raw)==row["sha256"];data[str(n)]=raw
assert set(data)==set(expected);names=["BASELINE-95E-20261110.json","CONTINUATION-PROGRESS-95E.json","RECONCILIATION-059.json","CONTINUATION.md"];before={n:data["before/"+n] for n in names};after=transform(before,json.loads(data["inputs/report.json"]),json.loads(data["inputs/pending.json"]),json.loads(data["inputs/main-observation.json"]))
assert all(after[n]==data["after/"+n] for n in names)
print("Verified exact before/after checkpoint transformation and its seven partial assessments and nine relationship additions; no product work inferred.")
