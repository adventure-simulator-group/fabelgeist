#!/usr/bin/env python3
"""Verify source, historical provenance, and symbolic paths; no filesystem probe."""
import gzip
import hashlib
import json
import posixpath
import tarfile
from pathlib import Path

r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-SIM-OUTPUT-ALIAS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1298-1300.json')
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
work=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/work['current_tree_inventory']).read_text().splitlines():
 header,path=line.split('\t',1);mode,kind,oid=header.split();tree[path]=oid
s={}
for item in a['current_sources']:
 raw=gzip.decompress((r/item['preserved_bytes']).read_bytes())
 assert item['revision']==rev and len(raw)==item['bytes'] and sha(raw)==item['sha256']
 assert blob(raw)==item['git_blob']==tree[item['path']] and len(raw.splitlines())==item['lines']
 assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']);s[item['path']]=raw
assert [x['original_index'] for x in h['historical_source_roles']]==[1298,1299,1300]
assert [x['lines'] for x in h['historical_source_roles']]==[561,789,237]
for item in h['historical_source_roles']:
 role=work['roles'][item['original_index']]
 assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1298-1300.json'
 assert item['new_personal_read_ranges']==[[1,item['lines']]]
 with tarfile.open(r/item['evidence_archive'],'r:gz') as ar:raw=ar.extractfile(item['sha256']).read()
 assert sha(raw)==item['sha256'] and len(raw.splitlines())==item['lines'] and raw==s[item['source_id'].split(':',1)[1]]
assert work['roles'][1301]['status']=='unassessed'
main=s['crates/adventuresim-strategic-sim/src/main.rs']
assert b'let lexical = lexical_normalize(&absolute);' in main and b'fs::canonicalize(existing)?.join(missing_tail)' in main
assert b'Component::ParentDir => {\n                normalized.pop();' in main
assert b'options.create(true).truncate(true);' in main and b'options.create_new(true);' in main
assert main.index(b'write_output(&public_output, &bundle.artifacts.public_json, overwrite)?') < main.index(b'&bundle.artifacts.developer_json,') < main.index(b'&bundle.artifacts.stories_markdown,')
report=s['crates/adventuresim-strategic-sim/src/investigation_eval/report.rs']
assert b'let privacy_audit = privacy_audit(&public_json, &private)?;' in report and b'cases: private,' in report
types=s['crates/adventuresim-strategic-sim/src/investigation_eval/types.rs']
assert b'This type must never be passed to a policy or serialized into public trace' in types and b'pub canonical_case_id: String' in types and b'pub true_site: String' in types
integration=s['crates/adventuresim-strategic-sim/tests/core_loop_live.rs']
assert b'#[ignore = "requires an explicitly published disposable loopback SpacetimeDB database"]' in integration
helper=integration.split(b'fn assert_ordered_subsequence(',1)[1]
assert b'event.kind == expected[cursor]' in helper and b'event.agent_id' not in helper and b'event.subject' not in helper
bootstrap=s['crates/adventuresim-strategic-sim/src/live_core/bootstrap.rs']
wrapper=bootstrap.split(b'pub fn run_core_loop(',1)[1].split(b'fn run_core_loop_inner(',1)[0]
assert b'Ok(report) => return Ok(report)' in wrapper and b'validate_quest_coverage' not in wrapper
assert b'if config.require_quest_coverage {' in bootstrap and b'if require_quest_coverage && let Err(error) = validate_quest_coverage(&report)' in main
paths=a['symbolic_path_scenario']
assert posixpath.normpath(paths['input_public'])==paths['validator_lexical_public']=='/a/report.json'
assert posixpath.normpath(paths['input_developer'])==paths['validator_lexical_developer']=='/b/report.json'
assert paths['assumed_symlink']=='/a/link -> /b/sub' and paths['linux_resolved_public']==paths['linux_resolved_developer']=='/b/report.json'
rust_path=gzip.decompress((r/'native-evidence/rust-nightly-2026-07-13/library/std/src/path.rs.gz').read_bytes())
rust_fs=gzip.decompress((r/'native-evidence/rust-nightly-2026-07-13/library/std/src/fs.rs.gz').read_bytes())
assert b'its parent isn\'t `a`' in rust_path and b'symbolic links resolved' in rust_fs
manual=gzip.decompress((r/'native-evidence/linux-man-pages-6.7/path_resolution.7.gz').read_bytes())
link=gzip.decompress((r/'native-evidence/linux-man-pages-6.7/link.2.gz').read_bytes())
assert b'we first resolve this symbolic link' in manual and b'the directory itself and to its parent directory' in manual
assert b'both names refer to the same file' in link
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()))
original={x['path']:x for x in manifest['entries']};records={}
for item in a['recovered_historical_records']:
 orig=original[item['original_path']];assert item['sha256']==orig['sha256'] and item['bytes']==orig['bytes'] and item['git_blob']==orig['git_blob']
 raw=gzip.decompress((r/item['preserved_bytes']).read_bytes());assert len(raw)==item['published_bytes'] and sha(raw)==item['published_sha256']
 assert item['redacted_local_path_occurrences'] or (sha(raw)==item['sha256'] and blob(raw)==item['git_blob'])
 records[item['original_path']]=json.loads(raw)
for batch,index,lines in [(350,1298,561),(351,1299,789),(352,1300,237)]:
 receipt=records[f'segovb/READ-L12-{batch}.json']
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert receipt['rows'][0]['original_index']==index and receipt['rows'][0]['fresh_personal_body_reads']==[[1,lines]]
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
receipt=records['segovb/READ-L12-352.json']
assert receipt['rows'][1]['original_index']==1301 and receipt['rows'][1]['fresh_personal_body_reads']==[[1,131]]
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1090,1091,1092))
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==a['candidates'][0]['id'] for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified three historical reads, native source and symbolic path reasoning; no product execution or path probe.')
