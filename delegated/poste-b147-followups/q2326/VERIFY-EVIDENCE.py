#!/usr/bin/env python3
"""Read-only verification of q2326 evidence, not product validation."""
import argparse,hashlib,json,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
def die(x):raise SystemExit(x)
def sha(b):return hashlib.sha256(b).hexdigest()
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def read(n):
 x=pathlib.PurePosixPath(n)
 if x.is_absolute() or '..' in x.parts:die('unsafe path')
 f=P.joinpath(*x.parts)
 if f.is_symlink() or not f.is_file():die('missing or symlink: '+n)
 return f.read_bytes()
def entries(raw):
 o={};i=0
 while i<len(raw):
  try:j=raw.index(b'\0',i);mode,name=raw[i:j].split(b' ',1);h=raw[j+1:j+21]
  except ValueError:die('bad tree')
  if len(h)!=20 or name in o:die('invalid tree entry')
  o[name]=(mode.lstrip(b'0'),h.hex());i=j+21
 return o
manifest=json.loads(read('MANIFEST.json'));expected={}
for row in manifest['files']:
 n=row['path']
 if n in expected or n=='MANIFEST.json':die('duplicate manifest path')
 b=read(n)
 if (len(b),sha(b))!=(row['bytes'],row['sha256']):die('manifest mismatch: '+n)
 expected[n]=row
actual=set()
for f in P.rglob('*'):
 if f.is_symlink():die('symlink in packet')
 if f.is_file():actual.add(f.relative_to(P).as_posix())
if actual!=set(expected)|{'MANIFEST.json'}:die('manifest member set mismatch')
proofs=json.loads(read('GIT-TREE-PROOFS.json'))['proofs']
for v in proofs:
 b=read(v['file']);parts=v['path'].split('/');chain=v['chain']
 if (len(b),sha(b))!=(v['bytes'],v['sha256']) or len(parts)!=len(chain) or not chain or chain[0]['tree']!=v['revision_tree']:die('source proof mismatch')
 for i,c in enumerate(chain):
  if c['name']!=parts[i]:die('chain path mismatch')
  raw=read('git-trees/'+c['tree'])
  if oid('tree',raw)!=c['tree'] or entries(raw).get(c['name'].encode())!=(c['mode'].encode().lstrip(b'0'),c['object']):die('tree membership mismatch')
  if i+1<len(chain):
   if c['kind']!='tree' or chain[i+1]['tree']!=c['object']:die('tree chain mismatch')
  elif c['kind']!='blob' or oid('blob',b)!=c['object']:die('source blob mismatch')
views=0
for f in sorted(P.glob('DISPLAY-*.txt.receipt.json')):
 r=json.loads(f.read_bytes());lines=read(r['source']).decode().splitlines();lo,hi=r['range'];v=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
 if v!=read(r['display']) or sha(v)!=r['view_sha256'] or r['truncated']:die('source display mismatch')
 reg=json.loads(read(r['registration'])); allowed=[]
 for w in reg.get('windows',[]):
  if w['file']==r['source']:allowed.extend(w['ranges'])
 for w in reg.get('items',[]):
  if w['file']==r['source']:allowed.extend(w['planned_ranges'])
 if reg.get('file')==r['source'] and 'corrected_range' in reg:allowed.append(reg['corrected_range'])
 if not any(lo>=a and hi<=b for a,b in allowed):die('display outside registered windows')
 if reg['registered_at_utc']>r['displayed_at_utc']:die('registration after source display')
 views+=1
rawq=read('QUEUE-2326-RAW.json');a=json.loads(read('QUEUE-2326-PROJECTION.json'));b=json.loads(read('QUEUE-2326-PUBLIC-PROJECTION.json'))
if json.loads(rawq)!=a or a!=b:die('selected queue projections differ')
q=json.loads(read('QUEUE-IDENTITY.json'))
if (len(rawq),sha(rawq))!=(q['raw_bytes'],q['raw_sha256']):die('queue raw identity mismatch')
h=json.loads(read('HISTORY-REGISTRATION.json'))
for row in h['items']:
 if row['found']:
  data=read(row['capture'])
  if (len(data),sha(data))!=(row['bytes'],row['actual_sha256']) or row['actual_sha256']!=row['expected_sha256']:die('historic receipt mismatch')
for i in range(5):
 if read(f'sources/e275-{i}.rs')!=read(f'sources/95e6-{i}.rs'):die('old/current per-file identity mismatch')
selector=json.loads(read('HISTORY-SELECTOR-REGISTRATION.json'))
for label in 'ABC':
 receipt=json.loads(read(f'HISTORY-RECEIPT-{label}.json')); body=read(receipt['display'])
 if sha(body)!=receipt['display_sha256'] or receipt['truncated']:die('historic display receipt mismatch')
 selected=json.loads(body)
 for name,part in selected.items():
  full=json.loads(read(name)); spec=selector['selectors'][name]
  if spec=='whole':
   if part!=full:die('historic whole projection mismatch')
  else:
   for k in selector['shared_metadata']:
    if k in full and part.get(k)!=full[k]:die('historic metadata projection mismatch')
   for ptr in spec:
    v=full
    for component in ptr.strip('/').split('/'):
     v=v[int(component)] if isinstance(v,list) else v[component]
    if part.get(ptr)!=v:die('historic row projection mismatch')
if len(json.loads(read('READ-LIMITS.json'))['personal_views'])!=views:die('view inventory mismatch')
parser=argparse.ArgumentParser();parser.add_argument('--repo');args=parser.parse_args()
if args.repo:
 for x in proofs:
  root=subprocess.check_output(['git','-C',args.repo,'rev-parse',x['revision']+'^{tree}']).decode().strip()
  if root!=x['revision_tree']:die('local commit tree mismatch')
  data=subprocess.check_output(['git','-C',args.repo,'show',x['revision']+':'+x['path']])
  if data!=read(x['file']):die('local source mismatch')
 raw=subprocess.check_output(['git','-C',args.repo,'show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json']);lo,hi=q['byte_range_half_open']
 if sha(raw)!=q['source_queue_sha256'] or raw[lo:hi]!=rawq:die('original queue slice mismatch')
 queue=json.loads(raw); qq=queue['entries'][2326]
 public_raw=subprocess.check_output(['git','-C',args.repo,'show',q['public_revision']+':baseline-059/queue.json'])
 if sha(public_raw)!=q['public_queue_sha256'] or json.loads(public_raw)['entries'][2326]!=qq or qq!=a:die('selected queue public projection mismatch')
 if queue['entries'][2327]!=json.loads(read('RELATION-Q2327-PROJECTION.json')):die('historical q2327 projection mismatch')
 ledger=subprocess.check_output(['git','-C',args.repo,'show','e709f11f0aa6bf736bd20819a628e4790cd03999:BASELINE-95E-20261110.json'])
 reg=json.loads(read('RELATION-SELECTION-REGISTRATION.json'))
 if sha(ledger)!=reg['selections'][1]['source_sha256'] or json.loads(ledger)['newly_assessed_ledger']['entries'][47]!=json.loads(read('RELATION-LEDGER-47-PROJECTION.json')):die('ledger projection mismatch')
 print('Local Git commit-to-tree, sources, original 059 slice and selected public/historical ledger relationships rechecked.')
else:print('No --repo supplied: commit-to-tree and original whole-queue slice binding not rechecked.')
print(f'Verified {len(expected)} packet files, {len(proofs)} source tree chains and {views} registered source display windows.')
print('Whole-tree cleanliness, global privacy, independent custody, product behavior and audit closure remain outside this verifier.')
