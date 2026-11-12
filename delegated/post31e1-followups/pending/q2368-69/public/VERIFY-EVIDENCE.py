#!/usr/bin/env python3
"""Read-only evidence verifier for q2368/q2369; no product execution."""
import argparse,datetime,hashlib,io,json,pathlib,subprocess,tarfile,tomllib
a=argparse.ArgumentParser();a.add_argument('--archive',default=str(pathlib.Path(__file__).with_name('evidence.tar.gz')));a.add_argument('--repo');args=a.parse_args()
def check(ok,msg):
    if not ok:raise ValueError(msg)
def h(b):return hashlib.sha256(b).hexdigest()
def objsha(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
with tarfile.open(args.archive,'r:gz') as t:
    ms=t.getmembers()
    check(all(x.isfile() for x in ms),'unexpected non-file archive member')
    check(len({x.name for x in ms})==len(ms),'duplicate archive member')
    check(all(not x.name.startswith('/') and '..' not in pathlib.PurePosixPath(x.name).parts for x in ms),'unsafe member name')
    files={x.name:t.extractfile(x).read() for x in ms}
get=lambda n:files[n]
load=lambda n:json.loads(get(n))
manifest=load('MANIFEST.json')
check(set(manifest['files'])==set(files)-{'MANIFEST.json'},'manifest member set mismatch')
for n,m in manifest['files'].items():check(h(get(n))==m['sha256'] and len(get(n))==m['bytes'],'member mismatch: '+n)
qcap=load('QUEUE-CAPTURE.json')
for row in qcap['entries']:
    n=row['pointer'].rsplit('/',1)[-1]
    check(load(f'QUEUE-{n}-RAW.json')==load(f'QUEUE-{n}-PROJECTION.json')==load(f'QUEUE-{n}-PUBLIC-PROJECTION.json'),'queue selection mismatch')
    check(h(get(f'QUEUE-{n}-RAW.json'))==row['raw_sha256'] and row['selected_objects_equal'],'queue selection identity mismatch')
for row in load('RELATED-QUEUE-CAPTURE.json')['rows']:
    check(h(get(row['file']))==row['sha256'] and row['selected_equal_public'],'related queue selection identity mismatch')
for row in load('HISTORY-CAPTURE.json')['rows']:
    check(row['exists'] and h(get(row['file']))==row['expected_sha256']==row['sha256'],'historical proof mismatch')
rs=[k for k in files if k.startswith('DISPLAY-V2-') and k.endswith('.receipt.json')]
for n in sorted(rs):
    r=load(n);data=get(r['source']);reg=load(r['registration'])
    matches=[i for i in reg['items'] if i['file']==r['source'] and r['range'] in i['display_ranges']]
    check(len(matches)==1 and h(data)==matches[0]['sha256'],'source window registration mismatch')
    check(datetime.datetime.fromisoformat(reg['registered_at_utc']) <= datetime.datetime.fromisoformat(r['displayed_at_utc']),'source display precedes registration')
    lo,hi=r['range'];lines=data.decode().splitlines();check(1<=lo<=hi<=len(lines),'display range out of bounds')
    view=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
    check(view==get(r['display']) and h(view)==r['view_sha256'] and r['truncated'] is False,'display content mismatch')
def entries(raw):
    i=0;out={}
    while i<len(raw):
        end=raw.index(b'\0',i);mode,name=raw[i:end].split(b' ',1);check(end+21<=len(raw),'truncated tree')
        check(name not in out,'duplicate tree entry');out[name]=(mode.decode(),raw[end+1:end+21].hex());i=end+21
    return out
allproof=[]
for pn in ('GIT-TREE-PROOFS.json','GIT-TREE-PROOFS-EXTRA.json'):
    proof=load(pn)
    for oid,m in proof['tree_objects'].items():
        raw=get(m['file']);check(h(raw)==m['sha256'] and objsha('tree',raw)==oid,'tree object mismatch')
    for x in proof['proofs']:
        c=x['chain'];check(c and c[0]['tree_oid']==x['root_tree'],'tree root mismatch')
        for i,s in enumerate(c):
            mode,child=entries(get(proof['tree_objects'][s['tree_oid']]['file']))[s['component'].encode()]
            check(mode==s['entry_mode'].lstrip('0') and child==s['child_oid'],'tree component mismatch')
            if i+1<len(c):check(child==c[i+1]['tree_oid'],'tree chain mismatch')
        b=get(x['file']);check(c[-1]['child_oid']==x['blob_oid']==objsha('blob',b) and h(b)==x['sha256'],'blob mismatch')
        allproof.append(x)
lock=tomllib.loads(get('SUPPORT2-CURRENT-4.lock').decode());external={x['local_capture_name']:x for x in load('ARCHIVE-CUSTODY.json')['rows']}
def git(*argv):return subprocess.check_output(['git','-C',args.repo,*argv])
native_count=0;external_checked=0
for cn in ('NATIVE-CAPTURE.json','NATIVE-REEXPORT-CAPTURE.json','NATIVE-PARRY-CAPTURE.json'):
    for x in load(cn)['rows']:
        member=x['member'];pkgver=member.split('/',1)[0];name,version=pkgver.rsplit('-',1)
        lockrows=[i for i in lock['package'] if i['name']==name and i['version']==version]
        check(len(lockrows)==1 and lockrows[0]['checksum']==x['archive_sha256'],'native lock mismatch')
        check(h(get(x['file']))==x['sha256'],'native member hash mismatch')
        if x['archive'] in files:b=get(x['archive'])
        elif args.repo:
            custody=external[x['archive']];check(custody['sha256']==x['archive_sha256'],'external archive identity mismatch')
            b=git('show',custody['committed_revision']+':'+custody['committed_evidence_path']);external_checked+=1
        else:continue
        check(h(b)==x['archive_sha256'],'native archive checksum mismatch')
        with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as t:memberdata=t.extractfile(member).read()
        check(memberdata==get(x['file']),'native archive member mismatch');native_count+=1
if args.repo:
    for x in allproof:check(git('rev-parse',x['revision']+'^{tree}').decode().strip()==x['root_tree'],'commit root mismatch')
    old=git('show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json');pub=git('show','e709f11f0aa6bf736bd20819a628e4790cd03999:baseline-059/queue.json')
    check(h(old)==qcap['original_whole_sha256'] and h(pub)==qcap['public_whole_sha256'],'whole queue hashes differ from capture')
    orig=json.loads(old)['entries'];public=json.loads(pub)['entries']
    for n in (2368,2369):check(orig[n]==public[n]==load(f'QUEUE-{n}-RAW.json'),'queue object Git mismatch')
    for x in load('RELATED-QUEUE-CAPTURE.json')['rows']:
        n=x['index'];check(orig[n]==public[n]==load(x['file']),'related queue object Git mismatch')
    baseline=git('show','9e9e237c0c3c28278342e831da9f243fc2229e58:BASELINE-95E-20261110.json')
    check(h(baseline)==load('LEDGER-NAV.json')['sha256'],'ledger hash mismatch')
    ledger=json.loads(baseline)['newly_assessed_ledger']['entries']
    for i in (318,321):check(ledger[i]==load(f'LEDGER-ROW-{i}.json'),'ledger row mismatch')
    for i,path in enumerate(('ASSESSMENT-GATE-AND-SUPPORT-95E.json','ASSESSMENT-FOUNDATION-VOLUME-95E.json')):
        check(git('show','9e9e237c0c3c28278342e831da9f243fc2229e58:'+path)==get(f'LINKED-ASSESSMENT-{i}.json'),'linked assessment mismatch')
print(json.dumps({'result':'ok','members':len(files),'source_displays':len(rs),'git_paths':len(allproof),'git_commit_roots_bound':bool(args.repo),'native_archives_checked':native_count,'external_native_archives_checked':external_checked,'external_native_archives_total':len(external),'qualification':'Evidence integrity only; native dependency activation, semantic proof, aliases, product validation and full audit closure are not asserted. --repo is required to bind commit roots and retrieve externally hash-linked archives.'},indent=2))
