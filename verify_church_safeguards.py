"""Verify bounded church admission and temporary-identity evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-CHURCH-SAFEGUARDS-95E.json'
doc = json.loads((root / name).read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
if doc['source_revision'] != revision or doc['lens'] != 1 or name not in progress['new_nonfinding_assessments']:
    raise ValueError('incorrect attribution')

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, oid = header.split()
        if name in out:
            raise ValueError('duplicate tree entry')
        out[name] = oid
    return out

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe source')
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
        hashlib.sha256(raw).hexdigest() != item['sha256'] or oid != item['git_blob'] or
        current.get(item['path']) != oid or previous.get(item['path']) != oid or
        any(not 1 <= a <= b <= item['lines'] for a, b in item['ranges'])):
        raise ValueError('source mismatch: ' + item['path'])
    if item['path'] in sources:
        raise ValueError('duplicate source')
    sources[item['path']] = raw.decode()
if len(sources) != 12:
    raise ValueError('incorrect source count')

def src(suffix):
    matches = [v for k, v in sources.items() if k.endswith(suffix)]
    if len(matches) != 1:
        raise ValueError('missing source ' + suffix)
    return matches[0]

def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

assembly = src('generator/church_assembly.rs')
must(assembly, ['let owner = GeometryOwnerId(70_000);', '(0..church_program.nave_bays)', 'let clerestory_west = nave_axes_metres[0] - bay;', 'let clerestory_owner = owner;', 'let clerestory_host = wall_solid(\n                geometry,\n                clerestory_owner,\n                0,', 'source: crate::WallSourceId::ChurchArcade {', 'church_window_schedule::schedule(church_program)', 'wall.source == source', 'openings.push(resolve_church_pointed_window('])
helper = src('generator/wall_derivation.rs')
must(helper, ['let id = ResolvedItemId((1_u64 << 60) | (u64::from(owner.0) << 32) | slot);', 'geometry.solids.push(ResolvedSolid::new(', 'ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | slot)'])
schedule = src('generator/church_window_schedule.rs')
must(schedule, ['for bay_index in 0..church_program.nave_bays', 'for side in [Direction::South, Direction::North]', 'source: crate::WallSourceId::ChurchArcade {'])
windows = src('generator/church_windows.rs')
must(windows, ['let slot = 0x20_000 + serial * 0x40;', 'let removed = wall.host_solids.clone();', 'geometry.solids.retain(|solid| !removed.contains(&solid.id));', '.retain(|interface| !removed_interfaces.contains(&interface.id));', 'wall.host_solids = host_solids;', 'wall.opening_ids = vec![opening_id];'])
must(src('church_programme.rs'), ['programme != ChurchProgram::URBAN_BRICK_BASILICA', 'return Err(GenerationError::InvalidChurchProgram)'])
must(src('model/programme.rs'), ['pub const URBAN_BRICK_BASILICA: Self = Self {', 'nave_bays: 4,'])
must(src('generator/urban_church.rs'), ['if program.church_program.is_none()', 'resolve_church_assembly('])
must(src('generator/architectural_envelope.rs'), ['self.church = urban_church::resolve('])
must(src('generator/orchestration.rs'), ['program.validate_church_program()?;', 'plan.resolve_architectural_envelope(program, edits)'])
must(src('audit/resolved_geometry.rs'), ['!item_ids.insert(solid.id)', '"invalid_resolved_geometry"'])
must(src('audit/core.rs'), ['pub fn audit_plan(', 'audit_resolved_geometry(plan, issues)?'])
must(src('generator/core.rs'), ['generate_unchecked(program, &[])', 'let issues = crate::audit_plan(&plan)?;'])
h = doc['historical_source']; roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']; r = roles[h['original_role_index']]
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as archive:
    assert r['source_id'] == h['source_id'] and r['historical_sha256'] == h['sha256'] == hashes[h['original_role_index']]
    assert hashlib.sha256(archive.extractfile(h['sha256']).read()).hexdigest() == h['sha256']
assert hashlib.sha256(assembly.encode()).hexdigest() == h['sha256'] and r['historical_assessment'] == h['assessment']
assert len(doc['dispositions']) == 2
print('verified 12 Git-bound sources and separately attributed church role; no product ran')
