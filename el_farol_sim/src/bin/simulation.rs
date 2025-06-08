use dotenvy;
use el_farol_lib::simulation_logic::{
    policy::{
        AlwaysGo, ComplexFormulaPolicy, DrunkardPolicy, EvenHistoryAveragePolicy,
        ExponentialMovingAveragePolicy, FullHistoryAveragePolicy, GeneralizedMeanPolicy,
        MovingAveragePolicy, NeverGo, PredictFromDayBeforeYesterday, PredictFromYesterday,
        RandomPolicy, SlidingWeightedAveragePolicy, StupidNerdPolicy, UniformPolicy,
        WeightedHistoryPolicy,
    },
    simulation::{Simulation, SimulationConfig},
};
use el_farol_lib::{SerializableSimulationConfig, SimulationData};
use indicatif::{ProgressBar, ProgressStyle};
use liblzma::write::XzEncoder;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use chrono;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    // Initialize logging
    env_logger::init();

    let initial_strategies: Vec<Arc<dyn el_farol_lib::simulation_logic::policy::Policy>> = vec![
       // Arc::new(AlwaysGo),
      //  Arc::new(NeverGo),
        Arc::new(PredictFromYesterday),
        Arc::new(PredictFromDayBeforeYesterday),
        Arc::new(RandomPolicy),
        Arc::new(MovingAveragePolicy::<3>),
        // Arc::new(MovingAveragePolicy::<5>),
        Arc::new(MovingAveragePolicy::<10>),
        Arc::new(FullHistoryAveragePolicy),
        Arc::new(EvenHistoryAveragePolicy),
        Arc::new(ComplexFormulaPolicy),
        Arc::new(DrunkardPolicy),
        Arc::new(StupidNerdPolicy),
        // Arc::new(UniformPolicy::new(0.0, 1.0)),
        // Arc::new(UniformPolicy::new(0.25, 0.75)),
        // Arc::new(UniformPolicy::new(0.4, 0.6)),
        // FArc::new(WeightedHistoryPolicy::new()),
        // Arc::new(SlidingWeightedAveragePolicy::new()),
        // Arc::new(ExponentialMovingAveragePolicy::new(0.1)),
        // Arc::new(ExponentialMovingAveragePolicy::new(0.5)),
        // Arc::new(ExponentialMovingAveragePolicy::new(0.9)),
         Arc::new(GeneralizedMeanPolicy::<5>::new(1.0)), // Arithmetic mean
        Arc::new(GeneralizedMeanPolicy::<5>::new(2.0)), // Quadratic mean
        Arc::new(GeneralizedMeanPolicy::<5>::new(-1.0)), // Harmonic mean
    ];
    let strategy_names: Vec<String> = initial_strategies.iter().map(|p| p.name()).collect();

    // Create simulation configuration
    let config = SimulationConfig {
        name: "really_high_retention_rate".to_string(),
        description: "Looks like the key is to have a really high retention rate".to_string(),
        grid_size: 100,
        neighbor_distance: 1,
        temperature: 0.1,
        policy_retention_rate: 0.8,
        num_iterations: 1000,
        rounds_per_update: 1,
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
        policy_retention_rate: config.policy_retention_rate,
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
