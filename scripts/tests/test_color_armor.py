"""Material changes preserve mesh payloads and compose with texture trim."""
import copy
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_glb import Asset
from check_armor_trim import audit
from color_armor import process, validate
from test_armor_surface_bands import fixture, STYLE
from trim_armor import process as trim


MATERIAL = dict(color='#5A6165', metallic=1, roughness=.43)


class MetalColorTests(unittest.TestCase):
    def test_unlit_material_preserves_payload_maps_and_nonmetal(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'breastplate--worn.glb'
            fixture(path)
            before = Asset(path)
            leather = copy.deepcopy(before.doc['materials'][0])
            leather['pbrMetallicRoughness']['metallicFactor'] = 0
            before.doc['materials'].append(leather)
            extra = copy.deepcopy(before.doc['meshes'][0]['primitives'][0])
            extra['material'] = 1
            before.doc['meshes'][0]['primitives'].append(extra)
            before.write(path)
            before = Asset(path)
            self.assertEqual(process(path, MATERIAL), 1)
            after = Asset(path)
            self.assertEqual(before.binary, after.binary)
            for key in ('nodes', 'skins', 'accessors', 'bufferViews', 'images', 'textures'):
                self.assertEqual(before.doc.get(key), after.doc.get(key))
            for old, new in zip(before.doc['meshes'][0]['primitives'], after.doc['meshes'][0]['primitives']):
                self.assertEqual({k: v for k, v in old.items() if k != 'material'},
                                 {k: v for k, v in new.items() if k != 'material'})
            updated = after.doc['materials'][after.doc['meshes'][0]['primitives'][0]['material']]
            self.assertEqual(updated['normalTexture'], before.doc['materials'][0]['normalTexture'])
            self.assertEqual(updated['occlusionTexture'], before.doc['materials'][0]['occlusionTexture'])
            self.assertEqual(after.doc['materials'][1], leather)
            rgb = np.array([90, 97, 101]) / 255
            np.testing.assert_allclose(updated['pbrMetallicRoughness']['baseColorFactor'][:3],
                                       ((rgb + .055) / 1.055) ** 2.4)
            self.assertEqual(updated['pbrMetallicRoughness']['roughnessFactor'], .43)
            self.assertNotIn('baseColorTexture', updated['pbrMetallicRoughness'])

    def test_colored_base_composes_with_trim_and_rejects_wrong_order(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'breastplate--worn.glb'
            fixture(path)
            process(path, MATERIAL)
            original = path.with_name('colored.glb')
            original.write_bytes(path.read_bytes())
            self.assertEqual(trim(path, {**STYLE, 'pattern': 'plain'}, 128), 1)
            self.assertEqual(audit(original, path), 1)
            payload = path.read_bytes()
            with self.assertRaises(ValueError):
                process(path, MATERIAL)
            self.assertEqual(path.read_bytes(), payload)

    def test_base_metal_classification_cannot_be_lost_before_trim(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'breastplate--worn.glb'
            fixture(path)
            payload = path.read_bytes()
            with self.assertRaises(ValueError):
                process(path, {**MATERIAL, 'metallic': .5})
            self.assertEqual(path.read_bytes(), payload)
            self.assertEqual(process(path, {**MATERIAL, 'metallic': .8}), 1)
            self.assertEqual(process(path, MATERIAL), 1)
            self.assertEqual(trim(path, {**STYLE, 'pattern': 'plain'}, 128), 1)

    def test_mail_and_padding_are_unchanged_and_recipe_is_strict(self):
        with tempfile.TemporaryDirectory() as directory:
            for item in ('mail_coif', 'arming_doublet', 'padded_chausses'):
                path = Path(directory) / f'{item}--worn.glb'
                fixture(path)
                before = path.read_bytes()
                self.assertEqual(process(path, MATERIAL), 0)
                self.assertEqual(path.read_bytes(), before)
        for invalid in ({**MATERIAL, 'color': '#gg0000'}, {**MATERIAL, 'roughness': True},
                        {**MATERIAL, 'metallic': float('nan')}, {**MATERIAL, 'extra': 0}):
            with self.assertRaises(ValueError):
                validate(invalid)


if __name__ == '__main__':
    unittest.main()
