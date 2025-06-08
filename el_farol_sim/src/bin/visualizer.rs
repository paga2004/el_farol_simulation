use chrono::prelude::*;
use clap::Parser;
use el_farol_lib::{Frame, SimulationData};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use indicatif::{ProgressBar, ProgressStyle};
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use liblzma::read::XzDecoder;
use toml;
use dotenvy;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the simulation data file
    input_file: PathBuf,
    /// Flag to enable video creation
    #[arg(long)]
    video: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let file = File::open(&args.input_file)?;
    let mut decompressor = XzDecoder::new(file);
    let mut decoded = Vec::new();
    decompressor.read_to_end(&mut decoded)?;
    let simulation_data: SimulationData = bincode::deserialize(&decoded)?;

    let mut base_output_dir = PathBuf::new();
    if let Ok(val) = std::env::var("EL_FARO_HOME") {
        base_output_dir.push(val);
        base_output_dir.push("visualisation");
    } else {
        base_output_dir.push("output");
    }

    let folder_name = &simulation_data.config.name;
    let experiment_dir = base_output_dir.join(folder_name);
    fs::create_dir_all(&experiment_dir)?;

    let grid_states_dir = experiment_dir.join("grid_states");
    fs::create_dir_all(&grid_states_dir)?;

    visualize_simulation(&simulation_data, &grid_states_dir.to_string_lossy(), &experiment_dir.to_string_lossy())?;

    if args.video {
        let video_path = experiment_dir.join("simulation.mp4");
        create_video(&grid_states_dir.to_string_lossy(), &video_path.to_string_lossy())?;
    }

    fs::write(experiment_dir.join("description.txt"), &simulation_data.config.description)?;

    let sim_conf_path = experiment_dir.join("sim.conf");
    match toml::to_string_pretty(&simulation_data.config) {
        Ok(conf_str) => {
            if let Err(e) = fs::write(&sim_conf_path, conf_str) {
                eprintln!("Failed to write sim.conf to {}: {}", sim_conf_path.display(), e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize simulation config to TOML: {}", e);
        }
    }

    println!("Experiment data saved to: {}", experiment_dir.display());

    Ok(())
}

fn create_video(frames_dir: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let framerate = 10;
    let output = Command::new("ffmpeg")
        .arg("-y") // Overwrite output file if it exists
        .arg("-framerate")
        .arg(framerate.to_string())
        .arg("-i")
        .arg(format!("{}/state_%04d.png", frames_dir))
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(output_path)
        .output()?;

    if !output.status.success() {
        eprintln!(
            "ffmpeg error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn visualize_simulation(
    simulation_data: &SimulationData,
    grid_states_dir: &str,
    plots_dir: &str,
) -> Result<(), Box<dyn Error>> {
    plot_statistics(simulation_data, plots_dir)?;
    plot_strategy_predictions(simulation_data, plots_dir)?;
    let pb = ProgressBar::new(simulation_data.frames.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    for (i, frame) in simulation_data.frames.iter().enumerate() {
        visualize_grid_state(
            frame,
            i,
            &simulation_data.config.initial_strategies,
            grid_states_dir,
        )?;
        pb.inc(1);
    }
    pb.finish_with_message("visualization complete");
    Ok(())
}

fn plot_statistics(
    simulation_data: &SimulationData,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    let mut statistics: HashMap<String, Vec<f64>> = HashMap::new();
    let total_agents = (simulation_data.frames[0].policy_ids.nrows()
        * simulation_data.frames[0].policy_ids.ncols()) as f64;

    for frame in &simulation_data.frames {
        statistics
            .entry("attendance_ratio".to_string())
            .or_insert_with(Vec::new)
            .push(frame.attendance_ratio);

        let mut strategy_counts: HashMap<String, usize> = HashMap::new();
        for policy_id in frame.policy_ids.iter() {
            let policy_name = &simulation_data.config.initial_strategies[*policy_id as usize];
            *strategy_counts.entry(policy_name.clone()).or_insert(0) += 1;
        }

        for (strategy, count) in strategy_counts {
            let ratio = count as f64 / total_agents;
            statistics
                .entry(format!("strategy_{}", strategy))
                .or_insert_with(Vec::new)
                .push(ratio);
        }
    }
    plot_attendance(&statistics, output_dir)?;
    plot_strategy_distribution(&statistics, output_dir)?;
    Ok(())
}

fn plot_strategy_predictions(
    simulation_data: &SimulationData,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir).join("strategy_predictions.png");
    let root = BitMapBackend::new(&path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let initial_strategies = &simulation_data.config.initial_strategies;
    let mut strategy_prediction_series: HashMap<String, Vec<(usize, f64)>> = HashMap::new();

    for (frame_idx, frame) in simulation_data.frames.iter().enumerate() {
        let mut predictions_this_frame: HashMap<String, f64> = HashMap::new();
        // Use indexed_iter to get both the position and the policy_id
        for (pos, policy_id) in frame.policy_ids.indexed_iter() {
            let strategy_name = &initial_strategies[*policy_id as usize];
            // If we haven't recorded a prediction for this strategy in this frame yet
            if !predictions_this_frame.contains_key(strategy_name) {
                // Get the prediction from the corresponding position in the predictions array
                let prediction = frame.predictions[pos];
                predictions_this_frame.insert(strategy_name.clone(), prediction);
            }
        }

        // Add the found predictions for this frame to our time series data
        for (strategy_name, prediction) in predictions_this_frame {
            strategy_prediction_series.entry(strategy_name).or_default().push((frame_idx, prediction));
        }
    }

    let max_iterations = simulation_data.frames.len();
    let mut chart = ChartBuilder::on(&root)
        .caption("Strategy Predictions Over Time", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f32..max_iterations as f32, 0f32..1f32)?;

    chart
        .configure_mesh()
        .x_desc("Iteration")
        .y_desc("Predicted Attendance Ratio")
        .draw()?;

    let colors = vec![&RED, &BLUE, &GREEN, &YELLOW, &MAGENTA, &CYAN, &BLACK, &RGBColor(255, 165, 0), &RGBColor(128, 0, 128), &RGBColor(255, 192, 203)];
    for (i, strategy_name) in initial_strategies.iter().enumerate() {
        if let Some(preds) = strategy_prediction_series.get(strategy_name) {
            let color = colors[i % colors.len()];
            chart
                .draw_series(LineSeries::new(
                    preds.iter().map(|(x, y)| (*x as f32, *y as f32)),
                    color,
                ))?
                .label(strategy_name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}

fn plot_attendance(
    statistics: &HashMap<String, Vec<f64>>,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let attendance = statistics
        .get("attendance_ratio")
        .ok_or("No attendance data found")?;

    let path = Path::new(output_dir).join("attendance.png");
    let root = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Attendance Ratio Over Time", ("sans-serif", 40))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..attendance.len() as f32, 0f32..1f32)?;

    chart
        .configure_mesh()
        .x_desc("Iteration")
        .y_desc("Attendance Ratio")
        .draw()?;

    chart.draw_series(LineSeries::new(
        attendance
            .iter()
            .enumerate()
            .map(|(x, &y)| (x as f32, y as f32)),
        &RED,
    ))?;

    Ok(())
}

fn plot_strategy_distribution(
    statistics: &HashMap<String, Vec<f64>>,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir).join("strategy_distribution.png");
    let root = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_iterations = statistics
        .iter()
        .filter(|(k, _)| k.starts_with("strategy_"))
        .filter_map(|(_, v)| if v.is_empty() { None } else { Some(v.len()) })
        .max()
        .unwrap_or(0);

    let x_axis_max = if max_iterations == 0 {
        1.0f32
    } else {
        max_iterations as f32
    };

    let mut chart = ChartBuilder::on(&root)
        .caption("Strategy Distribution Over Time", ("sans-serif", 40))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..x_axis_max, 0f32..1f32)?;

    chart
        .configure_mesh()
        .x_desc("Iteration")
        .y_desc("Strategy Ratio")
        .draw()?;

    let colors = vec![&RED, &BLUE, &GREEN, &YELLOW, &MAGENTA, &CYAN];
    let mut color_idx = 0;

    for (key, values) in statistics.iter() {
        if key.starts_with("strategy_") {
            if values.is_empty() {
                continue;
            }
            let strategy_name = key.trim_start_matches("strategy_");
            let color = colors[color_idx % colors.len()];

            chart
                .draw_series(LineSeries::new(
                    values
                        .iter()
                        .enumerate()
                        .map(|(x, &y)| (x as f32, y as f32)),
                    *color,
                ))?
                .label(strategy_name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], *color));
            color_idx += 1;
        }
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}

fn visualize_grid_state(
    frame: &Frame,
    iteration_num: usize,
    _strategies: &[String],
    grid_states_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (grid_height, grid_width) = (
        frame.policy_ids.nrows(),
        frame.policy_ids.ncols(),
    );
    let cell_size = 20u32;
    let img_width = grid_width as u32 * cell_size;
    let img_height = grid_height as u32 * cell_size + 80;

    let mut img = RgbImage::new(img_width, img_height);
    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(img_width, img_height), Rgb([255u8, 255, 255]));

    let strategy_colors = [
        Rgb([255, 0, 0]), Rgb([0, 0, 255]), Rgb([0, 255, 0]),
        Rgb([255, 255, 0]), Rgb([255, 0, 255]), Rgb([0, 255, 255]),
        Rgb([128, 0, 0]), Rgb([0, 128, 0]), Rgb([0, 0, 128]),
        Rgb([128, 128, 0]), Rgb([128, 0, 128]), Rgb([0, 128, 128]),
        Rgb([128, 128, 128]), Rgb([192, 192, 192]), Rgb([255, 165, 0]),
        Rgb([255, 192, 203]), Rgb([75, 0, 130]),
    ];

    for r in 0..grid_height {
        for c in 0..grid_width {
            let policy_id = frame.policy_ids[[r, c]] as usize;
            let color = strategy_colors[policy_id % strategy_colors.len()];
            draw_filled_rect_mut(
                &mut img,
                Rect::at((c as i32) * (cell_size as i32), (r as i32) * (cell_size as i32))
                    .of_size(cell_size, cell_size),
                color,
            );
        }
    }

    {
        let root = BitMapBackend::with_buffer(img.as_mut(), (img_width, img_height)).into_drawing_area();

        let text_style = TextStyle::from(("sans-serif", 20.0).into_font()).color(&BLACK);

        let text_y_pos = grid_height as i32 * cell_size as i32 + 10;
        root.draw(&Text::new(
            format!("Iteration: {}", iteration_num),
            (10, text_y_pos),
            text_style.clone(),
        ))?;
        
        let attendance_y_pos = text_y_pos + 25;
        root.draw(&Text::new(
            format!("Attendance: {:.2}%", frame.attendance_ratio * 100.0),
            (10, attendance_y_pos),
            text_style,
        ))?;
    }

    let path = format!("{}/state_{:04}.png", grid_states_dir, iteration_num);
    img.save(&path)?;

    Ok(())
} 