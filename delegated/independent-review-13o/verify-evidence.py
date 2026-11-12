#!/usr/bin/env python3
"""Read-only identity and scope verifier; never runs product code."""
import argparse,gzip,hashlib,json,pathlib,subprocess
R=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def obj(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(p):return json.loads((R/p).read_bytes())
def tree_entry(raw,name):
 i=0;found=[]
 while i<len(raw):
  j=raw.index(b'\0',i);mode,n=raw[i:j].split(b' ',1);v=raw[j+1:j+21];assert len(v)==20
  if n.decode()==name:found.append((mode.decode(),v.hex()))
  i=j+21
 assert len(found)==1
 return found[0]
def verify_receipt(base,regname,receiptname,displayname,coordinator=False):
 reg=json.loads((base/regname).read_bytes());rec=json.loads((base/receiptname).read_bytes());assert H((base/regname).read_bytes())==rec['registration_sha256'];assert H((base/displayname).read_bytes())==rec['display_sha256'];assert len(reg['entries'])==len(rec['rows']);out=[]
 for e,row in zip(reg['entries'],rec['rows'],strict=True):
  data=(base/e['capture']).read_bytes();assert len(data)==e['bytes'] and H(data)==e['sha256'] and obj('blob',data)==e['git_blob'];lines=data.decode().splitlines();assert len(lines)==e['eof']
  ranges=e['registered_ranges' if coordinator else 'planned_ranges'];assert row['displayed_ranges']==ranges;seen=set()
  for a,b in ranges:
   out.append(f"--- {e['tag']} {a}-{b} ---\n");out.extend(f'{i:4d} {lines[i-1]}\n' for i in range(a,b+1));seen.update(range(a,b+1))
  complement=row['unread_complement' if coordinator else 'unread_complement_per_registration'];unread={i for a,b in complement for i in range(a,b+1)};assert unread==set(range(1,e['eof']+1))-seen
 assert ''.join(out).encode()==(base/displayname).read_bytes()
a=argparse.ArgumentParser();a.add_argument('--publication',type=pathlib.Path,help='optional existing published Git evidence repository');args=a.parse_args()
m=load('MANIFEST.json');listed=set()
for e in m['files']:
 p=R/e['path'];assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];listed.add(e['path'])
actual={str(p.relative_to(R)) for p in R.rglob('*') if p.is_file()};assert actual==listed|{'MANIFEST.json'}
c=R/'coordinator-packet';cm=json.loads((c/'MANIFEST.json').read_bytes());clisted=set()
for e in cm['files']:
 b=(c/e['path']).read_bytes();assert len(b)==e['bytes'] and H(b)==e['sha256'];clisted.add(e['path'])
assert {str(x.relative_to(c)) for x in c.rglob('*') if x.is_file()}==clisted|{'MANIFEST.json'}
for x in [('REGISTRATION.json','READ-RECEIPT.json','DISPLAY.txt'),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json','EXT-DISPLAY.txt')]:verify_receipt(c,*x,coordinator=True)
for x in [('REGISTRATION.json','READ-RECEIPT.json','DISPLAY.txt'),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json','EXT-DISPLAY.txt'),('READBACK-REGISTRATION.json','READBACK-RECEIPT.json','READBACK-DISPLAY.txt')]:verify_receipt(R,*x)
proof=load('GIT-TREE-PROOFS.json')
for e in proof['proofs']:
 cur=e['root_tree'];names=e['path'].split('/');assert len(names)==len(e['chain'])
 for name,hop in zip(names,e['chain'],strict=True):
  assert (hop['tree'],hop['name'])==(cur,name);meta=proof['objects'][cur];raw=gzip.decompress((R/meta['file']).read_bytes());assert len(raw)==meta['bytes'] and H(raw)==meta['sha256'] and obj('tree',raw)==cur;mode,cur=tree_entry(raw,name);assert (mode,cur)==(hop['mode'],hop['object'])
 data=(R/e['capture']).read_bytes();data=gzip.decompress(data) if e['compression']=='gzip' else data;assert cur==e['git_blob']==obj('blob',data)
for archive,rows in [('findings.jsonl.gz',[89]),('source-range-registration.jsonl.gz',[104,105]),('additional-contracts.jsonl.gz',[90,91])]:
 lines=gzip.decompress((R/'history'/archive).read_bytes()).splitlines(keepends=True)
 for n in rows:assert lines[n-1]==(R/'history'/f'{archive[:-9]}-{n}.jsonl').read_bytes()
for reg,rec in [('HISTORY-REGISTRATION.json','HISTORY-RECEIPT.json'),('LEDGER-REGISTRATION.json','LEDGER-RECEIPT.json'),('ORIGINAL-QUEUE-REGISTRATION.json','ORIGINAL-QUEUE-RECEIPT.json')]:assert H((R/reg).read_bytes())==load(rec)['registration_sha256']
assert H((R/'history/ledger-selection-reconstructed.json').read_bytes())==load('LEDGER-RECEIPT.json')['selection_sha256']
final=load('COORDINATOR-ASSESSMENT.json');draft=json.loads((c/'ASSESSMENT-DRAFT.json').read_bytes());draft['central_display_evidence']=final['central_display_evidence'];draft['review_qualification']=final['review_qualification'];assert draft==final
prior=load('PRIOR-IDENTITIES.json')
for e in prior['prior_reports']:assert H((R/e['copy']).read_bytes())==e['sha256']
if args.publication:
 P=args.publication;git=lambda *x:subprocess.check_output(['git','-C',str(P),*x]);seen={}
 for e in proof['proofs']:
  if e['revision'] not in seen:seen[e['revision']]=git('rev-parse',e['revision']+'^{tree}').decode().strip()
  assert seen[e['revision']]==e['root_tree'];b=git('show',e['revision']+':'+e['path']);data=(R/e['capture']).read_bytes();data=gzip.decompress(data) if e['compression']=='gzip' else data;assert b==data
 history=load('HISTORY-REGISTRATION.json');sel=load('history/queue-selected-reconstructed.json')['entries']
 for e in history['entries']:
  b=(P/e['published_path']).read_bytes();assert H(b)==e['stored_sha256'];raw=gzip.decompress(b);assert H(raw)==e['decoded_sha256']
  if 'planned_json_pointers' in e:
   q=json.loads(raw)
   for row in sel:assert q['entries'][int(row['pointer'].split('/')[-1])]==row['value']
 qreg=load('ORIGINAL-QUEUE-REGISTRATION.json');raw=git('show',qreg['revision']+':'+qreg['git_path']);assert H(raw)==qreg['sha256'] and obj('blob',raw)==qreg['git_blob'];q=json.loads(raw)
 for row in sel:assert q['entries'][int(row['pointer'].split('/')[-1])]==row['value']
 lreg=load('LEDGER-REGISTRATION.json');raw=git('show',lreg['head_commit']+':'+lreg['path']);assert H(raw)==lreg['committed_sha256'];ledger=json.loads(raw)
 for row in load('history/ledger-selection-reconstructed.json')['entries']['committed']:
  v=ledger
  for seg in row['pointer'].split('/')[1:]:v=v[int(seg)] if isinstance(v,list) else v[seg]
  assert v==row['value']
 for e in prior['prior_reports']:assert H((P/e['published_path']).read_bytes())==e['sha256']
 print('Optional Git commit-to-tree, original selected queue values, prior transports and committed ledger checks passed.')
else:print('Offline tree-to-root checks passed; commit-to-root and full historical queue comparisons require --publication.')
print('Verified manifest, coordinator packet, read scopes, historical streams, tree chains and draft/final qualification. No product execution or semantic closure implied.')
