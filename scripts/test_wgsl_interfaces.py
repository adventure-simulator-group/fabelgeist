"""Behavioral tests for nominal shader signatures and retained assembly debt."""

from pathlib import Path
import unittest

from scripts.semantic_interfaces.model import InterfaceRule, SourceUnit
from scripts.semantic_interfaces.repository import SourceUnits, census
from scripts.semantic_interfaces.wgsl import ShaderInputs, ShaderScanner
from scripts.semantic_interfaces.wgsl_tokens import (
    ShaderLine, ShaderLineAdmissionFailure,
)


ROOT = Path(__file__).resolve().parents[1]


class ShaderInterfacesTests(unittest.TestCase):
    def test_source_lines_reject_nonpositive_sdk_coordinates(self) -> None:
        with self.assertRaises(ShaderLineAdmissionFailure):
            ShaderLine(0)
        with self.assertRaises(ShaderLineAdmissionFailure):
            ShaderLine(-1)

    def test_unsupported_type_arguments_cannot_claim_nominal_resolution(self) -> None:
        source = SourceUnit(Path("arguments.wgsl"), """
            struct Count { value: u32, }
            fn invalid(a: ptr<unknown, Count>, b: ptr<function, Count, unknown>,
                       c: array<Count, 2, 3>, d: vec2<Count>, e: atomic<Count>) { }
        """)
        found = ShaderScanner(ShaderInputs([source])).inspect()
        self.assertEqual(len(found.entries), 5)
        self.assertTrue(all(row.leaf.rule is InterfaceRule.UNRESOLVED
                            for row in found.entries))

    def test_function_name_references_are_data_without_a_prototype(self) -> None:
        source = SourceUnit(Path("references.rs"), '''
            const LABEL: &str = "pub(crate) fn consume_stack";
            const STRUCT_REFERENCE: &str = "pub struct ContractAuthority {";
        ''')
        found = census(SourceUnits([source]), ROOT)
        self.assertEqual(found.entries, [])

    def test_nested_aliases_vectors_arrays_and_pointers_retain_scalar_debt(self) -> None:
        source = SourceUnit(Path("types.wgsl"), """
            alias Word = u32;
            alias Blend = array<vec2<f32>, 4>;
            fn shade(a: Word, b: Blend, c: ptr<function, array<f32>, read>)
                -> mat4x4<f32> { }
        """)
        found = ShaderScanner(ShaderInputs([source])).inspect()
        raw = [row for row in found.entries if row.leaf.rule is InterfaceRule.RAW]
        self.assertEqual([(row.slot, row.leaf.spelling) for row in raw], [
            ("a", "u32"), ("b", "f32"), ("c", "f32"), ("return", "f32"),
        ])
        self.assertEqual(sum(row.slot == "alias" for row in found.entries), 2)

    def test_distinct_declared_structs_are_nominal_but_aliases_stay_visible(self) -> None:
        source = SourceUnit(Path("nominal.wgsl"), """
            struct Position { value: vec3<f32>, }
            struct Direction { value: vec3<f32>, }
            alias PositionAlias = Position;
            fn move(position: Position, direction: Direction) -> Position { }
            fn same(position: PositionAlias) -> PositionAlias { }
        """)
        found = ShaderScanner(ShaderInputs([source])).inspect()
        self.assertEqual(len(found.entries), 1)
        self.assertEqual(found.entries[0].slot, "alias")
        self.assertEqual(found.entries[0].leaf.rule, InterfaceRule.UNRESOLVED)

    def test_unknown_cycles_and_conflicting_declarations_are_unresolved(self) -> None:
        source = SourceUnit(Path("unknown.wgsl"), """
            alias A = B; alias B = A;
            alias Collision = u32; alias Collision = f32;
            struct Duplicated { value: u32, }
            struct Duplicated { value: f32, }
            fn pick(a: A, b: Collision, c: Duplicated, d: External) { }
        """)
        found = ShaderScanner(ShaderInputs([source])).inspect()
        parameters = [row for row in found.entries if row.slot != "alias"]
        self.assertEqual(len(parameters), 4)
        self.assertTrue(all(row.leaf.rule is InterfaceRule.UNRESOLVED
                            for row in parameters))
        self.assertTrue(all(row.opaque is not None for row in parameters))

    def test_entry_point_and_result_attributes_do_not_hide_builtin_scalars(self) -> None:
        source = SourceUnit(Path("attributes.wgsl"), """
            @vertex fn main(@builtin(vertex_index) vertex: u32)
                -> @builtin(position) vec4f { }
        """)
        found = ShaderScanner(ShaderInputs([source])).inspect()
        self.assertEqual([(row.slot, row.leaf.spelling) for row in found.entries],
                         [("vertex", "u32"), ("return", "vec4f")])

    def test_nested_comments_and_line_comment_tokens_preserve_real_headers(self) -> None:
        source = SourceUnit(Path("comments.wgsl"), """
            /* fn hidden(x: bool) -> bool {} // text
               /* nested */ */
            // fn also_hidden(x: bool) {}
            fn actual(x: f32) -> f32 { }
        """)
        found = ShaderScanner(ShaderInputs([source])).inspect()
        self.assertEqual(len(found.entries), 2)
        self.assertTrue(all(row.item.parts == ("actual",) for row in found.entries))

    def test_incomplete_and_templated_headers_do_not_claim_nominal_resolution(self) -> None:
        sources = ShaderInputs([
            SourceUnit(Path("template.wgsl"),
                       "fn {name}(x: {Input}) -> {Output} {{ return x; }}"),
            SourceUnit(Path("incomplete.wgsl"), "fn unfinished(x: f32"),
        ])
        found = ShaderScanner(sources).inspect()
        self.assertTrue(any(row.path == Path("template.wgsl")
                            and row.slot == "signature" for row in found.entries))
        self.assertTrue(any(row.path == Path("incomplete.wgsl")
                            and row.slot == "signature" for row in found.entries))
        self.assertTrue(any(row.leaf.spelling.startswith("unknown-type:")
                            for row in found.entries))

    def test_embedded_declarations_share_only_one_host_and_conflicts_stay_unknown(self) -> None:
        sources = ShaderInputs([
            SourceUnit(Path("one.rs#wgsl-1.wgsl"), "struct Count { value: u32, }"),
            SourceUnit(Path("one.rs#wgsl-2.wgsl"), "fn first(n: Count) {}"),
            SourceUnit(Path("two.rs#wgsl-1.wgsl"), "fn second(n: Count) {}"),
        ])
        found = ShaderScanner(sources).inspect()
        self.assertEqual(len(found.entries), 1)
        self.assertEqual(found.entries[0].item.parts, ("second",))

    def test_nominal_header_does_not_remove_opaque_assembly_coverage(self) -> None:
        source = SourceUnit(Path("nominal.rs"), '''
            const TYPES: &str = "struct Count { value: u32, }";
            const FN: &str = "fn choose(n: Count) -> Count { return n; }";
        ''')
        found = census(SourceUnits([source]), ROOT)
        self.assertEqual([row.leaf.rule for row in found.entries],
                         [InterfaceRule.OPAQUE])

    def test_standalone_wgsl_retains_host_fingerprint_and_scalar_signatures(self) -> None:
        source = SourceUnit(Path("standalone.wgsl"),
                            "fn helper(x: f32) -> f32 { return x; }")
        found = census(SourceUnits([source]), ROOT)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.OPAQUE
                             for row in found.entries), 1)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.RAW
                             for row in found.entries), 2)

    def test_unrelated_rust_literals_do_not_renumber_shader_fragments(self) -> None:
        shader = 'const CODE: &str = "fn shade(x: f32) -> f32 { return x; }";'
        original = census(SourceUnits([SourceUnit(Path("source.rs"), shader)]), ROOT)
        edited = census(SourceUnits([SourceUnit(Path("source.rs"),
            'const LABEL: &str = "label";\n' + shader)]), ROOT)
        old = {row.as_wire_key() for row in original.entries
               if row.leaf.rule is not InterfaceRule.OPAQUE}
        new = {row.as_wire_key() for row in edited.entries
               if row.leaf.rule is not InterfaceRule.OPAQUE}
        self.assertEqual(old, new)


if __name__ == "__main__":
    unittest.main()
