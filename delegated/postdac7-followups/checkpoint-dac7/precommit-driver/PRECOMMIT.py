from pathlib import Path
import hashlib,json,os,subprocess,sys,tempfile
base=Path('../private-originals'); capture=base/'checkpoint-post0a24-staged-copy-wrnavxlq'; saved=json.loads((capture/'CAPTURE.json').read_bytes())
full=base/'checkpoint-post0a24-final-evidence-wgrf5a62'; result=json.loads((full/'RESULT.json').read_bytes()); log=(full/'evidence.log').read_bytes()
obs=json.loads(Path(sys.argv[1]).read_bytes())
assert obs['observed_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and obs['observed_tree']=='1eec97de340d9b7debbb30ca0c0fb2240a73b93f' and obs['ref']=='refs/heads/main'
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()=='0a2466a7485388575ef272e0a8355b9714204a37'
changed=[x.decode() for x in subprocess.check_output(['git','diff','--cached','--name-only','--diff-filter=ACMRT','-z']).split(b'\0') if x]
assert len(changed)==len(saved['rows'])==148 and set(changed)=={x['path'] for x in saved['rows']}
idx={}
for item in subprocess.check_output(['git','ls-files','--stage','-z']).split(b'\0'):
 if item:
  meta,name=item.split(b'\t',1); mode,oid,stage=meta.decode().split();idx[name.decode()]=(mode,oid,stage)
aggregate=hashlib.sha256()
for row in saved['rows']:
 n=row['path'];source=Path(n);scanned=capture/'tree'/n
 assert idx[n]==(row['mode'],row['git_blob'],'0') and source.is_file() and scanned.is_file() and not source.is_symlink() and not scanned.is_symlink()
 a=source.read_bytes();b=scanned.read_bytes(); assert a==b and len(a)==row['bytes'] and hashlib.sha256(a).hexdigest()==row['sha256']
 assert bool(source.stat().st_mode&0o111)==bool(scanned.stat().st_mode&0o111)==(row['mode']=='100755')
 aggregate.update(n.encode()+b'\0'+row['mode'].encode()+b'\0'+row['sha256'].encode()+b'\n')
assert aggregate.hexdigest()==saved['aggregate']
assert subprocess.run(['git','diff','--quiet']).returncode==0
assert not subprocess.check_output(['git','ls-files','--others','--exclude-standard','-z'])
assert subprocess.run(['git','diff','--cached','--check'],capture_output=True).returncode==0
assert result['exit']==0 and (len(log),hashlib.sha256(log).hexdigest())==(result['transcript_bytes'],result['transcript_sha256'])
assert result['assertion_preflight']=={'assertions_enabled':True,'optimization_level':0}
for path in ['post0a24-staged-scans-o2q89lgi/digest.out','post0a24-staged-scans-o2q89lgi/pattern.out']:
 assert (base/path).is_file()
digest=json.loads((base/'post0a24-staged-scans-o2q89lgi/digest.out').read_bytes())
assert digest['forbidden_digest_total']==0 and not digest['other_candidate_counts'] and not digest['exceptions']
pattern=(base/'post0a24-staged-scans-o2q89lgi/pattern.out').read_bytes();assert b'issues 0' in pattern
scans=json.loads((base/'post0a24-staged-scans-o2q89lgi/RESULT.json').read_bytes());assert len(scans['results'])==2 and all(row['exit']==0 and row['stderr_bytes']==0 for row in scans['results']);assert scans['results'][0]['forbidden_digest_total']==0 and not scans['results'][0]['candidate_categories'] and scans['results'][0]['exception_count']==0 and scans['results'][1]['zero_issues']
out=Path(tempfile.mkdtemp(prefix='checkpoint-post0a24-precommit-compare-',dir=base))
record={'parent':'0a2466a7485388575ef272e0a8355b9714204a37','staged_files':len(changed),'aggregate_names_modes_and_sha256':aggregate.hexdigest(),'scanned_capture_matches_index_and_worktree':True,'unstaged_or_untracked_files':False,'diff_check_exit':0,'driver_exit':result['exit'],'driver_transcript_sha256':result['transcript_sha256'],'driver_transcript_bytes':result['transcript_bytes'],'direct_child_assertion_preflight':result['assertion_preflight'],'staged_digest_scan_sha256':hashlib.sha256((base/'post0a24-staged-scans-o2q89lgi/digest.out').read_bytes()).hexdigest(),'staged_pattern_scan_sha256':hashlib.sha256(pattern).hexdigest(),'main_recheck':obs,'qualification':'Point-in-time local evidence integrity and same-machine observation only; ciphertext opacity, nested-child flags, independent custody, global privacy, product behavior and audit closure remain separately qualified.'}
with (out/'RECORD.json').open('x') as f:json.dump(record,f,indent=2);f.write('\n')
print('precommit record',out,'files',len(changed),'aggregate',aggregate.hexdigest())
