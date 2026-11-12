#!/usr/bin/env python3
"""Verify attributed followup evidence, archives and qualified connections."""
import argparse, hashlib, json, subprocess, sys, tarfile, tempfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
h = lambda b: hashlib.sha256(b).hexdigest()
def load(path):
    return json.loads(path.read_bytes())
def packet_manifest(packet, expected):
    raw = (packet/'MANIFEST.json').read_bytes()
    assert h(raw) == expected
    rows = json.loads(raw)['files']
    if isinstance(rows,dict):
        rows = [{'path':k,**v} for k,v in rows.items()]
    seen = set()
    for row in rows:
        p = PurePosixPath(row['path'])
        assert not p.is_absolute() and '..' not in p.parts and p.as_posix() not in seen
        seen.add(p.as_posix())
        f = packet/p
        assert f.is_file() and not f.is_symlink()
        b = f.read_bytes()
        assert (len(b),h(b)) == (row['bytes'],row['sha256'])
    assert {p.relative_to(packet).as_posix() for p in packet.rglob('*') if p.is_file()} == seen | {'MANIFEST.json'}
    return seen
sets = [
 ('independent-capture-clock-q2300','7c1fc9fcb2638bfb4b374cc6bfb7b1d6e4a5ba83219d2f68894e9988a1acfcf6','250c85cb742b9d7812b215e183803e55cb080ed12e3d77229a6833eda7a9f25e','bundle'),
 ('capture-readiness-q2303','bddc8ad1907350215db9ca4057a4c117d5bb0009db4a723c8a68f7d70901de33','9f456dce7756e321865baac3fca9d688559f1d11a5bf4397f49b7c74c540fa2b','q2303'),
 ('npy-q2306','4e1b8e46c37b0a1f496c818dadd60ded51faf59b212ca73132545f3a6b3bbc7c','cae31741c9eda230ccf994cd3e02d72be5810aff5fe07e9ddfb7ccadae61aa1c','bundle'),
 ('npz-q2307','4495b62f3f35765d2e75b0a09401f778e81ece90c6a45f47c8a345e7ef8454ce','b01c8b9035cf5af00322e9dd364b184f0e99456c7e9b4ecc06d7420823b9ff32','bundle'),
 ('central-blood-q2297','4e860fbe18d34632f2096ba9e0812d8e904aac21b462455f02f0b7a6c6c9bd2c','eb94b5c90ef95f19f8750edcdaf9bd3fd07957f2e5594f3cffd0d54253bc1068','bundle'),
 ('independent-blood-q2297','52bd61b391ef1e3690b19a977f9a36db9529f9b0229eb98289f28ef7763cf56e','04dc189fc797fc94cb4ed0010ef78f0138f9ce637e1576deaa57f9a6b79bf00d','q2297'),
 ('live-state-successor-design','6f924a339162eaaa733e794ab93704800b2837b5c2fbcd4635607594d1875dd0','0ce414dffbcddc9e0d2a183c6c73290de1e6474a792530d8e5a93ba430607575','bundle')]
for name, mh, ah, prefix in sets:
    packet = root/'delegated'/name
    seen = packet_manifest(packet,mh)
    archive = root/'delegated'/(name+'-archive.tar.gz')
    assert h(archive.read_bytes()) == ah
    with tarfile.open(archive,'r:gz') as t:
        members = t.getmembers()
        assert len(members) == len(seen)+1
        archived = set()
        for member in members:
            p = PurePosixPath(member.name)
            assert member.isfile() and not p.is_absolute() and p.parts[0] == prefix and '..' not in p.parts
            rel = PurePosixPath(*p.parts[1:])
            assert rel.as_posix() not in archived
            archived.add(rel.as_posix())
            assert t.extractfile(member).read() == (packet/rel).read_bytes()
        assert archived == seen | {'MANIFEST.json'}
packet_manifest(root/'delegated/coordinator-post26e-review','f6fc4588c751fa48d7955d9844e0111d109e975cca862b65bc80df1d6d188249')
packet_manifest(root/'delegated/post26e-ledger-selection','9dbfba7117ea4bcacedaf6c55ffb7c02c6a2f18abbc4ac6d94a089c88bd5245d')
packet_manifest(root/'delegated/live-state-successor-design-coordinator-review','341c521e10254cbb00855afa0ee8eb2c80392868964f9561e2aa3e1878344f79')
packet_manifest(root/'delegated/post26e-agent-scans/clock','ccffaf46832f06039aba31db58ad9fe48800d50a22945cdfca59351d85d8b241')
packet_manifest(root/'delegated/live-state-successor-design-agent-scan','3290c81e6e9a1e00078c3b9a48839a2b12140233243d06e142c7f215d8961dbc')
scan = root/'delegated/post26e-coordinator-scans'
summary = (scan/'SUMMARY.json').read_bytes()
assert h(summary) == '36a5d1702c0295e268efeb7627b9f38e0f48e1262f7f6f576eb79825acb41b07'
archive_hashes = {'clock':sets[0][2],'readiness':sets[1][2],'npy':sets[2][2],'npz':sets[3][2],'blood-coordinator':sets[4][2],'blood-independent':sets[5][2]}
for row in json.loads(summary)['scans']:
    b = (scan/(row['tag']+'-scan.json')).read_bytes()
    x = json.loads(b)
    assert row['archive_sha256'] == archive_hashes[row['tag']] and row['scan_sha256'] == h(b)
    assert x['forbidden_digest_total'] == 0 and not x['other_candidate_counts'] and not x['exceptions']
assert set(archive_hashes) == {x['tag'] for x in json.loads(summary)['scans']}
for tag, target in [('clock','independent-capture-clock-q2300'),('npy','npy-q2306'),('npz','npz-q2307')]:
    d = root/'delegated/post26e-agent-scans'/tag
    rec = load(d/'RECEIPT.json')
    if tag == 'clock':
        digest = (d/'DIGEST-RESULT.json').read_bytes()
        pattern = (d/'PATTERN-RESULT.json').read_bytes()
        assert rec['stage_manifest_sha256'] == h((root/'delegated'/target/'MANIFEST.json').read_bytes())
        assert rec['recursive_digest_result_sha256'] == h(digest) and rec['pattern_result_sha256'] == h(pattern)
        px = json.loads(pattern)
        assert not px['candidate_counts'] and not px['exceptions']
    else:
        digest = (d/'digest.json').read_bytes()
        pattern = (d/'pattern.json').read_bytes()
        assert rec['packet_manifest_sha256'] == h((root/'delegated'/target/'MANIFEST.json').read_bytes())
        assert rec['digest_result_sha256'] == h(digest) and rec['pattern_result_sha256'] == h(pattern)
        assert not any(json.loads(pattern)['counts'].values())
    x = json.loads(digest)
    assert x['forbidden_digest_total'] == 0 and not x['other_candidate_counts'] and not x['exceptions']
assert h((root/'delegated/post26e-agent-scans/readiness-receipt.json').read_bytes()) == '1bd0530c117f0035cb496c6b64bfa7dfbba9c926228e445fd6f760ae92d3fc89'
assert h((root/'delegated/post26e-agent-scans/blood-independent-receipt.json').read_bytes()) == '94c0f9f4ccf4091cfcf3c29550612adebee5c96a45222199816917621a6edda3'
design_scan = root/'delegated/live-state-successor-design-coordinator-scan'
raw = (design_scan/'SUMMARY.json').read_bytes()
assert h(raw) == '43ad00b16f35677ad834350bbb27624ca4aa01b4bf7562c1ab712f0c0473f658'
for row in json.loads(raw)['scans']:
    b = (design_scan/(row['tag']+'-scan.json')).read_bytes()
    x = json.loads(b)
    assert row['sha256'] == h(b) and x['forbidden_digest_total'] == 0 and not x['other_candidate_counts'] and not x['exceptions']
replay = root/'delegated/post26e-evidence-replays'
raw = (replay/'RECORD.json').read_bytes()
assert h(raw) == '47f77232a7cac4a98b45742fbc29b4d3480eb092992c27bcc8f5b8174dbe61ce'
for row in json.loads(raw)['cases']:
    b = (replay/row['transcript']).read_bytes()
    assert row['exit'] == 0 and (len(b),h(b)) == (row['bytes'],row['sha256'])
reports = root/'delegated/coordinator-post26e-review/review'
for tag,path in [('clock-review','independent-capture-clock-q2300/REVIEW.json'),('readiness-report','capture-readiness-q2303/REPORT.json'),('npy-report','npy-q2306/REPORT.json')]:
    assert (reports/('DISPLAY-'+tag+'.json')).read_bytes() == (root/'delegated'/path).read_bytes()
assert (reports/'NPZ-REPORT-DISPLAY.json').read_bytes() == (root/'delegated/npz-q2307/REPORT.json').read_bytes()
assert (reports/'BLOOD-REVIEW-DISPLAY.json').read_bytes() == (root/'delegated/independent-blood-q2297/REPORT.json').read_bytes()
assert (root/'delegated/live-state-successor-design-coordinator-review/REPORT-DISPLAY.md').read_bytes() == (root/'delegated/live-state-successor-design/REPORT-v2.md').read_bytes()
baseline = load(root/'BASELINE-95E-20261110.json')
progress = load(root/'CONTINUATION-PROGRESS-95E.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert len(ledger) == progress['new_current_ledger_candidates'] == 1033
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
matches = [x for x in ledger if x.get('id') == 'candidate-blood-window-final-minute-95e']
assert len(matches) == 1 and matches[0]['record'] == 'ASSESSMENT-CENTRAL-BLOOD-WINDOW-13V-95E.json'
queue = load(root/'RECONCILIATION-059.json')['entries']
for i in (2297,2303,2306,2307):
    assert queue[i]['current_reconciliation'].startswith('partial')
assert load(root/'LIVE-STATE-SUCCESSOR-DESIGN-QUALIFICATION.json')['status'].startswith('Unimplemented')
parser = argparse.ArgumentParser()
parser.add_argument('--git-repo',type=Path)
parser.add_argument('--private-queue',type=Path)
args = parser.parse_args()
commands = [
 ['delegated/independent-capture-clock-q2300/verify-evidence.py','--publication',str(root)],
 ['delegated/capture-readiness-q2303/VERIFY-EVIDENCE.py'],
 ['delegated/npy-q2306/verify-evidence.py','--publication-root',str(root)],
 ['delegated/npz-q2307/verify-evidence.py','--publication-root',str(root)],
 ['delegated/central-blood-q2297/verify-evidence.py'],
 ['delegated/independent-blood-q2297/VERIFY-EVIDENCE.py'],
 ['delegated/live-state-successor-design/verify-packet.py']]
if args.git_repo:
    commands[0] += ['--git-repo',str(args.git_repo.resolve())]
    commands[1] += ['--repo',str(args.git_repo.resolve())]
    commands[4] += ['--publication',str(args.git_repo.resolve())]
    commands[5] += ['--repo',str(args.git_repo.resolve())]
    commands[6] += ['--git-repo',str(args.git_repo.resolve())]
    git = lambda *x: subprocess.check_output(['git','-C',str(args.git_repo.resolve()),*x])
    old_baseline = git('show','26e360920b7b2ec32d5c76238623eaf6d6617d7e:BASELINE-95E-20261110.json')
    old_queue = git('show','26e360920b7b2ec32d5c76238623eaf6d6617d7e:RECONCILIATION-059.json')
    records = root/'delegated/post26e-ledger-selection'
    assert h(old_baseline) == load(records/'CENTRAL-LEDGER-REGISTRATION.json')['source_sha256']
    assert h(old_queue) == load(records/'QUEUE-REGISTRATION.json')['source_sha256']
    for row in load(records/'CENTRAL-LEDGER-SELECTION-RECONSTRUCTED.json')['entries']:
        assert json.loads(old_baseline)['newly_assessed_ledger']['entries'][row['index']] == row['value']
    for row in load(records/'QUEUE-SELECTION-RECONSTRUCTED.json')['entries']:
        assert json.loads(old_queue)['entries'][row['index']] == row['value']
    print('Optional 26e Git baseline and queue selection comparisons passed.')
else:
    print('Earlier 26e Git baseline and queue comparisons require --git-repo.')
if args.private_queue:
    commands[2] += ['--original-queue',str(args.private_queue.resolve())]
    commands[3] += ['--original-queue',str(args.private_queue.resolve())]
for command in commands:
    subprocess.run([sys.executable,'-B',str(root/command[0]),*command[1:]],cwd=root,check=True)
print('Verified exact followup archives, registered evidence, bounded scans and working connections; source behavior and completion remain separately qualified.')
