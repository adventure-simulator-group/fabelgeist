"""Finite tangent frames for the exported normal and the actual UV chart."""
import numpy as np


def undefined_vertices(mesh, normals):
    mesh.normals_split_custom_set_from_vertices(normals.tolist())
    mesh.calc_tangents(uvmap='armor_material')
    return {loop.vertex_index for loop in mesh.loops
            if not np.isfinite(loop.tangent[:]).all()
            or np.linalg.norm(loop.tangent[:]) < 1e-10
            or loop.bitangent_sign not in (-1, 1)}


def condition_carrier_normals(mesh, normals):
    """Choose a bake carrier for which consumers can reconstruct Mikk frames.

    Detailed source normals are left intact. Undefined or nearly face-tangent
    carrier corners move toward their geometric normal; the detail map encodes
    the inverse change. Well-conditioned corners retain their carrier exactly.
    """
    original = np.array(normals, dtype=np.float64)
    result = original.copy()
    positions = np.array([v.co[:] for v in mesh.vertices], dtype=np.float64)
    faces = np.array([p.vertices[:] for p in mesh.polygons])
    if faces.ndim != 2 or faces.shape[1] != 3:
        raise ValueError('carrier conditioning requires triangles')
    triangle = positions[faces]
    cross = np.cross(triangle[:, 1] - triangle[:, 0], triangle[:, 2] - triangle[:, 0])
    face_length = np.linalg.norm(cross, axis=1)
    if np.any(face_length < 1e-20):
        raise ValueError('carrier conditioning requires nondegenerate triangles')
    face_normal = cross / face_length[:, None]
    longest_edge = np.maximum.reduce([
        np.linalg.norm(triangle[:, 1] - triangle[:, 0], axis=1),
        np.linalg.norm(triangle[:, 2] - triangle[:, 0], axis=1),
        np.linalg.norm(triangle[:, 2] - triangle[:, 1], axis=1)])
    altitude = face_length / longest_edge
    position_scale = np.max(np.linalg.norm(triangle, axis=2), axis=1)
    # Thin faces amplify float32 position errors into face-normal errors.
    # Allow several roundings during coordinate/bind-frame conversion rather
    # than using the same angular margin for a broad panel and a tiny rim.
    margin = np.maximum(.001, 8 * np.finfo(np.float32).eps * position_scale / altitude)
    geometric = np.zeros_like(original)
    for corner in range(3):
        np.add.at(geometric, faces[:, corner], cross)
    length = np.linalg.norm(geometric, axis=1)
    geometric /= np.maximum(length[:, None], 1e-30)
    # Mikk grouping/custom-normal storage can differ after glTF's Y-up to
    # Blender Z-up conversion, even without a singular face projection. Check
    # that consumer coordinate frame as well as the authored one.
    import bpy
    rotation = np.array([[1, 0, 0], [0, 0, -1], [0, 1, 0]])
    consumer = mesh.copy()
    consumer.vertices.foreach_set('co', (positions @ rotation.T).ravel())
    consumer.update()
    try:
        for amount in (0, .003, .01, .03, .1, .3, 1):
            if amount:
                if np.any(length[bad] < 1e-20):
                    raise ValueError('undefined carrier has no geometric normal')
                candidate = original[bad] * (1 - amount) + geometric[bad] * amount
                candidate_length = np.linalg.norm(candidate, axis=1)
                if np.any(candidate_length < 1e-12):
                    raise ValueError('undefined carrier normal blend')
                result[bad] = candidate / candidate_length[:, None]
            bad = undefined_vertices(mesh, result) | undefined_vertices(consumer, result @ rotation.T)
            # Near-plane normals also need a precision margin even if neither
            # coordinate frame happens to produce an exact zero direction.
            near_plane = np.abs(np.einsum('fci,fi->fc', result[faces], face_normal)) < margin[:, None]
            bad = sorted(bad | set(faces[near_plane].tolist()))
            if not bad:
                return result
        raise ValueError('carrier still has undefined Mikk frames after geometric conditioning')
    finally:
        bpy.data.meshes.remove(consumer)


def corner_frame(mesh, loop, normals):
    normal = np.asarray(normals[loop.vertex_index], dtype=np.float64)
    length = np.linalg.norm(normal)
    if not np.isfinite(length) or length < 1e-12:
        raise ValueError('tangent frame requires a finite nonzero normal')
    normal = normal / length
    tangent = np.array(loop.tangent, dtype=np.float64)
    tangent -= normal * np.dot(normal, tangent)
    length = np.linalg.norm(tangent)
    if np.isfinite(length) and length > 1e-10 and loop.bitangent_sign in (-1, 1):
        return np.append(tangent / length, loop.bitangent_sign)

    # Mikk can return a zero direction on a thin triangle whose smooth normal
    # is nearly in its plane. Recover the direction from its UV Jacobian;
    # the exporter splits corners with different frames into separate vertices.
    polygon = mesh.polygons[loop.index // 3]
    if len(polygon.loop_indices) != 3:
        raise ValueError('armor tangent frames require triangulated geometry')
    points = np.array([mesh.vertices[v].co[:] for v in polygon.vertices], dtype=np.float64)
    uv = np.array([mesh.uv_layers.active.data[i].uv[:] for i in polygon.loop_indices], dtype=np.float64)
    edges, chart = points[1:] - points[0], uv[1:] - uv[0]
    determinant = np.linalg.det(chart)
    if not np.isfinite(determinant) or abs(determinant) < 1e-14:
        raise ValueError('UV chart has no tangent frame')
    du = (edges[0] * chart[1, 1] - edges[1] * chart[0, 1]) / determinant
    tangent = du - normal * np.dot(normal, du)
    length = np.linalg.norm(tangent)
    if not np.isfinite(length) or length < 1e-12:
        raise ValueError('UV derivative is parallel to its shading normal')
    tangent /= length
    # Preserve Mikk's UV orientation group even when a smooth corner normal
    # points behind the geometric face. Recomputing handedness per corner can
    # introduce opposite signs within one triangle and a singular interpolation.
    if loop.bitangent_sign not in (-1, 1):
        raise ValueError('UV chart has no Mikk orientation group')
    return np.append(tangent, loop.bitangent_sign)
