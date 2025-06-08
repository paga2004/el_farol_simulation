use std::fmt::Debug;
use rand::distributions::{Distribution, Uniform};
use std::sync::{Arc, Mutex};

/// Trait defining the behavior of a policy
pub trait Policy: Send + Sync + Debug {
    /// Makes a prediction of bar attendance ratio (0.0-1.0) based on the history of past games
    fn decide(&self, history: &[f64]) -> f64;
    
    /// Returns a name for the policy
    fn name(&self) -> String;
}

/// Always goes to the bar
#[derive(Debug, Clone, Copy)]
pub struct AlwaysGo;

impl Policy for AlwaysGo {
    fn decide(&self, _history: &[f64]) -> f64 {
        0.0
    }

    fn name(&self) -> String {
        "Always Go".to_string()
    }
}

/// Never goes to the bar
#[derive(Debug, Clone, Copy)]
pub struct NeverGo;

impl Policy for NeverGo {
    fn decide(&self, _history: &[f64]) -> f64 {
        1.0
    }

    fn name(&self) -> String {
        "Never Go".to_string()
    }
}

/// Predicts attendance will be the same as yesterday
#[derive(Debug, Clone, Copy)]
pub struct PredictFromYesterday;

impl Policy for PredictFromYesterday {
    fn decide(&self, history: &[f64]) -> f64 {
        if let Some(last_ratio) = history.last() {
            *last_ratio
        } else {
            0.0 // If no history, predict 0.0 attendance ratio
        }
    }

    fn name(&self) -> String {
        "Predict from yesterday".to_string()
    }
}

/// Predicts attendance will be the same as the day before yesterday
#[derive(Debug, Clone, Copy)]
pub struct PredictFromDayBeforeYesterday;

impl Policy for PredictFromDayBeforeYesterday {
    fn decide(&self, history: &[f64]) -> f64 {
        history.iter().rev().nth(1).or(history.last()).copied().unwrap_or(0.0)
    }

    fn name(&self) -> String {
        "Predict from day before yesterday".to_string()
    }
}

/// Random decision
#[derive(Debug, Clone, Copy)]
pub struct RandomPolicy;

impl Policy for RandomPolicy {
    fn decide(&self, _history: &[f64]) -> f64 {
        rand::random::<f64>() // Predict a random ratio between 0.0 and 1.0
    }

    fn name(&self) -> String {
        "Random".to_string()
    }
}

/// Moving average based decision
#[derive(Debug, Clone, Copy)]
pub struct MovingAveragePolicy<const WINDOW_SIZE: usize>;

impl<const WINDOW_SIZE: usize> Policy for MovingAveragePolicy<WINDOW_SIZE> {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.is_empty() || WINDOW_SIZE == 0 {
            return 0.0;
        }

        let start = history.len().saturating_sub(WINDOW_SIZE);
        let relevant_history = &history[start..];

        if relevant_history.is_empty() {
            return 0.0;
        }

        let sum: f64 = relevant_history.iter().sum();
        sum / relevant_history.len() as f64
    }

    fn name(&self) -> String {
        format!("Moving Average ({})", WINDOW_SIZE)
    }
}

/// Predicts attendance will be the average of all past attendances
#[derive(Debug, Clone, Copy)]
pub struct FullHistoryAveragePolicy;

impl Policy for FullHistoryAveragePolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }
        history.iter().sum::<f64>() / history.len() as f64
    }

    fn name(&self) -> String {
        "Full History Average".to_string()
    }
}

/// Predicts attendance will be the average of past attendances on even days
#[derive(Debug, Clone, Copy)]
pub struct EvenHistoryAveragePolicy;

impl Policy for EvenHistoryAveragePolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        let even_day_history: Vec<f64> = history.iter().step_by(2).copied().collect();
        if even_day_history.is_empty() {
            0.0
        } else {
            even_day_history.iter().sum::<f64>() / even_day_history.len() as f64
        }
    }

    fn name(&self) -> String {
        "Even History Average".to_string()
    }
}

/// Complex formula: 1/2[sqrt(1/2(b_n^2+b_(n-1)^2)) + b_(n-2)]
#[derive(Debug, Clone, Copy)]
pub struct ComplexFormulaPolicy;

impl Policy for ComplexFormulaPolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.len() < 3 {
            return 0.0;
        }
        let b_n = history[history.len() - 1];
        let b_n_minus_1 = history[history.len() - 2];
        let b_n_minus_2 = history[history.len() - 3];

        0.5 * ((0.5 * (b_n.powi(2) + b_n_minus_1.powi(2))).sqrt() + b_n_minus_2)
    }

    fn name(&self) -> String {
        "Complex Formula".to_string()
    }
}

/// Drunkard: Average of b_0, ... b_n and subtract 0.05
#[derive(Debug, Clone, Copy)]
pub struct DrunkardPolicy;

impl Policy for DrunkardPolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        let avg = if history.is_empty() {
            0.0
        } else {
            history.iter().sum::<f64>() / history.len() as f64
        };
        (avg - 0.05).clamp(0.0, 1.0)
    }

    fn name(&self) -> String {
        "Drunkard".to_string()
    }
}

/// Stupid Nerd: Like drunkard, but instead add 0.05
#[derive(Debug, Clone, Copy)]
pub struct StupidNerdPolicy;

impl Policy for StupidNerdPolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        let avg = if history.is_empty() {
            0.0
        } else {
            history.iter().sum::<f64>() / history.len() as f64
        };
        (avg + 0.05).clamp(0.0, 1.0)
    }

    fn name(&self) -> String {
        "Stupid Nerd".to_string()
    }
}

/// Predicts a random ratio from a uniform distribution U(low, high)
#[derive(Debug, Clone, Copy)]
pub struct UniformPolicy {
    low: f64,
    high: f64,
}

impl UniformPolicy {
    pub fn new(low: f64, high: f64) -> Self {
        assert!(low <= high && (0.0..=1.0).contains(&low) && (0.0..=1.0).contains(&high));
        Self { low, high }
    }
}

impl Policy for UniformPolicy {
    fn decide(&self, _history: &[f64]) -> f64 {
        let mut rng = rand::thread_rng();
        let dist = Uniform::new(self.low, self.high);
        dist.sample(&mut rng)
    }

    fn name(&self) -> String {
        format!("Uniform [{}..{})", self.low, self.high)
    }
}

/// Weighted average of history. Weights are iid Unif([0,2]) generated at start of game.
#[derive(Debug)]
pub struct WeightedHistoryPolicy {
    weights: Mutex<Vec<f64>>,
    dist: Uniform<f64>,
}

impl WeightedHistoryPolicy {
    pub fn new() -> Self {
        Self {
            weights: Mutex::new(Vec::new()),
            dist: Uniform::new(0.0, 2.0),
        }
    }
}

impl Default for WeightedHistoryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WeightedHistoryPolicy {
    fn clone(&self) -> Self {
        Self {
            weights: Mutex::new(self.weights.lock().unwrap().clone()),
            dist: self.dist,
        }
    }
}

impl Policy for WeightedHistoryPolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }

        let mut weights = self.weights.lock().unwrap();

        if weights.len() < history.len() {
            let mut rng = rand::thread_rng();
            let additional_weights_to_generate = history.len() - weights.len();
            for _ in 0..additional_weights_to_generate {
                weights.push(self.dist.sample(&mut rng));
            }
        }

        let weighted_sum: f64 = history
            .iter()
            .rev()
            .zip(weights.iter())
            .map(|(b, w)| b * w)
            .sum();

        weighted_sum / history.len() as f64
    }

    fn name(&self) -> String {
        "Weighted History".to_string()
    }
}

/// Sliding weighted average with 5 random weights
#[derive(Debug, Clone)]
pub struct SlidingWeightedAveragePolicy {
    weights: [f64; 5],
}

impl SlidingWeightedAveragePolicy {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let dist = Uniform::new(0.0, 2.0);
        let mut weights = [0.0; 5];
        for w in &mut weights {
            *w = dist.sample(&mut rng);
        }
        Self { weights }
    }
}

impl Default for SlidingWeightedAveragePolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl Policy for SlidingWeightedAveragePolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }

        let window = &history[history.len().saturating_sub(5)..];
        let weighted_sum: f64 = window
            .iter()
            .rev()
            .zip(self.weights.iter())
            .map(|(b, w)| b * w)
            .sum();

        if window.is_empty() {
            0.0
        } else {
            weighted_sum / (window.len() as f64)
        }
    }

    fn name(&self) -> String {
        "Sliding Weighted Average (5)".to_string()
    }
}

/// Exponentially weighted moving average
#[derive(Debug, Clone)]
pub struct ExponentialMovingAveragePolicy {
    alpha: f64,
}

impl ExponentialMovingAveragePolicy {
    pub fn new(alpha: f64) -> Self {
        assert!(alpha > 0.0 && alpha < 1.0);
        Self { alpha }
    }
}

impl Policy for ExponentialMovingAveragePolicy {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }

        let n = history.len() - 1;

        let mut weighted_sum = 0.0;
        let mut current_alpha_power = 1.0;
        for b in history.iter().rev() {
            // b_n, b_{n-1}, ...
            weighted_sum += b * current_alpha_power;
            current_alpha_power *= self.alpha;
        }

        let normalization = if self.alpha == 1.0 {
            (n + 1) as f64
        } else {
            (1.0 - self.alpha.powi((n + 1) as i32)) / (1.0 - self.alpha)
        };

        weighted_sum / normalization
    }

    fn name(&self) -> String {
        format!("Exponential Moving Average (a={})", self.alpha)
    }
}

/// Generalized sliding window mean
#[derive(Debug, Clone)]
pub struct GeneralizedMeanPolicy<const M: usize> {
    r: f64,
}

impl<const M: usize> GeneralizedMeanPolicy<M> {
    pub fn new(r: f64) -> Self {
        assert!(r != 0.0);
        Self { r }
    }
}

impl<const M: usize> Policy for GeneralizedMeanPolicy<M> {
    fn decide(&self, history: &[f64]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }

        let n = history.len();
        let window_size = n.min(M);
        let window = &history[n - window_size..];

        if window.is_empty() {
            return 0.0;
        }

        let sum_of_powers: f64 = window.iter().map(|b| b.powf(self.r)).sum();
        let mean_of_powers = sum_of_powers / window_size as f64;

        mean_of_powers.powf(1.0 / self.r)
    }

    fn name(&self) -> String {
        format!("Generalized Mean (m={}, r={})", M, self.r)
    }
} 