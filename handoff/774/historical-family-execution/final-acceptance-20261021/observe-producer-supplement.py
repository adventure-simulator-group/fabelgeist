"""Observe explicit audit receipts; never infer source coverage or clearance."""
import datetime,hashlib,json,pathlib
A=pathlib.Path(__file__).resolve().parent;R=A.parent;S=A/'wave-01/producer-supplement-after-owner';now=datetime.datetime.now(datetime.timezone.utc).isoformat()
rows=[]
for who in ('root','camera','rig','native'):
 f=S/who/'progress.json'
 if not f.exists():rows.append({'owner':who,'status':'incomplete_not_started','accounted_obligations':0});continue
 j=json.loads(f.read_text());rows.append({'owner':who,'record':str(f.relative_to(A)),'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'status':j['status'],'accounted_obligations':j['accounted_obligations'],'assigned_obligations':j.get('assigned_obligations',j.get('obligations')),'first_unfinished_cell':j.get('first_unfinished_cell'),'scope_limit':'Receipt reports explicit source-contract dispositions; counts and old verdicts are not coverage or repair/validation.'})
f=A/'audit-ledger.json';j=json.loads(f.read_text());j['updated_at']=now;j['current_lens']='audit-producer-propagation';j['current_inventory_step'].update(observed_at=now,progress=rows)
lens=next(x for x in j['lenses'] if x['lens']=='audit-producer-propagation');lens.setdefault('original_scoped_variants',946);lens['initial_pass_expanded_scope_variants']=1641;lens['scoped_variants']=2279;lens['cells_scope_note']='Existing cells[] preserve original946 history; exact original reached-contract proofs preserve the1641 firstpass. Current expanded source scope2279 requires1650 supplemental source/role obligations; progress receipts below do not themselves change discovery/repair/validation status.';lens['current_supplement']={'assignments':'wave-01/producer-supplement-after-owner/assigned-*.json','obligations':1650,'scope':'wave-01/source-scope-after-owner.json','range_errata':'wave-01/source-range-errata.json','progress':rows,'discovery':'incomplete_pending_all_shard_contract_closure_and_reconciliation'}
root=next(x for x in rows if x['owner']=='root')
pending=next((x for x in rows if x.get('first_unfinished_cell') is not None or x.get('accounted_obligations',0)<x.get('assigned_obligations',0)),None)
j['first_unfinished_cell']=({'owner':pending['owner'],'cell':pending.get('first_unfinished_cell'),'evidence':pending['record']} if pending else None)
receipts=[]
for who in ('root','camera','rig','native'):
 for name in ('coverage.jsonl','traces.jsonl','findings.jsonl'):
  p=S/who/name
  if not p.exists():continue
  data=[json.loads(line) for line in p.read_text().splitlines() if line.strip()]
  receipts.append({'owner':who,'record':str(p.relative_to(A)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'rows':len(data),'role':'Supporting exact contract/trace/disposition records, not final clearance; provisional overlap reconciliation remains required.'})
lens['current_supplement']['supporting_receipts']=receipts
j['current_inventory_step']['supporting_receipts']=receipts
observation=A/'wave-01/producer-reached-scope-observation.json'
if observation.exists():
 reached=json.loads(observation.read_text())
 reached['observation_record']=str(observation.relative_to(A))
 reached['limits']='Derived explicit named source-receipt index only, not exhaustive new-role or alias reconciliation and not audit coverage. Frozen2279 owner inventory remains historical for exact recorded roles; new producer source roles require an explicit owner-fit supplement.'
 j['discovery_gate']['producer_reached_scope_observation']=reached
 j['current_inventory_step']['owner_fit_discovery']='Exhausted with findings for frozen2279 scope; newly reached producer roles await explicit owner-fit reconciliation before admission lens.'
 owner_lens=next(x for x in j['lenses'] if x['lens']=='audit-existing-type-owners')
 owner_lens['frozen_scope_discovery_preserved']=owner_lens.get('frozen_scope_discovery_preserved',owner_lens['discovery'])
 owner_lens['discovery']='incomplete_new_producer_reached_roles_pending_reconciliation; frozen2279_source_dispositions_preserved'
 owner_lens['new_role_observation']=str(observation.relative_to(A))
f.write_text(json.dumps(j,indent=2)+'\n')
f=R/'execution-ledger.json';j=json.loads(f.read_text());j['updated_at']=now
for owner,x in j['owners'].items():x['observed_claim_sha256']=hashlib.sha256((R/x['claims_file']).read_bytes()).hexdigest();x['claims_observed_at']=now
j['acceptance_gate']['current_inventory_step'].update(observed_at=now,progress=rows,range_metadata_errata='final-acceptance-20261021/wave-01/source-range-errata.json')
j['next_action']='Complete expanded producer supplement contracts (first unfinished '+str({'owner':pending['owner'],'cell':pending.get('first_unfinished_cell')} if pending else 'all shard cells accounted; reconcile reached queues')+'), reconcile completed bounded receiver packets and newly reached exact source roles, then inventory lenses03–13. All source repairs remain frozen until exhaustive initial13 discovery. Afterwards finish/integrate mandatory P16/P17 and connected shared functions in dependency order, repeat complete audit waves/reviews and hosted CI on same final revision; no current validation or family-completion credit from inventory.'
shared={'id':'typed-melee-telemetry-classification','responsible_owner':'/root/camera_targeting','ownership_claims':'parallel-ownership/camera_targeting-claims.json','contributes_to':[],'scope':'mandatory_final_audit_adjacent_whole_function_completion; no inferred family exit credit','state':'initial_discovery_only_repairs_frozen','source_commits':[],'integrated_commits':[],'evidence':['final-acceptance-20261021/wave-01/producer-supplement-after-owner/camera/findings.jsonl#CAM-PPS-F002'],'remaining':['Whole existing commitment/response enum codec and record_melee_result/apply_defender_response/timeline functions retain canonical classifications through complete MeleeContactTelemetry record; native report owner consumes agreed typed classification to actual serialization. Existing physical-fields slice superseded by whole-record claim; no duplicate type or temporary text adapter.'],'completion_rule':'All affected whole-function requirements must finish; shared repair never automatically completes P16/P17 or other families. No implementation/validation yet.'}
if not any(x['id']==shared['id'] for x in j['shared_repairs']):j['shared_repairs'].append(shared)
for x in j['items']:
 if x['id']=='P16':
  gap='CAM-PPS-F001: preserved Core melee_iteration existing fixture lines672–677 call unavailable SignedLength::abs and862 compares SignedLength to f32. Camera owns whole existing fixture/signature adaptation; retain original values/cases/oracles, no added tests and no compiler/CI claim.'
  if gap not in x['source_status']['remaining_gaps']:x['source_status']['remaining_gaps'].append(gap)
  ev='final-acceptance-20261021/wave-01/producer-supplement-after-owner/camera/findings.jsonl#CAM-PPS-F001'
  if ev not in x['source_status']['evidence']:x['source_status']['evidence'].append(ev)
 if x['id'] in ('P16','P17'):assert x['source_status']['state']=='implementing' and x['validation_status']['validated_head'] is None
j['producer_supplement_reconciliation']={'observed_at':now,'record':'final-acceptance-20261021/audit-ledger.json#current_inventory_step','source_repairs':'frozen_until_all13_initial_discovery','source_families':'P16/P17 mandatory implementing, separate source/integration/validation statuses unchanged'}
f.write_text(json.dumps(j,indent=2)+'\n');print(json.dumps({'progress':[(x['owner'],x['accounted_obligations'],x.get('assigned_obligations')) for x in rows],'ledger_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'no_source_or_project_execution':True},indent=2))
