#!/usr/bin/env python3
"""Read-only source-evidence verifier. Does not run product code."""
import argparse,datetime,hashlib,io,json,pathlib,subprocess,sys,tarfile
def check(ok,why):
    if not ok:raise ValueError(why)
def sha(b):return hashlib.sha256(b).hexdigest()
def gitobj(kind,b):return hashlib.sha1((kind+' '+str(len(b))+'\0').encode()+b).hexdigest()
def tree_entries(b):
    i=0
    while i<len(b):
        a=b.index(b' ',i);z=b.index(b'\0',a);check(z+21<=len(b),'truncated tree')
        yield b[i:a].decode(),b[a+1:z].decode(),b[z+1:z+21].hex()
        i=z+21
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',help='optional Git object database to bind commits and source records');args=ap.parse_args()
    folder=pathlib.Path(__file__).resolve().parent;external=json.loads((folder/'PUBLIC-MANIFEST.json').read_text());arc=(folder/external['archive']).read_bytes()
    check(sha(arc)==external['archive_sha256'],'archive hash mismatch')
    check(sha(pathlib.Path(__file__).read_bytes())==external['verifier_sha256'],'verifier hash mismatch')
    data={}
    with tarfile.open(fileobj=io.BytesIO(arc),mode='r:gz') as tf:
        ms=tf.getmembers();check(len(ms)<=10000,'member count');total=0
        for m in ms:
            check(m.isfile() and m.name.startswith('q2336-q2337/') and '..' not in pathlib.PurePosixPath(m.name).parts,'unsafe member')
            name=m.name[len('q2336-q2337/'):];check(name and name not in data,'duplicate member')
            check(m.size<=64*1024*1024,'member limit');total+=m.size;check(total<=128*1024*1024,'archive limit')
            data[name]=tf.extractfile(m).read();check(len(data[name])==m.size,'member length')
    check(sha(data['MANIFEST.json'])==external['embedded_manifest_sha256'],'embedded manifest mismatch')
    mf=json.loads(data['MANIFEST.json']);expected=mf['members']
    check(set(data)==set(expected)|{'MANIFEST.json'},'member set')
    for name,x in expected.items():check(len(data[name])==x['bytes'] and sha(data[name])==x['sha256'],'member mismatch: '+name)
    check(len(expected)==external['member_count'],'member count mismatch')
    history=[]
    for n in ('HISTORY-CAPTURE.json','HISTORY-EXTRA-CAPTURE.json'):
        for x in json.loads(data[n])['items']:
            check(x['found'] and x['match'],'historical identity qualification')
            check(sha(data[x['file']])==x['expected_sha256']==x['actual_sha256'],'historical identity mismatch');history.append(x)
    captures=json.loads(data['QUEUE-CAPTURE.json'])['entries']
    for x in captures:
        n=x['pointer'].split('/')[-1];raw=data['QUEUE-'+n+'-RAW.json']
        check(sha(raw)==x['raw_sha256'] and len(raw)==x['raw_bytes'],'selected exact queue bytes mismatch')
        check(json.loads(raw)==json.loads(data['QUEUE-'+n+'-PROJECTION.json'])==json.loads(data['QUEUE-'+n+'-PUBLIC-PROJECTION.json']),'selected queue equality')
    revisions={};count_proofs=0
    for name in ('GIT-TREE-PROOFS.json','E275-GIT-TREE-PROOFS.json'):
        allproofs=json.loads(data[name]);td=allproofs['tree_directory']
        for item in allproofs['proofs']:
            root=item['root_tree_oid'];revisions[item['revision']]=root;oid=root;parts=item['path'].split('/')
            check(len(parts)==len(item['chain']),'tree chain length')
            for index,part in enumerate(parts):
                node=item['chain'][index];check(node['tree_oid']==oid and node['entry_name']==part,'tree chain continuity')
                raw=data[td+'/'+oid+'.tree'];check(gitobj('tree',raw)==oid,'tree hash mismatch')
                match=[x for x in tree_entries(raw) if x[1]==part];check(len(match)==1,'missing tree entry')
                mode,_,oid=match[0];check(mode==node['entry_mode'] and oid==node['entry_oid'],'entry mismatch')
                if index<len(parts)-1:check(mode in ('40000','040000'),'not a tree')
            body=data[item['file']];check(sha(body)==item['sha256'] and gitobj('blob',body)==oid==item['blob_oid'],'blob hash mismatch')
            count_proofs+=1
    receipts=[k for k in data if k.startswith('DISPLAY-V2-') and k.endswith('.receipt.json')]
    for k in receipts:
        x=json.loads(data[k]);reg=json.loads(data[x['registration']]);lo,hi=x['range']
        entries=[r for r in reg['items'] if r['file']==x['source'] and [lo,hi] in r['display_ranges']]
        check(len(entries)==1,'unregistered display')
        check(datetime.datetime.fromisoformat(reg['registered_at_utc'])<=datetime.datetime.fromisoformat(x['displayed_at_utc']),'display predates registration')
        body=data[x['source']];check(sha(body)==entries[0]['sha256'],'registered source hash')
        lines=body.decode().splitlines();check(1<=lo<=hi<=len(lines),'display range')
        view=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
        check(view==data[x['display']] and sha(view)==x['view_sha256'] and x['truncated'] is False,'display receipt mismatch')
    error=json.loads(data['DISPLAY-FORMAT-ERROR.json'])
    for x in error['affected_artifacts']:
        check(sha(data[x['receipt']])==x['receipt_sha256'] and sha(data[x['display']])==x['display_sha256'],'first display artifact mismatch')
        check(data[x['receipt']].endswith(b'\\n'),'malformed first receipt qualification')
    add=json.loads(data['REPORT-ADDENDUM.json'])
    for x in add['items']:
        label='actions' if x['path'].endswith('/actions.rs') else 'capabilities'
        check(data[label+'-e275.rs']==data[label+'-historical.rs']==data[label+'-current.rs'],'e275 per-file equality mismatch')
        check(x['e275_personal_source_display_ranges']==[],'e275 read credit')
    if args.repo:
        for rev,root in revisions.items():
            observed=subprocess.check_output(['git','-C',args.repo,'rev-parse',rev+'^{tree}'],stderr=subprocess.DEVNULL).decode().strip()
            check(observed==root,'commit root mismatch')
        def gitshow(rev,path):return subprocess.check_output(['git','-C',args.repo,'show',rev+':'+path],stderr=subprocess.DEVNULL)
        qp=json.loads(data['QUEUE-PLAN.json']);qraw=gitshow(qp['original']['revision'],qp['original']['path'])
        qpub=gitshow(qp['comparison']['revision'],qp['comparison']['path'])
        check(sha(qraw)==qp['original']['sha256'] and sha(qpub)==qp['comparison']['sha256'],'whole queue mismatch')
        pubs=json.loads(qpub)['entries']
        for x in captures:
            n=x['pointer'].split('/')[-1];a,b=x['raw_byte_range_half_open']
            check(qraw[a:b]==data['QUEUE-'+n+'-RAW.json'] and pubs[int(n)]==json.loads(data['QUEUE-'+n+'-PROJECTION.json']),'original queue pointer mismatch')
        nav=json.loads(data['LEDGER-NAV-PLAN.json']);results=json.loads(data['LEDGER-NAV-RESULTS.json'])
        ledgerraw=gitshow(nav['ledger']['revision'],nav['ledger']['path']);check(sha(ledgerraw)==results['ledger']['sha256'],'ledger source mismatch')
        ledger=json.loads(ledgerraw)['newly_assessed_ledger']['entries'];queue=json.loads(qraw)['entries']
        for n in (102,104):check(ledger[n]==json.loads(data['RELATED-ledger-'+str(n)+'.json']),'related ledger mismatch')
        for n in (4,1045):check(ledger[n]==json.loads(data['CLOCK-RELATED-ledger-'+str(n)+'.json']),'clock ledger mismatch')
        for n in (211,2338):check(queue[n]==json.loads(data['RELATED-queue-'+str(n)+'.json']),'related queue mismatch')
    print(json.dumps({'result':'ok','members':len(expected),'history_originals':len(history),'source_git_path_proofs':count_proofs,'current_personal_display_receipts':len(receipts),'preserved_excluded_malformed_receipts':len(error['affected_artifacts']),'repository_commit_and_record_check':'performed' if args.repo else 'not performed; supply --repo','limits':'Checks artifact/source identities and registered displays, not product behavior, inferred time policy, reachability, whole-tree equality or queue/lens closure.'},indent=2))
if __name__=='__main__':
    try:main()
    except Exception as e:print('verification failed: '+str(e),file=sys.stderr);sys.exit(1)
