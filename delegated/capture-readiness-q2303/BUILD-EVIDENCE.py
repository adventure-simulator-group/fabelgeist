#!/usr/bin/env python3
"""Construct reproducible evidence transcripts and Git-tree proofs only.

Argument is a Git repository containing the pinned immutable objects. This script
reads registered source objects and writes only next to this script. It does not
run project code or recover personal credit from archived bytes.
"""
import datetime, hashlib, json, pathlib, subprocess, sys
P = pathlib.Path(__file__).resolve().parent
REPO = pathlib.Path(sys.argv[1])
REV = '95e63628f9d70b8dd6e6fede64334696c6995009'
OLD = 'e2755a732c84e67296b5e3441a4259a84f461b2f'
def git(*args):
    return subprocess.check_output(['git', '-C', str(REPO), *args])
def sha(data): return hashlib.sha256(data).hexdigest()
def jread(name): return json.loads((P/name).read_text())
def write_json(name, obj): (P/name).write_text(json.dumps(obj,indent=2,ensure_ascii=False)+'\n')
# These sources were prospectively registered before displayed. The body is kept
# whole only so that its Git identity is recoverable; range coverage is below.
extra = [
    ('HARNESS-REGISTRATION.json','sources/harness.rs'),
    ('GLOBAL-TIMEOUT-REGISTRATION.json','sources/viewer_mod.rs'),
]
for reg, out in extra:
    item=jread(reg)['sources'][0]; d=git('show', REV+':'+item['path'])
    assert sha(d)==item['sha256']; (P/out).write_bytes(d)
registrations=[
    ('REGISTRATION.json', {
      'readiness': [(1,32)], 'capture': [(1,168),(169,350),(351,526)], 'schedule': [(1,57)]}),
    ('CONSUMER-REGISTRATION.json', {'capture': [(527,620),(750,798),(900,914)]}),
    ('SUPPORT-REGISTRATION.json', {'animation_mod': [(90,160),(240,280)], 'playback': [(1,155)], 'admission': [(118,130)]}),
]
rows=[]; transcript=[]
for reg_name,chunks in registrations:
    reg=jread(reg_name)
    for e in reg['entries']:
        data=(P/e['capture']).read_bytes(); assert sha(data)==e['sha256']; lines=data.decode().splitlines(); actual=chunks[e['tag']]
        for a,b in actual:
            assert 1 <= a <= b <= len(lines)
            transcript.append(f'--- {reg_name} {e["tag"]} {e["revision"]} {a}-{b} ---\n')
            transcript.extend(f'{i:6d}\t{lines[i-1]}\n' for i in range(a,b+1))
        rows.append({'registration':reg_name,'registration_sha256':sha((P/reg_name).read_bytes()),'source_path':e['git_path'],'source_sha256':e['sha256'],'revision':e['revision'],'planned_ranges':e['planned_ranges'],'actual_display_chunks':actual,'source_eof':len(lines),'source_capture':e['capture'],'display_status':'personally displayed, untruncated as observed','source_body_capture_is_not_full_read_credit':True})
for reg_name, capture, chunks in [
    ('HARNESS-REGISTRATION.json','sources/harness.rs',[(1,100),(230,345)]),
    ('HARNESS-INIT-REGISTRATION.json','sources/harness.rs',[(145,225)]),
    ('GLOBAL-TIMEOUT-REGISTRATION.json','sources/viewer_mod.rs',[(55,80)]),
]:
    e=jread(reg_name)['sources'][0]; data=(P/capture).read_bytes(); assert sha(data)==e['sha256']; lines=data.decode().splitlines()
    for a,b in chunks:
        transcript.append(f'--- {reg_name} {e["path"]} {REV} {a}-{b} ---\n')
        transcript.extend(f'{i:6d}\t{lines[i-1]}\n' for i in range(a,b+1))
    rows.append({'registration':reg_name,'registration_sha256':sha((P/reg_name).read_bytes()),'source_path':e['path'],'source_sha256':e['sha256'],'revision':REV,'planned_ranges':e['planned_ranges'],'actual_display_chunks':chunks,'source_eof':len(lines),'source_capture':capture,'display_status':'personally displayed, untruncated as observed','source_body_capture_is_not_full_read_credit':True})
(P/'DISPLAY-RECONSTRUCTED.txt').write_text(''.join(transcript))
# Compute union and unread complements from actual chunks, not stored byte counts.
union={}
for r in rows:
    key=(r['source_path'],r['revision'],r['source_eof']); union.setdefault(key,set())
    for a,b in r['actual_display_chunks']: union[key].update(range(a,b+1))
def ranges(indices):
    seq=sorted(indices); out=[]
    if not seq: return out
    a=b=seq[0]
    for i in seq[1:]:
        if i==b+1: b=i
        else: out.append([a,b]); a=b=i
    out.append([a,b]); return out
coverage=[{'source_path':path,'revision':rev,'eof':eof,'actual_display_union':ranges(got),'unread_complement':ranges(set(range(1,eof+1))-got)} for (path,rev,eof),got in sorted(union.items())]
write_json('READ-RECEIPTS.json', {'reader':'/root/mhr_character','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'qualification':'Reconstructed exact line transcripts from preserved source bytes after the described registered personal displays; the transcript itself is not a claim of an independently logged terminal capture. No product execution. Locator outputs and an earlier unregistered ledger preview are not credited as personal source reads. Tests only appear as source text.','rows':rows,'same_reader_display_union_and_unread_complement':coverage})
write_json('HISTORY-READ-RECEIPT.json',{'reader':'/root/mhr_character','registration':'HISTORY-REGISTRATION.json','registration_sha256':sha((P/'HISTORY-REGISTRATION.json').read_bytes()),'selected_values':'HISTORY-SELECTED-RECONSTRUCTED.json','selected_values_sha256':sha((P/'HISTORY-SELECTED-RECONSTRUCTED.json').read_bytes()),'actual_display':'all registered selected values personally displayed in three bounded, untruncated calls; full historical records retained as exact bytes but are not claimed as fully personally read','qualification':'Original parallel lens11_reconcile e275 source reads and subsequent root adoption without reread retain their separate attributions. Queue proposal still says pending. This is not checkpoint3821.'})
write_json('OTHER-READ-RECEIPTS.json',{'reader':'/root/mhr_character','entries':[
 {'registration':'QUEUE-REGISTRATION.json','actual_display':'/entries/2303 selected value, entire, untruncated','reconstruction':'QUEUE-2303-RECONSTRUCTED.json'},
 {'registration':'RELATED-QUEUE-REGISTRATION.json','actual_display':'/entries/1214 selected value, entire, untruncated','reconstruction':'QUEUE-1214-RECONSTRUCTED.json'},
 {'registration':'LEDGER-REGISTRATION.json','actual_display':'one matching committed ledger record, entire, untruncated; earlier unregistered rg/sed preview not credited','reconstruction':'LEDGER-SELECTION-RECONSTRUCTED.json'},
 {'registration':'PRIOR-ASSESSMENT-REGISTRATION.json','actual_display':'entire prior JSON record, lines 1-210, 211-420, 421-622, untruncated','capture':'prior/ASSESSMENT-CAPTURE-READINESS-COMPARISON-95E.json'}], 'qualification':'Source histories, selected queue records and central assessment retain distinct original reader/revision and cannot be promoted to missing checkpoint3821.'})
# Preserve raw tree objects and ancestry chains without commit author metadata.
treed=P/'git-trees'; treed.mkdir(exist_ok=True)
def tree_entries(raw):
    k=0; out={}
    while k<len(raw):
        sp=raw.index(b' ',k); nul=raw.index(b'\0',sp+1); mode=raw[k:sp].decode(); name=raw[sp+1:nul].decode(); oid=raw[nul+1:nul+21].hex(); out[name]=(mode,oid); k=nul+21
    return out
def proof(rev,path):
    root=git('rev-parse',rev+'^{tree}').decode().strip(); t=root; chain=[]; parts=path.split('/')
    for idx,part in enumerate(parts):
        raw=git('cat-file','tree',t); assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==t
        (treed/(t+'.tree')).write_bytes(raw); entries=tree_entries(raw); mode,oid=entries[part]; chain.append({'tree':t,'name':part,'mode':mode,'object':oid})
        if idx<len(parts)-1: assert mode=='40000'; t=oid
    return {'revision':rev,'root_tree':root,'path':path,'chain':chain,'blob':chain[-1]['object']}
paths={r['source_path'] for r in rows}
proofs=[proof(REV,path) for path in sorted(paths)]
for path in ['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs','crates/adventuresim-tactical-client/src/animation_viewer/capture/readiness.rs']:
    proofs.append(proof(OLD,path))
for pr in proofs:
    data=git('show',pr['revision']+':'+pr['path']); calculated=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest(); assert calculated==pr['blob']; pr['source_sha256']=sha(data); pr['source_bytes']=len(data)
write_json('GIT-TREE-PROOFS.json',{'qualification':'Raw tree objects support the per-file membership chains; revision-to-root identity uses read-only Git object inspection and is optionally verifiable with a repository containing these commits. Commit bodies and author metadata are deliberately omitted. Same per-file blob does not imply identical complete commit trees or historical personal read credit.','proofs':proofs})
print(json.dumps({'source_receipt_rows':len(rows),'tree_proofs':len(proofs),'raw_tree_objects':len(list(treed.glob('*.tree'))),'coverage':coverage},indent=2))
