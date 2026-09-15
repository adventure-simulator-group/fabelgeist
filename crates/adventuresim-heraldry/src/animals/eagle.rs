//! Displayed eagle: independently formed wing fans, necks, feet and tail.
use super::*;
pub(super) fn draw(
    p: &mut Painter<'_>,
    heads: EagleHeads,
    crowned: bool,
    armed: Tincture,
    langued: Tincture,
    facing: Facing,
) {
    let d = p.style.eagle.clone();
    for side in [-1.0, 1.0] {
        wing(p, &d, side);
        foot(p, &d, side, armed);
    }
    tail(p, &d);
    let w = 0.13 * d.body_width.0;
    p.plate(
        Path::new(0.5 - w, 0.27)
            .curve([0.5 - w * 1.35, 0.42], [0.5 - w, 0.66], [0.5, 0.76])
            .curve([0.5 + w, 0.66], [0.5 + w * 1.35, 0.42], [0.5 + w, 0.27])
            .curve([0.56, 0.21], [0.44, 0.21], [0.5 - w, 0.27])
            .close(),
    );
    let necks: &[f32] = if heads == EagleHeads::Two {
        &[-1.0, 1.0]
    } else if facing == Facing::Dexter {
        &[-1.0]
    } else {
        &[1.0]
    };
    for &side in necks {
        head(p, &d, side, heads, crowned, armed, langued);
    }
    if p.style.detail.0 > 0.0 {
        for row in 0..9 {
            for col in 0..3 {
                let x = 0.5 + (col as f32 - 1.0) * w * 0.52 + (row % 2) as f32 * w * 0.12;
                let y = 0.32 + row as f32 * 0.041;
                let t = 0.012;
                p.detail(Path::new(x - t, y).curve(
                    [x - t, y + 0.02],
                    [x, y + 0.034],
                    [x + t, y + 0.01],
                ));
            }
        }
    }
}
fn wing(p: &mut Painter<'_>, d: &EagleDrawing, side: f32) {
    let map = |[x, y]: [f32; 2]| {
        [
            0.5 + side * x * d.wing_span.0,
            y + side * p.style.asymmetry.0 * x * 0.07,
        ]
    };
    let n = d.feather_count;
    let lift = (d.wing_lift.0 - 1.0) * 0.16;
    // Feathers share attachment roots along the curved ulna, with separate fans.
    for i in (0..n).rev() {
        let t = i as f32 / (n - 1) as f32;
        let root = map([0.10 + 0.28 * t, 0.34 - 0.15 * t - lift * t]);
        let tip = map([
            0.12 + 0.34 * t,
            0.66 - 0.27 * t + (d.feather_length.0 - 1.0) * 0.22,
        ]);
        p.plate(leaf(
            root,
            tip,
            0.017 + 0.006 * (1.0 - t),
            side * 0.025 * p.style.contour_character.0,
        ));
        p.detail(Path::new(root[0], root[1]).curve(
            [root[0], root[1] + 0.07],
            [tip[0] - side * 0.008, tip[1] - 0.07],
            tip,
        ));
    }
    let shoulder = Path::new(0.08, 0.35)
        .curve([0.15, 0.25], [0.27, 0.25], [0.39, 0.11 - lift])
        .curve(
            [0.43, 0.07 - lift],
            [0.45, 0.08 - lift],
            [0.45, 0.14 - lift],
        )
        .curve([0.40, 0.26], [0.30, 0.36], [0.11, 0.43])
        .close()
        .mapped(map);
    p.plate(shoulder);
    for row in 0..3 {
        for i in 0..7 {
            let t = i as f32 / 7.0;
            let x = 0.12 + t * 0.25;
            let y = 0.32 - 0.16 * t + row as f32 * 0.021 - lift * t;
            let a = map([x, y]);
            let b = map([x + 0.033, y + 0.035]);
            p.detail(Path::new(a[0], a[1]).curve([a[0], b[1]], [b[0], b[1]], b));
        }
    }
}
fn head(
    p: &mut Painter<'_>,
    d: &EagleDrawing,
    side: f32,
    heads: EagleHeads,
    crowned: bool,
    armed: Tincture,
    langued: Tincture,
) {
    let x = if heads == EagleHeads::Two {
        0.5 + side * 0.10
    } else {
        0.5
    };
    let y = 0.19 - (d.neck_length.0 - 1.0) * 0.08;
    let map = |[u, v]: [f32; 2]| [x + side * u * d.head_size.0, y + v * d.head_size.0];
    p.plate(
        Path::new(-0.04, 0.20)
            .curve([-0.01, 0.12], [-0.04, 0.05], [-0.065, 0.01])
            .curve([-0.065, -0.045], [-0.01, -0.06], [0.045, -0.025])
            .curve([0.08, -0.018], [0.065, 0.028], [0.04, 0.035])
            .curve([0.012, 0.07], [0.045, 0.15], [0.065, 0.20])
            .close()
            .mapped(map),
    );
    p.color(
        Path::new(0.033, -0.012)
            .curve([0.08, -0.02], [0.13, 0.005], [0.12, 0.035])
            .line(0.093, 0.025)
            .line(0.05, 0.034)
            .close()
            .mapped(map),
        armed,
    );
    p.color(
        leaf(map([0.073, 0.036]), map([0.14, 0.062]), 0.006, side * 0.018),
        langued,
    );
    p.color(
        Path::ellipse(
            map([0.018, -0.018])[0],
            map([0.018, -0.018])[1],
            0.006 * d.head_size.0,
            0.006 * d.head_size.0,
        ),
        p.ink,
    );
    p.detail(
        Path::new(-0.03, 0.0)
            .curve([-0.05, 0.08], [0.025, 0.11], [0.01, 0.18])
            .mapped(map),
    );
    if crowned {
        p.crown([x, y - 0.044], 0.11 * d.head_size.0);
    }
}
fn foot(p: &mut Painter<'_>, d: &EagleDrawing, side: f32, armed: Tincture) {
    let root = [0.5 + side * 0.08, 0.64];
    let heel = [0.5 + side * 0.19 * d.leg_spread.0, 0.76];
    p.plate(leaf(root, heel, 0.022, -side * 0.015));
    for i in 0..3 {
        let tip = [
            heel[0] + side * (0.035 + i as f32 * 0.025),
            heel[1] + (i as f32 - 1.0) * 0.037,
        ];
        p.color(leaf(heel, tip, 0.009, side * 0.012), armed);
        p.line(
            Path::new(tip[0], tip[1]).curve(
                [tip[0] + side * 0.02, tip[1] - 0.012],
                [tip[0] + side * 0.02, tip[1] + 0.012],
                [tip[0] + side * 0.012, tip[1] + 0.018],
            ),
            p.style.stroke_width.0 * 0.7,
        );
    }
}
fn tail(p: &mut Painter<'_>, d: &EagleDrawing) {
    for i in 0..7 {
        let t = i as f32 - 3.0;
        let root = [0.5 + t * 0.008, 0.67];
        let tip = [0.5 + t * 0.028 * d.tail_spread.0, 0.94 - t.abs() * 0.035];
        p.plate(leaf(
            root,
            tip,
            0.016,
            t * 0.012 * p.style.contour_character.0,
        ));
        p.detail(Path::new(root[0], root[1]).curve([root[0], 0.78], [tip[0], tip[1] - 0.03], tip));
    }
}
