#!/usr/bin/env python3
"""Construct the bounded, evidence-only 8ca successor after-image."""
import copy,json,sys,textwrap

if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
NAMES=['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']
REPORT='ASSESSMENT-CENTRAL-FOLLOWUPS-23-95E.json';MAIN='MAIN-RECHECK-POST8CA.json'
PARENT='8caeb2003d10d08320566206817aa0c694231778';REV='95e63628f9d70b8dd6e6fede64334696c6995009';TREE='1eec97de340d9b7debbb30ca0c0fb2240a73b93f';IDS=[24,185,689,3100]
COUNTS={'queue_records':3713,'partial_before':1400,'unassessed_before':2313,'partial_after':1404,'unassessed_after':2309,'ledger_before':1128,'proposed_relationships':7,'ledger_after':1135,'queue_closures':0,'lens_completions':0,'accepted_unique_defect_increment':0}
def render(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode()
def transform(before,projection,report,source_map,history,observation):
 assert set(before)==set(NAMES)
 assert projection['schema']=='726-post8ca-public-projection-v1' and projection['status']=='public projection of a draft; not installed or adopted'
 assert report['schema']=='726-post8ca-central-assessment-v1' and report['status']=='adopted as four bounded partial assessments and seven planning relationships; no semantic closure'
 assert all(report[k]==projection[k] for k in projection if k not in ('schema','status')) and set(report)-set(projection)=={'adoption_qualification','independent_projection_review'}
 assert projection['published_checkpoint']==source_map['published_checkpoint']==history['published_checkpoint']==PARENT
 assert projection['source_revision']==observation['observed_revision']==REV and projection['source_tree']==observation['observed_tree']==TREE and observation['ref']=='refs/heads/main'
 assert [a['index'] for a in report['assessments']]==IDS and report['counts_if_adopted']==COUNTS and len(report['proposed_relationships'])==7
 by_label={x['label']:x for x in source_map['reports']};assert len(by_label)==len(source_map['reports'])==10
 baseline,progress,queue=(json.loads(before[n]) for n in NAMES[:3]);doc=before[NAMES[3]].decode();oldb=copy.deepcopy(baseline);oldp=copy.deepcopy(progress);oldq=copy.deepcopy(queue)
 assert baseline['source_revision']==progress['source_revision']==REV and baseline['historical_queue_records_semantically_reconciled']==progress['queue_records_semantically_reconciled']==progress['lenses_complete']==0
 assert progress['repair']=='not started' and progress['product_validation']=='not performed' and progress['new_current_ledger_candidates']==1128
 entries=queue['entries'];ledger=baseline['newly_assessed_ledger']['entries'];assert len(entries)==3713 and len(ledger)==1128
 assert sum(r['current_reconciliation'].startswith('partial') for r in entries)==1400 and sum(r['current_reconciliation']=='unassessed' for r in entries)==2313
 relations=report['proposed_relationships'];relids=[r['id'] for r in relations];assert len(set(relids))==7 and not ({r['id'] for r in ledger}&set(relids))
 assessed={a['index']:a['historical_lens'] for a in report['assessments']}
 hist={'original_queue_sha256':history['original_queue_sha256'],'reconstructed_queue_sha256':history['reconstructed_queue_sha256'],'selected_queue_objects_equal_as_parsed':True,'whole_stream_bytes_equal':False,'qualification':'Selected original and reconstructed 059 objects match as parsed; complete streams differ and neither is missing checkpoint3821 authority.'}
 for rel in relations:
  assert all(0<=i<len(entries) for i in rel['queue']) and all(0<=i<1128 for i in rel['ledger'])
  ledger.append({'id':rel['id'],'assessed_lenses':sorted({assessed[i] for i in rel['queue'] if i in assessed}),'classification':rel['classification'],'historical_related_records':['baseline-059/queue.json#/entries/'+str(i) for i in rel['queue']],'current_related_records':['BASELINE-95E-20261110.json#/newly_assessed_ledger/entries/'+str(i) for i in rel['ledger']],'relationship_qualification':rel['qualification'],'connected_repair_direction':rel['direction'],'discovery':'partial; supported reach, policy, source complements and exact aliases remain open','repair':'not started','validation':'not performed','record':REPORT,'historical_original_and_reconstruction':copy.deepcopy(hist)})
 for a in report['assessments']:
  i=a['index'];row=entries[i];assert (row['contract_id'],row['historical_lens'],row['historical_status'],row['current_reconciliation'])==(a['contract_id'],a['historical_lens'],a['historical_status_projection'],'unassessed')
  assert 'further_current_evidence' not in row and all(x in by_label for x in a['reports'])
  archives=list(dict.fromkeys('delegated/post8ca-followups/'+by_label[x]['archive'] for x in a['reports']))
  row['current_reconciliation']='partial current 95e source assessment and historical relationship; see '+REPORT+'; no semantic queue or lens closure and no checkpoint3821 attribution'
  row['further_current_evidence']=[REPORT,'delegated/post8ca-followups/reconciliation/SOURCE-MAP.json',*archives]
 assert sum(r['current_reconciliation'].startswith('partial') for r in entries)==1404 and sum(r['current_reconciliation']=='unassessed' for r in entries)==2309 and len(ledger)==1135
 progress['new_current_ledger_candidates']=1135
 progress['central_post8ca_bounded_reconciliations']=REPORT+'; four partial assessments and seven planning relationships, with no semantic closure or accepted unique-defect increment.'
 progress['post8ca_queue_snapshot']='3,713 indexed: 1,404 partial and 2,309 unassessed; all remain semantically unreconciled.'
 progress['post8ca_later_evidence_qualification']='Later q2561, q3044 and q2743 source evidence remains separately attributed and pending central reconciliation; its capture does not update these four queue records or create historical checkpoint3821 authority.'
 progress['checkpoint_8ca_same_machine_recovery']='delegated/post8ca-followups/INDEX.md describes the separately preserved successor recovery, point-in-time observations, minimal Git alternate and later custody cutoffs.'
 progress['main_recheck_post8ca']=MAIN
 progress['current_main_qualification']='GitHub connector observed main at 95e between '+observation['before_utc']+' and '+observation['after_utc']+'; this is point-in-time and main may advance. See '+MAIN+'.'
 baseline['latest_main_recheck']=MAIN
 marker='product validation is authorized here.\n\n';assert doc.count(marker)==1
 paras=[REPORT+' records four bounded partial assessments and seven planning relationships. The working queue has 1,404 partial and 2,309 unassessed records, all semantically unreconciled; the ledger has 1,135 entries and relationships, not unique defects. No queue record or lens is closed and no unique-defect increment is accepted.','delegated/post8ca-followups/INDEX.md locates separately attributed source packets, independent reviews, custody and recovery records. Selected original and reconstructed 059 records agree as parsed while full streams differ. Missing checkpoint 3821, SOURCE004 and the 73 nontext/no-positive-EOF contracts remain open.','MAIN-RECHECK-POST8CA.json records a point-in-time main observation; main may advance. Later q2561, q3044 and q2743 evidence remains separately attributed and pending reconciliation. Earlier paragraphs below retain their historical snapshot counts.']
 text='\n\n'.join(textwrap.fill('`'+s.split(' ',1)[0]+'` '+s.split(' ',1)[1],width=80,break_long_words=False,break_on_hyphens=False) for s in paras)+'\n\n';doc=doc.replace(marker,marker+text,1)
 assert queue.keys()==oldq.keys() and all(queue[k]==oldq[k] for k in queue if k!='entries')
 for i,(old,new) in enumerate(zip(oldq['entries'],entries)):
  if i not in IDS:assert old==new
  else:assert {k:v for k,v in new.items() if k not in ('current_reconciliation','further_current_evidence')}=={k:v for k,v in old.items() if k!='current_reconciliation'}
 assert ledger[:1128]==oldb['newly_assessed_ledger']['entries'] and baseline['newly_assessed_ledger']['qualification']==oldb['newly_assessed_ledger']['qualification']
 assert all(baseline[k]==oldb[k] for k in oldb if k not in ('newly_assessed_ledger','latest_main_recheck'))
 assert all(progress[k]==oldp[k] for k in oldp if k not in ('new_current_ledger_candidates','current_main_qualification'))
 return {NAMES[0]:render(baseline),NAMES[1]:render(progress),NAMES[2]:render(queue),NAMES[3]:doc.encode()}
