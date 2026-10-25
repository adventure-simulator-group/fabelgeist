#!/usr/bin/env python3
"""Resolve exported evidence paths without rewriting original records."""
from pathlib import Path
import hashlib, json, sys
ROOT=Path(__file__).resolve().parent
OLD_A="/home/brunosegovia/.codex/worktrees/ae46/adventure-simulator/target/bespoke-audit-774-merged-20261022"
OLD_H="/home/brunosegovia/.codex/worktrees/ae46/adventure-simulator/target/semantic-goal/pr774-plan-execution-20261020"
def resolve(value):
    value=str(value)
    extra=json.loads((ROOT/"EXTERNAL-PATHS.json").read_text())
    if value in extra:return ROOT/extra[value]
    for old,new in [("target/bespoke-audit-774-merged-20261022",ROOT/"audit"),(OLD_A,ROOT/"audit"),(OLD_H,ROOT/"historical-family-execution"),("/home/brunosegovia/.codex/skills",ROOT/"skills")]:
        if value==old or value.startswith(old+"/"):
            return new/value[len(old):].lstrip("/")
    p=Path(value)
    if not p.is_absolute(): return ROOT/p
    raise ValueError("Not an exported evidence-root path; use exact source_id/hash catalog, not filename guessing: "+value)
def source(index):
    rows=json.loads((ROOT/"audit/reconcile-adler-20261023/recovered-sources/catalog.json").read_text())["rows"]
    row=rows[index];p=resolve(row["object"]);b=p.read_bytes()
    assert hashlib.sha256(b).hexdigest()==(row.get("expected_sha256") or row.get("verified_object_sha256")),row["source_id"]
    return row,b
if __name__=="__main__":
    if sys.argv[1]=="source":
        row,b=source(int(sys.argv[2]));lo=int(sys.argv[3]) if len(sys.argv)>3 else 1;hi=int(sys.argv[4]) if len(sys.argv)>4 else (row["line_count"] or 0)
        print(row["source_id"],row.get("expected_sha256") or row.get("verified_object_sha256"),row.get("source_state","present"))
        if row.get("kind")=="binary":print("Binary source; bytes verified, no text projection.");raise SystemExit(0)
        for n,l in enumerate(b.decode().splitlines(),1):
            if lo<=n<=hi:print(str(n)+": "+l)
    else:print(resolve(sys.argv[1]))
