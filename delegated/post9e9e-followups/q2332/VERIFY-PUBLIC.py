#!/usr/bin/env python3
"""Read-only evidence verification for the q2332 public reconstruction.

This does not execute product or macro code. It checks public representations;
replaying redaction or proving commit-to-tree identity needs private originals or
an independently supplied Git object database, respectively.
"""
import argparse, datetime, hashlib, io, json, pathlib, subprocess, sys, tarfile, tomllib

def check(condition, why):
    if not condition:
        raise ValueError(why)
def sha(data):
    return hashlib.sha256(data).hexdigest()
def obj(kind, data):
    return hashlib.sha1((kind + ' ' + str(len(data)) + '\0').encode() + data).hexdigest()
def tree_entries(raw):
    pos = 0
    while pos < len(raw):
        space = raw.index(b' ', pos)
        nul = raw.index(b'\0', space)
        check(nul + 21 <= len(raw), 'truncated raw Git tree entry')
        yield raw[pos:space].decode(), raw[space+1:nul].decode(), raw[nul+1:nul+21].hex()
        pos = nul + 21

def main():
    a=argparse.ArgumentParser();a.add_argument('--repo',help='optional repository containing the pinned commits');args=a.parse_args()
    folder=pathlib.Path(__file__).resolve().parent
    external=json.loads((folder/'PUBLIC-MANIFEST.json').read_text())
    archive=(folder/external['archive']).read_bytes()
    check(sha(archive)==external['archive_sha256'],'public archive hash mismatch')
    check(sha(pathlib.Path(__file__).read_bytes())==external['verifier_sha256'],'verifier hash mismatch')
    data={}
    with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as tf:
        members=tf.getmembers();check(len(members)<=10000,'member count limit')
        total=0
        for m in members:
            check(m.isfile() and m.name.startswith('q2332/') and '..' not in pathlib.PurePosixPath(m.name).parts,'unsafe archive member')
            rel=m.name[len('q2332/'):];check(rel and rel not in data,'duplicate or empty member')
            check(m.size<=64*1024*1024,'member size limit');total+=m.size;check(total<=128*1024*1024,'archive total limit')
            f=tf.extractfile(m);check(f is not None,'member unreadable');data[rel]=f.read();check(len(data[rel])==m.size,'member length')
    check(sha(data['MANIFEST.json'])==external['embedded_manifest_sha256'],'embedded manifest hash mismatch')
    manifest=json.loads(data['MANIFEST.json']); expected=manifest['members']
    check(set(data)==set(expected)|{'MANIFEST.json'},'public member set mismatch')
    for k,v in expected.items():check(len(data[k])==v['bytes'] and sha(data[k])==v['sha256'],'public member identity mismatch: '+k)
    check(len(expected)==external['member_count'],'member count mismatch')
    mp=json.loads(data['RECONSTRUCTION-MAP.json']);check(sha(data['RECONSTRUCTION-MAP.json'])==external['reconstruction_map_sha256'],'map mismatch')
    check(sha(data['REPORT.json'])==external['report_sha256'],'report mismatch')
    originals={}
    for r in json.loads(data['HISTORY-CAPTURE.json'])['items']:
        if r['found']:
            check(r['matches_expected'],'historic direct capture mismatch'); originals[r['capture']]=r['actual_sha256']
    for r in json.loads(data['HISTORY-EXTRA-CAPTURE.json'])['items']:
        if r['exists']:
            check(r['matches_expected'],'historic extra capture mismatch');originals[r['capture']]=r['actual_sha256']
    check(set(originals)=={r['path'] for r in mp['records']},'historical mapping coverage mismatch')
    for r in mp['records']:
        check(r['original_whole_file_sha256']==originals[r['path']],'historic original identity mismatch')
        check(sha(data[r['path']])==r['public_whole_file_sha256'],'historic reconstruction identity mismatch')
    q=json.loads(data['QUEUE-2332-RAW.json']);check(q==json.loads(data['QUEUE-2332-PROJECTION.json'])==json.loads(data['QUEUE-2332-PUBLIC-PROJECTION.json']),'selected queue object mismatch')
    proof=json.loads(data['GIT-TREE-PROOFS.json']); tree_dir=proof['tree_directory']; revisions={}
    for item in proof['proofs']:
        oid=item['root_tree_oid'];revisions[item['revision']]=oid
        parts=item['path'].split('/');check(len(parts)==len(item['chain']),'tree chain length')
        for index,part in enumerate(parts):
            c=item['chain'][index];check(c['tree_oid']==oid and c['entry_name']==part,'tree chain continuity')
            raw=data[tree_dir+'/'+oid+'.tree'];check(obj('tree',raw)==oid,'tree object hash mismatch')
            matches=[x for x in tree_entries(raw) if x[1]==part];check(len(matches)==1,'missing/ambiguous tree entry')
            mode,_,oid=matches[0];check(mode==c['entry_mode'] and oid==c['entry_oid'],'tree entry mismatch')
            if index < len(parts)-1:check(mode in ('40000','040000'),'non-tree intermediate')
        body=data[item['file']];check(sha(body)==item['sha256'] and obj('blob',body)==oid==item['blob_oid'],'blob identity mismatch')
    if args.repo:
        for rev,root in revisions.items():
            observed=subprocess.check_output(['git','-C',args.repo,'rev-parse',rev+'^{tree}'],stderr=subprocess.DEVNULL).decode().strip()
            check(observed==root,'commit to root tree mismatch')
    lock=tomllib.loads(data['sources/current-Cargo.lock'].decode())
    names=[('spacetimedb-bindings-macro','2.6.1'),('spacetimedb-sats','2.6.1'),('spacetimedb-lib','2.6.1'),('bevy_reflect','0.19.1'),('bevy_reflect_derive','0.19.1')]
    for name,version in names:
        records=[x for x in lock['package'] if x['name']==name and x['version']==version]
        check(len(records)==1,'lock package ambiguity'); archive_name='native/'+name+'-'+version+'.crate'
        check(sha(data[archive_name])==records[0]['checksum'],'native archive/checksum mismatch')
    native=json.loads(data['NATIVE-MEMBER-CAPTURE.json'])['members']
    native += [{'archive':'native/spacetimedb-lib-2.6.1.crate','member':'spacetimedb-lib-2.6.1/src/lib.rs','capture':'sources/native-spacetimedb-lib.rs'}, {'archive':'native/spacetimedb-sats-2.6.1.crate','member':'spacetimedb-sats-2.6.1/src/lib.rs','capture':'sources/native-sats-lib.rs'}, {'archive':'native/spacetimedb-sats-2.6.1.crate','member':'spacetimedb-sats-2.6.1/src/typespace.rs','capture':'sources/native-typespace.rs'}]
    for r in native:
        with tarfile.open(fileobj=io.BytesIO(data[r['archive']]),mode='r:gz') as tf:
            f=tf.extractfile(r['member']);check(f is not None,'native member absent');b=f.read()
        check(b==data[r['capture']],'native member content mismatch')
    receipts=[k for k in data if k.startswith('DISPLAY-') and k.endswith('.receipt.json')]
    for k in receipts:
        r=json.loads(data[k]);reg=json.loads(data[r['registration']]);lo,hi=r['range']
        check(any(x.get('file')==r['source'] and [lo,hi] in x.get('display_ranges',[]) for x in reg['items']),'display not prospectively registered')
        check(datetime.datetime.fromisoformat(reg['registered_at_utc'])<=datetime.datetime.fromisoformat(r['displayed_at_utc']),'registration after display')
        lines=data[r['source']].decode().splitlines();check(1<=lo<=hi<=len(lines),'display bounds')
        want=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
        check(want==data[r['display']] and sha(want)==r['view_sha256'] and r['truncated'] is False,'display/receipt mismatch')
    print(json.dumps({'result':'ok','public_members_verified':len(expected),'historic_record_identities':len(originals),'redacted_record_count':sum(x['replacement_count']>0 for x in mp['records']),'replacement_count':sum(x['replacement_count'] for x in mp['records']),'tree_proofs':len(proof['proofs']),'native_archives':len(names),'native_members':len(native),'source_display_receipts':len(receipts),'commit_tree_check':'performed' if args.repo else 'not performed: supply --repo','limitations':'Public reconstruction verifies its own bytes and recorded original whole-file identities; replay requires private originals. Native feature selection, generated code, product behavior and all uninspected ranges are not verified.'},indent=2))
if __name__=='__main__':
    try:main()
    except Exception as e:
        print('verification failed: '+str(e),file=sys.stderr);sys.exit(1)
