use std::ops::Range;

// Map
pub const MAP_HEIGHT: f32 = 5000.0;
pub const MAP_WIDTH: f32 = 5000.0;

// Borders
pub const BORDER_THICKNESS: f32 = 100.0;

// Player
pub const PLAYER_SPEED: f32 = 5000.;
pub const PLAYER_HEALTH: f32 = 9.99;

// Enemy
pub const ENEMY_SPEED: f32 = 0.75;
pub const ENEMY_HEALTH: f32 = 75.0;
// pub const ENEMY_DAMAGE: f32 = 10.0;
pub const SPAWN_RADIUS: Range<f32> = 1000.0..2000.0;

// Experience Mechanic
pub const BASE_ENEMY_XP: u32 = 5;
pub const EXPERIENCE_SPEED: f32 = 100.;
pub const EXPERIENCE_RADIUS: f32 = 200.;
