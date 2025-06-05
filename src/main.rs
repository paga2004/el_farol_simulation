use clap::Parser;
use rand::Rng;
use image::{ImageBuffer, Rgb};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Width of the grid
    #[arg(short, long, default_value_t = 50)]
    width: usize,

    /// Height of the grid
    #[arg(short = 'H', long, default_value_t = 50)]
    height: usize,

    /// Number of strategies each cell can choose from
    #[arg(short, long, default_value_t = 5)]
    num_strategies: usize,

    /// Learning rate (temperature) for softmax selection
    #[arg(short, long, default_value_t = 1.0)]
    temperature: f64,

    /// Number of iterations to run
    #[arg(short, long, default_value_t = 100)]
    iterations: usize,

    /// Output directory for frames
    #[arg(short, long, default_value = "frames")]
    output_dir: String,
}

/// Trait for prediction strategies
trait Strategy {
    fn predict(&self, history: &[usize]) -> usize;
}

/// A simple strategy that predicts the average of the last n weeks
struct AverageStrategy {
    lookback: usize,
}

impl Strategy for AverageStrategy {
    fn predict(&self, history: &[usize]) -> usize {
        if history.is_empty() {
            return 0;
        }
        let start = history.len().saturating_sub(self.lookback);
        let sum: usize = history[start..].iter().sum();
        sum / (history.len() - start)
    }
}

/// Represents a cell in the grid (and the agent occupying it)
struct GridCell {
    current_strategy: Box<dyn Strategy>,
    prediction_history: Vec<usize>, // TODO: maybe use a ring buffer
    attendance_history: Vec<usize>,
}

impl GridCell {
    fn new(strategy: Box<dyn Strategy>) -> Self {
        Self {
            current_strategy: strategy,
            prediction_history: Vec::new(),
            attendance_history: Vec::new(),
        }
    }

    fn predict(&mut self) -> usize {
        let prediction = self.current_strategy.predict(&self.attendance_history);
        self.prediction_history.push(prediction);
        prediction
    }
}

/// The main grid structure
struct Grid {
    cells: Vec<GridCell>,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(width * height);
        let mut rng = rand::thread_rng();

        for _ in 0..height {
            for _ in 0..width {
                // For now, just use AverageStrategy with random lookback
                let lookback = rng.gen_range(1..=5);
                let strategy = Box::new(AverageStrategy { lookback });
                cells.push(GridCell::new(strategy));
            }
        }

        Self {
            cells,
            width,
            height,
        }
    }

    fn get_cell(&self, x: usize, y: usize) -> &GridCell {
        &self.cells[y * self.width + x]
    }

    fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut GridCell {
        &mut self.cells[y * self.width + x]
    }

    fn save_frame(&self, iteration: usize, output_dir: &str) {
        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);
        
        // For now, just color based on the last prediction
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get_cell(x, y);
                let last_prediction = cell.prediction_history.last().unwrap_or(&0);
                
                // Normalize prediction to a color (0-255)
                let color = (*last_prediction as f64 / 100.0 * 255.0) as u8;
                img.put_pixel(x as u32, y as u32, Rgb([color, color, color]));
            }
        }

        let filename = format!("{}/frame_{:04}.png", output_dir, iteration);
        img.save(&filename).expect("Failed to save frame");
    }
}

fn main() {
    let args = Args::parse();
    println!("Starting simulation with args: {:?}", args);

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");

    // Initialize grid
    let mut grid = Grid::new(args.width, args.height);

    // Main simulation loop
    for iteration in 0..args.iterations {
        println!("Iteration {}", iteration);
        
        // 1. Get predictions from all cells
        let mut predictions = Vec::new();
        for y in 0..args.height {
            for x in 0..args.width {
                let cell = grid.get_cell_mut(x, y);
                predictions.push((x, y, cell.predict()));
            }
        }

        // 2. Calculate actual attendance
        let total_attendance = predictions.iter().map(|(_, _, p)| p).sum::<usize>();
        
        // 3. Update strategies based on performance
        for (x, y, _prediction) in predictions {
            let cell = grid.get_cell_mut(x, y);
            cell.attendance_history.push(total_attendance);

            // For now, just keep the same strategy
            // TODO: Implement strategy selection based on performance
        }

        // 4. Generate visualization frame
        grid.save_frame(iteration, &args.output_dir);
    }

    println!("Simulation complete. Frames saved in {}", args.output_dir);
}
