use std::fmt::Debug;

/// Trait defining the behavior of a policy
pub trait Policy: Send + Sync + Debug + ClonePolicy {
    /// Makes a prediction of bar attendance ratio (0.0-1.0) based on the history of past games
    fn decide(&self, history: &[f64]) -> f64;
    
    /// Returns a name for the policy
    fn name(&self) -> String;
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
        history.iter().rev().nth(1).copied().unwrap_or(0.0)
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