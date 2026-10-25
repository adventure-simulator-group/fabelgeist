"""Metadata only: reads ignored receipts/registries, never source bytes."""
import json,pathlib,hashlib,datetime,collections,argparse
ap=argparse.ArgumentParser();ap.add_argument('--label',default='001');a=ap.parse_args()
B=pathlib.Path(__file__).resolve().parents[3]; O=B/'continuation/native/owner81'
paths={'base':'continuation/cumulative-source-scope-producer-followup-008.json','overlays':'continuation/owner-range-extensions.jsonl','accepted':'continuation/root/producer-followup-acceptance-028.json','producer_index':'continuation/producer-reached-role-index.json','camera':'continuation/camera/owner81/source-range-registration.jsonl','rig':'continuation/rig/producer-reached-owner81/additional-sources.jsonl','native':'continuation/native/owner81/local-owner-source-registrations.jsonl'}
blobs={k:(B/v).read_bytes() for k,v in paths.items()}
pins={k:dict(record=paths[k],sha256=hashlib.sha256(v).hexdigest(),bytes=len(v)) for k,v in blobs.items()}
js=lambda k:json.loads(blobs[k].decode('utf-8-sig'))
jl=lambda k:[json.loads(l) for l in blobs[k].decode('utf-8-sig').splitlines() if l.strip()]
base=js('base'); known={r['source_id']:r for r in base['files']}; overlays=jl('overlays'); accepted=js('accepted'); idx=js('producer_index')
assert pins['base']['sha256']==accepted['scope']['sha256']
assert pins['overlays']['sha256']==accepted['overlays']['sha256']
def merge(ranges):
 out=[]
 for lo,hi in sorted(ranges):
  if out and lo<=out[-1][1]+1:out[-1][1]=max(out[-1][1],hi)
  else:out.append([lo,hi])
 return out
def subtract(wanted,old):
 result=[]
 for lo,hi in merge(wanted):
  cursor=lo
  for a,b in merge(old):
   if b<cursor:continue
   if a>hi:break
   if a>cursor:result.append([cursor,min(hi,a-1)])
   cursor=max(cursor,b+1)
   if cursor>hi:break
  if cursor<=hi:result.append([cursor,hi])
 return result
def ranges(v):
 return v if isinstance(v,list) and all(isinstance(x,list) and len(x)==2 and all(isinstance(y,int) for y in x) for x in v) else []
def byte_scope(old,eof,sid):
 if not old:return [],'new source variant; no accepted baseline material range'
 if old.get('scope_kind')=='frozen_requested_change':return [[1,eof]],'original changed variant full requested scope (metadata scope, not semantic proof)'
 rr=[]
 for key in ['material_ranges','requested_ranges']:rr+=ranges(old.get(key))
 for ev in old.get('evidence',[]):rr+=ranges(ev.get('ranges'))
 for ob in old.get('producer_reached_contract_observations',[]):rr+=ranges(ob.get('read_ranges_as_recorded'))
 for ext in old.get('producer_followup_scope_additions',[]):rr+=ranges(ext.get('effective_local_material_ranges'))
 for ob in overlays:
  if ob.get('source_id')==sid:rr+=ranges(ob.get('material_ranges'))
 return merge(rr),'union of explicit baseline material/requested/reached-producer ranges and accepted overlays; no owner-proof-derived range credit'
records=[]; raw=[]; quals=[]; groups={}; errors=[]
for supplier in ['camera','rig','native']:
 parsed=jl(supplier); qualified={}
 for line,r in enumerate(parsed,1):
  raw.append(dict(registry=paths[supplier],row_1based=line,supplier=supplier,original_record=r))
  if 'source_id' not in r:
   quals.append(raw[-1])
   for key,value in r.get('qualified_material_ranges_by_registration',{}).items():qualified[key]=dict(ranges=value,qualification=raw[-1])
 for line,r in enumerate(parsed,1):
  if 'source_id' not in r:continue
  sid=r['source_id']; old=known.get(sid); rr=r.get('ranges',r.get('read_ranges',r.get('material_ranges'))); eff=qualified.get(r['id'],{}).get('ranges',rr); eof=r.get('actual_EOF',r.get('actual_eof',r.get('actual_line_count')))
  assert rr and eff and eof and all(1<=lo<=hi<=eof for lo,hi in rr)
  assert all(any(a<=lo<=hi<=b for a,b in rr) for lo,hi in eff)
  if old:assert old['sha256']==r['sha256']
  oldranges,reason=byte_scope(old,eof,sid); new=subtract(eff,oldranges)
  classification='new_source_variant' if not old else ('existing_source_new_material_ranges' if new else 'already_registered_definition_or_dependency_role_candidate')
  role=r.get('actual_reach',r.get('actual_use_and_candidate',r.get('actual_use')))
  nav=[]
  if old:
   for field in ['producer_source_proof','evidence','producer_reached_contract_observations','expanded_producer_contract_observations','continuation_actual_producer_source_proofs']:
    for n,value in enumerate(old.get(field,[])):
     nav.append(dict(kind=field,pointer=f"{paths['base']}#/files/{base['files'].index(old)}/{field}/{n}",original_proof_metadata=value,semantic_status='navigation only; exact current producer role comparison not performed'))
   if old.get('producer_inventory_accounting'):nav.append(dict(kind='accepted_accounting_navigation_only_not_actual_body_pointer',pointer=f"{paths['base']}#/files/{base['files'].index(old)}/producer_inventory_accounting",original_proof_metadata=old['producer_inventory_accounting']))
  for n,value in enumerate(idx['explicit_source_range_observations']):
   if value.get('source_id')==sid:nav.append(dict(kind='prior_producer_receipt_index_observation',pointer=f"{paths['producer_index']}#/explicit_source_range_observations/{n}",original_proof_metadata=value,semantic_status='navigation only; preserve original range qualifications/status, no role equality inferred'))
  for field in ['new_source_proof_map','overlay_role_proof_map']:
   for n,value in enumerate(accepted.get(field,[])):
    if value.get('source_id')==sid:nav.append(dict(kind='accepted028_exact_role_pointer_map',pointer=f"{paths['accepted']}#/{field}/{n}",original_proof_metadata=value,semantic_status='prior exact role only; new owner029 role not semantically compared'))
  actual_nav=[v for v in nav if v['kind']!='accepted_accounting_navigation_only_not_actual_body_pointer']
  entry=dict(id=f'NATIVE-PROPOSED-OWNER81-PP-{len(records)+1:03}',supplier=supplier,registry=paths[supplier],row_1based=line,registration_id=r['id'],source_id=sid,sha256=r['sha256'],declared_EOF=eof,original_ranges=rr,effective_material_ranges=eff,local_qualification=qualified.get(r['id']),exact_registered_role=role,original_registration=r,classification=classification,already_global=old is not None,accepted_material_range_navigation=oldranges,accepted_range_basis=reason,proposed_new_byte_ranges=new,proposed_new_byte_ranges_qualification='Metadata subtraction only, not semantic producer gap or coverage.',accepted_scope_record=old,accepted_overlay_records=[ob for ob in overlays if ob.get('source_id')==sid],existing_producer_evidence_navigation=nav,actual_body_pointer_found_in_available_metadata=bool(actual_nav),producer_followup_status='not_started_current_lens_remains_owner; exact existing narrative may support future semantic carry only after explicit comparison',unknowns=(['No actual producer body receipt pointer located in available metadata for this source/role; accounting/source identity not equivalent.'] if not actual_nav else ['Whether prior actual producer narrative covers this exact new role is deliberately unassessed.']),coverage_credit=False)
  records.append(entry);groups.setdefault(sid,[]).append(entry['id'])
changes=[k for k,v in paths.items() if hashlib.sha256((B/v).read_bytes()).hexdigest()!=pins[k]['sha256']]
counter=collections.Counter(r['classification'] for r in records)
per={s:dict(registrations=len([r for r in records if r['supplier']==s]),classifications=dict(collections.Counter(r['classification'] for r in records if r['supplier']==s))) for s in ['camera','rig','native']}
newids=sorted({r['source_id'] for r in records if not r['already_global']})
result=dict(id='NATIVE-OWNER81-PROPOSED-PRODUCER-FOLLOWUP-INDEX-'+a.label,at=datetime.datetime.now(datetime.timezone.utc).isoformat(),mode='metadata-only proposal while audit-existing-type-owners remains active; no producer semantics or coverage',input_pins=pins,registrations=records,all_raw_registry_rows_including_qualifications=raw,qualifications=quals,source_membership=groups,new_source_ids=newids,counts=dict(registrations=len(records),qualification_rows=len(quals),source_variants=len(groups),new_source_variants=len(newids),classification_rows=dict(counter),per_supplier=per),changed_inputs_during_snapshot=changes,issues=errors,registry_finality='Snapshot only. Camera/Rig and Native registries may grow; refresh after final supplier pins before authoritative scope reconciliation.',no_source_bytes_read=True,project_execution=0,source_edits=0,authoritative_writes=0,producer_or_other_lens_credit=False,next='Root reviews proposal, final registries and exact material qualifications. No new producer lens dispatched or semantic comparison begun; hold.')
dest=O/f'proposed-producer-followup-index-{a.label}.json';assert not dest.exists();dest.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
summary=dict(id='NATIVE-OWNER81-PROPOSED-PRODUCER-FOLLOWUP-SUMMARY-'+a.label,index=dest.relative_to(B).as_posix(),sha256=hashlib.sha256(dest.read_bytes()).hexdigest(),counts=result['counts'],rows=[dict(id=r['id'],supplier=r['supplier'],registration=r['registration_id'],source_id=r['source_id'],classification=r['classification'],effective_ranges=r['effective_material_ranges'],new_byte_ranges=r['proposed_new_byte_ranges'],producer_navigation_entries=len(r['existing_producer_evidence_navigation']),actual_body_pointer_available=r['actual_body_pointer_found_in_available_metadata']) for r in records],changed_inputs=changes,credit=False,final_refresh_required=True)
(O/f'proposed-producer-followup-summary-{a.label}.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(json.dumps({k:v for k,v in summary.items() if k!='rows'}))
