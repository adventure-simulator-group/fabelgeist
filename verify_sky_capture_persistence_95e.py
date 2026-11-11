#!/usr/bin/env python3
"""Verify sky/targeting records and pinned source, without product execution."""
import gzip
import hashlib
import json
import re
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
def entries(data):
    result, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']

rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-SKY-CAPTURE-PERSISTENCE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1742-1744.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('SKY-CAPTURE-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1742:787,1743:60,1744:84}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert old == body and c['byte_identical'] is True and c['current_personal_ranges'] == []
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
sky = current[1742]
cli = sources['crates/adventuresim-tactical-client/src/tactical_sky_viewer_main.rs']
assert b'output: PathBuf' in cli and b'tactical_sky_viewer::run(args.view, args.output, args.settle_frames)' in cli
assert sky.index(b'save_to_disk(&path)(captured)') < sky.index(b'let manifest_path = path.with_extension("manifest.json")') < sky.index(b'if manifest.validation.passed') < sky.index(b'exit.write(AppExit::Success)')
assert b'fs::remove_file(output.with_extension("failure.txt"))' in sky and b'fs::create_dir_all(parent)' in sky
bevy = load(a['native_bevy_source'])
image = load(a['native_image_source'])
lockbytes = (root / a['current_cargo_lock']).read_bytes()
assert oid('blob', lockbytes) == tree['Cargo.lock']
packages = tomllib.loads(lockbytes.decode())['package']
for meta in (bevy,image):
    package = next(x for x in packages if x['name'] == meta['package'] and x['version'] == meta['version'])
    archive = root / 'native-evidence' / (meta['package']+'-'+meta['version']) / meta['archive']
    assert package['checksum'] == meta['sha256'] == sha(archive.read_bytes())
with tarfile.open(root / 'native-evidence/bevy_render-0.19.1/bevy_render-0.19.1.crate','r:gz') as archive:
    native = archive.extractfile('bevy_render-0.19.1/src/view/window/screenshot.rs').read()
assert b'pub fn save_to_disk(path: impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)' in native
assert b'match img.try_into_dynamic()' in native and b'image::ImageFormat::from_path(&path)' in native
assert b'requested format not recognized' in native and b'screen format cannot be understood' in native and b'Cannot save screenshot, IO error:' in native
with tarfile.open(root / 'native-evidence/image-0.25.10/image-0.25.10.crate','r:gz') as archive:
    fmt = archive.extractfile('image-0.25.10/src/io/format.rs').read().decode()
part = fmt[fmt.index('pub fn from_extension'):fmt.index('pub fn from_path')]
assert '"png" | "apng" => ImageFormat::Png' in part and '_ => return None' in part and '"notimage"' not in part
assert '.and_then(ImageFormat::from_extension)' in fmt and 'ImageError::Unsupported(format_hint.into())' in fmt
player = sources['crates/adventuresim-tactical-client/src/player.rs']
grab = sources['crates/adventuresim-tactical-client/src/equipment/grab_world.rs']
assert b'attack_target_within_angular_threshold(' in player and b'body_stable_auto_aim(' in player and b'stable_key: hitbox.to_bits()' in player
assert b'entity.to_bits()' in grab and b'.chain(self.doors' in grab and b'.chain(' in grab
assert b'camera_forward.try_normalize()?' in current[1744] and b'.then(left.3.cmp(&right.3))' in current[1744]

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(561,[1742]), (562,[1743]), (563,[1744])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for x in receipt.get('partial_sequence', []):
        assert x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
    for key in ('qualified_supporting_same_lens_reuse','qualified_supporting_native_same_lens_reuse'):
        for x in receipt.get(key, []):
            assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
            assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
    if n == 561:
        assert len(receipt['partial_sequence']) == 2 and len(receipt['qualified_supporting_native_same_lens_reuse']) == 1
finding = a['existing_finding_connections'][0]
assert finding['id'] == 'SEGOVB-L12-SKY-CAPTURE-PERSISTENCE-OUTCOME'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
q = load('RECONCILIATION-059.json')['entries']
for i in (259,1439,1440,1441):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified sky and targeting roles, pinned format/save chain and prior attribution; no product execution.')
