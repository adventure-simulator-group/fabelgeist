#!/usr/bin/env python3
"""Read-only independent-review source, identity and attribution checks."""
import gzip,hashlib,json,pathlib
P=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def read(n):
 p=pathlib.PurePosixPath(n);assert not p.is_absolute() and '..' not in p.parts;q=P.joinpath(*p.parts);assert q.is_file() and not q.is_symlink();return q.read_bytes()
def load(n):return json.loads(read(n))
def entries(d):
 out={};i=0
 while i<len(d):
  j=d.index(b' ',i);k=d.index(b'\0',j+1);mode=d[i:j].decode();name=d[j+1:k].decode();oid=d[k+1:k+21].hex();assert name not in out and len(d[k+1:k+21])==20;out[name]=(mode,oid);i=k+21
 return out
m=load('MANIFEST.json');fs={x['path']:x for x in m['files']};actual=set()
for f in P.rglob('*'):
 assert not f.is_symlink()
 if f.is_file() and f!=P/'MANIFEST.json':actual.add(f.relative_to(P).as_posix())
assert actual==set(fs)
for n,v in fs.items():b=read(n);assert H(b)==v['sha256'] and len(b)==v['bytes']
reg=load('REGISTRATION.json');rec=load('READ-RECEIPT.json');review=load('INITIAL-REVIEW.json');assert H(read('coordinator/ASSESSMENT.json'))==reg['assessment_sha256']==review['reviewed_assessment']['sha256']=='3b03e72fb781089e7974fe20c2e96ce66ceaa505963f281af61f6d3056f02106'
for x in rec['registration_hashes']:assert H(read(x['path']))==x['sha256']
for x in reg['sources']:
 b=read(x['file']);assert H(b)==x['sha256'] and len(b.splitlines())==x['eof'] and hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==x['git_blob']
assert len(rec['source_displays'])==7
for n in ['REGISTRATION-SUPPORT.json','REGISTRATION-VEC3.json','REGISTRATION-NO-COLLIDER.json']:
 x=load(n)['source'];b=read(x['file']);assert H(b)==x['sha256'] and len(b.splitlines())==x['eof']
cr=load('coordinator/REGISTRATION.json');rr=load('coordinator/READ-RECEIPT.json');assert H(read('coordinator/REGISTRATION.json'))==rr['registration_sha256'] and H(read('coordinator/DISPLAY.txt'))==rr['display_sha256'];lines=[]
for x,y in zip(cr['entries'],rr['rows']):
 assert x['tag']==y['tag'] and x['registered_ranges']==y['displayed_ranges'];raw=read('source/'+x['tag']+'.rs');assert H(raw)==x['sha256'];source=raw.decode().splitlines()
 for a,b in x['registered_ranges']:
  lines.append(f'--- {x["tag"]} {a}-{b} ---');lines.extend(f'{i:4d} {source[i-1]}' for i in range(a,b+1))
assert ('\n'.join(lines)+'\n').encode()==read('coordinator/DISPLAY.txt')
g=load('git-proofs.json')
def obj(oid,typ):
 x=g['objects'][oid];assert x['type']==typ;b=gzip.decompress(read(x['file']));assert H(b)==x['sha256'] and len(b)==x['bytes'] and hashlib.sha1((typ+' '+str(len(b))+'\0').encode()+b).hexdigest()==oid;return b
for oid,x in g['objects'].items():obj(oid,x['type'])
for p in g['proofs']:
 c=obj(p['revision'],'commit');first=c.split(b'\n',1)[0].decode();assert first.startswith('tree ');tree=first[5:];parts=p['path'].split('/');assert len(parts)==len(p['chain'])
 for name,s in zip(parts,p['chain']):
  assert tree==s['tree'] and name==s['name'];mode,oid=entries(obj(tree,'tree'))[name];assert mode==s['mode'] and oid==s['object'];tree=oid
 d=read(p['local']);d=gzip.decompress(d) if p['compression']=='gzip' else d;assert H(d)==p['sha256'] and hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==p['blob']==tree
for folder,mn in [('prior19','MANIFEST.json'),('prior42','BUNDLE-MANIFEST.json')]:
 mi=load(folder+'/'+mn);idx={x['path']:x for x in mi['files']};
 for f in (P/folder).rglob('*'):
  if f.is_file() and f.name!=mn:
   name=f.relative_to(P/folder).as_posix();v=idx[name];d=f.read_bytes();assert H(d)==v['sha256'] and len(d)==v['bytes']
assert H(read('prior19/MANIFEST.json'))==reg['prior19_manifest_sha256'] and H(read('prior42/BUNDLE-MANIFEST.json'))==reg['prior42_manifest_sha256']
for fn,line,id in [('contract-groups.jsonl.gz',628,'RIG-ADMISSION03-CONTRACT-628'),('findings.jsonl.gz',57,'RIG-ADMISSION03-F054'),('source-range-registration.jsonl.gz',101,'RIG-ADMISSION03-REGISTER-101'),('additional-contracts.jsonl.gz',87,'RIG-ADMISSION03-ADDITIONAL-101')]:
 q=json.loads(gzip.decompress(read('prior42/historical-774/'+fn)).splitlines()[line-1]);assert q['id']==id
old=json.loads(gzip.decompress(read('history/baseline-059-queue-committed-2e1f.json.gz')))['entries'];new=json.loads(gzip.decompress(read('history/baseline-059-queue-committed-d7.json.gz')))['entries'];s=load('SELECTED-059-QUEUE.json')
for q in s['rows']:i=q['index'];assert q['value']==old[i]==new[i] and q['identical_in_both_captures']
w=gzip.decompress(read('ledger/working.json.gz'));c=gzip.decompress(read('ledger/committed.json.gz'));assert H(w)==reg['working_baseline_sha256'] and H(c)==reg['committed_baseline_sha256'] and (w==c)==reg['working_vs_committed_equal'];wa=json.loads(w)['newly_assessed_ledger']['entries'];ca=json.loads(c)['newly_assessed_ledger']['entries']
for q in reg['selected_ledger']:
 a=wa[q['working_index']];assert a['id']==q['id'];found=[x for x in ca if x.get('id')==q['id']];assert bool(found)==(q['committed_index'] is not None) and (bool(found) and found[0]==a)==q['selected_equal']
print(json.dumps({'status':'PASS','files':len(fs),'source_proofs':len(g['proofs']),'root_transcript_lines':len(lines),'historical_774_records':4,'historical_059_rows':len(s['rows']),'selected_ledger_relations':len(reg['selected_ledger']),'qualification':'Integrity/identity and recorded scopes only; no claim about readers’ mental state, product behavior, reachability or runtime validation.'},indent=2))
