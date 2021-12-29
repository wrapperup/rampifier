use brickadia::save::{Brick, BrickColor, Color, Size, Direction, Rotation};
use std::vec;
use std::ops;

#[derive(Copy, Clone, Debug)]
pub struct VoxVector (pub isize, pub isize, pub isize);

impl ops::Add<VoxVector> for VoxVector {
    type Output = VoxVector;

    fn add(self, rhs: VoxVector) -> Self::Output {
        VoxVector(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl ops::AddAssign<VoxVector> for VoxVector {
    fn add_assign(&mut self, rhs: VoxVector) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl ops::Sub<VoxVector> for VoxVector {
    type Output = VoxVector;

    fn sub(self, rhs: VoxVector) -> Self::Output {
        VoxVector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl ops::SubAssign<VoxVector> for VoxVector {
    fn sub_assign(&mut self, rhs: VoxVector) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl ops::Mul<VoxVector> for VoxVector {
    type Output = VoxVector;

    fn mul(self, rhs: VoxVector) -> Self::Output {
        VoxVector(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl ops::Mul<usize> for VoxVector {
    type Output = VoxVector;

    fn mul(self, rhs: usize) -> Self::Output {
        VoxVector(self.0 * rhs as isize, self.1 * rhs as isize, self.2 * rhs as isize)
    }
}

impl VoxVector {
    fn forward_vec(rot: Rotation) -> VoxVector {
        match rot {
            Rotation::Deg0 => VoxVector(-1, 0, 0),
            Rotation::Deg180 => VoxVector(1, 0, 0),
            Rotation::Deg90 => VoxVector(0, -1, 0),
            Rotation::Deg270 => VoxVector(0, 1, 0),
        }
    }
}

pub struct RampifyConfig {
    // How wide the ramp can be TODO: This doesn't do anything yet
    pub ramp_max_width: usize,

    // How long the ramp can be, in bricks.
    pub ramp_max_run: usize,

    // How high the ramp can go, in plates.
    pub ramp_max_rise: usize,

    // How high the wedge can go, in plates.
    pub wedge_max_rise: usize,

    // How many steps should the height increase? Eg. a value of 3 would only allow 1 brick high ramps.
    pub ramp_min_height: usize,

    // How many steps should the wedge increase? Eg. a value of 3 would only allow 1 brick high ramps.
    pub wedge_step_height: usize,

    // Index of the cube brick to use. Usually PB_DefaultBrick, but it can be any kind of cubic brick.
    pub brick_index: u32,

    // Index of the ramp brick. Usually PB_DefaultRamp, but it can be any kind of ramp.
    pub ramp_index: u32,

    // Index of the ramp brick. Usually PB_DefaultWedge, but it can be any kind of ramp/wedge.
    pub wedge_index: u32,

    // The size of a single brick when converting voxels to brick.
    // This must be set correctly or this will generate invalid brick sizes.
    pub brick_size: (u32, u32, u32)
}

impl Default for RampifyConfig {
    fn default() -> Self {
        Self {
            ramp_max_width: 2,
            ramp_max_run: 4,
            ramp_max_rise: 12,
            wedge_max_rise: 2,
            ramp_min_height: 2,
            wedge_step_height: 2,
            brick_index: 0,
            ramp_index: 1,
            wedge_index: 2,
            brick_size: (5, 5, 2),
        }
    }
}

impl RampifyConfig {
    pub fn new(
        ramp_max_width: usize,
        ramp_max_run: usize,
        ramp_max_rise: usize,
        wedge_max_rise: usize,
        ramp_step_height: usize,
        wedge_step_height: usize,
        brick_index: u32,
        ramp_index: u32,
        wedge_index: u32,
        brick_size: (u32, u32, u32),
    ) -> Self {
        Self {
            ramp_max_width,
            ramp_max_run,
            ramp_max_rise,
            wedge_max_rise,
            ramp_min_height: ramp_step_height,
            wedge_step_height,
            brick_index,
            ramp_index,
            wedge_index,
            brick_size,
        }
    }

    pub fn micro(micro_cube_index: u32, micro_ramp_index: u32) -> Self {
        Self {
            ramp_max_width: 2,
            ramp_max_run: 4,
            ramp_max_rise: 12,
            wedge_max_rise: 12,
            ramp_min_height: 1,
            wedge_step_height: 1,
            brick_index: micro_cube_index,
            ramp_index: micro_ramp_index,
            wedge_index: micro_ramp_index,
            brick_size: (1, 1, 1),
        }
    }

    pub fn x4cube(micro_cube_index: u32, micro_ramp_index: u32) -> Self {
        Self {
            ramp_max_width: 2,
            ramp_max_run: 4,
            ramp_max_rise: 12,
            wedge_max_rise: 12,
            ramp_min_height: 1,
            wedge_step_height: 1,
            brick_index: micro_cube_index,
            ramp_index: micro_ramp_index,
            wedge_index: micro_ramp_index,
            brick_size: (20, 20, 20),
        }
    }
}

pub struct Rampify {
    // Size of this voxel grid.
    size: (usize, usize, usize),

    // The Voxel grid, contains u8 corresponding to the brick's color id. u8::MAX = none.
    grid: Vec<u8>,

    // Configuration settings to alter how ramps are generated.
    config: RampifyConfig,
}

impl Rampify {
    pub fn new(size: (usize, usize, usize), grid: Vec<u8>, config: RampifyConfig) -> Self {
        let (w, l, h) = size;

        Self {
            size: (w, l, h),
            grid,
            config,
        }
    }

    pub fn move_grid(self) -> Vec<u8> {
        self.grid
    }

    pub fn with_config(mut self, config: RampifyConfig) -> Self {
        self.config = config;
        self
    }

    fn grid_index(&self, pos: (usize, usize, usize)) -> usize {
        pos.0 + pos.1 * self.size.0 as usize + pos.2  * self.size.0 as usize * self.size.1 as usize
    }

    fn set_point(&mut self, pos: (usize, usize, usize), value: u8) {
        let index = self.grid_index((pos.0, pos.1, pos.2));
        self.grid[index] = value;
    }

    fn get_point(&self, pos: (usize, usize, usize)) -> u8 {
        self.grid[self.grid_index((pos.0, pos.1, pos.2))]
    }

    // Is this point within the grid?
    fn is_bounded(&self, pos: VoxVector) -> bool {
        let VoxVector(x, y, z) = pos;
        let (w, l, h) = self.size;

        x >= 0 && y >= 0 && z >= 0 &&
            x < w as isize && y < l as isize && z < h as isize
    }

    fn set_point_safe(&mut self, pos: VoxVector, value: u8) {
        if self.is_bounded(pos) {
            self.set_point((pos.0 as usize, pos.1 as usize, pos.2 as usize), value);
        }
    }

    fn get_point_safe(&self, pos: VoxVector) -> u8 {
        if self.is_bounded(pos) {
            return self.get_point((pos.0 as usize, pos.1 as usize, pos.2 as usize));
        }
        u8::MAX
    }

    // This uses u8::MAX to identify an empty voxel. Should we use Option<u8> instead?
    fn vox_exists(&self, pos: VoxVector) -> bool {
        self.get_point_safe(pos) != u8::MAX
    }

    fn box_remove(&mut self, pos: VoxVector, size: VoxVector) {
        let VoxVector(x, y, z) = pos;
        let VoxVector(w, l, h) = size;

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    let p = VoxVector(x + i, y + j, z + k);
                    self.set_point_safe(p, 0);
                }
            }
        }
    }

    fn can_box(&self, pos: (usize, usize, usize), size: (usize, usize, usize)) -> bool {
        let x = pos.0 as isize;
        let y = pos.1 as isize;
        let z = pos.2 as isize;

        let w = size.0 as isize;
        let l = size.1 as isize;
        let h = size.2 as isize;

        for i in 0..w {
            for j in 0..l {
                for k in 0..h {
                    if !self.vox_exists(VoxVector(x + i, y + j, z + k)) {
                        return false;
                    }
                }
            }
        }

        true
    }

    // Returns change in height from test pt. This only goes upwards, since we scan from the bottom of the world.
    fn get_slope_from_offset(&self, pos: VoxVector, is_floor: bool) -> Option<i32> {
        // Invalid state, return none.
        if is_floor && !self.vox_exists(pos) || !is_floor && self.vox_exists(pos) {
            return None;
        }

        const MAX_SEARCH: usize = 32;

        for i in 1..MAX_SEARCH {
            let pos_y = pos + VoxVector(0, 0, i as isize);

            // Done is true if:
            // If we are searching from inside floor upwards and we hit air
            // If we are searching from below the ceiling upwards and hit a voxel.
            let done = is_floor && !self.vox_exists(pos_y) || !is_floor && self.vox_exists(pos_y);

            if done {
                return Some(i as i32);
            }
        }

        Some(MAX_SEARCH as i32 - 1)
    }

    // Returns length and height of a ramp.
    fn fit_ramp(&self, pos: VoxVector, rot: Rotation, is_floor: bool) -> Option<(usize, usize)> {
        let forward = VoxVector::forward_vec(rot);
        let up = if is_floor {
            VoxVector(0, 0,  1)
        }
        else {
            VoxVector(0, 0, -1)
        };

        let mut run = 0usize;
        let mut rise = 0usize;

        // Try increasing the run.
        for i in 0..self.config.ramp_max_run - 1 {
            // If the vox above is air (or below if ceiling), we continue running.
            if !self.vox_exists(pos + up + (forward * run)) && self.vox_exists(pos + (forward * (run + 1))) {
                run += 1;
            }
            else {
                // Can't run anymore, so break out.
                break;
            }
        }

        if run <= 0 {
            return None;
        }

        for i in 1..self.config.ramp_max_rise {
            // Rise until we hit the limit or we find air above (or below if ceiling).
            if self.vox_exists(pos + (up * (rise)) + (forward * (run))) {
                // We've rose too long, this won't be valid.
                if rise == self.config.ramp_max_rise {
                    return None;
                }
                rise += 1;
            }
            else {
                break;
            }
        }

        // BODGE? Perhaps it's a limitation of this algorithm... I'm down for suggestions.
        // As long as there is air ahead of us, we can guarantee there is no "ramp chaining",
        // so we can fully cover the top of the slope smoothly.
        let mut add_one = 0;
        for i in 1..self.config.ramp_max_run {
            let pos = pos + (up * (rise)) + (forward * (run));
            let pos = pos + forward * i;

            // Is there air above and ahead of this ramp?
            if !self.vox_exists(pos) {
                add_one = 1;
            }
            else {
                add_one = 0;
                break;
            }
        }

        rise += add_one;

        if rise <= 0 {
            return None;
        }

        if rise < self.config.ramp_min_height {
            return None;
        }

        Some((run as usize + 1, rise as usize - 1))
    }

    // Returns the direction and rotation best suited for this point heuristically.
    fn best_ramp_rotation(&self, pos: VoxVector, is_floor: bool) -> Option<Rotation> {
        // Floor must have air above it.
        if !self.vox_exists(pos + VoxVector(0, 0, 1)) && !self.vox_exists(pos + VoxVector(0, 0, -1)) {
            return None;
        }

        // Ceiling must have air below it.
        if false {
            return None;
        }

        /* This table combines together orientations and their forward
         * and back vectors.
         *
         * F = Forward Vector     B = Backwards Vector
         * O = Origin
         *
         *         *----*__
         *   F     |    |   \          B
         * <---    |    |      \      --->
         *         |    |         \
         *         |____|_______[O]_|
         *
         *         .____________[O]_.
         *         |    |          /
         *   F     |    |       /      B
         * <---    |    |    /        --->
         *         *----*---
         */
        const DIR_ROT_HEIGHT_TABLE: [(VoxVector, VoxVector, Rotation); 4] = [
            (
                VoxVector(-1,  0,  0), // forward
                VoxVector( 1,  0,  0), // back
                Rotation::Deg0,
            ),
            (
                VoxVector( 1,  0,  0), // forward
                VoxVector(-1,  0,  0), // back
                Rotation::Deg180,
            ),
            (
                VoxVector( 0, -1,  0), // forward
                VoxVector( 0,  1,  0), // back
                Rotation::Deg90,
            ),
            (
                VoxVector( 0,  1,  0), // forward
                VoxVector( 0, -1,  0), // back
                Rotation::Deg270,
            ),
        ];

        // Stored "slopes" for each direction.
        let mut heights: [Option<i32>; DIR_ROT_HEIGHT_TABLE.len()] = [None; DIR_ROT_HEIGHT_TABLE.len()];

        /* Valid rotations are ones who have air behind them.
         * They must also be either a ceiling or floor voxel to
         * get to this state.
         *
         * Hopefully this crappy drawing can help:
         *
         * C = voxel candidate   A = air    *---- = voxel cell.
         *
         * *----*----*
         * |    |  C |  A        -> If this case is true, then the orientation
         * *----*----*----*         that points left will be valid.
         * |    |    |    |
         * *----*----*----*
         */
        let mut valid_indexes: [bool; 4] = [false; 4];

        for i in 0..DIR_ROT_HEIGHT_TABLE.len() {
            let pos = pos + DIR_ROT_HEIGHT_TABLE[i].1;
            valid_indexes[i] = !self.vox_exists(pos);
        }

        // Populate height differences. Basically, slope.
        // If it is not a floor (instead, a ceiling) it will search for air instead of vox.
        for i in 0..valid_indexes.len() {
            if valid_indexes[i] {
                heights[i] = self.get_slope_from_offset(pos + DIR_ROT_HEIGHT_TABLE[i].0, is_floor);
            }
        }

        let mut max_height = 0;
        let mut best_rotation: Option<Rotation> = None;

        // We prefer larger slopes.
        for i in 0..heights.len() {
            let rot = DIR_ROT_HEIGHT_TABLE[i].2.clone();

            if let Some(height) = heights[i] {
                if max_height < height {
                    max_height = height;
                    best_rotation = Some(rot);
                }
            }
        }

        if let Some(rotation) = best_rotation {
            return Some(rotation);
        }

        // No need to do anything, continue.
        None
    }

    /* This offsets the position so that the end of the ramp is where the origin is.
     * O = Origin
     *
     * *----*__
     * |    |   \
     * |    |      \
     * |    |         \
     * |____|_______[O]_|
     *
     * .____________[O]_.
     * |    |          /
     * |    |       /
     * |    |    /
     * *----*---
     */
    fn offset_pos(pos: (i32, i32, i32), size: (u32, u32, u32), rot: Rotation) -> (i32, i32, i32) {
        let w = size.0 as i32;
        let l = size.1 as i32;
        let h = size.2 as i32;

        let mut pos = pos;

        match rot {
            Rotation::Deg0 => {
                pos = (pos.0 - w + 10, pos.1 + l, pos.2 + h);
            }
            Rotation::Deg90 => {
                pos = (pos.0 + l, pos.1 - w + 10, pos.2 + h);
            }
            Rotation::Deg180 => {
                pos = (pos.0 + w, pos.1 + l, pos.2 + h);
            }
            Rotation::Deg270 => {
                pos = (pos.0 + l, pos.1 + w, pos.2 + h);
            }
        }

        pos
    }

    fn create_ramp(
        &mut self,
        pos: VoxVector,
        run: usize,
        rise: usize,
        rotation: Rotation,
        color: BrickColor,
        delete_space: bool,
        is_floor: bool
    ) -> Brick {
        let mut ramp = Brick::default();

        let x = pos.0 as i32;
        let y = pos.1 as i32;
        let z = pos.2 as i32;

        let brick_w = self.config.brick_size.0;
        let brick_l = self.config.brick_size.1;
        let brick_h = self.config.brick_size.2;

        // Select ramp or wedge depending on the size.
        ramp.asset_name_index = if run < 2 { self.config.wedge_index } else { self.config.ramp_index };
        ramp.rotation = rotation;
        ramp.position = (x * brick_w as i32 * 2, y * brick_l as i32 * 2, z * brick_h as i32 * 2);
        ramp.color = color;

        {
            let run = run as u32;
            let rise = rise as u32;
            ramp.size = Size::Procedural(run * brick_w, 1 * brick_l, rise * brick_h);
        }

        if let Size::Procedural(w, l, h) = ramp.size {
            let size = (w, l, h);
            ramp.position = Self::offset_pos(ramp.position, size, ramp.rotation.clone());

            if !is_floor {
                ramp.position.2 -= h as i32 * 2 - brick_h as i32;
            }
        }

        println!("Ramp     pos: {:?}    size: {:?}", ramp.position, ramp.size);

        if delete_space {
            //self.box_remove(pos, size);
        }

        ramp
    }

    // Process voxel grid and return ramps generated by the algorithm.
    pub fn generate_ramps(&mut self, gen_floor_else_ceil: bool) -> Vec<Brick> {
        let mut ramps: Vec<Brick> = Vec::new();

        // Estimate amount to reserve, prevent allocations where possible.
        // Based on testing, ramps count for < ~2% of voxels, but this can vary.
        let est: usize = ((self.size.0 * self.size.1 * self.size.2) as f32 * 0.02) as usize;
        ramps.reserve(est);

        let w = self.size.0 as isize;
        let l = self.size.0 as isize;
        let h = self.size.0 as isize;

        for z in 0..w {
            for y in 0..l {
                for x in 0..h {
                    let pos = VoxVector(x, y, z);

                    // Is there a voxel here?
                    if self.vox_exists(pos) {
                        // Is there a candidate for a ramp?
                        if let Some(rot) = self.best_ramp_rotation(pos, gen_floor_else_ceil) {
                            if let Some((run, rise)) = self.fit_ramp(pos, rot.clone(), gen_floor_else_ceil) {
                                let color = BrickColor::Unique(Color {
                                    r: ((x as f32 / w as f32) * 255.0) as u8,
                                    g: ((y as f32 / l as f32) * 255.0) as u8,
                                    b: ((z as f32 / h as f32) * 255.0) as u8,
                                    a: 255
                                });

                                let ramp = self.create_ramp(pos, run, rise, rot, color, true, true);
                                ramps.push(ramp);
                            }
                        }
                    }
                }
            }
        }

        ramps
    }
}