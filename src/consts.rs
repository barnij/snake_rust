pub const GRID_SIZE: (i16, i16) = (30, 20);
pub const GRID_CELL_SIZE: (i16, i16) = (40, 40);

pub const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

pub const UPDATES_PER_SECOND: f32 = 10.0;
pub const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;
