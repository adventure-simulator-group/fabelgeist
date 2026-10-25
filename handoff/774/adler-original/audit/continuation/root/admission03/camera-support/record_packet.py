"""Bookkeeping only for manually supplied semantic narratives; never generates findings."""
import pathlib,json,datetime,hashlib,sys
b=pathlib.Path(__file__).resolve().parents[3];d=b/"root/admission03/camera-support";q=json.loads((b/"admission03-queue-v002/camera_root_support.json").read_text(encoding="utf-8"))["queue"]
p=json.load(sys.stdin);p.update(at=datetime.datetime.now(datetime.timezone.utc).isoformat(),authority="ROOT-ADM-SUPPORT-AUTHORITY-058",lens="audit-admission-contracts",repair="frozen_initial_all13",validation="not_established",later_lens_credit=False)
seen={json.loads(l)["support_queue_index"] for l in (d/"coverage.jsonl").read_text(encoding="utf-8").splitlines()};ix=[int(i) for i in p["source_roles"]];assert not seen.intersection(ix)
name=p.pop("file");assert not (d/name).exists();(d/name).write_text(json.dumps(p,indent=2)+"\n")
with (d/"coverage.jsonl").open("a") as f:
 for i in ix:
  r=q[i];f.write(json.dumps({"id":f"ROOT-CAMERA-ADM03-SOURCE-{r['camera_queue_index']}","camera_queue_index":r["camera_queue_index"],"support_queue_index":i,"source_id":r["source_id"],"sha256":r["sha256"],"contract_ref":name,"status":"exact_source_role_compared_stated_lower_dependencies","repair":"frozen","validation":"not_established"})+"\n")
seen.update(ix);pending=sorted(set(range(100))-seen);(d/"progress.json").write_text(json.dumps({"assigned":100,"accounted_local_roles":len(seen),"first_unfinished_support_index":pending[0] if pending else None,"first_unfinished_camera_index":q[pending[0]]["camera_queue_index"] if pending else None,"source_only":True,"Camera_final_acceptance_pending":True,"Root_original471_immutable":True},indent=2)+"\n");print("Recorded",len(seen),"local roles; no global clearance or implementation.")
