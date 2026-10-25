from pathlib import Path
import json,hashlib,datetime
B=Path("target/774-continuation-20261021/continuation")
D=B/"rig/admission03"
Q=json.loads((B/"admission03-queue-v002/rig.json").read_text())["queue"]
def pin(i,r,n):
 row=Q[i];p=Path(row["source_scope_record"]["path"])
 if not p.is_absolute():p=Path("target/774-snapshots")/row["source_scope_record"]["snapshot"]/p
 assert hashlib.sha256(p.read_bytes()).hexdigest()==row["sha256"]
 num=len((D/"contract-groups.jsonl").read_text().splitlines())+1;ref=f"RIG-ADMISSION03-CONTRACT-{num:03}"
 c={"id":ref,"queue_index":i,"source_id":row["source_id"],"sha256":row["sha256"],"actual_source_ranges":r,"contract":p.name+" actual admission contracts","invariant_routes_counterexample_and_current_role":n,"connected_repair_acceptance_dependencies":"Preserve stated raw/native/open policies; actual delegated definitions remain explicit dependencies where named. No source repair/execution/later lens credit.","actual_source_proof":"Fresh exact scoped actual body; original tests only where in ranges.","remaining_unread_local_material_routes":[]}
 with (D/"contract-groups.jsonl").open("a")as f:f.write(json.dumps(c)+"\n")
 with (D/"coverage.jsonl").open("a")as f:f.write(json.dumps({"source_id":row["source_id"],"sha256":row["sha256"],"queue_index":i,"scope_pointer":row["scope_pointer"],"contract_refs":[ref],"actual_source_ranges":r,"status":"exact_admission_source_inventory_with_shared_dependencies/limits","repair":"not_started","validation":"not_executed"})+"\n")
 return ref
def progress():
 c=[json.loads(x)for x in(D/"coverage.jsonl").read_text().splitlines()];got={x["queue_index"]for x in c};left=[i for i in range(802)if i not in got]
 num=len((D/"progress.jsonl").read_text().splitlines())+1
 v={"id":f"RIG-ADMISSION03-PROGRESS-{num:03}","at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"assigned":802,"accounted_local_admission_inventory":len(got),"coverage_receipts":len(c),"remaining":len(left),"first_unfinished_queue_index":left[0]if left else None,"first_unfinished_source_id":Q[left[0]]["source_id"]if left else None,"source_registry_rows":len((D/"source-range-registration.jsonl").read_text().splitlines()),"last_contract":json.loads((D/"contract-groups.jsonl").read_text().splitlines()[-1])["id"],"repair":"frozen","validation":"not_run","later_lens_credit":False}
 with(D/"progress.jsonl").open("a")as f:f.write(json.dumps(v)+"\n")
 print(json.dumps(v))
