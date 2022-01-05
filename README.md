# `rampifier` repository
This repo contains the `rampifier` library, and some nifty little tools.

## Tools
* Plate Rampifier
* Rampified Terrain Generator

See releases for Plate Rampifier binaries.

## Using `rampifier` in your own tools
Here is some example code for using the `rampifier` crate:

```rust
// Populate this with values. The u8 value usually identifies
// the color index used, and the ramp algo. will use this.
let mut grid: Vec<Option<u8>> = ...;

let mut rampifier = Rampifier::new(
    (DEFAULT_LEN_X, DEFAULT_LEN_Y, DEFAULT_LEN_Z),
    grid,
    RampifierConfig::default()
);

// Generate floor ramps
let ramps_floor = &mut rampifier.generate_ramps(true);

// Generate ceiling ramps
let ramps_ceiling = &mut rampifier.generate_ramps(false);

save.bricks.append(ramps_floor);
save.bricks.append(ramps_ceiling);

// Sets the voxels occupied by ramps to empty.
rampifier.remove_occupied_voxels();

// Move grid back out of the rampifier to do further processing.
let mut grid = rampifier.move_grid();
```
