#!/usr/bin/env python3
"""Verify bounded evidence transformations without running a whole verifier."""
import argparse,ast,hashlib,json,os,pathlib,re,runpy,subprocess
HERE=pathlib.Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def require(ok,msg):
 if not ok:raise ValueError(msg)
def no_symlink_read(base,relative):
 base=base.absolute()
 for p in [base,*base.parents]:require(not p.is_symlink(),'symbolic-link ancestor')
 p=base
 for part in pathlib.Path(relative).parts:
  require(part not in ('..','.','/'),'invalid evidence path');p=p/part;require(not p.is_symlink(),'symbolic-link evidence path')
 return p.read_bytes()
def helper(source,begin,end,root):
 b=source.index(begin.encode());e=source.index(end.encode())+len(end);env={'root':root,'json':json,'hashlib':hashlib,'Path':pathlib.Path};exec(compile(source[b:e],'bounded-evidence-helper','exec'),env);return env
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--originals-root',type=pathlib.Path);ap.add_argument('--publication-root',type=pathlib.Path);ap.add_argument('--prior-stage',type=pathlib.Path);a=ap.parse_args()
 manifest=json.loads((HERE/'MANIFEST.json').read_bytes());listed={x['path']:x for x in manifest['files']};actual=set()
 for dirname,dirs,files in os.walk(HERE,followlinks=False):
  for name in dirs+files:require(not (pathlib.Path(dirname)/name).is_symlink(),'symbolic link in packet')
  for name in files:
   rel=(pathlib.Path(dirname)/name).relative_to(HERE).as_posix()
   if rel!='MANIFEST.json':actual.add(rel)
 require(actual==set(listed),'manifest membership mismatch')
 for name,r in listed.items():
  b=no_symlink_read(HERE,name);require((len(b),sha(b))==(r['bytes'],r['sha256']),'manifest member mismatch')
 build=runpy.run_path(str(HERE/'build_successor.py'),run_name='evidence_build_module');old=(HERE/'ORIGINAL-verify_recovered_public_redacted.py').read_bytes();new=(HERE/'verify_recovered_public_redacted_plus2.py').read_bytes();require(sha(old)==build['SOURCE_SHA'] and build['transform'](old)==new,'deterministic transformation mismatch');ast.parse(new.decode('utf-8'))
 t=json.loads((HERE/'TRANSFORMATION.json').read_bytes());require(t['predecessor_sha256']==sha(old) and t['successor_sha256']==sha(new) and t['unique_non_overlapping_anchor_count']==len(t['operations'])==len(build['REPLACEMENTS']),'transformation record mismatch')
 ot=old.decode();nt=new.decode();intervals=[]
 for row,(x,y) in zip(t['operations'],build['REPLACEMENTS']):
  require(ot.count(x)==nt.count(y)==1,'non-unique anchor');i=ot.index(x);j=nt.index(y);intervals.append((i,i+len(x)))
  def bounds(text,start,block):
   n=text.count('\n',0,start)+1;return n,n+block.count('\n')-(1 if block.endswith('\n') else 0)
  require(bounds(ot,i,x)==(row['predecessor_line_start'],row['predecessor_line_end']) and bounds(nt,j,y)==(row['successor_line_start'],row['successor_line_end']) and sha(x.encode())==row['predecessor_text_sha256'] and sha(y.encode())==row['successor_text_sha256'],'anchor record mismatch')
 v=sorted(intervals);require(all(x[1]<=y[0] for x,y in zip(v,v[1:])),'overlapping anchors')
 sample=HERE/'sample-publication';env=helper(new,'# BEGIN ADDITIONAL QUALIFIED PUBLIC REDACTIONS','# END ADDITIONAL QUALIFIED PUBLIC REDACTIONS',sample)
 cross=json.loads((sample/'recovered/privacy-redaction-evidence/ADDITIONAL-CROSSWALK.json').read_bytes());manifests={p:json.loads(env['_extra_historical_manifest'](p)) for p in env['_extra_manifests']}
 for row in cross['records']:
  pub=no_symlink_read(sample,row['publication_path']);env['_extra_public_object'](row['publication_path'],row['original_sha256'],pub)
  if a.originals_root:
   src=no_symlink_read(a.originals_root,row['original_sha256']);require((sha(src),len(src),blob(src))==(row['original_sha256'],row['original_bytes'],row['original_git_blob_sha1']),'retained original identity mismatch');patched=src;edits=[]
   for span in row['redactions']:
    start,n=span['original_byte_offset'],span['original_token_bytes'];pos,m=span['public_byte_offset'],span['public_token_bytes'];prior=src[start:start+n];replacement=pub[pos:pos+m]
    if row['kind']=='writer-output':require(bool(re.fullmatch(rb'(?i)[a-z]:[\\/]+Users[\\/]+[^\r\n\s]+',prior)),'unexpected writer path scope')
    else:require(prior.startswith(b'/tmp/') and not any(ch in b' \t\r\n' for ch in prior),'unexpected portability path scope')
    require(replacement==span['replacement'].encode(),'public marker span mismatch');edits.append((start,start+n,replacement))
   for start,end,replacement in sorted(edits,reverse=True):patched=patched[:start]+replacement+patched[end:]
   require(patched==pub,'byte-for-byte reconstruction mismatch')
   if row['kind']=='historical-temp-locators':
    original_doc=json.loads(src);public_doc=json.loads(pub);original_locator=original_doc['external_locator_constraints'][4]['locator'];public_doc['external_locator_constraints'][4]['locator']=original_locator;require(original_doc==public_doc,'additional non-path semantic change')
 require(env['_extra_seen']==set(env['_extra_objects']),'missing additional qualified record')
 writer=manifests['recovered/final-writer-static-trace/MANIFEST.json'];late=manifests['recovered/late-current-cell-count-evidence/MANIFEST.json'];port=manifests['recovered/transcript-other-candidates/portability-review.manifest.json'];writer_sha='57acc4af7fb1ed29ab96640a6f584028be9898dc9ff790add5ae5f2cca6af0ad';port_sha='21c676a13fb8e24af1bbbcdf4f19ffd911fc33e19d65b4f939b6c5da6bcf3e84'
 require(writer['apply_output_sha256']==late['output_sha256']==writer_sha and late['output']=='recovered/final-writer-static-trace/OUTPUT-419499.txt','historical writer manifest relationship mismatch')
 require(port['file']=='portability-review.redacted.json' and port['redacted_sha256']==port_sha and port['original_candidate_sha256']=='f55670c200ecc5ea82af8e7cb44a39a5d10a6e1cde86b2008bfa07c67411db18' and port['redaction_count']==5,'prior-redacted portability manifest relationship mismatch')
 port_public=json.loads(no_symlink_read(sample,'recovered/transcript-other-candidates/portability-review.redacted.json'))
 require(port_public['author']=='/root/handoff_portability_review','historical reviewer attribution mismatch')
 text=no_symlink_read(sample,late['output']).decode('utf-8');modes=[json.loads(line) for line in text.splitlines() if line.startswith('{')];require(modes==late['reported_modes'] and [x.get('mode') for x in modes]==['--preflight','--apply','--verify'] and all(x.get('status')=='pass' for x in modes),'historical reported-mode relationship mismatch')
 if a.originals_root:
  original=no_symlink_read(a.originals_root,writer_sha).decode('utf-8');original_modes=[json.loads(line) for line in original.splitlines() if line.startswith('{')];require(original_modes==modes,'historical report lines changed')
 if a.publication_root:
  root=a.publication_root;require(sha(no_symlink_read(root,'verify_recovered.py'))=='5fbd014581bbfc412596040ef923f680c72284115d3a5f4f7287f214212cb458','observed original verifier drift')
  for path in env['_extra_manifests']:require(no_symlink_read(root,path)==no_symlink_read(sample,path),'observed historical manifest drift')
  for row in cross['records']:
   orig=no_symlink_read(root,row['publication_path']);require((sha(orig),len(orig),blob(orig))==(row['original_sha256'],row['original_bytes'],row['original_git_blob_sha1']),'observed original object drift')
   out=subprocess.run(['git','-C',str(root),'ls-tree',cross['source_commit'],'--',row['publication_path']],text=True,capture_output=True,check=True).stdout.strip().split();require(len(out)>=3 and out[2]==row['original_git_blob_sha1'],'historical Git tree binding mismatch')
   b=subprocess.check_output(['git','-C',str(root),'cat-file','blob',out[2]]);require(b==orig,'historical Git original bytes mismatch')
 if a.prior_stage:
  prior=a.prior_stage;pm=no_symlink_read(prior,'MANIFEST.json');require(sha(pm)=='633ecf794f0a11bc837a32e0b2e120abcb8d2b14bb98543414de135b8fd9b3fa','prior frozen manifest mismatch'); require(no_symlink_read(prior,'verify_recovered_public_redacted.py')==old,'prior frozen successor mismatch')
  prior_listed={x['path']:x for x in json.loads(pm)['files']};prior_actual=set()
  for dirname,dirs,files in os.walk(prior,followlinks=False):
   for name in dirs+files:require(not (pathlib.Path(dirname)/name).is_symlink(),'symbolic link in prior packet')
   for name in files:
    rel=(pathlib.Path(dirname)/name).relative_to(prior).as_posix()
    if rel!='MANIFEST.json':prior_actual.add(rel)
  require(prior_actual==set(prior_listed),'prior frozen membership mismatch')
  for rel,rec in prior_listed.items():
   data=no_symlink_read(prior,rel);require((sha(data),len(data))==(rec['sha256'],rec['bytes']),'prior frozen member mismatch')
  ps=prior/'sample-publication';penv=helper(new,'# BEGIN QUALIFIED PUBLIC REDACTIONS','# END QUALIFIED PUBLIC REDACTIONS',ps)
  pc=json.loads(no_symlink_read(ps,'recovered/privacy-redaction-evidence/CROSSWALK.json'))
  for name in penv['_qualified_manifests']:
   no_symlink_read(ps,name);penv['_qualified_historical_manifest'](name)
  for row in pc['objects']:
   o=row['original'];penv['_qualified_public_object'](o['publication_path'],o['sha256'],no_symlink_read(ps,o['publication_path']))
  require(penv['_qualified_seen']==set(penv['_qualified_objects']),'prior qualified inputs missing')
 print(json.dumps({'status':'pass','predecessor_sha256':sha(old),'successor_sha256':sha(new),'unique_transformation_anchors':len(build['REPLACEMENTS']),'additional_historical_manifests_checked':len(manifests),'additional_qualified_objects_checked':len(env['_extra_seen']),'redaction_spans':sum(len(x['redactions']) for x in cross['records']),'reported_modes_checked':len(modes),'private_originals_checked':bool(a.originals_root),'observed_publication_checked':bool(a.publication_root),'prior_frozen_packet_and_three_object_helper_checked':bool(a.prior_stage),'qualification':'Syntax, deterministic source transformation, manifest relationships and public redaction identities were checked. Only bounded evidence helper declarations were executed; neither whole verifier nor product code ran. Historical reported modes do not establish recovery of checkpoint 3821. The earlier portability input was already redacted.'},indent=2))
if __name__=='__main__':main()
