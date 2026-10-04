//! A single swept round bar, without separate forged hilt furniture.
use super::*;
pub(super) fn bar(p: &BentBarParameters, detail: Detail) -> Result<Solid, String> {
    let samples = detail.samples(p.samples.0 as usize, 4);
    let points: Vec<_> = (0..=samples)
        .map(|i| {
            let t = i as f64 / samples as f64;
            match p.centerline {
                BarCenterline::Opposed { span, sweep } => [
                    span.get() * (t - 0.5),
                    sweep.get() * (t * 2.0 - 1.0).powi(3),
                    0.0,
                ],
                BarCenterline::Arch {
                    width,
                    length,
                    bulge,
                    side,
                } => {
                    let arch = (std::f64::consts::PI * t).sin();
                    [
                        side.sign() * (width.get() * arch + bulge.get() * arch * arch),
                        length.get() * t,
                        0.0,
                    ]
                }
            }
        })
        .collect();
    Solid::sweep(
        &points,
        &Sweep {
            width: p.radius.get() * 2.0,
            depth: p.radius.get() * 2.0,
            radial_segments: p.radial_segments.0 as usize,
            ..Sweep::default()
        },
        detail,
    )
}
