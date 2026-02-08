# Simple-CaveGen-Rust
## Description
A little TUI 'toy' to play around with random generation algorithms visually.

You can apply them sequentially, pause and run them one step at time.

The viewport isn't fixed so the terminal size/zoom does not break the visualisation.

You can move the viewport using the cursor (hard-follow).

## Algorithms implemented

### Paint
<1> This algorithm doesn't really do anything since you can paint in any algorithm
### Game of Life
<2> switches to the game of life algorithm
### DrunkWalk cave generator
<3> switches to the drunk walk cave generation algorithm

## Pause/Step behavior
When an algorithm is selected, running it will keep iterating:
- Life: Keeps changing the grid's cell states in accordance with the established rules
- DrunkWalk: Keeps generating a new cave every tick (note that this will snap the viewport to the middle of the grid every iteration)

'Step' will only do one iteration.

You can 'run' and 'step' an algorithm at the same time to speed up the iteration. Can't be bothered to add a check against it.

## Ways to control the program
Program is keyboard only with the following keys:
- r - regen random grid cell state
- n - regen random grid cell state after incrementing the seed value (new random grid cell state)
- s - step current algorithm
- p - run/pause current algorithm
- q - quit application
