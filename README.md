# El Farol Bar Simulation

This repository contains part of the code used for a report written for a game theory course. It is a grid-based simulation of the classic El Farol Bar problem implemented in Rust, where agents learn and adapt their strategies based on neighboring agents' performance.


## Running a simulation

Adjust the parameters in `simulation.rs` then run the following comand which will run the simulation and safe the compressed data in the `simulations` folder.

```bash
cargo run --release --bin simulation
```


### Generating graphs, pictures and videos
```bash
cargo run --release --bin visualizer path/to/simulation_data.bin.xz --video
```