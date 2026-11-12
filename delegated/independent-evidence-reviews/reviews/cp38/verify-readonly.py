#!/usr/bin/env python3
"""Read-only stage integrity and qualifications review; never invokes product or packet verifiers."""
import collections,datetime,gzip,hashlib,io,json,pathlib,re,tarfile
S=pathlib.Path('<PRIVATE_PATH_0001>');P=S/'publication';V=pathlib.Path('<PRIVATE_PATH_0002>');priv=S/'private-originals/checkpoint38';here=pathlib.Path(__file__).parent
origins={'extremity-joint-2617-2618':pathlib.Path('<PRIVATE_PATH_0012>'),'besagew-radial-2619-2620':pathlib.Path('<PRIVATE_PATH_0013>'),'joint-pauldron-2621-2622':pathlib.Path('<PRIVATE_PATH_0014>')}; names={'extremity-joint-2617-2618':'MANIFEST.json','besagew-radial-2619-2620':'MANIFEST.json','joint-pauldron-2621-2622':'BUNDLE-MANIFEST.json'}
sha=lambda b:hashlib.sha256(b).hexdigest();oid=lambda k,b:hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest();j=lambda f:json.loads(f.read_bytes());cp=j(P/'DELEGATED-EVIDENCE-CHECKPOINT-38.json');original=j(priv/'MANIFEST.json');pm={(e['bundle_id'],e['path']):e for e in original['members']};regex=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and len(cp['bundles'])==3
assert len(pm)==len(original['members'])==22
transport=json.loads(gzip.decompress((V/'recovered/original-transport-manifest.json.gz').read_bytes()));transport_bysha=collections.defaultdict(list)
for e in transport['entries']:transport_bysha[e['sha256']].append(e)
roles_json=json.loads(gzip.decompress((V/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()));coverage=j(V/'COVERAGE-L01-ROLES.json')['roles'];qprior=j(P/'QUALIFICATION-CHECKPOINT38-PRIOR-MANIFESTS.json');q=j(P/'DELEGATED-CHECKPOINT-38-QUALIFICATION.json');allmeta=[];matches=[];redacted_matches=[];totals=collections.Counter();bresults=[];false_positives=[];proofs=[];redaction_categories=collections.Counter()
def iterate(m):
 yield m
 for c in m.get('members',[]):yield from iterate(c)
def recursive(body,m,where,depth=0):
 assert depth<=9 and (len(body),sha(body))==(m['published_bytes'],m['published_sha256']),where
 enc=m['content_encoding'];assert enc in ('identity','gzip','tar','gzip-tar');plain=gzip.decompress(body) if enc.startswith('gzip') else body
 assert (len(plain),sha(plain))==(m['published_uncompressed_bytes'],m['published_uncompressed_sha256']);assert m['changed']==(m['published_sha256']!=m['original_sha256'])
 if not m['changed']:assert (len(body),len(plain),sha(plain),m['total_local_path_redactions'])==(m['original_bytes'],m['original_uncompressed_bytes'],m['original_uncompressed_sha256'],0)
 else:assert 'Reconstructed public copy' in m['qualification'] and m['total_local_path_redactions']>0
 if m.get('original_sha256') in transport_bysha:
  matches.append((where,m['original_sha256']))
  for e in transport_bysha[m['original_sha256']]:
   assert e['bytes']==m['original_bytes']
   if not m['changed']:assert oid('blob',body)==e['git_blob']
  if m['changed']:redacted_matches.append(where)
 if enc.endswith('tar'):
  rows={x['path']:x for x in m['members']};assert len(rows)==len(m['members']) and m['direct_local_path_redactions']==0
  with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
   xs=t.getmembers();assert len(xs)==len(rows) and set(x.name for x in xs)==set(rows)
   for x in xs:
    assert not x.name.startswith('/') and '..' not in pathlib.PurePosixPath(x.name).parts and (x.isfile() or x.isdir());assert not regex.search(x.name.encode()) and not secret.search(x.name.encode())
    assert all(not regex.search(str(v).encode()) and not secret.search(str(v).encode()) for v in [x.uname,x.gname,*x.pax_headers.values()])
    if x.isdir():assert rows[x.name]['entry_type']=='directory' and not rows[x.name]['changed'] and rows[x.name]['total_local_path_redactions']==0
    else:recursive(t.extractfile(x).read(),rows[x.name],where+'!'+x.name,depth+1)
  assert m['total_local_path_redactions']==sum(c['total_local_path_redactions'] for c in rows.values())
 else:assert not regex.search(plain) and not secret.search(plain) and plain.count(b'<LOCAL_PATH_')>=m['direct_local_path_redactions'] and m['total_local_path_redactions']==m['direct_local_path_redactions']
 totals['recursive_nodes']+=1

def tree(b):
 d={};i=0
 while i<len(b):
  z=b.index(b'\0',i);mode,name=b[i:z].split(b' ',1);d[name.decode()]=(mode.decode().lstrip('0'),b[z+1:z+21].hex());i=z+21
 assert i==len(b);return d
role_info=[]
with tarfile.open(V/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as registry_archive:
 for b in cp['bundles']:
  n=b['bundle_id'];local=origins[n];raw=(P/b['archive']).read_bytes();assert (len(raw),sha(raw))==(b['archive_bytes'],b['archive_sha256']);items={x['path']:x for x in b['members']};assert len(items)==len(b['members']);totals['top_members']+=len(items)
  with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
   arr=t.getmembers();assert len(arr)==len(items) and {x.name for x in arr}==set(items) and all(x.isfile() and not x.name.startswith('/') and '..' not in pathlib.PurePosixPath(x.name).parts and (x.uid,x.gid,x.uname,x.gname)==(0,0,'','') for x in arr);data={x.name:t.extractfile(x).read() for x in arr}
  for path,body in data.items():recursive(body,items[path],n+'/'+path)
  assert data[b['report_archive_member']]==(P/b['report_path']).read_bytes();assert sha(data[names[n]])==sha((local/names[n]).read_bytes())
  manifest=json.loads(data[names[n]]);rows={x['path']:x for x in manifest['files']};assert len(rows)==len(manifest['files'])
  extras=set(items)-set(rows)-{names[n]};allow={x['unlisted_member'] for x in qprior['rows'] if x['packet']==n};assert extras==allow
  changed=[]
  for path,m in items.items():
   o=(local/path).read_bytes();assert (len(o),sha(o))==(m['original_bytes'],m['original_sha256'])
   if path in rows:assert (rows[path]['bytes'],rows[path]['sha256'])==(len(o),sha(o))
   if not m['changed']:assert data[path]==o
   else:
    x=pm[(n,path)];private=(priv/n/path).read_bytes();assert private==o and (len(private),sha(private))==(x['bytes'],x['sha256']);changed.append(path);totals['changed_top']+=1
    a=gzip.decompress(o) if m['content_encoding']=='gzip' else o;z=gzip.decompress(data[path]) if m['content_encoding']=='gzip' else data[path];mapping={}
    def subst(match):
     key=match.group(0)
     if key not in mapping:mapping[key]=len(mapping)
     return ('<LOCAL_PATH_%d>'%mapping[key]).encode()
    assert regex.sub(subst,a)==z,('redaction altered more than matches',n,path);assert len(regex.findall(a))==m['direct_local_path_redactions'];totals['redactions']+=len(regex.findall(a))
    for v in regex.findall(a):
     if v.startswith(b'<PRIVATE_PATH_0015>'):redaction_categories['local_user_absolute']+=1
     elif v.startswith(b'/tmp/726-'):redaction_categories['temporary_absolute']+=1
     else:redaction_categories['prose_false_positives']+=1
    for target,description in [(b'/home/noerrors','774 contract-groups row 767 source/home/noerrors prose'),(b'/home/window/viewport/overlay','059 ledger source/home/window/viewport/overlay prose'),(b'/users/dependencies','bespoke audit skill definitions/users/dependencies prose')]:
     if target in regex.findall(a):false_positives.append({'bundle':n,'member':path,'matching_substring':target.decode(),'occurrences':regex.findall(a).count(target),'context':description})
  for i in b['scope_original_roles']:
   r=roles_json['files'][i];c=coverage[i];h=r['sha256'];assert r['snapshot']==c['snapshot']=='pr-working' and h==c['historical_sha256'];body=registry_archive.extractfile(h).read();assert sha(body)==h and len(body)==r['bytes'];assert any(m.get('original_sha256')==h for top in items.values() for m in iterate(top));role_info.append({'role':i,'source_id':r['source_id'],'snapshot':r['snapshot'],'sha256':h,'material_ranges':r['material_ranges'],'bytes':r['bytes'],'reconciliation_reviewer':b['agent'],'original_registry_audit_owner':r['audit_owner'],'original_registration_ids':[z['registration_id'] for z in r['admission03_scope_observations']]})
  gp=json.loads(data['git-proofs.json']);objects={}
  for k,x in gp['objects'].items():
   path=x.get('archive',x.get('preserved_bytes',x.get('preserved',x.get('local'))));body=gzip.decompress(data[path]);assert (len(body),sha(body),oid(x['type'],body))==(x.get('bytes',x.get('size')),x['sha256'],k);objects[k]=body
  for x in gp.get('proofs',gp.get('paths',[])):
   revision=x['revision'];commit=objects[revision];rt=commit.splitlines()[0].split()[1].decode();assert rt==x.get('commit_tree',x.get('root_tree',gp.get('commits',{}).get(revision,{}).get('root_tree')));parts=x['path'].split('/');chain=x.get('chain',x.get('path_chain'));assert len(parts)==len(chain);cur=rt
   for part,link in zip(parts,chain):
    assert link['tree']==cur and link['name']==part;mode,child=tree(objects[cur])[part];assert mode==link['mode'].lstrip('0') and child==link['object'];cur=child
   assert cur==x.get('blob',x.get('git_blob',cur));path=x.get('archive',x.get('local',x.get('preserved_bytes',x.get('preserved'))));m=items[path];body=(priv/n/path).read_bytes() if m['changed'] else data[path]
   if m['content_encoding']=='gzip' and x.get('preserved_encoding')!='raw':body=gzip.decompress(body)
   assert oid('blob',body)==cur
   if x.get('sha256'):assert sha(body)==x['sha256']
   if x.get('bytes') is not None:assert len(body)==x['bytes']
   proofs.append({'bundle':n,'path':x['path'],'revision':revision,'sha256':sha(body)})
  bresults.append({'bundle':n,'agent':b['agent'],'archive':b['archive'],'archive_sha256':sha(raw),'top_members':len(items),'changed_top_members':len(changed),'prior_manifest_extra':sorted(extras),'original_manifest_sha256':sha(data[names[n]])})
assert totals['top_members']==393 and totals['changed_top']==22 and len(proofs)==81 and len(matches)==q['matched_nested_member_occurrences']==132 and not redacted_matches and q['unique_redacted_original_sha_count']==q['redacted_public_member_occurrences']==0
assert len(false_positives)==6 and sum(x['occurrences'] for x in false_positives)==6 and redaction_categories=={'local_user_absolute':129,'temporary_absolute':15,'prose_false_positives':6}
assert {x['role'] for x in role_info}==set(range(2617,2623))
for x in role_info:
 path=x['source_id'].split(':',1)[1]
 for rev in ('6533e444fa37552927070cc8d8190907fd3b413e','95e63628f9d70b8dd6e6fede64334696c6995009'):
  assert any(z['path']==path and z['revision']==rev and z['sha256']==x['sha256'] for z in proofs),(x['role'],rev)
 assert any(z['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee' and z['path'].endswith('/objects/'+x['sha256']) for z in proofs)
# One prior manifest omitted by the later root manifest, bound to exact public prior checkpoint 17.
r=qprior['rows'][0];prior_row=r;prior_cp=j(V/r['earlier_checkpoint']);old=next(x for x in prior_cp['bundles'] if x['bundle_id']==r['earlier_bundle']);prior_raw=(V/r['earlier_archive']).read_bytes();assert sha(prior_raw)==r['earlier_archive_sha256']==old['archive_sha256']
nb=next(x for x in cp['bundles'] if x['bundle_id']==r['packet']);
with tarfile.open(P/nb['archive']) as later,tarfile.open(fileobj=io.BytesIO(prior_raw),mode='r:gz') as earlier:
 l=later.extractfile(r['unlisted_member']).read();o=earlier.extractfile(r['earlier_member']).read();assert l==o and (len(l),sha(l))==(r['bytes'],r['original_sha256']);assert r['unlisted_member'] not in {x['path'] for x in json.load(later.extractfile('MANIFEST.json'))['files']}
 comparison=json.load(later.extractfile('history/prior-role/published-comparison.json'));assert comparison['archive_sha256']==sha(prior_raw)
 for x in comparison['comparisons']:
  y=next(e for e in old['members'] if e['path']==x['member']);v=earlier.extractfile(x['member']).read();assert (x['original_sha256'],x['published_sha256'])==(y['original_sha256'],y['published_sha256']) and sha(v)==x['published_sha256'] and x['byte_identical']==(x['original_sha256']==x['published_sha256'])
 assert [x['member'] for x in comparison['comparisons'] if not x['byte_identical']]==['historical-manifest.json']
 committed=later.extractfile('history/published/BASELINE-95E-20261110.json').read();working=later.extractfile('history/published/working-baseline-observed.json').read();assert sha(committed)=='28138b4548101716246fee073984868472996f0e350d3f3526d237d61c6a67f9' and sha(working)=='fc828ecf7e09c2b563c46ec6b8ef5c67267baeb5455f1ea62d3739b2e2ded6ad' and committed!=working
 assert any(x['revision']=='56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e' and x['path']=='BASELINE-95E-20261110.json' and x['sha256']==sha(committed) for x in proofs)
 key='REVIEW-METAL-SEED-DRAW-OVERLAP-95E';assert next(x for x in json.loads(committed)['newly_assessed_ledger']['entries'] if x['id']==key)==next(x for x in json.loads(working)['newly_assessed_ledger']['entries'] if x['id']==key)
# Earlier unregistered locator is not a personal-read credit; later registration cannot make it first exposure.
b=next(x for x in cp['bundles'] if x['bundle_id']=='joint-pauldron-2621-2622')
with tarfile.open(P/b['archive']) as t:
 report=json.load(t.extractfile('report.json'));priorread=next(x for x in report['historical_attribution'] if x['reviewer']=='/root/mhr_character earlier checkpoint17 packet');assert 'pre-registration locator display of joint_cup.rs line142 with no credit' in priorread['scope'] and 'later reread' in priorread['scope'];prior_registration=json.load(t.extractfile('prior/support-registration-03.json'));assert 'unregistered joint_cup.rs line 142' in prior_registration['navigation_qualification'] and 'gets no credit' in prior_registration['navigation_qualification']
# Direct links checked against the live overlay inputs, not incorrectly treated as standalone stage files.
links=j(P/'DELEGATED-CHECKPOINT-38-DIRECT-LINKS.json')['links'];linkresults=[]
for x in links:
 v=V/x['path'];raw=v.read_bytes();assert (len(raw),sha(raw))==(x['bytes'],x['sha256']);sp=P/x['path'];linkresults.append({'path':x['path'],'sha256':sha(raw),'bytes':len(raw),'stage_exists':sp.exists(),'stage_resolves_inside':sp.resolve().is_relative_to(P.resolve())})
stageprog=j(P/'CONTINUATION-PROGRESS-95E.json');liveprog=j(V/'CONTINUATION-PROGRESS-95E.json');diff={'stage_only':sorted(set(stageprog)-set(liveprog)),'live_only':sorted(set(liveprog)-set(stageprog)),'common_different':sorted(k for k in set(stageprog)&set(liveprog) if stageprog[k]!=liveprog[k])};assert stageprog['delegated_batch38_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-38.json'
# Receipt authors stay distinct; timestamps and ranges checked against each local frozen registration.
receipt=[]
for b in cp['bundles']:
 n=b['bundle_id'];local=origins[n];rfile='personal-read-receipt.json' if n.startswith('extremity') else 'read-receipt.json';rr=j(local/rfile);author=rr.get('author',rr.get('reader'));assert author==b['agent'];registration=j(local/'registration.json');rhash=sha((local/'registration.json').read_bytes());at=registration.get('registered_utc',registration.get('at_utc'));assert datetime.datetime.fromisoformat(rr['recorded_utc'])>datetime.datetime.fromisoformat(at)
 if n.startswith('extremity'):
  group=next(g for g in rr['registrations'] if g['registration']=='registration.json');assert group['registration_sha256']==rhash;displays={z['id']:z for z in group['reads']};entries=[z for z in registration['entries'] if z.get('role') in b['scope_original_roles']]
  for z in entries:
   d=displays[z['id']];r=roles_json['files'][z['role']];assert d['sha256']==z['sha256']==r['sha256'] and d['actual_display_ranges']==z['prospective_ranges'] and not d['unread_complement'];assert sum(hi-lo+1 for lo,hi in d['actual_display_ranges'])==r['actual_line_count']
 elif n.startswith('besagew'):
  assert rr['registration_sha256']['registration.json']==rhash
  for role in b['scope_original_roles']:
   reg=next(z for z in registration['entries'] if z.get('original_index')==role);r=roles_json['files'][role];assert reg['original_registry_row']==r
   for version in reg['versions']:
    d=next(z for z in rr['actual_displays'] if z.get('local')==version['local']);assert d['sha256']==version['sha256']==r['sha256'] and d['displayed_ranges']==version['registered_windows'] and not d['unread_complement_per_registration']
 else:
  assert rr['registration_sha256']==rhash;displays={z['tag']:z for z in rr['entries']};entries=[z for z in registration['entries'] if z.get('role') in b['scope_original_roles']]
  for z in entries:
   d=displays[z['tag']];r=roles_json['files'][z['role']];assert d['sha256']==z['sha256']==r['sha256'] and {i for a,b in d['displayed_chunks'] for i in range(a,b+1)}=={i for a,b in z['registered_ranges'] for i in range(a,b+1)} and not d['unread_complement']
 receipt.append({'bundle':n,'reconciliation_reviewer':author,'receipt':rfile,'sha256':sha((local/rfile).read_bytes()),'registration_sha256':rhash,'qualification':'Both historical and current displays verified as separate registrations/receipts, retaining original historical supplier.'})
inputs={str(f):sha(f.read_bytes()) for f in [P/'DELEGATED-EVIDENCE-CHECKPOINT-38.json',P/'DELEGATED-CHECKPOINT-38-DIRECT-LINKS.json',P/'DELEGATED-CHECKPOINT-38-QUALIFICATION.json',P/'QUALIFICATION-CHECKPOINT38-PRIOR-MANIFESTS.json',P/'CONTINUATION-PROGRESS-95E.json',V/'CONTINUATION-PROGRESS-95E.json',priv/'MANIFEST.json',V/'COVERAGE-L01-ROLES.json']}
out={'schema':'726-checkpoint38-independent-readonly-review-results-v1','checked_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'pass_with_qualifications','inputs':inputs,'bundle_checks':bresults,'counts':dict(totals),'redaction_categories':dict(redaction_categories),'git_path_proofs':len(proofs),'original_transport_sha_matches':len(matches),'redacted_059_transport_matches':len(redacted_matches),'prior_manifest_binding':{'bytes':prior_row['bytes'],'sha256':prior_row['original_sha256'],'prior_archive_sha256':sha(prior_raw)},'committed_vs_working_baseline':{'committed_revision':'56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e','committed_sha256':sha(committed),'observed_working_sha256':sha(working),'equal':committed==working,'selected_ledger_id':key,'selected_ledger_equal':True},'earlier_unregistered_locator':'joint_cup.rs line 142: prior unregistered print gets no read credit; subsequent registered display is a reread','roles':role_info,'reader_receipts':receipt,'false_positive_redactions':false_positives,'direct_links':linkresults,'progress_difference':diff,'limits':'Read-only byte, metadata, Git membership, regex and provenance analysis only; no source-read credit from parsing, no product execution or validation, no whole privacy guarantee. Stage direct-link containment applies after overlay; live inputs and stage snapshots may subsequently change.'}
print(json.dumps(out,indent=2))
