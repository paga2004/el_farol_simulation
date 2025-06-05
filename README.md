# Current implementations
## nandor-oneshot
### specs
- Policies take a vector of past games, output a bool, to go or not to go. Therefore performance is binary too, happy or not.

### issues
- agents are evaluated on whole history, not just for the tail for which they used the same strategy; probably unintended behavior

- code is a bit janky, and inefficient, but we'll fix that when it becomes a performance issue, for 1000x1000 its fine

- no grid visualisation yet
## discrete-clear_history_on_switch
### specs
- agents clear their performance history when switching policies
- agents prefer not to switch policies, to incentivize longer chains, so that the predictions are less random

### issues

## predict_visitor_count
### specs
- agents predict how many people will attend the night, then base their decision if they go or not on that number.
- agent performance history gets cleared when switching policies.

# Ideas
## Evaluation of history of policies locally, instead of performance of agents
Instead of evaluating each agents success since last policy switch, we could evaluate for each policy, how well it did proportionally to how many times it was used.

This could lead to fancy dynamics, where died out strategies come to life again.

This might go hand in hand with multiple rounds, see next point.
## Play multiple rounds between evaluation


