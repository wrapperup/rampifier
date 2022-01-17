# `rampifier` repository
This repo contains the `rampifier` library, and some useful tools using it.

## Crates
* [Plate Rampifier](https://github.com/Wrapperup/rampifier/tree/master/plate-rampifier)
  - Also useful with [obj2brs](https://github.com/Kmschr/obj2brs)!
* [Rampified Terrain Generator](https://github.com/Wrapperup/rampifier/tree/master/terrain-gen-example)
* [Rampifier Crate](https://github.com/Wrapperup/rampifier/tree/master/rampifier)

<h1 align=center>Plate Rampifier</h1>

<p align=center>
<img src=https://user-images.githubusercontent.com/7478134/148273037-be45d3f8-75d2-4a96-9b5c-eeae76c170ea.png>
</p>
<br>

This is a sample tool created that takes plates and rampifies it. May be super useful for creating (organic) brick props like trees, rocks, terrain, etc.

## Using Plate Rampifier
[Download Plate Rampifier from here](https://github.com/Wrapperup/rampifier/releases)

### Preparing in.brs
Simply make sure your build is plate aligned. Plates can be resized, and you can use microbricks as long as they are plate sized and plate shaped (so may as well use plates!) Ensure it is aligned to Plate's grid for best results. Your bricks must use a color from your save's color palette. Rampifier will not rampify bricks with custom colors outside of your color palette.

### Generating out.brs
Rampifier takes two arguments, the input of the save file and the output `.brs` path. 

For example, use
`plate-rampifier my_input.brs the_output.brs` or any path to rampify a save. If either are not specified, `in.brs` and `out.brs` are used in the same directory as the binary file.


# `rampifier` crate

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
