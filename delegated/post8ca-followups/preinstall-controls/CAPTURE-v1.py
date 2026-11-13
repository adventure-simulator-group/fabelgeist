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
run = base / 'post8ca-preinstall-scan-run-v1'
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
guard_peer = '93b255b7a99b789a36ee1300b765296c90a9b188cb8ed475959bd2c1dce937b3'
fixed = {
    'peer-guard-v2/MANIFEST.json': guard_peer,
    'peer-installer-v3/MANIFEST.json': '09555e15e8efc8141de47418c8e2f2d8e295828776b037fac3dc7c686a86d413',
    'final-guard/VERIFY-v1.py': '5f425dcb153b83fa34877690cb7da2f63080efcb6e5e69c09da118d14c906214',
    'final-guard/ROOT-VERIFY-v1.py': '4daa586011b91cb5dc354261b312d526eeead47a419447aac1a27df3954acd36',
    'final-guard/VERIFY.py': '58f3a7710c40874504b6a1f757fc8629f96bc2760f1abb4c4c0748e08a4964fe',
    'final-guard/INSTALLER-CORRECTION-v3.json': 'd6d716c139d3abf6c583349346ee351eb43c01ee4543c6c06c479d8daded906e',
    'ROOT-CANDIDATE-CAPTURE-v3.json': 'b64936bcb597b42f114a50543b3a4e035df501601b0abef73182c37c468edf9c',
    'pending-q2962-source/MANIFEST.json': '69fd1d627d8bbb6b047d6306e910a19afcc2344701aed1d9f826c5655d0ea47d',
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
assert (run / 'EXIT').read_text().strip() == '0' and (run / 'STDERR').read_bytes() == b''
result_raw = scan_result.read_bytes()
result = json.loads(result_raw)
assert result['schema'] == '726-post8ca-preinstall-scan-v2'
assert result['forbidden_digest_total'] == result['exceptions'] == result['tar_pattern_issues'] == 0
assert not result['other_candidate_counts']
assert all(r['exit'] == 0 and r['stderr_bytes'] == 0 for r in result['checks'])
assert (stage / 'PREINSTALL-SCAN.json').read_bytes() == result_raw
before = snapshot(stage)
expected_scanned = [x for x in before if x['path'] != 'PREINSTALL-SCAN.json']
assert expected_scanned == snapshot(scanned)
assert result['stage_files'] + 1 == sum(x['kind'] == 'file' for x in before)
assert result['stage_directories_including_root'] == sum(x['kind'] == 'directory' for x in before)
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
    'stage_root_mode': before[0]['mode'], 'directories': directories, 'rows': rows,
    'qualification': 'Same-machine capture of the scanned stage and added scan receipt, excluding this capture and the future installation status. Full local file and directory modes require the installer comparison; Git preserves owner executable state, not other modes. The capture requires a separately verified Git commit anchor. Scan metadata and later evidence have separate cutoffs; no authenticated restoration, independent custody, source semantics, audit closure or product behavior is inferred.'
}
data = (json.dumps(out, indent=2) + '\n').encode()
assert not (p / 'CAPTURE.json').exists()
assert snapshot(stage) == before
verify_fixed()
for dest in (p / 'CAPTURE.json', stage / 'STAGE-CAPTURE.json'):
    with dest.open('xb') as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())
verify_fixed()
after = [x for x in snapshot(stage) if x['path'] != 'STAGE-CAPTURE.json']
assert after == before
print(json.dumps({'files': len(rows), 'directories': len(directories), 'capture_sha256': sha(data)}))
