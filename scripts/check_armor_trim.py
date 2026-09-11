"""Audit texture-only armor finishing and preservation of geometry/shading data."""
import argparse
from pathlib import Path
import numpy as np
from PIL import Image
from armor_glb import Asset


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
                    assert len(colors)==2, f'albedo must contain only base steel and trim colors: {len(colors)}'
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
