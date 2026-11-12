#!/usr/bin/env python3
"""Verify source-evidence identity; never execute product code or archive contents."""
import argparse
import datetime as dt
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tomllib

P=Path(__file__).resolve().parent

def load(n): return json.loads((P/n).read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def timestamp(s): return dt.datetime.fromisoformat(s)
def canonical(v): return json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def git(repo,*args): return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args])
def spans(ns):
    out=[]
    for n in sorted(ns):
        if not out or out[-1][1]+1!=n:out.append([n,n])
        else:out[-1][1]=n
    return out

def check_manifest(root):
    m=json.loads((root/'MANIFEST.json').read_text());actual={str(f.relative_to(root)) for f in root.rglob('*') if f.is_file() and f.name!='MANIFEST.json'};assert actual==set(m['files'])
    for name,v in m['files'].items():
        path=Path(name);assert not path.is_absolute() and '..' not in path.parts and not (root/path).is_symlink();b=(root/path).read_bytes();assert len(b)==v['bytes'] and sha(b)==v['sha256'],name

def check_package(archive, verification, member):
    assert sha(archive)==verification['archive_sha256'] and len(archive)==verification['archive_bytes'] and sha(archive)==verification['locked_package_checksum'] if 'locked_package_checksum' in verification else sha(archive)==verification['archive_sha256'] and len(archive)==verification['archive_bytes']
    with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
        matches=[x for x in t.getmembers() if x.name==verification['archive_member']];assert len(matches)==1 and matches[0].isfile();f=t.extractfile(matches[0]);assert f is not None;inside=f.read()
    assert inside==member and sha(member)==verification['member_sha256'] and len(member)==verification['member_bytes']

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=Path);ap.add_argument('--publication',type=Path);ap.add_argument('--predecessor',type=Path);ap.add_argument('--raw-archive',type=Path);a=ap.parse_args();check_manifest(P)
    boundary=load('PRIVACY-BOUNDARY.json');excluded=boundary['excluded_from_safe_packet'];assert not (P/excluded['path']).exists()
    sources=[]
    for name in ['SOURCE-REGISTRATION.json','SOURCE-EXTENSION-REGISTRATION.json']:sources+=load(name)['entries']
    by={e['capture'].split('/')[-1]:e for e in sources};seen={n:set() for n in by}
    for e in sources:
        b=(P/e['capture']).read_bytes();assert sha(b)==e['sha256'] and len(b)==e['bytes'] and len(b.splitlines())==e['eof'];assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==e['git_blob']
    receipt=load('SOURCE-READ-RECEIPT.json')
    for c in receipt['chunks']:
        out=''
        for s in c['sections']:
            e=by[s['file']];reg=load(s['registration']);assert timestamp(reg['registered_utc'])<=timestamp(c['saved_utc']);assert any(lo<=s['start']<=s['end']<=hi for lo,hi in e['planned_ranges']);lines=(P/e['capture']).read_text().splitlines();out+=f"### {s['file']}:{s['start']}-{s['end']}\n"+''.join(f'{i}: {line}\n' for i,line in enumerate(lines,1) if s['start']<=i<=s['end']);seen[s['file']].update(range(s['start'],s['end']+1))
        assert (P/c['display']).read_text()==out and sha(out.encode())==c['sha256']
    for c in receipt['coverage']:
        n=c['file'];assert spans(seen[n])==c['actual_credited'] and spans(set(range(1,c['eof']+1))-seen[n])==c['unread']
    assert sum(map(len,seen.values()))==receipt['total_unique_source_lines_credited']
    for c in load('OTHER-READ-RECEIPT.json')['chronology']:assert timestamp(load(c['registration'])['registered_utc'])<=timestamp(c['saved_utc'])
    qh=load('HISTORY-REGISTRATION.json');o=(P/'Q2309-ORIGINAL-EXACT-SLICE.json').read_bytes();pub=(P/'Q2309-PUBLIC-EXACT-SLICE.json').read_bytes();assert (o==pub)==qh['original_and_public_bytes_equal'];q=json.loads(o)
    for stream,b in zip(qh['streams'],[o,pub]):assert sha(b)==stream['entry_sha256'] and len(b)==stream['entry_bytes']
    assert all(q[k]==v for k,v in load('Q2309-DISPLAY-A.json').items()) and q['exact_source_identities_and_assessed_ranges']==load('Q2309-DISPLAY-B.json')
    recovery=load('RECOVERY-REGISTRATION.json');results={e['historical_relative_path']:e for e in load('RECOVERY-RESULTS.json')['results']};refs={}
    def add(path,d):
        path=path.removeprefix('target/726-audit-adler-20261027/');refs.setdefault(path,d);assert refs[path]==d
    for row in q['exact_source_identities_and_assessed_ranges']:
        add(row['body_receipt']['path'],row['body_receipt']['sha256']);add(row['registration_ref_and_sha256']['path'],row['registration_ref_and_sha256']['sha256']);r=row['root_adoption'];add(r['path'],r['sha256']);add(r['verification'],r['verification_sha256'])
    add(q['proposal']['path'],q['proposal']['sha256']);assert len(refs)==len(recovery['records'])==5
    for e in recovery['records']:
        path=e['historical_relative_path'];r=results[path];b=(P/r['retained']).read_bytes();assert refs[path]==e['expected_sha256']==sha(b)==r['observed_sha256'] and len(b)==r['bytes'] and r['expected_hash_matches']
    read=json.loads((P/'historical-originals/parallel-l11-20261029/READ-172.json').read_text());fields=['registration','lens','kind','revision','path','sha256','actual_EOF','fresh_personal_body_ranges','assessed_ranges','whole_file_assessed','archive_sha256','archive_member','locked_package_checksum','lock_sha256','lock_registration','exact_archive_member_matches_local','attribution'];assert [{'pointer':f'/rows/{i}','selected':{k:read['rows'][i][k] for k in fields if k in read['rows'][i]}} for i in range(3)]==load('HISTORY-READ-DISPLAY.json')
    hist=load('HISTORY-ADOPTION-DISPLAY.json');ad=json.loads((P/'historical-originals/segovb/READ-L11-708.json').read_text());v=json.loads((P/'historical-originals/segovb/VERIFY-L11-708.json').read_text());proposal=json.loads((P/'historical-originals/parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-004.json').read_text())['contract_updates'][0];assert hist['adoption']['mapping']==ad['adopted_registration_mapping'] and all(v[k]==x for k,x in hist['verification']['selected'].items()) and all(proposal[k]==x for k,x in hist['proposal_selected'].items())
    lock=tomllib.loads((P/'sources/lock-current.lock').read_text());sel=load('CHILD-LOCK-SELECTION-DISPLAY.json')
    for item in sel:
        matches=[x for x in lock['package'] if x.get('name')==item['package']];assert len(matches)==item['count'];assert [{k:v for k,v in x.items() if k in (['name','version','dependencies'] if item['package']=='fabelgeist-numpy-storage' else ['name','version','source','checksum'])} for x in matches]==item['selected']
    native=load('NATIVE-VERIFICATION.json');nr=load('NATIVE-REGISTRATION.json');member=(P/native['member']).read_bytes();assert sha(member)==native['member_sha256']==nr['member_expected_sha256'] and len(member)==native['member_bytes'];assert native['archive_sha256']==nr['locked_package_checksum']==excluded['sha256'] and native['archive_bytes']==excluded['bytes'];assert sel[1]['selected'][0]['checksum']==native['archive_sha256']
    print('PASS safe packet file set/hashes, registered source displays/complements and chronology, exact 059 q2309 slices, five original historical records/attribution, child lock selections and retained native member.')
    if a.git_repo:
        for e in sources:
            b=git(a.git_repo,'show',f"{e['revision']}:{e['path']}");assert b==(P/e['capture']).read_bytes();ls=git(a.git_repo,'ls-tree',e['revision'],'--',e['path']).decode().strip();assert ls.split('\t')[0].split()[2]==e['git_blob']
            if 'historical_revision' in e:
                old=git(a.git_repo,'show',f"{e['historical_revision']}:{e['path']}");assert sha(old)==e['historical_sha256'] and (old==b)==e['historical_bytes_equal']
        cr=load('SOURCE-REGISTRATION.json');le=next(e for e in cr['entries'] if e['tag']=='lock-current');old=git(a.git_repo,'show',f"{cr['historical_lock']['revision']}:{le['path']}");assert sha(old)==cr['historical_lock']['sha256'] and old==(P/le['capture']).read_bytes()
        for stream,part in zip(qh['streams'],[o,pub]):
            b=git(a.git_repo,'show',f"{stream['revision']}:{stream['path']}");assert sha(b)==stream['stream_sha256'] and len(b)==stream['stream_bytes'] and b[stream['entry_start_byte']:stream['entry_end_byte_exclusive']]==part and json.loads(b)['entries'][2309]==q
        rh=load('RELATED-HISTORY-REGISTRATION.json');b=git(a.git_repo,'show',f"{rh['revision']}:{rh['path']}");row=json.loads(b)['entries'][2306];assert sha(canonical(row))==rh['selected_canonical_sha256'] and all(row[k]==v for k,v in load('RELATED-HISTORY-DISPLAY.json').items())
        roots=load('ROOT-LOCK-REGISTRATION.json');saved=load('ROOT-LOCK-SELECTION-DISPLAY.json')
        for e,expected in zip(roots['entries'],saved):
            b=git(a.git_repo,'show',f"{e['revision']}:{e['path']}");assert sha(b)==e['sha256'];d=tomllib.loads(b.decode());matches=[{k:v for k,v in x.items() if k in ('name','version','source','checksum')} for x in d['package'] if x.get('name')=='rawzip'];assert expected['revision']==e['revision'] and expected['rawzip_package_count']==len(matches) and expected['rawzip_packages']==matches
        records={}
        for e in load('ASSESSMENT-REGISTRATION.json')['files']:
            b=git(a.git_repo,'show',f"{e['revision']}:{e['path']}");assert sha(b)==e['sha256'] and len(b)==e['bytes'];records[e['path']]=json.loads(b)
        x=load('ASSESSMENT-RELATED-DISPLAY.json');d=records['ASSESSMENT-DELEGATED-RECONCILIATION-5-95E.json'];role=records['ASSESSMENT-DELEGATED-SOURCE-ROLES-5.json'];assert x['delegated_observation_6']==d['delegated_observations'][6] and x['source_role_3']==role['historical_source_roles'][3];assert all(x[k]==d[k] for k in ['schema','source_revision','evidence_checkpoint','central_decisions','open','repair','validation']) and x['source_role_qualification']==role['qualification']
        print('PASS fixed Git source identities, e275/95e child lock equality, both 059 streams, original q2306, root lock package selections and committed assessment excerpts.')
    else:print('NOT REPEATED Git commit membership, child-lock historical comparison, original queue streams, root lock selections or prior committed assessments: supply --git-repo.')
    verified_archive=False
    if a.raw_archive:
        check_package(a.raw_archive.read_bytes(),native,member);verified_archive=True;print('PASS supplied exact raw archive checksum and its exact locator member.')
    if a.predecessor:
        assert sha((a.predecessor/'MANIFEST.json').read_bytes())==boundary['restricted_predecessor_manifest_sha256'];check_manifest(a.predecessor);archive=(a.predecessor/excluded['path']).read_bytes();check_package(archive,native,member);verified_archive=True;print('PASS restricted predecessor manifest, exact archive checksum and archived member; predecessor is not privacy-cleared.')
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
        dep=boundary['recovery_dependency'];outer=(a.publication/dep['existing_publication_path']).read_bytes();assert sha(outer)==dep['existing_publication_sha256'];reg=load('PRIOR-ARCHIVE-COMPARISON-REGISTRATION.json')
        with tarfile.open(fileobj=io.BytesIO(outer),mode='r:gz') as t:
            found=[]
            for n in reg['members']:
                ms=[x for x in t.getmembers() if x.name==n];assert len(ms)==1 and ms[0].isfile();f=t.extractfile(ms[0]);assert f;found.append(f.read())
        assert sha(found[0])==dep['member_sha256'] and found[1]==member;check_package(found[0],native,member);verified_archive=True;print('PASS published transport inventory and preexisting outer archive exact package/member identities; preexisting archive is not privacy-cleared.')
    else:print('NOT REPEATED published transport inventory or outer archive comparison: supply --publication.')
    if not verified_archive:print('NOT REPEATED complete raw-archive checksum/member verification: supply --publication, --predecessor, or --raw-archive.')
    print('LIMITS: recorded chronology is custody metadata; checksums do not prove remote provenance, dependency resolution, source semantics, personal comprehension, native behavior or runtime results.')

if __name__=='__main__':main()
