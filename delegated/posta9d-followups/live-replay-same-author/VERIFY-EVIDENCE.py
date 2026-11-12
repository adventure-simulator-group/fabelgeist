#!/usr/bin/env python3
"""Verify source and observation evidence; never run the historical driver."""
import argparse
import datetime as dt
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import subprocess

P=Path(__file__).resolve().parent

def load(n):return json.loads((P/n).read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def timestamp(s):return dt.datetime.fromisoformat(s)
def git(repo,*args,input=None):
    env=os.environ.copy();env['GIT_NO_LAZY_FETCH']='1';env['GIT_OPTIONAL_LOCKS']='0'
    return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args],input=input,env=env)
def spans(ns):
    out=[]
    for n in sorted(ns):
        if not out or out[-1][1]+1!=n:out.append([n,n])
        else:out[-1][1]=n
    return out

def tree_index(repo,rev):
    out={}
    for r in git(repo,'ls-tree','-r','-z',rev).split(b'\0'):
        if r:
            h,path=r.split(b'\t',1);mode,typ,oid=h.decode().split();out[path.decode()]={'mode':mode,'type':typ,'git_blob':oid}
    return out

def batch_blob_hashes(repo,oids):
    ids=sorted(oids);b=git(repo,'cat-file','--batch',input=('\n'.join(ids)+'\n').encode());pos=0;out={}
    for wanted in ids:
        z=b.index(b'\n',pos);h=b[pos:z].decode().split();assert len(h)==3 and h[0]==wanted and h[1]=='blob';n=int(h[2]);raw=b[z+1:z+1+n];assert len(raw)==n and b[z+1+n:z+2+n]==b'\n';out[wanted]={'bytes':n,'sha256':sha(raw)};pos=z+2+n
    assert pos==len(b);return out

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=Path);ap.add_argument('--recovery-repo',type=Path);a=ap.parse_args();m=load('MANIFEST.json');actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name!='MANIFEST.json'};assert actual==set(m['files'])
    for name,v in m['files'].items():
        path=Path(name);assert not path.is_absolute() and '..' not in path.parts and not (P/path).is_symlink();b=(P/path).read_bytes();assert sha(b)==v['sha256'] and len(b)==v['bytes'],name
    report=load('REPORT-REGISTRATION.json');b=(P/'INPUT-REPORT.md').read_bytes();assert sha(b)==report['working_sha256'] and len(b)==report['bytes'];lines=b.decode().splitlines()
    for name,start,end in [('INPUT-REPORT-DISPLAY-A.txt',1,56),('INPUT-REPORT-DISPLAY-B.txt',57,112)]:assert (P/name).read_text()==''.join(f'{i}: {line}\n' for i,line in enumerate(lines,1) if start<=i<=end)
    reg=load('TOOL-SOURCE-REGISTRATION.json');by={e['capture'].split('/')[-1]:e for e in reg['entries']};seen={n:set() for n in by}
    for e in reg['entries']:
        b=(P/e['capture']).read_bytes();assert sha(b)==e['sha256'] and len(b)==e['bytes'] and len(b.splitlines())==e['eof'];assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==e['git_blob']
    receipt=load('TOOL-READ-RECEIPT.json')
    for c in receipt['chunks']:
        assert timestamp(reg['registered_utc'])<=timestamp(c['saved_utc']);out=''
        for s in c['sections']:
            e=by[s['file']];assert any(a<=s['start']<=s['end']<=z for a,z in e['planned_ranges']);lines=(P/e['capture']).read_text().splitlines();out+=f"### {s['file']}:{s['start']}-{s['end']}\n"+''.join(f'{i}: {line}\n' for i,line in enumerate(lines,1) if s['start']<=i<=s['end']);seen[s['file']].update(range(s['start'],s['end']+1))
        assert (P/c['display']).read_text()==out and sha(out.encode())==c['sha256']
    for c in receipt['coverage']:
        n=c['file'];assert spans(seen[n])==c['actual_credited'] and spans(set(range(1,c['eof']+1))-seen[n])==c['unread']
    assert sum(map(len,seen.values()))==receipt['total_unique_source_lines_credited']
    for c in load('OTHER-READ-RECEIPT.json')['chronology']:assert timestamp(load(c['registration'])['registered_utc'])<=timestamp(c['saved_utc'])
    assert 'truncated' in load('LOCATOR-QUALIFICATION.json')['events'][0]['kind'];assert load('REPORT-REGISTRATION.json')['working_equals_head'] is True
    print('PASS packet file set/hashes, registered fixed-report displays, nine evidence-source blobs, display scope/complements and recorded chronology. Same author limitation remains.')
    if a.git_repo:
        rev=report['observed_publication_head'];assert git(a.git_repo,'show',f"{rev}:{report['path']}")==(P/'INPUT-REPORT.md').read_bytes()
        for e in reg['entries']:
            assert git(a.git_repo,'show',f"{e['revision']}:{e['git_path']}")== (P/e['capture']).read_bytes();old=git(a.git_repo,'show',f"{e['older_revision']}:{e['git_path']}");assert sha(old)==e['older_sha256'] and (old==(P/e['capture']).read_bytes())==e['older_bytes_equal']
        for f in load('DESIGN-METADATA-REGISTRATION.json')['files']:
            b=git(a.git_repo,'show',f"{rev}:{f['path']}");assert sha(b)==f['sha256'] and len(b)==f['bytes']
        for f in load('PRIVACY-STATUS-REGISTRATION.json')['files']:
            b=git(a.git_repo,'show',f"{rev}:{f['path']}");assert sha(b)==f['sha256'] and json.loads(b)==load('PRIVACY-STATUS-DISPLAY.json')[f['path']]
        tree=tree_index(a.git_repo,rev);stage='delegated/qualified-059-successors';early='delegated/qualified-059-early-links';x=json.loads(git(a.git_repo,'show',f'{rev}:{stage}/DRIVER-INTEGRATION.json'));y=json.loads(git(a.git_repo,'show',f'{rev}:{early}/INVENTORY.json'));repl={v['original']:stage+'/'+v['successor'] for v in x['exact_replacements']};assert len(repl)==15 and set(repl)=={'verify_reconciliation_proposal_links_15.py'}|{f'verify_delegated_links_{i}.py' for i in range(16,30)};assert {v['original'] for v in y['rows']}=={f'verify_delegated_links_{i}.py' for i in range(11,16)};repl.update({v['original']:early+'/'+v['successor'] for v in y['rows']});assert len(repl)==20
        first=['verify_recovered.py','verify_selected_states.py','verify_lock_inventory.py','verify_delta_sources.py','verify_role_worklist.py'];opt={'verify_local_checkpoint_059.py','verify_recovered_public_redacted_plus2.py'};root=sorted(n for n in tree if '/' not in n and n.startswith('verify_') and n.endswith('.py'));native=sorted(n for n in tree if len(n.split('/'))==3 and n.startswith('native-evidence/') and n.endswith('/verify.py'));remaining=sorted(n for n in root if n not in first and n not in opt);pred=[]
        for name in first+remaining+native:
            if name=='verify_recovered.py':target='verify_recovered_public_redacted_plus2.py';kind='qualified-five-object-successor'
            elif name in repl:target=repl[name];kind='qualified-059-successor'
            elif name=='verify_city_streaming_readiness_95e.py':target='delegated/city-streaming-privacy-proposal/candidate/verify_city_streaming_readiness_redacted_95e.py';kind='qualified-city-successor'
            else:target=name;kind='direct'
            pred.append({'logical_name':name,'selected_script':target,'selection':kind,**tree[target]})
        info=batch_blob_hashes(a.git_repo,{r['git_blob'] for r in pred})
        for r in pred:r.update(info[r['git_blob']])
        suite=load('DRIVER-SUITE-INVENTORY.json');assert pred==suite['rows'] and len(root)==suite['root_glob_count'] and len(native)==suite['native_glob_count'] and len(pred)==suite['selected_invocation_count'] and sum(r['selection']!='direct' for r in pred)==suite['qualified_replacement_count']
        print('PASS fixed candidate Git report/tool bytes, earlier-tool comparisons, design metadata, privacy status and reconstructed selected-driver order, substitutions and script SHA-256 identities.')
    else:print('NOT REPEATED fixed Git commit/source membership, privacy status or selected-driver Git tree/blob reconstruction: supply --git-repo.')
    if a.recovery_repo:
        obs=load('OBJECT-HEADER-DISPLAY.json');tree=tree_index(a.recovery_repo,obs['candidate']);oids=sorted({v['git_blob'] for v in tree.values() if v['type']=='blob'});raw=git(a.recovery_repo,'cat-file','--batch-check=%(objectname) %(objecttype) %(objectsize)',input=('\n'.join(oids)+'\n').encode()).decode().splitlines();assert len(raw)==len(oids);sizes={};missing=0
        for oid,line in zip(oids,raw):
            v=line.split();assert v[0]==oid
            if len(v)==2 and v[1]=='missing':missing+=1
            else:assert len(v)==3 and v[1]=='blob';sizes[oid]=int(v[2])
        assert len(tree)==obs['tracked_entry_count'] and len(oids)==obs['unique_blob_count'] and len(sizes)==obs['locally_available_blob_headers'] and missing==obs['missing_blob_headers'] and sum(sizes[v['git_blob']] for v in tree.values() if v['git_blob'] in sizes)==obs['summed_path_blob_bytes_from_headers']
        now=git(a.recovery_repo,'rev-parse','HEAD').decode().strip();record=load('DISCOVERY-COMPARISON-DISPLAY.json')
        if now==record['checkout_observed_head']:
            actual={'root_verifiers':sorted(x.relative_to(a.recovery_repo).as_posix() for x in a.recovery_repo.glob('verify_*.py')),'native_verifiers':sorted(x.relative_to(a.recovery_repo).as_posix() for x in (a.recovery_repo/'native-evidence').glob('*/verify.py')),'root_checkpoints':sorted(x.relative_to(a.recovery_repo).as_posix() for x in a.recovery_repo.glob('DELEGATED-EVIDENCE-CHECKPOINT*.json'))}
            assert all(len(v)==record['sections'][k]['checkout_count'] and sha(('\n'.join(v)+'\n').encode())==record['sections'][k]['checkout_name_sha256'] for k,v in actual.items());print('PASS recovery candidate object-header counts and repeatable discovery-name snapshot at the recorded HEAD.')
        else:print('PASS recovery candidate object-header counts; current HEAD differs from recorded observation, so discovery snapshot was not repeated.')
    else:print('NOT REPEATED recovery-store object-header or observed discovery checks: supply --recovery-repo.')
    print('LIMITS: Same-author challenge only. No historical driver was run; chronology is custody metadata; script hashes and object headers do not prove complete dynamic inputs, byte-exact checkout, semantics, privacy, runtime behavior or independent custody.')

if __name__=='__main__':main()
