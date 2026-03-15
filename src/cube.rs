#![allow(dead_code)]

use crate::item::{BLOCKS, PLANTS};

const POSITIONS: [[[f32; 3]; 4]; 6] = [
    [[-1.0, -1.0, -1.0], [-1.0, -1.0, 1.0], [-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0]],
    [[1.0, -1.0, -1.0], [1.0, -1.0, 1.0], [1.0, 1.0, -1.0], [1.0, 1.0, 1.0]],
    [[-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0], [1.0, 1.0, -1.0], [1.0, 1.0, 1.0]],
    [[-1.0, -1.0, -1.0], [-1.0, -1.0, 1.0], [1.0, -1.0, -1.0], [1.0, -1.0, 1.0]],
    [[-1.0, -1.0, -1.0], [-1.0, 1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0]],
    [[-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0]],
];

const NORMALS: [[f32; 3]; 6] = [
    [-1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, -1.0],
    [0.0, 0.0, 1.0],
];

const UVS: [[[f32; 2]; 4]; 6] = [
    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
    [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    [[0.0, 1.0], [0.0, 0.0], [1.0, 1.0], [1.0, 0.0]],
    [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
    [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
    [[1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [0.0, 1.0]],
];

const INDICES: [[usize; 6]; 6] = [
    [0, 3, 2, 0, 1, 3],
    [0, 3, 1, 0, 2, 3],
    [0, 3, 2, 0, 1, 3],
    [0, 3, 1, 0, 2, 3],
    [0, 3, 2, 0, 1, 3],
    [0, 3, 1, 0, 2, 3],
];

const FLIPPED: [[usize; 6]; 6] = [
    [0, 1, 2, 1, 3, 2],
    [0, 2, 1, 2, 3, 1],
    [0, 1, 2, 1, 3, 2],
    [0, 2, 1, 2, 3, 1],
    [0, 1, 2, 1, 3, 2],
    [0, 2, 1, 2, 3, 1],
];

/// Generate cube face geometry into `data`.
/// Each vertex: position(3) + normal(3) + uv(2) + ao(1) + light(1) = 10 floats
/// Returns number of floats written.
pub fn make_cube_faces(
    data: &mut [f32],
    ao: &[[f32; 4]; 6],
    light: &[[f32; 4]; 6],
    faces: [bool; 6],
    tiles: [i32; 6],
    x: f32, y: f32, z: f32, n: f32,
) -> usize {
    let s: f32 = 0.0625;
    let a: f32 = 1.0 / 2048.0;
    let b: f32 = s - 1.0 / 2048.0;
    let mut offset = 0;
    for i in 0..6 {
        if !faces[i] {
            continue;
        }
        let du = (tiles[i] % 16) as f32 * s;
        let dv = (tiles[i] / 16) as f32 * s;
        let flip = (ao[i][0] + ao[i][3]) > (ao[i][1] + ao[i][2]);
        for v in 0..6 {
            let j = if flip { FLIPPED[i][v] } else { INDICES[i][v] };
            data[offset] = x + n * POSITIONS[i][j][0]; offset += 1;
            data[offset] = y + n * POSITIONS[i][j][1]; offset += 1;
            data[offset] = z + n * POSITIONS[i][j][2]; offset += 1;
            data[offset] = NORMALS[i][0]; offset += 1;
            data[offset] = NORMALS[i][1]; offset += 1;
            data[offset] = NORMALS[i][2]; offset += 1;
            data[offset] = du + if UVS[i][j][0] != 0.0 { b } else { a }; offset += 1;
            data[offset] = dv + if UVS[i][j][1] != 0.0 { b } else { a }; offset += 1;
            data[offset] = ao[i][j]; offset += 1;
            data[offset] = light[i][j]; offset += 1;
        }
    }
    offset
}

pub fn make_cube(
    data: &mut [f32],
    ao: &[[f32; 4]; 6],
    light: &[[f32; 4]; 6],
    faces: [bool; 6],
    x: f32, y: f32, z: f32, n: f32, w: i32,
) -> usize {
    let tiles = BLOCKS[w as usize];
    make_cube_faces(data, ao, light, faces, tiles, x, y, z, n)
}

const PLANT_POSITIONS: [[[f32; 3]; 4]; 4] = [
    [[0.0, -1.0, -1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, 1.0, 1.0]],
    [[0.0, -1.0, -1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, 1.0, 1.0]],
    [[-1.0, -1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0]],
    [[-1.0, -1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0]],
];

const PLANT_NORMALS: [[f32; 3]; 4] = [
    [-1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 0.0, -1.0],
    [0.0, 0.0, 1.0],
];

const PLANT_UVS: [[[f32; 2]; 4]; 4] = [
    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
    [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
    [[1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [0.0, 1.0]],
];

const PLANT_INDICES: [[usize; 6]; 4] = [
    [0, 3, 2, 0, 1, 3],
    [0, 3, 1, 0, 2, 3],
    [0, 3, 2, 0, 1, 3],
    [0, 3, 1, 0, 2, 3],
];

pub fn make_plant(
    data: &mut [f32],
    ao: f32, light: f32,
    px: f32, py: f32, pz: f32,
    n: f32, w: i32, rotation: f32,
) -> usize {
    let s: f32 = 0.0625;
    let a: f32 = 0.0;
    let b: f32 = s;
    let du = (PLANTS[w as usize] % 16) as f32 * s;
    let dv = (PLANTS[w as usize] / 16) as f32 * s;

    let mut offset = 0;
    for i in 0..4 {
        for v in 0..6 {
            let j = PLANT_INDICES[i][v];
            data[offset] = n * PLANT_POSITIONS[i][j][0]; offset += 1;
            data[offset] = n * PLANT_POSITIONS[i][j][1]; offset += 1;
            data[offset] = n * PLANT_POSITIONS[i][j][2]; offset += 1;
            data[offset] = PLANT_NORMALS[i][0]; offset += 1;
            data[offset] = PLANT_NORMALS[i][1]; offset += 1;
            data[offset] = PLANT_NORMALS[i][2]; offset += 1;
            data[offset] = du + if PLANT_UVS[i][j][0] != 0.0 { b } else { a }; offset += 1;
            data[offset] = dv + if PLANT_UVS[i][j][1] != 0.0 { b } else { a }; offset += 1;
            data[offset] = ao; offset += 1;
            data[offset] = light; offset += 1;
        }
    }

    // Apply rotation around Y axis, then translate
    let rot_rad = rotation.to_radians();
    let cos_r = rot_rad.cos();
    let sin_r = rot_rad.sin();
    // 24 vertices, stride 10, rotate normals at offset 3, positions at offset 0
    for v in 0..24 {
        let base = v * 10;
        // Rotate normals
        let nx = data[base + 3];
        let nz = data[base + 5];
        data[base + 3] = cos_r * nx + sin_r * nz;
        data[base + 5] = -sin_r * nx + cos_r * nz;
        // Rotate + translate positions
        let vx = data[base];
        let vz = data[base + 2];
        data[base]     = cos_r * vx + sin_r * vz + px;
        data[base + 1] += py;
        data[base + 2] = -sin_r * vx + cos_r * vz + pz;
    }

    offset
}

pub fn make_player(
    data: &mut [f32],
    x: f32, y: f32, z: f32, rx: f32, ry: f32,
) -> usize {
    let ao = [[0.0f32; 4]; 6];
    let light = [[0.8f32; 4]; 6];
    let tiles = [226, 224, 241, 209, 225, 227];
    let faces = [true; 6];
    let count = make_cube_faces(data, &ao, &light, faces, tiles, 0.0, 0.0, 0.0, 0.4);

    // Rotate and translate: 36 vertices, stride 10
    let cos_rx = rx.cos();
    let sin_rx = rx.sin();
    let cos_ry = ry.cos();
    let sin_ry = ry.sin();
    let num_verts = count / 10;
    for v in 0..num_verts {
        let base = v * 10;
        // Rotate normals around Y
        let nx = data[base + 3];
        let nz = data[base + 5];
        data[base + 3] = cos_rx * nx + sin_rx * nz;
        data[base + 5] = -sin_rx * nx + cos_rx * nz;
        // Rotate positions around Y then translate
        let vx = data[base];
        let vy = data[base + 1];
        let vz = data[base + 2];
        let rx2 = cos_rx * vx + sin_rx * vz;
        let rz = -sin_rx * vx + cos_rx * vz;
        // Rotate around tilted axis for ry
        let ry2 = cos_ry * vy - sin_ry * rz;
        let rz2 = sin_ry * vy + cos_ry * rz;
        data[base]     = rx2 + x;
        data[base + 1] = ry2 + y;
        data[base + 2] = rz2 + z;
    }

    count
}

pub fn make_cube_wireframe(data: &mut [f32], x: f32, y: f32, z: f32, n: f32) -> usize {
    let positions: [[f32; 3]; 8] = [
        [-1.0, -1.0, -1.0], [-1.0, -1.0, 1.0],
        [-1.0, 1.0, -1.0],  [-1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0],  [1.0, -1.0, 1.0],
        [1.0, 1.0, -1.0],   [1.0, 1.0, 1.0],
    ];
    let indices: [usize; 24] = [
        0, 1, 0, 2, 0, 4, 1, 3,
        1, 5, 2, 3, 2, 6, 3, 7,
        4, 5, 4, 6, 5, 7, 6, 7,
    ];
    let mut offset = 0;
    for &idx in &indices {
        data[offset] = x + n * positions[idx][0]; offset += 1;
        data[offset] = y + n * positions[idx][1]; offset += 1;
        data[offset] = z + n * positions[idx][2]; offset += 1;
    }
    offset
}

pub fn make_character(
    data: &mut [f32], x: f32, y: f32, n: f32, m: f32, c: char,
) -> usize {
    let s: f32 = 0.0625;
    let a = s;
    let b = s * 2.0;
    let w = (c as i32) - 32;
    let du = (w % 16) as f32 * a;
    let dv = 1.0 - (w / 16) as f32 * b - b;
    let mut offset = 0;
    // 6 vertices for 2 triangles forming a quad
    let verts: [(f32, f32, f32, f32); 6] = [
        (x - n, y - m, du,     dv),
        (x + n, y - m, du + a, dv),
        (x + n, y + m, du + a, dv + b),
        (x - n, y - m, du,     dv),
        (x + n, y + m, du + a, dv + b),
        (x - n, y + m, du,     dv + b),
    ];
    for (vx, vy, vu, vv) in &verts {
        data[offset] = *vx; offset += 1;
        data[offset] = *vy; offset += 1;
        data[offset] = *vu; offset += 1;
        data[offset] = *vv; offset += 1;
    }
    offset
}

const CHAR3D_POSITIONS: [[[[f32; 3]; 6]; 1]; 0] = []; // Will be filled if needed

pub fn make_character_3d(
    data: &mut [f32], x: f32, y: f32, z: f32, n: f32, face: i32, c: char,
) -> usize {
    let positions: [[[f32; 3]; 6]; 8] = [
        [[ 0.0, -2.0, -1.0], [ 0.0,  2.0,  1.0], [ 0.0,  2.0, -1.0],
         [ 0.0, -2.0, -1.0], [ 0.0, -2.0,  1.0], [ 0.0,  2.0,  1.0]],
        [[ 0.0, -2.0, -1.0], [ 0.0,  2.0,  1.0], [ 0.0, -2.0,  1.0],
         [ 0.0, -2.0, -1.0], [ 0.0,  2.0, -1.0], [ 0.0,  2.0,  1.0]],
        [[-1.0, -2.0,  0.0], [ 1.0,  2.0,  0.0], [ 1.0, -2.0,  0.0],
         [-1.0, -2.0,  0.0], [-1.0,  2.0,  0.0], [ 1.0,  2.0,  0.0]],
        [[-1.0, -2.0,  0.0], [ 1.0, -2.0,  0.0], [ 1.0,  2.0,  0.0],
         [-1.0, -2.0,  0.0], [ 1.0,  2.0,  0.0], [-1.0,  2.0,  0.0]],
        [[-1.0,  0.0,  2.0], [ 1.0,  0.0,  2.0], [ 1.0,  0.0, -2.0],
         [-1.0,  0.0,  2.0], [ 1.0,  0.0, -2.0], [-1.0,  0.0, -2.0]],
        [[-2.0,  0.0,  1.0], [ 2.0,  0.0, -1.0], [-2.0,  0.0, -1.0],
         [-2.0,  0.0,  1.0], [ 2.0,  0.0,  1.0], [ 2.0,  0.0, -1.0]],
        [[ 1.0,  0.0,  2.0], [-1.0,  0.0, -2.0], [-1.0,  0.0,  2.0],
         [ 1.0,  0.0,  2.0], [ 1.0,  0.0, -2.0], [-1.0,  0.0, -2.0]],
        [[ 2.0,  0.0, -1.0], [-2.0,  0.0,  1.0], [ 2.0,  0.0,  1.0],
         [ 2.0,  0.0, -1.0], [-2.0,  0.0, -1.0], [-2.0,  0.0,  1.0]],
    ];
    let uvs: [[[f32; 2]; 6]; 8] = [
        [[0.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 1.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
        [[0.0, 1.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
        [[0.0, 1.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
    ];
    let offsets: [[f32; 3]; 8] = [
        [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
    ];

    if face < 0 || face >= 8 { return 0; }
    let fi = face as usize;

    let s: f32 = 0.0625;
    let pu = s / 5.0;
    let pv = s / 2.5;
    let u1 = pu;
    let v1 = pv;
    let u2 = s - pu;
    let v2 = s * 2.0 - pv;
    let p = 0.5f32;
    let w = (c as i32) - 32;
    let du = (w % 16) as f32 * s;
    let dv = 1.0 - (w / 16 + 1) as f32 * s * 2.0;
    let fx = x + p * offsets[fi][0];
    let fy = y + p * offsets[fi][1];
    let fz = z + p * offsets[fi][2];

    let mut off = 0;
    for i in 0..6 {
        data[off] = fx + n * positions[fi][i][0]; off += 1;
        data[off] = fy + n * positions[fi][i][1]; off += 1;
        data[off] = fz + n * positions[fi][i][2]; off += 1;
        data[off] = du + if uvs[fi][i][0] != 0.0 { u2 } else { u1 }; off += 1;
        data[off] = dv + if uvs[fi][i][1] != 0.0 { v2 } else { v1 }; off += 1;
    }
    off
}

fn normalize_vec(v: &mut [f32; 3]) {
    let d = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v[0] /= d;
    v[1] /= d;
    v[2] /= d;
}

fn make_sphere_recursive(
    data: &mut [f32], offset: &mut usize,
    r: f32, detail: i32,
    a: [f32; 3], b: [f32; 3], c: [f32; 3],
    ta: [f32; 2], tb: [f32; 2], tc: [f32; 2],
) -> i32 {
    if detail == 0 {
        let verts = [(a, ta), (b, tb), (c, tc)];
        for (pos, uv) in &verts {
            data[*offset] = pos[0] * r; *offset += 1;
            data[*offset] = pos[1] * r; *offset += 1;
            data[*offset] = pos[2] * r; *offset += 1;
            data[*offset] = pos[0]; *offset += 1;
            data[*offset] = pos[1]; *offset += 1;
            data[*offset] = pos[2]; *offset += 1;
            data[*offset] = uv[0]; *offset += 1;
            data[*offset] = uv[1]; *offset += 1;
        }
        return 1;
    }

    let mut ab = [(a[0]+b[0])/2.0, (a[1]+b[1])/2.0, (a[2]+b[2])/2.0];
    let mut ac = [(a[0]+c[0])/2.0, (a[1]+c[1])/2.0, (a[2]+c[2])/2.0];
    let mut bc = [(b[0]+c[0])/2.0, (b[1]+c[1])/2.0, (b[2]+c[2])/2.0];
    normalize_vec(&mut ab);
    normalize_vec(&mut ac);
    normalize_vec(&mut bc);
    let tab = [0.0, 1.0 - ab[1].acos() / std::f32::consts::PI];
    let tac = [0.0, 1.0 - ac[1].acos() / std::f32::consts::PI];
    let tbc = [0.0, 1.0 - bc[1].acos() / std::f32::consts::PI];

    let mut total = 0;
    total += make_sphere_recursive(data, offset, r, detail-1, a, ab, ac, ta, tab, tac);
    total += make_sphere_recursive(data, offset, r, detail-1, b, bc, ab, tb, tbc, tab);
    total += make_sphere_recursive(data, offset, r, detail-1, c, ac, bc, tc, tac, tbc);
    total += make_sphere_recursive(data, offset, r, detail-1, ab, bc, ac, tab, tbc, tac);
    total
}

/// Generate sphere geometry. Returns number of triangles.
/// Each vertex: position(3) + normal(3) + uv(2) = 8 floats, 3 verts per tri.
pub fn make_sphere(data: &mut [f32], r: f32, detail: i32) -> i32 {
    let idx: [[usize; 3]; 8] = [
        [4, 3, 0], [1, 4, 0],
        [3, 4, 5], [4, 1, 5],
        [0, 3, 2], [0, 2, 1],
        [5, 2, 3], [5, 1, 2],
    ];
    let pos: [[f32; 3]; 6] = [
        [0.0, 0.0, -1.0], [1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0], [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],  [0.0, 0.0, 1.0],
    ];
    let uvs: [[f32; 2]; 6] = [
        [0.0, 0.5], [0.0, 0.5],
        [0.0, 0.0], [0.0, 0.5],
        [0.0, 1.0], [0.0, 0.5],
    ];
    let mut total = 0;
    let mut offset = 0;
    for i in 0..8 {
        let n = make_sphere_recursive(
            data, &mut offset, r, detail,
            pos[idx[i][0]], pos[idx[i][1]], pos[idx[i][2]],
            uvs[idx[i][0]], uvs[idx[i][1]], uvs[idx[i][2]],
        );
        total += n;
    }
    total
}
