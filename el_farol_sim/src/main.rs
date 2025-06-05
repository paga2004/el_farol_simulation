mod agent;
mod policy;
mod game;
mod simulation;
mod visualization;

use crate::policy::{AlwaysGo, NeverGo, GoIfLessThanSixty, RandomPolicy, MovingAveragePolicy};
use crate::simulation::{Simulation, SimulationConfig};
use crate::visualization::Visualizer;
use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Create output directory
    std::fs::create_dir_all("output")?;
    std::fs::create_dir_all("output/grid_states")?;

    // Create simulation configuration
    let config = SimulationConfig {
        grid_size: 100,
        neighbor_distance: 2,
        capacity: 60,
        temperature: 0.5,
        num_iterations: 200,
        initial_strategies: vec![
            Box::new(AlwaysGo),
            Box::new(NeverGo),
            Box::new(GoIfLessThanSixty),
            Box::new(RandomPolicy),
            Box::new(MovingAveragePolicy::new(5)),
        ],
    };

    // Create and run simulation
    let pb = ProgressBar::new(config.num_iterations as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    let mut simulation = Simulation::new(config);

    simulation.run(&pb);
    pb.finish_with_message("simulation complete");

    // Visualize results
    let visualizer = Visualizer::new("output".to_string());
    visualizer.visualize_simulation(&simulation)?;

    Ok(())
}
