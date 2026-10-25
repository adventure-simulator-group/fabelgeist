import hashlib,json,sys
from pathlib import Path
C=Path(__file__).resolve().parents[1]
authority=json.loads((C/"root/initial04-invariant-operations/scope-authority-001.json").read_text(encoding="utf-8"))
s=json.loads((C.parent/authority["scope"]["record"]).read_text(encoding="utf-8"))
for suffix in sys.argv[1:]:
    for r in s["files"]:
        if r["source_id"].endswith(suffix) and r["source_id"].split(":",1)[0] in ["physical-p16","pr-working"]:
            p=Path(r["path"])
            if not p.is_absolute():
                snap={"physical-p16":"p16","pr-working":"pr-working"}[r["source_id"].split(":",1)[0]]
                p=C.parents[1]/"774-snapshots"/snap/r["source_id"].split(":",1)[1]
            print(json.dumps({"source_id":r["source_id"],"source_scope_keys":list(r),"scope_kind":r.get("scope_kind"),"ranges":r.get("material_ranges",r.get("ranges")),"path":str(p),"actualEOF":len(p.read_text(encoding="utf-8").splitlines()),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}))

