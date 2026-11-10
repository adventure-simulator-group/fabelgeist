"""Verify HUD evidence and static arithmetic, not product behavior.

Run verify_lock_inventory.py first to reconstruct the referenced Git tree.
"""
import gzip
import hashlib
import json
import re
import tarfile
import tomllib
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-HUD-IDENTITY.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, _, oid = header.split()
    if path in tree:
        raise ValueError('duplicate tree path')
    tree[path] = oid


def safe_path(name):
    path = (root / name).resolve()
    if not path.is_relative_to(root):
        raise ValueError('evidence path escapes checkpoint')
    return path


sources = {}
for source in doc['current_sources']:
    raw = gzip.decompress(safe_path(source['preserved_bytes']).read_bytes())
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (source['revision'] != doc['current_revision']
            or hashlib.sha256(raw).hexdigest() != source['sha256']
            or len(raw) != source['bytes'] or len(raw.splitlines()) != source['lines']
            or blob != source['git_blob'] or blob != tree.get(source['path'])):
        raise ValueError('source mismatch: ' + source['path'])
    if any(not 1 <= lo <= hi <= source['lines'] for lo, hi in source['ranges']):
        raise ValueError('invalid source range')
    sources[source['path']] = raw.decode()

lock = tomllib.loads((root / 'lock-evidence/current-Cargo.lock').read_text())
locked = {item['name']: item for item in lock['package']}
native = doc['contracts'][1]['native_evidence']
if len(native) != 5 or len({item['package'] for item in native}) != 5:
    raise ValueError('native archive population mismatch')
for item in native:
    manifest = json.loads(safe_path(item['manifest']).read_text())
    raw = safe_path(item['archive']).read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if (digest != item['sha256'] or digest != manifest['sha256']
            or len(raw) != manifest['bytes']
            or locked[item['package']]['version'] != item['version']
            or locked[item['package']]['checksum'] != digest):
        raise ValueError('native archive or lock mismatch')
    with tarfile.open(safe_path(item['archive']), 'r:gz') as archive:
        for member in manifest['inspected_members']:
            name = PurePosixPath(member['path'])
            if name.is_absolute() or '..' in name.parts:
                raise ValueError('unsafe archive member')
            data = archive.extractfile(name.as_posix()).read()
            if len(data) != member['bytes'] or hashlib.sha256(data).hexdigest() != member['sha256']:
                raise ValueError('native member mismatch')

core = sources['crates/adventuresim-core/src/equipment.rs']
client = sources['crates/adventuresim-tactical-client/src/equipment.rs']
mapping = core.split('pub const INPUT_ADDRESS_MAPPINGS:')[1].split('\n];', 1)[0]
conversion = client.split('fn key_code(input: &str)')[1].split('\n}', 1)[0]
inputs = re.findall(r'input:\s*"([^"\n]+)"', mapping)
branches = re.findall(r'^\s*"([^"\n]+)"\s*=>\s*KeyCode::', conversion, re.M)
if (inputs != doc['contracts'][0]['mechanical_source_comparison']['mapping_inputs']
        or branches != doc['contracts'][0]['mechanical_source_comparison']['native_key_branches']
        or len(inputs) != 20 or len(set(inputs)) != 20
        or len(branches) != 20 or len(set(branches)) != 20
        or set(inputs) != set(branches)):
    raise ValueError('input comparison mismatch')

# Integer-only verification of the stated binary64 tie-to-even example.
a = (2**21 << 32) | (2**32 - 4)
b = a + 1
if (a, b) != (9007203549708284, 9007203549708285):
    raise ValueError('example identity mismatch')
if ((a & 0xffffffff) ^ 0xffffffff, (b & 0xffffffff) ^ 0xffffffff) != (3, 2):
    raise ValueError('example pinned index representation mismatch')
q, remainder = divmod(b, 2)
rounded_b = 2 * (q if remainder == 0 or q % 2 == 0 else q + 1)
if a % 2 != 0 or rounded_b != a:
    raise ValueError('example tie-to-even mismatch')

baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
ids = {entry['id'] for entry in baseline['repair_ledger']['entries']}
if not {'NEW-HUD-SLOT-INPUT-001', 'NEW-HUD-JOIN-IDENTITY-001'} <= ids:
    raise ValueError('missing connected ledger entries')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('connected ledger count mismatch')
print('verified HUD source, five native archives and integer identity example; reachability remains open')
