#!/usr/bin/env python3
"""Verify the captured CP7 reconstruction without writing or executing product.
Requires CP7_ORIGINAL and CP7_OUTER to point at the local original and earlier
outer public archive. Exact originals can contain private paths; remain local.
"""
import gzip,hashlib,io,json,os,re,tarfile
from collections import Counter
from pathlib import Path
p=Path(__file__).resolve().parent;e=p/'evidence';h=lambda b:hashlib.sha256(b).hexdigest()
def load(p):return json.loads(p.read_bytes())
if (p/'MANIFEST.json').exists():
 for x in load(p/'MANIFEST.json')['files']:
  b=(p/x['path']).read_bytes();assert h(b)==x['sha256'] and len(b)==x['bytes'],x['path']
indexraw=(e/'INDEX.json').read_bytes();cpraw=(e/'CHECKPOINT-7-INDEX.json').read_bytes();idx=json.loads(indexraw);cp=json.loads(cpraw)
assert h(indexraw)=='5b3a8e01241c0d00d5f8a0220f0964062de6323edda4cf0131924b93630922c0'
assert h(cpraw)=='174a072454e9d8dea2448bdd0f8fdec9fd3920fb8d9d9dbd20e4a2e649da8c27'
original=Path(os.environ['CP7_ORIGINAL']).read_bytes();public=(e/'reconstructed.tar.gz').read_bytes();outer=Path(os.environ['CP7_OUTER']).read_bytes()
assert (h(original),len(original))==(idx['source_original_sha256'],idx['source_original_bytes'])
pm=load(e/'PRIVATE-ORIGINAL-MANIFEST.json');assert (pm['sha256'],pm['bytes'])==(h(original),len(original))
assert (h(public),len(public))==(idx['public_sha256'],idx['public_bytes'])
b=next(x for x in cp['bundles'] if x['bundle_id']=='strategic-map');assert (h(outer),len(outer))==(b['archive_sha256'],b['archive_bytes'])
m=next(x for x in b['members'] if x['path']=='prior-web-map-route-packet.tar.gz')
assert m['original_sha256']==idx['source_original_sha256'] and m['original_bytes']==idx['source_original_bytes'] and m['published_sha256']==idx['previous_public_sha256']
with tarfile.open(fileobj=io.BytesIO(outer),mode='r:gz') as f:damaged=f.extractfile(m['path']).read()
assert (h(damaged),len(damaged))==(m['published_sha256'],m['published_bytes'])
old_plain=gzip.decompress(original);damaged_plain=gzip.decompress(damaged)
assert h(old_plain)==m['original_uncompressed_sha256'] and h(damaged_plain)==m['published_uncompressed_sha256']
# Reproduce old packaging as opaque byte-stream replacements, without writing.
old_re=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/' rb'726-delegate-[^"\'\s]+',re.I)
opaque=old_plain; count=0
for n,value in enumerate(dict.fromkeys(old_re.findall(old_plain))):
 count+=opaque.count(value);opaque=opaque.replace(value,f'<LOCAL_PATH_{n}>'.encode())
assert opaque==damaged_plain and count==m['redacted_nonessential_local_path_occurrences']==1060
# Python tar iteration silently stops, so inspect the first unread 512-byte header.
with tarfile.open(fileobj=io.BytesIO(damaged),mode='r:gz') as f:
 damaged_members=list(f);next_offset=f.offset
assert len(damaged_members)==5 and next_offset==69120
try:
 tarfile.TarInfo.frombuf(damaged_plain[next_offset:next_offset+512],encoding='utf-8',errors='surrogateescape')
except tarfile.InvalidHeaderError as ex:assert str(ex)=='bad checksum'
else:raise AssertionError('old damaged next header unexpectedly valid')
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/' rb'726-[^"\'\s]+',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
allowed={(b'garden',b'/home/building'),(b'source',b'/home/window/viewport/overlay')};restored=[];regex_source=[];meta_diffs=Counter();pax_changed=[];file_count=0;dir_count=0;changed=0;gz_count=0;redactions=0;other_matches=0;oldfiles={};newfiles={};magics_old=Counter();magics_new=Counter()
with tarfile.open(fileobj=io.BytesIO(original),mode='r:gz') as old,tarfile.open(fileobj=io.BytesIO(public),mode='r:gz') as new:
 oms=old.getmembers();nms=new.getmembers();entries=idx['entries'];assert len(oms)==len(nms)==len(entries)==idx['entry_count']==186
 assert len({x.name for x in oms})==len(oms) and len({x.name for x in nms})==len(nms)
 new_raw=gzip.decompress(public)
 for x,y,row in zip(oms,nms,entries):
  assert x.name==y.name==row['path'] and not x.name.startswith('/') and '..' not in Path(x.name).parts
  assert (x.isfile() and y.isfile() and row['kind']=='file') or (x.isdir() and y.isdir() and row['kind']=='directory')
  fields=['name','type','mode','uid','gid','uname','gname','mtime','linkname','devmajor','devminor','sparse']
  assert all(getattr(x,k)==getattr(y,k) for k in fields)
  for k in ['size','chksum','offset','offset_data','pax_headers']:
   if getattr(x,k)!=getattr(y,k):meta_diffs[k]+=1
  if x.pax_headers!=y.pax_headers:
   assert x.pax_headers=={} and y.pax_headers=={'path':x.name};pax_changed.append(x.name)
  magics_old[old_plain[x.offset_data-512+257:x.offset_data-512+265]]+=1
  magics_new[new_raw[y.offset_data-512+257:y.offset_data-512+265]]+=1
  for val in [y.name,y.uname,y.gname,y.linkname,*y.pax_headers.keys(),*y.pax_headers.values()]:
   assert not local.search(str(val).encode()) and not secret.search(str(val).encode())
  if x.isdir():dir_count+=1;continue
  file_count+=1;old_body=old.extractfile(x).read();pub_body=new.extractfile(y).read();oldfiles[x.name]=old_body;newfiles[y.name]=pub_body
  assert (h(old_body),len(old_body))==(row['original_sha256'],row['original_bytes'])
  assert (h(pub_body),len(pub_body))==(row['public_sha256'],row['public_bytes'])
  enc=row['encoding']; assert enc in ('identity','gzip');gz_count+=enc=='gzip'
  a=gzip.decompress(old_body) if enc=='gzip' else old_body;c=gzip.decompress(pub_body) if enc=='gzip' else pub_body
  assert h(a)==row['original_uncompressed_sha256'] and h(c)==row['public_uncompressed_sha256']
  assert row['changed']==(old_body!=pub_body);changed+=row['changed'];assert not secret.search(c)
  pieces=[];last=0;mapping={};nred=0
  for hit in local.finditer(a):
   pos=hit.start();val=hit.group();pieces.append(a[last:pos]);keep=any(val==s and a[max(0,pos-len(pre)):pos]==pre for pre,s in allowed)
   if keep:
    pieces.append(val);restored.append((x.name,pos,a.count(b'\n',0,pos)+1,val.decode()))
   else:
    if val not in mapping:mapping[val]=len(mapping)
    token=f'<LOCAL_PATH_{mapping[val]}>'.encode();pieces.append(token);nred+=1
    if val==b'/home/[A-Za-z0-9_.-]+/':regex_source.append((x.name,pos,a.count(b'\n',0,pos)+1,token.decode(),h(old_body),h(pub_body)))
    else:other_matches+=1
   last=hit.end()
  pieces.append(a[last:]);expected_plain=b''.join(pieces)
  assert expected_plain==c and nred==row['redacted_path_occurrences'];redactions+=nred
  expected_stored=gzip.compress(expected_plain,mtime=0) if enc=='gzip' and expected_plain!=a else old_body if expected_plain==a else expected_plain
  assert expected_stored==pub_body
  for hit in local.finditer(c):
   pos=hit.start();assert any(hit.group()==s and c[max(0,pos-len(pre)):pos]==pre for pre,s in allowed)
assert (file_count,dir_count,changed,gz_count,redactions)==(123,63,8,52,1085)
assert len(restored)==2 and len(regex_source)==1 and other_matches==1084
assert sorted((path,offset,suffix) for path,offset,_,suffix in restored)==sorted((x['path'],x['original_uncompressed_offset'],x['restored_slash_text']) for x in idx['prose_restorations'])
assert regex_source[0]==('726-delegate-web-map/build-manifest.py',213,7,'<LOCAL_PATH_1>','0e6ed7195b1f83c2bee88cd0b3a69337856db7bb9753b4a213c814cd1f43b8bf','5b04ab6ea37eb44f50be736354e25fd6f973df0956395490435ee4813fd10bae')
assert meta_diffs=={'pax_headers':12,'size':8,'chksum':186,'offset':86,'offset_data':86}
assert magics_old=={b'ustar  \x00':186} and magics_new=={b'ustar\x0000':186}
# The historical embedded manifest remains original-byte evidence, not a new one.
manifest_name='726-delegate-web-map/MANIFEST.json';assert oldfiles[manifest_name]==newfiles[manifest_name]
embedded=json.loads(oldfiles[manifest_name]);assert len(embedded['files'])==122 and embedded['self_excluded']
for item in embedded['files']:
 name='726-delegate-web-map/'+item['path'];raw=oldfiles[name];assert (h(raw),len(raw))==(item['sha256'],item['bytes'])
assert set(oldfiles)=={manifest_name}|{'726-delegate-web-map/'+x['path'] for x in embedded['files']}
print(json.dumps({'result':'verified_with_one_additional_false_positive_and_tar_encoding_qualification','counts':{'entries':186,'files':file_count,'directories':dir_count,'changed_files':changed,'gzip_file_members':gz_count,'redactions':redactions,'retained_prose_occurrences':len(restored),'redacted_nonpath_regex_occurrences':len(regex_source),'other_path_pattern_matches':other_matches,'opaque_prior_replacements':count,'damaged_public_tar_readable_entries':len(damaged_members),'embedded_original_manifest_entries':len(embedded['files'])},'tarinfo_difference_counts':dict(meta_diffs),'pax_path_addition_members':pax_changed,'original_gzip_os_byte':original[9],'reconstructed_gzip_os_byte':public[9],'retained_prose':restored,'additional_redacted_regex_source':regex_source,'damaged_next_header_offset':next_offset,'damaged_next_header_error':'bad checksum','qualification':'Evidence bytes only; retained original copy does not establish independent custody, source-reading, checkpoint3821, or product behavior.'},indent=2))
