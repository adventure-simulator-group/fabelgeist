// Usage: node summarize-startup-profile.cjs <benchmark directory>
const fs = require("node:fs");
const path = require("node:path");
const directory = path.resolve(process.argv[2]);
const categories = [
  ["Cache restoration / decoding", /wasm_receive_job|generation::receive|strategic-generation-cache|strategic-generation\.js/],
  ["Building planning", /building_generator::generator::generate/],
  ["Building mesh and LOD generation", /building_generator::(?:lod::|detail::)|SolidDetailCompiler/],
  ["Mesh conversion / tangents", /recipe_mesh|generate_tangents|mikktspace/],
  ["Equipment generation", /generate_runtime_equipment|generate_runtime_armor|generate_runtime_clothing|prepare_runtime_equipment_body/],
  ["Terrain / vegetation", /present_pending_terrain|::vista::on_scene_vista_bundle|ground_scatter::|procedural::/],
  ["GPU city assembly", /buildings::gpu::assembly/],
  ["Venue preparation", /strategic_scene::world::prepare/],
  ["Deferred entity changes", /command_queue::.*apply|Commands.*apply|apply_deferred/],
  ["Render preparation / drawing", /bevy_render::run_render_schedule/],
];

function summarize(name) {
  const profile = JSON.parse(fs.readFileSync(path.join(directory, `${name}.cpuprofile`)));
  const trace = JSON.parse(fs.readFileSync(path.join(directory, `${name}-trace.json`)));
  const nodes = new Map(profile.nodes.map(node => [node.id, node]));
  const parents = new Map(profile.nodes.flatMap(node => (node.children || []).map(child => [child, node.id])));
  const stacks = new Map();
  function stack(id) {
    if (!stacks.has(id)) stacks.set(id, [nodes.get(id).callFrame.functionName.replace(/\[[a-f0-9]{8,}\]/g, ""),
      ...(parents.has(id) ? stack(parents.get(id)) : [])]);
    return stacks.get(id);
  }
  const inclusive = new Map(), self = new Map(), buckets = new Map();
  for (let index = 0; index < profile.samples.length; index++) {
    const frames = stack(profile.samples[index]), milliseconds = profile.timeDeltas[index] / 1000;
    for (const frame of new Set(frames)) inclusive.set(frame, (inclusive.get(frame) || 0) + milliseconds);
    self.set(frames[0], (self.get(frames[0]) || 0) + milliseconds);
    const label = frames[0] === "(idle)" ? "Browser idle" :
      categories.find(([, pattern]) => frames.some(frame => pattern.test(frame)))?.[0] || "Other engine / browser";
    buckets.set(label, (buckets.get(label) || 0) + milliseconds);
  }
  const sorted = map => [...map].sort((a, b) => b[1] - a[1]).map(([name, milliseconds]) => ({ name, milliseconds }));
  const states = trace.events.filter(event => event.kind === "readiness");
  const first = predicate => states.find(predicate)?.at;
  const markers = trace.events.filter(event => !["readiness", "receive", "decompress", "cache-read", "renderer", "animation-frame"].includes(event.kind) && !event.kind.startsWith("gpu-"));
  const readiness = {
    textures: first(state => state.textures_ready),
    allCharactersPosed: first(state => state.characters > 0 && state.posed_characters === state.characters),
    equipment: first(state => state.characters > 0 && state.equipment_ready),
    cityInstalled: first(state => state.buildings > 0 && state.city_pending === 0),
    sky: first(state => state.buildings > 0 && state.sky_ready),
    allAssets: first(state => state.assets_ready),
    allSnapshots: first(state => state.assets_ready && state.snapshots_pending === 0),
    ready: first(state => state.ready),
  };
  const restores = trace.events.filter(event => ["receive", "decompress", "cache-read"].includes(event.kind));
  const result = { elapsedMilliseconds: (profile.endTime - profile.startTime) / 1000,
    generation: trace.generation, markers, readiness, buckets: sorted(buckets),
    gpu: require("./startup-gpu-profile.cjs").summarize(trace.events),
    topSelf: sorted(self).slice(0, 70), inclusive: sorted(inclusive),
    restoration: Object.fromEntries(["receive", "decompress", "cache-read"].map(kind => {
      const events = restores.filter(event => event.kind === kind);
      return [kind, { count: events.length, summedMilliseconds: events.reduce((sum, event) => sum + event.duration, 0),
        maxMilliseconds: Math.max(0, ...events.map(event => event.duration)) }];
    })),
    longestMainThreadTasks: trace.tasks.sort((a, b) => b.duration - a.duration).slice(0, 15),
  };
  fs.writeFileSync(path.join(directory, `${name}-summary.json`), JSON.stringify(result, null, 2));
  console.log(name, JSON.stringify({ markers, readiness, buckets: result.buckets, restoration: result.restoration,
    topSelf: result.topSelf.slice(0, 8) }, null, 2));
}
for (const name of ["cold-startup", "reload-startup", "travel-travel-destination", "travel-travel-second", "travel-scene-review"])
  if (fs.existsSync(path.join(directory, `${name}.cpuprofile`))) summarize(name);
