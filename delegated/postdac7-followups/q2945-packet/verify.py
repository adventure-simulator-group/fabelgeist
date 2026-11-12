#!/usr/bin/env python3
import argparse,hashlib,json,os,pathlib,stat,subprocess,sys
root=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--skip-manifest',action='store_true');p.add_argument('--private');p.add_argument('--anchors');p.add_argument('--ledger');p.add_argument('--historic-values');a=p.parse_args()
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def js(n):return json.loads((root/n).read_bytes())
def manifest(d,expected=None):
 raw=(d/'MANIFEST.json').read_bytes();assert expected is None or sha(raw)==expected;m=json.loads(raw);listed={x['path'] for x in m['files']};actual={str(x.relative_to(d)) for x in d.rglob('*') if x.is_file()}-{'MANIFEST.json'};assert listed==actual
 for x in m['files']:
  f=d/x['path'];b=f.read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'] and stat.S_IMODE(f.stat().st_mode)==x['mode'],x['path']
 return len(listed)
if a.skip_manifest:print('NOT CHECKED: public manifest (--skip-manifest)')
else:print('public manifest and file modes checked:',manifest(root))
def tree(t):
 b=(root/'tree-objects'/(t+'.tree')).read_bytes();assert oid('tree',b)==t;d={};i=0
 while i<len(b):
  j=b.index(b' ',i);k=b.index(b'\0',j+1);name=b[j+1:k].decode();assert name not in d and len(b[k+1:k+21])==20;d[name]=(b[i:j].decode(),b[k+1:k+21].hex());i=k+21
 assert i==len(b);return d
def chain(row,start):
 steps=row['chain'];parts=row['path'].split('/') if 'path' in row else [x['component'] for x in steps];assert steps[0]['tree']==start and parts==[x['component'] for x in steps]
 for i,x in enumerate(steps):
  assert tree(x['tree'])[x['component']]==(x['mode'],x['oid'])
  if i+1<len(steps):assert x['mode']=='40000' and x['oid']==steps[i+1]['tree']
 assert steps[-1]['mode']=='100644' and steps[-1]['oid']==row['blob']
proof=js('TREE-PROOF-020-RESULT.json');assert proof['status']=='pass' and len(proof['rows'])==3;root95=None
for row in proof['rows']:
 chain(row,row['chain'][0]['tree'])
 if row['label']=='95e':root95=row['chain'][0]['tree']
 n='95e-recovered.rs' if row['label']=='95e' else row['label']+'.rs';src=(root/'source'/n).read_bytes();assert oid('blob',src)==row['blob'] and sha(src)=='9db0cfc4b4c3053884d835faaca49a404c9942f7f0599b3069b89729fd8d7ae3' and len(src.splitlines())==209
assert root95 and proof['all_file_blob_ids_equal']
for row in js('ADJ-PROOF-029-RESULT.json')['rows']:
 assert row['status']=='pass';chain(row,root95);src=(root/'adjacent'/row['file']).read_bytes();assert sha(src)==row['sha256'] and oid('blob',src)==row['blob'] and len(src.splitlines())==row['eof']
for filename in ['OWNER-PROOF-035-RESULT.json','ID-PROOF-040-RESULT.json']:
 for row in js(filename)['rows']:
  assert row['status']=='pass';chain(row,root95);src=(root/'candidate'/(row['label']+'.rs')).read_bytes();assert oid('blob',src)==row['blob'];assert 'sha256' not in row or sha(src)==row['sha256']
def render(src,start,end):
 lines=src.splitlines(keepends=True);assert 1<=start<=end<=len(lines);return b''.join(str(i).encode()+b': '+lines[i-1] for i in range(start,end+1))
for row in js('DISPLAY-021-RECEIPT.json')['rows']:
 n='95e-recovered.rs' if row['label']=='95e' else row['label']+'.rs';v=(root/'displays'/(row['label']+'-1-209.txt')).read_bytes();assert sha(v)==row['sha256'] and v==render((root/'source'/n).read_bytes(),*row['range'])
for row in js('ADJ-DISPLAY-032-RECEIPT.json')['rows']:
 v=(root/'adjacent-displays'/(row['name']+'.txt')).read_bytes();assert sha(v)==row['sha256'] and v==render((root/'adjacent'/row['source']).read_bytes(),*row['range'])
for reg,res in [('OWNER-DISPLAY-037-REG.json','OWNER-DISPLAY-037-RESULT.json'),('ID-DISPLAY-041-REG.json','ID-DISPLAY-041-RESULT.json')]:
 plan={x['name']:x for x in js(reg)['ranges']}
 for x in js(res):
  v=(root/'adjacent-displays'/(x['name']+'.txt')).read_bytes();r=plan[x['name']];assert sha(v)==x['sha256'] and v==render((root/'candidate'/r['file']).read_bytes(),r['start'],r['end'])
for name,source,result in [('orientation.txt','candidate/buildings.rs','ANGLE-DISPLAY-046-RESULT.json'),('fixed-members.txt','adjacent/DESCRIPTOR-CURRENT-1.rs','BOUNDARY-DISPLAY-048-RESULT.json')]:
 x=js(result);v=(root/'adjacent-displays'/name).read_bytes();assert sha(v)==x['sha256'] and v==render((root/source).read_bytes(),*x['range'])
assert sha((root/'SKILL-EXISTING-TYPE-OWNERS.md').read_bytes())=='0876e49f57d2b5397d23735b9c49916d5d20087b33c110746beaa73c1c8cba3f'
print('source blobs, raw tree paths, 3 full and 13 adjacent displays checked; source roots not yet bound to commits')
if any([a.anchors,a.ledger,a.historic_values]) and not a.private:raise SystemExit('external anchors require --private')
if a.private:
 d=pathlib.Path(a.private);n=manifest(d,'bc5921dbac1302cca7785c69d98837bdcc4b79949fbfb429775df559691318f3');cmd=[sys.executable,str(d/'verify-evidence.py')]
 for flag,value in [('--anchors',a.anchors),('--ledger',a.ledger),('--historic-values',a.historic_values)]:
  if value:cmd.extend([flag,value])
 r=subprocess.run(cmd,capture_output=True,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'});assert r.returncode==0,('restricted evidence replay failed',r.returncode)
 print('restricted manifest, commit/tree bindings and named historical receipts checked:',n)
 for value,label in [(a.anchors,'full 059 queue membership'),(a.ledger,'dac7d ledger membership'),(a.historic_values,'historic lens mapping membership')]:print(('checked: ' if value else 'NOT CHECKED: ')+label)
else:print('NOT CHECKED: commit-to-root binding, original queue and receipts, immutable ledger membership, historic lens mapping; restricted originals required')
print('evidence-only replay; no product execution or runtime validation')
