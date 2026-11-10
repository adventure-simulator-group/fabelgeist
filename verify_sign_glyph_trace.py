"""Verify font maps and source provenance, without invoking product name or render code."""
import gzip
import hashlib
import json
import struct
import tarfile
import tomllib
import unicodedata
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-SIGN-GLYPH-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, oid = header.split()
        assert name not in result
        result[name] = oid
    return result

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
def load(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip' and item['revision'] == revision
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert oid == item['git_blob'] == current[item['path']] == previous[item['path']]
    if 'lines' in item:
        assert len(raw.splitlines()) == item['lines'] and all(1 <= a <= b <= item['lines'] for a,b in item['ranges'])
    return raw

sources = {item['path']: load(item).decode() for item in doc['current_sources']}
assert len(sources) == len(doc['current_sources']) == 18
assets = {item['path'].split('/')[-1]: load(item) for item in doc['font_assets']}
assert len(assets) == 4
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
must(sources['crates/adventuresim-world-schema/src/person_names.rs'], ['pub const MAX_CHARS: usize = 160', 'value.trim() != value', 'value.chars().any(char::is_control)', 'RenderedPersonalName::new(full_name.clone())'])
must(sources['crates/adventuresim-world-schema/src/person_names/identity_validation.rs'], ['GivenNameResolution::Authored { full_name }', 'RenderedPersonalName::new(full_name.clone())?'])
must(sources['crates/adventuresim-stdb-module/src/character/name_reducers.rs'], ['require_living_character(ctx, id)?', 'assign_authored_character_name'])
must(sources['crates/adventuresim-stdb-module/src/character/name_identity.rs'], ['render_personal_name(', 'character.name = display.into_string();'])
must(sources['crates/adventuresim-stdb-module/src/tactical/request.rs'], ['settlement_business_operator()', '.find(assignment.operator_character_id)', 'operator_name,'])
must(sources['crates/adventuresim-tactical-server-dispatcher/src/main.rs'], ['RenderedPersonalName::try_from(', 'operator.operator_name.clone()'])
must(sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input/establishments.rs'], ['ShopName::for_operator(&operator_name, site.key.usage)'])
must(sources['crates/adventuresim-tactical-core/src/scene_input/establishments.rs'], ['establishment.shop_name != ShopName::for_operator(&establishment.operator_name, usage)'])
must(sources[p+'signs.rs'], ['proprietor: format!("{operator}’s")', 'Inn => "Tavern"', 'if name.trade != shop_trade(usage)?', 'font: SignFont::GrenzeGotisch'])
must(sources[p+'signs/lettering.rs'], ['GrenzeGotisch-Bold.ttf', 'UnifrakturCook-Bold.ttf', 'line_width(&face, &full, single_px)', 'let id = scaled.glyph_id(character);', 'assert_ne!(id.0, 0, "sign font lacks {character:?}");'])
must(sources[p+'signs/rendering.rs'], ['SignDetail::Lettering', 'self.painted_material(sign, board, detail, materials, images)'])
must(sources[p+'signs/rendering/materials.rs'], ['if detail == SignDetail::Board', 'SignTexture::paint(sign, board.size)'])
must(sources['crates/adventuresim-tactical-client/src/presentation/buildings/signs.rs'], ['SignDetail::Board', 'LETTERING_LOAD_DISTANCE_METRES: f32 = 48.0', 'distance < LETTERING_LOAD_DISTANCE_METRES', 'SignDetail::Lettering'])
must(sources['crates/adventuresim-building-generator/Cargo.toml'], ['sign-render = ["dep:ab_glyph"'])
must(sources['crates/adventuresim-tactical-client/Cargo.toml'], ['features = ["sign-render"]'])

packages = {(x['name'], x['version']): x for x in tomllib.loads(sources['Cargo.lock'])['package']}
assert 'owned_ttf_parser' in packages[('ab_glyph', '0.2.32')]['dependencies']
assert 'ttf-parser' in packages[('owned_ttf_parser', '0.25.1')]['dependencies']
native = {}
for dep in doc['native_dependencies']:
    manifest_path = (root / dep['manifest']).resolve()
    assert manifest_path.is_relative_to(root)
    manifest = json.loads(manifest_path.read_text())
    archive = manifest_path.parent / manifest['archive']
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == dep['archive_sha256'] == manifest['sha256'] == packages[(manifest['package'], manifest['version'])]['checksum']
    with tarfile.open(archive, 'r:gz') as tar:
        for item in manifest['inspected_members']:
            raw = tar.extractfile(item['path']).read()
            assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
            native[item['path']] = raw.decode()
must(native['ab_glyph-0.2.32/src/ttfp.rs'], ['self.0.glyph_index(c).map(|id| id.0).unwrap_or(0)'])
must(native['ab_glyph-0.2.32/src/scale.rs'], ['fn glyph_id(&self, c: char)', 'self.font().glyph_id(c)'])
must(native['owned_ttf_parser-0.25.1/src/preparse.rs'], ['.filter(|st| st.is_unicode())', '.find_map(|t| t.glyph_index(c.into()))'])
must(native['ttf-parser-0.25.1/src/tables/cmap/mod.rs'], ['PlatformId::Unicode => true', 'WINDOWS_UNICODE_BMP_ENCODING_ID', 'SegmentMappingToDeltaValues(ref subtable)'])
must(native['ttf-parser-0.25.1/src/tables/cmap/format4.rs'], ['if start_value > code_point', '0 indicates missing glyph', 'return None;'])

# Inspect the exact OpenType cmap tables, not product code. All Unicode records
# in these two pinned fonts are format 4. Validate offsets before reading and
# check positive controls so absence is not an empty-table artifact.
def cmap4(font):
    def u16(offset):
        assert 0 <= offset <= len(font)-2
        return struct.unpack_from('>H', font, offset)[0]
    def i16(offset):
        assert 0 <= offset <= len(font)-2
        return struct.unpack_from('>h', font, offset)[0]
    def u32(offset):
        assert 0 <= offset <= len(font)-4
        return struct.unpack_from('>I', font, offset)[0]
    tables = {}
    for i in range(u16(4)):
        pos = 12 + 16*i
        assert pos+16 <= len(font)
        tag = font[pos:pos+4]
        off, size = u32(pos+8), u32(pos+12)
        assert off+size <= len(font)
        tables[tag] = (off, size)
    base, size = tables[b'cmap']
    assert u16(base) == 0
    records = []
    for i in range(u16(base+2)):
        pos = base+4+8*i
        assert pos+8 <= base+size
        platform, encoding, relative = u16(pos), u16(pos+2), u32(pos+4)
        if platform != 0 and (platform, encoding) != (3,1):
            continue
        sub = base+relative
        assert u16(sub) == 4
        length = u16(sub+2)
        assert sub+length <= base+size
        count = u16(sub+6)//2
        ends = sub+14
        starts = ends+2*count+2
        deltas = starts+2*count
        offsets = deltas+2*count
        assert offsets+2*count <= sub+length
        records.append((platform, encoding, sub, length, count, ends, starts, deltas, offsets))
    assert {(r[0],r[1]) for r in records} == {(0,3),(3,1)}
    def lookup(record, code):
        _,_,sub,length,count,ends,starts,deltas,offsets = record
        for i in range(count):
            end, start = u16(ends+2*i), u16(starts+2*i)
            if start <= code <= end:
                delta, offset = i16(deltas+2*i), u16(offsets+2*i)
                if offset == 0:
                    return (code+delta) & 0xffff
                assert offset != 0xffff
                pos = offsets+2*i+offset+2*(code-start)
                assert sub <= pos and pos+2 <= sub+length
                value = u16(pos)
                return ((value+delta) & 0xffff) if value else 0
        return 0
    for record in records:
        assert lookup(record, 0x0416) == 0
        assert lookup(record, 0x0041) != 0 and lookup(record, 0x2019) != 0
    return len(records)

assert cmap4(assets['GrenzeGotisch-Bold.ttf']) == 2
assert cmap4(assets['UnifrakturCook-Bold.ttf']) == 2
character = doc['font_observation']['character']
assert character == 'Ж' and ord(character) == 0x0416 and unicodedata.category(character) == 'Lu'
assert character.strip() == character and len(character) == 1 and character in 'Ж’s Tavern'
h = doc['historical_source']
ledger = json.loads((root / h['ledger']).read_text())
assert h['related_site_record'] in ledger['discovery_findings_and_dependencies']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
reg = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in reg['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for item in h['historical_source_roles']:
        r = roles[item['index']]
        path = item['source_id'].split(':',1)[1]
        assert r['source_id'] == item['source_id'] and r['historical_sha256'] == item['sha256'] == hashes[item['index']]
        assert r['historical_assessment'] == item['assessment'] and hashlib.sha256(tar.extractfile(item['sha256']).read()).hexdigest() == item['sha256']
        assert hashlib.sha256(sources[path].encode()).hexdigest() == item['sha256']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified 18 Git-bound sources, four font/license assets, three locked archives and Unicode cmap absence; no product ran')
