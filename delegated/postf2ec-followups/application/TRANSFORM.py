#!/usr/bin/env python3
"""Transform qualified audit records only; never execute product code."""
import copy,json
NAMES=['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']
REPORT='ASSESSMENT-CENTRAL-FOLLOWUPS-19-95E.json'
PENDING='delegated/postf2ec-followups/PENDING-DELEGATES.json'
RECHECK='MAIN-RECHECK-20261113-0314.json'
REV='95e63628f9d70b8dd6e6fede64334696c6995009'
PARENT='f2ec8cbad7becbcc9a8e9da22ba512f52975768f'
def render(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode()
def transform(before,report,pending,obs,previous_pending):
 assert set(before)==set(NAMES)
 b,p,q=(json.loads(before[n]) for n in NAMES[:3]);doc=before[NAMES[3]].decode()
 assert b['source_revision']==report['source_revision']==pending['source_revision']==obs['observed_revision']==REV
 assert report['parent_published_checkpoint']==pending['parent_checkpoint']==PARENT
 assert obs['ref']=='refs/heads/main' and obs['observed_tree']=='1eec97de340d9b7debbb30ca0c0fb2240a73b93f' and obs['method']=='GitHub connector fetch branches/main API'
 assessments=[x['record'] for x in report['partial_queue_assessments']];rels=[x['record'] for x in report['ledger_relationships']]
 wanted=[3243,3246,3255,3258,3244,3247,3256,3259]
 assert [x['queue_index'] for x in assessments]==wanted and len(rels)==2 and len(pending['packets'])==28
 assert report['counts']=={'new_partial_queue_assessments':8,'semantic_queue_closures':0,'new_planning_relationships':2,'accepted_unique_defect_increment':0,'lens_completions':0}
 entries=b['newly_assessed_ledger']['entries'];oldids={x['id'] for x in entries};ids=[x['id'] for x in rels]
 assert len(entries)==len(oldids)==1101 and len(set(ids))==2 and not set(ids)&oldids
 for row in rels:
  x=copy.deepcopy(row);assert x['repair']=='not started' and x['validation']=='not performed';x['record']=REPORT;entries.append(x)
 assert len(entries)==1103 and len(q['entries'])==3713
 for x in assessments:
  row=q['entries'][x['queue_index']];assert row['contract_id']==x['historical_contract_id'] and row['historical_lens']==x['historical_lens'] and row['current_reconciliation']=='unassessed'
  row['current_reconciliation']='partial current 95e source assessment and historical relationship; see '+REPORT+'; no semantic queue or lens closure and no checkpoint3821 attribution'
  packet='yuv-four-lens.tar.gz' if x['queue_index'] in [3243,3244,3246,3247] else 'space-four-lens.tar.gz'
  row['further_current_evidence']=['delegated/postede12-followups/transport/'+packet,'delegated/postf2ec-followups/transport-v2/color-independent-review.tar.gz','delegated/postf2ec-followups/encrypted-originals/README.md']
 for x in pending['packets']:
  row=q['entries'][x['queue_index']];assert row['contract_id']==x['historical_contract_id'] and row['historical_lens']==x['historical_lens'] and row['current_reconciliation']=='unassessed'
 assert len(previous_pending['packets'])==28
 prior=[q['entries'][x['queue_index']]['current_reconciliation'] for x in previous_pending['packets']]
 assert sum(s=='unassessed' for s in prior)==20 and sum(s.startswith('partial') for s in prior)==8
 assert sum(x['current_reconciliation']=='unassessed' for x in q['entries'])==2349
 assert sum(x['current_reconciliation'].startswith('partial') for x in q['entries'])==1364
 assert p['new_current_ledger_candidates']==1101 and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
 p['new_current_ledger_candidates']=1103
 p['central_postf2ec_bounded_reconciliations']=REPORT+'; eight partial assessments, two planning relationships, no semantic closure or accepted unique-defect increment.'
 p['postf2ec_pending_delegated_packets']=PENDING+'; 28 later source/lens obligations remain pending central reconciliation.'
 p['postede12_pending_snapshot_qualification']='Its 28-row index is time-bound: eight color rows now have partial assessments; the other 20 remain unassessed.'
 p['checkpoint_f2ec_same_machine_recovery']='delegated/postf2ec-followups/RECOVERY-F2EC.json; 182 captured path identities matched; the evidence transcript was byte-identical in a separate same-machine checkout.'
 p['postf2ec_private_recovery']='delegated/postf2ec-followups/RESTORATION-RECEIPT.json; same-machine primary key, previous q2812 anchors and named nested keys remain dependencies.'
 p['main_recheck_20261113_0314']=RECHECK
 p['current_main_qualification']='GitHub connector observed main at 95e between 2026-11-13 03:13:59 and 03:14:00 UTC; this is a point-in-time observation and main may advance. See '+RECHECK+'.'
 b['latest_main_recheck']=RECHECK
 marker='product validation is authorized here.\n\n';assert doc.count(marker)==1
 insert=('`'+REPORT+'` connects eight bounded partial\n'
 'assessments and two planning relationships. They do not close a queue\n'
 'record or lens or add a unique-defect count. Eight color obligations in\n'
 'the older 28-row pending snapshot are now partial; its other 20 remain\n'
 'unassessed. The older index retains its historical snapshot.\n\n'
 '`delegated/postf2ec-followups/PENDING-DELEGATES.json` indexes 28 later,\n'
 'separately attributed source/lens obligations awaiting reconciliation.\n'
 '`delegated/postf2ec-followups/INDEX.md` locates checkpoint evidence,\n'
 'same-machine predecessor recovery and restricted custody dependencies.\n\n'
 '`'+RECHECK+'` records a point-in-time connector\n'
 'observation of main at 95e. Main may have moved since.\n\n')
 doc=doc.replace(marker,marker+insert,1)
 assert b['historical_queue_records_semantically_reconciled']==0 and p['repair']=='not started' and p['product_validation']=='not performed'
 return {NAMES[0]:render(b),NAMES[1]:render(p),NAMES[2]:render(q),NAMES[3]:doc.encode()}
