import copy
import importlib.util
import pathlib
import struct
import unittest


PATH = pathlib.Path(__file__).parents[1] / "prepare_rig_base.py"
SPEC = importlib.util.spec_from_file_location("prepare_rig_base", PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class PrepareRigBaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source, cls.binary = MODULE.read_glb(pathlib.Path("assets/animations/biped/unarmed/base.glb"))

    def test_runtime_contract_is_mhr(self):
        document = MODULE.validate_and_prepare(copy.deepcopy(self.source), self.binary)
        roots = document["scenes"][document["scene"]]["nodes"]
        root_names = {document["nodes"][index]["name"] for index in roots}
        self.assertIn("MHR base body", root_names)
        self.assertIn("Skeleton", root_names)
        joint_names = {document["nodes"][i]["name"] for i in document["skins"][0]["joints"]}
        self.assertEqual(len(joint_names), 130)
        self.assertIn("body_world", joint_names)
        self.assertIn("l_wrist", joint_names)
        self.assertIn("r_wrist", joint_names)
        self.assertIn("l_weapon", joint_names)
        self.assertIn("r_weapon", joint_names)
        self.assertIn("c_camera", joint_names)
        self.assertEqual(document.get("animations", []), [])

    def test_duplicate_joint_is_rejected(self):
        document = copy.deepcopy(self.source)
        document["skins"][0]["joints"][1] = document["skins"][0]["joints"][0]
        with self.assertRaises(MODULE.GlbError):
            MODULE.validate_and_prepare(document, self.binary)

    def test_reparented_runtime_joint_is_rejected(self):
        document = copy.deepcopy(self.source)
        indices = {node.get("name"): index for index, node in enumerate(document["nodes"])}
        foot = indices["l_foot"]
        document["nodes"][indices["l_lowleg"]]["children"].remove(foot)
        document["nodes"][indices["l_upleg"]].setdefault("children", []).append(foot)
        with self.assertRaisesRegex(MODULE.GlbError, "expected l_foot parent l_lowleg"):
            MODULE.validate_and_prepare(document, self.binary)

    def test_malformed_json_shapes_are_glb_errors(self):
        for document in [
            [],
            {**copy.deepcopy(self.source), "nodes": [[]]},
            {**copy.deepcopy(self.source), "skins": [None]},
            {**copy.deepcopy(self.source), "scenes": [None]},
            {**copy.deepcopy(self.source), "extras": []},
            {**copy.deepcopy(self.source), "skins": [{"joints": [[]] * 130}]},
        ]:
            with self.assertRaises(MODULE.GlbError):
                MODULE.validate_and_prepare(document, self.binary)

    def test_encoding_is_deterministic(self):
        first = MODULE.encode_glb(MODULE.validate_and_prepare(copy.deepcopy(self.source), self.binary), self.binary)
        second = MODULE.encode_glb(MODULE.validate_and_prepare(copy.deepcopy(self.source), self.binary), self.binary)
        self.assertEqual(first, second)


class RuntimeSkinContractTests(unittest.TestCase):
    def validate(self, weights, extra=None):
        document = {
            "bufferViews": [{"byteOffset": 0, "byteLength": 16},
                            {"byteOffset": 16, "byteLength": 8}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC4", "count": 1},
                          {"bufferView": 1, "componentType": 5123, "type": "VEC4", "count": 1}],
        }
        attributes = {"WEIGHTS_0": 0, "JOINTS_0": 1, **(extra or {})}
        binary = struct.pack("<4f4H", *weights, 0, 1, 2, 3)
        MODULE.validate_skin_weights(document, binary, attributes, 1, 4)

    def test_complete_primary_weights_preserve_world_translation(self):
        weights = [0.4, 0.3, 0.2, 0.1]
        self.validate(weights)
        points = [(1, 2, 3), (4, 1, 6), (-1, 2, 8), (7, 0, 1)]
        translation = (100, -50, 30)
        for axis, movement in enumerate(translation):
            before = sum(w * point[axis] for w, point in zip(weights, points))
            after = sum(w * (point[axis] + movement) for w, point in zip(weights, points))
            self.assertAlmostEqual(after - before, movement)

    def test_missing_mass_negative_and_nonfinite_primary_weights_are_rejected(self):
        for weights in [(0.4, 0.3, 0.2, 0), (-0.1, 0.5, 0.3, 0.3),
                        (float("nan"), 0, 0, 1), (float("inf"), 0, 0, 0)]:
            with self.subTest(weights=weights), self.assertRaisesRegex(MODULE.GlbError, "sum to one"):
                self.validate(weights)

    def test_unused_secondary_sets_are_not_a_runtime_contract(self):
        for name in ["JOINTS_1", "WEIGHTS_1", "JOINTS_2"]:
            with self.subTest(name=name), self.assertRaisesRegex(MODULE.GlbError, "exactly four"):
                self.validate([1, 0, 0, 0], {name: 0})


if __name__ == "__main__":
    unittest.main()
