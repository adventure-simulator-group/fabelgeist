#!/usr/bin/env python3
"""Verify bounded 059 room, tower and opening reconciliation."""
import gzip
import json
import tarfile
from pathlib import Path
R = Path(__file__).resolve().parent
load = lambda p: json.loads((R / p).read_text())
name = 'ASSESSMENT-DELEGATED-RECONCILIATION-13A-95E.json'
a = load(name)
ids = ['SEGOVB-L13-RETAINED-ROOM-ORDINALS', 'SEGOVB-L13-ROUND-TOWER-WALL-DIMENSIONS', 'SEGOVB-L13-OPENING-DIMENSION-PROPAGATION']
old = load('baseline-059/queue.json')['entries']
index = load('RECONCILIATION-059.json')['entries']
assert [old[i]['contract_id'] for i in (404, 406, 408)] == ids
assert [x['historical_id'] for x in a['central_observations']] == ids
assert all(index[i]['current_reconciliation'].startswith('partial ') and name in index[i]['current_reconciliation'] for i in (404, 406, 408))
assert a['new_qualified_ledger_rows'] == ids[1:] and a['further_evidence_for_existing_rows'] == ids[:1]
ledger = {x['id']: x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
assert name in ledger[ids[0]]['further_current_evidence']
for ident in ids[1:]:
    row = ledger[ident]
    assert row['record'] == name and row['repair'] == 'not started' and row['validation'] == 'not performed' and row['discovery'].startswith('partial')
reg = load(a['central_display_evidence']['registration'])
assert a['source_references'] == [{'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows']]
with tarfile.open(R / 'delegated/central-13a-windows-proof.tar.gz') as t:
    proof = json.load(t.extractfile('proof.json'))
    src = {(x['revision'], x['path']):t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
cur = a['source_revision']; root = 'crates/adventuresim-building-generator/src/'
def line(path, n): return src[(cur, root + path)][n - 1]
assert 'if cells == 0' in line('model.rs',43) and 'Self::new(cells)' in line('model.rs',78)
assert 'let room_index = RoomIndex::from_ordinal' in line('generator/room_allocation.rs',164) and 'room_index.serialized_ordinal()' in line('generator/room_allocation.rs',166)
assert 'MissingCellAssignment' in line('generator/wall_derivation.rs',16) and 'serialized_ordinal()' in line('generator/wall_derivation.rs',19)
assert 'pub width_metres: f32' in line('model/layout.rs',52) and 'pub sill_metres: f32' in line('model/layout.rs',53) and 'pub height_metres: f32' in line('model/layout.rs',54)
assert 'tower_anchor_matches_diameter' in line('model/layout.rs',314) and 'wall_thickness_metres,' in line('model/layout.rs',321)
assert 'wire.wall_thickness_metres,' in line('model/layout.rs',394) and 'tower.radius_metres() - tower.wall_thickness_metres' in line('generator/round_towers.rs',36)
assert a['repair'] == 'not started' and a['validation'] == 'not performed'
assert load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled'] == 0
print('Verified three bounded historical relationships, two qualified ledger reconnections and cited source lines; no lens closure or product execution.')
