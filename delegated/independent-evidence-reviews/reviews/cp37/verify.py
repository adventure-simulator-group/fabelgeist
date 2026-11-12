#!/usr/bin/env python3
"""Independent, read-only verification of the captured CP37 evidence snapshot.

Requires ../checked-stage, which contains local-only unredacted originals. Reads
files only, writes no output artifacts, executes no staged or product code.
"""
import gzip,hashlib,io,json,os,re,tarfile
from pathlib import Path
here=Path(__file__).resolve().parent
root=here.parent/'checked-stage'
pub=root/'publication'
private=root/'private-originals/checkpoint37'
h=lambda x:hashlib.sha256(x).hexdigest()
def oid(kind,data):return hashlib.sha1(kind.encode()+b' '+str(len(data)).encode()+b'\0'+data).hexdigest()
def load(p):return json.loads(p.read_bytes())
manifest=load(here/'MANIFEST.json')
for item in manifest['files']:
 data=(here/item['path']).read_bytes()
 assert len(data)==item['bytes'] and h(data)==item['sha256'],item['path']
def untar(data):
 with tarfile.open(fileobj=io.BytesIO(data),mode='r:*') as t:
  rows=t.getmembers(); assert all((m.isfile() or m.isdir()) and not m.name.startswith('/') and '..' not in Path(m.name).parts for m in rows)
  assert len({m.name for m in rows})==len(rows)
  return {m.name:(t.extractfile(m).read() if m.isfile() else None) for m in rows}
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I)
token=re.compile(rb'<LOCAL_PATH_\d+>')
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
cp=load(pub/'DELEGATED-EVIDENCE-CHECKPOINT-37.json')
assert h((pub/'DELEGATED-EVIDENCE-CHECKPOINT-37.json').read_bytes())=='9f5ae7584cc93ee200fd1c279272e1267cbfceb60b4ce87945e3a742f2967375'
assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
pri=load(private/'MANIFEST.json'); priv={(x['bundle_id'],x['path']):x for x in pri['members']}; assert len(priv)==len(pri['members'])==21
transport=loadjson=None
transport=json.loads(gzip.decompress((pub/'recovered/original-transport-manifest.json.gz').read_bytes()))
transport_hashes={x['sha256'] for x in transport['entries']}
counts={'archive_files':0,'recursive_files':0,'redacted_members':0,'path_redactions':0,'stored_matches':0,'uncompressed_matches':0,'union_matches':0,'redacted_transport_occurrences':0,'git_objects':0,'git_path_proofs':0,'prior_manifest_bindings':0}
redacted_originals=set(); seen_private=set(); bundles={}; verified_paths=set()
def verify_member(body,m,where):
 counts['recursive_files']+=1
 assert h(body)==m['published_sha256'] and len(body)==m['published_bytes'],where
 enc=m['content_encoding']; plain=gzip.decompress(body) if enc.startswith('gzip') else body
 assert h(plain)==m['published_uncompressed_sha256'] and len(plain)==m['published_uncompressed_bytes'],where
 assert m['changed']==(m['published_sha256']!=m['original_sha256'])
 s=m['original_sha256'] in transport_hashes; u=m['original_uncompressed_sha256'] in transport_hashes
 counts['stored_matches']+=s;counts['uncompressed_matches']+=u;counts['union_matches']+=s or u
 if (s or u) and m['changed']:
  counts['redacted_transport_occurrences']+=1; redacted_originals.add(m['original_uncompressed_sha256'] if u else m['original_sha256'])
 if not m['changed']:
  assert h(body)==m['original_sha256'] and len(body)==m['original_bytes']
  assert h(plain)==m['original_uncompressed_sha256'] and len(plain)==m['original_uncompressed_bytes']
 if enc.endswith('tar'):
  kids=untar(plain);rows={r['path']:r for r in m['members']};assert set(kids)==set(rows)
  for k,v in kids.items():
   if v is None: assert rows[k]['entry_type']=='directory'
   else: verify_member(v,rows[k],where+'!'+k)
  assert sum(r['total_local_path_redactions'] for r in rows.values())==m['total_local_path_redactions']
 else:
  assert not local.search(plain) and not secret.search(plain),where
  assert m['direct_local_path_redactions']==m['total_local_path_redactions']

def tree_entries(raw):
 d={};pos=0
 while pos<len(raw):
  end=raw.index(b'\0',pos);mode,name=raw[pos:end].split(b' ',1); assert name not in d;d[name.decode()]=(mode.decode().lstrip('0'),raw[end+1:end+21].hex());pos=end+21
 assert pos==len(raw);return d
for b in cp['bundles']:
 raw=(pub/b['archive']).read_bytes();assert h(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes'];members=untar(raw)
 rows={m['path']:m for m in b['members']};assert len(rows)==len(b['members']) and set(members)==set(rows)
 assert (pub/b['report_path']).read_bytes()==members[b['report_archive_member']]
 report=json.loads(members[b['report_archive_member']]); assert report['author']==b['agent']
 counts['archive_files']+=len(rows); bundles[b['bundle_id']]=members
 for path,body in members.items():
  assert body is not None
  m=rows[path]; verify_member(body,m,b['bundle_id']+'!'+path); counts['path_redactions']+=m['total_local_path_redactions']
  if m['changed']:
   key=(b['bundle_id'],path); entry=priv[key];orig=(private/key[0]/key[1]).read_bytes();seen_private.add(key)
   assert h(orig)==entry['sha256']==m['original_sha256'] and len(orig)==entry['bytes']==m['original_bytes']
   o=gzip.decompress(orig) if m['content_encoding'].startswith('gzip') else orig
   pp=gzip.decompress(body) if m['content_encoding'].startswith('gzip') else body
   assert h(o)==m['original_uncompressed_sha256'] and len(o)==m['original_uncompressed_bytes']
   # Segment equality proves the explicitly detected substitutions preserve all other bytes.
   assert local.split(o)==token.split(pp) and len(local.findall(o))==len(token.findall(pp))==m['direct_local_path_redactions']
   counts['redacted_members']+=1
 assert set((b['bundle_id'],x) for x in rows if rows[x]['changed']) <= set(priv)
 # Independently hash every supplied raw Git object and walk every path proof.
 gp=json.loads(members['git-proofs.json']);objects={}
 for ident,meta in gp['objects'].items():
  loc=meta.get('preserved_bytes',meta.get('local',meta.get('archive'))); body=gzip.decompress(members[loc]); typ=meta['type']
  assert h(body)==meta['sha256'] and len(body)==meta.get('bytes',meta.get('size')) and oid(typ,body)==ident
  objects[ident]=(typ,body);counts['git_objects']+=1
 for proof in gp.get('paths',gp.get('proofs',[])):
  revision=proof['revision']; typ,commit=objects[revision];assert typ=='commit';commit_tree=commit.split(b'\n',1)[0].split()[1].decode()
  chain=proof.get('path_chain',proof.get('chain')); assert chain[0]['tree']==commit_tree and '/'.join(x['name'] for x in chain)==proof['path']
  for index,x in enumerate(chain):
   typ,tree=objects[x['tree']];assert typ=='tree'; mode,obj=tree_entries(tree)[x['name']];assert mode==x['mode'].lstrip('0') and obj==x['object']
   if index+1<len(chain):assert obj==chain[index+1]['tree']
  loc=proof.get('preserved_bytes',proof.get('local',proof.get('archive')));item=members[loc]
  if (b['bundle_id'],loc) in priv:item=(private/b['bundle_id']/loc).read_bytes()
  encoding=proof.get('preserved_encoding',proof.get('encoding')); data=gzip.decompress(item) if encoding=='gzip' else item
  assert h(data)==proof['sha256'] and oid('blob',data)==proof.get('git_blob',proof.get('blob'))==chain[-1]['object']; counts['git_path_proofs']+=1
  verified_paths.add((b['bundle_id'],revision,proof['path'],h(data)))
assert seen_private==set(priv)
prior=load(pub/'QUALIFICATION-CHECKPOINT37-PRIOR-MANIFESTS.json');oldcp=load(pub/'DELEGATED-EVIDENCE-CHECKPOINT-17.json')
for x in prior['rows']:
 old=next(b for b in oldcp['bundles'] if b['bundle_id']==x['earlier_bundle']); oldraw=(pub/x['earlier_archive']).read_bytes(); assert h(oldraw)==x['earlier_archive_sha256']==old['archive_sha256']
 exact=untar(oldraw)[x['earlier_member']]; later=bundles[x['packet']][x['unlisted_member']]; assert exact==later and h(exact)==x['original_sha256'] and len(exact)==x['bytes']
 fm=json.loads(bundles[x['packet']]['MANIFEST.json']);assert x['unlisted_member'] not in {y['path'] for y in fm['files']};counts['prior_manifest_bindings']+=1
# Verify the six source identities through registry, personal archive and Git path proof results above.
registry=json.loads(gzip.decompress((pub/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
role_paths={2611:('helmet-crown-buffe-2611-2612','sources/crown-old','sources/crown-now'),2612:('helmet-crown-buffe-2611-2612','sources/buffe-old','sources/buffe-now'),2613:('visor-buffe-2613-2614','source/2613-historical.rs','source/2613-current.rs'),2614:('visor-buffe-2613-2614','source/2614-historical.rs','source/2614-current.rs'),2615:('pierced-sampling-2615-2616','source/pr-working-2615.rs','source/current-2615.rs'),2616:('pierced-sampling-2615-2616','source/pr-working-2616.rs','source/current-2616.rs')}
roles=[]
for n,(packet,old,now) in role_paths.items():
 r=registry[n];o=bundles[packet][old]; c=bundles[packet][now];assert h(o)==r['sha256'] and len(o)==r['bytes']
 path=r['source_id'].split(':',1)[1];base='6533e444fa37552927070cc8d8190907fd3b413e';nowrev=cp['working_revision'];f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
 assert (packet,base,path,h(o)) in verified_paths and (packet,nowrev,path,h(c)) in verified_paths
 assert any(x[0]==packet and x[1]==f82 and x[2].endswith('/objects/'+h(o)) and x[3]==h(o) for x in verified_paths)
 roles.append({'role':n,'historical_sha256':h(o),'current_sha256':h(c),'historical_material_ranges':r['material_ranges']})
assert all(x['historical_sha256']==x['current_sha256'] for x in roles[:-1]);assert roles[-1]['historical_sha256']!=roles[-1]['current_sha256']
third=bundles['pierced-sampling-2615-2616'];receipt=json.loads(third['read-receipt.json']);assert receipt['author']=='/root/mhr_character'
a={x['tag']:x for x in receipt['entries']};assert a['pr-working-2616']['displayed_chunks']==[[1,45]] and a['pr-working-2616']['unread_complement']==[[46,71]] and a['current-2616']['displayed_chunks']==[[1,49]] and a['current-2616']['unread_complement']==[[50,75]]
first=bundles['helmet-crown-buffe-2611-2612'];reg1=json.loads(first['registration.json']); rec1=json.loads(first['personal-read-receipt.json']); assert reg1['author']==rec1['author']=='/root/rig_identity'
rc1=next(x for x in rec1['registrations'] if x['registration']=='registration.json');assert rc1['registration_sha256']==h(first['registration.json']) and rc1['registration_utc']==reg1['registered_utc'] and reg1['registered_utc']<rec1['recorded_utc']
for entry in reg1['entries']:
 if entry.get('role') in (2611,2612):
  r=next(x for x in rc1['reads'] if x['id']==entry['id']);assert r['sha256']==entry['sha256']==h(first[entry['archive']]) and r['actual_display_ranges']==entry['prospective_ranges']
second=bundles['visor-buffe-2613-2614'];reg2=json.loads(second['registration.json']);rec2=json.loads(second['read-receipt.json']);assert reg2['reader']==rec2['reader']=='/root/hair_ops'
assert rec2['registration_sha256']['registration.json']==h(second['registration.json']) and reg2['at_utc']<rec2['recorded_utc']
for entry in reg2['entries']:
 if entry.get('original_index') in (2613,2614):
  for version in entry['versions']:
   r=next(x for x in rec2['actual_displays'] if x['local']==version['local']);assert r['sha256']==version['sha256']==h(second[version['local']]) and r['displayed_ranges']==version['registered_windows']
reg3=json.loads(third['registration.json']);assert reg3['author']==receipt['author'] and receipt['registration_sha256']==h(third['registration.json']) and reg3['registered_utc']<receipt['recorded_utc']
for entry in reg3['entries']:
 if entry.get('role') in (2615,2616):
  r=a[entry['tag']];assert r['sha256']==entry['sha256']==h(third[entry['archive']])
# Compare the baseline copies; the c152 membership proof was walked above.
hat=bundles['helmet-crown-buffe-2611-2612'];committed=hat['history/published/BASELINE-95E-20261110.json'];working=hat['history/published/working-baseline-observed.json']
assert h(committed)=='decd0b95699edfc1a59039358bc3a171511c2c0199afab8967cfd7169aca3157'
assert h(working)=='28138b4548101716246fee073984868472996f0e350d3f3526d237d61c6a67f9'
assert committed!=working and json.loads(committed)['source_revision']==json.loads(working)['source_revision']==cp['working_revision']
assert counts['archive_files']==384 and counts['redacted_members']==21 and counts['stored_matches']==125 and counts['union_matches']==134 and counts['redacted_transport_occurrences']==6 and len(redacted_originals)==2 and counts['prior_manifest_bindings']==3
qual=load(pub/'DELEGATED-CHECKPOINT-37-QUALIFICATION.json'); assert (qual['matched_nested_member_occurrences'],qual['unique_redacted_original_sha_count'],qual['redacted_public_member_occurrences'])==(125,0,0)
oldpaths={'evidence/continuation-segovb/ledger.json':'history/baseline-059-ledger.json.gz','workflow-20261030/CLOSURE-QUEUE-100.json':'history/baseline-059-queue.json.gz'}
oldchecks=[]
for path,member in oldpaths.items():
 r=next(x for x in transport['entries'] if x['path']==path); data=gzip.decompress((private/'helmet-crown-buffe-2611-2612'/member).read_bytes())
 assert len(data)==r['bytes'] and h(data)==r['sha256'] and oid('blob',data)==r['git_blob']
 extraction=os.environ.get('EXTRACTION_059')
 if extraction:assert (Path(extraction)/path).read_bytes()==data
 oldchecks.append({'path':path,'sha256':h(data),'git_blob':oid('blob',data),'bytes':len(data),'optional_extraction_compared':bool(extraction)})
# The direct link mapping names two published files that were not staged as
# symlinks. Those exact public files are retained alongside this verifier.
for link in load(pub/'DELEGATED-CHECKPOINT-37-DIRECT-LINKS.json')['links']:
 path=pub/link['path']
 if not path.exists():path=here/'evidence'/Path(link['path']).name
 data=path.read_bytes();assert len(data)==link['bytes'] and h(data)==link['sha256']
print(json.dumps({'checks':'pass_except_recorded_qualification_omission','counts':counts,'redacted_transport_originals':sorted(redacted_originals),'original_059_checks':oldchecks,'roles':roles,'baseline_committed_sha256':h(committed),'baseline_working_sha256':h(working),'qualification_document_recorded':[125,0,0],'interpretation':'125 stored-hash matches; union of stored and uncompressed transport hashes 134; six redacted gzip occurrences of two originals are omitted from the staged qualification. No product execution or source read credit inferred.'},indent=2))
