//! Closed polynomial definitions and checked WGSL coefficient encoding.
//!
//! A series owns its symbol, truncation, and coefficient law. Its iterator
//! admits only the terms of that series, so code generation cannot pair a
//! sine symbol with an exponential's coefficients. Primitive floating-point
//! admission is confined to literal conversions; internal generation keeps
//! terms and coefficients nominal until their WGSL serialization.

use std::fmt::{self, Write};

/// A quarter turn expanded from a 100-digit pi. Decimal spelling must retain
/// every binary32 word, including the small residuals used in reduction.
const QUARTER_TURN: [f32; 4] = [
    f32::from_bits(0x3fc9_0fdb),
    f32::from_bits(0xb33b_bd2e),
    f32::from_bits(0xa6f7_2ced),
    f32::from_bits(0x194c_5170),
];
/// The natural logarithm of two, expanded in the same way.
const LN_2: [f32; 3] = [
    f32::from_bits(0x3f31_7218),
    f32::from_bits(0xb102_e308),
    f32::from_bits(0xa4ca_86c4),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PolynomialSeries {
    Sine,
    Cosine,
    Atanh,
    Exponential,
}

impl fmt::Display for PolynomialSeries {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Sine => "df_sine_series",
            Self::Cosine => "df_cosine_series",
            Self::Atanh => "df_atanh_series",
            Self::Exponential => "df_exp_series",
        })
    }
}

/// A term admitted by the closed series iterator, in ascending power order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeriesTerm {
    series: PolynomialSeries,
    ordinal: u32,
}

struct PolynomialTerms {
    series: PolynomialSeries,
    front: u32,
    back: u32,
}

impl From<PolynomialSeries> for PolynomialTerms {
    fn from(series: PolynomialSeries) -> Self {
        let back = match series {
            // Over an eighth turn, the next sine term is below 2^-64 of
            // the first; cosine uses the corresponding even-power series.
            PolynomialSeries::Sine => 9,
            PolynomialSeries::Cosine => 10,
            // t = (m - 1)/(m + 1) is bounded by
            // (sqrt(2) - 1)/(sqrt(2) + 1) in logarithm reduction.
            PolynomialSeries::Atanh => 14,
            // The exponential's reduced argument spans half a doubling.
            PolynomialSeries::Exponential => 17,
        };
        Self {
            series,
            front: 0,
            back,
        }
    }
}

impl Iterator for PolynomialTerms {
    type Item = SeriesTerm;

    fn next(&mut self) -> Option<SeriesTerm> {
        if self.front == self.back {
            return None;
        }
        let ordinal = self.front;
        self.front += 1;
        Some(SeriesTerm {
            series: self.series,
            ordinal,
        })
    }
}

impl DoubleEndedIterator for PolynomialTerms {
    fn next_back(&mut self) -> Option<SeriesTerm> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        Some(SeriesTerm {
            series: self.series,
            ordinal: self.back,
        })
    }
}

#[derive(Clone, Copy)]
enum CoefficientSign {
    Positive,
    Negative,
}

impl From<SeriesTerm> for CoefficientSign {
    fn from(term: SeriesTerm) -> Self {
        if term.ordinal.is_multiple_of(2) {
            Self::Positive
        } else {
            Self::Negative
        }
    }
}

struct FactorialOrder(u32);

impl FactorialOrder {
    fn reciprocal(self, sign: CoefficientSign) -> SeriesCoefficient {
        // Multiplication order is part of coefficient reproducibility.
        let factorial: f64 = (1..=self.0).map(f64::from).product();
        let numerator = match sign {
            CoefficientSign::Positive => 1.0,
            CoefficientSign::Negative => -1.0,
        };
        SeriesCoefficient(numerator / factorial)
    }
}

/// A coefficient of one of the admitted finite polynomial terms.
struct SeriesCoefficient(f64);

impl From<SeriesTerm> for SeriesCoefficient {
    fn from(term: SeriesTerm) -> Self {
        match term.series {
            PolynomialSeries::Sine => {
                FactorialOrder(2 * term.ordinal + 1).reciprocal(CoefficientSign::from(term))
            }
            PolynomialSeries::Cosine => {
                FactorialOrder(2 * term.ordinal).reciprocal(CoefficientSign::from(term))
            }
            PolynomialSeries::Atanh => Self(2.0 / f64::from(2 * term.ordinal + 1)),
            PolynomialSeries::Exponential => {
                FactorialOrder(term.ordinal).reciprocal(CoefficientSign::Positive)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum LiteralAdmissionError {
    NonFinite { value: f64 },
    LeadingOverflow { value: f64 },
    Underflow { value: f64 },
}

impl fmt::Display for LiteralAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { value } => write!(formatter, "nonfinite WGSL literal {value}"),
            Self::LeadingOverflow { value } => {
                write!(formatter, "double-float leading word overflows for {value}")
            }
            Self::Underflow { value } => {
                write!(
                    formatter,
                    "double-float literal rounds a nonzero {value} to zero"
                )
            }
        }
    }
}

impl std::error::Error for LiteralAdmissionError {}

/// A finite binary32 word serialized with WGSL's direct-to-float suffix.
struct WgslFloatLiteral(f32);

impl TryFrom<f32> for WgslFloatLiteral {
    type Error = LiteralAdmissionError;

    fn try_from(value: f32) -> Result<Self, LiteralAdmissionError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(LiteralAdmissionError::NonFinite {
                value: f64::from(value),
            })
        }
    }
}

impl fmt::Display for WgslFloatLiteral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:e}f", self.0)
    }
}

/// A leading word and its nearest residual, admitted together for WGSL.
struct DoubleFloatLiteral {
    high: WgslFloatLiteral,
    low: WgslFloatLiteral,
}

impl TryFrom<f64> for DoubleFloatLiteral {
    type Error = LiteralAdmissionError;

    fn try_from(value: f64) -> Result<Self, LiteralAdmissionError> {
        if !value.is_finite() {
            return Err(LiteralAdmissionError::NonFinite { value });
        }
        let high = value as f32;
        if !high.is_finite() {
            return Err(LiteralAdmissionError::LeadingOverflow { value });
        }
        let low = (value - f64::from(high)) as f32;
        if value != 0.0 && high == 0.0 && low == 0.0 {
            return Err(LiteralAdmissionError::Underflow { value });
        }
        Ok(Self {
            high: WgslFloatLiteral::try_from(high)?,
            low: WgslFloatLiteral::try_from(low)?,
        })
    }
}

impl From<SeriesCoefficient> for DoubleFloatLiteral {
    fn from(coefficient: SeriesCoefficient) -> Self {
        Self::try_from(coefficient.0)
            .expect("closed series coefficients fit two finite float words")
    }
}

impl fmt::Display for DoubleFloatLiteral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "DoubleFloat({}, {})", self.high, self.low)
    }
}

/// Owns assembly of the operations, admitted constants, series, and functions.
pub(in crate::host_float) struct DoubleFloatLibrary {
    source: String,
}

impl DoubleFloatLibrary {
    pub(in crate::host_float) fn new() -> Self {
        let mut library = Self {
            source: String::from(super::OPERATIONS),
        };
        for (k, part) in QUARTER_TURN.into_iter().enumerate() {
            let literal = WgslFloatLiteral::try_from(part).expect("finite quarter-turn word");
            writeln!(
                library.source,
                "const DF_QUARTER_TURN_{k}: f32 = {literal};"
            )
            .expect("writing to a string");
        }
        for (k, part) in LN_2.into_iter().enumerate() {
            let literal = WgslFloatLiteral::try_from(part).expect("finite logarithm word");
            writeln!(library.source, "const DF_LN_2_{k}: f32 = {literal};")
                .expect("writing to a string");
        }
        let two_over_pi = WgslFloatLiteral::try_from(std::f32::consts::FRAC_2_PI)
            .expect("finite reciprocal quarter turn");
        let log2_e =
            WgslFloatLiteral::try_from(std::f32::consts::LOG2_E).expect("finite base conversion");
        writeln!(
            library.source,
            "const DF_TWO_OVER_PI: f32 = {two_over_pi};\nconst DF_LOG2_E: f32 = {log2_e};"
        )
        .expect("writing to a string");
        for series in [
            PolynomialSeries::Sine,
            PolynomialSeries::Cosine,
            PolynomialSeries::Atanh,
            PolynomialSeries::Exponential,
        ] {
            library.append_series(series);
        }
        library.source += super::FUNCTIONS;
        library
    }

    fn append_series(&mut self, series: PolynomialSeries) {
        let mut terms = PolynomialTerms::from(series).rev();
        let leading = terms.next().expect("every closed series has a term");
        let leading = DoubleFloatLiteral::from(SeriesCoefficient::from(leading));
        write!(
            self.source,
            "fn {series}(z: DoubleFloat) -> DoubleFloat {{\n    var sum = {leading};\n"
        )
        .expect("writing to a string");
        for term in terms {
            let coefficient = DoubleFloatLiteral::from(SeriesCoefficient::from(term));
            writeln!(
                self.source,
                "    sum = df_add(df_mul(sum, z), {coefficient});"
            )
            .expect("writing to a string");
        }
        self.source.push_str("    return sum;\n}\n");
    }
}

impl fmt::Display for DoubleFloatLibrary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

#[cfg(test)]
mod tests;
