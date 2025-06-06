use super::agent::Agent;
use ndarray::Array2;

pub struct Game {
    grid: Array2<Agent>,
    pub history: Vec<f64>,
}

impl Game {
    pub fn new(grid: Array2<Agent>) -> Self {
        Self {
            grid,
            history: Vec::new(),
        }
    }

    pub fn run(&mut self) -> f64 {
        let total_agents = self.grid.len();

        let predictions: Vec<f64> = self.grid.iter()
            .map(|agent| agent.current_policy().decide(&self.history))
            .collect();

        let mut attendance = 0;
        for (agent, &prediction) in self.grid.iter_mut().zip(predictions.iter()) {
            agent.last_prediction = Some(prediction);
            if prediction < 0.6 {
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
        self.history.push(actual_attendance_ratio);
        actual_attendance_ratio
    }

    pub fn get_grid(&self) -> &Array2<Agent> {
        &self.grid
    }

    pub fn set_grid(&mut self, new_grid: Array2<Agent>) {
        self.grid = new_grid;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::policy::AlwaysGo;
    use ndarray::array;

    #[test]
    fn test_game_creation() {
        let grid = array![[
            Agent::new(Box::new(AlwaysGo)),
            Agent::new(Box::new(AlwaysGo))
        ]];
        let game = Game::new(grid);
        assert!(game.history.is_empty());
    }

    #[test]
    fn test_game_run() {
        let grid = array![[
            Agent::new(Box::new(AlwaysGo)), // AlwaysGo predicts 0.0 (ratio), decides to go
            Agent::new(Box::new(AlwaysGo))  // AlwaysGo predicts 0.0 (ratio), decides to go
        ]];
        let mut game = Game::new(grid);
        let ratio = game.run();
        assert_eq!(ratio, 1.0); // Both go, so ratio is 2/2 = 1.0

        // Check agent performance update
        // Actual attendance ratio = 1.0
        // Agents predicted 0.0. Error = |0.0 - 1.0| = 1.0
        // Performance = (1.0 - 1.0) * 100.0 = 0.0
        for agent in game.grid.iter() {
            assert_eq!(agent.performance_history.len(), 1);
            assert_eq!(agent.performance_history[0], 1.0); // Error should be 1.0 (ratio)
            assert_eq!(agent.performance(), 0.0); // Performance score
        }
    }
} 