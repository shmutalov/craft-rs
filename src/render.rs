#![allow(dead_code)]

use portablegl::gl_context::GlContext;
use portablegl::gl_types::*;
use portablegl::math::*;
use crate::config::CHUNK_SIZE;

/// Set up a perspective projection + view matrix for 3D rendering.
pub fn set_matrix_3d(
    width: i32, height: i32,
    x: f32, y: f32, z: f32,
    rx: f32, ry: f32,
    fov: f32, ortho: i32,
    radius: i32,
) -> Mat4 {
    let aspect = width as f32 / height as f32;
    let znear = 0.125;
    let zfar = (radius * CHUNK_SIZE + CHUNK_SIZE) as f32 * 2.0;
    let matrix = if ortho != 0 {
        let size = ortho as f32 * aspect;
        make_orthographic_m4(-size, size, -(ortho as f32), ortho as f32, -zfar, zfar)
    } else {
        make_perspective_m4(radians(fov), aspect, znear, zfar)
    };
    // Build view matrix: translate, then pitch (ry), then yaw (-rx)
    // Note: Craft's mat_rotate produces R^T (inverse rotation), so angles are negated
    // compared to portablegl-rs's load_rotation_m4 which uses the standard formula.
    let trans = translation_m4(-x, -y, -z);
    let rot_pitch = load_rotation_m4(Vec3 { x: rx.cos(), y: 0.0, z: rx.sin() }, -ry);
    let rot_yaw = load_rotation_m4(Vec3 { x: 0.0, y: 1.0, z: 0.0 }, rx);

    mult_m4_m4(matrix, mult_m4_m4(rot_yaw, mult_m4_m4(rot_pitch, trans)))
}

/// Set up a 2D orthographic projection matrix.
pub fn set_matrix_2d(width: i32, height: i32) -> Mat4 {
    make_orthographic_m4(0.0, width as f32, 0.0, height as f32, -1.0, 1.0)
}

/// Set up the item preview matrix.
pub fn set_matrix_item(width: i32, height: i32, scale: i32) -> Mat4 {
    let aspect = width as f32 / height as f32;
    let size = 64.0 * scale as f32;
    let bx = height as f32 / size / 2.0;
    let xoffset = 1.0 - size / width as f32 * 2.0;
    let yoffset = 1.0 - size / height as f32 * 2.0;
    // Craft's mat_rotate is transposed, so negate angles for portablegl-rs
    // C: mat_rotate(Y, -PI/4) → effective +PI/4; mat_rotate(X, -PI/10) → effective +PI/10
    let rot_y = load_rotation_m4(Vec3 { x: 0.0, y: 1.0, z: 0.0 }, std::f32::consts::PI / 4.0);
    let rot_x = load_rotation_m4(Vec3 { x: 1.0, y: 0.0, z: 0.0 }, std::f32::consts::PI / 10.0);
    let ortho = make_orthographic_m4(-bx * aspect, bx * aspect, -bx, bx, -1.0, 1.0);
    let trans = translation_m4(-xoffset, -yoffset, 0.0);
    // Order: identity → rot_y → rot_x → ortho → translate
    mult_m4_m4(trans, mult_m4_m4(ortho, mult_m4_m4(rot_x, rot_y)))
}

/// Extract frustum planes from a matrix.
pub fn frustum_planes(_radius: i32, matrix: &Mat4) -> [[f32; 4]; 6] {
    let m = &matrix.0;
    let mut planes = [[0.0f32; 4]; 6];
    // Left
    planes[0] = [m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]];
    // Right
    planes[1] = [m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]];
    // Bottom
    planes[2] = [m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]];
    // Top
    planes[3] = [m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]];
    // Near
    planes[4] = [m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]];
    // Far
    planes[5] = [m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]];
    planes
}

/// Check if a chunk AABB is visible in the frustum.
pub fn chunk_visible(planes: &[[f32; 4]; 6], p: i32, q: i32, miny: i32, maxy: i32, ortho: i32) -> bool {
    let x = (p * CHUNK_SIZE - 1) as f32;
    let z = (q * CHUNK_SIZE - 1) as f32;
    let d = (CHUNK_SIZE + 1) as f32;
    let points: [[f32; 3]; 8] = [
        [x,     miny as f32, z],
        [x + d, miny as f32, z],
        [x,     miny as f32, z + d],
        [x + d, miny as f32, z + d],
        [x,     maxy as f32, z],
        [x + d, maxy as f32, z],
        [x,     maxy as f32, z + d],
        [x + d, maxy as f32, z + d],
    ];
    let n = if ortho != 0 { 4 } else { 6 };
    for i in 0..n {
        let mut inside = 0;
        let mut outside = 0;
        for j in 0..8 {
            let d = planes[i][0] * points[j][0]
                  + planes[i][1] * points[j][1]
                  + planes[i][2] * points[j][2]
                  + planes[i][3];
            if d < 0.0 { outside += 1; } else { inside += 1; }
            if inside > 0 && outside > 0 { break; }
        }
        if inside == 0 { return false; }
    }
    true
}

/// Upload vertex data to a GL buffer and return the buffer handle.
pub fn gen_buffer(ctx: &mut GlContext, data: &[f32]) -> GLuint {
    let bufs = ctx.gl_gen_buffers(1);
    let buf = bufs[0];
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, buf).unwrap();
    let bytes = unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4)
    };
    ctx.gl_buffer_data(GL_ARRAY_BUFFER, bytes, GL_STATIC_DRAW).unwrap();
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, 0).unwrap();
    buf
}

/// Allocate faces: each face has `components` floats per vertex, 6 vertices per face.
pub fn malloc_faces(components: usize, faces: usize) -> Vec<f32> {
    vec![0.0f32; components * 6 * faces]
}

/// Upload face data and return buffer handle.
pub fn gen_faces(ctx: &mut GlContext, components: usize, faces: usize, data: &[f32]) -> GLuint {
    let total = components * 6 * faces;
    gen_buffer(ctx, &data[..total])
}

/// Draw triangles with position(3) + normal(3) + uv(4) = stride 10 (block/chunk data)
pub fn draw_triangles_3d_ao(ctx: &mut GlContext, buffer: GLuint, count: i32, fs: &impl FragmentShader) {
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, buffer).unwrap();
    ctx.gl_enable_vertex_attrib_array(0);
    ctx.gl_enable_vertex_attrib_array(1);
    ctx.gl_enable_vertex_attrib_array(2);
    ctx.gl_vertex_attrib_pointer(0, 3, GL_FLOAT, false, 40, 0);
    ctx.gl_vertex_attrib_pointer(1, 3, GL_FLOAT, false, 40, 12);
    ctx.gl_vertex_attrib_pointer(2, 4, GL_FLOAT, false, 40, 24);
    ctx.gl_draw_arrays_with_fs(GL_TRIANGLES, 0, count, fs);
    ctx.gl_disable_vertex_attrib_array(0);
    ctx.gl_disable_vertex_attrib_array(1);
    ctx.gl_disable_vertex_attrib_array(2);
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, 0).unwrap();
}

/// Draw triangles with position(3) + normal(3) + uv(2) = stride 8 (sky data)
pub fn draw_triangles_3d(ctx: &mut GlContext, buffer: GLuint, count: i32, fs: &impl FragmentShader) {
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, buffer).unwrap();
    ctx.gl_enable_vertex_attrib_array(0);
    ctx.gl_enable_vertex_attrib_array(1);
    ctx.gl_enable_vertex_attrib_array(2);
    ctx.gl_vertex_attrib_pointer(0, 3, GL_FLOAT, false, 32, 0);
    ctx.gl_vertex_attrib_pointer(1, 3, GL_FLOAT, false, 32, 12);
    ctx.gl_vertex_attrib_pointer(2, 2, GL_FLOAT, false, 32, 24);
    ctx.gl_draw_arrays_with_fs(GL_TRIANGLES, 0, count, fs);
    ctx.gl_disable_vertex_attrib_array(0);
    ctx.gl_disable_vertex_attrib_array(1);
    ctx.gl_disable_vertex_attrib_array(2);
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, 0).unwrap();
}

/// Draw triangles with position(3) + uv(2) = stride 5 (3D text/sign data)
pub fn draw_triangles_3d_text(ctx: &mut GlContext, buffer: GLuint, count: i32, fs: &impl FragmentShader) {
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, buffer).unwrap();
    ctx.gl_enable_vertex_attrib_array(0);
    ctx.gl_enable_vertex_attrib_array(1);
    ctx.gl_vertex_attrib_pointer(0, 3, GL_FLOAT, false, 20, 0);
    ctx.gl_vertex_attrib_pointer(1, 2, GL_FLOAT, false, 20, 12);
    ctx.gl_draw_arrays_with_fs(GL_TRIANGLES, 0, count, fs);
    ctx.gl_disable_vertex_attrib_array(0);
    ctx.gl_disable_vertex_attrib_array(1);
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, 0).unwrap();
}

/// Draw 2D triangles with position(2) + uv(2) = stride 4 (2D text data)
pub fn draw_triangles_2d(ctx: &mut GlContext, buffer: GLuint, count: i32, fs: &impl FragmentShader) {
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, buffer).unwrap();
    ctx.gl_enable_vertex_attrib_array(0);
    ctx.gl_enable_vertex_attrib_array(1);
    ctx.gl_vertex_attrib_pointer(0, 2, GL_FLOAT, false, 16, 0);
    ctx.gl_vertex_attrib_pointer(1, 2, GL_FLOAT, false, 16, 8);
    ctx.gl_draw_arrays_with_fs(GL_TRIANGLES, 0, count, fs);
    ctx.gl_disable_vertex_attrib_array(0);
    ctx.gl_disable_vertex_attrib_array(1);
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, 0).unwrap();
}

/// Draw lines with position only.
pub fn draw_lines(ctx: &mut GlContext, buffer: GLuint, components: i32, count: i32) {
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, buffer).unwrap();
    ctx.gl_enable_vertex_attrib_array(0);
    ctx.gl_vertex_attrib_pointer(0, components, GL_FLOAT, false, components * 4, 0);
    ctx.gl_draw_arrays(GL_LINES, 0, count);
    ctx.gl_disable_vertex_attrib_array(0);
    ctx.gl_bind_buffer(GL_ARRAY_BUFFER, 0).unwrap();
}

pub fn del_buffer(ctx: &mut GlContext, buffer: GLuint) {
    if buffer != 0 {
        ctx.gl_delete_buffers(&[buffer]);
    }
}
