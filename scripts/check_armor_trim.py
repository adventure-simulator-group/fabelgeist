"""Audit texture-only armor finishing and preservation of geometry/shading data."""
import argparse
from pathlib import Path
import numpy as np
from PIL import Image
from armor_glb import Asset
from trim_armor import band_styles, color


def texture_uri(asset, texture):
    return asset.doc['images'][asset.doc['textures'][texture['index']]['source']]['uri']


def audit(source, finished):
    before, after = Asset(source), Asset(finished)
    for key in ['nodes','scenes','skins','extras']:
        assert before.doc.get(key) == after.doc.get(key), f'changed {key}'
    assert len(before.doc['meshes']) == len(after.doc['meshes'])
    count = 0
    for old_mesh,new_mesh in zip(before.doc['meshes'],after.doc['meshes']):
        assert len(old_mesh['primitives']) == len(new_mesh['primitives'])
        for old,new in zip(old_mesh['primitives'],new_mesh['primitives']):
            np.testing.assert_array_equal(before.array(old['indices']),after.array(new['indices']))
            assert old['attributes'].keys() == new['attributes'].keys()
            for name,index in old['attributes'].items():
                np.testing.assert_array_equal(before.array(index),after.array(new['attributes'][name]))
            assert len(old.get('targets',[])) == len(new.get('targets',[]))
            for a,b in zip(old.get('targets',[]),new.get('targets',[])):
                assert a.keys() == b.keys()
                for name,index in a.items(): np.testing.assert_array_equal(before.array(index),after.array(b[name]))
            previous = before.doc['materials'][old['material']]
            material = after.doc['materials'][new['material']]
            trim = new.get('extras',{}).get('adventuresim_trim')
            if not trim:
                assert previous == material, 'untrimmed material changed'
                continue
            for channel in ['normalTexture','occlusionTexture']:
                assert texture_uri(before,previous[channel]) == texture_uri(after,material[channel]), f'changed {channel}'
            assert trim['painted_texels'] > 0, 'empty trim'
            pbr=material['pbrMetallicRoughness']
            for channel in ['baseColorTexture','metallicRoughnessTexture']:
                info=pbr[channel]; assert info['texCoord']==0
                image=np.asarray(Image.open(finished.parent/texture_uri(after,info)).convert('RGB'))
                assert image.shape[:2] == (trim['resolution'],trim['resolution'])
                if channel == 'baseColorTexture':
                    # Colors are unlit palette entries, never AO or highlight ramps.
                    colors=np.unique(image.reshape(-1,3),axis=0)
                    base = np.array(previous.get('pbrMetallicRoughness', {}).get('baseColorFactor', [1, 1, 1, 1]))[:3]
                    base = np.where(base <= .0031308, base * 12.92, 1.055 * base ** (1/2.4) - .055)
                    styles = [trim, *band_styles(trim)]
                    palette = [base, *(color(style['color']) for style in styles if style['pattern'] != 'none')]
                    allowed = {tuple(np.uint8(np.clip(value, 0, 1) * 255 + .5)) for value in palette}
                    assert all(tuple(value) in allowed for value in colors), 'albedo contains colors outside its unlit palette'
            assert pbr['baseColorFactor']==[1,1,1,1]
            assert pbr['metallicFactor']==pbr['roughnessFactor']==1
            count+=1
    return count


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('finished',type=Path)
    args=parser.parse_args();total=0
    paths=sorted(args.finished.glob('*.glb'));assert paths,'no assets'
    for path in paths:
        total+=audit(args.source/path.name,path)
        print(path.name,'PASS',flush=True)
    print(total,'trimmed components')


if __name__=='__main__': main()
