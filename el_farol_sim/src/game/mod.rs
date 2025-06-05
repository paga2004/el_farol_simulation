use crate::agent::Agent;
use crate::policy::GameResult;
use ndarray::Array2;

pub struct Game {
    grid: Array2<Agent>,
    capacity: usize,
    history: Vec<GameResult>,
}

impl Game {
    pub fn new(grid: Array2<Agent>, capacity: usize) -> Self {
        Self {
            grid,
            capacity,
            history: Vec::new(),
        }
    }

    pub fn run(&mut self) -> GameResult {
        let total_agents = self.grid.len();
        let mut attendance = 0;
        // agent_decisions is not strictly needed anymore if we don't use it for performance update directly
        // However, if there was any other part of the code relying on individual decisions,
        // it might be kept. For this refactor, it seems unused after the first loop.
        // let mut agent_decisions: Vec<bool> = Vec::with_capacity(total_agents);

        // Collect decisions from all agents
        for agent in self.grid.iter_mut() {
            let decision_went_to_bar = agent.decide(&self.history);
            // agent_decisions.push(decision_went_to_bar); // No longer strictly needed for performance
            if decision_went_to_bar {
                attendance += 1;
            }
        }

        // Calculate actual attendance ratio
        let actual_attendance_ratio = if total_agents > 0 {
            attendance as f64 / total_agents as f64
        } else {
            0.0
        };

        // Update agent performances based on their prediction accuracy
        for agent in self.grid.iter_mut() {
            agent.update_performance(actual_attendance_ratio);
        }

        // Record game result
        let result = GameResult {
            total_attendance: attendance,
            total_agents,
        };
        self.history.push(result);
        result
    }

    pub fn get_attendance(&self) -> usize {
        if let Some(last_game) = self.history.last() {
            last_game.total_attendance
        } else {
            0
        }
    }

    pub fn get_grid(&self) -> &Array2<Agent> {
        &self.grid
    }

    pub fn get_history(&self) -> &[GameResult] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::AlwaysGo;
    use ndarray::array;

    #[test]
    fn test_game_creation() {
        let grid = array![[
            Agent::new((0, 0), Box::new(AlwaysGo)),
            Agent::new((0, 1), Box::new(AlwaysGo))
        ]];
        let game = Game::new(grid, 1);
        assert_eq!(game.get_attendance(), 0);
    }

    #[test]
    fn test_game_run() {
        let grid = array![[
            Agent::new((0, 0), Box::new(AlwaysGo)), // AlwaysGo predicts 0.0 (ratio), decides to go
            Agent::new((0, 1), Box::new(AlwaysGo))  // AlwaysGo predicts 0.0 (ratio), decides to go
        ]];
        let mut game = Game::new(grid, 1); // Capacity 1
        let result = game.run();
        assert_eq!(result.total_attendance, 2); // Both go
        assert_eq!(result.total_agents, 2);

        // Check agent performance update
        // Actual attendance ratio = 2 / 2 = 1.0
        // Agents predicted 0.0. Error = |0.0 - 1.0| = 1.0
        // Performance = (1.0 - 1.0) * 100.0 = 0.0
        for agent in game.grid.iter() {
            assert_eq!(agent.performance_history.len(), 1);
            assert_eq!(agent.performance_history[0], 1.0); // Error should be 1.0 (ratio)
            assert_eq!(agent.performance(), 0.0); // Performance score
        }
    }
} 