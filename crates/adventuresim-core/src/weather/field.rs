//! Correlated fixed-point weather lattice and interpolation.
use super::*;

/// Smooth deterministic field with synoptic-scale spatial correlation,
/// eastward advection, and day-scale evolution.
pub(super) fn correlated_field(
    seed: u64,
    domain: StreamId,
    interval: u64,
    lat: i32,
    lon: i32,
) -> u16 {
    let interval = interval.min(i64::MAX as u64 / (4 * FIELD_FRACTION as u64)) as i64;
    let x = i64::from(lon) * FIELD_FRACTION - interval * 4 * FIELD_FRACTION;
    let y = i64::from(lat) * FIELD_FRACTION + interval * FIELD_FRACTION;
    let t = interval * FIELD_FRACTION / SYNOPTIC_TIME_INTERVALS;
    let spatial_scale = SYNOPTIC_SPATIAL_CELLS * FIELD_FRACTION;
    let x0 = x.div_euclid(spatial_scale);
    let y0 = y.div_euclid(spatial_scale);
    let t0 = t.div_euclid(FIELD_FRACTION);
    let xf = fade_fraction(x.rem_euclid(spatial_scale), spatial_scale);
    let yf = fade_fraction(y.rem_euclid(spatial_scale), spatial_scale);
    let tf = fade_fraction(t.rem_euclid(FIELD_FRACTION), FIELD_FRACTION);
    let sample = |dx: i64, dy: i64, dt: i64| {
        domain
            .rng(
                seed,
                &[
                    u64::from(WEATHER_RULES_VERSION),
                    (t0 + dt) as u64,
                    (y0 + dy) as u64,
                    (x0 + dx) as u64,
                ],
            )
            .index(usize::from(BASIS_POINTS_PER_WHOLE) + 1) as u16
    };
    let lower = lerp_bps(
        lerp_bps(sample(0, 0, 0), sample(1, 0, 0), xf),
        lerp_bps(sample(0, 1, 0), sample(1, 1, 0), xf),
        yf,
    );
    let upper = lerp_bps(
        lerp_bps(sample(0, 0, 1), sample(1, 0, 1), xf),
        lerp_bps(sample(0, 1, 1), sample(1, 1, 1), xf),
        yf,
    );
    lerp_bps(lower, upper, tf)
}

pub(super) fn fade_fraction(remainder: i64, scale: i64) -> u32 {
    let fraction = (remainder as u64 * 65_535 / scale as u64) as u32;
    let f = u64::from(fraction);
    ((f * f * (3 * 65_535 - 2 * f)) / (65_535 * 65_535)) as u32
}

pub(super) fn lerp_bps(a: u16, b: u16, fraction: u32) -> u16 {
    let a = i64::from(a);
    let delta = i64::from(b) - a;
    (a + delta * i64::from(fraction) / 65_535).clamp(0, i64::from(BASIS_POINTS_PER_WHOLE)) as u16
}
