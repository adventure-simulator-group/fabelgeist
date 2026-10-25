//! Dished guard plates with rounded matched apertures and rolled rims.
use super::*;
fn rounded(
    p: &GuardAssemblyParameters,
    names: &[String],
    detail: Detail,
) -> Result<Vec<Point>, String> {
    let controls: Vec<_> = names
        .iter()
        .map(|name| {
            p.nodes
                .get(name)
                .map(|v| v.map(Metres::get))
                .ok_or("missing plate node".to_string())
        })
        .collect::<Result<_, _>>()?;
    if controls.len() < 3 {
        return Err("plate needs three nodes".into());
    }
    let samples = detail.samples(8, 5);
    let mut points = Vec::new();
    for i in 0..controls.len() {
        let point = controls[i];
        let start = lerp(
            point,
            controls[(i + controls.len() - 1) % controls.len()],
            0.5,
        );
        let end = lerp(point, controls[(i + 1) % controls.len()], 0.5);
        for sample in 0..samples {
            let t = sample as f64 / samples as f64;
            let u = 1.0 - t;
            points.push(add(
                add(mul(start, u * u), mul(point, 2.0 * u * t)),
                mul(end, t * t),
            ));
        }
    }
    Ok(points)
}
pub(super) fn guard_plate(
    r: &ResolvedComponent,
    p: &GuardAssemblyParameters,
    plate: &GuardPlate,
    index: usize,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut outer = rounded(p, &plate.outline, detail)?;
    let mut hole = rounded(p, &plate.cutout, detail)?;
    if outer.len() != hole.len() {
        return Err("guard plate loops need equal station counts".into());
    }
    if signed_area(&outer.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>()) < 0.0 {
        outer.reverse();
        hole.reverse();
    }
    let half = plate.thickness.get() / 2.0;
    let bands = detail.samples(4, 2);
    let dish = plate.dish_depth.map_or(0.0, Metres::get);
    let mut solid = Solid::default();
    let vertex = |band: usize, index: usize, side: f64| {
        let t = band as f64 / bands as f64;
        let mut point = lerp(outer[index], hole[index], t);
        point[2] += dish * (PI * t).sin() + half * side;
        point
    };
    for i in 0..outer.len() {
        let j = (i + 1) % outer.len();
        for band in 0..bands {
            for side in [-1.0, 1.0] {
                let [a, b, c, d] = [
                    vertex(band, i, side),
                    vertex(band, j, side),
                    vertex(band + 1, j, side),
                    vertex(band + 1, i, side),
                ];
                if side > 0.0 {
                    solid.quad(a, b, c, d, 1);
                } else {
                    solid.quad(a, d, c, b, 2);
                }
            }
        }
        for (band, reverse) in [(0, false), (bands, true)] {
            let [a, b, c, d] = [
                vertex(band, i, -1.0),
                vertex(band, j, -1.0),
                vertex(band, j, 1.0),
                vertex(band, i, 1.0),
            ];
            if reverse {
                solid.quad(a, d, c, b, 0);
            } else {
                solid.quad(a, b, c, d, 0);
            }
        }
    }
    let material = plate
        .material
        .or(r.component.material)
        .unwrap_or(Material::Steel);
    let suffix = format!("plate {}", index + 1);
    let mut parts = vec![part(solid.positive(), r, &suffix, material)];
    if let Some(radius) = plate.rim_radius
        && radius.get() > 0.0
    {
        outer.push(outer[0]);
        parts.push(part(
            Solid::sweep(
                &outer,
                &Sweep {
                    width: radius.get() * 2.0,
                    depth: radius.get() * 2.0,
                    radial_segments: 10,
                    fit_bends: true,
                    ..Sweep::default()
                },
                detail,
            )?,
            r,
            &format!("{suffix} rolled rim"),
            material,
        ));
    }
    Ok(parts)
}
