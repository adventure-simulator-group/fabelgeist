//! Crossbow tillers, prods, open lock cavities, spanning hardware and strings.
use super::*;
mod components;
mod hardware;
use components::*;
use hardware::*;
use std::f64::consts::TAU;

fn part(solid: Solid, material: Material, label: &str, r: &ResolvedComponent) -> PartSource {
    PartSource::new(solid, material, label, &r.id)
}
pub(super) fn prod(p: &CrossbowParameters, detail: Detail) -> Vec<Point> {
    let half = p.prod_span.get() / 2.0;
    let samples = detail.samples(p.samples.map_or(18, |n| n.0 as usize), 10);
    (0..=samples)
        .map(|i| {
            let u = i as f64 / samples as f64 * 2.0 - 1.0;
            [
                u * half,
                p.prod_position.get() + p.prod_sweep.get() * u.abs().powf(1.7),
                0.0,
            ]
        })
        .collect()
}
pub(super) fn tip_loop(
    p: &CrossbowParameters,
    points: &[Point],
    right: bool,
    detail: Detail,
) -> (Point, Vec<Point>) {
    let tip = points[if right { points.len() - 1 } else { 0 }];
    let adjacent = points[if right { points.len() - 2 } else { 1 }];
    let tangent = normalize(sub(tip, adjacent));
    let normal = normalize(cross([0.0, 0.0, 1.0], tangent));
    let binormal = normalize(cross(tangent, normal));
    let depth = p.prod_depth.get() * p.prod_tip_scale.get() * 0.55 + p.tip_loop_clearance.get();
    let thickness =
        p.prod_thickness.get() * p.prod_tip_scale.get() * 0.55 + p.tip_loop_clearance.get();
    let toward = normalize([-tip[0], p.nut_position.get() - tip[1], 0.0]);
    let sign = if dot(toward, normal) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let attachment = add(tip, mul(normal, depth * sign));
    let samples = detail.samples(24, 20);
    let loop_points = (0..=samples)
        .map(|i| {
            let a = i as f64 / samples as f64 * TAU;
            add(
                add(tip, mul(normal, a.cos() * depth)),
                mul(binormal, a.sin() * thickness),
            )
        })
        .collect();
    (attachment, loop_points)
}

pub(super) fn crossbow(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut parts = stock(r, p, detail)?;
    limbs(r, p, detail, &mut parts)?;
    strings(r, p, detail, &mut parts)?;
    bridles(r, p, detail, &mut parts)?;
    nut(r, p, detail, &mut parts)?;
    let mut boxes = notch_blocks(p);
    runners(p, &mut boxes);
    trigger(r, p, detail, &mut parts, &mut boxes)?;
    facing(p, &mut boxes);
    stirrup(r, p, detail, &mut parts)?;
    spanning(r, p, detail, &mut parts, &mut boxes)?;
    for (size, offset, material, label) in boxes {
        parts.push(part(
            Solid::cuboid(size, detail)?.transform([0.0; 3], offset),
            material,
            &label,
            r,
        ));
    }
    sights(r, p, detail, &mut parts)?;
    Ok(parts)
}

#[derive(Debug)]
pub(super) struct StockLayout {
    pub(super) rear: [StockStation; 3],
    pub(super) fore: [StockStation; 3],
    pub(super) cavity_start: f64,
    pub(super) cavity_end: f64,
    pub(super) gap: f64,
    pub(super) cheek: f64,
}
impl StockLayout {
    pub(super) fn new(p: &CrossbowParameters) -> Self {
        let nut = p.nut_position.get();
        let nr = p.nut_radius.get();
        let length = p.length.get();
        let butt = p.butt_width.get();
        let stock = p.stock_thickness.get();
        let rail = p.rail_height.get();
        let table = p.lock_table_height.get();
        let cavity_start = nut - nr - 0.004;
        let cavity_end = nut + nr + 0.004;
        let gap = p.nut_width.get() + 0.008;
        let lock_width = p.waist_width.get().max(gap + 0.012);
        let cheek = (lock_width - gap) / 2.0;
        let waist_scale = match p.stock_style {
            CrossbowStockStyle::Hunting => 0.90,
            CrossbowStockStyle::Swollen => 1.14,
            CrossbowStockStyle::Straight => 1.0,
        };
        let rear = [
            StockStation::new(0.0, butt, -stock - p.butt_drop.get(), -p.butt_drop.get()),
            StockStation::new(
                cavity_start * 0.55,
                p.waist_width.get() * waist_scale,
                -stock,
                0.0,
            ),
            StockStation::new(cavity_start, lock_width, -table, 0.0),
        ];
        let fore = [
            StockStation::new(cavity_end, lock_width, -stock, -rail),
            StockStation::new(
                cavity_end + (length - cavity_end) * 0.65,
                p.nose_width.get() * 1.06,
                -stock + p.fore_end_rise.get() * 0.65,
                -rail,
            ),
            StockStation::new(
                length,
                p.nose_width.get(),
                -stock + p.fore_end_rise.get(),
                -rail,
            ),
        ];
        Self {
            rear,
            fore,
            cavity_start,
            cavity_end,
            gap,
            cheek,
        }
    }
}
