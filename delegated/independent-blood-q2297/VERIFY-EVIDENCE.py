#!/usr/bin/env python3
"""Read-only evidence integrity verification; no product behavior validation.

Without --repo verifies packet files, source/Git blob identities, raw tree
chains, reconstructed displays and historical selected values. With --repo,
also binds commit roots and validates queue/ledger selections against the Git
objects. This script does not execute product code, tests, fixtures or macros.
"""
import argparse,hashlib,json,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser(); ap.add_argument('--repo'); args=ap.parse_args()
def sha(d):return hashlib.sha256(d).hexdigest()
def read(name):return json.loads((P/name).read_text())
def blob(d):return hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()
def ptr(v,path):
 if path in ('','/'):return v
 for s in path.strip('/').split('/'):
  s=s.replace('~1','/').replace('~0','~'); v=v[int(s)] if isinstance(v,list) else v[s]
 return v
def tree(raw):
 out={}; i=0
 while i<len(raw):
  sp=raw.index(b' ',i); z=raw.index(b'\0',sp+1); out[raw[sp+1:z].decode()]=(raw[i:sp].decode(),raw[z+1:z+21].hex()); i=z+21
 return out
manifest=read('MANIFEST.json'); indexed={x['path'] for x in manifest['files']}; actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'}; assert indexed==actual,'manifest membership'
for row in manifest['files']:
 d=(P/row['path']).read_bytes(); assert len(d)==row['bytes'] and sha(d)==row['sha256'],row['path']
receipts=read('READ-RECEIPTS.json'); paths={}; transcript=[]
for r in receipts['rows']:
 assert sha((P/r['registration']).read_bytes())==r['registration_sha256']; reg=read(r['registration']); found=[e for e in reg['sources'] if e['path']==r['path']]; assert len(found)==1; e=found[0]; assert e['planned_ranges']==r['planned_ranges']==r['actual_display_chunks']
 d=(P/r['source_capture']).read_bytes(); assert sha(d)==r['sha256'] and blob(d)==r['git_blob']; lines=d.decode().splitlines(); assert len(lines)==r['eof']; paths[r['path']]=d
 for a,b in r['actual_display_chunks']:
  transcript.append(f'--- {r["registration"]} {r["path"]} {r["revision"]} {a}-{b} ---\n'); transcript.extend(f'{i:6d}\t{lines[i-1]}\n' for i in range(a,b+1))
assert ''.join(transcript).encode()==(P/'DISPLAY-RECONSTRUCTED.txt').read_bytes()
proofs=read('GIT-TREE-PROOFS.json')['proofs']
for pr in proofs:
 assert pr['chain'][0]['tree']==pr['root_tree']; parts=pr['path'].split('/'); assert len(parts)==len(pr['chain'])
 for i,row in enumerate(pr['chain']):
  raw=(P/'git-trees'/(row['tree']+'.tree')).read_bytes(); assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==row['tree']; assert row['name']==parts[i] and tree(raw)[row['name']]==(row['mode'],row['object'])
  if i+1<len(parts):assert row['mode']=='40000' and row['object']==pr['chain'][i+1]['tree']
 assert pr['chain'][-1]['object']==pr['blob']; d=paths[pr['path']]; assert len(d)==pr['source_bytes'] and sha(d)==pr['source_sha256'] and blob(d)==pr['blob']
hreg=read('HISTORY-REGISTRATION.json'); selection=read('HISTORY-SELECTED-RECONSTRUCTED.json'); selected={x['record']:x['selections'] for x in selection['records']}; assert len(hreg['sources'])==len(selected)==4
for e in hreg['sources']:
 data=(P/'history-exact'/e['historical_relative_path']).read_bytes(); assert sha(data)==e['expected_sha256'] and len(data)==e['bytes']; doc=json.loads(data); rows=selected[e['historical_relative_path']]; assert [x['pointer'] for x in rows]==e['planned_selectors']
 for x in rows: assert ptr(doc,x['pointer'])==x['value']
hr=read('HISTORY-READ-RECEIPTS.json'); assert sha((P/hr['registration']).read_bytes())==hr['registration_sha256'] and sha((P/hr['selected_values']).read_bytes())==hr['selected_sha256']
q=read('QUEUE-2297-RECONSTRUCTED.json')['value']; row=q['exact_source_identities_and_assessed_ranges'][0]; expected={row['registration']['sha256'],row['body_receipt']['sha256'],row['root_adoption']['sha256'],q['proposal']['sha256']}; assert expected=={e['expected_sha256'] for e in hreg['sources']}
arch=(P/'history-exact/blood_attempts-archived.source').read_bytes(); assert sha(arch)==row['source_sha256']==sha((P/'sources/blood_attempts.rs').read_bytes())
old=[x for x in proofs if x['revision']=='6533e444fa37552927070cc8d8190907fd3b413e']; assert len(old)==1 and old[0]['source_sha256']==sha(arch)
if args.repo:
 def git(*a):return subprocess.check_output(['git','-C',args.repo,*a])
 for pr in proofs:
  assert git('rev-parse',pr['revision']+'^{tree}').decode().strip()==pr['root_tree']; d=git('show',pr['revision']+':'+pr['path']); assert sha(d)==pr['source_sha256'] and blob(d)==pr['blob']
 s=read('INITIAL-REGISTRATION.json')['queue']; d=git('show',s['revision']+':'+s['path']); assert sha(d)==s['sha256'] and blob(d)==s['blob']; qdoc=json.loads(d)
 for name in ['QUEUE-2297-RECONSTRUCTED.json','QUEUE-894-RECONSTRUCTED.json']:
  v=read(name); assert ptr(qdoc,v['pointer'])==v['value']
 related=read('RELATED-QUEUE-SELECTIONS.json')
 for item in related:
  original=ptr(qdoc,item['pointer']); assert item['fields']=={key:original[key] for key in item['fields']}
 s=read('LEDGER-REGISTRATION.json')['source']; d=git('show',s['revision']+':'+s['path']); assert sha(d)==s['sha256'] and blob(d)==s['git_blob']; v=read('LEDGER-ROW-RECONSTRUCTED.json'); assert ptr(json.loads(d),v['pointer'])==v['value']
print(json.dumps({'status':'passed','manifest_files':len(indexed),'tree_proofs':len(proofs),'historical_original_documents':len(hreg['sources']),'commit_roots_and_queue_ledger_selections_verified':bool(args.repo),'scope':'Evidence identity and selected reconstruction only; personal perception, mathematical validity, product behavior and audit closure are not verified.'}))
