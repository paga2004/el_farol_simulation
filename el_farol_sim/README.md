# El Farol Bar Simulation

This project implements a grid-based simulation of the El Farol Bar problem, where agents learn from their neighbors' strategies using softmax adaptation.

## Features

- Grid-based agent system
- Multiple decision-making policies
- Local strategy adaptation using softmax
- Real-time visualization of results
- Performance statistics tracking

## Installation

1. Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

2. Clone this repository:
```bash
git clone https://github.com/yourusername/el_farol_sim.git
cd el_farol_sim
```

3. Build the project:
```bash
cargo build --release
```

## Usage

Run the simulation:
```bash
cargo run --release
```

The program will:
1. Create a grid of agents with random initial strategies
2. Run the simulation for the specified number of iterations
3. Generate visualization plots in the `output` directory

## Configuration

You can modify the simulation parameters in `src/main.rs`:

- `grid_size`: Size of the grid (n × n agents)
- `neighbor_distance`: Manhattan distance for neighbor evaluation
- `capacity`: Bar capacity
- `temperature`: Softmax temperature for strategy adaptation
- `num_iterations`: Number of games to simulate
- `initial_strategies`: Set of available strategies

## Output

The simulation generates two plots in the `output` directory:

1. `attendance.png`: Shows the attendance ratio over time
2. `strategy_distribution.png`: Shows the distribution of strategies over time

## Available Strategies

- Always Go
- Never Go
- Go If Less Than 60%
- Random
- Moving Average

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 