"""Bake parametric, unlit plate-edge colors into the material atlas.

python scripts/trim_armor.py DIRECTORY --recipe assets_src/equipment/armor-finishes.json
Widths are reference-body millimetres. Pattern repeats follow each authored rim.
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
from armor_trim_math import boundary_field, pattern_mask


def color(value):
    if not isinstance(value, str) or len(value) != 7 or value[0] != '#':
        raise ValueError('trim color must be #RRGGBB')
    return np.array([int(value[i:i+2], 16) for i in (1, 3, 5)]) / 255


def validate(recipe):
    if set(recipe) != {'pattern', 'color', 'width_mm', 'repeats', 'metallic', 'roughness'}:
        raise ValueError('finish fields: pattern, color, width_mm, repeats, metallic, roughness')
    if recipe['pattern'] not in {'none', 'plain', 'double', 'chevron', 'scallop', 'vine'}:
        raise ValueError('unknown trim pattern')
    if not 1 <= recipe['width_mm'] <= 30 or not 1 <= recipe['repeats'] <= 128:
        raise ValueError('trim width must be 1..30 mm and repeats 1..128')
    if not isinstance(recipe['repeats'], int):
        raise ValueError('trim repeat count must be an integer')
    for field in ['metallic', 'roughness']:
        if not 0 <= recipe[field] <= 1:
            raise ValueError(f'trim {field} must be 0..1')
    color(recipe['color'])


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
    for mesh in asset.doc['meshes']:
        for primitive in mesh['primitives']:
            extras = primitive.get('extras', {})
            source = extras.get('adventuresim_plate_edges')
            material = asset.doc['materials'][primitive['material']]
            if not source or material.get('pbrMetallicRoughness', {}).get('metallicFactor', 0) < .8:
                continue
            if extras.get('adventuresim_material_uv', {}).get('channel') != 0:
                raise ValueError(f'unwrap {path.name} before trimming')
            if 'adventuresim_trim' in extras:
                raise ValueError(f'regenerate {path.name} before changing its finish')
            if recipe['pattern'] == 'none':
                continue
            if source['space'] != 'reference_body' or source['units'] != 'metres':
                raise ValueError('unsupported plate boundary coordinate space')
            a = primitive['attributes']; positions = asset.array(a['POSITION'])
            faces = asset.array(primitive['indices']).reshape(-1, 3); uv = asset.array(a['TEXCOORD_0'])
            edges, starts, lengths, edge_components, face_components = boundary_field(positions, faces, source['segments'])
            width = recipe['width_mm'] / 1000
            lower = edges.min(axis=1) - width; upper = edges.max(axis=1) + width
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
                candidate = np.flatnonzero((edge_components == face_components[face_index[tuple(face)]]) &
                    (lower <= points.max(axis=0)).all(axis=1) & (upper >= points.min(axis=0)).all(axis=1))
                if not len(candidate) or not len(x): continue
                point = weights @ points
                start = edges[candidate, 0]; delta = edges[candidate, 1] - start
                projection = np.clip(((point[:, None] - start) * delta).sum(axis=2) / (delta * delta).sum(axis=1), 0, 1)
                distance = np.linalg.norm(point[:, None] - (start + projection[:, :, None] * delta), axis=2)
                nearest = distance.argmin(axis=1); row = np.arange(len(point)); selected = candidate[nearest]
                phase = (starts[selected] + projection[row, nearest] * lengths[selected]) * recipe['repeats']
                mask = pattern_mask(distance[row, nearest], phase, width, recipe['pattern'])
                pixels[y[mask], x[mask], :3] = color(recipe['color'])
                mr[y[mask], x[mask], 1] = recipe['roughness']; mr[y[mask], x[mask], 2] = recipe['metallic']
                painted += int(mask.sum())
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
