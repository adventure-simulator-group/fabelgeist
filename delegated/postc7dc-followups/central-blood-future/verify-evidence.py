#!/usr/bin/env python3
"""Verify bounded source displays and historical selections without execution."""
import argparse, gzip, hashlib, json, pathlib, subprocess
from pathlib import PurePosixPath
R=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def load(name):return json.loads((R/name).read_bytes())
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def at(v,pointer):
 for part in pointer.split('/')[1:]:v=v[int(part)] if isinstance(v,list) else v[part]
 return v
def tree_entry(raw,name):
 i=0;matches=[]
 while i<len(raw):
  end=raw.index(b'\0',i);mode,n=raw[i:end].split(b' ',1);value=raw[end+1:end+21];assert len(value)==20
  if n.decode()==name:matches.append((mode.decode(),value.hex()))
  i=end+21
 assert i==len(raw) and len(matches)==1
 return matches[0]
def proof(e,root,objects):
 cur=root
 for part,hop in zip(e['path'].split('/'),e['chain'],strict=True):
  assert hop['tree']==cur and hop['name']==part
  meta=objects[cur];raw=gzip.decompress((R/meta['capture']).read_bytes());assert (len(raw),H(raw),oid('tree',raw))==(meta['bytes'],meta['sha256'],cur)
  mode,cur=tree_entry(raw,part);assert int(mode,8)==int(hop['mode'],8) and cur==hop['object']
 b=(R/e['capture']).read_bytes();assert (len(b),len(b.splitlines()),H(b),oid('blob',b))==(e['bytes'],e['eof'],e['sha256'],e['git_blob']) and cur==e['git_blob']
a=argparse.ArgumentParser();a.add_argument('--git-repo',type=pathlib.Path);a.add_argument('--baseline-snapshot',type=pathlib.Path);args=a.parse_args()
m=load('MANIFEST.json');seen=set()
for row in m['files']:
 q=PurePosixPath(row['path']);assert not q.is_absolute() and '..' not in q.parts and q.as_posix() not in seen;seen.add(q.as_posix());f=R/q;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
assert {x.relative_to(R).as_posix() for x in R.rglob('*') if x.is_file()}==seen|{'MANIFEST.json'}
nav=load('LINE-NAVIGATION-REGISTRATION.json')
for e in nav['entries']:proof(e,nav['root_tree'],nav['tree_objects'])
cal=load('CALENDAR-REGISTRATION.json');proof(cal,cal['root_tree'],cal['tree_objects'])
for regname,groups,prefix in [('SOURCE-REGISTRATION.json','ABC',''),('EXT-REGISTRATION.json','ABC','EXT-')]:
 reg=(R/regname).read_bytes();r=json.loads(reg);lookup={e['tag']:e for e in r['entries']};seen_ranges={tag:[] for tag in lookup}
 for group in groups:
  rec=load(f'{prefix}RECEIPT-{group}.json');assert rec['registration_sha256']==H(reg);out=[]
  for row in rec['rows']:
   e=lookup[row['tag']];lo,hi=row['range'];assert [lo,hi] in e['planned_ranges'];seen_ranges[row['tag']].append([lo,hi]);lines=(R/e['capture']).read_text().splitlines();out.append(f"--- {row['tag']} {lo}-{hi} ---\n");out.extend(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1))
  d=''.join(out).encode();assert H(d)==rec['display_sha256'] and d==(R/f'{prefix}DISPLAY-{group}.txt').read_bytes()
 assert all(seen_ranges[tag]==e['planned_ranges'] for tag,e in lookup.items())
reg=(R/'DISEASE-REGISTRATION.json').read_bytes();r=json.loads(reg);e=r['entry'];all_ranges=[]
for group in 'AB':
 rec=load(f'DISEASE-RECEIPT-{group}.json');assert rec['registration_sha256']==H(reg);out=[];lines=(R/e['capture']).read_text().splitlines()
 for lo,hi in rec['ranges']:
  assert [lo,hi] in e['planned_ranges'];all_ranges.append([lo,hi]);out.append(f'--- native-disease {lo}-{hi} ---\n');out.extend(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1))
 d=''.join(out).encode();assert H(d)==rec['display_sha256'] and d==(R/f'DISEASE-DISPLAY-{group}.txt').read_bytes()
assert all_ranges==e['planned_ranges']
reg=(R/'CALENDAR-REGISTRATION.json').read_bytes();r=json.loads(reg);lines=(R/r['capture']).read_text().splitlines();d=''.join(f'{i:4d} {lines[i-1]}\n' for lo,hi in r['planned_ranges'] for i in range(lo,hi+1)).encode();rec=load('CALENDAR-RECEIPT.json');assert rec['registration_sha256']==H(reg) and rec['display_sha256']==H(d) and d==(R/'CALENDAR-DISPLAY.txt').read_bytes()
for reg,result in [('NAVIGATION-REGISTRATION.json','NAVIGATION-RESULTS.json'),('LINE-NAVIGATION-REGISTRATION.json','LINE-NAVIGATION-RESULTS.json'),('PRODUCER-NAVIGATION-REGISTRATION.json','PRODUCER-NAVIGATION-RESULTS.json'),('WRITER-NAVIGATION-REGISTRATION.json','WRITER-NAVIGATION-RESULTS.json'),('LEDGER-NAVIGATION-REGISTRATION.json','LEDGER-NAVIGATION-RESULTS.json')]:assert load(result)['registration_sha256']==H((R/reg).read_bytes())
rec=load('RELATED-RECEIPT.json');assert rec['registration_sha256']==H((R/'RELATED-REGISTRATION.json').read_bytes()) and rec['display_sha256']==H((R/'RELATED-SELECTIONS-RECONSTRUCTED.json').read_bytes())
assert load('PRODUCER-SCOPE-ADDENDUM.json')['draft_sha256']==H((R/'ASSESSMENT-DRAFT.json').read_bytes())
if args.git_repo:
 root=args.git_repo.resolve();git=lambda *x:subprocess.check_output(['git','-C',str(root),*x]);assert git('rev-parse',nav['revision']+'^{tree}').decode().strip()==nav['root_tree']
 for e in [*nav['entries'],cal]:assert git('show',nav['revision']+':'+e['path'])==(R/e['capture']).read_bytes()
 q=git('show','26e360920b7b2ec32d5c76238623eaf6d6617d7e:baseline-059/queue.json');expected=next(x['sha256'] for x in load('RELATED-REGISTRATION.json')['sources'] if x['path']=='baseline-059/queue.json');assert H(q)==expected
 for item in load('RELATED-SELECTIONS-RECONSTRUCTED.json')['selections']:
  if item['source']=='baseline-059/queue.json':assert at(json.loads(q),item['pointer'])==item['value']
 print('Optional pinned Git tree/source and historical queue selection comparisons passed.')
else:print('Git commit-to-root and external queue comparisons require --git-repo.')
if args.baseline_snapshot:
 b=args.baseline_snapshot.read_bytes();expected=next(x['sha256'] for x in load('RELATED-REGISTRATION.json')['sources'] if x['path']=='BASELINE-95E-20261110.json');assert H(b)==expected
 for item in load('RELATED-SELECTIONS-RECONSTRUCTED.json')['selections']:
  if item['source']=='BASELINE-95E-20261110.json':assert at(json.loads(b),item['pointer'])==item['value']
 print('Optional time-bound working ledger selection comparison passed.')
else:print('Time-bound ledger comparison requires --baseline-snapshot.')
print('Verified source object proofs, registered displays and selected evidence; semantic conclusions and product behavior remain separately qualified.')
