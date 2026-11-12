#!/usr/bin/env python3
"""Verify a recorded same-machine public-transport replay."""
from pathlib import Path
import json,hashlib,sys
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
p=Path(__file__).resolve().parent;r=json.loads((p/"RESULT.json").read_bytes());assert len(r["rows"])==8
for x in r["rows"]:
 assert x["exit"]==0
 for ext,key in [("out","stdout"),("err","stderr")]:
  b=(p/(x["name"]+"."+ext)).read_bytes();assert len(b)==x[key+"_bytes"] and hashlib.sha256(b).hexdigest()==x[key+"_sha256"]
print("Verified eight same-machine transport and evidence replay receipts; independent custody and product behavior remain unproved.")
