use super::*;
use crate::prelude::*;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

/// Samples per test: enough to meet the rare hard case of every operation.
const SAMPLES: u32 = 1 << 18;

/// Deterministic random words, xorshift64 so that a failure is reproducible
/// from the test name alone.
struct Words(u64);

impl Words {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Uniform in `[low, high)`.
    fn uniform(&mut self, low: f32, high: f32) -> f32 {
        let unit = (self.next() >> 40) as f32 / (1u64 << 24) as f32;
        low + (high - low) * unit
    }

    /// A normal float of either sign whose exponent lies in `exponents`,
    /// every significand equally likely.
    fn normal(&mut self, exponents: std::ops::RangeInclusive<i32>) -> f32 {
        let word = self.next();
        let span = (exponents.end() - exponents.start() + 1) as u64;
        let exponent = exponents.start() + ((word >> 32) % span) as i32;
        let bits = ((word & 1) << 31) as u32
            | ((exponent + 127) as u32) << 23
            | (word >> 8) as u32 & 0x007f_ffff;
        f32::from_bits(bits)
    }
}

/// Runs `expressions` of `a`, `b`, `c` and `d` -- the four components of each
/// input -- on the device, returning each input's results in order.
async fn run(inputs: &[[f32; 4]], expressions: &[&str]) -> Result<Vec<f32>> {
    let context = WgpuContext::new().await?;
    let stores: String = expressions
        .iter()
        .enumerate()
        .map(|(k, expression)| {
            format!(
                "    outputs[i * {}u + {k}u] = {expression};\n",
                expressions.len()
            )
        })
        .collect();
    let source = format!(
        r#"
@group(0) @binding(0) var<storage, read> inputs: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;
struct Params {{ count: u32, zero: u32, pad0: u32, pad1: u32 }};
@group(0) @binding(2) var<uniform> params: Params;
{PARAMS_ZERO_HOOK}
{library}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.count) {{
        return;
    }}
    let a = inputs[i].x;
    let b = inputs[i].y;
    let c = inputs[i].z;
    let d = inputs[i].w;
{stores}}}
"#,
        library = wgsl(),
    );
    let kernel = Kernel::new(&context, source)?;
    let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
    let outputs = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&vec![0f32; inputs.len() * expressions.len()]),
        definition.clone(),
    )?;
    let mut parameters = PassParameters::new();
    parameters.insert("count", inputs.len() as u32);
    parameters.insert(ZERO_FIELD, 0u32);
    parameters.insert("pad0", 0u32);
    parameters.insert("pad1", 0u32);
    parameters.insert(
        "inputs",
        Buffer::from_upload(&context, BufferUpload::from_elements(inputs), definition)?,
    );
    parameters.insert("outputs", outputs.clone());
    let mut batch = KernelBatch::new(&context);
    batch.dispatch_items(&kernel, &parameters, inputs.len() as u32)?;
    batch.submit();
    outputs.read(&context).await
}

/// How many results differ from `expected` in any bit, per expression.
fn mismatches(
    inputs: &[[f32; 4]],
    device: &[f32],
    expected: impl Fn([f32; 4]) -> Vec<f32>,
) -> Vec<usize> {
    let mut counts = Vec::new();
    for (input, results) in inputs
        .iter()
        .zip(device.chunks(device.len() / inputs.len()))
    {
        let expected = expected(*input);
        counts.resize(expected.len(), 0);
        for (k, (got, want)) in results.iter().zip(expected).enumerate() {
            // NaNs of any payload agree.
            counts[k] +=
                usize::from(got.to_bits() != want.to_bits() && !(got.is_nan() && want.is_nan()));
        }
    }
    counts
}

#[tokio::test]
async fn quotients_and_roots_are_correctly_rounded() -> Result<()> {
    let mut words = Words(0x2545_f491_4f6c_dd1d);
    let inputs: Vec<[f32; 4]> = (0..SAMPLES)
        .map(|_| {
            let b = words.normal(-40..=40);
            // A root just beside an exact square: the hard case of a root.
            let root = words.normal(-60..=60).abs();
            let near_square = f32::from_bits(
                (root * root)
                    .to_bits()
                    .wrapping_add(words.next() as u32 % 3)
                    .wrapping_sub(1),
            );
            [
                words.normal(-40..=40),
                b,
                words.normal(-126..=127).abs(),
                near_square,
            ]
        })
        .collect();
    let device = run(&inputs, &["host_div(a, b)", "host_sqrt(c)", "host_sqrt(d)"]).await?;
    let counts = mismatches(&inputs, &device, |[a, b, c, d]| {
        vec![a / b, c.sqrt(), d.sqrt()]
    });
    assert_eq!(counts, [0; 3], "mismatches out of {SAMPLES}");
    Ok(())
}

#[tokio::test]
async fn arithmetic_rounds_in_the_hosts_order() -> Result<()> {
    let mut words = Words(0x9e37_79b9_7f4a_7c15);
    let inputs: Vec<[f32; 4]> = (0..SAMPLES)
        .map(|_| {
            [(); 4].map(|_| words.uniform(-1.0, 1.0) * 10f32.powi((words.next() % 5) as i32 - 2))
        })
        .collect();
    let device = run(
        &inputs,
        &[
            // A product the compiler would fuse into the sum.
            "host_add(a * b, c)",
            "host_sub(host_mul(a, b), host_mul(c, d))",
            // A finite difference, which reassociation would cancel.
            "host_sub(host_add(c, 0.0001), host_sub(c, 0.0001))",
            "host_dot(vec3<f32>(a, b, c), vec3<f32>(d, c, b))",
            "host_cross(vec3<f32>(a, b, c), vec3<f32>(c, d, a)).y",
            "host_length(vec3<f32>(a, b, d))",
            "host_lerp(a, b, c)",
            "host_smoothstep(d)",
            "host_div3(host_scale3(host_add3(vec3<f32>(a), vec3<f32>(b)), c), d).z",
        ],
    )
    .await?;
    let counts = mismatches(&inputs, &device, |[a, b, c, d]| {
        let t = d.clamp(0.0, 1.0);
        vec![
            a * b + c,
            a * b - c * d,
            (c + 0.0001) - (c - 0.0001),
            (a * d + b * c) + c * b,
            c * c - a * a,
            ((a * a + b * b) + d * d).sqrt(),
            a + (b - a) * c,
            t * t * (3.0 - 2.0 * t),
            (a + b) * c / d,
        ]
    });
    assert_eq!(counts, [0; 9], "mismatches out of {SAMPLES}");
    Ok(())
}

/// Each function against the host's in double precision, rounded once: the
/// correctly rounded float. A handful of arguments whose value lies within
/// the double-float's error of a rounding boundary may differ.
#[tokio::test]
async fn transcendentals_are_correctly_rounded() -> Result<()> {
    /// The most results, per million, allowed to differ.
    const TOLERATED_PER_MILLION: usize = 20;
    let mut words = Words(0xd1b5_4a32_d192_ed03);
    let inputs: Vec<[f32; 4]> = (0..SAMPLES)
        .map(|_| {
            [
                words.uniform(-100.0, 100.0),
                words.uniform(-1.0, 1.0),
                words.uniform(0.01, 4.0),
                words.uniform(-8.0, 8.0),
            ]
        })
        .collect();
    let magnitudes: Vec<[f32; 4]> = (0..SAMPLES)
        .map(|_| {
            [
                words.normal(-126..=127).abs(),
                words.uniform(-87.0, 88.0),
                0.0,
                0.0,
            ]
        })
        .collect();
    let device = run(
        &inputs,
        &[
            "host_sin(a)",
            "host_cos(a)",
            "host_asin(b)",
            "host_pow(c, d)",
            "host_sin(b)",
        ],
    )
    .await?;
    let mut counts = mismatches(&inputs, &device, |input| {
        let [a, b, c, d] = input.map(f64::from);
        vec![
            a.sin() as f32,
            a.cos() as f32,
            b.asin() as f32,
            c.powf(d) as f32,
            b.sin() as f32,
        ]
    });
    let device = run(&magnitudes, &["host_log(a)", "host_exp(b)"]).await?;
    counts.extend(mismatches(&magnitudes, &device, |input| {
        let [a, b, ..] = input.map(f64::from);
        vec![a.ln() as f32, b.exp() as f32]
    }));
    let names = ["sin", "cos", "asin", "pow", "sin near zero", "log", "exp"];
    for (name, count) in names.iter().zip(&counts) {
        println!("{name}: {count} of {SAMPLES} differ");
    }
    let tolerated = SAMPLES as usize * TOLERATED_PER_MILLION / 1_000_000;
    assert!(
        counts.iter().all(|count| *count <= tolerated),
        "{counts:?} of {SAMPLES}"
    );
    Ok(())
}

/// A kernel that leaves `host_zero` undefined does not compile: the fence
/// cannot silently vanish.
#[tokio::test]
async fn the_zero_hook_is_required() -> Result<()> {
    let context = WgpuContext::new().await?;
    let source = format!(
        "{}\n@group(0) @binding(0) var<storage, read_write> values: array<f32>;\n\
         @compute @workgroup_size(1)\nfn main() {{ values[0] = host_add(values[0], 1.0); }}\n",
        wgsl()
    );
    assert!(Kernel::new(&context, source).is_err());
    Ok(())
}
