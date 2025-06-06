use dotenvy;
use el_farol_lib::simulation_logic::{
    policy::{
        AlwaysGo, MovingAveragePolicy, NeverGo, PredictFromDayBeforeYesterday,
        PredictFromYesterday, RandomPolicy,
    },
    simulation::{Simulation, SimulationConfig},
};
use el_farol_lib::{SerializableSimulationConfig, SimulationData};
use indicatif::{ProgressBar, ProgressStyle};
use liblzma::write::XzEncoder;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use chrono;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    // Initialize logging
    env_logger::init();

    let initial_strategies: Vec<Box<dyn el_farol_lib::simulation_logic::policy::Policy>> = vec![
        // Box::new(AlwaysGo),
        // Box::new(NeverGo),
        Box::new(PredictFromYesterday),
        Box::new(PredictFromDayBeforeYesterday),
        // Box::new(RandomPolicy),
        // Box::new(MovingAveragePolicy::<3>),
        // Box::new(MovingAveragePolicy::<5>),
        // Box::new(MovingAveragePolicy::<10>),
    ];
    let strategy_names: Vec<String> = initial_strategies.iter().map(|p| p.name()).collect();

    // Create simulation configuration
    let config = SimulationConfig {
        name: "yesterday_vs_day_before".to_string(),
        description: "A simulation where agents predict attendance based on yesterday's or the day before yesterday's results.".to_string(),
        grid_size: 100,
        neighbor_distance: 1,
        temperature: 1.0,
        num_iterations: 1000,
        rounds_per_update: 10,
        initial_strategies,
        start_random: true,
    };

    let num_iterations = config.num_iterations;

    // Create and run simulation
    let pb = ProgressBar::new(num_iterations as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    let mut simulation = Simulation::new(config.clone());

    let mut data_vec = vec![];

    for _ in 0..num_iterations {
        data_vec.push(simulation.run_iteration());
        pb.inc(1);
    }
    pb.finish_with_message("simulation complete");

    let serializable_config = SerializableSimulationConfig {
        name: config.name.clone(),
        description: config.description,
        grid_size: config.grid_size,
        neighbor_distance: config.neighbor_distance,
        temperature: config.temperature,
        num_iterations: config.num_iterations,
        rounds_per_update: config.rounds_per_update,
        initial_strategies: strategy_names.clone(),
        start_random: config.start_random,
    };

    let simulation_data = SimulationData {
        config: serializable_config,
        frames: data_vec,
    };

    // Serialize and save simulation data
    let encoded = bincode::serialize(&simulation_data)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("{}_{}.bin.xz", config.name, timestamp);

    let mut output_path = PathBuf::new();
    if let Ok(val) = std::env::var("EL_FARO_HOME") {
        output_path.push(val);
        output_path.push("simulations");
    }

    fs::create_dir_all(&output_path)?;
    output_path.push(filename);

    let file = File::create(&output_path)?;
    let mut encoder = XzEncoder::new_parallel(file, 6);
    encoder.write_all(&encoded)?;
    encoder.finish()?;

    println!(
        "Simulation data successfully compressed to {}",
        output_path.display()
    );

    Ok(())
}
