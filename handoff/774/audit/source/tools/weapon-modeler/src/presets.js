// Catalog and composition are owned by the native/WASM weapon kernel.
import { kernelRequest } from './kernel.js';
const catalog=kernelRequest({operation:'authoring-catalog'});
export const PRESETS=catalog.presets;
export const MUSEUM_STUDIES=kernelRequest({operation:"museum-studies"});
export const HAFT_MODULES=catalog.hafts;
export const HEAD_ASSEMBLIES=catalog.heads;
const deepCopy=value=>structuredClone(value);
export function composeWeapon(haft,head) {return kernelRequest({operation:'compose',haft,head});}
export function compositionControls(recipe) {return kernelRequest({operation:'composition-controls',recipe});}

export const HEAD_KINDS = ["axe", "hammer", "beak", "spear", "blade", "mace"];
export function copyPreset(preset) {
  return {
    ...preset,
    definition: deepCopy(preset.definition),
    controls: deepCopy(preset.controls),
    choiceControls: deepCopy(preset.choiceControls ?? []),
  };
}
export function getPath(object, path) {
  return path.split(".").reduce((current, part) => current[part], object);
}
export function controlVisible(definition, control) {
  const evaluate = (condition) => {
    if (!condition) return true;
    if (condition.all) return condition.all.every(evaluate);
    if (condition.any) return condition.any.some(evaluate);
    const value = getPath(definition, condition.path);
    return condition.equals !== undefined ? value === condition.equals : condition.in.includes(value);
  };
  return evaluate(control.when);
}
export function setPath(object, path, value) {
  const parts = path.split(".");
  const key = parts.pop();
  parts.reduce((current, part) => current[part], object)[key] = value;
}
export function getControlValue(object, control) {
  if (control.target === "shaft") return object.shaft?.[control.key];
  if (control.componentId) return object.components.find((part) => part.id === control.componentId)?.[control.key];
  return getPath(object, control.path ?? control.paths[0]);
}
export function setControlValue(object, control, value) {
  if (control.target === "shaft") {
    object.shaft[control.key] = value;
    return;
  }
  if (control.componentId) {
    const part = object.components.find((candidate) => candidate.id === control.componentId);
    if (!part) throw new Error(`missing control component ${control.componentId}`);
    part[control.key] = value;
    return;
  }
  for (const path of control.paths ?? [control.path]) setPath(object, path, value);
}
