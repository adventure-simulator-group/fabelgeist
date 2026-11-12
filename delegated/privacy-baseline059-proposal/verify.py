#!/usr/bin/env python3
"""Read-only verification of the proposed 059 public representations."""
from pathlib import Path
import argparse,gzip,hashlib,json,re
P=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def ptr(doc,path):
    obj=doc
    for token in path.lstrip('/').split('/'):
        token=token.replace('~1','/').replace('~0','~')
        obj=obj[int(token)] if isinstance(obj,list) else obj[token]
    return obj
ap=argparse.ArgumentParser();ap.add_argument('--private-dir',type=Path);ap.add_argument('--check-extraction',action='store_true');args=ap.parse_args()
if args.check_extraction and not args.private_dir:ap.error('--check-extraction requires --private-dir')
manifest=json.loads((P/'MANIFEST.json').read_bytes());actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file()}-{'MANIFEST.json'}
assert manifest['schema']=='726-baseline059-public-stage-manifest-v1' and actual==set(manifest['files'])
for n,x in manifest['files'].items():
    b=(P/n).read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes'],n
idx=json.loads((P/'RECONSTRUCTION.json').read_bytes());records={x['name']:x for x in idx['records']};assert set(records)=={'queue','ledger'}
assert records['queue']['original_sha256']=='2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08'
assert records['ledger']['original_sha256']=='3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac'
expected_counts={'queue':12,'ledger':14};docs={}
pathpat=re.compile(rb'(?<![A-Za-z0-9_./-])/(?:home|Users|tmp|private|mnt)/[^\s"\x27<>]+|[A-Za-z]:\\{1,2}Users\\{1,2}[^\s"\x27<>]+',re.I)
secretpat=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
emailpat=re.compile(rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}')
for name,x in records.items():
    b=(P/x['public_path']).read_bytes();assert len(b)==x['public_bytes'] and sha(b)==x['public_sha256'] and blob(b)==x['public_git_blob']
    assert x['original_sha256']!=x['public_sha256'] and x['redacted_occurrences']==expected_counts[name] and len(x['changed_json_pointers'])==expected_counts[name]
    assert not pathpat.search(b) and not secretpat.search(b) and not emailpat.search(b),name
    doc=json.loads(b);docs[name]=doc
    for y in x['changed_json_pointers']:assert ptr(doc,y['pointer'])==y['marker']
    assert len({y['pointer'] for y in x['changed_json_pointers']})==expected_counts[name]
for x in idx['marker_mapping']:
    for name in ('queue','ledger'):
        b=(P/records[name]['public_path']).read_bytes();assert b.count(x['marker'].encode())==x[name+'_occurrences']
assert len(idx['marker_mapping'])==10
assert docs['queue']['entries'][1818]['contract_id']=='Q-L12-ae99db0adb61f27b6ebb' and docs['queue']['entries'][1818]['status']=='open-evidence-or-disposition'
loose=json.loads((P/'LOOSE-REFERENCE-INVENTORY.json').read_bytes());ar=json.loads((P/'DELEGATED-ARCHIVE-REFERENCE-INVENTORY.json').read_bytes());links=json.loads((P/'HASH-LINK-INVENTORY.json').read_bytes());hist=json.loads((P/'HISTORICAL-ARCHIVE-IDENTITIES.json').read_bytes());overlap=json.loads((P/'AFFECTED-POINTERS.json').read_bytes())
assert loose['enumerated_regular_files']==21706 and len(loose['files'])==loose['matching_files_successfully_reread']==845 and not loose['changed_or_unread']
assert ar['archive_count_enumerated']==len(ar['archive_records'])==354 and ar['outer_regular_member_count']==34990 and len(ar['bounded_unscanned'])==315 and not ar['errors']
assert len(links['direct_link_manifest_records'])==20 and len(links['archived_hash_link_records_checkpoint16_19'])==35
assert sum(x['document']=='queue' for x in hist['records'])==280 and sum(x['document']=='ledger' for x in hist['records'])==125 and len(overlap['records'])==7
scan=json.loads((P/'SCAN-INPUTS.json').read_bytes())
for name in ('all-loose-paths.bin','loose-hit-paths.bin'):
    x=scan['files'][name];b=(P/x['path']).read_bytes();plain=gzip.decompress(b)
    assert sha(b)==x['stored_sha256'] and sha(plain)==x['uncompressed_sha256'] and len(plain)==x['uncompressed_bytes']
assert len(gzip.decompress((P/scan['files']['all-loose-paths.bin']['path']).read_bytes()).split(b'\0'))-1==21706
assert len(gzip.decompress((P/scan['files']['loose-hit-paths.bin']['path']).read_bytes()).split(b'\0'))-1==845
if args.private_dir:
    q=args.private_dir;private=json.loads((q/'RECOVERY.json').read_bytes());mapping={k.encode():v.encode() for k,v in private['original_path_mapping'].items()};assert len(mapping)==10 and len(set(mapping.values()))==10
    for name,x in records.items():
        gz=(q/private['local_original_compressed'][name]).read_bytes();assert sha(gz)==x['local_original_gzip_sha256'] and len(gz)==x['local_original_gzip_bytes'];old=gzip.decompress(gz)
        assert sha(old)==x['original_sha256'] and len(old)==x['original_bytes'] and blob(old)==x['original_git_blob']
        public=(P/x['public_path']).read_bytes();recon=old
        for source,marker in sorted(mapping.items(),key=lambda pair:-len(pair[0])):
            assert marker not in old;recon=recon.replace(source,marker)
        assert recon==public
        restored=public
        for source,marker in mapping.items():restored=restored.replace(marker,source)
        assert restored==old
        olddoc=json.loads(old)
        for y in x['changed_json_pointers']:
            val=ptr(olddoc,y['pointer']).encode();assert sha(val)==y['path_sha256'] and mapping[val].decode()==y['marker']
        if args.check_extraction: assert Path(private['original_paths'][name]).read_bytes()==old
    print('Checked retained originals, exact redaction and reversal'+(', and original extraction read-only.' if args.check_extraction else '.'))
else:
    print('Private originals not supplied: original-to-public transformation and reversal were not checked in this invocation.')
print('Verified public SHA/Git-blob identities, marker pointers, historical nonclosure, bounded reference inventories and reported scan limits. Privacy pattern scans do not prove absence of every possible sensitive item.')
