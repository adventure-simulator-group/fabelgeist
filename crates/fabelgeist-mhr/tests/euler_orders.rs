#[path = "support/euler_rig.rs"]
mod euler_rig;

use fabelgeist_mhr::character::Character;
use fabelgeist_mhr::math::{EulerOrder, quat_from_euler, quat_from_euler_degrees};

const ORDERS: [EulerOrder; 6] = [
    EulerOrder::Xyz,
    EulerOrder::Xzy,
    EulerOrder::Yzx,
    EulerOrder::Yxz,
    EulerOrder::Zxy,
    EulerOrder::Zyx,
];

fn bits<const N: usize>(field: &str) -> [u64; N] {
    field
        .split(',')
        .map(|value| u64::from_str_radix(value, 16).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

#[test]
fn quaternion_helpers_preserve_native_float_bits() {
    // Frozen from the unchanged public API, including signed zeros, subnormal
    // inputs and large finite angles. These are observations, not a second
    // implementation of quaternion composition.
    for line in include_str!("fixtures/euler-bits.txt").lines() {
        let fields: Vec<_> = line.split('|').collect();
        let order = ORDERS[fields[0].parse::<usize>().unwrap()];
        let angles = bits::<3>(fields[1]).map(f64::from_bits);
        assert_eq!(
            quat_from_euler(angles, order).map(f64::to_bits),
            bits::<4>(fields[2]),
            "radians: {line}",
        );
        assert_eq!(
            quat_from_euler_degrees(angles, order).map(f64::to_bits),
            bits::<4>(fields[3]),
            "degrees: {line}",
        );
    }
}

struct NativeCase {
    code: Option<i64>,
    effective_order: EulerOrder,
    prerotation_bits: [u32; 4],
}

fn native_cases() -> Vec<NativeCase> {
    let supported = [
        NativeCase {
            code: Some(0),
            effective_order: EulerOrder::Xyz,
            prerotation_bits: [0x3df95fdd, 0xba2ef032, 0x3ea34d54, 0x3f709e69],
        },
        NativeCase {
            code: Some(1),
            effective_order: EulerOrder::Xzy,
            prerotation_bits: [0x3d069f22, 0xbbb73bf2, 0x3ea969eb, 0x3f716dcb],
        },
        NativeCase {
            code: Some(2),
            effective_order: EulerOrder::Yzx,
            prerotation_bits: [0x3df7c215, 0xbd3cb655, 0x3e95aba8, 0x3f729077],
        },
        NativeCase {
            code: Some(3),
            effective_order: EulerOrder::Yxz,
            prerotation_bits: [0x3df0eb28, 0xba987498, 0x3e94c5da, 0x3f731894],
        },
        NativeCase {
            code: Some(4),
            effective_order: EulerOrder::Zxy,
            prerotation_bits: [0x3d144cfc, 0xbd4eda2e, 0x3eaa4fb9, 0x3f70e5ae],
        },
        NativeCase {
            code: Some(5),
            effective_order: EulerOrder::Zyx,
            prerotation_bits: [0x3d036391, 0xbd50e212, 0x3e9bc840, 0x3f735fd9],
        },
    ];
    let fallback = [
        None,
        Some(i64::MIN),
        Some(-7),
        Some(-1),
        Some(6),
        Some(7),
        Some(42),
        Some(i64::MAX),
    ];
    supported
        .into_iter()
        .chain(fallback.map(|code| NativeCase {
            code,
            effective_order: EulerOrder::Xyz,
            prerotation_bits: [0x3df95fdd, 0xba2ef032, 0x3ea34d54, 0x3f709e69],
        }))
        .collect()
}

#[test]
fn native_fbx_codes_preserve_effective_composition() {
    for case in native_cases() {
        if let Some(code) = case.code {
            let order = EulerOrder::from_fbx_code(code);
            assert_eq!(
                quat_from_euler([0.3, -0.7, 1.1], order),
                quat_from_euler([0.3, -0.7, 1.1], case.effective_order),
                "native code {code}",
            );
        }
    }
}

#[test]
fn public_character_composes_local_and_pre_rotations() {
    for case in native_cases() {
        let character = Character::from_fbx_bytes(&euler_rig::rig(case.code), false).unwrap();
        assert_eq!(
            character.skeleton.prerotations[0].map(f32::to_bits),
            case.prerotation_bits,
            "native code {:?}",
            case.code
        );
        assert_eq!(character.skeleton.names, ["item1"]);
        assert_eq!(character.skeleton.parents, [-1]);
        assert_eq!(character.skeleton.translation_offsets, [[1.5, -2.0, 0.25]]);
        assert_eq!(character.mesh.faces, [[0, 1, 2]]);
        assert_eq!(
            character.skin_weights.weight,
            [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3]
        );
    }
}
