use super::*;

#[test]
fn pigment_tracks_growth_and_knot_flow_without_tracking_relief_amplitude() {
    let params = TextureParameters::default();
    let grain = Parameters {
        knot_fraction: 1.0,
        ..Default::default()
    };
    let flat = Parameters {
        ring_depth: 0.0,
        fiber_depth: 0.0,
        ..grain.clone()
    };
    let straight = Parameters {
        knot_flow: 0.0,
        ..grain.clone()
    };
    let mut bent = 0;
    let mut coverage = 0;
    for y in 0..32 {
        for x in 0..64 {
            let uv = Vec2::new(x as f32 / 64.0, y as f32 / 32.0);
            let footprint = Vec2::new(1.0 / 64.0, 1.0 / 32.0);
            let a = grain.filtered(&params, uv, footprint, 17);
            let b = flat.filtered(&params, uv, footprint, 17);
            assert_eq!(a.dark, b.dark);
            assert_eq!(b.height, 0.0);
            coverage += usize::from(a.dark > 0.0 && a.dark < 1.0);
            bent += usize::from(
                (a.height - straight.filtered(&params, uv, footprint, 17).height).abs() > 0.001,
            );
            // Unit-local cut grain may differ at its ends; the owning board
            // layout supplies the tile period and hides the cut under its joint.
        }
    }
    assert!(bent > 10, "knot flow must bend the actual relief");
    assert!(
        coverage > 10,
        "pigment edges must receive footprint antialiasing"
    );
}
