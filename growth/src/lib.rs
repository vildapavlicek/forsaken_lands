use serde::{Deserialize, Serialize};

pub trait GrowthStrategy {
    /// Calculate the value for a given level (0-indexed or 1-indexed depending on usage, but typically 0 is base).
    fn calculate(&self, level: u32) -> f64;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinearGrowth {
    /// The starting value (at level 0)
    pub base: f64,
    /// The amount added per level
    pub increment: f64,
}

impl LinearGrowth {
    pub fn new(base: f64, increment: f64) -> Self {
        Self { base, increment }
    }
}

impl GrowthStrategy for LinearGrowth {
    fn calculate(&self, level: u32) -> f64 {
        self.base + (self.increment * level as f64)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExponentialGrowth {
    /// The starting value (at level 0)
    pub base: f64,
    /// The multiplier per level (e.g., 2.0 for doubling)
    pub factor: f64,
}

impl ExponentialGrowth {
    pub fn new(base: f64, factor: f64) -> Self {
        Self { base, factor }
    }
}

impl GrowthStrategy for ExponentialGrowth {
    fn calculate(&self, level: u32) -> f64 {
        self.base * self.factor.powi(level as i32)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StepGrowth {
    pub base: f64,
    pub step_at: u32,
    pub step_increment: f64,
}

impl StepGrowth {
    pub fn new(base: f64, step_at: u32, step_increment: f64) -> Self {
        Self {
            base,
            step_at,
            step_increment,
        }
    }
}

impl GrowthStrategy for StepGrowth {
    fn calculate(&self, level: u32) -> f64 {
        let steps = level / self.step_at;
        self.base + (self.step_increment * steps as f64)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StaticGrowth {
    pub base: f64,
}

impl GrowthStrategy for StaticGrowth {
    fn calculate(&self, _: u32) -> f64 {
        self.base
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Growth {
    Linear(LinearGrowth),
    Exponential(ExponentialGrowth),
    Step(StepGrowth),
    Static(StaticGrowth),
}

impl GrowthStrategy for Growth {
    fn calculate(&self, level: u32) -> f64 {
        match self {
            Growth::Linear(g) => g.calculate(level),
            Growth::Exponential(g) => g.calculate(level),
            Growth::Step(g) => g.calculate(level),
            Growth::Static(g) => g.calculate(level),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_growth() {
        let growth = LinearGrowth::new(10.0, 5.0);
        assert_eq!(growth.calculate(0), 10.0);
        assert_eq!(growth.calculate(1), 15.0);
        assert_eq!(growth.calculate(2), 20.0);
    }

    #[test]
    fn test_exponential_growth() {
        let growth = ExponentialGrowth::new(10.0, 2.0);
        assert_eq!(growth.calculate(0), 10.0);
        assert_eq!(growth.calculate(1), 20.0);
        assert_eq!(growth.calculate(2), 40.0);
        assert_eq!(growth.calculate(3), 80.0);
    }

    #[test]
    fn test_step_growth() {
        let growth = StepGrowth::new(10.0, 5, 2.0); // Increase by 2 every 5 levels
        assert_eq!(growth.calculate(0), 10.0);
        assert_eq!(growth.calculate(4), 10.0);
        assert_eq!(growth.calculate(5), 12.0);
        assert_eq!(growth.calculate(9), 12.0);
        assert_eq!(growth.calculate(10), 14.0);
    }

    #[test]
    fn test_serialization() {
        let growth = Growth::Linear(LinearGrowth::new(10.0, 5.0));
        let serialized = ron::to_string(&growth).unwrap();
        assert_eq!(serialized, "Linear((base:10.0,increment:5.0))");

        let deserialized: Growth = ron::from_str(&serialized).unwrap();
        match deserialized {
            Growth::Linear(g) => {
                assert_eq!(g.base, 10.0);
                assert_eq!(g.increment, 5.0);
            }
            _ => panic!("Expected Linear growth"),
        }
    }
}
