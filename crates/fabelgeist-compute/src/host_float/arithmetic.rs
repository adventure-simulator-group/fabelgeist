//! The fence, the correctly rounded operations, and the vector forms the
//! host's `[f32; 3]` arithmetic takes.

/// Every function fences its operands as well as its result: an operand the
/// caller computed inline -- `host_add(a * b, c)` -- rounds before the
/// operation meets it, as the host's `a * b + c` does, instead of fusing.
pub(super) const ARITHMETIC: &str = r#"
// The largest finite float. Beyond it -- and for NaN -- a quotient or root
// is the device's own, which is the host's special value too.
const HOST_FLOAT_MAX: f32 = 0x1.fffffep+127f;
// How many units in the last place a quotient or root may walk. The
// device's division and square root are within a few units of the nearest.
const HOST_FLOAT_WALK: u32 = 4u;

// `x` exactly as rounded. Its bits pass through an integer OR with a zero
// the compiler cannot see, so no float operation fuses with it or is
// reassociated across it.
fn host_fence(x: f32) -> f32 {
    return bitcast<f32>(bitcast<u32>(x) | host_zero());
}

fn host_fence3(v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(host_fence(v.x), host_fence(v.y), host_fence(v.z));
}

fn host_add(a: f32, b: f32) -> f32 {
    return host_fence(host_fence(a) + host_fence(b));
}

fn host_sub(a: f32, b: f32) -> f32 {
    return host_fence(host_fence(a) - host_fence(b));
}

fn host_mul(a: f32, b: f32) -> f32 {
    return host_fence(host_fence(a) * host_fence(b));
}

// The float next to `x` toward positive (`up`) or negative infinity.
fn host_next_float(x: f32, up: bool) -> f32 {
    if (x == 0.0) {
        return select(-bitcast<f32>(1u), bitcast<f32>(1u), up);
    }
    let bits = bitcast<u32>(x);
    return bitcast<f32>(select(bits - 1u, bits + 1u, up == (x > 0.0)));
}

// Of two adjacent candidates bracketing the true quotient, the one whose
// exact residual is smaller; residuals scale with the distance to the
// quotient by the same divisor, so smaller is nearer.
fn host_nearer(q: f32, r: f32, q2: f32, r2: f32) -> f32 {
    if (abs(r2) < abs(r)) {
        return q2;
    }
    if (abs(r2) > abs(r)) {
        return q;
    }
    return select(q2, q, (bitcast<u32>(q) & 1u) == 0u);
}

// The correctly rounded quotient: the device's, walked toward the true
// quotient by exact fused residuals until two neighbours bracket it.
fn host_div(a_: f32, b_: f32) -> f32 {
    let a = host_fence(a_);
    let b = host_fence(b_);
    var q = host_fence(a / b);
    if (!(abs(q) <= HOST_FLOAT_MAX) || q == 0.0) {
        return q;
    }
    for (var k = 0u; k < HOST_FLOAT_WALK; k = k + 1u) {
        // a - q b, exact once q is within a unit of a / b.
        let r = fma(-q, b, a);
        if (r == 0.0) {
            return q;
        }
        let q2 = host_next_float(q, (r > 0.0) == (b > 0.0));
        let r2 = fma(-q2, b, a);
        if (r2 == 0.0) {
            return q2;
        }
        if ((r2 > 0.0) != (r > 0.0)) {
            return host_nearer(q, r, q2, r2);
        }
        q = q2;
    }
    return q;
}

// The correctly rounded square root. With s's significand an integer S
// units u, x - s^2 and s u are both whole multiples of u^2, and the root
// lies above the midpoint s + u/2 exactly when x - (s + u/2)^2 =
// (x - s^2) - s u - u^2/4 is positive: when the exact residual
// `fma(-s, s, x)` exceeds s u. Below, likewise, with the unit under s.
// Unlike comparing the two neighbours' residuals, which weigh the distance
// to the root by different factors, this never picks the farther one.
fn host_sqrt(x_: f32) -> f32 {
    let x = host_fence(x_);
    if (!(x > 0.0) || !(x <= HOST_FLOAT_MAX)) {
        return host_fence(sqrt(x));
    }
    // s u would underflow for a root this small; scaling by an even power
    // of two moves the root by exactly half as many, so it stays nearest.
    if (x < HOST_SQRT_SCALED_BELOW) {
        return host_fence(host_sqrt_walked(host_fence(x * HOST_SQRT_SCALE)) * HOST_SQRT_UNSCALE);
    }
    return host_sqrt_walked(x);
}

// Beneath this, a square root is taken of the argument times 2^64.
const HOST_SQRT_SCALED_BELOW: f32 = 0x1p-64f;
const HOST_SQRT_SCALE: f32 = 0x1p64f;
const HOST_SQRT_UNSCALE: f32 = 0x1p-32f;

// The nearest root of a positive x at least 2^-64.
fn host_sqrt_walked(x: f32) -> f32 {
    var s = host_fence(sqrt(x));
    for (var k = 0u; k < HOST_FLOAT_WALK; k = k + 1u) {
        let r = fma(-s, s, x);
        let above = host_next_float(s, true);
        let below = host_next_float(s, false);
        if (r > host_fence(s * (above - s))) {
            s = above;
        } else if (-r >= host_fence(s * (s - below))) {
            s = below;
        } else {
            break;
        }
    }
    return s;
}

fn host_add3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(host_add(a.x, b.x), host_add(a.y, b.y), host_add(a.z, b.z));
}

fn host_sub3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(host_sub(a.x, b.x), host_sub(a.y, b.y), host_sub(a.z, b.z));
}

fn host_scale3(v: vec3<f32>, s: f32) -> vec3<f32> {
    return vec3<f32>(host_mul(v.x, s), host_mul(v.y, s), host_mul(v.z, s));
}

fn host_div3(v: vec3<f32>, s: f32) -> vec3<f32> {
    return vec3<f32>(host_div(v.x, s), host_div(v.y, s), host_div(v.z, s));
}

// The host's `(a[0] * b[0] + a[1] * b[1]) + a[2] * b[2]`.
fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return host_add(host_add(host_mul(a.x, b.x), host_mul(a.y, b.y)), host_mul(a.z, b.z));
}

fn host_cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        host_sub(host_mul(a.y, b.z), host_mul(a.z, b.y)),
        host_sub(host_mul(a.z, b.x), host_mul(a.x, b.z)),
        host_sub(host_mul(a.x, b.y), host_mul(a.y, b.x)),
    );
}

fn host_length(v: vec3<f32>) -> f32 {
    return host_sqrt(host_dot(v, v));
}

// The host's `a + (b - a) * t`.
fn host_lerp(a: f32, b: f32, t: f32) -> f32 {
    return host_add(a, host_mul(host_sub(b, a), t));
}

// The host's clamped smoothstep, `t * t * (3 - 2 t)`.
fn host_smoothstep(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return host_mul(host_mul(c, c), host_sub(3.0, host_mul(2.0, c)));
}
"#;
