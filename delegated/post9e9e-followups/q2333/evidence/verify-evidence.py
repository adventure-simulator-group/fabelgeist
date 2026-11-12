#!/usr/bin/env python3
"""Verify q2333 evidence identities and registered text displays only."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re

P = Path(__file__).resolve().parent
CURRENT = '95e63628f9d70b8dd6e6fede64334696c6995009'
FROZEN = '6533e444fa37552927070cc8d8190907fd3b413e'
LATER = 'e2755a732c84e67296b5e3441a4259a84f461b2f'

def load(name):
    return json.loads((P/name).read_text())

def sha(b):
    return hashlib.sha256(b).hexdigest()

def oid(kind,b):
    return hashlib.sha1(f'{kind} {len(b)}\0'.encode()+b).hexdigest()

def pointer(x,p):
    for k in p.strip('/').split('/') if p!='/' else []:
        k=k.replace('~1','/').replace('~0','~')
        x=x[int(k)] if isinstance(x,list) else x[k]
    return x

def entry_at(raw,n):
    t=raw.decode();pos=re.search(r'"entries"\s*:\s*\[',t).end();decoder=json.JSONDecoder()
    for _ in range(n+1):
        while t[pos] in ' \n\r\t,':pos+=1
        start=pos;_,pos=decoder.raw_decode(t,pos)
    return t[start:pos].encode(),len(t[:start].encode()),len(t[:pos].encode())

def tree_entries(b):
    d={};i=0
    while i<len(b):
        j=b.index(b' ',i);k=b.index(b'\0',j+1)
        d[b[j+1:k].decode()]=(b[i:j].decode(),b[k+1:k+21].hex());i=k+21
    return d

def check_chain(root,path,chain,src):
    cur=root; parts=path.split('/');assert len(parts)==len(chain)
    for part,item in zip(parts,chain):
        b=(P/'proof/git-trees'/(cur+'.tree')).read_bytes();assert oid('tree',b)==cur
        mode,nxt=tree_entries(b)[part];assert item=={'tree':cur,'name':part,'mode':mode,'oid':nxt};cur=nxt
    assert oid('blob',src)==cur
    return cur

def check_view(rec,reg,source_base,display_path):
    a,b=rec['range'];name=rec['source'];raw=(P/source_base/reg['revision']/name).read_bytes();lines=raw.decode().splitlines()
    assert 1<=a<=b<=len(lines) and sha(raw)==rec['source_sha256']
    assert datetime.datetime.fromisoformat(rec['at_utc'])>=datetime.datetime.fromisoformat(reg['at_utc'])
    text=f'SOURCE {name}:{a}-{b}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(a,b+1))
    assert text==(P/display_path).read_text() and sha(text.encode())==rec['display_sha256']

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--current-commit',type=Path,required=True)
    ap.add_argument('--frozen-commit',type=Path,required=True)
    ap.add_argument('--later-commit',type=Path,required=True)
    ap.add_argument('--original-queue',type=Path,required=True)
    ap.add_argument('--reconstructed-queue',type=Path,required=True)
    ap.add_argument('--ledger-snapshot',type=Path,required=True)
    ap.add_argument('--skip-manifest',action='store_true')
    a=ap.parse_args()
    if not a.skip_manifest:
        m=load('MANIFEST.json');actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name!='MANIFEST.json'}
        assert actual==set(m['files'])
        for name,info in m['files'].items():
            b=(P/name).read_bytes();assert len(b)==info['bytes'] and sha(b)==info['sha256'],name
    ext=load('initial/QUEUE-EXTRACTION.json');entries=[]
    for n,fn in enumerate([a.original_queue,a.reconstructed_queue]):
        raw=fn.read_bytes();v=ext['entries'][n];assert sha(raw)==v['whole_sha256']
        b,start,end=entry_at(raw,2333);assert b==(P/f'initial/QUEUE-ENTRY-{n}.json').read_bytes()
        assert sha(b)==v['entry_sha256'] and (start,end)==(v['byte_start'],v['byte_end_exclusive']);entries.append(b)
    assert entries[0]==entries[1] and ext['selected_entry_byte_equal']
    lines=entries[0].decode().splitlines();assert len(lines)==489
    for n,start in enumerate(range(1,len(lines)+1,70),1):
        end=min(start+69,len(lines));text=''.join(f'{i:4} {lines[i-1]}\n' for i in range(start,end+1))
        assert text==(P/f'initial/HISTORY-DISPLAY-{n:02}.txt').read_text()
    inv={x['path'].removeprefix('REDACTED-LOCAL-PATH-0140'):x['sha256'] for x in load('initial/HISTORY-REFERENCE-INVENTORY.json')}
    recs=load('initial/HISTORY-COPY-RECEIPT.json');assert len(inv)==len(recs)==20
    for r in recs:
        b=(P/'initial/history'/r['path']).read_bytes();assert r['status']=='matched' and sha(b)==r['observed_sha256']==r['expected_sha256']==inv[r['path']] and len(b)==r['bytes']
    nested=load('initial/HISTORY-NESTED-RECEIPT.json');assert len(nested)==2
    for r in nested:
        b=(P/'initial/history'/r['path']).read_bytes();assert r['status']=='matched' and sha(b)==r['observed_sha256']==r['expected_sha256'] and len(b)==r['bytes']
    frozen_rows=load('initial/HISTORY-FROZEN-ROWS-VIEW.json');assert len(frozen_rows)==5
    for x in frozen_rows:
        row=pointer(json.loads((P/'initial/history'/x['file']).read_text()),x['pointer'])
        assert all(row[k]==v for k,v in x['metadata'].items())
        data=(P/'initial/sources'/FROZEN/row['source_id'].removeprefix('pr-working:')).read_bytes();assert sha(data)==row['sha256']
    old=json.loads((P/'initial/history/parallel-l13-20261029/READ-PL13-CURRENT-255-262-001.json').read_text())
    assert old['rows'][6]==load('initial/HISTORY-CURRENT-ROW6-VIEW.json')
    assert old['rows'][10]==load('initial/HISTORY-CURRENT-ROW-VIEW.json')['row']
    adreg=load('followup/HISTORY-ADOPTION-REGISTRATION.json')
    for name,fields in adreg['selections'].items():
        obj=json.loads((P/'initial/history'/name).read_text());assert {k:obj.get(k) for k in fields}==load('followup/HISTORY-ADOPTION-VIEW.json')[name]
    roots={};commits={CURRENT:a.current_commit,FROZEN:a.frozen_commit,LATER:a.later_commit}
    for rev,fn in commits.items():
        raw=fn.read_bytes();assert oid('commit',raw)==rev;roots[rev]=raw.split(b'\n',1)[0].split()[1].decode()
    meta=load('proof/COMMIT-METADATA.json')
    for x in meta:
        if x['revision'] in (CURRENT,LATER):assert x['root_tree']==roots[x['revision']] and x['sha256']==sha(commits[x['revision']].read_bytes())
    assert roots[FROZEN]==load('proof/FROZEN-ANCHOR-VIEW.json')['candidates'][0]['root_tree']
    assert sha(a.frozen_commit.read_bytes())==load('proof/FROZEN-ANCHOR-VIEW.json')['anchor_sha256']
    proofs=load('proof/FINAL-PROOF.json'); assert len(proofs)==15
    expected={(rev,path) for rev,paths in load('proof/PATHS-REGISTRATION.json')['revision_paths'].items() for path in paths}
    assert expected=={(x['revision'],x['path']) for x in proofs}
    for x in proofs:
        raw=(P/x['source_rel']).read_bytes();assert len(raw)==x['bytes'] and sha(raw)==x['source_sha256'] and roots[x['revision']]==x['root_tree']
        assert check_chain(roots[x['revision']],x['path'],x['chain'],raw)==x['blob']
    clock=load('proof/CLOCK-PROOF.json');cr=load('proof/CLOCK-PROOF-REGISTRATION.json');raw=(P/cr['source_rel']).read_bytes();assert clock['complete'] and sha(raw)==clock['source_sha256']
    assert check_chain(roots[CURRENT],cr['path'],clock['chain'],raw)==clock['blob']
    loc=load('initial/SOURCE-LOCATOR-VIEW.json');assert len(loc)==5
    for x in loc:
        old=(P/'initial/sources'/FROZEN/x['path']).read_bytes();new=(P/'initial/sources'/CURRENT/x['path']).read_bytes()
        assert sha(old)==x['old_sha256'] and sha(new)==x['current_sha256'] and (old==new)==x['byte_equal']
    for f in sorted((P/'proof').glob('TREE-FETCH-[12]-*.json')):
        if f.name.endswith('REGISTRATION.json') or f.name.endswith('VIEW.json'):continue
        j=json.loads(f.read_text());assert not j.get('isError');o=j['structuredContent'];obj=o.get('structuredContent')
        if not isinstance(obj,dict) or 'tree' not in obj:obj=json.loads(o['content'])
        assert not obj.get('truncated',False)
        raw=b''.join((e['mode'].lstrip('0')+' '+e['path']).encode()+b'\0'+bytes.fromhex(e['sha']) for e in obj['tree'])
        assert oid('tree',raw)==obj['sha'] and raw==(P/'proof/git-trees'/(obj['sha']+'.tree')).read_bytes()
    seen=[]
    for f in sorted((P/'initial').glob('SOURCE*-RECEIPT-*.json')):
        rec=json.loads(f.read_text());m=re.fullmatch(r'SOURCE(?:-(.+?))?-RECEIPT-(.+)\.json',f.name);assert m
        rn='SOURCE'+('-'+m.group(1) if m.group(1) else '')+'-REGISTRATION.json';reg=load('initial/'+rn);a0,b0=rec['range']
        assert any(a0>=lo and b0<=hi for lo,hi in reg['windows'][rec['source']]);check_view(rec,reg,'initial/sources','initial/'+rec['display']);seen.append(('initial/'+f.name,rec['source'],a0,b0))
    for n in ['A','B']:
        rec=load(f'followup/WEB-RECEIPT-{n}.json');reg=load('followup/WEB-DISPLAY-REGISTRATION.json');assert rec['range'] in reg['windows'];check_view(rec,reg,'followup/sources','followup/'+rec['display']);seen.append((f'followup/WEB-RECEIPT-{n}.json',rec['source'],*rec['range']))
    rec=load('followup/TIME-RECEIPT.json');reg=load('followup/TIME-DISPLAY-REGISTRATION.json');assert rec['range']==reg['range'];check_view(rec,reg,'followup/sources','followup/TIME-DISPLAY.txt');seen.append(('followup/TIME-RECEIPT.json',rec['source'],*rec['range']))
    limits=load('followup/READ-LIMITS.json')['display_receipts'];assert len(seen)==len(limits)==33 and set(seen)=={(x['receipt'],x['source'],*x['range']) for x in limits}
    b=a.ledger_snapshot.read_bytes();nav=load('initial/LEDGER-NAV-VIEW.json');assert len(b)==nav['bytes'] and sha(b)==nav['sha256'];obj=json.loads(b);rows=obj['newly_assessed_ledger']['entries'];assert len(rows)==nav['rows']
    for term,result in nav['terms'].items():
        hits=[{'index':i,'id':x.get('id')} for i,x in enumerate(rows) if term.casefold() in json.dumps(x,ensure_ascii=False).casefold()]
        assert len(hits)==result['count'] and hits[:30]==result['shown'] and (len(hits)>30)==result['truncated']
    sel=load('initial/LEDGER-SELECT-VIEW.json');assert sel['sha256']==sha(b)
    for x in sel['selections']:assert pointer(obj,x['pointer'])==x['value']
    print('Evidence verified: exact 059 entry, 22 historical records, five frozen objects, 16 Git-bound revision/path pairs, 33 registered current displays, and bounded ledger metadata.')
    print('No product code, product parser, build, test, fixture, macro or runtime probe executed.')

if __name__=='__main__':main()
