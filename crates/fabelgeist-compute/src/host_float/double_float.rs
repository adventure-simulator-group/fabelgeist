//! Double-float arithmetic and the transcendental functions built on it.
//!
//! A WGSL `DoubleFloat` holds the unevaluated sum `high + low`, with
//! `|low|` at most half a unit in the last place of `high`: about 48 bits. A
//! function evaluated in it and rounded once to a float is the correctly
//! rounded result unless its true value lies within about `2^-44` of a unit
//! of a rounding boundary -- about one argument in a million.

mod generation;

pub(super) use generation::DoubleFloatLibrary;

/// The error-free transformations fence their operands as well as their
/// steps: a compiler that could trace an operand back to how it was
/// computed could cancel the very rounding error they measure.
const OPERATIONS: &str = r#"
struct DoubleFloat {
    high: f32,
    low: f32,
};

fn df_from_scalar(value: f32) -> DoubleFloat {
    return DoubleFloat(host_fence(value), 0.0);
}

fn df_two_sum(a_: f32, b_: f32) -> DoubleFloat {
    let a = host_fence(a_);
    let b = host_fence(b_);
    let s = host_fence(a + b);
    let v = host_fence(s - a);
    return DoubleFloat(s, host_add(host_sub(a, host_fence(s - v)), host_sub(b, v)));
}

// Use the shorter transformation when |a| >= |b|; validate this on admission.
fn df_quick_two_sum(a_: f32, b_: f32) -> DoubleFloat {
    let a = host_fence(a_);
    let b = host_fence(b_);
    if (abs(a) < abs(b)) {
        return df_two_sum(a, b);
    }
    let s = host_fence(a + b);
    return DoubleFloat(s, host_sub(b, host_fence(s - a)));
}

fn df_add(a: DoubleFloat, b: DoubleFloat) -> DoubleFloat {
    let high = df_two_sum(a.high, b.high);
    // A single-word operand needs only one residual renormalization.
    if (b.low == 0.0) {
        return df_quick_two_sum(high.high, host_add(high.low, a.low));
    }
    let low = df_two_sum(a.low, b.low);
    let s = df_quick_two_sum(high.high, host_add(high.low, low.high));
    return df_quick_two_sum(s.high, host_add(s.low, low.low));
}

fn df_mul(a: DoubleFloat, b: DoubleFloat) -> DoubleFloat {
    let p = product_to_double_float(product_expansion(a.high, b.high));
    // Preserve the scalar multiplication's rounding order when its error
    // word is zero. The general case also accounts for the other low word.
    if (b.low == 0.0) {
        return df_quick_two_sum(p.high, host_add(p.low, host_mul(a.low, b.high)));
    }
    let cross = host_add(host_mul(a.high, b.low), host_mul(a.low, b.high));
    return df_quick_two_sum(p.high, host_add(p.low, cross));
}

fn df_negate(value: DoubleFloat) -> DoubleFloat {
    return DoubleFloat(-value.high, -value.low);
}

// Three quotient digits, each from the remainder of the ones before.
fn df_div(a: DoubleFloat, b: DoubleFloat) -> DoubleFloat {
    let q1 = host_fence(a.high / b.high);
    let r1 = df_add(a, df_negate(df_mul(b, df_from_scalar(q1))));
    let q2 = host_fence(r1.high / b.high);
    let r2 = df_add(r1, df_negate(df_mul(b, df_from_scalar(q2))));
    let q3 = host_fence(r2.high / b.high);
    return df_add(df_quick_two_sum(q1, q2), df_from_scalar(q3));
}

// The float nearest a double-float.
fn df_round(a: DoubleFloat) -> f32 {
    return host_add(a.high, a.low);
}

// `2^n` for the exponents of normal floats.
fn df_power_of_two(n: i32) -> f32 {
    return bitcast<f32>(u32(n + 127) << 23u);
}
"#;

/// Sine, cosine, arcsine, logarithm, exponential and power.
const FUNCTIONS: &str = r#"
struct QuarterTurnReduction {
    remainder: DoubleFloat,
    turns: f32,
};

// A function selection, independent of the reduced angle's quadrant.
struct TrigonometricComponent {
    sine: bool,
};
const TRIGONOMETRIC_SINE: TrigonometricComponent = TrigonometricComponent(true);
const TRIGONOMETRIC_COSINE: TrigonometricComponent = TrigonometricComponent(false);

// `x` less its nearest multiple k of a quarter turn, and k. Each part of the
// quarter turn times k is exact as a double-float.
fn df_quarter_turns(x: f32) -> QuarterTurnReduction {
    let k = round(host_mul(x, DF_TWO_OVER_PI));
    var r = df_add(df_from_scalar(x), product_to_double_float(product_expansion(DF_QUARTER_TURN_0, -k)));
    r = df_add(r, product_to_double_float(product_expansion(DF_QUARTER_TURN_1, -k)));
    r = df_add(r, product_to_double_float(product_expansion(DF_QUARTER_TURN_2, -k)));
    r = df_add(r, product_to_double_float(product_expansion(DF_QUARTER_TURN_3, -k)));
    return QuarterTurnReduction(r, k);
}

// Sine or cosine of a reduced angle, unrounded.
fn df_sin_or_cos(reduced: QuarterTurnReduction, component: TrigonometricComponent) -> DoubleFloat {
    let r = reduced.remainder;
    let quadrant = i32(reduced.turns) & 3;
    // An odd quadrant swaps sine and cosine.
    let sine = component.sine != ((quadrant & 1) == 1);
    let z = df_mul(r, r);
    var value = df_cosine_series(z);
    if (sine) {
        value = df_mul(r, df_sine_series(z));
    }
    // Sine is negative in quadrants 2 and 3, cosine in 1 and 2.
    let negative = select(quadrant == 1 || quadrant == 2, quadrant >= 2, component.sine);
    if (negative) { return df_negate(value); }
    return value;
}

fn host_sin(x: f32) -> f32 {
    // Keeps the sign of a zero.
    if (x == 0.0) {
        return x;
    }
    return df_round(df_sin_or_cos(df_quarter_turns(x), TRIGONOMETRIC_SINE));
}

fn host_cos(x: f32) -> f32 {
    return df_round(df_sin_or_cos(df_quarter_turns(x), TRIGONOMETRIC_COSINE));
}

// (sin x, cos x).
fn host_sin_cos(x: f32) -> vec2<f32> {
    return vec2<f32>(host_sin(x), host_cos(x));
}

// asin of x in [-1, 1]: bounded Newton corrections from the device's estimate,
// with sin(estimate) - x evaluated in double-float. Stop when binary32 rounding
// stabilizes; the built-in estimate need not be accurate enough for one step.
const ASIN_CORRECTION_STEPS: u32 = 3u;
fn host_asin(x: f32) -> f32 {
    if (abs(x) == 1.0) {
        return select(-DF_QUARTER_TURN_0, DF_QUARTER_TURN_0, x > 0.0);
    }
    if (x == 0.0) { return x; }
    var estimate = host_fence(asin(x));
    if (!(abs(x) < 1.0)) { return estimate; }
    for (var step = 0u; step < ASIN_CORRECTION_STEPS; step = step + 1u) {
        let c = host_cos(estimate);
        if (!(c > 0.0)) { return estimate; }
        let residual = df_add(df_sin_or_cos(df_quarter_turns(estimate), TRIGONOMETRIC_SINE), df_from_scalar(-x));
        let correction = host_div(df_round(residual), c);
        let next = df_round(df_two_sum(estimate, -correction));
        if (next == estimate) { return next; }
        estimate = next;
    }
    return estimate;
}

// The natural logarithm of a positive normal float.
fn df_log(x: f32) -> DoubleFloat {
    // x = m 2^n with m within a factor sqrt 2 of one.
    let bits = bitcast<u32>(x);
    var n = i32((bits >> 23u) & 0xffu) - 127;
    var m = bitcast<f32>((bits & 0x007fffffu) | 0x3f800000u);
    if (m > 1.41421356f) {
        m = m * 0.5;
        n = n + 1;
    }
    // log m = 2 atanh t, t = (m - 1) / (m + 1).
    let t = df_div(df_from_scalar(host_sub(m, 1.0)), df_two_sum(m, 1.0));
    let log_m = df_mul(t, df_atanh_series(df_mul(t, t)));
    // n ln 2, its first two parts exact as double-floats.
    let e = f32(n);
    let log_2n = df_add(product_to_double_float(product_expansion(DF_LN_2_0, e)), product_to_double_float(product_expansion(DF_LN_2_1, e)));
    return df_add(df_add(log_2n, df_from_scalar(host_mul(DF_LN_2_2, e))), log_m);
}

// Beyond these, e to the power over- or underflows a float.
const DF_EXP_UNDERFLOW: f32 = -104.0f;
const DF_EXP_OVERFLOW: f32 = 89.0f;

// e to a double-float power, rounded once.
fn df_exp_rounded(y: DoubleFloat) -> f32 {
    if (y.high < DF_EXP_UNDERFLOW) {
        return 0.0;
    }
    if (y.high > DF_EXP_OVERFLOW) {
        return bitcast<f32>(0x7f800000u);
    }
    // y = k ln 2 + r, |r| <= ln 2 / 2.
    let k = round(host_mul(y.high, DF_LOG2_E));
    var r = df_add(y, product_to_double_float(product_expansion(DF_LN_2_0, -k)));
    r = df_add(r, product_to_double_float(product_expansion(DF_LN_2_1, -k)));
    r = df_add(r, df_from_scalar(host_mul(DF_LN_2_2, -k)));
    let scaled = df_round(df_exp_series(r));
    // 2^k in two factors, neither of which overflows.
    let half = i32(k) / 2;
    return host_mul(host_mul(scaled, df_power_of_two(half)), df_power_of_two(i32(k) - half));
}

// The natural logarithm of a positive normal float; log 0 is -infinity,
// and a negative argument NaN.
fn host_log(x: f32) -> f32 {
    if (x == 0.0) {
        return -bitcast<f32>(0x7f800000u);
    }
    if (!(x > 0.0) || !(x <= HOST_FLOAT_MAX)) {
        return log(x);
    }
    return df_round(df_log(x));
}

fn host_exp(x: f32) -> f32 {
    // NaN, tested on its bits: a compiler may take `x != x` to be false.
    if ((bitcast<u32>(x) & 0x7fffffffu) > 0x7f800000u) {
        return x;
    }
    return df_exp_rounded(df_from_scalar(x));
}

// `base` to the `exponent`, for a positive normal base. Any exponent of
// zero and a base of one give one; a base of zero -- or any other
// non-positive base -- gives zero, as the host's `powf` does for zero to a
// positive exponent.
fn host_pow(base: f32, exponent: f32) -> f32 {
    if (exponent == 0.0 || base == 1.0) {
        return 1.0;
    }
    if (!(base > 0.0)) {
        return 0.0;
    }
    return df_exp_rounded(df_mul(df_log(base), df_from_scalar(exponent)));
}
"#;
