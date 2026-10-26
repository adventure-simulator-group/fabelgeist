import json,hashlib
from pathlib import Path
b=Path("target/recovery-20261025/audit07-delta")
def save(num,notes,extra=None):
 q=json.loads((b/"registered-queue.json").read_text());rs=[]
 for i,n in notes.items():
  e=q["entries"][int(i)-100];assert hashlib.sha256(Path(e["local_object"]).read_bytes()).hexdigest()==e["sha256"]
  rs.append(dict(extension_index=int(i),source_id=e["source_id"],sha256=e["sha256"],source_ranges=e["ranges"],actual_eof=e["actual_eof"],source_basis="Same exact personally consumed source reused as source evidence, freshly analyzed for07; critical bodies re-read where noted. No03verdict transfer.",analysis=n,repair="not_started",validation="not_run"))
 p=b/f"packets/authority-{num:03}.json";assert not p.exists();d=dict(id=f"ADLER-CONT07-LATER-{num:03}",lens="redundant-authority",receipts=rs,global_clearance=False,new_unique_count="not asserted; remedies require deduplication",extra=extra);p.write_text(json.dumps(d,indent=2)+"\n")
 with (b/"coverage.jsonl").open("a") as f:
  for r in rs:f.write(json.dumps(dict(index=r["extension_index"],source_id=r["source_id"],sha256=r["sha256"],ranges=r["source_ranges"],packet=str(p),status="fresh07analysis_with_findings_and_qualifications"))+"\n")
 c=json.loads((b/"CURRENT.json").read_text());c["newly_accounted"]+=list(map(int,notes));remaining=sorted(set(range(100,174))-set(c["newly_accounted"]));c["next_material"]=remaining[0] if remaining else None;c["latest_packet"]=str(p);(b/"CURRENT.json").write_text(json.dumps(c,indent=2)+"\n");print(str(p),"next",c["next_material"])
