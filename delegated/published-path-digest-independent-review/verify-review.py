#!/usr/bin/env python3
"""Read-only evidence packet check; does not run product or private code."""
import argparse,collections,hashlib,json,pathlib,subprocess
p=argparse.ArgumentParser();p.add_argument('--publication',type=pathlib.Path);p.add_argument('--stage40',type=pathlib.Path);p.add_argument('--stage42',type=pathlib.Path);p.add_argument('--private40',type=pathlib.Path);p.add_argument('--private42',type=pathlib.Path);a=p.parse_args();R=pathlib.Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest()
def data(name):
 f=R/name;assert f.is_file() and not f.is_symlink();return f.read_bytes()
m=json.loads(data('MANIFEST.json'));files={x['path']:x for x in m['files']};assert len(files)==len(m['files'])
actual={x.relative_to(R).as_posix() for x in R.rglob('*') if x.is_file() or x.is_symlink()}-{'MANIFEST.json'};assert actual==set(files)
for name,row in files.items():
 b=data(name);assert (len(b),h(b))==(row['bytes'],row['sha256'])
j=json.loads(data('REPORT.json'));q=json.loads(data('evidence/RECURSIVE-INVENTORY.json'));x=json.loads(data('evidence/EXCEPTION-RESCAN.json'));t=json.loads(data('evidence/OLDER-TEXT-MANIFEST.json'));e=json.loads(data('evidence/EXCEPTION-MANIFEST.json'));o=json.loads(data('evidence/OVERLAP-TEXT.json'))
assert x['prior_scan_report_sha256']==h(data('evidence/RECURSIVE-INVENTORY.json'))
assert j['input_identities']==q['inputs'] and j['older_committed_scope']['revision']==t['committed_revision']==e['committed_revision']==x['committed_revision']
assert (q['matches']['stage_field_occurrences'],q['matches']['stage_identifier_encodings'],q['matches']['shared_identifier_encodings'])==(94,29,24)
assert len(q['matches']['stage_locations'])==94 and len(q['matches']['older_locations'])==346
assert (t['file_count'],t['occurrences'],len(t['files']))==(15,522,15)
assert sum(len(r['other_committed_paths'])+1 for r in q['matches']['older_locations'])==522
assert all('!' not in r['location'] and '[gzip decoded]' not in r['location'] for r in q['matches']['older_locations'])
assert (x['prior_exception_count'],x['named_branches_reached'],x['direct_signature_blob_count_confirmed'],x['new_matches'])==(154,93,61,0) and not x['named_branches_missing']
assert len(x['new_exceptions'])==e['unread_member_occurrences']==4188 and len(e['files'])==e['file_count']==46
assert sum(r['unread_nested_member_occurrences'] for r in e['files'])==4188
assert len(o['stage_locations'])==94 and sum(r['same_identifier_in_older_text'] for r in o['stage_locations'])==85
for option,expected,filename in [(a.stage40,j['input_identities']['followup_40_41_stage_manifest_sha256'],'STAGE-MANIFEST.json'),(a.stage42,j['input_identities']['followup_42_stage_manifest_sha256'],'STAGE-MANIFEST.json'),(a.private40,j['input_identities']['followup_40_41_private_retention_manifest_sha256'],'MANIFEST.json'),(a.private42,j['input_identities']['followup_42_private_retention_manifest_sha256'],'MANIFEST.json')]:
 if option is not None:assert h((option/filename).read_bytes())==expected
verified=0
if a.publication is not None:
 for row in t['files']+e['files']:
  revision=t['committed_revision'];path=row['tracked_path'];b=subprocess.check_output(['git','-C',str(a.publication),'show',revision+':'+path]);oid=subprocess.check_output(['git','-C',str(a.publication),'rev-parse',revision+':'+path]).decode().strip();assert (len(b),h(b),oid)==(row['bytes'],row['whole_file_sha256'],row['git_blob']);verified+=1
print(json.dumps({'packet_members_verified':len(files),'older_git_blobs_verified':verified,'unread_nested_members_qualified':4188,'scope':'Integrity and internal count reconciliation only; no universal privacy conclusion, private-token comparison or product validation.'}))
