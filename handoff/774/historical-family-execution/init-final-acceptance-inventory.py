"""Pin source and preserved overlays; create incomplete audit bookkeeping only."""
from pathlib import Path
from datetime import datetime, timezone
from collections import Counter
import hashlib
import json
import subprocess
import zipfile

reports = Path(__file__).resolve().parent
output = reports / 'final-acceptance-20261021'
output.mkdir(exist_ok=True)
if (output / 'audit-ledger.json').exists():
    raise SystemExit('Existing inventory must be resumed, not overwritten')
previous = json.loads((reports / 'execution-ledger.json').read_text())
main = '3a50b708381ad00c116445c2c999a49c3943088b'
now = datetime.now(timezone.utc).isoformat()

def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args])

def owner(path):
    if any(path.startswith('crates/' + n + '/') for n in ['adventuresim-character-creator', 'fabelgeist-mhr', 'fabelgeist-rig', 'fabelgeist-armor', 'fabelgeist-fbx']):
        return 'rig'
    if path.startswith('crates/adventuresim-core/src/item_catalog') or path.startswith('crates/adventuresim-core/src/equipment_assets') or path.startswith('crates/adventuresim-tactical-client/src/equipment') or path.endswith('/armor_browser_probe.rs'):
        return 'rig'
    if path.startswith('crates/adventuresim-core/') or path.startswith('crates/adventuresim-stdb-module/') or path.startswith('crates/adventuresim-strategic-sim/'):
        return 'camera'
    if path.startswith('crates/adventuresim-tactical-client/src/animation'):
        return 'native'
    if path.startswith('crates/adventuresim-tactical-core/src/animation') or path.startswith('crates/adventuresim-tactical-core/src/audio'):
        return 'native'
    if path.startswith('crates/adventuresim-tactical-server/src/iteration') or '/melee_combat_iteration/' in path or path.endswith('/melee-combat-iteration.rs'):
        return 'native'
    if path.startswith('crates/adventuresim-tactical-server/src/combat') or path.startswith('crates/adventuresim-tactical-server/src/player_projection') or path.startswith('crates/adventuresim-tactical-server/src/equipment') or path.startswith('crates/adventuresim-tactical-server/src/physics'):
        return 'camera'
    if path.startswith('crates/adventuresim-tactical-client/src/camera') or path.startswith('crates/adventuresim-tactical-client/src/player') or path.startswith('crates/adventuresim-tactical-client/src/physics'):
        return 'camera'
    if path.startswith('crates/adventuresim-tactical-core/src/combat') or path.startswith('crates/adventuresim-tactical-core/src/inventory') or path.startswith('crates/adventuresim-tactical-netcode/'):
        return 'camera'
    return 'root'

snapshots = [
    ('pr-working', Path(previous['integration_worktree']), main, 'PR HEAD plus all existing staged/unstaged/untracked changes; primary source inventory'),
    ('rig-quality', Path(previous['owners']['/root/rig_garments']['worktree']), '6533e444fa37552927070cc8d8190907fd3b413e', 'Preserved complete cbbb quality delta, not yet integrated'),
    ('report-p17', Path(previous['owners']['/root/native_reports']['worktree']), '18360adba0fb43add88c75dfbd2f45d7e71d8ace', 'Mandatory P17 source delta90a; shared motor dependency still pending'),
    ('physical-p16', Path(previous['owners']['/root/camera_targeting']['worktree']), 'a120390947c89b703dc74c4255a13c035d054c50', 'Mandatory unfinished P16 dirty delta, frozen before further fixes'),
]
records = []
contexts = []
with zipfile.ZipFile(output / 'frozen-inputs.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
    for name, root, base, meaning in snapshots:
        head = git(root, 'rev-parse', 'HEAD').decode().strip()
        names = set(git(root, 'diff', '--name-only', '--no-renames', '-z', base).decode().split('\0'))
        names.update(git(root, 'ls-files', '--others', '--exclude-standard', '-z').decode().split('\0'))
        names.discard('')
        patch = git(root, 'diff', '--binary', '--no-renames', base)
        archive.writestr(name + '/source.patch', patch)
        current = []
        for path in sorted(names):
            location = root / path
            data = location.read_bytes() if location.is_file() else None
            before = subprocess.run(['git', '-C', str(root), 'show', base + ':' + path], capture_output=True)
            baseline = before.stdout if before.returncode == 0 else None
            row = dict(source_id=name + ':' + path, snapshot=name, path=path,
                       owner=owner(path), sha256=hashlib.sha256(data).hexdigest() if data is not None else None,
                       base_sha256=hashlib.sha256(baseline).hexdigest() if baseline is not None else None,
                       state='deleted' if data is None else 'new' if baseline is None else 'modified',
                       bytes=len(data) if data is not None else 0)
            records.append(row)
            current.append(row)
            if data is not None:
                archive.writestr(name + '/files/' + path, data)
        contexts.append(dict(id=name, root=str(root), base=base, head=head, meaning=meaning,
                             changed_variants=len(current), patch_sha256=hashlib.sha256(patch).hexdigest(),
                             status_porcelain=git(root, 'status', '--porcelain').decode().splitlines()))
    source_plan = reports.parent / 'pr774-canary-audit-20261020/migration-plan.json'
    archive.writestr('mandatory-twenty-family-plan.json', source_plan.read_bytes())

lenses = ['audit-producer-propagation', 'audit-existing-type-owners', 'audit-admission-contracts',
          'audit-invariant-operations', 'audit-aggregate-states', 'audit-type-ownership',
          'audit-redundant-authority', 'audit-behavior-preservation', 'audit-enum-shapes',
          'audit-derivability', 'audit-type-granularity', 'audit-magic-strings', 'audit-magic-numbers']
manifest = dict(created_at=now, base=main, snapshots=contexts, files=records,
                pending_work_is_mandatory=True,
                overlap_policy='Separate pinned deltas, never wholesale restore old branches or assume conflicting variants already integrate. Trace and reconcile overlapping functions before repair.',
                original_stash=dict(commit='6c109560050d97dd6deeb842b1d71ec8a280949b', status='Preserved, never restored wholesale; original recovery handoffs are provenance, not source clearance.'),
                archive_sha256=hashlib.sha256((output / 'frozen-inputs.zip').read_bytes()).hexdigest())
(output / 'source-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
for name in ['root', 'camera', 'rig', 'native']:
    assigned = [r for r in records if r['owner'] == name]
    (output / ('assigned-' + name + '.json')).write_text(json.dumps(assigned, indent=2) + '\n')
ledger = dict(version=1, authority='Sole audit/review coverage ledger; family status remains ../execution-ledger.json. Agent shards contain evidence, not independent clearance.',
              created_at=now, updated_at=now, source_manifest='source-manifest.json',
              source_manifest_sha256=hashlib.sha256((output / 'source-manifest.json').read_bytes()).hexdigest(),
              mandatory_families=['P%02d' % n for n in range(1, 21)],
              acceptance=dict(all_twenty_finished_and_integrated=True, all_thirteen_lenses_same_final_revision=True,
                              code_rule_comment_reviews_same_final_revision=True, required_hosted_ci_same_final_revision=True,
                              new_tests_allowed=False, merge_allowed=False, complete_overall_goal_allowed=False,
                              silent_deferrals_allowed=False, prior_verdicts_are_historical_only=True),
              discovery_gate=dict(status='incomplete', repairs_allowed=False,
                                  requirement='Exhaustive discovery of every selected lens across pinned PR and preserved P16/P17/quality work plus affected unchanged paths before any further source repair.'),
              current_wave=1, current_lens=lenses[0], first_unfinished_cell='wave-01/audit-producer-propagation/root',
              lenses=[dict(lens=n, skill='/home/brunosegovia/.codex/skills/' + n + '/SKILL.md',
                           skill_sha256=hashlib.sha256((Path('/home/brunosegovia/.codex/skills') / n / 'SKILL.md').read_bytes()).hexdigest(),
                           discovery='incomplete', repair='not_started_after_new_inventory_gate', validation='not_established',
                           scoped_variants=len(records), cells=[dict(source_id=x['source_id'], owner=x['owner'], discovery='untraced') for x in records],
                           affected_unchanged_paths=[], findings=[], evidence_shards=[]) for n in lenses],
              deduplicated_findings=[], shared_repairs=[], reviews={n:dict(status='not_run_on_final_revision') for n in ['Code Review', 'Rule Review', 'Comment Review']},
              hosted_ci=dict(status='no_active_request', latest_completed_run=39576450512, head='6533e444fa37552927070cc8d8190907fd3b413e', conclusion='failure', evidence='../hosted-6533e44-semantic-quality.log'),
              source_edit_freeze=dict(status='active_for_initial_inventory', root_existing_dirty_paths=4, interrupted_agent='/root/camera_targeting', preserve_all_existing_work=True))
(output / 'audit-ledger.json').write_text(json.dumps(ledger, indent=2) + '\n')
wave = output / 'wave-01' / lenses[0]
wave.mkdir(parents=True, exist_ok=True)
print(json.dumps(dict(output=str(output), snapshots={c['id']:c['changed_variants'] for c in contexts},
                      variants=len(records), distinct_paths=len({r['path'] for r in records}), ownership=dict(Counter(r['owner'] for r in records)),
                      all_thirteen_discovery='incomplete', source_repairs_performed=0), indent=2))
