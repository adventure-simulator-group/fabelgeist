#!/usr/bin/env python3
"""Read-only cross-packet inventory. Never emit original values or their digests.

Pass paths as arguments. Output only locations, counts and booleans; path-value
hashes and raw private values are maintained only in process memory. This is an
information-limited scan, not a proof that all encodings or private values were
identified. Does not import, execute or change evidence scripts or product code.
"""
import argparse, ast, collections, gzip, hashlib, io, json, pathlib, re, subprocess, tarfile

p=argparse.ArgumentParser()
for k in ('stage40','stage42','private40','private42','publication','collider-builder'):
 p.add_argument('--'+k,required=True,type=pathlib.Path)
p.add_argument('--commit',required=True)
p.add_argument('--out',required=True,type=pathlib.Path)
p.add_argument('--successor',required=True,type=pathlib.Path)
a=p.parse_args()
H=lambda x:hashlib.sha256(x).hexdigest()
EXPECTED40='aa2f8364932cc2bc3ca00e9d71501392dd301002d7ea1dfd9964d4aa8a6dd413'
EXPECTED42='c6a849084f746adbcc392e46937928b624dbec52f8626aba3f6464527361e8b0'
EXPECTED_P40='57ab22c864cfc41e3404e6dc5cdea4631961e817a8ced736fd8a8d23d2e0139e'
EXPECTED_P42='da8dc1d5c5d06da19ceb72fd92b1053a627bd534162333b4c84a855d86af3d36'

def read_file(p):
 assert p.is_file() and not p.is_symlink()
 return p.read_bytes()
def unpack_archive(p):
 with tarfile.open(p,mode='r:gz') as t:
  m=t.getmembers();names=[v.name for v in m];assert len(names)==len(set(names)) and all(x.isfile() for x in m)
  return {x.name:t.extractfile(x).read() for x in m}
def dec(raw):return gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw

def verify_stage(root, expected):
 b=read_file(root/'STAGE-MANIFEST.json');assert H(b)==expected
 j=json.loads(b); rows={x['path']:x for x in j['files']};assert len(rows)==len(j['files'])
 files={x.relative_to(root).as_posix() for x in root.rglob('*') if x.is_file() or x.is_symlink()}-{'STAGE-MANIFEST.json'}
 assert set(rows)==files
 for n,x in rows.items():
  b=read_file(root/n);assert (H(b),len(b))==(x['sha256'],x['bytes'])
 c=json.loads(read_file(root/'CATALOG.json'))
 for x in c['packets']:
  raw=read_file(root/x['archive']);assert (H(raw),len(raw))==(x['archive_sha256'],x['archive_bytes'])
 return c

def jptr(x,pointer):
 for k in pointer.split('/')[1:]:
  k=k.replace('~1','/').replace('~0','~');x=x[int(k)] if isinstance(x,list) else x[k]
 return x

c40=verify_stage(a.stage40,EXPECTED40);c42=verify_stage(a.stage42,EXPECTED42)
assert H(read_file(a.private40/'MANIFEST.json'))==EXPECTED_P40
assert H(read_file(a.private42/'MANIFEST.json'))==EXPECTED_P42
A40={x['id']:unpack_archive(a.stage40/x['archive']) for x in c40['packets']}
A42={x['id']:unpack_archive(a.stage42/x['archive']) for x in c42['packets']}
assert len(A40)==8 and len(A42)==3
# Inputs are scoped by original packet and logical member, not by later role or revision.
redactions=collections.Counter(); sources=collections.defaultdict(set); sensitive_values=set()
# Lookup maps the hexadecimal encoding of a digest to group/representation tags.
# No key of this lookup is serialized or printed.
digest_index=collections.defaultdict(set)

def add_token(packet,member,token,raw_and_json=False):
 assert isinstance(token,bytes) and token and len(token)<1024*1024
 redactions[packet]+=1;sensitive_values.add(token);sources[token].add((packet,member))
 forms=[('raw',token)]
 # The earlier privacy inventory hashed matched path prefixes, which can
 # reveal an identifying home or temporary directory without being equal
 # to an entire path token. Reconstruct bounded normalized prefix forms.
 norm=token.replace(b'\\/',b'/').replace(b'\\u002f',b'/').replace(b'\\u002F',b'/').replace(b'\\\\',b'\\')
 rx=re.compile(rb'(?<![A-Za-z0-9_/])/(?:home|Users)/[A-Za-z0-9._-]+|(?<![A-Za-z0-9_/])/(?:tmp|private/tmp|var/folders)/[A-Za-z0-9._-]+|(?i:(?<![A-Za-z0-9])(?:[a-z]:[\\/]+Users[\\/]+)[A-Za-z0-9._-]+)|(?i:(?<![A-Za-z0-9])[a-z]:[\\/]+[A-Za-z0-9._-]+)')
 for m in rx.finditer(norm):forms.append(('path-prefix',m.group()))
 if raw_and_json:
  try:
   s=token.decode('utf-8'); forms += [('json-string',json.dumps(s,ensure_ascii=False).encode()),('json-ascii',json.dumps(s,ensure_ascii=True).encode())]
  except UnicodeDecodeError:pass
 for form,b in forms:
  for alg in ('sha256','sha1','md5'):
   digest_index[hashlib.new(alg,b).hexdigest().encode()].add((packet,member,alg,form))

def assert_id(raw,row):
 assert (H(raw),len(raw))==(row['original_stored_sha256'],row['original_stored_bytes'])
 assert (H(dec(raw)),len(dec(raw)))==(row['original_decoded_sha256'],row['original_decoded_bytes'])

# Five offset-bearing public identity/index families.
for tag,root,group,idxname in [
 ('garment-pattern',a.private40,'garment-pattern','EVIDENCE-IDENTITIES.json'),
 ('hair-head',a.private40,'hair-head','EVIDENCE-IDENTITIES.json'),
 ('review-13l',a.private40,'central-13l-review','EVIDENCE-IDENTITIES.json'),
 ('hair-surface-grid',a.private42,'hair-surface-grid','EVIDENCE-IDENTITIES.json'),
 ('shell-mesh',a.private42,'shell-mesh','INDEX.json')]:
 packet=(A42 if tag in A42 else A40)[tag]; index=json.loads(packet[idxname])
 for row in index['members']:
  ops=row['redactions']
  if not ops:continue
  name=row.get('source_path',row.get('path'))
  if tag in ('garment-pattern','shell-mesh'):
   where='changed-originals' if tag=='garment-pattern' else 'originals';q=root/group/where/row['original_stored_sha256']
  else:q=root/group/'originals'/name
  raw=read_file(q);assert_id(raw,row);before=dec(raw);public=dec(packet[row['public_path']]);out=[];last=0
  for op in ops:
   start=op['original_decoded_offset_0based'];end=start+op['original_decoded_bytes'];assert start>=last
   token=before[start:end];out.extend((before[last:start],op['public_placeholder'].encode()));last=end
   add_token(tag,name,token,True)
  out.append(before[last:]);assert b''.join(out)==public

# Two cloth file-local same-named markers, proven individually by exact byte replay.
cloth=A40['cloth'];ci=json.loads(cloth['INDEX.json']);cloth_tokens=[]
for row in ci['files']:
 if not row['edits']:continue
 assert len(row['edits'])==1
 old=read_file(a.private40/'cloth/original'/row['path']);new=cloth['reconstructed/'+row['path']]
 assert (H(old),len(old),H(new),len(new))==(row['original_stored_sha256'],row['original_stored_bytes'],row['public_stored_sha256'],row['public_stored_bytes'])
 marker=row['edits'][0]['marker'].encode();assert new.count(marker)==1
 head,tail=new.split(marker);assert old.startswith(head) and old.endswith(tail)
 token=old[len(head):len(old)-len(tail) if tail else len(old)];assert old==head+token+tail
 add_token('cloth',row['path'],token,True);cloth_tokens.append(token)
assert len(cloth_tokens)==2 and cloth_tokens[0]!=cloth_tokens[1]

# Hair-style uses a separately retained mapping. Original and reconstructed
# files are replayed; only values actually present in changed members count.
style=A40['hair-style'];mi=json.loads(read_file(a.private40/'hair-style/MAPPING.json'))['mapping'];mi={k.encode():v.encode() for k,v in mi.items()}
for row in json.loads(style['INDEX.json'])['files']:
 if not row['edits']:continue
 old=read_file(a.private40/'hair-style/original'/row['path']);new=style['reconstructed/'+row['path']]
 assert (H(old),len(old),H(new),len(new))==(row['original_stored_sha256'],row['original_stored_bytes'],row['public_stored_sha256'],row['public_stored_bytes'])
 result=old
 for marker,token in sorted(mi.items(),key=lambda x:len(x[1]),reverse=True):
  n=result.count(token)
  for _ in range(n):add_token('hair-style',row['path'],token,True)
  result=result.replace(token,marker)
 assert result==new

# GPU field-only and literal substitutions, including their declared original
# value hash semantics. Verify each token against its map in memory.
gpu=A40['gpu-reflection'];ogpu=unpack_archive(a.private40/'gpu-reflection/exact-originals.tar.gz');gm=json.loads(gpu['RECONSTRUCTION-MAP.json']);gpu_value_entries=0
for row in gm['records']:
 old=ogpu[row['original_path']];new=gpu[row['public_path']];assert (H(old),len(old),H(new),len(new))==(row['original_stored_sha256'],row['original_stored_bytes'],row['public_stored_sha256'],row['public_stored_bytes'])
 for tr in row['transformations']:
  if tr['kind']=='path-literal-redaction':
   marker=tr['public_marker'].encode();pieces=new.split(marker);assert len(pieces)-1==tr['count'] and old.startswith(pieces[0]);pos=len(pieces[0]);hashes=[]
   for piece in pieces[1:]:
    end=old.find(piece,pos) if piece else len(old);assert end>=pos;token=old[pos:end];hashes.append(H(token));add_token('gpu-reflection',row['original_path'],token,True);pos=end+len(piece)
   assert pos==len(old) and sorted(hashes)==sorted(tr['original_value_sha256']);gpu_value_entries+=len(hashes)
  else:
   assert tr['kind'] in ('existing-public-variant','reconstructed-gzip-transport','json-path-redaction')
   oldobj=json.loads(dec(old));newobj=json.loads(dec(new));assert len(tr['fields'])==tr['count']
   for field in tr['fields']:
    ov=jptr(oldobj,field['pointer']);pv=jptr(newobj,field['pointer']);assert isinstance(ov,str) and ov!=pv
    if tr['kind']=='existing-public-variant':assert H(json.dumps(ov,ensure_ascii=False).encode())==field['original_value_sha256']
    else:assert H(ov.encode())==field['original_value_sha256']
    add_token('gpu-reflection',row['original_path'],ov.encode(),True);gpu_value_entries+=1
assert gpu_value_entries==47

# Hair-collider's frozen original/public comparison recovers removed path
# tokens from the two changed scripts; source is inspected as text only.
# The public map says three substitutions. Matching that count is a guard,
# not a claim that an unrestricted regex recognizes every possible path.
path_rx=re.compile(rb"(?<![A-Za-z0-9_/])(?:/(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s\"\x27<>\x00\\]+|[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s\"\x27<>\x00]+)",re.I)
oldcoll=unpack_archive(a.private42/'hair-collider/frozen-originals.tar.gz');pubcoll=A42['hair-collider'];transcount=0
for row in json.loads(pubcoll['TRANSPORT-MAP.json'])['entries']:
 if not row['transformation'].startswith('local scratch path substitution'):continue
 old=oldcoll[row['original_path']];new=pubcoll[row['public_path']];assert (H(old),len(old),H(new),len(new))==(row['original_sha256'],row['original_bytes'],row['public_sha256'],row['public_bytes'])
 for match in path_rx.finditer(old):
  token=match.group()
  if token not in new:add_token('hair-collider',row['original_path'],token,True);transcount+=1
assert transcount==3
assert dict(redactions)=={'garment-pattern':71,'hair-head':49,'review-13l':2,'hair-surface-grid':57,'shell-mesh':16,'cloth':2,'hair-style':7,'gpu-reflection':47,'hair-collider':3}

DIGEST=re.compile(rb'(?<![0-9A-Fa-f])(?:[0-9A-Fa-f]{64}|[0-9A-Fa-f]{40}|[0-9A-Fa-f]{32})(?![0-9A-Fa-f])')
SAFEKEY=re.compile(r'(?i)(?:[0-9a-f]{32,64}|(?:^|[\\/])(?:home|users|tmp|workspace|workspaces|private)(?:[\\/]|$))')
def pointer(p,k):
 k=str(k)
 if SAFEKEY.search(k):k='<redacted-key>'
 return p+'/'+k.replace('~','~0').replace('/','~1')
def scan_json_strings(x,path=''):
 if isinstance(x,dict):
  for k,v in x.items():
   # Keys can themselves contain a digest; do not emit the key itself.
   if isinstance(k,str):
    for m in DIGEST.finditer(k.encode('utf-8')):
     d=m.group().lower()
     if d in digest_index:yield path+'/<redacted-key>',d
   yield from scan_json_strings(v,pointer(path,k))
 elif isinstance(x,list):
  for i,v in enumerate(x):yield from scan_json_strings(v,pointer(path,i))
 elif isinstance(x,str):
  for m in DIGEST.finditer(x.encode('utf-8')):
   d=m.group().lower()
   if d in digest_index:yield path,d

stage_hits=[];old_hits=[];stats=collections.Counter();older_shared=set();stage_shared=set();errors=[]
MAX_DECOMPRESSED=256*1024*1024

def inspect_view(data,where,which,depth=0):
 stats[which+'_views']+=1;stats[which+'_bytes']+=len(data)
 # Recursively handle gzip and tar members, avoiding a second logical count of
 # the entire tar stream. Record limits rather than silently dropping a view.
 if data.startswith(b'\x1f\x8b'):
  stats[which+'_gzip']+=1
  try:
   f=gzip.GzipFile(fileobj=io.BytesIO(data));plain=f.read(MAX_DECOMPRESSED+1)
   if len(plain)>MAX_DECOMPRESSED:errors.append((which,where,'gzip decoded size limit'));return
   inspect_view(plain,where+' [gzip decoded]',which,depth+1)
  except (OSError,EOFError):errors.append((which,where,'gzip decode failure'))
  return
 if len(data)>262 and data[257:262]==b'ustar':
  if depth>=5:errors.append((which,where,'archive nesting limit'));return
  stats[which+'_tar']+=1
  try:
   with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as t:
    for x in t:
     if not x.isfile():continue
     if x.size>MAX_DECOMPRESSED:errors.append((which,where+'!<large-member>','member size limit'));continue
     name=x.name
     if name.startswith('/') or '..' in pathlib.PurePosixPath(name).parts:name='<unsafe-member-name>'
     elif SAFEKEY.search(name):name='<possibly-private-member-name>'
     inspect_view(t.extractfile(x).read(),where+'!'+name,which,depth+1)
  except (tarfile.TarError,EOFError):errors.append((which,where,'tar decode failure'))
  return
 if data.startswith(b'PK\x03\x04') or data.startswith(b'\x28\xb5\x2f\xfd'):
  stats[which+'_unsupported_compressed']+=1;errors.append((which,where,'zip or zstd not decoded'))
 matches=[m for m in DIGEST.finditer(data) if m.group().lower() in digest_index]
 if not matches:return
 found=[]
 try:
  value=json.loads(data);found=list(scan_json_strings(value))
 except (ValueError,UnicodeDecodeError,RecursionError):pass
 if not found:
  for m in matches:
   d=m.group().lower();found.append(('line:'+str(data.count(b'\n',0,m.start())+1),d))
 assert len(found)==len(matches) or len(found)>=1
 for loc,d in found:
  refs=digest_index[d];groups=sorted({v[0] for v in refs});algs=sorted({v[2] for v in refs});forms=sorted({v[3] for v in refs})
  # Sensitive identifier stays in process memory only, allowing old/public
  # overlap to be assessed after all scans have finished.
  rec={'location':where,'field':loc,'source_packets':groups,'algorithms':algs,'forms':forms,'_key':d}
  if which=='stage':stage_hits.append(rec);stage_shared.add(d)
  else:old_hits.append(rec);older_shared.add(d)


# Independent successor scan, with the union of recovered value sources from
# both stages. Identifiers never leave process memory.
root=a.successor;expected='5dfe0bfe427cf9089f636ac8e52ae1a7e479e5ce0372e2fa8d884cd3f6c07e15';assert H(read_file(root/'STAGE-MANIFEST.json'))==expected
raw_values={bytes.fromhex(k.decode()) for k in digest_index}; rawrx=re.compile(b'|'.join(re.escape(x) for x in sorted(raw_values,key=len,reverse=True)),re.DOTALL);valuerx=re.compile(b'|'.join(re.escape(x) for x in sorted(sensitive_values,key=len,reverse=True)),re.DOTALL)
records=[];errors2=[];sc=collections.Counter();cache={}
def data_search(b,where):
 sc['views']+=1;sc['bytes']+=len(b);k=hashlib.sha256(b).digest()
 if k not in cache:
  hexc=sum(1 for m in DIGEST.finditer(b) if m.group().lower() in digest_index);rawc=len(rawrx.findall(b));valc=len(valuerx.findall(b));cache[k]=(hexc,rawc,valc)
 else:sc['duplicate_views']+=1
 vals=cache[k]
 if any(vals):records.append({'location':where,'hex_digest_occurrences':vals[0],'raw_digest_occurrences':vals[1],'private_value_occurrences':vals[2]})
def visit(b,where,depth=0):
 if len(b)>256*1024*1024:errors2.append({'location':where,'reason':'view size limit'});return
 if depth>8:errors2.append({'location':where,'reason':'nesting limit'});return
 data_search(b,where)
 if b.startswith(b'\x1f\x8b'):
  try:
   x=gzip.GzipFile(fileobj=io.BytesIO(b)).read(256*1024*1024+1)
   if len(x)>256*1024*1024:errors2.append({'location':where,'reason':'gzip size limit'});return
   visit(x,where+' [gzip decoded]',depth+1)
  except (OSError,EOFError) as e:errors2.append({'location':where,'reason':'gzip decode error'})
  return
 if len(b)>262 and b[257:262]==b'ustar':
  try:
   with tarfile.open(fileobj=io.BytesIO(b),mode='r:') as t:
    for x in t:
     for label,v in [('name',x.name),('linkname',x.linkname),('uname',x.uname),('gname',x.gname),*[(str(k),str(v)) for k,v in x.pax_headers.items()]]:data_search(v.encode('utf-8','replace'),where+' [tar metadata '+label+']')
     if x.isfile():
      if x.size>256*1024*1024:errors2.append({'location':where+'!<member>','reason':'tar size limit'});continue
      name=x.name if not pathlib.PurePosixPath(x.name).is_absolute() and '..' not in pathlib.PurePosixPath(x.name).parts else '<unsafe-member>'
      if any(z in name.encode('utf-8','replace') for z in sensitive_values):name='<private-member>'
      visit(t.extractfile(x).read(),where+'!'+name,depth+1)
  except (tarfile.TarError,EOFError) as e:errors2.append({'location':where,'reason':'tar decode error'})
  return
 if b.startswith((b'PK\x03\x04',b'BZh',b'\xfd7zXZ\x00',b'\x28\xb5\x2f\xfd')):errors2.append({'location':where,'reason':'unsupported nested archive'})
for f in sorted(root.rglob('*')):
 if f.is_symlink():errors2.append({'location':'<symlink>','reason':'symlink'});continue
 if f.is_file():visit(f.read_bytes(),f.relative_to(root).as_posix())
assert H(read_file(root/'STAGE-MANIFEST.json'))==expected
result={'schema':'726-successor-independent-digest-scan-v1','stage_manifest_sha256':expected,'recovered_private_value_count':len(sensitive_values),'candidate_digest_encodings':len(digest_index),'findings':records,'exceptions':errors2,'statistics':dict(sc),'qualification':'Search compares 112 independently recovered private values and 681 raw/prefix/JSON encoded MD5, SHA-1 and SHA-256 identifiers, in hexadecimal and raw digest bytes. Gzip/tar are traversed recursively with stated bounds; raw tar, gzip, filenames and metadata are searched too. Unknown values, hashes, encodings and formats remain outside scope. No private values or individual digests are reported; no product code executed.'}
assert not a.out.exists();a.out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'matching_views':len(records),'exceptions':len(errors2),'stats':dict(sc)}))
