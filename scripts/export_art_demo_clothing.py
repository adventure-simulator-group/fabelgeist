"""Package native textile-review output as a static, body-visible museum GLB.

python scripts/export_art_demo_clothing.py REVIEW_DIRECTORY OUTPUT.glb

REVIEW_DIRECTORY comes from the creator's textile_outfit_review example. Keep
its canonical Y-up metre geometry, normals, fabric coordinates and PNGs intact.
Only the generated pose is displayed; skeletons and morph targets are omitted.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import numpy as np

from armor_glb import Asset


class DisplayAsset(Asset):
    """A single-buffer display scene using the existing lossless GLB writer."""

    def __init__(self):
        self.binary = bytearray()
        self.doc = {
            'asset': {'version': '2.0', 'generator': 'Fabelgeist clothing display packager'},
            'scene': 0, 'scenes': [{'nodes': []}], 'nodes': [], 'meshes': [],
            'materials': [], 'textures': [], 'images': [], 'samplers': [],
            'buffers': [{'byteLength': 0}], 'bufferViews': [], 'accessors': [],
        }

    def texture(self, path, boundary):
        wrap_u = {'Open': 33071, 'CyclicAcross': 10497}[boundary]
        self.binary.extend(b'\0' * (-len(self.binary) % 4))
        payload = path.read_bytes()
        view = len(self.doc['bufferViews'])
        self.doc['bufferViews'].append({
            'buffer': 0, 'byteOffset': len(self.binary), 'byteLength': len(payload),
        })
        self.binary.extend(payload)
        image = len(self.doc['images'])
        self.doc['images'].append({'bufferView': view, 'mimeType': 'image/png'})
        sampler = len(self.doc['samplers'])
        self.doc['samplers'].append({'wrapS': wrap_u, 'wrapT': 33071})
        index = len(self.doc['textures'])
        self.doc['textures'].append({'source': image, 'sampler': sampler})
        return {'index': index}

    def mesh(self, name, positions, normals, indices, material, texcoords=None):
        positions = np.asarray(positions, dtype=np.float32)
        normals = np.asarray(normals, dtype=np.float32)
        indices = np.asarray(indices, dtype=np.uint32).reshape(-1)
        if positions.shape != normals.shape or positions.ndim != 2 or positions.shape[1] != 3:
            raise ValueError(f'{name}: positions and normals must be matching VEC3 arrays')
        if not np.isfinite(positions).all() or not np.isfinite(normals).all():
            raise ValueError(f'{name}: nonfinite geometry')
        if not len(indices) or len(indices) % 3 or indices.max() >= len(positions):
            raise ValueError(f'{name}: invalid triangle indices')
        position = self.append(positions)
        self.doc['accessors'][position].update(
            min=positions.min(axis=0).tolist(), max=positions.max(axis=0).tolist())
        attributes = {'POSITION': position, 'NORMAL': self.append(normals)}
        if texcoords is not None:
            uv = np.asarray(texcoords, dtype=np.float32)
            if uv.shape != (len(positions), 2) or not np.isfinite(uv).all():
                raise ValueError(f'{name}: invalid fabric coordinates')
            attributes['TEXCOORD_0'] = self.append(uv)
        primitive = {'attributes': attributes, 'indices': self.append(indices, component=5125),
                     'material': len(self.doc['materials'])}
        self.doc['materials'].append(material)
        mesh = len(self.doc['meshes'])
        self.doc['meshes'].append({'name': name, 'primitives': [primitive]})
        node = len(self.doc['nodes'])
        self.doc['nodes'].append({'name': name, 'mesh': mesh})
        self.doc['scenes'][0]['nodes'].append(node)


def linear_color(srgb):
    return [v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4
            for v in srgb[:3]] + [srgb[3]]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('review', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    display = DisplayAsset()
    sources = []

    def source(path):
        payload = path.read_bytes()
        sources.append({'file': path.name, 'sha256': hashlib.sha256(payload).hexdigest()})
        return payload

    body = json.loads(source(args.review/'body.json'))
    display.mesh('Reference wearer', body['positions'], body['normals'], body['faces'], {
        'name': 'Matte display wearer',
        'pbrMetallicRoughness': {'baseColorFactor': linear_color([0.29, 0.17, 0.12, 1]),
                               'metallicFactor': 0, 'roughnessFactor': 0.8},
    })
    outfit = json.loads(source(args.review/'outfit.json'))
    articles = []
    for article_index, _ in enumerate(outfit):
        article = json.loads(source(args.review/f'article-{article_index}.json'))
        triangles = len(article['indices']) // 3
        if triangles > 10_000:
            raise ValueError(f'article {article_index} exceeds the 10,000-triangle budget')
        maps = {row['component']: row for row in article['textile_maps']}
        for component_index, component in enumerate(article['components']):
            begin, end = component['vertices']['start'], component['vertices']['end']
            first, last = component['indices']['start'], component['indices']['end']
            native = component['material']
            pbr = {'baseColorFactor': linear_color(native['base_color']),
                   'metallicFactor': native['metallic'], 'roughnessFactor': native['roughness']}
            material = {'name': component['role'], 'pbrMetallicRoughness': pbr,
                        'doubleSided': False,
                        'alphaMode': 'BLEND' if native['base_color'][3] < 1 else 'OPAQUE'}
            if component_index in maps:
                row = maps[component_index]
                for field, target in [('base_color', pbr), ('normal', material)]:
                    image = args.review/row[field]
                    source(image)
                    target['baseColorTexture' if field == 'base_color' else 'normalTexture'] = (
                        display.texture(image, row['chart_boundary']))
                pbr['baseColorFactor'] = [1, 1, 1, 1]
            display.mesh(f"article-{article_index}/{component['role']}",
                         article['positions'][begin:end], article['normals'][begin:end],
                         [value - begin for value in article['indices'][first:last]], material,
                         article['texcoords'][begin:end])
        articles.append({'generator_version': article['generator_version'], 'triangles': triangles})
    display.write(args.output)
    revision = subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=Path(__file__).resolve().parents[1], text=True).strip()
    provenance = {'source_revision': revision, 'scope': 'Static canonical generated pose; no animation',
                  'body_triangles': len(body['faces']), 'articles': articles,
                  'outfit': outfit, 'inputs': sources,
                  'output_sha256': hashlib.sha256(args.output.read_bytes()).hexdigest(),
                  'geometry': 'Native Y-up metres; positions, normals and indices unchanged',
                  'textures': 'Embedded native color and tangent-normal PNGs, unchanged; native UVs'}
    args.output.with_suffix('.sources.json').write_text(
        json.dumps(provenance, indent=2) + '\n', encoding='utf8')
    print(json.dumps({'output': str(args.output), 'bytes': args.output.stat().st_size,
                      'articles': articles}))


if __name__ == '__main__':
    main()
