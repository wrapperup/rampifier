mod rampify;

use std::{env, fs::File};
use noise::{RidgedMulti, Seedable};
use crate::rampify::{Rampify, RampifyConfig};
use brickadia::{
    save::*,
    write::SaveWriter,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    /////////////////////////////////////////////////////////////
    //                   CONSTANTS & CONFIG                    //
    /////////////////////////////////////////////////////////////

    // Size of a chunk to be processed on a thread.
    const DEFAULT_CHUNK_X_SIZE: usize = 64 * 2;
    const DEFAULT_CHUNK_Y_SIZE: usize = 64 * 2;
    const DEFAULT_CHUNK_Z_SIZE: usize = 64 * 2 * 2;

    // Length of grid vector.
    const DEFAULT_LEN_X: usize = DEFAULT_CHUNK_X_SIZE * 1;
    const DEFAULT_LEN_Y: usize = DEFAULT_CHUNK_Y_SIZE * 1;
    const DEFAULT_LEN_Z: usize = DEFAULT_CHUNK_Z_SIZE * 1;

    // Default save path.
    const DEFAULT_SAVE_PATH: &str = "./out.brs";

    let mut save_location = DEFAULT_SAVE_PATH;

    if args.len() > 1
    {
        save_location = &args[1];
    }

    // This uses u8::MAX to identify an empty voxel. Should we use Option<u8> instead?
    let mut grid: Vec<u8> = vec![u8::MAX; DEFAULT_LEN_X * DEFAULT_LEN_Y * DEFAULT_LEN_Z];

    let public = User {
        name: "Terrain".into(),
        id: "a8033bee-6c37-4118-b4a6-cecc1d966132".parse().unwrap(),
    };

    let mut save = SaveData::default();

    // set the first header
    save.header1.author = public.clone();
    save.header1.host = Some(public.clone());
    save.header1.description = "This was saved with brickadia-rs!".into();

    // set the second header
    save.header2
        .brick_owners
        .push(BrickOwner::from_user_bricks(public.clone(), 100));

    save.header2.brick_assets =
        vec![
            "PB_DefaultBrick".into(),
            "PB_DefaultRamp".into(),
            "PB_DefaultWedge".into(),
            "PB_DefaultRampCrest".into(),
        ];


    /////////////////////////////////////////////////////////////
    //                  PASS 1: GENERATE NOISE                 //
    /////////////////////////////////////////////////////////////

    use noise::{NoiseFn, OpenSimplex, Perlin};

    let simplex = OpenSimplex::new();
    let ridged = RidgedMulti::new();

    println!("Generating voxel data...");

    let now = Instant::now();

    let get_index = |pos: (usize, usize, usize)| -> usize {
        pos.0 + pos.1 * DEFAULT_LEN_X + pos.2  * DEFAULT_LEN_X * DEFAULT_LEN_Y
    };

    for z in 0..DEFAULT_LEN_Z {
        for y in 0..DEFAULT_LEN_Y {
            for x in 0..DEFAULT_LEN_X {
                // populate da grid

                let scale = 0.08;

                let val = simplex.get([
                    x as f64 * scale,
                    y as f64 * scale,
                    z as f64 * scale / 3.0
                ]);

                let val = val + 0.5;
                let mut val = (val * 254.0) as u8;

                if val < 128 {
                    val = u8::MAX;
                }

                grid[get_index((x, y, z))] = val;
            }
        }
    }

    println!(" - Done in {}s\n", now.elapsed().as_millis() as f64 / 1000.0);

    let box_remove = |g: &mut Vec<u8>, pos: &(usize, usize, usize), size: &(usize, usize, usize)| {
        let &(x, y, z) = pos;
        let &(w, l, h) = size;

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    let p = (x + i, y + j, z + k);

                    g[get_index((p.0, p.1, p.2))] = u8::MAX;
                }
            }
        }
    };

    let can_box = |g: &Vec<u8>, pos: &(usize, usize, usize), size: &(usize, usize, usize)| -> bool {
        let &(w, l, h) = size;

        if pos.0 + w > DEFAULT_LEN_X {
            return false;
        }
        if pos.1 + l > DEFAULT_LEN_Y {
            return false;
        }
        if pos.2 + h > DEFAULT_LEN_Z {
            return false;
        }

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    let pos = (pos.0 + i, pos.1 + j, pos.2 + k);
                    if g[get_index((pos.0, pos.1, pos.2))] == u8::MAX {
                        return false;
                    }
                }
            }
        }

        return true;
    };


    /////////////////////////////////////////////////////////////
    //                  PASS 2: GENERATE RAMPS                 //
    /////////////////////////////////////////////////////////////

    println!("Generating ramps...");

    let vox_count = grid.len();

    let mut rampifier = Rampify::new(
        (DEFAULT_LEN_X, DEFAULT_LEN_Y, DEFAULT_LEN_Z),
        grid,
        RampifyConfig::default()
    );

    use std::time::{Duration, Instant};
    let now = Instant::now();

    // Generate ramps for floor and ceiling.
    let ramps = &mut rampifier.generate_ramps(true);
    let ramps2 = &mut rampifier.generate_ramps(false);

    let ramp_count = ramps.len();
    let ramp2_count = ramps2.len();

    save.bricks.append(ramps);
    save.bricks.append(ramps2);

    println!(" - Processed {} voxels", vox_count);
    println!(" - Generated {} ramps in {}s\n", ramp_count + ramp2_count, now.elapsed().as_millis() as f64 / 1000.0);

    // Sets the voxels occupied by ramps to empty.
    rampifier.remove_occupied_voxels();

    // Move grid back out of the rampifier to do further processing.
    let mut grid = rampifier.move_grid();


    /////////////////////////////////////////////////////////////
    //         PASS 3: GENERATE OPTIMIZED BRICK FILL           //
    /////////////////////////////////////////////////////////////

    println!("Filling Gaps...");

    for z in 0..DEFAULT_LEN_Z {
        for y in 0..DEFAULT_LEN_Y {
            for x in 0..DEFAULT_LEN_X {
                let mut brick = Brick::default();

                if grid[get_index((x, y, z))] == u8::MAX {
                    continue
                }

                let mut w = 1;
                let mut l = 1;
                let mut h = 1;

                // todo: this can be done way better, but this is a shitty quick way to optimize bricks
                while can_box(&grid, &(x, y, z), &(w, l, h)) && w <= 64 {
                    w += 1;
                }

                w -= 1;

                if w > 0 {
                    while can_box(&grid, &(x, y, z), &(w, l, h)) && l <= 64 {
                        l += 1;
                    }

                    l -= 1;

                    if l > 0 {
                        while can_box(&grid, &(x, y, z), &(w, l, h)) && h <= 64 {
                            h += 1;
                        }

                        h -= 1;

                        if h > 0 {
                            box_remove(&mut grid, &(x, y, z), &(w, l, h));

                            let size = (w as u32 * 5, l as u32 * 5, h as u32 * 2);
                            {
                                let (x, y, z) = (x as i32 * 10, y as i32 * 10, z as i32 * 4);

                                brick.position = (x + size.0 as i32, y + size.1 as i32, z + size.2 as i32);
                                brick.size = Size::Procedural(size.0, size.1, size.2);
                            }

                            brick.color = BrickColor::Unique(Color {
                                r: ((x as f32 / DEFAULT_LEN_X as f32) * 255.0) as u8,
                                g: ((y as f32 / DEFAULT_LEN_Y as f32) * 255.0) as u8,
                                b: ((z as f32 / DEFAULT_LEN_Z as f32) * 255.0) as u8,
                                a: 255,
                            });

                            save.bricks.push(brick);
                        }
                    }
                }
            }
        }
    }

    println!(" - Gaps filled.");

    // write out the save
    let file = File::create(save_location);

    match file {
        Ok(file) => {
            SaveWriter::new(file, save)
                .write()
                .unwrap();

            println!("Save written to {}", save_location);
        },
        Err(error) => {
            println!("Could not write to {}, {}", save_location, error.to_string())
        }
    }
}
