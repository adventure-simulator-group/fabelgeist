#!/usr/bin/env python3
"""Verify the distinct historical/current facing sources and bounded error relationship."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();a=load('ASSESSMENT-CENTRAL-FACING-13H-95E.json');reg=load('CENTRAL-13H-REGISTRATION.json')
assert len(reg['rows'])==2 and reg['rows'][0]['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee' and reg['rows'][1]['revision']==a['source_revision']
assert reg['rows'][0]['sha256']==reg['rows'][1]['sha256']=='1a50ad5faa518b5ed479870cc03f08e55cd30322c639cd308608aa827fd78d7f'
assert a['source_references']==[{'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows']]
with tarfile.open(R/'delegated/central-13h-windows-proof.tar.gz') as t:
 p=json.load(t.extractfile('proof.json'));bodies=[t.extractfile(p['objects'][x['blob']]['archive']).read() for x in p['paths']];assert bodies[0]==bodies[1] and sha(bodies[0])==reg['rows'][0]['sha256'];s=bodies[0].decode().splitlines()
assert 'dependencies contain a cycle' in s[21] and 'component has no directional working end' in s[24] and 'names a missing component' in s[31]
assert 'c.resolved_id(i)' in s[30] and 'opposed_to.is_some()' in s[46] and 'unwrap_or(Direction::Positive)' in s[6] and 'unwrap_or(Direction::Negative)' in s[7]
def body(archive,path):
 with tarfile.open(R/archive) as t:
  p=json.load(t.extractfile('proof.json'));x=next(x for x in p['paths'] if x['path']==path);return t.extractfile(p['objects'][x['blob']]['archive']).read().decode().splitlines()
v=body('delegated/central-13f-windows-proof.tar.gz','crates/adventuresim-weapon-model/src/recipe/validation.rs');c=body('delegated/central-13f-support-windows-proof.tar.gz','crates/adventuresim-weapon-model/src/model/mod.rs');assert 'facing::directions(self).map_err(|_| RecipeError::Facing)' in v[165] and 'recipe.validate().map_err(|error| error.to_string())' in c[81]
ids=['L12-CLOSURE-WEAPON-FACING-CYCLE-REASON','L12-CLOSURE-WEAPON-FACING-DIRECTION-REASON','L12-CLOSURE-WEAPON-FACING-TARGET-REASON','L12-CLOSURE-WEAPON-FACING-ERROR-CONTEXT'];q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries']
for i,id in zip((2427,2428,2429,2432),ids):assert q[i]['contract_id']==old[i]['contract_id']==id and 'ASSESSMENT-CENTRAL-FACING-13H-95E.json' in q[i]['current_reconciliation']
rows=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(rows)==load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] and len(rows)>=1022
assert len([x for x in rows if x['id']=='REVIEW-WEAPON-FACING-ERROR-CONTEXT-001'])==1
r=load('COVERAGE-L01-ROLES.json')['roles'][2579];assert r['status']=='unassessed' and r['snapshot']=='pinned-native-dependency' and r['source_id'].startswith('physical-p16:')
assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified four separate historical questions and one conditional context relationship; no aliases, role/lens completion or product execution inferred.')
