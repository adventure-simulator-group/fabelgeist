"""Verify historical and current GLB layout evidence without export."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-GLB-LAYOUT-95E.json'
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
assert len(sources)==5
p='crates/adventuresim-character-creator/src/export/'
def must(source,markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: '+marker)
layout=sources[p+'glb/layout.rs']
must(layout,['.checked_next_multiple_of(ALIGNMENT)',
    'encode_length(padded, role)?', '.checked_add(json_bytes)',
    '.checked_add(binary_bytes)', 'encode_length(byte_length, GlbLengthRole::Container)?',
    'u32::try_from(bytes).map_err'])
glb=sources[p+'glb.rs']
must(glb,['GlbLayout::new(json_bytes.len(), bytes.len())?',
    'glb.try_reserve_exact(layout.byte_length())?',
    'layout.encoded_length().to_le_bytes()', 'layout.json_length().to_le_bytes()',
    'layout.binary_length().to_le_bytes()',
    'let packet = PreparedGlb::new(document, bytes)?;', 'fs::create_dir_all(parent)',
    'fs::write(path, packet.0)'])
assert glb.index('let packet = PreparedGlb::new(document, bytes)?;') < glb.index('fs::create_dir_all(parent)') < glb.index('fs::write(path, packet.0)')
must(sources[p+'glb/error.rs'], ['pub enum GlbLayoutError', 'Width {', 'pub enum GlbError', 'Reserve(#[from] TryReserveError)'])
export=sources['crates/adventuresim-character-creator/src/export.rs']
must(export,['texture_images.finish(output, &mut document)?;', 'glb::write(output.path(), &document, buffer.bytes)'])
assert export.index('texture_images.finish(output, &mut document)?;') < export.index('glb::write(output.path(), &document, buffer.bytes)')
must(sources[p+'textures.rs'], ['SharedTextures(&\'a Path)', 'if let Some(files) = self.shared_files',
    'fs::create_dir_all(directory)', 'fs::write(&path, &bytes)'])
work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())
hist=doc['historical_sources'][0];role=work['roles'][hist['original_role_index']]
assert role['source_id']==hist['source_id'] and role['historical_sha256']==hist['sha256']
assert not hist['byte_identical'] and hist['sha256']!=hist['current_sha256']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    raw=archive.extractfile(hist['sha256']).read()
assert hashlib.sha256(raw).hexdigest()==hist['sha256']
historical=raw.decode()
must(historical,['(total_length as u32).to_le_bytes()', '(json_bytes.len() as u32).to_le_bytes()',
    '(bytes.len() as u32).to_le_bytes()'])
ledger=json.loads((root/'baseline-059/ledger.json').read_text())
old=ledger['segovb_continuation']['current_connected_dispositions'][102]
assert old['scope_revision']==doc['historical_records'][0]['scope_revision']
assert old['disposition']['id']==doc['historical_records'][0]['id']
assert old['latest_current_target_reconciled'] is False
for reference,field in zip(doc['recovered_historical_receipts'],('sha256','verification_sha256')):
    path=(root/reference['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and reference['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==reference['sha256']==old['receipt'][field]
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['new_nonfinding_assessments']
assert progress['queue_records_semantically_reconciled']==0
assert doc['dispositions'][0]['validation'].startswith('not performed')
print('verified five Git-bound sources, distinct frozen writer and two recovered receipts; no export ran')
