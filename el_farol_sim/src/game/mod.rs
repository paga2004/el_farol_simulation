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
        let mut agent_decisions: Vec<bool> = Vec::with_capacity(total_agents);

        // Collect decisions from all agents and store them
        for agent in self.grid.iter_mut() {
            let decision_went_to_bar = agent.decide(&self.history);
            agent_decisions.push(decision_went_to_bar);
            if decision_went_to_bar {
                attendance += 1;
            }
        }

        // Update agent performances using their individual decisions
        let mut decision_idx = 0;
        for agent in self.grid.iter_mut() {
            let went_to_bar = agent_decisions[decision_idx];
            let agent_had_good_time = if went_to_bar {
                // Agent went to the bar
                attendance < self.capacity
            } else {
                // Agent stayed home
                attendance >= self.capacity // Good decision if bar was full or over capacity
            };
            agent.update_performance(agent_had_good_time);
            decision_idx += 1;
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
            Agent::new((0, 0), Box::new(AlwaysGo)),
            Agent::new((0, 1), Box::new(AlwaysGo))
        ]];
        let mut game = Game::new(grid, 1);
        let result = game.run();
        assert_eq!(result.total_attendance, 2);
        assert_eq!(result.total_agents, 2);
    }
} 