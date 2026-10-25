use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NumberSmoothstep;

impl NumberSmoothstep {
    pub fn eval(edge0: f64, edge1: f64, x: f64) -> f64 {
        let denom = edge1 - edge0;
        let t = if denom.abs() < 1e-12 {
            if x < edge0 { 0.0 } else { 1.0 }
        } else {
            ((x - edge0) / denom).clamp(0.0, 1.0)
        };
        t * t * (3.0 - 2.0 * t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoothstep() {
        assert_eq!(NumberSmoothstep::eval(0.0, 1.0, -1.0), 0.0);
        assert_eq!(NumberSmoothstep::eval(0.0, 1.0, 0.0), 0.0);
        assert_eq!(NumberSmoothstep::eval(0.0, 1.0, 0.5), 0.5);
        assert_eq!(NumberSmoothstep::eval(0.0, 1.0, 1.0), 1.0);
        assert_eq!(NumberSmoothstep::eval(0.0, 1.0, 2.0), 1.0);

        // Test custom range
        assert_eq!(NumberSmoothstep::eval(10.0, 20.0, 15.0), 0.5);
        assert_eq!(NumberSmoothstep::eval(10.0, 20.0, 5.0), 0.0);
        assert_eq!(NumberSmoothstep::eval(10.0, 20.0, 25.0), 1.0);
    }
}
