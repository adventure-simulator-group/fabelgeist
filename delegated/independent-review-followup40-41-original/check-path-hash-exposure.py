#!/usr/bin/env python3
"""Locate hashes of redacted cloth path values without emitting values or hashes."""
import argparse,pathlib,hashlib,json,tarfile,gzip,collections
ap=argparse.ArgumentParser();ap.add_argument('--stage',required=True,type=pathlib.Path);ap.add_argument('--private',required=True,type=pathlib.Path);a=ap.parse_args();A=a.stage/'archives';base=a.private/'cloth/original';tokens=[];names=[]
with tarfile.open(A/'cloth.tar.gz','r:gz') as t:
 index=json.load(t.extractfile('INDEX.json'))
 for x in index['files']:
  if not x['edits']:continue
  original=(base/x['path']).read_bytes();public=t.extractfile('reconstructed/'+x['path']).read();marker=x['edits'][0]['marker'].encode();before,after=public.split(marker);assert original.startswith(before) and original.endswith(after);token=original[len(before):len(original)-len(after) if after else len(original)];assert token.startswith(b'/');tokens.append(hashlib.sha256(token).hexdigest());names.append(x['path'])
assert len(tokens)==2 and tokens[0]!=tokens[1]
hits=[]
def walk(v,p):
 if isinstance(v,dict):
  for k,x in v.items():yield from walk(x,p+'/'+k.replace('~','~0').replace('/','~1'))
 elif isinstance(v,list):
  for i,x in enumerate(v):yield from walk(x,p+'/'+str(i))
 elif isinstance(v,str):
  for i,z in enumerate(tokens):
   if v==z:yield i,p
for file in sorted(A.glob('*.tar.gz')):
 with tarfile.open(file,'r:gz') as t:
  for x in t:
   if not x.isfile():continue
   raw=t.extractfile(x).read();raw=gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw
   if any(z.encode() in raw for z in tokens):
    try:found=list(walk(json.loads(raw),''))
    except (ValueError,UnicodeDecodeError):found=[]
    if found:hits.extend({'archive':file.name,'member':x.name,'source_placeholder_member':names[i],'json_pointer':p} for i,p in found)
    else:hits.append({'archive':file.name,'member':x.name,'source_placeholder_member':'undetermined','json_pointer':'non-json-or-nested'})
with tarfile.open(A/'gpu-reflection.tar.gz','r:gz') as t:gpu=json.load(t.extractfile('RECONSTRUCTION-MAP.json'))
c=collections.Counter()
def count(v):
 if isinstance(v,dict):
  for k,x in v.items():
   if k=='original_value_sha256':c['fields']+=1;c['digest_entries']+=len(x) if isinstance(x,list) else 1
   else:count(x)
 elif isinstance(v,list):
  for x in v:count(x)
count(gpu);assert c=={'fields':45,'digest_entries':47};assert len(hits)==3 and all(x['source_placeholder_member']=='build-git-proofs.py' for x in hits)
print(json.dumps({'schema':'726-followup-private-value-digest-exposure-v1','qualification':'The two cloth marker values differ, established by local private comparison. This output intentionally contains neither value nor either value digest. The public GPU map contains original-value digest fields distinct from whole-file integrity hashes. Matching digests enable equality correlation and may permit guessing of low-entropy paths; evaluate for public release.','same_marker_distinct_original_values':True,'cloth_members':names,'gpu_map_original_value_hash_fields':c['fields'],'gpu_map_original_value_digest_entries':c['digest_entries'],'matching_locations':hits,'second_cloth_value_hash_observed_in_scanned_members':False,'scope':'Search covered uncompressed stored or gzip-decoded members of the eight outer archives; it is not a global absence claim or a scan of arbitrary nested encodings.'},indent=2))
