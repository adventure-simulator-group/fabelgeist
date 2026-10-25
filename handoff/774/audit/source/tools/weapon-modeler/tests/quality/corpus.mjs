import { PRESETS, copyPreset, controlVisible, getControlValue, setControlValue, HAFT_MODULES, HEAD_ASSEMBLIES, composeWeapon, compositionControls } from "../../src/presets.js";
import { adversarialReviewCases } from "../../src/review-cases.js";

export const SEEDS=[0x1544cafe,0x1544beef,0x1544a11];
function random(seed,id){let state=[...id].reduce((s,c)=>Math.imul(s^c.charCodeAt(0),16777619)>>>0,seed);return ()=>((state=(Math.imul(state,1664525)+1013904223)>>>0)/4294967296);}
const at=(control,step)=>Number((control.min+step*control.step).toFixed(10));
const steps=c=>Math.floor((c.max-c.min)/c.step+1e-8);
export function* corpus(profile="sweep"){
  for(const source of PRESETS){
    for(const lod of ["low","medium","high"])yield {id:`${source.id}/default/${lod}`,preset:source.id,variant:"default",lod,definition:structuredClone(source.definition),controls:source.controls};
    if(profile==="defaults")continue;
    const emit=(definition,variant)=>({id:`${source.id}/${variant}/low`,preset:source.id,variant,lod:"low",definition,controls:source.controls});
    for(const mode of ["min","max","alternating","min-step","max-step"]){
      const p=copyPreset(source);
      p.controls.filter(c=>controlVisible(p.definition,c)).forEach((c,i)=>setControlValue(p.definition,c,at(c,mode==="min"?0:mode==="max"?steps(c):mode==="min-step"?Math.min(1,steps(c)):mode==="max-step"?Math.max(0,steps(c)-1):i%2?0:steps(c))));
      yield emit(p.definition,`all-${mode}`);
    }
    for(const seed of SEEDS){
      const p=copyPreset(source),rng=random(seed,source.id);
      for(const c of p.choiceControls)if(controlVisible(p.definition,c))setControlValue(p.definition,c,c.options[Math.floor(rng()*c.options.length)].value);
      for(const c of p.controls)if(controlVisible(p.definition,c))setControlValue(p.definition,c,at(c,Math.floor(rng()*(steps(c)+1))));
      yield emit(p.definition,`seed-${seed}`);
    }
    for(const [i,c]of (source.choiceControls??[]).entries())if(controlVisible(source.definition,c))for(const option of c.options){
      if(option.value===getControlValue(source.definition,c))continue;
      const p=copyPreset(source);setControlValue(p.definition,c,option.value);yield emit(p.definition,`choice-${i}-${String(option.value)}`);
    }
    // A three-way interaction of construction dimensions, not just equal-valued
    // endpoints. Each case records the exact definition for future replay.
    const dimensions=source.controls.filter(c=>controlVisible(source.definition,c)&&/length|width|thickness|curve|radius|depth/i.test(c.label)).slice(0,3);
    if(dimensions.length===3)for(let bits=0;bits<8;bits++){
      const p=copyPreset(source);dimensions.forEach((c,i)=>setControlValue(p.definition,c,bits&(1<<i)?c.max:c.min));
      yield emit(p.definition,`dimension-triple-${bits}`);
    }
    if(profile==="deep")for(const [i,c]of source.controls.entries())if(controlVisible(source.definition,c))for(const step of new Set([0,1,Math.max(0,steps(c)-1),steps(c)])){
      const p=copyPreset(source);setControlValue(p.definition,c,at(c,Math.min(step,steps(c))));yield emit(p.definition,`control-${i}-step-${step}`);
    }
  }
  if(profile!=="defaults"){
    for(const specimen of adversarialReviewCases())yield {id:`adversarial/${specimen.id}/low`,preset:specimen.id,variant:"adversarial",lod:"low",definition:specimen.definition,controls:[]};
    for(const haft of HAFT_MODULES)for(const head of HEAD_ASSEMBLIES){
      const definition=composeWeapon(haft.id,head.id),controls=compositionControls(definition);
      yield {id:`composer/${haft.id}/${head.id}/low`,preset:`${haft.id}+${head.id}`,variant:"composer",lod:"low",definition,controls};
    }
  }
}
