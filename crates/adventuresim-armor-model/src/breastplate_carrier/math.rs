//! Vector operations and interpolation for the carrier surface.

use super::*;

pub(super) fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

pub(super) fn normalized(a: [f32; 3]) -> Result<[f32; 3], GenerateError> {
    let magnitude = length(a);
    if magnitude > 1e-12 && magnitude.is_finite() {
        Ok(scale(a, magnitude.recip()))
    } else {
        eprintln!("breastplate cannot normalize {a:?}");
        Err(GenerateError::Degenerate)
    }
}

pub(super) fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

impl Frame {
    pub(super) fn from_front(front_hint: [f32; 3]) -> Result<Self, GenerateError> {
        // MHR armor is authored in its stable Y-up model frame. The semantic
        // spine regression can tilt with posture and must not rotate an entire
        // rigid plate (a 4.5-degree tilt moved the rear plate by over 10 cm).
        let vertical = [0.0, 1.0, 0.0];
        let front = normalized([front_hint[0], 0.0, front_hint[2]])?;
        let lateral = normalized(cross(vertical, front))?;
        Ok(Frame {
            lateral,
            vertical,
            front,
        })
    }
}

pub(super) fn local(point: [f32; 3], frame: Frame) -> [f32; 3] {
    [
        dot(point, frame.lateral),
        dot(point, frame.vertical),
        dot(point, frame.front),
    ]
}

pub(super) fn world(point: [f32; 3], frame: Frame) -> [f32; 3] {
    add(
        add(
            scale(frame.lateral, point[0]),
            scale(frame.vertical, point[1]),
        ),
        scale(frame.front, point[2]),
    )
}

pub(super) fn cubic_eval(x: f32, xs: &[f32], ys: &[f32], zero_start: bool) -> (f32, f32) {
    debug_assert_eq!(xs.len(), ys.len());
    let mut slopes = Vec::with_capacity(xs.len());
    for index in 0..xs.len() {
        let slope = if index == 0 {
            if zero_start {
                0.0
            } else {
                (ys[1] - ys[0]) / (xs[1] - xs[0])
            }
        } else if index + 1 == xs.len() {
            (ys[index] - ys[index - 1]) / (xs[index] - xs[index - 1])
        } else {
            (ys[index + 1] - ys[index - 1]) / (xs[index + 1] - xs[index - 1])
        };
        slopes.push(slope);
    }
    if x <= xs[0] {
        return (ys[0], slopes[0]);
    }
    if x >= xs[xs.len() - 1] {
        return (ys[ys.len() - 1], slopes[slopes.len() - 1]);
    }
    let index = xs
        .windows(2)
        .position(|span| x <= span[1])
        .unwrap_or(xs.len() - 2);
    let h = xs[index + 1] - xs[index];
    let t = (x - xs[index]) / h;
    let t2 = t * t;
    let t3 = t2 * t;
    let value = (2.0 * t3 - 3.0 * t2 + 1.0) * ys[index]
        + (t3 - 2.0 * t2 + t) * h * slopes[index]
        + (-2.0 * t3 + 3.0 * t2) * ys[index + 1]
        + (t3 - t2) * h * slopes[index + 1];
    let derivative = (6.0 * t2 - 6.0 * t) * ys[index] / h
        + (3.0 * t2 - 4.0 * t + 1.0) * slopes[index]
        + (-6.0 * t2 + 6.0 * t) * ys[index + 1] / h
        + (3.0 * t2 - 2.0 * t) * slopes[index + 1];
    (value, derivative)
}

pub(super) fn cubic(x: f32, xs: &[f32], ys: &[f32], zero_start: bool) -> f32 {
    cubic_eval(x, xs, ys, zero_start).0
}
