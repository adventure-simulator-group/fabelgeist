#!/usr/bin/env python3
"""Verify recovered and source evidence without executing product code."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-SHIELD-FORGE-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2128-2132.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2128:102,2129:231,2130:38,2131:187,2132:207};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
base='crates/adventuresim-weapon-model/';shield=current[2129];stock=current[2130];validate=sources[base+'src/recipe/validation/shields.rs'];model=sources[base+'src/model/shields.rs'];fitting=sources[base+'src/model/shields/fittings.rs'];catalog=sources[base+'catalog/authoring.json'];quote=sources['crates/adventuresim-core/src/smithing.rs'];measurement=sources['crates/adventuresim-core/src/inventory_measurement.rs'];reducer=sources['crates/adventuresim-stdb-module/src/smithing.rs'];amount=sources['crates/adventuresim-stdb-module/src/inventory_amount.rs'];itemcat=sources['content/items/catalog.yaml'];merchant=sources['crates/strategic-web/src/templates/settlement/trade/merchant.rs']
assert b'pub fitting_mode: RoundShieldFittingMode' in shield and b'pub strap_width: Option<Metres>' in shield and b'pub strap_thickness: Option<Metres>' in shield
assert b'$p.strap_width,' in validate and b'$p.strap_thickness,' in validate and b'positive(n.ok_or(RecipeError::Dimension)?.get())?' in validate and b'if $strapped {' in validate and b'.max(p.strap_width.unwrap().get() / 2.0)' in validate
assert b'p.fitting_mode == RoundShieldFittingMode::GripAndStrap' in model and b'if self.strapped {' in fitting and b'parts.extend(self.strap(r, body, &layout, length, detail)?)' in fitting
assert b'"id": "buckler"' in catalog and b'"fittingMode": "grip"' in catalog and b'"radius": 0.18' in catalog and b'"gripLength": 0.13' in catalog and b'"strapWidth": 0.035' in catalog
assert b'Self::Steel => "steel_stock"' in stock and b'Self::Wood => "wood_stock"' in stock and b'Self::Steel => 5.0' in stock and b'Self::Wood => 4.0' in stock
for key in (b'steel_stock',b'leather_stock',b'brass_stock',b'wood_stock'):assert key in stock and key in itemcat and key in amount and key in merchant
assert b'BTreeMap<String, u32>' in quote and b'ConsumableFractionMicros::MICROS_PER_WHOLE' in quote and b'entry(stock.item_id().into())' in quote and b'1_000_000' in measurement and b'row.item_id == item_id' in reducer and b'{required} milliunits required' in reducer
assert 1000000//1000==1000 and 65+200>177
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
path='segovb/READ-L12-713.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-713.json']);assert [x['original_index'] for x in data['rows']]==list(lengths) and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for row in data['rows']:
 i=row['original_index'];assert row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['source_sha256']==sha(current[i])
 if 'exact_locator_qualification' in row:assert row['exact_locator_qualification'] in records
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-332.json'])
for i in lengths:
 row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (125,287,1798):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical roles, retained reviewer attribution and selected shield and forge source traces; no product execution.')
