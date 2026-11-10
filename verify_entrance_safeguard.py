"""Verify bounded entrance ordinal source evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-ENTRANCE-SAFEGUARD-95E.json'
doc = json.loads((root / name).read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert doc['source_revision'] == revision and doc['lens'] == 1 and name in progress['new_nonfinding_assessments']
def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        assert path not in out
        out[path] = oid
    return out
current = tree('main-advance-95e/current-tree.tsv'); previous = tree('main-advance-95e/previous-tree.tsv'); sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip' and item['revision'] == revision
    raw = gzip.decompress(path.read_bytes()); oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert oid == item['git_blob'] == current[item['path']] == previous[item['path']]
    assert all(1 <= a <= b <= item['lines'] for a,b in item['ranges']) and item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 7
def src(suffix):
    found = [v for k,v in sources.items() if k.endswith(suffix)]
    assert len(found) == 1
    return found[0]
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
entrance = src('generator/entrances.rs')
must(entrance, ['.unwrap_or(0)', 'requirements[entrance_room.index()]', 'requirements[usize::from(wall.inside_room)]'])
storeys = src('generator/occupied_storeys.rs')
must(storeys, ['if let Some(storey) = small_church::occupied_storey(program)?', 'if storey_program.rooms.is_empty()', 'GenerationError::EmptyStorey', 'room_allocation::allocate(', '&storey_program.rooms,', 'derive_walls(footprint_cells, &assignments)?', 'derive_openings('])
allocation = src('generator/room_allocation.rs')
must(allocation, ['let mut assignments = reservations.clone()', '.get_mut(room_index.index())', 'AllocationFailure::UnknownRoomReservation', 'requirements.iter().enumerate()', 'RoomIndex::from_ordinal(room_index)?', 'assignments.insert(candidate.cell, candidate.room)', 'assignments.get(&cell.neighbour(direction)).copied()'])
derivation = src('generator/wall_derivation.rs')
must(derivation, ['assignments\n                .get(&cell)', 'assignment(cell)?.serialized_ordinal()', 'entrances::append(', 'requirements,'])
must(src('model/room_index.rs'), ['u16::try_from(ordinal)', 'pub const fn index(self) -> usize', 'pub const fn serialized_ordinal(self) -> u16'])
must(src('generator/orchestration.rs'), ['occupied_storeys::generate_storeys(program, edits)?'])
must(src('generator/core.rs'), ['generate_unchecked(program, &[])'])
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes = {x['original']: x['sha256'] for x in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as archive:
    for h in doc['historical_sources']:
        r = roles[h['original_role_index']]; path = h['source_id'].split(':', 1)[1]
        assert r['source_id'] == h['source_id'] and r['historical_sha256'] == h['sha256'] == hashes[h['original_role_index']]
        assert r['historical_assessment'] == h['assessment'] and hashlib.sha256(archive.extractfile(h['sha256']).read()).hexdigest() == h['sha256']
        assert hashlib.sha256(sources[path].encode()).hexdigest() == h['sha256']
assert len(doc['historical_sources']) == 3 and len(doc['dispositions']) == 1
print('verified seven Git-bound sources and three historical roles; no product ran')
