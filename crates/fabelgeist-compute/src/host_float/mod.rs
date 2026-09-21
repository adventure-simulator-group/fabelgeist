//! WGSL float arithmetic that rounds as the host's does.
//!
//! A device compiler is free to fuse a product into the sum that consumes
//! it, to reassociate sums -- `(y + h) - (y - h)` becomes `2 h` -- to divide
//! and take square roots a few units in the last place off, and to evaluate
//! `sin` or `pow` more loosely still. Mostly that is invisible. But where a
//! kernel must reproduce a host computation bit for bit -- a finite
//! difference, which amplifies every rounding difference; a weld that merges
//! vertices agreeing to their last bits; a normal whose sign picks the side
//! of a seam -- it must round exactly as the host does. [`wgsl`] is a library
//! of functions that do, which a kernel prepends to its source:
//!
//! * `host_fence(x)`, `host_fence3(v)`: `x` materialised exactly as rounded.
//! * `host_add`, `host_sub`, `host_mul`: one correctly rounded operation,
//!   its operands and result fenced, so that it neither fuses nor
//!   reassociates -- `host_add(a * b, c)` is the host's `a * b + c`.
//! * `host_div`, `host_sqrt`: the correctly rounded quotient and root.
//! * `host_add3`, `host_sub3`, `host_scale3`, `host_div3`, `host_dot`,
//!   `host_cross`, `host_length`: `[f32; 3]` arithmetic in the host's order,
//!   `host_dot` summing `(x + y) + z`.
//! * `host_lerp(a, b, t)` is `a + (b - a) * t`, and `host_smoothstep(t)` the
//!   clamped `t * t * (3 - 2 t)`.
//! * `host_sin`, `host_cos`, `host_sin_cos`, `host_asin`, `host_log`,
//!   `host_exp`, `host_pow`: evaluated in double-float arithmetic and
//!   rounded once, which is correctly rounded but for the rare argument
//!   whose value lies within about `2^-44` of a rounding boundary. A host
//!   whose own `sinf` or `powf` is not correctly rounded differs from them
//!   there instead. `host_pow` takes a positive base, or zero.
//! * `df_*`: the double-float arithmetic beneath them -- `vec2<f32>` values
//!   `x + y` -- for a kernel that needs more than a float's precision.
//!
//! # The fence
//!
//! Every function above passes floats through `host_fence`, which ORs their
//! bits with `host_zero()`: a `u32` zero that the including kernel defines,
//! and that must be read at run time -- a uniform field, or a buffer word the
//! host leaves at zero. Float optimisations act on float operations; a value
//! that went through an integer operation with an operand unknown at compile
//! time is opaque to them, and must be materialised as the float it was.
//! [`zero_hook`] writes the definition, [`PARAMS_ZERO_HOOK`] the one reading
//! a uniform `params.zero: u32`, the convention: set it to `0u` with
//! `parameters.insert(ZERO_FIELD, 0u32)`.
//!
//! Defining a function, rather than assigning a module-scope `var<private>`
//! at the top of each entry point, is deliberate: a kernel that forgets the
//! function does not compile, whereas an entry point that forgets the
//! assignment reads the variable's zero initialiser, a constant the compiler
//! folds away -- silently removing every fence. Reading a uniform in every
//! fence costs nothing measurable; the compiler loads it once.
//!
//! # Limits
//!
//! Residuals are taken with `fma`, which must be fused: the tests below
//! check that it is on the device they run on. Devices flush subnormals to
//! zero, so a result or residual beneath the normal range may still differ
//! from the host's. The quarter-turn reduction of `host_sin` and `host_cos`
//! is exact for arguments up to a few thousand radians.

mod arithmetic;
mod double_float;

#[cfg(test)]
mod tests;

use std::sync::OnceLock;

/// The name of the uniform field [`PARAMS_ZERO_HOOK`] reads.
pub const ZERO_FIELD: &str = "zero";

/// A `host_zero` reading the kernel's uniform `params.zero`, a `u32` the
/// host sets to zero.
pub const PARAMS_ZERO_HOOK: &str = "fn host_zero() -> u32 {\n    return params.zero;\n}\n";

/// The library's WGSL. The kernel including it defines `host_zero`; see
/// the module documentation.
pub fn wgsl() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| format!("{}{}", arithmetic::ARITHMETIC, double_float::source()))
}

/// A `host_zero` returning `expression`, a `u32` zero read at run time, such
/// as `bitcast<u32>(design[ZERO])` for a buffer word the host leaves at
/// `0.0`.
pub fn zero_hook(expression: &str) -> String {
    format!("fn host_zero() -> u32 {{\n    return {expression};\n}}\n")
}
