#!/usr/bin/env python3
"""Verify a registered evidence packet, without executing product code."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

P = Path(__file__).resolve().parent

def load(n): return json.loads((P / n).read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def time(s): return dt.datetime.fromisoformat(s)
def canonical(v): return json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def git(repo, *args): return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args])
def spans(ns):
    o=[]
    for n in sorted(ns):
        if not o or o[-1][1]+1 != n: o.append([n,n])
        else: o[-1][1]=n
    return o

def allowed(reg, tag):
    x=load(reg); out=[]
    for e in x.get('entries',[])+x.get('new_entries',[]):
        if e['tag']==tag: out+=e['planned_ranges']
    for e in x.get('existing_source_ranges',[]):
        if e['tag']==tag: out+=e['additional_ranges']
    if x.get('tag')==tag:
        for key in ['additional_range','corrected_range']:
            if key in x: out.append(x[key])
    return out

def main():
    a=argparse.ArgumentParser();a.add_argument('--git-repo',type=Path);a.add_argument('--publication',type=Path);a=a.parse_args()
    m=load('MANIFEST.json');actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x.name!='MANIFEST.json'};assert actual==set(m['files'])
    for name,v in m['files'].items():
        path=Path(name);assert not path.is_absolute() and '..' not in path.parts and not (P/path).is_symlink();b=(P/path).read_bytes();assert sha(b)==v['sha256'] and len(b)==v['bytes'],name
    entries=[]
    for n in ['SOURCE-REGISTRATION.json','SOURCE-EXTENSION-REGISTRATION.json','SOURCE-EXTENSION2-REGISTRATION.json','SOURCE-CONSUMER-REGISTRATION.json']:
        r=load(n);entries+=r.get('entries',[])+r.get('new_entries',[])
    sources={e['capture'].split('/')[-1]:e for e in entries};seen={n:set() for n in sources}
    for e in entries:
        b=(P/e['capture']).read_bytes();assert sha(b)==e['sha256'] and len(b)==e['bytes'] and len(b.splitlines())==e['eof'];assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==e['git_blob']
    receipt=load('SOURCE-READ-RECEIPT.json')
    for chunk in receipt['chunks']:
        out=''
        for s in chunk['sections']:
            e=sources[s['file']];start=s['start'];end=s['requested_end'];reg=load(s['registration']);assert time(reg['registered_utc'])<=time(chunk['saved_utc']);assert any(lo<=start<=end<=hi for lo,hi in allowed(s['registration'],e['tag']));assert s['actual_end']==min(end,e['eof']);lines=(P/e['capture']).read_text().splitlines();out+=f"### {s['file']}:{start}-{end}\n"+''.join(f'{i}: {line}\n' for i,line in enumerate(lines,1) if start<=i<=end)
            if s['credited']:assert end<=e['eof'];seen[s['file']].update(range(start,end+1))
            else:assert chunk['display']=='SOURCE-DISPLAY-M.txt' and s['file']=='state.rs' and end>e['eof']
        assert (P/chunk['display']).read_text()==out and sha(out.encode())==chunk['sha256']
    for c in receipt['coverage']:
        n=c['file'];assert spans(seen[n])==c['actual_credited'] and spans(set(range(1,c['eof']+1))-seen[n])==c['unread']
    assert sum(map(len,seen.values()))==receipt['total_unique_current_source_lines_credited']
    for c in load('OTHER-READ-RECEIPT.json')['chronology']:assert time(load(c['registration'])['registered_utc'])<=time(c['saved_utc'])
    h=load('HISTORY-REGISTRATION.json');original=(P/'Q2305-ORIGINAL-EXACT-SLICE.json').read_bytes();public=(P/'Q2305-PUBLIC-EXACT-SLICE.json').read_bytes();assert (original==public)==h['original_and_public_bytes_equal'];q=json.loads(original)
    for stream,b in zip(h['streams'],[original,public]):assert len(b)==stream['entry_bytes'] and sha(b)==stream['entry_sha256']
    assert all(q[k]==v for k,v in load('Q2305-DISPLAY-A.json').items()) and q['exact_source_identities_and_assessed_ranges']==load('Q2305-DISPLAY-B.json')
    recovery=load('RECOVERY-REGISTRATION.json');results={e['historical_relative_path']:e for e in load('RECOVERY-RESULTS.json')['results']};refs={}
    def add(path,digest):
        path=path.removeprefix('target/726-audit-adler-20261027/');refs.setdefault(path,digest);assert refs[path]==digest
    for row in q['exact_source_identities_and_assessed_ranges']:
        add(row['body_receipt']['path'],row['body_receipt']['sha256']);add(row['registration_ref_and_sha256']['path'],row['registration_ref_and_sha256']['sha256']);ad=row['root_adoption'];add(ad['path'],ad['sha256']);add(ad['verification'],ad['verification_sha256'])
    add(q['proposal']['path'],q['proposal']['sha256']);assert len(refs)==len(recovery['records'])==16
    for e in recovery['records']:
        path=e['historical_relative_path'];assert refs[path]==e['expected_sha256'];r=results[path];b=(P/r['retained']).read_bytes();assert r['available'] and r['expected_hash_matches'] and sha(b)==e['expected_sha256']==r['observed_sha256'] and len(b)==r['bytes']
    fields=['registration','revision','path','sha256','actual_EOF','fresh_personal_body_ranges','assessed_ranges','whole_file_assessed','attribution'];readers=[];adoptions=[]
    for e in q['exact_source_identities_and_assessed_ranges']:
        path=e['body_receipt']['path'].removeprefix('target/726-audit-adler-20261027/');data=json.loads((P/'historical-originals'/path).read_text());row=data['rows'][int(e['receipt_pointer'].split('/')[-1])];readers.append({'record':path,'selected_pointer':e['receipt_pointer'],'row':{k:row[k] for k in fields if k in row}})
        ad=e['root_adoption'];d=json.loads((P/'historical-originals'/ad['path']).read_text());v=json.loads((P/'historical-originals'/ad['verification']).read_text());adoptions.append({'adoption_record':ad['path'],'registration':row['registration'],'matching_adoption':[z for z in d.get('adopted_registration_mapping',[]) if z.get('registration')==row['registration']],'verification_record':ad['verification'],'verification_selected':{k:v[k] for k in ['delegated_attribution_preserved','agent_attribution_preserved','product_execution','implementation_gate_complete','other_lens_credit','gate'] if k in v}})
    assert readers[:2]==load('HISTORY-READERS-DISPLAY-A.json') and readers[2:]==load('HISTORY-READERS-DISPLAY-B.json') and adoptions==load('HISTORY-ADOPTION-DISPLAY.json')
    proposal=json.loads((P/'historical-originals'/q['proposal']['path']).read_text())['contract_updates'][5];assert all(proposal[k]==v for k,v in load('HISTORY-PROPOSAL-DISPLAY.json').items())
    print('PASS packet file set/hashes, current Git blob identities, registered displays/complements including corrected EOF, exact q2305 slices and 16 original historical record hashes/attribution reconstructions.')
    if a.git_repo:
        for e in entries:
            b=git(a.git_repo,'show',f"{e['revision']}:{e['git_path']}");assert b==(P/e['capture']).read_bytes();line=git(a.git_repo,'ls-tree',e['revision'],'--',e['git_path']).decode().strip();assert line.split('\t')[0].split()[2]==e['git_blob'];old=git(a.git_repo,'show',f"{e['historical_revision']}:{e['git_path']}");assert sha(old)==e['historical_sha256'] and (old==b)==e['historical_bytes_equal']
        ik=load('HISTORICAL-IK-COMPARISON.json');e=next(z for z in entries if z['tag']=='ik');old=git(a.git_repo,'show',f"{e['historical_revision']}:{e['git_path']}").splitlines(keepends=True);new=(P/e['capture']).read_bytes().splitlines(keepends=True);start,end=ik['compared_lines'];ob=b''.join(old[start-1:end]);nb=b''.join(new[start-1:end]);assert sha(ob)==ik['historical_range_sha256'] and sha(nb)==ik['current_range_sha256'] and (ob==nb)==ik['range_bytes_equal']
        for stream,part in zip(h['streams'],[original,public]):
            b=git(a.git_repo,'show',f"{stream['revision']}:{stream['path']}");assert sha(b)==stream['stream_sha256'] and len(b)==stream['stream_bytes'] and b[stream['entry_start_byte']:stream['entry_end_byte_exclusive']]==part and json.loads(b)['entries'][2305]==q
        related=load('RELATED-HISTORY-REGISTRATION.json');qb=git(a.git_repo,'show',f"{related['revision']}:{related['path']}");row=json.loads(qb)['entries'][45];assert sha(canonical(row))==related['selected_canonical_sha256'] and all(row[k]==v for k,v in load('HISTORY-Q45-DISPLAY.json').items())
        ar=load('ASSESSMENT-REGISTRATION.json');data={}
        for f in ar['files']:
            b=git(a.git_repo,'show',f"{f['revision']}:{f['path']}");assert sha(b)==f['sha256'] and len(b)==f['bytes'];data[f['path']]=json.loads(b)
        metric=data['ASSESSMENT-REPORT-METRIC-SUPPORT-95E.json'];assert all(metric[k]==v for k,v in load('ASSESSMENT-METRIC-DISPLAY.json').items())
        for x in load('ASSESSMENT-DELEGATED-DISPLAY.json'):
            d=data[x['file']];index=14 if x['file'].endswith('-9-95E.json') else 3;assert x['source_revision']==d['source_revision'] and x['delegated_observation']==d['delegated_observations'][index];assert all(x[k]==d[k] for k in ['central_decisions','new_qualified_ledger_rows','open','repair','validation'])
        print('PASS fixed current and historical Git sources, IK range comparison, original/public 059 queue streams, original q45 and committed related assessment selections.')
    else:print('NOT REPEATED Git commit membership, e275/95e comparisons, original/public queue streams, q45 or prior committed assessment bodies: supply --git-repo.')
    if a.publication:
        tr=load('TRANSPORT-REGISTRATION.json');b=(a.publication/tr['path']).read_bytes();assert sha(b)==tr['sha256'] and len(b)==tr['bytes'];nodes=[]
        def walk(x):
            if isinstance(x,dict):
                if 'sha256' in x:nodes.append(x)
                for v in x.values():walk(v)
            elif isinstance(x,list):
                for v in x:walk(v)
        walk(json.loads(gzip.decompress(b)))
        for row in load('TRANSPORT-MATCHES.json')['rows']:
            e=results[row['historical_relative_path']];matches=[n for n in nodes if n.get('sha256')==e['observed_sha256'] and n.get('bytes')==e['bytes']];assert len(matches)==row['digest_size_matches'] and sum(str(n.get('path','')).endswith(e['historical_relative_path']) for n in matches)==row['path_suffix_matches']
        print('PASS published transport inventory metadata for all 16 recovered files; off-machine custody not verified.')
    else:print('NOT REPEATED published transport inventory comparison: supply --publication.')
    print('LIMITS: chronology is recorded custody metadata; checksums do not prove personal comprehension, source semantics, supported reachability, native schedule or runtime behavior.')

if __name__=='__main__':main()
