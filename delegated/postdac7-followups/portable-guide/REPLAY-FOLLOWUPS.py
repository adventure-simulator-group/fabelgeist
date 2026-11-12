#!/usr/bin/env python3
"""Time-bound, evidence-only replay of the observed post-dac7 follow-ups.

Logs contain private local paths. Use a private scratch parent. This does not
restore encrypted-originals or run q2945; read RECOVERY-GUIDE.md first.
"""
import argparse,hashlib,json,os,pathlib,subprocess,sys,tempfile
P=pathlib.Path
EXPECTED={
 'pending':'4b2c44c25ce05f96131aa823803c791006dec100c0e802e2f5e04d3e110d7759',
 'pending_initial':'153d5d4fd4dd4f3f1bd7a9bbdaa8087c346a9b60beccd76d1c7fcdbd5bd472e1',
 'transport':'aebec43ab68747c65e255b8ee5454c7073b6e2d04dd2b8ff020c4a72e5387a11',
 'review_transport':'0a836ae049e3bfe32d1b379c5e80b84a967ca165ef261e3f1aa3a1545dffefcd',
 'extra_transport':'576991ece8acafdf017147552946566200dca098fd225edbda69d449fd284e22',
 'checkpoint':'c668cc12e5d5cda94d8b227d6665eaca5cb82087a99840f5940fb4650af78535',
 'store':'b9cc0410979d41a3417896e0e5a545ded16bbfcb2bc7b2a1368160838abbe4ec',
 'store_manifest':'68aedd4364eb45a6d9a2f360b9903a5b99f3935935910c248108747f4ea735cc',
 'q2904_store':'9b7c46f75d71bc7814be930f7112d37d04a03b2ce8b39debe038729b2b73de3c',
 'q2904_store_manifest':'9ee8ea36d792d922cfb08df640e320a32085a6326515112a61612354b6343185',
 'anchors':'68dc9bb7156c588afa6086965048478e54fc790dbfc9e8b6465e4aefccfaa76a',
 'anchor_manifest':'e3262991171c1b8279952062621c01b95803d573fbb4a20cd05cb8b2cc794a2d',
 'q2748':'e449420f189547a0b04a30552e61b933af11473b07f048e76f004fd7135c1407',
 'q2754':'dd3a3f8920592190282228be7593b8bb6a3329c9f852b96b58e94b4f3815a2e0',
 'q2896':'f0df9ff486bb092ce549e6bfad62416e8bf683bebd98507b92e4f6900dbc8ce5',
 'q2904':'34316eccae3de8c53cee9e81600c67e1873d7008497511c2c92501c2bc6da4dd',
 'q2910':'9786ad3d8e6be1c9101122c49704ccb5078832d6f9fe79afae3bad47c448f51b',
 'four_review':'539771cbeef39e0602b549ec6e8b8191e8dce94749f8eb6ae6577097b8b14bf1',
 'dialogue_review':'9f052a85251cecada5b0b753e2ad38a73a5c1df8725fe1df9c0d5337e43bbab5',
 'prepared_review':'df339931c4869156c61a3122223c1d6ef05995d52cb784c009cd4f4acaa04e5a',
 'stream_review':'4c344536bb73c311f60c6cb8b702453cc467bd44bf286dfd592a5a81230b1552',
 'stream_private':'9355888f8b2ed43071894994bfc33ad726becef568782f94686d2fd4b2e42563',
 'stream_scan':'8198a5839963579dfa633cd36795f36dbd27466c743003db5d4374d0f85a9f0d',
 'q2896_private_manifest':'dcd820322604cccada65714a11f74fc4fb6cf85bf08998d086a8c07f9ff5e701',
}
def die(message):
 print('STOP:',message,file=sys.stderr);raise SystemExit(2)
def check(label,path):
 try:b=path.read_bytes()
 except OSError:die('missing required '+label)
 if hashlib.sha256(b).hexdigest()!=EXPECTED[label]:die('identity mismatch: '+label+'; re-review the new checkpoint before proceeding')
def write_new(path,body):
 fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 with os.fdopen(fd,'wb') as f:f.write(body)
a=argparse.ArgumentParser(description=__doc__)
a.add_argument('--public',type=P,required=True,help='published post-dac7 follow-up root')
a.add_argument('--private',type=P,required=True,help='staged private-inputs root or restored collection root')
a.add_argument('--layout',choices=['stage','restored'],required=True)
a.add_argument('--anchors',type=P,required=True,help='exact private q2812 anchor collection')
a.add_argument('--scratch-parent',type=P,required=True,help='existing private directory for a new receipt directory')
a=a.parse_args()
if sys.flags.optimize:die('disable Python optimization')
pub=a.public.resolve(); private=a.private.resolve(); anchor=a.anchors.resolve()
def col(label):return private/label if a.layout=='stage' else private/label/'evidence'
v6=col('shared-replay-v6'); qstore=col('q2904')/'standalone-replay';repo=v6/'git-evidence';qrepo=qstore/'git-evidence'
checks=[('pending',pub/'PENDING-DELEGATES.json'),('pending_initial',pub/'PENDING-DELEGATES-INITIAL.json'),('transport',pub/'transport/VERIFY.py'),('review_transport',pub/'review-transport/VERIFY.py'),('extra_transport',pub/'review-extra-transport/VERIFY.py'),('checkpoint',pub/'checkpoint-dac7/VERIFY.py'),('store',v6/'VERIFY-STORE.py'),('store_manifest',v6/'STORE-MANIFEST.json'),('q2904_store',qstore/'VERIFY-REPLAY.py'),('q2904_store_manifest',qstore/'MANIFEST.json'),('anchors',anchor/'verify-anchor-collection.py'),('anchor_manifest',anchor/'ANCHOR-MANIFEST.json'),('q2910',pub/'q2910-packet/VERIFY.py'),('q2896_private_manifest',col('q2896')/'original/MANIFEST.json'),('stream_private',col('stream-review')/'restricted/VERIFY-PRIVATE.py'),('stream_scan',col('stream-review')/'scan/VERIFY-SCAN.py')]
for label,path in checks:check(label,path)
if not a.scratch_parent.is_dir():die('scratch parent must already exist')
scratch=P(tempfile.mkdtemp(prefix='postdac7-replay-',dir=a.scratch_parent));os.chmod(scratch,0o700)
env={k:v for k,v in os.environ.items() if not k.startswith('PYTHON') and not k.startswith('GIT_')}
env.update(PYTHONNOUSERSITE='1',PYTHONDONTWRITEBYTECODE='1',GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL=os.devnull,GIT_NO_LAZY_FETCH='1',GIT_NO_REPLACE_OBJECTS='1',GIT_OPTIONAL_LOCKS='0')
results=[]
def run(label,script,*args,identity=None):
 if identity:check(identity,script)
 command=[sys.executable,'-B',str(script)]+[str(x) for x in args]
 write_new(scratch/(label+'.command.private.json'),(json.dumps(command,indent=2)+'\n').encode())
 try:
  r=subprocess.run(command,cwd=scratch,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=180);code=r.returncode;out=r.stdout;err=r.stderr;timed=False
 except subprocess.TimeoutExpired as e:code=None;out=e.stdout or b'';err=e.stderr or b'';timed=True
 except OSError as e:code=None;out=b'';err=(type(e).__name__+'\n').encode();timed=False
 write_new(scratch/(label+'.stdout.private'),out);write_new(scratch/(label+'.stderr.private'),err)
 ok=code==0 and not timed;results.append({'label':label,'exit':code,'timeout':timed,'result':'PASS' if ok else 'FAIL'})
 print(label+': '+('PASS' if ok else 'FAIL'),flush=True)
 return ok
run('checkpoint',pub/'checkpoint-dac7/VERIFY.py')
store_ok=run('selected_store',v6/'VERIFY-STORE.py')
qstore_ok=run('q2904_standalone_store',qstore/'VERIFY-REPLAY.py')
anchor_ok=run('historical_anchors',anchor/'verify-anchor-collection.py')
pack_ok=run('restore_packets',pub/'transport/VERIFY.py','--restore-root',scratch/'packets')
review_ok=run('restore_reviews',pub/'review-transport/VERIFY.py','--restore-root',scratch/'reviews')
extra_ok=run('restore_extra_reviews',pub/'review-extra-transport/VERIFY.py','--restore-root',scratch/'extra')
if pack_ok and store_ok:
 run('q2748',scratch/'packets/q2748/evidence/VERIFY.py','--repo',repo,identity='q2748')
 run('q2754',scratch/'packets/q2754/evidence/VERIFY.py','--git-checkout',repo,'--key-file',col('q2754')/'private-key/KEY.bin',identity='q2754')
if pack_ok and anchor_ok:run('q2896',scratch/'packets/q2896/evidence/verify.py','--commits',anchor,'--anchors',anchor,'--private',col('q2896')/'original',identity='q2896')
if pack_ok and qstore_ok:run('q2904',scratch/'packets/q2904/evidence/VERIFY.py','--repo',qrepo,identity='q2904')
if store_ok:run('q2910',pub/'q2910-packet/VERIFY.py','--key',col('q2910')/'private-key/KEY.bin','--git',repo)
if review_ok and store_ok:
 run('four_review',scratch/'reviews/four-review/evidence/VERIFY-EVIDENCE.py','--key',col('independent-four')/'private-key/KEY.bin','--git',repo,identity='four_review')
if review_ok and anchor_ok:
 run('dialogue_review',scratch/'reviews/dialogue-review/evidence/verify.py','--prior',col('dialogue-review')/'restricted','--v4',col('dialogue-review')/'v4-spotcheck','--anchors',anchor,identity='dialogue_review')
if extra_ok and anchor_ok:
 run('prepared_review',scratch/'extra/prepared-review/evidence/verify.py','--prior',col('prepared-review')/'restricted','--v3',col('prepared-review')/'v3-spotcheck','--anchors',anchor,identity='prepared_review')
if extra_ok and store_ok:
 run('stream_review',scratch/'extra/stream-review/evidence/VERIFY.py','--repo',repo,'--draft-dir',col('stream-review')/'restricted/inputs','--coordinator-dir',col('stream-review')/'restricted/inputs/source',identity='stream_review')
 run('stream_private',col('stream-review')/'restricted/VERIFY-PRIVATE.py','--public',scratch/'extra/stream-review/evidence')
 run('stream_scan_receipts',col('stream-review')/'scan/VERIFY-SCAN.py','--public',scratch/'extra/stream-review/evidence','--private',col('stream-review')/'restricted')
summary={'selected_checks':results,'qualification':'Evidence integrity and selected bindings only. Not a fresh privacy corpus scan, product validation, full Git history, independent custody, queue/lens closure, or checkpoint 3821 recovery. q2945 is outside this driver.'}
write_new(scratch/'RESULTS.json',(json.dumps(summary,indent=2)+'\n').encode())
print('Private receipt directory basename:',scratch.name)
if not all(x['result']=='PASS' for x in results):raise SystemExit(1)
