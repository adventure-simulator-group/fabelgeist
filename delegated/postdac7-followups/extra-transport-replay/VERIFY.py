#!/usr/bin/env python3
from pathlib import Path
import json,sys
if sys.flags.optimize: raise RuntimeError("Unoptimized interpreter required")
r=json.loads((Path(__file__).resolve().parent/"RESULT.json").read_bytes());assert r["schema"]=="726-extra-transport-replay-v1"
assert [x["label"] for x in r["results"]]==["restore","prepared_public_full","stream_public_full","overflow-prior","overflow-v7","q2910-v6"]
assert all(x["exit"]==0 and x["stderr_bytes"]==0 for x in r["results"])
print("Six recorded evidence restoration and replay results passed; exact outputs remain restricted. No independent custody or product behavior inferred.")
