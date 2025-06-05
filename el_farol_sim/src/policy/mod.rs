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
    /// Makes a decision based on the history of past games
    fn decide(&self, history: &[GameResult]) -> bool;
    
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
    fn decide(&self, _history: &[GameResult]) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Always Go"
    }
}

/// Never goes to the bar
#[derive(Debug, Clone, Copy)]
pub struct NeverGo;

impl Policy for NeverGo {
    fn decide(&self, _history: &[GameResult]) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "Never Go"
    }
}

/// Goes if less than 60% went last time
#[derive(Debug, Clone, Copy)]
pub struct GoIfLessThanSixty;

impl Policy for GoIfLessThanSixty {
    fn decide(&self, history: &[GameResult]) -> bool {
        if let Some(last_game) = history.last() {
            let attendance_ratio = last_game.total_attendance as f64 / last_game.total_agents as f64;
            attendance_ratio < 0.6
        } else {
            true // If no history, go
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
    fn decide(&self, _history: &[GameResult]) -> bool {
        rand::random()
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
    fn decide(&self, history: &[GameResult]) -> bool {
        if history.is_empty() {
            return true;
        }

        let relevant_history: Vec<GameResult> = history.iter().rev().take(self.window_size).cloned().collect();
        if relevant_history.is_empty() { // Handle case where window_size > history.len()
            return true; // Or some other default behavior
        }

        let avg_attendance: f64 = relevant_history.iter()
            .map(|game| game.total_attendance as f64 / game.total_agents as f64)
            .sum::<f64>() / relevant_history.len() as f64;

        avg_attendance < 0.6
    }

    fn name(&self) -> &'static str {
        "Moving Average"
    }
} 