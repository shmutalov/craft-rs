#![allow(dead_code)]

use noise::{NoiseFn, Simplex};
use crate::config::*;

fn simplex_seed() -> u32 {
    // Deterministic seed for reproducible worlds
    0
}

fn simplex2(x: f64, z: f64, octaves: i32, persistence: f64, lacunarity: f64) -> f64 {
    let noise = Simplex::new(simplex_seed());
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;
    for _ in 0..octaves {
        total += noise.get([x * frequency, z * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    total / max_value
}

fn simplex3(x: f64, y: f64, z: f64, octaves: i32, persistence: f64, lacunarity: f64) -> f64 {
    let noise = Simplex::new(simplex_seed());
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;
    for _ in 0..octaves {
        total += noise.get([x * frequency, y * frequency, z * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    total / max_value
}

pub fn create_world<F: FnMut(i32, i32, i32, i32)>(p: i32, q: i32, mut func: F) {
    let pad = 1;
    for dx in -pad..(CHUNK_SIZE + pad) {
        for dz in -pad..(CHUNK_SIZE + pad) {
            let flag: i32 = if dx < 0 || dz < 0 || dx >= CHUNK_SIZE || dz >= CHUNK_SIZE {
                -1
            } else {
                1
            };
            let x = p * CHUNK_SIZE + dx;
            let z = q * CHUNK_SIZE + dz;
            let f = simplex2(x as f64 * 0.01, z as f64 * 0.01, 4, 0.5, 2.0);
            let g = simplex2(-x as f64 * 0.01, -z as f64 * 0.01, 2, 0.9, 2.0);
            let mh = g * 32.0 + 16.0;
            let h = (f * mh) as i32;
            let mut w = 1; // grass
            let t = 12;
            let h = if h <= t {
                w = 2; // sand
                t
            } else {
                h
            };
            // terrain fill
            for y in 0..h {
                func(x, y, z, w * flag);
            }
            if w == 1 {
                if SHOW_PLANTS {
                    // grass
                    if simplex2(-x as f64 * 0.1, z as f64 * 0.1, 4, 0.8, 2.0) > 0.6 {
                        func(x, h, z, 17 * flag);
                    }
                    // flowers
                    if simplex2(x as f64 * 0.05, -z as f64 * 0.05, 4, 0.8, 2.0) > 0.7 {
                        let fw = 18 + (simplex2(x as f64 * 0.1, z as f64 * 0.1, 4, 0.8, 2.0) * 7.0) as i32;
                        func(x, h, z, fw * flag);
                    }
                }
                // trees
                let mut ok = SHOW_TREES;
                if dx - 4 < 0 || dz - 4 < 0 || dx + 4 >= CHUNK_SIZE || dz + 4 >= CHUNK_SIZE {
                    ok = false;
                }
                if ok && simplex2(x as f64, z as f64, 6, 0.5, 2.0) > 0.84 {
                    for y in (h + 3)..(h + 8) {
                        for ox in -3..=3 {
                            for oz in -3..=3 {
                                let d = ox * ox + oz * oz + (y - (h + 4)) * (y - (h + 4));
                                if d < 11 {
                                    func(x + ox, y, z + oz, 15); // leaves
                                }
                            }
                        }
                    }
                    for y in h..(h + 7) {
                        func(x, y, z, 5); // wood
                    }
                }
            }
            // clouds
            if SHOW_CLOUDS {
                for y in 64..72 {
                    if simplex3(x as f64 * 0.01, y as f64 * 0.1, z as f64 * 0.01, 8, 0.5, 2.0) > 0.75 {
                        func(x, y, z, 16 * flag);
                    }
                }
            }
        }
    }
}
