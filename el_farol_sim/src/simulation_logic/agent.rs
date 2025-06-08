use super::policy::Policy;
use crate::simulation_logic::game::Game;
use rand::seq::SliceRandom;
use rand::Rng;
use std::sync::Arc;
use std::fmt::Debug;


#[derive(Debug)]
pub struct Agent {
    current_policy: Arc<dyn Policy>,
    pub performance_history: Vec<f64>, // Stores absolute prediction error
    pub last_prediction: Option<f64>, // Stores the last prediction made by the policy
}

impl Agent {
    pub fn new(initial_policy: Arc<dyn Policy>) -> Self {
        Self {
            current_policy: initial_policy,
            performance_history: Vec::new(),
            last_prediction: None,
        }
    }

    pub fn update_performance(&mut self, actual_attendance_ratio: f64) {
        if let Some(prediction) = self.last_prediction {
            let error = (prediction - actual_attendance_ratio).abs();
            self.performance_history.push(error);
        } else {
            panic!("update_performance called without a previous prediction");
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

    pub fn current_policy(&self) -> Arc<dyn Policy> {
        self.current_policy.clone()
    }

    pub fn decide(&mut self, history: &[f64]) -> f64 {
        let prediction = self.current_policy.decide(history);
        self.last_prediction = Some(prediction);
        prediction
    }

    pub fn adapt_strategy(&mut self, neighbors: &[(&Agent, f64)], temperature: f64, policy_retention_rate: f64) {
        if neighbors.is_empty() {
            return;
        }

        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < policy_retention_rate {
            return;
        }

        let new_policy = self.choose_new_policy(neighbors, temperature, &mut rng);
        
        if self.current_policy.name() != new_policy.name() {
            self.current_policy = new_policy;
            self.performance_history.clear();
        } else {
            self.current_policy = new_policy;
        }
    }
    
    fn choose_new_policy(&self, neighbors: &[(&Agent, f64)], temperature: f64, rng: &mut impl Rng) -> Arc<dyn Policy> {
        let performances: Vec<f64> = neighbors.iter().map(|(_, perf)| *perf).collect();
    
        if temperature < 1e-6 {
            self.greedy_policy_selection(neighbors, &performances, rng)
        } else {
            self.softmax_policy_selection(neighbors, &performances, temperature, rng)
        }
    }
    
    fn greedy_policy_selection(&self, neighbors: &[(&Agent, f64)], performances: &[f64], rng: &mut impl Rng) -> Arc<dyn Policy> {
        let max_perf = performances.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
        let best_indices: Vec<usize> = performances.iter().enumerate()
            .filter(|(_, &perf)| (perf - max_perf).abs() < 1e-6)
            .map(|(i, _)| i)
            .collect();
    
        if let Some(&chosen_index) = best_indices.choose(rng) {
            neighbors[chosen_index].0.current_policy()
        } else {
            // Fallback: This should ideally not be reached if neighbors is not empty.
            // Return the policy of a random neighbor.
            neighbors.choose(rng).unwrap().0.current_policy()
        }
    }
    
    fn softmax_policy_selection(&self, neighbors: &[(&Agent, f64)], performances: &[f64], temperature: f64, rng: &mut impl Rng) -> Arc<dyn Policy> {
        let max_perf = performances.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
        let weights: Vec<f64> = performances.iter()
            .map(|&perf| ((perf - max_perf) / temperature).exp())
            .collect();
    
        let dist = match rand::distributions::WeightedIndex::new(&weights) {
            Ok(dist) => dist,
            Err(_) => { // This can happen if all weights are zero (e.g., due to underflow)
                // Fallback to uniform random selection among neighbors
                let dist = rand::distributions::Uniform::new(0, neighbors.len());
                let chosen_index = rng.sample(dist);
                return neighbors[chosen_index].0.current_policy();
            }
        };
    
        let chosen_index = rng.sample(dist);
        neighbors[chosen_index].0.current_policy()
    }

    pub fn clear_performance_history(&mut self) {
        self.performance_history.clear();
    }
}

impl Clone for Agent {
    fn clone(&self) -> Self {
        Self {
            current_policy: self.current_policy.clone(),
            performance_history: self.performance_history.clone(),
            last_prediction: self.last_prediction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::policy::{AlwaysGo, NeverGo};

    #[test]
    fn test_agent_performance() {
        let mut agent = Agent::new(Arc::new(AlwaysGo)); // AlwaysGo predicts 0.0 (ratio)
        
        // Round 1: Agent predicts 0.0. Actual attendance is 0.2 (20% ratio).
        agent.last_prediction = Some(0.0);
        agent.update_performance(0.2); // Error = |0.0 - 0.2| = 0.2
        assert_eq!(agent.performance_history, vec![0.2]);
        // Performance = (1.0 - 0.2) * 100.0 = 80.0
        assert!((agent.performance() - 80.0).abs() < 1e-9);

        // Round 2: Agent predicts 0.0. Actual attendance is 0.7 (70% ratio).
        agent.last_prediction = Some(0.0);
        agent.update_performance(0.7); // Error = |0.0 - 0.7| = 0.7
        assert_eq!(agent.performance_history, vec![0.2, 0.7]);
        // avg_error = (0.2 + 0.7) / 2 = 0.45
        // Performance = (1.0 - 0.45) * 100.0 = 0.55 * 100.0 = 55.0
        assert!((agent.performance() - 55.0).abs() < 1e-9);

        // Test with empty history
        let agent_no_history = Agent::new(Arc::new(NeverGo));
        assert!((agent_no_history.performance() - 0.0).abs() < 1e-9);

        // Test max error results in 0 performance
        let mut agent_max_error = Agent::new(Arc::new(AlwaysGo));
        agent_max_error.last_prediction = Some(0.0);
        agent_max_error.update_performance(1.0); // prediction 0.0, actual 1.0 -> error 1.0
        // Performance = (1.0 - 1.0) * 100.0 = 0.0
        assert!((agent_max_error.performance() - 0.0).abs() < 1e-9);

        // Test zero error results in 100 performance
        let mut agent_zero_error = Agent::new(Arc::new(AlwaysGo));
        agent_zero_error.last_prediction = Some(0.0);
        agent_zero_error.update_performance(0.0); // prediction 0.0, actual 0.0 -> error 0.0
        // Performance = (1.0 - 0.0) * 100.0 = 100.0
        assert!((agent_zero_error.performance() - 100.0).abs() < 1e-9);
    }
} 