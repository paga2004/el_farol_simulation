use serde::{Serialize, Deserialize};
use std::fmt::Debug;

/// Represents the result of a single game
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameResult {
    pub total_attendance: usize,
    pub total_agents: usize,
}

/// Trait defining the behavior of a policy
pub trait Policy: Send + Sync + Debug + ClonePolicy {
    /// Makes a prediction of bar attendance ratio (0.0-1.0) based on the history of past games
    fn decide(&self, history: &[GameResult]) -> f64;
    
    /// Returns a name for the policy
    fn name(&self) -> &'static str;
}

pub trait ClonePolicy {
    fn clone_box(&self) -> Box<dyn Policy>;
}

impl<T> ClonePolicy for T
where
    T: 'static + Policy + Clone,
{
    fn clone_box(&self) -> Box<dyn Policy> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Policy> {
    fn clone(&self) -> Box<dyn Policy> {
        self.clone_box()
    }
}

/// Always goes to the bar
#[derive(Debug, Clone, Copy)]
pub struct AlwaysGo;

impl Policy for AlwaysGo {
    fn decide(&self, _history: &[GameResult]) -> f64 {
        0.59
    }

    fn name(&self) -> &'static str {
        "Always Go"
    }
}

/// Never goes to the bar
#[derive(Debug, Clone, Copy)]
pub struct NeverGo;

impl Policy for NeverGo {
    fn decide(&self, _history: &[GameResult]) -> f64 {
        0.61
    }

    fn name(&self) -> &'static str {
        "Never Go"
    }
}

/// Goes if less than 60% went last time
#[derive(Debug, Clone, Copy)]
pub struct GoIfLessThanSixty;

impl Policy for GoIfLessThanSixty {
    fn decide(&self, history: &[GameResult]) -> f64 {
        if let Some(last_game) = history.last() {
            if last_game.total_agents == 0 {
                0.0 // Avoid division by zero, predict 0.0 ratio
            } else {
                last_game.total_attendance as f64 / last_game.total_agents as f64
            }
        } else {
            0.0 // If no history, predict 0.0 attendance ratio
        }
    }

    fn name(&self) -> &'static str {
        "Go If < 60%"
    }
}

/// Random decision
#[derive(Debug, Clone, Copy)]
pub struct RandomPolicy;

impl Policy for RandomPolicy {
    fn decide(&self, _history: &[GameResult]) -> f64 {
        rand::random::<f64>() // Predict a random ratio between 0.0 and 1.0
    }

    fn name(&self) -> &'static str {
        "Random"
    }
}

/// Moving average based decision
#[derive(Debug, Clone, Copy)]
pub struct MovingAveragePolicy {
    window_size: usize,
}

impl MovingAveragePolicy {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

impl Policy for MovingAveragePolicy {
    fn decide(&self, history: &[GameResult]) -> f64 {
        if history.is_empty() {
            return 0.0; // Predict 0.0 ratio if no history
        }

        let relevant_history: Vec<GameResult> = history.iter().rev().take(self.window_size).cloned().collect();
        if relevant_history.is_empty() { // Handle case where window_size > history.len() or window_size is 0
            return 0.0; // Or some other default behavior, predict 0.0 ratio
        }

        let num_relevant_games_with_agents = relevant_history.iter().filter(|game| game.total_agents > 0).count();
        if num_relevant_games_with_agents == 0 {
            return 0.0; // Avoid division by zero if no games had agents, predict 0.0 ratio
        }

        let sum_ratios: f64 = relevant_history.iter()
            .filter(|game| game.total_agents > 0) // Avoid division by zero
            .map(|game| game.total_attendance as f64 / game.total_agents as f64)
            .sum::<f64>();
            
        sum_ratios / num_relevant_games_with_agents as f64
    }

    fn name(&self) -> &'static str {
        "Moving Average"
    }
} 