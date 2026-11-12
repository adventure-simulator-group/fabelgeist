#!/usr/bin/env python3
"""Read-only bounded public reconstruction checks; not product validation."""
import datetime,gzip,hashlib,json,pathlib
P=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def read(p):
 q=pathlib.PurePosixPath(p);assert not q.is_absolute() and '..' not in q.parts
 p=P.joinpath(*q.parts);assert p.is_file() and not p.is_symlink();return p.read_bytes()
def js(p):return json.loads(read(p))
manifest=js('MANIFEST.json');expected={x['path']:x for x in manifest['files']};assert len(expected)==len(manifest['files'])
actual=set()
for p in P.rglob('*'):
 assert not p.is_symlink()
 if p.is_file() and p!=P/'MANIFEST.json':actual.add(p.relative_to(P).as_posix())
assert actual==set(expected)
for name,row in expected.items():b=read(name);assert (H(b),len(b))==(row['sha256'],row['bytes']),name
idx=js('EVIDENCE-IDENTITIES.json');rows=idx['members'];by={x['source_path']:x for x in rows};assert len(by)==len(rows)==167
for r in rows:
 b=read(r['public_path']);assert (H(b),len(b))==(r['public_stored_sha256'],r['public_stored_bytes'])
 d=gzip.decompress(b) if r['encoding']=='gzip' else b;assert (H(d),len(d))==(r['public_decoded_sha256'],r['public_decoded_bytes'])
 for o in r['redactions']:
  mark=o['public_placeholder'].encode();a=o['public_decoded_offset_0based'];assert d[a:a+len(mark)]==mark
 assert (r['status']=='exact-copy')==(not r['redactions'])
 if r['status']=='exact-copy':assert (r['original_stored_sha256'],r['original_stored_bytes'])==(H(b),len(b))
# Source manifests are retained as historical assertions and bind originals,
# not the reconstructed bytes. Check every original/public crosswalk explicitly.
for label in ('original','corrected','verifier-addendum'):
 m=js('bundle/'+label+'/MANIFEST.json');assert H(read('bundle/'+label+'/MANIFEST.json'))==idx['source_manifests'][label]
 for x in m['files']:
  r=by[label+'/'+x['path']];assert (r['original_stored_sha256'],r['original_stored_bytes'])==(x['sha256'],x['bytes'])
assert H(read('bundle/corrected/initial/MANIFEST.json'))==idx['source_manifests']['original']
for r in rows:
 if r['source_path'].startswith('original/'):
  counterpart=by['corrected/initial/'+r['source_path'][len('original/'):]]
  assert r['original_stored_sha256']==counterpart['original_stored_sha256']
original='bundle/original/';corrected='bundle/corrected/'
assert H(read(original+'INITIAL-REVIEW.json'))=='e3136dc3ff3c8ab89c87909f47b36b241769e9f4718ab0c626e8fdbc2b2bb576'
assert H(read(original+'coordinator/ASSESSMENT.json'))=='3b03e72fb781089e7974fe20c2e96ce66ceaa505963f281af61f6d3056f02106'
assert H(read(corrected+'coordinator/FINAL-ASSESSMENT.json'))=='ac429433dd7c22ab6d0db209a410bd867c48e87a76516c4734841471fe9ad68f'
assert H(read(corrected+'FINAL-REVIEW.json'))=='6ad953cb1f1bef162a16a0f9605a827072e8595c14843b2976b2a8c42d5bea16'
old=js(original+'coordinator/ASSESSMENT.json');final=js(corrected+'coordinator/FINAL-ASSESSMENT.json')
assert {k for k in set(old)|set(final) if old.get(k)!=final.get(k)}=={'central_display_evidence','observations','review_qualification','source_references'}
reg=js(corrected+'coordinator/EXT-REGISTRATION.json');rec=js(corrected+'coordinator/EXT-READ-RECEIPT.json');assert H(read(corrected+'coordinator/EXT-REGISTRATION.json'))==rec['registration_sha256'];assert H(read(corrected+'coordinator/EXT-DISPLAY.txt'))==rec['display_sha256'];assert datetime.datetime.fromisoformat(reg['registered_utc'])<datetime.datetime.fromisoformat(rec['recorded_utc'])
lines=[]
for e in reg['entries']:
 raw=read(original+'source/'+e['tag']+'.rs');assert H(raw)==e['sha256'];src=raw.decode().splitlines();rr=next(q for q in rec['rows'] if q['tag']==e['tag']);assert rr['displayed_ranges']==e['registered_ranges']
 for a,b in e['registered_ranges']:
  lines.append(f"--- {e['tag']} {a}-{b} ---");lines.extend(f'{i:4d} {src[i-1]}' for i in range(a,b+1))
assert ('\n'.join(lines)+'\n').encode()==read(corrected+'coordinator/EXT-DISPLAY.txt')
entries=js(original+'coordinator/REGISTRATION.json')['entries']+reg['entries']
for ref in final['source_references']:
 coverage=set()
 for e in entries:
  if e['git_path']==ref['path']:
   for a,b in e['registered_ranges']:coverage.update(range(a,b+1))
 for a,b in ref['lines']:assert set(range(a,b+1))<=coverage
# Git tree identities and path chains can be checked publicly, but the
# redacted commit identity is a qualified crosswalk only.
g=js(original+'git-proofs.json')
def obj(oid,kind):
 x=g['objects'][oid];assert x['type']==kind;b=gzip.decompress(read(original+x['file']))
 if kind=='tree':assert H(b)==x['sha256'] and hashlib.sha1((kind+' '+str(len(b))+'\0').encode()+b).hexdigest()==oid
 else:assert b.startswith(b'tree ') and b'author <REDACTED_GIT_AUTHOR>' in b and b'committer <REDACTED_GIT_COMMITTER>' in b
 return b
def tree(d):
 out={};i=0
 while i<len(d):
  j=d.index(b' ',i);k=d.index(b'\0',j+1);out[d[j+1:k].decode()]=(d[i:j].decode(),d[k+1:k+21].hex());i=k+21
 return out
for oid,x in g['objects'].items():obj(oid,x['type'])
for proof in g['proofs']:
 root=obj(proof['revision'],'commit').split(b'\n',1)[0].decode().split(' ',1)[1]
 for part,step in zip(proof['path'].split('/'),proof['chain']):
  assert root==step['tree'] and part==step['name'];mode,oid=tree(obj(root,'tree'))[part];assert (mode,oid)==(step['mode'],step['object']);root=oid
 raw=read(original+proof['local']);raw=gzip.decompress(raw) if proof['compression']=='gzip' else raw
 assert H(raw)==proof['sha256'] and hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==proof['blob']==root
sel=js(original+'SELECTED-059-QUEUE.json');assert [x['index'] for x in sel['rows']]==[1843,1844,1845]
q1=json.loads(gzip.decompress(read(original+'history/baseline-059-queue-committed-2e1f.json.gz')))['entries'];q2=json.loads(gzip.decompress(read(original+'history/baseline-059-queue-committed-d7.json.gz')))['entries']
for x in sel['rows']:assert x['value']==q1[x['index']]==q2[x['index']]
rg=js(original+'REGISTRATION.json');w=gzip.decompress(read(original+'ledger/working.json.gz'));c=gzip.decompress(read(original+'ledger/committed.json.gz'));assert H(w)==rg['working_baseline_sha256'] and H(c)==rg['committed_baseline_sha256'] and w!=c
wa=json.loads(w)['newly_assessed_ledger']['entries'];ca=json.loads(c)['newly_assessed_ledger']['entries']
for x in rg['selected_ledger']:
 a=wa[x['working_index']];found=[z for z in ca if z.get('id')==x['id']];assert a['id']==x['id'] and bool(found)==(x['committed_index'] is not None) and (bool(found) and found[0]==a)==x['selected_equal']
for fn,line,id in [('contract-groups.jsonl.gz',628,'RIG-ADMISSION03-CONTRACT-628'),('findings.jsonl.gz',57,'RIG-ADMISSION03-F054'),('source-range-registration.jsonl.gz',101,'RIG-ADMISSION03-REGISTER-101'),('additional-contracts.jsonl.gz',87,'RIG-ADMISSION03-ADDITIONAL-101')]:
 assert json.loads(gzip.decompress(read(original+'prior42/historical-774/'+fn)).splitlines()[line-1])['id']==id
print(json.dumps({'status':'PASS','source_members':len(rows),'qualified_commit_roots':2,'exact_git_leaf_proofs':len(g['proofs']),'reconstructed_extension_lines':len(lines),'selected_059_rows':len(sel['rows']),'selected_ledger_relations':len(rg['selected_ledger']),'scope':'Evidence integrity and recorded scopes only; no original commit identity proof, actual read cognition, remote state or product validation.'},indent=2))
