import argparse,datetime,hashlib,json,pathlib
p=pathlib.Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--extraction',required=True);root=pathlib.Path(a.parse_args().extraction);plan=json.loads((p/'HISTORY-PLAN.json').read_text());items=[]
for row in plan['items']:
 src=root/row['relative_path'];x=dict(row);x['found']=src.is_file()
 if x['found']:
  b=src.read_bytes();x['actual_sha256']=hashlib.sha256(b).hexdigest();x['bytes']=len(b);x['matched']=x['actual_sha256']==x['expected_sha256'];dest=p/x['capture'];dest.parent.mkdir(parents=True,exist_ok=True)
  with dest.open('xb') as f:f.write(b)
 else:x['matched']=False
 items.append(x)
x={'captured_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'items':items,'qualification':'Only records found at user-provided extraction; missing elsewhere not globally absent. Capture does not imply full personal display.'};(p/'HISTORY-CAPTURE.json').write_text(json.dumps(x,indent=2)+'\n');print(json.dumps([{'path':x['relative_path'],'found':x['found'],'matched':x['matched']} for x in items]))
