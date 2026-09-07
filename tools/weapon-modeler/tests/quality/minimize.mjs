import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { PRESETS, getControlValue, setControlValue } from "../../src/presets.js";
import { validateWeapon } from "../../src/mesh.js";
import { auditPart } from "./audit.mjs";

// Minimize against resetting controls to the preset defaults. This finds a
// deterministic 1-minimal set of changed controls, not a mathematical global
// minimum over all possible slider values.
export function minimizeChanges(changes,stillFails,budget=200){
  let current=[...changes],evaluations=0,exhausted=false;
  const check=xs=>{if(evaluations>=budget){exhausted=true;return false;}evaluations++;return stillFails(xs);};
  if(!check(current))throw new Error("input does not reproduce the requested failure");
  for(let width=Math.max(1,Math.ceil(current.length/2));width>=1;width=Math.floor(width/2)){
    let changed=true;
    while(changed&&!exhausted){changed=false;for(let i=0;i<current.length;i+=width){const candidate=current.filter((_,j)=>j<i||j>=i+width);if(check(candidate)){current=candidate;changed=true;break;}}}
  }
  return {changes:current,evaluations,complete:!exhausted};
}
export function minimizeRecordedCase(rowsPath,id,code,outputPath){
  const specimen=readFileSync(rowsPath,"utf8").trim().split("\n").map(JSON.parse).find(r=>r.id===id);
  if(!specimen)throw new Error(`case not found: ${id}`);
  const source=PRESETS.find(p=>p.id===specimen.preset);if(!source)throw new Error("minimization requires a named preset");
  const target=specimen.findings.find(f=>f.code===code&&f.part!==undefined);if(!target)throw new Error("requires a recorded per-part finding");
  const controls=[...(source.choiceControls??[]),...source.controls];
  const changes=controls.map((control,index)=>({index,label:control.label,from:getControlValue(source.definition,control),to:getControlValue(specimen.definition,control)})).filter(c=>c.from!==c.to);
  const construct=active=>{const definition=structuredClone(specimen.definition),keep=new Set(active.map(c=>c.index));for(const c of changes)if(!keep.has(c.index))setControlValue(definition,controls[c.index],c.from);return definition;};
  const reduced=minimizeChanges(changes,active=>{
    const result=validateWeapon(construct(active),[],{lod:specimen.lod});if(!result.valid)return false;
    const part=result.mesh.parts.find(p=>p.label===target.label&&p.componentId===target.componentId);
    return part&&auditPart(part).findings.some(f=>f.code===code);
  });
  const definition=construct(reduced.changes),result=validateWeapon(definition,[],{lod:specimen.lod});
  const part=result.mesh.parts.find(p=>p.label===target.label&&p.componentId===target.componentId);
  const output={id:`${specimen.id}/minimized`,originalCase:specimen.id,preset:specimen.preset,variant:"minimized",lod:specimen.lod,definition,controls:[],target:{code,label:target.label,componentId:target.componentId},...reduced,evidence:auditPart(part).findings.find(f=>f.code===code)};
  writeFileSync(outputPath,JSON.stringify(output,null,2)+"\n");return output;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
  const [rows,id,code,output]=process.argv.slice(2);if(!output)throw new Error("usage: node tests/quality/minimize.mjs cases.jsonl case-id finding-code output.json");
  const result=minimizeRecordedCase(rows,id,code,output);console.log(JSON.stringify({id:result.id,changes:result.changes,evaluations:result.evaluations,complete:result.complete}));
}
