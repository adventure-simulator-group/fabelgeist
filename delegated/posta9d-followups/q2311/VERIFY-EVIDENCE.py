#!/usr/bin/env python3
"""Read-only evidence verification; optional original package and repo inputs."""
import argparse,hashlib,json,pathlib,subprocess,tarfile
P=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--repo');ap.add_argument('--native-archive');ap.add_argument('--private-provenance');args=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def read(n):return json.loads((P/n).read_text())
def ptr(v,p):
 if p in ('','/'):return v
 for s in p.strip('/').split('/'):
  s=s.replace('~1','/').replace('~0','~');v=v[int(s)] if isinstance(v,list) else v[s]
 return v
def entries(b):
 i=0;out={}
 while i<len(b):
  s=b.index(b' ',i);z=b.index(b'\0',s+1);out[b[s+1:z].decode()]=(b[i:s].decode(),b[z+1:z+21].hex());i=z+21
 return out
m=read('MANIFEST.json');idx={x['path'] for x in m['files']};actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'};assert idx==actual,'manifest membership mismatch'
for x in m['files']:
 b=(P/x['path']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'],x['path']
receipts=read('READ-RECEIPTS.json')['events'];transcript=['Reconstructed source displays from exact retained bytes; not independent proof of personal attention.\n']
for r in receipts:
 j=read(r['registration']);rows=j.get('rows',[j]);row=next(x for x in rows if x.get('path',x.get('member'))==r['path'] and x['capture']==r['capture']);assert r['planned_ranges']==row['planned_ranges'];b=(P/r['capture']).read_bytes();assert sha(b)==r['sha256'];lines=b.decode().splitlines();assert len(lines)==r['eof']; planned={n for a,z in r['planned_ranges'] for n in range(a,z+1)};displayed={n for a,z in r['actual_display_chunks'] for n in range(a,z+1)};unread={n for a,z in r['unread_complement_for_this_registration'] for n in range(a,z+1)};assert planned==displayed and not displayed&unread and displayed|unread==set(range(1,len(lines)+1))
 transcript.append('\n=== '+r['registration']+' '+r['revision']+':'+r['path']+' ===\n')
 for a,z in r['actual_display_chunks']:
  transcript.append('--- %d-%d ---\n'%(a,z));transcript.extend('%5d %s\n'%(n,lines[n-1]) for n in range(a,z+1))
assert ''.join(transcript).encode()==(P/'DISPLAY-RECONSTRUCTED.txt').read_bytes()
g=read('GIT-TREE-PROOFS.json');proofs=g['proofs']
for pr in proofs:
 chain=pr['chain'];parts=pr['path'].split('/');assert pr['root_tree']==g['anchors'][pr['revision']]==chain[0]['tree'] and len(parts)==len(chain)
 for i,x in enumerate(chain):
  b=(P/'git-trees'/(x['tree']+'.tree')).read_bytes();assert hashlib.sha1(b'tree '+str(len(b)).encode()+b'\0'+b).hexdigest()==x['tree'];assert x['name']==parts[i] and entries(b)[x['name']]==(x['mode'],x['oid'])
  if i+1<len(chain):assert x['mode']=='40000' and x['oid']==chain[i+1]['tree']
 b=(P/pr['capture']).read_bytes();assert pr['blob']==chain[-1]['oid']==blob(b) and pr['sha256']==sha(b)
q=read('QUEUE-2311-RECONSTRUCTED.json')['value'];history=read('HISTORY-REGISTRATION.json')['records'];expected=set()
for x in q['exact_source_identities_and_assessed_ranges']:
 expected.update([x['body_receipt']['sha256'],x['registration_ref_and_sha256']['sha256'],x['root_adoption']['sha256'],x['root_adoption']['verification_sha256']])
expected.add(q['proposal']['sha256']);assert expected=={x['sha256'] for x in history}
for x in history:
 b=(P/'history-exact'/x['record']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'];doc=json.loads(b);selection=read('history-selected/'+x['record'].replace('/','__'));assert set(selection)==set(x['planned_selectors'])
 for pointer,v in selection.items():
  try:a=ptr(doc,pointer)
  except (KeyError,IndexError,TypeError,ValueError):a={'selector_not_present':True}
  assert a==v
for filename,entry in read('HISTORY-ROOT-DISPLAY.json').items():
 doc=read('history-exact/segovb/'+filename)
 for x in entry['rows']:assert doc['rows'][x['index']]==x['value']
 for x in entry['adoption_maps']:assert doc['adopted_registration_mapping'][x['index']]==x['value']
for x in read('PRIOR-REGISTRATION.json')['records']:
 b=(P/x['capture']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256']
rec=read('PUBLIC-RECONSTRUCTION.json');pub=(P/rec['provenance']['public_path']).read_bytes();assert len(pub)==rec['provenance']['public_bytes'] and sha(pub)==rec['provenance']['public_sha256'];prov=json.loads(pub);assert prov['source_archive']=='<redacted-local-path>';assert not (P/'native/rawzip-0.5.2.crate').exists() and not (P/'native/MATERIAL-PROVENANCE.json').exists()
expected_members={e['archive_member']:e['sha256'] for e in prov['exactmembers']}
for name in ['locator.rs','archive.rs','extra_fields.rs']:
 member='rawzip-0.5.2/src/'+name;assert sha((P/'native'/name).read_bytes())==expected_members[member]
qmembers={x['source_id'].split(':')[2]:x['source_sha256'] for x in q['exact_source_identities_and_assessed_ranges'] if x['source_id'].startswith('native:')}
for member,h in qmembers.items():assert expected_members['rawzip-0.5.2/'+member]==h
if args.private_provenance:
 b=pathlib.Path(args.private_provenance).read_bytes();assert len(b)==rec['provenance']['original_bytes'] and sha(b)==rec['provenance']['original_sha256'];x=json.loads(b);assert x['source_archive'].startswith('/');x['source_archive']='<redacted-local-path>';assert x==prov
if args.native_archive:
 a=pathlib.Path(args.native_archive);b=a.read_bytes();assert len(b)==rec['native_package_archive']['original_bytes'] and sha(b)==rec['native_package_archive']['original_sha256']
 with tarfile.open(a,'r:gz') as t:
  for name in ['locator.rs','archive.rs','extra_fields.rs']:assert t.extractfile('rawzip-0.5.2/src/'+name).read()==(P/'native'/name).read_bytes()
if args.repo:
 def git(*a):return subprocess.check_output(['git','-C',args.repo,*a])
 for pr in proofs:
  assert git('rev-parse',pr['revision']+'^{tree}').decode().strip()==pr['root_tree'];b=git('show',pr['revision']+':'+pr['path']);assert blob(b)==pr['blob'] and sha(b)==pr['sha256']
 qr=read('QUEUE-REGISTRATION.json')['source'];b=git('show',qr['revision']+':'+qr['path']);assert len(b)==qr['bytes'] and sha(b)==qr['sha256'] and blob(b)==qr['git_blob'];s=read('QUEUE-2311-RECONSTRUCTED.json');assert ptr(json.loads(b),s['pointer'])==s['value']
 for x in read('PRIOR-REGISTRATION.json')['records']:
  if x['revision']:assert git('show',x['revision']+':'+x['path'])==(P/x['capture']).read_bytes()
 related=read('RELATED-RECONSTRUCTED.json');original=json.loads(git('show',qr['revision']+':'+qr['path']))['entries'];rr=read('RELATED-REGISTRATION.json');assert [x['index'] for x in related]==rr['indices']
 for x in related:
  assert x['fields']=={k:original[x['index']].get(k) for k in rr['fields']}
 lr=read('LEDGER-NAV-REGISTRATION.json');b=git('show',lr['revision']+':'+lr['path']);assert sha(b)==lr['sha256'] and len(b)==lr['bytes']
print(json.dumps({'status':'passed','manifest_files':len(idx),'display_events':len(receipts),'tree_proofs':len(proofs),'historical_records':len(history),'repo_checked':bool(args.repo),'native_archive_membership_checked':bool(args.native_archive),'private_provenance_replay_checked':bool(args.private_provenance),'scope':'Evidence integrity only; no product validation, full privacy assurance, feature resolution, source comprehension or audit closure.'}))
