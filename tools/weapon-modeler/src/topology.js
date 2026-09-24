export function* triangleVertices(mesh) {
  for (let offset = 0; offset < mesh.indices.length; offset += 3) {
    yield mesh.indices.slice(offset, offset + 3).map((index) => mesh.positions.slice(index * 3, index * 3 + 3));
  }
}
