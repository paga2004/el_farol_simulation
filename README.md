# El Farol Bar Simulation

This repository contains part of the code used for a report written for a game theory course. It is a grid-based simulation of the classic El Farol Bar problem implemented in Rust, where agents learn and adapt their strategies based on neighboring agents' performance.

## Project Overview

This simulation models the El Farol Bar problem in a spatial environment where agents are arranged on a grid and make decisions about whether to attend the bar based on past attendance patterns. Agents can observe their neighbors' strategies and adapt accordingly, leading to emergent patterns of behavior.

## Project Structure

### Core Simulation Code (`el_farol_sim/`)
The main Rust implementation containing:

- **`src/simulation_logic/`** - Core simulation components:
  - `simulation.rs` - Main simulation engine and configuration
  - `agent.rs` - Agent behavior and strategy adaptation logic  
  - `game.rs` - Individual game iteration management
  - `policy.rs` - Strategy implementations (AlwaysGo, NeverGo, RandomPolicy, MovingAverage, etc.)
  - `mod.rs` - Module declarations

- **`src/bin/`** - Executable binaries:
  - `simulation.rs` - Main simulation runner
  - `visualizer.rs` - Visualization and video generation tool

- **`src/lib.rs`** - Library interface and data structures

### Simulation Results (`simulations/`)
Contains numbered experiment directories with different configurations and parameters. Each simulation directory includes:
- Compressed simulation data files (`.bin.xz`)
- Configuration files and metadata

### Visualizations (`visualisation/`)
Generated output from the visualizer including:
- Grid state visualizations
- Statistical plots
- Generated videos

### Results Analysis (`results/`)
- `results.md` - Analysis and documentation of simulation results

## How to Run

### 1. Prerequisites

You will need Rust and FFmpeg installed on your system.

### 2. Environment Setup
```bash
# Navigate to the simulation directory
cd el_farol_sim

# Copy environment template (optional)
cp .env_template .env

# Set up environment variables (optional)
export EL_FARO_HOME=/path/to/your/project/root
```

### 3. Running a Simulation
```bash
# Build and run the simulation
cargo run --release --bin simulation
```

The simulation will generate compressed simulation data in the `simulations/` directory.

### 4. Generating Visualizations
```bash
# Visualize simulation results (without video)
cargo run --release --bin visualizer path/to/simulation_data.bin.xz

# Generate visualization with video
cargo run --release --bin visualizer path/to/simulation_data.bin.xz --video
```

This creates:
- Grid state images for each iteration
- Statistical plots (attendance, strategy distribution)
- Strategy prediction charts
- Optional MP4 video compilation

## Configuration

### Simulation Parameters
Key parameters in `SimulationConfig`:

- `grid_size`: Size of the agent grid (e.g., 100x100)
- `neighbor_distance`: Manhattan distance for neighbor evaluation
- `temperature`: Softmax temperature for strategy adaptation
- `policy_retention_rate`: Rate at which agents retain current strategies
- `num_iterations`: Number of simulation rounds
- `rounds_per_update`: Frequency of strategy updates
- `initial_strategies`: Available strategy types
