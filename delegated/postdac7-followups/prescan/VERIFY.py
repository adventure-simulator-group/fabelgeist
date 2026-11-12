#!/usr/bin/env python3
import json,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
r=json.loads((Path(__file__).resolve().parent/"RESULT.json").read_bytes());assert r["schema"]=="726-postdac7-prescan-v1" and r["forbidden_digest_total"]==0 and not r["exceptions"] and not r["other_candidate_counts"] and len(r["scan_results"])==2 and all(x["exit"]==0 and x["stderr_bytes"]==0 for x in r["scan_results"])
print("Verified bounded pre-final scan receipt; encrypted and unknown data remain outside scope.")
