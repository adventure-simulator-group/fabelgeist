import json,hashlib
from pathlib import Path
b=Path("target/recovery-20261025/audit01-delta")
def save(num,notes,extra=None):
 q=json.loads((b/"successor-queue-through176.json").read_text());qm={e["index"]:e for e in q["entries"]};rs=[]
 for i,n in notes.items():
  e=qm[int(i)];assert hashlib.sha256(Path(e["local_object"]).read_bytes()).hexdigest()==e["sha256"]
  rs.append(dict(extension_index=int(i),source_id=e["source_id"],sha256=e["sha256"],source_ranges=e["source_ranges"],actual_eof=e["actual_eof"],trace=n,repair="not_started",validation="not_run"))
 p=b/f"packets/producer-{num:03}.json";assert not p.exists();p.write_text(json.dumps(dict(id=f"ADLER-CONT01-LATER-{num:03}",lens="producer-propagation",registry_sha256=q["registry_sha256"],receipts=rs,global_clearance=False,new_unique_count="not asserted; existing-remedy dedup required",extra=extra),indent=2)+"\n")
 with (b/"coverage.jsonl").open("a") as f:
  for r in rs:f.write(json.dumps(dict(index=r["extension_index"],source_id=r["source_id"],sha256=r["sha256"],ranges=r["source_ranges"],packet=str(p),status="fresh01trace_with_findings_and_qualifications"))+"\n")
 s=json.loads((b/"STATE.json").read_text());s["newly_accounted"]+=list(map(int,notes));rem=sorted(set(range(62,177))-set(s["newly_accounted"]));s["first_unfinished"]=rem[0] if rem else None;s["latest_packet"]=str(p);s["status"]="BOUNDED_DISCOVERY_WITH_FINDINGS_IN_PROGRESS" if rem else "BOUNDED_LATER_QUEUE_ACCOUNTED_WITH_FINDINGS";(b/"STATE.json").write_text(json.dumps(s,indent=2)+"\n");print(p,"next",s["first_unfinished"])
