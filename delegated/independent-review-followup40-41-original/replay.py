#!/usr/bin/env python3
"""Read-only original/public comparisons; never emits path values or their hashes."""
import argparse,pathlib,json,tarfile,gzip,io,hashlib,collections
ap=argparse.ArgumentParser();ap.add_argument('--stage',required=True,type=pathlib.Path);ap.add_argument('--private',required=True,type=pathlib.Path);a=ap.parse_args();A=a.stage/'archives';Q=a.private;H=lambda b:hashlib.sha256(b).hexdigest()
def tarread(path):
 with tarfile.open(path,'r:gz') as t:
  names=t.getnames();assert len(names)==len(set(names));return {x.name:t.extractfile(x).read() for x in t if x.isfile()}
def dec(b):return gzip.decompress(b) if b.startswith(b'\x1f\x8b') else b
def diff(a,b,path=''):
 if isinstance(a,dict) and isinstance(b,dict):
  assert a.keys()==b.keys()
  for k in a:yield from diff(a[k],b[k],path+'/'+k.replace('~','~0').replace('/','~1'))
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b)
  for i,(x,y) in enumerate(zip(a,b)):yield from diff(x,y,path+'/'+str(i))
 elif a!=b:yield (path,a,b)
results={}
for tag,group in [('garment-pattern','garment-pattern'),('hair-head','hair-head'),('review-13l','central-13l-review')]:
 data=tarread(A/(tag+'.tar.gz'));i=json.loads(data['EVIDENCE-IDENTITIES.json']);changed=0;edits=0
 for x in i['members']:
  public=data[x['public_path']];original=(Q/group/'changed-originals'/x['original_stored_sha256']).read_bytes() if tag=='garment-pattern' and x['redactions'] else ((Q/group/'originals'/x['source_path']).read_bytes() if tag!='garment-pattern' else public)
  assert (H(original),len(original),H(dec(original)),len(dec(original)))==(x['original_stored_sha256'],x['original_stored_bytes'],x['original_decoded_sha256'],x['original_decoded_bytes']);assert (H(public),len(public),H(dec(public)),len(dec(public)))==(x['public_stored_sha256'],x['public_stored_bytes'],x['public_decoded_sha256'],x['public_decoded_bytes'])
  if x['redactions']:
   changed+=1;raw=dec(original);pieces=[];last=0
   for op in x['redactions']:
    start=op['original_decoded_offset_0based'];end=start+op['original_decoded_bytes'];assert start>=last;token=raw[start:end];assert token.startswith((b'/',b'\\')) or (len(token)>2 and token[1:2]==b':');pieces.extend((raw[last:start],op['public_placeholder'].encode()));last=end;edits+=1
   pieces.append(raw[last:]);assert b''.join(pieces)==dec(public)
  else:assert original==public
 assert (changed,edits)=={'garment-pattern':(7,71),'hair-head':(5,49),'review-13l':(2,2)}[tag];results[tag]={'members':len(i['members']),'changed_members':changed,'path_substitutions_replayed':edits}
# Compare each original GPU reflection file to its public mapped stored bytes,
# and check declared field-only changes structurally without exposing values.
gpu=tarread(A/'gpu-reflection.tar.gz');old=tarread(Q/'gpu-reflection/exact-originals.tar.gz');mapping=json.loads(gpu['RECONSTRUCTION-MAP.json']);assert len(old)==len(mapping['records'])==154;counts=collections.Counter();changed=0
for x in mapping['records']:
 original=old[x['original_path']];public=gpu[x['public_path']];assert (H(original),len(original),H(public),len(public))==(x['original_stored_sha256'],x['original_stored_bytes'],x['public_stored_sha256'],x['public_stored_bytes']);trans=x['transformations']
 if not trans:assert original==public;continue
 changed+=1;assert len(trans)==1;z=trans[0];kind=z['kind'];counts[kind]+=z['count']
 if kind=='path-literal-redaction':
  marker=z['public_marker'].encode();parts=public.split(marker);assert len(parts)-1==z['count'] and original.startswith(parts[0]);pos=len(parts[0]);tokens=[]
  for part in parts[1:]:
   end=original.find(part,pos) if part else len(original);assert end>=pos;token=original[pos:end];assert token.startswith((b'/',b'\\')) or (len(token)>2 and token[1:2]==b':');tokens.append(H(token));pos=end+len(part)
  assert pos==len(original) and sorted(tokens)==sorted(z['original_value_sha256'])
 elif kind in ('json-path-redaction','existing-public-variant','reconstructed-gzip-transport'):
  oldobj=json.loads(dec(original));pubobj=json.loads(dec(public));changes=list(diff(oldobj,pubobj));fields=z['fields'];assert len(changes)==len(fields)==z['count'];by={v['pointer']:v for v in fields};assert set(by)=={v[0] for v in changes}
  for pointer,ov,pv in changes:
   entry=by[pointer]
   if kind=='existing-public-variant':assert H(json.dumps(ov,ensure_ascii=False).encode())==entry['original_value_sha256'] and H(json.dumps(pv,ensure_ascii=False).encode())==entry['public_value_sha256']
   else:assert isinstance(ov,str) and H(ov.encode())==entry['original_value_sha256']
 else:raise AssertionError(kind)
assert counts=={'path-literal-redaction':5,'existing-public-variant':2,'reconstructed-gzip-transport':26,'json-path-redaction':14};results['gpu-reflection']={'members':len(mapping['records']),'changed_members':changed,'transformation_counts':dict(counts),'qualification':'Gzip and JSON comparisons are structural for field-only transformations; literal script substitutions are byte replayed.'}
# Hair GPU public names differ for inert scripts; byte payloads remain exact.
hair=tarread(A/'hair-gpu.tar.gz');old=tarread(Q/'hair-gpu/frozen-originals.tar.gz');m=json.loads(hair['TRANSPORT-MAP.json']);assert len(old)==len(m['entries'])==125;renamed=0
for x in m['entries']:
 before=old[x['original_path']];after=hair[x['public_path']];assert before==after and (H(before),len(before))==(x['original_sha256'],x['original_bytes'])==(x['public_sha256'],x['public_bytes']);renamed+=x['public_path']!='evidence/'+x['original_path']
results['hair-gpu']={'members':len(m['entries']),'byte_identical':len(m['entries']),'inert_script_renames':renamed}
print(json.dumps({'schema':'726-followup-reconstruction-replay-v1','results':results,'qualification':'Original/private input versus public bytes and declared transformations only. Paths and their value digests are intentionally not output. No product code executed.'},indent=2))
