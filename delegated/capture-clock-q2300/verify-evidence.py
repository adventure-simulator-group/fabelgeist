#!/usr/bin/env python3
"""Read-only audit evidence verification; no product, test, macro or fixture execution."""
import argparse,gzip,hashlib,io,json,pathlib,subprocess,tarfile,tomllib
R=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def load(p):return json.loads((R/p).read_bytes())
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entry(raw,name):
 i=0;out=[]
 while i<len(raw):
  j=raw.index(b'\0',i);mode,n=raw[i:j].split(b' ',1);v=raw[j+1:j+21];assert len(v)==20
  if n.decode()==name:out.append((mode.decode(),v.hex()))
  i=j+21
 assert len(out)==1
 return out[0]
def scope(regname,receiptname,displayname):
 reg=load(regname);rec=load(receiptname);assert H((R/regname).read_bytes())==rec['registration_sha256'] and H((R/displayname).read_bytes())==rec['display_sha256'];assert len(reg['entries'])==len(rec['rows']);out=[]
 for e,row in zip(reg['entries'],rec['rows'],strict=True):
  b=(R/e['capture']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];ls=b.decode().splitlines();assert len(ls)==e['eof'];assert row['displayed_ranges']==e['planned_ranges'];seen=set()
  for a,z in e['planned_ranges']:
   out.append(f"--- {e['tag']} {a}-{z} ---\n");out.extend(f'{i:4d} {ls[i-1]}\n' for i in range(a,z+1));seen.update(range(a,z+1))
  unread={i for a,z in row['unread_complement_per_registration'] for i in range(a,z+1)};assert unread==set(range(1,len(ls)+1))-seen
 assert ''.join(out).encode()==(R/displayname).read_bytes()
a=argparse.ArgumentParser();a.add_argument('--publication',type=pathlib.Path);a.add_argument('--private-capture',type=pathlib.Path);args=a.parse_args()
m=load('MANIFEST.json');listed=set()
for e in m['files']:
 f=R/e['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];listed.add(e['path'])
assert {str(p.relative_to(R)) for p in R.rglob('*') if p.is_file()}==listed|{'MANIFEST.json'}
for stem in ['','EXT-','ELAPSED-','ELAPSED-EXT-','CORE-ELAPSED-','CONSUMER-']:scope(stem+'REGISTRATION.json',stem+'READ-RECEIPT.json',stem+'DISPLAY.txt')
p=load('GIT-TREE-PROOFS.json')
for e in p['proofs']:
 cur=e['root_tree'];names=e['path'].split('/');assert len(names)==len(e['chain'])
 for n,hop in zip(names,e['chain'],strict=True):
  assert (hop['tree'],hop['name'])==(cur,n);meta=p['objects'][cur];raw=gzip.decompress((R/meta['file']).read_bytes());assert len(raw)==meta['bytes'] and H(raw)==meta['sha256'] and oid('tree',raw)==cur;mode,cur=entry(raw,n);assert (mode,cur)==(hop['mode'],hop['object'])
 b=(R/e['capture']).read_bytes();b=gzip.decompress(b) if e['compression']=='gzip' else b;assert cur==e['git_blob']==oid('blob',b)
n=load('NATIVE-NAVIGATION.json');archive=(R/n['archive']).read_bytes();assert H(archive)==n['archive_sha256']==n['locked_checksum'];lock=gzip.decompress((R/n['lock_capture']).read_bytes());assert H(lock)==n['lock_sha256'] and oid('blob',lock)==n['lock_git_blob'];pkgs=[x for x in tomllib.loads(lock.decode())['package'] if x['name']==n['package'] and x['version']==n['version']];assert len(pkgs)==1 and pkgs[0]['checksum']==H(archive)
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:member=t.extractfile(n['member']).read()
assert member==(R/n['member_capture']).read_bytes() and H(member)==n['member_sha256']
hist=load('HISTORY-REGISTRATION.json');originals={}
for e in hist['entries']:
 b=(R/'history-exact'/e['historical_relative_path']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['original_sha256'];originals[e['historical_relative_path']]=json.loads(b)
q=load('QUEUE-2300-RECONSTRUCTED.json')['value'];rows=q['exact_source_identities_and_assessed_ranges'];assert len(rows)==2
for row in rows:
 for k in ['body_receipt','registration_ref_and_sha256']:
  item=row[k];path=item['path'].removeprefix('target/726-audit-adler-20261027/');assert H((R/'history-exact'/path).read_bytes())==item['sha256']
 ad=row['root_adoption'];assert H((R/'history-exact'/ad['path']).read_bytes())==ad['sha256'];assert H((R/'history-exact'/ad['verification']).read_bytes())==ad['verification_sha256']
proposal=q['proposal'];assert H((R/'history-exact'/proposal['path']).read_bytes())==proposal['sha256']
selection=load('HISTORY-SELECTED-RECONSTRUCTED.json')
for rec in selection['records']:
 root=originals[rec['record']]
 for item in rec['selections']:
  v=root
  if item['pointer']!='/':
   for seg in item['pointer'].split('/')[1:]:v=v[int(seg)] if isinstance(v,list) else v.get(seg)
  assert v==item['value']
for reg,receipt in [('QUEUE-REGISTRATION.json','QUEUE-READ-RECEIPT.json'),('HISTORY-SELECTION-REGISTRATION.json','HISTORY-READ-RECEIPT.json'),('LEDGER-REGISTRATION.json','LEDGER-READ-RECEIPT.json')]:
 rec=load(receipt);assert H((R/reg).read_bytes())==rec['registration_sha256']
 for s in rec['selection_files']:assert H((R/s['path']).read_bytes())==s['sha256']
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
  assert roots[e['revision']]==e['root_tree'];b=(R/e['capture']).read_bytes();b=gzip.decompress(b) if e['compression']=='gzip' else b;assert git('show',e['revision']+':'+e['path'])==b
 qr=load('QUEUE-REGISTRATION.json');b=git('show',qr['revision']+':'+qr['git_path']);assert H(b)==qr['sha256'] and oid('blob',b)==qr['git_blob'];assert json.loads(b)['entries'][2300]==q
 lr=load('LEDGER-REGISTRATION.json');b=git('show',lr['head_commit']+':'+lr['path']);assert H(b)==lr['committed_sha256'];v=json.loads(b)
 for row in load('LEDGER-SELECTION-RECONSTRUCTED.json')['entries']['committed']:
  w=v
  for seg in row['pointer'].split('/')[1:]:w=w[int(seg)] if isinstance(w,list) else w[seg]
  assert w==row['value']
 for e in prior:assert H((P/e['published_path']).read_bytes())==e['sha256']
 print('Optional Git commit binding, exact historical queue selection, committed ledger and prior reports verified.')
else:print('Commit-to-root binding and original queue comparison require --publication.')
print('Verified manifest, registered windows, original historical records, package checksum and source identities; no execution or closure implied.')
