#!/usr/bin/env python3
"""Verify publication evidence only; do not execute product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GARMENT-DESIGN-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2261-2262.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2261,2263))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2261,2262)]==[19,763]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
design=bodies[2262];measure=sources['crates/fabelgeist-garment-code/src/measure/mod.rs'];cli=sources['crates/fabelgeist-garment-code/src/bin/garment_code.rs'];meta=sources['crates/fabelgeist-garment-code/src/programs/meta_garment.rs'];bodice=sources['crates/fabelgeist-garment-code/src/programs/bodice.rs'];tee=sources['crates/fabelgeist-garment-code/src/programs/tee.rs'];rules=sources['crates/AGENTS.md'];readme=sources['crates/fabelgeist-garment-code/README.md'];assets=sources['crates/fabelgeist-garment-code/src/assets.rs'];preset=sources['crates/fabelgeist-garment-code/assets/design_params/t-shirt.yaml'];inventory=sources['crates/adventuresim-core/src/inventory_measurement.rs']
assert b'pub fn from_yaml_str(text: &str) -> Result<Self>' in design and b'out.eval_dependencies();' in design and b'let height = self.get("height");' in design and b'Body::ERROR::unknown measurement' in design
assert b'pub fn insert(&mut self, key: &str, value: f64)' in design and b'let mut set = |name: &str, value: f32| body.insert(name, value as f64);' in measure and b'body.eval_dependencies();' in measure
assert b'let body = Body::load(&args.body)?;' in cli and b'let design = Design::load(&args.design)?;' in cli and b'let piece = MetaGarment::new(&name, &body, &design);' in cli
assert b'let upper_name = design.s("meta.upper");' in meta and b'"Shirt" => shirt(body, design, false)' in meta and b'let right = bodice_half("right", body, &design, fitted);' in bodice and b'tee::torso_half_panel(' in bodice and b'd.f("width") * body.get("bust")' in tee
for key in [b'height',b'head_l',b'waist_line',b'hips_line',b'shoulder_w',b'bust_line',b'hip_inclination',b'shoulder_incl',b'armscye_depth']:
 assert b'self.get("'+key+b'")' in design and b'set("'+key+b'",' in measure
assert b'self.contains("vert_bust_line")' in design and b'pub enum MeasurementKind' in inventory and b'Containerized' in inventory and b'pub struct MeasurementBasis' in inventory
assert b'pub fn from_value(v: Value) -> Self' in design and b'Ok(Self::from_value(from_yaml(design)))' in design and b'serde_yaml::Value::Sequence(s) => Value::List' in design and b'serde_yaml::Value::Mapping(m)' in design
assert b'for item in items {' in design and b'format!("{pad}- {}\\n", scalar_yaml(item))' in design and b'Value::List(_) => "[]".to_string()' in design and b'Value::Map(_) => "{}".to_string()' in design and b'Design::to_yaml()' in readme
assert b'path: path.join(".")' in design and b'key.split(\'.\').collect()' in design and b'Dot-separated path, ready for' in design
assert b'.map(|v| v.as_str().map(str::to_string))' in design and b'`None` option is the YAML `null`' in design and b'map.get_mut(k).unwrap()' in design and b'Never use `unwrap` or `expect` outside tests' in rules
assert b'include_str!("../assets/design_params/t-shirt.yaml")' in assets and b'type: select_null' in preset
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==31
adler=json.loads(records['adler/READ-L11-360.json']);assert adler['accepted_unique_count_change']==0
for i,scope in ((2261,[[1,19]]),(2262,[[472,608]])):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==scope and x['sha256']==roles[i-2261]['sha256']
for batch in (744,745):
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256']
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
 if batch==744:
  for i,scope in ((2261,[[1,19]]),(2262,[[472,608]])):
   x=next(r for r in data['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==scope and x['source_sha256']==roles[i-2261]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
 else:
  x=next(r for r in data['rows'] if r['original_index']==2263);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,791]] and 'Subagent /root/lens12_frontier' in x['attribution']
assert roles[1]['original_supplier_scopes'][0]['read_ranges']==[[472,608]] and roles[1]['original_producer_scopes'][0]['read_ranges_as_recorded']==[[472,608]]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (155,310,314,1829):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified garment source, partial historical scopes and conditional contracts; no product execution.')
