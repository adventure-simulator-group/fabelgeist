#!/usr/bin/env python3
"""Read-only evidence identity verification, not product validation.

Without --repo, verifies packet manifest, raw Git-tree chains, source byte/blob
identities, the reconstructed registered display, and historical selectors.
With --repo PATH, also verifies revision-to-root anchors and selected records
against the immutable Git objects in that repository. Writes nothing.
"""
import argparse, hashlib, json, pathlib, subprocess
P=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser(); ap.add_argument('--repo'); args=ap.parse_args()
def sha(b): return hashlib.sha256(b).hexdigest()
def read(name): return json.loads((P/name).read_text())
def blob_id(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def getptr(v,ptr):
    if ptr in ('','/'): return v
    assert ptr.startswith('/')
    for segment in ptr[1:].split('/'):
        segment=segment.replace('~1','/').replace('~0','~'); v=v[int(segment)] if isinstance(v,list) else v[segment]
    return v
def entries(raw):
    out={}; i=0
    while i<len(raw):
        sp=raw.index(b' ',i); z=raw.index(b'\0',sp+1); out[raw[sp+1:z].decode()]=(raw[i:sp].decode(),raw[z+1:z+21].hex()); i=z+21
    return out
manifest=read('MANIFEST.json'); tracked={x['path'] for x in manifest['files']}
actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x != P/'MANIFEST.json'}
assert tracked==actual, 'manifest membership mismatch'
for row in manifest['files']:
    data=(P/row['path']).read_bytes(); assert len(data)==row['bytes'] and sha(data)==row['sha256'],row['path']
receipts=read('READ-RECEIPTS.json'); by_path={}; transcript=[]
for r in receipts['rows']:
    assert sha((P/r['registration']).read_bytes())==r['registration_sha256']; d=(P/r['source_capture']).read_bytes(); assert sha(d)==r['source_sha256']; by_path[r['source_path']]=d
    lines=d.decode().splitlines(); assert len(lines)==r['source_eof']
    registered=read(r['registration']); regentries=registered.get('entries', registered.get('sources',[])); matching=[e for e in regentries if e.get('git_path',e.get('path'))==r['source_path']]; assert len(matching)==1
    plan=matching[0]['planned_ranges']; assert plan==r['planned_ranges']; planned={i for a,b in plan for i in range(a,b+1)}; actual={i for a,b in r['actual_display_chunks'] for i in range(a,b+1)}; assert actual==planned
    for a,b in r['actual_display_chunks']:
        if r['registration'] in ('REGISTRATION.json','CONSUMER-REGISTRATION.json','SUPPORT-REGISTRATION.json'):
            tag=matching[0]['tag']; label=tag
        else: label=r['source_path']
        transcript.append(f'--- {r["registration"]} {label} {r["revision"]} {a}-{b} ---\n'); transcript.extend(f'{i:6d}\t{lines[i-1]}\n' for i in range(a,b+1))
assert ''.join(transcript).encode()==(P/'DISPLAY-RECONSTRUCTED.txt').read_bytes()
proofs=read('GIT-TREE-PROOFS.json')['proofs']
for pr in proofs:
    assert pr['chain'][0]['tree']==pr['root_tree']; parts=pr['path'].split('/'); assert len(parts)==len(pr['chain'])
    for idx,row in enumerate(pr['chain']):
        raw=(P/'git-trees'/(row['tree']+'.tree')).read_bytes(); assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==row['tree']; assert row['name']==parts[idx]; assert entries(raw)[row['name']]==(row['mode'],row['object'])
        if idx+1<len(parts): assert row['mode']=='40000' and row['object']==pr['chain'][idx+1]['tree']
    assert pr['chain'][-1]['object']==pr['blob']; assert pr['path'] in by_path
    d=by_path[pr['path']]; assert len(d)==pr['source_bytes'] and sha(d)==pr['source_sha256'] and blob_id(d)==pr['blob']
for path in ['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs','crates/adventuresim-tactical-client/src/animation_viewer/capture/readiness.rs']:
    pair=[p for p in proofs if p['path']==path]; assert len(pair)==2 and pair[0]['blob']==pair[1]['blob'] and pair[0]['source_sha256']==pair[1]['source_sha256']
hreg=read('HISTORY-REGISTRATION.json'); selections=read('HISTORY-SELECTED-RECONSTRUCTED.json'); selected={r['record']:r['selections'] for r in selections['records']}
assert len(hreg['entries'])==len(selected)==9
for e in hreg['entries']:
    raw=(P/'history-exact'/e['historical_relative_path']).read_bytes(); assert len(raw)==e['bytes'] and sha(raw)==e['sha256']; doc=json.loads(raw); sel=selected[e['historical_relative_path']]; assert [s['pointer'] for s in sel]==e['planned_selectors']
    for s in sel: assert getptr(doc,s['pointer'])==s['value']
hr=read('HISTORY-READ-RECEIPT.json'); assert sha((P/hr['registration']).read_bytes())==hr['registration_sha256'] and sha((P/hr['selected_values']).read_bytes())==hr['selected_values_sha256']
q=read('QUEUE-2303-RECONSTRUCTED.json')['value']; expected=set()
for row in q['exact_source_identities_and_assessed_ranges']:
    expected.update([row['body_receipt']['sha256'],row['registration_ref_and_sha256']['sha256'],row['root_adoption']['sha256'],row['root_adoption']['verification_sha256']])
expected.add(q['proposal']['sha256']); assert expected=={e['sha256'] for e in hreg['entries']}
prior=read('PRIOR-ASSESSMENT-REGISTRATION.json')['source']; raw=(P/'prior'/prior['path']).read_bytes(); assert len(raw)==prior['bytes'] and sha(raw)==prior['sha256'] and blob_id(raw)==prior['git_blob']
if args.repo:
    def git(*a): return subprocess.check_output(['git','-C',args.repo,*a])
    for pr in proofs:
        assert git('rev-parse',pr['revision']+'^{tree}').decode().strip()==pr['root_tree']; data=git('show',pr['revision']+':'+pr['path']); assert sha(data)==pr['source_sha256'] and blob_id(data)==pr['blob']
    qr=read('QUEUE-REGISTRATION.json'); raw=git('show',qr['revision']+':'+qr['git_path']); assert sha(raw)==qr['sha256'] and blob_id(raw)==qr['git_blob']; original=json.loads(raw)
    for name in ['QUEUE-2303-RECONSTRUCTED.json','QUEUE-1214-RECONSTRUCTED.json']:
        rec=read(name); assert getptr(original,rec['pointer'])==rec['value']
    lr=read('LEDGER-REGISTRATION.json')['source']; raw=git('show',lr['revision']+':'+lr['path']); assert sha(raw)==lr['sha256'] and blob_id(raw)==lr['git_blob']; source=json.loads(raw)
    for selected in read('LEDGER-SELECTION-RECONSTRUCTED.json')['selected']: assert getptr(source,selected['pointer'])==selected['value']
    assert git('show',prior['revision']+':'+prior['path'])==(P/'prior'/prior['path']).read_bytes()
print(json.dumps({'status':'passed','manifest_files':len(tracked),'tree_proofs':len(proofs),'historical_records':len(hreg['entries']),'revision_roots_and_selected_queue_verified_with_repo':bool(args.repo),'scope':'Evidence identities and selected reconstructions only; no product validation, semantic closure, or proof of personally viewed text.'}))
