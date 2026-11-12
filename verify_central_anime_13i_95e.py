#!/usr/bin/env python3
"""Verify bounded source identities and connected source claims, without product execution."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();a=load('ASSESSMENT-CENTRAL-ANIME-13I-95E.json');refs=[];bodies={}
for tag in ('13I','13I-SUPPORT','13I-BRIDGE','13I-HISTORY','13I-OLD-ANIME'):
 reg=load('CENTRAL-'+tag+'-REGISTRATION.json');refs.extend({'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows'])
 with tarfile.open(R/('delegated/central-'+tag.lower()+'-windows-proof.tar.gz')) as t:
  p=json.load(t.extractfile('proof.json'))
  for x in p['paths']:
   raw=t.extractfile(p['objects'][x['blob']]['archive']).read();assert sha(raw)==x['sha256'];bodies[(x['revision'],x['path'])]=raw
assert refs==[{k:x[k] for k in ('revision','path','lines')} for x in a['source_references']] and len(refs)==16
assert a['source_references'][0]['revision_role']=='historical object transport, not the old project head' and a['source_references'][0]['recorded_historical_head']=='6533e444fa37552927070cc8d8190907fd3b413e'
h=next(raw for (rev,p),raw in bodies.items() if p.endswith('3fd27205e209b9133d640381c16458c180e205625a017b93138e363d14b04b86'));current=bodies[(a['source_revision'],'crates/fabelgeist-armor/src/anime_design.rs')];assert h==current and sha(h)=='3fd27205e209b9133d640381c16458c180e205625a017b93138e363d14b04b86'
def line(path,n):return bodies[(a['source_revision'],path)].decode().splitlines()[n-1]
an='crates/fabelgeist-armor/src/anime_design.rs';lib='crates/fabelgeist-armor/src/lib.rs';design='crates/fabelgeist-armor/src/design.rs';warm='crates/adventuresim-character-creator/src/runtime_equipment/warmup.rs';equip='crates/adventuresim-character-creator/src/runtime_equipment.rs';recipe='crates/adventuresim-character-creator/src/armor_recipes.rs';bridge='crates/adventuresim-character-creator/src/device_torso.rs';gpu='crates/fabelgeist-armor/src/gpu/breastplate/mod.rs'
assert 'pub struct Millimeters(pub u16)' in line(design,16) and 'lap_lift: Millimeters(7)' in line(an,35) and 'gauge.0 * 2' in line(an,50)
assert 'profile.validate()' in line(lib,185) and 'anime.validate(design.wall_thickness)' in line(lib,187) and '(1..=20).contains(&design.wall_thickness.0)' in line(lib,205)
assert 'serde_json::from_slice' in line('crates/adventuresim-character-creator/src/design_input.rs',22) and 'validate_breastplate(&design)' in line('crates/adventuresim-character-creator/src/design_input.rs',23)
assert 'breastplate.clone()' in line(warm,54) and 'BreastplateConstruction::Anime(Default::default())' in line(warm,56) and 'generate(body, cuirass_selection, bracer, &alternate' in line(warm,59)
assert 'generate_runtime_armor(' in line(equip,133) and 'generate_breastplate(gpu, body, breastplate_design)' in line(equip,160) and '"breastplate" | "cuirass"' in line(recipe,87)
assert 'DeviceBreastplate::record(' in line(bridge,231) and 'validate_breastplate(design)' in line(gpu,110)
assert 'if articulated { 6 } else { 20 }' in line('crates/adventuresim-character-creator/src/anime_controls.rs',21) and 'shape.lap_lift.0.max(minimum_lift)' in line('crates/adventuresim-character-creator/src/anime_controls.rs',31)
asset='assets_src/equipment/breastplate-design.json';assert '"wall_thickness": 2' in line(asset,14) and '"construction": "Solid"' in line(asset,36)
old=load('baseline-059/queue.json')['entries'][641];q=load('RECONCILIATION-059.json')['entries'][641];assert old['kind']=='pending-contract-or-revisit' and old['first_failed_contract_or_unresolved_question']['unique_count_change']==0 and old['first_failed_contract_or_unresolved_question']['id']=='SEGOVB-L11-ANIME-GAUGE-VALIDATION-ORDER' and 'ASSESSMENT-CENTRAL-ANIME-13I-95E.json' in q['current_reconciliation']
rows=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(rows)==load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] and len(rows)>=1024 and all(sum(x['id']==name for x in rows)==1 for name in a['new_qualified_ledger_rows'])
r=load('COVERAGE-L01-ROLES.json')['roles'][2609];assert r['status']=='unassessed' and r['snapshot']=='pr-working';assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified bounded historical/current source, pending 059 gauge record and separate warmup condition; no input, device or product execution inferred.')
