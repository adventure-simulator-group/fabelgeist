//! The device builder and bake, on this machine's compute device.

mod armor;
mod metal;

use std::sync::OnceLock;

use super::PlateGpu;
use crate::{Armor, Construction, Fauld};

fn gpu() -> &'static PlateGpu {
    static GPU: OnceLock<PlateGpu> = OnceLock::new();
    GPU.get_or_init(|| PlateGpu::open().expect("a compute device"))
}

/// Small plates start from their construction's usual shape, as the editor
/// sets them.
fn plates(a: &mut Armor, construction: Construction) {
    match construction {
        Construction::Scale => {
            a.plate.roundness = 1.0;
            a.plate.stagger = 0.5;
            a.plate.hole_pairs = 1;
        }
        Construction::Lamellar => {
            a.plate.roundness = 0.25;
            a.plate.stagger = 0.0;
            a.plate.hole_pairs = 3;
        }
        Construction::Solid => {}
    }
}

/// Designs covering every construction of breastplate and fauld, and the
/// extremes of the supported shape.
fn designs() -> Vec<(&'static str, Armor)> {
    let mut designs = vec![("default", Armor::default())];
    for (name, construction, fauld) in [
        ("lamellar", Construction::Lamellar, Construction::Solid),
        ("scale", Construction::Scale, Construction::Solid),
        (
            "solid, lamellar fauld",
            Construction::Solid,
            Construction::Lamellar,
        ),
        (
            "solid, scale fauld",
            Construction::Solid,
            Construction::Scale,
        ),
        (
            "scale, lamellar fauld",
            Construction::Scale,
            Construction::Lamellar,
        ),
    ] {
        let mut a = Armor {
            construction,
            ..Armor::default()
        };
        a.fauld.construction = fauld;
        plates(&mut a, construction);
        plates(&mut a, fauld);
        designs.push((name, a));
    }
    let mut a = Armor {
        construction: Construction::Lamellar,
        ..Armor::default()
    };
    a.plate.hole_radius = 0.0;
    a.plate.bevel = 0.0;
    a.fauld.layer_count = 0;
    designs.push(("lamellar without holes or bevel", a));
    designs.push(("reshaped scale, five fauld layers", reshaped_scale()));
    let mut a = Armor::default();
    a.fauld.layer_count = 12;
    a.fauld.flare = 0.05;
    a.ridge = 0.0;
    a.neck = 0.0;
    a.arm_cut = 0.0;
    a.center_point = 0.0;
    designs.push(("solid, twelve fauld layers, no openings", a));
    designs
}

/// Small, thick, heavily overlapping scales on a deep, moved breastplate.
fn reshaped_scale() -> Armor {
    let mut a = Armor {
        construction: Construction::Scale,
        width: 0.55,
        height: 0.5,
        depth: 0.2,
        waist: 1.1,
        neck: 0.11,
        arm_cut: 0.08,
        thickness: 0.006,
        ridge: 0.1,
        ridge_sharpness: 5.0,
        center_point: 0.07,
        translation: [0.3, 1.2, -0.1],
        ..Armor::default()
    };
    a.plate.width = 0.03;
    a.plate.height = 0.04;
    a.plate.gap = 0.003;
    a.plate.bevel = 0.002;
    a.plate.roundness = 0.6;
    a.plate.overlap = 0.4;
    a.plate.stagger = 0.3;
    a.plate.hole_radius = 0.0015;
    a.plate.hole_pairs = 2;
    a.fauld = Fauld {
        construction: Construction::Lamellar,
        layer_count: 5,
        layer_height: 0.05,
        overlap: 0.1,
        flare: 0.04,
    };
    a
}
