#!/usr/bin/env python3
"""Read-only evidence integrity checks; never execute product code or tests.

Without --repo verifies packet membership, stored source blobs/tree chains,
registered read reconstruction, historical selections, prior q2297 report and
working-stage snapshot identity. With --repo also binds product revisions to
Git roots, 059 queue selections and recorded staged-vs-HEAD distinction.
"""
import argparse,hashlib,json,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--repo');args=ap.parse_args()
def sha(d):return hashlib.sha256(d).hexdigest()
def read(n):return json.loads((P/n).read_text())
def blob(d):return hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()
def ptr(v,p):
 if p in ('','/'):return v
 for s in p.strip('/').split('/'):
  s=s.replace('~1','/').replace('~0','~');v=v[int(s)] if isinstance(v,list) else v[s]
 return v
def entries(raw):
 i=0;out={}
 while i<len(raw):
  sp=raw.index(b' ',i);z=raw.index(b'\0',sp+1);out[raw[sp+1:z].decode()]=(raw[i:sp].decode(),raw[z+1:z+21].hex());i=z+21
 return out
manifest=read('MANIFEST.json');index={x['path'] for x in manifest['files']};actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'};assert index==actual,'manifest membership mismatch'
for e in manifest['files']:
 d=(P/e['path']).read_bytes();assert len(d)==e['bytes'] and sha(d)==e['sha256'],e['path']
receipts=read('READ-RECEIPTS.json');paths={};display=[]
for r in receipts['rows']:
 assert sha((P/r['registration']).read_bytes())==r['registration_sha256'];reg=read(r['registration']);matches=[e for e in reg['sources'] if e['path']==r['path']];assert len(matches)==1 and matches[0]['planned_ranges']==r['planned_ranges']==r['actual_display_chunks']
 d=(P/r['capture']).read_bytes();assert sha(d)==r['sha256'] and blob(d)==r['git_blob'];lines=d.decode().splitlines();assert len(lines)==r['eof'];paths[r['path']]=d
 for a,b in r['actual_display_chunks']:
  display.append(f'--- {r["registration"]} {r["path"]} {r["revision"]} {a}-{b} ---\n');display.extend(f'{i:6d}\t{lines[i-1]}\n' for i in range(a,b+1))
assert ''.join(display).encode()==(P/'DISPLAY-RECONSTRUCTED.txt').read_bytes()
proofs=read('GIT-TREE-PROOFS.json')['proofs']
for pr in proofs:
 assert pr['chain'][0]['tree']==pr['root_tree'];parts=pr['path'].split('/');assert len(parts)==len(pr['chain'])
 for i,row in enumerate(pr['chain']):
  raw=(P/'git-trees'/(row['tree']+'.tree')).read_bytes();assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==row['tree'];assert row['name']==parts[i] and entries(raw)[row['name']]==(row['mode'],row['object'])
  if i+1<len(parts):assert row['mode']=='40000' and row['object']==pr['chain'][i+1]['tree']
 d=paths[pr['path']];assert pr['chain'][-1]['object']==pr['blob'] and len(d)==pr['source_bytes'] and sha(d)==pr['source_sha256'] and blob(d)==pr['blob']
hreg=read('HISTORY-REGISTRATION.json');selected={x['record']:x['selections'] for x in read('HISTORY-SELECTED-RECONSTRUCTED.json')['records']};assert len(hreg['sources'])==len(selected)==4
for e in hreg['sources']:
 d=(P/'history-exact'/e['historical_relative_path']).read_bytes();assert sha(d)==e['expected_sha256'];doc=json.loads(d);items=selected[e['historical_relative_path']];assert [x['pointer'] for x in items]==e['planned_selectors']
 for x in items:assert ptr(doc,x['pointer'])==x['value']
hr=read('HISTORY-READ-RECEIPTS.json');assert sha((P/hr['registration']).read_bytes())==hr['registration_sha256'] and sha((P/hr['selected_values']).read_bytes())==hr['selected_sha256']
q=read('QUEUE-SELECTED-RECONSTRUCTED.json')[0]['value'];s=q['exact_source_identities_and_assessed_ranges'][0];expected={s['registration']['sha256'],s['body_receipt']['sha256'],s['root_adoption']['sha256'],q['proposal']['sha256']};assert expected=={e['expected_sha256'] for e in hreg['sources']}
for x in read('PRIOR-REGISTRATION.json')['sources']:
 d=(P/x['capture']).read_bytes();assert len(d)==x['bytes'] and sha(d)==x['sha256']
prior=read('prior/prior-q2297-manifest.json');x=next(e for e in prior['files'] if e['path']=='REPORT.json');assert sha((P/'prior/prior-q2297-report.json').read_bytes())==x['sha256']
stage=read('STAGE-IDENTITY-RECEIPT.json');assert sha((P/'prior/staged-13V-assessment.json').read_bytes())==stage['registered_snapshot_sha256'] and stage['working_bytes_still_match']
idproof=read('HISTORICAL-IDENTITY-REGISTRATION.json');pair=[x for x in proofs if x['path']==idproof['path']];assert len(pair)==2 and len({x['blob'] for x in pair})==1 and all(x['source_sha256']==idproof['queue_recorded_object_sha256'] for x in pair)
if args.repo:
 def git(*a):return subprocess.check_output(['git','-C',args.repo,*a])
 for pr in proofs:
  assert git('rev-parse',pr['revision']+'^{tree}').decode().strip()==pr['root_tree'];d=git('show',pr['revision']+':'+pr['path']);assert sha(d)==pr['source_sha256'] and blob(d)==pr['blob']
 s=read('INITIAL-REGISTRATION.json')['queue'];d=git('show',s['revision']+':'+s['path']);assert sha(d)==s['sha256'] and blob(d)==s['git_blob'];doc=json.loads(d)
 for x in read('QUEUE-SELECTED-RECONSTRUCTED.json'):assert ptr(doc,x['pointer'])==x['value']
 assert not git('ls-tree',stage['observed_committed_head'],'--','ASSESSMENT-CENTRAL-BLOOD-WINDOW-13V-95E.json').strip()
print(json.dumps({'status':'passed','manifest_files':len(index),'source_tree_proofs':len(proofs),'historic_original_records':len(hreg['sources']),'git_roots_059_and_recorded_stage_vs_head_checked':bool(args.repo),'scope':'Evidence identity only; no product behavior, human perception, reachability or audit closure is proven.'}))
