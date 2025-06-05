mod agent;
mod policy;
mod game;
mod simulation;
mod visualization;

use crate::policy::{AlwaysGo, NeverGo, GoIfLessThanSixty, RandomPolicy, MovingAveragePolicy};
use crate::simulation::{Simulation, SimulationConfig};
use crate::visualization::Visualizer;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Create output directory
    std::fs::create_dir_all("output")?;

    // Create simulation configuration
    let config = SimulationConfig {
        grid_size: 10,
        neighbor_distance: 2,
        capacity: 60,
        temperature: 1.0,
        num_iterations: 1000,
        initial_strategies: vec![
            Box::new(AlwaysGo),
            Box::new(NeverGo),
            Box::new(GoIfLessThanSixty),
            Box::new(RandomPolicy),
            Box::new(MovingAveragePolicy::new(5)),
        ],
    };

    // Create and run simulation
    let mut simulation = Simulation::new(config);
    simulation.run();

    // Visualize results
    let visualizer = Visualizer::new("output".to_string());
    visualizer.visualize_simulation(&simulation)?;

    Ok(())
}
