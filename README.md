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
