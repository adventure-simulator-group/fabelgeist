import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021');C=R/'continuation'
prior=C/'cumulative-source-scope-owner-delta-006.json'
scope=json.loads(prior.read_text());now=datetime.datetime.now(datetime.timezone.utc).isoformat()
old={(x['source_id'],x['sha256']) for x in scope['files']}
requests=[]
camera=json.loads((C/'camera/owner-reached-producer-followup/pending-animation-receivers-001.json').read_text())
for x in camera['sources']:
 requests.append({**x,'audit_owner':'camera','reason':camera['purpose'],'scope_limits':'Exact actual P16 retained response/rate config and listed actual receiving operations only; no counterpart/hash/whole-file or unrelated fixture credit.'})
requests.append({'source_id':'physical-p16:crates/fabelgeist-geometry/src/cuboid.rs','sha256':'7afef5cd462b579637596de505b962b8eecc1fd818b8c8382d59ec3ab4fc6b7a','requested_ranges':[[1,80]],'audit_owner':'rig','reason':'Actual P16 CuboidDimensions constructors/axis/native Vec3 projection called by LeafDimensions; PR counterpart not source proof.','scope_limits':'Exact1–80 constructors/projections only;81+Serde excluded until actually needed and registered.'})
roof={
 'roof_geometry.rs':[[1,34],[56,470]],
 'roof_resolution.rs':[[1,738]],
 'roof_assemblies.rs':[[1,277]],
 'roof_cutting.rs':[[1,484]],
 'gable_enclosure.rs':[[1,210]],
 'roof_abutments.rs':[[1,251]],
}
for name,ranges in roof.items():
 path=Path('target/774-snapshots/pr-working/crates/adventuresim-building-generator/src/generator')/name
 requests.append({'source_id':'pr-working:crates/adventuresim-building-generator/src/generator/'+name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'requested_ranges':ranges,'audit_owner':'root','readonly_supplier':'native','reason':'Actual RoofFace/PlaneEquation/Edge/Abutment producer, mutation and geometric receiving paths reached by newly registered model/layout definitions. Constructor/signature navigation only before this registration.','scope_limits':'Whole connected roof-producing functions only. roof_geometry35–55 unrelated bounds fixture excluded. gable_enclosure reached existing fixture constructs actual RoofFace and is included with original cases/oracles; no new fixtures or other generator modules implicitly credited.'})
new=[]
for x in requests:
 snapshot,rel=x['source_id'].split(':',1)
 folder='p16' if snapshot=='physical-p16' else snapshot
 path=Path('target/774-snapshots')/folder/rel
 data=path.read_bytes();assert hashlib.sha256(data).hexdigest()==x['sha256'],x['source_id']
 assert (x['source_id'],x['sha256']) not in old,x['source_id']
 eof=len(data.decode('utf-8').splitlines())
 assert all(1<=lo<=hi<=eof for lo,hi in x['requested_ranges'])
 row={**x,'snapshot':snapshot,'path':str(path.resolve()),'bytes':len(data),'actual_line_count':eof,'material_ranges':x['requested_ranges'],'scope_kind':'materially_reached_producer_followup_contract','scope_extension_authority':'Root explicit pre-body registration during bounded producer followup; metadata/signature navigation only previously.','producer_inventory_accounting':{'status':'incomplete_new_followup_source_before_analysis'},'owner_fit_delta_status':'incomplete_after_current_producer_pass_for_new_contracts','source_or_other_lens_credit':'none from metadata registration'}
 scope['files'].append(row);new.append(row)
assert len(new)==15
over=C/'owner-range-extensions.jsonl';exts=[json.loads(l) for l in over.read_text(encoding='utf-8').splitlines()];assert len(exts)==17
more=[('physical-p16:crates/adventuresim-tactical-core/src/scene_input/buildings/window.rs','f1d40643528b12fce9e1062c6a2e23e0192c6216cb5a5f645bbc624ef90b5c26',[[46,76]],'SceneWindow actual nativeDTO deserialize tail; fixture77+excluded'),('physical-p16:crates/adventuresim-tactical-core/src/scene_input/buildings.rs','2ab63d7b16ae590554651b5dfcd574e8ee3ea6028e227a6ef96bf13b40936403',[[214,221]],'SceneDoorError actual sourcecause wrapper; GeneratedBuilding223+excluded')]
added_ext=[]
for number,(sid,sha,ranges,reason) in enumerate(more,18):
 assert any(x['source_id']==sid and x['sha256']==sha for x in scope['files'])
 x={'id':f'CONT-OWNER-RANGE-EXTENSION-{number:03d}','at':now,'source_id':sid,'sha256':sha,'material_ranges':ranges,'audit_owner':'rig','reason':reason,'scope_limits':reason+'; no wholefile/source equivalence credit.','producer_followup':'pending actual bounded trace after this registration','owner_fit':'pending owner-fit delta after producer evidence','source_or_other_lens_credit':'none from registration'}
 added_ext.append(x)
with over.open('a',encoding='utf-8') as f:
 for x in added_ext:f.write(json.dumps(x,ensure_ascii=False)+'\n')
scope['generated_at']=now;scope['previous_pinned_scope']={'record':str(prior.relative_to(R)),'sha256':hashlib.sha256(prior.read_bytes()).hexdigest()}
scope['status']='Current producer followup in progress;15 new producer sourcevariants and2 new range tails scoped before body analysis; prior2483 owner discovery preserved exact roles only.'
out=C/'cumulative-source-scope-producer-followup-007.json';assert not out.exists();out.write_text(json.dumps(scope,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
packet={'id':'CONT-PRODUCER-FOLLOWUP-SCOPE-EXTENSIONS-020','at':now,'current_lens':'audit-producer-propagation','new_variants':new,'range_extensions':added_ext,'previous_bounded_queue':'continuation/owner-reached-producer-followup-queue-002.json','source_credit':'none from registration','next':'Read exact newly scoped sources under current producer lens, then perform owner-fit delta for newly reached contracts before admission. No old2473 rerun or source repair.'}
p=C/'producer-followup-scope-extensions-020.json';assert not p.exists();p.write_text(json.dumps(packet,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ledger=R/'records/final-acceptance-20261021/audit-ledger.json';a=json.loads(ledger.read_text());a['updated_at']=now
a['current_cumulative_scope']={'record':'../../continuation/'+out.name,'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'variants':len(scope['files']),'limits':'Previous2483 exact roles preserved;15 new producer variants, mandatory19 overlays; no credit from metadata.','range_extensions':{'record':'../../continuation/owner-range-extensions.jsonl','sha256':hashlib.sha256(over.read_bytes()).hexdigest(),'mandatory_overlay':True}}
owner=next(x for x in a['lenses'] if x['lens']=='audit-existing-type-owners');owner['discovery']='prior2483 exhausted_with_findings; incomplete_new_producer_reached15variants_and2ranges_delta_after_current_producer'
for l in a['lenses']:l['scoped_variants']=len(scope['files'])
a['continuation']['producer_followup_scope_extensions']=str(p.relative_to(R));ledger.write_text(json.dumps(a,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'scope_variants':len(scope['files']),'new_variants':15,'range_overlays':19,'body_analysis_credit':0}))
