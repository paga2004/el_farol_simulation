use el_farol_lib::simulation_logic::{
    policy::{AlwaysGo, GoIfLessThanSixty, MovingAveragePolicy, NeverGo, RandomPolicy},
    simulation::{Simulation, SimulationConfig},
};
use el_farol_lib::SimulationData;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Create simulation configuration
    let config = SimulationConfig {
        grid_size: 100,
        neighbor_distance: 1,
        temperature: 1.0,
        num_iterations: 1000,
        rounds_per_update: 10,
        initial_strategies: vec![
            Box::new(AlwaysGo),
            Box::new(NeverGo),
            Box::new(GoIfLessThanSixty),
            Box::new(RandomPolicy),
            Box::new(MovingAveragePolicy::<3>),
        ],
        start_random: false,
    };

    let num_iterations = config.num_iterations;
    let initial_strategies: Vec<String> =
        config.initial_strategies.iter().map(|p| p.name()).collect();

    // Create and run simulation
    let pb = ProgressBar::new(num_iterations as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut simulation = Simulation::new(config);

    let mut data_vec = vec![];

    for _ in 0..num_iterations {
        data_vec.push(simulation.run_iteration());
        pb.inc(1);
    }
    pb.finish_with_message("simulation complete");

    let simulation_data = SimulationData {
        frames: data_vec,
        initial_strategies,
    };

    // Serialize and save simulation data
    let encoded = bincode::serialize(&simulation_data)?;
    let mut file = File::create("simulation_data.bin")?;
    file.write_all(&encoded)?;

    Ok(())
}
