import datetime,gzip,hashlib,json,pathlib,tarfile,tomllib
p=pathlib.Path(__file__).parent;sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();load=lambda f:json.loads((p/f).read_bytes())
raw=gzip.decompress((p/'reference/registry.json.gz').read_bytes());assert sha(raw)=='d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f';reg=json.loads(raw);sel=load('registry-selection-reconstructed.json');assert sel['registry_sha256']==sha(raw) and all(v==reg['files'][int(k)] for k,v in sel['roles'].items())
for e in load('reference-index.json')['records']:
 b=(p/e['file']).read_bytes();b=gzip.decompress(b) if e.get('encoding')=='gzip' else b;assert (len(b),sha(b))==(e['bytes'],e['sha256'])
for name in ('rules.json','skills.json'):
 for e in load(name)['records']:assert sha((p/e['file']).read_bytes())==e['sha256']
r=load('registration.json');receipt=load('read-receipt.json');assert receipt['registration_sha256']==sha((p/'registration.json').read_bytes()) and len(r['entries'])==len(receipt['actual_personal_displays'])==11 and datetime.datetime.fromisoformat(receipt['completed_utc'])>datetime.datetime.fromisoformat(r['registered_utc'])
def nums(ranges):return {i for lo,hi in ranges for i in range(lo,hi+1)}
for e,v in zip(r['entries'],receipt['actual_personal_displays']):
 b=(p/e['archive']).read_bytes();b=gzip.decompress(b) if e.get('encoding')=='gzip' else b;assert e['id']==v['id'] and sha(b)==e['sha256']==v['sha256'] and len(b)==e['bytes'] and len(b.splitlines())==e['eof'];seen=nums(v['displayed_ranges']);unseen=nums(v['unread_complement']);assert nums(e['registered_ranges'])==seen and seen|unseen==set(range(1,e['eof']+1)) and not seen&unseen
 if e['kind']=='native-package':
  with tarfile.open(p/e['package_archive'],'r:gz') as tar:assert tar.extractfile(e['path']).read()==b
  if e.get('role'):assert e['sha256']==reg['files'][e['role']]['sha256'] and e['bytes']==reg['files'][e['role']]['bytes']
for e in r['archives']:assert sha((p/e['path']).read_bytes())==e['sha256']
for e in [x for x in r['entries'] if x['kind']=='project-lock']:
 b=gzip.decompress((p/e['archive']).read_bytes());d=tomllib.loads(b.decode());pkgs={z['name']:z for z in d['package'] if z['name'] in ('num_enum','num_enum_derive')};assert len(pkgs)==2
 for n,h in [('num_enum','5d0bca838442ec211fa11de3a8b0e0e8f3a4522575b5c4c06ed722e005036f26'),('num_enum_derive','680998035259dcfcafe653688bf2aa6d3e2dc05e98be6ab46afb089dc84f1df8')]:assert pkgs[n]['version']=='0.7.6' and pkgs[n]['source']=='registry+https://github.com/rust-lang/crates.io-index' and pkgs[n]['checksum']==h
p16=(p/'reference/p16-dirty-Cargo.lock.gz').read_bytes();assert sha(p16)=='1046f0bf579b650b22e42b724ffe71114925bb01cca24dbafd7e70adcdcc43cc' and sha(gzip.decompress(p16))=='1191ee1e830f870d16ebaa30e6d84953656acaed04cf9c2a0367bbbcb1f78201'
qraw=gzip.decompress((p/'reference/queue.json.gz').read_bytes());lraw=gzip.decompress((p/'reference/ledger.json.gz').read_bytes());q=json.loads(qraw);l=json.loads(lraw);s=load('059-selection-reconstructed.json');assert s['queue_source_sha256']==sha(qraw) and s['ledger_source_sha256']==sha(lraw) and all(q['entries'][int(i)]==v for i,v in s['entries'].items())
for path,v in s['journal_entries'].items():
 z=l
 for part in path.strip('/').split('/'):z=z[int(part)] if isinstance(z,list) else z[part]
 assert z==v
assert q['entries'][152]['contract_id']=='ADLER-L11-FBX-RECORD-EXTENT-ADMISSION' and q['entries'][153]['contract_id']=='ADLER-L11-FBX-DECODE-BUDGET-ADMISSION' and q['entries'][2942]['contract_id']=='L12-CLOSURE-FBX-NUM-ENUM-NATIVE-ADAPTER'
for i in (152,153):assert all(x['revision_as_recorded'] is None and x['attribution_as_recorded'] is None for x in q['entries'][i]['exact_source_identities_and_assessed_ranges'])
m=(p/'reference/original-transport-manifest.json.gz').read_bytes();assert sha(m)=='7e45ce9d858f93e0c5e5d50f360f79ed8941e28049f305053f1fd2bdda364834';idx={e['path']:e for e in json.loads(gzip.decompress(m))['entries']};hist=load('historical-index.json');assert not hist['unavailable_requested_records']
for e in hist['records']:
 b=(p/e['file']).read_bytes();b=gzip.decompress(b) if e.get('encoding')=='gzip' else b;assert (len(b),sha(b),blob(b))==(e['bytes'],e['sha256'],e['git_blob'])
 if e['kind']=='original-059':assert (idx[e['transport_path']]['bytes'],idx[e['transport_path']]['sha256'],idx[e['transport_path']]['git_blob'])==(len(b),sha(b),blob(b))
for suffix in ('L11-429','L12-899','L12-900','L12-901'):
 v=load('history-059/segovb/VERIFY-'+suffix+'.json');assert v['receipt_sha256']==sha((p/('history-059/segovb/READ-'+suffix+'.json')).read_bytes()) and v['state_sha256']==sha((p/('history-059/segovb/STATE-'+suffix+'.json')).read_bytes())
 if suffix.startswith('L12'):
  read=load('history-059/segovb/READ-'+suffix+'.json');assert read['root_personal_rereads']==[] and read['proposal_sha256']==sha((p/'history-059'/read['proposal']).read_bytes())
checks=[]
def walk(v):
 if isinstance(v,dict):
  if v.get('path') in idx and isinstance(v.get('sha256'),str):
   rel=v['path'];actual=sha((p/'history-059'/rel).read_bytes());checks.append((rel,v['sha256']==actual))
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(s['entries']);assert len(checks)==57 and all(match for _,match in checks)
prior=load('prior-index.json');m=(p/'prior/MANIFEST.json').read_bytes();assert sha(m)==prior['original_manifest_sha256']=='479935e112b1bf71cd2b7908cad96cec99f72965f1fb2312828eaf615c9ceba9';idxprior={e['path']:e for e in json.loads(m)['files']}
for e in prior['selected_files']:
 b=(p/'prior'/e['path']).read_bytes();assert (len(b),sha(b))==(e['bytes'],e['sha256'])==(idxprior[e['path']]['bytes'],idxprior[e['path']]['sha256'])
entry=next(z for z in load('reference/DELEGATED-EVIDENCE-CHECKPOINT-18.json')['bundles'] if z['bundle_id']=='num-enum-derive');assert entry['archive_sha256']==prior['published_archive_sha256'];members=load('public-prior-members.json');assert members['archive_sha256']==entry['archive_sha256']
for v in members['members']:
 z=next(z for z in entry['members'] if z['path']==v['path']);assert (v['original_sha256'],v['published_sha256'],v['changed'])==(z['original_sha256'],z['published_sha256'],z['changed']) and v['original_sha256']==sha((p/'prior'/v['path']).read_bytes()) and v['published_sha256']==sha((p/v['file']).read_bytes())
assert next(v for v in members['members'] if v['path']=='registration.json')['changed'] and all(not v['changed'] for v in members['members'] if v['path']!='registration.json')
pin=hist['rig_registration_pin_qualification'];a=json.loads(gzip.decompress((p/'history-774/continuation/root/admission03-scope-reconciliation-046.json.gz').read_bytes()));b=json.loads(gzip.decompress((p/'history-774/continuation/root/admission03-scope-reconciliation-050.json.gz').read_bytes()));rr=gzip.decompress((p/'history-774/continuation/rig/admission03/source-range-registration.jsonl.gz').read_bytes());rq=gzip.decompress((p/'history-774/continuation/rig/admission03/source-qualifications.jsonl.gz').read_bytes());assert a['input_pins'][3]==pin['root046_registration_pin'] and b['input_pins'][2]==pin['root050_registration_pin'];assert a['input_pins'][3]['sha256']!=sha(b''.join(rr.splitlines(keepends=True)[:35]))==pin['later_registration_first35_sha256'];assert b['input_pins'][2]['sha256']!=sha(b''.join(rr.splitlines(keepends=True)[:139]))==pin['later_registration_first139_sha256'];assert a['input_pins'][4]['sha256']==sha(b''.join(rq.splitlines(keepends=True)[:31])) and b['input_pins'][3]['sha256']==sha(b''.join(rq.splitlines(keepends=True)[:45]))
for role,n in ((2626,80),(2627,81),(2628,82),(2627,83)):
 row=json.loads(rr.splitlines()[n-1]);obs=next(o for o in reg['files'][role]['admission03_scope_observations'] if o['row_1based']==n);assert row==obs['original_record']==pin['rows80_to_83'][n-80] and datetime.datetime.fromisoformat(a['at'])<datetime.datetime.fromisoformat(row['at'])<datetime.datetime.fromisoformat(b['at'])
adds=[json.loads(x) for x in gzip.decompress((p/'history-774/continuation/rig/admission03/additional-contracts.jsonl.gz').read_bytes()).splitlines()];assert all(any(z['id']=='RIG-ADMISSION03-ADDITIONAL-'+str(n).zfill(3) for z in adds) for n in (80,81,82,83))
groups=gzip.decompress((p/'history-774/continuation/rig/admission03/contract-groups.jsonl.gz').read_bytes()).splitlines();assert json.loads(groups[648])['id']=='RIG-ADMISSION03-CONTRACT-649' and json.loads(groups[651])['id']=='RIG-ADMISSION03-CONTRACT-652'
proof=load('git-proofs.json');objs={}
for oid,e in proof['objects'].items():
 bts=gzip.decompress((p/e['archive']).read_bytes());assert len(bts)==e['bytes'] and sha(bts)==e['sha256'] and hashlib.sha1(e['type'].encode()+b' '+str(len(bts)).encode()+b'\0'+bts).hexdigest()==oid;objs[oid]=(e['type'],bts)
def entries(b):
 out={};i=0
 while i<len(b):
  s=b.index(b' ',i);z=b.index(b'\0',s);out[b[s+1:z].decode()]=(b[i:s].decode(),b[z+1:z+21].hex());i=z+21
 return out
for x in proof['proofs']:
 typ,c=objs[x['revision']];assert typ=='commit';tree=c.splitlines()[0].split()[1].decode();assert tree==x['commit_tree'];parts=x['path'].split('/');assert len(parts)==len(x['chain'])
 for name,link in zip(parts,x['chain']):
  assert link['tree']==tree and link['name']==name;typ,body=objs[tree];assert typ=='tree';mode,tree=entries(body)[name];assert mode==link['mode'] and tree==link['object']
 body=(p/x['archive']).read_bytes();body=gzip.decompress(body) if x.get('encoding')=='gzip' else body;assert sha(body)==x['sha256'] and blob(body)==x['git_blob']==tree
report=load('report.json');baseline=load('reference/BASELINE-95E-20261110.json');assert len(proof['proofs'])==17 and len(report['lenses'])==13 and [e['lens'] for e in report['lenses']]==baseline['lens_names'];assert baseline['newly_assessed_ledger']['entries'][517]['id']=='ADLER-L11-FBX-RECORD-EXTENT-ADMISSION' and baseline['newly_assessed_ledger']['entries'][518]['id']=='ADLER-L11-FBX-DECODE-BUDGET-ADMISSION' and baseline['newly_assessed_ledger']['entries'][521]['id']=='L12-CLOSURE-FBX-NUM-ENUM-NATIVE-ADAPTER'
lookup={e['archive']:e for e in r['entries']}
def verify_report_ref(v):
 if isinstance(v,dict):
  if 'source' in v and 'lines' in v:
   e=lookup[v['source']];a,b=v['lines'];assert v['path']==e['path'] and v['sha256']==e['sha256'] and any(lo<=a<=b<=hi for lo,hi in e['registered_ranges'])
  for x in v.values():verify_report_ref(x)
 elif isinstance(v,list):
  for x in v:verify_report_ref(x)
verify_report_ref(report)
print(json.dumps({'status':'pass','verified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'registered_source_entries':len(r['entries']),'git_membership_proofs':len(proof['proofs']),'git_objects':len(objs),'historical_records':len(hist['records']),'selected_queue_entries':3,'historical_reference_hash_matches':len(checks),'lens_perspectives':13,'rig_registration_mismatches_retained':2,'published_registration_redaction_preserved':True,'qualification':'Evidence integrity only: native package and old/current lock identities, registered views and complements, 059/774 chronology, original/public identity and Git proofs. No product, macro, fixtures, test, build or probe execution.'},indent=2))
