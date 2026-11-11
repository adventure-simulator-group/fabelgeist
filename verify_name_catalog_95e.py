#!/usr/bin/env python3
"""Verify source and recovery evidence only; do not run product behavior."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-NAME-CATALOG-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2165-2171.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2165,2172))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])]
  expected=[x['read_ranges_as_recorded'] for x in r.get('expanded_producer_contract_observations',[])+r.get('producer_reached_contract_observations',[])];assert [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==expected
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['sha256']==item['sha256'] and regrow['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);assert sha(old)==row['historical_sha256'] and old==body and len(old.splitlines())==n and c['current_personal_ranges']==[[1,n]]
  assert c['byte_identical'] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
validator=bodies[2166];derivations=bodies[2167];names=bodies[2168];error=bodies[2169];identity=bodies[2170];digest=bodies[2171]
build=sources['crates/adventuresim-world-schema/build.rs'];stream=sources['crates/fabelgeist-determinism/src/stream.rs'];cargo=sources['crates/fabelgeist-determinism/Cargo.toml'];lock=sources['Cargo.lock'];start=sources['crates/adventuresim-core/src/starting_character/names.rs'];db=sources['crates/adventuresim-stdb-module/src/character/name_identity.rs'];dbtypes=sources['crates/adventuresim-stdb-module/src/character/name_types.rs']
assert b'name_catalog_validation::validate(&catalog)' in build and b'apply_repertoire_fragments(&mut catalog)' in build and b'serde_json::to_string(&catalog)' in build
family=validator.split(b'fn validate_family_frequencies',1)[1].split(b'fn validate_form_frequencies',1)[0];form=validator.split(b'fn validate_form_frequencies',1)[1].split(b'fn validate_surname_frequencies',1)[0];surname=validator.split(b'fn validate_surname_frequencies',1)[1].split(b'fn validate_observations',1)[0]
assert b'frequency.frequency == 0' in family and b'checked_add' not in family and b'checked_add(frequency.frequency)' in form and b'frequency.frequency == 0' in surname and b'checked_add' not in surname and b'targets.get(&target) != Some(&derivation.frequency)' in derivations
assert b'Error::Overflow => SamplingError::WeightOverflow' in stream and b'pub enum SamplingError' in stream and names.count(b'NameCatalogError::Sampling(error.to_string())')==3 and b'Sampling(String)' in error and b'impl std::error::Error for NameCatalogError {}' in error
assert b'generate_personal_name(context, NameStableSeed::from(stable_seed), None)' in start and b'.expect("the build-validated German repertoire covers the MVP period")' in start and b'InvalidName(adventuresim_world_schema::person_names::NameCatalogError)' in dbtypes
assert b'form.text.trim().is_empty()' in validator and b'value.chars().any(char::is_control)' in names and b'value.chars().count() > Self::MAX_CHARS' in names and b'pub const MAX_CHARS: usize = 160' in names
assert b'pub struct $name(pub String)' in names and b'Some(id) => Some(id)' in names and b'surname.id == id.as_str()' in identity and b'render_personal_name(' in db and db.index(b'render_personal_name(')<db.index(b'character.name = display.into_string()')
assert b'pub struct SourcePackageDigest(String)' in digest and b'Self::from_hex(&String::deserialize(decoder)?)' in digest
native=a['native_semantics_evidence'][0];archive=(root/native['archive']).read_bytes();assert sha(archive)==native['archive_sha256'] and native['archive_sha256'].encode() in lock and b'version = "=0.9.5"' in cargo
with tarfile.open(root/native['archive'],'r:gz') as ar:raw=ar.extractfile(native['member']).read()
assert sha(raw)==native['member_sha256'] and b'total_weight.checked_add_assign(w.borrow())' in raw and b'return Err(Error::Overflow)' in raw and 2**64-1+1>2**64-1
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(723,range(2165,2168)),(724,range(2168,2172))]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);assert x['source_sha256']==roles[i-2165]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
  if i<2171:assert x['fresh_personal_body_reads']==[[1,roles[i-2165]['lines']]]
  else:
   assert x['fresh_personal_body_reads']==[] and x['qualified_same_lens_reuse']['zero_new_rereads'] and x['qualified_same_lens_reuse']['receipt'] in records
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-341.json']);assert data['accepted_unique_count_change']==0
for i in range(2165,2172):
 row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2165]['lines']]] and row['sha256']==roles[i-2165]['sha256']
for i in (2165,2166,2167):
 qual=json.loads(records[f'parallel-l12-20261029/QUALIFICATION-PAR-L12-ORIGINAL{i}-WINDOWS.json']);assert qual['body_read_credit']==0 and qual['source_sha256']==roles[i-2165]['sha256'] and qual['raw_locator']==registry['files'][i]['path']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (134,135,284,285,286,1808):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified original naming attributions, registered current source reads, connected caller and pinned sampler evidence; no product execution.')
