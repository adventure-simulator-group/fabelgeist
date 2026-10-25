"""Reference shading frames and tangent-space detail transfer for armor."""
import numpy as np


def unit(values):
    return values / np.maximum(np.linalg.norm(values, axis=-1, keepdims=True), 1e-20)


def carrier_normals(positions, faces, fields, iterations=16):
    """Low-frequency shading normals; geometry and morph positions stay intact.

    UV splits share a shading sample only when both position and original normal
    agree. Never join the opposite faces of a plate, nor smear a hard rim.
    The same adjacency is used for every identity endpoint.
    """
    key = np.round(np.column_stack((positions, fields[:, 0])), 5)
    _, representatives, inverse = np.unique(key, axis=0, return_index=True, return_inverse=True)
    triangles = inverse[faces]
    edges = np.concatenate([triangles[:, [0, 1]], triangles[:, [1, 2]], triangles[:, [2, 0]]])
    edges = np.unique(np.sort(edges, axis=1), axis=0)
    base = fields[representatives, 0]
    edges = edges[(base[edges[:, 0]] * base[edges[:, 1]]).sum(axis=1) > np.cos(np.deg2rad(55))]
    edges = np.concatenate((edges, edges[:, ::-1]))
    values = fields[representatives].copy()
    count = np.bincount(edges[:, 0], minlength=len(values))[:, None, None]
    for _ in range(iterations):
        total = np.zeros_like(values)
        np.add.at(total, edges[:, 0], values[edges[:, 1]])
        values = unit((values + total) / (count + 1))
    return values[inverse]


def frame_tangents(normals, original):
    direction = original[:, :3] - normals * (normals * original[:, :3]).sum(axis=1)[:, None]
    return np.column_stack((unit(direction), original[:, 3]))


def raster_triangles(uv, faces, resolution):
    """Yield covered texels and barycentrics in glTF image coordinates (Y down)."""
    for face in faces:
        triangle = uv[face] * resolution - .5
        lower = np.maximum(np.ceil(triangle.min(axis=0)).astype(int), 0)
        upper = np.minimum(np.floor(triangle.max(axis=0)).astype(int), resolution - 1)
        if np.any(upper < lower):
            continue
        x, y = np.meshgrid(np.arange(lower[0], upper[0] + 1), np.arange(lower[1], upper[1] + 1))
        point = np.column_stack((x.ravel(), y.ravel())) - triangle[0]
        a, b = triangle[1] - triangle[0], triangle[2] - triangle[0]
        denominator = a[0] * b[1] - a[1] * b[0]
        if abs(denominator) < 1e-15:
            continue
        v = (point[:, 0] * b[1] - point[:, 1] * b[0]) / denominator
        w = (a[0] * point[:, 1] - a[1] * point[:, 0]) / denominator
        weights = np.column_stack((1 - v - w, v, w))
        inside = (weights >= -1e-7).all(axis=1)
        yield face, y.ravel()[inside], x.ravel()[inside], weights[inside]


def normal_atlas(uv, faces, detailed, carrier, tangents, resolution):
    image = np.zeros((resolution, resolution, 4), dtype=np.float32)
    image[:, :, :3] = [.5, .5, 1]
    image[:, :, 3] = 1
    covered = np.zeros((resolution, resolution), dtype=bool)
    for face, y, x, weights in raster_triangles(uv, faces, resolution):
        high, normal = unit(weights @ detailed[face]), weights @ carrier[face]
        tangent = weights @ tangents[face, :3]
        sign = weights @ tangents[face, 3]
        bitangent = np.cross(normal, tangent) * sign[:, None]
        # Invert the exact interpolated MikkTSpace frame used by Bevy's
        # calculate_tbn_mikktspace. Its columns are deliberately NOT normalized
        # or orthogonalized after interpolation; a transpose is not its inverse.
        frame = np.stack((tangent, bitangent, normal), axis=2)
        detail = np.linalg.solve(frame, high[:, :, None])[:, :, 0]
        image[y, x, :3] = unit(detail) * .5 + .5
        covered[y, x] = True
    return image, covered


def dilate(image, covered, pixels=2):
    """Small gutters inside the atlas's authored inter-island padding."""
    for _ in range(pixels):
        old = covered.copy()
        for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
            valid = np.roll(old, (dy, dx), axis=(0, 1)) & ~covered
            if dy > 0:
                valid[:dy] = False
            elif dy < 0:
                valid[dy:] = False
            if dx > 0:
                valid[:, :dx] = False
            elif dx < 0:
                valid[:, dx:] = False
            image[valid] = np.roll(image, (dy, dx), axis=(0, 1))[valid]
            covered[valid] = True
    return image
