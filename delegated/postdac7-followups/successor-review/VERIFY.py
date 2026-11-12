#!/usr/bin/env python3
"""Read-only verification of the follow-ups 16 successor review."""
import argparse,copy,hashlib,json,os,stat,subprocess
from pathlib import Path
R=Path(__file__).resolve().parent
def need(c,m):
    if not c:raise SystemExit('FAIL: '+m)
def sha(b):return hashlib.sha256(b).hexdigest()
def j(n):return json.loads((R/n).read_text())
def replay(old,new,c,added):
    state=copy.deepcopy(old);seen=set()
    for row in c['changes']:
        p=row['path'];need(p not in seen,'duplicate correction path');seen.add(p);parts=p.strip('/').split('/')
        if parts[0]=='ledger_relationships':
            hits=[i for i,e in enumerate(state['ledger_relationships']) if e['id']==parts[1]];need(len(hits)==1,'unknown relationship ID');i=hits[0];obj=state['ledger_relationships'][i];actual=new['ledger_relationships'][i][parts[2]];oldval=obj[parts[2]];obj[parts[2]]=row['after']
        else:
            oldval=state[parts[0]];actual=new[parts[0]];state[parts[0]]=row['after']
        need(oldval==row['before'] and actual==row['after'],'correction before/after mismatch')
    for k in added:
        need(k not in state and k in new,'invalid added field');state[k]=new[k]
    need(state==new,'unrecorded JSON change');return seen
ap=argparse.ArgumentParser();ap.add_argument('--repo');ap.add_argument('--prior-manifest');a=ap.parse_args();m=j('MANIFEST.json')['files'];found=set()
for p in R.rglob('*'):
    need(not p.is_symlink(),'symlink in packet')
    if p.is_file() and p!=R/'MANIFEST.json':found.add(p.relative_to(R).as_posix())
need(found==set(m),'manifest file set')
for n,v in m.items():
    b=(R/n).read_bytes();need(len(b)==v['bytes'] and sha(b)==v['sha256'],'manifest file identity')
x=[j('inputs/report-v1.json'),j('inputs/report-v2.json'),j('inputs/report-v3.json')];c1=j('inputs/corrections.json');c2=j('inputs/corrections-v2.json');ident=j('INPUT-IDENTITIES.json')
for k in ['assessments','counts','draft_predecessors']:
    need(x[0][k]==x[1][k]==x[2][k],'unchanged assessment/count/draft field')
need(len(x[2]['assessments'])==7 and len({e['queue_index'] for e in x[2]['assessments']})==7,'assessment count')
rows=x[2]['ledger_relationships'];ids=[e['id'] for e in rows];need(len(rows)==9 and len(set(ids))==9,'relationship count')
for key in ['id','lens','historical_related_records','current_related_records','repair','validation']:
    need([[e[key] for e in z['ledger_relationships']] for z in x].count([e[key] for e in rows])==3,'unchanged relationship field')
need(all(e['record']=='ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E-v3.json' for e in rows),'v3 record pointer')
need(c1['original_sha256']==sha((R/'inputs/report-v1.json').read_bytes()) and c1['successor_sha256']==sha((R/'inputs/report-v2.json').read_bytes()),'v2 report identity');need(c2['original_sha256']==sha((R/'inputs/report-v2.json').read_bytes()) and c2['successor_sha256']==sha((R/'inputs/report-v3.json').read_bytes()),'v3 report identity')
s1=replay(x[0],x[1],c1,c1['added_fields']);s2=replay(x[1],x[2],c2,[])
need(len(s1)==17 and set(c1['added_fields'])=={'supersedes_report','successor_corrections'} and len(s2)==12,'correction scope/count')
allowed={'relationship-dialogue-multi-ballot-order-95e','relationship-dialogue-claim-occurrence-identity-95e','relationship-vista-cache-prepared-asset-95e','relationship-prepared-venue-take-retry-95e','relationship-regional-city-cross-request-failure-95e','relationship-selected-production-expect-rule-95e'}
expected1={'/ledger_relationships/'+i+'/record' for i in ids}|{'/ledger_relationships/'+i+'/classification' for i in allowed}|{'/independent_review','/recovery_qualification'}
expected2={'/ledger_relationships/'+i+'/record' for i in ids}|{'/ledger_relationships/relationship-regional-install-phase-error-shape-95e/classification','/supersedes_report','/successor_corrections'}
need(s1==expected1 and s2==expected2,'unexpected correction path')
review=x[2]['independent_review'];need(all(t in review for t in ['/root/verifier_scope','/root/wrapped_scope','/root/mhr_character','not claimed as separately reviewed']),'review attribution')
conf=j('inputs/reviewer-confirmation.json');need(conf['respondent']=='/root/wrapped_scope' and len(conf['confirmed_roles'])==2,'reviewer confirmation')
q=x[2]['recovery_qualification'];need(all(t in q for t in ['3821','440','774','SOURCE004','73 nontext']),'recovery caveats')
rec=j('inputs/recovery-440-manifest.json');snap=(R/'inputs/recovery-440-snapshot.dat').read_bytes();need(sha(snap)==rec['exact_original_snapshot']['sha256'] and sha(snap)==ident['published_440_snapshot']['sha256'],'440 snapshot hash');need(rec['missing_selected_state_lenses']==[2,12,13] and len(rec['values'])==12 and len(rec['additional_verified_state_objects'])==8 and bool(rec['lens1_state_object']) and bool(rec['lens4_state_object']),'440 recorded population')
need('neither' in c2['chronology_correction'] and 'staging' in c2['chronology_correction'],'staging chronology qualification')
for meta,local in [(ident['published_recovery_readme'],'inputs/recovery-0.dat'),(ident['published_440_manifest'],'inputs/recovery-440-manifest.json'),(ident['published_440_snapshot'],'inputs/recovery-440-snapshot.dat')]:
    b=(R/local).read_bytes();gitid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();need(sha(b)==meta['sha256'] and gitid==meta['git_blob'],'local recovery blob identity')
bound={'prior_review_manifest':False,'published_recovery_git':False}
if a.prior_manifest:
    p=Path(a.prior_manifest);b=p.read_bytes();need(sha(b)==ident['previous_review_manifest_sha256'],'prior manifest identity');files=json.loads(b)['files']
    for name,public in [('ASSESSMENT.json','inputs/report-v1.json'),('REPORT.md','inputs/prior-review.md'),('ATTRIBUTION-CONFIRMATION.json','inputs/reviewer-confirmation.json')]:need(files[name]['sha256']==sha((R/public).read_bytes()),'prior manifest member')
    bound['prior_review_manifest']=True
if a.repo:
    env=dict(os.environ,GIT_NO_REPLACE_OBJECTS='1',GIT_OPTIONAL_LOCKS='0');rev='dac7d0055716053f245139971eb67ddfd87fa462'
    for meta,path in [(ident['published_recovery_readme'],'git_path'),(ident['published_440_manifest'],'path'),(ident['published_440_snapshot'],'path')]:
        cp=subprocess.run(['git','-C',a.repo,'rev-parse',rev+':'+meta[path]],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env);need(cp.returncode==0 and cp.stdout.decode().strip()==meta['git_blob'],'published evidence blob binding')
    cp=subprocess.run(['git','-C',a.repo,'ls-tree','--name-only',rev,'--','ASSESSMENT-CENTRAL-FOLLOWUPS-16-95E.json'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env);need(cp.returncode==0 and cp.stdout==b'','old report absent at dac7d');bound['published_recovery_git']=True
print(json.dumps({'result':'PASS','manifest_files':len(m),'unchanged_assessments':7,'relationships':9,'v2_recorded_changes':len(s1),'v3_recorded_changes':len(s2),'recorded_missing_440_lenses':[2,12,13],'external_bindings':bound,'qualification':'Evidence and structural checks only. Ten underlying 440 state objects, complete SOURCE004/774 history, later publication, ledger integration and product behavior are not validated.'},indent=2))
