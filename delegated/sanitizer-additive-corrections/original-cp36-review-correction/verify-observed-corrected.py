#!/usr/bin/env python3
"""Read-only independent checkpoint-36 evidence review; executes no product code."""
import collections, gzip, hashlib, io, json, re, sys, tarfile
from datetime import datetime
from pathlib import Path

assert len(sys.argv) == 4, 'usage: verify-observed.py SNAPSHOT_DIR ORIGINAL_059_EXTRACTION_DIR PUBLICATION_DIR'
S = Path(sys.argv[1]); USER = Path(sys.argv[2]); backing = Path(sys.argv[3])
P = S / 'publication'
V = S / 'private-originals/checkpoint36'
sha = lambda b: hashlib.sha256(b).hexdigest()
def readjson(path): return json.loads(path.read_bytes())
def oid(kind, raw): return hashlib.sha1(kind.encode() + b' ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
def rngset(ranges):
    result = set()
    for lo, hi in ranges:
        assert 1 <= lo <= hi
        result.update(range(lo, hi + 1))
    return result

def tree(raw):
    out = {}; at = 0
    while at < len(raw):
        end = raw.index(b'\0', at); mode, name = raw[at:end].split(b' ', 1)
        name = name.decode(); assert name not in out
        out[name] = (mode.decode().lstrip('0'), raw[end+1:end+21].hex()); at = end + 21
    assert at == len(raw)
    return out

def safe_tar(raw):
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:') as t:
        info = t.getmembers(); names = [x.name for x in info]; assert len(names) == len(set(names))
        assert all((x.isfile() or x.isdir()) and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in info)
        return {x.name: None if x.isdir() else t.extractfile(x).read() for x in info}

def loc(x): return x.get('archive', x.get('preserved_bytes', x.get('preserved', x.get('local'))))

index_raw = (P/'DELEGATED-EVIDENCE-CHECKPOINT-36.json').read_bytes()
assert sha(index_raw) == '3005967922bb10c7d8b674b6ea044256f55fcc6908d27260c1646c049e3a0d17'
cp = json.loads(index_raw); assert cp['working_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert [x['scope_original_roles'] for x in cp['bundles']] == [[2605,2606],[2607,2608],[2609,2610]]
private = readjson(V/'MANIFEST.json'); private_rows = {(x['bundle_id'],x['path']):x for x in private['members']}
assert len(private_rows) == len(private['members'])
actual_private = {(str(p.relative_to(V)).split('/',1)[0],str(p.relative_to(V)).split('/',1)[1]) for p in V.rglob('*') if p.is_file() and p.name != 'MANIFEST.json'}
assert actual_private == set(private_rows)
for key, x in private_rows.items():
    raw = (V/key[0]/key[1]).read_bytes(); assert (sha(raw),len(raw)) == (x['sha256'],x['bytes'])

placeholder = re.compile(rb'<LOCAL_PATH_\d+>')
local = re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+', re.I)
secret = re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}', re.I)
false_redactions = []; privacy_hits = []; recursive_count = 0; redaction_count = 0

def check_content(old, new, m, where):
    global recursive_count,redaction_count
    recursive_count += 1
    assert (sha(old),len(old)) == (m['original_sha256'],m['original_bytes']), where
    assert (sha(new),len(new)) == (m['published_sha256'],m['published_bytes']), where
    assert m['changed'] == (old != new), where
    enc = m['content_encoding']; assert enc in ('identity','gzip','tar','gzip-tar')
    plain_old = gzip.decompress(old) if enc.startswith('gzip') else old
    plain_new = gzip.decompress(new) if enc.startswith('gzip') else new
    assert (sha(plain_old),len(plain_old)) == (m['original_uncompressed_sha256'],m['original_uncompressed_bytes']), where
    assert (sha(plain_new),len(plain_new)) == (m['published_uncompressed_sha256'],m['published_uncompressed_bytes']), where
    if not m['changed']: assert m['total_local_path_redactions'] == 0 and plain_old == plain_new
    else: assert 'Reconstructed public copy' in m['qualification'] and m['total_local_path_redactions'] > 0
    if enc.endswith('tar'):
        original = safe_tar(plain_old); public = safe_tar(plain_new)
        rows = {x['path']:x for x in m['members']}; assert len(rows) == len(m['members']) and set(original) == set(public) == set(rows)
        assert m['direct_local_path_redactions'] == 0
        for name, child in rows.items():
            assert (original[name] is None) == (public[name] is None)
            if original[name] is None: assert child['entry_type'] == 'directory' and not child['changed']
            else: check_content(original[name],public[name],child,where+'!'+name)
        assert m['total_local_path_redactions'] == sum(x['total_local_path_redactions'] for x in rows.values())
    else:
        if local.search(plain_new) or secret.search(plain_new): privacy_hits.append(where)
        assert m['total_local_path_redactions'] == m['direct_local_path_redactions']
        if m['changed']:
            pieces = placeholder.split(plain_new); marks = placeholder.findall(plain_new)
            assert len(marks) == m['direct_local_path_redactions'] and plain_old.startswith(pieces[0]),where
            pos = len(pieces[0]); mapping = {}
            for i, mark in enumerate(marks):
                tail = pieces[i+1]; end = plain_old.find(tail,pos) if tail else len(plain_old)
                assert end >= 0, where
                capture = plain_old[pos:end]
                assert mark not in mapping or mapping[mark] == capture
                mapping[mark] = capture
                if capture == b'/users/dependencies':
                    false_redactions.append({'location':where,'original_fragment':'definitions/users/dependencies','published_fragment':'definitions<LOCAL_PATH_0>','reason':'ordinary prose matched case-insensitive /Users/ local-path pattern'})
                else:
                    assert capture.startswith((b'/home/',b'/Users/',b'/tmp/726-')) or re.match(rb'[A-Za-z]:\\+Users\\+',capture,re.I), (where,'non-path-looking capture')
                pos = end + len(tail)
            assert pos == len(plain_old), where
            redaction_count += len(marks)

archive_data = {}; old_data = {}; member_indexes = {}; bundle_results = []
allowed = readjson(P/'QUALIFICATION-CHECKPOINT36-PRIOR-MANIFESTS.json')['rows']
for b in cp['bundles']:
    raw = (P/b['archive']).read_bytes(); assert (sha(raw),len(raw)) == (b['archive_sha256'],b['archive_bytes'])
    public = safe_tar(gzip.decompress(raw)); rows = {m['path']:m for m in b['members']}
    assert len(rows) == len(b['members']) and set(rows) == set(public) and all(v is not None for v in public.values())
    original = {}; changed = 0
    for name,m in rows.items():
        new = public[name]
        if m['changed']:
            key = (b['bundle_id'],name); assert key in private_rows
            old = (V/key[0]/key[1]).read_bytes(); assert (m['original_sha256'],m['original_bytes']) == (private_rows[key]['sha256'],private_rows[key]['bytes']); changed += 1
        else: old = new
        original[name] = old; check_content(old,new,m,b['bundle_id']+'/'+name)
    assert public[b['report_archive_member']] == (P/b['report_path']).read_bytes()
    report = json.loads(public[b['report_archive_member']]); assert report['author'] == b['agent']
    manifest_names = [x for x in ('MANIFEST.json','BUNDLE-MANIFEST.json','packet-manifest.json','manifest.json') if x in original]; assert len(manifest_names) == 1; manifest_name = manifest_names[0]; manifest = json.loads(original[manifest_name]); originals = {x['path']:x for x in manifest['files']}; assert len(originals) == len(manifest['files'])
    for name,x in originals.items(): assert (x['sha256'],x['bytes']) == (rows[name]['original_sha256'],rows[name]['original_bytes'])
    extras = set(rows)-set(originals)-{manifest_name}
    assert extras == {x['unlisted_member'] for x in allowed if x['packet']==b['bundle_id']}
    archive_data[b['bundle_id']] = public; old_data[b['bundle_id']] = original; member_indexes[b['bundle_id']] = rows
    bundle_results.append({'bundle':b['bundle_id'],'agent':b['agent'],'archive_sha256':sha(raw),'top_members':len(rows),'changed_top_members':changed,'redactions':sum(x['total_local_path_redactions'] for x in rows.values()),'extra_prior_manifests':sorted(extras)})
assert sum(x['changed_top_members'] for x in bundle_results) == len(private_rows) == 22
assert redaction_count == 213 and not privacy_hits

# Bind direct references using the stage when present and the existing publication otherwise.
direct = readjson(P/'DELEGATED-CHECKPOINT-36-DIRECT-LINKS.json')
for x in direct['links']:
    candidate = P/x['path']; candidate = candidate if candidate.is_file() else backing/x['path']; raw = candidate.read_bytes(); assert (sha(raw),len(raw)) == (x['sha256'],x['bytes'])

# Bind each later-manifest omission to an exact already-published earlier member.
for x in allowed:
    b = next(z for z in cp['bundles'] if z['bundle_id']==x['packet'])
    prior_cp = readjson(P/x['earlier_checkpoint']); prior = next(z for z in prior_cp['bundles'] if z['bundle_id']==x['earlier_bundle'])
    assert prior['archive'] == x['earlier_archive'] and prior['archive_sha256'] == x['earlier_archive_sha256']
    raw = (P/prior['archive']).read_bytes(); assert sha(raw) == prior['archive_sha256']
    prior_members = safe_tar(gzip.decompress(raw)); a = old_data[b['bundle_id']][x['unlisted_member']]; c = prior_members[x['earlier_member']]
    assert a == c and (sha(a),len(a)) == (x['original_sha256'],x['bytes'])

# Independently verify every preserved commit/tree/blob chain, using private originals
# when a public path was redacted. No source display or reader credit follows.
proof_count = 0; changed_proofs = []; proofs_by_bundle = {}
for b in cp['bundles']:
    name = b['bundle_id']; pub = archive_data[name]; orig = old_data[name]; rows = member_indexes[name]; proof = json.loads(pub['git-proofs.json']); objects = {}
    for gitid,x in proof['objects'].items():
        path = loc(x); raw = gzip.decompress(orig[path]); typ = x['type']
        assert (sha(raw),len(raw),oid(typ,raw)) == (x['sha256'],x.get('bytes',x.get('size')),gitid)
        objects[gitid] = (typ,raw)
    proofs = proof.get('proofs',proof.get('paths',[])); proofs_by_bundle[name] = proofs
    for x in proofs:
        commit = objects[x['revision']]; assert commit[0] == 'commit'
        root = commit[1].split(b'\n',1)[0].split()[1].decode()
        claimed = x.get('commit_tree',x.get('root_tree',proof.get('commits',{}).get(x['revision'],{}).get('root_tree')))
        assert root == claimed
        chain = x.get('chain',x.get('path_chain')); assert [a['name'] for a in chain] == x['path'].split('/')
        cur = root
        for step in chain:
            assert step['tree'] == cur and objects[cur][0] == 'tree'
            mode,child = tree(objects[cur][1])[step['name']]
            assert mode == step['mode'].lstrip('0') and child == step['object']; cur = child
        assert cur == x.get('blob',x.get('git_blob',cur)); path = loc(x); member = rows[path]; raw = orig[path]
        if member['content_encoding']=='gzip' and x.get('preserved_encoding')!='raw': raw = gzip.decompress(raw)
        assert oid('blob',raw) == cur and (not x.get('sha256') or sha(raw)==x['sha256'])
        if x.get('bytes') is not None: assert len(raw) == x['bytes']
        if member['changed']: changed_proofs.append(name+'/'+path)
        proof_count += 1
assert proof_count == 74 and len(changed_proofs) == 3
old_lines=archive_data['screen-armor-2605-2606']['sources/armor-old'].splitlines(keepends=True); current_lines=archive_data['screen-armor-2605-2606']['sources/armor-current'].splitlines(keepends=True); assert old_lines[89:171]==current_lines[157:239] and old_lines!=current_lines

registry_raw = gzip.decompress((P/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()); registry = json.loads(registry_raw)
coverage = readjson(P/'COVERAGE-L01-ROLES.json')['roles']; role_results = []
with tarfile.open(P/'recovered/original-registry-source-archive/objects.tar.gz') as t:
    for b in cp['bundles']:
        proofs = proofs_by_bundle[b['bundle_id']]; originals = old_data[b['bundle_id']]
        for i in b['scope_original_roles']:
            r = registry['files'][i]; assert r['snapshot'] == coverage[i]['snapshot'] == 'pr-working' and r['sha256'] == coverage[i]['historical_sha256']
            raw = t.extractfile(r['sha256']).read(); assert (sha(raw),len(raw),len(raw.splitlines())) == (r['sha256'],r['bytes'],r['actual_line_count'])
            assert any(raw == v for v in originals.values()), i
            path = r['source_id'].split(':',1)[1]
            old = [x for x in proofs if x['revision']=='6533e444fa37552927070cc8d8190907fd3b413e' and x['path']==path]
            current = [x for x in proofs if x['revision']==cp['working_revision'] and x['path']==path]
            assert old and current and all(x['sha256']==r['sha256'] for x in old)
            role_results.append({'role':i,'source_id':r['source_id'],'registry_sha256':r['sha256'],'material_ranges':r['material_ranges'],'actual_eof':r['actual_line_count'],'historical_6533_matches_object':True,'current_sha256':current[0]['sha256'],'current_matches_historical':current[0]['sha256']==r['sha256'],'author':b['agent']})

# Receipt/registration consistency is not proof of actual personal display.
receipt_counts = {}
for b in cp['bundles']:
    name = b['bundle_id']; data = archive_data[name]
    if name.startswith('screen-armor'):
        pairs = [('registration.json','personal-read-receipt.json'),('supplement-registration.json','supplement-read-receipt.json')]; count=0
        for regname,recname in pairs:
            rg=json.loads(data[regname]); rc=json.loads(data[recname]); assert rg['author']==rc['author']==b['agent'] and rc['registration_sha256']==sha(data[regname]); assert datetime.fromisoformat(rg['registered_utc'])<datetime.fromisoformat(rc['recorded_utc'])
            entries={x['id']:x for x in rg['entries']}; assert len(entries)==len(rc['reads'])
            for x in rc['reads']:
                e=entries[x['id']];raw=data[e['archive']];assert sha(raw)==e['sha256']==x['sha256'] and len(raw.splitlines())==e['eof']==x['eof']
                actual=rngset(x['actual_display_ranges']);unread=rngset(x['unread_complement']);assert actual==rngset(e['prospective_display_ranges']) and not actual&unread and actual|unread==set(range(1,e['eof']+1));count+=1
        receipt_counts[name]=count
    elif name.startswith('armor-design'):
        rg=json.loads(data['registration.json']);rc=json.loads(data['read-receipt.json']);cl=json.loads(data['registration-clarification.json']);assert rg['reader']==rc['reader']==b['agent'];assert rc['registration_sha256']==sha(data['registration.json'])==cl['registration_sha256'] and rc['registration_clarification_sha256']==sha(data['registration-clarification.json']);assert datetime.fromisoformat(rg['at_utc'])<datetime.fromisoformat(cl['at_utc'])<datetime.fromisoformat(rc['recorded_utc'])
        entries={v['local']:v for e in rg['entries'] for v in e['versions']};assert len(entries)==len(rc['actual_displays'])
        for x in rc['actual_displays']:
            e=entries[x['local']];raw=data[x['local']];assert sha(raw)==e['sha256']==x['sha256'] and len(raw.splitlines())==e['eof']
            actual=rngset(x['displayed_ranges']);unread=rngset(x['unread_complement']);assert actual==rngset(e['registered_windows']) and not actual&unread and actual|unread==set(range(1,e['eof']+1))
        receipt_counts[name]=len(entries)
    else:
        rg=json.loads(data['registration.json']);rc=json.loads(data['read-receipt.json']);assert rg['author']==rc['author']==b['agent'] and rc['registration_sha256']==sha(data['registration.json']);assert datetime.fromisoformat(rg['registered_utc'])<datetime.fromisoformat(rc['recorded_utc'])
        entries={x['tag']:x for x in rg['entries']};assert len(entries)==len(rc['entries'])
        for x in rc['entries']:
            e=entries[x['tag']];raw=data[x['archive']];assert sha(raw)==e['sha256']==x['sha256'] and len(raw.splitlines())==e['EOF']==x['EOF']
            actual=rngset(x['displayed_chunks']);unread=rngset(x['unread_complement']);assert actual==rngset(e['registered_ranges']) and not actual&unread and actual|unread==set(range(1,e['EOF']+1))
        assert 'truncated' in rc['display_qualification'] and 'again untruncated' in rc['display_qualification'];receipt_counts[name]=len(entries)

# Check original 059 transport identities and the path-redacted exact original.
tq=readjson(P/'DELEGATED-CHECKPOINT-36-QUALIFICATION.json');transport=json.loads(gzip.decompress((P/tq['transport_manifest']).read_bytes()));bysha=collections.defaultdict(list)
for x in transport['entries']:bysha[x['sha256']].append(x)
matched=[]; redacted=[]
def visit(m,where):
    h=m.get('original_sha256')
    if h in bysha:
        matched.append(where)
        if m['changed']: redacted.append((h,where,m['published_sha256'],m['published_bytes']))
    for z in m.get('members',[]):visit(z,where+'!'+z['path'])
for b in cp['bundles']:
    for m in b['members']:visit(m,b['archive']+'!'+m['path'])
assert len(matched)==tq['matched_nested_member_occurrences']==161 and len(redacted)==1 and len(tq['entries'])==1
q=tq['entries'][0];assert redacted[0][0]==q['original_sha256'];assert q['public_reconstructed_copies']==[{'public_location':redacted[0][1],'public_sha256':redacted[0][2],'public_bytes':redacted[0][3]}];assert q['original_transport_paths']==[x['path'] for x in bysha[q['original_sha256']]];raw=(USER/q['original_transport_paths'][0]).read_bytes();private_raw=(V/'anime-helmets-2609-2610/history-059/parallel-l12-20261029/PROPOSAL-BUNDLE-PAR-L12-881-888.json').read_bytes();assert raw==private_raw and (sha(raw),len(raw),oid('blob',raw))==(q['original_sha256'],q['original_bytes'],q['original_git_blob'])

# Check the historical rig registration-pin disagreements and qualification exclusions.
screen=archive_data['screen-armor-2605-2606'];g=lambda d,n:gzip.decompress(d[n]);old=json.loads(g(screen,'history/774/admission03-scope-reconciliation-044.json.gz'));end=json.loads(g(screen,'history/774/completion-112.json.gz'));reg=g(screen,'history/774/source-range-registration.jsonl.gz');qual=g(screen,'history/774/source-qualifications.jsonl.gz');p=next(x for x in old['input_pins'] if x['supplier']=='rig' and x['record'].endswith('source-range-registration.jsonl'));assert p['rows']==13 and p['sha256']!=sha(b''.join(reg.splitlines(True)[:13])); qp=next(x for x in old['input_pins'] if x['supplier']=='rig' and x['record'].endswith('source-qualifications.jsonl')); assert qp['sha256']==sha(b''.join(qual.splitlines(True)[:13]))=='70717aa1c0908935341afa5297937f1bf79513146453c1328076848d00e94e53'; ep=next(x for x in end['pins'] if x['record'].endswith('source-range-registration.jsonl')); assert ep['sha256']=='698ec787519f08585b75f3ce7f2fd08f0e8793f4f4837f91d88151c2c7783db1' and ep['sha256']!=sha(reg)
anime=archive_data['anime-helmets-2609-2610'];base='history-774/continuation/';rg=g(anime,base+'rig/admission03/source-range-registration.jsonl.gz');ql=g(anime,base+'rig/admission03/source-qualifications.jsonl.gz');a=json.loads(g(anime,base+'root/admission03-scope-reconciliation-046.json.gz'));z=json.loads(g(anime,base+'root/admission03-scope-reconciliation-050.json.gz'));at=json.loads(anime['historical-index.json'])['rig_registration_pin_qualification']
def pin(d,s):return next(x for x in d['input_pins'] if x['supplier']=='rig' and x['record'].endswith(s))
for d,count,key in [(a,35,'root046_registration_pin'),(z,139,'root050_registration_pin')]:
    v=pin(d,'source-range-registration.jsonl');assert v==at[key] and v['sha256']!=sha(b''.join(rg.splitlines(True)[:count]))
for d,count,key in [(a,31,'root046_qualification_pin'),(z,45,'root050_qualification_pin')]:
    v=pin(d,'source-qualifications.jsonl');assert v==at[key] and v['sha256']==sha(b''.join(ql.splitlines(True)[:count]))
x=next(x for x in z['role_qualifications'] if x.get('registration_id_resolved')=='RIG-ADMISSION03-REGISTER-055')['original_record'];assert x['id']=='RIG-ADMISSION03-SOURCE-QUALIFICATION-016' and x['excluded_incidental_display']==[[391,391]] and [392,414] in x['material_ranges']

# Check original rig registrations against role-specific scopes and hashes.
rig_data={}
for name,path in [('screen-armor-2605-2606','history/774/source-range-registration.jsonl.gz'),('armor-design-fluting-2607-2608','prior/design-fluting/history-774/source-range-registration.jsonl.gz'),('anime-helmets-2609-2610','history-774/continuation/rig/admission03/source-range-registration.jsonl.gz')]:
    rig_data[name]={x['id']:x for x in map(json.loads,g(archive_data[name],path).splitlines())}
rig_targets=[('screen-armor-2605-2606',48,2605,[[1,50]]),('screen-armor-2605-2606',49,2606,[[86,171]]),('armor-design-fluting-2607-2608',50,2607,[[1,129]]),('armor-design-fluting-2607-2608',51,2608,[[1,100]]),('armor-design-fluting-2607-2608',60,2608,[[102,131]]),('armor-design-fluting-2607-2608',61,2608,[[147,150]]),('anime-helmets-2609-2610',52,2609,[[1,56]]),('anime-helmets-2609-2610',55,2610,[[1,221],[249,275],[295,380],[385,385],[391,414]])]
for bn,num,role,ranges in rig_targets:
    r=rig_data[bn]['RIG-ADMISSION03-REGISTER-%03d'%num];expected=registry['files'][role];assert r['source_id']==expected['source_id'] and r['sha256']==expected['sha256'] and r['material_ranges']==ranges

# Selected historical 059 receipts are independently byte-compared with the
# extraction and transport manifest; matches are not personal review credits.
selected={
 'screen-armor-2605-2606':['history/059/adler/READ-L11-328.json','history/059/segovb/READ-L11-426.json','history/059/segovb/READ-L11-427.json','history/059/segovb/READ-L12-887.json','history/059/segovb/READ-L12-888.json'],
 'armor-design-fluting-2607-2608':['prior/design-fluting/history-059/READ-L11-427.json','prior/design-fluting/history-059/READ-L12-889.json','prior/design-fluting/history-059/READ-L12-890.json'],
 'anime-helmets-2609-2610':['history-059/parallel-l12-20261029/PROPOSAL-L12-891.json','history-059/parallel-l12-20261029/READ-CONNECTED-PAR-L12-TEAM022-COMPLETED-VIEWS-v2.json','history-059/segovb/READ-L11-427.json','history-059/segovb/READ-L12-891.json']}
historical_checks=[]
for bn,names in selected.items():
    for name in names:
        raw=old_data[bn][name]; matches=bysha[sha(raw)];assert len(matches)==1; x=matches[0];assert (len(raw),oid('blob',raw))==(x['bytes'],x['git_blob']);assert (USER/x['path']).read_bytes()==raw
        historical_checks.append({'bundle':bn,'archive_member':name,'original_transport_path':x['path'],'sha256':sha(raw)})
seg=json.loads(old_data['screen-armor-2605-2606']['history/059/segovb/READ-L11-426.json']);a2605=next(x for x in seg['sources'] if x['original_index']==2605);a2606=next(x for x in seg['sources'] if x['original_index']==2606);assert a2605['fresh_personal_body_reads']==[] and a2605['same_lens_prior']['receipt'].endswith('adler/READ-L11-328.json');assert a2606['fresh_personal_body_reads']==[[86,171]]
seg2=json.loads(old_data['screen-armor-2605-2606']['history/059/segovb/READ-L11-427.json']);extra=next(x for x in seg2['sources'] if x.get('registration','').endswith('REGISTRATION-SEGOVB-1529.json'));assert extra['original_index']==2606 and extra['fresh_personal_body_reads']==[[1,85]]
for bn,name in [('screen-armor-2605-2606','history/059/segovb/READ-L12-887.json'),('screen-armor-2605-2606','history/059/segovb/READ-L12-888.json'),('armor-design-fluting-2607-2608','prior/design-fluting/history-059/READ-L12-889.json'),('armor-design-fluting-2607-2608','prior/design-fluting/history-059/READ-L12-890.json'),('anime-helmets-2609-2610','history-059/segovb/READ-L12-891.json')]:
    d=json.loads(old_data[bn][name]);assert d['root_personal_rereads']==[]


out={'observed_checkpoint_index_sha256':sha(index_raw),'private_manifest_sha256':sha((V/'MANIFEST.json').read_bytes()),'bundles':bundle_results,'nested_member_count_including_top':recursive_count,'verified_original_and_public_redaction_occurrences':redaction_count,'known_false_positive_redactions':false_redactions,'public_known_pattern_privacy_hits':privacy_hits,'old_frozen_manifest_omissions_bound_to_prior':len(allowed),'git_path_proofs_verified':proof_count,'redacted_paths_git_verified_with_private_originals':changed_proofs,'role_identities':role_results,'receipt_rows_checked':receipt_counts,'selected_historical_059_read_receipts_compared':historical_checks,'rig_registration_rows_checked':len(rig_targets),'transport_hash_matches':len(matched),'transport_redacted_unique_originals':len(redacted),'user_extraction_original_proposal_sha256':q['original_sha256'],'history_pin_result':'Distinct 044/046/050 earlier registration pins do not match the respective later prefixes; selected qualification prefixes match and 2610 line 391 is excluded from original effective rig material. No global absence, source display or checkpoint3821 credit inferred.','limitations':'Privacy scan is pattern-limited; extracted exact originals are local-only. The snapshot uses symlinks to prior publication dependencies. Read receipts are assertions, not independent observation; product behavior and exhaustive audit are outside this evidence review.'}
print(json.dumps(out,indent=2))
