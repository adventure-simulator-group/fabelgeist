#!/usr/bin/env python3
"""Install a captured audit-evidence candidate into the isolated checkout."""
from pathlib import Path
import datetime, hashlib, json, os, shutil, stat, subprocess, sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
base = p.parent
root = base.parent / 'publication'
stage = base / 'post8ca-stage-v1/public'
candidate = base / 'post8ca-application-v1/candidate-v2'
coord = base / 'post8ca-final-coordination-v2/root'
dest = root / 'delegated/post8ca-followups'
parent = '8caeb2003d10d08320566206817aa0c694231778'
tree = '246d1442c001f653da8f2ea3094620473aae1c32'
expected_capture = '__CAPTURE_SHA256__'
names = ['BASELINE-95E-20261110.json', 'CONTINUATION-PROGRESS-95E.json',
         'RECONCILIATION-059.json', 'CONTINUATION.md']
new = {
    'ASSESSMENT-CENTRAL-FOLLOWUPS-23-95E.json': candidate / 'root/ASSESSMENT-CENTRAL-FOLLOWUPS-23-95E.json',
    'MAIN-RECHECK-POST8CA.json': candidate / 'root/MAIN-RECHECK-POST8CA.json',
    'verify_post8ca_checkpoint.py': coord / 'verify_post8ca_checkpoint.py',
}
env = dict(os.environ, GIT_OPTIONAL_LOCKS='0')
git = lambda *args: subprocess.check_output(['git', *args], cwd=root, env=env)
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
cap_bytes = (stage / 'STAGE-CAPTURE.json').read_bytes()
assert sha(cap_bytes) == expected_capture
assert cap_bytes == (base / 'post8ca-preinstall-v1/CAPTURE.json').read_bytes()
cap = json.loads(cap_bytes)
assert cap['schema'] == '726-post8ca-stage-capture-v1' and cap['parent_checkpoint'] == parent
assert cap['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert cap['source_tree'] == '1eec97de340d9b7debbb30ca0c0fb2240a73b93f'

def verify_stage(directory):
    assert directory.is_dir() and not directory.is_symlink()
    assert oct(stat.S_IMODE(directory.stat().st_mode)) == cap['stage_root_mode']
    assert (directory / 'STAGE-CAPTURE.json').read_bytes() == cap_bytes
    paths = list(directory.rglob('*'))
    assert all(not x.is_symlink() and (x.is_file() or x.is_dir()) for x in paths)
    assert not any(x.name == '__pycache__' or x.suffix == '.pyc' for x in paths)
    files = {x.relative_to(directory).as_posix(): x for x in paths if x.is_file()}
    dirs = {x.relative_to(directory).as_posix(): x for x in paths if x.is_dir()}
    wanted = {r['path']: r for r in cap['rows']}
    wd = {r['path']: r for r in cap['directories']}
    assert len(wanted) == len(cap['rows']) and len(wd) == len(cap['directories'])
    assert set(files) == set(wanted) | {'STAGE-CAPTURE.json'} and set(dirs) == set(wd)
    for name, row in wanted.items():
        f = files[name]
        data = f.read_bytes()
        assert (len(data), sha(data), blob(data), oct(stat.S_IMODE(f.stat().st_mode))) == (
            row['bytes'], row['sha256'], row['git_blob'], row['mode'])
        assert row['git_mode'] == ('100755' if f.stat().st_mode & 0o100 else '100644')
    for name, row in wd.items():
        assert oct(stat.S_IMODE(dirs[name].stat().st_mode)) == row['mode']

verify_stage(stage)
assert git('rev-parse', 'HEAD').decode().strip() == parent
assert git('rev-parse', 'HEAD^{tree}').decode().strip() == tree
assert git('branch', '--show-current').decode().strip() == 'codex/726-audit-baseline-20261110'
assert not git('status', '--porcelain=v1', '-uall')
assert dest.parent.is_dir() and not dest.parent.is_symlink()
assert not dest.exists() and not dest.is_symlink()
assert not (stage / 'ADOPTION-STATUS.json').exists()
root_capture_raw = (stage / 'ROOT-CANDIDATE-CAPTURE-v3.json').read_bytes()
assert sha(root_capture_raw) == 'b64936bcb597b42f114a50543b3a4e035df501601b0abef73182c37c468edf9c'
rows = json.loads(root_capture_raw)['rows']
expected = {r['path']: r for r in rows}
assert len(rows) == len(expected) == 7 and set(expected) == set(names) | set(new)
for name in names:
    f = root / name
    assert f.is_file() and not f.is_symlink()
    assert f.read_bytes() == (candidate / 'before' / name).read_bytes() == git('show', parent + ':' + name)
    assert not f.stat().st_mode & 0o100
for name, src in {**{n: candidate / 'after' / n for n in names}, **new}.items():
    assert src.is_file() and not src.is_symlink()
    data = src.read_bytes()
    row = expected[name]
    assert row['mode'] == '100644'
    assert (len(data), sha(data), blob(data)) == (row['bytes'], row['sha256'], row['git_blob'])
for name in new:
    assert not (root / name).exists() and not (root / name).is_symlink()
for name in names:
    tmp = root / ('.' + name + '.post8ca-install.tmp')
    assert not tmp.exists() and not tmp.is_symlink()

log = (p / 'ACTIONS.jsonl').open('x', buffering=1)
def record(event, **kwargs):
    log.write(json.dumps({'event': event, **kwargs}, sort_keys=True) + '\n')
    log.flush()
    os.fsync(log.fileno())

record('preconditions_passed', parent=parent, tree=tree, root_paths=len(rows))
shutil.copytree(stage, dest, symlinks=False)
verify_stage(stage)
verify_stage(dest)
record('captured_stage_copied_and_compared', capture_sha256=sha(cap_bytes))
for name, src in new.items():
    with (root / name).open('xb') as f:
        f.write(src.read_bytes())
        f.flush()
        os.fsync(f.fileno())
    (root / name).chmod(0o644)
    record('new_root_file', path=name, sha256=sha((root / name).read_bytes()))
for name in names:
    tmp = root / ('.' + name + '.post8ca-install.tmp')
    with tmp.open('xb') as f:
        f.write((candidate / 'after' / name).read_bytes())
        f.flush()
        os.fsync(f.fileno())
    tmp.chmod(stat.S_IMODE((root / name).stat().st_mode))
    assert tmp.read_bytes() == (candidate / 'after' / name).read_bytes()
    record('replacement_prepared', path=name)
for name in names:
    assert (root / name).read_bytes() == (candidate / 'before' / name).read_bytes()
    os.replace(root / ('.' + name + '.post8ca-install.tmp'), root / name)
    record('replacement_installed', path=name, sha256=sha((root / name).read_bytes()))
for row in rows:
    data = (root / row['path']).read_bytes()
    assert (len(data), sha(data), blob(data)) == (row['bytes'], row['sha256'], row['git_blob'])
status = {
    'schema': '726-post8ca-installation-status-v1',
    'status_at_recording': 'installed in the local evidence checkout; publication is separate',
    'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'parent_checkpoint': parent, 'parent_tree': tree,
    'stage_capture_sha256': sha(cap_bytes),
    'root_candidate_capture_sha256': sha(root_capture_raw),
    'root_paths_matched': len(rows), 'stage_files_matched': len(cap['rows']),
    'qualification': 'Exact parent, clean worktree, before-image and absent-target checks preceded a sequential, nontransactional evidence-only installation. Later evidence verification, privacy scan, commit, publication and recovery are separate; source semantics, queue closure and product behavior are not established.',
}
with (dest / 'ADOPTION-STATUS.json').open('x') as f:
    json.dump(status, f, indent=2)
    f.write('\n')
    f.flush()
    os.fsync(f.fileno())
record('installation_status_written', sha256=sha((dest / 'ADOPTION-STATUS.json').read_bytes()))
log.close()
with (p / 'RESULT.json').open('x') as f:
    json.dump({'schema': '726-post8ca-install-result-v1', 'parent': parent,
               'root_paths': len(rows), 'stage_files': len(cap['rows']),
               'status_sha256': sha((dest / 'ADOPTION-STATUS.json').read_bytes()),
               'qualification': 'Local evidence-only installation; verification and publication are separate.'}, f, indent=2)
    f.write('\n')
    f.flush()
    os.fsync(f.fileno())
print(json.dumps({'root_paths': len(rows), 'stage_files': len(cap['rows']),
                  'status_sha256': sha((dest / 'ADOPTION-STATUS.json').read_bytes())}))
