// Presentation transport only. All recipe construction runs in the Rust kernel.
let request;
if (typeof window === "undefined") {
  const module = await import("../../../target/weapon-modeler-kernel/nodejs/weapon-kernel.js");
  request = module.weapon_model_request ?? module.default.weapon_model_request;
} else {
  const module = await import("/kernel/weapon-kernel.js");
  await module.default();
  request = module.weapon_model_request;
}

export function generateModel(recipe, { lod = "medium" } = {}) {
  return JSON.parse(request(JSON.stringify({ operation: "model", recipe, detail: lod })));
}
export function kernelRequest(value) { return JSON.parse(request(JSON.stringify(value))); }
export function validateWeapon(recipe, controls = [], { lod = "medium" } = {}) { try { const mesh = kernelRequest({ operation: "validate-model", recipe, controls, detail: lod }); return { valid: true, errors: [], resolved: mesh.resolvedDefinition, mesh }; } catch (error) { return { valid: false, errors: [String(error)], resolved: null, mesh: null }; } }
