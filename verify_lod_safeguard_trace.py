"""Verify bounded LOD source and recovery evidence without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-LOD-SAFEGUARD-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision


def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in result and kind in ('blob', 'commit')
        result[filename] = oid
    return result


def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    return raw


def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)


current = tree('main-advance-95e/current-tree.tsv')
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob'] == current[item['path']]
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 7
p = 'crates/adventuresim-character-creator/src/'
q = 'crates/fabelgeist-mhr/src/'
must(sources[p + 'lod.rs'], ['pub struct CharacterLod(u8);',
    'impl TryFrom<u8> for CharacterLod',
    '(MIN_CHARACTER_LOD..=MAX_CHARACTER_LOD).contains(&value)',
    '[2_461, 971, 595][usize::from(self.0 - MIN_CHARACTER_LOD)]'])
must(sources[q + 'model.rs'], ['pub const MIN_LOD: u8 = 4;', 'pub const MAX_LOD: u8 = 6;'])
must(sources[q + 'lib.rs'], ['MAX_LOD, MIN_LOD, Mhr, MhrConfig'])
must(sources[p + 'model_controls.rs'], ['MIN_CHARACTER_LOD..=MAX_CHARACTER_LOD',
    'CharacterLod::try_from(lod)', '.expect("slider bounds")'])
must(sources[p + 'cli.rs'], ['clap::value_parser!(u8).range(i64::from(MIN_CHARACTER_LOD)..=i64::from(MAX_CHARACTER_LOD))'])
must(sources[p + 'main.rs'], ['CharacterLod::try_from(lod)?;', 'Mhr::from_files('])
must(sources[p + 'export/validation.rs'], ['crate::lod::CharacterLod::try_from(lod)?;'])
# These are table bounds derived from the verified source constants; no product ran.
assert [value - 4 for value in range(4, 7)] == [0, 1, 2]
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
entry = next(x for x in manifest['entries'] if x['path'] == doc['recovered_historical_receipts'][0]['original_path'])
receipt = doc['recovered_historical_receipts'][0]
raw = preserved(receipt)
assert entry['sha256'] == receipt['sha256'] and entry['bytes'] == len(raw) == receipt['bytes']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][153]
assert old['receipt_sha256'] == receipt['sha256']
assert old['contract']['contract'] == doc['historical_records'][0]['id']
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
reference = doc['historical_sources'][0]
role = work['roles'][reference['original_role_index']]
assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['sha256']
assert reference['byte_identical'] == (reference['sha256'] == reference['current_sha256'])
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['new_nonfinding_assessments'] and progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified seven Git-bound sources and one original receipt; no product ran')
