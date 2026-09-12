"""Consumer-frame regression: run with Blender --background --python FILE."""
from pathlib import Path
import sys
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
try:
    import bpy
except ImportError:
    bpy = None


@unittest.skipUnless(bpy, "requires Blender's MikkTSpace consumer")
class ArmorConsumerFrameTests(unittest.TestCase):
    def thin_corner(self, mirrored=False):
        positions = np.array([
            [.1048315242, 1.4291589260, .0866251960],
            [.1250507534, 1.4408665895, .0639389902],
            [.1245378107, 1.4407285452, .0646687299]])
        normals = np.array([
            [.0354962610, .7944580913, .6062808633],
            [.7491884828, .0187977795, .6620902419],
            [.2672469616, .7238332033, .6361166239]])
        uv = np.array([
            [.7854832411, .6822821498],
            [.7854832411, .6782211065],
            [.7855101228, .6783303618]])
        if mirrored:
            uv[:, 0] = 1 - uv[:, 0]
        mesh = self.consumer_mesh(positions, np.array([[0, 1, 2]]), normals, uv)
        return mesh, normals

    def test_undefined_mikk_direction_uses_uv_derivative_and_face_orientation(self):
        from armor_tangents import corner_frame
        from bake_armor import carrier_tangents
        signs = []
        for mirrored in (False, True):
            mesh, normals = self.thin_corner(mirrored)
            mesh.calc_tangents(uvmap="armor_material")
            np.testing.assert_array_equal(mesh.loops[0].tangent, [0, 0, 0])
            sources, indices, frames = carrier_tangents(mesh, normals)
            np.testing.assert_array_equal(sources, [0, 1, 2])
            np.testing.assert_array_equal(indices, [0, 1, 2])
            self.assertTrue(np.isfinite(frames).all())
            np.testing.assert_allclose(np.linalg.norm(frames[:, :3], axis=1), 1)
            np.testing.assert_allclose((frames[:, :3] * normals).sum(axis=1), 0, atol=1e-12)
            np.testing.assert_array_equal(frames[:, 3], [mesh.loops[0].bitangent_sign] * 3)
            points = np.array([v.co[:] for v in mesh.vertices], dtype=np.float64)
            chart = np.array([v.uv[:] for v in mesh.uv_layers.active.data])
            du = np.linalg.solve(chart[1:] - chart[0], points[1:] - points[0])[0]
            n = normals[0] / np.linalg.norm(normals[0])
            expected = du - n * np.dot(n, du)
            np.testing.assert_allclose(frames[0, :3], expected / np.linalg.norm(expected))
            signs.append(frames[0, 3])
            # Frame interpolation must remain invertible across the triangle.
            for a in np.linspace(0, 1, 21):
                for b in np.linspace(0, 1 - a, 21):
                    weights = np.array([a, b, 1 - a - b])
                    n, t = weights @ normals, weights @ frames[:, :3]
                    frame = np.column_stack((t, np.cross(n, t) * frames[0, 3], n))
                    self.assertGreater(abs(np.linalg.det(frame)), .2)
            for uv in mesh.uv_layers.active.data:
                uv.uv = (0, 0)
            mesh.calc_tangents(uvmap="armor_material")
            # The recovery path must reject truly collapsed charts.
            from types import SimpleNamespace
            loop = SimpleNamespace(index=0, vertex_index=0, tangent=(0, 0, 0), bitangent_sign=1)
            with self.assertRaisesRegex(ValueError, 'UV chart'):
                corner_frame(mesh, loop, normals)
        self.assertEqual(signs[0], -signs[1])

    def consumer_mesh(self, positions, faces, normals, gltf_uv):
        mesh = bpy.data.meshes.new("consumer")
        mesh.from_pydata(positions.tolist(), [], faces.tolist())
        mesh.update()
        for face in mesh.polygons:
            face.use_smooth = True
        mesh.normals_split_custom_set_from_vertices(normals.tolist())
        layer = mesh.uv_layers.new(name="armor_material")
        for loop in mesh.loops:
            u, v = gltf_uv[loop.vertex_index]
            layer.data[loop.index].uv = (float(u), float(1 - v))
        self.addCleanup(bpy.data.meshes.remove, mesh)
        return mesh

    def test_unwrap_frame_matches_gltf_consumers_v_up_chart(self):
        from unwrap_armor import unwrap
        positions = np.array([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
        normals = np.tile([0., 0., 1.], (3, 1))
        source, indices, uv, tangents = unwrap(positions, np.array([[0, 1, 2]]), normals)
        mesh = self.consumer_mesh(positions[source], indices.reshape(-1, 3), normals[source], uv)
        mesh.calc_tangents(uvmap="armor_material")
        for loop in mesh.loops:
            np.testing.assert_allclose(tangents[loop.vertex_index, :3], loop.tangent, atol=1e-5)
            self.assertEqual(tangents[loop.vertex_index, 3], loop.bitangent_sign)

    def test_unwrap_excludes_other_selected_scene_objects(self):
        from unwrap_armor import unwrap
        bpy.ops.mesh.primitive_cube_add(size=100)
        other = bpy.context.object
        data = other.data
        before = np.array([v.uv[:] for v in data.uv_layers.active.data])
        self.addCleanup(bpy.data.meshes.remove, data)
        self.addCleanup(bpy.data.objects.remove, other, do_unlink=True)
        positions = np.array([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
        _, indices, uv, _ = unwrap(positions, np.array([[0, 1, 2]]), np.tile([0., 0., 1.], (3, 1)))
        np.testing.assert_array_equal(before, [v.uv[:] for v in data.uv_layers.active.data])
        triangle = uv[indices]
        self.assertGreater(abs(np.linalg.det(triangle[1:] - triangle[0])) / 2, .1)

    def recovered_corner_pair(self):
        positions = np.array([
            [.4522579312324524, 1.2975836992263794, .04412107169628143],
            [.45056581497192383, 1.295180320739746, .043343737721443176],
            [.45162492990493774, 1.2961738109588623, .042851515114307404],
            [.4511793255805969, 1.2965786457061768, .04463537782430649]])
        normals = np.array([
            [.002098342192767192, .4647355696576504, .8854470324389941],
            [.009256851992057483, -.45411542991509846, -.8908947676376948],
            [.006650032714666864, -.4565802524489634, -.889657377948688],
            [-.0018154050659782376, .4610728214872187, .8873604440081053]])
        uv = np.array([
            [.6507641077041626, .018621712923049927],
            [.6500176787376404, .036013852804899216],
            [.6452363729476929, .028651338070631027],
            [.6557037234306335, .026074012741446495]])
        faces = np.array([[0, 1, 2], [0, 3, 1]])
        mesh = self.consumer_mesh(positions, faces, normals, uv)
        return mesh, positions, faces, normals, uv

    def test_carrier_splits_distinct_recovered_corner_frames(self):
        from bake_armor import carrier_tangents
        from armor_tangents import corner_frame
        mesh, _, faces, normals, _ = self.recovered_corner_pair()
        sources, indices, frames = carrier_tangents(mesh, normals)
        self.assertEqual(np.count_nonzero(sources == 0), 2)
        np.testing.assert_array_equal(sources[indices], faces.flatten())
        self.assertNotEqual(indices[0], indices[3])
        for loop, index in zip(mesh.loops, indices):
            np.testing.assert_allclose(frames[index], corner_frame(mesh, loop, normals), atol=1e-5)

    def test_conditioning_repairs_consumer_frame_and_preserves_source_detail(self):
        from armor_tangents import condition_carrier_normals
        from bake_armor import carrier_tangents
        from armor_bake_math import normal_atlas, raster_triangles, unit
        mesh, positions, faces, detailed, uv = self.recovered_corner_pair()
        before = detailed.copy()
        mesh.calc_tangents(uvmap='armor_material')
        carrier = condition_carrier_normals(mesh, detailed)
        np.testing.assert_array_equal(before, detailed)
        angle = np.degrees(np.arccos(np.clip(np.dot(unit(carrier[0]), unit(detailed[0])), -1, 1)))
        self.assertLess(angle, .2)
        consumer = self.consumer_mesh(positions, faces, carrier.astype(np.float32), uv)
        consumer.calc_tangents(uvmap='armor_material')
        self.assertTrue(all(np.linalg.norm(loop.tangent[:]) > .99 for loop in consumer.loops))
        sources, indices, tangents = carrier_tangents(mesh, carrier)
        chart_faces = indices.reshape(-1, 3)
        image, covered = normal_atlas(uv[sources], chart_faces, detailed[sources], carrier[sources], tangents, 1024)
        self.assertGreater(covered.sum(), 5)
        for face, y, x, weights in raster_triangles(uv[sources], chart_faces, 1024):
            n, t = weights @ carrier[sources][face], weights @ tangents[face, :3]
            sign = weights @ tangents[face, 3]
            frame = np.stack((t, np.cross(n, t) * sign[:, None], n), axis=2)
            reconstructed = unit(np.einsum('nij,nj->ni', frame, image[y, x, :3] * 2 - 1))
            np.testing.assert_allclose(reconstructed, unit(weights @ detailed[sources][face]), atol=2e-4)

    def test_near_plane_carrier_survives_coordinate_conversion(self):
        from armor_tangents import condition_carrier_normals
        # A real cuisse strap corner: direct Mikk validity alone missed its
        # singular frame after float32 glTF coordinate/normal conversion.
        positions = np.array([
            [.1701909453, .6040785313, -.0898384750],
            [.1689541191, .6038363576, -.0882348493],
            [.1700048298, .6040617824, -.0899352953],
            [0, 0, 0], [1, 0, 0], [0, 1, 0]])
        normals = np.array([
            [.4524065852, .1029823720, -.8858458996],
            [-.4497184455, -.1057411879, .8868890405],
            [.4507263899, .1058462635, -.8863646388],
            [0, 0, 1], [0, 0, 1], [0, 0, 1]])
        uv = np.array([
            [.1600768864, .0692070499], [.1600768864, .0858593956],
            [.1583878547, .0695232674], [0, 0], [1, 0], [0, 1]])
        faces = np.array([[0, 1, 2], [3, 4, 5]])
        mesh = self.consumer_mesh(positions, faces, normals, uv)
        carrier = condition_carrier_normals(mesh, normals)
        np.testing.assert_array_equal(carrier[3:], normals[3:])
        self.assertGreater(np.linalg.norm(carrier[1] - normals[1]), .001)
        for transform in (np.eye(3), np.array([[1, 0, 0], [0, 0, -1], [0, 1, 0]])):
            consumer = self.consumer_mesh(
                (positions @ transform.T).astype(np.float32), faces,
                (carrier @ transform.T).astype(np.float32), uv.astype(np.float32))
            consumer.calc_tangents(uvmap='armor_material')
            self.assertTrue(all(np.linalg.norm(loop.tangent[:]) > .99
                                for loop in consumer.loops))

    def test_carrier_frame_reconstructs_detail_in_consumer(self):
        from bake_armor import carrier_tangents
        from armor_bake_math import normal_atlas, unit
        positions = np.array([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
        faces = np.array([[0, 1, 2]])
        uv = np.array([[0., 0.], [1., 0.], [0., 1.]])
        normals = unit(np.array([[.6, 0., 1.], [0., .6, 1.], [-.6, 0., 1.]]))
        detail = unit(np.tile([.2, .3, 1.], (3, 1)))
        mesh = self.consumer_mesh(positions, faces, normals, uv)
        sources, indices, tangents = carrier_tangents(mesh, normals)
        np.testing.assert_array_equal(sources, [0, 1, 2])
        np.testing.assert_array_equal(indices, [0, 1, 2])
        image, _ = normal_atlas(uv, faces, detail, normals, tangents, 32)
        # Independently reconstruct the consumer's interpolated frame.
        mesh.calc_tangents(uvmap="armor_material")
        x = y = 7
        weights = np.array([1 - 2 * (x + .5) / 32, (x + .5) / 32, (y + .5) / 32])
        n = weights @ normals
        t = weights @ np.array([loop.tangent[:] for loop in mesh.loops])
        sign = weights @ np.array([loop.bitangent_sign for loop in mesh.loops])
        frame = np.column_stack((t, np.cross(n, t) * sign, n))
        recovered = unit(frame @ (image[y, x, :3] * 2 - 1))
        np.testing.assert_allclose(recovered, detail[0], atol=2e-4)

    def test_thin_rim_accounts_for_float32_position_precision(self):
        from armor_tangents import condition_carrier_normals
        # This breastplate rim passed the fixed .001 angular margin but lost
        # its Mikk direction after the glTF skin bind-frame conversion.
        positions = np.array([
            [-.1605157256, 1.4504468441, .1122542247],
            [-.1601265967, 1.4517698288, .1117103398],
            [-.1600773185, 1.4514880180, .1116886735]])
        normals = np.array([
            [-.6511785984, .4241354167, .6293453574],
            [-.6752268076, .4234197438, .6039739251],
            [-.7873482108, .3991996050, .4698111117]])
        uv = np.array([
            [.9952887297, .0048075872], [.9944522381, .0048075872],
            [.9945823550, .0047113001]])
        faces = np.array([[0, 1, 2]])
        # Recorded positions/normals after the real skin bind-frame import.
        # Preserve its observed rounding error for both candidate carriers.
        imported_positions = np.array([
            [-.1605157107114792, -.11225421726703644, 1.4504468441009521],
            [-.16012658178806305, -.1117103323340416, 1.4517698287963867],
            [-.160077303647995, -.11168866604566574, 1.4514880180358887]])
        imported_normals = np.array([
            [-.6511480808258057, -.6293152570724487, .42422711849212646],
            [-.6752084493637085, -.604023277759552, .42337870597839355],
            [-.7873531579971313, -.4697796404361725, .3992268443107605]])
        imported_uv = np.array([
            [.9952887296676636, 1 - .9951924085617065],
            [.9944522380828857, 1 - .9951924085617065],
            [.9945823550224304, 1 - .9952887296676636]])
        broken = self.consumer_mesh(imported_positions, faces, imported_normals, imported_uv)
        broken.calc_tangents(uvmap='armor_material')
        np.testing.assert_array_equal(broken.loops[0].tangent, [0, 0, 0])
        before = normals.copy()
        mesh = self.consumer_mesh(positions, faces, normals, uv)
        carrier = condition_carrier_normals(mesh, normals)
        np.testing.assert_array_equal(normals, before)
        transform = np.array([[1, 0, 0], [0, 0, -1], [0, 1, 0]])
        conversion_error = imported_normals - normals @ transform.T
        consumer = self.consumer_mesh(
            imported_positions, faces,
            (carrier @ transform.T + conversion_error).astype(np.float32), imported_uv)
        consumer.calc_tangents(uvmap='armor_material')
        self.assertTrue(all(np.linalg.norm(loop.tangent[:]) > .99
                            for loop in consumer.loops))

    def test_conditioning_checks_the_consumer_coordinate_frame(self):
        from armor_tangents import condition_carrier_normals
        positions = np.array([
            [.20703308284282684, 1.4067752361297607, -.14365287125110626],
            [.20023894309997559, 1.3998297452926636, -.14601552486419678],
            [.19146794080734253, 1.3910870552062988, -.1472492218017578]])
        normals = np.array([
            [.2356783151626587, .11855484545230865, -.9645726680755615],
            [.2288602739572525, .033936452120542526, -.9728675484657288],
            [.21568389236927032, .0003763499844353646, -.9764631390571594]])
        uv = np.array([
            [.2149868756532669, .003908842336386442],
            [.21595527231693268, .016372544690966606],
            [.21498695015907288, .03190239518880844]])
        faces = np.array([[0, 1, 2]])
        transform = np.array([[1, 0, 0], [0, 0, -1], [0, 1, 0]])
        mesh = self.consumer_mesh(positions, faces, normals, uv)
        mesh.calc_tangents(uvmap='armor_material')
        self.assertTrue(all(np.linalg.norm(loop.tangent[:]) > .99 for loop in mesh.loops))
        rotated = self.consumer_mesh(positions @ transform.T, faces, normals @ transform.T, uv)
        rotated.calc_tangents(uvmap='armor_material')
        np.testing.assert_array_equal(rotated.loops[0].tangent, [0, 0, 0])
        carrier = condition_carrier_normals(mesh, normals)
        fixed = self.consumer_mesh(positions @ transform.T, faces, carrier @ transform.T, uv)
        fixed.calc_tangents(uvmap='armor_material')
        self.assertTrue(all(np.linalg.norm(loop.tangent[:]) > .99 for loop in fixed.loops))


if __name__ == "__main__":
    unittest.main(argv=[__file__])
