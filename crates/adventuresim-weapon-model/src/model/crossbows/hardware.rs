//! Release, runner, spanning and sight hardware built from shared stock datums.
use super::*;
type Blocks = Vec<(Point, Point, Material, String)>;
pub(super) fn notch_blocks(p: &CrossbowParameters) -> Blocks {
    let nut = p.nut_position.get();
    let radius = p.string_radius.get();
    let rail = p.rail_height.get();
    let notch = (radius * 3.0).max(0.006);
    vec![
        (
            [notch, radius * 3.0, p.nut_thickness.get()],
            [0.0, nut, -p.nut_thickness.get() / 2.0 - radius * 1.3],
            Material::Horn,
            "nut string notch floor".to_string(),
        ),
        (
            [p.groove_width.get(), 0.014, rail],
            [0.0, nut + 0.012, -rail / 2.0],
            Material::Horn,
            "bolt butt nut shelf".to_string(),
        ),
    ]
}
pub(super) fn runners(p: &CrossbowParameters, boxes: &mut Blocks) {
    let nut = p.nut_position.get();
    let rail = p.rail_height.get();
    let prod_position = p.prod_position.get();
    let butt = p.butt_width.get();
    let rail_width = ((butt - p.groove_width.get()) / 4.0).max(0.003);
    let rail_length = prod_position - nut;
    for (side, name) in [(-1.0, "left"), (1.0, "right")] {
        boxes.push((
            [rail_width, rail_length, rail],
            [
                side * (p.groove_width.get() + rail_width) / 2.0,
                nut + rail_length / 2.0,
                -rail / 2.0,
            ],
            if p.facing_style == CrossbowFacingStyle::Horn {
                Material::Horn
            } else {
                Material::Wood
            },
            format!("{name} recessed bolt runner rail"),
        ));
    }
}
pub(super) fn facing(p: &CrossbowParameters, boxes: &mut Blocks) {
    let rail = p.rail_height.get();
    let prod_position = p.prod_position.get();
    let cavity_end = StockLayout::new(p).cavity_end;
    if p.facing_style == CrossbowFacingStyle::Horn {
        boxes.push((
            [
                p.nose_width.get() * 0.78,
                prod_position - cavity_end,
                p.facing_thickness.get(),
            ],
            [
                0.0,
                (cavity_end + prod_position) / 2.0,
                -rail - p.facing_thickness.get() / 2.0,
            ],
            Material::Horn,
            "staghorn fore-end facing and inlay".into(),
        ));
    }
}
pub(super) fn trigger(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
    boxes: &mut Blocks,
) -> Result<(), String> {
    let nut = p.nut_position.get();
    let nr = p.nut_radius.get();
    let sear = -nr * 0.72;
    boxes.push((
        [0.010, 0.020, 0.010],
        [0.0, nut - nr * 0.72, sear],
        Material::DarkSteel,
        "nut sear notch and tooth".into(),
    ));
    let trigger = p.trigger_length.get();
    parts.push(part(
        Solid::prism(
            &[
                [-0.007, 0.0],
                [0.007, 0.0],
                [0.012, -trigger],
                [-0.004, -trigger * 1.05],
            ],
            0.008,
            detail,
        )?
        .transform([0.0; 3], [0.0, nut - nr * 0.72, sear - 0.002]),
        Material::DarkSteel,
        "long trigger to sear",
        r,
    ));

    Ok(())
}
pub(super) fn stirrup(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let rail = p.rail_height.get();
    let length = p.length.get();
    let stirrup = [
        [-p.nose_width.get() / 2.0, length, -rail],
        [
            -p.stirrup_width.get() / 2.0,
            length + p.stirrup_length.get(),
            -rail,
        ],
        [
            p.stirrup_width.get() / 2.0,
            length + p.stirrup_length.get(),
            -rail,
        ],
        [p.nose_width.get() / 2.0, length, -rail],
    ];
    parts.push(part(
        Solid::sweep(
            &stirrup,
            &Sweep {
                width: p.stirrup_bar.get() * 2.0,
                depth: p.stirrup_bar.get() * 2.0,
                ..Sweep::default()
            },
            detail,
        )?,
        Material::DarkSteel,
        "spanning stirrup",
        r,
    ));

    Ok(())
}
pub(super) fn spanning(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
    boxes: &mut Blocks,
) -> Result<(), String> {
    let nut = p.nut_position.get();
    let table = p.lock_table_height.get();
    let butt = p.butt_width.get();
    let bar = p.spanning_bar.get();
    match p.spanning_mode {
        CrossbowSpanningMode::Cranequin => boxes.extend([
            (
                [butt * 1.18, 0.018, bar],
                [0.0, nut - 0.12, -table / 2.0],
                Material::DarkSteel,
                "cranequin stock rest peg".into(),
            ),
            (
                [bar, 0.13, bar],
                [butt * 0.46, nut - 0.075, -table / 2.0],
                Material::DarkSteel,
                "cranequin rack purchase rail".into(),
            ),
        ]),
        CrossbowSpanningMode::GoatsFoot => {
            for (side, name) in [(-1.0, "left"), (1.0, "right")] {
                boxes.push((
                    [bar, 0.038, 0.025],
                    [side * butt / 2.0, nut + 0.075, 0.0],
                    Material::Steel,
                    format!("{name} goats-foot pivot lug"),
                ));
            }
            parts.push(part(
                Solid::lathe(
                    &[[-butt * 0.62, bar / 2.0], [butt * 0.62, bar / 2.0]],
                    10,
                    1.0,
                    false,
                    detail,
                )?
                .transform([0.0, 0.0, 90.0], [0.0, nut + 0.075, 0.0]),
                Material::Steel,
                "goats-foot pivot axle",
                r,
            ));
        }
        CrossbowSpanningMode::BeltHook => boxes.push((
            [butt * 1.12, 0.030, bar],
            [0.0, nut + 0.11, -table / 2.0],
            Material::Steel,
            "belt-hook purchase bar".into(),
        )),
    }

    Ok(())
}
pub(super) fn sights(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let nut = p.nut_position.get();
    let rear = StockLayout::new(p).rear;
    if p.sight_style != CrossbowSightStyle::None {
        let y = nut - 0.055;
        let station = StockStation::at(&rear, y);
        parts.push(part(
            Solid::lathe(&[[-0.001, 0.003], [0.018, 0.0025]], 10, 1.0, false, detail)?
                .transform([90.0, 0.0, 0.0], [0.0, y, station.top]),
            Material::Steel,
            "sight mounting stem",
            r,
        ));
        if p.sight_style == CrossbowSightStyle::Peep {
            let ring = RingGuardParameters {
                radius: Metres::new(0.012)?,
                bar: Some(Metres::new(0.0025)?),
                samples: Some(Count(18)),
                arc_start: None,
                arc_end: None,
                radial_segments: None,
            };
            parts.push(part(
                guards::ring(&ring, detail)?
                    .transform([90.0, 0.0, 0.0], [0.0, y, station.top + 0.028]),
                Material::Steel,
                "folding peep sight",
                r,
            ));
        }
    }

    Ok(())
}
