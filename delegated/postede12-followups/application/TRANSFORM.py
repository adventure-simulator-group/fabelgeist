#!/usr/bin/env python3
"""Apply reviewed partial audit records; does not execute product code."""
import copy,json
NAMES=['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']
REPORT='ASSESSMENT-CENTRAL-FOLLOWUPS-18-95E.json'
PENDING='delegated/postede12-followups/PENDING-DELEGATES.json'
RECHECK='MAIN-RECHECK-20261113-0140.json'
REV='95e63628f9d70b8dd6e6fede64334696c6995009'
PARENT='ede12d3f9f19d284abbc669b5ae38b814a585f77'
def render(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode()
def transform(before,report,pending,obs):
 assert set(before)==set(NAMES)
 b,p,q=(json.loads(before[n]) for n in NAMES[:3]);doc=before[NAMES[3]].decode()
 assert b['source_revision']==report['source_revision']==pending['source_revision']==obs['observed_revision']==REV
 assert report['parent_published_checkpoint']==pending['parent_checkpoint']==PARENT
 assert obs['ref']=='refs/heads/main' and obs['observed_tree']=='1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
 a=[x['record'] for x in report['partial_queue_assessments']];rels=[x['record'] for x in report['ledger_relationships']]
 assert [x['queue_index'] for x in a]==[3064,3207] and len(rels)==2 and len(pending['packets'])==28
 entries=b['newly_assessed_ledger']['entries'];oldids={x['id'] for x in entries};ids=[x['id'] for x in rels]
 assert len(entries)==len(oldids)==1099 and len(set(ids))==2 and not set(ids)&oldids
 for row in rels:
  x=copy.deepcopy(row);assert x['repair']=='not started' and x['validation']=='not performed';x['record']=REPORT;entries.append(x)
 assert len(entries)==1101 and len(q['entries'])==3713
 for x in a:
  row=q['entries'][x['queue_index']];assert row['contract_id']==x['historical_contract_id'] and row['historical_lens']==x['historical_lens']==1 and row['current_reconciliation']=='unassessed'
  row['current_reconciliation']='partial current 95e source assessment and historical relationship; see '+REPORT+'; no semantic queue or lens closure and no checkpoint3821 attribution'
  row['further_current_evidence']=['delegated/postede12-followups/late-transport/combined-draft-review.tar.gz','delegated/postede12-followups/encrypted-originals/README.md']
 for x in pending['packets']:
  row=q['entries'][x['queue_index']];assert row['contract_id']==x['historical_contract_id'] and row['historical_lens']==x['historical_lens'] and row['current_reconciliation']=='unassessed'
 assert sum(x['current_reconciliation']=='unassessed' for x in q['entries'])==2357
 assert sum(x['current_reconciliation'].startswith('partial') for x in q['entries'])==1356
 assert p['new_current_ledger_candidates']==1099 and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
 p['new_current_ledger_candidates']=1101
 p['central_postede12_bounded_reconciliations']=REPORT+'; two partial assessments, two planning relationships, no semantic closure or accepted unique-defect increment.'
 p['postede12_pending_delegated_packets']=PENDING+'; 28 later source/lens obligations are pending central reconciliation, with URL erratum and original report separately retained.'
 p['checkpoint_ede12_same_machine_recovery']='delegated/postede12-followups/RECOVERY-EDE12.json; 167 captured path identities matched and the evidence transcript was byte-identical in a separate same-machine checkout.'
 p['postede12_private_recovery']='delegated/postede12-followups/RESTORATION-RECEIPT.json; same-machine primary key, previous q2812 anchors and named nested keys remain dependencies.'
 p['main_recheck_20261113_0140']=RECHECK
 p['current_main_qualification']='GitHub connector observed main at 95e at 2026-11-13 01:40:02 UTC; this is a point-in-time observation and main may advance. See '+RECHECK+'.'
 b['latest_main_recheck']=RECHECK
 marker='product validation is authorized here.\n\n';assert doc.count(marker)==1
 insert=('`'+REPORT+'` connects two bounded partial assessments and two\n'
 'planning relationships. They do not close a queue record or lens or add a\n'
 'unique-defect count. `delegated/postede12-followups/PENDING-DELEGATES.json`\n'
 'indexes 28 later, separately attributed source/lens obligations awaiting\n'
 'central reconciliation, including a separately sealed URL correction.\n\n'
 '`delegated/postede12-followups/INDEX.md` locates the complete checkpoint\n'
 'evidence, same-machine predecessor recovery, restricted originals and key\n'
 'dependencies. Earlier pending indexes retain their time-bound snapshots.\n\n'
 '`'+RECHECK+'` records a point-in-time connector\n'
 'observation of main at 95e. Main may have moved since.\n\n')
 doc=doc.replace(marker,marker+insert,1)
 assert b['historical_queue_records_semantically_reconciled']==0 and p['repair']=='not started' and p['product_validation']=='not performed'
 return {NAMES[0]:render(b),NAMES[1]:render(p),NAMES[2]:render(q),NAMES[3]:doc.encode()}
