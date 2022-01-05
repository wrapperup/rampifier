use std::{env, fs, fs::File};
use rampifier::{RampifierConfig, Rampifier};
use brickadia::{
    save::*,
    write::SaveWriter,
    read::SaveReader,
};

const DEFAULT_SAVE_PATH: &str = "./out.brs";
const DEFAULT_LOAD_PATH: &str = "./in.brs";

fn main() {
    let args: Vec<String> = env::args().collect();

    /////////////////////////////////////////////////////////////
    //                   CONSTANTS & CONFIG                    //
    /////////////////////////////////////////////////////////////

    let mut out_save_location = DEFAULT_SAVE_PATH;
    let mut in_save_location = DEFAULT_LOAD_PATH;

    if args.len() == 2
    {
        out_save_location = &args[1];
    }

    if args.len() > 2
    {
        out_save_location = &args[1];
        in_save_location = &args[2];
    }

    let public = User {
        name: "Terrain".into(),
        id: "a8033bee-6c37-4118-b4a6-cecc1d966132".parse().unwrap(),
    };

    let mut out_save = SaveData::default();

    // set the first header
    out_save.header1.author = public.clone();
    out_save.header1.host = Some(public.clone());
    out_save.header1.description = "This was rampified with rampifier. Save written with brickadia-rs.".into();

    // set the second header
    out_save.header2
        .brick_owners
        .push(BrickOwner::from_user_bricks(public.clone(), 100));

    out_save.header2.brick_assets =
        vec![
            "PB_DefaultBrick".into(),
            "PB_DefaultRamp".into(),
            "PB_DefaultWedge".into(),
            "PB_DefaultRampCrest".into(),
        ];

    // Read colors from sample save.

    /////////////////////////////////////////////////////////////
    //               PASS 1: LOAD BRICKS AS VOX                //
    /////////////////////////////////////////////////////////////

    let fix_brick_pos = |brick: &Brick| -> (i32, i32, i32) {
        let (mut x, mut  y, mut  z) = brick.position;
        if let Size::Procedural(mut w_half, mut l_half, mut h_half) = brick.size {
            x -= w_half as i32;
            y -= l_half as i32;
            z -= h_half as i32;

            x /= 10;
            y /= 10;
            z /= 4;

            return (x, y, z);
        }

        (0, 0, 0)
    };

    let file = File::open(in_save_location);

    let mut reader = match file {
        Ok(file) => {
            SaveReader::new(file).unwrap()
        },
        Err(error) => {
            panic!("Could not read {}, {}", in_save_location, error.to_string())
        }
    };

    let in_save = reader.read_all().unwrap();

    out_save.header2.colors = in_save.header2.colors;

    println!("Converting .brs into voxels...");

    use std::time::{Duration, Instant};
    let now = Instant::now();

    // Find bounds for bricks.
    let mut min_bounds = (i32::MAX, i32::MAX, i32::MAX);
    let mut max_bounds = (i32::MIN, i32::MIN, i32::MIN);

    for brick in &in_save.bricks {
        if let Size::Procedural(mut w_half, mut l_half, mut h_half) = brick.size {
            let w = w_half as i32 / 5;
            let l = l_half as i32 / 5;
            let h = h_half as i32 / 2;

            let pos = fix_brick_pos(&brick);

            min_bounds.0 = min_bounds.0.min(pos.0);
            min_bounds.1 = min_bounds.1.min(pos.1);
            min_bounds.2 = min_bounds.2.min(pos.2);

            let pos = (
                pos.0 + w + 1,
                pos.1 + l + 1,
                pos.2 + h + 1,
            );

            max_bounds.0 = max_bounds.0.max(pos.0);
            max_bounds.1 = max_bounds.1.max(pos.1);
            max_bounds.2 = max_bounds.2.max(pos.2);
        }
    }

    let grid_size = (
        (max_bounds.0 - min_bounds.0) as usize,
        (max_bounds.1 - min_bounds.1) as usize,
        (max_bounds.2 - min_bounds.2) as usize,
    );

    let get_index = |pos: (usize, usize, usize)| -> usize {
        pos.0 + pos.1 * grid_size.0 + pos.2 * grid_size.0 * grid_size.1
    };

    let mut grid: Vec<Option<u8>> = vec![None; grid_size.0 * grid_size.1 * grid_size.2];

    for brick in &in_save.bricks {
        if let Size::Procedural(mut w_half, mut l_half, mut h_half) = brick.size {
            let pos = fix_brick_pos(&brick);
            let pos = (
                (pos.0 - min_bounds.0) as usize,
                (pos.1 - min_bounds.1) as usize,
                (pos.2 - min_bounds.2) as usize,
            );

            let w = w_half as usize / 5;
            let l = l_half as usize / 5;
            let h = h_half as usize / 2;

            for i in 0..w {
                for j in 0..l {
                    for k in 0..h {
                        let pos = (pos.0 + i, pos.1 + j, pos.2 + k);

                        if let BrickColor::Index(index) = brick.color {
                            grid[get_index(pos)] = Some(index as u8);
                        }
                    }
                }
            }
        }
    }

    println!(" - Done in {}s\n", now.elapsed().as_millis() as f64 / 1000.0);


    let box_remove = |g: &mut Vec<Option<u8>>, pos: &(usize, usize, usize), size: &(usize, usize, usize)| {
        let &(x, y, z) = pos;
        let &(w, l, h) = size;

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    let p = (x + i, y + j, z + k);

                    g[get_index((p.0, p.1, p.2))] = None;
                }
            }
        }
    };

    let can_box = |g: &Vec<Option<u8>>, value: u8, pos: &(usize, usize, usize), size: &(usize, usize, usize)| -> bool {
        let &(w, l, h) = size;

        if pos.0 + w > grid_size.0 {
            return false;
        }
        if pos.1 + l > grid_size.1 {
            return false;
        }
        if pos.2 + h > grid_size.2 {
            return false;
        }

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    let pos = (pos.0 + i, pos.1 + j, pos.2 + k);
                    if g[get_index((pos.0, pos.1, pos.2))] != Some(value) {
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

    let mut rampifier = Rampifier::new(
        grid_size,
        grid,
        RampifierConfig::default()
    );

    use std::time::{Duration, Instant};
    let now = Instant::now();

    // Generate ramps for floor and ceiling.
    let ramps = &mut rampifier.generate_ramps(true);
    let ramps2 = &mut rampifier.generate_ramps(false);

    let ramp_count = ramps.len();
    let ramp2_count = ramps2.len();

    out_save.bricks.append(ramps);
    out_save.bricks.append(ramps2);

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

    for z in 0..grid_size.2 {
        for y in 0..grid_size.1 {
            for x in 0..grid_size.0 {
                let mut brick = Brick::default();

                if let Some(val) = grid[get_index((x, y, z))] {
                    let mut w = 1;
                    let mut l = 1;
                    let mut h = 1;

                    // todo: this can be done way better, but this is a shitty quick way to optimize bricks
                    while can_box(&grid, val, &(x, y, z), &(w, l, h)) && h <= 64 {
                        h += 1;
                    }

                    h -= 1;

                    if h > 0 {
                        while can_box(&grid, val, &(x, y, z), &(w, l, h)) && w <= 64 {
                            w += 1;
                        }

                        w -= 1;

                        if w > 0 {
                            while can_box(&grid, val, &(x, y, z), &(w, l, h)) && l <= 64 {
                                l += 1;
                            }

                            l -= 1;

                            if l > 0 {
                                box_remove(&mut grid, &(x, y, z), &(w, l, h));

                                let size = (w as u32 * 5, l as u32 * 5, h as u32 * 2);
                                {
                                    let (x, y, z) = (x as i32 * 10, y as i32 * 10, z as i32 * 4);

                                    brick.position = (x + size.0 as i32, y + size.1 as i32, z + size.2 as i32);
                                    brick.size = Size::Procedural(size.0, size.1, size.2);
                                }

                                brick.color = BrickColor::Index(val as u32);

                                out_save.bricks.push(brick);
                            }
                        }
                    }
                }
            }
        }
    }

    println!(" - Gaps filled.");

    for mut brick in &mut out_save.bricks {
        brick.position.0 += min_bounds.0 * 10;
        brick.position.1 += min_bounds.1 * 10;
        brick.position.2 += min_bounds.2 * 4;
    }

    // write out the save
    let file = File::create(out_save_location);

    match file {
        Ok(file) => {
            SaveWriter::new(file, out_save)
                .write()
                .unwrap();

            println!("Save written to {}", out_save_location);
        },
        Err(error) => {
            println!("Could not write to {}, {}", out_save_location, error.to_string())
        }
    }
}
