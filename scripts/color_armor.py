"""Set unlit metal material factors before applying plate-edge trim.

python scripts/color_armor.py DIRECTORY RECIPE.json
The recipe has color (#RRGGBB in sRGB), metallic (0.8..1), roughness (0..1).
Body-derived mail/padding and nonmetal materials retain their own appearance.
Geometry, skinning, UVs, normal maps and occlusion maps are preserved.
"""
import argparse
import copy
import json
from pathlib import Path

import numpy as np

from armor_glb import Asset, retains_body_uvs
from trim_armor import color


def validate(recipe):
    if not isinstance(recipe, dict) or set(recipe) != {'color', 'metallic', 'roughness'}:
        raise ValueError('metal material requires color, metallic and roughness')
    rgb = color(recipe['color'])
    for field in ('metallic', 'roughness'):
        value = recipe[field]
        # The existing finishing pipeline identifies metal primitives at .8.
        # Keep that classification stable when changing their appearance.
        minimum = .8 if field == 'metallic' else 0
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not np.isfinite(value) or not minimum <= value <= 1:
            raise ValueError(f'material {field} must be finite and within {minimum}..1')
    return np.where(rgb <= .04045, rgb / 12.92, ((rgb + .055) / 1.055) ** 2.4)


def process(path, recipe):
    linear = validate(recipe)
    if retains_body_uvs(path):
        return 0
    asset = Asset(path)
    selected = []
    for mesh in asset.doc['meshes']:
        for primitive in mesh['primitives']:
            material = asset.doc['materials'][primitive['material']]
            pbr = material.get('pbrMetallicRoughness', {})
            if 'adventuresim_trim' in primitive.get('extras', {}):
                raise ValueError('set the base material before applying trim to fresh exports')
            if pbr.get('metallicFactor', 0) < .8:
                continue
            if 'baseColorTexture' in pbr or 'metallicRoughnessTexture' in pbr:
                raise ValueError('metal base material must not already be painted')
            selected.append((primitive, material))
    for primitive, source in selected:
        material = copy.deepcopy(source)
        pbr = material['pbrMetallicRoughness']
        alpha = pbr.get('baseColorFactor', [1, 1, 1, 1])[3]
        pbr.update(baseColorFactor=[*linear.tolist(), alpha],
                   metallicFactor=recipe['metallic'], roughnessFactor=recipe['roughness'])
        primitive['material'] = len(asset.doc['materials'])
        asset.doc['materials'].append(material)
    if selected:
        asset.write(path)
    return len(selected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('recipe', type=Path)
    args = parser.parse_args()
    recipe = json.loads(args.recipe.read_text())
    validate(recipe)
    paths = sorted(args.directory.glob('*.glb'))
    if not paths:
        parser.error('no equipment GLBs')
    for path in paths:
        print(path.name, process(path, recipe), flush=True)


if __name__ == '__main__':
    main()
