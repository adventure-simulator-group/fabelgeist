import assert from "node:assert/strict";
import test from "node:test";
import { mkdirSync, writeFileSync, appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { buildWeapon, validateWeapon } from "../../src/mesh.js";
import { auditPart, auditRelation, publicPartAudit, lodDifferences, POLICY } from "./audit.mjs";
import { overlaps } from "./geometry.mjs";
import { auditExport } from "./export-audit.mjs";
import { corpus, SEEDS } from "./corpus.mjs";

test("deterministic generator corpus satisfies mesh integrity contracts",()=>{
  const profile=process.env.QUALITY_PROFILE??"sweep";
  assert.ok(["defaults","sweep","deep"].includes(profile));
  const directory=resolve(process.env.QUALITY_OUTPUT??`output/mesh-quality/${profile}`);mkdirSync(directory,{recursive:true});
  const jsonl=resolve(directory,"cases.jsonl");writeFileSync(jsonl,"");
  const summary={profile,seeds:SEEDS,policy:POLICY,filters:{preset:process.env.QUALITY_PRESET??null,case:process.env.QUALITY_CASE??null,kind:process.env.QUALITY_KIND??null},cases:0,failedCases:0,reviewCases:0,triangles:0,findings:{},examples:{},presets:{},limitations:["Predicates use documented metre tolerances, not exact arithmetic.","Thin walls use at most 32 deterministic face-centroid rays per default part; aspect and thin-wall budgets are review diagnostics.","Cross-part overlaps are inventoried at defaults; only shield front/fitting exclusion and assembly connectivity have production contracts. General joint envelopes are supported and fixture-tested but require authored per-joint regions.","Source and float32 part geometry get full intersection and topology checks; actual GLB gets buffer, triangle-area and normal checks. Float32 findings report increases in defect counts over the source.","Sweep covers three dimension controls per preset, not all three-way interactions; deep adds each slider endpoint and adjacent step. Neither proves all slider combinations."]};
  const defaults=new Map();let lastProgress=Date.now();
  const specimens=process.env.QUALITY_REPLAY?[JSON.parse(readFileSync(process.env.QUALITY_REPLAY,"utf8"))]:corpus(profile);
  for(const specimen of specimens){
    if(process.env.QUALITY_PRESET&&specimen.preset!==process.env.QUALITY_PRESET)continue;
    if(process.env.QUALITY_CASE&&!specimen.id.includes(process.env.QUALITY_CASE))continue;
    if(process.env.QUALITY_KIND&&!specimen.definition.components.some(c=>c.kind===process.env.QUALITY_KIND))continue;
    const result={id:specimen.id,preset:specimen.preset,variant:specimen.variant,lod:specimen.lod,definition:specimen.definition,findings:[],parts:[],relations:[]};
    const push=(findings,context={})=>result.findings.push(...findings.map(f=>({...f,...context})));
    try{
      const live=validateWeapon(specimen.definition,specimen.controls,{lod:specimen.lod});
      result.productionValidation={valid:live.valid,errors:live.errors};
      // Even a rejected slider combination is worth auditing. Keep this separate
      // from geometry errors because some production rules are practical limits.
      if(!live.valid)push([{code:"production-rejection",severity:"error",count:1,examples:live.errors}]);
      const mesh=live.mesh??buildWeapon(specimen.definition,{lod:specimen.lod});
      const audits=mesh.parts.map((part,index)=>{
        const audit=auditPart(part,{thickness:specimen.variant==="default"});
        summary.triangles+=audit.triangles.length;
        push(audit.findings,{part:index,label:part.label,componentId:part.componentId});
        const rounded=auditPart({...part,positions:part.positions.map(Math.fround),normals:part.normals.map(Math.fround)});
        // Report only precision-induced errors, not source defects twice.
        const original=new Map(audit.findings.map(f=>[f.code,f.count]));
        for(const f of rounded.findings)if(f.severity==="error"&&f.count>(original.get(f.code)??0))push([{...f,code:`float32-${f.code}`,count:f.count-(original.get(f.code)??0)}],{part:index,label:part.label});
        result.parts.push({index,label:part.label,componentId:part.componentId,...publicPartAudit(audit)});
        return audit;
      });
      const exported=auditExport(mesh);push(exported.findings);result.export=exported;
      if(specimen.variant==="default"){
        if(specimen.lod==="low")defaults.set(specimen.preset,audits.map(a=>({shells:a.shells})));
        else if(defaults.has(specimen.preset))push(lodDifferences(defaults.get(specimen.preset),audits));
        // Contact graph: assembled parts must be connected by actual geometry,
        // rather than merely by overlapping attachment-frame bounding boxes.
        const adjacency=audits.map(()=>new Set());
        for(let i=0;i<audits.length;i++)for(let j=i+1;j<audits.length;j++){
          const a=audits[i],b=audits[j];if(!a.bounds||!b.bounds||!overlaps(a.bounds,b.bounds,POLICY.contactMetres))continue;
          const relation=auditRelation(a,b,{mode:"contact"});
          if(relation.contact){adjacency[i].add(j);adjacency[j].add(i);}
          // Record unclassified overlap candidates without declaring ordinary
          // mortises, sleeves, rivets, etc. broken.
          const overlap=auditRelation(a,b,{mode:"separate"});
          if(overlap.findings.length)result.relations.push({parts:[i,j],labels:[mesh.parts[i].label,mesh.parts[j].label],classification:"requires-joint-contract",...overlap});
        }
        const seen=new Set(),groups=[];
        for(let i=0;i<audits.length;i++)if(!seen.has(i)){const stack=[i],group=[];seen.add(i);while(stack.length){const n=stack.pop();group.push(n);for(const next of adjacency[n])if(!seen.has(next)){seen.add(next);stack.push(next);}}groups.push(group);}
        result.contactGroups=groups;
        if(groups.length>1)push([{code:"disconnected-assembly",severity:"error",count:1,examples:groups.map(g=>g.map(i=>({part:i,label:mesh.parts[i].label})))}]);
      }
      // Back fittings must never cross the shield's front surface. Normal Z is
      // evaluated in local component coordinates via the authored rotation.
      for(let i=0;i<mesh.parts.length;i++)if(mesh.parts[i].shieldRole==="body"){
        const component=mesh.resolvedDefinition.components.find(c=>c.id===mesh.parts[i].componentId),rotation=component.rotation??[0,0,0];
        // Derive the transformed front axis without depending on renderer code.
        let front=[0,0,1];
        for(let axis=0;axis<3;axis++){const angle=rotation[axis]*Math.PI/180,c=Math.cos(angle),s=Math.sin(angle),[x,y,z]=front;front=axis===0?[x,y*c-z*s,y*s+z*c]:axis===1?[x*c+z*s,y,-x*s+z*c]:[x*c-y*s,x*s+y*c,z];}
        for(let j=0;j<mesh.parts.length;j++)if(mesh.parts[j].componentId===component.id&&mesh.parts[j].shieldRole==="fitting"){
          // Triangle-region intersection, without interpreting a masked surface
          // as a closed volume for containment.
          const separated=auditRelation(audits[i],audits[j],{mode:"separate",regionA:t=>t.normal.reduce((s,v,k)=>s+v*front[k],0)>0.01});
          push(separated.findings.filter(f=>f.code==="forbidden-contact").map(f=>({...f,code:"shield-front-clipping"})),{parts:[i,j],labels:[mesh.parts[i].label,mesh.parts[j].label]});
          result.relations.push({parts:[i,j],classification:"shield-front-exclusion",contact:separated.findings.some(f=>f.code==="forbidden-contact")});
        }
      }
    }catch(error){push([{code:"audit-or-build-exception",severity:"error",count:1,examples:[{message:error.message,stack:error.stack}]}]);}
    summary.cases++;const bad=result.findings.some(f=>f.severity==="error"),review=result.findings.some(f=>f.severity==="review");
    if(bad)summary.failedCases++;if(review)summary.reviewCases++;
    const preset=summary.presets[result.preset]??{cases:0,failedCases:0,codes:{}};preset.cases++;if(bad)preset.failedCases++;
    for(const f of result.findings){const aggregate=summary.findings[f.code]??{severity:f.severity,cases:0,count:0};aggregate.count+=f.count;summary.findings[f.code]=aggregate;preset.codes[f.code]=(preset.codes[f.code]??0)+f.count;
      const examples=summary.examples[f.code]??[];if(examples.length<8)examples.push({case:result.id,...f});summary.examples[f.code]=examples;}
    for(const code of new Set(result.findings.map(f=>f.code)))summary.findings[code].cases++;
    summary.presets[result.preset]=preset;appendFileSync(jsonl,JSON.stringify(result)+"\n");
    if(Date.now()-lastProgress>10000){console.log(`quality: ${summary.cases} cases, ${summary.failedCases} failing; ${specimen.id}`);lastProgress=Date.now();writeFileSync(resolve(directory,"summary.json"),JSON.stringify(summary,null,2)+"\n");}
  }
  writeFileSync(resolve(directory,"summary.json"),JSON.stringify(summary,null,2)+"\n");
  const lines=["# Weapon mesh quality audit","",`Profile: ${profile}. Cases: ${summary.cases}. Failing cases: ${summary.failedCases}. Source triangles examined: ${summary.triangles}.`,"","| Finding | Severity | Cases | Instances |","| --- | --- | ---: | ---: |",...Object.entries(summary.findings).map(([code,f])=>`| ${code} | ${f.severity} | ${f.cases} | ${f.count} |`),"","## Reproduction","","Exact input definitions, part IDs, triangle IDs, counts and bounded evidence are in `cases.jsonl`. Re-run a case using `QUALITY_PRESET` and `QUALITY_CASE` filters with a distinct `QUALITY_OUTPUT` directory.","","## Coverage limits","",...summary.limitations.map(s=>`- ${s}`),""];
  writeFileSync(resolve(directory,"report.md"),lines.join("\n"));
  console.log(`quality: ${summary.cases} cases, ${summary.failedCases} failing. Evidence: ${directory}`);
  assert.ok(summary.cases>0,"filters must select at least one case");
  assert.equal(summary.failedCases,0,`Mesh integrity failures in ${summary.failedCases}/${summary.cases} cases; see ${directory}/report.md`);
});
