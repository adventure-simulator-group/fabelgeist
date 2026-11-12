#!/usr/bin/env python3
"""Verify q2333 reconstruction; optionally replay separately held originals."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys

P = Path(__file__).resolve().parent
ORIGINAL_MANIFEST_SHA256 = 'fbc3adf88a18e61e282ca2e4cd2557ce5f291761096626c703f519a33a1a09f6'

def sha(b):
    return hashlib.sha256(b).hexdigest()

def load(name):
    return json.loads((P/name).read_text())

def safe_rel(name):
    p=PurePosixPath(name)
    assert not p.is_absolute() and '..' not in p.parts
    return p

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--original-packet',type=Path,help='separately held exact private original')
    ap.add_argument('--verify-anchors',action='store_true',help='invoke evidence-only verifier from the exact original')
    for name in ('current-commit','frozen-commit','later-commit','original-queue','reconstructed-queue','ledger-snapshot'):
        ap.add_argument('--'+name,type=Path,help='separately held external evidence anchor')
    a=ap.parse_args();manifest=load('SUCCESSOR-MANIFEST.json')
    files={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name!='SUCCESSOR-MANIFEST.json'}
    assert not any(f.is_symlink() for f in P.rglob('*')) and files==set(manifest['files'])
    for name,info in manifest['files'].items():
        safe_rel(name);b=(P/name).read_bytes();assert len(b)==info['bytes'] and sha(b)==info['sha256']
    cross=load('REDACTION-CROSSWALK.json');assert cross['schema']=='726-q2333-path-redaction-no-value-digests-v2'
    assert cross['original_manifest_sha256']==ORIGINAL_MANIFEST_SHA256
    historical=(P/'evidence/MANIFEST.json').read_bytes();assert sha(historical)==ORIGINAL_MANIFEST_SHA256
    oldmanifest=json.loads(historical);rows=cross['files'];assert len(rows)==260 and set(rows)-{'MANIFEST.json'}==set(oldmanifest['files'])
    for name,info in oldmanifest['files'].items():
        assert rows[name]['original_sha256']==info['sha256'] and rows[name]['original_bytes']==info['bytes']
    assert rows['MANIFEST.json']['original_sha256']==ORIGINAL_MANIFEST_SHA256
    aliases={x['replacement']:x for x in cross['aliases']}
    assert len(aliases)==len(cross['aliases'])==140
    assert all(set(x)=={'replacement','original_bytes'} for x in cross['aliases'])
    seen=set();changes=0
    for name,row in rows.items():
        safe_rel(name);b=(P/'evidence'/name).read_bytes();assert len(b)==row['reconstructed_bytes'] and sha(b)==row['reconstructed_sha256']
        end=0;adjust=0
        for e in row['edits']:
            assert set(e)=={'original_start','original_bytes','replacement'}
            assert e['original_start']>=end and 0<e['original_bytes']<row['original_bytes']
            end=e['original_start']+e['original_bytes'];assert end<=row['original_bytes']
            alias=aliases[e['replacement']];assert alias['original_bytes']==e['original_bytes']
            adjust+=len(e['replacement'].encode())-e['original_bytes'];seen.add(e['replacement']);changes+=1
        assert row['reconstructed_bytes']==row['original_bytes']+adjust
    assert changes==596 and seen==set(aliases)
    evidence={str(f.relative_to(P/'evidence')) for f in (P/'evidence').rglob('*') if f.is_file()};assert evidence==set(rows)
    print('Successor manifest, original-manifest identity, no-value-digest crosswalk and reconstructed file hashes verified.')
    if a.original_packet:
        src=a.original_packet;assert sha((src/'MANIFEST.json').read_bytes())==ORIGINAL_MANIFEST_SHA256
        original_files={str(f.relative_to(src)) for f in src.rglob('*') if f.is_file()}
        assert original_files==set(rows) and not any(f.is_symlink() for f in src.rglob('*'))
        values={}
        for name,row in rows.items():
            raw=(src/name).read_bytes();assert len(raw)==row['original_bytes'] and sha(raw)==row['original_sha256']
            pieces=[];pos=0
            for e in row['edits']:
                start=e['original_start'];end=start+e['original_bytes'];value=raw[start:end];assert start>=pos
                if e['replacement'] in values:assert values[e['replacement']]==value
                else:values[e['replacement']]=value
                pieces.extend((raw[pos:start],e['replacement'].encode()));pos=end
            pieces.append(raw[pos:]);assert b''.join(pieces)==(P/'evidence'/name).read_bytes()
        assert len(values)==len(aliases) and len(set(values.values()))==len(values)
        print('Every original whole file and path-redaction byte replay verified against the private original.')
    if a.verify_anchors:
        opts=('current_commit','frozen_commit','later_commit','original_queue','reconstructed_queue','ledger_snapshot')
        if not a.original_packet or any(getattr(a,k) is None for k in opts):ap.error('anchor verification requires the original packet and all six external anchors')
        cmd=[sys.executable,str(a.original_packet/'verify-evidence.py')]
        for k in opts:cmd.extend(('--'+k.replace('_','-'),str(getattr(a,k))))
        result=subprocess.run(cmd,capture_output=True,text=True)
        if result.returncode:raise RuntimeError('Original evidence-only anchor verifier failed; inspect it privately.')
        assert 'Evidence verified:' in result.stdout
        print('Original evidence-only verification passed against the supplied external anchors.')
    elif any(getattr(a,k) is not None for k in ('current_commit','frozen_commit','later_commit','original_queue','reconstructed_queue','ledger_snapshot')):
        ap.error('external anchor arguments require --verify-anchors')
    print('No product code, build, test, fixture, macro or runtime probe executed.')

if __name__=='__main__':main()
