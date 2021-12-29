mod rampify;

use std::{fs::File};
use std::thread;

use vox_format;
use simdnoise::NoiseBuilder;

use brickadia::{
    save::{Brick, BrickColor, BrickOwner, Color, SaveData, Size, User},
    write::SaveWriter,
};
use brickadia::save::{Direction, Rotation};
use noise::{RidgedMulti, Seedable};
use crate::rampify::{Rampify, RampifyConfig};

fn main() {
    // Size of a chunk to be processed on a thread.
    const CHUNK_X_SIZE: usize = 64;
    const CHUNK_Y_SIZE: usize = 64;
    const CHUNK_Z_SIZE: usize = 64 * 3;

    // Length of grid vector.
    const LEN_X: usize = CHUNK_X_SIZE * 1;
    const LEN_Y: usize = CHUNK_Y_SIZE * 1;
    const LEN_Z: usize = CHUNK_Z_SIZE * 1;

    // This uses u8::MAX to identify an empty voxel. Should we use Option<u8> instead?
    let mut grid: Vec<u8> = vec![u8::MAX; LEN_X * LEN_Y * LEN_Z];

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
    let get_index = |pos: (usize, usize, usize)| -> usize {
        pos.0 + pos.1 * LEN_X + pos.2  * LEN_X * LEN_Y
    };

    for z in 0..LEN_Z {
        for y in 0..LEN_Y {
            for x in 0..LEN_X {
                // populate da grid

                let scale = 0.04;

                let val = simplex.get([
                    x as f64 * scale,
                    y as f64 * scale,
                    z as f64 * scale / 3.0
                ]);

                let val = val - ridged.get([
                    x as f64 * scale,
                    y as f64 * scale,
                    z as f64 * scale / 3.0
                ]) * 0.7;

                let val = (val + 0.3) - ((z as f64 * 0.06) + 0.0);

                let val = val + 0.5;
                let mut val = (val * 254.0) as u8;

                if val < 5 {
                    val = u8::MAX;
                }

                grid[get_index((x, y, z))] = val;
            }
        }
    }
    println!(" - Done!");

    println!("Generating ramps...");

    let mut rampifier = Rampify::new(
        (LEN_X, LEN_Y, LEN_Z),
        grid,
        RampifyConfig::default()
    );

    // Generate ramps
    let ramps = &mut rampifier.generate_ramps(true);

    let ramp_count = ramps.len();
    save.bricks.append(ramps);

    println!(" - Generated {} ramps", ramp_count);

    // Move grid back out of the rampifier to do further processing.
    let mut grid = rampifier.move_grid();

    println!("\nFilling Gaps...");

    for z in 0..LEN_Z {
        for y in 0..LEN_Y {
            for x in 0..LEN_X {
                if grid[get_index((x, y, z))] == u8::MAX {
                    continue
                }

                let mut brick = Brick::default();

                brick.position = (x as i32 * 10 + 5, y as i32 * 10 + 5, z as i32 * 4 + 2);
                brick.size = Size::Procedural(5, 5, 2);

                brick.color = BrickColor::Unique(Color {
                    r: ((x as f32 / LEN_X as f32) * 255.0) as u8,
                    g: ((y as f32 / LEN_X as f32) * 255.0) as u8,
                    b: ((z as f32 / LEN_Z as f32) * 255.0) as u8,
                    a: 255,
                });

                save.bricks.push(brick);
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