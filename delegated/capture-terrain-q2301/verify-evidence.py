#!/usr/bin/env python3
"""Read-only evidence verifier; no product, fixture, test or shader execution."""
import argparse,gzip,hashlib,json,pathlib,subprocess
R=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def load(p):return json.loads((R/p).read_bytes())
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_entry(raw,name):
 i=0;found=[]
 while i<len(raw):
  j=raw.index(b'\0',i);mode,n=raw[i:j].split(b' ',1);v=raw[j+1:j+21];assert len(v)==20
  if n.decode()==name:found.append((mode.decode(),v.hex()))
  i=j+21
 assert len(found)==1
 return found[0]
def scope(stem):
 rn=stem+'REGISTRATION.json';dn=stem+'DISPLAY.txt';rec=load(stem+'READ-RECEIPT.json');reg=load(rn);assert H((R/rn).read_bytes())==rec['registration_sha256'] and H((R/dn).read_bytes())==rec['display_sha256'];out=[];assert len(reg['entries'])==len(rec['rows'])
 for e,row in zip(reg['entries'],rec['rows'],strict=True):
  b=(R/e['capture']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'] and oid('blob',b)==e['git_blob'];ls=b.decode().splitlines();assert len(ls)==e['eof'] and row['displayed_ranges']==e['planned_ranges'];seen=set()
  for a,z in e['planned_ranges']:
   out.append(f"--- {e['tag']} {a}-{z} ---\n");out.extend(f'{i:4d} {ls[i-1]}\n' for i in range(a,z+1));seen.update(range(a,z+1))
  unread={i for a,z in row['unread_complement_per_registration'] for i in range(a,z+1)};assert unread==set(range(1,len(ls)+1))-seen
 assert ''.join(out).encode()==(R/dn).read_bytes()
a=argparse.ArgumentParser();a.add_argument('--publication',type=pathlib.Path);a.add_argument('--private-capture',type=pathlib.Path);args=a.parse_args()
m=load('MANIFEST.json');listed=set()
for e in m['files']:
 f=R/e['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];listed.add(e['path'])
assert {str(p.relative_to(R)) for p in R.rglob('*') if p.is_file()}==listed|{'MANIFEST.json'}
for stem in ['','SUPPORT-','QUERY-EXT-','DRIVER-EXT-']:scope(stem)
u=load('UNREGISTERED-QUERY-QUALIFICATION.json');qr=load('QUERY-EXT-REGISTRATION.json');assert 'before' in u['first_display_time'] and 'unregistered' in qr['qualification'];e=qr['entries'][0];ls=(R/e['capture']).read_text().splitlines();b=''.join(f'{i:6d}\t{ls[i-1]}\n' for i in range(101,171)).encode();assert b==(R/u['first_display_reconstruction']).read_bytes() and H(b)==u['first_display_reconstruction_sha256']
p=load('GIT-TREE-PROOFS.json')
for e in p['proofs']:
 cur=e['root_tree'];names=e['path'].split('/');assert len(names)==len(e['chain'])
 for name,hop in zip(names,e['chain'],strict=True):
  assert (hop['tree'],hop['name'])==(cur,name);meta=p['objects'][cur];raw=gzip.decompress((R/meta['file']).read_bytes());assert len(raw)==meta['bytes'] and H(raw)==meta['sha256'] and oid('tree',raw)==cur;mode,cur=tree_entry(raw,name);assert (mode,cur)==(hop['mode'],hop['object'])
 b=(R/e['capture']).read_bytes();assert cur==e['git_blob']==oid('blob',b)
hist=load('HISTORY-REGISTRATION.json');originals={}
for e in hist['entries']:
 b=(R/'history-exact'/e['historical_relative_path']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];originals[e['historical_relative_path']]=json.loads(b)
q=load('QUEUE-2301-RECONSTRUCTED.json')['value'];rows=q['exact_source_identities_and_assessed_ranges'];assert len(rows)==3
for row in rows:
 for k in ['body_receipt','registration_ref_and_sha256']:
  x=row[k];path=x['path'].removeprefix('target/726-audit-adler-20261027/');assert H((R/'history-exact'/path).read_bytes())==x['sha256']
 x=row['root_adoption'];assert H((R/'history-exact'/x['path']).read_bytes())==x['sha256'] and H((R/'history-exact'/x['verification']).read_bytes())==x['verification_sha256']
x=q['proposal'];assert H((R/'history-exact'/x['path']).read_bytes())==x['sha256']
for rec in load('HISTORY-SELECTED-RECONSTRUCTED.json')['records']:
 for s in rec['selections']:
  v=originals[rec['record']]
  if s['pointer']!='/':
   for k in s['pointer'].split('/')[1:]:v=v[int(k)] if isinstance(v,list) else v.get(k)
  assert v==s['value']
for reg,receipt in [('QUEUE-REGISTRATION.json','QUEUE-READ-RECEIPT.json'),('HISTORY-SELECTION-REGISTRATION.json','HISTORY-READ-RECEIPT.json'),('LEDGER-EXT-REGISTRATION.json','LEDGER-READ-RECEIPT.json')]:
 rec=load(receipt);assert H((R/reg).read_bytes())==rec['registration_sha256']
 for s in rec['selection_files']:assert H((R/s['path']).read_bytes())==s['sha256']
assert 'aborted' in load('LEDGER-FAILED-OBSERVATION.json')['qualification']
prior=load('PRIOR-IDENTITIES.json')['sources']
for e in prior:assert H((R/e['capture']).read_bytes())==e['sha256']
if args.private_capture:
 cp=load('PRIVATE-CAPTURE-MANIFEST.json');assert (args.private_capture/'CAPTURE-MANIFEST.json').read_bytes()==(R/'PRIVATE-CAPTURE-MANIFEST.json').read_bytes()
 for e in cp['files']:
  b=(args.private_capture/e['historical_relative_path']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256']
if args.publication:
 P=args.publication;git=lambda *x:subprocess.check_output(['git','-C',str(P),*x]);roots={}
 for e in p['proofs']:
  if e['revision'] not in roots:roots[e['revision']]=git('rev-parse',e['revision']+'^{tree}').decode().strip()
  assert roots[e['revision']]==e['root_tree'] and git('show',e['revision']+':'+e['path'])==(R/e['capture']).read_bytes()
 qr=load('QUEUE-REGISTRATION.json');b=git('show',qr['revision']+':'+qr['git_path']);assert H(b)==qr['sha256'] and oid('blob',b)==qr['git_blob'] and json.loads(b)['entries'][2301]==q
 lr=load('LEDGER-EXT-REGISTRATION.json');b=git('show',lr['head_commit']+':'+lr['path']);assert H(b)==lr['committed_sha256'];v=json.loads(b)
 for row in load('LEDGER-SELECTION-RECONSTRUCTED.json')['entries']['committed']:
  w=v
  for seg in row['pointer'].split('/')[1:]:w=w[int(seg)] if isinstance(w,list) else w[seg]
  assert w==row['value']
 for e in prior:assert H((P/e['published_path']).read_bytes())==e['sha256']
 print('Optional Git commit binding, exact 059 queue value, committed ledger and prior reports verified.')
else:print('Commit-to-root binding and original queue comparison require --publication.')
print('Verified manifests, registered displays and chronology, historical/adoption identities and source trees; no runtime or closure implied.')
