import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021')
base=R/'records/parallel-ownership'
paths=list(base.glob('*-claims.json'))
ledgers={p:json.loads(p.read_text()) for p in paths}
target=base/'rig_garments-claims.json'
groups=[
 ('F001','crates/fabelgeist-compute/src/sort/mod.rs',['RadixSort::record'],[]),
 ('F003','crates/adventuresim-core/src/alcohol.rs',['ethanol_ml','water_content_ml','properties_valid','emergency_hydration_ml','units_for_target','consumable_units','tavern_units_affordable','evening_id','evening_boundary','next_evening_boundary_after','EveningIds::next','checked_interval','crossed_evenings','RestEvenings::next','rest_evenings','TravelEveningSegments::next','travel_evening_segments','qualifying_heavy','hydration_after_expected_drinking','surgery_control_bonus','best_disinfectant'],['AlcoholProperties','EveningIds','RestEvenings','TravelEveningSegments','ScopedAlcoholSupply']),
 ('F003','crates/strategic-web/src/routes/settlements/rest_preview.rs',['consumable_fraction','soap_uses','calculate_rest_supply_availability','calculate_soap_rest_preview'],['RestSupplySources','SoapRestPreview']),
 ('F004','crates/adventuresim-character-creator/src/device_extremities.rs',['record_boot'],[]),
 ('F004','crates/adventuresim-character-creator/src/device_footwear_fit.rs',['record_section_fit','record_ankle_fairing','fit_source'],[]),
 ('F005','crates/fabelgeist-armor/src/gpu/helmet/surface.rs',['latitudes','CoordSurface::vertex','CoordSurface::coord','CoordSurface::connect','CoordSurface::full_dome','CoordSurface::styled_dome','CoordSurface::fan_dome','CoordSurface::shell'],['Latitude','CoordSurface']),
 ('F005','crates/fabelgeist-armor/src/gpu/helmet/parts.rs',['HelmetParts::parts','morion','kettle_hat','brimmed'],[]),
 ('F005','crates/fabelgeist-armor/src/gpu/helmet/burgonet.rs',['burgonet'],[])
]
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
existingpins=[{'record':str(p.relative_to(R)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]
added=[]
for finding,path,functions,records in groups:
 # Exact whole-function overlap check on bookkeeping, not source analysis.
 for p,a in ledgers.items():
  for old in a['claims']:
   if old.get('path',old.get('file'))!=path:continue
   funcs=old.get('functions',[])
   if isinstance(funcs,str):funcs=[funcs]
   assert not set(functions)&set(funcs),(p,path,functions,funcs)
 row={'file':path,'path':path,'functions':functions,'records':records,
      'finding':'RIG-OWNER-DELTA-'+finding,
      'requirements':['Whole connected function/retained-record repair for every affected family; canonical types through producers, collections, helpers and actual consumers, scalar extraction only at real numerical/native codec.',
       'Preserve raw invalid-input normalization, valid zero, precision, authored quantization, frame/authority, arithmetic/hash/draw/error/sort/mutation order and existing fixture inputs/count/oracles. No compatibility path, duplicate owner or callsite conversion-only repair.',
       'Source freeze until all13 initial inventories; other-lens new paths/ranges registered before analysis. Existing native/camera canonical leaves/errors/SDK/query and function scopes preserved.'],
      'evidence':'continuation/rig/owner-fit-new-claim-targets-001.json + mandatory path correction002; continuation/root/rig235-owner-reconciliation-013.json',
      'authority':'Root explicit whole-function reconciliation after exported claim metadata overlap check; source audit responsibility did not itself award repair ownership.',
      'status':'ownership only; all13 initial source freeze'}
 ledgers[target]['claims'].append(row)
 added.append({'claim_index':len(ledgers[target]['claims'])-1,'path':path,'functions':functions,'finding':finding})
ledgers[target].setdefault('claim_reconciliation',[])
cr=ledgers[target]['claim_reconciliation']
note={'at':now,'record':'continuation/root/rig-owner-claims-reconciliation-014.json','scope':'New sort/Core alcohol/rest helpers/boot/helmet whole functions; existing copy/soap forecast and Native/Camera door/grab retained.'}
if isinstance(cr,list):cr.append(note)
elif isinstance(cr,dict):cr['owner_delta_014']=note
else:raise TypeError(type(cr))
target.write_text(json.dumps(ledgers[target],ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
out={'id':'CONT-ROOT-RIG-OWNER-CLAIMS-RECONCILIATION-014','at':now,'status':'exact whole-function ownership reconciled; source freeze remains','prior_claim_pins':existingpins,'added_claims':added,
 'existing_claims_preserved':['Rig KernelBatch::copy_buffer claim272','Rig soap_rest_preview424 and forecast425','Native door/window predicates native-exact128','Camera WorldGrabTargets pointed/visible claim1','Native canonical SignedLength/scaled_by native-exact018 and Radians max/direction native-exact023'],
 'neutral_Radians_dependency':'Root coordinates relocating existing Radians from adventuresim-building-generator/src/spatial_geometry/orientation.rs to lowest coherent neutral geometry owner. Native max/direction whole-function scope remains; do not duplicate angle wrapper or add Armor->generator cycle. Exact destination/API synthesis follows all13 discovery.',
 'limits':'No new source or role analysis, no implementation authorization before all13; whole claims intentionally include later necessary signature/record propagation, requiring source-scope extension before any unregistered body analysis.',
 'claim_result':{'record':str(target.relative_to(R)),'sha256':hashlib.sha256(target.read_bytes()).hexdigest()}}
p=R/'continuation/root/rig-owner-claims-reconciliation-014.json'
assert not p.exists()
p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'new_whole_function_groups':len(added),'function_names':sum(len(x['functions']) for x in added),'source_edits':0}))
