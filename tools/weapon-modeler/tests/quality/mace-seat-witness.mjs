// Independent geometric witnesses for intentionally rejected slider proposals.
// A point outside a receiving half-plane proves impossibility. Finite sampling
// is not used to certify validity or to waive unrelated construction errors.
export function impossibleMaceSeats(definition) {
  const witnesses = [];
  for (const p of definition.components.filter(p => p.kind === "mace")) {
    const sides = p.segments ?? p.flanges;
    const half = p.length / 2;
    const core = p.coreProfile ?? [[-half, p.rootRadius], [half * 0.72, p.rootRadius * 0.9], [half, p.shoulderRadius]];
    const angle = Math.PI / sides;
    const radiusAt = y => {
      const i = core.findIndex((s, i) => i > 0 && s[0] >= y);
      if (i < 1) return core.at(-1)[1];
      const [a, b] = [core[i - 1], core[i]];
      return a[1] + (b[1] - a[1]) * ((y - a[0]) / (b[0] - a[0]));
    };
    const stations = [0, 1, ...core.map(s => (s[0] + half) / p.length).filter(t => t >= 0 && t <= 1)];
    for (const t of stations) {
      const faceWidth = 2 * radiusAt(-half + t * p.length) * Math.sin(angle);
      if (p.flangeThickness > faceWidth + 1e-9) {
        witnesses.push({ diagnostic: "mace flange thickness exceeds its receiving core face", t, faceWidth, thickness: p.flangeThickness });
        break;
      }
    }
    const cusp = p.cuspHeight ?? 0.58;
    const exponent = 1.03 + Math.min(0.98, Math.max(0, p.concavity ?? 0)) * 2.97;
    for (const t of [...stations, ...Array.from({ length: 513 }, (_, i) => i / 512)]) {
      const outer = t <= cusp
        ? p.rootRadius + (p.cuspRadius - p.rootRadius) * (t / cusp) ** exponent
        : p.shoulderRadius + (p.cuspRadius - p.shoulderRadius) * ((1 - t) / (1 - cusp)) ** exponent;
      const seat = radiusAt(-half + t * p.length) * Math.cos(angle);
      if (seat > outer + 1e-9) {
        witnesses.push({ diagnostic: "mace core face reaches outside the flange outline", t, seat, outer });
        break;
      }
    }
  }
  return witnesses;
}
