#!/usr/bin/env python3
"""Read-only verification of proposed after-b111 public evidence archives."""
import argparse,hashlib,json,pathlib,tarfile
P=pathlib.Path(__file__).resolve().parent

def need(ok,msg):
    if not ok:raise SystemExit('FAIL: '+msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--no-root-manifest',action='store_true',help='check archive/member integrity before root manifest creation');args=parser.parse_args()
    need(not any(x.is_symlink() for x in P.rglob('*')),'symlinks in stage')
    if not args.no_root_manifest:
        m=json.loads((P/'MANIFEST.json').read_text());rows={x['path']:x for x in m['files']};actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x.relative_to(P)!=pathlib.Path('MANIFEST.json')};need(actual==set(rows),'stage member inventory')
        for path,row in rows.items():
            f=P/path;b=f.read_bytes();need(len(b)==row['bytes'] and sha(b)==row['sha256'] and oct(f.stat().st_mode&0o777)==row['mode'],'stage file '+path)
    mapping=json.loads((P/'ARCHIVE-MAP.json').read_text());count=0
    for record in mapping['archives']:
        archive=P/record['archive'];raw=archive.read_bytes();need(len(raw)==record['archive_bytes'] and sha(raw)==record['archive_sha256'],'archive bytes '+record['id'])
        expected_files={'evidence/'+x['path']:x for x in record['members']};expected_dirs={'evidence'+('/'+x['path'] if x['path'] else ''):x for x in record['directories']};seen=set();source_manifest=None
        with tarfile.open(archive,'r:gz') as tar:
            for member in tar:
                name=member.name;parts=pathlib.PurePosixPath(name);need(not parts.is_absolute() and '..' not in parts.parts and name not in seen,'unsafe or duplicate tar member');seen.add(name)
                need(member.uid==0 and member.gid==0 and member.mtime==0 and not member.uname and not member.gname,'normalized tar metadata')
                if member.isdir():
                    need(name in expected_dirs and oct(member.mode)==expected_dirs[name]['mode'],'directory mode')
                elif member.isfile():
                    need(name in expected_files,'unexpected tar file');row=expected_files[name];need(member.size<=134217728,'member size bound');stream=tar.extractfile(member);need(stream is not None,'missing tar body');b=stream.read();need(len(b)==row['bytes'] and sha(b)==row['sha256'] and oct(member.mode)==row['mode'],'tar file identity')
                    if name=='evidence/'+record['source_manifest_name']:source_manifest=b
                else:raise SystemExit('FAIL: tar links and special entries are forbidden')
        need(seen==set(expected_files)|set(expected_dirs),'complete archive membership')
        need(source_manifest is not None and sha(source_manifest)==record['source_manifest_sha256'],'source manifest binding')
        original=json.loads(source_manifest);files=original['files'];original_rows=[dict(v,path=k) for k,v in files.items()] if isinstance(files,dict) else files;need({x['path'] for x in original_rows}|{record['source_manifest_name']}=={x['path'] for x in record['members']},'source manifest inventory')
        mapped={x['path']:x for x in record['members']}
        for row in original_rows:
            item=mapped[row['path']];need(row['sha256']==item['sha256'] and row['bytes']==item['bytes'],'source manifest member identity')
        count+=1
    print(json.dumps({'result':'pass','archives':count,'root_manifest_checked':not args.no_root_manifest,'external_private_dependencies_checked':False,'product_execution':False}))
if __name__=='__main__':main()
