#!/usr/bin/env python3
"""Independent, read-only verification of follow-up stage evidence identities."""
import argparse,hashlib,json,pathlib,tarfile,gzip,io,collections,stat
ap=argparse.ArgumentParser();ap.add_argument('--stage',required=True,type=pathlib.Path);ap.add_argument('--private',required=True,type=pathlib.Path);a=ap.parse_args();R=a.stage;Q=a.private;H=lambda b:hashlib.sha256(b).hexdigest();oid=lambda kind,b:hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def read(p):assert p.is_file() and not p.is_symlink();return p.read_bytes()
def treemap(raw):
 out={};i=0
 while i<len(raw):
  j=raw.index(b'\0',i);mode,name=raw[i:j].split(b' ',1);assert name not in out;out[name]=(mode.decode().lstrip('0'),raw[j+1:j+21].hex());i=j+21
 assert i==len(raw);return out
sm=read(R/'STAGE-MANIFEST.json');assert H(sm)=='aa2f8364932cc2bc3ca00e9d71501392dd301002d7ea1dfd9964d4aa8a6dd413';manifest=json.loads(sm);expected={x['path']:x for x in manifest['files']};assert len(expected)==len(manifest['files']);actual={str(x.relative_to(R)) for x in R.rglob('*') if x.is_file() or x.is_symlink()}-{'STAGE-MANIFEST.json'};assert actual==set(expected)
for n,x in expected.items():b=read(R/n);assert (H(b),len(b))==(x['sha256'],x['bytes'])
cat=json.loads(read(R/'CATALOG.json'));assert len(cat['packets'])==8 and len({x['id'] for x in cat['packets']})==8;assert cat['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cat['product_validation']=='not performed';roles=[y for x in cat['packets'] for y in x['original_roles']];assert len(set(roles))==len(roles)==14 and set(roles)==set(range(2629,2643))
arc={};archive_counts={}
for item in cat['packets']:
 b=read(R/item['archive']);assert (H(b),len(b))==(item['archive_sha256'],item['archive_bytes'])
 with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as tar:
  ms=tar.getmembers();names=[x.name for x in ms];assert len(names)==len(set(names));data={}
  for x in ms:
   p=pathlib.PurePosixPath(x.name);assert x.isfile() and not p.is_absolute() and '..' not in p.parts and str(p)==x.name;assert (x.uid,x.gid,x.uname,x.gname,x.mtime,x.mode)==(0,0,'','',0,0o644);data[x.name]=tar.extractfile(x).read()
 mf=data[item['manifest']];assert H(mf)==item['manifest_sha256'];files=json.loads(mf)['files'];rows={x['path']:(x['sha256'],x['bytes']) for x in files} if isinstance(files,list) else {k:(v['sha256'],v['bytes']) for k,v in files.items()};assert len(rows)==len(files) and set(data)==set(rows)|{item['manifest']}
 for n,(sha,size) in rows.items():assert (H(data[n]),len(data[n]))==(sha,size),(item['id'],n)
 archive_counts[item['id']]=len(ms);arc[item['id']]=data
for x in cat['same_machine_receipts']:
 b=read(R/x['path']);assert (H(b),len(b))==(x['sha256'],x['bytes'])
crossraw=read(R/cat['source_crosswalk']);assert H(crossraw)==cat['source_crosswalk_sha256'];cross=json.loads(crossraw);central=arc['central-13l'];cloth=arc['cloth'];review=arc['review-13l'];assert H(central['MANIFEST.json'])==cross['central_manifest_sha256'] and H(cloth['MANIFEST.json'])==cross['cloth_public_manifest_sha256']
reg=json.loads(central['REGISTRATION.json']);source={x['id']:x for x in reg['rows']};assert reg['reader']=='/root';proof=json.loads(cloth['reconstructed/GIT-PROOFS.json']);objects={}
for key,x in proof['objects'].items():
 b=gzip.decompress(cloth['reconstructed/'+x['archive']]);assert (H(b),len(b),oid(x['type'],b))==(x['sha256'],x['bytes'],key);objects[key]=b
proofpaths=[]
for x in cross['rows']:
 c=source[x['id']];one=central[c['stored_copy']];two=cloth[x['cloth_packet_path']]
 if x['id']=='rust-reference-cast':assert H(gzip.decompress(one))==H(gzip.decompress(two))==x['central_decoded_sha256'];continue
 assert (one==two and H(one)==x['sha256']==c['source_sha256'] and oid('blob',one)==x['git_blob']==c['git_blob']);assert c['revision']==x['revision'] and c['project_path']==x['path'] and c['bytes']==len(one)
 match=[r for r in proof['paths'] if (r['revision'],r['path'],r['blob'])==(x['revision'],x['path'],x['git_blob'])];assert len(match)==1;z=match[0];assert (z['sha256'],z['bytes'])==(H(one),len(one));root=objects[z['revision']].split(b'\n',1)[0];assert root.startswith(b'tree ');current=root[5:].decode();names=[]
 for step in z['chain']:
  assert current==step['tree'];found=treemap(objects[current])[step['name'].encode()];assert found==(step['mode'].lstrip('0'),step['object']);current=step['object'];names.append(step['name'])
 assert names==x['path'].split('/') and current==x['git_blob'];proofpaths.append({'id':x['id'],'revision':x['revision'],'path':x['path'],'blob':x['git_blob'],'sha256':x['sha256']})
# Rebuild each declared display, check receipt IDs and reader, timestamps and coverage.
covered={key:set() for key in source};display_counts={}
for regfile,receiptfile,displayfile,kind in [('REGISTRATION.json','READ-RECEIPT.json','DISPLAY.txt','base'),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json','EXT-DISPLAY.txt','extension'),('EXT2-REGISTRATION.json','EXT2-READ-RECEIPT.json','EXT2-DISPLAY.txt','single')]:
 r=json.loads(central[regfile]);receipt=json.loads(central[receiptfile]);assert r['reader']==receipt['reader']=='/root' and r['registered_utc']<receipt['displayed_utc'] and H(central[regfile])==receipt['registered_sha256'];assert len(r['rows'])==len(receipt['rows']);lines=[]
 if kind!='base':assert r['prior_registration_sha256']==H(central['REGISTRATION.json'])
 for row,rec in zip(r['rows'],receipt['rows']):
  assert row['id']==rec['id'];src=source[row['id']];content=central[src['stored_copy']];content=gzip.decompress(content) if row['id']=='rust-reference-cast' else content;text=content.decode().splitlines();assert rec['eof']==len(text) and row['planned_display_ranges']==rec['displayed_ranges']
  if kind=='base':lines.append('=== '+row['id']+' ===')
  elif kind=='extension':lines.append('=== '+row['id']+' extension ===')
  for lo,hi in row['planned_display_ranges']:
   assert 1<=lo<=hi<=len(text)
   for line in range(lo,hi+1):lines.append(f'{line:4d} {text[line-1]}');covered[row['id']].add(line)
  if 'unread_complement' in rec:
   included={i for lo,hi in rec['displayed_ranges'] for i in range(lo,hi+1)};unread={i for lo,hi in rec['unread_complement'] for i in range(lo,hi+1)};assert not included&unread and included|unread==set(range(1,len(text)+1))
 output=('\n'.join(lines)+'\n').encode();assert output==central[displayfile] and H(output)==receipt['display_sha256'];display_counts[kind]=len(lines)
assessment=json.loads(central['ASSESSMENT.json']);assert assessment['source_revision']==cat['working_revision'] and assessment['repair']=='not started' and assessment['product_validation']=='not performed';
for ref in assessment['source_references']:
 for lo,hi in ref['lines']:assert set(range(lo,hi+1))<=covered[ref['id']]
reviewed=central['ASSESSMENT-REVIEWED-REVISION.json'];final=central['ASSESSMENT.json'];assert reviewed==review['bundle/inputs/coordinator/ASSESSMENT-final.json'] and H(reviewed)=='7370c34c6e851d8ba8de247eb70aaf8412fa95cd9bb97547b5c5f709bb182052' and H(final)=='4897ae53b9c67c18017d8d361688cb3a1b2c05f5d5386c2c7f709b0e9d239479';old=json.loads(reviewed);new=json.loads(final)
def changed(a,b,path=''):
 if isinstance(a,dict) and isinstance(b,dict) and a.keys()==b.keys():
  for k in a:yield from changed(a[k],b[k],path+'/'+k)
 elif isinstance(a,list) and isinstance(b,list) and len(a)==len(b):
  for i,(x,y) in enumerate(zip(a,b)):yield from changed(x,y,path+'/'+str(i))
 elif a!=b:yield path
changes=list(changed(old,new));assert set(changes)=={'/historical_attribution/1','/review_qualification'} and 'not separately peer reviewed' in new['review_qualification']
# Verify restricted same-machine manifest and mode, without printing private values.
pm=read(Q/'MANIFEST.json');assert H(pm)==cat['private_retention_manifest_sha256'];private=json.loads(pm);pc=collections.Counter();fileset=set();violations=0
for x in private['files']:
 p=Q/x['group']/x['path'];raw=read(p);assert (H(raw),len(raw))==(x['sha256'],x['bytes']);pc[x['group']]+=1;fileset.add(str(p.relative_to(Q)));violations+=bool(stat.S_IMODE(p.stat().st_mode)&0o077)
for p in [Q,*[z for z in Q.rglob('*') if z.is_dir()]]:violations+=bool(stat.S_IMODE(p.stat().st_mode)&0o077)
assert violations==0
# Recheck all original/public members and changes for the two packets this reviewer authored.
transform={}
for tag in ['cloth','hair-style']:
 packet=arc[tag];i=json.loads(packet['INDEX.json']);origroot=Q/tag/'original';source_manifest=read(origroot/'MANIFEST.json');assert H(source_manifest)==i['original_manifest_sha256'];assert i['original_author']=='/root/rig_identity';assert json.loads(packet['reconstructed/REPORT.json'])['author']=='/root/rig_identity';changed_members=0;edits=0;labels={}
 mapping=json.loads(read(Q/tag/'MAPPING.json'))['mapping'] if tag=='hair-style' else None
 for row in i['files']:
  original=read(origroot/row['path']);public=packet['reconstructed/'+row['path']];assert (H(original),len(original),H(public),len(public))==(row['original_stored_sha256'],row['original_stored_bytes'],row['public_stored_sha256'],row['public_stored_bytes'])
  if not row['edits']:assert original==public;continue
  changed_members+=1
  if tag=='cloth':
   marker=row['edits'][0]['marker'].encode();assert len(row['edits'])==1 and public.count(marker)==1;before,after=public.split(marker);assert original.startswith(before) and original.endswith(after);end=len(original)-len(after) if after else len(original);token=original[len(before):end];assert token.startswith(b'/') and b'\n' not in token and b'\r' not in token;labels.setdefault(marker,set()).add(H(token));assert original==before+token+after;edits+=1
  else:
   out=original
   for marker,token in sorted(mapping.items(),key=lambda x:len(x[1]),reverse=True):
    token=token.encode();occ=out.count(token);out=out.replace(token,marker.encode());edits+=occ
   assert out==public
 assert changed_members==(2 if tag=='cloth' else 4) and edits==(2 if tag=='cloth' else 7);transform[tag]={'original_members':len(i['files']),'changed_members':changed_members,'replayed_substitutions':edits,'original_manifest_sha256':i['original_manifest_sha256']};
 if tag=='cloth':transform[tag]['same_marker_distinct_original_values']=len(labels[b'<LOCAL_PATH_0>'])
# Historical review provenance is preserved and the same-machine outputs are receipts, not reruns.
assert json.loads(review['REPORT.json'])['author']=='/root/hair_ops';assert json.loads(review['bundle/REPORT.json'])['reviewer']=='/root/hair_ops';assert json.loads(arc['hair-style']['INDEX.json'])['prior_author']=='/root/mhr_character'
result={'schema':'726-followup-independent-input-check-v1','stage_manifest_sha256':H(sm),'stage_bound_files':len(expected),'archive_count':len(arc),'archive_members':archive_counts,'original_roles':sorted(roles),'source_git_paths':proofpaths,'source_git_objects_checked':len(objects),'coordinator_display_line_counts':display_counts,'coordinator_assessment_cites_within_receipts':True,'reviewed_assessment_sha256':H(reviewed),'final_assessment_sha256':H(final),'reviewed_to_final_changed_json_paths':changes,'private_retention_manifest_sha256':H(pm),'private_members_checked':len(private['files']),'private_groups':dict(pc),'private_mode_violations':violations,'authored_packet_original_public_replay':transform,'qualification':'Evidence identities and source-display receipts only. Transcript reproduction does not prove a human/model actually read the text, semantic correctness, original reviewer approval of later wording, global privacy or product behavior. Same-machine receipts for other packets are retained assertions, not rerun by this checker.'};print(json.dumps(result,indent=2))
