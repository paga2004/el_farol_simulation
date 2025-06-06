pub mod simulation_logic;

use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type StrategyId = u8;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableSimulationConfig {
    pub name: String,
    pub description: String,
    pub grid_size: usize,
    pub neighbor_distance: usize,
    pub temperature: f64,
    pub num_iterations: usize,
    pub rounds_per_update: usize,
    pub initial_strategies: Vec<String>,
    pub start_random: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    pub policy_ids: Array2<StrategyId>,
    pub predictions: Array2<f64>,
    pub attendance_ratio: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimulationData {
    pub config: SerializableSimulationConfig,
    pub frames: Vec<Frame>,
} 