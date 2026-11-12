#!/usr/bin/env python3
"""Read-only provenance and integrity checks for this packet; no product code."""
import gzip,hashlib,json,pathlib
P=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def read(n):
 p=pathlib.PurePosixPath(n);assert not p.is_absolute() and '..' not in p.parts;q=P.joinpath(*p.parts);assert q.is_file() and not q.is_symlink();return q.read_bytes()
def load(n):return json.loads(read(n))
def tree_entries(raw):
 out={};i=0
 while i<len(raw):
  j=raw.index(b' ',i);k=raw.index(b'\0',j+1);name=raw[j+1:k].decode('utf-8');mode=raw[i:j].decode('ascii');oid=raw[k+1:k+21].hex();assert name not in out and len(raw[k+1:k+21])==20;out[name]=(mode,oid);i=k+21
 return out
manifest=load('MANIFEST.json');expected={x['path']:x for x in manifest['files']};assert len(expected)==len(manifest['files']);actual=set()
for f in P.rglob('*'):
 assert not f.is_symlink()
 if f.is_file() and f!=P/'MANIFEST.json':actual.add(f.relative_to(P).as_posix())
assert actual==set(expected)
for path,q in expected.items():
 d=read(path);assert len(d)==q['bytes'] and H(d)==q['sha256'],path
reg=load('registration.json');ext=load('support-extension-registration.json');receipt=load('read-receipt.json');assert len(receipt['source_displays'])==len(reg['sources'])+len(ext['sources'])
for q in receipt['registrations']:assert H(read(q['path']))==q['sha256']
for row,rr in zip(reg['sources']+ext['sources'],receipt['source_displays']):
 d=read(row['file']);assert H(d)==row['sha256'] and len(d.splitlines())==row['eof'] and row['prospective_ranges']==rr['actual_display_ranges']
 assert hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==row['git_blob']
 seen={i for a,b in row['prospective_ranges'] for i in range(a,b+1)};unread={i for a,b in rr['unread_complement'] for i in range(a,b+1)};assert not seen.intersection(unread) and seen|unread==set(range(1,row['eof']+1))
for r in reg['historical_records']:
 d=read(r['file']);assert H(d)==r['stored_sha256'];u=gzip.decompress(d);assert H(u)==r['decoded_sha256']
 if 'eof' in r:
  assert len(u.splitlines())==r['eof']
  for a,b in r['prospective_ranges']:assert 1<=a<=b<=r['eof']
g=load('git-proofs.json')
def obj(oid,typ):
 m=g['objects'][oid];assert m['type']==typ;d=gzip.decompress(read(m['file']));assert len(d)==m['bytes'] and H(d)==m['sha256'];assert hashlib.sha1((typ+' '+str(len(d))+'\0').encode()+d).hexdigest()==oid;return d
for oid,v in g['objects'].items():obj(oid,v['type'])
for p in g['proofs']:
 c=obj(p['revision'],'commit');head=c.split(b'\n',1)[0].decode();assert head.startswith('tree ');tree=head[5:];parts=p['path'].split('/');assert len(parts)==len(p['chain'])
 for i,(name,s) in enumerate(zip(parts,p['chain'])):
  assert s['tree']==tree and s['name']==name;mode,oid=tree_entries(obj(tree,'tree'))[name];assert s['mode']==mode and s['object']==oid;assert (i==len(parts)-1 and mode in ('100644','100755')) or (i<len(parts)-1 and mode=='40000');tree=oid
 d=read(p['local']);d=gzip.decompress(d) if p['compression']=='gzip' else d;assert H(d)==p['sha256'] and hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==tree==p['blob']
registry=json.loads(gzip.decompress(read('historical/original-registry.json.gz')))
for role in (2648,2649):
 q=next(x for x in reg['sources'] if x.get('role')==role and x['identity'].startswith('Recovered'));row=registry['files'][role];assert row['snapshot']=='pr-working' and row['sha256']==q['sha256'] and row['material_ranges']==q['original_material_ranges']
prior=load('prior/BUNDLE-MANIFEST.json');assert H(read('prior/BUNDLE-MANIFEST.json'))==reg['prior_manifest_sha256'];pm={x['path']:x for x in prior['files']};prior_checked=0
for n in ['report.json','registration.json','read-receipt.json','support-registration.json','support-read-receipt.json','historical-index.json','queue-entry-3138.json','git-proofs.json']:
 b=read('prior/'+n);q=pm[n];assert H(b)==q['sha256'] and len(b)==q['bytes'];prior_checked+=1
hi=load('prior/historical-index.json');transport=json.loads(gzip.decompress(read('historical/original-059-transport-manifest.json.gz')));te={v['path']:v for v in transport['entries']};history_count=0
for q in hi['records']:
 rel='historical/'+q['file'];stored=read(rel);m=pm[q['file']];assert H(stored)==m['sha256'] and len(stored)==m['bytes'];prior_checked+=1;d=gzip.decompress(stored) if q.get('encoding')=='gzip' else stored;assert len(d)==q['bytes'] and H(d)==q['sha256'] and hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==q['git_blob']
 if q['kind']=='original-059':
  t=te[q['transport_path']];assert (len(d),H(d),q['git_blob'])==(t['bytes'],t['sha256'],t['git_blob']);history_count+=1
assert history_count==12
oldraw=gzip.decompress(read('historical/baseline-059-queue-committed-2e1f.json.gz'));newraw=gzip.decompress(read('historical/baseline-059-queue-committed-d7.json.gz'));q=load('selected-059-queue.json');assert H(oldraw)==q['old_decoded_sha256'] and H(newraw)==q['later_decoded_sha256'] and oldraw!=newraw
old=json.loads(oldraw)['entries'];new=json.loads(newraw)['entries'];assert len(q['rows'])==5
for r in q['rows']:
 i=r['index'];assert r['equal_across_snapshots'] and r['value']==old[i]==new[i]
assert old[3138]==load('prior/queue-entry-3138.json')['entry']
w=gzip.decompress(read('reference/working-baseline.json.gz'));c=gzip.decompress(read('reference/committed-baseline.json.gz'));assert H(w)==reg['working_baseline_sha256'] and H(c)==reg['committed_baseline_sha256'] and (w==c)==reg['snapshot_equal'];wa=json.loads(w)['newly_assessed_ledger']['entries'];ca=json.loads(c)['newly_assessed_ledger']['entries']
for r in reg['selected_current_ledger']:
 i=int(r['working_pointer'].split('/')[-1]);j=int(r['committed_pointer'].split('/')[-1]);assert (wa[i]==ca[j])==r['selected_entry_equal'] and wa[i]['id']==ca[j]['id']==r['id']
report=load('REPORT-FINAL.json');assert report['earlier_draft']['sha256']==H(read(report['earlier_draft']['path'])) and report['scope']['personal_read_receipt']=='read-receipt.json'
print(json.dumps({'status':'PASS','manifest_files':len(expected),'raw_git_objects':len(g['objects']),'git_membership_proofs':len(g['proofs']),'prior_manifest_members_checked':prior_checked,'exact_original_059_records_transport_checked':history_count,'selected_historical_rows_equal':len(q['rows']),'selected_working_committed_entries':len(reg['selected_current_ledger']),'qualification':'Evidence integrity and identity only. Does not prove reader claims, source reasoning, privacy completeness, caller reachability or product behavior.'},indent=2))
