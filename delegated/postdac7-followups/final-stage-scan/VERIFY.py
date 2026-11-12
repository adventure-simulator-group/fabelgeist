#!/usr/bin/env python3
import json,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
r=json.loads((Path(__file__).resolve().parent/"RESULT.json").read_bytes());assert r["schema"]=="726-postdac7-final-stage-scan-v1" and not r["forbidden_digest_total"] and not r["other_candidate_counts"] and not r["exceptions"] and len(r["scan_results"])==2 and all(x["exit"]==0 and x["stderr_bytes"]==0 for x in r["scan_results"])
print("Verified bounded final-stage scan receipt; ciphertext and unknown data remain outside scope.")
