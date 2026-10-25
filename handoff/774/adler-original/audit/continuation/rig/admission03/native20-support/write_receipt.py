from pathlib import Path
import json,hashlib,datetime
BASE=Path("target/774-continuation-20261021/continuation")
OUT=BASE/"rig/admission03/native20-support"
QUEUE=json.loads((BASE/"admission03-queue-v002/camera_rig_native20.json").read_text(encoding="utf-8"))["queue"]
def add(index,role,invariant,routes,limits,deps=None,findings=None,ranges=None):
 row=next(x for x in QUEUE if x["camera_queue_index"]==index); source=row["source_scope_record"]
 p=Path("target/774-snapshots")/source["snapshot"]/source["path"];raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==row["sha256"]
 eof=len(raw.decode("utf-8").splitlines());ident=f"RIG-NATIVE20-ADM03-{index:03}"
 rec=dict(id=ident,camera_queue_index=index,source_id=row["source_id"],sha256=row["sha256"],actual_read_ranges=ranges or [[1,eof]],actual_eof=eof,queue_pointer=f"admission03-queue-v002/camera_rig_native20.json#/queue/{QUEUE.index(row)}",role=role,invariant=invariant,all_admission_routes=routes,counterexample_or_intentional_boundary=limits,connected_dependencies=deps or [],finding_refs=findings or [],authority="root/admission03/support-authority-071.json; current03 only; Camera final source/audit authority, Root final audit/repair",source_proof="Fresh actual frozen current bounded/full source including original tests where scoped; no execution/newcases or earlier/laterlens credit",repair="frozen",validation="not_run")
 for name,obj in [("contracts.jsonl",rec),("coverage.jsonl",{k:rec[k] for k in ["id","camera_queue_index","source_id","sha256","actual_read_ranges","actual_eof","connected_dependencies","finding_refs"]})]:
  p=OUT/name
  if p.exists():assert not any(json.loads(x)["camera_queue_index"]==index for x in p.read_text(encoding="utf-8").splitlines())
  with p.open("a",encoding="utf-8") as f:f.write(json.dumps(obj,ensure_ascii=False)+"\n")
 done={json.loads(x)["camera_queue_index"] for x in (OUT/"coverage.jsonl").read_text(encoding="utf-8").splitlines()}
 pending=[r["camera_queue_index"] for r in QUEUE if r["camera_queue_index"] not in done]
 with (OUT/"progress.jsonl").open("a",encoding="utf-8") as f:f.write(json.dumps(dict(at=datetime.datetime.now(datetime.timezone.utc).isoformat(),handled=len(done),assigned=20,first_unfinished=pending[0] if pending else None,last=ident,scope="OriginalRig802/Root112/Camera162 unchanged; exact20supplieronly, no global clearance"))+"\n")
