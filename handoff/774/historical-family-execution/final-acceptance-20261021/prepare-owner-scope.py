"""Derive provisional exact audit obligations from recorded source proofs only."""
from pathlib import Path
import json,hashlib,datetime,collections
A=Path(__file__).resolve().parent;D=A/'wave-01/audit-existing-type-owners';R=A.parent
manifest=json.loads((A/'source-manifest.json').read_text());snapshots={x['id']:x for x in manifest['snapshots']}
old=json.loads((A/'wave-01/source-scope-after-producer.json').read_text());original={(x['source_id'],x.get('sha256')):x for x in old['files']}
frozen_keys={(x['source_id'],x.get('sha256')) for x in manifest['files']}
rows={k:dict(v) for k,v in original.items()};mismatches=[];notes=[];proofs=[]
claims=collections.defaultdict(set)
for owner,name in [('root','root'),('camera','camera_targeting'),('rig','rig_garments'),('native','native_reports')]:
 for x in json.loads((R/f'parallel-ownership/{name}-claims.json').read_text()).get('claims',[]):
  if isinstance(x.get('path',x.get('file')),str):claims[x.get('path',x.get('file'))].add(owner)
for owner in ['root','camera','rig','native']:
 for name in ['coverage.jsonl','additional-sources.jsonl']:
  f=D/owner/name
  for number,line in enumerate(f.read_text().splitlines()):
   x=json.loads(line);shared=x.get('shared_proof',{});sid=x.get('source_id');path=x.get('path',shared.get('path',sid.split(':',1)[-1] if sid and ':' in sid else None));sha=x.get('sha256',x.get('observed_sha256',shared.get('observed_sha256')))
   if not sid or not path:
    notes.append({'record':str(f.relative_to(A)),'row':number,'reason':'Supporting note requires supplier current closure; no source path guessed.'});continue
   if sid.startswith('owner-reached:'):sid='pr-working:'+path
   key=(sid,sha);ranges=x.get('read_ranges',[])
   if not ranges:
    notes.append({'source_id':sid,'record':str(f.relative_to(A)),'row':number,'reason':'No range in this historical/supporting note; retain note and exact supplier closure, do not infer a whole-file audit obligation or coverage.'});continue
   native_path=Path(x.get('absolute_path',path));prefix=sid.split(':',1)[0]
   if not native_path.is_absolute():
    if prefix=='native-dependency':
     matches=[q for q in Path('/home/brunosegovia/.cargo/registry/src').glob('*/'+str(native_path)) if q.is_file() and hashlib.sha256(q.read_bytes()).hexdigest()==sha]
     if len(matches)==1:native_path=matches[0]
     else:native_path=Path(snapshots['pr-working']['root'])/native_path
    else:native_path=Path(snapshots.get(prefix,snapshots['pr-working'])['root'])/native_path
   if sha:
    if not native_path.is_file() or hashlib.sha256(native_path.read_bytes()).hexdigest()!=sha:
     mismatches.append({'source_id':sid,'record':str(f.relative_to(A)),'row':number,'reason':'Recorded source fingerprint does not match resolved path','resolved_path':str(native_path)});continue
   elif key not in original or not x.get('observed_baseline_sha256',x.get('base_sha256')):
    mismatches.append({'source_id':sid,'record':str(f.relative_to(A)),'row':number,'reason':'Unverified absent source; not silently admitted.'});continue
   row=rows.setdefault(key,{'source_id':sid,'snapshot':prefix,'path':path,'sha256':sha,'scope_kind':'materially_reached_owner_contract','bytes':native_path.stat().st_size if native_path.is_file() else None})
   proof={'supplier':owner,'record':str(f.relative_to(A)),'row':number,'read_ranges':ranges,'contract_refs':x.get('contract_refs',x.get('trace_refs',[])),'read_status':x.get('read_status'),'resolved_path':str(native_path)}
   row.setdefault('owner_contract_proofs',[]).append(proof);proofs.append((key,proof));row.setdefault('current_owner_suppliers',[])
   if owner not in row['current_owner_suppliers']:row['current_owner_suppliers'].append(owner)
for key,row in rows.items():
 prior=original.get(key);suppliers=row.get('current_owner_suppliers',[]);exact=claims.get(row['path'],set())
 row['audit_owner']=prior.get('audit_owner',prior.get('owner')) if prior else next(iter(exact)) if len(exact)==1 else suppliers[0] if len(suppliers)==1 else 'root'
 row['audit_owner']=row['audit_owner'] or 'root';row['audit_assignment_limits']='Audit-source responsibility only; exact repair function ownership remains authoritative in existing claim files. Mixed shared functions consume supplier source proofs; no ownership transfer.'
 row['scope_limits']='Original changed variants retain full requested scope. Reached unchanged/foreign sources require exact materially reached contracts/ranges and newly reached signature dependencies; no unrelated whole-file/SDK exemption or implied coverage.'
# Review all current reached proofs at the producer checkpoint, including new
# materially affected uses/ranges in known files. Matching names/ranges alone
# cannot determine that an old trace accounts for the newly reached role.
obligations=[]
for key,row in rows.items():
 reached=[x for x in row.get('owner_contract_proofs',[]) if x['record'].endswith('additional-sources.jsonl')]
 if not reached and key in frozen_keys:continue
 if not reached:reached=row.get('owner_contract_proofs',[])
 obligations.append({'source_id':row['source_id'],'path':row['path'],'sha256':row['sha256'],'audit_owner':row['audit_owner'],'new_variant':key not in original,'scope_basis':'All reached unchanged/foreign contracts plus any newly reached contract proofs on original frozen variants; original producer inventory remains preserved and exact existing trace can be reconciled, not restarted.','owner_contract_proofs':reached,'existing_producer_proofs':row.get('producer_source_proof',[]),'discovery':'incomplete_not_started','required_disposition':'Trace newly reached source/use through retained state/operations/receiving bodies and actual boundary; reconcile exact existing producer proof only where it actually covers that contract. Record concrete repair or source-backed non-applicability; no inference from same file/hash/read count.','source_repair':'frozen_until_all13_initial_inventories'})
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
out={'generated_at':now,'status':'provisional_while_final_owner_shard_reconciles','authority':'Derived scope/assignment preparation only. audit-ledger.json sole discovery/repair/validation authority; existing exact claim files sole repair ownership. No inventory or acceptance judgement supplied by this script.','snapshots':old['snapshots'],'frozen_source_manifest':'../source-manifest.json','scope_limits':'Original PR+preserved isolated variants and all material reached contracts retained. Producer supplement includes expanded material contracts/ranges on known variants, not just new file names. No source/generator/parser/project execution.','files':sorted(rows.values(),key=lambda x:x['source_id']),'producer_supplement_obligations':obligations,'unmatched_proof_observations':mismatches,'historical_or_supporting_notes_without_ranges':notes}
p=A/'wave-01/source-scope-after-owner.provisional.json';p.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'source_variants':len(rows),'new_variants':len(set(rows)-set(original)),'producer_supplement_source_variants':len(obligations),'actual_range_proof_records':len(proofs),'unmatched_proof_observations':len(mismatches),'no_range_notes_retained':len(notes),'status':out['status'],'source_edits':False,'audit_coverage_claimed':False}))
if mismatches:print(json.dumps(mismatches[:8]))
