mod common;

use adventuresim_armor_model::{
    BuiltPart, GreaveDesign, LimbArmorDesign, Millimeters, PartFrame, PlateFluting, PlateGauge,
    gpu::generate_limb_armor_on,
};
use common::{assert_closed_solid, frame, gpu, largest_difference, mean_gauge, reflected};

fn shin() -> PartFrame {
    frame([0.1, 0.45, 0.02], [0.065, 0.18, 0.06])
}

fn greave(thickness: u16, fluting: Option<PlateFluting>, fit: &PartFrame) -> BuiltPart {
    let design = LimbArmorDesign::Greave(GreaveDesign {
        fluting,
        gauge: PlateGauge {
            thickness: Millimeters(thickness),
            ..Default::default()
        },
        ..Default::default()
    });
    generate_limb_armor_on(gpu(), &design, fit).unwrap()
}

fn flutes(depth: u16) -> Option<PlateFluting> {
    Some(PlateFluting {
        depth: Millimeters(depth),
        ..Default::default()
    })
}

#[test]
fn plate_gauge_is_the_design_wall_thickness() {
    for thickness in [1, 2, 4] {
        let plate = greave(thickness, None, &shin());
        let expected = f64::from(thickness) / 1000.0;
        let gauge = mean_gauge(&plate);
        assert!(
            (gauge - expected).abs() < expected * 0.05,
            "{thickness} mm plate is {gauge} m thick"
        );
    }
}

#[test]
fn flute_depth_moves_the_surface_without_changing_topology_or_gauge() {
    let fit = shin();
    let shallow = greave(2, flutes(1), &fit);
    let deep = greave(2, flutes(4), &fit);
    assert_eq!(shallow.indices, deep.indices);
    assert_closed_solid(&deep, "deeply fluted greave");
    // Three more millimetres of relief move the crests by about that much.
    let moved = largest_difference(&shallow.positions, &deep.positions);
    assert!((0.002..0.004).contains(&moved), "crests moved {moved} m");
    // Relief is carried by both walls: the plate stays one gauge thick.
    for plate in [&shallow, &deep] {
        let gauge = mean_gauge(plate);
        assert!((gauge - 0.002).abs() < 0.0002, "fluted plate is {gauge} m");
    }
}

#[test]
fn fluted_relief_follows_reflected_and_resized_frames() {
    let fit = shin();
    let reference = greave(2, flutes(3), &fit);
    for placed in [
        reflected(fit),
        PartFrame {
            half_extents: fit.half_extents.map(|v| v * 1.3),
            ..fit
        },
    ] {
        let plate = greave(2, flutes(3), &placed);
        // A reflection rewinds the triangles but keeps the plate's layout.
        assert_eq!(plate.positions.len(), reference.positions.len());
        assert_eq!(plate.indices.len(), reference.indices.len());
        assert_closed_solid(&plate, &format!("{placed:?}"));
        let gauge = mean_gauge(&plate);
        assert!((gauge - 0.002).abs() < 0.0002, "fluted plate is {gauge} m");
    }
}
