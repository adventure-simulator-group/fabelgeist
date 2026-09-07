// Construction algorithms, not mesh acceptance tests. Each refinement splits
// shared edges once, so caps and walls can consume the same boundary identities.
const edgeKey = (a, b) => a < b ? `${a}:${b}` : `${b}:${a}`;
const area2 = (a, b, c) => (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
const length2 = (a, b) => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2;
export function triangleShape(points, face) {
  const [a, b, c] = face.map(i => points[i]);
  return Math.abs(area2(a, b, c)) / Math.max(length2(a, b), length2(b, c), length2(c, a));
}

// Lawson-style diagonal improvement; polygon boundaries are immutable. This
// improves the worst angle without moving a silhouette or retrying a model.
export function improveDiagonals(points, input) {
  const faces = input.map(face => [...face]);
  for (let pass = 0; pass < 16; pass++) {
    const edges = new Map();
    faces.forEach((face, index) => face.forEach((a, i) => {
      const b = face[(i + 1) % 3], key = edgeKey(a, b), entries = edges.get(key) ?? [];
      entries.push({ index, a, b, opposite: face[(i + 2) % 3] }); edges.set(key, entries);
    }));
    const touched = new Set(); let changed = false;
    for (const entries of edges.values()) {
      if (entries.length !== 2) continue;
      const [x, y] = entries;
      if (touched.has(x.index) || touched.has(y.index)) continue;
      const p = x.opposite, q = y.opposite;
      const candidates = [[p, x.a, q], [p, q, x.b]];
      if (candidates.some(f => area2(...f.map(i => points[i])) <= 1e-16)) continue;
      const before = Math.min(triangleShape(points, faces[x.index]), triangleShape(points, faces[y.index]));
      const after = Math.min(...candidates.map(f => triangleShape(points, f)));
      if (after <= before * (1 + 1e-8)) continue;
      faces[x.index] = candidates[0]; faces[y.index] = candidates[1];
      touched.add(x.index); touched.add(y.index); changed = true;
    }
    if (!changed) break;
  }
  return faces;
}

export function refineRegion(region, maxEdge) {
  const points = region.points.map(p => [...p]);
  let faces = region.triangles, boundary = points.map((_, i) => i);
  // The number of rounds follows directly from the longest input edge.
  let longest = 0;
  for (const f of faces) for (let i = 0; i < 3; i++) longest = Math.max(longest, Math.sqrt(length2(points[f[i]], points[f[(i + 1) % 3]])));
  const rounds = Math.max(0, Math.ceil(Math.log2(longest / maxEdge))) + 1;
  for (let pass = 0; pass < rounds; pass++) {
    const midpoints = new Map();
    for (const f of faces) for (let i = 0; i < 3; i++) {
      const a = f[i], b = f[(i + 1) % 3], key = edgeKey(a, b);
      if (midpoints.has(key) || length2(points[a], points[b]) <= maxEdge ** 2) continue;
      midpoints.set(key, points.length); points.push(points[a].map((v, axis) => (v + points[b][axis]) / 2));
    }
    if (!midpoints.size) break;
    const next = [];
    for (const f of faces) {
      const mids = f.map((a, i) => midpoints.get(edgeKey(a, f[(i + 1) % 3]))), count = mids.filter(i => i !== undefined).length;
      if (!count) next.push(f);
      else if (count === 3) {
        next.push([f[0], mids[0], mids[2]], [mids[0], f[1], mids[1]], [mids[2], mids[1], f[2]], mids);
      } else if (count === 1) {
        const i = mids.findIndex(m => m !== undefined), a = f[i], b = f[(i + 1) % 3], c = f[(i + 2) % 3], m = mids[i];
        next.push([a, m, c], [m, b, c]);
      } else {
        const i = mids.findIndex(m => m === undefined), a = f[i], b = f[(i + 1) % 3], c = f[(i + 2) % 3], bc = mids[(i + 1) % 3], ca = mids[(i + 2) % 3];
        next.push([c, ca, bc]);
        const options = [[[a, b, bc], [a, bc, ca]], [[a, b, ca], [b, bc, ca]]];
        options.sort((x, y) => Math.min(...y.map(t => triangleShape(points, t))) - Math.min(...x.map(t => triangleShape(points, t))));
        next.push(...options[0]);
      }
    }
    boundary = boundary.flatMap((a, i) => {
      const mid = midpoints.get(edgeKey(a, boundary[(i + 1) % boundary.length]));
      return mid === undefined ? [a] : [a, mid];
    });
    faces = improveDiagonals(points, next);
  }
  return { points, triangles: improveDiagonals(points, faces), boundary };
}

export function stationBetween(start, end, fraction) {
  return start + (end - start) * fraction;
}

// Placement uses the receiving face's inward direction, not global minus Y.
export function jointAnchor(target, offset, inward, insertion) {
  return target.map((v, i) => v + (offset[i] ?? 0) + inward[i] * insertion);
}

const subtract = (a, b) => a.map((v, i) => v - b[i]);
const dot = (a, b) => a.reduce((sum, v, i) => sum + v * b[i], 0);
const magnitude = a => Math.hypot(...a);
const clamp01 = v => Math.max(0, Math.min(1, v));
function segmentDistance(a, b, c, d) {
  const u = subtract(b, a), v = subtract(d, c), w = subtract(a, c), aa = dot(u, u), bb = dot(u, v), cc = dot(v, v), dd = dot(u, w), ee = dot(v, w), denominator = aa * cc - bb * bb;
  let s = denominator > aa * cc * 1e-14 ? clamp01((bb * ee - cc * dd) / denominator) : 0;
  let t = cc ? (bb * s + ee) / cc : 0;
  if (t < 0) { t = 0; s = aa ? clamp01(-dd / aa) : 0; }
  if (t > 1) { t = 1; s = aa ? clamp01((bb - dd) / aa) : 0; }
  return magnitude(w.map((x, i) => x + u[i] * s - v[i] * t));
}

export function subdividePath(input, maxChord, scales = null) {
  const points = [], outputScales = [], progress = [];
  for (let i = 0; i < input.length - 1; i++) {
    const steps = Math.max(1, Math.ceil(magnitude(subtract(input[i + 1], input[i])) / maxChord));
    for (let step = 0; step < steps; step++) {
      const t = step / steps;
      points.push(step === 0 ? input[i] : input[i].map((v, axis) => v + (input[i + 1][axis] - v) * t));
      progress.push((i + t) / (input.length - 1));
      outputScales.push((scales?.[i] ?? 1) * (1 - t) + (scales?.[i + 1] ?? 1) * t);
    }
  }
  points.push(input.at(-1)); outputScales.push(scales?.at(-1) ?? 1);
  progress.push(1);
  return { points, scales: outputScales, progress };
}

// Size a bar inside its construction envelope before emitting any triangles.
// Local bends reserve turning room; distant spans reserve disjoint swept
// envelopes. This is a deterministic dimension calculation, not mesh repair.
export function bendScales(points, radius, closed) {
  const n = points.length, scales = new Array(n).fill(1), arc = [0];
  for (let i = 1; i < n; i++) arc.push(arc.at(-1) + magnitude(subtract(points[i], points[i - 1])));
  const count = closed ? n - 1 : n;
  for (let i = 0; i < count; i++) {
    if (!closed && (i === 0 || i === count - 1)) continue;
    const incoming = subtract(points[i], points[(i - 1 + count) % count]), outgoing = subtract(points[(i + 1) % count], points[i]),
      a = magnitude(incoming), b = magnitude(outgoing), cosine = Math.max(-1, Math.min(1, dot(incoming, outgoing) / (a * b))),
      tangentHalf = Math.sqrt((1 - cosine) / Math.max(1e-12, 1 + cosine));
    if (tangentHalf > 1e-8) scales[i] = Math.min(1, 0.3 * Math.min(a, b) / (radius * tangentHalf));
  }
  for (let i = 0; i < n - 1; i++) for (let j = i + 2; j < n - 1; j++) {
    const forward = arc[j] - arc[i + 1], separated = closed ? Math.min(forward, arc.at(-1) - arc[j + 1] + arc[i]) : forward;
    if (separated <= radius * 4) continue;
    const scale = Math.min(1, segmentDistance(points[i], points[i + 1], points[j], points[j + 1]) * 0.3 / radius);
    for (const vertex of [i, i + 1, j, j + 1]) scales[vertex] = Math.min(scales[vertex], scale);
  }
  // A narrow bend also limits its approach and exit. Bound the radius slope
  // so neighboring full-width rings cannot overhang a sharp, narrow corner.
  const limits = [...scales];
  for (let i = 0; i < count; i++) for (let j = 0; j < count; j++) {
    const forward = Math.abs(arc[i] - arc[j]), distance = closed ? Math.min(forward, arc.at(-1) - forward) : forward;
    scales[i] = Math.min(scales[i], limits[j] + distance / (radius * 3));
  }
  if (closed) scales[n - 1] = scales[0] = Math.min(scales[0], scales[n - 1]);
  return scales;
}
