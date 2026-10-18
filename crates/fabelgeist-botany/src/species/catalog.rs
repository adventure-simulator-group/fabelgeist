//! The paper's authored Table 4 and its checked row identities.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table4Row(usize);

impl Table4Row {
    pub const A: Self = Self(0);
    pub const B: Self = Self(1);
    pub const C: Self = Self(2);
    pub const D: Self = Self(3);
    pub const E: Self = Self(4);
    pub const F: Self = Self(5);
    pub const G: Self = Self(6);
    pub const H: Self = Self(7);
    pub const I: Self = Self(8);
    pub const J: Self = Self(9);
    pub const K: Self = Self(10);
    pub const L: Self = Self(11);
    pub const M: Self = Self(12);
    pub const N: Self = Self(13);
    pub const O: Self = Self(14);
    pub const P: Self = Self(15);
}
impl TryFrom<char> for Table4Row {
    type Error = crate::error::BotanyError;
    fn try_from(letter: char) -> crate::error::Result<Self> {
        TABLE4
            .iter()
            .position(|(row, _)| *row == letter.to_ascii_lowercase())
            .map(Self)
            .ok_or(crate::error::BotanyError::UnknownTableRow { letter })
    }
}
/// One row of the paper's Table 4.
struct Row {
    max_age: f32,
    root_vigor_max: f32,
    growth_rate: f32,
    lambda: (f32, Option<f32>),
    d: (f32, Option<f32>),
    flowering_age: f32,
    alpha: f32,
    w2: f32,
    g2: f32,
    phi_cm: f32,
    beta: f32,
}

/// Table 4, rows a..p.
///
/// The printed table's last five columns are headed `alpha, w2, g1, phi,
/// beta`. Read literally they do not all make sense, so:
///
/// * `alpha` is read as `cos(alpha)`. Its values run from -0.27 to 1, and
///   rows j and k differ only in it (0.45 against -0.2) -- an upright and a
///   weeping form of one tree. As a cosine that is exactly the difference.
/// * The third-from-last column carries the negative values the text gives
///   to `g2`, the signed tropism strength, so it is read as `g2`; `g1` keeps
///   its default.
/// * `phi` is read in centimetres -- five metres is not a twig.
const TABLE4: [(char, Row); 16] = {
    #[allow(clippy::too_many_arguments)]
    const fn row(
        max_age: f32,
        root_vigor_max: f32,
        growth_rate: f32,
        lambda: (f32, Option<f32>),
        d: (f32, Option<f32>),
        flowering_age: f32,
        alpha: f32,
        w2: f32,
        g2: f32,
        phi_cm: f32,
        beta: f32,
    ) -> Row {
        Row {
            max_age,
            root_vigor_max,
            growth_rate,
            lambda,
            d,
            flowering_age,
            alpha,
            w2,
            g2,
            phi_cm,
            beta,
        }
    }
    [
        (
            'a',
            row(
                20.0,
                42.0,
                0.19,
                (0.62, None),
                (0.25, None),
                0.0,
                0.52,
                0.63,
                -0.38,
                0.57,
                0.47,
            ),
        ),
        (
            'b',
            row(
                200.0,
                78.0,
                0.30,
                (0.84, None),
                (1.0, None),
                0.0,
                0.52,
                0.63,
                -1.2,
                1.00,
                0.79,
            ),
        ),
        (
            'c',
            row(
                80.0,
                11.0,
                0.80,
                (1.0, None),
                (1.0, None),
                0.0,
                0.9,
                0.5,
                1.0,
                5.00,
                1.95,
            ),
        ),
        (
            'd',
            row(
                16.0,
                1.7,
                0.23,
                (0.44, None),
                (0.31, None),
                0.0,
                1.0,
                1.0,
                1.0,
                5.00,
                3.00,
            ),
        ),
        (
            'e',
            row(
                430.0,
                600.0,
                0.15,
                (1.0, Some(0.5)),
                (1.0, Some(0.33)),
                58.0,
                0.52,
                0.63,
                0.56,
                3.00,
                1.23,
            ),
        ),
        (
            'f',
            row(
                550.0,
                450.0,
                0.20,
                (0.76, None),
                (0.82, None),
                0.0,
                0.17,
                0.5,
                0.47,
                1.20,
                1.90,
            ),
        ),
        (
            'g',
            row(
                550.0,
                700.0,
                0.20,
                (0.9, Some(0.5)),
                (0.93, Some(0.74)),
                55.0,
                0.17,
                0.5,
                0.47,
                1.38,
                0.94,
            ),
        ),
        (
            'h',
            row(
                500.0,
                570.0,
                0.24,
                (1.0, Some(0.5)),
                (1.0, Some(0.5)),
                55.0,
                0.5,
                0.27,
                -0.66,
                1.38,
                1.29,
            ),
        ),
        (
            'i',
            row(
                950.0,
                900.0,
                0.12,
                (0.87, Some(0.34)),
                (0.93, Some(0.55)),
                57.0,
                0.66,
                0.14,
                0.2,
                1.41,
                1.29,
            ),
        ),
        (
            'j',
            row(
                950.0,
                600.0,
                0.14,
                (1.0, Some(0.5)),
                (1.0, Some(0.51)),
                57.0,
                0.45,
                0.63,
                -0.9,
                0.82,
                0.93,
            ),
        ),
        (
            'k',
            row(
                950.0,
                600.0,
                0.14,
                (1.0, Some(0.5)),
                (1.0, Some(0.51)),
                57.0,
                -0.2,
                0.63,
                -0.9,
                0.82,
                0.93,
            ),
        ),
        (
            'l',
            row(
                1000.0,
                815.0,
                0.19,
                (0.92, Some(0.7)),
                (0.59, Some(0.56)),
                80.0,
                -0.19,
                0.72,
                -0.21,
                5.00,
                1.54,
            ),
        ),
        (
            'm',
            row(
                130.0,
                400.0,
                0.21,
                (0.88, Some(0.43)),
                (0.9, Some(0.7)),
                66.0,
                0.85,
                0.55,
                0.9,
                1.42,
                1.11,
            ),
        ),
        (
            'n',
            row(
                52.0,
                200.0,
                0.55,
                (0.96, Some(0.43)),
                (0.48, Some(0.7)),
                0.0,
                -0.27,
                0.43,
                0.73,
                1.50,
                2.50,
            ),
        ),
        (
            'o',
            row(
                300.0,
                600.0,
                0.20,
                (0.8, None),
                (0.86, None),
                0.0,
                -0.19,
                0.81,
                1.0,
                1.00,
                1.60,
            ),
        ),
        (
            'p',
            row(
                450.0,
                450.0,
                0.15,
                (1.0, Some(0.5)),
                (0.66, Some(0.33)),
                135.0,
                0.52,
                0.32,
                0.42,
                1.50,
                1.06,
            ),
        ),
    ]
};

impl PlantType {
    /// The structural parameters of Table 4, row `letter` (`'a'..='p'`),
    /// with default plasticity.
    ///
    /// A flowering age of zero in the table means the architecture does not
    /// change at maturity; the plant still has to flower to seed, so it is
    /// given one at a fifth of its lifespan.
    pub fn table4(row: Table4Row) -> Self {
        let (letter, row) = &TABLE4[row.0];
        let small = row.root_vigor_max < 50.0;
        Self {
            name: format!("table4-{letter}"),
            max_age: row.max_age,
            root_vigor_max: row.root_vigor_max,
            growth_rate: row.growth_rate,
            apical_control: row.lambda.0,
            apical_control_mature: row.lambda.1.unwrap_or(row.lambda.0),
            determinacy: row.d.0,
            determinacy_mature: row.d.1.unwrap_or(row.d.0),
            flowering_age: if row.flowering_age > 0.0 {
                row.flowering_age
            } else {
                row.max_age * 0.2
            },
            tropism_angle: row.alpha.clamp(-1.0, 1.0).acos(),
            tropism_weight: row.w2,
            tropism_strength: row.g2 * 0.25,
            tip_diameter: row.phi_cm * 0.01,
            length_scale: row.beta,
            segment_length_max: if small { 0.25 } else { 0.5 },
            senescence_years: (row.max_age * 0.1).max(3.0),
            leaf_size: if small { 0.15 } else { 0.35 },
            max_height: if small { 4.0 } else { 30.0 },
            ..Self::default()
        }
    }

    /// Every row of Table 4.
    pub fn table4_all() -> Vec<Self> {
        TABLE4
            .iter()
            .enumerate()
            .map(|(index, _)| Self::table4(Table4Row(index)))
            .collect()
    }
}
