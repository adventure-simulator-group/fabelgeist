import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021')
B=R/'records/parallel-ownership'
all_ledgers={p.name:json.loads(p.read_text()) for p in B.glob('*-claims.json')}
before={n:hashlib.sha256((B/n).read_bytes()).hexdigest() for n in all_ledgers}
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
targets=[
 ('root-claims.json','crates/adventuresim-world-schema/src/lib.rs',['RouteTerrain::class_for','expected_route_seasonal_risks','expected_route_encounter_tags'],[],'CONT-ROOT-OWNER-F001','Carry exact existing route slope/roughness/relief/elevation leaves through helpers and actual consumers, preserving each unit, zero, threshold and error/Serde order.'),
 ('root-claims.json','crates/adventuresim-tactical-core/src/regional_terrain.rs',['RegionalTerrainScale::spacing_metres'],[],'CONT-ROOT-OWNER-F002','Retain exact shared positive metre leaf for five actual f32 spacings; preserve original f32 constants and actual f64 promotion point through request/capture helpers. Those separately scoped consumers keep existing owners.'),
 ('root-claims.json','crates/adventuresim-strategic-sim/src/live_core/config.rs',['default_simulation_disease','CoreLoopConfig::validate'],['CoreLoopConfig.disease'],'CONT-ROOT-OWNER-F003','Use existing DiseaseId while preserving narrower nine-disease fixture whitelist, default Influenza, first validation order, and all existing configuration/caller semantics. Canonical twelve variants do not broaden fixture support.'),
 ('camera_targeting-claims.json','crates/adventuresim-core/src/smithing.rs',['quote_weapon'],['ForgeQuote'],'CONT-CAMERA-OWNER-F001','Use exact ForgeStock keys through quote requirements/materials; preserve brass/leather/steel/wood lexical first-error and consumption order, open rejected material observations, mass/fraction precision and actual lower causes. Lower Core cannot depend on DB operation aggregate.'),
 ('camera_targeting-claims.json','crates/adventuresim-stdb-module/src/smithing.rs',['available','consume'],[],'CONT-CAMERA-OWNER-F001','Compose these helpers once with existing Camera forge_weapon claim235; canonical stock key until real native lookup/update, exact sorted native row consumption and preflight/error/mutation order. No quote-map string adapter.'),
 ('native_reports-claims.json','crates/adventuresim-core/src/world_event.rs',['WorldEventEnvelope::validate','validate_id'],['WorldEventPlace'],'CONT-NATIVE-OWNER-F005 / NATIVE-CONT-WORLD-F001','Coordinate lower event place record/validation with Camera DB food infection/commit_world_event261. Actual admitted StrategicPlaceId up to4096 differs generic raw event identifier192 policy; retain exact namespaces and input/rejection evidence rather than blanket narrow or broad string erasure. Final aggregate/enum/admission synthesis follows all13 discovery.')
]
added=[]
for name,path,functions,records,finding,requirement in targets:
 for oldname,a in all_ledgers.items():
  for old in a['claims']:
   if old.get('path',old.get('file'))!=path:continue
   oldf=old.get('functions',old.get('items',[]))
   if isinstance(oldf,str):oldf=[oldf]
   assert not set(functions)&set(oldf),(oldname,path,functions,oldf)
 row={'file':path,'functions':functions,'records':records,'requirements':[requirement,'Whole connected function/record repair across all affected families after all13 initial discovery. No source edits, new tests/cases/assertions, compatibility alias or duplicate owner authorized by this bookkeeping.','Keep current PR/P15 chronology and newer parent APIs while composing P16/P17. Existing claims/canonical operations and precise source scopes preserved; register additional body/range before analysis.'],'finding':finding,'authority':'Root explicit exact whole-function reconciliation from owner findings and gap map; source audit assignment was not automatic transfer.','evidence':'continuation/native/owner-delta-claim-gap-map-001.json; Root current scoped signature navigation verifies exact identifiers, not new body coverage.','status':'ownership only; source freeze until all13 initial inventories'}
 if name=='root-claims.json':row['path']=path
 if name=='native_reports-claims.json':
  row['claim_id']='native-owner-world-place-001';row['items']=row.pop('functions');row['context_refs']=[row['evidence']]
 all_ledgers[name]['claims'].append(row)
 added.append({'ledger':name,'claim_index':len(all_ledgers[name]['claims'])-1,'path':path,'functions':functions,'records':records,'finding':finding})
for name in {x['ledger'] for x in added}:
 (B/name).write_text(json.dumps(all_ledgers[name],ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
out={'id':'CONT-ROOT-OWNER-CLAIM-GAP-RECONCILIATION-016','at':now,'input':'continuation/native/owner-delta-claim-gap-map-001.json','before_claim_sha256':before,'added_claims':added,
 'source_signature_navigation':'Exact existing paths and signatures only. RouteTerrain::class_for3190; expected_route_seasonal_risks3299; expected_route_encounter_tags3388; RegionalTerrainScale::spacing_metres64; default_simulation_disease44/CoreLoopConfig::validate72; quote_weapon19; available11/consume27; WorldEventEnvelope::validate261/validate_id359. No broader signature-search output or source body receives additional audit credit.',
 'neutral_Radians':'Root014 neutral owner relocation dependency preserved; exact API/location choice intentionally follows all13 synthesis with Native max/direction ownership retained. No human decision needed.',
 'already_closed':'Camera383/384 complete prior World029 missing DB targets; Rig426–433/014 cover sort/alcohol/rest/boot/helmet. No duplicate assignments.',
 'remaining_limits':'Consumers and plan constructors already held by existing whole-function scopes retain them. Any genuinely new connected function needs its own explicit reconciliation before repair. Broad existing raw event-place policy is an invariant/aggregate design obligation, not waived.',
 'source_edits':0,'other_lens_or_validation_credit':'none','claim_result_sha256':{name:hashlib.sha256((B/name).read_bytes()).hexdigest() for name in all_ledgers}}
p=R/'continuation/root/owner-claim-gap-reconciliation-016.json';assert not p.exists();p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ledger=R/'records/execution-ledger.json';a=json.loads(ledger.read_text())
a['events'].append({'at':now,'kind':'exact_whole_function_ownership_reconciliation','record':'continuation/root/owner-claim-gap-reconciliation-016.json','new_groups':6,'source_edits':0,'family_status':'unchanged'})
a['shared_repairs'].append({'id':'CONT-OWNER-DELTA-CLAIMS-016','record':'continuation/root/owner-claim-gap-reconciliation-016.json','status':'ownership_reconciled_source_frozen','scope':'Root route/spacing/fixtureconfig; Camera Core quote and DB stock helpers; Native lower Core event-place validation composed with Camera261.'})
a['updated_at']=now;ledger.write_text(json.dumps(a,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'new_groups':len(added),'function_names':sum(len(x['functions']) for x in added),'status':'metadata only; no source/family advance'}))
