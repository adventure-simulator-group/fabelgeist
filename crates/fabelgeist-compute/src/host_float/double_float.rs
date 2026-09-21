//! Double-float arithmetic and the transcendental functions built on it.
//!
//! A double-float is a `vec2<f32>` holding the unevaluated sum `x + y`, with
//! `|y|` at most half a unit in the last place of `x`: about 48 bits. A
//! function evaluated in it and rounded once to a float is the correctly
//! rounded result unless its true value lies within about `2^-44` of a unit
//! of a rounding boundary -- about one argument in a million.

use std::fmt::Write;

/// Terms of the sine series `r^(2k+1) / (2k+1)!` over an eighth turn; the
/// next is below `2^-64` of the first.
const SINE_TERMS: u32 = 9;
/// Terms of the cosine series `r^2k / (2k)!`.
const COSINE_TERMS: u32 = 10;
/// Terms of the logarithm's series `2 t^(2k+1) / (2k+1)`, whose argument
/// `t = (m - 1) / (m + 1)` is at most `(sqrt 2 - 1) / (sqrt 2 + 1)`.
const LOG_TERMS: u32 = 14;
/// Terms of the exponential series `r^k / k!` over half a doubling.
const EXP_TERMS: u32 = 17;

/// A quarter turn as the sum of four floats, the error of each the next,
/// from a 100-digit pi: a reduction by it is exact to about `2^-100` for
/// arguments of a few thousand radians. Given as bits, which a decimal
/// might not round back to.
const QUARTER_TURN: [f32; 4] = [
    f32::from_bits(0x3fc9_0fdb),
    f32::from_bits(0xb33b_bd2e),
    f32::from_bits(0xa6f7_2ced),
    f32::from_bits(0x194c_5170),
];
/// The natural logarithm of two as the sum of three floats, likewise.
const LN_2: [f32; 3] = [
    f32::from_bits(0x3f31_7218),
    f32::from_bits(0xb102_e308),
    f32::from_bits(0xa4ca_86c4),
];

/// A float as a WGSL literal. The `f` suffix makes the parser round the
/// decimal straight to a float, never through a double.
fn literal(value: f32) -> String {
    format!("{value:e}f")
}

/// `value` as a leading float and the float nearest the rest.
fn split(value: f64) -> (f32, f32) {
    let high = value as f32;
    (high, (value - f64::from(high)) as f32)
}

fn double_float(value: f64) -> String {
    let (high, low) = split(value);
    format!("vec2<f32>({}, {})", literal(high), literal(low))
}

/// A WGSL function evaluating a double-float polynomial in `z` by Horner's
/// rule; `coefficients` run from the constant term up.
fn horner(name: &str, coefficients: impl DoubleEndedIterator<Item = f64>) -> String {
    let mut coefficients = coefficients.rev();
    let leading = coefficients.next().expect("a polynomial has a term");
    let mut source = format!(
        "fn {name}(z: vec2<f32>) -> vec2<f32> {{\n    var sum = {};\n",
        double_float(leading)
    );
    for coefficient in coefficients {
        writeln!(
            source,
            "    sum = df_add(df_mul(sum, z), {});",
            double_float(coefficient)
        )
        .expect("writing to a string");
    }
    source.push_str("    return sum;\n}\n");
    source
}

fn factorial(n: u32) -> f64 {
    (1..=n).map(f64::from).product()
}

fn alternating(k: u32) -> f64 {
    if k.is_multiple_of(2) { 1.0 } else { -1.0 }
}

/// The double-float operations, the series and the functions.
pub(super) fn source() -> String {
    let mut source = String::from(OPERATIONS);
    for (k, part) in QUARTER_TURN.iter().enumerate() {
        writeln!(
            source,
            "const DF_QUARTER_TURN_{k}: f32 = {};",
            literal(*part)
        )
        .expect("writing to a string");
    }
    for (k, part) in LN_2.iter().enumerate() {
        writeln!(source, "const DF_LN_2_{k}: f32 = {};", literal(*part))
            .expect("writing to a string");
    }
    writeln!(
        source,
        "const DF_TWO_OVER_PI: f32 = {};\nconst DF_LOG2_E: f32 = {};",
        literal(std::f32::consts::FRAC_2_PI),
        literal(std::f32::consts::LOG2_E)
    )
    .expect("writing to a string");
    source += &horner(
        "df_sine_series",
        (0..SINE_TERMS).map(|k| alternating(k) / factorial(2 * k + 1)),
    );
    source += &horner(
        "df_cosine_series",
        (0..COSINE_TERMS).map(|k| alternating(k) / factorial(2 * k)),
    );
    source += &horner(
        "df_atanh_series",
        (0..LOG_TERMS).map(|k| 2.0 / f64::from(2 * k + 1)),
    );
    source += &horner("df_exp_series", (0..EXP_TERMS).map(|k| 1.0 / factorial(k)));
    source += FUNCTIONS;
    source
}

/// The error-free transformations fence their operands as well as their
/// steps: a compiler that could trace an operand back to how it was
/// computed could cancel the very rounding error they measure.
const OPERATIONS: &str = r#"
fn df_two_sum(a_: f32, b_: f32) -> vec2<f32> {
    let a = host_fence(a_);
    let b = host_fence(b_);
    let s = host_fence(a + b);
    let v = host_fence(s - a);
    return vec2<f32>(s, host_add(host_sub(a, host_fence(s - v)), host_sub(b, v)));
}

// The same, for |a| >= |b|.
fn df_quick_two_sum(a_: f32, b_: f32) -> vec2<f32> {
    let a = host_fence(a_);
    let b = host_fence(b_);
    let s = host_fence(a + b);
    return vec2<f32>(s, host_sub(b, host_fence(s - a)));
}

fn df_two_product(a_: f32, b_: f32) -> vec2<f32> {
    let a = host_fence(a_);
    let b = host_fence(b_);
    let p = host_fence(a * b);
    return vec2<f32>(p, fma(a, b, -p));
}

fn df_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let high = df_two_sum(a.x, b.x);
    let low = df_two_sum(a.y, b.y);
    let s = df_quick_two_sum(high.x, host_add(high.y, low.x));
    return df_quick_two_sum(s.x, host_add(s.y, low.y));
}

fn df_add_f32(a: vec2<f32>, b: f32) -> vec2<f32> {
    let high = df_two_sum(a.x, b);
    return df_quick_two_sum(high.x, host_add(high.y, a.y));
}

fn df_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let p = df_two_product(a.x, b.x);
    let cross = host_add(host_mul(a.x, b.y), host_mul(a.y, b.x));
    return df_quick_two_sum(p.x, host_add(p.y, cross));
}

fn df_mul_f32(a: vec2<f32>, b: f32) -> vec2<f32> {
    let p = df_two_product(a.x, b);
    return df_quick_two_sum(p.x, host_add(p.y, host_mul(a.y, b)));
}

// Three quotient digits, each from the remainder of the ones before.
fn df_div(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let q1 = host_fence(a.x / b.x);
    let r1 = df_add(a, -df_mul_f32(b, q1));
    let q2 = host_fence(r1.x / b.x);
    let r2 = df_add(r1, -df_mul_f32(b, q2));
    let q3 = host_fence(r2.x / b.x);
    return df_add_f32(df_quick_two_sum(q1, q2), q3);
}

// The float nearest a double-float.
fn df_round(a: vec2<f32>) -> f32 {
    return host_add(a.x, a.y);
}

// `2^n` for the exponents of normal floats.
fn df_power_of_two(n: i32) -> f32 {
    return bitcast<f32>(u32(n + 127) << 23u);
}
"#;

/// Sine, cosine, arcsine, logarithm, exponential and power.
const FUNCTIONS: &str = r#"
// `x` less its nearest multiple k of a quarter turn, and k. Each part of the
// quarter turn times k is exact as a double-float.
fn df_quarter_turns(x: f32) -> vec3<f32> {
    let k = round(host_mul(x, DF_TWO_OVER_PI));
    var r = df_add(vec2<f32>(x, 0.0), df_two_product(DF_QUARTER_TURN_0, -k));
    r = df_add(r, df_two_product(DF_QUARTER_TURN_1, -k));
    r = df_add(r, df_two_product(DF_QUARTER_TURN_2, -k));
    r = df_add(r, df_two_product(DF_QUARTER_TURN_3, -k));
    return vec3<f32>(r, k);
}

// sin (`odd`) or cos of `x`, unrounded.
fn df_sin_or_cos(x: f32, odd: bool) -> vec2<f32> {
    let reduced = df_quarter_turns(x);
    let r = reduced.xy;
    let quadrant = i32(reduced.z) & 3;
    // An odd quadrant swaps sine and cosine.
    let sine = odd != ((quadrant & 1) == 1);
    let z = df_mul(r, r);
    var value = df_cosine_series(z);
    if (sine) {
        value = df_mul(r, df_sine_series(z));
    }
    // Sine is negative in quadrants 2 and 3, cosine in 1 and 2.
    let negative = select(quadrant == 1 || quadrant == 2, quadrant >= 2, odd);
    return select(value, -value, negative);
}

fn host_sin(x: f32) -> f32 {
    // Keeps the sign of a zero.
    if (x == 0.0) {
        return x;
    }
    return df_round(df_sin_or_cos(x, true));
}

fn host_cos(x: f32) -> f32 {
    return df_round(df_sin_or_cos(x, false));
}

// (sin x, cos x).
fn host_sin_cos(x: f32) -> vec2<f32> {
    return vec2<f32>(host_sin(x), host_cos(x));
}

// asin of x in [-1, 1]: one Newton step from the device's estimate, its
// residual sin(estimate) - x taken in double-float.
fn host_asin(x: f32) -> f32 {
    if (abs(x) == 1.0) {
        return select(-DF_QUARTER_TURN_0, DF_QUARTER_TURN_0, x > 0.0);
    }
    let estimate = asin(x);
    let c = host_cos(estimate);
    if (!(c > 0.0) || !(abs(x) < 1.0)) {
        return estimate;
    }
    let residual = df_add_f32(df_sin_or_cos(estimate, true), -x);
    let correction = host_div(df_round(residual), c);
    return df_round(df_two_sum(estimate, -correction));
}

// The natural logarithm of a positive normal float.
fn df_log(x: f32) -> vec2<f32> {
    // x = m 2^n with m within a factor sqrt 2 of one.
    let bits = bitcast<u32>(x);
    var n = i32((bits >> 23u) & 0xffu) - 127;
    var m = bitcast<f32>((bits & 0x007fffffu) | 0x3f800000u);
    if (m > 1.41421356f) {
        m = m * 0.5;
        n = n + 1;
    }
    // log m = 2 atanh t, t = (m - 1) / (m + 1).
    let t = df_div(vec2<f32>(host_sub(m, 1.0), 0.0), df_two_sum(m, 1.0));
    let log_m = df_mul(t, df_atanh_series(df_mul(t, t)));
    // n ln 2, its first two parts exact as double-floats.
    let e = f32(n);
    let log_2n = df_add(df_two_product(DF_LN_2_0, e), df_two_product(DF_LN_2_1, e));
    return df_add(df_add_f32(log_2n, host_mul(DF_LN_2_2, e)), log_m);
}

// Beyond these, e to the power over- or underflows a float.
const DF_EXP_UNDERFLOW: f32 = -104.0f;
const DF_EXP_OVERFLOW: f32 = 89.0f;

// e to a double-float power, rounded once.
fn df_exp_rounded(y: vec2<f32>) -> f32 {
    if (y.x < DF_EXP_UNDERFLOW) {
        return 0.0;
    }
    if (y.x > DF_EXP_OVERFLOW) {
        return bitcast<f32>(0x7f800000u);
    }
    // y = k ln 2 + r, |r| <= ln 2 / 2.
    let k = round(host_mul(y.x, DF_LOG2_E));
    var r = df_add(y, df_two_product(DF_LN_2_0, -k));
    r = df_add(r, df_two_product(DF_LN_2_1, -k));
    r = df_add_f32(r, host_mul(DF_LN_2_2, -k));
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
    return df_exp_rounded(vec2<f32>(x, 0.0));
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
    return df_exp_rounded(df_mul_f32(df_log(base), exponent));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_sum_to_their_values() {
        let quarter_turn: f64 = QUARTER_TURN.iter().map(|part| f64::from(*part)).sum();
        let ln_2: f64 = LN_2.iter().map(|part| f64::from(*part)).sum();
        assert_eq!(quarter_turn, std::f64::consts::FRAC_PI_2);
        assert_eq!(ln_2, std::f64::consts::LN_2);
    }

    #[test]
    fn literals_parse_back_to_their_floats() {
        for value in QUARTER_TURN
            .iter()
            .chain(&LN_2)
            .copied()
            .chain([0.5, -1.0, 3.0e38, 1.0e-30])
        {
            let text = literal(value);
            let parsed: f32 = text.trim_end_matches('f').parse().unwrap();
            assert_eq!(parsed.to_bits(), value.to_bits(), "{text}");
        }
    }
}
