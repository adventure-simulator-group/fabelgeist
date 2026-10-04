use super::*;
use crate::prelude::*;

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
    let kernel = Kernel::new(&context, ShaderSource::from(source))?;
    let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
    let outputs = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&vec![0f32; inputs.len() * expressions.len()]),
        definition.clone(),
    )?;
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (inputs.len() as u32).into());
    parameters.insert(ZERO_FIELD.into(), (0u32).into());
    parameters.insert("pad0".into(), (0u32).into());
    parameters.insert("pad1".into(), (0u32).into());
    parameters.insert(
        "inputs".into(),
        (Buffer::from_upload(&context, BufferUpload::from_elements(inputs), definition)?).into(),
    );
    parameters.insert("outputs".into(), (outputs.clone()).into());
    let mut batch = KernelBatch::new(&context);
    batch.dispatch_items(&kernel, &parameters, (inputs.len() as u32).into())?;
    batch.submit();
    Ok(outputs.read(&context).await?)
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
    assert!(Kernel::new(&context, ShaderSource::from(source)).is_err());
    Ok(())
}

#[tokio::test]
async fn product_expansions_preserve_low_parts_across_binary_scales() {
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    // Split boundaries, halfway patterns, and the largest fractional word.
    let fractions = [
        0x3f00_0000,
        0x3f00_0fff,
        0x3f00_1000,
        0x3f40_0800,
        0x3f7f_ffff,
    ];
    for a_exponent in [-120, -60, 0, 60, 120] {
        for b_exponent in [-120, -60, 0, 60, 120] {
            for a_bits in fractions {
                for b_bits in fractions {
                    for sign in [-1.0f32, 1.0] {
                        let a_fraction = sign * f32::from_bits(a_bits);
                        let b_fraction = f32::from_bits(b_bits);
                        let product = f64::from(a_fraction) * f64::from(b_fraction);
                        let high = product as f32;
                        let low = (product - f64::from(high)) as f32;
                        inputs.push([
                            a_fraction * 2f32.powi(a_exponent),
                            b_fraction * 2f32.powi(b_exponent),
                            0.0,
                            0.0,
                        ]);
                        expected.push([high, low, (a_exponent + b_exponent) as f32]);
                    }
                }
            }
        }
    }
    let mut words = Words(0xaeb2_0645_88ac_71d3);
    for _ in 0..16384 {
        let a = words.normal(-1..=-1);
        let b = words.normal(-1..=-1);
        let product = f64::from(a) * f64::from(b);
        let high = product as f32;
        let low = (product - f64::from(high)) as f32;
        inputs.push([a, b, 0.0, 0.0]);
        expected.push([high, low, 0.0]);
    }
    let device = run(
        &inputs,
        &[
            "product_expansion(a, b).high",
            "product_expansion(a, b).low",
            "f32(product_expansion(a, b).exponent)",
        ],
    )
    .await
    .unwrap();
    let (actual, remainder) = device.as_chunks::<3>();
    assert!(remainder.is_empty());
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual[0].to_bits(), expected[0].to_bits());
        assert_eq!(actual[1], expected[1]);
        assert_eq!(actual[2], expected[2]);
    }
}

#[tokio::test]
async fn rounded_functions_preserve_signed_zero_and_extreme_normal_inputs() {
    let zeros = [[-0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]];
    let device = run(&zeros, &["host_asin(a)", "host_sin(a)"]).await.unwrap();
    let (actual, remainder) = device.as_chunks::<2>();
    assert!(remainder.is_empty());
    assert_eq!(actual.len(), zeros.len());
    for (input, actual) in zeros.into_iter().zip(actual) {
        assert_eq!(actual[0].to_bits(), input[0].to_bits());
        assert_eq!(actual[1].to_bits(), input[0].to_bits());
    }
    let inputs = [
        [-0.0, 1.0, f32::MAX, 0.0],
        [0.0, 1.0, f32::MIN_POSITIVE, 0.0],
        [f32::MAX, 3.0, f32::MAX, 0.0],
        [f32::MIN_POSITIVE, 0.5, f32::MIN_POSITIVE, 0.0],
    ];
    let device = run(&inputs, &["host_div(a,b)", "host_sqrt(c)"])
        .await
        .unwrap();
    let (actual, remainder) = device.as_chunks::<2>();
    assert!(remainder.is_empty());
    assert_eq!(actual.len(), inputs.len());
    for (input, actual) in inputs.into_iter().zip(actual) {
        assert_eq!(actual[0].to_bits(), (input[0] / input[1]).to_bits());
        assert_eq!(actual[1].to_bits(), input[2].sqrt().to_bits());
    }
}

#[tokio::test]
async fn double_float_operations_keep_cancellation_and_low_order_bits() {
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    let mut words = Words(0xbf53_46d8_2a09_c317);
    for scale in [-60, 0, 60] {
        for separation in [-20, 0, 20] {
            for _ in 0..256 {
                let a = words.normal(-1..=-1) * 2f32.powi(scale);
                let b = words.normal(-1..=-1) * 2f32.powi(scale + separation);
                for b in [b, -a, -f32::from_bits(a.to_bits() - 1)] {
                    // At most 45 significant bits: this sum is exact in f64.
                    let sum = f64::from(a) + f64::from(b);
                    let high = sum as f32;
                    let low = (sum - f64::from(high)) as f32;
                    inputs.push([a, b, 0.0, 0.0]);
                    expected.push([high, low]);
                }
            }
        }
    }
    let device = run(
        &inputs,
        &[
            "df_two_sum(a, b).high",
            "df_two_sum(a, b).low",
            "df_quick_two_sum(a, b).high",
            "df_quick_two_sum(a, b).low",
            "df_add(df_two_sum(a, b), df_from_scalar(0.0)).high",
            "df_add(df_two_sum(a, b), df_from_scalar(0.0)).low",
            "df_add(df_from_scalar(0.0), df_two_sum(a, b)).high",
            "df_add(df_from_scalar(0.0), df_two_sum(a, b)).low",
            "df_mul(df_two_sum(a, b), df_from_scalar(8.0)).high",
            "df_mul(df_two_sum(a, b), df_from_scalar(8.0)).low",
            "df_mul(df_from_scalar(8.0), df_two_sum(a, b)).high",
            "df_mul(df_from_scalar(8.0), df_two_sum(a, b)).low",
        ],
    )
    .await
    .unwrap();
    let (actual, remainder) = device.as_chunks::<12>();
    assert!(remainder.is_empty());
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        for pair in actual[..8].as_chunks::<2>().0 {
            assert_eq!(pair[0].to_bits(), expected[0].to_bits());
            assert_eq!(pair[1], expected[1]);
        }
        for pair in actual[8..].as_chunks::<2>().0 {
            assert_eq!(pair[0].to_bits(), (expected[0] * 8.0).to_bits());
            assert_eq!(pair[1], expected[1] * 8.0);
        }
    }
}

#[tokio::test]
async fn vectors_and_boolean_flags_cannot_replace_expansion_types() {
    let context = WgpuContext::new().await.unwrap();
    let prefix = wgsl().to_owned() + &zero_hook("0u");
    let valid = prefix.clone()
        + "@compute @workgroup_size(1) fn main() {\n\
           let value = df_add(df_from_scalar(1.0), df_from_scalar(2.0));\n\
           let cosine = df_sin_or_cos(df_quarter_turns(0.0), TRIGONOMETRIC_COSINE);\n}\n";
    Kernel::new(&context, ShaderSource::from(valid)).unwrap();
    for body in [
        "let value = df_add(vec2<f32>(1.0, 0.0), df_from_scalar(2.0));",
        "let value = df_add(df_from_scalar(1.0), 2.0);",
        "let value = df_mul(df_from_scalar(1.0), 2.0);",
        "let value = df_sin_or_cos(vec3<f32>(0.0), TRIGONOMETRIC_COSINE);",
        "let value = df_sin_or_cos(df_quarter_turns(0.0), true);",
    ] {
        let source = prefix.clone() + "@compute @workgroup_size(1) fn main() {\n" + body + "\n}";
        assert!(Kernel::new(&context, ShaderSource::from(source)).is_err());
    }
}
