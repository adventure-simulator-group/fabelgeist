#!/usr/bin/env python3
"""Preserve selected original/public variants and distinct historical prefix pins."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-35.json');bs={x['bundle_id']:x for x in cp['bundles']};prior=next(x for x in load('DELEGATED-EVIDENCE-CHECKPOINT-30.json')['bundles'] if x['bundle_id']=='sats-semantic-queue-2561-2562');old={x['path']:x for x in prior['members']}
b=bs['sats-macro-caller-2561-2562'];members={x['path']:x for x in b['members']};different=[]
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('history/prior-current/published-comparison.json'));assert d['archive']==prior['archive'] and d['archive_sha256']==prior['archive_sha256']==sha((R/prior['archive']).read_bytes())
 with tarfile.open(R/prior['archive']) as pub:
  for x in d['comparison']:
   m=old[x['member']];assert (m['original_sha256'],m['published_sha256'])==(x['original_sha256'],x['published_sha256']);assert sha(pub.extractfile(x['member']).read())==x['published_sha256'];assert members[x['original']]['original_sha256']==x['original_sha256'] and members[x['published_copy']]['original_sha256']==x['published_sha256'];assert x['byte_identical']==(x['original_sha256']==x['published_sha256'])
   if not x['byte_identical']:different.append(x['member'])
assert different==['historical-attribution.json'] and members['history/prior-current/historical-attribution.json']['changed']
b=bs['weapon-assets-2591-2592'];m={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 h=json.load(t.extractfile('historical-index.json'));root=json.loads(gzip.decompress(t.extractfile('history-774/continuation/root/admission03-scope-reconciliation-046.json.gz').read()));pins={(x['supplier'],x['record']):x['sha256'] for x in root['input_pins']};rig=gzip.decompress(t.extractfile('history-774/continuation/rig/admission03/source-range-registration.jsonl.gz').read());qual=gzip.decompress(t.extractfile('history-774/continuation/rig/admission03/source-qualifications.jsonl.gz').read());camera=gzip.decompress(t.extractfile('history-774/continuation/camera/admission03/source-range-registration.jsonl.gz').read());q=h['rig_registration_pin_qualification'];c=h['camera_registration_pin_qualification']
 assert pins[('rig','continuation/rig/admission03/source-range-registration.jsonl')]==q['root046_rig_registration_expected']!=sha(b''.join(rig.splitlines(keepends=True)[:35]))==q['later_first35_sha256']
 assert pins[('rig','continuation/rig/admission03/source-qualifications.jsonl')]==q['root046_qualification_expected']==sha(b''.join(qual.splitlines(keepends=True)[:31]))==q['later_first31_qualification_sha256']
 assert pins[('camera','continuation/camera/admission03/source-range-registration.jsonl')]==c['root046_camera_first7_pin']!=sha(b''.join(camera.splitlines(keepends=True)[:7]))==c['later_f82_first7_sha256']
 row=json.loads(camera.splitlines()[11]);assert row['id']=='CAMERA-ADM03-REG012-physical-p16' and root['at']<row['at'];assert 'does not establish absence' in c['qualification']
assert bs['placement-facing-terrain-2577-2580']['scope_original_roles']==[2579,2580] and 'does not contain' in bs['placement-facing-terrain-2577-2580']['role_scope_qualification']
print('Verified distinct prior redaction variants, 046 prefix mismatches and later camera-row chronology; missing originals and excluded role bodies are not inferred recovered.')
