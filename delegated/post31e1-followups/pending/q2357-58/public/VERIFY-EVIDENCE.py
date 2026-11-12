#!/usr/bin/env python3
"""Read-only verifier for q2357/q2358 source-evidence transport."""
import argparse,datetime,hashlib,io,json,pathlib,subprocess,tarfile,tomllib
p=argparse.ArgumentParser()
p.add_argument('--archive',default=str(pathlib.Path(__file__).with_name('evidence.tar.gz')))
p.add_argument('--repo')
args=p.parse_args()
def check(cond,message):
    if not cond: raise ValueError(message)
def h(b):return hashlib.sha256(b).hexdigest()
def object_sha(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
with tarfile.open(args.archive,'r:gz') as tar:
    members=tar.getmembers()
    check(all(m.isfile() for m in members),'unexpected non-file tar member')
    check(len({m.name for m in members})==len(members),'duplicate tar name')
    check(all(not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts for m in members),'unsafe tar path')
    files={m.name:tar.extractfile(m).read() for m in members}
get=lambda n:files[n]
load=lambda n:json.loads(get(n))
manifest=load('MANIFEST.json')
check(set(manifest['files'])==set(files)-{'MANIFEST.json'},'manifest member set differs')
for name,meta in manifest['files'].items():
    check(h(get(name))==meta['sha256'] and len(get(name))==meta['bytes'],'member mismatch: '+name)
for n in (2357,2358):
    raw=load(f'QUEUE-{n}-RAW.json')
    check(raw==load(f'QUEUE-{n}-PROJECTION.json')==load(f'QUEUE-{n}-PUBLIC-PROJECTION.json'),'selected queue projection differs')
capture=load('QUEUE-CAPTURE.json')
for ent in capture['entries']:
    n=ent['pointer'].rsplit('/',1)[-1]
    check(h(get(f'QUEUE-{n}-RAW.json'))==ent['raw_sha256'],'queue raw hash differs')
    check(h(get(f'QUEUE-{n}-PROJECTION.json'))==ent['projection_sha256'],'queue projection hash differs')
    check(ent['selected_objects_equal'],'queue equality not recorded')
for row in load('HISTORY-CAPTURE.json')['rows']:
    if row['exists']:check(h(get(row['file']))==row['sha256'],'historical receipt mismatch')
for receipt_name in sorted(k for k in files if k.startswith('DISPLAY-V2-') and k.endswith('.receipt.json')):
    receipt=load(receipt_name);source=get(receipt['source']); reg=load(receipt['registration'])
    found=[i for i in reg['items'] if i['file']==receipt['source'] and receipt['range'] in i['display_ranges']]
    check(len(found)==1 and h(source)==found[0]['sha256'],'source registration mismatch')
    check(datetime.datetime.fromisoformat(reg['registered_at_utc']) <= datetime.datetime.fromisoformat(receipt['displayed_at_utc']),'display precedes registration')
    lo,hi=receipt['range'];lines=source.decode().splitlines()
    check(1<=lo<=hi<=len(lines),'display range out of bounds')
    view=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
    check(view==get(receipt['display']) and h(view)==receipt['view_sha256'] and receipt['truncated'] is False,'display mismatch')
n=load('NATIVE-CAPTURE.json'); archive=get(n['archive'])
check(h(archive)==n['archive_sha256'],'native package archive mismatch')
lock=tomllib.loads(get('CARGO-LOCK-95E').decode())
rows=[x for x in lock['package'] if x['name']=='bevy_mesh' and x['version']=='0.19.1']
check(len(rows)==1 and rows[0]['checksum']==n['archive_sha256'],'lock checksum mismatch')
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
    native=t.extractfile(n['member']).read()
check(native==get('NATIVE-BEVY-MESH.rs') and h(native)==n['member_sha256'],'native source member mismatch')
proof=load('GIT-TREE-PROOFS.json')
for oid,meta in proof['tree_objects'].items():
    raw=get(meta['file'])
    check(h(raw)==meta['sha256'] and object_sha('tree',raw)==oid,'Git tree object mismatch')
def entries(raw):
    i=0;out={}
    while i<len(raw):
        end=raw.index(b'\0',i);mode,name=raw[i:end].split(b' ',1)
        check(end+21<=len(raw),'bad tree bytes')
        check(name not in out,'duplicate tree name')
        out[name]=(mode.decode(),raw[end+1:end+21].hex())
        i=end+21
    return out
for item in proof['proofs']:
    chain=item['chain'];check(chain and chain[0]['tree_oid']==item['root_tree'],'root chain mismatch')
    for i,step in enumerate(chain):
        tree=proof['tree_objects'][step['tree_oid']]['file']
        mode,child=entries(get(tree))[step['component'].encode()]
        check(mode==step['entry_mode'].lstrip('0') and child==step['child_oid'],'tree entry mismatch')
        if i+1<len(chain):check(child==chain[i+1]['tree_oid'],'tree chain breaks')
    body=get(item['file'])
    check(chain[-1]['child_oid']==item['blob_oid']==object_sha('blob',body) and h(body)==item['sha256'],'source blob mismatch')
if args.repo:
    def git(*a):return subprocess.check_output(['git','-C',args.repo,*a])
    for item in proof['proofs']:
        root=git('rev-parse',item['revision']+'^{tree}').decode().strip()
        check(root==item['root_tree'],'commit root mismatch')
    original=git('show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json')
    published=git('show','e709f11f0aa6bf736bd20819a628e4790cd03999:baseline-059/queue.json')
    check(h(original)==capture['original_whole_sha256'] and h(published)==capture['public_whole_sha256'],'queue whole-file mismatch')
    orig=json.loads(original);pub=json.loads(published)
    for n in (2357,2358):
        check(orig['entries'][n]==pub['entries'][n]==load(f'QUEUE-{n}-RAW.json'),'selected queue mismatch against Git')
    baseline=git('show','9e9e237c0c3c28278342e831da9f243fc2229e58:BASELINE-95E-20261110.json')
    check(h(baseline)==load('LEDGER-NAV.json')['sha256'],'ledger whole-file mismatch')
    led=json.loads(baseline)['newly_assessed_ledger']['entries']
    for i in (266,870):check(led[i]==load(f'LEDGER-ROW-{i}.json'),'selected ledger row mismatch')
    assessment=git('show','9e9e237c0c3c28278342e831da9f243fc2229e58:ASSESSMENT-REGIONAL-MAP-SURFACE-95E.json')
    check(assessment==get('LINKED-REGIONAL-ASSESSMENT.json'),'linked assessment mismatch')
print(json.dumps({'result':'ok','members':len(files),'source_displays':sum(k.startswith('DISPLAY-V2-') and k.endswith('.receipt.json') for k in files),'git_paths':len(proof['proofs']),'commit_roots_independently_bound':bool(args.repo),'limits':'Evidence integrity only; no claim of product execution, alias, full queue/lens closure or complete privacy scan.'},indent=2))
