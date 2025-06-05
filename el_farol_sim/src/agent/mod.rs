use crate::policy::{Policy, GameResult};
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use rand::Rng;


#[derive(Debug, Clone)]
pub struct Agent {
    position: (usize, usize),
    current_policy: Box<dyn Policy>,
    performance_history: Vec<f64>, // Stores absolute prediction error
    last_prediction: Option<f64>, // Stores the last prediction made by the policy
}

impl Agent {
    pub fn new(position: (usize, usize), initial_policy: Box<dyn Policy>) -> Self {
        Self {
            position,
            current_policy: initial_policy,
            performance_history: Vec::new(),
            last_prediction: None,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    pub fn decide(&mut self, history: &[GameResult]) -> bool {
        let prediction = self.current_policy.decide(history);
        self.last_prediction = Some(prediction);
        prediction < 0.6
    }

    pub fn update_performance(&mut self, actual_attendance_ratio: f64) {
        if let Some(prediction) = self.last_prediction {
            let error = (prediction - actual_attendance_ratio).abs();
            self.performance_history.push(error);
        } else {
            // This case should ideally not happen if decide() is always called before update_performance()
            // For robustness, one might push a max error or log a warning.
            // For now, we'll assume last_prediction is always Some.
            // Alternatively, push a high error value (e.g., 1.0 for ratio based error):
            // self.performance_history.push(1.0); 
        }
    }

    pub fn performance(&self) -> f64 {
        if self.performance_history.is_empty() {
            return 0.0; // Neutral performance if no history. Score 0-100 range.
        }
        // avg_error will be in the range [0.0, 1.0]
        let avg_error: f64 = self.performance_history.iter().sum::<f64>() / self.performance_history.len() as f64;
        // Performance score is 0-100: (1.0 - avg_error) * 100.0. Higher is better.
        ( (1.0 - avg_error).max(0.0) ) * 100.0
    }

    pub fn current_policy(&self) -> &dyn Policy {
        self.current_policy.as_ref()
    }

    pub fn adapt_strategy(&mut self, neighbors: &[(&Agent, f64)], temperature: f64) {
        if neighbors.is_empty() {
            return;
        }

        let max_perf = neighbors.iter()
            .map(|(_, perf)| *perf)
            .fold(f64::NEG_INFINITY, f64::max);

        let probabilities: Vec<f64> = if temperature < 1e-6 { // Handle near-zero temperature as greedy selection
            let mut probs = vec![0.0; neighbors.len()];
            let best_indices: Vec<usize> = neighbors.iter().enumerate()
                .filter(|(_, (_, perf))| (*perf - max_perf).abs() < 1e-6) // Find all ties for best performance
                .map(|(i, _)| i)
                .collect();
            
            if !best_indices.is_empty() {
                let chosen_index_within_best = rand::thread_rng().gen_range(0..best_indices.len());
                probs[best_indices[chosen_index_within_best]] = 1.0;
            }
            // If best_indices is somehow empty (should not happen if neighbors is not empty and max_perf is derived from it),
            // this would leave probs as all zeros, leading to no change. 
            // A more robust fallback might be needed if that edge case is possible.
            probs
        } else {
            let scaled_perfs: Vec<f64> = neighbors.iter()
                .map(|(_, perf)| (perf - max_perf) / temperature)
                .collect();
            
            let exp_values: Vec<f64> = scaled_perfs.iter()
                .map(|&sp| sp.exp())
                .collect();
            
            let exp_sum: f64 = exp_values.iter().sum();

            if exp_sum.abs() < 1e-9 { // Avoid division by zero if sum of exponentials is too small
                // Fallback: equal probability or pick best (current: equal for non-zero temp if sum is zero)
                // This case is unlikely if performances differ or temperature is reasonable.
                let mut probs = vec![0.0; neighbors.len()];
                if !neighbors.is_empty() {
                    let equal_prob = 1.0 / neighbors.len() as f64;
                    for i in 0..probs.len() { probs[i] = equal_prob; }
                }
                probs
            } else {
                exp_values.iter().map(|&exp_val| exp_val / exp_sum).collect()
            }
        };

        // Select new policy based on probabilities
        let mut rng = rand::thread_rng();
        let random_value = rng.gen::<f64>();
        
        let mut cumulative = 0.0;
        for (i, prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if random_value <= cumulative {
                self.current_policy = neighbors[i].0.current_policy.clone();
                self.performance_history = Vec::new();
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{AlwaysGo, NeverGo};

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new((0, 0), Box::new(AlwaysGo));
        assert_eq!(agent.position(), (0, 0));
    }

    #[test]
    fn test_agent_performance() {
        let mut agent = Agent::new((0, 0), Box::new(AlwaysGo)); // AlwaysGo predicts 0.0 (ratio)
        
        // Round 1: Agent predicts 0.0. Actual attendance is 0.2 (20% ratio).
        agent.last_prediction = Some(0.0);
        agent.update_performance(0.2); // Error = |0.0 - 0.2| = 0.2
        assert_eq!(agent.performance_history, vec![0.2]);
        // Performance = (1.0 - 0.2) * 100.0 = 80.0
        assert_eq!(agent.performance(), 80.0);

        // Round 2: Agent predicts 0.0. Actual attendance is 0.7 (70% ratio).
        agent.last_prediction = Some(0.0);
        agent.update_performance(0.7); // Error = |0.0 - 0.7| = 0.7
        assert_eq!(agent.performance_history, vec![0.2, 0.7]);
        // avg_error = (0.2 + 0.7) / 2 = 0.45
        // Performance = (1.0 - 0.45) * 100.0 = 0.55 * 100.0 = 55.0
        assert_eq!(agent.performance(), 55.0);

        // Test with empty history
        let agent_no_history = Agent::new((0,0), Box::new(NeverGo));
        assert_eq!(agent_no_history.performance(), 0.0);

        // Test max error results in 0 performance
        let mut agent_max_error = Agent::new((0,0), Box::new(AlwaysGo));
        agent_max_error.last_prediction = Some(0.0);
        agent_max_error.update_performance(1.0); // prediction 0.0, actual 1.0 -> error 1.0
        // Performance = (1.0 - 1.0) * 100.0 = 0.0
        assert_eq!(agent_max_error.performance(), 0.0);

        // Test zero error results in 100 performance
        let mut agent_zero_error = Agent::new((0,0), Box::new(AlwaysGo));
        agent_zero_error.last_prediction = Some(0.0);
        agent_zero_error.update_performance(0.0); // prediction 0.0, actual 0.0 -> error 0.0
        // Performance = (1.0 - 0.0) * 100.0 = 100.0
        assert_eq!(agent_zero_error.performance(), 100.0);
    }
} 