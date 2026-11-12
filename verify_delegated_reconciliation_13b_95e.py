#!/usr/bin/env python3
"""Verify bounded cell-neighbour and player wall coordinate reconciliation."""
import json
import tarfile
from pathlib import Path
R = Path(__file__).resolve().parent
load = lambda p: json.loads((R / p).read_text())
name = 'ASSESSMENT-DELEGATED-RECONCILIATION-13B-95E.json'
a = load(name)
ids = ['SEGOVB-04-BUILDING-CELL-NEIGHBOR-ENDPOINT-CANDIDATE-001', 'CANDIDATE-PLAYER-WALL-ENDPOINT-COORDINATE-OWNER-NARROWING']
assert [x['id'] for x in a['central_observations']] == a['new_qualified_ledger_rows'] == ids
ledger = {x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in ids:
    assert ledger[ident]['record'] == name and ledger[ident]['repair'] == 'not started' and ledger[ident]['validation'] == 'not performed' and ledger[ident]['discovery'].startswith('partial')
assert name in ledger['REVIEW-EDITOR-LEGACY-PARTS-COMPATIBILITY-001']['further_current_evidence']
with tarfile.open(R / 'delegated/building-model-hair-ops-evidence.tar.gz') as t:
    old = json.load(t.extractfile('historical/774/audit-continuation/operations/contract-groups-614.json'))
    owner = json.load(t.extractfile('historical/774/audit/continuation/rig/initial06-type-ownership/contracts-382.json'))
    assert old['findings'][0]['id'] == ids[0] and 'Playerbuild/maxendpoint actualcaller stilluntraced' in old['findings'][0]['qualification']
    assert owner['id'] == 'RIG-INITIAL06-GROUP-382' and 'GridPoint' in owner['groups']['architectural-lattice']['comparison']
reg = load(a['central_display_evidence']['registration'])
with tarfile.open(R / 'delegated/central-13b-windows-proof.tar.gz') as t:
    proof = json.load(t.extractfile('proof.json'))
    src = {x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
with tarfile.open(R / 'delegated/central-13a-windows-proof.tar.gz') as t:
    proof = json.load(t.extractfile('proof.json'))
    model = next(x for x in proof['paths'] if x['path'].endswith('/model.rs') and x['revision'] == a['source_revision'])
    src[model['path']] = t.extractfile(proof['objects'][model['blob']]['archive']).read().decode().splitlines()
root='crates/adventuresim-building-generator/src/'
def line(p,n): return src[root+p][n-1]
assert 'GRID_UNITS_PER_CELL: i32 = 30' in line('model.rs',16) and 'self.x as f32 * GRID_UNIT_METRES' in line('model.rs',32)
assert 'self.x + offset.x as i16' in line('model.rs',130)
assert 'let mut next = self.clone()' in line('model/player_build.rs',141)
assert 'Cell::new(x as i16, (start.z - 1) as i16)' in line('model/player_build.rs',217)
assert 'Cell::new((start.x - 1) as i16, z as i16)' in line('model/player_build.rs',241)
assert 'wall.cell.neighbour(wall.direction)' in line('viewer/player_build_scene.rs',241)
assert '(start.x / CELL_SIZE_METRES).round() as i32' in line('viewer/editor_input.rs',117)
assert 'width > i16::MAX as u16' in line('generator/layout.rs',4)
assert a['repair'] == 'not started' and a['validation'] == 'not performed'
print('Verified two conditional endpoint relationships, their historical attribution and cited source lines; no product execution or lens closure.')
