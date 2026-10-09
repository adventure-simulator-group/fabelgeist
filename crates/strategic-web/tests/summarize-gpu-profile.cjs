// Usage: node summarize-gpu-profile.cjs <gpu-profile directory>
const fs = require('node:fs');
const path = require('node:path');
const directory = path.resolve(process.argv[2]);
const stats = values => {
  const sorted = values.slice().sort((a, b) => a - b);
  const at = quantile => sorted[Math.min(sorted.length - 1, Math.floor(quantile * sorted.length))];
  return {min: sorted[0], median: at(0.5), p95: at(0.95), max: sorted.at(-1)};
};
const histogram = draws => {
  const result = {zero: 0, one: 0, twoToNine: 0, tenTo99: 0, hundredPlus: 0};
  for (const draw of draws) result[draw.instances === 0 ? 'zero' : draw.instances === 1 ? 'one' :
    draw.instances < 10 ? 'twoToNine' : draw.instances < 100 ? 'tenTo99' : 'hundredPlus']++;
  return result;
};
function frameSummary(frame) {
  const draws = frame.passes.flatMap(pass => pass.draws);
  return {cpuMs: frame.cpuMs, gpuPassMs: frame.passes.reduce((sum, pass) => sum + pass.gpuMs, 0),
    renderPasses: frame.passes.filter(pass => pass.kind === 'render').length,
    computePasses: frame.passes.filter(pass => pass.kind === 'compute').length,
    drawCalls: draws.length, indirectCalls: draws.filter(draw => draw.indirect).length,
    nonemptyDraws: draws.filter(draw => draw.instances && draw.elements).length,
    instances: draws.reduce((sum, draw) => sum + draw.instances, 0),
    triangles: draws.reduce((sum, draw) => sum + draw.triangles, 0),
    maximumInstancesPerDraw: Math.max(0, ...draws.map(draw => draw.instances)),
    instanceHistogram: histogram(draws), bufferWrites: frame.bufferWrites, bufferBytes: frame.bufferBytes};
}
const output = {};
for (const name of fs.readdirSync(directory).filter(name => name.endsWith('-gpu.json'))) {
  const capture = JSON.parse(fs.readFileSync(path.join(directory, name), 'utf8'));
  const frames = capture.frames.map(frameSummary), passes = new Map();
  for (const frame of capture.frames) for (const pass of frame.passes) {
    const label = `${pass.kind}:${pass.label}`;
    const summary = passes.get(label) || {gpuMs: [], calls: 0, instances: 0, triangles: 0};
    summary.gpuMs.push(pass.gpuMs); summary.calls += pass.draws.length;
    summary.instances += pass.draws.reduce((sum, draw) => sum + draw.instances, 0);
    summary.triangles += pass.draws.reduce((sum, draw) => sum + draw.triangles, 0);
    passes.set(label, summary);
  }
  output[name] = {capabilities: capture.capabilities, failures: capture.failures, frames,
    aggregate: Object.fromEntries(['cpuMs', 'gpuPassMs', 'drawCalls', 'nonemptyDraws', 'instances', 'triangles', 'bufferBytes']
      .map(key => [key, stats(frames.map(frame => frame[key]))])),
    passes: [...passes].map(([label, summary]) => ({label, occurrences: summary.gpuMs.length,
      gpuMs: stats(summary.gpuMs), callsPerFrame: summary.calls / frames.length,
      instancesPerFrame: summary.instances / frames.length, trianglesPerFrame: summary.triangles / frames.length}))};
  const first = capture.frames[0];
  output[name].firstFramePasses = first.passes.filter(pass => pass.draws.length).map(pass => {
    const labels = new Map();
    for (const draw of pass.draws) {
      const label = draw.pipeline?.label || '(unlabelled)';
      const item = labels.get(label) || {label, calls: 0, instances: 0, triangles: 0};
      item.calls++; item.instances += draw.instances; item.triangles += draw.triangles; labels.set(label, item);
    }
    return {label: pass.label, histogram: histogram(pass.draws), pipelines: [...labels.values()]};
  });
}
for (const name of fs.readdirSync(directory).filter(name => name.endsWith('-timing.json'))) {
  const capture = JSON.parse(fs.readFileSync(path.join(directory, name), 'utf8'));
  output[name] = {cpuMs: stats(capture.frames.map(frame => frame.cpuMs)),
    rafIntervalMs: stats(capture.frames.slice(1).map((frame, index) => frame.rafTime - capture.frames[index].rafTime)),
    bufferBytes: stats(capture.frames.map(frame => frame.bufferBytes))};
}
for (const name of fs.readdirSync(directory).filter(name => name.endsWith('.cpuprofile'))) {
  const profile = JSON.parse(fs.readFileSync(path.join(directory, name), 'utf8'));
  const nodes = new Map(profile.nodes.map(node => [node.id, node]));
  const parents = new Map();
  for (const node of profile.nodes) for (const child of node.children || []) parents.set(child, node.id);
  const samples = new Map();
  for (let i = 0; i < profile.samples.length; i++) samples.set(profile.samples[i],
    (samples.get(profile.samples[i]) || 0) + profile.timeDeltas[i]);
  const families = {
    cpuVisibility: /check_visibility_cpu_culling/,
    visibilityRanges: /visibility::range::check_visibility_ranges/,
    shadowVisibility: /check_dir_light_mesh_visibility|check_light_mesh_visibility|update_directional_light_frusta/,
    modelLayerInheritance: /strategic_scene::inherit_model_layers/,
    transforms: /mark_dirty_trees|propagate_parent_transforms|sync_simple_transforms/,
  };
  const inclusiveMs = Object.fromEntries(Object.keys(families).map(key => [key, 0]));
  for (const [sampleId, microseconds] of samples) {
    const ancestors = []; let cursor = sampleId;
    while (cursor !== undefined) { ancestors.push(nodes.get(cursor).callFrame.functionName); cursor = parents.get(cursor); }
    for (const [key, pattern] of Object.entries(families)) if (ancestors.some(name => pattern.test(name)))
      inclusiveMs[key] += microseconds / 1000;
  }
  output[name] = {durationMs: (profile.endTime - profile.startTime) / 1000, inclusiveMs,
    topSelfTime: [...samples].sort((a, b) => b[1] - a[1]).slice(0, 40).map(([id, microseconds]) =>
      ({milliseconds: microseconds / 1000, ...nodes.get(id).callFrame}))};
}
fs.writeFileSync(path.join(directory, 'summary.json'), JSON.stringify(output, null, 2));
console.log(JSON.stringify(Object.fromEntries(Object.entries(output).map(([name, value]) =>
  [name, value.aggregate || value.cpuMs || value.topSelfTime?.slice(0, 8)])), null, 2));
