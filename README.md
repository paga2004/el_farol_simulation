# really_high_retention_rate

Looks like the key is to have a really high retention rate

## Configuration

```
name = "really_high_retention_rate"
description = "Looks like the key is to have a really high retention rate"
grid_size = 100
neighbor_distance = 1
temperature = 0.1
policy_retention_rate = 0.8
num_iterations = 1000
rounds_per_update = 1
initial_strategies = [
    "Predict from yesterday",
    "Predict from day before yesterday",
    "Random",
    "Moving Average (3)",
    "Moving Average (10)",
    "Full History Average",
    "Even History Average",
    "Complex Formula",
    "Drunkard",
    "Stupid Nerd",
    "Generalized Mean (m=5, r=1)",
    "Generalized Mean (m=5, r=2)",
    "Generalized Mean (m=5, r=-1)",
]
start_random = true

```

## Statistics

![attendance.png](readme_pictures/attendance.png)
![strategy_distribution.png](readme_pictures/strategy_distribution.png)
![strategy_predictions.png](readme_pictures/strategy_predictions.png)

## States

![state_0000.png](readme_pictures/state_0000.png)
![state_0249.png](readme_pictures/state_0249.png)
![state_0499.png](readme_pictures/state_0499.png)
![state_0749.png](readme_pictures/state_0749.png)
![state_0999.png](readme_pictures/state_0999.png)

