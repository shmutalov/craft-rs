#![allow(dead_code)]

// App parameters
pub const WINDOW_WIDTH: u32 = 1024;
pub const WINDOW_HEIGHT: u32 = 768;
pub const VSYNC: bool = true;
pub const SCROLL_THRESHOLD: f64 = 0.1;
pub const MAX_MESSAGES: usize = 4;
pub const DB_PATH: &str = "craft.db";
pub const USE_CACHE: bool = true;
pub const DAY_LENGTH: i32 = 600;
pub const INVERT_MOUSE: bool = false;
pub const DEFAULT_PORT: i32 = 4080;

// Rendering options
pub const SHOW_LIGHTS: bool = true;
pub const SHOW_PLANTS: bool = true;
pub const SHOW_CLOUDS: bool = true;
pub const SHOW_TREES: bool = true;
pub const SHOW_ITEM: bool = true;
pub const SHOW_CROSSHAIRS: bool = true;
pub const SHOW_WIREFRAME: bool = true;
pub const SHOW_INFO_TEXT: bool = true;
pub const SHOW_CHAT_TEXT: bool = true;
pub const SHOW_PLAYER_NAMES: bool = true;

// Advanced parameters
pub const CREATE_CHUNK_RADIUS: i32 = 10;
pub const RENDER_CHUNK_RADIUS: i32 = 10;
pub const RENDER_SIGN_RADIUS: i32 = 4;
pub const DELETE_CHUNK_RADIUS: i32 = 14;
pub const CHUNK_SIZE: i32 = 32;
pub const COMMIT_INTERVAL: f64 = 5.0;

// Limits
pub const MAX_CHUNKS: usize = 8192;
pub const MAX_PLAYERS: usize = 128;
pub const WORKERS: usize = 4;
pub const MAX_TEXT_LENGTH: usize = 256;
pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_PATH_LENGTH: usize = 256;
pub const MAX_ADDR_LENGTH: usize = 256;
pub const MAX_SIGN_LENGTH: usize = 256;

// Alignment
pub const ALIGN_LEFT: i32 = 0;
pub const ALIGN_CENTER: i32 = 1;
pub const ALIGN_RIGHT: i32 = 2;

// Modes
pub const MODE_OFFLINE: i32 = 0;
pub const MODE_ONLINE: i32 = 1;

// Worker states
pub const WORKER_IDLE: i32 = 0;
pub const WORKER_BUSY: i32 = 1;
pub const WORKER_DONE: i32 = 2;

pub fn radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

pub fn degrees(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}
