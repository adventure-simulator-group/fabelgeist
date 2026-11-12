#!/usr/bin/env python3
"""Independent read-only successor comparison and restricted source replay.
No private values or individual digest encodings are printed or saved.
"""
import argparse,collections,copy,gzip,hashlib,io,json,pathlib,stat,tarfile
p=argparse.ArgumentParser();p.add_argument('--old-stage',required=True,type=pathlib.Path);p.add_argument('--new-stage',required=True,type=pathlib.Path);p.add_argument('--private',required=True,type=pathlib.Path);p.add_argument('--original-source',required=True,type=pathlib.Path);p.add_argument('--out',required=True,type=pathlib.Path);a=p.parse_args();H=lambda b:hashlib.sha256(b).hexdigest();oid=lambda kind,b:hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
OLD='aa2f8364932cc2bc3ca00e9d71501392dd301002d7ea1dfd9964d4aa8a6dd413';NEW='5dfe0bfe427cf9089f636ac8e52ae1a7e479e5ce0372e2fa8d884cd3f6c07e15'
def data(p):assert p.is_file() and not p.is_symlink();return p.read_bytes()
def unpackage(p,normalized=True):
 with tarfile.open(p,'r:gz') as t:
  z=t.getmembers();assert len(z)==len({x.name for x in z});out={}
  for x in z:
   path=pathlib.PurePosixPath(x.name);assert x.isfile() and not path.is_absolute() and '..' not in path.parts and str(path)==x.name
   if normalized:assert (x.uid,x.gid,x.uname,x.gname,x.mtime,x.mode)==(0,0,'','',0,0o644)
   out[x.name]=t.extractfile(x).read()
  return out
def stage(root,expected):
 b=data(root/'STAGE-MANIFEST.json');assert H(b)==expected;m=json.loads(b);rows={x['path']:x for x in m['files']};assert len(rows)==len(m['files'])
 files={x.relative_to(root).as_posix() for x in root.rglob('*') if x.is_file() or x.is_symlink()}-{'STAGE-MANIFEST.json'};assert files==set(rows)
 for n,x in rows.items():z=data(root/n);assert (H(z),len(z))==(x['sha256'],x['bytes'])
 c=json.loads(data(root/'CATALOG.json'));assert c['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and c['product_validation']=='not performed';out={}
 for x in c['packets']:
  raw=data(root/x['archive']);assert (H(raw),len(raw))==(x['archive_sha256'],x['archive_bytes']);arc=unpackage(root/x['archive']);mf=arc[x['manifest']];assert H(mf)==x['manifest_sha256'];files=json.loads(mf)['files'];rows={v['path']:(v['sha256'],v['bytes']) for v in files} if isinstance(files,list) else {k:(v['sha256'],v['bytes']) for k,v in files.items()};assert len(rows)==len(files) and set(arc)==set(rows)|{x['manifest']}
  for n,(sha,size) in rows.items():assert (H(arc[n]),len(arc[n]))==(sha,size)
  out[x['id']]=arc
 return c,out
oc,oa=stage(a.old_stage,OLD);nc,na=stage(a.new_stage,NEW);assert set(oa)==set(na) and len(na)==8
oldrows={x['id']:x for x in oc['packets']};newrows={x['id']:x for x in nc['packets']};same=[]
for key in oa:
 if key!='gpu-reflection':
  assert data(a.old_stage/oldrows[key]['archive'])==data(a.new_stage/newrows[key]['archive']);same.append(key)
assert len(same)==7
old=oa['gpu-reflection'];new=na['gpu-reflection'];tm=json.loads(new['SUCCESSOR-TRANSPORT-MAP.json']);assert tm['source_member_count']==len(tm['records'])==len(old)==161;counts=collections.Counter()
for row in tm['records']:
 before=old[row['original_path']];after=new[row['successor_path']];assert (H(before),len(before))==(row['original_sha256'],row['original_bytes']);counts[row['status']]+=1
 if row['status']=='replaced-manifest':assert row['successor_path']=='PUBLIC-MANIFEST.json' and 'successor_identity' in row
 else:assert (H(after),len(after))==(row['successor_sha256'],row['successor_bytes'])
 if row['status']=='exact-copy':assert before==after
assert counts=={'exact-copy':154,'reconstructed':6,'replaced-manifest':1}
# Compare normalized mapping structures: only individual digest fields are
# replaced by counts, with the declared added top-level qualification.
om=json.loads(old['RECONSTRUCTION-MAP.json']);nm=json.loads(new['RECONSTRUCTION-MAP.json']);scrub=copy.deepcopy(om);removed=collections.Counter()
def scrub_map(v):
 if isinstance(v,dict):
  for oldkey,newkey in [('original_value_sha256','original_value_digest_omitted_count'),('public_value_sha256','public_value_digest_omitted_count')]:
   if oldkey in v:
    x=v.pop(oldkey);n=len(x) if isinstance(x,list) else 1;v[newkey]=n;removed[oldkey+'_fields']+=1;removed[oldkey+'_values']+=n
  for x in list(v.values()):scrub_map(x)
 elif isinstance(v,list):
  for x in v:scrub_map(x)
scrub_map(scrub);nm2=copy.deepcopy(nm);assert nm2.pop('individual_value_digest_omission');assert scrub==nm2;assert removed=={'original_value_sha256_fields':45,'original_value_sha256_values':47,'public_value_sha256_fields':2,'public_value_sha256_values':2}
scan_counts={}
for name in ['ORIGINAL-PRIVACY-SCAN.json','PUBLIC-PRIVACY-SCAN.json']:
 before=json.loads(old[name]);after=json.loads(new[name]);assert len(before['hits'])==len(after['hits'])
 for hit in before['hits']:assert 'matched_prefix_sha256' in hit;hit.pop('matched_prefix_sha256');hit['matched_prefix_digest_omitted']=True
 assert before==after;scan_counts[name]=len(after['hits'])
assert scan_counts=={'ORIGINAL-PRIVACY-SCAN.json':53,'PUBLIC-PRIVACY-SCAN.json':7}
# Restricted predecessor capture and independent same-machine original replay.
cap=json.loads(data(a.private/'CAPTURE.json'));assert cap['original_stage_manifest_sha256']==OLD
assert H(data(a.private/'original-gpu-archive.tar.gz'))==cap['original_gpu_archive_sha256']==H(data(a.old_stage/oldrows['gpu-reflection']['archive']))
assert data(a.private/'original-source-capture/exact-originals.tar.gz')==data(a.original_source/'exact-originals.tar.gz')
assert stat.S_IMODE(a.private.stat().st_mode)&0o077==0
sensitive=['original-gpu-archive.tar.gz','original-unpublished-stage.tar.gz','original-reconstruction-map.json','sensitive-digest-corpus.json','sensitive-digest-corpus2.json','sensitive-digest-corpus3.json','original-source-capture/exact-originals.tar.gz']
for n in sensitive:assert stat.S_IMODE((a.private/n).stat().st_mode)&0o077==0
source=unpackage(a.original_source/'exact-originals.tar.gz',normalized=False);mapping=om;assert len(source)==len(mapping['records'])==154

def dec(b):return gzip.decompress(b) if b.startswith(b'\x1f\x8b') else b
def diff(x,y,path=''):
 if isinstance(x,dict) and isinstance(y,dict):
  assert x.keys()==y.keys()
  for k in x:yield from diff(x[k],y[k],path+'/'+k.replace('~','~0').replace('/','~1'))
 elif isinstance(x,list) and isinstance(y,list):
  assert len(x)==len(y)
  for i,(u,v) in enumerate(zip(x,y)):yield from diff(u,v,path+'/'+str(i))
 elif x!=y:yield(path,x,y)
replayed=collections.Counter();changed=0
for row in mapping['records']:
 original=source[row['original_path']];public=new[row['public_path']];assert (H(original),len(original),H(public),len(public))==(row['original_stored_sha256'],row['original_stored_bytes'],row['public_stored_sha256'],row['public_stored_bytes'])
 if not row['transformations']:assert original==public;continue
 changed+=1;assert len(row['transformations'])==1;tr=row['transformations'][0];kind=tr['kind'];replayed[kind]+=tr['count']
 if kind=='path-literal-redaction':
  marker=tr['public_marker'].encode();parts=public.split(marker);assert len(parts)-1==tr['count'] and original.startswith(parts[0]);position=len(parts[0]);h=[]
  for part in parts[1:]:
   end=original.find(part,position) if part else len(original);assert end>=position;token=original[position:end];h.append(H(token));position=end+len(part)
  assert position==len(original) and sorted(h)==sorted(tr['original_value_sha256'])
 else:
  assert kind in ('json-path-redaction','existing-public-variant','reconstructed-gzip-transport')
  changes=list(diff(json.loads(dec(original)),json.loads(dec(public))));fields={x['pointer']:x for x in tr['fields']};assert len(changes)==len(fields)==tr['count'] and set(fields)=={x[0] for x in changes}
  for ptr,ov,pv in changes:
   assert isinstance(ov,str) and ov!=pv;z=fields[ptr]
   if kind=='existing-public-variant':assert H(json.dumps(ov,ensure_ascii=False).encode())==z['original_value_sha256'] and H(json.dumps(pv,ensure_ascii=False).encode())==z['public_value_sha256']
   else:assert H(ov.encode())==z['original_value_sha256']
assert replayed=={'path-literal-redaction':5,'existing-public-variant':2,'reconstructed-gzip-transport':26,'json-path-redaction':14}
# Central 13L: source crosswalk and the five Git tree paths, plus the separate
# decoded reference. The seven unchanged packets retain their original reads.
cb=data(a.new_stage/'CENTRAL-13L-SOURCE-CROSSWALK.json');cross=json.loads(cb);assert H(cb)==nc['source_crosswalk_sha256'];central=na['central-13l'];cloth=na['cloth'];review=na['review-13l'];assert H(central['MANIFEST.json'])==cross['central_manifest_sha256'] and H(cloth['MANIFEST.json'])==cross['cloth_public_manifest_sha256']
reg=json.loads(central['REGISTRATION.json']);src={x['id']:x for x in reg['rows']};proof=json.loads(cloth['reconstructed/GIT-PROOFS.json']);objects={}
for key,v in proof['objects'].items():
 b=gzip.decompress(cloth['reconstructed/'+v['archive']]);assert (H(b),len(b),oid(v['type'],b))==(v['sha256'],v['bytes'],key);objects[key]=b
def tree(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,name=b[i:end].split(b' ',1);assert name not in out;out[name]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
bound=[]
for row in cross['rows']:
 r=src[row['id']];one=central[r['stored_copy']];two=cloth[row['cloth_packet_path']]
 if row['id']=='rust-reference-cast':assert H(gzip.decompress(one))==H(gzip.decompress(two))==row['central_decoded_sha256'];continue
 assert one==two and H(one)==row['sha256']==r['source_sha256'] and oid('blob',one)==row['git_blob']==r['git_blob'];assert r['revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
 choices=[x for x in proof['paths'] if (x['revision'],x['path'],x['blob'])==(row['revision'],row['path'],row['git_blob'])];assert len(choices)==1;x=choices[0];current=objects[x['revision']].split(b'\n',1)[0][5:].decode();names=[]
 for step in x['chain']:
  assert current==step['tree'] and tree(objects[current])[step['name'].encode()]==(step['mode'].lstrip('0'),step['object']);names.append(step['name']);current=step['object']
 assert names==x['path'].split('/') and current==x['blob'];bound.append(row['id'])
assert len(bound)==5 and len(cross['rows'])==6
reviewed=central['ASSESSMENT-REVIEWED-REVISION.json'];final=central['ASSESSMENT.json'];assert reviewed==review['bundle/inputs/coordinator/ASSESSMENT-final.json'];assert H(reviewed)=='7370c34c6e851d8ba8de247eb70aaf8412fa95cd9bb97547b5c5f709bb182052' and H(final)=='4897ae53b9c67c18017d8d361688cb3a1b2c05f5d5386c2c7f709b0e9d239479';changes=[x[0] for x in diff(json.loads(reviewed),json.loads(final))];assert set(changes)=={'/historical_attribution/1','/review_qualification'} and 'not separately peer reviewed' in json.loads(final)['review_qualification']
assert len({y for x in nc['packets'] for y in x['original_roles']})==14
assert H(data(a.new_stage/'STAGE-MANIFEST.json'))==NEW
result={'schema':'726-followup-successor-independent-proof-v1','reviewer':'/root/rig_identity','old_stage_manifest_sha256':OLD,'successor_stage_manifest_sha256':NEW,'archive_count':len(na),'archive_members':{k:len(v) for k,v in na.items()},'seven_unchanged_archives':sorted(same),'gpu_transport_status_counts':dict(counts),'gpu_map_omitted_counts':dict(removed),'scan_prefix_field_counts':scan_counts,'restricted_gpu_source_files_replayed':len(source),'restricted_gpu_changed_files':changed,'replayed_transform_counts':dict(replayed),'13l_project_git_membership_paths':bound,'13l_reference_qualification':'Sixth source is a separately identified decoded language reference; it is not a project Git blob.','13l_reviewed_assessment_sha256':H(reviewed),'13l_final_assessment_sha256':H(final),'13l_changes_since_review':changes,'restricted_custody_qualification':'Restricted predecessor and source bytes were compared on the same machine and the containing directory and seven sensitive files have owner-only modes. This is not independent custody. Nine sanitized scan receipt files have group/other file mode bits, but their ancestor directory is owner-only; ACLs were not inspected.','reviewer_prior_involvement':'This reviewer authored the earlier cloth and hair-style packets and independently reviewed the earlier candidate stage; these activities are not independent authorship.','scope_limit':'Evidence byte identity, transformations, Git membership and assessment revision relationships only; this does not replay semantic reasoning, credit other reviewers as new reads, establish live-remote state, prove complete privacy or validate product behavior.'}
assert not a.out.exists();a.out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'stage_archives':len(na),'unchanged':len(same),'gpu_source_files_replayed':len(source),'gpu_changed_files':changed,'13l_git_paths':len(bound),'same_machine_custody_only':True}))
