#!/usr/bin/env python3
"""Read-only check for this privacy-reconstruction proposal.

By default checks only the self-contained public stage. Optional original and
publication roots strengthen verification, without modifying those roots.
This does not execute any historical verifier or any product code.
"""
import argparse, hashlib, json, pathlib, re, subprocess, sys

BASE = pathlib.Path(__file__).resolve().parent

def digest(b): return hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def pointer_escape(s): return s.replace('~','~0').replace('/','~1')

class Tokens:
    def __init__(self, raw):
        self.raw=raw; self.n=len(raw); self.i=0; self.strings={}
        value=self.value(''); self.ws()
        if self.i != self.n or value != json.loads(raw): raise ValueError('JSON tokenization mismatch')
        self.value_tree=value
    def ws(self):
        while self.i < self.n and self.raw[self.i] in b' \t\r\n': self.i+=1
    def string(self):
        start=self.i
        if self.raw[self.i] != 34: raise ValueError('expected string')
        self.i+=1
        while self.i < self.n:
            c=self.raw[self.i]; self.i+=1
            if c==34:
                end=self.i
                return start,end,json.loads(self.raw[start:end])
            if c==92:
                if self.i>=self.n: raise ValueError('bad escape')
                self.i+=1
        raise ValueError('unterminated string')
    def value(self, ptr):
        self.ws()
        if self.i>=self.n: raise ValueError('unexpected EOF')
        c=self.raw[self.i]
        if c==34:
            s,e,v=self.string(); self.strings[ptr]=(s,e,v); return v
        if c==123:
            self.i+=1; self.ws(); out={}
            if self.raw[self.i:self.i+1]==b'}': self.i+=1; return out
            while True:
                self.ws(); _,_,key=self.string()
                if key in out: raise ValueError('duplicate JSON key')
                self.ws()
                if self.raw[self.i:self.i+1]!=b':': raise ValueError('missing colon')
                self.i+=1; out[key]=self.value(ptr+'/'+pointer_escape(key)); self.ws()
                sep=self.raw[self.i:self.i+1]; self.i+=1
                if sep==b'}': return out
                if sep!=b',': raise ValueError('bad object')
        if c==91:
            self.i+=1; self.ws(); out=[]
            if self.raw[self.i:self.i+1]==b']': self.i+=1; return out
            while True:
                out.append(self.value(ptr+'/'+str(len(out)))); self.ws()
                sep=self.raw[self.i:self.i+1]; self.i+=1
                if sep==b']': return out
                if sep!=b',': raise ValueError('bad array')
        s=self.i
        while self.i<self.n and self.raw[self.i] not in b' \t\r\n,]}': self.i+=1
        return json.loads(self.raw[s:self.i])

def leaves(v, ptr=''):
    if isinstance(v,dict):
        for k,x in v.items():
            yield ptr+'/<key>', k
            yield from leaves(x, ptr+'/'+pointer_escape(k))
    elif isinstance(v,list):
        for i,x in enumerate(v): yield from leaves(x, ptr+'/'+str(i))
    elif isinstance(v,str): yield ptr,v

def privacy_check(raw, decoded, original_home=None):
    # Work with decoded JSON strings as well as bytes, so escaped newlines,
    # unicode escapes and escaped slashes cannot bypass path checks.
    home=original_home or str(pathlib.Path.home()); failures=[]
    if home.encode() in raw or home.replace('/','\\/').encode() in raw:
        failures.append('local-home literal')
    path=re.compile(r'(?:^|[\s"\'=:(])/(?:home|Users|tmp|root)/[^\s"\'<>]+',re.I)
    token=re.compile(r'/' + r'root/(?:lens12_frontier|rig_identity|hair_ops)(?=$|[\s,;:.)\(])')
    secrets=[re.compile(p) for p in [r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----',r'\bAKIA[0-9A-Z]{16}\b',r'\bgh[pousr]_[A-Za-z0-9_]{30,}\b',r'\bgithub_pat_[A-Za-z0-9_]{40,}\b',r'\bxox[baprs]-[A-Za-z0-9-]{20,}\b',r'\bsk-[A-Za-z0-9_-]{32,}\b',r'\bglpat-[A-Za-z0-9_-]{20,}\b',r'\bnpm_[A-Za-z0-9]{30,}\b']]
    for ptr, value in leaves(decoded):
        if home in value: failures.append('home in decoded value')
        # These three exact canonical strings are historical agent identifiers,
        # not filesystem paths. Remove only these exact tokens for this scan.
        normalized=value.replace('\\n','\n').replace('\\r','\r').replace('\\/','/')
        normalized=re.sub(r'\\u002[fF]', '/', normalized)
        normalized=token.sub('AGENT_ID',normalized)
        if path.search(normalized) or re.search(r'\b[A-Za-z]:\\Users\\',normalized,re.I):
            failures.append('potential absolute user path')
        if any(p.search(value) for p in secrets): failures.append('potential secret')
    return failures

def require(ok, message):
    if not ok: raise ValueError(message)

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--originals-root',type=pathlib.Path)
    ap.add_argument('--publication-root',type=pathlib.Path)
    ap.add_argument('--original-home-prefix',default=str(pathlib.Path.home()),help='Private original path prefix; defaults to current user home.')
    args=ap.parse_args()
    mf=json.loads((BASE/'MANIFEST.json').read_bytes())
    listed={x['path']:x for x in mf['files']}
    found={p.relative_to(BASE).as_posix() for p in BASE.rglob('*') if p.is_file() and p!=BASE/'MANIFEST.json'}
    require(found==set(listed),'manifest file set mismatch')
    for rel, rec in listed.items():
        b=(BASE/rel).read_bytes(); require(len(b)==rec['bytes'] and digest(b)==rec['sha256'],'manifest identity mismatch')
    cross=json.loads((BASE/'CROSSWALK.json').read_bytes()); inv=json.loads((BASE/'DEPENDENCY-INVENTORY.json').read_bytes())
    count=0
    for row in cross['objects']:
        raw=(BASE/row['variant']['path']).read_bytes()
        require(digest(raw)==row['variant']['sha256'] and len(raw)==row['variant']['bytes'] and blob(raw)==row['variant']['git_blob_sha1'],'variant identity mismatch')
        tokens=Tokens(raw); require(not privacy_check(raw,tokens.value_tree,args.original_home_prefix),'variant privacy pattern detected')
        for red in row['redactions']:
            s,e,v=tokens.strings[red['json_pointer']]
            require((s,e-s,v)==(red['public_byte_offset'],red['public_token_bytes'],red['replacement']),'public span mismatch'); count+=1
        if args.originals_root:
            old=(args.originals_root/row['original']['sha256']).read_bytes()
            require(digest(old)==row['original']['sha256'] and len(old)==row['original']['bytes'] and blob(old)==row['original']['git_blob_sha1'],'original identity mismatch')
            ot=Tokens(old); patches=[]
            home=args.original_home_prefix.rstrip('/')+'/'
            home_pointers={ptr for ptr,(_,_,value) in ot.strings.items() if home in value}
            require(old.count(args.original_home_prefix.encode())==len(row['redactions']),'unexpected raw home-prefix occurrence')
            require(not any(home in value for ptr,value in leaves(ot.value_tree) if ptr.endswith('/<key>')),'home prefix in JSON key')
            require(home_pointers=={red['json_pointer'] for red in row['redactions']},'incomplete original home-field mapping')
            for red in row['redactions']:
                s,e,v=ot.strings[red['json_pointer']]
                require((s,e-s)==(red['original_byte_offset'],red['original_token_bytes']),'original span mismatch')
                require(v.startswith(args.original_home_prefix.rstrip('/')+'/'),'original field does not have expected home prefix')
                ps,pe,_=tokens.strings[red['json_pointer']]; patches.append((s,e,raw[ps:pe]))
            rebuilt=old
            for s,e,replace in sorted(patches,reverse=True): rebuilt=rebuilt[:s]+replace+rebuilt[e:]
            require(rebuilt==raw,'non-redacted bytes differ')
    require(count==cross['redaction_count'],'redaction count mismatch')
    history=json.loads((BASE/'HISTORICAL-QUALIFICATIONS.json').read_bytes())
    first=Tokens((BASE/cross['objects'][0]['variant']['path']).read_bytes())
    old_agent=history['objects'][0]['historical_read_agent']
    agent_locs=sorted(ptr for ptr,(_,_,value) in first.strings.items() if old_agent in value)
    require(agent_locs==sorted(history['objects'][0]['agent_identity_locations']),'historical read attribution mismatch')
    if args.publication_root:
        root=args.publication_root
        for dep in inv['files']:
            b=(root/dep['path']).read_bytes(); require(len(b)==dep['bytes'] and digest(b)==dep['sha256'],'observed dependency drift')
        def at(obj, pointer):
            for part in pointer.lstrip('/').split('/'):
                part=part.replace('~1','/').replace('~0','~')
                obj=obj[int(part)] if isinstance(obj,list) else obj[part]
            return obj
        for binding in inv['direct_bindings']:
            doc=json.loads((root/binding['file']).read_bytes())
            for ptr in binding.get('pointers',[binding.get('pointer')]):
                require(at(doc,ptr)==binding['original_sha256'],'direct manifest binding mismatch')
        for row in cross['objects']:
            old=(root/row['original']['publication_path']).read_bytes()
            require(digest(old)==row['original']['sha256'] and len(old)==row['original']['bytes'],'published original drift')
            out=subprocess.run(['git','-C',str(root),'ls-tree',cross['observed_git']['local_head'], '--',row['original']['publication_path']],capture_output=True,check=True,text=True).stdout.strip()
            require(out and out.split()[2]==row['original']['git_blob_sha1'],'observed commit/tree binding mismatch')
            out=subprocess.run(['git','-C',str(root),'ls-tree',cross['observed_git']['locally_cached_remote_ref'],'--',row['original']['publication_path']],capture_output=True,check=True,text=True).stdout.strip()
            require(out and out.split()[2]==row['original']['git_blob_sha1'],'cached remote commit/tree binding mismatch')
        for item in inv['indexed_original_to_public_relationships']:
            doc=json.loads((root/item['index']).read_bytes()); rec=at(doc,item['pointer'])
            require(rec['original_sha256']==item['original_sha256'] and rec['published_sha256']==item['published_sha256'] and rec['path']==item['member_path'],'indexed historical identity mismatch')
        provenance=cross['objects'][0]['provenance']
        archive=(root/provenance['archive']).read_bytes()
        require(digest(archive)==provenance['archive_sha256'] and len(archive)==provenance['archive_bytes'],'published archive drift')
        import tarfile,io
        with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as tf:
            member=tf.extractfile(provenance['member']).read()
        require(member==(BASE/cross['objects'][0]['variant']['path']).read_bytes(),'published member mismatch')
        provenance=cross['objects'][1]['provenance']
        require((root/provenance['prior_public_path']).read_bytes()==(BASE/cross['objects'][1]['variant']['path']).read_bytes(),'published prior correction mismatch')
    # Scan every public textual stage member, including decoded strings where JSON.
    for p in BASE.rglob('*'):
        if not p.is_file(): continue
        raw=p.read_bytes()
        try:
            obj=json.loads(raw) if p.suffix=='.json' else {'text':raw.decode('utf-8')}
        except (UnicodeDecodeError,json.JSONDecodeError): raise ValueError('non-text public stage member')
        require(not privacy_check(raw,obj,args.original_home_prefix),'public stage privacy pattern detected')
    print(json.dumps({'status':'pass','public_file_count':len(found),'objects':len(cross['objects']),'redaction_spans':count,'originals_checked':bool(args.originals_root),'publication_inventory_checked':bool(args.publication_root),'limitations':'Pattern scanning is bounded; verified redacted content requires private originals. No historical verifier or product code was executed.'},indent=2))

if __name__=='__main__':
    try: main()
    except Exception as exc:
        print('Verification failed:',type(exc).__name__,str(exc),file=sys.stderr); sys.exit(1)
