"""Archive Git/text metadata without running Rust or repository check tools."""
from datetime import datetime, timezone
from pathlib import Path
import difflib
import hashlib
import json
import subprocess
import zipfile

reports = Path(__file__).resolve().parent
worktree = reports.parent / 'rig-review-774/worktree'
source_base = '82cb2a3acccc852bba6a7ef9c706c337ba9fe765'
def git(*args):
    return subprocess.check_output(['git', *args], cwd=worktree)
def digest(data):
    return hashlib.sha256(data).hexdigest()
head = git('rev-parse','HEAD').decode().strip()
paths = set(git('diff',source_base,'--no-renames','--name-only','-z').decode().split('\0'))
untracked = set(git('ls-files','--others','--exclude-standard','-z').decode().split('\0'))
paths |= untracked
paths.discard('')
paths = sorted(paths)
previous_index = json.loads((reports/'P06-current-source-review-index.json').read_text())
previous_groups = {item['file']:item['group'] for item in previous_index['files']}
groups = {name:[] for name in previous_index['groups']}
groups['viewer-fixtures'] = []
upstream_path = reports/'P06-upstream-source-deltas.json'
upstream = json.loads(upstream_path.read_text()) if upstream_path.exists() else None
upstream_files = set(upstream['files']) if upstream else set()
own_files = set(git('diff', upstream['base_main'], upstream['own_source_head'], '--name-only').decode().splitlines()) if upstream else set()
groups['upstream-main'] = []
groups['creator-integration'] = []

files = {}
index = []
patch = git('diff',source_base,'--no-renames','--binary')
archive = reports/'P06-current-source-files.zip'
with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED) as zipped:
    for name in paths:
        p = worktree/name
        current = p.read_bytes() if p.is_file() else None
        files[name] = digest(current) if current is not None else None
        if current is not None:
            zipped.writestr(name, current)
        original_result = subprocess.run(['git','show',source_base+':'+name], cwd=worktree, capture_output=True)
        original = original_result.stdout.decode() if original_result.returncode == 0 else ''
        after = current.decode() if current is not None else ''
        if name in untracked:
            patch += ''.join(difflib.unified_diff([],after.splitlines(keepends=True),fromfile='/dev/null',tofile='b/'+name)).encode()
        if not name.endswith('.rs'):
            continue
        group = previous_groups.get(name)
        if name in upstream_files and name not in own_files:
            upstream_source = git('show',upstream['incoming_main']+':'+name)
            group = 'upstream-main' if current == upstream_source else 'creator-integration'

        if group is None:
            if '/ground_scatter/' in name:group = 'presentation'
            elif '/tactical_scene_viewer/' in name:group = 'viewer'
            elif '/strategic_scene/' in name or '/art_demo/' in name:group = 'strategic-art'
            else:group = 'core'
        if name == 'crates/adventuresim-tactical-client/src/tactical_scene_viewer.rs':
            marker = '#[cfg(test)]\nmod capture_lighting_tests {'
            old_start = original.index(marker)
            old_end = original.index('fn absolute_from_current(',old_start)
            old_tests = original[old_start:old_end]
            original = original[:old_start]+original[old_end:]
            new_start = after.index(marker)
            new_tests = after[new_start:]
            after = after[:new_start]
            test_diff = list(difflib.unified_diff(old_tests.splitlines(),new_tests.splitlines(),fromfile='before/'+name+'#capture_lighting_tests',tofile='after/'+name+'#capture_lighting_tests',lineterm=''))
            groups['viewer-fixtures'].extend(test_diff)
        lines = list(difflib.unified_diff(original.splitlines(),after.splitlines(),fromfile='before/'+name,tofile='after/'+name,lineterm=''))
        groups[group].extend(lines)
        index.append({'file':name,'group':group,'diff_lines':len(lines),'sha256':files[name]})
for name, lines in groups.items():
    (reports/f'P06-current-{name}-source-review.diff').write_text('\n'.join(lines)+'\n')
(reports/'P06-current-source.patch').write_bytes(patch)
with zipfile.ZipFile(archive) as zipped:
    assert all(digest(zipped.read(name)) == value for name,value in files.items() if value is not None)
recorded = datetime.now(timezone.utc).isoformat()
(reports/'P06-current-source-backup.json').write_text(json.dumps({
    'head':head,'source_comparison_base':source_base,'recorded_at':recorded,
    'status':'Source-only preservation snapshot; this helper does not mutate source/index/stash/commits or run validation.',
    'paths':paths,'files':files,'patch_sha256':digest(patch),
    'archive_sha256':digest(archive.read_bytes()),'archive_bytes':archive.stat().st_size,
    'verified_archive_file_hashes':True,
}, indent=2)+'\n')
(reports/'P06-current-source-review-index.json').write_text(json.dumps({
    'head':head,'source_comparison_base':source_base,'recorded_at':recorded,
    'scope':'Aggregate source context against the fixed pre-P06 base, including incoming main. Unchanged upstream context is grouped separately from goal edits; no whole-PR or upstream review clearance is claimed. The relocated capture_lighting_tests module has its own viewer-fixtures diff; no changed fixture text is excluded.',
    'groups':{name:{'diff_lines':len(lines)} for name,lines in groups.items()},'files':index,
}, indent=2)+'\n')
print(json.dumps({'source_paths':len(paths),'rust_paths':len(index),'archive_bytes':archive.stat().st_size,'archive_verified':True,'review_diff_lines':{name:len(lines) for name,lines in groups.items()}},indent=2))
