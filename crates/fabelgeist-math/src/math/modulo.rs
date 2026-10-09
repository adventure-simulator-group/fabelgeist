use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NumberMod;

impl NumberMod {
    pub fn eval(x: f64, y: f64) -> f64 {
        if y.abs() < 1e-12 {
            0.0
        } else {
            x.rem_euclid(y)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod() {
        assert_eq!(NumberMod::eval(5.0, 3.0), 2.0);
        assert_eq!(NumberMod::eval(3.0, 3.0), 0.0);
        assert_eq!(NumberMod::eval(-1.0, 4.0), 3.0);
        assert_eq!(NumberMod::eval(0.0, 5.0), 0.0);
        assert_eq!(NumberMod::eval(5.0, 0.0), 0.0);
    }
}
