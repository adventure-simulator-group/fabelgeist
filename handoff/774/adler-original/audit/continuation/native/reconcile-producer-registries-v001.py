import argparse,collections,datetime,hashlib,json,pathlib
P=argparse.ArgumentParser();P.add_argument('--label',default='001');args=P.parse_args()
B=pathlib.Path(r'audit');O=B/'continuation/native/producer-registry-reconciliation';O.mkdir(exist_ok=True)
out=O/f'candidate-{args.label}.json'
if out.exists():raise SystemExit('Candidate already exists; use a fresh --label to preserve prior receipts.')
paths={'base':'continuation/cumulative-source-scope-producer-followup-007.json','overlay':'continuation/owner-range-extensions.jsonl','camera':'continuation/camera/owner-reached-producer-followup/source-range-registration.jsonl','rig':'continuation/rig/owner-reached-producer-followup/additional-sources.jsonl','native':'continuation/native/root-roof-producer-followup/local-source-scope-extensions.jsonl'}
blobs={k:(B/v).read_bytes() for k,v in paths.items()};pins={k:{'record':paths[k],'sha256':hashlib.sha256(v).hexdigest(),'bytes':len(v)} for k,v in blobs.items()}
base=json.loads(blobs['base'].decode('utf-8-sig'));baseline={r['source_id']:r for r in base['files']};issues=[]
def rows(k):
 result=[]
 for n,line in enumerate(blobs[k].decode('utf-8-sig').splitlines(),1):
  if not line.strip():continue
  try:result.append((n,json.loads(line)))
  except ValueError as e:issues.append({'kind':'registry_parse_error','registry':k,'line':n,'error':str(e),'disposition':'Require refresh; no dropped row silently accepted.'})
 return result
overlays=[];later=[]
for n,r in rows('overlay'):
 try:i=int(r['id'].rsplit('-',1)[1])
 except (KeyError,ValueError):issues.append({'kind':'unrecognized_overlay_id','line':n,'record':r});continue
 (overlays if i<=22 else later).append({'row':n,'record':r})
def ranges(r):
 for k in ['material_ranges','requested_ranges','ranges','read_ranges','contract_ranges']:
  if k in r:return r[k]
 return []
def merge(rs):
 out=[]
 for a,b in sorted(set(tuple(x) for x in rs)):
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out
def minus(rs,covered):
 out=[]
 for a,b in merge(rs):
  bits=[[a,b]]
  for x,y in merge(covered):
   nxt=[]
   for lo,hi in bits:
    if hi<x or lo>y:nxt.append([lo,hi])
    else:
     if lo<x:nxt.append([lo,x-1])
     if hi>y:nxt.append([y+1,hi])
   bits=nxt
  out.extend(bits)
 return out
observations=[];groups={};qualifications=[]
for supplier in ['camera','rig','native']:
 for line,r in rows(supplier):
  ref={'registry':paths[supplier],'row_1based':line,'registration_id':r.get('id'),'supplier':supplier,'original_record':r};observations.append(ref)
  if 'effective_material_ranges' in r or 'effective_material_roles' in r or 'qualifies' in r:qualifications.append(ref)
  sid=r.get('source_id')
  if sid and ranges(r):groups.setdefault(sid,[]).append(ref)
# Explicit qualifications are projections on local role scope only, never global removal.
qualified={}
for q in qualifications:
 r=q['original_record']
 if 'effective_material_ranges' in r:qualified[r['source_id']]={'ranges':r['effective_material_ranges'],'qualification':q}
 for sid,rr in r.get('effective_material_roles',{}).items():qualified[sid]={'ranges':rr,'qualification':q}
def base_ranges(record,sid):
 found=[]
 def walk(v):
  if isinstance(v,dict):
   if v.get('source_id',sid)!=sid:return
   for k,z in v.items():
    if k in ['read_ranges','material_ranges','requested_ranges','ranges','contract_ranges'] and isinstance(z,list) and all(isinstance(a,list) and len(a)==2 and all(isinstance(b,int) for b in a) for a in z):found.extend(z)
    elif isinstance(z,(dict,list)):walk(z)
  elif isinstance(v,list):
   for z in v:walk(z)
 walk(record)
 for o in overlays:
  if o['record'].get('source_id')==sid:found.extend(ranges(o['record']))
 return merge(found)
variants=[]
for sid,obs in sorted(groups.items()):
 try:snapshot,rel=sid.split(':',1)
 except ValueError:issues.append({'kind':'bad_source_id','source_id':sid});continue
 folder={'pr-working':'pr-working','physical-p16':'p16'}.get(snapshot)
 if folder is None:issues.append({'kind':'unapproved_snapshot','source_id':sid});continue
 root=(B.parent/'774-snapshots'/folder).resolve();actual=(root/rel).resolve()
 if root not in actual.parents:issues.append({'kind':'path_outside_snapshot','source_id':sid});continue
 if not actual.is_file():issues.append({'kind':'missing_frozen_file','source_id':sid,'path':actual.as_posix()});continue
 raw=actual.read_bytes();sha=hashlib.sha256(raw).hexdigest();eof=len(raw.decode('utf8').splitlines());original_ranges=[];hash_conflicts=[];metadata=[]
 for o in obs:
  r=o['original_record'];rr=ranges(r);original_ranges.extend(rr)
  if r.get('sha256')!=sha:hash_conflicts.append({'registration':r.get('id'),'expected':r.get('sha256'),'observed':sha})
  declared=r.get('actual_EOF',r.get('actual_eof_metadata',r.get('actual_eof_metadata_only',r.get('actual_line_count'))))
  if declared is not None and declared!=eof:metadata.append({'registration':r.get('id'),'declared_EOF':declared,'actual_EOF':eof})
  if r.get('snapshot') not in [None,snapshot,folder]:metadata.append({'registration':r.get('id'),'declared_snapshot':r.get('snapshot'),'source_identity_snapshot':snapshot})
 effective=qualified.get(sid,{}).get('ranges',original_ranges);effective=merge(effective)
 for a,b in effective:
  if not(1<=a<=b<=eof):issues.append({'kind':'invalid_effective_material_range','source_id':sid,'range':[a,b],'actual_EOF':eof})
 prior=baseline.get(sid);prior_ranges=base_ranges(prior,sid) if prior else []
 if prior and prior.get('scope_kind')=='frozen_requested_change':prior_ranges=merge(prior_ranges+[[1,eof]])
 prior_sha=prior.get('sha256') if prior else None
 if prior and prior_sha!=sha:hash_conflicts.append({'baseline_source':sid,'expected':prior_sha,'observed':sha})
 if hash_conflicts:issues.append({'kind':'hash_identity_mismatch','source_id':sid,'conflicts':hash_conflicts})
 if metadata:issues.append({'kind':'metadata_discrepancy','source_id':sid,'details':metadata})
 delta=minus(effective,prior_ranges)
 variants.append({'source_id':sid,'snapshot':snapshot,'path':rel,'resolved_path':actual.as_posix(),'sha256':sha,'actual_EOF':eof,'bytes':len(raw),'identity_validation':'verified_frozen_bytes' if not hash_conflicts else 'blocked_hash_mismatch','baseline_source_exists':prior is not None,'classification':'new_source_variant' if not prior else 'existing_variant_range_and_role_addition' if delta else 'existing_variant_material_role_addition_only','baseline_scope_kind':prior.get('scope_kind') if prior else None,'baseline_ranges_as_registered':prior_ranges,'local_ranges_as_registered':merge(original_ranges),'effective_local_material_ranges':effective,'new_byte_ranges_vs_scope007_plus_overlay022':delta,'existing_byte_overlap':minus(effective,delta),'proposed_total_registered_ranges':merge(prior_ranges+effective),'qualification_refs':[qualified[sid]['qualification']] if sid in qualified else [],'registration_observations':obs,'proposed_audit_owner':prior.get('audit_owner',prior.get('owner')) if prior else ('root' if any(o['supplier']=='native' for o in obs) else obs[0]['supplier']),'read_only_suppliers':sorted(set(o['supplier'] for o in obs)),'role_preservation':'Every original registration/role/caller/reason remains embedded independently. Byte overlap is not semantic role coverage, producer acceptance, owner-fit acceptance or edit ownership.','scope_removal':'none; Native qualifications trim local supplier roles only, never historical global scope.'})
end_changes=[k for k,v in paths.items() if hashlib.sha256((B/v).read_bytes()).hexdigest()!=pins[k]['sha256']]
counts=collections.Counter(x['classification'] for x in variants)
result={'id':'NATIVE-THREE-REGISTRY-SCOPE-CANDIDATE-'+args.label,'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'mode':'metadata-only proposed reconciliation; no global ledger writes, source semantics or later-lens credit','global_base_file_count':len(base['files']),'overlay_through':22,'input_pins':pins,'overlay_rows_through022':overlays,'later_overlay_rows_not_applied':later,'original_registry_rows':observations,'qualifications_preserved':qualifications,'source_variant_dispositions':variants,'counts':{'registration_rows':len(observations),'qualification_rows':len(qualifications),'distinct_source_variants':len(variants),'new_source_variants':counts['new_source_variant'],'existing_variants_with_new_byte_ranges':counts['existing_variant_range_and_role_addition'],'existing_variants_role_only':counts['existing_variant_material_role_addition_only'],'candidate_total_source_variants':len(base['files'])+counts['new_source_variant'],'issues':len(issues)},'issues':issues,'registries_changed_during_snapshot':end_changes,'refresh_required':{'rig_may_still_grow':True,'reason':'Rig additional-sources registry is active. Re-run with a fresh --label after Rig final packet; compare prefix/pins and preserve every new record. This candidate is exact only for input hashes above.','concurrent_input_change_detected':bool(end_changes)},'root_review_actions':['Review all identities/EOF/range deltas and Native narrowing qualifications before any global mutation.','Retain every role even when its byte interval already exists; do not equate request new_variant flags or same hash with role coverage.','Resolve any listed metadata/hash/range issue; append exact new identities/ranges/role references into global registry only by Root authority.','Join separately accepted producer narratives and perform necessary owner-fit delta before admission.'],'source_reads_for_semantics':0,'source_byte_reads':'Only frozen file bytes/hash/EOF for explicitly registered identities; no body analysis outside prior Native producer scope.','source_edits':0,'global_scope_audit_claim_mutations':0,'producer_owner_family_implementation_credit':'none'}
out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
summary={'candidate':out.as_posix(),'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'counts':result['counts'],'issues':issues,'changed_during_snapshot':end_changes,'rig_refresh_required':True};print(json.dumps(summary,ensure_ascii=True))
