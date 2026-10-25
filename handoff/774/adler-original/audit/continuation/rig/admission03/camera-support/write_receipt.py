import pathlib,json,hashlib
BASE=pathlib.Path("target/774-continuation-20261021/continuation")
OUT=BASE/"rig/admission03/camera-support"
QUEUE=json.loads((BASE/"admission03-queue-v002/camera_rig_support.json").read_text(encoding="utf-8"))["queue"]
def add(index,role,invariant,routes,limits,findings=None,deps=None,ranges=None):
 row=next(r for r in QUEUE if r["camera_queue_index"]==index)
 source=row["source_scope_record"]
 p=pathlib.Path("target/774-snapshots")/source["snapshot"]/source["path"]
 if source.get("state")=="deleted":
  p=BASE/"deleted-source"/source["snapshot"]/source["path"]
 raw=p.read_bytes(); expected=row["sha256"] or source.get("base_sha256"); assert hashlib.sha256(raw).hexdigest()==expected
 eof=len(raw.decode("utf-8").splitlines())
 read=ranges or [[1,eof]]
 ident=f"RIG-CAMERA-ADM03-{index:03}"
 record=dict(id=ident,camera_queue_index=index,source_id=row["source_id"],sha256=row["sha256"],queue_ref=f"continuation/admission03-queue-v002/camera_rig_support.json#/queue/{QUEUE.index(row)}",scope_pointer=row["scope_pointer"],actual_read_ranges=read,actual_eof=eof,current_source_absent=source.get("state")=="deleted",base_sha256=source.get("base_sha256") if source.get("state")=="deleted" else None,current_roles=role,invariant=invariant,all_admission_routes=routes,counterexample_or_intentional_boundary=limits,finding_refs=findings or [],connected_dependencies=deps or [],mode="Fresh actual current admission source comparison; original tests read as source only. No project execution or earlier-lens verdict carry.",authority="root/admission03/support-authority-053.json; Camera retains source/audit ownership; Root final audit/repair authority",disposition="exact_source_role_compared_with_stated_limits_and_dependencies")
 for filename,obj in [("contracts.jsonl",record),("coverage.jsonl",{k:record[k] for k in ["id","camera_queue_index","source_id","sha256","actual_read_ranges","current_roles","finding_refs","connected_dependencies","disposition"]})]:
  path=OUT/filename
  if path.exists(): assert not any(json.loads(s)["camera_queue_index"]==index for s in path.read_text(encoding="utf-8").splitlines())
  with path.open("a",encoding="utf-8") as f:f.write(json.dumps(obj,ensure_ascii=False)+"\n")
 done={json.loads(s)["camera_queue_index"] for s in (OUT/"coverage.jsonl").read_text(encoding="utf-8").splitlines()}
 pending=[r["camera_queue_index"] for r in QUEUE if r["camera_queue_index"] not in done]
 with (OUT/"progress.jsonl").open("a",encoding="utf-8") as f:f.write(json.dumps(dict(handled=len(done),assigned=200,first_unfinished=pending[0] if pending else None,last_receipt=ident,original_rig802_unchanged=True,scope="supplier only; not Camera/Root acceptance or lens clearance"))+"\n")
