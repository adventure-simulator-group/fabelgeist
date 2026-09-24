import unittest

from scripts.check_deterministic_rng import scan_text


class DeterministicRngCheckTests(unittest.TestCase):
    def assert_forbidden(self, source: str, label: str) -> None:
        self.assertIn(label, [finding for _, finding in scan_text(source)])

    def test_rejects_every_uniform_construction_outside_the_shared_crate(self) -> None:
        for source in [
            "let draw = Uniform::new(0_usize, count);",
            "let draw = Uniform::<usize>::new(0, count);",
            "let draw = rand::distr::Uniform::new(0usize, count);",
            "use rand::distr::Uniform;",
        ]:
            with self.subTest(source=source):
                expected = (
                    "Rand Uniform outside shared crate"
                    if "rand::distr::Uniform" in source
                    else "Uniform construction outside shared crate"
                )
                self.assert_forbidden(source, expected)

    def test_ignores_prose_in_comments_and_strings(self) -> None:
        source = r'''
// Do not call rng.random_range(...) or Uniform::<usize>::new(...).
/* A nested comment may say /* fn mix64 */ Uniform::new. */
const NOTE: &str = "Uniform::<usize> and random_range are forbidden";
const RAW: &str = r#"fn splitmix64 and rand::seq::SliceRandom"#;
'''
        self.assertEqual(scan_text(source), [])

    def test_keeps_identifier_boundaries(self) -> None:
        self.assertEqual(scan_text("struct Uniformity; fn remix64() {}"), [])

    def test_rejects_sampling_helpers_in_code(self) -> None:
        self.assert_forbidden("rng.random_range(0..4);", "inferred-width random range")
        self.assert_forbidden("use rand::seq::SliceRandom;", "Rand slice selection helper")

    def test_rejects_direct_hash_seed_derivation(self) -> None:
        self.assert_forbidden(
            """
fn settlement_seed(value: &str) -> u64 {
    let digest = Sha256::digest(value.as_bytes());
    u64::from_le_bytes(digest[..8].try_into().unwrap())
}
""",
            "direct hash seed derivation",
        )

    def test_allows_hash_derived_non_rng_identifiers(self) -> None:
        self.assertEqual(
            scan_text(
                """
fn adapter_id(value: &str) -> u64 {
    let digest = Sha256::digest(value.as_bytes());
    u64::from_le_bytes(digest[..8].try_into().unwrap())
}
"""
            ),
            [],
        )


if __name__ == "__main__":
    unittest.main()
