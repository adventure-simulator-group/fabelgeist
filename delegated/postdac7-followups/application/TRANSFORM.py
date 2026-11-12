#!/usr/bin/env python3
"""Pure transformation of four historical checkpoint artifacts; no product work."""
import copy,json

def render(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode()
def transform(before,report,pending,observation):
 names=['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']
 assert set(before)==set(names)
 baseline=json.loads(before[names[0]]);progress=json.loads(before[names[1]]);queue=json.loads(before[names[2]]);document=before[names[3]].decode()
 assert baseline['source_revision']==report['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and report['parent_published_checkpoint']=='dac7d0055716053f245139971eb67ddfd87fa462'
 assert observation['observed_revision']==report['source_revision'] and observation['ref']=='refs/heads/main'
 assessments=report['assessments'];relationships=report['ledger_relationships'];ids=[x['id'] for x in relationships];indexes=[x['queue_index'] for x in assessments]
 assert indexes==[2708,2721,2734,2747,2750,2760,2773] and len(ids)==len(set(ids))==9
 entries=baseline['newly_assessed_ledger']['entries'];assert len(entries)==1084 and len({x['id'] for x in entries})==len(entries) and not set(ids)&{x['id'] for x in entries}
 entries.extend(copy.deepcopy(relationships));assert len(entries)==1093
 baseline['latest_main_recheck']='MAIN-RECHECK-20261112-2101.json'
 assert len(queue['entries'])==3713
 for a in assessments:
  row=queue['entries'][a['queue_index']];assert row['contract_id']==a['historical_contract_id'] and row['historical_lens']==a['historical_lens'] and row['current_reconciliation']=='unassessed'
  row['current_reconciliation']='partial current 95e source assessment and historical relationship; see ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v3.json; no semantic queue or lens closure and no checkpoint3821 attribution'
  row['further_current_evidence']=['delegated/postdac7-followups/central/REPORT-SUCCESSOR-CORRECTIONS-v2.json']
 assert sum(x['current_reconciliation']=='unassessed' for x in queue['entries'])==2365
 assert sum(x['current_reconciliation'].startswith('partial') for x in queue['entries'])==1348
 assert progress['new_current_ledger_candidates']==1084 and progress['lens_count']==13 and progress['lenses_complete']==0 and progress['queue_records_indexed']==3713 and progress['queue_records_semantically_reconciled']==0
 progress['new_current_ledger_candidates']=1093
 progress['central_postdac7_bounded_reconciliations']='ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v3.json; seven partial assessments and nine planning relationships; no queue or lens closure or accepted unique count increment.'
 progress['checkpoint_dac7_same_machine_recovery']='delegated/postdac7-followups/checkpoint-dac7/README.md; identical evidence transcript in a separate checkout on the same machine; independent custody and product behavior remain unproved.'
 progress['postdac7_pending_delegated_packets']='delegated/postdac7-followups/PENDING-DELEGATES.json; '+str(len(pending['packets']))+' packets remain pending central reconciliation at this snapshot.'
 progress['post0a24_pending_resolution']='Seven of the eleven prior pending packets now have partial assessment; four remain pending. The historical index is retained unchanged and does not imply closure.'
 progress['postdac7_private_recovery']='delegated/postdac7-followups/encrypted-originals/README.md; separately held same-machine primary key and named prior dependencies remain required.'
 progress['main_recheck_20261112_2101']='MAIN-RECHECK-20261112-2101.json'
 progress['current_main_qualification']='GitHub connector resolved main to 95e at 2026-11-12 21:01:35 UTC; this is a point-in-time observation and main may advance. See MAIN-RECHECK-20261112-2101.json.'
 marker='`ASSESSMENT-CENTRAL-FOLLOWUPS-15B-95E.json` and\n'
 assert document.count(marker)==1
 insert=('`delegated/postdac7-followups/checkpoint-dac7/README.md` records the\n'
         'same-machine separate-checkout recovery of checkpoint dac7 with an\n'
         'identical evidence transcript. This does not establish independent\n'
         'custody or product behavior.\n\n'
         '`ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v3.json` connects seven bounded\n'
         'partial assessments and nine planning relationships, preserving the\n'
         'original report and independently reviewed corrections. '
         +str(len(pending['packets']))+' delegated\n'
         'packets remain pending central reconciliation in\n'
         '`delegated/postdac7-followups/PENDING-DELEGATES.json`. None closes a\n'
         'semantic queue record or lens or adds an accepted unique-defect count.\n\n'
         '`MAIN-RECHECK-20261112-2101.json` records a later point-in-time\n'
         'connector observation of main at 95e; main may have moved since.\n\n'
         'The following 15B/15C paragraph describes its earlier time-bound\n'
         'snapshot; seven of its pending records now have partial assessment.\n\n')
 document=document.replace(marker,insert+marker,1)
 assert baseline['historical_queue_records_semantically_reconciled']==0 and progress['repair']=='not started' and progress['product_validation']=='not performed'
 return {names[0]:render(baseline),names[1]:render(progress),names[2]:render(queue),names[3]:document.encode()}
