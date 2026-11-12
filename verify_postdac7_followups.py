#!/usr/bin/env python3
"""Verify the time-bound post-dac7 evidence checkpoint, not product behavior."""
import hashlib,json,subprocess,sys
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
R=Path(__file__).resolve().parent;P=R/'delegated/postdac7-followups';H=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes())
cat=load(P/'CATALOG.json');assert cat['schema']=='726-postdac7-followup-catalog-v1' and cat['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cat['parent_checkpoint']=='dac7d0055716053f245139971eb67ddfd87fa462'
expected={'CATALOG.json'}
for row in cat['bundle_files']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;p=P/n;assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);assert bool(p.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111);expected.add(row['path'])
assert {p.relative_to(P).as_posix() for p in P.rglob('*') if p.is_file() and not p.is_symlink()}==expected
for row in cat['root_files']:
 n=PurePosixPath(row['path']);assert len(n.parts)==1;p=R/n;assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);assert bool(p.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111)
for script in ['central/VERIFY.py','checkpoint-dac7/VERIFY.py','transport/VERIFY.py','review-transport/VERIFY.py','review-extra-transport/VERIFY.py','transport-replay/VERIFY.py','extra-transport-replay/VERIFY.py','q2910-packet/VERIFY.py','q2945-packet/verify.py','later-q2968/verify.py','successor-review/VERIFY.py','portable-guide/VERIFY-GUIDE.py','portable-guide-successor/VERIFY-GUIDE.py','restored-evidence/VERIFY.py','control-restored/VERIFY.py','prescan/VERIFY.py','final-stage-scan/VERIFY.py','independent-replay/VERIFY.py','encrypted-originals/VERIFY.py','post-encryption-control/VERIFY.py','application/VERIFY.py']:
 subprocess.run([sys.executable,'-B',str(P/script)],cwd=R,check=True)
for name in ['ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E.json','ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v2.json','ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v3.json','MAIN-RECHECK-20261112-2101.json']:
 assert (R/name).read_bytes()==(P/name).read_bytes()
report=load(R/'ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v3.json');assess=report['assessments'];rels=report['ledger_relationships'];assert [x['queue_index'] for x in assess]==[2708,2721,2734,2747,2750,2760,2773] and len(rels)==9
snap=load(R/'FOLLOWUP-POSTDAC7-EVIDENCE.json');c=snap['observed_counts'];assert (c['lenses'],c['lenses_complete'],c['queue_records'],c['queue_semantically_reconciled'],c['current_ledger_entries_and_relationships'],c['queue_partial'],c['queue_unassessed'])==(13,0,3713,0,1093,1348,2365)
assert snap['parent_published_checkpoint']==cat['parent_checkpoint'] and snap['bundle_catalog_sha256']==H((P/'CATALOG.json').read_bytes())
queue=load(R/'RECONCILIATION-059.json')['entries'];history=load(R/'baseline-059/queue.json')['entries'];assert len(queue)==len(history)==3713
for a,selected in zip(assess,snap['queue_selections']):
 i=a['queue_index'];assert selected['index']==i and selected['contract_id']==a['historical_contract_id']==queue[i]['contract_id']==history[i]['contract_id'];assert selected['current_reconciliation_at_snapshot'].startswith('partial ') and queue[i]['current_reconciliation']!='unassessed'
ledger=load(R/'BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids=[x['id'] for x in ledger];assert len(ids)==len(set(ids)) and len(ids)>=1093 and snap['added_ledger_relationship_ids']==[x['id'] for x in rels] and all(x['id'] in ids for x in rels)
progress=load(R/'CONTINUATION-PROGRESS-95E.json');assert progress['new_current_ledger_candidates']==len(ledger) and progress['lens_count']==13 and 0<=progress['lenses_complete']<=13 and 0<=progress['queue_records_semantically_reconciled']<=3713
pending=load(P/'PENDING-DELEGATES.json');assert len(pending['packets'])==snap['pending_delegated_packets'];assert len({x['queue_index'] for x in pending['packets']})==len(pending['packets'])
new_labels={x['label'] for x in load(P/'encrypted-originals/MANIFEST.json')['objects']};old_labels={x['label'] for x in load(R/'delegated/post0a24-followups/encrypted-originals/MANIFEST.json')['objects']}
for x in pending['packets']:assert x['encrypted_collection'] in new_labels|old_labels and (R/x['public_transport']).exists() and x['central_status'].startswith('pending')
incident=load(P/'COLLABORATION-INCIDENT.json');assert incident['schema']=='726-postdac7-collaboration-incident-v1' and incident['reviewer']=='/root/mhr_character'
later=load(P/'POST-FREEZE-PACKETS.json');assert later['frozen_pending_count']==len(pending['packets'])==10 and [x['queue_index'] for x in later['packets']]==[2968]
assert snap['repair']=='not started' and snap['product_validation']=='not performed'
print('Verified time-bound post-dac7 evidence and current record identities; later state changes require their own provenance. No product build, test or probe ran.')
