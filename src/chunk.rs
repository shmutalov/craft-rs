#![allow(dead_code)]

use crate::config::*;
use crate::cube;
use crate::item::*;
use crate::map::Map;
use crate::render;
use crate::sign::SignList;
use crate::world;
use portablegl::gl_context::GlContext;
use portablegl::gl_types::*;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub struct Chunk {
    pub map: Map,
    pub lights: Map,
    pub signs: SignList,
    pub p: i32,
    pub q: i32,
    pub faces: i32,
    pub sign_faces: i32,
    pub dirty: bool,
    pub miny: i32,
    pub maxy: i32,
    pub buffer: GLuint,
    pub sign_buffer: GLuint,
}

impl Chunk {
    pub fn new(p: i32, q: i32) -> Self {
        let dx = p * CHUNK_SIZE - 1;
        let dy = 0;
        let dz = q * CHUNK_SIZE - 1;
        Chunk {
            map: Map::new(dx, dy, dz, 0x7fff),
            lights: Map::new(dx, dy, dz, 0xf),
            signs: SignList::new(),
            p,
            q,
            faces: 0,
            sign_faces: 0,
            dirty: true,
            miny: 0,
            maxy: 0,
            buffer: 0,
            sign_buffer: 0,
        }
    }
}

pub fn chunked(x: f32) -> i32 {
    (x.round() / CHUNK_SIZE as f32).floor() as i32
}

pub fn chunk_distance(chunk_p: i32, chunk_q: i32, p: i32, q: i32) -> i32 {
    let dp = (chunk_p - p).abs();
    let dq = (chunk_q - q).abs();
    dp.max(dq)
}

// Occlusion / AO computation
const XZ_SIZE: usize = (CHUNK_SIZE as usize) * 3 + 2;
const XZ_LO: usize = CHUNK_SIZE as usize;
const XZ_HI: usize = CHUNK_SIZE as usize * 2 + 1;
const Y_SIZE: usize = 258;

fn xyz(x: usize, y: usize, z: usize) -> usize {
    y * XZ_SIZE * XZ_SIZE + x * XZ_SIZE + z
}

fn xz(x: usize, z: usize) -> usize {
    x * XZ_SIZE + z
}

fn light_fill(
    opaque: &[u8], light: &mut [u8],
    x: i32, y: i32, z: i32, w: i32, force: bool,
) {
    let xu = x as usize;
    let yu = y as usize;
    let zu = z as usize;
    if x + w < XZ_LO as i32 || z + w < XZ_LO as i32 { return; }
    if x - w > XZ_HI as i32 || z - w > XZ_HI as i32 { return; }
    if y < 0 || y >= Y_SIZE as i32 { return; }
    if xu >= XZ_SIZE || zu >= XZ_SIZE { return; }
    let idx = xyz(xu, yu, zu);
    if idx >= opaque.len() { return; }
    if light[idx] >= w as u8 { return; }
    if !force && opaque[idx] != 0 { return; }
    light[idx] = w as u8;
    let w = w - 1;
    if w <= 0 { return; }
    light_fill(opaque, light, x - 1, y, z, w, false);
    light_fill(opaque, light, x + 1, y, z, w, false);
    light_fill(opaque, light, x, y - 1, z, w, false);
    light_fill(opaque, light, x, y + 1, z, w, false);
    light_fill(opaque, light, x, y, z - 1, w, false);
    light_fill(opaque, light, x, y, z + 1, w, false);
}

fn occlusion(
    neighbors: &[u8; 27], lights: &[u8; 27], shades: &[f32; 27],
    ao: &mut [[f32; 4]; 6], light_out: &mut [[f32; 4]; 6],
) {
    const LOOKUP3: [[[usize; 3]; 4]; 6] = [
        [[0, 1, 3], [2, 1, 5], [6, 3, 7], [8, 5, 7]],
        [[18, 19, 21], [20, 19, 23], [24, 21, 25], [26, 23, 25]],
        [[6, 7, 15], [8, 7, 17], [24, 15, 25], [26, 17, 25]],
        [[0, 1, 9], [2, 1, 11], [18, 9, 19], [20, 11, 19]],
        [[0, 3, 9], [6, 3, 15], [18, 9, 21], [24, 15, 21]],
        [[2, 5, 11], [8, 5, 17], [20, 11, 23], [26, 17, 23]],
    ];
    const LOOKUP4: [[[usize; 4]; 4]; 6] = [
        [[0,1,3,4], [1,2,4,5], [3,4,6,7], [4,5,7,8]],
        [[18,19,21,22], [19,20,22,23], [21,22,24,25], [22,23,25,26]],
        [[6,7,15,16], [7,8,16,17], [15,16,24,25], [16,17,25,26]],
        [[0,1,9,10], [1,2,10,11], [9,10,18,19], [10,11,19,20]],
        [[0,3,9,12], [3,6,12,15], [9,12,18,21], [12,15,21,24]],
        [[2,5,11,14], [5,8,14,17], [11,14,20,23], [14,17,23,26]],
    ];
    const CURVE: [f32; 4] = [0.0, 0.25, 0.5, 0.75];
    for i in 0..6 {
        for j in 0..4 {
            let corner = neighbors[LOOKUP3[i][j][0]] as i32;
            let side1 = neighbors[LOOKUP3[i][j][1]] as i32;
            let side2 = neighbors[LOOKUP3[i][j][2]] as i32;
            let value = if side1 != 0 && side2 != 0 { 3 } else { (corner + side1 + side2) as usize };
            let mut shade_sum = 0.0f32;
            let mut light_sum = 0.0f32;
            let is_light = lights[13] == 15;
            for k in 0..4 {
                shade_sum += shades[LOOKUP4[i][j][k]];
                light_sum += lights[LOOKUP4[i][j][k]] as f32;
            }
            if is_light {
                light_sum = 15.0 * 4.0 * 10.0;
            }
            let total = CURVE[value.min(3)] + shade_sum / 4.0;
            ao[i][j] = total.min(1.0);
            light_out[i][j] = light_sum / 15.0 / 4.0;
        }
    }
}

pub struct WorkerResult {
    pub p: i32,
    pub q: i32,
    pub miny: i32,
    pub maxy: i32,
    pub faces: i32,
    pub data: Vec<f32>,
    pub load: bool,
    pub block_map: Option<Map>,
    pub light_map: Option<Map>,
}

/// Compute chunk mesh from block maps. This is the heavy computation.
pub fn compute_chunk(
    p: i32, q: i32,
    block_maps: &[Option<Map>; 9], // 3x3 grid, index = (dp+1)*3 + (dq+1)
    light_maps: &[Option<Map>; 9],
) -> (i32, i32, i32, Vec<f32>) {
    let total_cells = XZ_SIZE * XZ_SIZE * Y_SIZE;
    let mut opaque = vec![0u8; total_cells];
    let mut light = vec![0u8; total_cells];
    let mut highest = vec![0u8; XZ_SIZE * XZ_SIZE];

    let ox = p * CHUNK_SIZE - CHUNK_SIZE - 1;
    let oy = -1i32;
    let oz = q * CHUNK_SIZE - CHUNK_SIZE - 1;

    // Check for lights
    let mut has_light = false;
    if SHOW_LIGHTS {
        for map_opt in light_maps.iter() {
            if let Some(map) = map_opt {
                if map.size > 0 {
                    has_light = true;
                    break;
                }
            }
        }
    }

    // Populate opaque array
    for map_opt in block_maps.iter() {
        if let Some(map) = map_opt {
            for (ex, ey, ez, ew) in map.iter() {
                let x = ex - ox;
                let y = ey - oy;
                let z = ez - oz;
                if x < 0 || y < 0 || z < 0 { continue; }
                let xu = x as usize;
                let yu = y as usize;
                let zu = z as usize;
                if xu >= XZ_SIZE || yu >= Y_SIZE || zu >= XZ_SIZE { continue; }
                opaque[xyz(xu, yu, zu)] = if !is_transparent(ew) { 1 } else { 0 };
                if opaque[xyz(xu, yu, zu)] != 0 {
                    highest[xz(xu, zu)] = highest[xz(xu, zu)].max(yu as u8);
                }
            }
        }
    }

    // Flood fill light
    if has_light {
        for map_opt in light_maps.iter() {
            if let Some(map) = map_opt {
                for (ex, ey, ez, ew) in map.iter() {
                    let x = ex - ox;
                    let y = ey - oy;
                    let z = ez - oz;
                    light_fill(&opaque, &mut light, x, y, z, ew, true);
                }
            }
        }
    }

    // Use center map (index 4 = (1)*3+(1))
    let center_map = block_maps[4].as_ref().unwrap();

    // Count faces
    let mut miny = 256i32;
    let mut maxy = 0i32;
    let mut face_count = 0;
    for (ex, ey, ez, ew) in center_map.iter() {
        if ew <= 0 { continue; }
        let x = (ex - ox) as usize;
        let y = (ey - oy) as usize;
        let z = (ez - oz) as usize;
        if x == 0 || y == 0 || z == 0 || x >= XZ_SIZE-1 || y >= Y_SIZE-1 || z >= XZ_SIZE-1 { continue; }
        let f1 = opaque[xyz(x-1, y, z)] == 0;
        let f2 = opaque[xyz(x+1, y, z)] == 0;
        let f3 = opaque[xyz(x, y+1, z)] == 0;
        let f4 = opaque[xyz(x, y-1, z)] == 0 && ey > 0;
        let f5 = opaque[xyz(x, y, z-1)] == 0;
        let f6 = opaque[xyz(x, y, z+1)] == 0;
        let total = f1 as i32 + f2 as i32 + f3 as i32 + f4 as i32 + f5 as i32 + f6 as i32;
        if total == 0 { continue; }
        let total = if is_plant(ew) { 4 } else { total };
        miny = miny.min(ey);
        maxy = maxy.max(ey);
        face_count += total;
    }

    // Generate geometry
    let mut data = render::malloc_faces(10, face_count as usize);
    let mut offset = 0usize;
    for (ex, ey, ez, ew) in center_map.iter() {
        if ew <= 0 { continue; }
        let x = (ex - ox) as usize;
        let y = (ey - oy) as usize;
        let z = (ez - oz) as usize;
        if x == 0 || y == 0 || z == 0 || x >= XZ_SIZE-1 || y >= Y_SIZE-1 || z >= XZ_SIZE-1 { continue; }
        let f1 = opaque[xyz(x-1, y, z)] == 0;
        let f2 = opaque[xyz(x+1, y, z)] == 0;
        let f3 = opaque[xyz(x, y+1, z)] == 0;
        let f4 = opaque[xyz(x, y-1, z)] == 0 && ey > 0;
        let f5 = opaque[xyz(x, y, z-1)] == 0;
        let f6 = opaque[xyz(x, y, z+1)] == 0;
        let total = f1 as i32 + f2 as i32 + f3 as i32 + f4 as i32 + f5 as i32 + f6 as i32;
        if total == 0 { continue; }

        // Compute AO neighbors
        let mut neighbors = [0u8; 27];
        let mut lights_n = [0u8; 27];
        let mut shades = [0.0f32; 27];
        let mut idx = 0;
        for ddx in -1i32..=1 {
            for ddy in -1i32..=1 {
                for ddz in -1i32..=1 {
                    let nx = (x as i32 + ddx) as usize;
                    let ny = (y as i32 + ddy) as usize;
                    let nz = (z as i32 + ddz) as usize;
                    if nx < XZ_SIZE && ny < Y_SIZE && nz < XZ_SIZE {
                        neighbors[idx] = opaque[xyz(nx, ny, nz)];
                        lights_n[idx] = light[xyz(nx, ny, nz)];
                        if (y as i32 + ddy) as u8 <= highest[xz(nx, nz)] {
                            for oy in 0..8 {
                                let test_y = ny + oy;
                                if test_y < Y_SIZE && opaque[xyz(nx, test_y, nz)] != 0 {
                                    shades[idx] = 1.0 - oy as f32 * 0.125;
                                    break;
                                }
                            }
                        }
                    }
                    idx += 1;
                }
            }
        }

        let mut ao = [[0.0f32; 4]; 6];
        let mut light_ao = [[0.0f32; 4]; 6];
        occlusion(&neighbors, &lights_n, &shades, &mut ao, &mut light_ao);

        if is_plant(ew) {
            let mut min_ao = 1.0f32;
            let mut max_light = 0.0f32;
            for a in 0..6 {
                for b in 0..4 {
                    min_ao = min_ao.min(ao[a][b]);
                    max_light = max_light.max(light_ao[a][b]);
                }
            }
            // Use simplex noise for rotation
            let rotation = {
                use noise::{NoiseFn, Simplex};
                let n = Simplex::new(0);
                let val = n.get([ex as f64 * 0.1, ez as f64 * 0.1]);
                (val * 360.0) as f32
            };
            let written = cube::make_plant(
                &mut data[offset..], min_ao, max_light,
                ex as f32, ey as f32, ez as f32, 0.5, ew, rotation,
            );
            offset += written;
        } else {
            let faces = [f1, f2, f3, f4, f5, f6];
            let written = cube::make_cube(
                &mut data[offset..], &ao, &light_ao, faces,
                ex as f32, ey as f32, ez as f32, 0.5, ew,
            );
            offset += written;
        }
    }

    (miny, maxy, face_count, data)
}

/// Load a chunk: generate terrain + load from DB
pub fn load_chunk_terrain(p: i32, q: i32, map: &mut Map) {
    world::create_world(p, q, |x, y, z, w| {
        map.set(x, y, z, w);
    });
}

pub struct WorkerPool {
    workers: Vec<WorkerHandle>,
}

struct WorkerHandle {
    state: Arc<Mutex<WorkerState>>,
    cond: Arc<Condvar>,
    _thread: thread::JoinHandle<()>,
}

struct WorkerState {
    busy: bool,
    done: bool,
    result: Option<WorkerResult>,
    task: Option<WorkerTask>,
}

struct WorkerTask {
    p: i32,
    q: i32,
    load: bool,
    block_maps: [Option<Map>; 9],
    light_maps: [Option<Map>; 9],
}

impl WorkerPool {
    pub fn new(count: usize) -> Self {
        let mut workers = Vec::with_capacity(count);
        for _i in 0..count {
            let state = Arc::new(Mutex::new(WorkerState {
                busy: false,
                done: false,
                result: None,
                task: None,
            }));
            let cond = Arc::new(Condvar::new());
            let s = state.clone();
            let c = cond.clone();
            let thread = thread::spawn(move || {
                worker_run(s, c);
            });
            workers.push(WorkerHandle {
                state,
                cond,
                _thread: thread,
            });
        }
        WorkerPool { workers }
    }

    pub fn is_idle(&self, index: usize) -> bool {
        let s = self.workers[index].state.lock().unwrap();
        !s.busy && !s.done
    }

    pub fn is_done(&self, index: usize) -> bool {
        let s = self.workers[index].state.lock().unwrap();
        s.done
    }

    pub fn take_result(&self, index: usize) -> Option<WorkerResult> {
        let mut s = self.workers[index].state.lock().unwrap();
        s.done = false;
        s.result.take()
    }

    pub fn submit(&self, index: usize, task: WorkerTask) {
        let mut s = self.workers[index].state.lock().unwrap();
        s.task = Some(task);
        s.busy = true;
        self.workers[index].cond.notify_one();
    }

    pub fn submit_chunk(
        &self, index: usize,
        p: i32, q: i32, load: bool,
        block_maps: [Option<Map>; 9],
        light_maps: [Option<Map>; 9],
    ) {
        self.submit(index, WorkerTask { p, q, load, block_maps, light_maps });
    }

    pub fn len(&self) -> usize {
        self.workers.len()
    }
}

fn worker_run(state: Arc<Mutex<WorkerState>>, cond: Arc<Condvar>) {
    loop {
        let task;
        {
            let mut s = state.lock().unwrap();
            while !s.busy {
                s = cond.wait(s).unwrap();
            }
            task = s.task.take().unwrap();
        }

        // Load terrain if needed
        let mut block_maps = task.block_maps;
        let mut light_maps = task.light_maps;
        if task.load {
            if let Some(ref mut map) = block_maps[4] {
                load_chunk_terrain(task.p, task.q, map);
            }
        }

        let (miny, maxy, faces, data) = compute_chunk(
            task.p, task.q, &block_maps, &light_maps,
        );

        let result = WorkerResult {
            p: task.p,
            q: task.q,
            miny,
            maxy,
            faces,
            data,
            load: task.load,
            block_map: if task.load { block_maps[4].take() } else { None },
            light_map: if task.load { light_maps[4].take() } else { None },
        };

        {
            let mut s = state.lock().unwrap();
            s.result = Some(result);
            s.busy = false;
            s.done = true;
        }
    }
}
