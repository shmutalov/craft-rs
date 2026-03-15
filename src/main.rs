#![allow(non_snake_case, non_upper_case_globals, dead_code)]

mod chunk;
mod config;
mod cube;
mod item;
mod map;
mod player;
mod render;
mod shaders;
mod sign;
mod util;
mod world;

use chunk::*;
use config::*;
use item::*;
use map::Map;
use player::*;
use render::*;
use shaders::*;
use util::*;

use portablegl::gl_context::GlContext;
use portablegl::gl_types::*;
use portablegl::math::*;
use core::ffi::c_void;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;

struct GameState {
    chunks: Vec<Chunk>,
    players: Vec<Player>,
    flying: bool,
    item_index: usize,
    ortho: i32,
    fov: f32,
    scale: i32,
    typing: bool,
    typing_buffer: String,
    messages: Vec<String>,
    message_index: usize,
    width: i32,
    height: i32,
    create_radius: i32,
    render_radius: i32,
    delete_radius: i32,
    sign_radius: i32,
    day_length: i32,
    observe1: usize,
    observe2: usize,
    mode: i32,
    dy: f32,
    exclusive: bool,
    time: f64,
}

impl GameState {
    fn new() -> Self {
        let mut players = vec![Player::new()];
        players[0].id = 0;
        GameState {
            chunks: Vec::new(),
            players,
            flying: false,
            item_index: 0,
            ortho: 0,
            fov: 65.0,
            scale: 1,
            typing: false,
            typing_buffer: String::new(),
            messages: vec![String::new(); MAX_MESSAGES],
            message_index: 0,
            width: WINDOW_WIDTH as i32,
            height: WINDOW_HEIGHT as i32,
            create_radius: CREATE_CHUNK_RADIUS,
            render_radius: RENDER_CHUNK_RADIUS,
            delete_radius: DELETE_CHUNK_RADIUS,
            sign_radius: RENDER_SIGN_RADIUS,
            day_length: DAY_LENGTH,
            observe1: 0,
            observe2: 0,
            mode: MODE_OFFLINE,
            dy: 0.0,
            exclusive: true,
            time: 0.0,
        }
    }

    fn find_chunk(&self, p: i32, q: i32) -> Option<usize> {
        self.chunks.iter().position(|c| c.p == p && c.q == q)
    }

    fn time_of_day(&self) -> f32 {
        if self.day_length <= 0 { return 0.5; }
        let t = self.time / self.day_length as f64;
        let t = t - t.floor();
        t as f32
    }

    fn get_daylight(&self) -> f32 {
        let timer = self.time_of_day();
        if timer < 0.5 {
            let t = (timer - 0.25) * 100.0;
            1.0 / (1.0 + 2.0f32.powf(-t))
        } else {
            let t = (timer - 0.85) * 100.0;
            1.0 - 1.0 / (1.0 + 2.0f32.powf(-t))
        }
    }

    fn highest_block(&self, x: f32, z: f32) -> i32 {
        let mut result = -1;
        let nx = x.round() as i32;
        let nz = z.round() as i32;
        let p = chunked(x);
        let q = chunked(z);
        if let Some(idx) = self.find_chunk(p, q) {
            for (ex, ey, ez, ew) in self.chunks[idx].map.iter() {
                if is_obstacle(ew) && ex == nx && ez == nz {
                    result = result.max(ey);
                }
            }
        }
        result
    }

    fn hit_test(&self, previous: bool, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> (i32, i32, i32, i32) {
        let (vx, vy, vz) = get_sight_vector(rx, ry);
        let p = chunked(x);
        let q = chunked(z);
        let mut best_d = 0.0f32;
        let mut result = (0, 0, 0, 0);

        for chunk in &self.chunks {
            if chunk_distance(chunk.p, chunk.q, p, q) > 1 { continue; }
            let r = self.hit_test_map(&chunk.map, 8.0, previous, x, y, z, vx, vy, vz);
            if r.3 > 0 {
                let d = ((r.0 as f32 - x).powi(2) + (r.1 as f32 - y).powi(2) + (r.2 as f32 - z).powi(2)).sqrt();
                if best_d == 0.0 || d < best_d {
                    best_d = d;
                    result = r;
                }
            }
        }
        result
    }

    fn hit_test_map(&self, map: &Map, max_dist: f32, previous: bool,
                    mut x: f32, mut y: f32, mut z: f32,
                    vx: f32, vy: f32, vz: f32) -> (i32, i32, i32, i32) {
        let m = 32;
        let mut px = 0i32;
        let mut py = 0i32;
        let mut pz = 0i32;
        for _ in 0..(max_dist as i32 * m) {
            let nx = x.round() as i32;
            let ny = y.round() as i32;
            let nz = z.round() as i32;
            if nx != px || ny != py || nz != pz {
                let hw = map.get(nx, ny, nz);
                if hw > 0 {
                    if previous {
                        return (px, py, pz, hw);
                    } else {
                        return (nx, ny, nz, hw);
                    }
                }
                px = nx; py = ny; pz = nz;
            }
            x += vx / m as f32;
            y += vy / m as f32;
            z += vz / m as f32;
        }
        (0, 0, 0, 0)
    }
}

fn main() {
    // SDL2 Initialization
    let sdl_context = sdl2::init().expect("Failed to init SDL2");
    let video = sdl_context.video().expect("Failed to init SDL2 video");
    let window = video
        .window("Craft", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("Failed to create window");

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build()
        .expect("Failed to create canvas");

    let texture_creator = canvas.texture_creator();
    let mut sdl_texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::ABGR8888,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )
        .expect("Failed to create texture");

    sdl_context.mouse().set_relative_mouse_mode(true);
    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");
    let timer = sdl_context.timer().expect("Failed to get timer");

    // PortableGL Initialization
    let mut ctx = GlContext::new();
    ctx.init(WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);

    ctx.gl_enable(GL_CULL_FACE);
    ctx.gl_enable(GL_DEPTH_TEST);
    ctx.gl_logic_op(GL_INVERT);
    ctx.gl_clear_color(0.0, 0.0, 0.0, 1.0);

    // Load textures
    let block_tex = ctx.gl_gen_textures(1)[0];
    ctx.gl_bind_texture(GL_TEXTURE_2D, block_tex).unwrap();
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as GLint);
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as GLint);
    load_png_texture(&mut ctx, "textures/texture.png");

    let font_tex = ctx.gl_gen_textures(1)[0];
    ctx.gl_bind_texture(GL_TEXTURE_2D, font_tex).unwrap();
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as GLint);
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as GLint);
    load_png_texture(&mut ctx, "textures/font.png");

    let sky_tex = ctx.gl_gen_textures(1)[0];
    ctx.gl_bind_texture(GL_TEXTURE_2D, sky_tex).unwrap();
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as GLint);
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as GLint);
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as GLint);
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as GLint);
    load_png_texture(&mut ctx, "textures/sky.png");

    let sign_tex = ctx.gl_gen_textures(1)[0];
    ctx.gl_bind_texture(GL_TEXTURE_2D, sign_tex).unwrap();
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as GLint);
    ctx.gl_tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as GLint);
    load_png_texture(&mut ctx, "textures/sign.png");

    // Create shader programs
    let block_interp = [PGL_SMOOTH; BLOCK_VS_OUT];
    let block_prog = ctx.pgl_create_program(
        block_vs, block_fs, BLOCK_VS_OUT as i32, &block_interp, true,
    );

    let sky_interp = [PGL_SMOOTH; SKY_VS_OUT];
    let sky_prog = ctx.pgl_create_program(
        sky_vs, sky_fs, SKY_VS_OUT as i32, &sky_interp, false,
    );

    let text_interp = [PGL_SMOOTH; TEXT_VS_OUT];
    let text_prog = ctx.pgl_create_program(
        text_vs, text_fs, TEXT_VS_OUT as i32, &text_interp, true,
    );

    let line_prog = ctx.pgl_create_program(
        line_vs, line_fs, LINE_VS_OUT as i32, &[], false,
    );

    // Generate sky sphere buffer
    let mut sky_data = vec![0.0f32; 12288];
    cube::make_sphere(&mut sky_data, 1.0, 3);
    let sky_buffer = gen_buffer(&mut ctx, &sky_data);

    // Create VAO
    let vao = ctx.gl_gen_vertex_arrays(1)[0];
    ctx.gl_bind_vertex_array(vao).unwrap();

    // Game state
    let mut g = GameState::new();
    g.time = g.day_length as f64 / 3.0;

    // Worker pool for async chunk generation
    let worker_pool = WorkerPool::new(WORKERS);

    let mut fps_counter = FpsCounter::new();
    let mut previous_time = timer.ticks() as f64 / 1000.0;
    let start_time = previous_time;

    // Uniforms (allocated on heap so pointers remain stable)
    let mut block_uniforms = BlockUniforms {
        mvp_mat: Mat4::identity(),
        camera: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        fog_distance: 0.0,
        ortho: 0,
        daylight: 1.0,
        timer: 0.0,
        block_tex,
        sky_tex,
        ctx: &ctx as *const GlContext,
    };
    let mut sky_uniforms = SkyUniforms {
        mvp_mat: Mat4::identity(),
        timer: 0.0,
        sky_tex,
        ctx: &ctx as *const GlContext,
    };
    let mut text_uniforms = TextUniforms {
        mvp_mat: Mat4::identity(),
        is_sign: 0,
        tex: font_tex,
        ctx: &ctx as *const GlContext,
    };
    let mut line_uniforms = LineUniforms {
        mvp_mat: Mat4::identity(),
    };

    // Force initial chunks around player
    {
        let s = &g.players[0].state;
        let p = chunked(s.x);
        let q = chunked(s.z);
        for dp in -1..=1 {
            for dq in -1..=1 {
                let cp = p + dp;
                let cq = q + dq;
                let mut chunk = Chunk::new(cp, cq);
                load_chunk_terrain(cp, cq, &mut chunk.map);
                g.chunks.push(chunk);
            }
        }
        // Generate buffers for initial chunks
        for i in 0..g.chunks.len() {
            gen_chunk_buffer_sync(&mut ctx, &mut g.chunks, i);
        }
        // Set player Y to highest block
        {
            let sx = g.players[0].state.x;
            let sz = g.players[0].state.z;
            g.players[0].state.y = g.highest_block(sx, sz) as f32 + 2.0;
        }
    }

    // Main game loop
    'main_loop: loop {
        let now = timer.ticks() as f64 / 1000.0;
        let dt = (now - previous_time).min(0.2).max(0.0);
        previous_time = now;
        g.time += dt;
        fps_counter.update(now);

        // Update context pointer for shaders (ctx may have moved)
        block_uniforms.ctx = &ctx as *const GlContext;
        sky_uniforms.ctx = &ctx as *const GlContext;
        text_uniforms.ctx = &ctx as *const GlContext;

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode: Some(key), .. } => {
                    match key {
                        Keycode::Escape => {
                            if g.exclusive {
                                g.exclusive = false;
                                sdl_context.mouse().set_relative_mouse_mode(false);
                            } else {
                                break 'main_loop;
                            }
                        }
                        Keycode::Tab => g.flying = !g.flying,
                        Keycode::Num1 => g.item_index = 0,
                        Keycode::Num2 => g.item_index = 1,
                        Keycode::Num3 => g.item_index = 2,
                        Keycode::Num4 => g.item_index = 3,
                        Keycode::Num5 => g.item_index = 4,
                        Keycode::Num6 => g.item_index = 5,
                        Keycode::Num7 => g.item_index = 6,
                        Keycode::Num8 => g.item_index = 7,
                        Keycode::Num9 => g.item_index = 8,
                        Keycode::E => g.item_index = (g.item_index + 1) % ITEMS.len(),
                        Keycode::R => {
                            if g.item_index == 0 { g.item_index = ITEMS.len() - 1; }
                            else { g.item_index -= 1; }
                        }
                        _ => {}
                    }
                }
                Event::MouseButtonDown { mouse_btn, .. } => {
                    if !g.exclusive {
                        g.exclusive = true;
                        sdl_context.mouse().set_relative_mouse_mode(true);
                    } else {
                        let s = g.players[0].state;
                        match mouse_btn {
                            MouseButton::Left => {
                                let (hx, hy, hz, hw) = g.hit_test(false, s.x, s.y, s.z, s.rx, s.ry);
                                if hy > 0 && hy < 256 && is_destructable(hw) {
                                    set_block(&mut g, hx, hy, hz, 0);
                                }
                            }
                            MouseButton::Right => {
                                let (hx, hy, hz, hw) = g.hit_test(true, s.x, s.y, s.z, s.rx, s.ry);
                                if hy > 0 && hy < 256 && is_obstacle(hw) {
                                    if !player_intersects_block(2, s.x, s.y, s.z, hx, hy, hz) {
                                        let item = ITEMS[g.item_index];
                                        set_block(&mut g, hx, hy, hz, item);
                                    }
                                }
                            }
                            MouseButton::Middle => {
                                let (_, _, _, hw) = g.hit_test(false, s.x, s.y, s.z, s.rx, s.ry);
                                for (i, &item) in ITEMS.iter().enumerate() {
                                    if item == hw { g.item_index = i; break; }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::MouseMotion { xrel, yrel, .. } if g.exclusive => {
                    let m = 0.0025;
                    let s = &mut g.players[0].state;
                    s.rx += xrel as f32 * m;
                    s.ry -= yrel as f32 * m;
                    if s.rx < 0.0 { s.rx += 2.0 * std::f32::consts::PI; }
                    if s.rx >= 2.0 * std::f32::consts::PI { s.rx -= 2.0 * std::f32::consts::PI; }
                    s.ry = s.ry.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
                }
                Event::MouseWheel { y: scroll_y, .. } => {
                    if scroll_y > 0 {
                        if g.item_index == 0 { g.item_index = ITEMS.len() - 1; }
                        else { g.item_index -= 1; }
                    } else if scroll_y < 0 {
                        g.item_index = (g.item_index + 1) % ITEMS.len();
                    }
                }
                _ => {}
            }
        }

        // Handle continuous key input
        let keys = event_pump.keyboard_state();
        if g.exclusive && !g.typing {
            let mut sz: i32 = 0;
            let mut sx: i32 = 0;
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::W) { sz -= 1; }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::S) { sz += 1; }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::A) { sx -= 1; }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::D) { sx += 1; }
            g.ortho = if keys.is_scancode_pressed(sdl2::keyboard::Scancode::F) { 64 } else { 0 };
            g.fov = if keys.is_scancode_pressed(sdl2::keyboard::Scancode::LShift) { 15.0 } else { 65.0 };

            let s = g.players[0].state;
            let (mut vx, mut vy, mut vz) = get_motion_vector(g.flying, sz, sx, s.rx, s.ry);

            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
                if g.flying {
                    vy = 1.0;
                } else if g.dy == 0.0 {
                    g.dy = 8.0;
                }
            }

            let speed = if g.flying { 20.0 } else { 5.0 };
            let estimate = ((vx * speed).powi(2) + (vy * speed + g.dy.abs() * 2.0).powi(2) + (vz * speed).powi(2)).sqrt() * dt as f32 * 8.0;
            let step = 8.max(estimate.round() as i32);
            let ut = dt as f32 / step as f32;
            vx *= ut * speed;
            vy *= ut * speed;
            vz *= ut * speed;

            for _ in 0..step {
                if g.flying {
                    g.dy = 0.0;
                } else {
                    g.dy -= ut * 25.0;
                    g.dy = g.dy.max(-250.0);
                }
                g.players[0].state.x += vx;
                g.players[0].state.y += vy + g.dy * ut;
                g.players[0].state.z += vz;
                // Collision
                let cp = chunked(g.players[0].state.x);
                let cq = chunked(g.players[0].state.z);
                if let Some(idx) = g.find_chunk(cp, cq) {
                    let map_clone = g.chunks[idx].map.clone();
                    let s = &mut g.players[0].state;
                    if player::collide(2, &mut s.x, &mut s.y, &mut s.z, &map_clone) {
                        g.dy = 0.0;
                    }
                }
            }
            if g.players[0].state.y < 0.0 {
                let sx = g.players[0].state.x;
                let sz = g.players[0].state.z;
                g.players[0].state.y = g.highest_block(sx, sz) as f32 + 2.0;
            }
        }

        // Check worker results
        for i in 0..worker_pool.len() {
            if worker_pool.is_done(i) {
                if let Some(result) = worker_pool.take_result(i) {
                    if let Some(idx) = g.find_chunk(result.p, result.q) {
                        if result.load {
                            if let Some(bm) = result.block_map {
                                g.chunks[idx].map = bm;
                            }
                            if let Some(lm) = result.light_map {
                                g.chunks[idx].lights = lm;
                            }
                        }
                        // Upload buffer
                        del_buffer(&mut ctx, g.chunks[idx].buffer);
                        g.chunks[idx].buffer = gen_faces(&mut ctx, 10, result.faces as usize, &result.data);
                        g.chunks[idx].faces = result.faces;
                        g.chunks[idx].miny = result.miny;
                        g.chunks[idx].maxy = result.maxy;
                        g.chunks[idx].dirty = false;
                    }
                }
            }
        }

        // Ensure chunks around player
        {
            let s = g.players[0].state;
            let p = chunked(s.x);
            let q = chunked(s.z);

            // Delete far chunks
            let dr = g.delete_radius;
            g.chunks.retain(|c| chunk_distance(c.p, c.q, p, q) < dr);

            // Create/load nearby chunks
            let r = g.create_radius;
            for dp in -r..=r {
                for dq in -r..=r {
                    let cp = p + dp;
                    let cq = q + dq;
                    if g.find_chunk(cp, cq).is_none() && g.chunks.len() < MAX_CHUNKS {
                        let mut chunk = Chunk::new(cp, cq);
                        load_chunk_terrain(cp, cq, &mut chunk.map);
                        g.chunks.push(chunk);
                        let idx = g.chunks.len() - 1;
                        gen_chunk_buffer_sync(&mut ctx, &mut g.chunks, idx);
                    }
                }
            }

            // Rebuild dirty chunks
            for i in 0..g.chunks.len() {
                if g.chunks[i].dirty && chunk_distance(g.chunks[i].p, g.chunks[i].q, p, q) <= 1 {
                    gen_chunk_buffer_sync(&mut ctx, &mut g.chunks, i);
                }
            }
        }

        // ============ RENDER ============
        ctx.gl_viewport(0, 0, g.width, g.height);
        ctx.gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        let s = g.players[0].state;

        // Render sky
        {
            let matrix = set_matrix_3d(
                g.width, g.height, 0.0, 0.0, 0.0,
                s.rx, s.ry, g.fov, 0, g.render_radius,
            );
            sky_uniforms.mvp_mat = matrix;
            sky_uniforms.timer = g.time_of_day();
            ctx.gl_use_program(sky_prog);
            ctx.pgl_set_uniform(&mut sky_uniforms as *mut _ as *mut c_void);
            draw_triangles_3d(&mut ctx, sky_buffer, 512 * 3);
        }

        ctx.gl_clear(GL_DEPTH_BUFFER_BIT);

        // Render chunks
        {
            let matrix = set_matrix_3d(
                g.width, g.height, s.x, s.y, s.z,
                s.rx, s.ry, g.fov, g.ortho, g.render_radius,
            );
            let planes = frustum_planes(g.render_radius, &matrix);
            let p = chunked(s.x);
            let q = chunked(s.z);
            let daylight = g.get_daylight();

            block_uniforms.mvp_mat = matrix;
            block_uniforms.camera = Vec3 { x: s.x, y: s.y, z: s.z };
            block_uniforms.fog_distance = (g.render_radius * CHUNK_SIZE) as f32;
            block_uniforms.ortho = g.ortho;
            block_uniforms.daylight = daylight;
            block_uniforms.timer = g.time_of_day();
            ctx.gl_use_program(block_prog);
            ctx.pgl_set_uniform(&mut block_uniforms as *mut _ as *mut c_void);

            let mut face_count = 0;
            for chunk in &g.chunks {
                if chunk_distance(chunk.p, chunk.q, p, q) > g.render_radius { continue; }
                if !chunk_visible(&planes, chunk.p, chunk.q, chunk.miny, chunk.maxy, g.ortho) { continue; }
                if chunk.buffer == 0 || chunk.faces == 0 { continue; }
                draw_triangles_3d_ao(&mut ctx, chunk.buffer, chunk.faces * 6);
                face_count += chunk.faces;
            }

            // Render wireframe
            if SHOW_WIREFRAME {
                let (hx, hy, hz, hw) = g.hit_test(false, s.x, s.y, s.z, s.rx, s.ry);
                if is_obstacle(hw) {
                    line_uniforms.mvp_mat = matrix;
                    ctx.gl_use_program(line_prog);
                    ctx.pgl_set_uniform(&mut line_uniforms as *mut _ as *mut c_void);
                    ctx.gl_line_width(1.0);
                    ctx.gl_enable(GL_COLOR_LOGIC_OP);
                    let mut wire_data = [0.0f32; 72];
                    cube::make_cube_wireframe(&mut wire_data, hx as f32, hy as f32, hz as f32, 0.53);
                    let wire_buf = gen_buffer(&mut ctx, &wire_data);
                    draw_lines(&mut ctx, wire_buf, 3, 24);
                    del_buffer(&mut ctx, wire_buf);
                    ctx.gl_disable(GL_COLOR_LOGIC_OP);
                }
            }

            // HUD
            ctx.gl_clear(GL_DEPTH_BUFFER_BIT);

            // Crosshairs
            if SHOW_CROSSHAIRS {
                let matrix_2d = set_matrix_2d(g.width, g.height);
                line_uniforms.mvp_mat = matrix_2d;
                ctx.gl_use_program(line_prog);
                ctx.pgl_set_uniform(&mut line_uniforms as *mut _ as *mut c_void);
                ctx.gl_line_width(4.0 * g.scale as f32);
                ctx.gl_enable(GL_COLOR_LOGIC_OP);
                let cx = g.width as f32 / 2.0;
                let cy = g.height as f32 / 2.0;
                let cp = 10.0 * g.scale as f32;
                let cross_data: [f32; 8] = [cx, cy - cp, cx, cy + cp, cx - cp, cy, cx + cp, cy];
                let cross_buf = gen_buffer(&mut ctx, &cross_data);
                draw_lines(&mut ctx, cross_buf, 2, 4);
                del_buffer(&mut ctx, cross_buf);
                ctx.gl_disable(GL_COLOR_LOGIC_OP);
            }

            // Item preview
            if SHOW_ITEM {
                let item_mat = set_matrix_item(g.width, g.height, g.scale);
                block_uniforms.mvp_mat = item_mat;
                block_uniforms.camera = Vec3 { x: 0.0, y: 0.0, z: 5.0 };
                ctx.gl_use_program(block_prog);
                ctx.pgl_set_uniform(&mut block_uniforms as *mut _ as *mut c_void);
                let w = ITEMS[g.item_index];
                if is_plant(w) {
                    let mut data = vec![0.0f32; 10 * 6 * 4];
                    cube::make_plant(&mut data, 0.0, 1.0, 0.0, 0.0, 0.0, 0.5, w, 45.0);
                    let buf = gen_faces(&mut ctx, 10, 4, &data);
                    draw_triangles_3d_ao(&mut ctx, buf, 24);
                    del_buffer(&mut ctx, buf);
                } else {
                    let mut data = vec![0.0f32; 10 * 6 * 6];
                    let ao = [[0.0f32; 4]; 6];
                    let light = [[0.5f32; 4]; 6];
                    cube::make_cube(&mut data, &ao, &light, [true; 6], 0.0, 0.0, 0.0, 0.5, w);
                    let buf = gen_faces(&mut ctx, 10, 6, &data);
                    draw_triangles_3d_ao(&mut ctx, buf, 36);
                    del_buffer(&mut ctx, buf);
                }
            }

            // Info text
            if SHOW_INFO_TEXT {
                let matrix_2d = set_matrix_2d(g.width, g.height);
                text_uniforms.mvp_mat = matrix_2d;
                text_uniforms.is_sign = 0;
                text_uniforms.tex = font_tex;
                ctx.gl_use_program(text_prog);
                ctx.pgl_set_uniform(&mut text_uniforms as *mut _ as *mut c_void);

                let ts = 12.0 * g.scale as f32;
                let tx = ts / 2.0;
                let ty = g.height as f32 - ts;

                let hour_f = g.time_of_day() * 24.0;
                let hour = hour_f as i32;
                let am_pm = if hour < 12 { 'a' } else { 'p' };
                let hour12 = { let h = hour % 12; if h == 0 { 12 } else { h } };
                let text = format!(
                    "({}, {}) ({:.1}, {:.1}, {:.1}) [{}, {}] {}{}m {}fps",
                    chunked(s.x), chunked(s.z), s.x, s.y, s.z,
                    g.chunks.len(), face_count * 2,
                    hour12, am_pm, fps_counter.fps,
                );
                render_text_2d(&mut ctx, tx, ty, ts, &text, &mut text_uniforms, text_prog);
            }
        }

        // Blit framebuffer to SDL2
        {
            let fb = ctx.pgl_get_back_buffer();
            sdl_texture.update(None, &fb.buf, fb.w as usize * 4).unwrap();
            canvas.copy(&sdl_texture, None, None).unwrap();
            canvas.present();
        }
    }
}

fn render_text_2d(
    ctx: &mut GlContext,
    x: f32, y: f32, n: f32,
    text: &str,
    uniforms: &mut TextUniforms,
    prog: GLuint,
) {
    let length = text.len();
    if length == 0 { return; }
    let mut data = vec![0.0f32; 4 * 6 * length];
    let mut cx = x;
    for (i, c) in text.chars().enumerate() {
        cube::make_character(&mut data[i * 24..], cx, y, n / 2.0, n, c);
        cx += n;
    }
    let buf = gen_faces(ctx, 4, length, &data);
    ctx.gl_enable(GL_BLEND);
    ctx.gl_blend_func(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    ctx.gl_use_program(prog);
    ctx.pgl_set_uniform(uniforms as *mut _ as *mut c_void);
    draw_triangles_2d(ctx, buf, length as i32 * 6);
    ctx.gl_disable(GL_BLEND);
    del_buffer(ctx, buf);
}

fn set_block(g: &mut GameState, x: i32, y: i32, z: i32, w: i32) {
    let p = chunked(x as f32);
    let q = chunked(z as f32);
    if let Some(idx) = g.find_chunk(p, q) {
        g.chunks[idx].map.set(x, y, z, w);
        g.chunks[idx].dirty = true;
    }
    // Also update neighboring chunks if on border
    for dx in -1..=1 {
        for dz in -1..=1 {
            if dx == 0 && dz == 0 { continue; }
            let np = p + dx;
            let nq = q + dz;
            if let Some(idx) = g.find_chunk(np, nq) {
                g.chunks[idx].map.set(x, y, z, -w);
                g.chunks[idx].dirty = true;
            }
        }
    }
}

fn gen_chunk_buffer_sync(ctx: &mut GlContext, chunks: &mut Vec<Chunk>, chunk_idx: usize) {
    let p = chunks[chunk_idx].p;
    let q = chunks[chunk_idx].q;

    // Build block_maps array from neighboring chunks
    let mut block_maps: [Option<Map>; 9] = Default::default();
    let mut light_maps: [Option<Map>; 9] = Default::default();

    for dp in -1..=1i32 {
        for dq in -1..=1i32 {
            let idx = ((dp + 1) * 3 + (dq + 1)) as usize;
            let cp = p + dp;
            let cq = q + dq;
            if let Some(ci) = chunks.iter().position(|c| c.p == cp && c.q == cq) {
                block_maps[idx] = Some(chunks[ci].map.clone());
                light_maps[idx] = Some(chunks[ci].lights.clone());
            }
        }
    }

    if block_maps[4].is_none() { return; }

    let (miny, maxy, faces, data) = compute_chunk(p, q, &block_maps, &light_maps);

    del_buffer(ctx, chunks[chunk_idx].buffer);
    if faces > 0 {
        chunks[chunk_idx].buffer = gen_faces(ctx, 10, faces as usize, &data);
    } else {
        chunks[chunk_idx].buffer = 0;
    }
    chunks[chunk_idx].faces = faces;
    chunks[chunk_idx].miny = miny;
    chunks[chunk_idx].maxy = maxy;
    chunks[chunk_idx].dirty = false;
}
