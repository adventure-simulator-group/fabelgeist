"""Verify selected leaf material evidence; no product or shader code runs."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L07-LEAF-ROUGHNESS-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 7 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 15
material = sources['crates/adventuresim-procedural-materials/src/lib.rs']
must(material, ['/// Perceptual roughness, physical thickness in metres, ground-litter',
                'pub physical_parameters: Vec4', 'TREE_LEAF_CARD_SHADER.into()'])
shader = sources['crates/adventuresim-procedural-materials/src/shaders/tactical_tree_leaf_card.wgsl']
must(shader, ['let arm = textureSample(leaf_arm, leaf_arm_sampler, in.uv).rgb;',
              'pbr_input.material.perceptual_roughness = arm.g;',
              'leaf_card.physical_parameters.y', 'leaf_card.physical_parameters.z'])
# Check every selected shader reference to this uniform, not a product shader invocation.
uses = [line.strip() for line in shader.splitlines() if 'physical_parameters' in line]
assert uses == ['physical_parameters: vec4<f32>,',
                '// leaves physical_parameters.z at zero, so its vertex color remains free',
                'albedo = mix(albedo, in.color.rgb, leaf_card.physical_parameters.z);',
                'pbr_input.material.thickness = leaf_card.physical_parameters.y;']
factory = sources['crates/adventuresim-tactical-client/src/presentation/obstacles/tree/materials.rs']
must(factory, ['arm: textures.arm.clone()', 'physical_parameters: Vec4::new(0.86, 0.001, 0.0, 0.0)'])
litter = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/litter.rs']
must(litter, ['leaf_material(&assets.dry_oak_leaf, 0.28, 0.72, 0.0, 0.035)',
              'material.physical_parameters.x = 0.96;'])
scatter = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/mod.rs']
must(scatter, ['leaf_materials.add(forest_floor_leaf_material(procedural_assets))',
               'dry_leaf_material,'])
batches = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/litter/batching.rs']
must(batches, ['MeshMaterial3d(assets.dry_leaf_material.clone())'])
bake = sources['crates/adventuresim-procedural-textures/src/leaf/baking.rs']
must(bake, ['maps.arm.extend_from_slice(&[',
            '(f32::from(colors.roughness) * leaf.relief.roughness_gain).round() as u8'])
foliage = sources['crates/adventuresim-procedural-textures/src/foliage.rs']
must(foliage, ['DryWhiteOak => params.leaf_colors.dry_white_oak', 'TextureFormat::Rgba8Unorm'])
pixels = sources['crates/adventuresim-procedural-textures/src/leaf_pixels.rs']
must(pixels, ['arm: images.add(leaf_mipped_image(', 'LeafMipSemantic::Scalar'])
handles = sources['crates/adventuresim-procedural-textures/src/runtime/handles.rs']
must(handles, ['dry_oak_leaf: LeafTextureSet::load(residency, images, DryWhiteOakLeaf)',
               'arm: load_map(residency, images, recipe, MapChannel::Arm)'])
studio = sources['crates/adventuresim-texture-studio/src/scene/special.rs']
must(studio, ['arm: map(MapChannel::Arm)', 'document.surface.roughness'])
shrub = sources['assets/shaders/tactical_shrub_leaf_instanced.wgsl']
must(shrub, ['fast foliage model', 'or specular', 'leaf.physical_parameters.z'])
assert len(doc['search_evidence']) == 2
for search in doc['search_evidence']:
    raw = (root / search['path']).read_bytes()
    assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
    assert all(line.startswith(revision + ':') for line in raw.decode().splitlines())
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 6
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any('Physicalparameters.xroughness documentation notreadbyshader' in x
           for x in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
part = support['parallel-l13-20261029/READ-PL13-229-PARTIAL-001.json'][1]
assert part['rows'][0]['fresh_personal_assessed_ranges'] == [[1, 460]]
assert part['registered_unread_originals'] == [1057]
union = old['evidence']['partial_read_union'][0]
assert union['previous_personal_ranges'] == [[1, 460]]
assert union['new_personal_ranges'] == [[461, 517]] and union['union'] == [[1, 517]]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['discovery_receipts'][226]
assert prior['path'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['sha256'] == doc['recovered_historical_receipts'][0]['sha256']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_authority_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected leaf material source and 6 historical records; no product ran')
