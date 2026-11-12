#!/usr/bin/env python3
"""Verify qualified q2328 public successor, not product behavior."""
import argparse,hashlib,json,subprocess,sys
from pathlib import Path
if sys.flags.optimize: raise RuntimeError("Assertions require an unoptimized interpreter")
P=Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
load=lambda n:json.loads((P/n).read_bytes())
m=load("MANIFEST.json");actual={x.relative_to(P).as_posix() for x in P.rglob("*") if x.is_file() and x.name!="MANIFEST.json"};assert actual==set(m["files"])
for name,row in m["files"].items():
 p=P/name;assert not p.is_symlink();b=p.read_bytes();assert len(b)==row["bytes"] and H(b)==row["sha256"]
boundary=load("PRIVACY-BOUNDARY.json");original=(P/"ORIGINAL-MANIFEST.json").read_bytes();assert H(original)==boundary["original_manifest_sha256"]
old=json.loads(original)["files"];changed={r["file"]:r for r in boundary["changes"]};assert len(changed)==5
for name,row in old.items():
 b=(P/name).read_bytes()
 if name in changed: assert row["sha256"]==changed[name]["original_file_sha256"] and H(b)==changed[name]["public_file_sha256"]
 else: assert len(b)==row["bytes"] and H(b)==row["sha256"]
assert set(old)|{"ORIGINAL-MANIFEST.json","PRIVACY-BOUNDARY.json","VERIFY-PUBLIC-SUCCESSOR.py"}==set(m["files"])
ap=argparse.ArgumentParser();ap.add_argument("--current-commit");ap.add_argument("--historical-commit");ap.add_argument("--original-queue");ap.add_argument("--reconstructed-queue");ap.add_argument("--ledger-snapshot");a=ap.parse_args();vals=[a.current_commit,a.historical_commit,a.original_queue,a.reconstructed_queue,a.ledger_snapshot]
if any(vals):
 assert all(vals);cmd=[sys.executable,"-B",str(P/"verify-evidence.py"),"--skip-manifest"]
 for k,v in zip(("--current-commit","--historical-commit","--original-queue","--reconstructed-queue","--ledger-snapshot"),vals):cmd.extend([k,v])
 subprocess.run(cmd,check=True)
else:print("NOT REPEATED dependency-rich original verifier: five separately held inputs required.")
print("Verified exact unchanged original members and five qualified path-safe replacements; no product behavior or independent custody established.")
