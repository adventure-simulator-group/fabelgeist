#!/usr/bin/env python3
"""Bounded, read-only evidence verification; never executes either verifier."""
import argparse, ast, hashlib, json, os, pathlib, runpy, subprocess
HERE=pathlib.Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def require(ok,msg):
    if not ok: raise ValueError(msg)
def no_symlink_read(base, relative):
    base=base.absolute()
    for p in [base,*base.parents]: require(not p.is_symlink(),'symbolic-link ancestor')
    path=base
    for part in pathlib.Path(relative).parts:
        require(part not in ('..', '.', '/'),'invalid evidence path')
        path=path/part;require(not path.is_symlink(),'symbolic-link evidence path')
    return path.read_bytes()

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--originals-root',type=pathlib.Path)
    ap.add_argument('--publication-root',type=pathlib.Path)
    ap.add_argument('--original-home-prefix',default=str(pathlib.Path.home()));a=ap.parse_args()
    manifest=json.loads((HERE/'MANIFEST.json').read_bytes()); listed={x['path']:x for x in manifest['files']}
    actual=set()
    for dirname, dirs, files in os.walk(HERE,followlinks=False):
        for name in dirs+files: require(not (pathlib.Path(dirname)/name).is_symlink(),'symbolic link in packet')
        for name in files:
            rel=(pathlib.Path(dirname)/name).relative_to(HERE).as_posix()
            if rel!='MANIFEST.json':actual.add(rel)
    require(actual==set(listed),'manifest membership mismatch')
    for name,r in listed.items():
        p=HERE/name;require(not p.is_symlink(),'unexpected symbolic link');b=p.read_bytes()
        require((len(b),sha(b))==(r['bytes'],r['sha256']),'manifest member mismatch')
    build=runpy.run_path(str(HERE/'build_successor.py'),run_name='evidence_build_module')
    source=(HERE/'ORIGINAL-verify_recovered.py').read_bytes();new=(HERE/'verify_recovered_public_redacted.py').read_bytes()
    require(sha(source)==build['SOURCE_SHA'],'original verifier mismatch')
    require(build['transform'](source)==new,'narrow deterministic transformation mismatch')
    ast.parse(new.decode('utf-8'))
    transformations=json.loads((HERE/'TRANSFORMATION.json').read_bytes())
    require(transformations['original_sha256']==sha(source) and transformations['successor_sha256']==sha(new),'transformation identities mismatch')
    require(transformations['unique_non_overlapping_anchor_count']==len(build['REPLACEMENTS'])==len(transformations['operations']),'transformation count mismatch')
    old_text=source.decode('utf-8');new_text=new.decode('utf-8');intervals=[]
    for recorded,(old_block,new_block) in zip(transformations['operations'],build['REPLACEMENTS']):
        require(old_text.count(old_block)==new_text.count(new_block)==1,'non-unique transformation anchor')
        oi=old_text.index(old_block);ni=new_text.index(new_block);intervals.append((oi,oi+len(old_block)))
        def line_bounds(full,start,block):
            first=full.count('\n',0,start)+1
            return first,first+block.count('\n')-(1 if block.endswith('\n') else 0)
        require(line_bounds(old_text,oi,old_block)==(recorded['original_line_start'],recorded['original_line_end']),'original anchor lines mismatch')
        require(line_bounds(new_text,ni,new_block)==(recorded['successor_line_start'],recorded['successor_line_end']),'successor anchor lines mismatch')
        require(sha(old_block.encode())==recorded['original_text_sha256'] and sha(new_block.encode())==recorded['successor_text_sha256'],'anchor hashes mismatch')
    ordered=sorted(intervals);require(all(a[1]<=b[0] for a,b in zip(ordered,ordered[1:])),'overlapping original anchors')
    begin=new.index(b'# BEGIN QUALIFIED PUBLIC REDACTIONS');end=new.index(b'# END QUALIFIED PUBLIC REDACTIONS')+len(b'# END QUALIFIED PUBLIC REDACTIONS')
    sample=HERE/'sample-publication';env={'root':sample,'json':json,'hashlib':hashlib,'Path':pathlib.Path}
    exec(compile(new[begin:end],'qualified-evidence-helper','exec'),env)
    validated_manifests={path:json.loads(env['_qualified_historical_manifest'](path)) for path in env['_qualified_manifests']}
    c=json.loads((sample/'recovered/privacy-redaction-evidence/CROSSWALK.json').read_bytes());
    for o in c['objects']:
        orig=o['original']; data=(sample/orig['publication_path']).read_bytes()
        env['_qualified_public_object'](orig['publication_path'],orig['sha256'],data)
        if a.originals_root:
            old=no_symlink_read(a.originals_root,orig['sha256']);require((len(old),sha(old))==(orig['bytes'],orig['sha256']),'private original identity mismatch')
            patches=[]
            for r in o['redactions']:
                s,n=r['original_byte_offset'],r['original_token_bytes'];p,q=r['public_byte_offset'],r['public_token_bytes']
                old_value=json.loads(old[s:s+n]);require(isinstance(old_value,str) and old_value.startswith(a.original_home_prefix.rstrip('/')+'/'),'unexpected private path scope')
                require(json.loads(data[p:p+q])==r['replacement'],'public marker span mismatch');patches.append((s,s+n,data[p:p+q]))
            for s,e,b in sorted(patches,reverse=True):old=old[:s]+b+old[e:]
            require(old==data,'original-to-public bytes do not match')
    require(env['_qualified_seen']==set(env['_qualified_objects']),'missing qualified object')
    l12=validated_manifests['recovered/l12-final-registration-evidence/MANIFEST.json']['prior_read_proof']['direct_dependencies']
    r=l12['references'][67]
    require(len(l12['references'])==72 and l12['unique_objects']==65 and r['sha256']=='0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0' and r['parent_sha256']=='ccebae9e3349d54e4592a3051988dc1ba3e8f4ba077d38b6bcc56eddfd95299d' and r['pointer']=='/validated_bundle','historical lens-12 reference mismatch')
    comp=validated_manifests['recovered/l2-canonical-comparison-evidence/MANIFEST.json'];x='74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48'
    require(comp['inherited_master_sha256']==x and comp['files'][3]['file']==x and comp['files'][3]['sha256']==x,'historical comparison reference mismatch')
    m=validated_manifests['recovered/prior-current-transition-evidence/MANIFEST.json']; y='7ab4700aa59883fb73e3c7ef55ea2c05ed5e13661ee8a5b82dae1732d76188a5'
    require(m['transition_sha256']==y and m['path_count']==154,'historical transition reference mismatch')
    cmpdoc=json.loads((sample/('recovered/l2-canonical-comparison-evidence/'+x)).read_bytes());require(len(cmpdoc['findings'])==173 and cmpdoc['findings'][55]['id']=='M40','historical comparison semantic mismatch')
    trans=json.loads((sample/('recovered/prior-current-transition-evidence/'+y)).read_bytes());require(trans['prior_target']==m['from_revision'] and trans['new_target']==m['to_revision'] and len(trans['rows'])==154,'historical transition semantic mismatch')
    if a.publication_root:
        root=a.publication_root
        require(sha(no_symlink_read(root,'verify_recovered.py'))==build['SOURCE_SHA'],'observed verifier drift')
        for path in env['_qualified_manifests']:
            require(no_symlink_read(root,path)==(sample/path).read_bytes(),'observed historical manifest drift')
        for o in c['objects']:
            b=no_symlink_read(root,o['original']['publication_path']);require(sha(b)==o['original']['sha256'],'observed original object drift')
        parent=no_symlink_read(root,'recovered/l12-final-registration-evidence/prior-read-objects/'+r['parent_sha256']);require(sha(parent)==r['parent_sha256'],'historical parent receipt mismatch')
        node=json.loads(parent)
        for part in r['pointer'].split('/')[1:]:node=node[int(part)] if isinstance(node,list) else node[part]
        require(node[r['path_key']]==r['path'] and node[r['hash_key']]==r['sha256'],'historical parent relationship mismatch')
    print(json.dumps({'status':'pass','original_verifier_sha256':sha(source),'successor_sha256':sha(new),'replacement_anchors':len(build['REPLACEMENTS']),'manifest_members':len(actual),'historical_manifests_checked':len(validated_manifests),'qualified_public_objects_checked':len(env['_qualified_seen']),'private_originals_checked':bool(a.originals_root),'observed_publication_checked':bool(a.publication_root),'qualification':'Syntax, deterministic transformation, three manifest identities, three public digests and all recorded markers checked. Only the bounded helper declarations were executed with retained evidence inputs; neither whole verifier nor product code ran. Full integration remains unexecuted. Without private originals the historical original bytes are not verified.'},indent=2))
if __name__=='__main__':main()
