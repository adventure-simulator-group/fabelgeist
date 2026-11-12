#!/usr/bin/env python3
"""Verify q2319 audit evidence; never import or execute product code."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess

P = Path(__file__).resolve().parent

def load(path): return json.loads((P/path).read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def time(s): return dt.datetime.fromisoformat(s)
def safe(s): return not Path(s).is_absolute() and '..' not in Path(s).parts
def git(repo,*args): return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args],env=dict(os.environ,GIT_NO_LAZY_FETCH='1'))
def spans(values):
    out=[]
    for n in sorted(values):
        if not out or out[-1][1]+1 != n: out.append([n,n])
        else: out[-1][1]=n
    return out

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--git-repo',type=Path);parser.add_argument('--publication',type=Path);args=parser.parse_args()
    m=load('MANIFEST.json');actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x!=P/'MANIFEST.json'};assert actual==set(m['files'])
    for name,v in m['files'].items():
        assert safe(name) and not (P/name).is_symlink();b=(P/name).read_bytes();assert sha(b)==v['sha256'] and len(b)==v['bytes'],name
    cs=load('SOURCE-CAPTURES.json')+[load('SOURCE-EXTENSION-CAPTURE.json')]+load('SOURCE-WRITER-CAPTURES.json')+[load('SOURCE-CATALOG-CAPTURE.json')]+load('SOURCE-CONDITION-CAPTURES.json')
    captures={x['path']:x for x in cs};assert len(captures)==9
    for x in captures.values():
        b=(P/x['capture']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and len(b.splitlines())==x['eof']
    rr=load('SOURCE-READ-RECEIPT.json');seen={n:set() for n in captures}
    for x in rr['chunks']:
        r=load(x['registration']);assert time(r['registered_utc'])<=time(x['saved_utc']);assert any(s['path']==x['source_path'] and s['range'][0]<=x['start']<=x['end']<=s['range'][1] for s in r['sources'])
        l=(P/captures[x['source_path']]['capture']).read_text().splitlines();b=''.join(f'{i:4d} {l[i-1]}\n' for i in range(x['start'],x['end']+1)).encode();assert b==(P/x['display']).read_bytes() and sha(b)==x['sha256'];seen[x['source_path']].update(range(x['start'],x['end']+1))
    for x in rr['coverage']:
        assert x['eof']==captures[x['source_path']]['eof'] and x['personally_displayed']==spans(seen[x['source_path']]) and x['unread']==spans(set(range(1,x['eof']+1))-seen[x['source_path']])
    assert sum(map(len,seen.values()))==rr['unique_lines_personally_displayed']
    for x in load('OTHER-READ-RECEIPT.json')['chronology']:assert time(load(x['registration'])['registered_utc'])<=time(x['saved_utc'])
    h=load('HISTORY-IDENTITIES.json');bodies=[(P/x['packet_path']).read_bytes() for x in h];assert bodies[0]==bodies[1]
    for x,b in zip(h,bodies):assert sha(b)==x['entry_sha256'] and len(b)==x['entry_bytes']
    q=json.loads(bodies[0]);assert all(q[k]==v for k,v in load('Q2319-DISPLAY-SUMMARY.json').items());assert q['exact_source_identities_and_assessed_ranges']==load('Q2319-DISPLAY-SOURCES.json')
    gran=q['granular_contract_views'][0];assert all(gran[k]==v for k,v in load('Q2319-GRANULAR-SUMMARY.json').items());assert gran['exact_source_identities_and_assessed_ranges']==load('Q2319-GRANULAR-SOURCES.json')
    refs={}
    def collect(v):
        if isinstance(v,dict):
            if isinstance(v.get('path'),str) and isinstance(v.get('sha256'),str):refs[v['path']]=v['sha256']
            for x in v.values():collect(x)
        elif isinstance(v,list):
            for x in v:collect(x)
    collect(q);reg=load('RECOVERY-REGISTRATION.json');res={x['reference']:x for x in load('RECOVERY-RESULT.json')};assert len(refs)==len(reg['references'])==len(res)==8
    for x in reg['references']:
        v=res[x['path']];b=(P/v['packet_path']).read_bytes();assert v['matched'] and refs[x['path']]==x['expected_sha256']==v['actual_sha256']==sha(b) and len(b)==v['bytes']
    for n in [248,256]:
        base=P/'historical-originals/parallel-l13-20261029';d=json.loads((base/f'READ-PL13-{n}.json').read_text());r=json.loads((base/f'REGISTRATION-PL13-{n}.json').read_text());a=json.loads((P/f'historical-originals/segovb/READ-L13-PARALLEL-{n-2}.json').read_text())
        expected={'delegated_status':d.get('status'),'registered_before_read_attestation':d.get('registered_before_read_attestation'),'delegated_row_0':d['rows'][0],'source_registration_0':r['sources'][0],'root_kind':a['kind'],'root_at':a['at'],'root_verified_source_registrations':a['verified_source_registrations'],'root_personal_rereads':a['root_personal_rereads'],'root_current_source_reconciled':a['current_source_reconciled'],'root_runtime_validation':a['runtime_validation'],'root_implementation_gate_complete':a['implementation_gate_complete']}
        assert expected==load(f'HISTORICAL-SELECT-{n}.json')
    related=load('related/ASSESSMENT-L01-PARTY-MERGE-FOOD-95E.json');assert all(related[k]==v for k,v in load('RELATED-SELECT-SUMMARY.json').items());assert all(related['candidates'][0][k]==v for k,v in load('RELATED-SELECT-CANDIDATE.json').items())
    for x in load('RELATED-HISTORY-IDENTITIES.json'):
        b=(P/f"Q{x['index']}-ORIGINAL-EXACT-SLICE.json").read_bytes();assert sha(b)==x['entry_sha256'] and len(b)==x['entry_bytes'];row=json.loads(b);assert all(row[k]==v for k,v in load(f"Q{x['index']}-SELECT.json").items())
    comparison=load('RANGE-COMPARISON-RESULT.json');src=(P/captures['crates/adventuresim-stdb-module/src/inventory_container.rs']['capture']).read_bytes().splitlines(keepends=True)
    for x in comparison:
        a,z=x['current_range'];b=b''.join(src[a-1:z]);assert sha(b)==x['sha256'] and len(b)==x['bytes']
    print('PASS exact packet set/hashes, current source blobs and registered display coverage, historical entry selections, eight recovered originals, receipt attributions and current helper range hashes.')
    if args.git_repo:
        for x in captures.values():assert git(args.git_repo,'show',rr['source_revision']+':'+x['path'])==(P/x['capture']).read_bytes()
        for x in load('HISTORICAL-SOURCE-IDENTITIES.json'):
            b=git(args.git_repo,'show',x['revision']+':'+x['path']);assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes']
        for x,part in zip(h,bodies):
            b=git(args.git_repo,'show',x['revision']+':'+x['path']);assert sha(b)==x['stream_sha256'] and blob(b)==x['git_blob'] and len(b)==x['stream_bytes'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==part and json.loads(b)['entries'][2319]==q
        hr=load('RELATED-HISTORY-SELECTION-REGISTRATION.json');b=git(args.git_repo,'show',hr['revision']+':'+hr['path']);entries=json.loads(b)['entries'];hl=load('RELATED-HISTORY-LOCATOR-REGISTRATION.json');ho=load('RELATED-HISTORY-LOCATOR-RESULT.json');assert sha(b)==ho['stream_sha256'];assert {key:[i for i,v in enumerate(entries) if v.get('contract_id')==key] for key in hl['keys']}==ho['results']
        for x in load('RELATED-HISTORY-IDENTITIES.json'):
            part=(P/f"Q{x['index']}-ORIGINAL-EXACT-SLICE.json").read_bytes();assert b[x['start_byte']:x['end_byte_exclusive']]==part and entries[x['index']]==json.loads(part)
        ar=load('RELATED-REGISTRATION.json');ab=git(args.git_repo,'show',ar['revision']+':'+ar['path']);ai=load('RELATED-IDENTITY.json');assert sha(ab)==ai['sha256'] and blob(ab)==ai['blob'] and len(ab)==ai['bytes'] and ab==(P/ai['packet_path']).read_bytes()
        cr=load('RANGE-COMPARISON-REGISTRATION.json');old=git(args.git_repo,'show',cr['old_revision']+':'+cr['path'])
        for x in comparison:
            a,z=x['current_range'];piece=b''.join(src[a-1:z]);pos=[];i=0
            while True:
                i=old.find(piece,i)
                if i<0:break
                pos.append(old[:i].count(b'\n')+1);i+=1
            assert pos==x['old_match_lines'] and len(pos)==x['exact_match_count']
        def locate(rev,scopes,pattern,insensitive):
            cmd=['git','--no-replace-objects','-C',str(args.git_repo),'grep','-n']+(['-i'] if insensitive else [])+['-F',pattern,rev,'--',*scopes];z=subprocess.run(cmd,capture_output=True,text=True,env=dict(os.environ,GIT_NO_LAZY_FETCH='1'));out=[]
            for line in z.stdout.splitlines():
                a=line.split(':',3)
                if len(a)==4:
                    try:out.append({'path':a[1],'line':int(a[2])})
                    except ValueError:pass
            return z.returncode,out
        lr=load('LOCATOR-REGISTRATION.json')
        for key,v in load('LOCATOR-RESULT.json').items():
            group,pat=key.split(':',1);status,found=locate(lr[group+'_revision'],lr[group+'_scopes'],pat,True);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted']
        for label,insensitive in [('EXTENSION',True),('WRITER',True),('HELPER',False),('CONDITION',False)]:
            lr=load(f'LOCATOR-{label}-REGISTRATION.json')
            for pat,v in load(f'LOCATOR-{label}-RESULT.json').items():
                status,found=locate(lr['revision'],lr['scopes'],pat,insensitive);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted']
        lr=load('LOCATOR-CATALOG-REGISTRATION.json')
        for spec in lr['searches']:
            pat=spec['pattern'];v=load('LOCATOR-CATALOG-RESULT.json')[pat];status,found=locate(lr['revision'],spec['scopes'],pat,False);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted']
        print('PASS fixed Git source and queue membership, 6533/95e local helper comparisons, related assessment and bounded locator reproduction.')
    else:print('NOT REPEATED fixed Git membership, original/public full streams, old helper range matches, related assessment membership or locators: supply --git-repo.')
    if args.publication:
        tr=load('TRANSPORT-IDENTITY.json');b=(args.publication/tr['path']).read_bytes();assert sha(b)==tr['sha256'] and len(b)==tr['bytes'];nodes=[]
        def walk(v):
            if isinstance(v,dict):
                if 'sha256' in v:nodes.append(v)
                for x in v.values():walk(x)
            elif isinstance(v,list):
                for x in v:walk(x)
        walk(json.loads(gzip.decompress(b)))
        for x in load('TRANSPORT-MATCHES.json'):
            v=res[x['reference']];rel=x['reference'].removeprefix('target/726-audit-adler-20261027/');matches=[n for n in nodes if n.get('sha256')==v['actual_sha256'] and n.get('bytes')==v['bytes']];assert len(matches)==x['digest_size_matches'] and sum(str(n.get('path','')).endswith(rel) for n in matches)==x['path_suffix_matches']
        print('PASS eight published transport metadata matches; independent off-machine custody not verified.')
    else:print('NOT REPEATED transport inventory metadata comparison: supply --publication.')
    print('LIMITS: chronology is a process attestation; hashes and scans do not establish comprehension, semantic truth, supported reachability, runtime effects or off-machine custody.')

if __name__=='__main__':main()
