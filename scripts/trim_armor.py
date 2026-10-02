"""Bake parametric, unlit plate-edge colors into the material atlas.

python scripts/trim_armor.py DIRECTORY --recipe assets_src/equipment/armor-finishes.json
Widths are reference-body millimetres. Pattern repeats follow each authored rim.
Optional bands use object-space slabs, independent of UV charts. Their axis
sets the slab normal and position is normalized across the metal piece bounds.
The phase axis defaults to Y for X/Z bands, and X for Y bands. Bands paint over
rim trim in list order and inherit its style controls unless overridden.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image
from armor_glb import Asset, retains_body_uvs
from armor_bake_math import raster_triangles, dilate
from armor_trim_math import boundary_field, pattern_mask, surface_band_mask

STYLE_FIELDS = {'pattern', 'color', 'width_mm', 'repeats', 'metallic', 'roughness'}


def color(value):
    if not isinstance(value, str) or len(value) != 7 or value[0] != '#':
        raise ValueError('trim color must be #RRGGBB')
    return np.array([int(value[i:i+2], 16) for i in (1, 3, 5)]) / 255


def validate_style(recipe, maximum_width=30):
    if recipe['pattern'] not in ('none', 'plain', 'double', 'chevron', 'scallop', 'vine'):
        raise ValueError('unknown trim pattern')
    for field in ['width_mm', 'repeats', 'metallic', 'roughness']:
        if isinstance(recipe[field], bool) or not isinstance(recipe[field], (float, int)) or not np.isfinite(recipe[field]):
            raise ValueError(f'trim {field} must be a finite number')
    if not 1 <= recipe['width_mm'] <= maximum_width or not 1 <= recipe['repeats'] <= 128:
        raise ValueError(f'trim width must be 1..{maximum_width} mm and repeats 1..128')
    if not isinstance(recipe['repeats'], int):
        raise ValueError('trim repeat count must be an integer')
    for field in ['metallic', 'roughness']:
        if not 0 <= recipe[field] <= 1:
            raise ValueError(f'trim {field} must be 0..1')
    color(recipe['color'])


def band_styles(recipe):
    style = {field: recipe[field] for field in STYLE_FIELDS}
    return [{**style, **band} for band in recipe.get('bands', [])]


def validate(recipe):
    if not isinstance(recipe, dict) or not STYLE_FIELDS <= set(recipe) or set(recipe) - STYLE_FIELDS - {'bands'}:
        raise ValueError('finish requires style controls and optional bands')
    validate_style(recipe)
    if not isinstance(recipe.get('bands', []), list):
        raise ValueError('finish bands must be a list')
    for band in recipe.get('bands', []):
        if not isinstance(band, dict) or not {'axis', 'position'} <= set(band) or set(band) - STYLE_FIELDS - {'axis', 'position', 'phase_axis'}:
            raise ValueError('band requires axis, position, and optional style/phase_axis controls')
        if band['axis'] not in ('x', 'y', 'z') or band.get('phase_axis', 'y' if band['axis'] != 'y' else 'x') not in ('x', 'y', 'z'):
            raise ValueError('band axes must be x, y, or z')
        if band.get('phase_axis') == band['axis']:
            raise ValueError('band phase_axis must differ from its width axis')
        value = band['position']
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not np.isfinite(value) or not 0 <= value <= 1:
            raise ValueError('band position must be finite and normalized to 0..1')
    for band in band_styles(recipe):
        validate_style(band, maximum_width=100)


def plate_primitives(asset):
    for mesh in asset.doc['meshes']:
        for primitive in mesh['primitives']:
            material = asset.doc['materials'][primitive['material']]
            if primitive.get('extras', {}).get('adventuresim_plate_edges') and material.get('pbrMetallicRoughness', {}).get('metallicFactor', 0) >= .8:
                yield primitive, material


def edge_mask(points, point, face_component, field, limits, recipe):
    edges, starts, lengths, edge_components, _ = field
    width = recipe['width_mm'] / 1000
    lower, upper = limits
    candidate = np.flatnonzero((edge_components == face_component) &
        (lower <= points.max(axis=0)).all(axis=1) & (upper >= points.min(axis=0)).all(axis=1))
    if not len(candidate):
        return np.zeros(len(point), bool)
    start = edges[candidate, 0]; delta = edges[candidate, 1] - start
    projection = np.clip(((point[:, None] - start) * delta).sum(axis=2) / (delta * delta).sum(axis=1), 0, 1)
    distance = np.linalg.norm(point[:, None] - (start + projection[:, :, None] * delta), axis=2)
    nearest = distance.argmin(axis=1); row = np.arange(len(point)); selected = candidate[nearest]
    phase = (starts[selected] + projection[row, nearest] * lengths[selected]) * recipe['repeats']
    return pattern_mask(distance[row, nearest], phase, width, recipe['pattern'])


def save_texture(asset, pixels, directory, channel):
    temporary = directory / f'armor-{channel}.png'
    Image.fromarray(np.uint8(np.clip(pixels, 0, 1) * 255 + .5)).save(temporary)
    filename = f'armor-{channel}-{hashlib.sha256(temporary.read_bytes()).hexdigest()}.png'
    temporary.replace(directory / filename)
    images, textures = asset.doc.setdefault('images', []), asset.doc.setdefault('textures', [])
    images.append({'uri': filename, 'mimeType': 'image/png'})
    textures.append({'source': len(images)-1})
    return {'index': len(textures)-1, 'texCoord': 0}


def process(path, recipe, resolution):
    validate(recipe)
    if retains_body_uvs(path):
        return 0
    asset = Asset(path); count = 0
    plates = list(plate_primitives(asset))
    if not plates:
        return 0
    points = np.concatenate([asset.array(p['attributes']['POSITION']) for p, _ in plates])
    bounds = points.min(axis=0), points.max(axis=0)
    bands = [band for band in band_styles(recipe) if band['pattern'] != 'none']
    for primitive, material in plates:
        extras = primitive.get('extras', {})
        source = extras.get('adventuresim_plate_edges')
        if extras.get('adventuresim_material_uv', {}).get('channel') != 0:
            raise ValueError(f'unwrap {path.name} before trimming')
        if 'adventuresim_trim' in extras:
            raise ValueError(f'regenerate {path.name} before changing its finish')
        if recipe['pattern'] == 'none' and not bands:
            continue
        if source['space'] != 'reference_body' or source['units'] != 'metres':
            raise ValueError('unsupported plate boundary coordinate space')
        a = primitive['attributes']; positions = asset.array(a['POSITION'])
        faces = asset.array(primitive['indices']).reshape(-1, 3); uv = asset.array(a['TEXCOORD_0'])
        field = boundary_field(positions, faces, source['segments']) if recipe['pattern'] != 'none' else None
        limits = (field[0].min(axis=1) - recipe['width_mm'] / 1000,
                  field[0].max(axis=1) + recipe['width_mm'] / 1000) if field is not None else None
        material = copy.deepcopy(material); pbr = material.setdefault('pbrMetallicRoughness', {})
        if 'baseColorTexture' in pbr:
            raise ValueError('plate finish requires an unpainted base material')
        base = np.array(pbr.get('baseColorFactor', [1, 1, 1, 1]))
        srgb = np.where(base[:3] <= .0031308, base[:3] * 12.92, 1.055 * base[:3] ** (1/2.4) - .055)
        pixels = np.ones((resolution, resolution, 4), np.float32); pixels[:, :, :3] = srgb; pixels[:, :, 3] = base[3]
        mr = np.ones((resolution, resolution, 3), np.float32)
        mr[:, :, 1] = pbr.get('roughnessFactor', 1); mr[:, :, 2] = pbr.get('metallicFactor', 1)
        covered = np.zeros((resolution, resolution), bool); painted = 0
        face_index = {tuple(face): i for i, face in enumerate(faces)}
        for face, y, x, weights in raster_triangles(uv, faces, resolution):
            covered[y, x] = True
            points = positions[face]
            if not len(x): continue
            point = weights @ points
            layers = []
            if field is not None:
                layers.append((recipe, edge_mask(points, point, field[4][face_index[tuple(face)]], field, limits, recipe)))
            layers.extend((band, surface_band_mask(point, bounds, band)) for band in bands)
            combined = np.zeros(len(point), bool)
            for style, mask in layers:
                pixels[y[mask], x[mask], :3] = color(style['color'])
                mr[y[mask], x[mask], 1] = style['roughness']; mr[y[mask], x[mask], 2] = style['metallic']
                combined |= mask
            painted += int(combined.sum())
        if not painted:
            continue
        pbr['baseColorFactor'] = [1, 1, 1, 1]; pbr['metallicFactor'] = 1; pbr['roughnessFactor'] = 1
        pbr['baseColorTexture'] = save_texture(asset, dilate(pixels, covered.copy()), path.parent, 'color')
        pbr['metallicRoughnessTexture'] = save_texture(asset, dilate(mr, covered.copy()), path.parent, 'metal-rough')
        primitive['material'] = len(asset.doc['materials']); asset.doc['materials'].append(material)
        extras['adventuresim_trim'] = {**recipe, 'resolution': resolution, 'painted_texels': painted}
        count += 1
    if count: asset.write(path)
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--recipe', type=Path, default=Path('assets_src/equipment/armor-finishes.json'))
    parser.add_argument('--resolution', type=int, choices=[512, 1024, 2048], default=1024)
    args = parser.parse_args()
    recipes = json.loads(args.recipe.read_text())
    for path in sorted(args.directory.glob('*.glb')):
        recipe = {**recipes['defaults'], **recipes.get('items', {}).get(path.stem.split('--')[0], {})}
        print(path.name, process(path, recipe, args.resolution), flush=True)


if __name__ == '__main__': main()
