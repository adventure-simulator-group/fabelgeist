#!/usr/bin/env python3
"""Read-only proof check; optional restricted/original and Git-root replay."""
import argparse, hashlib, json, pathlib, subprocess, sys

def require(x, why):
    if not x: raise RuntimeError(why)
def sha(b): return hashlib.sha256(b).hexdigest()
def git_sha(kind,b): return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_lookup(raw,name):
    pos=0
    while pos < len(raw):
        space=raw.index(b' ',pos); end=raw.index(b'\0',space+1)
        mode=raw[pos:space].decode(); n=raw[space+1:end].decode(); oid=raw[end+1:end+21].hex()
        if n==name: return mode,oid
        pos=end+21
    raise RuntimeError('tree member absent')
def raw_entry(text,index):
    key=text.find('"entries"'); require(key>=0,'queue entries missing')
    pos=text.find('[',key)+1; d=json.JSONDecoder(); i=0
    while True:
        while text[pos].isspace() or text[pos]==',': pos+=1
        require(text[pos]!=']','queue index out of range')
        start=pos; _,pos=d.raw_decode(text,pos)
        if i==index: return text[start:pos].encode()
        i+=1

def main():
    a=argparse.ArgumentParser();a.add_argument('--repository',type=pathlib.Path);a.add_argument('--restricted',type=pathlib.Path);args=a.parse_args();base=pathlib.Path(__file__).resolve().parent
    manifest=json.loads((base/'MANIFEST.json').read_text());expected={x['path']:x for x in manifest['files']};actual={str(p.relative_to(base)) for p in base.rglob('*') if p.is_file() and p != base/'MANIFEST.json'};require(not any(p.is_symlink() for p in base.rglob('*')),'symlink forbidden')
    require(actual==set(expected),'manifest file set mismatch')
    for name,entry in expected.items():
        p=base/name;require(not p.is_symlink(),'symlink forbidden');b=p.read_bytes();require(len(b)==entry['bytes'] and sha(b)==entry['sha256'],'file identity: '+name)
    proof=json.loads((base/'GIT-TREE-PROOFS.json').read_text())['proofs']
    for row in proof:
        b=(base/row['source_file']).read_bytes();require(sha(b)==row['sha256'] and git_sha('blob',b)==row['blob'],'blob proof '+row['path']);cur=row['root_tree']
        for item in row['steps']:
            require(item['tree']==cur,'tree chain')
            raw=(base/'trees'/(cur+'.tree')).read_bytes();require(git_sha('tree',raw)==cur,'tree object')
            mode,cur=tree_lookup(raw,item['entry']);require(mode==item['mode'] and cur==item['oid'],'tree entry')
        require(cur==row['blob'],'leaf mismatch')
        if args.repository:
            root=subprocess.check_output(['git','-C',str(args.repository),'rev-parse',row['revision']+'^{tree}'],text=True).strip();require(root==row['root_tree'],'external commit anchor')
    receipt=json.loads((base/'READ-RECEIPT.json').read_text());s1=json.loads((base/'SOURCE-REGISTRATION.json').read_text())['sources'];s2=json.loads((base/'SUPPORT-SOURCE-REGISTRATION.json').read_text())['sources'];s3=json.loads((base/'OWNER2-SOURCE-REGISTRATION.json').read_text())['sources'];regs={x['label']:x for x in s1+s2+s3}
    for r in receipt['sources']:
        reg=regs[r['label']];f=base/(r['label']+'.source' if r['label'].startswith(('historical-','current-')) else 'support/'+r['label']+'.source');lines=f.read_text().splitlines();require(len(lines)==r['eof'] and sha(f.read_bytes())==r['source_sha256'],'source read identity')
        ranges=reg.get('prospective_personal_display_ranges',reg.get('intended_displays',reg.get('ranges')));require(ranges==[v['range'] for v in r['displayed']],'range registration mismatch');covered=set()
        for v in r['displayed']:
            a,b=v['range'];fmt=(lambda n:f'{n:5}\t') if r['label'].startswith(('historical-','current-')) else (lambda n:f'{n:4} ')
            view=''.join(fmt(i)+lines[i-1]+'\n' for i in range(a,b+1)).encode();require((base/v['file']).read_bytes()==view and sha(view)==v['sha256'],'view mismatch');covered.update(range(a,b+1))
        unread={i for a,b in r['unread'] for i in range(a,b+1)};require(covered.isdisjoint(unread) and covered|unread==set(range(1,len(lines)+1)),'read complement')
    if args.restricted:
        h=json.loads((base/'HISTORICAL-SUMMARY.json').read_text());d=args.restricted/'historical'
        for key,file in [('original_queue','queue-original-56f.json'),('reconstructed_queue','queue-reconstructed-dac7d.json')]:
            raw=(d/file).read_bytes();require(sha(raw)==h[key]['sha256'],'queue stream identity');text=raw.decode()
            for n,expected_hash in h['selected_original_and_reconstruction_identical_sha256'].items():
                selected=raw_entry(text,int(n));require(sha(selected)==expected_hash,'selected queue identity')
        checks={'READ-ROOT-L2-EXT-021-033.json':h['prior_root_l2']['receipt_6533'],'READ-ROOT-L2-CURRENT-008.json':h['prior_root_l2']['receipt_e275'],'READ-ROOT-L3-EXT-020-027.json':h['prior_root_l3']['receipt_6533'],'READ-ROOT-L3-CURRENT-004.json':h['prior_root_l3']['receipt_e275'],'REGISTRATION-ADLER-846.json':h['adler_l11']['registration_sha256'],'READ-L11-308.json':h['adler_l11']['receipt_measured_sha256']}
        for name,want in checks.items():require(sha((d/name).read_bytes())==want,'historic file identity')
        led=(d/'ledger-observed-working.json').read_bytes();require(sha(led)==h['ledger_observation']['sha256'],'ledger identity');rows=json.loads(led)['newly_assessed_ledger']['entries'];require(len(rows)==1084,'ledger count')
    print('Evidence verification passed: manifest, source/view scopes and tree-to-blob proofs.')
    print('External commit anchors '+('checked.' if args.repository else 'NOT checked; supply --repository.'))
    print('Restricted original replay '+('checked.' if args.restricted else 'NOT checked; supply --restricted.'))
if __name__=='__main__':
    try:main()
    except Exception as e:print('Evidence verification FAILED:',str(e),file=sys.stderr);sys.exit(1)
