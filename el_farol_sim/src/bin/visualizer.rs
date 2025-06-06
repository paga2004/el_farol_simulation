use chrono::prelude::*;
use clap::Parser;
use el_farol_lib::{Frame, SimulationData};
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use indicatif::{ProgressBar, ProgressStyle};
use plotters::prelude::*;
use ab_glyph::{FontArc, FontVec};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
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
                    color.stroke_width(2),
                ))?
                .label(strategy_name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));

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
    strategies: &[String],
    grid_states_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let grid_size = frame.policy_ids.nrows();
    let cell_size = 20u32;
    let legend_width = 250u32;
    let padding = 10u32;

    const FONT_PATH: &str = "/usr/share/fonts/TTF/Arial.TTF";
    let font_data = fs::read(FONT_PATH).ok();
    let font = font_data.and_then(|data| FontVec::try_from_vec(data).ok().map(FontArc::from));

    let font_scale = 20.0;
    let text_color = Rgb([0u8, 0u8, 0u8]);

    let mut policy_colors: HashMap<String, Rgb<u8>> = HashMap::new();
    let base_colors = [
        Rgb([255, 0, 0]),
        Rgb([0, 0, 255]),
        Rgb([0, 255, 0]),
        Rgb([255, 255, 0]),
        Rgb([255, 0, 255]),
        Rgb([0, 255, 255]),
        Rgb([128, 0, 0]),
        Rgb([0, 0, 128]),
        Rgb([0, 128, 0]),
    ];

    for (i, name) in strategies.iter().enumerate() {
        policy_colors.insert(name.clone(), base_colors[i % base_colors.len()]);
    }

    let img_width = grid_size as u32 * cell_size + legend_width + 3 * padding;
    let img_height = grid_size as u32 * cell_size + 2 * padding;

    let mut img = RgbImage::new(img_width, img_height);

    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    for ((r, c), policy_id) in frame.policy_ids.indexed_iter() {
        let policy_name = &strategies[*policy_id as usize];
        let color = policy_colors
            .get(policy_name)
            .unwrap_or(&Rgb([0, 0, 0]));

        let x = (c as u32 * cell_size + padding) as i32;
        let y = (r as u32 * cell_size + padding) as i32;

        draw_filled_rect_mut(
            &mut img,
            Rect::at(x, y).of_size(cell_size, cell_size),
            *color,
        );
    }

    let legend_x_start = grid_size as u32 * cell_size + 2 * padding;
    let mut current_y = padding;
    let legend_box_size = cell_size / 2;
    let legend_spacing = 5u32;
    let text_x_offset = legend_box_size + 5;

    for policy_name in strategies {
        if let Some(color) = policy_colors.get(policy_name) {
            let rect_x = legend_x_start as i32;
            let rect_y = current_y as i32;
            draw_filled_rect_mut(
                &mut img,
                Rect::at(rect_x, rect_y).of_size(legend_box_size, legend_box_size),
                *color,
            );

            if let Some(ref f) = font {
                draw_text_mut(
                    &mut img,
                    text_color,
                    rect_x + text_x_offset as i32,
                    rect_y,
                    font_scale,
                    &f,
                    policy_name,
                );
            }

            current_y += legend_box_size + legend_spacing;
            if current_y > img_height - padding - legend_box_size {
                eprintln!("Warning: Legend too long to fit in the image.");
                break;
            }
        }
    }

    let output_path_str = format!("{}/state_{:04}.png", grid_states_dir, iteration_num);
    let output_path = Path::new(&output_path_str);
    img.save(output_path)?;

    Ok(())
} 