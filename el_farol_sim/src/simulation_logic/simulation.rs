use super::agent::Agent;
use super::game::Game;
use super::policy::Policy;
use crate::{Frame, StrategyId};
use ndarray::Array2;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct SimulationConfig {
    pub name: String,
    pub description: String,
    pub grid_size: usize,
    pub neighbor_distance: usize,
    pub temperature: f64,
    pub policy_retention_rate: f64,
    pub num_iterations: usize,
    pub rounds_per_update: usize,
    pub initial_strategies: Vec<Arc<dyn Policy>>,
    pub start_random: bool,
}

pub struct Simulation {
    game: Game,
    config: SimulationConfig,
    statistics: HashMap<String, Vec<f64>>,
    strategy_map: HashMap<String, StrategyId>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = rand::thread_rng();
        let mut grid: Array2<Agent>;

        if config.initial_strategies.is_empty() {
            panic!("Initial strategies cannot be empty for random setup.");
        }

        let strategy_map: HashMap<String, StrategyId> = config
            .initial_strategies
            .iter()
            .enumerate()
            .map(|(i, policy)| (policy.name(), i as StrategyId))
            .collect();

        if config.start_random {
            // Initialize with a temporary agent for Array2::from_elem, then fill randomly
            grid = Array2::from_elem(
                (config.grid_size, config.grid_size),
                Agent::new(config.initial_strategies[0].clone()),
            ); // Placeholder

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
            let base_policy = config
                .initial_strategies
                .iter()
                .find(|p| p.name() == base_policy_name)
                .map(|p| p.clone())
                .unwrap_or_else(|| {
                    eprintln!(
                        "Warning: Policy 'Never Go' not found in initial_strategies. Using the first available strategy as base."
                    );
                    if config.initial_strategies.is_empty() {
                        panic!("Initial strategies cannot be empty for non-random setup if 'Never Go' is missing.");
                    }
                    config.initial_strategies[0].clone()
                });

            let other_policies: Vec<Arc<dyn Policy>> = config
                .initial_strategies
                .iter()
                .filter(|p| p.name() != base_policy.name()) // Filter out the base policy by name
                .map(|p| p.clone())
                .collect();

            // Initialize all cells with the base policy
            grid = Array2::from_elem(
                (config.grid_size, config.grid_size),
                Agent::new(base_policy.clone()),
            ); // Placeholder coords, actual in loop

            for r in 0..config.grid_size {
                for c in 0..config.grid_size {
                    grid[[r, c]] = Agent::new(base_policy.clone());
                }
            }

            if !other_policies.is_empty() {
                let gs = config.grid_size;
                if gs > 0 {
                    // Top-left
                    grid[[0, 0]] =
                        Agent::new(other_policies[0 % other_policies.len()].clone());

                    // Top-right
                    if gs > 1 {
                        grid[[0, gs - 1]] =
                            Agent::new(other_policies[1 % other_policies.len()].clone());
                    }

                    // Bottom-left
                    if gs > 1 {
                        // Also implies gs > 0 already checked
                        grid[[gs - 1, 0]] =
                            Agent::new(other_policies[2 % other_policies.len()].clone());
                    }

                    // Bottom-right
                    if gs > 1 {
                        // Also implies gs > 0 already checked
                        grid[[gs - 1, gs - 1]] =
                            Agent::new(other_policies[3 % other_policies.len()].clone());
                    }
                }
            } else {
                eprintln!("Warning: No 'other' policies available for corners. All agents will start with the base policy.");
            }
        }

        let game = Game::new(grid);

        let sim = Self {
            game,
            config,
            statistics: HashMap::new(),
            strategy_map,
        };

        sim
    }

    pub fn run_iteration(&mut self) -> Frame {
        for _round in 0..self.config.rounds_per_update {
            self.game.run();
        }
        self.adapt_strategies();

        
        let grid = self.game.get_grid();
        let policy_ids = grid.mapv(|agent| {
            let name = agent.current_policy().name();
            *self.strategy_map.get(&name).unwrap() as StrategyId
        });
        let predictions = grid.mapv(|agent| agent.last_prediction.unwrap_or(0.0));
        let attendance_ratio = *self.game.history.last().unwrap_or(&0.0);


        Frame {
            policy_ids,
            predictions,
            attendance_ratio,
        }
    }

    fn adapt_strategies(&mut self) {
        let grid = self.game.get_grid();
        let mut new_grid = grid.clone();
        let temperature = self.config.temperature;
        let policy_retention_rate = self.config.policy_retention_rate;

        for i in 0..self.config.grid_size {
            for j in 0..self.config.grid_size {
                let mut neighbors = Vec::new();

                // Find neighbors within Manhattan distance
                let neighbor_distance = self.config.neighbor_distance as isize;
                for ni in (i as isize - neighbor_distance).max(0)
                    ..=(i as isize + neighbor_distance).min(self.config.grid_size as isize - 1)
                {
                    for nj in (j as isize - neighbor_distance).max(0)
                        ..=(j as isize + neighbor_distance).min(self.config.grid_size as isize - 1)
                    {
                        let distance = (i as isize - ni).abs() + (j as isize - nj).abs();
                        if distance <= neighbor_distance {
                            neighbors.push((
                                &grid[[ni as usize, nj as usize]],
                                grid[[ni as usize, nj as usize]].performance(),
                            ));
                        }
                    }
                }

                // Adapt strategy - agent.performance() will use accumulated history
                new_grid[[i, j]].adapt_strategy(&neighbors, temperature, policy_retention_rate);
            }
        }

        // Clear performance history for all agents for the next batch of rounds
        for i in 0..self.config.grid_size {
            for j in 0..self.config.grid_size {
                new_grid[[i, j]].clear_performance_history();
            }
        }

        // Update grid with new strategies, preserving the game's attendance history
        self.game.set_grid(new_grid);
    }

    pub fn get_statistics(&self) -> &HashMap<String, Vec<f64>> {
        &self.statistics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::policy::{AlwaysGo, NeverGo};
    use std::sync::Arc;

    #[test]
    fn test_simulation_creation() {
        let config = SimulationConfig {
            name: "test".to_string(),
            description: "test description".to_string(),
            grid_size: 2,
            neighbor_distance: 1,
            temperature: 1.0,
            policy_retention_rate: 0.5,
            num_iterations: 10,
            rounds_per_update: 1,
            initial_strategies: vec![Arc::new(AlwaysGo), Arc::new(NeverGo)],
            start_random: true,
        };
        let simulation = Simulation::new(config);
        assert_eq!(simulation.get_statistics().len(), 0);
    }
} 