#![allow(dead_code, non_snake_case)]

use core::ffi::c_void;
use portablegl::gl_context::GlContext;
use portablegl::gl_types::*;
use portablegl::math::*;

// ============================================================
// Block Shader Uniforms
// ============================================================
#[repr(C)]
pub struct BlockUniforms {
    pub mvp_mat: Mat4,
    pub camera: Vec3,
    pub fog_distance: f32,
    pub ortho: i32,
    pub daylight: f32,
    pub timer: f32,
    pub block_tex: GLuint,
    pub sky_tex: GLuint,
    pub ctx: *const GlContext,
}

// Block vertex shader: 6 varyings (fragment_uv.xy, fragment_ao, fragment_light, fog_factor, fog_height, diffuse)
// = 7 floats output
pub const BLOCK_VS_OUT: usize = 7;

pub unsafe extern "C" fn block_vs(
    vs_output: *mut f32,
    vertex_attribs: *mut Vec4,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const BlockUniforms);
    let position = *vertex_attribs.add(0); // position (x,y,z,1)
    let normal = *vertex_attribs.add(1);   // normal (nx,ny,nz,0)
    let uv = *vertex_attribs.add(2);       // uv (u, v, ao, light)

    (*builtins).gl_Position = u.mvp_mat.mult_m4_v4(position);

    let out = vs_output;
    // fragment_uv
    *out.add(0) = uv.x;
    *out.add(1) = uv.y;
    // fragment_ao
    *out.add(2) = 0.3 + (1.0 - uv.z) * 0.7;
    // fragment_light
    *out.add(3) = uv.w;

    let light_direction = Vec3 { x: -1.0, y: 1.0, z: -1.0 };
    let ld = light_direction.normalize();
    let n = Vec3 { x: normal.x, y: normal.y, z: normal.z };
    let diffuse = Vec3::dot(n, ld).max(0.0);

    if u.ortho != 0 {
        *out.add(4) = 0.0; // fog_factor
        *out.add(5) = 0.0; // fog_height
    } else {
        let cam = Vec3 { x: u.camera.x, y: u.camera.y, z: u.camera.z };
        let pos3 = Vec3 { x: position.x, y: position.y, z: position.z };
        let dx = pos3.x - cam.x;
        let dy = pos3.y - cam.y;
        let dz = pos3.z - cam.z;
        let camera_distance = (dx*dx + dy*dy + dz*dz).sqrt();
        let fog_factor = (camera_distance / u.fog_distance).clamp(0.0, 1.0).powi(4);
        let dx_horiz = ((pos3.x - cam.x).powi(2) + (pos3.z - cam.z).powi(2)).sqrt();
        let fog_height = (dy.atan2(dx_horiz) + std::f32::consts::PI / 2.0) / std::f32::consts::PI;
        *out.add(4) = fog_factor;
        *out.add(5) = fog_height;
    }
    *out.add(6) = diffuse;
}

pub unsafe extern "C" fn block_fs(
    fs_input: *mut f32,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const BlockUniforms);
    let ctx = &*u.ctx;

    let frag_u = *fs_input.add(0);
    let frag_v = *fs_input.add(1);
    let fragment_ao = *fs_input.add(2);
    let fragment_light = *fs_input.add(3);
    let fog_factor = *fs_input.add(4);
    let fog_height = *fs_input.add(5);
    let diffuse = *fs_input.add(6);

    let texel = ctx.texture2d(u.block_tex, frag_u, frag_v);
    let mut r = texel.x;
    let mut g = texel.y;
    let mut b = texel.z;

    // Discard magenta pixels
    if r == 1.0 && g == 0.0 && b == 1.0 {
        (*builtins).discard = true;
        return;
    }

    let cloud = r == 1.0 && g == 1.0 && b == 1.0;
    if cloud && u.ortho != 0 {
        (*builtins).discard = true;
        return;
    }

    let df = if cloud { 1.0 - diffuse * 0.2 } else { diffuse };
    let ao = if cloud { 1.0 - (1.0 - fragment_ao) * 0.2 } else { fragment_ao };
    let ao = (ao + fragment_light).min(1.0);
    let df = (df + fragment_light).min(1.0);
    let value = (u.daylight + fragment_light).min(1.0);
    let light_c = value * 0.3 + 0.2;
    let ambient = value * 0.3 + 0.2;
    let lr = ambient + light_c * df;
    let lg = ambient + light_c * df;
    let lb = ambient + light_c * df;
    r = (r * lr * ao).clamp(0.0, 1.0);
    g = (g * lg * ao).clamp(0.0, 1.0);
    b = (b * lb * ao).clamp(0.0, 1.0);

    // Fog blending with sky color
    let sky_color = ctx.texture2d(u.sky_tex, u.timer, fog_height);
    r = r + (sky_color.x - r) * fog_factor;
    g = g + (sky_color.y - g) * fog_factor;
    b = b + (sky_color.z - b) * fog_factor;

    (*builtins).gl_FragColor = Vec4::new(r, g, b, 1.0);
}

// ============================================================
// Sky Shader Uniforms
// ============================================================
#[repr(C)]
pub struct SkyUniforms {
    pub mvp_mat: Mat4,
    pub timer: f32,
    pub sky_tex: GLuint,
    pub ctx: *const GlContext,
}

pub const SKY_VS_OUT: usize = 2;

pub unsafe extern "C" fn sky_vs(
    vs_output: *mut f32,
    vertex_attribs: *mut Vec4,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const SkyUniforms);
    let position = *vertex_attribs.add(0);
    let _normal = *vertex_attribs.add(1);
    let uv = *vertex_attribs.add(2);

    (*builtins).gl_Position = u.mvp_mat.mult_m4_v4(position);
    *vs_output.add(0) = uv.x;
    *vs_output.add(1) = uv.y;
}

pub unsafe extern "C" fn sky_fs(
    fs_input: *mut f32,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const SkyUniforms);
    let ctx = &*u.ctx;
    let _frag_u = *fs_input.add(0);
    let frag_v = *fs_input.add(1);
    let color = ctx.texture2d(u.sky_tex, u.timer, frag_v);
    (*builtins).gl_FragColor = color;
}

// ============================================================
// Text Shader Uniforms
// ============================================================
#[repr(C)]
pub struct TextUniforms {
    pub mvp_mat: Mat4,
    pub is_sign: i32,
    pub tex: GLuint,
    pub ctx: *const GlContext,
}

pub const TEXT_VS_OUT: usize = 2;

pub unsafe extern "C" fn text_vs(
    vs_output: *mut f32,
    vertex_attribs: *mut Vec4,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const TextUniforms);
    let position = *vertex_attribs.add(0);
    let uv = *vertex_attribs.add(1);

    (*builtins).gl_Position = u.mvp_mat.mult_m4_v4(position);
    *vs_output.add(0) = uv.x;
    *vs_output.add(1) = uv.y;
}

pub unsafe extern "C" fn text_fs(
    fs_input: *mut f32,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const TextUniforms);
    let ctx = &*u.ctx;
    let fu = *fs_input.add(0);
    let fv = *fs_input.add(1);
    let mut color = ctx.texture2d(u.tex, fu, fv);
    if u.is_sign != 0 {
        if color.x == 1.0 && color.y == 1.0 && color.z == 1.0 && color.w == 1.0 {
            (*builtins).discard = true;
            return;
        }
    } else {
        color.w = color.w.max(0.4);
    }
    (*builtins).gl_FragColor = color;
}

// ============================================================
// Text3D Shader (for signs in 3D space)
// Uses position(3) + uv(2) = attribute 0 (vec4 pos) + attribute 1 (vec4 uv)
// ============================================================
pub const TEXT3D_VS_OUT: usize = 2;

pub unsafe extern "C" fn text3d_vs(
    vs_output: *mut f32,
    vertex_attribs: *mut Vec4,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const TextUniforms);
    let position = *vertex_attribs.add(0);
    let uv = *vertex_attribs.add(1);

    (*builtins).gl_Position = u.mvp_mat.mult_m4_v4(position);
    *vs_output.add(0) = uv.x;
    *vs_output.add(1) = uv.y;
}

pub unsafe extern "C" fn text3d_fs(
    fs_input: *mut f32,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    text_fs(fs_input, builtins, uniforms);
}

// ============================================================
// Line Shader Uniforms
// ============================================================
#[repr(C)]
pub struct LineUniforms {
    pub mvp_mat: Mat4,
}

pub const LINE_VS_OUT: usize = 0;

pub unsafe extern "C" fn line_vs(
    _vs_output: *mut f32,
    vertex_attribs: *mut Vec4,
    builtins: *mut ShaderBuiltins,
    uniforms: *mut c_void,
) {
    let u = &*(uniforms as *const LineUniforms);
    let position = *vertex_attribs.add(0);
    (*builtins).gl_Position = u.mvp_mat.mult_m4_v4(position);
}

pub unsafe extern "C" fn line_fs(
    _fs_input: *mut f32,
    builtins: *mut ShaderBuiltins,
    _uniforms: *mut c_void,
) {
    (*builtins).gl_FragColor = Vec4::new(0.0, 0.0, 0.0, 1.0);
}
