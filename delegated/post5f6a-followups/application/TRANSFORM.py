#!/usr/bin/env python3
"""Pure source-evidence reconciliation; never execute product code."""
import copy,json,hashlib
import sys
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
NAMES=['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']
REPORT='ASSESSMENT-CENTRAL-FOLLOWUPS-20-95E.json'
MAIN='MAIN-RECHECK-20261113-0536.json'
REV='95e63628f9d70b8dd6e6fede64334696c6995009'
TREE='1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
PARENT='5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0'
QNAME='delegated/posta422-followups/proposals/Q10-Q13-v2.json'
RNAME='delegated/posta422-followups/proposals/ROLE0057-v3.json'
IDS=[10,13,3447,3448,3450,3451]
LENSES=[13,12,1,2,5,6]
CONTRACTS=['LENS13-DURABILITY-TIER-INTERMEDIATE-OVERFLOW','LENS12-PROJECTILE-ID-SUBSTRING','OBLIGATION-ROOT-L3-EXT-0057-LENS-01','OBLIGATION-ROOT-L3-EXT-0057-LENS-02','OBLIGATION-ROOT-L3-EXT-0057-LENS-05','OBLIGATION-ROOT-L3-EXT-0057-LENS-06']
RELIDS=['relationship-durability-tier-intermediate-95e','relationship-projectile-item-name-classification-95e','relationship-organization-instance-global-transition-95e']
def render(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode()
def transform(before,report,obs,proposal_bytes):
 assert set(before)==set(NAMES) and set(proposal_bytes)=={QNAME,RNAME}
 b,p,q=(json.loads(before[n]) for n in NAMES[:3]);doc=before[NAMES[3]].decode();origb=copy.deepcopy(b);origp=copy.deepcopy(p);origq=copy.deepcopy(q)
 assert b['source_revision']==p['source_revision']==report['source_revision']==obs['observed_revision']==REV and report['source_tree']==obs['observed_tree']==TREE and report['parent_checkpoint']==PARENT and obs['ref']=='refs/heads/main' and obs['method']=='GitHub connector fetch branches/main API'
 assert all(hashlib.sha256(proposal_bytes[n]).hexdigest()==report['proposal_sha256'][n] for n in [QNAME,RNAME]);a=json.loads(proposal_bytes[QNAME]);r=json.loads(proposal_bytes[RNAME])
 assessments=[x['record'] for x in report['partial_queue_assessments']];rels=[x['record'] for x in report['ledger_relationships']]
 assert len(assessments)==6 and len(rels)==3 and assessments==a['partial_queue_assessments']+[x['record'] for x in r['partial_queue_assessments']] and rels==a['ledger_relationships']+[x['record'] for x in r['ledger_relationships']]
 assert [x['queue_index'] for x in assessments]==IDS and [x['historical_lens'] for x in assessments]==LENSES and [x['historical_contract_id'] for x in assessments]==CONTRACTS and [x['id'] for x in rels]==RELIDS
 assert report['counts']=={'queue_records':3713,'partial_before':1364,'unassessed_before':2349,'new_partial_assessments':6,'partial_after':1370,'unassessed_after':2343,'ledger_before':1103,'new_planning_relationships':3,'ledger_after':1106,'semantic_queue_closures':0,'accepted_unique_defect_increment':0,'lens_completions':0}
 assert len(q['entries'])==3713 and sum(x['current_reconciliation']=='unassessed' for x in q['entries'])==2349 and sum(x['current_reconciliation'].startswith('partial') for x in q['entries'])==1364
 ledger=b['newly_assessed_ledger']['entries'];oldids={x['id'] for x in ledger};assert len(ledger)==len(oldids)==1103 and len(set(RELIDS))==3 and not set(RELIDS)&oldids
 history=report['historical_provenance'];assert history['original_queue_sha256']=='2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08' and history['original_ledger_sha256']=='3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac' and history['reconstructed_queue_sha256']=='3695264e6d4531a1137f0c3232e0d5163d941c49849cb4bddef31cd8403fd173' and history['reconstructed_ledger_sha256']=='f9501199085284117ef88969fae443654af353429c657fd6bd1efb916582b180'
 for rel in rels:
  x=copy.deepcopy(rel);assert x['repair']=='not started' and x['validation']=='not performed' and 'record' not in x;x['record']=REPORT;x['historical_original_and_reconstruction']=copy.deepcopy(history);ledger.append(x)
 for i,(idx,lens,cid) in enumerate(zip(IDS,LENSES,CONTRACTS)):
  row=q['entries'][idx];assert row['contract_id']==cid and row['historical_lens']==lens and row['current_reconciliation']=='unassessed' and 'further_current_evidence' not in row
  row['current_reconciliation']='partial current 95e source assessment and historical relationship; see '+REPORT+'; no semantic queue or lens closure and no checkpoint3821 attribution'
  if idx==10:refs=[QNAME,'delegated/posta422-followups/transport/q10-public.tar.gz','delegated/posta422-followups/late-independent/transport/q10-v2-structural.tar.gz']
  elif idx==13:refs=[QNAME,'delegated/posta422-followups/transport/q13-public.tar.gz','delegated/posta422-followups/peer-reviews-late/transport/q13-successor-review.tar.gz']
  else:refs=[RNAME,'delegated/postf2ec-followups/postfreeze-transport/role0057.tar.gz','delegated/posta422-followups/peer-reviews-late/transport/role0057-v3-review.tar.gz']
  row['further_current_evidence']=[REPORT,*refs]
 assert sum(x['current_reconciliation']=='unassessed' for x in q['entries'])==2343 and sum(x['current_reconciliation'].startswith('partial') for x in q['entries'])==1370 and len(ledger)==1106
 assert p['new_current_ledger_candidates']==1103 and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and b['historical_queue_records_semantically_reconciled']==0 and p['repair']=='not started' and p['product_validation']=='not performed'
 p['new_current_ledger_candidates']=1106;p['central_post5f6a_bounded_reconciliations']=REPORT+'; six partial assessments, three planning relationships, no semantic closure or accepted unique-defect increment.'
 p['post5f6a_queue_snapshot']='3,713 indexed: 1,370 partial and 2,343 unassessed; all remain semantically unreconciled.'
 p['post5f6a_prior_pending_qualification']='Earlier pending indexes are time-bound: q10, q13, q3447, q3448, q3450 and q3451 now have partial assessments; no historical index bytes were changed.'
 p['checkpoint_5f6a_same_machine_recovery']='delegated/post5f6a-followups/REMOTE-RECOVERY.json; 179 captured path identities matched in a separate same-machine checkout and evidence guards passed.'
 p['main_recheck_20261113_0536']=MAIN;p['current_main_qualification']='GitHub connector observed main at 95e between '+obs['before_utc']+' and '+obs['after_utc']+'; this is a point-in-time observation and main may advance. See '+MAIN+'.';b['latest_main_recheck']=MAIN
 marker='product validation is authorized here.\n\n';assert doc.count(marker)==1
 insert=('`'+REPORT+'` connects six bounded partial assessments and three\n'
 'planning relationships. The working queue now has 1,370 partial and 2,343\n'
 'unassessed records, all semantically unreconciled; the ledger has 1,106\n'
 'entries and relationships, not unique defects. No lens or queue record is\n'
 'closed and no unique-defect increment is accepted. Older pending indexes and\n'
 'the post-a422 progress records retain their historical snapshot counts.\n\n'
 'The six assessments use reconstructed public projections of separately\n'
 'preserved originals. Public `baseline-059` pointers identify reconstructed\n'
 'streams; their selected queue objects match the recovered originals as parsed.\n'
 'Original checkpoint 3821 remains missing, and the 73 nontext/no-positive-EOF\n'
 'contracts remain open. Attribution and later review qualifications remain in\n'
 'the assessment and the preserved source packets.\n\n'
 '`delegated/post5f6a-followups/REMOTE-RECOVERY.json` records same-machine\n'
 'recovery of the preceding checkpoint. `'+MAIN+'` records a later\n'
 'point-in-time main observation; main may advance.\n\n')
 insert+=('Snapshot-specific predecessor verifiers must run against an exact 5f6a\n'
 'checkout; their old counts and after-images do not describe this successor.\n'
 '`delegated/post5f6a-followups/application/README.md` records the replay.\n\n')
 import textwrap
 insert='\n\n'.join(textwrap.fill(' '.join(par.splitlines()),width=80,break_long_words=False,break_on_hyphens=False) for par in insert.rstrip('\n').split('\n\n'))+'\n\n'
 doc=doc.replace(marker,marker+insert,1)
 assert q.keys()==origq.keys() and all(q[k]==origq[k] for k in q if k!='entries')
 for i,(old,new) in enumerate(zip(origq['entries'],q['entries'])):
  if i not in IDS:assert old==new
  else:assert {k:v for k,v in new.items() if k not in ('current_reconciliation','further_current_evidence')}=={k:v for k,v in old.items() if k!='current_reconciliation'}
 assert b['newly_assessed_ledger']['entries'][:1103]==origb['newly_assessed_ledger']['entries']
 for k in b:
  if k not in ('newly_assessed_ledger','latest_main_recheck'):assert b[k]==origb[k]
 assert b['newly_assessed_ledger']['qualification']==origb['newly_assessed_ledger']['qualification']
 return {NAMES[0]:render(b),NAMES[1]:render(p),NAMES[2]:render(q),NAMES[3]:doc.encode()}
