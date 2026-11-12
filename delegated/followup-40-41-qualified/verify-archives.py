#!/usr/bin/env python3
"""Verify complete public evidence archives and bounded central source proofs."""
import gzip,hashlib,io,json,tarfile
from pathlib import Path,PurePosixPath
R=Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,name=b[i:end].split(b' ',1);assert name not in out;out[name]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
catalog=json.loads((R/'CATALOG.json').read_bytes());assert catalog['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and catalog['product_validation']=='not performed'
assert len(catalog['packets'])==8 and len({x['id'] for x in catalog['packets']})==8
ids={x['id'] for x in catalog['packets']};assert ids=={'garment-pattern','cloth','gpu-reflection','hair-head','hair-style','hair-gpu','central-13l','review-13l'}
roles=[n for x in catalog['packets'] for n in x['original_roles']];assert len(roles)==len(set(roles))==14 and set(roles)==set(range(2629,2643))
kept={};total=0
for item in catalog['packets']:
 raw=(R/item['archive']).read_bytes();assert (H(raw),len(raw))==(item['archive_sha256'],item['archive_bytes'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  ms=t.getmembers();names=[m.name for m in ms];assert len(names)==len(set(names))
  for m in ms:
   path=PurePosixPath(m.name);assert m.isfile() and not path.is_absolute() and '..' not in path.parts and str(path)==m.name
   assert (m.uid,m.gid,m.uname,m.gname,m.mtime,m.mode)==(0,0,'','',0,0o644)
  data={m.name:t.extractfile(m).read() for m in ms};total+=len(data)
 manifest=data[item['manifest']];assert H(manifest)==item['manifest_sha256'];doc=json.loads(manifest);files=doc['files']
 rows={x['path']:(x['sha256'],x['bytes']) for x in files} if isinstance(files,list) else {k:(v['sha256'],v['bytes']) for k,v in files.items()}
 assert len(rows)==len(files) and set(data)==set(rows)|{item['manifest']}
 for n,(sha,size) in rows.items():assert (H(data[n]),len(data[n]))==(sha,size),(item['id'],n)
 if item['id'] in ('central-13l','review-13l','cloth','gpu-reflection'):kept[item['id']]=data
# Verify successor public representations and digest-field omission without
# claiming replay of inaccessible private original values.
gpu=kept['gpu-reflection'];mapping=json.loads(gpu['RECONSTRUCTION-MAP.json']);old_manifest=json.loads(gpu['evidence/BUNDLE-MANIFEST.json']);oldrows={x['path']:x for x in old_manifest['files']};oldrows['BUNDLE-MANIFEST.json']={'bytes':len(gpu['evidence/BUNDLE-MANIFEST.json']),'sha256':H(gpu['evidence/BUNDLE-MANIFEST.json'])};assert len(mapping['records'])==mapping['record_count']==154 and {x['original_path'] for x in mapping['records']}==set(oldrows)
counts={}
def count_keys(value):
 if isinstance(value,dict):
  assert not ({'original_value_sha256','public_value_sha256','matched_prefix_sha256'} & value.keys())
  for k,v in value.items():
   if k.endswith('value_digest_omitted_count'):counts[k+'_fields']=counts.get(k+'_fields',0)+1;counts[k+'_values']=counts.get(k+'_values',0)+v
   elif k=='matched_prefix_digest_omitted':assert v is True;counts[k]=counts.get(k,0)+1
   else:count_keys(v)
 elif isinstance(value,list):
  for v in value:count_keys(v)
count_keys(mapping);assert counts['original_value_digest_omitted_count_fields']==45 and counts['original_value_digest_omitted_count_values']==47 and counts['public_value_digest_omitted_count_fields']==2 and counts['public_value_digest_omitted_count_values']==2
for path,expected in [('ORIGINAL-PRIVACY-SCAN.json',53),('PUBLIC-PRIVACY-SCAN.json',7)]:
 counts.clear();count_keys(json.loads(gpu[path]));assert counts['matched_prefix_digest_omitted']==expected
for x in mapping['records']:
 r=oldrows[x['original_path']];assert (x['original_stored_bytes'],x['original_stored_sha256'])==(r['bytes'],r['sha256']);b=gpu[x['public_path']];assert (len(b),H(b))==(x['public_stored_bytes'],x['public_stored_sha256'])
 if not x['transformations']:assert (len(b),H(b))==(x['original_stored_bytes'],x['original_stored_sha256'])
qual=json.loads((R/'DIGEST-OMISSION-QUALIFICATION.json').read_bytes());assert qual['successor_gpu_archive_sha256']==next(x for x in catalog['packets'] if x['id']=='gpu-reflection')['archive_sha256'] and qual['digest_counts']['unique_restricted_digest_values']==40
for x in catalog['same_machine_receipts']:
 b=(R/x['path']).read_bytes();assert (H(b),len(b))==(x['sha256'],x['bytes'])
raw=(R/catalog['source_crosswalk']).read_bytes();assert H(raw)==catalog['source_crosswalk_sha256'];cross=json.loads(raw);central=kept['central-13l'];cloth=kept['cloth'];review=kept['review-13l']
assert H(central['MANIFEST.json'])==cross['central_manifest_sha256'] and H(cloth['MANIFEST.json'])==cross['cloth_public_manifest_sha256']
reg=json.loads(central['REGISTRATION.json']);src={x['id']:x for x in reg['rows']};proof=json.loads(cloth['reconstructed/GIT-PROOFS.json']);objects={}
for key,obj in proof['objects'].items():
 body=gzip.decompress(cloth['reconstructed/'+obj['archive']]);assert (H(body),len(body),oid(obj['type'],body))==(obj['sha256'],obj['bytes'],key);objects[key]=body
for x in cross['rows']:
 c=src[x['id']];a=central[c['stored_copy']];b=cloth[x['cloth_packet_path']]
 if x['id']=='rust-reference-cast':assert H(gzip.decompress(a))==H(gzip.decompress(b))==x['central_decoded_sha256'];continue
 assert a==b and H(a)==x['sha256']==c['source_sha256'] and oid('blob',a)==x['git_blob']==c['git_blob']
 matches=[v for v in proof['paths'] if v['revision']==x['revision'] and v['path']==x['path'] and v['blob']==x['git_blob']];assert len(matches)==1
 v=matches[0];assert (v['sha256'],v['bytes'])==(x['sha256'],len(a));cur=objects[v['revision']].splitlines()[0].decode().split()[1];parts=[]
 for step in v['chain']:
  assert step['tree']==cur;parts.append(step['name']);assert tree_entries(objects[cur])[step['name'].encode()]==(step['mode'].lstrip('0'),step['object']);cur=step['object']
 assert parts==x['path'].split('/') and cur==x['git_blob']
# Reconstruct registered coordinator displays, then verify all assessment cites.
covered={x:set() for x in src}
for regname,recname,displayname,kind in [('REGISTRATION.json','READ-RECEIPT.json','DISPLAY.txt','base'),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json','EXT-DISPLAY.txt','extension'),('EXT2-REGISTRATION.json','EXT2-READ-RECEIPT.json','EXT2-DISPLAY.txt','single')]:
 r=json.loads(central[regname]);rec=json.loads(central[recname]);assert H(central[regname])==rec['registered_sha256'] and r['registered_utc']<rec['displayed_utc'];lines=[]
 if kind!='base':assert r['prior_registration_sha256']==H(central['REGISTRATION.json'])
 for row,read in zip(r['rows'],rec['rows']):
  s=src[row['id']];body=central[s['stored_copy']];body=gzip.decompress(body) if row['id']=='rust-reference-cast' else body;source=body.decode().splitlines();assert row['planned_display_ranges']==read['displayed_ranges'] and len(source)==read['eof']
  if kind=='base':lines.append('=== '+row['id']+' ===')
  elif kind=='extension':lines.append('=== '+row['id']+' extension ===')
  for lo,hi in row['planned_display_ranges']:
   assert 1<=lo<=hi<=len(source)
   for n in range(lo,hi+1):lines.append(f'{n:4d} {source[n-1]}');covered[row['id']].add(n)
 expected=('\n'.join(lines)+'\n').encode();assert expected==central[displayname] and H(expected)==rec['display_sha256'] and len(r['rows'])==len(rec['rows'])
a=json.loads(central['ASSESSMENT.json']);assert a['source_revision']==catalog['working_revision'] and a['proposed_ledger_identity']=='local-candidate-cloth-subdivision-count-overflow' and a['repair']=='not started' and a['product_validation']=='not performed'
for x in a['source_references']:
 for lo,hi in x['lines']:assert set(range(lo,hi+1))<=covered[x['id']]
assert central['ASSESSMENT-REVIEWED-REVISION.json']==review['bundle/inputs/coordinator/ASSESSMENT-final.json']
assert H(central['ASSESSMENT-REVIEWED-REVISION.json'])=='7370c34c6e851d8ba8de247eb70aaf8412fa95cd9bb97547b5c5f709bb182052'
assert H(central['ASSESSMENT.json'])!='7370c34c6e851d8ba8de247eb70aaf8412fa95cd9bb97547b5c5f709bb182052'
stage=json.loads((R/'STAGE-MANIFEST.json').read_bytes());actual={x.relative_to(R).as_posix() for x in R.rglob('*') if x.is_file()}-{'STAGE-MANIFEST.json'};assert actual=={x['path'] for x in stage['files']}
for x in stage['files']:
 b=(R/x['path']).read_bytes();assert (H(b),len(b))==(x['sha256'],x['bytes'])
print('Verified',len(catalog['packets']),'complete public archives,',total,'members, 14 distinct role identities, five current Git membership chains and the bounded 13L displays. Private originals and exact replay require restricted inputs; historical attribution, privacy limitations and product behavior remain separately qualified.')
