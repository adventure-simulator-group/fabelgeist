#!/usr/bin/env python3
"""Verify language evidence and attribution without executing product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
 d={};i=0
 while i<len(body):
  end=body.index(b'\0',i);mode,n=body[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),body[end+1:end+21].hex());i=end+21
 assert i==len(body);return d
def chain(path,proof,body,start,nodes):
 assert [x['name'] for x in proof['chain']]==path.split('/');current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-LANGUAGE-ADMISSION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2163.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
item=h['historical_source_roles'][0];assert len(h['historical_source_roles'])==1 and item['original_index']==2163 and item['original_ordered_ranges']==[[120,308],[315,530]] and item['separately_registered_historical_gap_ranges']==[[1,119],[309,314],[531,756]] and item['new_personal_read_ranges']==[[1,756]]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));assert registry['files'][2163]['owner_contract_proofs'][1]['read_ranges']==item['original_ordered_ranges']
reg=item['connected_gap_registration'];assert sha((root/reg['path']).read_bytes())==reg['sha256'] and load(reg['path'])['connected_ranges_to_read']==item['separately_registered_historical_gap_ranges']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:before=archive.extractfile(item['sha256']).read()
c=item['current_comparison'];body=contents(c['preserved_bytes']);row=work['roles'][2163]
assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and len(before.splitlines())==756 and sha(before)==row['historical_sha256'] and before==body
assert c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,756]] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes)
connected=h['historical_connected_sources'][0];init=contents(connected['preserved_bytes']);initp=connected['source_id'].split(':',1)[1];reg=connected['new_local_registration'];assert connected['historical_extension']=='ROOT-L3-EXT-0028' and connected['new_personal_read_ranges']==[[1,54]] and sha(init)==connected['sha256'] and len(init.splitlines())==54 and oid('blob',init)==connected['git_blob']==tree[initp] and sha((root/reg['path']).read_bytes())==reg['sha256'];chain(initp,proof['paths'][initp],init,proof['root_tree'],nodes)
sources={}
for item in a['current_supporting_sources']:
 source=contents(item['preserved_bytes']);assert item['revision']==rev and sha(source)==item['sha256'] and len(source)==item['bytes'] and len(source.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',source)==item['git_blob']==tree[item['path']];sources[item['path']]=source
stream=sources['crates/fabelgeist-determinism/src/stream.rs'];simulation=sources['crates/adventuresim-stdb-module/src/simulation.rs'];native_import=sources['crates/adventuresim-stdb-module/src/strategic/world_import.rs'];party=sources['crates/adventuresim-stdb-module/src/strategic/party_readiness.rs'];member=sources['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
assert b'#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]' in body and b'pub struct SettlementLanguageProfile' in body and b'self.german_total_bp() == crate::BASIS_POINTS_PER_WHOLE as u32' in body and b'.weighted_index(&[' in init and b'.expect("normalized language profile has a positive total")' in init and b'Error::InsufficientNonZero => SamplingError::ZeroTotalWeight' in stream
assert b'!settlement.languages.is_valid()' in native_import and b'infer_settlement_language_profile(' in native_import and b'.written_languages\n            .direct_fields_valid' in simulation
assert b'if self.direct(l) <= 0.0 {' in body and b'self.direct(s).max(0.0) * l.correlation(s)' in body and b'[0.35, 1.0, 0.05' in body and 1000*35//100==350
assert b'let mut top = [[(0_u64, -1.0_f32); 2]; 8]' in body and b'if values[0].0 != *id' in body and b'values[1].1' in body and b'character_ids.sort_unstable()' in party and b'pub character_id: u64' in member and b'#[primary_key]\n    #[auto_inc]\n    pub id: u64' in member
assert b'living_party_member_ids' in sources['crates/adventuresim-stdb-module/src/time/camp_rest.rs'] and b'party_common_oral_choices_capped' in sources['crates/adventuresim-stdb-module/src/time/settlement_rest.rs']
native=a['native_semantics_evidence'];arc=(root/native[0]['archive']).read_bytes();assert sha(arc)==native[0]['archive_sha256'] and native[0]['archive_sha256'].encode() in sources['Cargo.lock'] and b'version = "=0.9.5"' in sources['crates/fabelgeist-determinism/Cargo.toml']
with tarfile.open(root/native[0]['archive'],'r:gz') as ar:weighted=ar.extractfile(native[0]['member']).read()
assert sha(weighted)==native[0]['member_sha256'] and b'if total_weight == zero' in weighted and b'return Err(Error::InsufficientNonZero)' in weighted and native[0]['personally_inspected_ranges']==[[89,130]]
f32=(root/native[1]['path']).read_bytes();assert sha(f32)==native[1]['sha256'] and b'If exactly one of the arguments is NaN' in f32 and native[1]['personally_inspected_ranges']==[[1028,1053]]
ref=contents(native[2]['path']);assert sha(ref)==native[2]['sha256_uncompressed'] and b'It is also neither smaller nor greater than any float' in ref
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert records[connected['historical_object_original_path']]==init
path='segovb/READ-L12-721.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-721.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
r=next(x for x in data['rows'] if x['original_index']==2163);assert r['fresh_personal_body_reads']==[[120,308],[315,530]] and r['connected_gap_personal_reads'][0]['ranges']==[[1,119],[309,314],[531,756]] and r['combined_actual_body_assessed']==[[1,756]] and r['source_sha256']==sha(body) and r['connected_gap_personal_reads'][0]['registration'] in records
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-339.json']);r=next(x for x in data['sources'] if x['original_index']==2163);assert data['accepted_unique_count_change']==0 and r['fresh_reads']==[[1,756]] and r['sha256']==sha(body)
l2=json.loads(records['root-l2-extension-20261030/READ-ROOT-L2-EXT-021-033.json'])['rows'][7];l3=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-020-027.json'])['rows'][7];ini=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-028-031.json'])['rows'][0]
assert l2['fresh_personal_ranges']==[[1,119],[309,314],[531,756]] and l2['qualified_inherited_same_lens_ranges']==[[120,308],[315,530]] and l3['fresh_personal_ranges']==[[1,756]] and ini['fresh_personal_ranges']==[[1,54]] and ini['source_sha256']==sha(init)
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (3131,3132,2374,1806):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert old[3132]['contract_id']=='REVIEW-L3-INITIAL-LANGUAGE-PROFILE-WEIGHTS'
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified scoped language role and gaps, independent historical attribution, initialization and pinned source semantics; no product execution.')
