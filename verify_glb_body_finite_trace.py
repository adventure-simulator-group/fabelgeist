"""Verify source and recovered evidence for a GLB normal admission trace."""
import gzip
import hashlib
import json
import struct
from pathlib import Path

root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-GLB-BODY-NONFINITE-95E.json'
doc=json.loads((root/name).read_text())
assert doc['lens']==1 and doc['source_revision']==revision

def tree(path):
    result={}
    for line in (root/path).read_text().splitlines():
        header,filename=line.split('\t',1)
        _,kind,oid=header.split()
        assert filename not in result and kind in ('blob','commit')
        result[filename]=oid
    return result

current=tree('main-advance-95e/current-tree.tsv')
previous=tree('main-advance-95e/previous-tree.tsv')
sources={}
for item in doc['current_sources']:
    path=(root/item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
    assert item['revision']==revision and oid==item['git_blob']
    assert current[item['path']]==previous[item['path']]==oid
    assert hashlib.sha256(raw).hexdigest()==item['sha256']
    assert len(raw)==item['bytes'] and len(raw.splitlines())==item['lines']
    assert all(1<=a<=b<=item['lines'] for a,b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']]=raw.decode()
assert len(sources)==10
p='crates/adventuresim-character-creator/src/'
def must(source,markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: '+marker)
must(sources[p+'lib.rs'], ['#[cfg(feature = "offline-creator")]\npub mod export;'])
must(sources[p+'export/rigged.rs'], ['pub struct RiggedMesh', 'pub positions: &\'a [[f32; 3]]',
    'pub normals: &\'a [[f32; 3]]', 'pub export_body: bool'])
validation=sources[p+'export/validation.rs']
must(validation,['mesh.normals.len() != vertices', 'mesh.joint_indices.len() != vertices',
    'mesh.joint_weights.len() != vertices', '.any(|index| *index as usize >= vertices)',
    'if !sum.is_finite()', '.normals\n            .iter()\n            .flatten()\n            .any(|value| !value.is_finite())'])
assert validation.count('mesh.normals')==1
export=sources[p+'export.rs']
must(export,['validate(lod, mesh, shells, sockets)?;',
    '&f32_bytes(mesh.normals.iter().flatten().copied())',
    'accessor(normals, 5_126, mesh.normals.len(), "VEC3", None)',
    'if mesh.export_body {', '"NORMAL": normal_accessor',
    'fn exports_four_normalized_skin_influences_and_a_zero_animation_rig()',
    'let normals = [[0.0, -1.0, 0.0]; 3];', 'let faces = [[0, 1, 2]];'])
assert export.index('validate(lod, mesh, shells, sockets)?;') < export.index('&f32_bytes(mesh.normals.iter().flatten().copied())') < export.index('"NORMAL": normal_accessor')
must(sources[p+'export/buffer.rs'], ['.flat_map(f32::to_le_bytes)', 'self.bytes.extend_from_slice(bytes);'])
must(sources[p+'export/morphs.rs'], ['for target in *targets', '.any(|value| !value.is_finite())'])
must(sources[p+'export/skinning.rs'], ['.strongest_four()', 'skin.native_u16()'])
must(sources[p+'export/skeleton.rs'], ['validate_proportions(mesh)?;', 'basis.is_finite()'])
must(sources[p+'lod.rs'], ['(MIN_CHARACTER_LOD..=MAX_CHARACTER_LOD).contains(&value)'])
must(sources[p+'export/glb.rs'], ['GlbLayout::new(json_bytes.len(), bytes.len())?', 'glb.extend_from_slice(&bytes);'])
# Verify a representative IEEE-754 quiet-NaN word independently of product code.
word=0x7fc00000
value=struct.unpack('<f',struct.pack('<I',word))[0]
assert value != value and struct.unpack('<I',struct.pack('<f',value))[0]==word
ledger=json.loads((root/'baseline-059/ledger.json').read_text())
old=ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][155]
prior=ledger['segovb_continuation']['current_connected_dispositions'][102]
assert old['contract']['contract']==doc['historical_records'][0]['id']
assert prior['disposition']['id']==doc['historical_records'][1]['id']
assert prior['scope_revision']==doc['historical_records'][1]['scope_revision']
assert prior['latest_current_target_reconciled'] is False
for reference,expected in zip(doc['recovered_historical_receipts'],(old['receipt_sha256'],prior['receipt']['sha256'])):
    path=(root/reference['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and reference['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==reference['sha256']==expected
work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role=work['roles'][reference['original_index']]
    assert role['source_id']==reference['source_id'] and role['historical_sha256']==reference['historical_sha256']
    assert reference['byte_identical']==(reference['historical_sha256']==reference['current_sha256'])
base=json.loads((root/'BASELINE-95E-20261110.json').read_text())
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
candidate=doc['candidates'][0];entries=base['newly_assessed_ledger']['entries']
assert sum(x['id']==candidate['id'] and x['record']==name for x in entries)==1
assert progress['new_current_ledger_candidates']==len(entries) and progress['queue_records_semantically_reconciled']==0
assert candidate['discovery'].startswith('partial') and candidate['repair']=='not started' and candidate['validation']=='not performed'
print('verified ten Git-bound sources, two recovered receipts and IEEE word; no product ran')
