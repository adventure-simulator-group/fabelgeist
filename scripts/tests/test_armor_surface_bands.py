"""Exercise object-space painting through genuinely separate GLB UV charts."""
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_bake_math import raster_triangles
from armor_glb import Asset
from armor_trim_math import surface_band_mask
from check_armor_trim import audit, texture_uri
from trim_armor import band_styles, process, validate


STYLE = dict(pattern='none', color='#C7A45B', width_mm=6, repeats=8,
             metallic=1, roughness=.3)


def fixture(path, metallic=1):
    asset = Asset.__new__(Asset)
    asset.binary = bytearray()
    asset.doc = dict(asset={'version': '2.0'}, buffers=[{}], bufferViews=[],
                     accessors=[], meshes=[], materials=[], images=[], textures=[])
    points = np.array([[0, 0, 0], [.1, 0, 0], [.1, .1, 0], [0, .1, 0],
                       [0, .1, 0], [.1, .1, 0], [.1, .2, 0], [0, .2, 0]])
    uv = np.array([[.05, .05], [.45, .05], [.45, .95], [.05, .95],
                   [.55, .05], [.95, .05], [.95, .95], [.55, .95]])
    faces = np.array([[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]])
    edges = points[[[0, 1], [1, 2], [2, 6], [6, 7], [7, 3], [3, 0]]]
    for name, pixel in [('normal', [128, 128, 255]), ('ao', [191, 191, 191])]:
        Image.new('RGB', (2, 2), tuple(pixel)).save(path.parent / f'{name}.png')
        asset.doc['textures'].append({'source': len(asset.doc['images'])})
        asset.doc['images'].append({'uri': f'{name}.png'})
    asset.doc['materials'] = [{
        'pbrMetallicRoughness': {'baseColorFactor': [.12, .12, .12, 1],
                                'metallicFactor': metallic, 'roughnessFactor': .4},
        'normalTexture': {'index': 0, 'texCoord': 0},
        'occlusionTexture': {'index': 1, 'texCoord': 0, 'strength': .75}}]
    primitive = {'attributes': {'POSITION': asset.append(points),
                                 'TEXCOORD_0': asset.append(uv)},
                 'indices': asset.append(faces.flatten(), component=5125),
                 'material': 0, 'extras': {
                     'adventuresim_material_uv': {'channel': 0},
                     'adventuresim_plate_edges': {'space': 'reference_body',
                         'units': 'metres', 'segments': edges.tolist()}}}
    asset.doc['meshes'] = [{'primitives': [primitive]}]
    asset.write(path)


class SurfaceBandTests(unittest.TestCase):
    def test_actual_uv_charts_paint_one_continuous_physical_band(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            before, after = directory / 'before.glb', directory / 'plate.glb'
            fixture(before)
            after.write_bytes(before.read_bytes())
            original_maps = {name: hashlib.sha256((directory / name).read_bytes()).hexdigest()
                             for name in ['normal.png', 'ao.png']}
            recipe = {**STYLE, 'bands': [{'axis': 'x', 'position': .5,
                                        'width_mm': 20, 'pattern': 'plain'}]}
            self.assertEqual(process(after, recipe, 128), 1)
            self.assertEqual(audit(before, after), 1)
            asset = Asset(after); primitive = asset.doc['meshes'][0]['primitives'][0]
            material = asset.doc['materials'][primitive['material']]
            image = np.array(Image.open(directory / texture_uri(asset, material['pbrMetallicRoughness']['baseColorTexture'])))
            positions = asset.array(primitive['attributes']['POSITION'])
            uv = asset.array(primitive['attributes']['TEXCOORD_0'])
            faces = asset.array(primitive['indices']).reshape(-1, 3)
            painted_by_chart = [0, 0]
            for face, y, x, weights in raster_triangles(uv, faces, 128):
                point = weights @ positions[face]
                expected = np.abs(point[:, 0] - .05) < .01
                actual = (image[y, x, :3] == [199, 164, 91]).all(axis=1)
                np.testing.assert_array_equal(actual, expected)
                painted_by_chart[int(face[0] >= 4)] += int(actual.sum())
            self.assertTrue(all(painted_by_chart))
            for name, digest in original_maps.items():
                self.assertEqual(hashlib.sha256((directory / name).read_bytes()).hexdigest(), digest)

    def test_pattern_phase_is_translation_invariant_and_width_is_physical(self):
        band = band_styles({**STYLE, 'bands': [{'axis': 'x', 'position': .5,
                                               'pattern': 'vine', 'width_mm': 20}]})[0]
        lower = np.array([0., 0., 0.]); upper = np.array([.1, .2, .05])
        points = np.array([[x, y, 0] for x in np.linspace(.04, .06, 40)
                           for y in np.linspace(.095, .105, 40)])
        mask = surface_band_mask(points, (lower, upper), band)
        self.assertTrue(mask.any()); self.assertFalse(mask.all())
        offset = np.array([.3, 1., -.8])
        np.testing.assert_array_equal(mask, surface_band_mask(points + offset, (lower + offset, upper + offset), band))
        plain = {**band, 'pattern': 'plain'}
        sample = np.array([[.041, .1, 0], [.059, .1, 0]])
        self.assertTrue(surface_band_mask(sample, (lower, upper), plain).all())
        self.assertFalse(surface_band_mask(sample * 2, (lower * 2, upper * 2), plain).any())
        for axis in 'xyz':
            validate({**STYLE, 'bands': [{'axis': axis, 'position': .5}]})

    def test_invalid_band_recipe_rejected_before_writing(self):
        bad = [{'axis': 'q', 'position': .5}, {'axis': 'x'},
               {'axis': 'x', 'position': float('nan')}, {'axis': 'x', 'position': True},
               {'axis': 'x', 'position': 1.1}, {'axis': 'x', 'position': .5, 'phase_axis': 'x'},
               {'axis': 'x', 'position': .5, 'width_mm': 101},
               {'axis': 'x', 'position': .5, 'roughness': float('inf')},
               {'axis': 'x', 'position': .5, 'color': 'gold'},
               {'axis': 'x', 'position': .5, 'repeats': True},
               {'axis': 'x', 'position': .5, 'unknown': 1}, 'x']
        for band in bad:
            with self.subTest(band=band), self.assertRaises(ValueError):
                validate({**STYLE, 'bands': [band]})
        with self.assertRaises(ValueError): validate({**STYLE, 'bands': {}})

    def test_overlapping_bands_have_authored_order_and_pbr_overrides(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            before, after = directory / 'before.glb', directory / 'plate.glb'
            fixture(before); after.write_bytes(before.read_bytes())
            recipe = {**STYLE, 'bands': [
                {'axis': 'x', 'position': .5, 'width_mm': 40, 'pattern': 'plain'},
                {'axis': 'x', 'position': .5, 'width_mm': 10, 'pattern': 'plain',
                 'color': '#FF0000', 'roughness': .8, 'metallic': .6}]}
            process(after, recipe, 128)
            self.assertEqual(audit(before, after), 1)
            asset = Asset(after); primitive = asset.doc['meshes'][0]['primitives'][0]
            pbr = asset.doc['materials'][primitive['material']]['pbrMetallicRoughness']
            image = np.array(Image.open(directory / texture_uri(asset, pbr['baseColorTexture'])))
            mr = np.array(Image.open(directory / texture_uri(asset, pbr['metallicRoughnessTexture'])))
            red = (image[:, :, :3] == [255, 0, 0]).all(axis=2)
            gold = (image[:, :, :3] == [199, 164, 91]).all(axis=2)
            self.assertTrue(red.any()); self.assertTrue(gold.any())
            self.assertTrue((mr[red, 1] == 204).all())
            self.assertTrue((mr[red, 2] == 153).all())

    def test_empty_bands_preserve_edge_trim_output_and_skips(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            paths = [directory / name for name in ['first.glb', 'second.glb']]
            for path in paths: fixture(path)
            recipe = {**STYLE, 'pattern': 'plain'}
            process(paths[0], recipe, 64)
            process(paths[1], {**recipe, 'bands': []}, 64)
            materials = []
            for path in paths:
                asset = Asset(path); primitive = asset.doc['meshes'][0]['primitives'][0]
                materials.append(asset.doc['materials'][primitive['material']])
            self.assertEqual(materials[0], materials[1])
            for name, metallic in [('mail_coif.glb', 1), ('leather.glb', 0)]:
                path = directory / name; fixture(path, metallic)
                original = path.read_bytes()
                self.assertEqual(process(path, {**recipe, 'bands': [{'axis': 'x', 'position': .5}]}, 64), 0)
                self.assertEqual(path.read_bytes(), original)


if __name__ == '__main__': unittest.main()
