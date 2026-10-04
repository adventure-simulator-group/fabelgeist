//! The WGSL every solver kernel shares.
//!
//! A constraint kernel is mostly its own geometry -- what `C` is and what its
//! gradient is. Everything around that is the same for all of them, so it
//! lives here as source to paste in: the particle bindings, the compliance
//! arithmetic, and the position write-back.

/// Particle storage. Positions carry the inverse mass in `w`, so a constraint
/// solve reads one buffer rather than two.
///
/// An inverse mass of zero pins the particle: every correction is scaled by
/// it, so a pinned particle simply never moves, with no special case anywhere.
use fabelgeist_gpu::prelude::ShaderSource;
pub const PARTICLES: &str = r#"
// xyz = position, w = inverse mass
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;

fn particle_position(index: u32) -> vec3<f32> {
    return positions[index].xyz;
}

fn particle_inverse_mass(index: u32) -> f32 {
    return positions[index].w;
}

fn particle_move(index: u32, delta: vec3<f32>) {
    positions[index] = vec4<f32>(positions[index].xyz + delta, positions[index].w);
}
"#;

/// The XPBD update itself.
///
/// For a constraint `C` with compliance `alpha`, over a substep of `h`:
///
/// ```text
/// alpha_tilde = alpha / h^2
/// d_lambda    = (-C - alpha_tilde * lambda) / (sum_i w_i |grad_i C|^2 + alpha_tilde)
/// dx_i        = w_i * grad_i C * d_lambda
/// ```
///
/// The compliance term is what makes this XPBD rather than PBD: stiffness
/// stops depending on the iteration count and the timestep, so a fabric keeps
/// the same feel whatever the solver settings are. `alpha = 0` is the
/// infinitely stiff limit and reduces to plain PBD.
pub const SOLVE: &str = r#"
struct XpbdSolve {
    delta_lambda: f32,
    valid: bool,
};

// `gradient_norm_squared` is `sum_i w_i |grad_i C|^2` -- the weighted
// denominator, gathered by the caller because only it knows the arity.
fn xpbd_solve(
    value: f32,
    gradient_norm_squared: f32,
    lambda: f32,
    alpha_tilde: f32,
) -> XpbdSolve {
    let denominator = gradient_norm_squared + alpha_tilde;
    // Every particle pinned, or a gradient that vanished: there is no
    // correction to make, and dividing would produce one out of nothing.
    if (denominator < 1e-12) {
        return XpbdSolve(0.0, false);
    }
    return XpbdSolve((-value - alpha_tilde * lambda) / denominator, true);
}

fn xpbd_alpha_tilde(compliance: f32, substep: f32) -> f32 {
    return compliance / (substep * substep);
}
"#;

/// The uniform block every constraint kernel takes.
///
/// `first` and `count` bound the colour being solved: constraints are stored
/// sorted by colour, so a colour is a contiguous range and a dispatch is just
/// an offset and a length.
pub const PARAMS: &str = r#"
struct SolveParams {
    first: u32,
    count: u32,
    compliance: f32,
    substep: f32,
};
"#;

/// Integrate forces and predict where each particle wants to be.
///
/// The previous position is kept because XPBD reads velocity back out of the
/// position change at the end of the substep -- which is what makes every
/// positional correction, including a collision response, show up in the
/// velocity for free.
pub const PREDICT: &str = r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> velocities: array<vec4<f32>>;

struct PredictParams {
    gravity: vec4<f32>,
    substep: f32,
    damping: f32,
    count: u32,
    max_speed: f32,
};
@group(0) @binding(3) var<uniform> params: PredictParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }

    let current = positions[index];
    let inverse_mass = current.w;
    previous[index] = vec4<f32>(current.xyz, inverse_mass);

    if (inverse_mass == 0.0) {
        velocities[index] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return;
    }

    var velocity = velocities[index].xyz;
    velocity = velocity + params.gravity.xyz * params.substep;
    // Exponential drag rather than a per-step multiplier, so the amount of
    // damping does not change when the substep count does.
    velocity = velocity * exp(-params.damping * params.substep);

    // A speed cap is the difference between a solver that recovers from a bad
    // frame and one that throws the cloth off the screen. It only ever binds
    // when something has already gone wrong.
    let speed = length(velocity);
    if (speed > params.max_speed && speed > 0.0) {
        velocity = velocity * (params.max_speed / speed);
    }

    velocities[index] = vec4<f32>(velocity, 0.0);
    positions[index] = vec4<f32>(current.xyz + velocity * params.substep, inverse_mass);
}
"#;

/// Read velocity back out of the substep's total position change.
pub const FINALIZE: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> velocities: array<vec4<f32>>;

struct FinalizeParams {
    substep: f32,
    count: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(3) var<uniform> params: FinalizeParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    if (positions[index].w == 0.0) {
        velocities[index] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return;
    }
    let delta = positions[index].xyz - previous[index].xyz;
    velocities[index] = vec4<f32>(delta / params.substep, 0.0);
}
"#;

/// Distance constraints: the workhorse. Cloth stretch, seams and tethers are
/// all this kernel with different rest lengths and compliances.
pub const DISTANCE: &str = r#"
@group(0) @binding(1) var<storage, read_write> lambdas: array<f32>;
// Two particle indices per constraint.
@group(0) @binding(2) var<storage, read> particles: array<u32>;
@group(0) @binding(3) var<storage, read> rest_lengths: array<f32>;
@group(0) @binding(4) var<uniform> params: SolveParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local = global_id.x;
    if (local >= params.count) {
        return;
    }
    let constraint = params.first + local;

    let a = particles[constraint * 2u];
    let b = particles[constraint * 2u + 1u];
    let inverse_mass_a = particle_inverse_mass(a);
    let inverse_mass_b = particle_inverse_mass(b);

    let delta = particle_position(a) - particle_position(b);
    let distance = length(delta);
    // Two coincident particles have no direction to separate along. Leaving
    // them alone lets the next substep, when something else has nudged them
    // apart, do the work.
    if (distance < 1e-9) {
        return;
    }

    let normal = delta / distance;
    let value = distance - rest_lengths[constraint];
    // |grad| is 1 for both particles, so the weighted denominator is just the
    // sum of the inverse masses.
    let alpha_tilde = xpbd_alpha_tilde(params.compliance, params.substep);
    let solved = xpbd_solve(value, inverse_mass_a + inverse_mass_b, lambdas[constraint], alpha_tilde);
    if (!solved.valid) {
        return;
    }

    lambdas[constraint] = lambdas[constraint] + solved.delta_lambda;
    particle_move(a, normal * (inverse_mass_a * solved.delta_lambda));
    particle_move(b, normal * (-inverse_mass_b * solved.delta_lambda));
}
"#;

/// Spring constraints with viscous damping and compression/rebound bump stops.
pub const SPRING: &str = r#"
@group(0) @binding(1) var<storage, read_write> lambdas: array<f32>;
// Two particle indices per constraint.
@group(0) @binding(2) var<storage, read> particles: array<u32>;
@group(0) @binding(3) var<storage, read> rest_lengths: array<f32>;
// xy = travel ratios (min_ratio, max_ratio), z = damping, w = bump factor
@group(0) @binding(4) var<storage, read> spring_params: array<vec4<f32>>;
@group(0) @binding(5) var<uniform> params: SolveParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local = global_id.x;
    if (local >= params.count) {
        return;
    }
    let constraint = params.first + local;

    let a = particles[constraint * 2u];
    let b = particles[constraint * 2u + 1u];
    let inverse_mass_a = particle_inverse_mass(a);
    let inverse_mass_b = particle_inverse_mass(b);

    let delta = particle_position(a) - particle_position(b);
    let distance = length(delta);
    if (distance < 1e-9) {
        return;
    }

    let normal = delta / distance;
    let rest = rest_lengths[constraint];
    let sparams = spring_params[constraint];
    let min_limit = rest * sparams.x;
    let max_limit = rest * sparams.y;
    let bump_factor = max(sparams.w, 1.0);

    var compliance = params.compliance;
    if (distance < min_limit || distance > max_limit) {
        compliance = compliance / bump_factor;
    }

    let value = distance - rest;
    let alpha_tilde = xpbd_alpha_tilde(compliance, params.substep);
    let solved = xpbd_solve(value, inverse_mass_a + inverse_mass_b, lambdas[constraint], alpha_tilde);
    if (!solved.valid) {
        return;
    }

    lambdas[constraint] = lambdas[constraint] + solved.delta_lambda;
    particle_move(a, normal * (inverse_mass_a * solved.delta_lambda));
    particle_move(b, normal * (-inverse_mass_b * solved.delta_lambda));
}
"#;

/// Zero the multipliers. XPBD accumulates `lambda` across the iterations of
/// one substep and resets it at the start of the next -- that reset is what
/// makes the compliance behave like a real stiffness rather than drifting.
pub const CLEAR_LAMBDAS: &str = r#"
@group(0) @binding(0) var<storage, read_write> lambdas: array<f32>;

struct ClearParams {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(1) var<uniform> params: ClearParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    lambdas[index] = 0.0;
}
"#;

/// Assemble a constraint kernel: the shared preamble, then the body.
///
/// The body declares its own bindings from 1 upwards -- binding 0 is always
/// the particle buffer -- and its own `main`.
pub fn constraint_kernel(body: &str) -> ShaderSource {
    ShaderSource::from(format!("{PARTICLES}\n{SOLVE}\n{PARAMS}\n{body}"))
}
