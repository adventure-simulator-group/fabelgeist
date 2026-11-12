#!/usr/bin/env python3
"""Read-only verification of q2330 source evidence; not product validation."""
import argparse,hashlib,io,json,pathlib,subprocess,tarfile,tomllib
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
def tree_entries(raw):
 o={};i=0
 while i<len(raw):
  try:j=raw.index(b'\0',i);mode,name=raw[i:j].split(b' ',1);h=raw[j+1:j+21]
  except ValueError:die('bad tree')
  if len(h)!=20 or name in o:die('invalid tree entry')
  o[name]=(mode.lstrip(b'0'),h.hex());i=j+21
 return o
def ptr(value,pointer):
 for c in pointer.strip('/').split('/'):value=value[int(c)] if isinstance(value,list) else value[c]
 return value
manifest=json.loads(read('MANIFEST.json'));expected={}
for row in manifest['files']:
 n=row['path']
 if n in expected or n=='MANIFEST.json':die('duplicate or forbidden manifest path')
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
  if oid('tree',raw)!=c['tree'] or tree_entries(raw).get(c['name'].encode())!=(c['mode'].encode().lstrip(b'0'),c['object']):die('tree membership mismatch')
  if i+1<len(chain):
   if c['kind']!='tree' or chain[i+1]['tree']!=c['object']:die('tree chain mismatch')
  elif c['kind']!='blob' or oid('blob',b)!=c['object']:die('source blob mismatch')
views=0
for f in sorted(P.glob('DISPLAY-*.txt.receipt.json')):
 r=json.loads(f.read_bytes());lines=read(r['source']).decode().splitlines();lo,hi=r['range'];v=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
 if v!=read(r['display']) or sha(v)!=r['view_sha256'] or r['truncated']:die('source display mismatch')
 reg=json.loads(read(r['registration']));allowed=[]
 for row in reg.get('items',[]):
  if row.get('file')==r['source']:allowed.extend(row.get('display_ranges',[]))
 if not any(lo>=a and hi<=b for a,b in allowed):die('display outside registered windows')
 at=reg.get('registered_at_utc') or reg.get('captured_at_utc')
 if not at or at>r['displayed_at_utc']:die('registration after display')
 views+=1
rawq=read('QUEUE-2330-RAW.json');a=json.loads(read('QUEUE-2330-PROJECTION.json'));b=json.loads(read('QUEUE-2330-PUBLIC-PROJECTION.json'));q=json.loads(read('QUEUE-IDENTITY.json'))
if json.loads(rawq)!=a or a!=b or (len(rawq),sha(rawq))!=(q['raw_bytes'],q['raw_sha256']):die('queue selected identity mismatch')
h=json.loads(read('HISTORY-CAPTURE.json'))
for row in h['items']:
 if row['found']:
  data=read(row['capture'])
  if (len(data),sha(data))!=(row['bytes'],row['actual_sha256']) or row['actual_sha256']!=row['expected_sha256']:die('historical 059 receipt mismatch')
for n in ['L13-CAPTURE.json','L13-PRIMARY-CAPTURE.json']:
 for row in json.loads(read(n))['items']:
  if row['found']:
   data=read(row['capture']);expected_hash=row.get('actual_sha256') or row['sha256']
   if (len(data),sha(data))!=(row['bytes'],expected_hash):die('historical L13 capture mismatch')
   if 'actual_sha256' in row and row['actual_sha256']!=row['sha256']:die('historical primary expected hash mismatch')
first=read('sources/95e6-inventory.rs')
for f in ['sources/e275-inventory.rs','sources/6533-inventory.rs','sources/original-809.rs']:
 if read(f)!=first:die('per-file source identity mismatch')
sel=json.loads(read('HISTORY-SELECTOR-REGISTRATION.json'))
for label in 'ABC':
 receipt=json.loads(read('HISTORY-RECEIPT-'+label+'.json'));data=read(receipt['display'])
 if sha(data)!=receipt['sha256'] or receipt['truncated']:die('history display receipt mismatch')
 for name,part in json.loads(data).items():
  full=json.loads(read(name));spec=sel['selectors'][name]
  if spec=='whole':
   if part!=full:die('history full projection mismatch')
  else:
   for k in sel['shared_metadata']:
    if k in full and part.get(k)!=full[k]:die('history metadata mismatch')
   for pointer in spec:
    if part.get(pointer)!=ptr(full,pointer):die('history row mismatch')
for regfile,project,receipt in [('L13-SELECTOR-REGISTRATION.json','L13-SELECTED.json','L13-RECEIPT.json'),('L13-PRIMARY-SELECTOR.json','L13-PRIMARY-SELECTED.json','L13-PRIMARY-RECEIPT.json')]:
 reg=json.loads(read(regfile));data=read(project);r=json.loads(read(receipt))
 if sha(data)!=r['sha256'] or r['truncated']:die('L13 display mismatch')
 for name,part in json.loads(data).items():
  full=json.loads(read(name))
  for pointer in reg['selectors'][name]:
   if part.get(pointer)!=ptr(full,pointer):die('L13 selected pointer mismatch')
for row in json.loads(read('TRANSFER-PROVENANCE.json'))['items']:
 data=read(row['path'])
 if (len(data),sha(data))!=(row['bytes'],row['sha256']):die('prior scratch transfer mismatch')
lock=tomllib.loads(read('sources/current-Cargo.lock').decode());packages={(x['name'],x['version']):x for x in lock['package']}
for capture,member,package,version in [('NATIVE-CAPTURE.json','serde_json-1.0.151/src/de.rs','serde_json','1.0.151'),('CORE-CAPTURE.json','serde_core-1.0.229/src/de/impls.rs','serde_core','1.0.229')]:
 x=json.loads(read(capture));x=x['items'][1] if 'items' in x else x
 archive=read(x['archive_file']);source=read(x.get('source_file') or x['file']);expected_hash=x.get('source_sha256') or x['sha256']
 if sha(archive)!=x['archive_sha256'] or packages[(package,version)]['checksum']!=sha(archive) or sha(source)!=expected_hash:die('native package identity mismatch')
 with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
  if t.extractfile(member).read()!=source:die('native archive member mismatch')
if len(json.loads(read('READ-LIMITS.json'))['current_personal_views'])!=views:die('view inventory mismatch')
parser=argparse.ArgumentParser();parser.add_argument('--repo');args=parser.parse_args()
if args.repo:
 for x in proofs:
  root=subprocess.check_output(['git','-C',args.repo,'rev-parse',x['revision']+'^{tree}']).decode().strip()
  if root!=x['revision_tree']:die('local commit tree mismatch')
  if subprocess.check_output(['git','-C',args.repo,'show',x['revision']+':'+x['path']])!=read(x['file']):die('local Git source mismatch')
 raw=subprocess.check_output(['git','-C',args.repo,'show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json']);lo,hi=q['byte_range_half_open']
 if sha(raw)!=q['source_queue_sha256'] or raw[lo:hi]!=rawq or json.loads(raw)['entries'][2330]!=a:die('original queue slice mismatch')
 pub=subprocess.check_output(['git','-C',args.repo,'show',q['public_revision']+':baseline-059/queue.json'])
 if sha(pub)!=q['public_queue_sha256'] or json.loads(pub)['entries'][2330]!=a:die('public selected queue projection mismatch')
 reg=json.loads(read('LEDGER-REGISTRATION.json'));led=subprocess.check_output(['git','-C',args.repo,'show',reg['source_commit']+':'+reg['source_path']])
 if sha(led)!=reg['source_sha256'] or json.loads(led)['newly_assessed_ledger']['entries'][59]!=json.loads(read('LEDGER-59-PROJECTION.json')):die('ledger projection mismatch')
 print('Local Git commit-to-tree, sources, original 059 slice and selected ledger relationships rechecked.')
else:print('Without --repo, commit-to-tree and original whole-queue slice binding were not rechecked.')
print(f'Verified {len(expected)} files, {len(proofs)} source tree chains and {views} registered source displays; two native archives match lock checksums and source members.')
print('Whole-tree cleanliness, global privacy, independently recovered custody, product behavior and audit closure remain outside this verifier.')
