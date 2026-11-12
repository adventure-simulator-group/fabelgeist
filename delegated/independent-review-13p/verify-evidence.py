#!/usr/bin/env python3
"""Read-only source/evidence verifier; does not execute product code."""
import argparse,gzip,hashlib,json,pathlib,subprocess
R=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def read(p):return json.loads((R/p).read_bytes())
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_entry(raw,name):
 i=0;found=[]
 while i<len(raw):
  j=raw.index(b'\0',i);mode,n=raw[i:j].split(b' ',1);v=raw[j+1:j+21];assert len(v)==20
  if n.decode()==name:found.append((mode.decode(),v.hex()))
  i=j+21
 assert len(found)==1
 return found[0]
def scopes(base,regfile,recfile,displays,central=False):
 reg=json.loads((base/regfile).read_bytes());rec=json.loads((base/recfile).read_bytes());assert H((base/regfile).read_bytes())==rec['registration_sha256']
 if central:
  assert [x['path'] for x in rec['displays']]==displays
  for row in rec['displays']:assert H((base/row['path']).read_bytes())==row['sha256']
 else:assert H((base/displays[0]).read_bytes())==rec['display_sha256']
 out=[];assert len(reg['entries'])==len(rec['rows'])
 for e,row in zip(reg['entries'],rec['rows'],strict=True):
  data=(base/e['capture']).read_bytes();assert len(data)==e['bytes'] and H(data)==e['sha256'] and oid('blob',data)==e['git_blob'];ls=data.decode().splitlines();assert len(ls)==e['eof']
  ranges=e['registered_ranges' if central else 'planned_ranges'];assert row['displayed_ranges']==ranges;seen=set()
  for a,b in ranges:
   out.append(f"--- {e['tag']} {a}-{b} ---\n");out.extend(f'{i:4d} {ls[i-1]}\n' for i in range(a,b+1));seen.update(range(a,b+1))
  unread={i for a,b in row['unread_complement'] for i in range(a,b+1)};assert unread==set(range(1,len(ls)+1))-seen
 assert ''.join(out).encode()==b''.join((base/x).read_bytes() for x in displays)
a=argparse.ArgumentParser();a.add_argument('--publication',type=pathlib.Path);a.add_argument('--capture-file',type=pathlib.Path);a.add_argument('--stage-manifest-file',type=pathlib.Path);args=a.parse_args()
m=read('MANIFEST.json');listed=set()
for e in m['files']:
 f=R/e['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];listed.add(e['path'])
assert {str(x.relative_to(R)) for x in R.rglob('*') if x.is_file()}==listed|{'MANIFEST.json'}
c=R/'coordinator';cm=json.loads((c/'MANIFEST.json').read_bytes());cl=set()
for e in cm['files']:
 b=(c/e['path']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];cl.add(e['path'])
assert {str(x.relative_to(c)) for x in c.rglob('*') if x.is_file()}==cl|{'MANIFEST.json'}
for reg,rec,ds in [('REGISTRATION.json','READ-RECEIPT.json',['DISPLAY.txt']),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json',['EXT-DISPLAY.txt']),('EXT2-REGISTRATION.json','EXT2-READ-RECEIPT.json',['EXT2-REDISPLAY-A.txt','EXT2-REDISPLAY-B.txt'])]:scopes(c,reg,rec,ds,True)
assert (c/'EXT2-DISPLAY.txt').read_bytes()==(c/'EXT2-REDISPLAY-A.txt').read_bytes()+(c/'EXT2-REDISPLAY-B.txt').read_bytes()
assert 'truncated' in json.loads((c/'EXT2-READ-RECEIPT.json').read_bytes())['qualification']
cp=json.loads((c/'TREE-PROOFS.json').read_bytes());assert cp['revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
for e in cp['proofs']:
 cur=cp['root_tree'];names=e['path'].split('/');assert len(names)==len(e['chain'])
 for name,hop in zip(names,e['chain'],strict=True):
  assert (hop['tree'],hop['name'])==(cur,name);meta=cp['objects'][cur];raw=gzip.decompress((c/meta['file']).read_bytes());assert len(raw)==meta['decoded_bytes'] and H(raw)==meta['decoded_sha256'] and oid('tree',raw)==cur;mode,cur=tree_entry(raw,name);assert (mode,cur)==(hop['mode'],hop['object'])
 assert cur==e['blob']==oid('blob',(c/'sources'/f"{e['tag']}.rs").read_bytes())
for reg,rec,ds in [('REGISTRATION.json','READ-RECEIPT.json',['DISPLAY.txt']),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json',['EXT-DISPLAY.txt'])]:scopes(R,reg,rec,ds)
proof=read('GIT-TREE-PROOFS.json')
assert all(e['root_tree']==cp['root_tree'] for e in proof['proofs'] if e['revision']==cp['revision'])
for e in proof['proofs']:
 cur=e['root_tree'];names=e['path'].split('/');assert len(names)==len(e['chain'])
 for name,hop in zip(names,e['chain'],strict=True):
  assert (hop['tree'],hop['name'])==(cur,name);meta=proof['objects'][cur];raw=gzip.decompress((R/meta['file']).read_bytes());assert len(raw)==meta['bytes'] and H(raw)==meta['sha256'] and oid('tree',raw)==cur;mode,cur=tree_entry(raw,name);assert (mode,cur)==(hop['mode'],hop['object'])
 b=(R/e['capture']).read_bytes();b=gzip.decompress(b) if e['compression']=='gzip' else b;assert cur==e['git_blob']==oid('blob',b)
for regfile,recfile in [('HISTORY-REGISTRATION.json','HISTORY-RECEIPT.json'),('HISTORY-EXT-REGISTRATION.json','HISTORY-EXT-RECEIPT.json'),('LEDGER-REGISTRATION.json','LEDGER-RECEIPT.json')]:
 rec=read(recfile);assert H((R/regfile).read_bytes())==rec['registration_sha256']
 for e in rec['selected_files']:assert H((R/e['path']).read_bytes())==e['sha256']
for arc,items in [('additional-contracts.jsonl.gz',[101,102]),('findings.jsonl.gz',[78,91,101])]:
 lines=gzip.decompress((R/'history'/arc).read_bytes()).splitlines(keepends=True);pre='additional' if arc.startswith('additional') else 'findings'
 for i in items:assert lines[i-1]==(R/'history'/f'{pre}-{i}.jsonl').read_bytes()
prior=read('PRIOR-IDENTITIES.json')['checkpoint19_report'];assert H((R/prior['copy']).read_bytes())==prior['sha256']
corr=read('CAPTURE-IDENTITY-CORRECTION.json')
if args.capture_file:assert H(args.capture_file.read_bytes())==corr['verified_capture_whole_file_sha256']
if args.stage_manifest_file:assert H(args.stage_manifest_file.read_bytes())==corr['verified_stage_manifest_whole_file_sha256']
if args.publication:
 P=args.publication;git=lambda *x:subprocess.check_output(['git','-C',str(P),*x]);seen={}
 for e in proof['proofs']:
  if e['revision'] not in seen:seen[e['revision']]=git('rev-parse',e['revision']+'^{tree}').decode().strip()
  assert seen[e['revision']]==e['root_tree'];b=(R/e['capture']).read_bytes();b=gzip.decompress(b) if e['compression']=='gzip' else b;assert git('show',e['revision']+':'+e['path'])==b
 historical=read('HISTORY-REGISTRATION.json')['sources'][0];b=git('show',historical['revision']+':'+historical['path']);assert H(b)==historical['sha256'] and oid('blob',b)==historical['git_blob'];q=json.loads(b)
 for row in read('history/queue-selection-reconstructed.json')['entries']:assert q['entries'][int(row['pointer'].rsplit('/',1)[-1])]==row['value']
 ledgerreg=read('LEDGER-REGISTRATION.json');b=git('show',ledgerreg['head_commit']+':'+ledgerreg['path']);assert H(b)==ledgerreg['committed_sha256'];ledger=json.loads(b)
 for row in read('history/ledger-selection-reconstructed.json')['entries']:
  v=ledger
  for seg in row['pointer'].split('/')[1:]:v=v[int(seg)] if isinstance(v,list) else v[seg]
  assert v==row['value']
 assert H((P/prior['published_path']).read_bytes())==prior['sha256']
 print('Optional Git commit-to-root, file, original 059 selections, prior report and committed ledger checks passed.')
else:print('Offline tree-to-root checks passed; commit binding and original queue comparison require --publication.')
print('Verified manifests, registered source windows and redisplays, historical selections, and identities; no runtime or semantic closure implied.')
