#!/usr/bin/env python3
import hashlib,json,pathlib
p=pathlib.Path(__file__).parent;sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda n:json.loads((p/n).read_bytes());m=load('MANIFEST.json');files={x.relative_to(p).as_posix() for x in p.rglob('*') if x.is_file() and x.relative_to(p).as_posix()!='MANIFEST.json'};assert len(m['files'])==m['file_count'] and files=={x['path'] for x in m['files']}
for x in m['files']:
 b=(p/x['path']).read_bytes();assert (len(b),sha(b))==(x['bytes'],x['sha256'])
for x in load('reference-index.json')['records']:
 b=(p/x['saved']).read_bytes();assert (len(b),sha(b))==(x['bytes'],x['sha256'])
r=load('review-results.json');report=load('report.json');assert report['reviewer']=='/root/mhr_character' and report['reviewed_snapshot']['index_sha256']==sha((p/'reference/checkpoint38-index.json').read_bytes()) and r['status']=='pass_with_qualifications';assert r['counts']['top_members']==393 and r['counts']['changed_top']==22 and r['counts']['redactions']==150 and r['redaction_categories']=={'local_user_absolute':129,'temporary_absolute':15,'prose_false_positives':6} and len(r['false_positive_redactions'])==6
assert sha((p/'reference/prior-checkpoint17-manifest.json').read_bytes())==r['prior_manifest_binding']['sha256'];assert sha((p/'reference/committed-baseline-at-56f.json').read_bytes())==r['committed_vs_working_baseline']['committed_sha256'];assert sha((p/'reference/working-baseline-observed.json').read_bytes())==r['committed_vs_working_baseline']['observed_working_sha256']
print('PASS: frozen review packet and captured metadata; stage, publication and product are not touched.')
