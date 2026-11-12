#!/usr/bin/env python3
"""Verify a time-bound audit evidence checkpoint, not product behavior."""
import hashlib,json,subprocess,sys
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
R=Path(__file__).resolve().parent;P=R/'delegated/post0a24-followups';H=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes())
cat=load(P/'CATALOG.json');assert cat['schema']=='726-post0a24-followup-catalog-v1' and cat['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cat['parent_checkpoint']=='0a2466a7485388575ef272e0a8355b9714204a37'
expected={'CATALOG.json'}
for row in cat['bundle_files']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;p=P/n;assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);assert bool(p.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111);expected.add(row['path'])
assert {p.relative_to(P).as_posix() for p in P.rglob('*') if p.is_file() and not p.is_symlink()}==expected
for row in cat['root_files']:
 n=PurePosixPath(row['path']);assert len(n.parts)==1;p=R/n;assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);assert bool(p.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111)
for script,args in [('central/VERIFY-v4.py',['--reports-root',str(R)]),('checkpoint-0a24/VERIFY.py',[]),('transport/VERIFY.py',[]),('transport-extra/VERIFY.py',[]),('encrypted-originals/VERIFY.py',[]),('post-encryption-control/VERIFY.py',[]),('PRIVATE-BINDINGS-v2.py',[])]:
 subprocess.run([sys.executable,'-B',str(P/script),*args],cwd=R,check=True)
er=load(P/'encrypted-replay/RESULT.json')['results'];cr=load(P/'control-replay/RESULT.json')['results'];assert all(er[k]['exit']==0 for k in ('keyless','keyed','bindings')) and all(cr[k]['exit']==0 for k in ('keyless','keyed'))
obs=load(R/'MAIN-RECHECK-20261112-2030.json');assert obs['ref']=='refs/heads/main' and obs['observed_revision']==cat['source_revision']
reports=[load(R/'ASSESSMENT-CENTRAL-FOLLOWUPS-15B-95E.json'),load(R/'ASSESSMENT-CENTRAL-FOLLOWUPS-15C-95E.json')];assess=[a for r in reports for a in r['assessments']];relationships=[x for r in reports for x in r['ledger_relationships']];assert [a['queue_index'] for a in assess]==list(range(2355,2370)) and len(relationships)==6
snap=load(R/'FOLLOWUP-POST0A24-EVIDENCE.json');counts=snap['observed_counts'];assert (counts['lenses'],counts['lenses_complete'],counts['queue_records'],counts['queue_semantically_reconciled'],counts['current_ledger_entries_and_relationships'],counts['queue_partial'],counts['queue_unassessed'])==(13,0,3713,0,1084,1341,2372)
assert snap['parent_published_checkpoint']==cat['parent_checkpoint'] and snap['bundle_catalog_sha256']==H((P/'CATALOG.json').read_bytes())
q=load(R/'RECONCILIATION-059.json')['entries'];historical=load(R/'baseline-059/queue.json')['entries'];assert len(q)==len(historical)==3713
for a,selected in zip(assess,snap['queue_selections']):
 i=a['queue_index'];assert selected['index']==i and selected['contract_id']==a['historical_contract_id']==q[i]['contract_id']==historical[i]['contract_id'];assert selected['current_reconciliation_at_snapshot'].startswith('partial ') and q[i]['current_reconciliation']!='unassessed'
ledger=load(R/'BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids=[x['id'] for x in ledger];assert len(ids)==len(set(ids)) and len(ids)>=counts['current_ledger_entries_and_relationships'];assert snap['added_ledger_relationship_ids']==[x['id'] for x in relationships] and all(x['id'] in ids for x in relationships)
progress=load(R/'CONTINUATION-PROGRESS-95E.json');assert progress['new_current_ledger_candidates']==len(ledger) and progress['lens_count']==13 and 0<=progress['lenses_complete']<=13 and 0<=progress['queue_records_semantically_reconciled']<=3713
pending=load(P/'PENDING-DELEGATES.json');encrypted=load(P/'encrypted-originals/MANIFEST.json');labels={x['label'] for x in encrypted['objects']};assert len(pending['packets'])==11
for x in pending['packets']:assert x['encrypted_collection'] in labels and (P/x['public_transport']).exists() and x['central_status'].startswith('pending')
assert snap['repair']=='not started' and snap['product_validation']=='not performed'
print('Verified the time-bound post-0a24 evidence and current record identities; later status changes require their own provenance. No product build, test or probe ran.')
