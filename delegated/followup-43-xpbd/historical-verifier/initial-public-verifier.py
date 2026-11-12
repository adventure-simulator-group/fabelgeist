#!/usr/bin/env python3
"""Read-only transport integrity and qualified source-proof verification."""
import gzip,hashlib,json,pathlib
P=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def read(n):
 p=pathlib.PurePosixPath(n);assert not p.is_absolute() and '..' not in p.parts;q=P.joinpath(*p.parts);assert q.is_file() and not q.is_symlink();return q.read_bytes()
def load(n):return json.loads(read(n))
def tree_entries(d):
 out={};i=0
 while i<len(d):
  j=d.index(b' ',i);k=d.index(b'\0',j+1);name=d[j+1:k].decode();mode=d[i:j].decode();oid=d[k+1:k+21].hex();assert name not in out and len(d[k+1:k+21])==20;out[name]=(mode,oid);i=k+21
 return out
m=load('MANIFEST.json');expected={x['path']:x for x in m['files']};assert len(expected)==len(m['files']);actual=set()
for p in P.rglob('*'):
 assert not p.is_symlink()
 if p.is_file() and p!=P/'MANIFEST.json':actual.add(p.relative_to(P).as_posix())
assert actual==set(expected)
for n,v in expected.items():
 d=read(n);assert H(d)==v['sha256'] and len(d)==v['bytes']
idx=load('EVIDENCE-IDENTITIES.json');rows=idx['members'];by={(r['packet'],r['source_path']):r for r in rows};assert len(by)==len(rows)==128
for label,sha in idx['source_packets'].items():
 row=by[(label,'MANIFEST.json')];d=read(row['public_path']);assert H(d)==sha==row['original_stored_sha256']==row['public_stored_sha256'];old={x['path']:x for x in json.loads(d)['files']};assert {r['source_path'] for r in rows if r['packet']==label}==set(old)|{'MANIFEST.json'}
 for r in [x for x in rows if x['packet']==label and x['source_path']!='MANIFEST.json']:
  a=old[r['source_path']];assert r['source_manifest_bound'] and r['original_stored_sha256']==a['sha256'] and r['original_stored_bytes']==a['bytes']
counts={'path':0,'git-identity':0};changed=0
for r in rows:
 raw=read(r['public_path']);assert H(raw)==r['public_stored_sha256'] and len(raw)==r['public_stored_bytes'];u=gzip.decompress(raw) if r['encoding']=='gzip' else raw;assert H(u)==r['public_decoded_sha256'] and len(u)==r['public_decoded_bytes']
 if r['redactions']:
  changed+=1;assert r['status']=='privacy-reconstructed';delta=0;last=-1
  for op in r['redactions']:
   assert set(op)=={'kind','original_decoded_offset_0based','original_decoded_bytes','original_decoded_line_1based','public_decoded_offset_0based','public_placeholder'}
   a=op['original_decoded_offset_0based'];b=a+op['original_decoded_bytes'];assert a>=last and op['kind'] in counts;last=b;p=op['public_placeholder'].encode();pos=op['public_decoded_offset_0based'];assert pos==a-delta and u[pos:pos+len(p)]==p;delta+=op['original_decoded_bytes']-len(p);counts[op['kind']]+=1
  assert len(u)==r['original_decoded_bytes']-delta
 else:assert r['status']=='exact-copy' and r['original_stored_sha256']==r['public_stored_sha256'] and r['original_decoded_sha256']==r['public_decoded_sha256']
assert changed==8 and counts=={'path':13,'git-identity':12}
cross=load('HISTORICAL-ASSERTION-CROSSWALK.json');assert {(r['packet'],r['source_path']) for r in cross['members']}=={(r['packet'],r['source_path']) for r in rows if r['redactions']}
for q in cross['members']:
 r=by[(q['packet'],q['source_path'])];assert all(q[k]==r[k] for k in ('public_path','original_stored_sha256','original_decoded_sha256','public_stored_sha256','public_decoded_sha256'))
def member(packet,name):return by[(packet,name)]
def body(packet,name):
 q=member(packet,name);d=read(q['public_path']);return gzip.decompress(d) if q['encoding']=='gzip' else d
reg=json.loads(body('reconciliation','registration.json'));g=json.loads(body('reconciliation','git-proofs.json'))
def git_object(oid,typ):
 v=g['objects'][oid];r=member('reconciliation',v['file']);d=body('reconciliation',v['file']);assert H(d)==r['public_decoded_sha256'] and r['original_decoded_sha256']==v['sha256'] and r['original_decoded_bytes']==v['bytes']
 if typ=='tree':assert r['status']=='exact-copy' and hashlib.sha1((typ+' '+str(len(d))+'\0').encode()+d).hexdigest()==oid
 else:assert typ=='commit' and r['status']=='privacy-reconstructed' and sum(x['kind']=='git-identity' for x in r['redactions'])==2
 return d
for oid,v in g['objects'].items():git_object(oid,v['type'])
exact_leaf=0;qualified_leaf=0
for p in g['proofs']:
 c=git_object(p['revision'],'commit');header=c.split(b'\n',1)[0].decode();assert header.startswith('tree ');tree=header[5:];names=p['path'].split('/');assert len(names)==len(p['chain'])
 for i,(name,s) in enumerate(zip(names,p['chain'])):
  assert s['tree']==tree and s['name']==name;mode,oid=tree_entries(git_object(tree,'tree'))[name];assert s['mode']==mode and s['object']==oid;assert (i==len(names)-1 and mode in ('100644','100755')) or (i<len(names)-1 and mode=='40000');tree=oid
 r=member('reconciliation',p['local']);assert tree==p['blob']
 if r['status']=='exact-copy':
  d=body('reconciliation',p['local']);assert H(d)==p['sha256'] and hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==tree;exact_leaf+=1
 else:assert p['local']=='historical/baseline-059-queue-committed-2e1f.json.gz' and r['original_decoded_sha256']==p['sha256'];qualified_leaf+=1
assert len(g['proofs'])==23 and exact_leaf==22 and qualified_leaf==1
registry=json.loads(body('reconciliation','historical/original-registry.json.gz'))
for role in (2648,2649):
 q=next(x for x in reg['sources'] if x.get('role')==role and x['identity'].startswith('Recovered'));v=registry['files'][role];assert v['snapshot']=='pr-working' and v['sha256']==q['sha256'] and v['material_ranges']==q['original_material_ranges'];assert member('reconciliation',q['file'])['status']=='exact-copy'
hi=json.loads(body('reconciliation','prior/historical-index.json'));t=json.loads(body('reconciliation','historical/original-059-transport-manifest.json.gz'));te={x['path']:x for x in t['entries']};n059=0
for q in hi['records']:
 d=body('reconciliation','historical/'+q['file']);assert H(d)==q['sha256'] and len(d)==q['bytes'] and hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==q['git_blob']
 if q['kind']=='original-059':
  v=te[q['transport_path']];assert (H(d),len(d),q['git_blob'])==(v['sha256'],v['bytes'],v['git_blob']);n059+=1
assert n059==12
prov=load('PROVENANCE.json');linebytes=body('reconciliation','historical/history-774/findings.jsonl.gz').splitlines()
for q in prov['historical_774_stream']['findings']:
 d=read(q['exact_line']);a=json.loads(d);assert d==linebytes[q['line_1based']-1] and H(d)==q['line_sha256_without_terminator'] and a['id']==q['id'] and q['source_ids_as_recorded']==a.get('source_ids',[a['source_id']] if 'source_id' in a else [])
old=json.loads(body('reconciliation','historical/baseline-059-queue-committed-2e1f.json.gz'))['entries'];new=json.loads(body('reconciliation','historical/baseline-059-queue-committed-d7.json.gz'))['entries'];sel=json.loads(body('reconciliation','selected-059-queue.json'));prior=json.loads(body('reconciliation','prior/queue-entry-3138.json'))
assert len(sel['rows'])==len(prov['selected_059_rows'])==5
for r,p in zip(sel['rows'],prov['selected_059_rows']):
 i=r['index'];assert i==p['index'] and r['value']==old[i]==new[i] and r['equal_across_snapshots'] and p['contract_id']==old[i]['contract_id']
assert old[3138]==prior['entry']
w=body('reconciliation','reference/working-baseline.json.gz');c=body('reconciliation','reference/committed-baseline.json.gz');assert H(w)==reg['working_baseline_sha256']==prov['working_and_committed']['working_decoded_sha256'] and H(c)==reg['committed_baseline_sha256']==prov['working_and_committed']['committed_decoded_sha256'] and w!=c and not reg['snapshot_equal'];wa=json.loads(w)['newly_assessed_ledger']['entries'];ca=json.loads(c)['newly_assessed_ledger']['entries']
for q in reg['selected_current_ledger']:
 i=int(q['working_pointer'].split('/')[-1]);j=int(q['committed_pointer'].split('/')[-1]);assert wa[i]==ca[j] and wa[i]['id']==ca[j]['id']==q['id'] and q['selected_entry_equal']
add=json.loads(body('attribution-addendum','REPORT.json'));assert add['subject_report_sha256']==prov['source_reports']['reconciliation_final']['original_sha256']
for p in add['packets']:
 d=body('attribution-addendum',p['manifest_file']);assert H(d)==p['manifest_sha256'];oldmap={x['path']:x for x in json.loads(d)['files']}
 for q in p['members']:
  v=member('attribution-addendum',q['file']);assert v['original_stored_sha256']==q['sha256']==oldmap[q['source_name']]['sha256'] and v['original_stored_bytes']==q['bytes']==oldmap[q['source_name']]['bytes']
print(json.dumps({'status':'PASS','stage_manifest_files':len(expected),'source_members':len(rows),'reconstructed_members':changed,'path_substitutions':counts['path'],'git_identity_substitutions':counts['git-identity'],'exact_git_tree_objects':sum(v['type']=='tree' for v in g['objects'].values()),'commit_identities_qualified':sum(v['type']=='commit' for v in g['objects'].values()),'proofs_with_exact_leaf_bytes':exact_leaf,'proofs_with_redacted_leaf':qualified_leaf,'proofs_with_qualified_commit_root':len(g['proofs']),'historical_059_records_bound':n059,'historical_finding_lines':len(prov['historical_774_stream']['findings']),'selected_059_rows':len(sel['rows']),'selected_current_entries':len(reg['selected_current_ledger']),'addendum_prior_manifests':len(add['packets']),'qualification':'Public stored/decoded integrity, exact tree/leaf links and qualified original commit crosswalks only. It cannot verify original commit IDs or redacted original bytes without restricted originals; no product, runtime, privacy-completeness or caller-reachability claim.'},indent=2))
