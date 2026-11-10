"""Verify recovered cache-policy evidence, without running MHR or product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-CORRECTIVE-CACHE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in out and kind in ('blob', 'commit')
        out[filename] = oid
    return out

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob']
    assert current[item['path']] == previous[item['path']] == oid
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 11
p = 'crates/adventuresim-character-creator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

must(sources[p + 'model_controls.rs'], ['checkbox(&mut studio.selected_correctives, "Pose-corrective model")'])
must(sources[p + 'main.rs'], ['reload_model.before(regenerate_mesh)', 'studio.selected_correctives == model.correctives',
     'pose_correctives: correctives', '*model = loaded;', 'studio.dirty = true;'])
cache = sources[p + 'studio_cache.rs']
must(cache, ['model.lod,', '&recipe.proportions,', '&recipe.identity,', '&recipe.expression,',
     'return Ok(body.clone());', 'self.fits.clear();', 'self.drape = None;'])
key = cache[cache.index('let key = serde_json::to_vec(&('):cache.index('if let Some((cached, body))')]
assert 'correctives' not in key
must(sources[p + 'studio_generation.rs'], ['let generated = cache.body(&model, &recipe)?;',
     '.drape_changed(&loadout)?', 'drape_job.request(inputs);'])
must(sources[p + 'generation.rs'], ['model.mhr.forward(identity, pose, Some(expression))?',
     'proportions::joint_bases('])
must(sources[p + 'proportions.rs'], ['ModelParameterTable::filled(model.num_model_parameters(), 0.0)',
     'for channel in JointChannel::ROTATION_AND_SCALE', 'translation_only'])
must(sources[p + 'bare_body.rs'], ['correctives: model.correctives'])
must(sources['crates/fabelgeist-mhr/src/model.rs'], ['let correctives = if config.pose_correctives {',
     'joint_parameters = joint_parameters + offsets.clone();',
     'if apply_correctives && let Some(correctives) = &self.correctives',
     'rest = rest + correctives.forward(joint_parameters)?;'])
must(sources['crates/fabelgeist-mhr/src/model_def.rs'], ['// A bare constant is an offset on this joint channel.'])
must(sources['crates/fabelgeist-mhr/src/correctives.rs'], ['features.matmul(self.activation.clone())',
     'hidden.matmul(self.basis.clone())'])
must(sources['crates/fabelgeist-mhr/src/correctives/input.rs'], ['JointChannel::ROTATIONS[0].native_index()',
     'JointChannel::ROTATIONS.len()'])
receipts = {}
for item in doc['recovered_historical_receipts']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    receipts[item['original_path']] = json.loads(raw)
assert len(receipts) == 5
assert receipts['segovb/VERIFY-L11-716.json']['receipt_sha256'] == '21a52e797712f42a6296032d93cb2735f034922d5543da74834017e83ff2cb1d'
assert receipts['segovb/READ-L13-PARALLEL-117.json']['delegated_receipt']['sha256'] == '5e564ec40c7b5dfd101f45e31967d5e7eb1c5e1707ff8156131f22a923b7b404'
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
queue = json.loads((root / 'baseline-059/queue.json').read_text())
old = ledger['segovb_continuation']['current_connected_dispositions'][275]
proposal = ledger['segovb_continuation']['lens13']['delegated_findings_pending_primary_provenance_and_alias'][13]
item = queue['entries'][2327]
assert old['disposition']['id'] == doc['historical_records'][0]['id']
assert proposal['finding']['proposed_id'] == doc['historical_records'][1]['id']
assert item['contract_id'] == doc['historical_records'][2]['id']
assert not old['latest_current_target_reconciled'] and not proposal['current_reconciled']
assert old['disposition']['accepted_unique_count_change'] == proposal['accepted_unique_count_change'] == 0
queue_sources = {x['source_id'].split(':')[2]: x['source_sha256'] for x in item['exact_source_identities_and_assessed_ranges']}
for path, sha in queue_sources.items():
    assert hashlib.sha256(sources[path].encode()).hexdigest() == sha
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
recon = json.loads((root / 'RECONCILIATION-059.json').read_text())
match = [x for x in recon['entries'] if x['source_pointer'] == '/entries/2327']
assert len(match) == 1 and match[0]['contract_id'] == item['contract_id']
assert match[0]['current_reconciliation'].startswith('partial current 95e')
assert name in match[0]['current_reconciliation']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started'
assert candidate['validation'] == 'not performed'
print('verified eleven current sources and five recovered receipts; no assets or product ran')
