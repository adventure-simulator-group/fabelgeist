#!/usr/bin/env python3
"""Verify source provenance and symbolic selected-ring arithmetic only."""
import difflib
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
    out,pos={},0
    while pos<len(b):
        end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
    assert pos==len(b);return out
def chain(path,r,body,root_tree,nodes):
    assert [s['name'] for s in r['chain']]==path.split('/')
    current=root_tree
    for s in r['chain']:
        info=nodes[current];data=contents(info['preserved_bytes']);assert oid('tree',data)==current==s['tree'] and sha(data)==info['sha256'] and len(data)==info['bytes']
        assert entries(data)[s['name']]==(s['mode'],s['child']);current=s['child']
    assert current==r['git_blob']==oid('blob',body) and sha(body)==r['sha256'] and len(body)==r['bytes'] and len(body.splitlines())==r['lines']
revision='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-VISTA-SELECTED-LOD-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1636.json');w=load('COVERAGE-L01-ROLES.json');proof=load('VISTA-SELECTED-LOD-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
x,=h['historical_source_roles'];role=w['roles'][1636];assert x['original_index']==1636 and x['prior_role_status']=='unassessed'
assert x['new_personal_read_ranges']==[[1,200],[201,400],[401,600],[601,800],[801,1000],[1001,1187]] and role['status']=='full-historical-body-trace-open'
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];current=contents(c['preserved_bytes'])
    assert sha(old)==role['historical_sha256']==x['sha256'] and len(old.splitlines())==1187 and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1636.json'
    assert sha(current)==c['sha256'] and len(current)==c['bytes'] and len(current.splitlines())==1188 and oid('blob',current)==c['git_blob']==tree[c['path']]
    assert c['current_personal_ranges']==[[1,200],[201,329]] and c['byte_identical'] is False
    diff=''.join(difflib.unified_diff(old.decode().splitlines(keepends=True),current.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode();assert diff==contents(c['exact_diff']['preserved_bytes']) and sha(diff)==c['exact_diff']['sha256']
    chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes)
    assert [e['original_index'] for e in a['earlier_historical_source_identities']]==list(range(1637,1642))
    for e in a['earlier_historical_source_identities']:
        b=ar.extractfile(e['sha256']).read();r=w['roles'][e['original_index']];path=e['source_id'].split(':',1)[1]
        assert sha(b)==r['historical_sha256'] and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0
        chain(path,proof['paths'][path],b,proof['root_tree'],nodes)
sources={}
for s in a['current_supporting_sources']:
    b=contents(s['preserved_bytes']);assert s['revision']==revision and sha(b)==s['sha256'] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
    assert oid('blob',b)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];b=contents(r['preserved_bytes']);assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(b)==r['published_sha256'] and len(b)==r['published_bytes'] and (r['redacted_local_path_occurrences'] or (sha(b)==r['sha256'] and oid('blob',b)==r['git_blob']));records[r['original_path']]=b
p='adler/READ-L11-235.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-235.json'])
assert [r['original_index'] for r in z['sources']]==list(range(1636,1642)) and z['accepted_unique_count_change']==0 and z['current_reconciled'] is False and z['validated'] is False
assert z['sources'][0]['fresh_reads']==[[1,190],[191,380],[381,570],[571,760],[761,950],[951,1140],[1141,1187]]
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-235.json']['sha256']
for r in z['sources']+z['connected_reinspection']:
    if r.get('registration_sha256'):
        reg='adler/'+r['registration'].replace('\\','/').split('/adler/')[-1];assert reg in records and r['registration_sha256']==lookup[reg]['sha256']
for r in z.get('same_lens_context',[]):
    ref='adler/'+r['receipt'].replace('\\','/').split('/adler/')[-1];assert ref in records and r['sha256']==lookup[ref]['sha256']
p='segovb/READ-L12-518.json';z=json.loads(records[p]);check=json.loads(records['segovb/VERIFY-L12-518.json'])
assert [r['original_index'] for r in z['rows']]==[1636] and z['rows'][0]['fresh_personal_body_reads']==[[1,1187]]
assert z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False and z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['segovb/STATE-L12-518.json']['sha256']
for r in z['rows']:assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
for r in z['partial_sequence']:assert r['path'] in records and r['sha256']==lookup[r['path']]['sha256']
surface=sources['crates/adventuresim-tactical-client/src/presentation/vista/surface.rs'];prepared=sources['crates/adventuresim-tactical-client/src/presentation/vista/prepared.rs'];terrain=sources['crates/adventuresim-tactical-client/src/presentation/terrain.rs'];core=sources['crates/adventuresim-tactical-core/src/vista_surface.rs'];pending=sources['crates/adventuresim-tactical-client/src/presentation/vista/pending.rs'];config=sources['crates/adventuresim-tactical-client/src/presentation/config.rs'];natural=sources['crates/adventuresim-tactical-client/src/presentation/vista/natural.rs'];setup=sources['crates/adventuresim-tactical-server/src/scene_setup.rs']
assert b'.take(settings.config.rendering.vista.maximum_lods)' in current and b'PreparedVistaLod::prepare_all(' in current and b'active_surface.update(&bundle)' in current
assert b'lods.get(source_index + 1).copied()' in prepared and b'bundle.lods.iter().take(2).cloned().collect()' in surface and b'presented_height_at(lod, world, self.lods.get(1))' in surface
assert b'vista.presented_height_at(&environment.scene_digest, terrain, point)' in terrain and b'DETAIL_PATCH_RADIUS_METRES: f32 = 12.0' in terrain and b'DETAIL_PATCH_SPACING_METRES: f32 = 0.5' in terrain
assert b'let Some(coarser) = coarser_lod else' in core and b'own.lerp(height, weight)' in core and b'((radius - (half_extent - transition_width)) / transition_width).clamp(0.0, 1.0)' in core
assert b'cell_rectangles_outside_inner_rectangle(' in natural and b'geometry.elevation().metres()' in natural
assert b'self.rendering.vista.maximum_lods = 1' in config and b'maximum_lods: 3' in sources['assets/config/tactical-graphics.yaml']
assert b'self.0 = (!ready).then(|| bundle.clone())' in pending and current.index(b'pending.accept(')<current.index(b'PreparedVistaLod::prepare_all(')<current.index(b'active_surface.update(&bundle)')
assert b'lods: input.vista.lods.clone()' in setup
# Grid and morph arithmetic only; no product functions are run.
playable_half=Fraction(1,8); first_half=Fraction((9-1)*2,2); second_spacing=Fraction(4);point=Fraction(8);width=max(min(second_spacing,first_half),Fraction(2));weight=max(Fraction(0),min(Fraction(1),(point-(first_half-width))/width));seam=max(Fraction(0),min(Fraction(1),(point-playable_half)/2))
assert (first_half,width,weight,seam)==(8,4,1,1) and (12+(38-12)*weight)==38 and point < 12
q=load('RECONCILIATION-059.json')['entries'];assert q[1358]['current_reconciliation'].startswith('partial ') and q[1604]['contract_id']=='Q-L12-9903ce51fd5c5c004618' and q[1609]['contract_id']=='Q-L12-28bd4882ec33269c17e8'
finding,=a['conditional_findings'];assert finding['id']=='REVIEW-VISTA-RETAINED-UNSELECTED-LOD-001' and finding['accepted_historical_unique_count_change']==0
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries'];assert sum(x['id']==finding['id'] and x['record']=='ASSESSMENT-VISTA-SELECTED-LOD-95E.json' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified historical vista role, qualified prior receipts and symbolic selected-ring trace; no product execution.')
