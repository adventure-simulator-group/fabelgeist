#!/usr/bin/env python3
"""Read-only CP7 successor evidence comparison; no project code is executed.
Requires CP7_ORIGINAL, CP7_PRIOR_REVIEW and CP7_EARLIER_STAGE. The private
original remains local and its bytes are never printed or copied here.
"""
import gzip,hashlib,io,json,os,re,tarfile
from collections import Counter
from pathlib import Path
p=Path(__file__).resolve().parent;e=p/'evidence';h=lambda b:hashlib.sha256(b).hexdigest()
def load(f):return json.loads(f.read_bytes())
if (p/'MANIFEST.json').exists():
 for x in load(p/'MANIFEST.json')['files']:
  data=(p/x['path']).read_bytes();assert (h(data),len(data))==(x['sha256'],x['bytes']),x['path']
reg=load(p/'REGISTRATION.json')
for row in reg['frozen_inputs']:assert h((p/row['path']).read_bytes())==row['sha256']
indexraw=(e/'successor-index.json').read_bytes(); prior_index_raw=(e/'prior-index.json').read_bytes();idx=json.loads(indexraw);prioridx=json.loads(prior_index_raw)
assert h(indexraw)=='32f9312f9f1958ba9a19e82b76069046d92bfda7b1a8d39699809a6b21a1ef03' and h(prior_index_raw)=='5b3a8e01241c0d00d5f8a0220f0964062de6323edda4cf0131924b93630922c0'
new=(e/'successor-archive.tar.gz').read_bytes();assert (h(new),len(new))==(idx['public_sha256'],idx['public_bytes'])==('5b471b148151e841596bc253c45ac0bd23edf5db174d0c251b659a3543e3426e',8233826)
original=Path(os.environ['CP7_ORIGINAL']).read_bytes();assert (h(original),len(original))==(idx['source_original_sha256'],idx['source_original_bytes'])==('e4e4b906817641c33fe3d436ff1db32bf27879fc2dcd4bdc1a69f0487e9cf21c',8247789)
oldroot=Path(os.environ['CP7_PRIOR_REVIEW']);oldmf=(oldroot/'MANIFEST.json').read_bytes();oldreport=(oldroot/'REPORT.json').read_bytes()
assert h(oldmf)=='e5ac543d106e57e071b0c28d04f41c02ea7e9151c1916bdcfab65b22993528e0' and h(oldreport)=='ea1315a604d3744b77f31e81fa07f60ef49d74b4c54c459acc6e69a76d4cdb12'
for row in json.loads(oldmf)['files']:
 data=(oldroot/row['path']).read_bytes();assert (h(data),len(data))==(row['sha256'],row['bytes'])
previous=(oldroot/'evidence/reconstructed.tar.gz').read_bytes();assert h(previous)==prioridx['public_sha256']=='8ef3dc97ed66a6b20c08456945780b6a01295407d2974e9cf86f68076d5a5d93'
assert (oldroot/'evidence/INDEX.json').read_bytes()==prior_index_raw
stage=Path(os.environ['CP7_EARLIER_STAGE']);assert (stage/'INDEX.json').read_bytes()==prior_index_raw and (stage/'prior-web-map-route-reconstructed.tar.gz').read_bytes()==previous
cp=load(oldroot/'evidence/CHECKPOINT-7-INDEX.json');bundle=next(x for x in cp['bundles'] if x['bundle_id']=='strategic-map');m=next(x for x in bundle['members'] if x['path']=='prior-web-map-route-packet.tar.gz')
assert (m['original_sha256'],m['original_bytes'],m['published_sha256'])==(h(original),len(original),idx['previous_public_sha256'])
assert h(gzip.decompress(original))==m['original_uncompressed_sha256']
assert idx['source_original_sha256']==prioridx['source_original_sha256'] and idx['previous_public_sha256']==prioridx['previous_public_sha256']
assert 'GNU' in idx['tar_encoding_qualification'] and 'PAX' in idx['tar_encoding_qualification'] and '12' in idx['tar_encoding_qualification'] and 'raw headers' in idx['tar_encoding_qualification']
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/' rb'726-[^"\'\s]+',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
allowed={('726-delegate-web-map/build-manifest.py',213,b"rb'",b'/home/[A-Za-z0-9_.-]+/'),('726-delegate-web-map/historical-774-variant/producer-supplement-after-owner/root/traces.jsonl',188293,b'garden',b'/home/building'),('726-delegate-web-map/original-059-ledger.json.gz',4460781,b'source',b'/home/window/viewport/overlay')}
assert {(r['path'],r['original_uncompressed_offset'],r['preceding_prose'].encode(),r['restored_slash_text'].encode()) for r in idx['non_path_matches_retained']}==allowed
assert len(idx['entries'])==len(prioridx['entries'])==186
changed_rows=[]
for a,b in zip(prioridx['entries'],idx['entries']):
 assert a['path']==b['path'];delta={k for k in set(a)|set(b) if a.get(k)!=b.get(k)}
 if delta:changed_rows.append((a['path'],delta))
assert changed_rows==[('726-delegate-web-map/build-manifest.py',{'public_sha256','public_uncompressed_sha256','public_bytes','redacted_path_occurrences'})]
counts=Counter();meta=Counter();restored=[];pax=[];changed_bodies=[];orig_files={};orig_tar=gzip.decompress(original);new_tar=gzip.decompress(new)
with tarfile.open(fileobj=io.BytesIO(original),mode='r:gz') as a,tarfile.open(fileobj=io.BytesIO(previous),mode='r:gz') as b,tarfile.open(fileobj=io.BytesIO(new),mode='r:gz') as c:
 aa=a.getmembers();bb=b.getmembers();cc=c.getmembers();assert len(aa)==len(bb)==len(cc)==186 and len({x.name for x in aa})==186
 for x,y,z,row in zip(aa,bb,cc,idx['entries']):
  assert x.name==y.name==z.name==row['path'] and not x.name.startswith('/') and '..' not in Path(x.name).parts
  assert (x.isfile() and y.isfile() and z.isfile() and row['kind']=='file') or (x.isdir() and y.isdir() and z.isdir() and row['kind']=='directory')
  assert all(getattr(x,k)==getattr(z,k) for k in ['name','type','mode','uid','gid','uname','gname','mtime','linkname','devmajor','devminor','sparse'])
  for k in ['size','chksum','offset','offset_data','pax_headers']:
   if getattr(x,k)!=getattr(z,k):meta[k]+=1
  if x.pax_headers!=z.pax_headers:
   assert x.pax_headers=={} and z.pax_headers=={'path':x.name};pax.append(x.name)
  assert orig_tar[x.offset_data-512+257:x.offset_data-512+265]==b'ustar  \x00' and new_tar[z.offset_data-512+257:z.offset_data-512+265]==b'ustar\x0000'
  for val in [z.name,z.uname,z.gname,z.linkname,*z.pax_headers.keys(),*z.pax_headers.values()]:
   assert not local.search(str(val).encode()) and not secret.search(str(val).encode())
  if x.isdir():counts['directories']+=1;continue
  counts['files']+=1;ob=a.extractfile(x).read();pb=b.extractfile(y).read();nb=c.extractfile(z).read();orig_files[x.name]=ob
  assert (h(ob),len(ob))==(row['original_sha256'],row['original_bytes']) and (h(nb),len(nb))==(row['public_sha256'],row['public_bytes'])
  assert row['changed']==(ob!=nb);counts['changed_against_original']+=row['changed']
  if pb!=nb:changed_bodies.append(x.name)
  enc=row['encoding'];assert enc in ('identity','gzip');counts['gzip_members']+=enc=='gzip'
  o=gzip.decompress(ob) if enc=='gzip' else ob;n=gzip.decompress(nb) if enc=='gzip' else nb
  assert h(o)==row['original_uncompressed_sha256'] and h(n)==row['public_uncompressed_sha256'] and not secret.search(n)
  pieces=[];last=0;mapping={};nred=0
  for hit in local.finditer(o):
   start=hit.start();val=hit.group();pieces.append(o[last:start]);matches=[q for q in allowed if q[0]==x.name and q[1]==start and q[3]==val and o[max(0,start-len(q[2])):start]==q[2]]
   if matches:
    assert len(matches)==1;pieces.append(val);restored.append((x.name,start,matches[0][2].decode(),val.decode()))
   else:
    if val not in mapping:mapping[val]=len(mapping)
    pieces.append(f'<LOCAL_PATH_{mapping[val]}>'.encode());nred+=1
   last=hit.end()
  pieces.append(o[last:]);plain=b''.join(pieces);assert plain==n and nred==row['redacted_path_occurrences'];counts['redactions']+=nred
  exp=gzip.compress(plain,mtime=0) if enc=='gzip' and plain!=o else ob if plain==o else plain;assert exp==nb
  # No additional path-pattern match is admitted outside the three exact
  # member/value/context exceptions. Tokens for actual paths remain intact.
  for hit in local.finditer(n):
   start=hit.start();candidate=[q for q in allowed if q[0]==x.name and q[3]==hit.group() and n[max(0,start-len(q[2])):start]==q[2]];assert len(candidate)==1
  if x.name=='726-delegate-web-map/build-manifest.py':
   assert o.splitlines()[6]==b" 'absolute_home_path':re.compile(rb'/home/[A-Za-z0-9_.-]+/'),"
   assert n.splitlines()[6]==o.splitlines()[6]
   assert pb.count(b'<LOCAL_PATH_1>')==1 and pb.replace(b'<LOCAL_PATH_1>',b'/home/[A-Za-z0-9_.-]+/')==nb
assert counts=={'files':123,'directories':63,'changed_against_original':8,'gzip_members':52,'redactions':1084}
assert set(restored)=={(q[0],q[1],q[2].decode(),q[3].decode()) for q in allowed} and len(restored)==3
assert changed_bodies==['726-delegate-web-map/build-manifest.py']
assert meta=={'chksum':186,'size':8,'pax_headers':12,'offset':86,'offset_data':86} and len(pax)==12 and original[9]==3 and new[9]==255
manifest='726-delegate-web-map/MANIFEST.json';embedded=json.loads(orig_files[manifest]);assert len(embedded['files'])==122
for row in embedded['files']:
 item=orig_files['726-delegate-web-map/'+row['path']];assert (h(item),len(item))==(row['sha256'],row['bytes'])
print(json.dumps({'result':'pass_with_explicit_privacy_and_custody_limits','original_sha256':h(original),'previous_reviewed_archive_sha256':h(previous),'successor_archive_sha256':h(new),'successor_index_sha256':h(indexraw),'prior_review_report_sha256':h(oldreport),'prior_review_manifest_sha256':h(oldmf),'counts':dict(counts),'entries':186,'retained_nonpath_contexts':restored,'previous_to_successor_changed_member_bodies':changed_bodies,'tarinfo_difference_counts':dict(meta),'pax_long_path_additions':len(pax),'raw_header_qualification':'GNU original; POSIX ustar/PAX successor; changed raw checksums/offsets and gzip OS byte are not original-byte preservation.','privacy_qualification':'No unclassified path-pattern or configured credential-pattern match in decoded member bodies and metadata, after three precise context exceptions. Other encodings and secrets outside the configured patterns are not exhaustively covered.','historic_scope':'Same-machine exact original; no product execution, newly observed source read credit, checkpoint3821 promotion, or custody inference.'},indent=2))
