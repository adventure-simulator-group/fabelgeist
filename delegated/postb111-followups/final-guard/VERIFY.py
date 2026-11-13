#!/usr/bin/env python3
"""Verify the installed evidence successor and its bounded custody receipts."""
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import json
import os
import subprocess
import sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--publication', type=Path, required=True)
parser.add_argument('--predecessor', type=Path)
args = parser.parse_args()
root = args.publication.resolve()
expected_stage = root / 'delegated/postb111-followups'
assert expected_stage.is_dir() and not expected_stage.is_symlink()
assert not (root / 'delegated').is_symlink()
assert not (expected_stage / 'final-guard').is_symlink()
assert not (expected_stage / 'final-guard/VERIFY.py').is_symlink()
assert expected_stage.resolve() == p and p.is_relative_to(root)
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda path: json.loads(path.read_bytes())
env = dict(os.environ)
env.pop('PYTHONOPTIMIZE', None)
env['PYTHONNOUSERSITE'] = '1'
env['PYTHONDONTWRITEBYTECODE'] = '1'
app = [str(p / 'application/VERIFY.py'), '--publication', str(root)]
receipt_command = [str(p / 'supplemental-receipt/VERIFY.py'),
                   '--ciphertexts', str(p / 'supplemental-encrypted')]
if args.predecessor:
    app += ['--predecessor', str(args.predecessor)]
    receipt_command += ['--git-checkout', str(args.predecessor),
                        '--primary', str(p / 'encrypted-originals')]
# The preinstallation capture excludes itself and later status/closing files.
stage_capture = load(p / 'PREINSTALL-STAGE-CAPTURE.json')
assert stage_capture['schema'] == '726-postb111-preinstall-stage-capture-v1'
assert stage_capture['parent_checkpoint'] == 'b111f7444ab7e3b13b2099d5ce9d59f08faf9424'
assert stage_capture['predecessor_capture_sha256'] == 'd29e8186fa91a8a6299502ee303ec6278d6fb52779a2ba1f134d85206bdb834c'
assert stage_capture['privacy_correction_sha256'] == sha((p / 'PREINSTALL-CORRECTION.json').read_bytes())
stage_rows = {row['path']: row for row in stage_capture['rows']}
assert len(stage_rows) == len(stage_capture['rows']) and stage_rows
actual_files = {}
for item in p.rglob('*'):
    assert not item.is_symlink() and (item.is_file() or item.is_dir())
    if item.is_file():
        actual_files[item.relative_to(p).as_posix()] = item
fixed = {'PREINSTALL-STAGE-CAPTURE.json', 'ADOPTION-STATUS.json'}
closing = {'closing/CAPTURE.json', 'closing/SCAN.json', 'closing/README.md', 'closing/VERIFY.py'}
assert not (set(stage_rows) & (fixed | closing))
closing_present = set(actual_files) & closing
assert not closing_present or closing_present == closing
assert set(actual_files) == set(stage_rows) | fixed | closing_present
for name, row in stage_rows.items():
    rel = PurePosixPath(name)
    assert not rel.is_absolute() and '..' not in rel.parts
    item = actual_files[name]
    body = item.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(body)).encode() + b'\0' + body).hexdigest()
    assert (len(body), sha(body), blob) == (row['bytes'], row['sha256'], row['git_blob'])
    assert row['git_mode'] in ('100644', '100755')
    assert bool(item.stat().st_mode & 0o111) == (row['git_mode'] == '100755')
index_correction = load(p / 'INDEX-CORRECTION.json')
assert index_correction['before_sha256'] == sha((p / 'INDEX-PREVIOUS.md').read_bytes())
assert index_correction['after_sha256'] == sha((p / 'INDEX-PREVIOUS-LATE.md').read_bytes())
late_index = load(p / 'INDEX-CORRECTION-LATE.json')
assert late_index['before_sha256'] == index_correction['after_sha256']
assert late_index['after_sha256'] == sha((p / 'INDEX.md').read_bytes())
pre_status = load(p / 'ADOPTION-STATUS.json')
assert pre_status['schema'] == '726-postb111-installation-status-v1'
assert pre_status['stage_capture_sha256'] == sha((p / 'PREINSTALL-STAGE-CAPTURE.json').read_bytes())

# Pin the separate privacy-correction group before invoking its verifier.
privacy_receipt_manifest_bytes = (p / 'privacy-receipt/MANIFEST.json').read_bytes()
assert sha(privacy_receipt_manifest_bytes) == 'fdcaf315feb04c414921766716d48bbd948b74b090f46ef617e5df10c2111ca5'
privacy_receipt_manifest = json.loads(privacy_receipt_manifest_bytes)
privacy_members = privacy_receipt_manifest['files']
assert set(privacy_members) == {'PUBLIC-SCAN.json', 'RESTORATION.json', 'VERIFY-CUSTODY.py'}
assert {x.name for x in (p / 'privacy-receipt').iterdir()} == set(privacy_members) | {'MANIFEST.json'}
for name, row in privacy_members.items():
    item = p / 'privacy-receipt' / name
    assert item.is_file() and not item.is_symlink()
    body = item.read_bytes()
    assert (len(body), sha(body)) == (row['bytes'], row['sha256'])
assert (p / 'privacy-receipt/RESTORATION.json').read_bytes() == (p / 'PRIVACY-RESTORATION.json').read_bytes()
assert (p / 'privacy-receipt/PUBLIC-SCAN.json').read_bytes() == (p / 'PRIVACY-PRESCAN.json').read_bytes()
privacy_correction = load(p / 'PREINSTALL-CORRECTION.json')
assert privacy_correction['prior_capture_sha256'] == 'd29e8186fa91a8a6299502ee303ec6278d6fb52779a2ba1f134d85206bdb834c'
guard_correction = load(p / 'FINAL-GUARD-CORRECTION-v3.json')
assert guard_correction['predecessor_sha256'] == '0a4c62a4facb7a547076f6a7403520f6ffc9f44d405fbb1a826f4f20b3f714ac'
assert guard_correction['successor_sha256'] == sha((p / 'final-guard/VERIFY.py').read_bytes())
assert guard_correction['privacy_manifest_sha256'] == '7f5ad3f7bc9d2e495015c3db1908162aee97ea263e0f167a861559cfd9ee377e'
assert guard_correction['privacy_restoration_sha256'] == '4e051d691b003e01ea574517fe933af2ffe02705296ddc95fcb19f58a81c90dd'
assert sha((p / 'privacy-encrypted/MANIFEST.json').read_bytes()) == '7f5ad3f7bc9d2e495015c3db1908162aee97ea263e0f167a861559cfd9ee377e'
assert sha((p / 'PRIVACY-RESTORATION.json').read_bytes()) == '4e051d691b003e01ea574517fe933af2ffe02705296ddc95fcb19f58a81c90dd'
assert sha((p / 'privacy-encrypted/VERIFY.py').read_bytes()) == '50502589301e23919b0e1904101df4418de2fc4a2e9cfee6a3c60e62faa96de9'
assert sha((p / 'PRIVACY-PRESCAN.json').read_bytes()) == '404096ebec788cfcf68f9773597ccf68c9960f907dabf484072c2d3e2528d378'
privacy_receipt = load(p / 'PRIVACY-RESTORATION.json')
assert privacy_receipt['schema'] == '726-postb111-privacy-restoration-v1'
assert privacy_receipt['ciphertext_manifest_sha256'] == '7f5ad3f7bc9d2e495015c3db1908162aee97ea263e0f167a861559cfd9ee377e'
assert (privacy_receipt['restore_exit'], privacy_receipt['collections'],
        privacy_receipt['matched_file_bytes_and_modes'], privacy_receipt['matched_directory_modes']) == (0, 14, 151, 18)
assert privacy_receipt['selected_replay_exits'] == [0, 0, 0, 0]
assert [(x['label'], x['exit'], x['stderr_bytes']) for x in privacy_receipt['selected_replay_checks']] == [('requested-inventory-comparison', 0, 0), ('custody-control-comparison', 0, 0), ('restored-control-manifest-replay', 0, 0), ('nonce-uniqueness', 0, 0)]
privacy_scan = load(p / 'PRIVACY-PRESCAN.json')
assert privacy_scan['positive_controls_pass'] and privacy_scan['private_source_literal_hits'] == 0
assert all(value == 0 for value in privacy_scan['pattern_counts'].values())

# Bind public receipt members and the final group before executing their tools.
supp_manifest_bytes = (p / 'supplemental-receipt/MANIFEST.json').read_bytes()
assert sha(supp_manifest_bytes) == 'f61c307ec53e620a1c4851c66de2d157fcc41281a42f0c394a07b50d2757829b'
supp_manifest = json.loads(supp_manifest_bytes)
assert supp_manifest['schema'] == '726-postb111-supplemental-process-public-receipt-v1'
supp_rows = {row['path']: row for row in supp_manifest['files']}
assert len(supp_rows) == len(supp_manifest['files']) == 4
assert set(supp_rows) == {'README.md', 'REPLAYS.json', 'RESTORATION.json', 'VERIFY.py'}
assert {x.name for x in (p / 'supplemental-receipt').iterdir()} == set(supp_rows) | {'MANIFEST.json'}
for name, row in supp_rows.items():
    assert len(PurePosixPath(name).parts) == 1
    item = p / 'supplemental-receipt' / name
    assert item.is_file() and not item.is_symlink()
    body = item.read_bytes()
    assert (len(body), sha(body)) == (row['bytes'], row['sha256'])
assert sha((p / 'REMOTE-LOG-BINDING.json').read_bytes()) == '152a4f6bac841d032f8d3c0d20c04645536e8ea78ac5131ba386a70b91e5b0f7'
assert sha((p / 'structural-review-transport/MANIFEST.json').read_bytes()) == '2068ae4cb5fb7c19d10e5a996e61da2bb71921bb5f309583a2ce244f18b2b427'
assert sha((p / 'guard-review-transport/MANIFEST.json').read_bytes()) == 'a9901427b1765c43216029c7842cfbc6c9a7b3ab77d685864eac269022b4c3b2'
assert sha((p / 'final-encrypted/MANIFEST.json').read_bytes()) == '582d3a09a922a155ff35091c69b9ec53a9dda211e1facc9bc4a78a4d7a7875f2'
assert sha((p / 'FINAL-RESTORATION.json').read_bytes()) == '2c72247abc525ad0d4d3016ccaf8033ebfe1303ae80e73abf7febcfdb45d3ef5'
assert sha((p / 'final-encrypted/VERIFY.py').read_bytes()) == '50502589301e23919b0e1904101df4418de2fc4a2e9cfee6a3c60e62faa96de9'
assert sha((p / 'FINAL-PRESCAN.json').read_bytes()) == '3758e3c2a56bdfe1bb9183ce47921f865fe882cd5ae5c5e4b9d4a90119664804'
final_prescan = load(p / 'FINAL-PRESCAN.json')
assert final_prescan['positive_controls_pass'] and final_prescan['private_source_literal_hits'] == 0
assert all(value == 0 for value in final_prescan['pattern_counts'].values())
for command in [[str(p / 'VERIFY.py'), '--publication', str(root)], app,
                [str(p / 'structural-review-transport/VERIFY.py')],
                [str(p / 'guard-review-transport/VERIFY.py')],
                [str(p / 'supplemental-encrypted/VERIFY.py')], receipt_command,
                [str(p / 'final-encrypted/VERIFY.py')],
                [str(p / 'privacy-encrypted/VERIFY.py')],
                [str(p / 'privacy-receipt/VERIFY-CUSTODY.py'), '--ciphertexts', str(p / 'privacy-encrypted')]]:
    subprocess.run([sys.executable, '-B', *command], cwd=root, env=env, check=True)

primary_bytes = (p / 'PRIMARY-RESTORATION.json').read_bytes()
assert sha(primary_bytes) == '4975f54dc05a89d75d3dd36db6a649b24c3d65c74aec81619eed4df7d6fb36ef'
primary = json.loads(primary_bytes)
assert primary['separate_earlier_delegate_restoration']['ciphertext_sha256'] == (
    '807c05dfbcf54a25ffd87130459de0a919427b8aed99bd80472f6c6048b8c987')
recovery_bytes = (p / 'REMOTE-RECOVERY.json').read_bytes()
assert sha(recovery_bytes) == 'bcef93fe7b3087b99fdbe91122a7d2c459d6162f90371f904c2bb71fe0df15ba'
recovery = json.loads(recovery_bytes)
assert (recovery['commit'], recovery['tree'], recovery['prior_commit'],
        recovery['captured_paths_matched']) == (
    'b111f7444ab7e3b13b2099d5ce9d59f08faf9424',
    '84668f9ace2a06e346564f2185d5e57747403363',
    '5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0', 141)
names = {'predecessor-0', 'predecessor-1', 'predecessor-2', 'fetch', 'checkout',
         'successor-final-guard', 'successor-checkpoint', 'successor-closing'}
assert len(recovery['checks']) == len(names)
assert {item['name'] for item in recovery['checks']} == names
assert all(item['exit'] == 0 for item in recovery['checks'])
binding = load(p / 'REMOTE-LOG-BINDING.json')
assert binding['schema'] == '726-postb111-remote-log-binding-v1'
assert binding['receipt_sha256'] == sha(recovery_bytes)
assert binding['primary_manifest_sha256'] == sha((p / 'encrypted-originals/MANIFEST.json').read_bytes())
assert binding['primary_restoration_sha256'] == sha(primary_bytes)
assert binding['ciphertext_label'] == 'process-selection'
assert binding['ciphertext_sha256'] == '04e8b42bf650bf5554c0e0c12f03da7f3d6db10f6517fe96fac475cc4fb19224'
assert binding['matched_log_files'] == 16 and set(binding['check_names']) == names

supplement_manifest = (p / 'supplemental-encrypted/MANIFEST.json').read_bytes()
assert sha(supplement_manifest) == '4474472c6cb737e104aeccd414f2fe0fc9ebf0cec0f80b85a73f463283197883'
assert sha((p / 'supplemental-receipt/MANIFEST.json').read_bytes()) == 'f61c307ec53e620a1c4851c66de2d157fcc41281a42f0c394a07b50d2757829b'
supplement = (p / 'SUPPLEMENTAL-RESTORATION.json').read_bytes()
assert supplement == (p / 'supplemental-receipt/RESTORATION.json').read_bytes()
assert sha(supplement) == 'fb2506142b79a96bd9a8645a875277524490f1ec1a79557b8de69d65370aa6e0'
supplement = json.loads(supplement)
assert (supplement['collections'], supplement['files'],
        supplement['directories_including_collection_roots']) == (48, 1289, 103)
assert supplement['all_captured_files_and_modes_match'] and supplement['all_captured_directories_and_root_modes_match']
assert supplement['ciphertext_manifest_sha256'] == sha(supplement_manifest)
replays = load(p / 'supplemental-receipt/REPLAYS.json')
assert len(replays['checks']) == 9 and all(x['exit'] == 0 and x['stderr_bytes'] == 0 for x in replays['checks'])
assert sha((p / 'SUPPLEMENTAL-PRESCAN.json').read_bytes()) == 'fe9326446372d22e832ac72f3009b6df72644b044800371c6db8515491a720e5'

reviews = load(p / 'structural-review-transport/MANIFEST.json')
review_ids = {r['label']: r['source_manifest_sha256'] for r in reviews['bundles']}
for label, digest in {
    'transform-v1': '505e7fe18747f284ec49ecd98947cdaac086c2de270bba5681e856504bf9c3cf',
    'transform-successor': 'c1480788bf00a7bcf2583ba504696d7ceab7130a7f8875eda1167be5a7cfdda8',
    'application-review': '5b01ab727bdd752ed66c5c9caa84629f3b3325fef38d2b898f59c88b17df1cbd',
    'application-receipt': '44f9cde49ac2f778e60271441de14032724f7a05bde58df60ac670247122101b',
}.items():
    assert review_ids[label] == digest
readme = (p / 'application/README.md').read_bytes()
correction = load(p / 'application/README-CORRECTION.json')
assert correction['before_sha256'] == 'e201be7f0c8fb7880d3bf60200f3396103de2c3ce0edfd69ebb6b73e3d6e4cfd'
assert correction['after_sha256'] == sha(readme) == '8e00ea8a23cd48623c8a5ed3d32219afd438a52e2c379e3f772f3bff64a5387a'
assert sha((p / 'CHECKPOINT-QUALIFICATIONS.md').read_bytes()) == 'a992f5605fe5f5a308a350a47d8c152ac3d65f45b5e1a46cb3fffc03d79efda3'

final_manifest = load(p / 'final-encrypted/MANIFEST.json')
final_receipt = load(p / 'FINAL-RESTORATION.json')
assert final_receipt['schema'] == '726-postb111-final-restoration-v1'
assert final_receipt['ciphertext_manifest_sha256'] == sha((p / 'final-encrypted/MANIFEST.json').read_bytes())
assert final_receipt['restore_exit'] == 0
assert final_receipt['collections'] == len(final_manifest['objects']) > 0
assert final_receipt['matched_file_bytes_and_modes'] > 0
assert final_receipt['matched_directory_modes'] >= 0
assert final_receipt['selected_replay_exits'] == [0, 0, 0, 0, 0]
assert [(x['label'], x['exit'], x['stderr_bytes']) for x in final_receipt['selected_replay_checks']] == [('application-review', 0, 0), ('structural-review', 0, 0), ('structural-public', 0, 0), ('guard-v1-review', 0, 0), ('guard-v1-public', 0, 0)]

capture_bytes = (p / 'ROOT-CANDIDATE-CAPTURE.json').read_bytes()
assert sha(capture_bytes) == '0664f3064ee85dd24101cb8468f9382b994c595962ab0637d6576ee28923cbfd'
capture = json.loads(capture_bytes)
scan = load(p / 'ROOT-CANDIDATE-SCAN.json')
expected = {'BASELINE-95E-20261110.json', 'CONTINUATION-PROGRESS-95E.json',
            'RECONCILIATION-059.json', 'CONTINUATION.md',
            'ASSESSMENT-CENTRAL-FOLLOWUPS-21-95E.json',
            'MAIN-RECHECK-20261113-0655.json', 'verify_postb111_checkpoint.py'}
assert len(capture['rows']) == len(expected) == scan['scanned_paths']
assert {row['path'] for row in capture['rows']} == expected
assert scan['capture_sha256'] == sha(capture_bytes)
assert scan['forbidden_digest_total'] == scan['exceptions'] == scan['tar_pattern_issues'] == 0
assert not scan['other_candidate_counts'] and all(x['exit'] == 0 for x in scan['checks'])
for row in capture['rows']:
    name = PurePosixPath(row['path'])
    assert len(name.parts) == 1
    file = root / name.name
    assert file.is_file() and not file.is_symlink()
    data = file.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    assert (len(data), sha(data), blob, row['mode']) == (row['bytes'], row['sha256'], row['git_blob'], '100644')
    assert not (file.stat().st_mode & 0o111)
status = load(p / 'ADOPTION-STATUS.json')
assert status['schema'] == '726-postb111-installation-status-v1'
assert status['status_at_recording'] == 'installed in the local evidence checkout; publication is separate'
assert status['parent_checkpoint'] == 'b111f7444ab7e3b13b2099d5ce9d59f08faf9424'
assert status['parent_tree'] == '84668f9ace2a06e346564f2185d5e57747403363'
assert status['root_candidate_capture_sha256'] == sha(capture_bytes)
assert status['candidate_record_sha256'] == sha((p / 'application/ADOPTION-CANDIDATE.json').read_bytes())
assert status['stage_capture_sha256'] == sha((p / 'PREINSTALL-STAGE-CAPTURE.json').read_bytes())
assert status['privacy_correction_sha256'] == sha((p / 'PREINSTALL-CORRECTION.json').read_bytes())
assert status['root_paths_matched'] == 7
print('Verified installed evidence images, exact recovery receipt, bounded scans '
      'and same-machine custody receipts. Decryption, independent custody, '
      'historical completeness, semantic closure and product behavior remain '
      'separately qualified.')
