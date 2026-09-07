import { readFileSync, writeFileSync } from "node:fs";
import { buildWeapon } from "../../src/mesh.js";
import { auditPart } from "./audit.mjs";
import { closestSurfaces } from "./geometry.mjs";

const [rowsPath,id,output]=process.argv.slice(2);
if(!output)throw new Error("usage: node tests/quality/gaps.mjs cases.jsonl case-id output.json");
const row=readFileSync(rowsPath,"utf8").trim().split("\n").map(JSON.parse).find(r=>r.id===id);
if(!row?.contactGroups)throw new Error("requires a recorded default case with contact groups");
const mesh=buildWeapon(row.definition,{lod:row.lod}),audits=mesh.parts.map(p=>auditPart(p,{intersections:false})),gaps=[];
for(let i=0;i<row.contactGroups.length;i++)for(let j=i+1;j<row.contactGroups.length;j++){
  let nearest={metres:Infinity};
  for(const a of row.contactGroups[i])for(const b of row.contactGroups[j]){
    const gap=closestSurfaces(audits[a].tree,audits[b].tree);
    if(gap.metres<nearest.metres)nearest={...gap,parts:[a,b],labels:[mesh.parts[a].label,mesh.parts[b].label]};
  }
  gaps.push({groups:[i,j],...nearest});
}
const result={id,definition:row.definition,lod:row.lod,gaps};writeFileSync(output,JSON.stringify(result,null,2)+"\n");console.log(JSON.stringify({id,gaps}));
