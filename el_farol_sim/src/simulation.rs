use crate::agent::Agent;
use crate::game::Game;
use crate::policy::{Policy, GameResult};
use ndarray::Array2;
use std::collections::HashMap;
use rand::Rng;
use image::{RgbImage, Rgb};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use std::path::Path;
use indicatif::ProgressBar;
use rusttype::{Font, Scale};
use std::fs;

pub struct SimulationConfig {
    pub grid_size: usize,
    pub neighbor_distance: usize,
    pub temperature: f64,
    pub num_iterations: usize,
    pub rounds_per_update: usize,
    pub initial_strategies: Vec<Box<dyn Policy>>,
    pub start_random: bool,
}

pub struct Simulation {
    game: Game,
    config: SimulationConfig,
    statistics: HashMap<String, Vec<f64>>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = rand::thread_rng();
        let mut grid: Array2<Agent>;

        if config.start_random {
            // Initialize with a temporary agent for Array2::from_elem, then fill randomly
            grid = Array2::from_elem((config.grid_size, config.grid_size), 
                Agent::new(config.initial_strategies[0].clone())); // Placeholder
            
            if config.initial_strategies.is_empty() {
                panic!("Initial strategies cannot be empty for random setup.");
            }

            for i in 0..config.grid_size {
                for j in 0..config.grid_size {
                    let strategy_idx = rng.gen_range(0..config.initial_strategies.len());
                    let strategy = config.initial_strategies[strategy_idx].clone();
                    grid[[i, j]] = Agent::new(strategy);
                }
            }
        } else {
            // Specific "Never Go with corners" setup
            let base_policy_name = "Never Go";
            let base_policy = config.initial_strategies.iter()
                .find(|p| p.name() == base_policy_name)
                .map(|p| p.clone_box())
                .unwrap_or_else(|| {
                    eprintln!(
                        "Warning: Policy 'Never Go' not found in initial_strategies. Using the first available strategy as base."
                    );
                    if config.initial_strategies.is_empty() {
                        panic!("Initial strategies cannot be empty for non-random setup if 'Never Go' is missing.");
                    }
                    config.initial_strategies[0].clone_box()
                });

            let other_policies: Vec<Box<dyn Policy>> = config.initial_strategies.iter()
                .filter(|p| p.name() != base_policy.name()) // Filter out the base policy by name
                .map(|p| p.clone_box())
                .collect();

            // Initialize all cells with the base policy
            grid = Array2::from_elem((config.grid_size, config.grid_size), 
                Agent::new(base_policy.clone_box())); // Placeholder coords, actual in loop

            for r in 0..config.grid_size {
                for c in 0..config.grid_size {
                    grid[[r,c]] = Agent::new(base_policy.clone_box());
                }
            }

            if !other_policies.is_empty() {
                let gs = config.grid_size;
                if gs > 0 {
                    // Top-left
                    grid[[0, 0]] = Agent::new(other_policies[0 % other_policies.len()].clone_box());
                    
                    // Top-right
                    if gs > 1 {
                        grid[[0, gs - 1]] = Agent::new(other_policies[1 % other_policies.len()].clone_box());
                    }
                    
                    // Bottom-left
                    if gs > 1 { // Also implies gs > 0 already checked
                        grid[[gs - 1, 0]] = Agent::new(other_policies[2 % other_policies.len()].clone_box());
                    }

                    // Bottom-right
                    if gs > 1 { // Also implies gs > 0 already checked
                        grid[[gs - 1, gs - 1]] = Agent::new(other_policies[3 % other_policies.len()].clone_box());
                    }
                }
            } else {
                eprintln!("Warning: No 'other' policies available for corners. All agents will start with the base policy.");
            }
        }

        let game = Game::new(grid);

        // Print policy color mapping for the legend
        println!("--- Policy Color Legend ---");
        let policy_names: Vec<String> = config.initial_strategies.iter()
            .map(|p| p.name().to_string())
            .collect();
        let base_colors = [
            Rgb([255, 0, 0]), Rgb([0, 0, 255]), Rgb([0, 255, 0]), 
            Rgb([255, 255, 0]), Rgb([255, 0, 255]), Rgb([0, 255, 255]),
            Rgb([128, 0, 0]), Rgb([0, 0, 128]), Rgb([0, 128, 0]),
        ];
        for (i, name) in policy_names.iter().enumerate() {
            let color = base_colors[i % base_colors.len()];
            println!("{}: Rgb({}, {}, {})", name, color[0], color[1], color[2]);
        }
        println!("-------------------------");
        
        let sim = Self {
            game,
            config,
            statistics: HashMap::new(),
        };

        // Visualize initial state (iteration 0)
        if let Err(e) = sim.visualize_grid_state(0) {
            eprintln!("Error visualizing initial grid state: {}", e);
        }

        sim
    }

    pub fn run_iteration(&mut self, adaptation_step_num: usize) {
        for _round in 0..self.config.rounds_per_update {
            // Run a single game round
            let result = self.game.run();
            
            // Update statistics for this game round
            // Note: update_statistics collects data per game round. If you later want statistics
            // per adaptation step (e.g., average performance of agents in that step), 
            // that would require additional logic.
            self.update_statistics(&result);
        }
        
        // Adapt strategies based on the performance over the last rounds_per_update
        self.adapt_strategies();

        // Visualize grid state after adaptation
        // adaptation_step_num is 0-indexed for the completed adaptation step.
        // So, after adaptation_step 0, we save state 1, etc.
        if let Err(e) = self.visualize_grid_state(adaptation_step_num + 1) {
            eprintln!(
                "Error visualizing grid state for adaptation_step_num {}: {}", 
                adaptation_step_num + 1, e
            );
        }
    }

    pub fn run(&mut self, pb: &ProgressBar) {
        for i in 0..self.config.num_iterations {
            self.run_iteration(i);
            pb.inc(1);
        }
    }

    fn update_statistics(&mut self, result: &GameResult) {
        // Update attendance statistics
        let attendance_ratio = result.total_attendance as f64 / result.total_agents as f64;
        self.statistics
            .entry("attendance_ratio".to_string())
            .or_insert_with(Vec::new)
            .push(attendance_ratio);

        // Update strategy distribution
        let mut strategy_counts: HashMap<String, usize> = HashMap::new();
        for agent in self.game.get_grid().iter() {
            let strategy_name = agent.current_policy().name().to_string();
            *strategy_counts.entry(strategy_name).or_insert(0) += 1;
        }

        for (strategy, count) in strategy_counts {
            let ratio = count as f64 / (self.config.grid_size * self.config.grid_size) as f64;
            self.statistics
                .entry(format!("strategy_{}", strategy))
                .or_insert_with(Vec::new)
                .push(ratio);
        }
    }

    fn adapt_strategies(&mut self) {
        let grid = self.game.get_grid();
        let mut new_grid = grid.clone();
        let temperature = self.config.temperature;

        for i in 0..self.config.grid_size {
            for j in 0..self.config.grid_size {
                let mut neighbors = Vec::new();
                
                // Find neighbors within Manhattan distance
                for ni in 0..self.config.grid_size {
                    for nj in 0..self.config.grid_size {
                        let distance = (i as isize - ni as isize).abs() + 
                                     (j as isize - nj as isize).abs();
                        if distance <= self.config.neighbor_distance as isize {
                            neighbors.push((&grid[[ni, nj]], grid[[ni, nj]].performance()));
                        }
                    }
                }

                // Adapt strategy - agent.performance() will use accumulated history
                new_grid[[i, j]].adapt_strategy(&neighbors, temperature);
            }
        }

        // Clear performance history for all agents for the next batch of rounds
        for i in 0..self.config.grid_size {
            for j in 0..self.config.grid_size {
                new_grid[[i, j]].clear_performance_history();
            }
        }

        // Update grid with new strategies and cleared histories
        self.game = Game::new(new_grid);
    }

    fn visualize_grid_state(&self, iteration_num: usize) -> Result<(), Box<dyn std::error::Error>> {
        let grid_size = self.config.grid_size;
        let cell_size = 20u32; // Size of each cell in pixels
        let legend_width = 250u32; // Increased width for text
        let padding = 10u32;

        // Attempt to load the specified font for the legend text
        const FONT_PATH: &str = "/usr/share/fonts/TTF/Arial.TTF";

        let font_data = match fs::read(FONT_PATH) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!(
                    "Warning: Could not load font from {}. Error: {}. Legend text will be missing.",
                    FONT_PATH, e
                );
                None
            }
        };

        let font = match font_data {
            Some(data) => Font::try_from_vec(data),
            None => None, // Error already printed
        };
        
        let font_scale = Scale::uniform(20.0); // Font size for legend text - Increased from 12.0
        let text_color = Rgb([0u8, 0u8, 0u8]); // Black for legend text

        let policy_names: Vec<String> = self.config.initial_strategies.iter()
            .map(|p| p.name().to_string())
            .collect();
        
        let mut policy_colors: HashMap<String, Rgb<u8>> = HashMap::new();
        let base_colors = [
            Rgb([255, 0, 0]), Rgb([0, 0, 255]), Rgb([0, 255, 0]), 
            Rgb([255, 255, 0]), Rgb([255, 0, 255]), Rgb([0, 255, 255]),
            Rgb([128, 0, 0]), Rgb([0, 0, 128]), Rgb([0, 128, 0]),
        ];

        for (i, name) in policy_names.iter().enumerate() {
            policy_colors.insert(name.clone(), base_colors[i % base_colors.len()]);
        }

        let img_width = grid_size as u32 * cell_size + legend_width + 3 * padding;
        let img_height = grid_size as u32 * cell_size + 2 * padding;

        let mut img = RgbImage::new(img_width, img_height);

        // Draw background (white)
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw grid cells
        let grid = self.game.get_grid();
        for r in 0..grid_size {
            for c in 0..grid_size {
                let agent = &grid[[r, c]];
                let policy_name = agent.current_policy().name().to_string();
                let color = policy_colors.get(&policy_name).unwrap_or(&Rgb([0, 0, 0])); // Default to black

                let x = (c as u32 * cell_size + padding) as i32;
                let y = (r as u32 * cell_size + padding) as i32;
                
                draw_filled_rect_mut(
                    &mut img,
                    Rect::at(x, y).of_size(cell_size, cell_size),
                    *color,
                );
            }
        }

        // Draw legend with color boxes and text
        let legend_x_start = grid_size as u32 * cell_size + 2 * padding;
        let mut current_y = padding;
        let legend_box_size = cell_size / 2;
        let legend_spacing = 5u32;
        let text_x_offset = legend_box_size + 5; // Space between color box and text

        // Iterate through initial_strategies to maintain order and get names for the map lookup
        for policy_instance in self.config.initial_strategies.iter() {
            let policy_name = policy_instance.name().to_string();
            if let Some(color) = policy_colors.get(&policy_name) {
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
                        rect_y, // Align text with top of the color box
                        font_scale,
                        f,
                        &policy_name,
                    );
                }

                current_y += legend_box_size + legend_spacing;
                if current_y > img_height - padding - legend_box_size { 
                    eprintln!("Warning: Legend too long to fit in the image.");
                    break;
                }
            }
        }
        
        let output_path_str = format!("output/grid_states/state_{:04}.png", iteration_num);
        let output_path = Path::new(&output_path_str);
        img.save(output_path)?;

        Ok(())
    }

    pub fn get_statistics(&self) -> &HashMap<String, Vec<f64>> {
        &self.statistics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{AlwaysGo, NeverGo};

    #[test]
    fn test_simulation_creation() {
        let config = SimulationConfig {
            grid_size: 2,
            neighbor_distance: 1,
            temperature: 1.0,
            num_iterations: 10,
            rounds_per_update: 1,
            initial_strategies: vec![Box::new(AlwaysGo), Box::new(NeverGo)],
            start_random: true,
        };
        let simulation = Simulation::new(config);
        assert_eq!(simulation.get_statistics().len(), 0);
    }
} 