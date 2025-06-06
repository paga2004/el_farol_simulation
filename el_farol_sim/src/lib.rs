pub mod simulation_logic;

use ndarray::Array2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub policy_names: Array2<String>,
    pub predictions: Array2<f64>,
    pub attendance_ratio: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimulationData {
    pub frames: Vec<Frame>,
    pub initial_strategies: Vec<String>,
} 