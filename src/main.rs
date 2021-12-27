use std::{fs::File};

use brickadia::{
    save::{Brick, BrickColor, BrickOwner, Color, SaveData, Size, User},
    write::SaveWriter,
};
use brickadia::save::Rotation;
use brickadia::save::Rotation::{Deg0, Deg270, Deg90};
use noise::{RidgedMulti, Seedable};

fn main() {
    const LEN_X: usize = 50;
    const LEN_Y: usize = 50;
    const LEN_Z: usize = 100;

    let mut grid: Vec<Vec<Vec<u8>>> = vec![vec![vec![0u8; LEN_Z]; LEN_Y]; LEN_X];

    const MAX_BRICK_SIZE: usize = 100;

    const RAMP_LIM_WIDTH: usize = 2; // bricks
    const RAMP_LIM_RUN: usize = 6; // bricks
    const RAMP_LIM_HEIGHT: usize = 12; // plates

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

    use noise::{NoiseFn, OpenSimplex};

    let simplex = OpenSimplex::new();
    let ridged = RidgedMulti::new();

    println!("Generating voxel data...");

    for y in 0..LEN_Y {
        for x in 0..LEN_X {
            for z in 0..LEN_Z {
                // populate da grid

                let scale = 0.01;

                let val = simplex.get([
                    x as f64 * scale,
                    y as f64 * scale,
                    z as f64 * scale / 3.0
                ]);

                let val = val + ridged.get([
                    x as f64 * scale * 2.0,
                    y as f64 * scale * 2.0,
                    z as f64 * scale / 3.0
                ]) * 0.2;

                let val = (val + 0.3) - ((z as f64 * 0.01) + 0.0);

                let val = val + 0.5;
                let val = (val * 255.0) as u8;

                grid[x][y][z] = val;
            }
        }
    }
    println!(" - Done!");


    let mut vox_exists = |g: &Vec<Vec<Vec<u8>>>, pos: (usize, usize, usize)| -> bool {
        let (x, y, z) = pos;

        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        if x >= LEN_X || y >= LEN_Y || z >= LEN_Z {
            return false;
        }

        g[x][y][z] >= 64
    };

    let is_safe_test = |pos: &(usize, usize, usize)| -> bool {
        let &(x, y, z) = pos;

        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        if x >= LEN_X || y >= LEN_Y || z >= LEN_Z {
            return false;
        }

        return true;
    };

    let box_remove = |g: &mut Vec<Vec<Vec<u8>>>, pos: &(usize, usize, usize), size: &(usize, usize, usize)|
        {
            let &(x, y, z) = pos;
            let &(w, l, h) = size;

            for i in 0..w {
                for j in 0..l {
                    for k in 0..h {
                        let p = (x + i, y + j, z + k);

                        if(is_safe_test(&p)) {
                            g[p.0][p.1][p.2] = 0;
                        }
                    }
                }
            }
        };

    let can_box = |g: &Vec<Vec<Vec<u8>>>, pos: &(usize, usize, usize), size: &(usize, usize, usize)| -> bool {
        let &(x, y, z) = pos;
        let &(w, l, h) = size;

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    if !vox_exists(&g, (x + i, y + j, z + k)) {
                        return false;
                    }
                }
            }
        }

        return true;
    };

    /*let grid_2 = grid.clone();

    // visualize end result w/o ramp
    for y in 0..LEN_Y {
        for x in 0..LEN_X {
            for z in 0..LEN_Z {
                let mut brick = Brick::default();

                if !vox_exists(&grid, (x, y, z)) {
                    continue
                }

                let mut w = 1;
                let mut l = 1;
                let mut h = 1;

                while can_box(&grid, &(x, y, z), &(w, l, h)) {
                    h += 1;
                }

                h -= 1;

                while can_box(&grid, &(x, y, z), &(w, l, h)) {
                    w += 1;
                }

                w -= 1;

                while can_box(&grid, &(x, y, z), &(w, l, h)) {
                    l += 1;
                }

                l -= 1;

                box_remove(&mut grid, &(x, y, z), &(w, l, h));

                if w > 0 && l > 0 && h > 0 {
                    println!("Brickj  p: ({},{},{})   s: ({},{},{})", &x, &y, &z, &w, &l, &h);

                    let size = (w as u32 * 5, l as u32 * 5, h as u32 * 2);
                    let (x, y, z) = (x as i32 * 10, y as i32 * 10, z as i32 * 4);

                    brick.position = (x + size.0 as i32, y + size.1 as i32, z + size.2 as i32);
                    brick.size = Size::Procedural(size.0, size.1, size.2);
                    brick.color = BrickColor::Unique(Color {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    });

                    brick.position.0 += ((LEN_X + 1) * 10) as i32;

                    save.bricks.push(brick);
                }
            }
        }
    }

    grid = grid_2;*/

    // fit suitable ramps

    let mut ramps = 0;
    println!("\nTesting for rampability...");

    for z in 0..LEN_Z {
        for y in 0..LEN_Y {
            for x in 0..LEN_X {
                // we'll test for rampability now
                if vox_exists(&grid, (x, y, z)) {

                    // if there is air above this...
                    if !vox_exists(&grid, (x, y, z + 1)) {
                        let mut ramp_run = 1;
                        let mut ramp_height = 1;

                        let mut break_when_finished = false;
                        let mut should_place_ramp = false;

                        let mut place_backwards = false;
                        let mut do_it_sideways = false;

                        // Lets try it normally, and if we don't have a good fit we will try it sideways.
                        if !do_it_sideways {
                            if vox_exists(&grid, (x + 1, y, z + 1)) {
                                for i in 0..32 {
                                    let mut pt = (x + ramp_run - 1, y, z + ramp_height - 1);
                                    let mut test_pt_forward = (x + ramp_run, y, z + ramp_height - 1);
                                    let mut test_pt_air = (x + ramp_run - 1, y, z + ramp_height);

                                    should_place_ramp = true;

                                    if vox_exists(&grid, pt) && !vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_RUN {
                                        if break_when_finished {
                                            break;
                                        }
                                        ramp_run += 1;
                                    } else if vox_exists(&grid, pt) && vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_HEIGHT {
                                        break_when_finished = true;
                                        ramp_height += 1;
                                    }
                                }
                            }

                            if !break_when_finished {
                                should_place_ramp = false;
                            }

                            if !should_place_ramp {
                                ramp_run = 1;
                                ramp_height = 1;

                                break_when_finished = false;

                                place_backwards = true;
                                for i in 0..32 {
                                    if ramp_run > x {
                                        break;
                                    }
                                    if ramp_height > z {
                                        break;
                                    }

                                    let mut pt = (x - ramp_run + 1, y, z + ramp_height - 1);
                                    let mut test_pt_forward = (x - ramp_run, y, z + ramp_height - 1);
                                    let mut test_pt_air = (x - ramp_run + 1, y, z + ramp_height);

                                    should_place_ramp = true;

                                    if vox_exists(&grid, pt) && !vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_RUN {
                                        if break_when_finished {
                                            break;
                                        }
                                        ramp_run += 1;
                                    } else if vox_exists(&grid, pt) && vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_HEIGHT {
                                        break_when_finished = true;
                                        ramp_height += 1;
                                    }
                                }
                            }

                            if !break_when_finished {
                                should_place_ramp = false;

                                // Nothing worked, so fuck it doe. Admin, he's doing it sideways.
                                do_it_sideways = true;
                            }
                        }

                        // was the other way not a good fit? or is doing it sideways heuristically better?
                        if do_it_sideways {
                            place_backwards = false;
                            break_when_finished = false;
                            should_place_ramp = false;

                            ramp_run = 1;
                            ramp_height = 1;

                            if vox_exists(&grid, (x, y + 1, z + 1)) {
                                for i in 0..32 {
                                    let mut pt = (x, y + ramp_run - 1, z + ramp_height - 1);
                                    let mut test_pt_forward = (x, y + ramp_run, z + ramp_height - 1);
                                    let mut test_pt_air = (x, y + ramp_run - 1, z + ramp_height);

                                    should_place_ramp = true;

                                    if vox_exists(&grid, pt) && !vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_RUN {
                                        if break_when_finished {
                                            break;
                                        }
                                        ramp_run += 1;
                                    } else if vox_exists(&grid, pt) && vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_HEIGHT {
                                        break_when_finished = true;
                                        ramp_height += 1;
                                    }
                                }
                            }

                            if !break_when_finished {
                                should_place_ramp = false;
                            }

                            if !should_place_ramp {
                                ramp_run = 1;
                                ramp_height = 1;

                                break_when_finished = false;

                                place_backwards = true;
                                for i in 0..32 {
                                    if ramp_run > y {
                                        break;
                                    }
                                    if ramp_height > z {
                                        break;
                                    }

                                    let mut pt = (x, y - ramp_run + 1, z + ramp_height - 1);
                                    let mut test_pt_forward = (x, y - ramp_run, z + ramp_height - 1);
                                    let mut test_pt_air = (x, y - ramp_run + 1, z + ramp_height);

                                    should_place_ramp = true;

                                    if vox_exists(&grid, pt) && !vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_RUN {
                                        if break_when_finished {
                                            break;
                                        }
                                        ramp_run += 1;
                                    } else if vox_exists(&grid, pt) && vox_exists(&grid, test_pt_air) && ramp_run < RAMP_LIM_HEIGHT {
                                        break_when_finished = true;
                                        ramp_height += 1;
                                    }
                                }
                            }
                        }

                        let mut rotation = Rotation::Deg180;
                        if place_backwards {
                            rotation = Rotation::Deg0;
                        }

                        if do_it_sideways {
                            if place_backwards {
                                rotation = Rotation::Deg90;
                            }
                            else {
                                rotation = Rotation::Deg270;
                            }
                        }

                        if !break_when_finished {
                            should_place_ramp = false;
                        }

                        if should_place_ramp {
                            let ramp_run = ramp_run as u32;
                            let ramp_height = ramp_height as u32;

                            let mut ramp = Brick::default();
                            ramp.asset_name_index = if ramp_run < 2 { 2 } else { 1 };
                            ramp.rotation = rotation;

                            match ramp.rotation {
                                Rotation::Deg0 => {
                                    ramp.size = Size::Procedural(ramp_run * 5, 5, ramp_height * 2);
                                    ramp.position = ((x - 1) as i32 * 10 + ramp_run as i32 * 5, y as i32 * 10 + 5, z as i32 * 4 + ramp_height as i32 * 2);
                                },
                                Rotation::Deg180 => {
                                    ramp.size = Size::Procedural(ramp_run * 5, 5, ramp_height * 2);
                                    ramp.position = (x as i32 * 10 + ramp_run as i32 * 5, y as i32 * 10 + 5, z as i32 * 4 + ramp_height as i32 * 2);
                                },
                                Rotation::Deg90 => {
                                    ramp.size = Size::Procedural(ramp_run * 5, 5, ramp_height * 2);
                                    ramp.position = (x as i32 * 10 + 5, (y - 1) as i32 * 10 + ramp_run as i32 * 5, z as i32 * 4 + ramp_height as i32 * 2);
                                },
                                Rotation::Deg270 => {
                                    ramp.size = Size::Procedural(ramp_run * 5, 5, ramp_height * 2);
                                    ramp.position = (x as i32 * 10 + 5, y as i32 * 10 + ramp_run as i32 * 5, z as i32 * 4 + ramp_height as i32 * 2);
                                }
                            }

                            ramp.color = BrickColor::Unique(Color {
                                r: ((x as f32 / LEN_X as f32) * 255.0) as u8,
                                g: ((y as f32 / LEN_Y as f32) * 255.0) as u8,
                                b: ((z as f32 / LEN_Z as f32) * 255.0) as u8,
                                a: 255,
                            });

                            save.bricks.push(ramp);
                            ramps += 1;

                            if !do_it_sideways {
                                let x = if place_backwards { x - 1 } else { x };
                                box_remove(&mut grid, &(x, y, z), &(ramp_run as usize, 1, ramp_height as usize));
                            }
                            else {
                                let y = if place_backwards { y - 1 } else { y };
                                box_remove(&mut grid, &(x, y, z), &(1, ramp_run as usize, ramp_height as usize));
                            }
                        }
                    }
                }
            }
        }
    }

    println!(" - Ramps added: {}", ramps);

    println!("\nFilling & Optimizing gaps...");

    // fill in gaps optimally
    for z in 0..LEN_Z {
        for y in 0..LEN_Y {
            for x in 0..LEN_X {
                let mut brick = Brick::default();

                if !vox_exists(&grid, (x, y, z)) {
                    continue
                }

                let mut w = 1;
                let mut l = 1;
                let mut h = 1;

                // todo: this can be done way better, but this is a shitty quick way to optimize bricks
                while can_box(&grid, &(x, y, z), &(w, l, h)) && h <= MAX_BRICK_SIZE {
                    h += 1;
                }

                h -= 1;

                while can_box(&grid, &(x, y, z), &(w, l, h)) && w <= MAX_BRICK_SIZE {
                    w += 1;
                }

                w -= 1;

                while can_box(&grid, &(x, y, z), &(w, l, h)) && l <= MAX_BRICK_SIZE {
                    l += 1;
                }

                l -= 1;

                box_remove(&mut grid, &(x, y, z), &(w, l, h));

                if w > 0 && l > 0 && h > 0 {
                    let size = (w as u32 * 5, l as u32 * 5, h as u32 * 2);
                    {
                        let (x, y, z) = (x as i32 * 10, y as i32 * 10, z as i32 * 4);

                        brick.position = (x + size.0 as i32, y + size.1 as i32, z + size.2 as i32);
                        brick.size = Size::Procedural(size.0, size.1, size.2);
                    }

                    brick.color = BrickColor::Unique(Color {
                        r: ((x as f32 / LEN_X as f32) * 255.0) as u8,
                        g: ((y as f32 / LEN_Y as f32) * 255.0) as u8,
                        b: ((z as f32 / LEN_Z as f32) * 255.0) as u8,
                        a: 255,
                    });

                    save.bricks.push(brick);
                }
            }
        }
    }

    println!(" - Gaps filled.");

    // write out the save
    let save_location = "C:/Users/wrapp/AppData/Local/Brickadia/Saved/Builds/out.brs";
    let file = File::create(save_location);

    match file {
        Ok(file) => {
            SaveWriter::new(file, save)
                .write()
                .unwrap();

            println!("Save written");
        },
        Err(error) => {
            println!("{}", error.to_string())
        }
    }
}