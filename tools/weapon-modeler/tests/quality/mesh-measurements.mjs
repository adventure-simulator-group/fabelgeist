// Independent numerical audit: this file does not construct weapons.
import {triangleVertices} from "../../src/topology.js";
export function signedVolume(mesh) {
  let volume = 0;
  for (const [a, b, c] of triangleVertices(mesh)) {
    volume += (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0])) / 6;
  }
  return volume;
}


function tetrahedralMassIntegrals(mesh, density, controlPoint) {
  let mass = 0;
  const firstMoment = [0, 0, 0];
  let transverseMoment = 0;
  for (const vertices of triangleVertices(mesh)) {
    const [a, b, c] = vertices;
    const signedVolume = (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0])) / 6;
    const tetraMass = density * signedVolume;
    mass += tetraMass;
    for (let axis = 0; axis < 3; axis += 1) firstMoment[axis] += (tetraMass * (a[axis] + b[axis] + c[axis])) / 4;
    const secondMoment = (axis) =>
      (density * signedVolume * (a[axis] ** 2 + b[axis] ** 2 + c[axis] ** 2 + a[axis] * b[axis] + a[axis] * c[axis] + b[axis] * c[axis])) / 10;
    const shiftedSecondMoment = (axis) => secondMoment(axis) - 2 * controlPoint[axis] * ((tetraMass * (a[axis] + b[axis] + c[axis])) / 4) + controlPoint[axis] ** 2 * tetraMass;
    transverseMoment += shiftedSecondMoment(1) + (shiftedSecondMoment(0) + shiftedSecondMoment(2)) / 2;
  }
  return { mass, firstMoment, transverseMoment };
}

export function measureMassProperties(mesh, controlPoint = [0, 0, 0]) {
  const components = [];
  let massKg = 0;
  const firstMoment = [0, 0, 0];
  let momentOfInertiaKgM2 = 0;
  for (const part of mesh.parts ?? []) {
    const density = part.material?.density;
    if (!(density > 0)) continue;
    const integrals = tetrahedralMassIntegrals(part, density, controlPoint);
    if (!(integrals.mass > 0)) continue;
    massKg += integrals.mass;
    momentOfInertiaKgM2 += integrals.transverseMoment;
    for (let axis = 0; axis < 3; axis += 1) firstMoment[axis] += integrals.firstMoment[axis];
    components.push({ id: part.componentId ?? part.label ?? "part", label: part.label ?? part.componentId ?? "part", massKg: integrals.mass, centerOfMass: integrals.firstMoment.map((value) => value / integrals.mass) });
  }
  const centerOfMass = firstMoment.map((value) => value / massKg);
  const gripToTipM = Math.max(0, mesh.stats.bounds.max[1] - controlPoint[1]);
  return {
    massKg,
    centerOfMass,
    centerOfMassFromGripM: centerOfMass[1] - controlPoint[1],
    momentOfInertiaKgM2,
    balance: gripToTipM > 0 ? Math.sqrt(momentOfInertiaKgM2 / massKg) / gripToTipM : 1,
    gripToTipM,
    components,
  };
}


export function closedManifoldErrors(mesh, label = "component") {
  const precision = 1e7,
    edges = new Map();
  const key = (offset) =>
    mesh.positions
      .slice(offset, offset + 3)
      .map((value) => Math.round(value * precision))
      .join(",");
  for (let index = 0; index < mesh.indices.length; index += 3) {
    const vertices = mesh.indices.slice(index, index + 3).map((vertex) => key(vertex * 3));
    for (const [from, to] of [
      [vertices[0], vertices[1]],
      [vertices[1], vertices[2]],
      [vertices[2], vertices[0]],
    ]) {
      const edge = from < to ? `${from}|${to}` : `${to}|${from}`,
        direction = from < to ? 1 : -1;
      const state = edges.get(edge) ?? [0, 0];
      state[direction > 0 ? 0 : 1] += 1;
      edges.set(edge, state);
    }
  }
  const invalid = [...edges.values()].filter(([forward, reverse]) => forward !== 1 || reverse !== 1).length;
  return invalid ? [`${label}: ${invalid} boundary/non-manifold edges`] : [];
}
