"""Verify retained artillery route evidence and ideal waypoint arithmetic; no product execution."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction as F
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-ARTILLERY-ROUTE-95E.json'
doc = json.loads((root / name).read_text())
if doc['lens'] != 1 or doc['source_revision'] != revision:
    raise ValueError('wrong source attribution')

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, oid = header.split()
        if name in out:
            raise ValueError('duplicate tree path')
        out[name] = oid
    return out

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe or misattributed source')
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
if len(sources) != 10:
    raise ValueError('unexpected source count')

def src(suffix):
    found = [v for k, v in sources.items() if k.endswith(suffix)]
    if len(found) != 1:
        raise ValueError('missing source: ' + suffix)
    return found[0]

def must(text, markers):
    for marker in markers:
        if marker not in text:
            raise ValueError('missing marker: ' + marker)

producer = src('generator/artillery_castle.rs')
must(producer, ['sweep_path: vec![\n            curtain_operator_start,', 'Vec3::new(8.35, 5.84, -8.35)', 'Vec3::new(6.8, 5.84, -8.35)', 'Vec3::new(6.0, 5.84, -10.5)', '.windows(2)', '(pair[0].distance(pair[1]) / 0.30).ceil() as usize', '(0..steps).filter_map', '((segment == 0) || step > 0)', '.chain([to])', 'ResolvedSurfaceShape::RouteCorridor'])
must(src('generator/architectural_envelope.rs'), ['self.artillery_castle = resolve_artillery_castle('])
must(src('generator/core.rs'), ['pub fn generate(program: &BuildingProgram)', 'generate_unchecked(program, &[])'])
must(src('generator/orchestration.rs'), ['plan.resolve_architectural_envelope(program, edits)'])
must(src('model/artillery.rs'), ['Ordered floor-centre samples used by the physical occupant sweep', 'pub sweep_path: Vec<Vec3>'])
clearance = src('audit/artillery_clearance.rs')
must(clearance, ['const MAX_ROUTE_SEGMENT_METRES: f32 = 3.0', 'const SWEEP_SAMPLE_SPACING_METRES: f32 = 0.35', 'let path_valid = edge.sweep_path.len() >= 2', 'self.route_sweep_blocker(edge, &sweep)?', 'for pair in sweep.windows(2)', 'for step in 0..=steps'])
audit = src('audit/artillery.rs')
must(audit, ['visibility.first_route_failure(', '"disconnected_artillery_route"'])
must(src('audit/core.rs'), ['pub fn audit_plan(', 'audit_structural_assemblies(plan, &mut issues)?', 'audit_artillery_castle(plan, issues)?'])
must(src('viewer/proof_markers.rs'), ['for pair in edge.sweep_path.windows(2)', 'artillery authoritative swept circulation edge'])
must(src('audit/resolved_geometry_helpers.rs'), ['fn artillery_route_solid_contains(', 'resolved_solid_contains_point(solid, point, tolerance)'])
# An ideal source-shaped illustration, not product f32 execution.
a = (F(835, 100), -F(835, 100)); b = (F(680, 100), -F(835, 100)); c = (F(600, 100), -F(1050, 100))
assert F(15,10) < a[0] - b[0] < F(18,10)
length2 = (c[0]-b[0])**2 + (c[1]-b[1])**2
assert F(21,10)**2 < length2 < F(24,10)**2
p = tuple(a[i] + (b[i]-a[i])*F(5,6) for i in range(2))
q = tuple(b[i] + (c[i]-b[i])*F(1,8) for i in range(2))
assert p == (F(847,120), -F(167,20)) and q == (F(67,10), -F(1379,160))
cross = (q[0]-p[0])*(b[1]-p[1]) - (q[1]-p[1])*(b[0]-p[0])
assert cross != 0
h = doc['historical_source']; role = h['historical_source_role']
ledger = json.loads((root / h['ledger']).read_text())
def find(value):
    if isinstance(value, dict):
        if value.get('contract_id') == h['contract_id']:
            yield value
        for v in value.values():
            yield from find(v)
    elif isinstance(value, list):
        for v in value:
            yield from find(v)
records = list(find(ledger))
assert len(records) == 1 and any('route sample-gap/corridor' in x for x in records[0]['acceptance_requirements'])
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in manifest['sources']}
r = roles[role['index']]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as archive:
    assert r['source_id'] == role['source_id'] and r['historical_sha256'] == role['sha256'] == hashes[role['index']]
    assert hashlib.sha256(archive.extractfile(role['sha256']).read()).hexdigest() == role['sha256']
assert hashlib.sha256(producer.encode()).hexdigest() == role['sha256'] and r['historical_assessment'] == role['assessment']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and candidate['discovery'].startswith('partial')
assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
observation = json.loads((root / baseline['latest_main_recheck']).read_text())
assert observation['observed_revision'] == revision and observation['method'] == 'git ls-remote origin refs/heads/main'
print('verified ten Git-bound sources, historical attribution and ideal waypoint arithmetic; no product ran')
