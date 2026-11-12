#!/usr/bin/env python3
"""Read-only verification of the public evidence transport; no private inputs."""
import collections,gzip,hashlib,io,json,pathlib,re,tarfile
p=pathlib.Path(__file__).parent;sha=lambda b:hashlib.sha256(b).hexdigest();get=lambda n:(p/n).read_bytes();load=lambda n:json.loads(get(n));m=load('PUBLIC-MANIFEST.json');files={f.relative_to(p).as_posix() for f in p.rglob('*') if f.is_file() and f.relative_to(p).as_posix()!='PUBLIC-MANIFEST.json'};assert m['file_count']==len(m['files']) and files=={x['path'] for x in m['files']}
for x in m['files']:
 b=get(x['path']);assert (len(b),sha(b))==(x['bytes'],x['sha256'])
map=load('transport-map.json');rows=map['files'];assert len(rows)==map['file_count']==69 and len({x['public_transport_path'] for x in rows})==69
assert {x['public_transport_path'] for x in rows}=={x for x in files if x.startswith('evidence/')}
redactions=0;packed=0
for x in rows:
 b=get(x['public_transport_path']);assert (len(b),sha(b))==(x['public_stored_bytes'],x['public_stored_sha256']);trans=x['transformations']
 if not trans:assert (len(b),sha(b))==(x['source_original_bytes'],x['source_original_sha256'])
 for t in trans:
  if t['kind']=='replace-local-scratch-token':
   assert len(t['original_value_sha256'])==t['occurrences'] and b.count(t['public_marker'].encode())>=t['occurrences'] and not re.search(rb'/tmp/'+rb'726-[A-Za-z0-9._-]+',b);assert sha(b)!=x['source_original_sha256'];redactions+=t['occurrences']
  elif t['kind']=='gzip-transport':
   raw=gzip.decompress(b);assert (len(raw),sha(raw))==(t['decoded_bytes'],t['decoded_sha256'])==(x['source_original_bytes'],x['source_original_sha256']);packed+=1
  else:raise AssertionError('unexpected transformation')
assert redactions==map['local_scratch_token_redactions']==11 and packed==map['gzip_transport_count']==5
for label,digest in map['source_packet_manifests_sha256'].items():
 original=next(x for x in rows if x['source_packet']==label and x['source_logical_path']=='BUNDLE-MANIFEST.json');assert original['source_original_sha256']==digest and original['public_stored_sha256']==digest
index=load('evidence/json-candidates/reconstruction-index.json');record_count=0;marker=index['redaction_marker'];pat=re.compile(rb'(?<![A-Za-z0-9_/])/(?:home|Users|tmp)/[A-Za-z0-9._-]+|(?i:(?<![A-Za-z0-9])[a-z]:[\\/]+Users[\\/]+[A-Za-z0-9._-]+)|(?<![A-Za-z0-9_/])/root/(?:\.codex|\.cache|\.local|tmp)(?:/|$)')
for x in index['records']:
 row=next(y for y in rows if y['source_packet']=='json-candidates' and y['source_logical_path']==x['public_candidate']);b=gzip.decompress(get(row['public_transport_path']));assert (len(b),sha(b))==(x['public_candidate_bytes'],x['public_candidate_sha256']);v=json.loads(b);norm=re.sub(rb'(?i)\\u002f',b'/',b.replace(b'\\/',b'/'));assert not pat.search(norm)
 for f in x['redacted_fields']:
  q=v
  for k in f['pointer'].strip('/').split('/'):
   k=k.replace('~1','/').replace('~0','~');q=q[int(k)] if isinstance(q,list) else q[k]
  assert q==marker;record_count+=1
assert record_count==56
extra=load('evidence/extras/reconstruction-index.json');assert len(extra['records'])==5 and sum(x['redaction_occurrences'] for x in extra['records'])==8
for x in extra['records']:
 b=get('evidence/extras/'+x['candidate']);assert sha(b)==x.get('candidate_sha256',x.get('candidate_stored_sha256'))
 if x['kind']=='historical-registration':
  d=gzip.decompress(b);assert sha(d)==x['candidate_decoded_sha256'];value=json.loads(d)['provenance']['cache_path'];assert not re.search(r'(?i)[a-z]:[\\/]+Users[\\/]+[A-Za-z0-9._-]+',value)
 if x['kind']=='native-archive-reconstruction':
  with tarfile.open(fileobj=io.BytesIO(b),mode='r:*') as t:
   members=t.getmembers();assert len(members)==x['member_count'];member=t.extractfile(x['changed_member']).read();assert sha(member)==x['candidate_member_sha256'] and not pat.search(member)
  assert x['original_sha256']!=x['candidate_sha256']
report=load('REPORT.json');proof=load('evidence/inventory/native/registry-identity.json');official=load('evidence/inventory/native/official-index-web-row.json');local=load('evidence/inventory/native/local-index-row.json');assert official==local and official['name']=='serde-saphyr' and official['vers']=='1.2.0' and official['cksum']==report['registry_identity']['original_package_sha256']==proof['original_archive']['sha256']==proof['historical_lock']['checksum']==proof['current_pinned_lock']['checksum'];assert report['registry_identity']['reconstructed_package_sha256']==proof['reconstructed_archive']['sha256'] and proof['reconstructed_archive']['registry_checksum_matches'] is False
assert proof['current_pinned_lock']['revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and proof['current_pinned_lock']['git_blob']=='8cee024c538ebc473ef64c709eaba8c585fa4c55'
check=load('bounded-path-check.json');assert check['private_token_hits']==0 and check['exceptions']==0 and check['broad_candidate_count']==1 and check['broad_candidates'][0]['type']=='temp' and check['broad_candidates'][0]['chain'][-1]=='serde-saphyr-1.2.0/examples/spaced_yaml.rs'
print('PASS: public manifest, 69 transport mappings, 11 scratch redactions, five gzip JSON candidates (56 markers), eight further redactions, package/lock identity metadata; no original private inputs used')
