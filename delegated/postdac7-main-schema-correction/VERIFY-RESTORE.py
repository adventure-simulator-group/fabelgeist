#!/usr/bin/env python3
from pathlib import Path
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
p=Path(__file__).resolve().parent;r=json.loads((p/"RESTORE-RESULT.json").read_bytes());f=json.loads((p/"FAILED-DRIVER-RESULT.json").read_bytes());assert r["schema"]=="726-postdac7-main-schema-restore-v1" and r["encrypted_manifest_sha256"]==hashlib.sha256((p/"encrypted-originals/MANIFEST.json").read_bytes()).hexdigest() and r["restore_exit"]==r["stderr_bytes"]==0 and r["restored_failed_transcript_sha256"]==f["transcript_sha256"]
print("Verified recorded same-machine restoration of failed evidence-run originals.")
