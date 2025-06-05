use crate::agent::Agent;
use crate::game::Game;
use crate::policy::{Policy, GameResult};
use ndarray::Array2;
use std::collections::HashMap;
use rand::Rng;

pub struct SimulationConfig {
    pub grid_size: usize,
    pub neighbor_distance: usize,
    pub capacity: usize,
    pub temperature: f64,
    pub num_iterations: usize,
    pub initial_strategies: Vec<Box<dyn Policy>>,
}

pub struct Simulation {
    game: Game,
    config: SimulationConfig,
    statistics: HashMap<String, Vec<f64>>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        // Create grid with random initial strategies
        let mut rng = rand::thread_rng();
        let mut grid = Array2::from_elem((config.grid_size, config.grid_size), 
            Agent::new((0, 0), config.initial_strategies[0].clone()));
        
        for i in 0..config.grid_size {
            for j in 0..config.grid_size {
                let strategy_idx = rng.gen_range(0..config.initial_strategies.len());
                let strategy = config.initial_strategies[strategy_idx].clone();
                grid[[i, j]] = Agent::new((i, j), strategy);
            }
        }

        let game = Game::new(grid, config.capacity);
        
        Self {
            game,
            config,
            statistics: HashMap::new(),
        }
    }

    pub fn run_iteration(&mut self) {
        // Run the game
        let result = self.game.run();
        
        // Update statistics
        self.update_statistics(&result);
        
        // Adapt strategies
        self.adapt_strategies();
    }

    pub fn run(&mut self) {
        for _ in 0..self.config.num_iterations {
            self.run_iteration();
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

                // Adapt strategy
                new_grid[[i, j]].adapt_strategy(&neighbors);
            }
        }

        // Update grid
        self.game = Game::new(new_grid, self.config.capacity);
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
            capacity: 2,
            temperature: 1.0,
            num_iterations: 10,
            initial_strategies: vec![Box::new(AlwaysGo), Box::new(NeverGo)],
        };
        let simulation = Simulation::new(config);
        assert_eq!(simulation.get_statistics().len(), 0);
    }
} 