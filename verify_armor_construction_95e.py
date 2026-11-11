#!/usr/bin/env python3
"""Verify source and recovery evidence only; do not run product behavior."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-ARMOR-CONSTRUCTION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2172-2178.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));before={};current={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2172,2179))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical_sha256']==item['sha256'] and regrow['historical_ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]]
  assert c['byte_identical']==(old==body)==(i!=2175) and c['current_personal_ranges']==[[1,c['lines']]] and regrow['current_sha256']==c['sha256'] and regrow['current_ranges']==c['current_personal_ranges']
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==c['lines'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);before[i]=old;current[i]=body
assert b'pub struct Micrometers(pub u16)' not in before[2175] and b'pub struct Micrometers(pub u16)' in current[2175] and len(before[2175].splitlines())==171 and len(current[2175].splitlines())==183
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
construction=current[2174];engraving=current[2176];frame=current[2178];bracer=sources['crates/fabelgeist-armor/src/gpu/bracer.rs'];caller=sources['crates/adventuresim-character-creator/src/device_bracer.rs'];staging=sources['crates/fabelgeist-armor/src/gpu/staging.rs'];template=sources['crates/fabelgeist-armor/src/tiling/template.rs'];tiling=sources['crates/fabelgeist-armor/src/tiling/mod.rs'];metal=sources['crates/fabelgeist-armor/src/material.rs'];bake=sources['crates/fabelgeist-armor/src/gpu/metal/bake.rs'];export=sources['crates/adventuresim-character-creator/src/export/textures.rs'];plan=sources['crates/adventuresim-character-creator/src/waist_support_plan.rs'];bodice=sources['crates/fabelgeist-armor/src/bodice.rs'];gown=sources['crates/fabelgeist-armor/src/gown.rs'];gownplan=sources['crates/adventuresim-character-creator/src/gown_plan.rs'];garment=sources['crates/adventuresim-character-creator/src/garment/fitted.rs'];lock=sources['Cargo.lock']
assert b'std::fs::read(image)' in engraving and b'format!("Engraving {}: {error}", image.display())' in engraving and b'image::load_from_memory(bytes)' in engraving and b'format!("Engraving {name}: {error}")' in engraving
assert b'Engraving::validate' in metal and b'engraving.cut().map(|cut| (engraving, cut))' in bake and b'.map_err(anyhow::Error::msg)' in export
assert b'.chain(&self.morphs)' in bracer and b'.zip(morphs)' in bracer and b'if results.status(slots[0]) != 0' in bracer and b'read_mesh(*target)?' in bracer and b'let bytes = readback.read(self.context()).await?' in staging
assert b'for morph in input.morphs' in caller and b'.morphs\n        .iter()' in caller and b'.map(|morph| morph.name.clone())' in caller and b'bracer.read_async(gpu, input.domain, &names)' in caller
assert b'pub const MAX_BEVEL: f32 = 0.45' in construction and b'const FIRST_HOLE_RADII: f32 = 2.2' in construction and b'const HOLE_CLEARANCE_RADII: f32 = 5.0' in construction and b'1.0 - plate.bevel * 2.0 / plate.width.min(plate.height)' in template and b'template.face(&cap, &holes, front' in template and b'for hole in &holes' in template and b'construction.validate()?' in tiling
width=Fraction(12);height=Fraction(20);thickness=Fraction(12);bevel=Fraction(5);radius=Fraction(23,10);assert bevel<=thickness*Fraction(45,100) and radius<width/5 and height/2-Fraction(22,10)*radius-radius>height/2*(1-2*bevel/width)
assert b'pub const THICKNESS_RANGE: RangeInclusive<u16> = 1..=6' in bodice and b'self.bodice.validate()?' in gown and b'design.validate()?' in gownplan and b'gown.bodice.thickness.0 * Micrometers::PER_MILLIMETRE as u16' in plan and 6*1000<2**16
assert b'v.is_finite()' in frame and b'self.origin[axis] +' in frame and b'vector(frame.point(p))' in garment
for x in a['native_semantics_evidence']:
 arc=(root/x['archive']).read_bytes();assert sha(arc)==x['archive_sha256'] and x['archive_sha256'].encode() in lock
 with tarfile.open(root/x['archive'],'r:gz') as ar:raw=ar.extractfile(x['member']).read()
 assert sha(raw)==x['member_sha256']
 if x['member'].endswith('/src/error.rs'):assert b'pub enum ImageError' in raw and b'Decoding(DecodingError)' in raw and b'IoError(io::Error)' in raw
 if x['member'].endswith('/dynimage.rs'):assert b'pub fn load_from_memory(buffer: &[u8]) -> ImageResult<DynamicImage>' in raw
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(725,range(2172,2178)),(726,range(2178,2179))]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);assert x['source_sha256']==roles[i-2172]['sha256'] and x['fresh_personal_body_reads']==[[1,roles[i-2172]['lines']]] and 'Subagent /root/lens12_frontier' in x['attribution']
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(341,range(2172,2174)),(342,range(2174,2175)),(343,range(2175,2179))]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2172]['lines']]] and row['sha256']==roles[i-2172]['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (136,137,142,2277,2278):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified original armor attributions, distinct current design, selected source traces and native image evidence; no product execution.')
