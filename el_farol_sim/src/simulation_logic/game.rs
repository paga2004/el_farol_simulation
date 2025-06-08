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
        let mut went_to_bar_list = Vec::new();
        for (agent, &prediction) in self.grid.iter_mut().zip(predictions.iter()) {
            agent.last_prediction = Some(prediction);
            let went_to_bar = prediction < 0.6;
            went_to_bar_list.push(went_to_bar);
            if went_to_bar {
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
        for (i, agent) in self.grid.iter_mut().enumerate() {
            agent.update_performance(went_to_bar_list[i], actual_attendance_ratio);
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
