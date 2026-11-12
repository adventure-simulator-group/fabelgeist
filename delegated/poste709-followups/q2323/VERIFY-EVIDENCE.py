#!/usr/bin/env python3
"""Verify q2323 source evidence without importing or executing product code."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess

P=Path(__file__).resolve().parent

def load(n): return json.loads((P/n).read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def time(s): return dt.datetime.fromisoformat(s)
def safe(s): return not Path(s).is_absolute() and '..' not in Path(s).parts
def git(repo,*args): return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args],env=dict(os.environ,GIT_NO_LAZY_FETCH='1'))
def spans(v):
    out=[]
    for n in sorted(v):
        if not out or out[-1][1]+1!=n:out.append([n,n])
        else:out[-1][1]=n
    return out

def pointer(obj,ptr):
    for s in ptr.split('/')[1:]:
        s=s.replace('~1','/').replace('~0','~');obj=obj[int(s)] if isinstance(obj,list) else obj[s]
    return obj

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=Path);ap.add_argument('--publication',type=Path);ap.add_argument('--old-target',type=Path);args=ap.parse_args()
    m=load('MANIFEST.json');actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'};assert actual==set(m['files'])
    for name,x in m['files'].items():
        assert safe(name) and not (P/name).is_symlink();b=(P/name).read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes'],name
    rows=load('SOURCE-CAPTURES.json')+load('SOURCE-CACHE-CAPTURES.json')+[load('SOURCE-MHR-CAPTURE.json'),load('SOURCE-UPLOAD-CAPTURE.json'),load('SOURCE-GPU-CAPTURE.json')];captures={x['path']:x for x in rows};assert len(captures)==18
    for x in captures.values():
        b=(P/x['capture']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and len(b.splitlines())==x['eof']
    rr=load('SOURCE-READ-RECEIPT.json');seen={x:set() for x in captures}
    for x in rr['chunks']:
        r=load(x['registration']);assert time(r['registered_utc'])<=time(x['saved_utc']);assert any(v['path']==x['source_path'] and v['range'][0]<=x['start']<=x['end']<=v['range'][1] for v in r['sources']);l=(P/captures[x['source_path']]['capture']).read_text().splitlines();b=''.join(f'{i:4d} {l[i-1]}\n' for i in range(x['start'],x['end']+1)).encode();assert b==(P/x['display']).read_bytes() and sha(b)==x['sha256'];seen[x['source_path']].update(range(x['start'],x['end']+1))
    for x in rr['coverage']:
        assert x['eof']==captures[x['source_path']]['eof'] and x['personally_displayed']==spans(seen[x['source_path']]) and x['unread']==spans(set(range(1,x['eof']+1))-seen[x['source_path']])
    assert sum(map(len,seen.values()))==rr['unique_lines_personally_displayed']
    for x in load('OTHER-READ-RECEIPT.json')['chronology']:assert time(load(x['registration'])['registered_utc'])<=time(x['saved_utc'])
    h=load('HISTORY-IDENTITIES.json');bodies=[(P/x['packet_path']).read_bytes() for x in h];assert bodies[0]==bodies[1]
    for x,b in zip(h,bodies):assert sha(b)==x['entry_sha256'] and len(b)==x['entry_bytes']
    q=json.loads(bodies[0]);assert all(q[k]==v for k,v in load('Q2323-DISPLAY-SUMMARY.json').items());assert all(q[k]==v for k,v in load('Q2323-DISPLAY-DISPOSITION.json').items())
    qs=load('Q2323-SOURCES-0-1.json')+load('Q2323-SOURCES-2-4.json')+load('Q2323-SOURCES-5-7.json')+load('Q2323-SOURCES-8-10.json');assert qs==q['exact_source_identities_and_assessed_ranges']
    refs={}
    def add(path,digest):
        if path in refs:assert refs[path]==digest
        refs[path]=digest
    def collect(v):
        if isinstance(v,dict):
            if isinstance(v.get('path'),str) and isinstance(v.get('sha256'),str):add(v['path'],v['sha256'])
            if isinstance(v.get('verification'),str) and isinstance(v.get('verification_sha256'),str):add(v['verification'],v['verification_sha256'])
            for x in v.values():collect(x)
        elif isinstance(v,list):
            for x in v:collect(x)
    collect(q);reg=load('RECOVERY-REGISTRATION.json');res={x['reference']:x for x in load('RECOVERY-RESULT.json')};assert len(refs)==len(reg['references'])==len(res)==18
    for x in reg['references']:
        v=res[x['path']];b=(P/v['packet_path']).read_bytes();assert v['matched'] and refs[x['path']]==x['expected_sha256']==v['actual_sha256']==sha(b) and len(b)==v['bytes']
    more=load('RECOVERY-EXTENSION-REGISTRATION.json');more_results={x['reference']:x for x in load('RECOVERY-EXTENSION-RESULT.json')};assert len(more['refs'])==len(more_results)==2
    for x in more['refs']:
        v=more_results[x['path']];b=(P/v['packet_path']).read_bytes();assert v['matched'] and x['sha256']==v['actual_sha256']==sha(b) and len(b)==v['bytes']
    oldrec=load('UV-OLD-RECEIPT-IDENTITIES.json');oldbytes=(P/'historical-originals/adler/READ-L11-371.json').read_bytes();assert oldrec['copied_if_two_candidates_match'] and not oldrec['ledger_citation_had_expected_digest']
    for x in oldrec['candidates']:assert sha(oldbytes)==x['sha256'] and len(oldbytes)==x['bytes']
    fields=['registration','lens','revision','path','sha256','actual_EOF','prospective_ranges','fresh_personal_body_ranges','assessed_ranges','whole_file_assessed','attribution'];readers=[];roots={}
    for e in q['exact_source_identities_and_assessed_ranges']:
        rp=e['body_receipt']['path'].removeprefix('target/726-audit-adler-20261027/');pp=e['registration_ref_and_sha256']['path'].removeprefix('target/726-audit-adler-20261027/');d=load('historical-originals/'+rp);plan=load('historical-originals/'+pp);row=pointer(d,e['receipt_pointer']);pr=pointer(plan,e['registration_ref_and_sha256']['json_pointer']);root=load('historical-originals/'+e['root_adoption']['path']);mapping=[x for x in root['adopted_registration_mapping'] if x.get('registration')==row['registration']];readers.append({'source_row_key':e['source_row_key'],'receipt':rp,'read_pointer':e['receipt_pointer'],'read':{k:row[k] for k in fields if k in row},'plan':pp,'plan_pointer':e['registration_ref_and_sha256']['json_pointer'],'registered':{k:pr[k] for k in fields if k in pr},'root_record':e['root_adoption']['path'],'matching_root_mapping':mapping});roots[e['root_adoption']['path']]=e['root_adoption']['verification']
    assert readers==load('HISTORICAL-READERS-0-3.json')+load('HISTORICAL-READERS-4-7.json')+load('HISTORICAL-READERS-8-10.json')
    rootkeys=['lens','kind','batch','at','source_revision','actual_current_target','new_target_reconciliation_complete','other_lens_coverage_transferred','runtime_validation','implementation_gate_complete'];vkeys=['delegated_attribution_preserved','agent_attribution_preserved','other_lens_credit','product_execution','implementation_gate_complete','gate'];rs=[]
    for name,verify in roots.items():
        a=load('historical-originals/'+name);v=load('historical-originals/'+verify);rs.append({'root_record':name,'root_selected':{k:a[k] for k in rootkeys if k in a},'verification_record':verify,'verification_selected':{k:v[k] for k in vkeys if k in v}})
    assert rs==load('HISTORICAL-ROOTS-SELECT.json')
    proposal=load('historical-originals/parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-005.json')
    for i in [0,4]:assert all(proposal['contract_updates'][i][k]==v for k,v in load(f'HISTORICAL-PROPOSAL-SELECT-{i}.json').items())
    q27=(P/'Q2327-ORIGINAL-EXACT-SLICE.json').read_bytes();qi=load('Q2327-IDENTITY.json');assert sha(q27)==qi['sha256'] and len(q27)==qi['bytes'];q27obj=json.loads(q27);assert all(q27obj[k]==v for k,v in load('Q2327-SELECT.json').items())
    rel=load('related/ASSESSMENT-DELEGATED-RECONCILIATION-95E.json');sel=load('RELATED-CENTRAL-SELECT.json');assert sel['delegated_observation_19']==rel['delegated_observations'][19] and all(rel[k]==v for k,v in sel.items() if k!='delegated_observation_19')
    rig=load('related/delegated/rig-identity-report.json');rs=load('RELATED-RIG-SELECT.json');assert rs['reader']==rig['reader'] and rs['pinned_revision']==rig['pinned_authoritative_working_source_revision'] and rs['finding_2']==rig['findings_and_open_questions'][2] and rs['status']==rig['status']
    for name,summary,cand in [('ASSESSMENT-L01-DEVICE-CACHE-95E.json','RELATED-DEVICE-CACHE-SUMMARY.json','RELATED-DEVICE-CACHE-CANDIDATE.json'),('ASSESSMENT-L01-CORRECTIVE-CACHE-95E.json','RELATED-CORRECTIVE-SUMMARY.json','RELATED-CORRECTIVE-CANDIDATE.json')]:
        r=load('related/'+name);assert all(r[k]==v for k,v in load(summary).items());assert all(r['candidates'][0][k]==v for k,v in load(cand).items())
    u=load('related/ASSESSMENT-L01-SURFACE-CUT-95E.json');assert all(u[k]==v for k,v in load('RELATED-UV-SUMMARY.json').items());assert len(u['candidates'])==len(load('RELATED-UV-CANDIDATES.json'));assert all(all(v[k]==z for k,z in sel.items()) for v,sel in zip(u['candidates'],load('RELATED-UV-CANDIDATES.json')))
    for x in load('UV-HISTORICAL-RECONSTRUCTION-IDENTITIES.json'):
        b=(P/x['packet_path']).read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes']
    uv0=load('UV-HISTORICAL-RECONSTRUCTED-0.json');assert all(uv0[k]==v for k,v in load('UV-HISTORICAL-SELECT-0.json').items())
    for i in [1,2,3]:
        x=load(f'UV-HISTORICAL-RECONSTRUCTED-{i}.json');sel=load(f'UV-HISTORICAL-SELECT-{i}.json');assert all(x[k]==v for k,v in sel.items() if k!='disposition') and all(x['disposition'][k]==v for k,v in sel['disposition'].items())
    old=json.loads(oldbytes);sel=load('UV-OLD-RECEIPT-SELECT.json');assert all(old[k]==v for k,v in sel.items() if k not in ['matching_dispositions','sources_metadata']);assert sel['sources_metadata']==old['sources'] and sel['matching_dispositions']==[v for v in old['role_dispositions'] if 'MHR-TOPOLOGY-ADMISSION' in json.dumps(v)] and load('UV-OLD-ROLE-SELECT.json')==old['role_dispositions'][2]
    root=load('historical-originals/segovb/READ-L11-540.json');ver=load('historical-originals/segovb/VERIFY-L11-540.json');sel=load('UV-ROOT-SELECT.json');assert all(root[k]==v for k,v in sel['root_selected'].items()) and all(ver[k]==v for k,v in sel['verification_selected'].items()) and sel['mapping_for_character']==[x for x in root['adopted_registration_mapping'] if 'crates/fabelgeist-mhr/src/character.rs' in json.dumps(x)]
    print('PASS exact packet set/hashes, current source blobs and registered displays, q2323/q2327 selections, 20 hash-cited historical originals, qualified older receipt, reader/root attribution and reconstructed ledger selections.')
    if args.git_repo:
        rev=rr['source_revision']
        for x in captures.values():assert git(args.git_repo,'show',rev+':'+x['path'])==(P/x['capture']).read_bytes()
        for x in load('SOURCE-IDENTITIES.json'):
            for rev,v in x['revisions'].items():
                b=git(args.git_repo,'show',rev+':'+x['path']);assert sha(b)==v['sha256'] and blob(b)==v['blob'] and len(b)==v['bytes']
            assert x['old_matches_historical_record'] and x['old_current_sha_equal']
        ex=load('SOURCE-EXTRA-COMPARISON-REGISTRATION.json')
        for x in load('SOURCE-EXTRA-COMPARISON-RESULT.json'):
            b=git(args.git_repo,'show',ex['old_revision']+':'+x['path']);assert sha(b)==x['old_sha256'] and blob(b)==x['old_blob'] and len(b)==x['old_bytes'] and (b==(P/captures[x['path']]['capture']).read_bytes())==x['bytes_equal']
        streams={}
        for x,part in zip(h,bodies):
            b=git(args.git_repo,'show',x['revision']+':'+x['path']);streams[x['role']]=b;assert sha(b)==x['stream_sha256'] and blob(b)==x['git_blob'] and len(b)==x['stream_bytes'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==part and json.loads(b)['entries'][2323]==q
        assert streams['original'][qi['start_byte']:qi['end_byte_exclusive']]==q27 and json.loads(streams['original'])['entries'][2327]==q27obj
        rh=load('UV-HISTORICAL-SELECTION-REGISTRATION.json');lb=git(args.git_repo,'show',rh['revision']+':'+rh['path']);loc=load('UV-HISTORICAL-LOCATOR2-RESULT.json');assert sha(lb)==loc['stream_sha256'] and len(lb)==loc['stream_bytes'];ledger=json.loads(lb)
        for x in load('UV-HISTORICAL-RECONSTRUCTION-IDENTITIES.json'):assert pointer(ledger,x['pointer'])==load(x['packet_path'])
        for x in load('RELATED-IDENTITIES.json'):
            ar=load('RELATED-REGISTRATION.json');b=git(args.git_repo,'show',ar['revision']+':'+x['path']);assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and b==(P/x['packet_path']).read_bytes()
        for label in ['CORRECTIVE','UV']:
            ar=load(f'RELATED-{label}-REGISTRATION.json');x=load(f'RELATED-{label}-IDENTITY.json');b=git(args.git_repo,'show',ar['revision']+':'+ar['path']);assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and b==(P/x['packet_path']).read_bytes()
        def locate(rev,scopes,pat,insensitive):
            cmd=['git','--no-replace-objects','-C',str(args.git_repo),'grep','-n']+(['-i'] if insensitive else [])+['-F',pat,rev,'--',*scopes];z=subprocess.run(cmd,capture_output=True,text=True,env=dict(os.environ,GIT_NO_LAZY_FETCH='1'));out=[]
            for line in z.stdout.splitlines():
                a=line.split(':',3)
                if len(a)==4:
                    try:out.append({'path':a[1],'line':int(a[2])})
                    except ValueError:pass
            return z.returncode,out
        for label in ['','UV-']:
            lr=load('LOCATOR-'+label+'REGISTRATION.json');out=load('LOCATOR-'+label+'RESULT.json')
            for key,v in out.items():
                group,pat=key.split(':',1);status,found=locate(lr[group+'_revision'],lr[group+'_scopes'],pat,True);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted']
        for label,insensitive in [('CACHE',False),('RELATED-CACHE',True),('MHR',False),('CONSTRUCTOR',False),('NEW',False),('GPU',False)]:
            lr=load(f'LOCATOR-{label}-REGISTRATION.json')
            for pat,v in load(f'LOCATOR-{label}-RESULT.json').items():
                status,found=locate(lr['revision'],lr['scopes'],pat,insensitive);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted']
        print('PASS fixed e275/95e source memberships, original/public 059 streams, original q2327, semantic ledger reconstructions, related records and bounded fixed-string locators.')
    else:print('NOT REPEATED Git memberships, whole 059 streams, original ledger reconstruction comparisons, historical source equality or locator results: supply --git-repo.')
    if args.publication:
        tr=load('TRANSPORT-IDENTITY.json');b=(args.publication/tr['path']).read_bytes();assert sha(b)==tr['sha256'] and len(b)==tr['bytes'];nodes=[]
        def walk(v):
            if isinstance(v,dict):
                if 'sha256' in v:nodes.append(v)
                for x in v.values():walk(x)
            elif isinstance(v,list):
                for x in v:walk(x)
        walk(json.loads(gzip.decompress(b)));allres={**res,**more_results}
        for x in load('TRANSPORT-MATCHES.json'):
            v=allres[x['reference']];rel=x['reference'].removeprefix('target/726-audit-adler-20261027/');ms=[n for n in nodes if n.get('sha256')==v['actual_sha256'] and n.get('bytes')==v['bytes']];assert len(ms)==x['digest_size_matches'] and sum(str(n.get('path','')).endswith(rel) for n in ms)==x['path_suffix_matches']
        for x in oldrec['candidates']:
            ms=[n for n in nodes if n.get('sha256')==x['sha256'] and n.get('bytes')==x['bytes']];assert len(ms)==x['inventory_digest_size_matches'] and sum(str(n.get('path','')).endswith('adler/READ-L11-371.json') for n in ms)==x['inventory_basename_suffix_matches']
        print('PASS transport metadata for 20 hash-cited records and qualified older receipt; independent off-machine custody not verified.')
    else:print('NOT REPEATED published transport metadata comparison: supply --publication.')
    if args.old_target:
        for x in oldrec['candidates']:
            b=(args.old_target/x['relative_candidate']).read_bytes();assert b==oldbytes
        print('PASS two local older receipt candidates match retained bytes; original ledger citation itself did not provide an expected digest.')
    else:print('NOT REPEATED original two local older receipt candidates: supply --old-target.')
    print('LIMITS: timestamps are process attestations; hashes/scans do not prove semantic truth, comprehension, supported assets, native or runtime behavior, or independent custody.')

if __name__=='__main__':main()
