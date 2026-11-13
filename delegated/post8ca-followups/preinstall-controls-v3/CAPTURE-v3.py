#!/usr/bin/env python3
"""Capture the reviewed and scanned evidence stage before installation."""
from pathlib import Path, PurePosixPath
import hashlib, json, os, stat, sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
base = p.parent
stage = base / 'post8ca-stage-v1/public'
scanned = base / 'post8ca-preinstall-scan-v2/tree/stage'
scan_result = base / 'post8ca-preinstall-scan-v2/RESULT.json'
run = base / 'post8ca-preinstall-scan-run-v2'
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
guard_peer = '93b255b7a99b789a36ee1300b765296c90a9b188cb8ed475959bd2c1dce937b3'
fixed = {
    'peer-guard-v1/MANIFEST.json': '4fcb3f5795c9bf88c650937219e681636347528ada74e4b064e4012ded902d3d',
    'peer-guard-v2/MANIFEST.json': guard_peer,
    'peer-installer-v3/MANIFEST.json': '09555e15e8efc8141de47418c8e2f2d8e295828776b037fac3dc7c686a86d413',
    'final-guard/PINS.json': '5b41da119a95aa46dd9ba99cfb732a288c47034e90322a184bf5f0c715a0ad01',
    'final-guard/STAGING-RECEIPT.json': '1638c470964ff6b0d570a77bc3090ef4e39a5137df118f0292189ebf1e1034cd',
    'final-guard/VERIFY-v1.py': '5f425dcb153b83fa34877690cb7da2f63080efcb6e5e69c09da118d14c906214',
    'final-guard/ROOT-VERIFY-v1.py': '4daa586011b91cb5dc354261b312d526eeead47a419447aac1a27df3954acd36',
    'final-guard/VERIFY.py': '58f3a7710c40874504b6a1f757fc8629f96bc2760f1abb4c4c0748e08a4964fe',
    'final-guard/INSTALLER-CORRECTION-v3.json': 'd6d716c139d3abf6c583349346ee351eb43c01ee4543c6c06c479d8daded906e',
    'ROOT-CANDIDATE-CAPTURE-v3.json': 'b64936bcb597b42f114a50543b3a4e035df501601b0abef73182c37c468edf9c',
    'pending-q2962-source/MANIFEST.json': '69fd1d627d8bbb6b047d6306e910a19afcc2344701aed1d9f826c5655d0ea47d',
}
pin_names = {
    'source-transport/MANIFEST.json', 'pending-source/MANIFEST.json',
    'source-encrypted/MANIFEST.json', 'source-custody-receipt/MANIFEST.json',
    'source-custody-git/MANIFEST.json', 'source-mode-peer-review/MANIFEST.json',
    'process-encrypted/MANIFEST.json', 'process-receipt/MANIFEST.json',
    'later-encrypted/MANIFEST.json', 'later-receipt/MANIFEST.json',
    'central-peer-review/MANIFEST.json', 'central-v4-peer-review/MANIFEST.json',
    'central-v5-peer-review/MANIFEST.json', 'central-v6-peer-review/MANIFEST.json',
    'projection-peer-review/MANIFEST.json', 'peer-application-v1/MANIFEST.json',
    'peer-application-v2/MANIFEST.json', 'reconciliation/VERIFY.py',
    'reconciliation/REVIEWER-QUALIFICATION.json', 'application/MANIFEST.json',
    'application-v2/MANIFEST.json', 'APPLICATION-CORRECTION-v2.json',
    'ROOT-CANDIDATE-CAPTURE.json', 'ROOT-CANDIDATE-CAPTURE-v2.json',
    'ROOT-CANDIDATE-CAPTURE-v3.json', 'INDEX.md',
}

def snapshot(root):
    assert root.is_dir() and not root.is_symlink()
    rows = []
    for f in [root, *sorted(root.rglob('*'))]:
        assert not f.is_symlink() and (f.is_dir() or f.is_file())
        name = '.' if f == root else f.relative_to(root).as_posix()
        assert not any(part in ('__pycache__',) for part in f.relative_to(root).parts)
        assert f.suffix != '.pyc'
        row = {'path': name, 'kind': 'directory' if f.is_dir() else 'file',
               'mode': oct(stat.S_IMODE(f.stat().st_mode))}
        if f.is_file():
            data = f.read_bytes()
            row.update(bytes=len(data), sha256=sha(data), git_blob=blob(data))
        rows.append(row)
    return rows

def verify_fixed():
    for name, expected in fixed.items():
        f = stage / name
        assert f.is_file() and not f.is_symlink() and sha(f.read_bytes()) == expected
    derivation_raw = (base / 'post8ca-final-guard-draft-v2/DERIVATION.json').read_bytes()
    assert sha(derivation_raw) == '3fc567dd540de03c09c84668ab292e218036125d80f2e28193f18e44fb5dcfcb'
    derivation = json.loads(derivation_raw)
    pins = json.loads((stage / 'final-guard/PINS.json').read_bytes())
    assert pins['schema'] == '726-post8ca-stage-pins-v1' and set(pins['sha256']) == pin_names and len(pin_names) == 26
    staging_receipt = json.loads((stage / 'final-guard/STAGING-RECEIPT.json').read_bytes())
    assert staging_receipt['pins_sha256'] == fixed['final-guard/PINS.json'] and staging_receipt['pin_count'] == 26
    assert staging_receipt['guard_peer_v2_manifest_sha256'] == guard_peer
    for peer in ('peer-guard-v1', 'peer-guard-v2', 'peer-installer-v3'):
        directory = stage / peer
        manifest = json.loads((directory / 'MANIFEST.json').read_bytes())
        members = manifest['files']
        assert isinstance(members, dict) and len(members) == 6
        assert {x.name for x in directory.iterdir()} == set(members) | {'MANIFEST.json'}
        for name, row in members.items():
            rel = PurePosixPath(name)
            assert len(rel.parts) == 1 and not rel.is_absolute() and '..' not in rel.parts
            f = directory / name
            assert f.is_file() and not f.is_symlink()
            data = f.read_bytes()
            assert (len(data), sha(data)) == (row['bytes'], row['sha256'])
    for name, expected in pins['sha256'].items():
        rel = PurePosixPath(name)
        assert not rel.is_absolute() and rel.parts and '..' not in rel.parts
        f = stage.joinpath(*rel.parts)
        assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(stage)
        assert sha(f.read_bytes()) == expected
    expected = {
        'schema': '726-post8ca-final-guard-correction-v2',
        'predecessor_guard_sha256': fixed['final-guard/VERIFY-v1.py'],
        'successor_guard_sha256': fixed['final-guard/VERIFY.py'],
        'predecessor_root_wrapper_sha256': fixed['final-guard/ROOT-VERIFY-v1.py'],
        'successor_root_wrapper_sha256': 'b52a3c6d3f20d94dcae073a8a4fad7c2851066fc96665c19d64276d9cbfebf94',
        'root_capture_v2_sha256': pins['sha256']['ROOT-CANDIDATE-CAPTURE-v2.json'],
        'root_capture_v3_sha256': fixed['ROOT-CANDIDATE-CAPTURE-v3.json'],
        'reviewer_v1_manifest_sha256': '4fcb3f5795c9bf88c650937219e681636347528ada74e4b064e4012ded902d3d',
        'changes': derivation['changes'],
        'qualification': derivation['qualification'] + ' The staged guard is not an independently trusted outer anchor.',
    }
    assert (stage / 'final-guard/CORRECTION-v2.json').read_bytes() == (json.dumps(expected, indent=2) + '\n').encode()
    assert {f.name for f in (stage / 'final-guard').iterdir()} == {
        'VERIFY-v1.py', 'VERIFY.py', 'ROOT-VERIFY-v1.py', 'PINS.json',
        'CORRECTION-v2.json', 'INSTALLER-CORRECTION-v3.json', 'STAGING-RECEIPT.json'}

assert not (stage / 'STAGE-CAPTURE.json').exists()
assert not (stage / 'ADOPTION-STATUS.json').exists()
assert not (stage / 'final-guard-prepared-v3').exists()
runner_sha = 'db40c05a0f689595a3ca8e960bfc2502b825fa385eef809c3319c9757d73ec54'
inputs = {
    'scan': (base / 'post8ca-preinstall-scan-v2/RUN.py', '309188e55860c4893698ea6fdb88bd27adf3ec29fa823bdbd6c36ce0d481ab70'),
    'digest': (base / 'next-15a-digest-corpus-358yvabw/corpus/scan-private-digests.py', 'a2cb3c124802385e0edd4e80731bfeb334b95dcd4721589a87f507208d069917'),
    'pattern': (base / 'checkpoint-after13v-prepared-success-paAYncnb/tar-aware-pattern-scanner.py', '4982a500f168f6f5e5ae82a50b4b7e98edda6e5c622e8dc365d0aad10a08bf0f'),
}
assert sha((run / 'RUN.py').read_bytes()) == runner_sha
runner_receipt = json.loads((run / 'RECEIPT.json').read_bytes())
assert runner_receipt['schema'] == '726-post8ca-preinstall-scan-run-v2'
assert runner_receipt['runner_sha256'] == runner_sha
assert runner_receipt['input_script_sha256'] == {name: expected for name, (_, expected) in inputs.items()}
assert runner_receipt['exit'] == 0 and runner_receipt['stderr_bytes'] == 0
assert (run / 'EXIT').read_text().strip() == '0' and (run / 'STDERR').read_bytes() == b''
assert sha((run / 'STDOUT').read_bytes()) == runner_receipt['stdout_sha256']
assert sha((run / 'STDERR').read_bytes()) == runner_receipt['stderr_sha256']
for path, expected in inputs.values():
    assert path.is_file() and not path.is_symlink() and sha(path.read_bytes()) == expected
result_raw = scan_result.read_bytes()
result = json.loads(result_raw)
assert result['schema'] == '726-post8ca-preinstall-scan-v2'
assert result['root_files'] == 7 and result['scanned_paths'] == result['stage_files'] + 7
assert result['windows_single_and_doubled_positive_controls'] is True
assert result['forbidden_digest_total'] == result['exceptions'] == result['tar_pattern_issues'] == 0
assert not result['other_candidate_counts']
assert len(result['checks']) == 2 and {r['label'] for r in result['checks']} == {'digest', 'pattern'}
for r in result['checks']:
    assert r['exit'] == 0 and r['stderr_bytes'] == 0
    assert sha((scan_result.parent / (r['label'] + '.stdout')).read_bytes()) == r['stdout_sha256']
    stderr = (scan_result.parent / (r['label'] + '.stderr')).read_bytes()
    assert not stderr and sha(stderr) == r['stderr_sha256']
assert (stage / 'PREINSTALL-SCAN.json').read_bytes() == result_raw
before = snapshot(stage)
expected_scanned = [x for x in before if x['path'] != 'PREINSTALL-SCAN.json']
assert expected_scanned == snapshot(scanned)
assert result['stage_files'] + 1 == sum(x['kind'] == 'file' for x in before)
assert result['stage_directories_including_root'] == sum(x['kind'] == 'directory' for x in before)
root_capture = json.loads((stage / 'ROOT-CANDIDATE-CAPTURE-v3.json').read_bytes())
root_rows = root_capture['rows']
assert len(root_rows) == 7 and len({r['path'] for r in root_rows}) == 7
scanned_roots = scan_result.parent / 'tree/root'
assert scanned_roots.is_dir() and not scanned_roots.is_symlink()
assert {x.name for x in scanned_roots.iterdir()} == {r['path'] for r in root_rows}
candidate = base / 'post8ca-application-v1/candidate-v2'
coord = base / 'post8ca-final-coordination-v2/root'
for row in root_rows:
    name = row['path']
    rel = PurePosixPath(name)
    assert len(rel.parts) == 1 and not rel.is_absolute() and '..' not in rel.parts
    f = scanned_roots / name
    assert f.is_file() and not f.is_symlink()
    data = f.read_bytes()
    assert (len(data), sha(data), blob(data)) == (row['bytes'], row['sha256'], row['git_blob'])
    source = candidate / 'after' / name
    if not source.is_file():
        source = candidate / 'root' / name
    if not source.is_file():
        source = coord / name
    assert source.is_file() and not source.is_symlink() and source.read_bytes() == data
verify_fixed()
rows, directories = [], []
for x in before:
    if x['path'] == '.':
        continue
    if x['kind'] == 'directory':
        directories.append({'path': x['path'], 'mode': x['mode']})
    else:
        rows.append({k: x[k] for k in ('path', 'bytes', 'sha256', 'git_blob', 'mode')})
        rows[-1]['git_mode'] = '100755' if int(x['mode'], 8) & 0o100 else '100644'
out = {
    'schema': '726-post8ca-stage-capture-v1',
    'parent_checkpoint': '8caeb2003d10d08320566206817aa0c694231778',
    'source_revision': '95e63628f9d70b8dd6e6fede64334696c6995009',
    'source_tree': '1eec97de340d9b7debbb30ca0c0fb2240a73b93f',
    'stage_root_mode': before[0]['mode'], 'capture_file_mode': '0o644',
    'directories': directories, 'rows': rows,
    'qualification': 'Same-machine capture of the scanned stage and added scan receipt, excluding this capture and the future installation status. Full local file and directory modes require the installer comparison; Git preserves owner executable state, not other modes. The capture requires a separately verified Git commit anchor. Scan metadata and later evidence have separate cutoffs; no authenticated restoration, independent custody, source semantics, audit closure or product behavior is inferred.'
}
data = (json.dumps(out, indent=2) + '\n').encode()
assert not (p / 'CAPTURE.json').exists()
assert snapshot(stage) == before
verify_fixed()
for dest in (p / 'CAPTURE.json', stage / 'STAGE-CAPTURE.json'):
    fd = os.open(dest, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o644)
    os.fchmod(fd, 0o644)
    with os.fdopen(fd, 'wb') as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())
    assert oct(stat.S_IMODE(dest.stat().st_mode)) == out['capture_file_mode']
verify_fixed()
after = [x for x in snapshot(stage) if x['path'] != 'STAGE-CAPTURE.json']
assert after == before
print(json.dumps({'files': len(rows), 'directories': len(directories), 'capture_sha256': sha(data)}))
