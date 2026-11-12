#!/usr/bin/env python3
"""Read-only evidence-integrity checks, not semantic or product validation."""
import argparse,hashlib,json,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--repo');args=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def read(p):return json.loads((P/p).read_text())
def ptr(v,p):
 if p in ('','/'):return v
 for s in p.strip('/').split('/'):
  s=s.replace('~1','/').replace('~0','~');v=v[int(s)] if isinstance(v,list) else v[s]
 return v
def tree_entries(b):
 i=0;out={}
 while i<len(b):
  s=b.index(b' ',i);z=b.index(b'\0',s+1);out[b[s+1:z].decode()]=(b[i:s].decode(),b[z+1:z+21].hex());i=z+21
 return out
m=read('MANIFEST.json');index={r['path'] for r in m['files']};actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'};assert index==actual,'manifest membership'
for r in m['files']:
 b=(P/r['path']).read_bytes();assert len(b)==r['bytes'] and sha(b)==r['sha256'],r['path']
receipts=read('READ-RECEIPTS.json');transcript=['Reconstructed text of the personally displayed registered source windows; this transcript is generated from captured exact source bytes and is not independent proof of human/model attention.\n']
for r in receipts['events']:
 reg=read(r['registration']);candidates=reg.get('sources',reg.get('rows',[reg]));row=next(x for x in candidates if x['path']==r['path'] and x['capture']==r['capture']);assert row['planned_ranges']==r['actual_ranges'];b=(P/r['capture']).read_bytes();assert sha(b)==r['sha256'];lines=b.decode().splitlines();assert len(lines)==r['eof'];transcript.append('\n=== '+r['registration']+' '+r['revision']+':'+r['path']+' ===\n')
 covered=set()
 for a,z in r['actual_ranges']:
  assert 1<=a<=z<=len(lines);transcript.append('--- %d-%d ---\n'%(a,z));covered.update(range(a,z+1));transcript.extend('%5d %s\n'%(n,lines[n-1]) for n in range(a,z+1))
 unread={n for a,z in r['unread_complement_for_this_registration'] for n in range(a,z+1)};assert covered.isdisjoint(unread) and covered|unread==set(range(1,len(lines)+1))
assert ''.join(transcript).encode()==(P/'DISPLAY-RECONSTRUCTED.txt').read_bytes()
p=read('GIT-TREE-PROOFS.json');proofs=p['proofs']
for pr in proofs:
 assert p['anchors'][pr['revision']]==pr['root_tree'];parts=pr['path'].split('/');chain=pr['tree_chain'];assert len(parts)==len(chain) and chain[0]['tree']==pr['root_tree']
 for i,row in enumerate(chain):
  b=(P/'git-trees'/(row['tree']+'.tree')).read_bytes();assert hashlib.sha1(b'tree '+str(len(b)).encode()+b'\0'+b).hexdigest()==row['tree'];assert row['name']==parts[i] and tree_entries(b)[row['name']]==(row['mode'],row['oid'])
  if i+1<len(chain):assert row['mode']=='40000' and row['oid']==chain[i+1]['tree']
 b=(P/pr['capture']).read_bytes();assert chain[-1]['oid']==pr['blob']==blob(b) and sha(b)==pr['sha256']
hr=read('HISTORY-REGISTRATION.json')['records'];q=read('QUEUE-2304-RECONSTRUCTED.json')['value'];expected=set()
for x in q['exact_source_identities_and_assessed_ranges']:
 expected.update([x['body_receipt']['sha256'],x['registration_ref_and_sha256']['sha256'],x['root_adoption']['sha256'],x['root_adoption']['verification_sha256']])
expected.add(q['proposal']['sha256']);assert expected=={x['sha256'] for x in hr}
for x in hr:
 b=(P/'history-exact'/x['record']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'];original=json.loads(b);selection=read('history-selected/'+x['record'].replace('/','__'));assert set(selection)==set(x['prospective_selectors'])
 for pointer,v in selection.items():
  try: actual_value=ptr(original,pointer)
  except (KeyError,IndexError,TypeError,ValueError):actual_value={'selector_not_present':True}
  assert v==actual_value
for x in read('PRIOR-REGISTRATION.json')['records']:
 b=(P/x['capture']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256']
for x in read('HISTORICAL-IDENTITIES.json'):
 assert x['blob_and_sha_equal'] and len({(z['blob'],z['sha256'],z['bytes']) for z in x['identities'].values()})==1
 for rev,info in x['identities'].items():
  pr=next(z for z in proofs if z['revision']==rev and z['path']==x['path']);assert pr['blob']==info['blob'] and pr['sha256']==info['sha256']
if args.repo:
 def git(*a):return subprocess.check_output(['git','-C',args.repo,*a])
 for pr in proofs:
  assert git('rev-parse',pr['revision']+'^{tree}').decode().strip()==pr['root_tree'];b=git('show',pr['revision']+':'+pr['path']);assert blob(b)==pr['blob'] and sha(b)==pr['sha256']
 qreg=read('QUEUE-REGISTRATION.json')['source'];b=git('show',qreg['revision']+':'+qreg['path']);assert sha(b)==qreg['sha256'] and blob(b)==qreg['git_blob'];selected=read('QUEUE-2304-RECONSTRUCTED.json');assert ptr(json.loads(b),selected['pointer'])==selected['value']
 lr=read('LEDGER-REGISTRATION.json');b=git('show',lr['revision']+':'+lr['path']);assert sha(b)==lr['source_sha256'];doc=json.loads(b)
 for pointer,v in read('LEDGER-RECONSTRUCTED.json').items():assert ptr(doc,pointer)==v
 for x in read('PRIOR-REGISTRATION.json')['records']:
  if x['revision']:assert git('show',x['revision']+':'+x['path'])==(P/x['capture']).read_bytes()
print(json.dumps({'status':'passed','manifest_files':len(index),'source_read_events':len(receipts['events']),'tree_proofs':len(proofs),'historical_records':len(hr),'repo_anchors_queue_ledger_and_committed_prior_checked':bool(args.repo),'staged_prior_note':'working prior records verified against recorded content hash only, not committed or independent custody','scope':'Evidence integrity only; does not establish semantic correctness, personal perception, runtime behavior, or audit closure.'}))
