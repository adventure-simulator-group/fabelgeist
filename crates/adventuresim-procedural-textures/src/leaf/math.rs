//! WGSL scalar semantics used by the renderer-independent field evaluator.
pub(super) use bevy::math::Vec2;
pub(super) fn mix<
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<f32, Output = T>,
>(
    a: T,
    b: T,
    t: f32,
) -> T {
    a + (b - a) * t
}
pub(super) fn select<T>(a: T, b: T, test: bool) -> T {
    if test { b } else { a }
}
pub(super) fn clamp(v: f32, a: f32, b: f32) -> f32 {
    v.clamp(a, b)
}
pub(super) fn smoothstep(a: f32, b: f32, v: f32) -> f32 {
    let t = clamp((v - a) / (b - a), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub(super) fn min(a: f32, b: f32) -> f32 {
    a.min(b)
}
pub(super) fn max(a: f32, b: f32) -> f32 {
    a.max(b)
}
pub(super) fn abs(v: f32) -> f32 {
    v.abs()
}
pub(super) fn sin(v: f32) -> f32 {
    v.sin()
}
pub(super) fn cos(v: f32) -> f32 {
    v.cos()
}
pub(super) fn pow(v: f32, e: f32) -> f32 {
    v.powf(e)
}
pub(super) fn fract(v: f32) -> f32 {
    v - v.floor()
}
pub(super) fn length(v: Vec2) -> f32 {
    v.length()
}
pub(super) fn dot(a: Vec2, b: Vec2) -> f32 {
    a.dot(b)
}
pub(super) fn normalize(v: Vec2) -> Vec2 {
    v.normalize_or_zero()
}
