use crate::simulation::Simulation;
use plotters::prelude::*;
use std::collections::HashMap;
use std::path::Path;

pub struct Visualizer {
    output_dir: String,
}

impl Visualizer {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    pub fn plot_attendance(&self, statistics: &HashMap<String, Vec<f64>>) -> Result<(), Box<dyn std::error::Error>> {
        let attendance = statistics.get("attendance_ratio")
            .ok_or("No attendance data found")?;

        let path = Path::new(&self.output_dir).join("attendance.png");
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
            attendance.iter().enumerate().map(|(x, &y)| (x as f32, y as f32)),
            &RED,
        ))?;

        Ok(())
    }

    pub fn plot_strategy_distribution(&self, statistics: &HashMap<String, Vec<f64>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.output_dir).join("strategy_distribution.png");
        let root = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        // Find the maximum number of iterations for the x-axis from strategy data
        let max_iterations = statistics.iter()
            .filter(|(k, _)| k.starts_with("strategy_"))
            .filter_map(|(_, v)| if v.is_empty() { None } else { Some(v.len()) })
            .max()
            .unwrap_or(0); // Default to 0 if no data or all strategy vectors are empty

        // Ensure x_axis_max is at least 1.0 to have a valid plotting range.
        // If max_iterations is L (e.g., 100 data points), data points are at x = 0, 1, ..., L-1.
        // The range should be 0..L to include all points.
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
                if values.is_empty() { // Skip if there's no data for this strategy
                    continue;
                }
                let strategy_name = key.trim_start_matches("strategy_");
                let color = colors[color_idx % colors.len()];
                
                chart.draw_series(LineSeries::new(
                    values.iter().enumerate().map(|(x, &y)| (x as f32, y as f32)),
                    color.stroke_width(2), // Apply color and stroke width to the line
                ))?
                .label(strategy_name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))); // Use the same style for legend marker

                color_idx += 1;
            }
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            // .position(SeriesLabelPosition::UpperRight) // Example: uncomment to set position
            .draw()?;

        Ok(())
    }

    pub fn visualize_simulation(&self, simulation: &Simulation) -> Result<(), Box<dyn std::error::Error>> {
        let statistics = simulation.get_statistics();
        self.plot_attendance(statistics)?;
        self.plot_strategy_distribution(statistics)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_visualizer_creation() {
        let visualizer = Visualizer::new("test_output".to_string());
        assert_eq!(visualizer.output_dir, "test_output");
    }
} 