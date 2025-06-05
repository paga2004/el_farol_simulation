# El Farol Simulation

A Rust implementation of the El Farol Bar Problem in a 2D grid setting. This project simulates multiple agents distributed on a 2D grid, each employing different strategies to solve the classic El Farol Bar Problem.

## Overview

The El Farol Bar Problem is a classic example of a coordination game where agents must decide whether to visit a bar based on their predictions of attendance. In this grid-based implementation:

- Agents are distributed on a 2D grid
- Each agent uses a strategy to predict the number of visitors
- After each round, agents compare their predictions with their four nearest neighbors
- Agents select new strategies using a softmax function based on performance
- The simulation runs for multiple iterations to study possible equilibria and end states

## Visualization

The project includes visualization capabilities to help understand the simulation:

- 2D grid visualization with different colors representing different active strategies
- Support for creating videos of the simulation evolution
- Real-time visualization of strategy distribution and changes

## Technical Implementation

### Visualization Libraries

For implementing the visualization components, we can consider the following libraries:

1. **Image Generation**:
   - `image`: For frame generation (current approach)

2. **Video Encoding**:
   - `ffmpeg-cli` or direct `ffmpeg` calls: For compiling frames into a video.

## Implementation Questions

These questions need to be addressed before or during implementation:

1. **Prediction Evaluation**: How should we value close but incorrect predictions? For example, if the threshold is 60 visitors:
   - Strategy A predicts 63 visitors
   - Strategy B predicts 40 visitors
   - Actual attendance is 59 visitors
   While Strategy A was numerically closer, it led to the wrong decision (going vs. not going).

2. **Boundary Conditions**: How should agents on the grid boundaries interact with their neighbors?

3. **Strategy Implementation**:
   - What is the structure of a strategy? (e.g., fixed prediction, historical average, etc.)
   - How do we represent and store different strategies?
   - How do we implement strategy comparison and selection?

4. **System Parameters**:
   - What should be the grid size?
   - How many different strategies should be available?
   - What should be the learning rate (softmax temperature)?
   - How many iterations should we run?

5. **Performance Measurement**:
   - What metrics should we track during the simulation?
   - How do we store and analyze the results?
   - What visualization tools should we use?

## Research Questions

These are the scientific questions we aim to answer through the simulation:

1. **Strategy Evolution**: 
   - Do certain strategies tend to cluster together in the grid?
   - How do strategy clusters evolve over time?
   - Are there stable patterns that emerge in the grid?

2. **Information Flow**:
   - How quickly does information (successful strategies) spread across the grid?
   - Are there "information barriers" where certain strategies get trapped?
   - How does the grid topology affect information propagation?

3. **System Behavior**:
   - How sensitive is the final state to the initial distribution of strategies?
   - Do random initial conditions lead to similar end states?
   - Are there critical points in the initial strategy distribution that lead to different outcomes?

4. **Emergent Properties**:
   - What metrics best capture the "success" of the system?
   - How do we measure the stability of the system?
   - Is there a relationship between local and global performance?

5. **Adaptation Dynamics**:
   - How does the learning rate affect strategy evolution?
   - What happens if agents can only observe a subset of their neighbors?
   - How does the grid size affect the system's ability to find good solutions?

## References

- The original El Farol Bar Problem paper is included in this repository as `ElFarolArtur1994.pdf`