#!/usr/bin/env python3
"""Read-only verification of public 059 match representations and legacy tar scope."""
import argparse,collections,gzip,hashlib,io,json,pathlib,tarfile
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--root',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parent,help='Publication root containing public checkpoint indexes and archives.')
parser.add_argument('--stage',action='append',default=[],metavar='CHECKPOINT=PUBLICATION_DIRECTORY',help='Optional stage-directory override for checkpoint 36, 37 or 38 before overlay.')
args=parser.parse_args();here=pathlib.Path(__file__).resolve().parent;root=args.root.resolve();overrides={}
for spec in args.stage:
 number,sep,path=spec.partition('=');assert sep and number in ('36','37','38') and number not in overrides;overrides[number]=pathlib.Path(path).resolve()
sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();load=lambda p:json.loads(p.read_bytes());package=load(here/'TRANSPORT-MATCH-059-1-38-MANIFEST.json');expected={'TRANSPORT-MATCH-059-1-38.json','TRANSPORT-MATCH-059-1-38-OCCURRENCES.json.gz','TRANSPORT-MATCH-059-1-38-LEGACY-OPAQUE.json','TRANSPORT-MATCH-059-1-38-README.md','TRANSPORT-MATCH-059-1-38-VERIFICATION.json','TRANSPORT-MATCH-059-1-38-PRIVACY.json','verify_transport_match_059_1_38.py'};assert len(package['files'])==len(expected) and {x['path'] for x in package['files']}==expected
for x in package['files']:
 content=(here/x['path']).read_bytes();assert (len(content),sha(content))==(x['bytes'],x['sha256'])
report=load(here/'TRANSPORT-MATCH-059-1-38.json');op=here/report['summary']['indexed_occurrences_original_and_public_evidence'];oz=op.read_bytes();raw=gzip.decompress(oz);assert (sha(oz),sha(raw))==(report['summary']['indexed_occurrences_gzip_sha256'],report['summary']['indexed_occurrences_uncompressed_sha256']);occs=json.loads(raw)['occurrences'];lp=here/report['summary']['legacy_unindexed_content'];assert sha(lp.read_bytes())==report['summary']['legacy_unindexed_sha256'];legacy=load(lp);assert len(occs)==1213 and sum(x['reconstructed_public'] for x in occs)==451
manifest=root/report['original_transport_manifest']['path'];mb=manifest.read_bytes();assert sha(mb)==report['original_transport_manifest']['stored_sha256'];transport=json.loads(gzip.decompress(mb));by=collections.defaultdict(list)
for x in transport['entries']:by[x['sha256']].append(x)
cp={};bases={};rows={x['checkpoint']:x for x in report['checkpoint_counts']};seen=set();groups=collections.defaultdict(list)
changed_groups=collections.defaultdict(list)
for x in occs:
 k=(x['checkpoint'],x['public_archive'],x['member_chain']);assert k not in seen;seen.add(k);groups[(x['checkpoint'],x['public_archive'])].append(x);assert x['original_transport_identities']==by[x['original_uncompressed_sha256']] and not by[x['original_stored_sha256']];assert all(e['bytes']==x['original_uncompressed_bytes'] for e in x['original_transport_identities']);assert x['reconstructed_public']==(x['original_stored_sha256']!=x['public_stored_sha256'])
 if x['reconstructed_public']:changed_groups[x['original_uncompressed_sha256']].append(x)
assert len(changed_groups)==len(report['new_changed_original_059_contents'])==5
for x in report['new_changed_original_059_contents']:
 group=changed_groups[x['uncompressed_sha256']];assert x['new_reconstructed_occurrences']==len(group) and x['original_transport_identities']==by[x['uncompressed_sha256']] and {z['original_stored_sha256'] for z in group}=={x['original_stored_gzip_sha256']};assert {z['public_stored_sha256'] for z in group}=={z['sha256'] for z in x['public_variants']}
def walk(x,path):
 if x.get('entry_type')!='directory':yield path,x
 for c in x.get('members',[]):yield from walk(c,path+'!'+c['path'])
for i in range(1,39):
 r=rows[i];base=overrides.get(str(i),root);bases[i]=base;indexbytes=(base/r['evidence_index']).read_bytes();assert sha(indexbytes)==r['evidence_index_sha256'];c=json.loads(indexbytes);cp[i]=c;stored=0;gzip_extra=0;changedstored=0;changedunion=0;storedids=set();unionids=set();found=[]
 if r['qualification_file']:
  qbytes=(base/r['qualification_file']).read_bytes();assert sha(qbytes)==r['qualification_sha256'];q=json.loads(qbytes)
 for b in c['bundles']:
  for top in b['members']:
   for chain,m in walk(top,top['path']):
    h=m.get('original_sha256');u=m.get('original_uncompressed_sha256');sm=by.get(h,[]);um=by.get(u,[]) if m.get('content_encoding') in ('gzip','gzip-tar') else [];changed=m.get('changed',h!=m.get('published_sha256'))
    if sm:stored+=1
    if um and not sm:gzip_extra+=1
    if changed and (sm or um):
     changedunion+=1
     if sm:changedstored+=1
     storedids.update(z['sha256'] for z in sm);unionids.update(z['sha256'] for z in sm+um)
    if um:
     k=(i,b['archive'],chain);assert k in seen;x=next(z for z in groups[(i,b['archive'])] if z['member_chain']==chain);assert x['public_archive_sha256']==b['archive_sha256'] and x['original_stored_sha256']==h and x['original_uncompressed_sha256']==u and x['public_stored_sha256']==m['published_sha256'] and x['public_uncompressed_sha256']==m['published_uncompressed_sha256'] and x['original_stored_bytes']==m['original_bytes'] and x['public_stored_bytes']==m['published_bytes'];found.append(k)
 if i in (1,2):
  # Legacy indexes have no compression metadata; the two listed unchanged gzip members are checked from archive bytes below.
  old=[x for x in occs if x['checkpoint']==i];assert len(old)==1 and not old[0]['reconstructed_public'];gzip_extra+=1
  b=next(x for x in c['bundles'] if x['archive']==old[0]['public_archive']);m=next(x for x in b['members'] if x['path']==old[0]['member_chain']);assert m['original_sha256']==m['published_sha256']==old[0]['original_stored_sha256'] and b['archive_sha256']==old[0]['public_archive_sha256']
 else:assert len(found)==len([x for x in occs if x['checkpoint']==i])
 assert (stored,gzip_extra,stored+gzip_extra,changedunion,len(unionids))==(r['stored_occurrences'],r['new_uncompressed_only_occurrences'],r['corrected_union_occurrences'],r['corrected_redacted_occurrences'],r['corrected_unique_redacted_original_sha'])
 if i>=10:assert (q['matched_nested_member_occurrences'],q['unique_redacted_original_sha_count'],q['redacted_public_member_occurrences'])==(stored,len(storedids),changedstored)
assert sum(x['stored_occurrences'] for x in rows.values())==107704 and sum(x['new_uncompressed_only_occurrences'] for x in rows.values())==1213 and sum(x['corrected_union_occurrences'] for x in rows.values())==108917
cache={};actual=0;archives=0
for (i,archive),targets in sorted(groups.items()):
 b=next(z for z in cp[i]['bundles'] if z['archive']==archive);z=(bases[i]/archive).read_bytes();assert (len(z),sha(z))==(b['archive_bytes'],b['archive_sha256']);archives+=1;tree={'children':{},'targets':[]};meta={}
 def idx(m,path):
  meta[path]=m
  for c in m.get('members',[]):idx(c,path+'!'+c['path'])
 for m in b['members']:idx(m,m['path'])
 for x in targets:
  node=tree
  for part in x['member_chain'].split('!'):node=node['children'].setdefault(part,{'children':{},'targets':[]})
  node['targets'].append(x)
 def visit(tarbytes,node,prefix=''):
  global actual
  found=set()
  with tarfile.open(fileobj=io.BytesIO(tarbytes),mode='r|*') as t:
   for f in t:
    if f.name not in node['children']:continue
    assert f.isfile();found.add(f.name);v=t.extractfile(f).read();path=(prefix+'!' if prefix else '')+f.name;m=meta[path];assert (len(v),sha(v))==(m['published_bytes'],m['published_sha256']);sub=node['children'][f.name]
    if sub['targets']:
     h=sha(v)
     if h not in cache:
      plain=gzip.decompress(v);cache[h]=(len(plain),sha(plain),blob(plain))
     n,plainsha,gitblob=cache[h]
     for x in sub['targets']:
      assert (len(v),h,n,plainsha)==(x['public_stored_bytes'],x['public_stored_sha256'],x['public_uncompressed_bytes'],x['public_uncompressed_sha256'])
      if not x['reconstructed_public']:
       assert (h,n,plainsha)==(x['original_stored_sha256'],x['original_uncompressed_bytes'],x['original_uncompressed_sha256']);assert all((n,plainsha,gitblob)==(e['bytes'],e['sha256'],e['git_blob']) for e in x['original_transport_identities'])
      actual+=1
    if sub['children']:visit(v,sub,path)
  assert found==set(node['children']),(i,archive,prefix,set(node['children'])-found)
 visit(z,tree)
assert actual==1213 and archives==222
# Early opaque containers were not part of the corrected indexed counts.
for c in legacy['containers']:
 i=c['checkpoint'];b=next(z for z in cp[i]['bundles'] if z['archive']==c['public_archive']);raw=(bases[i]/c['public_archive']).read_bytes();assert sha(raw)==c['public_archive_sha256']==b['archive_sha256'];m=next(z for z in b['members'] if z['path']==c['member']);assert (m['original_sha256'],m['original_bytes'],m['published_sha256'],m['published_bytes'])==(c['original_stored_sha256'],c['original_stored_bytes'],c['public_stored_sha256'],c['public_stored_bytes'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:inner=t.extractfile(c['member']).read()
 assert (len(inner),sha(inner))==(c['public_stored_bytes'],c['public_stored_sha256']);plain=gzip.decompress(inner)
 with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
  entries=t.getmembers();assert len(entries)==c['public_tar_entries_visible'];names={x.name for x in entries}
  if i in (2,5):
   assert c['original_stored_sha256']==c['public_stored_sha256'] and len(entries)==c['original_tar_entries'];found=[]
   for e in entries:
    if not e.isfile():continue
    data=t.extractfile(e).read();sm=by.get(sha(data),[]);un=gzip.decompress(data) if data.startswith(b'\x1f\x8b') else None;um=by.get(sha(un),[]) if un is not None else []
    if sm or um:
     for z in sm:assert (len(data),blob(data))==(z['bytes'],z['git_blob'])
     for z in um:assert (len(un),blob(un))==(z['bytes'],z['git_blob'])
     found.append((e.name,sha(data),sha(un) if un is not None else None))
   expected=[(x['member'],x['original_stored_sha256'],x['original_uncompressed_sha256']) for x in c['original_transport_inner_matches']];assert found==expected
  else:
   assert i==7 and len(entries)==5 and c['original_tar_entries']==186 and len(c['original_transport_inner_matches'])==51;last=entries[-1];off=last.offset_data+((last.size+511)//512)*512;assert off==c['after_last_visible_header_offset']==69120
   try:tarfile.TarInfo.frombuf(plain[off:off+512],'utf-8','surrogateescape')
   except tarfile.HeaderError:pass
   else:raise AssertionError('expected recorded CP7 invalid tar header')
   for x in c['original_transport_inner_matches']:
    assert x['member'] not in names and x['stored_transport_identities']==by[x['original_stored_sha256']] and x['uncompressed_transport_identities']==by.get(x['original_uncompressed_sha256'],[]) and not x['public_accessible_at_review']
assert [(x['checkpoint'],len(x['original_transport_inner_matches'])) for x in legacy['containers']]==[(2,0),(5,10),(7,51)]
c7=next(x for x in legacy['containers'] if x['checkpoint']==7);r7=report['checkpoint7_opaque_exception'];assert (r7['public_archive'],r7['public_archive_sha256'],r7['public_member'],r7['original_stored_sha256'],r7['original_stored_bytes'],r7['public_stored_sha256'],r7['public_stored_bytes'])==(c7['public_archive'],c7['public_archive_sha256'],c7['member'],c7['original_stored_sha256'],c7['original_stored_bytes'],c7['public_stored_sha256'],c7['public_stored_bytes'])
print('PASS: 38 captured public indexes and qualifications; 1,213 uncompressed occurrences in 222 public archives; stored-versus-uncompressed unions, CP5 supplemental content and damaged CP7 public tar. CP7 original entries, inner observations and private custody were not publicly reverified.')
