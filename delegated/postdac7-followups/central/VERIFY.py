#!/usr/bin/env python3
"""Check the central evidence archive and bounded draft transcription.

The optional repository and publication anchors add revision/path and locked
native archive bindings. Captured unread text is not personal read credit.
"""
import argparse,hashlib,io,json,stat,subprocess,sys,tarfile,tomllib
from pathlib import Path,PurePosixPath
if sys.flags.optimize: raise RuntimeError('Unoptimized interpreter required')
p=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--repo',type=Path);ap.add_argument('--publication',type=Path);args=ap.parse_args()
h=lambda b:hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def check_git(rev,path,oid,raw):
 if args.repo:
  got=subprocess.check_output(['git','-C',str(args.repo),'rev-parse',rev+':'+path],text=True).strip()
  assert got==oid and subprocess.check_output(['git','-C',str(args.repo),'cat-file','blob',got])==raw
m=json.loads((p/'MANIFEST.json').read_bytes()); assert m['schema']=='726-postdac7-central-evidence-v1'
arc=(p/m['archive']).read_bytes();assert len(arc)==m['archive_bytes'] and h(arc)==m['archive_sha256']
expected={x['path']:x for x in m['members']};assert len(expected)==len(m['members']);data={}
with tarfile.open(fileobj=io.BytesIO(arc),mode='r:gz') as tar:
 for item in tar:
  name=str(PurePosixPath(item.name));assert not PurePosixPath(name).is_absolute() and '..' not in PurePosixPath(name).parts and name in expected and name not in data
  row=expected[name];assert item.isfile() and not item.issym() and not item.islnk() and stat.S_IMODE(item.mode)==int(row['mode'],8)
  raw=tar.extractfile(item).read();assert len(raw)==row['bytes'] and h(raw)==row['sha256'];data[name]=raw
assert set(data)==set(expected)
read=lambda name:json.loads(data[name])
def display(raw,start,end):
 lines=raw.decode().splitlines(keepends=True);assert 1<=start<=end<=len(lines)
 return ''.join(f'{i}: {lines[i-1]}' for i in range(start,end+1)).encode()
nfiles=ndisplays=0
for group in ['primary','dialogue','materials','prepared-stream','city-overlap-corrected']:
 c=read(group+'/CAPTURE.json');r=read(group+'/REGISTRATION.json');v=read(group+'/READ-RECEIPT.json')
 assert c['revision']==r['revision']==v['current_revision' if group=='materials' else 'revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and len(c['rows'])==len(r['planned'])
 for row,plan in zip(c['rows'],r['planned']):
  assert (row['label'],row['path'])==(plan['label'],plan['path']);raw=data[group+'/'+row['label']+'.source'];lines=raw.decode().splitlines(keepends=True)
  assert len(raw)==row['source_bytes'] and len(lines)==row['line_count'] and h(raw)==row['source_sha256'] and blob(raw)==row['git_blob']
  check_git(c['revision'],row['path'],row['git_blob'],raw)
  assert [(d['start'],d['end']) for d in row['displays']]==[tuple(x) for x in plan['ranges']]
  for d in row['displays']:
   actual=data[group+'/'+d['file']];assert actual==display(raw,d['start'],d['end']) and len(actual)==d['bytes'] and h(actual)==d['sha256'];ndisplays+=1
  nfiles+=1
 assert v['current_ranges' if group=='materials' else 'displayed_ranges']==sum(len(row['displays']) for row in c['rows'])
 assert v['current_lines' if group=='materials' else 'displayed_lines']==sum(d['end']-d['start']+1 for row in c['rows'] for d in row['displays'])
r=read('materials/HISTORICAL-REGISTRATION.json');c=read('materials/HISTORICAL-CAPTURE.json');raw=data['materials/historical-materials.source'];shown=data['materials/historical-materials-1-35.txt']
assert h(raw)==c['source_sha256'] and blob(raw)==c['git_blob_6533_and_e275'] and shown==display(raw,1,35) and h(shown)==c['display_sha256']
for rev in [r['revision'],'e2755a732c84e67296b5e3441a4259a84f461b2f']:check_git(rev,r['path'],c['git_blob_6533_and_e275'],raw)
r=read('prepared-stream/GUIDE-REGISTRATION.json');c=read('prepared-stream/GUIDE-CAPTURE.json');raw=data['prepared-stream/guide.source'];shown=data['prepared-stream/guide-112-127.txt']
assert h(raw)==c['source_sha256'] and blob(raw)==c['git_blob'] and shown==display(raw,112,127) and h(shown)==c['display_sha256'];check_git(r['revision'],r['path'],c['git_blob'],raw)
r=read('materials/NATIVE-REGISTRATION.json');c=read('materials/NATIVE-CAPTURE.json')
assert len(r['planned'])==len(c['rows'])==3
for row,plan in zip(c['rows'],r['planned']):
 assert (row['label'],row['crate'],row['source'])==(plan['label'],plan['crate'],plan['source']);raw=data['materials/'+row['label']+'.source'];assert h(raw)==row['source_sha256'] and len(raw.decode().splitlines(keepends=True))==row['source_lines']
 assert [(d['start'],d['end']) for d in row['displays']]==[tuple(x) for x in plan['ranges']]
 for d in row['displays']:
  actual=data['materials/'+d['file']];assert actual==display(raw,d['start'],d['end']) and h(actual)==d['sha256']
if args.publication:
 assert args.repo,'--publication requires --repo for the locked Cargo.lock binding'
 lock=subprocess.check_output(['git','-C',str(args.repo),'show',r['revision']+':Cargo.lock']);assert h(lock)==c['lockfile_sha256'];packages=tomllib.loads(lock.decode())['package']
 with tarfile.open(args.publication/r['outer_public_packet'],'r:gz') as outer:
  for row,plan in zip(c['rows'],r['planned']):
   pkg=[x for x in packages if x['name']==row['crate'] and x['version']==row['version']];assert len(pkg)==1
   archive=outer.extractfile('NATIVE-'+row['crate']+'.crate').read();assert h(archive)==pkg[0]['checksum']==row['archive_sha256']
   with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as inner:raw=inner.extractfile(row['crate']+'-'+row['version']+'/'+row['source']).read()
   assert raw==outer.extractfile(plan['outer_source']).read()==data['materials/'+row['label']+'.source']
report_raw=(p.parent/'ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E.json').read_bytes();assert h(report_raw)==m['report_sha256'];report=json.loads(report_raw)
assert report['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and report['parent_published_checkpoint']=='dac7d0055716053f245139971eb67ddfd87fa462'
drafts={}
for row in report['draft_predecessors']:
 raw=data['drafts/'+row['file']];assert h(raw)==row['sha256'];drafts[row['file']]=json.loads(raw)
a={x['queue_index']:x for x in report['assessments']};assert set(a)=={2708,2721,2734,2747,2750,2760,2773} and len(a)==7
for name in ['DRAFT-v7.json','MATERIAL-DRAFT-v2.json','STREAM-DRAFT-v5.json']:
 for item in drafts[name]['assessments']:assert a[item['queue_index']]==item
q=drafts['DIALOGUE-DRAFT-v4.json'];assert all(a[2721][key]==q[key] for key in a[2721])
ids=['relationship-texture-recipe-channel-install-95e','relationship-dialogue-multi-ballot-order-95e','relationship-dialogue-claim-occurrence-identity-95e','relationship-vista-cache-prepared-asset-95e','relationship-chunk-render-subapp-startup-95e','relationship-prepared-venue-take-retry-95e','relationship-regional-install-phase-error-shape-95e','relationship-regional-city-cross-request-failure-95e','relationship-selected-production-expect-rule-95e']
assert [x['id'] for x in report['ledger_relationships']]==ids and all(x['repair']=='not started' and x['validation']=='not performed' for x in report['ledger_relationships'])
assert report['counts']=={'new_partial_queue_assessments':7,'semantic_queue_closures':0,'new_planning_relationships':9,'accepted_unique_defect_increment':0,'lens_completions':0}
correction=json.loads((p/'REPORT-SUCCESSOR-CORRECTIONS.json').read_bytes());assert correction['original']=="ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E.json" and correction['original_sha256']==h(report_raw)
successor_raw=(p.parent/correction['successor']).read_bytes();assert h(successor_raw)==correction['successor_sha256'];successor=json.loads(successor_raw)
import copy
rebuilt=copy.deepcopy(report)
for change in correction['changes']:
 parts=change['path'].strip('/').split('/')
 if len(parts)==1:target=rebuilt;key=parts[0]
 else:
  assert len(parts)==3 and parts[0]=='ledger_relationships';target=next(x for x in rebuilt['ledger_relationships'] if x['id']==parts[1]);key=parts[2]
 assert target[key]==change['before'];target[key]=change['after']
assert correction['added_fields']==['supersedes_report','successor_corrections']
for key in correction['added_fields']:rebuilt[key]=successor[key]
assert rebuilt==successor and successor['supersedes_report']['sha256']==h(report_raw) and successor['assessments']==report['assessments'] and successor['counts']==report['counts']
c2=json.loads((p/'REPORT-SUCCESSOR-CORRECTIONS-v2.json').read_bytes());assert c2['original']==correction['successor'] and c2['original_sha256']==h(successor_raw)
v3raw=(p.parent/c2['successor']).read_bytes();assert h(v3raw)==c2['successor_sha256'];v3=json.loads(v3raw);rebuilt=copy.deepcopy(successor)
for change in c2['changes']:
 parts=change['path'].strip('/').split('/')
 if len(parts)==1:target=rebuilt;key=parts[0]
 else:
  assert len(parts)==3 and parts[0]=='ledger_relationships';target=next(x for x in rebuilt['ledger_relationships'] if x['id']==parts[1]);key=parts[2]
 assert target[key]==change['before'];target[key]=change['after']
assert rebuilt==v3 and v3['supersedes_report']['sha256']==h(successor_raw) and v3['assessments']==report['assessments'] and v3['counts']==report['counts']
print('Verified',len(data),'archived members,',nfiles,'captured source files,',ndisplays,'displays, seven draft transcriptions nine relationship IDs and both exact corrected successors.')
print('Git revision/path bindings:',bool(args.repo),'Cargo.lock/native archive binding:',bool(args.publication),'; no product execution or independent custody inferred.')
