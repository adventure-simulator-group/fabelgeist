#!/usr/bin/env python3
"""Read-only evidence verification; no product code is executed."""
import argparse,datetime,gzip,hashlib,io,json,pathlib,subprocess,sys,tarfile
def check(v,why):
    if not v:raise ValueError(why)
def sha(b):return hashlib.sha256(b).hexdigest()
def gitobj(k,b):return hashlib.sha1((k+' '+str(len(b))+'\0').encode()+b).hexdigest()
def entries(b):
    i=0
    while i<len(b):
        a=b.index(b' ',i);z=b.index(b'\0',a);check(z+21<=len(b),'truncated Git tree')
        yield b[i:a].decode(),b[a+1:z].decode(),b[z+1:z+21].hex()
        i=z+21
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',help='optional Git object database for commit and original record checks');args=ap.parse_args()
    root=pathlib.Path(__file__).resolve().parent;external=json.loads((root/'PUBLIC-MANIFEST.json').read_text());arc=(root/external['archive']).read_bytes()
    check(sha(arc)==external['archive_sha256'],'archive hash');check(sha(pathlib.Path(__file__).read_bytes())==external['verifier_sha256'],'verifier hash')
    data={}
    with tarfile.open(fileobj=io.BytesIO(arc),mode='r:gz') as tf:
        ms=tf.getmembers();check(len(ms)<=10000,'member count');total=0
        for m in ms:
            check(m.isfile() and m.name.startswith('q2352-q2353/') and '..' not in pathlib.PurePosixPath(m.name).parts,'unsafe member')
            name=m.name[len('q2352-q2353/'):];check(name and name not in data,'duplicate member')
            check(m.size<=64*1024*1024,'member too large');total+=m.size;check(total<=128*1024*1024,'total limit')
            data[name]=tf.extractfile(m).read();check(len(data[name])==m.size,'member length')
    check(sha(data['MANIFEST.json'])==external['embedded_manifest_sha256'],'embedded manifest')
    mf=json.loads(data['MANIFEST.json']);expect=mf['members'];check(set(data)==set(expect)|{'MANIFEST.json'},'member set')
    for name,x in expect.items():check(len(data[name])==x['bytes'] and sha(data[name])==x['sha256'],'member identity '+name)
    check(len(expect)==external['member_count'],'member count')
    history=json.loads(data['HISTORY-CAPTURE.json'])['items']
    for x in history:check(x['found'] and x['match'] and sha(data[x['file']])==x['actual_sha256']==x['expected_sha256'],'historical record identity')
    queue=json.loads(data['QUEUE-CAPTURE.json'])['entries']
    for x in queue:
        n=x['pointer'].split('/')[-1];raw=data['QUEUE-'+n+'-RAW.json']
        check(sha(raw)==x['raw_sha256'] and len(raw)==x['raw_bytes'],'queue bytes')
        check(json.loads(raw)==json.loads(data['QUEUE-'+n+'-PROJECTION.json'])==json.loads(data['QUEUE-'+n+'-PUBLIC-PROJECTION.json']),'queue projection equality')
    proof=json.loads(data['GIT-TREE-PROOFS.json']);td=proof['tree_directory'];revs={}
    for x in proof['proofs']:
        oid=x['root_tree_oid'];revs[x['revision']]=oid;parts=x['path'].split('/');check(len(parts)==len(x['chain']),'tree chain length')
        for i,part in enumerate(parts):
            step=x['chain'][i];check(step['tree_oid']==oid and step['entry_name']==part,'tree chain continuity')
            raw=data[td+'/'+oid+'.tree'];check(gitobj('tree',raw)==oid,'tree hash');matches=[e for e in entries(raw) if e[1]==part];check(len(matches)==1,'tree entry')
            mode,_,oid=matches[0];check(mode==step['entry_mode'] and oid==step['entry_oid'],'tree entry identity')
            if i<len(parts)-1:check(mode in ('40000','040000'),'tree intermediate')
        body=data[x['file']];check(sha(body)==x['sha256'] and gitobj('blob',body)==oid==x['blob_oid'],'blob identity')
    for label in ('sites','provenance','clock','chronology'):
        check(data[label+'-historic.rs']==data[label+'-e275.rs']==data[label+'-current.rs'],'per-file three-revision mismatch')
    receipts=[k for k in data if k.startswith('DISPLAY-V2-') and k.endswith('.receipt.json')]
    for name in receipts:
        r=json.loads(data[name]);reg=json.loads(data[r['registration']]);lo,hi=r['range'];found=[x for x in reg['items'] if x['file']==r['source'] and [lo,hi] in x['display_ranges']]
        check(len(found)==1,'display registration');check(datetime.datetime.fromisoformat(reg['registered_at_utc'])<=datetime.datetime.fromisoformat(r['displayed_at_utc']),'registration chronology')
        raw=data[r['source']];check(sha(raw)==found[0]['sha256'],'display source hash');lines=raw.decode().splitlines();check(1<=lo<=hi<=len(lines),'display bounds')
        want=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
        check(want==data[r['display']] and sha(want)==r['view_sha256'] and r['truncated'] is False,'display receipt')
    fcap=json.loads(data['F023-RECORD-CAPTURE.json'])
    for x in fcap['lines']:check(len(data[x['file']])==x['bytes'] and sha(data[x['file']])==x['sha256'],'774 exact line identity')
    check(sha(data['F023-774-SELECTED.json'])==fcap['selected_view_sha256'],'774 selected projection')
    correction=json.loads(data['F023-JSONL-CORRECTION.json']);check(len(correction['matches'])==42 and sum(x['value_exact_id'] for x in correction['matches'])==24,'774 locator correction')
    if args.repo:
        def show(rev,path):return subprocess.check_output(['git','-C',args.repo,'show',rev+':'+path],stderr=subprocess.DEVNULL)
        for rev,oid in revs.items():check(subprocess.check_output(['git','-C',args.repo,'rev-parse',rev+'^{tree}'],stderr=subprocess.DEVNULL).decode().strip()==oid,'commit root')
        qp=json.loads(data['QUEUE-PLAN.json']);qraw=show(qp['original']['revision'],qp['original']['path']);qpub=show(qp['comparison']['revision'],qp['comparison']['path'])
        check(sha(qraw)==qp['original']['sha256'] and sha(qpub)==qp['comparison']['sha256'],'queue whole-source hash')
        pubentries=json.loads(qpub)['entries'];original=json.loads(qraw)['entries']
        for x in queue:
            n=int(x['pointer'].split('/')[-1]);a,b=x['raw_byte_range_half_open'];check(qraw[a:b]==data['QUEUE-'+str(n)+'-RAW.json'] and original[n]==pubentries[n],'selected queue slice')
        check(original[940]==json.loads(data['RELATED-q940.json']),'q940 related object')
        ledgerplan=json.loads(data['LEDGER-RELATED-PLAN.json']);lb=show(ledgerplan['revision'],ledgerplan['path']);check(sha(lb)==json.loads(data['LEDGER-RELATED-CAPTURE.json'])['source_sha256'],'ledger source hash')
        led=json.loads(lb)['newly_assessed_ledger']['entries']
        for n in (178,825,743,787):check(led[n]==json.loads(data['LEDGER-'+str(n)+'.json']),'related ledger object')
        fp=json.loads(data['F023-RECORD-PLAN.json']);stored=show(fp['revision'],fp['path']);plain=gzip.decompress(stored)
        check(sha(stored)==fcap['stored_sha256'] and sha(plain)==fcap['decoded_sha256'],'774 stored/decoded identity')
        lines=plain.splitlines(keepends=True)
        for x in fcap['lines']:check(lines[x['line']-1]==data[x['file']],'774 original line')
    print(json.dumps({'result':'ok','archive_members':len(expect),'historic_record_originals':len(history),'source_git_path_proofs':len(proof['proofs']),'current_personal_display_receipts':len(receipts),'older_774_exact_lines':len(fcap['lines']),'repository_commit_and_original_record_check':'performed' if args.repo else 'not performed; supply --repo','limits':'Evidence identity and registered source displays only; does not establish product behavior, reachability, policy, whole-tree equality or queue/lens closure.'},indent=2))
if __name__=='__main__':
    try:main()
    except Exception as e:print('verification failed: '+str(e),file=sys.stderr);sys.exit(1)
