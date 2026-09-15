//! Forged quillon terminal variants.
use super::*;
pub(super) fn left_terminal(v: &GuardLeftTerminal) -> Option<GuardTerminal> {
    Some(match v {
        GuardLeftTerminal::Shared => return None,
        GuardLeftTerminal::None => GuardTerminal::None,
        GuardLeftTerminal::Ball => GuardTerminal::Ball,
        GuardLeftTerminal::Disk => GuardTerminal::Disk,
        GuardLeftTerminal::Pyramidal => GuardTerminal::Pyramidal,
        GuardLeftTerminal::Scroll => GuardTerminal::Scroll,
        GuardLeftTerminal::Fishtail => GuardTerminal::Fishtail,
        GuardLeftTerminal::Vase => GuardTerminal::Vase,
    })
}
pub(super) fn right_terminal(v: &GuardRightTerminal) -> Option<GuardTerminal> {
    Some(match v {
        GuardRightTerminal::Shared => return None,
        GuardRightTerminal::None => GuardTerminal::None,
        GuardRightTerminal::Ball => GuardTerminal::Ball,
        GuardRightTerminal::Disk => GuardTerminal::Disk,
        GuardRightTerminal::Pyramidal => GuardTerminal::Pyramidal,
        GuardRightTerminal::Scroll => GuardTerminal::Scroll,
        GuardRightTerminal::Fishtail => GuardTerminal::Fishtail,
        GuardRightTerminal::Vase => GuardTerminal::Vase,
    })
}

pub(super) fn terminal(
    style: GuardTerminal,
    size: f64,
    tangent: Point,
    detail: Detail,
) -> Result<Option<Solid>, String> {
    let rotation = [
        tangent[2].atan2(tangent[0].hypot(tangent[1])).to_degrees(),
        0.0,
        (-tangent[0]).atan2(tangent[1]).to_degrees(),
    ];
    let solid = match style {
        GuardTerminal::None => return Ok(None),
        GuardTerminal::Ball => {
            return Ok(Some(Solid::lathe(
                &[
                    [-size, 0.002],
                    [-size * 0.72, size * 0.7],
                    [0.0, size],
                    [size * 0.72, size * 0.7],
                    [size, 0.002],
                ],
                14,
                1.0,
                false,
                detail,
            )?));
        }
        GuardTerminal::Disk => Solid::lathe(
            &[[-size * 0.22, size], [size * 0.22, size]],
            14,
            1.0,
            false,
            detail,
        )?,
        GuardTerminal::Pyramidal => Solid::prism(
            &[[-size, -size], [size, -size], [0.0, size * 1.25]],
            size * 1.4,
            detail,
        )?,
        GuardTerminal::Fishtail => Solid::rounded_plate(
            &[
                [-size * 0.45, -size],
                [-size, size],
                [0.0, size * 0.45],
                [size, size],
                [size * 0.45, -size],
            ],
            size * 0.65,
            0.14,
        )?,
        GuardTerminal::Scroll => {
            let count = detail.samples(18, 8);
            let points: Vec<_> = (0..count)
                .map(|i| {
                    let t = i as f64 / (count - 1) as f64;
                    let a = t * PI * 1.6;
                    let radius = size * (1.0 - t * 0.65);
                    [a.cos() * radius - size, a.sin() * radius, 0.0]
                })
                .collect();
            Solid::sweep(
                &points,
                &Sweep {
                    width: size * 0.35,
                    depth: size * 0.35,
                    fit_bends: true,
                    ..Sweep::default()
                },
                detail,
            )?
        }
        GuardTerminal::Vase => Solid::lathe(
            &[
                [-size, size * 0.45],
                [-size * 0.45, size],
                [size * 0.35, size * 0.7],
                [size, size * 0.35],
            ],
            14,
            1.0,
            false,
            detail,
        )?,
    };
    Ok(Some(solid.transform(rotation, [0.0; 3])))
}
