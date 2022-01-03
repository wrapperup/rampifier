<h1 align=center>Rampified Terrain Generator</h1>

<p align=center>
<img src=https://user-images.githubusercontent.com/7478134/147893120-d88c29b9-d013-447b-b8f5-f3c7f0b961af.png>
</p>
<br>

This repository hosts a sample terrain generation tool and the `rampifier` module, which
works best for organic voxel meshes.

## Terrain Generation
This sample tool generates bricks in 3 passes:
- Voxel noise generation (Simple 3D array)
- Rampifier, which generates ramps based upon a voxel input.
- Optimize voxels into bricks (box-fill)

## How do I use this?
This may not be useful by itself for many people. If you are familiar with Rust, you can download this project and modify `main.rs` to generate different kinds of terrain with the `noise-rs` library included. Otherwise, you can implement the `rampifier.rs` module in your own Rust programs and play with the output.

## Using rampifier.rs in your own tools
Here is some example code for using the `rampifier.rs` module:

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
