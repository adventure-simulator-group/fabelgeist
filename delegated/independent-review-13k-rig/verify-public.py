#!/usr/bin/env python3
"""Verify a privacy-qualified 13K review transport without writing files."""
from pathlib import Path
import argparse, datetime, gzip, hashlib, io, json, re, tarfile
P = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, b):
    return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_entries(b):
    o={}; i=0
    while i<len(b):
        j=b.index(b'\0',i); mode,name=b[i:j].split(b' ',1); name=name.decode()
        assert name not in o and j+21<=len(b)
        o[name]=(mode.decode(),b[j+1:j+21].hex()); i=j+21
    assert i==len(b); return o

def local_pat():
    return re.compile(rb'(?<![A-Za-z0-9_./-])/(?:home|Users)/[^\s"\x27<>]+|(?<![A-Za-z0-9_./-])/tmp/726-[A-Za-z0-9_./-]+|[A-Za-z]:\\{1,2}Users\\{1,2}[^\s"\x27<>]+',re.I)
def secret_pat():
    return re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
email=re.compile(rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}')

ap=argparse.ArgumentParser(); ap.add_argument('--proof', type=Path); ap.add_argument('--original-dir', type=Path); args=ap.parse_args()
manifest=json.loads((P/'MANIFEST.json').read_bytes())
assert manifest['schema']=='726-independent-review-13k-public-manifest-v1'
files={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file()}-{'MANIFEST.json'}
assert files==set(manifest['files'])
for name,entry in manifest['files'].items():
    b=(P/name).read_bytes(); assert sha(b)==entry['sha256'] and len(b)==entry['bytes'],name

idx=json.loads((P/'INDEPENDENT-REVIEW-13K-RIG-IDENTITY.json').read_bytes())
archive=(P/idx['archive']).read_bytes(); assert sha(archive)==idx['archive_sha256'] and len(archive)==idx['archive_bytes']
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
    members=t.getmembers(); assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts and (x.uid,x.gid,x.uname,x.gname,x.mtime)==(0,0,'','',0) and not x.pax_headers for x in members)
    data={x.name:t.extractfile(x).read() for x in members}; assert len(data)==len(members)
rows={x['path']:x for x in idx['members']}; assert len(rows)==len(idx['members'])==39 and set(rows)==set(data)
for name,b in data.items():
    x=rows[name]; assert len(b)==x['published_bytes'] and sha(b)==x['published_sha256']; plain=gzip.decompress(b) if x['encoding']=='gzip' else b
    assert len(plain)==x['published_uncompressed_bytes'] and sha(plain)==x['published_uncompressed_sha256']
    assert not local_pat().search(name.encode()) and not local_pat().search(plain),name
    assert not secret_pat().search(plain) and not email.search(plain),name
    assert x['changed']==(x['original_sha256']!=x['published_sha256'])
    if not x['changed']:
        assert x['original_bytes']==x['published_bytes'] and x['original_uncompressed_sha256']==x['published_uncompressed_sha256'] and x['local_path_redaction_occurrences']==0
assert {n for n,x in rows.items() if x['changed']}=={'REPORT.json','inputs/original-059-queue.json.gz'}
assert sum(x['local_path_redaction_occurrences'] for x in rows.values())==13
assert b'source/home/noerrors' in gzip.decompress(data['inputs/historical-774-contract-groups.jsonl.gz'])
assert sha(data['MANIFEST.json'])==idx['original_review_manifest_sha256']
assert rows['REPORT.json']['original_sha256']==idx['original_review_report_sha256']
original_manifest=json.loads(data['MANIFEST.json'])
for name,x in original_manifest['files'].items():
    if name=='inputs/central-proof.tar.gz':
        assert x['sha256']==idx['external_dependency']['original_sha256'] and x['bytes']==idx['external_dependency']['original_bytes']; continue
    assert name in rows and rows[name]['original_sha256']==x['sha256'] and rows[name]['original_bytes']==x['bytes']
assert set(original_manifest['files'])==set(rows)-{'MANIFEST.json'}|{'inputs/central-proof.tar.gz'}
assessment=json.loads(data['inputs/assessment.json']); assert sha(data['inputs/assessment.json'])==idx['reviewed_draft_assessment_sha256']
assert 'earlier' in idx['draft_qualification'].lower() and 'corrected final' in idx['draft_qualification'].lower()
working_bytes=gzip.decompress(data['inputs/BASELINE-95E-20261110.json.gz']); head_bytes=gzip.decompress(data['inputs/baseline-head.json.gz'])
assert sha(working_bytes)==idx['baseline_qualification']['working_baseline_uncompressed_sha256']; assert sha(head_bytes)==idx['baseline_qualification']['committed_baseline_uncompressed_sha256']
ident='local-candidate-garment-tube-kind-admission'; w=json.loads(working_bytes); h=json.loads(head_bytes)
assert sum(x.get('id')==ident for x in w['newly_assessed_ledger']['entries'])==1
assert sum(x.get('id')==ident for x in h['newly_assessed_ledger']['entries'])==0
q=json.loads(gzip.decompress(data['inputs/original-059-queue.json.gz']))['entries'][1818]
assert q==json.loads(data['inputs/q1818-reconstructed.json'])['entry'] and q['status']=='open-evidence-or-disposition' and q['body_or_population_credit']==0
cov=json.loads(gzip.decompress(data['inputs/COVERAGE-L01-ROLES.json.gz']))
assert [cov['roles'][i]['status'] for i in (2623,2624)]==['unassessed','unassessed']
reg=json.loads(data['REGISTRATION.json']); receipt=json.loads(data['READ-RECEIPT.json'])
assert sha(data['REGISTRATION.json'])==receipt['registration_sha256'] and sha(data['SUPPLEMENT-REGISTRATION.json'])==receipt['supplement_registration_sha256']
assert len(reg['rows'])==len(receipt['rows'])==10
for r,rec in zip(reg['rows'],receipt['rows']):
    b=data[r['file']]; assert sha(b)==r['sha256'] and oid('blob',b)==r['blob']
    assert (r['id'],r['revision'],r['path'],r['planned_ranges'])==(rec['id'],rec['revision'],rec['path'],rec['actual_display_ranges'])
assert json.loads(data['inputs/prior-report.json'])['author']=='/root/rig_identity'
assert json.loads(data['inputs/earlier-candidate-report.json'])['author']=='/root/mhr_character'

if args.proof:
    raw=args.proof.read_bytes(); dep=idx['external_dependency']; assert sha(raw)==dep['original_sha256']==dep['published_sha256'] and len(raw)==dep['original_bytes']==dep['published_bytes']
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
        m=t.getmembers(); assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in m)
        proofdata={x.name:t.extractfile(x).read() for x in m}; assert len(proofdata)==len(m)
    p=json.loads(proofdata['proof.json']); objects={}
    for key,v in p['objects'].items():
        b=proofdata[v['archive']]; assert sha(b)==v['sha256'] and len(b)==v['bytes'] and oid(v['type'],b)==key; objects[key]=b
    assert len(objects)==23 and set(proofdata)=={'proof.json'}|{v['archive'] for v in p['objects'].values()}
    central=json.loads(data['inputs/central-registration.json']); centralreceipt=json.loads(data['inputs/central-receipt.json']); assert sha(data['inputs/central-registration.json'])==centralreceipt['registration_sha256']
    assert len(p['paths'])==len(central['rows'])==len(centralreceipt['reads'])==12
    for path,row,rec in zip(p['paths'],central['rows'],centralreceipt['reads']):
        assert (path['revision'],path['path'],path['blob'],path['sha256'])==(row['revision'],row['path'],row['git_blob'],row['sha256'])
        assert all(row[k]==rec[k] for k in ('revision','path','git_blob','sha256')) and row['planned_display_ranges']==rec['actual_display_ranges']
        assert objects[path['revision']].splitlines()[0]==b'tree '+path['root_tree'].encode(); cur=path['root_tree']; names=[]
        for link in path['chain']:
            assert link['tree']==cur and tree_entries(objects[cur])[link['name']]==(link['mode'],link['object']); cur=link['object']; names.append(link['name'])
        assert names==path['path'].split('/') and cur==path['blob'] and sha(objects[cur])==path['sha256'] and len(objects[cur])==path['bytes']
    for r in reg['rows']: assert data[r['file']]==objects[r['blob']]
    print('External proof checked: 12 Git commit/tree/blob paths and 23 original objects. The external archive contains public commit identity metadata and is outside the embedded privacy scan.')
else:
    print('External proof was not supplied; commit/tree binding is not checked in this invocation.')

if args.original_dir:
    original=args.original_dir; files={str(f.relative_to(original)) for f in original.rglob('*') if f.is_file()}
    assert files==set(rows)|{'inputs/central-proof.tar.gz'}
    for name,row in rows.items():
        raw=(original/name).read_bytes(); assert len(raw)==row['original_bytes'] and sha(raw)==row['original_sha256']
        if name=='REPORT.json':
            expected=re.sub(rb'(?<![A-Za-z0-9_./-])/tmp/726-[A-Za-z0-9_./-]+',b'<PRIOR_LOCAL_REVIEW_PATH>',raw); assert expected==data[name]
        elif name=='inputs/original-059-queue.json.gz':
            plain=gzip.decompress(raw); matches=re.findall(rb'(?<![A-Za-z0-9_./-])/(?:home|Users)/[^\s"\x27<>]+',plain); vals=sorted(set(matches)); assert len(matches)==12 and len(vals)==8
            for i,v in enumerate(vals,1):
                m=f'<LOCAL_SOURCE_PATH_{i:02}>'.encode(); meta=idx['local_path_redactions']['private_original_path_sha256_mapping'][i-1]
                assert meta['redaction_marker']==m.decode() and meta['original_path_sha256']==sha(v) and meta['occurrences']==matches.count(v); plain=plain.replace(v,m)
            assert gzip.compress(plain,mtime=0)==data[name]
        else: assert raw==data[name]
    proof=(original/'inputs/central-proof.tar.gz').read_bytes(); assert sha(proof)==idx['external_dependency']['original_sha256']
    print('Retained originals checked read-only; report and queue transformations reproduce exactly, and the original proof remains separate.')
print('Verified public archive, original/public hashes, bounded draft and provenance, nonclosure, and embedded privacy patterns. Pattern scans cannot prove absence of every possible sensitive item.')
