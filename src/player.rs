#![allow(dead_code)]

use crate::config::*;
use crate::item::is_obstacle;
use crate::map::Map;

#[derive(Clone, Copy, Default)]
pub struct State {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
    pub t: f64,
}

#[derive(Clone)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub state: State,
    pub state1: State,
    pub state2: State,
    pub buffer: GLuint,
}

use portablegl::gl_types::GLuint;

impl Player {
    pub fn new() -> Self {
        Player {
            id: 0,
            name: String::new(),
            state: State::default(),
            state1: State::default(),
            state2: State::default(),
            buffer: 0,
        }
    }
}

pub fn get_sight_vector(rx: f32, ry: f32) -> (f32, f32, f32) {
    let m = ry.cos();
    let vx = (rx - radians(90.0)).cos() * m;
    let vy = ry.sin();
    let vz = (rx - radians(90.0)).sin() * m;
    (vx, vy, vz)
}

pub fn get_motion_vector(flying: bool, sz: i32, sx: i32, rx: f32, ry: f32) -> (f32, f32, f32) {
    if sz == 0 && sx == 0 {
        return (0.0, 0.0, 0.0);
    }
    let strafe = (sz as f32).atan2(sx as f32);
    if flying {
        let mut m = ry.cos();
        let mut y = ry.sin();
        if sx != 0 {
            if sz == 0 {
                y = 0.0;
            }
            m = 1.0;
        }
        if sz > 0 {
            y = -y;
        }
        let vx = (rx + strafe).cos() * m;
        let vz = (rx + strafe).sin() * m;
        (vx, y, vz)
    } else {
        let vx = (rx + strafe).cos();
        let vz = (rx + strafe).sin();
        (vx, 0.0, vz)
    }
}

pub fn collide(height: i32, x: &mut f32, y: &mut f32, z: &mut f32, map: &Map) -> bool {
    let mut result = false;
    let nx = x.round() as i32;
    let ny = y.round() as i32;
    let nz = z.round() as i32;
    let px = *x - nx as f32;
    let py = *y - ny as f32;
    let pz = *z - nz as f32;
    let pad = 0.25;
    for dy in 0..height {
        if px < -pad && is_obstacle(map.get(nx - 1, ny - dy, nz)) {
            *x = nx as f32 - pad;
        }
        if px > pad && is_obstacle(map.get(nx + 1, ny - dy, nz)) {
            *x = nx as f32 + pad;
        }
        if py < -pad && is_obstacle(map.get(nx, ny - dy - 1, nz)) {
            *y = ny as f32 - pad;
            result = true;
        }
        if py > pad && is_obstacle(map.get(nx, ny - dy + 1, nz)) {
            *y = ny as f32 + pad;
            result = true;
        }
        if pz < -pad && is_obstacle(map.get(nx, ny - dy, nz - 1)) {
            *z = nz as f32 - pad;
        }
        if pz > pad && is_obstacle(map.get(nx, ny - dy, nz + 1)) {
            *z = nz as f32 + pad;
        }
    }
    result
}

pub fn player_intersects_block(height: i32, x: f32, y: f32, z: f32, hx: i32, hy: i32, hz: i32) -> bool {
    let nx = x.round() as i32;
    let ny = y.round() as i32;
    let nz = z.round() as i32;
    for i in 0..height {
        if nx == hx && ny - i == hy && nz == hz {
            return true;
        }
    }
    false
}

pub fn interpolate_player(player: &mut Player, now: f64) {
    let s1 = player.state1;
    let s2 = player.state2;
    let mut t1 = s2.t - s1.t;
    let t2 = now - s2.t;
    t1 = t1.min(1.0).max(0.1);
    let p = (t2 / t1).min(1.0) as f32;
    player.state.x = s1.x + (s2.x - s1.x) * p;
    player.state.y = s1.y + (s2.y - s1.y) * p;
    player.state.z = s1.z + (s2.z - s1.z) * p;
    player.state.rx = s1.rx + (s2.rx - s1.rx) * p;
    player.state.ry = s1.ry + (s2.ry - s1.ry) * p;
}
