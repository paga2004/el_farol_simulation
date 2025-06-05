# El Farol Bar Simulation Specification

## Overview
This program simulates the El Farol Bar problem in a grid-based environment where agents learn from their neighbors' strategies. The simulation runs multiple games, with agents adapting their strategies based on local performance.

## Program Structure

### Core Components

1. **Agent**
   - Position in grid (x, y coordinates)
   - Current strategy (policy)
   - Performance history
   - Methods:
     - `decide()`: Makes decision for current game
     - `evaluate_neighbors()`: Evaluates performance of neighboring agents
     - `adapt_strategy()`: Updates strategy based on neighbor performance

2. **Policy**
   - Trait defining decision-making behavior
   - Input: Vector of past game results
   - Output: Boolean (go/not go)
   - Example implementations:
     - Always go
     - Never go
     - Go if less than 60% went last time
     - Go if more than 60% went last time
     - Random decision
     - Moving average based decision

3. **Game**
   - Manages single game iteration
   - Tracks attendance
   - Methods:
     - `run()`: Executes single game
     - `get_attendance()`: Returns number of agents who went

4. **Simulation**
   - Manages overall simulation
   - Grid of agents
   - Game history
   - Methods:
     - `run_iteration()`: Runs single game and adaptation phase
     - `run_simulation()`: Runs multiple iterations
     - `get_statistics()`: Returns simulation statistics

### Data Structures

1. **Grid**
   - 2D array of agents
   - Size: n × n
   - Methods for neighbor lookup within Manhattan distance k

2. **Game History**
   - Vector of past game results
   - Contains:
     - Total attendance
     - Individual agent decisions
     - Outcomes

## Configuration Parameters

- `n`: Grid size (n × n agents)
- `k`: Manhattan distance for neighbor evaluation
- `capacity`: Bar capacity (default: 60% of total agents)
- `temperature`: Softmax temperature for strategy adaptation
- `num_iterations`: Number of games to simulate
- `initial_strategy_distribution`: Distribution of initial strategies

## Visualization

### Real-time Display
- Grid visualization showing:
  - Agent positions
  - Current strategies (color-coded)
  - Performance indicators
- Attendance graph
- Strategy distribution graph

### Statistics and Analysis
- Average attendance over time
- Strategy distribution over time
- Local strategy convergence patterns
- Performance metrics:
  - Individual agent success rate
  - Strategy success rate
  - Spatial correlation of strategies

## Implementation Details

### Rust-specific Considerations

1. **Dependencies**
   - `rand`: Random number generation
   - `ndarray`: Grid operations
   - `plotters`: Visualization
   - `serde`: Serialization for saving/loading states

2. **Concurrency**
   - Parallel processing for agent decisions
   - Thread-safe data structures for shared state

3. **Error Handling**
   - Proper error types for invalid configurations
   - Graceful handling of edge cases

### Performance Considerations
- Efficient neighbor lookup using spatial indexing
- Optimized strategy evaluation
- Memory-efficient history tracking

## Output and Logging

1. **Logging Levels**
   - DEBUG: Detailed agent decisions
   - INFO: Game summaries
   - WARN: Unusual patterns
   - ERROR: Simulation issues

2. **Data Export**
   - CSV format for analysis
   - JSON format for state saving/loading
   - Visualization exports (PNG, SVG)

## Testing Strategy

1. **Unit Tests**
   - Individual agent behavior
   - Policy implementations
   - Game mechanics

2. **Integration Tests**
   - Full simulation runs
   - Strategy adaptation
   - Grid operations

3. **Property Tests**
   - Invariant checking
   - Edge case handling

## Future Extensions

1. **Additional Features**
   - Multiple bar locations
   - Dynamic capacity
   - Heterogeneous agent preferences
   - Network-based connections instead of grid

2. **Analysis Tools**
   - Advanced statistics
   - Machine learning integration
   - Strategy evolution tracking

## Usage Examples

```rust
// Example configuration
let config = SimulationConfig {
    grid_size: 10,
    neighbor_distance: 2,
    capacity: 60,
    temperature: 1.0,
    num_iterations: 1000,
};

// Run simulation
let mut simulation = Simulation::new(config);
simulation.run();

// Analyze results
let stats = simulation.get_statistics();
println!("Average attendance: {}", stats.average_attendance);
```

## Documentation Requirements

1. **Code Documentation**
   - Rust doc comments
   - Examples for public APIs
   - Architecture overview

2. **User Documentation**
   - Installation instructions
   - Configuration guide
   - Visualization guide
   - Analysis tools guide 