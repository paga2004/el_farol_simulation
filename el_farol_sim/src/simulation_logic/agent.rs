use super::policy::Policy;
use crate::simulation_logic::game::Game;
use rand::seq::SliceRandom;
use rand::Rng;
use std::sync::Arc;
use std::fmt::Debug;


#[derive(Debug)]
pub struct Agent {
    current_policy: Arc<dyn Policy>,
    pub performance_history: Vec<f64>, // Stores points now, not error
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

    pub fn update_performance(&mut self, went_to_bar: bool, actual_attendance_ratio: f64) {
        let bar_is_overcrowded = actual_attendance_ratio >= 0.6;
        let score = match (went_to_bar, bar_is_overcrowded) {
            (true, false) => 1.0, // Went to a non-crowded bar
            (false, true) => 1.0, // Stayed home from a crowded bar
            _ => 0.0,             // Other cases
        };
        self.performance_history.push(score);
    }

    pub fn performance(&self) -> f64 {
        if self.performance_history.is_empty() {
            return 0.0;
        }
        self.performance_history.iter().sum()
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
        let mut agent = Agent::new(Arc::new(AlwaysGo));

        // Scenario 1: Agent goes to a non-crowded bar (2 points)
        // Agent predicts 0.0, so `went_to_bar` will be true.
        // Actual attendance is 0.2, so `bar_is_overcrowded` is false.
        agent.update_performance(true, 0.2);
        assert_eq!(agent.performance_history, vec![2.0]);
        assert!((agent.performance() - 2.0).abs() < 1e-9);

        // Scenario 2: Agent stays home from a crowded bar (1 point)
        // Agent predicts 0.7, so `went_to_bar` will be false.
        // Actual attendance is 0.7, so `bar_is_overcrowded` is true.
        agent.update_performance(false, 0.7);
        assert_eq!(agent.performance_history, vec![2.0, 1.0]);
        assert!((agent.performance() - 3.0).abs() < 1e-9);

        // Scenario 3: Agent goes to a crowded bar (0 points)
        agent.update_performance(true, 0.8);
        assert_eq!(agent.performance_history, vec![2.0, 1.0, 0.0]);
        assert!((agent.performance() - 3.0).abs() < 1e-9);

        // Scenario 4: Agent stays home from a non-crowded bar (0 points)
        agent.update_performance(false, 0.3);
        assert_eq!(agent.performance_history, vec![2.0, 1.0, 0.0, 0.0]);
        assert!((agent.performance() - 3.0).abs() < 1e-9);

        // Test with empty history
        let agent_no_history = Agent::new(Arc::new(NeverGo));
        assert!((agent_no_history.performance() - 0.0).abs() < 1e-9);
    }
} 