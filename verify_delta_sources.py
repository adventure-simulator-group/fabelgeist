"""Verify preserved source bytes and declared coverage, not product behavior.

Run verify_lock_inventory.py first; it reconstructs the Git trees represented
by the two inventories that this check uses to bind paths to blobs.
"""
import hashlib
import gzip
import json
from pathlib import Path

root = Path(__file__).resolve().parent
current_revision = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
selected_revision = 'ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04'


def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, kind, oid = header.split()
        if kind not in ('blob', 'commit') or name in result:
            raise ValueError('invalid tree entry')
        result[name] = oid
    return result


trees = {
    current_revision: tree('CURRENT-TREE.tsv'),
    selected_revision: tree('lock-evidence/selected-tree.tsv'),
}


def verify_source(path, source):
    revision = source['revision']
    if revision not in trees or source['preserved_bytes'].startswith('/'):
        raise ValueError('invalid source reference')
    source_path = (root / source['preserved_bytes']).resolve()
    if not source_path.is_relative_to(root):
        raise ValueError('source reference escapes checkpoint')
    raw = source_path.read_bytes()
    if source.get('content_encoding') == 'gzip':
        raw = gzip.decompress(raw)
    elif source.get('content_encoding') is not None:
        raise ValueError('unknown source encoding')
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (hashlib.sha256(raw).hexdigest() != source['sha256']
            or len(raw) != source['bytes']
            or len(raw.splitlines()) != source['lines']
            or blob != source['git_blob']
            or blob != trees[revision].get(path)):
        raise ValueError('source bytes or tree binding mismatch: ' + path)
    for first, last in source.get('ranges', []):
        if not (1 <= first <= last <= source['lines']):
            raise ValueError('invalid bounded source range: ' + path)


manifest = json.loads((root / 'ASSESSMENT-L01-DELTA-MANIFESTS.json').read_text())
workflow = json.loads((root / 'ASSESSMENT-L01-WORKFLOW-FEATURE.json').read_text())
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
paths = {item['path']: item for item in baseline['source_delta']}
if len(manifest['records']) != 6 or len({r['path'] for r in manifest['records']}) != 6:
    raise ValueError('manifest population mismatch')
for item in manifest['records']:
    path = item['path']
    verify_source(path, item['current'])
    verify_source(path, item['reported_selected'])
    if item['current']['revision'] != current_revision or item['reported_selected']['revision'] != selected_revision:
        raise ValueError('manifest revision mismatch')
    if item['personally_inspected_current_ranges'] != [[1, item['current']['lines']]]:
        raise ValueError('manifest read range mismatch')
    if 'ASSESSMENT-L01-DELTA-MANIFESTS.json' not in paths[path]['lens_status'][0]:
        raise ValueError('missing manifest disposition')

for key, revision in [('current_sources', current_revision), ('reported_selected_sources', selected_revision)]:
    if len(workflow[key]) != 2:
        raise ValueError('workflow source population mismatch')
    for item in workflow[key]:
        if item['revision'] != revision:
            raise ValueError('workflow revision mismatch')
        verify_source(item['path'], item)
if 'ASSESSMENT-L01-WORKFLOW-FEATURE.json' not in paths['.github/workflows/rust-quality.yml']['lens_status'][0]:
    raise ValueError('missing workflow disposition')
noncode = json.loads((root / 'ASSESSMENT-L01-NONCODE-DELTA.json').read_text())
readme_path = 'crates/adventuresim-tactical-client/README.md'
readmes = [item for item in noncode['paths'] if item['path'] == readme_path]
if len(readmes) != 1:
    raise ValueError('client guide disposition missing')
readme = readmes[0]
verify_source(readme_path, readme['current_source'])
verify_source(readme_path, readme['previous_source'])
if (readme['current_source']['revision'] != current_revision or
        readme['previous_source']['revision'] != selected_revision or
        readme['personally_inspected_current_ranges'] != [[1, readme['current_source']['lines']]] or
        'ASSESSMENT-L01-NONCODE-DELTA.json' not in paths[readme_path]['lens_status'][0]):
    raise ValueError('client guide disposition mismatch')
statuses = [item['lens_status'][0] for item in baseline['source_delta']]
if sum(s.startswith('assessed:') for s in statuses) != progress['delta_lens_cells_assessed']:
    raise ValueError('assessed delta count mismatch')
if sum(s.startswith('partial') for s in statuses) != progress['delta_lens_cells_partial']:
    raise ValueError('partial delta count mismatch')
print('verified manifest, workflow and client guide source pairs and qualified delta counts')
