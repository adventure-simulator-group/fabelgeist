import json,pathlib,difflib,hashlib,sys
sys.stdout.reconfigure(encoding="utf-8")
b=pathlib.Path(__file__).resolve().parents[3]
q=json.loads((b/"admission03-queue-v002/camera_root_support.json").read_text(encoding="utf-8"))["queue"]
loc={r["source_id"]:r for r in json.loads((b/"source-locators-2559-verified.json").read_text(encoding="utf-8"))["matched"]}
for arg in sys.argv[1:]:
 i=int(arg);r=q[i];p=pathlib.Path(loc.get(r["source_id"],{}).get("resolved_path",r["source_scope_record"].get("resolved_path",r["source_scope_record"]["path"])));a=p.read_bytes();assert hashlib.sha256(a).hexdigest()==r["sha256"];rel=r["source_id"].split(":",1)[1];z=pathlib.Path("target/774-snapshots/pr-working")/rel
 print("SUPPORT",i,"CAMERA",r["camera_queue_index"],rel)
 if z.exists():print("".join(difflib.unified_diff(z.read_text(encoding="utf-8").splitlines(True),p.read_text(encoding="utf-8").splitlines(True),fromfile="currentPR",tofile="preservedP16",n=3)) or "COMPLETE BODY COMPARISON: no differences")
 else:print("NO PR COUNTERPART")
