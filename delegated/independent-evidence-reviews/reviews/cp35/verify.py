#!/usr/bin/env python3
"""Read-only CP35 evidence verifier. No staged script or product is executed.

Set CHECKPOINT35_STAGE to the retained stage and EXTRACTION_059 to the original
user-supplied extraction to repeat the optional historical comparison.
"""
import gzip,hashlib,io,json,os,re,tarfile
from pathlib import Path
here=Path(__file__).resolve().parent
stage=Path(os.environ['CHECKPOINT35_STAGE'])
pub=stage/'publication';private=stage/'private-originals/checkpoint35'
h=lambda b:hashlib.sha256(b).hexdigest()
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(path):return json.loads(path.read_bytes())
manifest=here/'MANIFEST.json'
if manifest.exists():
 for x in load(manifest)['files']:
  data=(here/x['path']).read_bytes();assert h(data)==x['sha256'] and len(data)==x['bytes'],x['path']
idx=(pub/'DELEGATED-EVIDENCE-CHECKPOINT-35.json').read_bytes();qualraw=(pub/'DELEGATED-CHECKPOINT-35-QUALIFICATION.json').read_bytes();pmraw=(private/'MANIFEST.json').read_bytes()
assert h(idx)=='5e61c0a984e0c9c1cb7b651dc61f0093cefc9304844df4a67605bff27cc49605'
assert h(qualraw)=='90cc44347175b58333ede46edec3375b4ed990b5bbb1c98e7af401c36654db19'
assert h(pmraw)=='3ef1e6c138b93a38c5adb2e3bec72aec806996bb0ff1ca7630d79c7cdb9300f3'
cp=json.loads(idx);qual=json.loads(qualraw);pm=json.loads(pmraw);proposal=load(here/'PROPOSAL.json');ident=load(here/'EVIDENCE-IDENTITIES.json')
assert proposal['index_sha256']==h(idx) and proposal['qualification_sha256']==h(qualraw)
assert (here/'evidence/DELEGATED-EVIDENCE-CHECKPOINT-35.json').read_bytes()==idx and (here/'evidence/DELEGATED-CHECKPOINT-35-QUALIFICATION.json').read_bytes()==qualraw and (here/'evidence/private-originals-MANIFEST.json').read_bytes()==pmraw
transraw=(pub/'recovered/original-transport-manifest.json.gz').read_bytes(); assert h(transraw)==ident['transport_manifest_sha256'];transport=json.loads(gzip.decompress(transraw));trans={x['sha256'] for x in transport['entries']}
priv={(x['bundle_id'],x['path']):x for x in pm['members']};assert len(priv)==len(pm['members'])==37
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/' rb'726-[^"\'\s]+',re.I)
secrets=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
semantic={b'/home/window/viewport/overlay',b'/home/org/conversationequality,',b'/home/requiredclock/attrs/episodes/terminal/die/kinship,',b'/home/noerrors',b'/users/dependencies'}
counts={'top_files':0,'recursive_files':0,'private_originals_checked':0,'substitutions':0,'confirmed_slash_prose_occurrences':0,'other_pathlike_substitutions':0,'stored_transport_matches':0,'uncompressed_transport_matches':0,'union_transport_matches':0,'redacted_transport_matches':0}
seenpriv=set();outrows=[];redacted_originals=set();matched=[]
def untar(data):
 with tarfile.open(fileobj=io.BytesIO(data),mode='r:*') as f:
  items=f.getmembers();assert len(items)==len({x.name for x in items})
  assert all((x.isfile() or x.isdir()) and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in items)
  return {x.name:(f.extractfile(x).read() if x.isfile() else None) for x in items}
def membercheck(body,m,where):
 counts['recursive_files']+=1
 assert h(body)==m['published_sha256'] and len(body)==m['published_bytes'],where
 enc=m['content_encoding'];plain=gzip.decompress(body) if enc.startswith('gzip') else body
 assert h(plain)==m['published_uncompressed_sha256'] and len(plain)==m['published_uncompressed_bytes']
 assert m['changed']==(m['published_sha256']!=m['original_sha256'])
 stored=m['original_sha256'] in trans;unc=m['original_uncompressed_sha256'] in trans
 counts['stored_transport_matches']+=stored;counts['uncompressed_transport_matches']+=unc;counts['union_transport_matches']+=stored or unc
 if stored or unc:
  matched.append({'where':where,'stored_match':stored,'uncompressed_match':unc,'changed':m['changed']})
  if m['changed']:
   counts['redacted_transport_matches']+=1;redacted_originals.add(m['original_uncompressed_sha256'] if unc else m['original_sha256'])
 if not m['changed']:
  assert h(body)==m['original_sha256'] and len(body)==m['original_bytes'] and h(plain)==m['original_uncompressed_sha256'] and len(plain)==m['original_uncompressed_bytes']
 if enc.endswith('tar'):
  items=untar(plain);rows={x['path']:x for x in m['members']};assert set(items)==set(rows)
  for p,b in items.items():
   if b is not None:membercheck(b,rows[p],where+'!'+p)
   else:assert rows[p]['entry_type']=='directory'
 else:assert not local.search(plain) and not secrets.search(plain),where
 return plain
for b in cp['bundles']:
 raw=(pub/b['archive']).read_bytes();assert h(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes']
 entries=untar(raw);ms={m['path']:m for m in b['members']};assert len(ms)==len(b['members']) and set(ms)==set(entries)
 assert entries[b['report_archive_member']]==(pub/b['report_path']).read_bytes()
 assert json.loads(entries[b['report_archive_member']])['author']==b['agent'];counts['top_files']+=len(entries)
 for path,pbody in entries.items():
  assert pbody is not None;m=ms[path];where=b['bundle_id']+'!'+path;pplain=membercheck(pbody,m,where)
  if not m['changed']:continue
  assert 'members' not in m
  key=(b['bundle_id'],path);rec=priv[key];orig=(private/key[0]/key[1]).read_bytes();seenpriv.add(key)
  assert h(orig)==m['original_sha256']==rec['sha256'] and len(orig)==m['original_bytes']==rec['bytes']
  oplain=gzip.decompress(orig) if m['content_encoding'].startswith('gzip') else orig
  assert h(oplain)==m['original_uncompressed_sha256'] and len(oplain)==m['original_uncompressed_bytes']
  matches=list(local.finditer(oplain));unique=list(dict.fromkeys(x.group() for x in matches));transformed=oplain
  for j,s in enumerate(unique):transformed=transformed.replace(s,f'<LOCAL_PATH_{j}>'.encode())
  assert transformed==pplain and len(matches)==m['direct_local_path_redactions']==m['total_local_path_redactions']
  counts['private_originals_checked']+=1;counts['substitutions']+=len(matches)
  for x in matches:
   fragment=x.group()
   if fragment not in semantic:
    # Prefix qualification does not assert the absence of further false positives.
    assert fragment.startswith((b'/home/',b'/tmp/726-',b'/Users/')) or b'\\Users\\' in fragment
    counts['other_pathlike_substitutions']+=1;continue
   counts['confirmed_slash_prose_occurrences']+=1;token=f'<LOCAL_PATH_{unique.index(fragment)}>';off=[k.start() for k in re.finditer(re.escape(token.encode()),pplain)];assert len(off)==1
   outrows.append((b['bundle_id'],path,fragment.decode(),x.start(),oplain.count(b'\n',0,x.start())+1,token,off[0],h(orig),h(oplain),h(pbody),h(pplain)))
assert seenpriv==set(priv)
side=[]
for x in proposal['semantic_redaction_corrections']:
 side.append((x['bundle_id'],x['archive_member'],x['semantic_text_restoration'],x['original_uncompressed_byte_offset_zero_based'],x['original_uncompressed_line_one_based'],x['public_uncompressed_token'],x['public_uncompressed_byte_offset_zero_based'],x['original_stored_sha256'],x['original_uncompressed_sha256'],x['public_stored_sha256'],x['public_uncompressed_sha256']))
assert sorted(side)==sorted(outrows) and len(side)==7
for row in proposal['semantic_redaction_corrections']:
 m=next(x for b in cp['bundles'] if b['bundle_id']==row['bundle_id'] for x in b['members'] if x['path']==row['archive_member'])
 raw=(private/row['bundle_id']/row['archive_member']).read_bytes(); plain=gzip.decompress(raw) if m['content_encoding'].startswith('gzip') else raw
 assert row['safe_context'].encode() in plain
 lines=plain.splitlines();num=row['original_uncompressed_line_one_based'];assert row['semantic_text_restoration'].encode() in lines[num-1]
 if 'CAMERA-ADM03-' in row['historical_record']:
  record=json.loads(lines[num-1]);assert row['historical_record'].split()[-1]==record['id']
 elif 'RIG-ADMISSION03-' in row['historical_record']:
  record=json.loads(lines[num-1]);assert row['historical_record'].split()[-1]==record['id']
 elif row['archive_member'].endswith('059-ledger.json.gz') or row['archive_member']=='reference/ledger.json.gz':
  ledger=json.loads(plain);r=ledger['segovb_continuation']['current_connected_dispositions'][164]
  assert r['scope_revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' and r['receipt']['path']=='segovb/READ-L11-616.json'
  assert row['safe_context']==r['disposition']['acceptance_requirements'][0]
# Independently bind the two 774 original contract files to the f82 evidence
# transport commit. This does not turn f82 into a product source revision.
weapon=next(b for b in cp['bundles'] if b['bundle_id']=='weapon-assets-2591-2592')
wb=untar((pub/weapon['archive']).read_bytes());gp=json.loads(wb['git-proofs.json']);cache={}
def git_object(ident):
 if ident not in cache:
  meta=gp['objects'][ident];raw=gzip.decompress(wb[meta['archive']]);assert h(raw)==meta['sha256'] and len(raw)==meta['bytes'] and oid(meta['type'],raw)==ident;cache[ident]=(meta['type'],raw)
 return cache[ident]
def tree_entries(raw):
 entries={};pos=0
 while pos<len(raw):
  end=raw.index(b'\0',pos);mode,name=raw[pos:end].split(b' ',1);entries[name.decode()]=(mode.decode().lstrip('0'),raw[end+1:end+21].hex());pos=end+21
 assert pos==len(raw);return entries
for target in ('history-774/continuation/camera/admission03/contracts.jsonl.gz','history-774/continuation/rig/admission03/contract-groups.jsonl.gz'):
 pf=next(x for x in gp['proofs'] if x['archive']==target);assert pf['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
 typ,commit=git_object(pf['revision']);assert typ=='commit';root=commit.split(b'\n',1)[0].split()[1].decode();assert root==pf['chain'][0]['tree'] and '/'.join(x['name'] for x in pf['chain'])==pf['path']
 for j,x in enumerate(pf['chain']):
  typ,tree=git_object(x['tree']);assert typ=='tree';mode,child=tree_entries(tree)[x['name']];assert mode==x['mode'].lstrip('0') and child==x['object']
  if j+1<len(pf['chain']):assert child==pf['chain'][j+1]['tree']
 body=gzip.decompress((private/'weapon-assets-2591-2592'/target).read_bytes());assert h(body)==pf['sha256'] and oid('blob',body)==pf['blob']==pf['chain'][-1]['object']
assert (counts['top_files'],counts['private_originals_checked'],counts['substitutions'],counts['confirmed_slash_prose_occurrences'],counts['other_pathlike_substitutions'])==(398,37,173,7,166)
assert (counts['stored_transport_matches'],counts['union_transport_matches'],counts['redacted_transport_matches'],len(redacted_originals))==(115,124,15,11)
assert (qual['matched_nested_member_occurrences'],qual['redacted_public_member_occurrences'],qual['unique_redacted_original_sha_count'])==(115,9,9)
assert len(qual['entries'])==9
# The 059 uncompressed identities represented only within gzip members.
oldchecks=[];extraction=os.environ.get('EXTRACTION_059')
for item in proposal['transport_match_correction']['newly_disclosed_original_records']:
 row=next(e for e in transport['entries'] if e['path']==item['path'])
 assert (row['sha256'],row['bytes'],row['git_blob'])==(item['original_uncompressed_sha256'],item['original_uncompressed_bytes'],item['original_git_blob'])
 for occurrence in item['occurrences']:
  metadata=next(m for b in cp['bundles'] if b['bundle_id']==occurrence['bundle_id'] for m in b['members'] if m['path']==occurrence['archive_member'])
  assert metadata['changed'] and metadata['original_sha256']==item['original_stored_gzip_sha256'] and metadata['original_uncompressed_sha256']==item['original_uncompressed_sha256'] and metadata['published_sha256']==item['public_stored_gzip_sha256'] and metadata['published_uncompressed_sha256']==item['public_uncompressed_sha256']
  raw=(private/occurrence['bundle_id']/occurrence['archive_member']).read_bytes();data=gzip.decompress(raw);assert h(data)==row['sha256'] and len(data)==row['bytes'] and oid('blob',data)==row['git_blob']
 if extraction:assert (Path(extraction)/item['path']).read_bytes()==data
 oldchecks.append({'original_transport_path':item['path'],'sha256':row['sha256'],'bytes':row['bytes'],'git_blob':row['git_blob'],'extraction_compared':bool(extraction)})
if extraction:
 for x in qual['entries']:
  data=(Path(extraction)/x['original_transport_paths'][0]).read_bytes();assert h(data)==x['original_sha256'] and len(data)==x['original_bytes'] and oid('blob',data)==x['original_git_blob']
print(json.dumps({'result':'confirmed_two_additive_qualification_corrections','counts':counts,'unique_redacted_original_identities':len(redacted_originals),'reported_qualification':[115,9,9],'original_059_checks':oldchecks,'existing_nine_qualified_originals_extraction_checked':bool(extraction),'exact_false_positive_member_occurrences':len(side),'affected_member_count':len({(x[0],x[1]) for x in side}),'current_vs_historical_note':'Project 95e, earlier 059 e275 and 774 supplier records remain separate; no source-read or checkpoint3821 credit.'},indent=2))
