#!/usr/bin/env python3
"""Verify central role and ledger links, not audit completion or product behavior."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda p:json.loads((R/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
def normalize(ranges):
    out=[]
    for a,b in sorted(ranges):
        assert 1<=a<=b
        if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
        else:out.append([a,b])
    return out
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-9.json');bundles={x['bundle_id']:x for x in cp['bundles']}
full=load('ASSESSMENT-DELEGATED-SOURCE-ROLES-9.json')['historical_source_roles'];part=load('ASSESSMENT-DELEGATED-PARTIAL-9.json')['historical_source_roles'];allroles=full+part
assert len(full)==28 and len(part)==2 and {x['original_index'] for x in part}=={2434,2452}
assert {x['original_index'] for x in allroles}=={2354,*range(2431,2460)}
work=load('COVERAGE-L01-ROLES.json')['roles']; data={}
for name,b in bundles.items():
    with tarfile.open(R/b['archive'],'r:gz') as t:
        names=set(t.getnames());need={b['report_archive_member']}
        for n in ('read-receipt.json','personal-read-receipt.json'):
            if n in names:need.add(n)
        data[name]={n:json.load(t.extractfile(n)) for n in need}
    assert data[name][b['report_archive_member']]==load(b['report_path'])
for x in allroles:
    i=x['original_index']; matches=[b for b in cp['bundles'] if i in b['scope_original_roles']];assert len(matches)==1
    b=matches[0];name=b['bundle_id'];report=data[name][b['report_archive_member']];item=work[i];cc=x['current_comparison']
    assert x['source_id']==item['source_id'] and x['sha256']==item['historical_sha256'] and x['new_personal_read_attribution']==b['agent']
    assert x['evidence_archive']==b['archive'] and x['delegate_report']==b['report_path'] and x['receipt_member'] in data[name]
    if name in ('scene-tests','p17-orientation','p17-capture','p17-biomechanics','p17-metrics','p17-height'):
        receipt=data[name]['read-receipt.json'];old=[r for r in receipt['reads'] if r.get('role',r.get('original_role'))==i and r['revision']==x['snapshot_head']]
        now=[r for r in receipt['reads'] if r.get('role',r.get('original_role'))==i and r['revision']==cc['source_revision']]
        assert len(old)==1 and normalize(old[0]['actual_display_ranges'])==x['new_personal_read_ranges']
        assert old[0]['actual_display_ranges']==x['actual_personal_display_chunks']
        assert (now[0]['actual_display_ranges'] if now else [])==cc['actual_personal_display_chunks']
    elif name=='quality-rules':
        rec=next(r for r in data[name]['personal-read-receipt.json']['original_roles'] if r['original_role']==i)
        assert rec['historical_ranges']==x['actual_personal_display_chunks'] and rec['current_ranges']==cc['actual_personal_display_chunks']
    elif name in ('report-p17','p17-continuity','p17-metric-facing'):
        rows=data[name]['personal-read-receipt.json']['assigned_personal_reads']
        old=[r for r in rows if r.get('role')==i and r['revision']==x['snapshot_head']]
        now=[r for r in rows if r.get('role')==i and r['revision']==cc['source_revision']]
        assert len(old)==1 and old[0]['ranges']==x['actual_personal_display_chunks']
        assert (now[0]['ranges'] if now else [])==cc['actual_personal_display_chunks']
    elif name=='ahoy':
        rec=next(r for r in report['role_records'] if r['original_index']==i)
        assert rec['historical_body_personal_display_ranges']==[] and x['new_personal_read_revision']==cc['source_revision']
        assert rec['fresh_personally_displayed_body_ranges']==x['actual_personal_display_chunks']==cc['actual_personal_display_chunks'] and cc['byte_identical_to_historical']
    elif name in ('viewer-capture','viewer-limits'):
        rec=next(r for r in report['role_records'] if r['original_index']==i)
        assert rec['fresh_personal_snapshot_ranges']==x['actual_personal_display_chunks'] and rec['fresh_personal_current_ranges']==cc['actual_personal_display_chunks']
    elif name=='p17-landing':
        rec=next(r for r in report['role_records'] if r['original_index']==i)['personal_display_receipt']
        assert rec['fresh_p17_actual_display_ranges']==x['actual_personal_display_chunks']
        assert rec.get('fresh_current_actual_display_ranges',rec.get('current_actual_display_ranges',[]))==cc['actual_personal_display_chunks']
    elif name=='p17-peaks-pose-hair-ops':
        rec=next(r for r in report['roles'] if r['original_index']==i)['actual_personal_display']
        assert rec['p17_actual_display_ranges']==x['actual_personal_display_chunks'] and rec['current_actual_display_ranges']==cc['actual_personal_display_chunks']
    else: raise AssertionError(name)
    assert normalize(x['actual_personal_display_chunks'])==x['new_personal_read_ranges']
    assert normalize(cc['actual_personal_display_chunks'])==cc['personal_ranges']
    if i in (2434,2452):assert item['status']=='source-metadata-partial' and x['historical_unread_ranges'] and cc['unread_ranges']
    else:assert item['status']=='full-historical-body-trace-open' and x['new_personal_read_ranges']==[[1,x['lines']]]
    if cc['status']=='absent':assert not cc['actual_personal_display_chunks'] and cc['sha256'] is None
    else:
        assert cc['status']=='present' and cc['personal_ranges'] and cc['git_blob'] and cc['sha256']
        if i not in (2434,2452): assert cc['personal_ranges']==[[1,cc['lines']]]
    if x['snapshot']=='report-p17': assert x['snapshot_head']=='90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696'
rec=load('ASSESSMENT-DELEGATED-RECONCILIATION-9-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger}
assert len(ids)==len(ledger) and len(rec['delegated_observations'])==55 and len(rec['new_qualified_ledger_rows'])==23
for ident in rec['new_qualified_ledger_rows']:
    row=ids[ident];assert row['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-9-95E.json' and row['repair']=='not started' and row['validation']=='not performed'
for ident in ('REVIEW-BOUNCY-SPRING-FREQUENCY-OVERFLOW-001','REVIEW-CAPTURE-PREAPPLIED-READINESS-TIMEOUT-001','REVIEW-CAPTURE-SCENARIO-TIMESTAMP-RATE-001'):
    assert 'ASSESSMENT-DELEGATED-RECONCILIATION-9-95E.json' in ids[ident]['further_current_evidence']
prog=load('CONTINUATION-PROGRESS-95E.json');assert prog['lenses_complete']==0 and prog['queue_records_semantically_reconciled']==0 and prog['new_current_ledger_candidates']==len(ledger)
assert rec['repair']=='not started' and rec['validation']=='not performed'
print('Verified 28 full and 2 partial delegated role scopes, 55 observations and 23 qualified ledger rows; no lens or queue closure asserted.')
