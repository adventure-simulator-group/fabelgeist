#!/usr/bin/env python3
"""Read-only integrity and scope reconciliation for the review packet."""
import argparse,hashlib,json,pathlib
p=argparse.ArgumentParser();p.add_argument('--stage',type=pathlib.Path);p.add_argument('--old-stage',type=pathlib.Path);a=p.parse_args();R=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def data(n):p=R/n;assert p.is_file() and not p.is_symlink();return p.read_bytes()
m=json.loads(data('MANIFEST.json'));rows={x['path']:x for x in m['files']};assert len(rows)==len(m['files']);actual={x.relative_to(R).as_posix() for x in R.rglob('*') if x.is_file() or x.is_symlink()}-{'MANIFEST.json'};assert actual==set(rows)
for n,x in rows.items():b=data(n);assert (H(b),len(b))==(x['sha256'],x['bytes'])
r=json.loads(data('REPORT.json'));proof=json.loads(data('PROOF.json'));scan=json.loads(data('SCAN.json'));display=json.loads(data('DISPLAY.json'));pattern=json.loads(data('PATTERN-SCAN.json'));classification=json.loads(data('PATTERN-CLASSIFICATION.json'))
assert r['reviewed_stage_manifest_sha256']==proof['successor_stage_manifest_sha256']==scan['stage_manifest_sha256']==display['stage_manifest_sha256']==classification['stage_manifest_sha256']
assert r['predecessor_stage_manifest_sha256']==proof['old_stage_manifest_sha256'];assert proof['archive_count']==8 and len(proof['seven_unchanged_archives'])==7 and proof['restricted_gpu_source_files_replayed']==154
assert proof['gpu_map_omitted_counts']['original_value_sha256_values']==47 and proof['gpu_map_omitted_counts']['public_value_sha256_values']==2
assert sum(proof['scan_prefix_field_counts'].values())==60 and not scan['findings'] and not scan['exceptions'] and scan['recovered_private_value_count']==112 and scan['candidate_digest_encodings']==681
assert display['source_citations_within_recorded_ranges'] and len(proof['13l_project_git_membership_paths'])==5 and set(proof['13l_changes_since_review'])=={'/historical_attribution/1','/review_qualification'}
assert pattern['candidate_count']==5 and pattern['unsupported_count']==0 and classification['source_code_regex_literal_occurrences']==3 and classification['tar_stream_duplicate_occurrences']==2 and classification['disclosed_commit_objects']['copies']==15
if a.stage is not None:assert H((a.stage/'STAGE-MANIFEST.json').read_bytes())==r['reviewed_stage_manifest_sha256']
if a.old_stage is not None:assert H((a.old_stage/'STAGE-MANIFEST.json').read_bytes())==r['predecessor_stage_manifest_sha256']
print(json.dumps({'packet_files_verified':len(rows),'successor_manifest_bound':a.stage is not None,'predecessor_manifest_bound':a.old_stage is not None,'qualification':'Internal evidence integrity and stated scope only; scans and restricted replay are recorded results, not rerun by this verifier. No product validation.'}))
