use crate::policy::{Policy, GameResult};
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use rand::Rng;


#[derive(Debug, Clone)]
pub struct Agent {
    position: (usize, usize),
    current_policy: Box<dyn Policy>,
    performance_history: Vec<bool>, // true if the agent had a good time
}

impl Agent {
    pub fn new(position: (usize, usize), initial_policy: Box<dyn Policy>) -> Self {
        Self {
            position,
            current_policy: initial_policy,
            performance_history: Vec::new(),
        }
    }

    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    pub fn decide(&self, history: &[GameResult]) -> bool {
        self.current_policy.decide(history)
    }

    pub fn update_performance(&mut self, had_good_time: bool) {
        self.performance_history.push(had_good_time);
    }

    pub fn performance(&self) -> f64 {
        if self.performance_history.is_empty() {
            return 0.0;
        }
        self.performance_history.iter()
            .filter(|&&x| x)
            .count() as f64 / self.performance_history.len() as f64
    }

    pub fn current_policy(&self) -> &dyn Policy {
        self.current_policy.as_ref()
    }

    pub fn adapt_strategy(&mut self, neighbors: &[(&Agent, f64)]) {
        if neighbors.is_empty() {
            return;
        }

        // Calculate softmax probabilities
        let max_perf = neighbors.iter()
            .map(|(_, perf)| *perf)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let exp_sum: f64 = neighbors.iter()
            .map(|(_, perf)| (perf - max_perf).exp())
            .sum();

        let probabilities: Vec<f64> = neighbors.iter()
            .map(|(_, perf)| ((perf - max_perf).exp() / exp_sum))
            .collect();

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
        let mut agent = Agent::new((0, 0), Box::new(AlwaysGo));
        agent.update_performance(true);
        agent.update_performance(false);
        agent.update_performance(true);
        assert_eq!(agent.performance(), 2.0/3.0);
    }
} 