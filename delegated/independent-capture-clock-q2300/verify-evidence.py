#!/usr/bin/env python3
"""Verify a bounded capture-clock review; never execute project/native source."""
import argparse,datetime,gzip,hashlib,io,json,subprocess,tarfile,tomllib
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--git-repo',type=Path);args=ap.parse_args();pub=args.publication.resolve()
sha=lambda b:hashlib.sha256(b).hexdigest()
oid=lambda kind,b:hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(path):return json.loads(path.read_bytes())
def safe(rel):
 p=PurePosixPath(rel);assert not p.is_absolute() and '..' not in p.parts;return p
reg=load(root/'REGISTRATION.json');ext=load(root/'EXT-REGISTRATION.json');histreg=load(root/'HIST-EQUALITY-REGISTRATION.json');transport=load(root/'TRANSPORT-REGISTRATION.json');receipt=load(root/'READ-RECEIPT.json');review=load(root/'REVIEW.json')
assert sha((root/'REGISTRATION.json').read_bytes())==receipt['registration_sha256']
assert sha((root/'EXT-REGISTRATION.json').read_bytes())==receipt['extension_registration_sha256']
assert sha((root/'HIST-EQUALITY-REGISTRATION.json').read_bytes())==receipt['historical_equality_registration_sha256']
for obj in (reg,ext,histreg):assert datetime.datetime.fromisoformat(obj['registered_utc'])<datetime.datetime.fromisoformat(receipt['recorded_utc'])
for n,row in receipt['displays'].items():
 raw=(root/safe(n)).read_bytes();assert (len(raw),sha(raw))==(row['bytes'],row['sha256'])
assert 'not prospectively registered' in receipt['chronology_qualifications'][0]
assert 'not credited as wholly displayed' in receipt['chronology_qualifications'][1]
for e in load(root/'EXTERNAL-DEPENDENCIES.json')['files']:
 raw=(pub/safe(e['path'])).read_bytes();assert (len(raw),sha(raw))==(e['bytes'],e['sha256'])
stage=pub/'delegated/capture-clock-q2300';proof=load(stage/'GIT-TREE-PROOFS.json')
raw=(pub/'delegated/xpbd-incidence-address-hair-ops-evidence.tar.gz').read_bytes()
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
 f=tar.extractfile('git-objects/95e63628f9d70b8dd6e6fede64334696c6995009.commit.gz');assert f is not None;current=gzip.decompress(f.read())
old=gzip.decompress((pub/'recovered/e275-investigation-model-git/e2755a732c84e67296b5e3441a4259a84f461b2f.commit.gz').read_bytes())
commits={}
for expected,body in [('95e63628f9d70b8dd6e6fede64334696c6995009',current),('e2755a732c84e67296b5e3441a4259a84f461b2f',old)]:
 assert oid('commit',body)==expected;commits[expected]=body.splitlines()[0].split(b' ',1)[1].decode()
objects={}
for h,info in proof['objects'].items():
 raw=gzip.decompress((stage/safe(info['file'])).read_bytes());assert len(raw)==info['bytes'] and sha(raw)==info['sha256'] and oid('tree',raw)==h
 pos=0;members={}
 while pos<len(raw):
  end=raw.index(b'\0',pos);mode,name=raw[pos:end].split(b' ',1);child=raw[end+1:end+21].hex();pos=end+21;assert name.decode() not in members;members[name.decode()]=(mode.decode(),child)
 assert pos==len(raw);objects[h]=members
def check_tree(pr,body):
 current=commits[pr['revision']];assert current==pr['root_tree']
 assert [x['name'] for x in pr['chain']]==pr['path'].split('/')
 for part in pr['chain']:
  assert part['tree']==current and objects[current][part['name']]==(part['mode'],part['object']);current=part['object']
 assert current==pr['git_blob']==oid('blob',body)
lookup={x['tag']:x for x in reg['entries']};bodies={}
for e in reg['entries']:
 if e['kind']!='project-source':continue
 body=(pub/safe(e['external_capture'])).read_bytes();assert (len(body),sha(body),len(body.splitlines()))==(e['bytes'],e['sha256'],e['eof'])
 pr=next(x for x in proof['proofs'] if x['revision']==e['revision'] and x['path']==e['path']);check_tree(pr,body);assert oid('blob',body)==e['git_blob'];bodies[e['tag']]=body
native=lookup['bevy_fixed'];archive=(pub/safe(native['archive'])).read_bytes();assert sha(archive)==native['archive_sha256']
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as tar:
 f=tar.extractfile(native['member']);assert f is not None;native_body=f.read()
assert native_body==(pub/safe(native['external_capture'])).read_bytes() and (len(native_body),sha(native_body),len(native_body.splitlines()))==(native['bytes'],native['sha256'],native['eof']);bodies['bevy_fixed']=native_body
lock=lookup['lock'];lock_body=gzip.decompress((pub/safe(lock['external_capture'])).read_bytes());assert (len(lock_body),sha(lock_body),oid('blob',lock_body))==(lock['bytes'],lock['sha256'],lock['git_blob'])
check_tree(next(x for x in proof['proofs'] if x['revision']==lock['revision'] and x['path']=='Cargo.lock'),lock_body)
packages=tomllib.loads(lock_body.decode())['package'];selected=[{k:r[k] for k in ('name','version','source','checksum') if k in r} for r in packages if r['name']=='bevy_time' and r['version']=='0.19.1'];assert len(selected)==1 and selected[0]['checksum']==native['archive_sha256']
assert load(root/'HISTORY-E.txt')=={'source_revision':lock['revision'],'structural_selection':selected}
chunks={'DISPLAY-A.txt':[('clock',[(1,58)]),('bevy_fixed',[(75,90),(118,135)])],
'DISPLAY-B.txt':[('rate',[(1,86)]),('measurement',[(40,65)])],
'DISPLAY-C.txt':[('core_elapsed',[(1,90)])],
'DISPLAY-D.txt':[('main',[(68,105)]),('viewer',[(89,100),(121,177)])],
'DISPLAY-E.txt':[('locomotion_config',[(1,15)]),('runtime',[(1,64)]),('locomotion',[(84,100)])],
'DISPLAY-F.txt':[('capture',[(184,220),(360,400)])],
'DISPLAY-G.txt':[('capture',[(401,436)]),('procedural_clock',[(1,36)]),('schedule',[(10,30)])],
'DISPLAY-H.txt':[('subject_motion',[(1,55)])],
'DISPLAY-I.txt':[('main',[(106,132)]),('capture',[(437,490)])]}
for name,parts in chunks.items():
 out=[]
 for tag,ranges in parts:
  lines=bodies[tag].decode().splitlines()
  for lo,hi in ranges:out.extend(f'{tag}:{n:04}: {lines[n-1]}\n' for n in range(lo,hi+1))
 assert (root/name).read_text()==''.join(out)
queue=load(stage/'QUEUE-2300-RECONSTRUCTED.json')['value'];keys=('contract_id','kind','lens','canonical_owner','first_failed_contract_or_unresolved_question','producer_api_retained_state_consumer_native_trace','remaining_evidence','dependencies','closure_condition','disposition','policy_decisions_remaining','later_repair_acceptance_requirements','named_disposition_recorded','audit_discovery_closed_within_exact_named_scope','all_lenses_or_coarse_parent_closed','global_exact_alias_inventory_complete','repair_runtime_passed','body_or_population_credit','implementation_gate_complete');expected={k:queue[k] for k in keys};expected['selected_source_identities']=[{k:x[k] for k in ('revision_or_explicit_transport_qualification','source_sha256','assessed_ranges','contract_relevant_ranges','read_attribution','body_credit_from_queue')} for x in queue['exact_source_identities_and_assessed_ranges']];assert load(root/'HISTORY-A.txt')==expected
raw=(pub/safe(transport['input'])).read_bytes();assert (len(raw),sha(raw))==(transport['bytes'],transport['sha256']);manifest=json.loads(gzip.decompress(raw));m={x['path']:x for x in manifest['entries']}
history={}
for e in reg['entries']:
 if e['kind']!='recovered-history-selection':continue
 data=(pub/safe(e['external_capture'])).read_bytes();assert (len(data),sha(data))==(e['bytes'],e['sha256']);rel=e['external_capture'].split('/history-exact/',1)[1];t=m[rel];assert (len(data),sha(data),oid('blob',data))==(t['bytes'],t['sha256'],t['git_blob']);history[rel]=json.loads(data)
names=('parallel-l11-20261029/READ-168.json','parallel-l11-20261029/READ-169.json','segovb/READ-L11-704.json','segovb/READ-L11-705.json');expected={}
for name in names:
 x=history[name];r=x['rows'][0];expected[name]={'kind':x.get('kind'),'source_revision':x.get('revision',x.get('source_revision')),'accepted_unique_count_change':x.get('accepted_unique_count_change'),'runtime_validation':x.get('native_runtime_validation',x.get('runtime_validation')),'implementation_gate_complete':x.get('implementation_gate_complete'),'first_row':{k:r[k] for k in ('sha256','actual_EOF','fresh_personal_body_ranges','attribution')}}
assert load(root/'HISTORY-B2.txt')==expected
lrows=load(stage/'LEDGER-SELECTION-RECONSTRUCTED.json')['entries']['committed'];keys=('id','lens','classification','historical_related_records','relationship_qualification','discovery','repair','validation','record');expected=[]
for x in lrows:
 r=x['value'];expected.append({'index':int(x['pointer'].split('/')[-1]),**{k:r[k] for k in keys if k in r}})
expected.sort(key=lambda x:x['index']);assert load(root/'HISTORY-C.txt')==expected
scenario=load(stage/'prior/scenario-clock-assessment.json');p16=load(stage/'prior/p16-body-values-sample-rate-report.json');item=next(x for x in scenario['conditional_findings'] if x['id']=='REVIEW-CAPTURE-SCENARIO-TIMESTAMP-RATE-001');ak=('id','classification','trace','symbolic_case','connected_presentation','first_failed_contract','canonical_owner','discovery','repair','validation');bk=('id','kind','first_contract','condition','relationships');expected={'scenario_assessment':{'source_revision':scenario['source_revision'],'scope':scenario['scope'],'selected_finding':{k:item[k] for k in ak if k in item},'remaining':scenario['remaining']},'p16_report':{'author':p16['author'],'scope':p16['scope'],'selected_findings':[{k:r[k] for k in bk if k in r} for r in p16['findings_and_questions'] if r['id'] in ('P16-POSE-BAKE-UNBOUNDED-GRID-WORK','P16-SAMPLE-RATE-SPAN-CONVERSION')]}};assert load(root/'HISTORY-D.txt')==expected
for e in reg['entries']:
 if e['kind'] not in ('previous-report-selection','recovered-history-selection'):continue
 raw=(pub/safe(e['external_capture'])).read_bytes();assert (len(raw),sha(raw))==(e['bytes'],e['sha256'])
report=load(stage/'REPORT.json')
for n,keys in [('PARENT-READ-A.txt',('recommended_scoped_disposition','producer_and_admission','retained_state_and_selected_consumers','native_package')),('PARENT-READ-B.txt',('historical_provenance','related_records','remaining_open','read_scope_and_recovery','repair','product_validation'))]:assert load(root/n)=={k:report[k] for k in keys}
assert sha((stage/'REPORT.json').read_bytes())==review['reviewed_report']['sha256']
expected=[]
for e in histreg['entries']:
 body=bodies[e['tag']]
 for revision in (e['historical_revision'],e['current_revision']):check_tree(next(x for x in proof['proofs'] if x['revision']==revision and x['path']==e['path']),body)
 expected.append({'tag':e['tag'],'historical_revision':e['historical_revision'],'current_revision':e['current_revision'],'path':e['path'],'historical_sha256':sha(body),'current_sha256':sha(body),'bytes_equal':True,'bytes':len(body)})
assert load(root/'HISTORY-F.txt')==expected
if args.git_repo:
 def show(revision,path):return subprocess.check_output(['git','--no-replace-objects','-C',str(args.git_repo),'show',revision+':'+path])
 for e in reg['entries']:
  if e['kind'] in ('historical-record-selection','project-lock-selection'):
   data=show(e['revision'],e['path']);assert (len(data),sha(data),oid('blob',data))==(e['bytes'],e['sha256'],e['git_blob'])
 original=json.loads(show('56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e','baseline-059/queue.json'))['entries'][2300];assert original==queue
 ledger=json.loads(show('8608c88ad0cd75ca188ce03908c7662c1aa436e8','BASELINE-95E-20261110.json'))['newly_assessed_ledger']['entries'];assert all(ledger[int(x['pointer'].split('/')[-1])]==x['value'] for x in lrows)
 print('Original 059 queue selection and committed 8608 ledger comparison checked against supplied local Git objects.')
else:print('No Git repository supplied; original full 059 queue and committed ledger comparisons were not repeated.')
print('Checked commit-anchored source and native archive identities, exact registered displays, selected historical transport identities and reconstructed record selections.')
print('The prior unregistered report exposure, truncated historical display, native/runtime behavior and semantic conclusions remain separately qualified.')
